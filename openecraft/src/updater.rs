//! Updates of the installed editor from the releases on GitHub
//! ([`openecraft_client::update`]). [`Updater::start`] looks for a newer release on a
//! thread of its own, downloads the file of the installation, checks it against the
//! `SHA256SUMS` of the release and installs it: an AppImage and the macOS app bundles
//! are replaced at once (the running program keeps its files), the Windows setup runs
//! silently when the editor ends ([`Updater::finish`]). Other installations (packages
//! of a distribution, the Linux archive, a build of the sources) are only told of the
//! new version. Nothing waits for the network: without a connection the check ends
//! quietly. No UI code; `App` shows the [`State`].

use openecraft_client::ehttp;
use openecraft_client::update::{self, Package, Version};
use openecraft_i18n::{tr, tr_fmt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Set (to anything) it keeps the editor from looking for updates when it starts.
pub const DISABLE_VAR: &str = "OPENECRAFT_NO_UPDATE_CHECK";

/// The setup runs without questions but with its progress; Inno Setup closes what still
/// uses the files and keeps the folder, components and tasks of the installation.
const SETUP_ARGS: [&str; 5] = ["/SILENT", "/SUPPRESSMSGBOXES", "/NORESTART", "/SP-", "/CLOSEAPPLICATIONS"];

/// How the running editor was installed.
#[derive(Clone, Debug, PartialEq)]
pub enum Install {
    /// By the Windows setup: the folder of the program.
    WindowsSetup(PathBuf),
    /// The AppImage file (`$APPIMAGE`).
    AppImage(PathBuf),
    /// The app bundle (`…/OpenECraft.app`).
    MacBundle(PathBuf),
    /// Packages of a distribution, the archives, a build of the sources.
    Other,
}

impl Install {
    pub fn detect() -> Install {
        match std::env::current_exe() {
            Ok(exe) => Install::of(std::env::consts::OS, &exe, std::env::var_os("APPIMAGE").map(PathBuf::from)),
            Err(_) => Install::Other,
        }
    }

    /// The installation of the program `exe` on `os` (`$APPIMAGE` on Linux).
    fn of(os: &str, exe: &Path, appimage: Option<PathBuf>) -> Install {
        match os {
            "windows" => setup_dir(exe).map_or(Install::Other, Install::WindowsSetup),
            "linux" => appimage.filter(|p| p.is_file()).map_or(Install::Other, Install::AppImage),
            "macos" => bundle_of(exe).map_or(Install::Other, Install::MacBundle),
            _ => Install::Other,
        }
    }

    /// The file of a release that updates this installation; None for one that is
    /// updated otherwise (package manager, by hand).
    pub fn package(&self) -> Option<Package> {
        match self {
            Install::WindowsSetup(_) => Some(Package::WindowsSetup),
            Install::AppImage(_) => Some(Package::AppImage),
            Install::MacBundle(_) => Some(Package::MacDiskImage),
            Install::Other => None,
        }
    }

    /// Installs the checked download `bytes` (named `file`); the Windows setup is only
    /// saved in `staging` and its path returned, it runs when the editor ends.
    fn apply(&self, bytes: &[u8], file: &str, staging: &Path) -> Result<Option<PathBuf>, String> {
        match self {
            Install::WindowsSetup(_) => save(staging, file, bytes).map(Some),
            Install::AppImage(path) => replace_file(path, bytes).map(|()| None),
            Install::MacBundle(bundle) => {
                let dmg = save(staging, file, bytes)?;
                let done = install_disk_image(&dmg, bundle, staging);
                let _ = std::fs::remove_file(&dmg);
                done.map(|()| None)
            }
            Install::Other => Ok(None),
        }
    }
}

/// The folder of a program installed by the setup: its uninstaller is next to it.
fn setup_dir(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    let uninstaller = std::fs::read_dir(dir).ok()?.flatten().any(|e| {
        let name = e.file_name().to_string_lossy().to_lowercase();
        name.starts_with("unins") && name.ends_with(".exe")
    });
    uninstaller.then(|| dir.to_path_buf())
}

/// The app bundle of a program in `….app/Contents/MacOS/`.
fn bundle_of(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    (macos.file_name()? == "MacOS" && contents.file_name()? == "Contents" && bundle.extension()? == "app").then(|| bundle.to_path_buf())
}

/// Where the update of an installation is shown in the editor.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    #[default]
    Idle,
    Checking,
    UpToDate,
    /// A newer release this installation does not install itself; `page` is its page.
    Available { version: String, page: String },
    Downloading { version: String },
    /// Installed (AppImage, app bundle) or installed when the editor ends (Windows).
    Ready { version: String, page: String },
    /// The check or the installation failed; `page` of the release if it was found.
    Failed { error: String, page: String },
}

