//! Panel layout checks: collisions, clearances (mounting, ventilation, EMC), service
//! space, heights above the floor, collision classes, the thermal simulation and
//! manufacturability ([`openecraft_core::panel::validate`]), with commands for their
//! rules.

use crate::builtin::common::{num_in, NO_MAX};
use crate::params::{optional, param, Args, Kind};
use crate::Registry;
use crate::{Info, Plugin, Text};
use openecraft_core::model::Project;
use openecraft_core::panel::{self, thermal, ClassRule, Enclosure, PanelRules};
use openecraft_core::issue::Issue;
use openecraft_i18n::{tr, tr_const, tr_fmt};
use serde_json::{json, Value};

const ENCLOSURE: &str = tr_const!("Enclosure (location, e.g. +CAB1)");
const CLASS: &str = tr_const!("Collision class: power, drive, control, signal, terminal, rail, duct, busbar, cutout, accessory, an own class or * for any");

pub struct PanelChecks;

impl Plugin for PanelChecks {
    fn info(&self) -> Info {
        Info {
            id: "panel_checks".into(),
            name: Text::Borrowed(tr_const!("Panel layout checks")),
            version: env!("CARGO_PKG_VERSION").into(),
            description: Text::Borrowed(tr_const!("Clearances, service space, heights, collision classes, thermal simulation and manufacturability of the panel layout")),
        }
    }

    fn register(&self, reg: &mut Registry) -> Result<(), String> {
        reg.check("panel_layout", tr_const!("Panel layout checks"), |prj, _| {
            Ok(prj.layout.enclosures.iter().flat_map(|e| panel::validate(prj, e)).map(|f| Issue::plain(f.severity, f.message, None)).collect())
        })
        .command(
            "check_panel",
            tr_const!("Checks the panel layout: collisions, clearances (mounting, ventilation, EMC), service space, heights above the floor, collision classes, temperatures and manufacturability"),
            vec![optional("enclosure", Kind::Str, ENCLOSURE)],
            |prj, a| check_panel(prj, a),
        )
        .command(
            "panel_thermal",
            tr_const!("Thermal simulation of the mounting plate: air temperatures, hot spot, heat sources and the temperature at every device"),
            vec![param("enclosure", Kind::Str, ENCLOSURE)],
            |prj, a| panel_thermal(prj, a),
        )
        .command(
            "panel_rules",
            tr_const!("Shows or changes the rules of the panel layout checks; values in mm (temperature in °C)"),
            vec![
                optional("device_gap", Kind::Num, tr_const!("Gap between devices side by side")),
                optional("duct_gap", Kind::Num, tr_const!("Distance of devices to cable ducts")),
                optional("edge_distance", Kind::Num, tr_const!("Distance of devices to the plate edge")),
                optional("service_space", Kind::Num, tr_const!("Free space in front of every device on the plate")),
                optional("operating_min", Kind::Num, tr_const!("Lowest device above the floor")),
                optional("operating_max", Kind::Num, tr_const!("Highest device above the floor (0: not checked)")),
                optional("actuator_min", Kind::Num, tr_const!("Lowest actuator on the door above the floor")),
                optional("actuator_max", Kind::Num, tr_const!("Highest actuator on the door above the floor (0: not checked)")),
                optional("terminal_min", Kind::Num, tr_const!("Lowest terminal above the floor")),
                optional("device_temperature", Kind::Num, tr_const!("Highest air temperature at devices without data (0: not checked)")),
                optional("stratification", Kind::Num, tr_const!("Temperature rise at the roof against the middle (1 … 2)")),
                optional("hole_web", Kind::Num, tr_const!("Smallest web between holes")),
                optional("hole_edge", Kind::Num, tr_const!("Smallest distance of holes from the sheet edge")),
                optional("cutout_web", Kind::Num, tr_const!("Smallest web at cutouts")),
                optional("min_profile", Kind::Num, tr_const!("Shortest rail, duct or busbar")),
                optional("stock_length", Kind::Num, tr_const!("Stock length of rails, ducts and busbars")),
                optional("reset", Kind::Bool, tr_const!("Back to the default rules first")),
            ],
            panel_rules,
        )
        .command(
            "set_collision_rule",
            tr_const!("Sets the distance between two collision classes or lets them overlap; replaces the rule of the same classes"),
            vec![
                param("a", Kind::Str, CLASS),
                param("b", Kind::Str, CLASS),
                optional("distance", Kind::Num, tr_const!("Smallest distance in mm")),
                optional("ignore", Kind::Bool, tr_const!("The classes may overlap (no collision)")),
                optional("remove", Kind::Bool, tr_const!("Removes the rule")),
            ],
            set_collision_rule,
        )
        .command(
            "set_item_class",
            tr_const!("Sets the collision class of a panel item (empty: automatic)"),
            vec![param("enclosure", Kind::Str, ENCLOSURE), param("id", Kind::Int, tr_const!("Item id")), param("class", Kind::Str, CLASS)],
            set_item_class,
        )
        .command(
            "set_enclosure_elevation",
            tr_const!("Sets the height of the enclosure's bottom above the floor (heights of devices, terminals and actuators are checked)"),
            vec![param("enclosure", Kind::Str, ENCLOSURE), param("elevation", Kind::Num, tr_const!("Height in mm without plinth; negative: unknown (wall mounted)"))],
            set_enclosure_elevation,
        )
        .command(
            "set_enclosure_site",
            tr_const!("Places the line-up of an enclosure in the room (3D view of all enclosures); without x and z it is placed from the plant layout or next to the other line-ups"),
            vec![
                param("enclosure", Kind::Str, ENCLOSURE),
                optional("x", Kind::Num, tr_const!("Rear left corner of the line-up in mm, to the right")),
                optional("z", Kind::Num, tr_const!("Rear left corner of the line-up in mm, towards the front")),
                optional("angle", Kind::Num, tr_const!("Turn about the vertical in degrees, counter-clockwise seen from above (90: the fronts face to the right)")),
            ],
            set_enclosure_site,
        );
        Ok(())
    }
}

