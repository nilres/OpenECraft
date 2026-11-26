use super::*;

#[test]
fn creation_menu_offers_both_actions_and_disables_terminals_without_a_strip() {
    for (cmd, terminal, expected) in [
        (Cmd::NewTerminalStrip, false, Some(Cmd::NewTerminalStrip)),
        (Cmd::NewTerminal, false, None),
        (Cmd::NewTerminal, true, Some(Cmd::NewTerminal)),
    ] {
        let ctx = egui::Context::default();
        let mut picked = None;
        let mut frame = |events| {
            let mut out = ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
                if let Some(cmd) = creation_menu(ui, true, terminal) { picked = Some(cmd); }
            });
            out.textures_delta.clear();
            out
        };
        frame(Vec::new());
        let out = frame(Vec::new());
        let text = tr(crate::commands::def(cmd).title);
        let at = out.shapes.iter().find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.job.text == text => Some(t.pos + t.galley.size() * 0.5),
            _ => None,
        }).unwrap();
        frame(vec![egui::Event::PointerMoved(at)]);
        for pressed in [true, false] {
            frame(vec![egui::Event::PointerButton { pos: at, button: egui::PointerButton::Primary, pressed, modifiers: Default::default() }]);
        }
        assert_eq!(picked, expected);
    }
}
