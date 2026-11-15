//! Bug reports from within the apps: a form the user fills in and reviews, with the
//! technical details the app collected (version, system, what was open) and a
//! screenshot of the window. Nothing is sent from the app: Open GitHub issue form
//! opens a new issue on GitHub in the browser with the report filled in (and copies
//! it), where the user reviews and submits it.

use egui::{ColorImage, RichText, TextEdit, TextureHandle};
use openecraft_i18n::{tr, tr_fmt};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

/// The form for a new issue of the project.
pub const ISSUES_URL: &str = "https://github.com/nilres/OpenECraft/issues/new";
/// The most an answer keeps, in characters.
pub const MAX_ANSWER: usize = 2000;
pub const MAX_TITLE: usize = 160;
/// The most the technical details keep, in bytes.
pub const MAX_DETAILS: usize = 20_000;
/// GitHub answers a longer link with `414 URI Too Long` (from about 8,200 characters).
const MAX_URL: usize = 6000;
/// Someone not signed in is sent to sign in with the link to return to, encoded once
/// more; GitHub drops a longer one and the form comes back empty after signing in.
const MAX_SIGN_IN_URL: usize = 7000;
/// How long the form waits for the screenshot before it opens without one (seconds).
const CAPTURE_WAIT: f64 = 2.0;

/// What the app does after the form was drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    /// Save the screenshot as PNG ([`Draft::screenshot_png`], then [`Draft::saved`]).
    SaveScreenshot,
    Close,
}

/// What the form says about the last thing done.
#[derive(Clone, Debug, PartialEq)]
enum Note {
    Copied,
    ScreenshotCopied,
    ScreenshotSaved,
    ScreenshotRemoved,
    SaveFailed(String),
    NoScreenshot,
}

/// A report being written.
pub struct Draft {
    pub title: String,
    pub what: String,
    pub steps: String,
    pub expected: String,
    /// What the app collected, reviewable and editable.
    pub details: String,
    screenshot: Option<Arc<ColorImage>>,
    texture: Option<TextureHandle>,
    note: Option<Note>,
    /// The issue form was opened since the report was last changed.
    opened: bool,
    /// Open was clicked without a description.
    missing: bool,
    /// The description takes the focus in the next frame.
    focus: bool,
}

impl Draft {
    pub fn new(details: String) -> Draft {
        let mut details = details;
        truncate(&mut details, MAX_DETAILS);
        Draft {
            title: String::new(),
            what: String::new(),
            steps: String::new(),
            expected: String::new(),
            details,
            screenshot: None,
            texture: None,
            note: None,
            opened: false,
            missing: false,
            focus: true,
        }
    }

    /// The title of the issue: the summary, else the first line of the description.
    pub fn issue_title(&self) -> String {
        let title = if self.title.trim().is_empty() { self.what.trim().lines().next().unwrap_or_default() } else { self.title.trim() };
        limited_chars(title, MAX_TITLE)
    }

    /// The text of the issue (Markdown, in English for the developers).
    pub fn issue_body(&self) -> String {
        let answer = |text: &str| if text.trim().is_empty() { "Not provided".to_string() } else { limited_chars(text.trim(), MAX_ANSWER) };
        let mut body = format!(
            "## What happened\n{}\n\n## Steps to reproduce\n{}\n\n## Expected result\n{}\n\n## Technical details\n{}\n",
            answer(&self.what),
            answer(&self.steps),
            answer(&self.expected),
            limited(self.details.trim(), MAX_DETAILS)
        );
        if self.screenshot.is_some() {
            body.push_str("\n## Screenshot\n<!-- Paste or drop the screenshot here. -->\n");
        }
        body
    }

    pub fn screenshot(&self) -> Option<&ColorImage> {
        self.screenshot.as_deref()
    }

    pub fn set_screenshot(&mut self, image: Arc<ColorImage>) {
        self.screenshot = Some(image);
        self.texture = None;
        self.opened = false;
    }

    /// The screenshot as PNG file.
    pub fn screenshot_png(&self) -> Option<Result<Vec<u8>, String>> {
        self.screenshot.as_deref().map(png)
    }

    /// The screenshot was saved (`Ok(true)`), the save was cancelled (`Ok(false)`) or failed.
    pub fn saved(&mut self, result: Result<bool, String>) {
        match result {
            Ok(true) => self.note = Some(Note::ScreenshotSaved),
            Ok(false) => {}
            Err(e) => self.note = Some(Note::SaveFailed(e)),
        }
    }