fn enclosure_index(prj: &Project, name: &str) -> Result<usize, String> {
    prj.layout.enclosures.iter().position(|e| e.name == name).ok_or_else(|| tr_fmt!("No enclosure named {}", name))
}

fn rules_json(r: &PanelRules) -> Value {
    json!({
        "device_gap": r.device_gap,
        "duct_gap": r.duct_gap,
        "edge_distance": r.edge_distance,
        "service_space": r.service_space,
        "operating": [r.operating_min, r.operating_max],
        "actuator": [r.actuator_min, r.actuator_max],
        "terminal_min": r.terminal_min,
        "device_temperature": r.device_temperature,
        "stratification": r.stratification,
        "hole_web": r.hole_web,
        "hole_edge": r.hole_edge,
        "cutout_web": r.cutout_web,
        "min_profile": r.min_profile,
        "stock_length": r.stock_length,
        "classes": r.classes.iter().map(|c| json!({ "a": c.a, "b": c.b, "distance": c.distance, "ignore": c.ignore })).collect::<Vec<_>>(),
    })
}

fn findings_json(prj: &Project, e: &Enclosure) -> Vec<Value> {
    panel::validate(prj, e)
        .into_iter()
        .map(|f| json!({ "enclosure": e.name, "check": f.kind.key(), "severity": f.severity.key(), "items": f.items, "message": f.message }))
        .collect()
}

fn check_panel(prj: &Project, a: &Args) -> Result<Value, String> {
    let list: Vec<&Enclosure> = match a.str("enclosure") {
        Some(name) => vec![&prj.layout.enclosures[enclosure_index(prj, name)?]],
        None => prj.layout.enclosures.iter().collect(),
    };
    let findings: Vec<Value> = list.into_iter().flat_map(|e| findings_json(prj, e)).collect();
    let warnings = findings.iter().filter(|f| f["severity"] == "warning").count();
    Ok(json!({ "warnings": warnings, "findings": findings }))
}

fn panel_thermal(prj: &Project, a: &Args) -> Result<Value, String> {
    let e = &prj.layout.enclosures[enclosure_index(prj, a.req("enclosure"))?];
    let map = thermal::heat_map(prj, e);
    let tag = |id: u64| panel::item_name(e, id);
    let round = |v: f32| (v * 10.0).round() / 10.0;
    Ok(json!({
        "model": if map.open { "open_assembly" } else { "enclosure" },
        "mean_c": round(map.mean),
        "bottom_c": round(map.bottom),
        "top_c": round(map.top),
        "min_c": round(map.min),
        "max_c": round(map.max),
        "hottest": [round(map.hottest.x), round(map.hottest.y)],
        "sources": map.sources.iter().map(|s| json!({ "device": tag(s.id), "power_loss_w": s.loss, "rise_k": round(s.rise), "duct_above_mm": s.ceiling.map(round) })).collect::<Vec<_>>(),
        "devices": thermal::device_temperatures(prj, e, &map)
            .into_iter()
            .map(|t| json!({ "device": t.tag, "air_c": round(t.temperature), "limit_c": t.limit, "too_warm": t.limit.is_some_and(|l| t.temperature > l) }))
            .collect::<Vec<_>>(),
    }))
}

