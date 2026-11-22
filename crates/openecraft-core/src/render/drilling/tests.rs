use super::*;
use crate::sample::demo_project;

#[derive(Default)]
struct Rec {
    circles: Vec<(P, f32)>,
    polys: usize,
    texts: Vec<(P, String)>,
    vertical: Vec<String>,
}

impl Draw for Rec {
    fn line(&mut self, _: P, _: P, _: f32, _: Ink, _: bool) {}
    fn poly(&mut self, _: &[P], _: bool, _: f32, _: Ink, _: bool) {
        self.polys += 1;
    }
    fn circle(&mut self, c: P, r: f32, _: f32, _: Ink, _: bool) {
        self.circles.push((c, r));
    }
    fn text(&mut self, pos: P, text: &str, _: f32, _: Anchor, _: VAnchor, _: Ink) {
        self.texts.push((pos, text.into()));
    }
    fn styled_text(&mut self, pos: P, text: &str, style: &TextStyle, ink: Ink) {
        if style.angle == 90.0 {
            self.vertical.push(text.into());
        }
        self.text(pos, text, style.size, style.h, style.v, ink);
    }
}

#[test]
fn the_drilling_page_shows_every_hole_and_cutout_with_its_dimensions() {
    let mut prj = demo_project();
    let e = prj.layout.enclosures[0].clone();
    let at = { let n = prj.pages.len(); crate::ops::add_page(&mut prj, PageKind::Drilling(e.id), n) };
    assert!(prj.pages[at].description.contains(&e.name));
    let faces = drilling(&e, &prj.parts);
    let holes: usize = faces.iter().flat_map(|f| &f.features).filter(|x| x.outline.is_empty()).count();
    let cutouts: usize = faces.iter().flat_map(|f| &f.features).filter(|x| !x.outline.is_empty()).count();
    assert!(holes > 10 && cutouts > 0, "{holes} holes, {cutouts} cutouts");
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    super::super::draw_page(&mut d, &prj, &an.index, &an, at, &Opts::default());
    // A circle per hole (tapped ones a second for their thread), an outline per cutout.
    let tapped = faces.iter().flat_map(|f| f.features.iter().map(|x| &f.kinds[x.kind])).filter(|k| k.hole.as_ref().is_some_and(|h| !h.thread.is_empty())).count();
    assert_eq!(d.circles.len(), holes + tapped);
    assert!(d.polys >= cutouts);
    // Every x of a middle below the face, every y left of it, from the bottom left.
    for f in &faces {
        for x in &f.features {
            assert!(d.vertical.contains(&dim(x.x)), "x {} of {:?}", dim(x.x), f.face);
            assert!(d.texts.iter().any(|(_, t)| *t == dim(x.y)), "y {} of {:?}", dim(x.y), f.face);
        }
        // The legend: every kind with its letter, size and count.
        for k in &f.kinds {
            assert!(d.texts.iter().any(|(_, t)| *t == k.size()) && d.texts.iter().any(|(_, t)| *t == k.count.to_string()), "{k:?}");
        }
    }
    assert!(d.texts.iter().any(|(_, t)| t.contains("1:")), "the scale");
}

#[test]
fn a_drilling_page_without_its_enclosure_says_so() {
    let mut prj = demo_project();
    let at = { let n = prj.pages.len(); crate::ops::add_page(&mut prj, PageKind::Drilling(9_999_999), n) };
    let an = Analysis::with_builtin(&prj);
    let mut d = Rec::default();
    super::super::draw_page(&mut d, &prj, &an.index, &an, at, &Opts::default());
    assert!(d.texts.iter().any(|(_, t)| t == tr("Enclosure not found - choose another one in the page properties")));
    // The check of the project names the page.
    let issues = crate::checks::run_builtin(&prj, &an);
    assert!(issues.iter().any(|i| i.target.map(|t| t.0) == Some(at)), "{issues:?}");
}