    /// Opens the issue form in the browser and copies the report; false (and the
    /// description asks for an answer) when it is empty.
    pub fn open_issue(&mut self, ctx: &egui::Context) -> bool {
        self.missing = self.what.trim().is_empty();
        if self.missing {
            self.focus = true;
            return false;
        }
        let body = self.issue_body();
        let (url, _) = issue_url(&self.issue_title(), &body);
        // Copied also when the link carries it: the browser may not open, or be offline.
        ctx.copy_text(body);
        ctx.open_url(egui::OpenUrl::new_tab(url));
        self.opened = true;
        self.note = None;
        true
    }

    /// The report text is too long for the link to the issue form: it is pasted there.
    pub fn too_long(&self) -> bool {
        issue_url(&self.issue_title(), &self.issue_body()).1
    }
}

/// The link to a new issue with `title` and `body` filled in, and whether `body` was
/// too long for it: the link then carries the title alone and the body is pasted.
pub fn issue_url(title: &str, body: &str) -> (String, bool) {
    let base = format!("{ISSUES_URL}?title={}", encode(&limited_chars(title, MAX_TITLE)));
    if body.len() <= MAX_URL {
        let url = format!("{base}&body={}", encode(body));
        if url.len() <= MAX_URL && sign_in_length(&url) <= MAX_SIGN_IN_URL {
            return (url, false);
        }
    }
    (base, true)
}

/// A query value as GitHub reads it: a space is `+`, so `+` is encoded, as are `&`, `#`,
/// `%`, `=`, quotes, line breaks and every byte outside ASCII; the rest of what RFC 3986
/// allows in a query stays, which keeps the link short.
fn encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b == b' ' {
            out.push('+');
        } else if b.is_ascii_alphanumeric() || b"-_.~!$()*,;:@/?".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// The length of `url` as GitHub's sign-in page carries it back: encoded once more.
fn sign_in_length(url: &str) -> usize {
    url.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.*~".contains(&b) { 1 } else { 3 }).sum()
}

fn truncate(value: &mut String, limit: usize) {
    if value.len() > limit {
        let mut end = limit;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value.truncate(end);
    }
}

fn limited(value: &str, limit: usize) -> String {
    let mut value = value.to_string();
    truncate(&mut value, limit);
    value
}

fn limited_chars(value: &str, limit: usize) -> String {
    value.chars().take(limit).collect()
}

/// `text` with the user's home folder shown as `~` (paths in messages name the user).
pub fn hide_home(text: &str) -> String {
    match home() {
        Some(home) => hide_folder(text, &home),
        None => text.to_string(),
    }
}

fn home() -> Option<String> {
    let home = std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).ok()?;
    Some(home.trim_end_matches(['/', '\\']).to_string())
}

fn hide_folder(text: &str, home: &str) -> String {
    // A bare root (`/`, `C:`) is no home folder worth hiding.
    if home.chars().count() <= 2 {
        return text.to_string();
    }
    text.replace(home, "~")
}

