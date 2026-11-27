use super::*;
use crate::installation::Kind;

#[derive(Default)]
struct Rec {
    texts: Vec<String>,
    fills: usize,
    colored: usize,
    dashed: usize,
    links: Vec<usize>,
}

impl Draw for Rec {
    fn line(&mut self, _: P, _: P, _: f32, ink: Ink, dashed: bool) {
        self.colored += usize::from(matches!(ink, Ink::Color(_)));
        self.dashed += usize::from(dashed);
    }
    fn poly(&mut self, _: &[P], _: bool, _: f32, ink: Ink, fill: bool) {
        self.fills += usize::from(fill && ink == Ink::Fill);
        self.colored += usize::from(matches!(ink, Ink::Color(_)));
    }
    fn circle(&mut self, _: P, _: f32, _: f32, ink: Ink, _: bool) {
        self.colored += usize::from(matches!(ink, Ink::Color(_)));
    }
    fn text(&mut self, _: P, text: &str, _: f32, _: Anchor, _: VAnchor, _: Ink) {
        self.texts.push(text.into());
    }
    fn link(&mut self, _: P, _: P, page: usize) {
        self.links.push(page);
    }
}

/// The office building of the examples with an installation plan page of its ground
/// level.
fn office() -> (Project, u64) {
    let mut prj = crate::sample::example("office_building").expect("example");
    let b = prj.layout.buildings.iter().position(|b| b.name == "+B3").expect("building");
    let level = prj.layout.buildings[b].levels[0].id;
    let id = prj.new_id();
    prj.pages.push(Page::new(id, "IP1", "Ground floor", PageKind::InstallationPlan(level)));
    (prj, level)
}

fn draw(prj: &Project) -> Rec {
    let an = Analysis::with_builtin(prj);
    let idx = crate::index::Index::build(prj);
    let mut rec = Rec::default();
    let i = prj.pages.len() - 1;
    super::super::draw_page(&mut rec, prj, &idx, &an, i, &Opts::default());
    rec
}

#[test]
fn installation_plan_draws_walls_rooms_devices_and_tags() {
    let (mut prj, level) = office();
    // A socket outlet of the example gets a tag.
    let (ri, k) = panel::rooms(&prj.layout)
        .into_iter()
        .filter(|&ri| prj.layout.enclosures[ri].level == level)
        .find_map(|ri| prj.layout.enclosures[ri].items.iter().position(|it| it.part == "GIRA.418803").map(|k| (ri, k)))
        .expect("a socket outlet on the ground level");
    prj.layout.enclosures[ri].items[k].tag = "-XD7".into();
    let rec = draw(&prj);
    let rooms: Vec<&str> = panel::rooms(&prj.layout).into_iter().map(|ri| &prj.layout.enclosures[ri]).filter(|e| e.level == level).map(|e| e.name.as_str()).collect();
    assert!(!rooms.is_empty());
    for r in rooms {
        assert!(rec.texts.iter().any(|t| t == r), "room {r} not named");
    }
    assert!(rec.fills >= 4, "walls drawn as filled bands");
    assert!(rec.texts.iter().any(|t| t == "-XD7"), "tag beside the symbol");
    assert!(rec.texts.iter().any(|t| t.contains("+B3")), "caption names the building");
}

#[test]
fn plan_view_switches_off_tags_names_and_colours() {
    let (mut prj, level) = office();
    let all = draw(&prj);
    let last = prj.pages.len() - 1;
    prj.pages[last].plan = PlanView { tags: false, circuit_colors: false, connections: false, room_names: false, ..Default::default() };
    let rec = draw(&prj);
    assert!(rec.texts.len() < all.texts.len());
    assert_eq!(rec.colored, 0);
    assert_eq!(rec.dashed, 0);
    let poses = panel::site_poses(&prj);
    for i in installation::items(&prj, &poses).into_iter().filter(|i| i.level == level && !i.tag.is_empty()) {
        assert!(!rec.texts.contains(&i.tag));
    }
}

#[test]
fn a_missing_level_is_reported_on_the_page() {
    let (mut prj, _) = office();
    let last = prj.pages.len() - 1;
    prj.pages[last].kind = PageKind::InstallationPlan(u64::MAX);
    let rec = draw(&prj);
    assert!(rec.texts.iter().any(|t| t == tr("Level not found - choose another one in the page properties")));
}

