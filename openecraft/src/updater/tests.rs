use super::*;
use std::cell::RefCell;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("openecraft-updater-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn answer(status: u16, body: &[u8]) -> ehttp::Result<ehttp::Response> {
    Ok(ehttp::Response { url: String::new(), ok: (200..300).contains(&status), status, status_text: String::new(), headers: Default::default(), bytes: body.to_vec() })
}

const SETUP: &str = "OpenECraft-9.0.0-setup.exe";

/// A server with the release 9.0.0: its JSON, the sums and the setup; `sum` is the
/// checksum listed for the setup.
fn server(setup: &'static [u8], sum: String) -> impl Fn(&ehttp::Request) -> ehttp::Result<ehttp::Response> {
    let release = format!(
        r#"{{"tag_name":"v9.0.0","html_url":"https://example.org/v9","assets":[
            {{"name":"SHA256SUMS","browser_download_url":"https://example.org/sums"}},
            {{"name":"{SETUP}","browser_download_url":"https://example.org/setup"}}]}}"#
    );
    move |req: &ehttp::Request| match req.url.as_str() {
        u if u.ends_with("/releases/latest") => answer(200, release.as_bytes()),
        "https://example.org/sums" => answer(200, format!("{sum}  {SETUP}\n").as_bytes()),
        "https://example.org/setup" => answer(200, setup),
        _ => answer(404, b""),
    }
}

fn current() -> Version {
    Version::parse("0.1.0").unwrap()
}

