use super::*;

/// One frame of `ctx`; the textures it made are dropped (no painter here).
fn run(ctx: &egui::Context, input: egui::RawInput, ui: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
    let mut output = ctx.run_ui(input, ui);
    output.textures_delta.clear();
    output
}

fn query(url: &str) -> Vec<(String, String)> {
    let (_, query) = url.split_once('?').expect("a query");
    query
        .split('&')
        .map(|pair| {
            let (key, value) = pair.split_once('=').expect("key=value");
            let mut bytes = Vec::new();
            let raw = value.as_bytes();
            let mut i = 0;
            while i < raw.len() {
                match raw[i] {
                    b'+' => bytes.push(b' '),
                    b'%' => {
                        bytes.push(u8::from_str_radix(std::str::from_utf8(&raw[i + 1..i + 3]).unwrap(), 16).unwrap());
                        i += 2;
                    }
                    b => bytes.push(b),
                }
                i += 1;
            }
            (key.to_string(), String::from_utf8(bytes).unwrap())
        })
        .collect()
}

#[test]
fn the_link_carries_a_typical_report_whole_and_exactly() {
    let mut draft = Draft::new("- OpenECraft 1.0\n- System: windows x86_64".into());
    draft.what = "Wire tool: a 50 % wire & a \"quoted\" + plus #1 = ändern\nsecond line".into();
    draft.steps = "1. Draw\n2. Undo".into();
    let body = draft.issue_body();
    let (url, too_long) = issue_url(&draft.issue_title(), &body);
    assert!(!too_long);
    assert!(url.starts_with("https://github.com/nilres/OpenECraft/issues/new?title="));
    let q = query(&url);
    assert_eq!(q.len(), 2, "no other query parameters: {url}");
    assert_eq!(q[0], ("title".into(), "Wire tool: a 50 % wire & a \"quoted\" + plus #1 = ändern".into()));
    assert_eq!(q[1], ("body".into(), body.clone()));
    assert!(body.contains("## What happened\nWire tool"));
    assert!(body.contains("## Steps to reproduce\n1. Draw\n2. Undo"));
    assert!(body.contains("## Expected result\nNot provided"));
    assert!(body.contains("## Technical details\n- OpenECraft 1.0"));
    assert!(!body.contains("## Screenshot"));
}

#[test]
fn a_long_report_goes_by_the_clipboard_and_the_link_keeps_the_title() {
    let mut draft = Draft::new("x".repeat(MAX_DETAILS));
    draft.what = "Crash".into();
    assert!(draft.too_long());
    let (url, too_long) = issue_url(&draft.issue_title(), &draft.issue_body());
    assert!(too_long);
    assert_eq!(query(&url), vec![("title".to_string(), "Crash".to_string())]);
    // Text that grows when encoded twice (the sign-in page) also falls back.
    let (url, too_long) = issue_url("t", &"\n".repeat(2000));
    assert!(too_long && !url.contains("body="));
    assert!(issue_url("t", &"a".repeat(1000)).0.len() < MAX_URL);
}

#[test]
fn the_title_is_the_summary_or_the_first_line_and_bounded() {
    let mut draft = Draft::new(String::new());
    draft.what = "\n  First line\nsecond".into();
    assert_eq!(draft.issue_title(), "First line");
    draft.title = "  Summary ".into();
    assert_eq!(draft.issue_title(), "Summary");
    draft.title = "ä".repeat(500);
    assert_eq!(draft.issue_title().chars().count(), MAX_TITLE);
}

#[test]
fn answers_and_details_are_bounded_in_characters_and_bytes() {
    let mut draft = Draft::new("ß".repeat(MAX_DETAILS));
    assert!(draft.details.len() <= MAX_DETAILS);
    draft.what = "漢".repeat(MAX_ANSWER + 10);
    let body = draft.issue_body();
    assert!(body.contains(&"漢".repeat(MAX_ANSWER)));
    assert!(!body.contains(&"漢".repeat(MAX_ANSWER + 1)));
}

#[test]
fn the_issue_form_needs_a_description_and_then_opens_with_a_copy() {
    let ctx = egui::Context::default();
    let mut draft = Draft::new("- details".into());
    let output = run(&ctx, Default::default(), |ui| {
        assert!(!draft.open_issue(ui.ctx()));
    });
    assert!(draft.missing && draft.focus);
    assert!(output.platform_output.commands.is_empty());
    draft.what = "It broke".into();
    let output = run(&ctx, Default::default(), |ui| {
        assert!(draft.open_issue(ui.ctx()));
    });
    let commands = &output.platform_output.commands;
    assert!(commands.iter().any(|c| matches!(c, egui::OutputCommand::CopyText(t) if *t == draft.issue_body())));
    let url = commands.iter().find_map(|c| match c {
        egui::OutputCommand::OpenUrl(u) => Some(u.url.clone()),
        _ => None,
    });
    assert_eq!(url, Some(issue_url("It broke", &draft.issue_body()).0));
}