#[test]
fn page_scale_is_kept_and_written_in_the_caption() {
    let (mut prj, _) = office();
    let last = prj.pages.len() - 1;
    prj.pages[last].scale = 100.0;
    let rec = draw(&prj);
    assert!(rec.texts.iter().any(|t| t.ends_with(&tr_fmt!("Scale {}", "1 : 100"))));
}

#[test]
fn every_plan_symbol_is_in_the_library_and_draws() {
    let prj = Project::default();
    for (key, _) in crate::parts::PLAN_SYMBOLS {
        assert!(prj.symbol(key).is_some(), "{key} missing in the library");
        let mut r = Rec::default();
        symbol(&mut r, &prj, key, p(10.0, 10.0), Some(p(0.0, -1.0)), Ink::Color([1, 2, 3]));
        assert!(r.colored > 0, "{key} draws nothing");
    }
    for k in Kind::ALL {
        let key = installation::default_plan_symbol(k, None);
        assert!(crate::parts::PLAN_SYMBOLS.iter().any(|(s, _)| *s == key), "{key} of {k:?} not offered");
    }
}

#[test]
fn a_part_brings_its_own_plan_symbol() {
    let (mut prj, level) = office();
    let poses = panel::site_poses(&prj);
    let socket = installation::items(&prj, &poses).into_iter().find(|i| i.level == level && i.part == "GIRA.418803").expect("socket outlet");
    assert_eq!(socket.symbol, "inst_socket");
    let part = prj.parts.iter_mut().find(|p| p.number == "GIRA.418803").expect("project part");
    part.plan_symbol = "inst_thermostat".into();
    let poses = panel::site_poses(&prj);
    let item = installation::items(&prj, &poses).into_iter().find(|i| i.item == socket.item).unwrap();
    assert_eq!(item.symbol, "inst_thermostat");
    assert!(draw(&prj).texts.iter().any(|t| t == "T"), "the letter of the thermostat symbol is drawn");
    // An unknown key falls back to the symbol of the kind.
    prj.parts.iter_mut().find(|p| p.number == "GIRA.418803").unwrap().plan_symbol = "no_such_symbol".into();
    let poses = panel::site_poses(&prj);
    assert_eq!(installation::items(&prj, &poses).into_iter().find(|i| i.item == socket.item).unwrap().symbol, "inst_socket");
}

#[test]
fn default_plan_symbols_follow_the_subgroup() {
    let part = |sub: &str, number: &str| crate::parts::Part { subgroup: sub.into(), number: number.into(), ..Default::default() };
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Dimmer", "X.DIM"))), "inst_dimmer");
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Switch", "GEN.SW-2WAY-UP"))), "inst_switch2");
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Blind switch", "X"))), "inst_switch_blind");
    assert_eq!(installation::default_plan_symbol(Kind::Socket, Some(&part("Socket outlet", "X.DOUBLE"))), "inst_socket2");
    assert_eq!(installation::default_plan_symbol(Kind::Light, Some(&part("Wall light", "X"))), "inst_wall_light");
    assert_eq!(installation::default_plan_symbol(Kind::Light, None), "inst_lamp");
}

#[test]
fn plan_symbols_are_chosen_by_whole_words() {
    let part = |sub: &str, number: &str| crate::parts::Part { subgroup: sub.into(), number: number.into(), ..Default::default() };
    // "inter" of "interface", "3x" and "pb" inside part numbers choose nothing.
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Switch interface", "X"))), "inst_switch");
    assert_eq!(installation::default_plan_symbol(Kind::Socket, Some(&part("Socket outlet", "1032X00"))), "inst_socket");
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Switch", "APB-12"))), "inst_switch");
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Intermediate switch", "X"))), "inst_switch_inter");
    assert_eq!(installation::default_plan_symbol(Kind::Socket, Some(&part("Socket outlet", "GEN.SOCKET-F-2X-UP"))), "inst_socket2");
    assert_eq!(installation::default_plan_symbol(Kind::Switch, Some(&part("Switch", "X.TWO-WAY"))), "inst_switch2");
}