/// Encodes an image as PNG.
pub fn png(image: &ColorImage) -> Result<Vec<u8>, String> {
    let [w, h] = image.size;
    let rgba = image::RgbaImage::from_raw(w as u32, h as u32, image.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect()).ok_or_else(|| "image size".to_string())?;
    let mut out = std::io::Cursor::new(Vec::new());
    rgba.write_to(&mut out, image::ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(out.into_inner())
}

/// The lines of the technical details every app shares: app, version, system, language,
/// window and theme.
pub fn environment(ctx: &egui::Context, app: &str, version: &str) -> String {
    let rect = ctx.content_rect();
    format!(
        "- {app} {version}\n- System: {}\n- Language: {}\n- Window: {:.0} × {:.0} points, {:.2} pixels per point\n- Theme: {}\n",
        crate::io::platform(),
        openecraft_i18n::lang().code(),
        rect.width(),
        rect.height(),
        ctx.pixels_per_point(),
        if ctx.global_style().visuals.dark_mode { "dark" } else { "light" },
    )
}

/// A screenshot asked of the window; it is taken when the frame has been drawn.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Option<Arc<ColorImage>>>>);

impl Capture {
    fn request(ctx: &egui::Context) -> Capture {
        let capture = Capture::default();
        let slot = capture.0.clone();
        let repaint = ctx.clone();
        ctx.request_screenshot(move |image| {
            if let Ok(mut slot) = slot.lock() {
                *slot = Some(image);
            }
            repaint.request_repaint();
        });
        capture
    }

    fn take(&self) -> Option<Arc<ColorImage>> {
        self.0.lock().ok()?.take()
    }
}

/// The screenshot being taken: asked for in the frame after `after` (the form and the
/// menu that opened it are gone then), given up at the time `until`.
struct Pending {
    after: u64,
    asked: Option<(Capture, f64)>,
}

/// The report of an app: the draft and the screenshot being taken for it.
#[derive(Default)]
pub struct Report {
    pub draft: Option<Draft>,
    pending: Option<Pending>,
}

impl Report {
    /// Opens the form with `details` and takes a screenshot of the window as it is; a
    /// form already open keeps its answers and gets a new screenshot.
    pub fn begin(&mut self, ctx: &egui::Context, details: String) {
        if self.draft.is_none() {
            self.draft = Some(Draft::new(details));
        }
        self.retake(ctx);
    }

    /// Takes a new screenshot: the form is hidden until it is taken.
    pub fn retake(&mut self, ctx: &egui::Context) {
        self.pending = Some(Pending { after: ctx.cumulative_frame_nr(), asked: None });
        ctx.request_repaint();
    }

    /// The form is hidden while the screenshot is taken.
    pub fn capturing(&self) -> bool {
        self.pending.is_some()
    }

    /// Asks for the screenshot and takes it when it comes; a window that gives none
    /// opens the form without one after a while.
    pub fn poll(&mut self, ctx: &egui::Context) {
        let Some(pending) = &mut self.pending else { return };
        let now = ctx.input(|i| i.time);
        match &pending.asked {
            None if ctx.cumulative_frame_nr() > pending.after => {
                pending.asked = Some((Capture::request(ctx), now + CAPTURE_WAIT));
                ctx.request_repaint();
            }
            None => ctx.request_repaint(),
            Some((capture, until)) => {
                if let Some(image) = capture.take() {
                    if let Some(draft) = &mut self.draft {
                        draft.set_screenshot(image);
                    }
                    self.pending = None;
                } else if now > *until {
                    if let Some(draft) = &mut self.draft {
                        draft.note = Some(Note::NoScreenshot);
                    }
                    self.pending = None;
                } else {
                    ctx.request_repaint_after(std::time::Duration::from_millis(50));
                }
            }
        }
    }

    pub fn close(&mut self) {
        self.draft = None;
        self.pending = None;
    }
}

fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).strong());
}

fn weak(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).weak().small());
}