fn panel_rules(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let r = &mut prj.layout.checks;
    if a.flag("reset") {
        *r = PanelRules::default();
    }
    for (name, field) in [
        ("device_gap", &mut r.device_gap),
        ("duct_gap", &mut r.duct_gap),
        ("edge_distance", &mut r.edge_distance),
        ("service_space", &mut r.service_space),
        ("operating_min", &mut r.operating_min),
        ("operating_max", &mut r.operating_max),
        ("actuator_min", &mut r.actuator_min),
        ("actuator_max", &mut r.actuator_max),
        ("terminal_min", &mut r.terminal_min),
        ("device_temperature", &mut r.device_temperature),
        ("hole_web", &mut r.hole_web),
        ("hole_edge", &mut r.hole_edge),
        ("cutout_web", &mut r.cutout_web),
        ("min_profile", &mut r.min_profile),
        ("stock_length", &mut r.stock_length),
    ] {
        if let Some(v) = num_in(a, name, 0.0, NO_MAX)? {
            *field = v;
        }
    }
    if let Some(v) = num_in(a, "stratification", 1.0, 2.0)? {
        r.stratification = v;
    }
    Ok(rules_json(r))
}

fn set_collision_rule(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let (ca, cb) = (a.req("a").trim().to_ascii_lowercase(), a.req("b").trim().to_ascii_lowercase());
    if ca.is_empty() || cb.is_empty() {
        return Err(tr("Both classes are needed").to_string());
    }
    let classes = &mut prj.layout.checks.classes;
    let same = |r: &ClassRule| (r.a == ca && r.b == cb) || (r.a == cb && r.b == ca);
    let at = classes.iter().position(same);
    if a.flag("remove") {
        let i = at.ok_or_else(|| tr_fmt!("No rule for {} and {}", ca, cb))?;
        classes.remove(i);
    } else {
        let rule = ClassRule { a: ca, b: cb, distance: num_in(a, "distance", 0.0, NO_MAX)?.unwrap_or(0.0), ignore: a.flag("ignore") };
        match at {
            Some(i) => classes[i] = rule,
            None => classes.push(rule),
        }
    }
    Ok(rules_json(&prj.layout.checks))
}

fn set_item_class(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let ei = enclosure_index(prj, a.req("enclosure"))?;
    let id = a.int("id").unwrap_or(0) as u64;
    let it = prj.layout.enclosures[ei].items.iter_mut().find(|i| i.id == id).ok_or_else(|| tr_fmt!("No item {}", id))?;
    it.class = a.req("class").trim().to_ascii_lowercase();
    let it = &prj.layout.enclosures[ei].items.iter().find(|i| i.id == id).unwrap();
    let class = panel::item_class(prj, &panel::device_kinds(prj), it);
    Ok(json!({ "id": id, "class": class, "automatic": it.class.is_empty() }))
}

fn set_enclosure_elevation(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let ei = enclosure_index(prj, a.req("enclosure"))?;
    let v = a.num("elevation").unwrap_or(-1.0);
    let e = &mut prj.layout.enclosures[ei];
    e.elevation = (v >= 0.0).then_some(v);
    Ok(json!({ "enclosure": e.name, "elevation": e.elevation, "floor_height": panel::floor_height(e) }))
}

fn set_enclosure_site(prj: &mut Project, a: &Args) -> Result<Value, String> {
    let ei = enclosure_index(prj, a.req("enclosure"))?;
    let place = match (a.num("x"), a.num("z")) {
        (None, None) => None,
        (x, z) => Some(panel::SitePlace { x: x.unwrap_or_default(), z: z.unwrap_or_default(), angle: a.num("angle").unwrap_or_default() }),
    };
    panel::set_site(prj, ei, place);
    let poses = panel::site_poses(prj);
    let lineup: Vec<Value> = panel::lineup_of(&prj.layout, ei)
        .into_iter()
        .map(|i| json!({ "enclosure": prj.layout.enclosures[i].name, "at": poses[i].at, "angle": poses[i].angle.to_degrees() }))
        .collect();
    Ok(json!({ "automatic": place.is_none(), "lineup": lineup }))
}

#[cfg(test)]
mod tests;