#[test]
fn symbols_turn_towards_their_wall() {
    #[derive(Default)]
    struct Pts(Vec<P>);
    impl Draw for Pts {
        fn line(&mut self, a: P, b: P, _: f32, _: Ink, _: bool) {
            self.0.extend([a, b]);
        }
        fn poly(&mut self, pts: &[P], _: bool, _: f32, _: Ink, _: bool) {
            self.0.extend_from_slice(pts);
        }
        fn circle(&mut self, c: P, _: f32, _: f32, _: Ink, _: bool) {
            self.0.push(c);
        }
        fn text(&mut self, _: P, _: &str, _: f32, _: Anchor, _: VAnchor, _: Ink) {}
    }
    let prj = Project::default();
    // The wall above: the symbol hangs below the point; the wall to the right: left of it.
    let mut above = Pts::default();
    symbol(&mut above, &prj, "inst_socket", p(0.0, 0.0), Some(p(0.0, -1.0)), Ink::Line);
    assert!(above.0.iter().all(|q| q.y >= -0.01));
    let mut right = Pts::default();
    symbol(&mut right, &prj, "inst_socket", p(0.0, 0.0), Some(p(1.0, 0.0)), Ink::Line);
    assert!(right.0.iter().all(|q| q.x <= 0.01));
}

#[test]
fn plan_view_round_trips_and_is_left_out_when_default() {
    let mut page = Page::new(1, "1", "", PageKind::InstallationPlan(7));
    let json = serde_json::to_string(&page).unwrap();
    assert!(!json.contains("\"plan\""));
    page.plan.tags = false;
    let back: Page = serde_json::from_str(&serde_json::to_string(&page).unwrap()).unwrap();
    assert_eq!(back.plan, page.plan);
    assert_eq!(back.kind, PageKind::InstallationPlan(7));
}

#[test]
fn tag_parts_shorten_the_tags() {
    let (mut prj, level) = office();
    let (ri, k) = panel::rooms(&prj.layout)
        .into_iter()
        .filter(|&ri| prj.layout.enclosures[ri].level == level)
        .find_map(|ri| prj.layout.enclosures[ri].items.iter().position(|it| it.part == "GIRA.418803").map(|k| (ri, k)))
        .expect("a socket outlet on the ground level");
    prj.layout.enclosures[ri].items[k].tag = "=SD1+F1-X1.6".into();
    assert!(draw(&prj).texts.iter().any(|t| t == "=SD1+F1-X1.6"), "the full tag by default");
    let last = prj.pages.len() - 1;
    prj.pages[last].plan.tag_parts = TagParts { function: false, installation: false, location: false, product: true };
    let rec = draw(&prj);
    assert!(rec.texts.iter().any(|t| t == "-X1.6"));
    assert!(!rec.texts.iter().any(|t| t == "=SD1+F1-X1.6"));
    prj.pages[last].plan.tag_parts.location = true;
    assert!(draw(&prj).texts.iter().any(|t| t == "+F1-X1.6"));
}

#[test]
fn cabinets_in_the_rooms_are_drawn_with_their_names() {
    let mut prj = crate::sample::example("plant_buildings").expect("example");
    let (ri, ci) = panel::rooms(&prj.layout)
        .into_iter()
        .find_map(|ri| panel::cabinets_in(&prj.layout, ri).first().map(|&ci| (ri, ci)))
        .expect("a cabinet standing in a room");
    let level = prj.layout.enclosures[ri].level;
    assert_ne!(level, 0, "the room is on a level");
    let id = prj.new_id();
    prj.pages.push(Page::new(id, "IP1", "", PageKind::InstallationPlan(level)));
    let name = prj.layout.enclosures[ci].name.clone();
    let rec = draw(&prj);
    assert!(rec.texts.contains(&name), "{name} not written");
}

#[test]
fn tag_presets_shorten_the_tags() {
    let names: Vec<String> = TagParts::PRESETS.iter().map(|(_, t)| t.apply("=SD1+F1-X1.6")).collect();
    assert_eq!(names, ["=SD1+F1-X1.6", "+F1-X1.6", "=SD1-X1.6", "-X1.6"]);
}

/// Every point the plan itself (without frame and title block) draws.
#[derive(Default)]
struct Pts(Vec<P>);