#[test]
fn a_screenshot_is_mentioned_and_saved_as_png() {
    let mut draft = Draft::new(String::new());
    assert!(draft.screenshot_png().is_none());
    draft.set_screenshot(Arc::new(ColorImage::new([3, 2], vec![egui::Color32::RED; 6])));
    assert!(draft.issue_body().ends_with("## Screenshot\n<!-- Paste or drop the screenshot here. -->\n"));
    let bytes = draft.screenshot_png().unwrap().unwrap();
    let back = image::load_from_memory(&bytes).unwrap().to_rgba8();
    assert_eq!((back.width(), back.height()), (3, 2));
    assert_eq!(back.get_pixel(2, 1).0, [255, 0, 0, 255]);
    draft.saved(Err("disk full".into()));
    assert_eq!(draft.note, Some(Note::SaveFailed("disk full".into())));
    draft.saved(Ok(false));
    assert_eq!(draft.note, Some(Note::SaveFailed("disk full".into())), "a cancelled save says nothing");
}

#[test]
fn the_home_folder_is_hidden_but_not_a_bare_root() {
    assert_eq!(hide_folder("cannot open C:\\Users\\Ann\\plan.oec", "C:\\Users\\Ann"), "cannot open ~\\plan.oec");
    assert_eq!(hide_folder("/home/ann/a and /home/ann/b", "/home/ann"), "~/a and ~/b");
    assert_eq!(hide_folder("C:\\x", "C:"), "C:\\x");
    assert_eq!(hide_folder("/x", ""), "/x");
}

#[test]
fn the_form_waits_for_the_screenshot_and_opens_without_one_in_the_end() {
    let ctx = egui::Context::default();
    let mut report = Report::default();
    let frame = |report: &mut Report, time: f64| {
        let input = egui::RawInput { time: Some(time), ..Default::default() };
        let _ = run(&ctx, input, |ui| report.poll(ui.ctx()));
    };
    let _ = run(&ctx, Default::default(), |ui| report.begin(ui.ctx(), "- details".into()));
    assert!(report.capturing());
    // Not asked for in the frame that opened it (the menu that did is still drawn).
    assert!(report.pending.as_ref().is_some_and(|p| p.asked.is_none()));
    frame(&mut report, 0.1);
    assert!(report.pending.as_ref().is_some_and(|p| p.asked.is_some()));
    frame(&mut report, 1.0);
    assert!(report.capturing(), "still waiting");
    frame(&mut report, 3.0);
    assert!(!report.capturing());
    let draft = report.draft.as_ref().unwrap();
    assert_eq!(draft.note, Some(Note::NoScreenshot));
    assert_eq!(draft.details, "- details");
    // Opened again, the answers stay.
    report.draft.as_mut().unwrap().what = "kept".into();
    let _ = run(&ctx, Default::default(), |ui| report.begin(ui.ctx(), "- other".into()));
    assert_eq!(report.draft.as_ref().unwrap().what, "kept");
    report.close();
    assert!(report.draft.is_none() && !report.capturing());
}

#[test]
fn a_screenshot_that_comes_goes_into_the_draft() {
    let ctx = egui::Context::default();
    let mut report = Report::default();
    let _ = run(&ctx, Default::default(), |ui| report.begin(ui.ctx(), String::new()));
    let capture = Capture::default();
    report.pending = Some(Pending { after: 0, asked: Some((capture.clone(), 10.0)) });
    *capture.0.lock().unwrap() = Some(Arc::new(ColorImage::new([1, 1], vec![egui::Color32::WHITE])));
    let _ = run(&ctx, Default::default(), |ui| report.poll(ui.ctx()));
    assert!(!report.capturing());
    assert_eq!(report.draft.as_ref().unwrap().screenshot().map(|i| i.size), Some([1, 1]));
}

#[test]
fn ctrl_enter_in_the_form_opens_the_issue_form() {
    let ctx = egui::Context::default();
    let mut report = Report { draft: Some(Draft::new(String::new())), pending: None };
    report.draft.as_mut().unwrap().what = "x".into();
    let key = egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::COMMAND };
    let input = egui::RawInput { events: vec![key], ..Default::default() };
    let output = run(&ctx, input, |ui| {
        assert_eq!(form(ui, &mut report, 400.0), Action::None);
    });
    assert!(output.platform_output.commands.iter().any(|c| matches!(c, egui::OutputCommand::OpenUrl(_))));
    assert!(report.draft.as_ref().unwrap().opened);
}

#[test]
fn environment_names_app_version_and_language() {
    let ctx = egui::Context::default();
    let mut text = String::new();
    let _ = run(&ctx, Default::default(), |ui| text = environment(ui.ctx(), "OpenECraft", "9.9.9"));
    assert!(text.starts_with("- OpenECraft 9.9.9\n- System: "));
    assert!(text.contains(&format!("- Language: {}\n", openecraft_i18n::lang().code())));
}