#[test]
fn installations_are_told_apart() {
    let dir = temp("detect");
    let exe = dir.join("openecraft.exe");
    std::fs::write(&exe, b"").unwrap();
    assert_eq!(Install::of("windows", &exe, None), Install::Other, "a build without the setup");
    std::fs::write(dir.join("unins000.exe"), b"").unwrap();
    assert_eq!(Install::of("windows", &exe, None), Install::WindowsSetup(dir.clone()));
    let image = dir.join("OpenECraft.AppImage");
    assert_eq!(Install::of("linux", &exe, Some(image.clone())), Install::Other, "no such file");
    std::fs::write(&image, b"").unwrap();
    assert_eq!(Install::of("linux", &exe, Some(image.clone())), Install::AppImage(image));
    assert_eq!(Install::of("linux", &exe, None), Install::Other, "a package of the distribution");
    let bundle = Path::new("/Applications/OpenECraft.app");
    assert_eq!(Install::of("macos", &bundle.join("Contents/MacOS/openecraft"), None), Install::MacBundle(bundle.to_path_buf()));
    assert_eq!(Install::of("macos", Path::new("/usr/local/bin/openecraft"), None), Install::Other);
    assert_eq!(Install::Other.package(), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_windows_setup_waits_for_the_end_of_the_editor() {
    let staging = temp("setup");
    let fetch = server(b"new setup", update::sha256(b"new setup"));
    let states = RefCell::new(Vec::new());
    let install = Install::WindowsSetup(staging.join("app"));
    let (state, setup) = run(&install, &current(), &fetch, &staging, &|s| states.borrow_mut().push(s));
    assert_eq!(state, State::Ready { version: "9.0.0".into(), page: "https://example.org/v9".into() });
    assert_eq!(states.into_inner(), vec![State::Downloading { version: "9.0.0".into() }]);
    let setup = setup.unwrap();
    assert_eq!(setup, staging.join(SETUP));
    assert_eq!(std::fs::read(&setup).unwrap(), b"new setup");
    // The next start takes the setup downloaded before.
    let offline_setup = |req: &ehttp::Request| if req.url.ends_with("/setup") { Err("io: Connection refused".to_string()) } else { fetch(req) };
    let (state, again) = run(&install, &current(), &offline_setup, &staging, &|_| {});
    assert!(matches!(state, State::Ready { .. }), "{state:?}");
    assert_eq!(again, Some(setup));
    // Once the version is installed the setup is removed.
    let (state, _) = run(&install, &Version::parse("9.0.0").unwrap(), &fetch, &staging, &|_| {});
    assert_eq!(state, State::UpToDate);
    assert!(!staging.exists());
}

#[test]
fn an_appimage_is_replaced_at_once() {
    let dir = temp("appimage");
    let image = dir.join("OpenECraft.AppImage");
    std::fs::write(&image, b"old").unwrap();
    let install = Install::AppImage(image.clone());
    let file = Package::AppImage.file_name(&Version::parse("9.0.0").unwrap(), std::env::consts::ARCH);
    let release = format!(
        r#"{{"tag_name":"v9.0.0","html_url":"p","assets":[{{"name":"SHA256SUMS","browser_download_url":"s"}},{{"name":"{file}","browser_download_url":"f"}}]}}"#
    );
    let sums = format!("{}  {file}\n", update::sha256(b"new"));
    let fetch = |req: &ehttp::Request| match req.url.as_str() {
        "s" => answer(200, sums.as_bytes()),
        "f" => answer(200, b"new"),
        _ => answer(200, release.as_bytes()),
    };
    let (state, setup) = run(&install, &current(), &fetch, &dir.join("staging"), &|_| {});
    assert!(matches!(state, State::Ready { .. }), "{state:?}");
    assert_eq!(setup, None);
    assert_eq!(std::fs::read(&image).unwrap(), b"new");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "no download left beside it");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_damaged_download_is_not_installed() {
    let staging = temp("damaged");
    let fetch = server(b"tampered", update::sha256(b"new setup"));
    let (state, setup) = run(&Install::WindowsSetup(staging.clone()), &current(), &fetch, &staging, &|_| {});
    assert!(matches!(&state, State::Failed { page, .. } if page == "https://example.org/v9"), "{state:?}");
    assert_eq!(setup, None);
    assert!(!staging.join(SETUP).exists());
    let _ = std::fs::remove_dir_all(&staging);
}

#[test]
fn other_installations_are_only_told() {
    let staging = temp("other");
    let fetch = server(b"new setup", update::sha256(b"new setup"));
    let downloads = RefCell::new(0);
    let counting = |req: &ehttp::Request| {
        *downloads.borrow_mut() += 1;
        fetch(req)
    };
    let (state, _) = run(&Install::Other, &current(), &counting, &staging, &|_| {});
    assert_eq!(state, State::Available { version: "9.0.0".into(), page: "https://example.org/v9".into() });
    assert_eq!(downloads.into_inner(), 1, "only the release was asked for");
    // A release without the file of the installation is only told as well.
    let (state, _) = run(&Install::AppImage(staging.join("x.AppImage")), &current(), &fetch, &staging, &|_| {});
    assert!(matches!(state, State::Available { .. }), "{state:?}");
    let _ = std::fs::remove_dir_all(&staging);
}

#[test]
fn without_a_connection_the_check_fails_quietly() {
    let staging = temp("offline");
    let offline = |_: &ehttp::Request| Err("io: Connection refused (os error 111)".to_string());
    let states = RefCell::new(Vec::new());
    let (state, setup) = run(&Install::WindowsSetup(staging.clone()), &current(), &offline, &staging, &|s| states.borrow_mut().push(s));
    assert!(matches!(&state, State::Failed { page, .. } if page.is_empty()), "{state:?}");
    assert_eq!(setup, None);
    assert!(states.into_inner().is_empty(), "nothing was downloaded");
    // No release published yet.
    let none = |_: &ehttp::Request| answer(404, b"");
    assert_eq!(run(&Install::Other, &current(), &none, &staging, &|_| {}).0, State::UpToDate);
    let _ = std::fs::remove_dir_all(&staging);
}

#[test]
fn bundles_are_swapped_whole() {
    let dir = temp("bundle");
    let target = dir.join("OpenECraft.app");
    let new = sibling(&target, "new");
    for (d, text) in [(&target, "old"), (&new, "new")] {
        std::fs::create_dir_all(d.join("Contents")).unwrap();
        std::fs::write(d.join("Contents/version"), text).unwrap();
    }
    swap_in(&new, &target).unwrap();
    assert_eq!(std::fs::read_to_string(target.join("Contents/version")).unwrap(), "new");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "the old bundle is removed");
    // Without the new bundle the old one stays.
    assert!(swap_in(&new, &target).is_err());
    assert!(target.join("Contents/version").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_state_is_polled_once_per_change() {
    let mut updater = Updater { install: Install::Other, shared: Arc::default(), seen: State::Idle, manual: false, restart: false };
    assert_eq!(updater.poll(), None);
    updater.shared.lock().unwrap().state = State::UpToDate;
    assert_eq!(updater.poll(), Some(State::UpToDate));
    assert_eq!(updater.poll(), None);
    assert!(!updater.busy());
    updater.shared.lock().unwrap().state = State::Checking;
    assert!(updater.busy());
}