impl Draw for Pts {
    fn line(&mut self, a: P, b: P, _: f32, _: Ink, _: bool) {
        self.0.extend([a, b]);
    }
    fn poly(&mut self, pts: &[P], _: bool, _: f32, _: Ink, _: bool) {
        self.0.extend_from_slice(pts);
    }
    fn circle(&mut self, c: P, r: f32, _: f32, _: Ink, _: bool) {
        self.0.extend([p(c.x - r, c.y - r), p(c.x + r, c.y + r)]);
    }
    fn text(&mut self, pos: P, _: &str, _: f32, _: Anchor, _: VAnchor, _: Ink) {
        self.0.push(pos);
    }
}

#[test]
fn a_large_scale_is_kept_and_cut_off_at_the_edge() {
    let (mut prj, level) = office();
    let last = prj.pages.len() - 1;
    prj.pages[last].scale = 10.0;
    let an = Analysis::with_builtin(&prj);
    let frame = crate::frame::of_page(&prj, last);
    let mut pts = Pts::default();
    super::draw_page(&mut pts, &prj, &an, &prj.pages[last], level, &frame);
    let a = frame.area();
    assert!(!pts.0.is_empty());
    for q in &pts.0 {
        assert!(q.x >= a.min.x - 0.01 && q.x <= a.max.x + 0.01 && q.y >= a.min.y - 0.01 && q.y <= a.max.y + 0.01, "{q:?} outside {a:?}");
    }
    assert!(draw(&prj).texts.iter().any(|t| t.ends_with(&tr_fmt!("Scale {}", "1 : 10"))));
}