/// What the thread of the update hands over.
#[derive(Default)]
struct Shared {
    state: State,
    /// The downloaded Windows setup.
    setup: Option<PathBuf>,
}

pub struct Updater {
    install: Install,
    shared: Arc<Mutex<Shared>>,
    /// The state `poll` returned last.
    seen: State,
    /// The user asked for the check (every outcome is shown, not only news).
    pub manual: bool,
    /// The editor starts again after the update (when it ends).
    pub restart: bool,
}

impl Default for Updater {
    fn default() -> Self {
        Updater { install: Install::detect(), shared: Arc::default(), seen: State::Idle, manual: false, restart: false }
    }
}

/// Updates need a computer of their own (not the browser, whose page is updated with
/// the server).
pub fn supported() -> bool {
    openecraft_ui::io::has_file_system()
}

/// The check at start is not switched off for this computer.
pub fn allowed_at_start() -> bool {
    std::env::var_os(DISABLE_VAR).is_none()
}

impl Updater {
    pub fn state(&self) -> State {
        self.shared.lock().map(|s| s.state.clone()).unwrap_or_default()
    }

    pub fn install(&self) -> &Install {
        &self.install
    }

    pub fn busy(&self) -> bool {
        matches!(self.state(), State::Checking | State::Downloading { .. })
    }

    /// Looks for an update in the background; false if one is being looked for or is
    /// ready already (or updates are not supported here).
    pub fn start(&mut self, ctx: &egui::Context, manual: bool) -> bool {
        if !supported() || self.busy() || matches!(self.state(), State::Ready { .. }) {
            return false;
        }
        let Some(current) = Version::parse(env!("CARGO_PKG_VERSION")) else { return false };
        self.manual = manual;
        if let Ok(mut s) = self.shared.lock() {
            s.state = State::Checking;
        }
        let (install, shared, ctx) = (self.install.clone(), self.shared.clone(), ctx.clone());
        std::thread::spawn(move || {
            let report = |state: State| {
                if let Ok(mut s) = shared.lock() {
                    s.state = state;
                }
                ctx.request_repaint();
            };
            let (state, setup) = run(&install, &current, &fetch, &staging_dir(), &report);
            if let State::Failed { error, .. } = &state {
                log::info!("update: {error}");
            }
            if let Ok(mut s) = shared.lock() {
                s.setup = setup;
            }
            report(state);
        });
        true
    }

    /// The state when it changed since the last call.
    pub fn poll(&mut self) -> Option<State> {
        let now = self.state();
        (now != self.seen).then(|| {
            self.seen = now.clone();
            now
        })
    }

    /// The end of the editor: runs the downloaded Windows setup and starts the updated
    /// editor again if asked to.
    pub fn finish(&mut self) {
        let Ok(shared) = self.shared.lock() else { return };
        if !matches!(shared.state, State::Ready { .. }) {
            return;
        }
        let started = match (&self.install, &shared.setup) {
            (Install::WindowsSetup(_), Some(setup)) => {
                let mut cmd = Command::new(setup);
                cmd.args(SETUP_ARGS);
                if self.restart {
                    // `[Run]` of `installer/windows/openecraft.iss`.
                    cmd.arg("/RESTART=1");
                }
                cmd.spawn()
            }
            (Install::AppImage(path), _) if self.restart => Command::new(path).spawn(),
            (Install::MacBundle(bundle), _) if self.restart => Command::new("open").arg("-n").arg(bundle).spawn(),
            _ => return,
        };
        if let Err(e) = started {
            log::warn!("update: {e}");
        }
    }
}

/// Where downloads wait to be installed.
fn staging_dir() -> PathBuf {
    std::env::temp_dir().join("openecraft-update")
}

/// Sends a request and waits for the answer (on the thread of the update).
fn fetch(request: &ehttp::Request) -> ehttp::Result<ehttp::Response> {
    let (tx, rx) = std::sync::mpsc::channel();
    ehttp::fetch(request.clone(), move |r| {
        let _ = tx.send(r);
    });
    rx.recv().unwrap_or_else(|_| Err(tr("no answer").to_string()))
}

type Fetch<'a> = &'a dyn Fn(&ehttp::Request) -> ehttp::Result<ehttp::Response>;

