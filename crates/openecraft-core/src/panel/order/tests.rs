use super::*;
use crate::panel::buildings::{add_building, add_level, add_room};
use crate::panel::{bay_right_of, lineups, new_enclosure, SitePlace};

fn names(prj: &Project) -> Vec<&str> {
    prj.layout.enclosures.iter().map(|e| e.name.as_str()).collect()
}

fn index(prj: &Project, name: &str) -> usize {
    prj.layout.enclosures.iter().position(|e| e.name == name).unwrap()
}

/// Moves the row named `from` before or after the row named `to`.
fn mv(prj: &mut Project, from: &str, to: &str, after: bool) -> usize {
    let (from, to) = (index(prj, from), index(prj, to));
    move_in_tree(prj, from, to, after).unwrap()
}

fn lineup_names(prj: &Project) -> Vec<Vec<&str>> {
    lineups(&prj.layout).into_iter().map(|lu| lu.into_iter().map(|i| prj.layout.enclosures[i].name.as_str()).collect()).collect()
}

fn cabinets(names: &[&str]) -> Project {
    let mut prj = Project::default();
    for name in names {
        let e = new_enclosure(&mut prj, "", name);
        prj.layout.enclosures.push(e);
    }
    prj
}

#[test]
fn enclosures_move_before_or_after_another_one() {
    let mut prj = cabinets(&["+A", "+B", "+C", "+D"]);
    assert_eq!(move_in_tree(&mut prj, 3, 0, false), Ok(0));
    assert_eq!(names(&prj), ["+D", "+A", "+B", "+C"]);
    assert_eq!(move_in_tree(&mut prj, 0, 2, true), Ok(2));
    assert_eq!(names(&prj), ["+A", "+B", "+D", "+C"]);
    assert_eq!(move_in_tree(&mut prj, 1, 1, false), Ok(1), "onto itself");
    assert_eq!(names(&prj), ["+A", "+B", "+D", "+C"]);
    assert!(move_in_tree(&mut prj, 1, 9, false).is_err());
}

#[test]
fn a_bayed_enclosure_moves_within_its_lineup_or_with_it() {
    let mut prj = cabinets(&["+A", "+B", "+C", "+D"]);
    assert!(bay_right_of(&mut prj, 1, 0) && bay_right_of(&mut prj, 2, 1));
    assert_eq!(lineup_names(&prj), [vec!["+A", "+B", "+C"], vec!["+D"]]);
    // Within the line-up: bayed again in the new order, the enclosures stay where they are.
    mv(&mut prj, "+C", "+A", false);
    assert_eq!(lineup_names(&prj), [vec!["+C", "+A", "+B"], vec!["+D"]]);
    assert_eq!(names(&prj), ["+A", "+B", "+C", "+D"]);
    mv(&mut prj, "+C", "+B", true);
    assert_eq!(lineup_names(&prj), [vec!["+A", "+B", "+C"], vec!["+D"]]);
    // Onto another line-up: the whole line-up moves, still bayed.
    let at = mv(&mut prj, "+B", "+D", true);
    assert_eq!(names(&prj), ["+D", "+A", "+B", "+C"]);
    assert_eq!(at, 2);
    assert_eq!(lineup_names(&prj), [vec!["+D"], vec!["+A", "+B", "+C"]]);
}

#[test]
fn the_room_and_place_of_a_lineup_stay_with_its_left_end() {
    let mut prj = cabinets(&["+A", "+B"]);
    let room = crate::panel::new_room(&mut prj, "+R1", 4000.0, 3000.0, 2800.0);
    prj.layout.enclosures.push(room);
    bay_right_of(&mut prj, 1, 0);
    let place = SitePlace { x: 500.0, z: 100.0, angle: 0.0 };
    set_room(&mut prj, 0, Some(2), Some(place)).unwrap();
    move_in_tree(&mut prj, 1, 0, false).unwrap();
    assert_eq!(lineup_names(&prj)[0], ["+B", "+A"]);
    let (a, b) = (&prj.layout.enclosures[0], &prj.layout.enclosures[1]);
    assert_eq!((a.room, a.site, a.bayed_to), (0, None, b.id));
    assert_eq!((b.room, b.site, b.bayed_to), (prj.layout.enclosures[2].id, Some(place), 0));
    assert_eq!(room_of(&prj.layout, 0), Some(2));
}

