use super::*;

#[test]
fn commands_in_settings_keep_the_window_contents_open() {
    let ctx = egui::Context::default();
    let mut open = true;
    for frame in 0..4 {
        ctx.run_ui(egui::RawInput { time: Some(frame as f64), ..Default::default() }, |ui| {
            let response = egui::Window::new(tr("Settings")).collapsible(false).open(&mut open).show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label(tr("Updates"));
                    close_command_menu(ui);
                    assert!(!ui.will_parent_close());
                });
            }).unwrap();
            assert!(response.inner.is_some(), "Settings lost its contents in frame {frame}");
        }).textures_delta.clear();
        assert!(open);
    }
}

#[test]
fn commands_in_menus_still_close_the_menu() {
    let ctx = egui::Context::default();
    let mut open = true;
    let mut shown = false;
    ctx.run_ui(egui::RawInput::default(), |ui| {
        let button = ui.button(tr("Help"));
        egui::Popup::menu(&button).open_bool(&mut open).show(|ui| {
            shown = true;
            ui.horizontal(|ui| {
                close_command_menu(ui);
                assert!(ui.will_parent_close());
            });
        });
    }).textures_delta.clear();
    assert!(shown);
    assert!(!open);
}

#[test]
fn undo_analyses_the_project_before_the_panes_read_it() {
    use openecraft_core::installation as inst;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.replace_project(openecraft_core::sample::example("electrical_room").expect("example"), None);
    app.session.refresh(&app.plugins, false);
    while app.session.opening().is_some() {
        app.session.opening_shown();
        app.session.refresh(&app.plugins, false);
    }
    // A socket outlet put into a room and into the schematic of a circuit.
    let breaker = inst::breakers(&app.session.prj, &app.session.analysis.netlist).into_iter().next().expect("breaker").tag;
    let prj = &mut app.session.prj;
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room()).expect("room");
    let (lo, _) = openecraft_core::panel::bounds(&openecraft_core::panel::outline(&prj.layout.enclosures[ri]));
    let q = openecraft_core::panel::site_poses(prj)[ri].apply([lo[0] + 10.0, 0.0, lo[1] + 1000.0]);
    let id = inst::place_at(prj, ri, "GEN.SOCKET-F-UP", [q[0], q[2]], None).expect("placed");
    inst::into_schematic(prj, &breaker, id, &inst::Options::default()).expect("in the schematic");
    app.session.commit(true);
    app.session.refresh(&app.plugins, false);
    app.run(&ctx, Cmd::Undo);
    let an = &app.session.analysis;
    inst::circuits(&app.session.prj, &an.netlist, &an.connections, None);
    app.run(&ctx, Cmd::Redo);
    let an = &app.session.analysis;
    inst::circuits(&app.session.prj, &an.netlist, &an.connections, None);
}

#[test]
fn undo_from_the_ribbon_leaves_the_device_index_to_the_next_frame() {
    use openecraft_core::installation as inst;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.replace_project(openecraft_core::sample::example("electrical_room").expect("example"), None);
    app.session.refresh(&app.plugins, false);
    while app.session.opening().is_some() {
        app.session.opening_shown();
        app.session.refresh(&app.plugins, false);
    }
    let devices = |app: &App| app.session.index.devices.keys().cloned().collect::<Vec<String>>();
    let before = devices(&app);
    let breaker = inst::breakers(&app.session.prj, &app.session.analysis.netlist).into_iter().next().expect("breaker").tag;
    let prj = &mut app.session.prj;
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room()).expect("room");
    let (lo, _) = openecraft_core::panel::bounds(&openecraft_core::panel::outline(&prj.layout.enclosures[ri]));
    let q = openecraft_core::panel::site_poses(prj)[ri].apply([lo[0] + 10.0, 0.0, lo[1] + 1000.0]);
    let id = inst::place_at(prj, ri, "GEN.SOCKET-F-UP", [q[0], q[2]], None).expect("placed");
    inst::into_schematic(prj, &breaker, id, &inst::Options::default()).expect("in the schematic");
    app.session.commit(true);
    app.session.refresh(&app.plugins, false);
    let placed = devices(&app);
    assert_ne!(before, placed);
    // As in `ui`: the index is held while the ribbon runs the command, put back after.
    let frame =|app: &mut App, cmd| {
        let idx = std::mem::take(&mut app.session.index);
        app.run(&ctx, cmd);
        app.session.index = idx;
        app.session.refresh(&app.plugins, false);
    };
    frame(&mut app, Cmd::Undo);
    assert_eq!(devices(&app), before);
    frame(&mut app, Cmd::Redo);
    assert_eq!(devices(&app), placed);
}
