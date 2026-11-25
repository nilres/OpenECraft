//! The leaves of a cabinet's door. A door of several leaves (a double door) splits the
//! front into leaves, each hinged at one of its edges ([`DoorLeaf`]); the door stays one
//! face for its devices, which are on the leaf under their middle: they turn with it in
//! the 3D view and their wires run over its hinge. Part data `Door leaves: 275 + 525`
//! gives the widths of the leaves from the left (`275 left + 525 right` with their
//! hinges; without, a leaf left of the middle is hinged on the left, one right of it on
//! the right), `Door leaves: 2` leaves of equal width.

use super::{DoorHinge, DoorLeaf, Enclosure, Face, PanelItem};

/// The leaves of the door of enclosure `e` from the left: one over the whole front
/// hinged at [`Enclosure::door_hinge`] when it has no leaves of its own.
pub fn leaves(e: &Enclosure) -> Vec<DoorLeaf> {
    if e.door_leaves.len() < 2 {
        return vec![DoorLeaf { x: 0.0, w: e.width, hinge: e.door_hinge }];
    }
    e.door_leaves.clone()
}

/// The index of the leaf at `x` across the front (the nearest one beyond the ends).
pub fn leaf_index(e: &Enclosure, x: f32) -> usize {
    let all = leaves(e);
    all.iter().position(|l| x < l.x + l.w).unwrap_or(all.len().saturating_sub(1))
}

/// The leaf at `x` across the front.
pub fn leaf_at(e: &Enclosure, x: f32) -> DoorLeaf {
    leaves(e)[leaf_index(e, x)]
}

/// The leaf a device on the door is on: the one under its middle.
pub fn leaf_of(e: &Enclosure, it: &PanelItem) -> usize {
    leaf_index(e, it.x + it.w / 2.0)
}

/// Where the leaves of the door meet, across the front.
pub fn joints(e: &Enclosure) -> Vec<f32> {
    let all = leaves(e);
    all.iter().skip(1).map(|l| l.x).collect()
}

/// True if item `it` on the door reaches over a joint between two leaves: it could not
/// be mounted, the leaves open apart.
pub fn crosses_joint(e: &Enclosure, it: &PanelItem) -> bool {
    it.face == Face::Door && joints(e).iter().any(|&j| it.x < j - 0.5 && j + 0.5 < it.x + it.w)
}

/// The leaves of part data `Door leaves` for a front `width` mm wide: widths from the
/// left with an optional hinge (`275 + 525`, `275 left + 525 right`), or a number of
/// leaves of equal width (`2`); scaled to the width when they do not add up to it.
/// `None` for one leaf or what cannot be read.
pub fn parse(text: &str, width: f32) -> Option<Vec<DoorLeaf>> {
    let text = text.trim().trim_end_matches("mm").trim();
    let mut out: Vec<(f32, Option<DoorHinge>)> = Vec::new();
    if let Ok(n) = text.parse::<usize>() {
        if !(2..=8).contains(&n) {
            return None;
        }
        out = vec![(width / n as f32, None); n];
    } else {
        for piece in text.split('+') {
            let mut words = piece.split_whitespace();
            let w: f32 = words.next()?.trim_end_matches("mm").parse().ok().filter(|w: &f32| *w > 0.0)?;
            let hinge = match words.next() {
                Some(h) => Some(DoorHinge::from_key(h)?),
                None => None,
            };
            out.push((w, hinge));
        }
    }
    if out.len() < 2 {
        return None;
    }
    let sum: f32 = out.iter().map(|l| l.0).sum();
    let scale = if (sum - width).abs() > 1.0 && width > 0.0 { width / sum } else { 1.0 };
    let mut x = 0.0;
    let mut leaves = Vec::new();
    for (w, hinge) in out {
        let w = w * scale;
        let hinge = hinge.unwrap_or(if x + w / 2.0 < width / 2.0 { DoorHinge::Left } else { DoorHinge::Right });
        leaves.push(DoorLeaf { x, w, hinge });
        x += w;
    }
    Some(leaves)
}

/// The leaves as part data and command parameters write them (`275 + 525`), the hinges
/// only where they are not the usual ones; empty for one leaf.
pub fn text(e: &Enclosure) -> String {
    if e.door_leaves.len() < 2 {
        return String::new();
    }
    let usual = |l: &DoorLeaf| if l.x + l.w / 2.0 < e.width / 2.0 { DoorHinge::Left } else { DoorHinge::Right };
    e.door_leaves
        .iter()
        .map(|l| if l.hinge == usual(l) { format!("{:.0}", l.w) } else { format!("{:.0} {}", l.w, l.hinge.key()) })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Gives the door of enclosure `e` the leaves of `text` ([`parse`]); an empty text, `1`
/// or `0` makes it one leaf again. False if the text cannot be read.
pub fn set(e: &mut Enclosure, text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() || t == "1" || t == "0" {
        e.door_leaves.clear();
        return true;
    }
    match parse(t, e.width) {
        Some(leaves) => {
            e.door_leaves = leaves;
            true
        }
        None => false,
    }
}

/// Splits the door of enclosure `e` into two leaves meeting at `x` across the front
/// (the left one hinged on the left, the right one on the right); `None` makes it one
/// leaf.
pub fn split(e: &mut Enclosure, x: Option<f32>) {
    e.door_leaves = match x.filter(|x| *x > 1.0 && *x < e.width - 1.0) {
        Some(x) => vec![DoorLeaf { x: 0.0, w: x, hinge: DoorHinge::Left }, DoorLeaf { x, w: e.width - x, hinge: DoorHinge::Right }],
        None => Vec::new(),
    };
}

/// Fits the leaves to a new width of the enclosure: each keeps its share.
pub fn fit_width(e: &mut Enclosure, old_width: f32) {
    if e.door_leaves.is_empty() || old_width <= 0.0 || (old_width - e.width).abs() < 0.01 {
        return;
    }
    let k = e.width / old_width;
    for l in &mut e.door_leaves {
        l.x *= k;
        l.w *= k;
    }
}

#[cfg(test)]
mod tests;