/// Looks for a release newer than `current` and installs it. `report` gets the
/// states on the way; returns the final state and the Windows setup to run.
fn run(install: &Install, current: &Version, fetch: Fetch, staging: &Path, report: &dyn Fn(State)) -> (State, Option<PathBuf>) {
    let release = match update::latest(update::REPOSITORY).run(fetch) {
        Ok(Some(release)) => release,
        Ok(None) => return (State::UpToDate, None),
        Err(error) => return (State::Failed { error, page: String::new() }, None),
    };
    let page = release.html_url.clone();
    let Some(version) = release.version().filter(|_| release.is_newer_than(current)) else {
        // Setups of earlier updates are not needed any more.
        let _ = std::fs::remove_dir_all(staging);
        return (State::UpToDate, None);
    };
    let available = State::Available { version: version.to_string(), page: page.clone() };
    let Some(package) = install.package() else { return (available, None) };
    let file = package.file_name(&version, std::env::consts::ARCH);
    let (Some(asset), Some(sums)) = (release.asset(&file), release.asset(update::SUMS)) else {
        // The release lacks the file of this installation: only tell.
        return (available, None);
    };
    report(State::Downloading { version: version.to_string() });
    let installed = (|| {
        let sums = update::download(sums).run(fetch)?;
        let sums = String::from_utf8_lossy(&sums);
        // A download of an earlier start that waited for the end of the editor.
        let kept = std::fs::read(staging.join(&file)).ok().filter(|b| update::verify(b, &sums, &file).is_ok());
        let bytes = match kept {
            Some(bytes) => bytes,
            None => update::download(asset).run(fetch)?,
        };
        update::verify(&bytes, &sums, &file)?;
        install.apply(&bytes, &file, staging)
    })();
    match installed {
        Ok(setup) => (State::Ready { version: version.to_string(), page }, setup),
        Err(error) => (State::Failed { error, page }, None),
    }
}

fn io_error(path: &Path, e: std::io::Error) -> String {
    format!("{}: {e}", path.display())
}

/// Saves `bytes` as `staging/file`.
fn save(staging: &Path, file: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(staging).map_err(|e| io_error(staging, e))?;
    let path = staging.join(file);
    std::fs::write(&path, bytes).map_err(|e| io_error(&path, e))?;
    Ok(path)
}

/// Writes `bytes` next to `path` and moves them over it: a running program keeps
/// its old file.
fn replace_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut new = path.as_os_str().to_owned();
    new.push(".download");
    let new = PathBuf::from(new);
    std::fs::write(&new, bytes).map_err(|e| io_error(&new, e))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755)).map_err(|e| io_error(&new, e))?;
    }
    std::fs::rename(&new, path).map_err(|e| {
        let _ = std::fs::remove_file(&new);
        io_error(path, e)
    })
}

/// Mounts the disk image and replaces the editor's bundle and the other bundles of the
/// image installed next to it (the parts manager).
fn install_disk_image(dmg: &Path, bundle: &Path, staging: &Path) -> Result<(), String> {
    let volume = staging.join("volume");
    std::fs::create_dir_all(&volume).map_err(|e| io_error(&volume, e))?;
    tool(Command::new("hdiutil").args(["attach", "-nobrowse", "-readonly", "-noautoopen", "-mountpoint"]).arg(&volume).arg(dmg))?;
    let done = replace_bundles(&volume, bundle);
    let _ = tool(Command::new("hdiutil").arg("detach").arg(&volume).arg("-quiet"));
    done
}

fn replace_bundles(volume: &Path, bundle: &Path) -> Result<(), String> {
    let dir = bundle.parent().ok_or_else(|| bundle.display().to_string())?;
    let mut editor = false;
    for entry in std::fs::read_dir(volume).map_err(|e| io_error(volume, e))?.flatten() {
        let source = entry.path();
        let is_bundle = source.extension().is_some_and(|e| e == "app") && entry.file_type().is_ok_and(|t| t.is_dir());
        let target = dir.join(entry.file_name());
        if is_bundle && (target == bundle || target.exists()) {
            let new = sibling(&target, "new");
            let _ = std::fs::remove_dir_all(&new);
            // `ditto` keeps the signature, links and attributes of the bundle.
            tool(Command::new("ditto").arg(&source).arg(&new))?;
            swap_in(&new, &target)?;
            editor |= target == bundle;
        }
    }
    if editor {
        Ok(())
    } else {
        Err(tr_fmt!("{} is not in the disk image", bundle.file_name().unwrap_or_default().to_string_lossy()))
    }
}

/// `.<name>.<suffix>` next to `path`.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    path.with_file_name(format!(".{name}.{suffix}"))
}

/// Puts the folder `new` in the place of `target`; the old one is removed (or put back
/// if the move fails).
fn swap_in(new: &Path, target: &Path) -> Result<(), String> {
    let old = sibling(target, "old");
    let _ = std::fs::remove_dir_all(&old);
    std::fs::rename(target, &old).map_err(|e| io_error(target, e))?;
    if let Err(e) = std::fs::rename(new, target) {
        let _ = std::fs::rename(&old, target);
        return Err(io_error(target, e));
    }
    let _ = std::fs::remove_dir_all(&old);
    Ok(())
}

/// Runs a program of the system; its error output if it fails.
fn tool(cmd: &mut Command) -> Result<(), String> {
    let program = cmd.get_program().to_string_lossy().to_string();
    let out = cmd.output().map_err(|e| format!("{program}: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

#[cfg(test)]
mod tests;