#[test]
fn chosen_rooms_are_fitted_onto_the_page() {
    // The office of the example and a small room beside it on the same level.
    let (mut prj, level) = office();
    let wc = panel::buildings::add_room(&mut prj, "+WC", Some(level)).expect("room");
    let e = &mut prj.layout.enclosures[wc];
    (e.width, e.depth) = (1500.0, 1200.0);
    e.site = Some(crate::model::panel::SitePlace { x: 9000.0, z: 0.0, angle: 0.0 });
    let n = |rec: &Rec| -> f32 {
        let caption = rec.texts.iter().find(|t| t.contains(" 1 : ")).expect("caption");
        caption.split(" : ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap()
    };
    let whole = n(&draw(&prj));
    let rooms = panel::buildings::rooms_on(&prj.layout, level);
    let area = |ri: usize| prj.layout.enclosures[ri].width * prj.layout.enclosures[ri].depth;
    let small = rooms.iter().copied().min_by(|&a, &b| area(a).total_cmp(&area(b))).unwrap();
    let last = prj.pages.len() - 1;
    prj.pages[last].plan.rooms = vec![prj.layout.enclosures[small].name.clone()];
    let one = n(&draw(&prj));
    assert!(one < whole, "1:{one} for one room, 1:{whole} for the level");
    assert!(draw(&prj).texts.iter().any(|t| t.ends_with(&tr_fmt!("Scale {} (fitted)", crate::graphics::scale_text(one)))));
}

#[test]
fn installation_plans_have_the_column_and_row_strips() {
    let (prj, _) = office();
    let f = crate::frame::of_page(&prj, prj.pages.len() - 1);
    assert!(f.strips);
    let rec = draw(&prj);
    assert!(rec.texts.contains(&f.column_labels.label(f.columns - 1)));
    assert!(rec.texts.contains(&f.row_labels.label(f.rows - 1)));
}

#[test]
fn fitting_chooses_a_fine_usual_scale() {
    // A bath of 3.76 m x 2 m on an A3 drawing area: 1:12.5, not the next coarse step 1:20.
    assert_eq!(fit_scale((3760.0, 2000.0), (330.0, 250.0)), 12.5);
    assert_eq!(fit_scale((8240.0, 4000.0), (330.0, 250.0)), 25.0);
    assert_eq!(fit_scale((8300.0, 4000.0), (330.0, 250.0)), 30.0);
}

/// The texts of a page with where and how tall they are drawn.
#[derive(Default)]
struct Texts(Vec<(P, String, f32)>);

impl Draw for Texts {
    fn line(&mut self, _: P, _: P, _: f32, _: Ink, _: bool) {}
    fn poly(&mut self, _: &[P], _: bool, _: f32, _: Ink, _: bool) {}
    fn circle(&mut self, _: P, _: f32, _: f32, _: Ink, _: bool) {}
    fn text(&mut self, pos: P, text: &str, size: f32, _: Anchor, _: VAnchor, _: Ink) {
        self.0.push((pos, text.into(), size));
    }
}

fn texts(prj: &Project) -> Texts {
    let an = Analysis::with_builtin(prj);
    let idx = crate::index::Index::build(prj);
    let mut t = Texts::default();
    super::super::draw_page(&mut t, prj, &idx, &an, prj.pages.len() - 1, &Opts::default());
    t
}

/// The electrical room of the examples on a level of a building, a socket outlet of its
/// circuit -F2 put in, and an installation plan page of the level: a legend with a line.
fn office_with_circuit() -> Project {
    let mut prj = crate::sample::example("electrical_room").expect("example");
    let ri = prj.layout.enclosures.iter().position(|e| e.is_room()).expect("room");
    let b = panel::buildings::add_building(&mut prj, "+B9").expect("building");
    let level = prj.layout.buildings[b].levels[0].id;
    prj.layout.enclosures[ri].level = level;
    let an = Analysis::with_builtin(&prj);
    let f2 = installation::breakers(&prj, &an.netlist).into_iter().map(|b| b.tag).find(|t| t.ends_with("-F2")).expect("-F2");
    let w = panel::side(&prj.layout.enclosures[ri], crate::model::panel::Face::Wall(0)).expect("wall");
    let mid = [(w.a[0] + w.b[0]) / 2.0 + w.normal[0] * 200.0, (w.a[1] + w.b[1]) / 2.0 + w.normal[1] * 200.0];
    let q = panel::site_poses(&prj)[ri].apply([mid[0], 0.0, mid[1]]);
    let id = installation::place_at(&mut prj, ri, "GEN.SOCKET-F-UP", [q[0], q[2]], None).expect("placed");
    installation::into_schematic(&mut prj, &f2, id, &installation::Options::default()).expect("in the schematic");
    let page = prj.new_id();
    prj.pages.push(Page::new(page, "IP1", "", PageKind::InstallationPlan(level)));
    assert!(texts(&prj).0.iter().any(|t| t.1 == tr("Circuits")), "a legend of the circuit");
    prj
}

#[test]
fn the_legend_can_be_switched_off_and_moved() {
    let mut prj = office_with_circuit();
    let last = prj.pages.len() - 1;
    let frame = crate::frame::of_page(&prj, last);
    let a = frame.area();
    let title = |prj: &Project| texts(prj).0.into_iter().find(|t| t.1 == tr("Circuits")).map(|t| t.0);
    let right = title(&prj).expect("legend on the right");
    assert!(right.x > (a.min.x + a.max.x) / 2.0);
    prj.pages[last].plan.legend_at = LegendAt::Left;
    assert!(title(&prj).expect("legend on the left").x < (a.min.x + a.max.x) / 2.0);
    prj.pages[last].plan.legend_at = LegendAt::Top;
    assert!(title(&prj).expect("legend at the top").y < a.min.y + 15.0);
    prj.pages[last].plan.legend_at = LegendAt::Bottom;
    assert!(title(&prj).expect("legend at the bottom").y > (a.min.y + a.max.y) / 2.0);
    prj.pages[last].plan.legend = false;
    assert!(title(&prj).is_none());
}

#[test]
fn rooms_show_their_description_and_text_heights_are_kept() {
    let (mut prj, level) = office();
    let ri = panel::buildings::rooms_on(&prj.layout, level)[0];
    let name = prj.layout.enclosures[ri].name.clone();
    prj.structures.define(crate::structure::Aspect::Location, &room_location(&name), "Office");
    let t = texts(&prj);
    let (n, d) = (t.0.iter().find(|t| t.1 == name).expect("identifier"), t.0.iter().find(|t| t.1 == "Office").expect("description"));
    assert!(d.0.y > n.0.y, "the description below the identifier");
    let last = prj.pages.len() - 1;
    prj.pages[last].plan.room_names = false;
    let t = texts(&prj);
    assert!(t.0.iter().any(|t| t.1 == "Office") && !t.0.iter().any(|t| t.1 == name));
    prj.pages[last].plan.room_descriptions = false;
    prj.pages[last].plan.room_names = true;
    prj.pages[last].plan.room_size = 14.0;
    let t = texts(&prj);
    assert!(!t.0.iter().any(|t| t.1 == "Office"));
    assert_eq!(t.0.iter().find(|t| t.1 == name).unwrap().2, pt_to_mm(14.0));
    assert_eq!(room_location("+KIT"), "KIT");
    assert_eq!(room_location("KIT"), "KIT");
}


#[test]
fn long_legend_texts_are_wrapped_inside_the_frame() {
    let mut prj = office_with_circuit();
    // A long function text of the circuit's protective device.
    let long = "Sockets of the living room, the dining area and the terrace by the garden";
    for pg in &mut prj.pages {
        for s in &mut pg.symbols {
            if s.tag.ends_with("-F2") {
                s.function_text = long.into();
            }
        }
    }
    let last = prj.pages.len() - 1;
    let a = crate::frame::of_page(&prj, last).area();
    let t = texts(&prj);
    let mid = (a.min.x + a.max.x) / 2.0;
    let lines: Vec<&(P, String, f32)> = t.0.iter().filter(|x| x.0.x > mid && (x.1.contains("  Sockets") || (!x.1.is_empty() && long.contains(x.1.as_str())))).collect();
    assert!(lines.len() >= 2, "the text is wrapped: {:?}", t.0.iter().map(|x| &x.1).collect::<Vec<_>>());
    for (pos, text, size) in lines {
        let right = pos.x + printed_width(text, *size);
        assert!(right <= a.max.x + 0.5, "{text} ends at {right}, the frame at {}", a.max.x);
    }
}

#[test]
fn a_legend_line_as_long_as_the_column_is_wrapped_too() {
    // 38 characters fit by the canvas estimate but not as printed (capitals, Helvetica).
    let lines = wrap_printed("=HV+HWR-F14  Diele, WC, HWR Steckdosen", 2.2, LEGEND - 8.0 - 4.0);
    assert!(lines.len() >= 2, "{lines:?}");
    for l in &lines {
        assert!(printed_width(l, 2.2) <= LEGEND - 12.0, "{l}");
    }
}

#[test]
fn font_sizes_are_points_as_printed() {
    // 10 pt as printed: a text of 2.82 mm height (the exports print 1.25 × the height).
    assert!((pt_to_mm(10.0) * 1.25 * 72.0 / 25.4 - 10.0).abs() < 1e-4);
    assert!((pt_to_mm(PlanView::default().tag_size) - 1.8).abs() < 0.05, "the default looks as before");
}

#[test]
fn device_and_cabinet_symbols_can_be_enlarged() {
    let extent = |prj: &Project, key: &str| {
        let mut pts = Pts::default();
        symbol_sized(&mut pts, prj, key, p(0.0, 0.0), None, Ink::Line, 2.4 * if key.is_empty() { 1.0 } else { 2.0 });
        let xs: Vec<f32> = pts.0.iter().map(|q| q.x).collect();
        xs.iter().cloned().fold(f32::MIN, f32::max) - xs.iter().cloned().fold(f32::MAX, f32::min)
    };
    let prj = Project::default();
    let mut normal = Pts::default();
    symbol(&mut normal, &prj, "inst_lamp", p(0.0, 0.0), None, Ink::Line);
    let w = normal.0.iter().map(|q| q.x).fold(f32::MIN, f32::max) - normal.0.iter().map(|q| q.x).fold(f32::MAX, f32::min);
    assert!((extent(&prj, "inst_lamp") - 2.0 * w).abs() < 1e-3, "twice the size");
    // The cabinet symbol at 300 % on the page of the plant example.
    let mut prj = crate::sample::example("plant_buildings").expect("example");
    let ri = panel::rooms(&prj.layout).into_iter().find(|&ri| !panel::cabinets_in(&prj.layout, ri).is_empty()).expect("a cabinet in a room");
    let level = prj.layout.enclosures[ri].level;
    let id = prj.new_id();
    prj.pages.push(Page::new(id, "IP1", "", PageKind::InstallationPlan(level)));
    let name = prj.layout.enclosures[panel::cabinets_in(&prj.layout, ri)[0]].name.clone();
    let below = |prj: &Project| texts(prj).0.into_iter().find(|t| t.1 == name).expect("cabinet name").0.y;
    let small = below(&prj);
    let last = prj.pages.len() - 1;
    prj.pages[last].plan.cabinet_scale = 300.0;
    assert!(below(&prj) > small, "the name stands below a larger symbol");
}


#[test]
fn a_word_wider_than_the_column_is_broken() {
    let lines = wrap_printed("-F1  =HAUPTVERTEILUNG+HAUSWIRTSCHAFTSRAUM-STECKDOSEN", 2.2, 30.0);
    assert!(lines.len() >= 2, "{lines:?}");
    for l in &lines {
        assert!(printed_width(l, 2.2) <= 30.0, "{l}");
    }
}

#[test]
fn a_page_without_room_left_is_fitted_at_the_largest_scale() {
    assert_eq!(fit_scale((5000.0, 4000.0), (300.0, -20.0)), FIT_SCALES[FIT_SCALES.len() - 1]);
    assert_eq!(fit_scale((5000.0, 4000.0), (0.0, 200.0)), FIT_SCALES[FIT_SCALES.len() - 1]);
}

#[test]
fn a_large_legend_at_the_bottom_leaves_room_for_the_plan() {
    let mut prj = office_with_circuit();
    let last = prj.pages.len() - 1;
    prj.pages[last].plan.legend_at = LegendAt::Bottom;
    prj.pages[last].plan.legend_size = 72.0;
    let t = texts(&prj).0;
    let title = t.iter().find(|t| t.1 == tr("Circuits")).expect("legend");
    let room = &prj.layout.enclosures.iter().find(|e| e.is_room()).expect("room").name;
    let name = t.iter().find(|t| &t.1 == room).expect("the room is still drawn");
    assert!(name.0.y < title.0.y);
}

#[test]
fn a_typed_scale_is_written_as_in_the_page_properties() {
    let (mut prj, _) = office();
    let last = prj.pages.len() - 1;
    prj.pages[last].scale = 100.0 / 3.0;
    assert!(draw(&prj).texts.iter().any(|t| t.ends_with(&tr_fmt!("Scale {}", "1 : 33.333"))));
}

/// The colours drawn on a page before its legend and in it.
#[derive(Default)]
struct Colours {
    legend: bool,
    plan: Vec<[u8; 3]>,
    listed: Vec<[u8; 3]>,
}

impl Draw for Colours {
    fn line(&mut self, _: P, _: P, _: f32, ink: Ink, _: bool) {
        self.ink(ink);
    }
    fn poly(&mut self, _: &[P], _: bool, _: f32, ink: Ink, _: bool) {
        self.ink(ink);
    }
    fn circle(&mut self, _: P, _: f32, _: f32, ink: Ink, _: bool) {
        self.ink(ink);
    }
    fn text(&mut self, _: P, text: &str, _: f32, _: Anchor, _: VAnchor, _: Ink) {
        self.legend |= text == tr("Circuits");
    }
}

impl Colours {
    fn ink(&mut self, ink: Ink) {
        if let Ink::Color(c) = ink {
            if self.legend { self.listed.push(c) } else { self.plan.push(c) }
        }
    }
}

#[test]
fn only_circuits_of_the_legend_are_coloured() {
    // One room of the office: devices of neighbouring rooms seen beside it stay black.
    let (mut prj, level) = office();
    let last = prj.pages.len() - 1;
    for ri in panel::buildings::rooms_on(&prj.layout, level) {
        prj.pages[last].plan.rooms = vec![prj.layout.enclosures[ri].name.clone()];
        let an = Analysis::with_builtin(&prj);
        let frame = crate::frame::of_page(&prj, last);
        let mut c = Colours::default();
        super::draw_page(&mut c, &prj, &an, &prj.pages[last], level, &frame);
        for col in &c.plan {
            assert!(c.listed.contains(col), "{col:?} on the plan of {} but not in its legend", prj.layout.enclosures[ri].name);
        }
    }
}