/// The form of a report (inside the window of the app); `height` is the most the
/// scrolling part takes. Ctrl+Enter (⌘Enter on a Mac) opens the issue form.
pub fn form(ui: &mut egui::Ui, report: &mut Report, height: f32) -> Action {
    let ctx = ui.ctx().clone();
    let Some(draft) = &mut report.draft else { return Action::None };
    let shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Enter);
    // Taken before the fields see it, so it adds no line break.
    let submit = ctx.input_mut(|i| i.consume_shortcut(&shortcut));
    let mut action = Action::None;
    let mut retake = false;
    let mut changed = false;
    egui::ScrollArea::vertical().id_salt("bug_report_form").max_height(height).auto_shrink([false, true]).show(ui, |ui| {
        let guidance = match (draft.opened, draft.too_long()) {
            (false, false) => tr("Opens your browser with the report filled in and copies it. Review and submit it on GitHub."),
            (false, true) => tr("Opens your browser and copies the report for you to paste into the GitHub issue form."),
            (true, false) => tr("Review your report on GitHub and submit it there."),
            (true, true) => tr("Report copied. Paste it into the GitHub issue form and submit it there."),
        };
        weak(ui, guidance);
        weak(ui, tr("Reports are public. A GitHub account is needed to submit them."));
        weak(ui, tr("Offline, or GitHub does not open? Copy the report and submit it later."));
        ui.separator();
        heading(ui, tr("Summary"));
        changed |= ui
            .add(TextEdit::singleline(&mut draft.title).hint_text(tr("A short title for the problem (optional)")).char_limit(MAX_TITLE).desired_width(f32::INFINITY))
            .changed();
        heading(ui, tr("What happened?"));
        let what = ui.add(TextEdit::multiline(&mut draft.what).hint_text(tr("Describe the problem you saw")).desired_rows(3).char_limit(MAX_ANSWER).desired_width(f32::INFINITY));
        if std::mem::take(&mut draft.focus) {
            what.request_focus();
            what.scroll_to_me(None);
        }
        changed |= what.changed();
        if draft.missing {
            if draft.what.trim().is_empty() {
                ui.colored_label(ui.visuals().error_fg_color, tr("Describe what happened before continuing."));
            } else {
                draft.missing = false;
            }
        }
        heading(ui, tr("Steps to reproduce (optional)"));
        changed |= ui
            .add(TextEdit::multiline(&mut draft.steps).hint_text(tr("What did you do just before the problem? A short list is enough")).desired_rows(2).char_limit(MAX_ANSWER).desired_width(f32::INFINITY))
            .changed();
        heading(ui, tr("Expected result (optional)"));
        changed |= ui
            .add(TextEdit::multiline(&mut draft.expected).hint_text(tr("What did you expect to see?")).desired_rows(2).char_limit(MAX_ANSWER).desired_width(f32::INFINITY))
            .changed();
        heading(ui, tr("Screenshot (optional)"));
        if let Some(image) = &draft.screenshot {
            let texture = draft.texture.get_or_insert_with(|| ctx.load_texture("bug_report_screenshot", (**image).clone(), egui::TextureOptions::LINEAR)).clone();
            ui.add(egui::Image::new(&texture).max_size(egui::vec2(ui.available_width(), 160.0)).maintain_aspect_ratio(true));
            weak(ui, tr("The screenshot is not attached by itself: copy or save it, then add it to the GitHub issue form."));
        }
        ui.horizontal_wrapped(|ui| {
            retake = ui.button(tr("Retake screenshot")).on_hover_text(tr("Hides this form and takes a screenshot of the window as it is now")).clicked();
            let has = draft.screenshot.is_some();
            if ui.add_enabled(has, egui::Button::new(tr("Remove screenshot"))).clicked() {
                draft.screenshot = None;
                draft.texture = None;
                draft.note = Some(Note::ScreenshotRemoved);
                changed = true;
            }
            if ui.add_enabled(has, egui::Button::new(tr("Copy screenshot"))).clicked() {
                if let Some(image) = &draft.screenshot {
                    ctx.copy_image((**image).clone());
                    draft.note = Some(Note::ScreenshotCopied);
                }
            }
            if ui.add_enabled(has, egui::Button::new(tr("Save screenshot…"))).clicked() {
                action = Action::SaveScreenshot;
            }
        });
        ui.add_space(4.0);
        weak(ui, tr("The report includes the version, the system and counts of what the open project holds, but no project contents. Review the details before sharing them."));
        egui::CollapsingHeader::new(tr("Technical details")).id_salt("bug_report_details").show(ui, |ui| {
            changed |= ui.add(TextEdit::multiline(&mut draft.details).code_editor().desired_rows(8).desired_width(f32::INFINITY)).changed();
            truncate(&mut draft.details, MAX_DETAILS);
        });
    });
    if changed {
        draft.opened = false;
    }
    if let Some(note) = &draft.note {
        let text = match note {
            Note::Copied => tr("Report copied. Keep it to submit on GitHub when you are online.").to_string(),
            Note::ScreenshotCopied => tr("Screenshot copied. Paste it into the GitHub issue form.").to_string(),
            Note::ScreenshotSaved => tr("Screenshot saved. Attach the PNG file in the GitHub issue form.").to_string(),
            Note::ScreenshotRemoved => tr("Screenshot removed from this report.").to_string(),
            Note::SaveFailed(e) => tr_fmt!("Could not save the screenshot: {}", e),
            Note::NoScreenshot => tr("No screenshot could be taken of this window.").to_string(),
        };
        ui.add_space(4.0);
        ui.label(text);
    }
    ui.separator();
    ui.horizontal(|ui| {
        let open = ui.button(RichText::new(tr("Open GitHub issue form")).strong()).on_hover_text(ctx.format_shortcut(&shortcut));
        if open.clicked() || submit {
            draft.open_issue(&ctx);
        }
        if ui.button(tr("Copy report")).clicked() {
            ctx.copy_text(draft.issue_body());
            draft.note = Some(Note::Copied);
        }
        if ui.button(tr("Close")).clicked() {
            action = Action::Close;
        }
    });
    if retake {
        report.retake(&ctx);
    }
    action
}

#[cfg(test)]
mod tests;