#[test]
fn an_enclosure_dropped_into_another_room_moves_there() {
    let mut prj = cabinets(&["+A", "+B"]);
    for name in ["+R1", "+R2"] {
        let room = crate::panel::new_room(&mut prj, name, 4000.0, 3000.0, 2800.0);
        prj.layout.enclosures.push(room);
    }
    set_room(&mut prj, 0, Some(2), None).unwrap();
    // Onto a room: into it, after its cabinets.
    let at = mv(&mut prj, "+A", "+R2", true);
    assert_eq!(room_of(&prj.layout, at), Some(index(&prj, "+R2")));
    assert_eq!(names(&prj), ["+B", "+R1", "+R2", "+A"]);
    // Onto a cabinet standing in a room: into that room; onto one outside: out of it.
    let at = mv(&mut prj, "+B", "+A", false);
    assert_eq!(room_of(&prj.layout, at), Some(index(&prj, "+R2")));
    let a = index(&prj, "+A");
    set_room(&mut prj, a, None, None).unwrap();
    let at = mv(&mut prj, "+B", "+A", true);
    assert_eq!(room_of(&prj.layout, at), None);
}

#[test]
fn rooms_move_among_the_rooms_and_onto_their_levels() {
    let mut prj = Project::default();
    let bi = add_building(&mut prj, "+B1").unwrap();
    let ground = add_level(&mut prj, bi, None, None, None).unwrap();
    let upper = add_level(&mut prj, bi, None, None, None).unwrap();
    add_room(&mut prj, "+R1", Some(ground)).unwrap();
    add_room(&mut prj, "+R2", Some(ground)).unwrap();
    add_room(&mut prj, "+R3", Some(upper)).unwrap();
    add_room(&mut prj, "+R4", None).unwrap();
    mv(&mut prj, "+R2", "+R1", false);
    assert_eq!(names(&prj), ["+R2", "+R1", "+R3", "+R4"]);
    // Onto a room of another level: onto that level; onto one outside the buildings: out.
    let at = mv(&mut prj, "+R1", "+R3", true);
    assert_eq!(prj.layout.enclosures[at].level, upper);
    let at = mv(&mut prj, "+R2", "+R4", false);
    assert_eq!(prj.layout.enclosures[at].level, 0);
    assert_eq!(names(&prj), ["+R3", "+R1", "+R2", "+R4"]);
    // A room goes only among rooms.
    let cabinet = new_enclosure(&mut prj, "", "+A");
    prj.layout.enclosures.push(cabinet);
    assert!(move_in_tree(&mut prj, 0, 4, false).is_err());
}

#[test]
fn buildings_change_their_order() {
    let mut prj = Project::default();
    for name in ["+B1", "+B2", "+B3"] {
        add_building(&mut prj, name).unwrap();
    }
    let order = |prj: &Project| prj.layout.buildings.iter().map(|b| b.name.clone()).collect::<Vec<_>>();
    assert_eq!(move_building(&mut prj, 2, 0, false), Ok(0));
    assert_eq!(order(&prj), ["+B3", "+B1", "+B2"]);
    assert_eq!(move_building(&mut prj, 0, 2, true), Ok(2));
    assert_eq!(order(&prj), ["+B1", "+B2", "+B3"]);
    assert_eq!(move_building(&mut prj, 0, 1, false), Ok(0));
    assert_eq!(move_building(&mut prj, 1, 1, true), Ok(1));
    assert_eq!(order(&prj), ["+B1", "+B2", "+B3"]);
}
