//! Cable bundles and their branches: stretches that several routed items share. The
//! routes are traced through a grid; where the same items run along consecutive
//! cells they form one bundle, and where items join or leave there is a branch. Ways
//! running together lie side by side, each on a lane of its own ([`spread_ways`]);
//! where they still run inside or through each other they are pushed apart
//! ([`separate`]).

use super::{Class, RoutedItem};
use crate::parts::vec3::{add, cross, dist, dot, fillet_each, norm, segment_in_box, sub, V3};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Stretch shared by two or more routed items.
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    /// Indices into the routed items.
    pub items: Vec<usize>,
    pub points: Vec<V3>,
    pub length: f32,
    /// Outer diameter of the bundle, mm.
    pub diameter: f32,
    pub classes: Vec<Class>,
}

/// Where items join or leave bundles.
#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    pub at: V3,
    /// Bundles that meet there (indices).
    pub bundles: Vec<usize>,
    /// Items passing through the branch point.
    pub items: usize,
}

type Key = [i32; 3];

/// Diameter of a bundle of items with the given outer diameters (packed round, 15 %
/// gaps).
pub fn diameter(d: impl Iterator<Item = f32>) -> f32 {
    d.map(|x| x * x).sum::<f32>().sqrt() * 1.15
}

type Edge = (Key, Key);

fn key_of(q: V3, cell: f32) -> Key {
    [0, 1, 2].map(|i| (q[i] / cell).round() as i32)
}

/// The cells a stretch runs through, in order.
fn keys_along(points: &[V3], cell: f32) -> Vec<Key> {
    let mut keys: Vec<Key> = Vec::new();
    for s in points.windows(2) {
        let len = ((s[1][0] - s[0][0]).powi(2) + (s[1][1] - s[0][1]).powi(2) + (s[1][2] - s[0][2]).powi(2)).sqrt();
        let steps = (len / (cell * 0.5)).ceil().max(1.0) as usize;
        for k in 0..=steps {
            let t = k as f32 / steps as f32;
            let q = key_of([0, 1, 2].map(|j| s[0][j] + (s[1][j] - s[0][j]) * t), cell);
            if keys.last() != Some(&q) {
                keys.push(q);
            }
        }
    }
    keys
}

/// The edges between neighbouring cells along the keys (only neighbours along one
/// axis: no jumps).
fn edges_along(keys: &[Key]) -> impl Iterator<Item = Edge> + '_ {
    keys.windows(2).filter(|w| (0..3).map(|j| (w[0][j] - w[1][j]).abs()).sum::<i32>() == 1).map(|w| (w[0].min(w[1]), w[0].max(w[1])))
}

/// The edges between consecutive cells with what runs along them (the owners of the
/// `stretches`), and the chains of edges two or more share (the nodes, the owners).
#[allow(clippy::type_complexity)]
fn chains(stretches: &[(usize, &[V3])], cell: f32) -> (BTreeMap<Edge, BTreeSet<usize>>, Vec<(Vec<Key>, Vec<usize>)>) {
    let mut edges: BTreeMap<Edge, BTreeSet<usize>> = BTreeMap::new();
    for &(owner, points) in stretches {
        for e in edges_along(&keys_along(points, cell)) {
            edges.entry(e).or_default().insert(owner);
        }
    }
    let shared: BTreeMap<Edge, &BTreeSet<usize>> = edges.iter().filter(|(_, s)| s.len() >= 2).map(|(k, s)| (*k, s)).collect();
    let mut at: BTreeMap<Key, Vec<Edge>> = BTreeMap::new();
    for e in edges.keys() {
        at.entry(e.0).or_default().push(*e);
        at.entry(e.1).or_default().push(*e);
    }
    // A node in the middle of a bundle has two edges with the same items.
    let through = |n: Key| -> bool {
        let list = &at[&n];
        list.len() == 2 && edges[&list[0]] == edges[&list[1]]
    };
    let mut out = Vec::new();
    let mut seen: BTreeSet<Edge> = BTreeSet::new();
    for (&e, set) in &shared {
        if seen.contains(&e) {
            continue;
        }
        seen.insert(e);
        let mut chain: Vec<Key> = vec![e.0, e.1];
        // Extend at both ends while the next node continues the same bundle.
        for front in [false, true] {
            loop {
                let (end, before) = if front { (chain[0], chain[1]) } else { (chain[chain.len() - 1], chain[chain.len() - 2]) };
                if !through(end) {
                    break;
                }
                let Some(next) = at[&end].iter().find(|x| **x != (before.min(end), before.max(end))).copied() else { break };
                if seen.contains(&next) {
                    break;
                }
                seen.insert(next);
                let other = if next.0 == end { next.1 } else { next.0 };
                if front {
                    chain.insert(0, other);
                } else {
                    chain.push(other);
                }
            }
        }
        out.push((chain, set.iter().copied().collect()));
    }
    (edges, out)
}

/// Bundles and branches of the routed items (`cell`: grid edge length).
pub fn find(items: &[RoutedItem], cell: f32) -> (Vec<Bundle>, Vec<Branch>) {
    let centre = |k: Key| -> V3 { [0, 1, 2].map(|i| k[i] as f32 * cell) };
    let stretches: Vec<(usize, &[V3])> = items.iter().enumerate().filter(|(_, it)| it.routed()).flat_map(|(i, it)| it.stretches().into_iter().map(move |(_, p)| (i, p))).collect();
    let (edges, chains) = chains(&stretches, cell);
    let shared: BTreeMap<Edge, &BTreeSet<usize>> = edges.iter().filter(|(_, s)| s.len() >= 2).map(|(k, s)| (*k, s)).collect();
    let mut at: BTreeMap<Key, Vec<Edge>> = BTreeMap::new();
    for e in edges.keys() {
        at.entry(e.0).or_default().push(*e);
        at.entry(e.1).or_default().push(*e);
    }
    let through = |n: Key| -> bool {
        let list = &at[&n];
        list.len() == 2 && edges[&list[0]] == edges[&list[1]]
    };
    let mut bundles: Vec<Bundle> = Vec::new();
    for (chain, members) in chains {
        let points = super::grid::simplify(chain.iter().map(|k| centre(*k)).collect());
        let mut classes: Vec<Class> = members.iter().map(|&i| items[i].class).collect();
        classes.sort();
        classes.dedup();
        bundles.push(Bundle {
            length: (chain.len() - 1) as f32 * cell,
            diameter: diameter(members.iter().map(|&i| items[i].diameter)),
            items: members,
            points,
            classes,
        });
    }
    // Branches: nodes where bundles end and items go different ways.
    let mut branches = Vec::new();
    for (&n, list) in &at {
        if through(n) || list.len() < 2 || !list.iter().any(|e| shared.contains_key(e)) {
            continue;
        }
        let c = centre(n);
        let touching: Vec<usize> = (0..bundles.len()).filter(|&b| bundles[b].points.first().is_some_and(|q| same(*q, c)) || bundles[b].points.last().is_some_and(|q| same(*q, c))).collect();
        let passing: BTreeSet<usize> = list.iter().flat_map(|e| edges[e].iter().copied()).collect();
        if !touching.is_empty() && passing.len() >= 2 {
            branches.push(Branch { at: c, bundles: touching, items: passing.len() });
        }
    }
    (bundles, branches)
}

/// The inside of a cable duct the ways may run along: its box and the direction away
/// from the surface it stands on (its base).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Channel {
    pub inside: (V3, V3),
    pub floor: V3,
    /// Height above the floor of its inside up to which its walls are closed (the rim
    /// below the slots of a cable duct): a way crosses them above it.
    pub rim: f32,
    /// Thickness of its side walls (0: open, a cable tray): the strands keep out of
    /// them but where they cross them.
    pub wall: f32,
}

/// A polyline laid out with others by [`spread_ways`]: its corners, their bend radii, its
/// outer diameter, and the surface it runs along (the mounting plate): the direction
/// away from it (none: free) and how far it may come nearer to it.
pub struct Way<'a> {
    pub points: &'a [V3],
    pub radii: &'a [f32],
    pub diameter: f32,
    pub rest: V3,
    pub below: f32,
    /// Points per quarter turn of its bends where [`lay_out`] bends it round its corners.
    pub steps: usize,
    /// Laid as it is (the cores of a cable where they part): the others keep off it.
    pub held: bool,
    /// The direction it leaves its first and its last point in, where known (a
    /// connection point, a cable gland; none: zero): it runs straight out of them. As
    /// long as the cells of the routing there are wide.
    pub leave: [V3; 2],
}

/// The ways drawn apart from those they run together with, with the bend radius of each
/// corner. Straight stretches along the same axis that overlap and lie closer than
/// their thicknesses together (or to one that does) form a group with a lane for each,
/// the lanes side by side around the middle of the group as far apart as its thickest
/// way, so that no two ways run inside each other. Along a surface (the mounting plate)
/// the lanes lie in rows away from it as far as the ways may move towards it and away
/// (as deep as they lie in front of it), each row side by side along the surface;
/// elsewhere they form a square turned like the bundles round their corners (traced
/// through a grid of `cell`). In one of the `channels` (along it) they lie in it as
/// [`crate::panel::duct_lanes`] packs them. Elsewhere the lanes keep out of the
/// `obstacles` (boxes) a way did not run through before and in the `insides` (boxes,
/// the cabinets) it ran in: centred on the group, otherwise all to one side of it (or a
/// corner), otherwise (along a surface) in one stack away from it; where none of them
/// does, the group stays as it is. Its ends stay where they are; from them, and from
/// lane to lane at its corners, a way runs on slanting pieces ([`lay`]).
pub fn spread_ways(ways: &[Way], cell: f32, obstacles: &[(V3, V3)], channels: &[Channel], insides: &[(V3, V3)]) -> Vec<(Vec<V3>, Vec<f32>)> {
    let stretches: Vec<(usize, &[V3])> = ways.iter().enumerate().filter(|(_, w)| !w.held).map(|(i, w)| (i, w.points)).collect();
    let (_, chains) = chains(&stretches, cell);
    // Across each line of the grid the directions of the bundles on it, turned with
    // them round their corners.
    let line_of = |e: &Edge| -> Line {
        let axis = (0..3).find(|&k| e.0[k] != e.1[k]).unwrap_or(0);
        let mut at = e.0;
        at[axis] = 0;
        (axis, at)
    };
    let mut across: BTreeMap<Line, (V3, V3)> = BTreeMap::new();
    for (chain, _) in &chains {
        let dir = |j: usize| -> V3 { [0, 1, 2].map(|k| (chain[j + 1][k] - chain[j][k]) as f32) };
        let first = dir(0);
        let axis = (0..3).find(|&k| first[k] != 0.0).unwrap_or(0);
        let (mut u, mut v) = (unit((axis + 1) % 3), unit((axis + 2) % 3));
        for j in 0..chain.len() - 1 {
            let d = dir(j);
            if j > 0 && d != dir(j - 1) {
                // What pointed along the bundle now points along the new way, what
                // pointed the new way now points back.
                let (before, w) = (dir(j - 1), cross(dir(j - 1), d));
                let turn = |x: V3| add(add(scale(d, dot(x, before)), scale(before, -dot(x, d))), scale(w, dot(x, w)));
                (u, v) = (turn(u), turn(v));
            }
            across.entry(line_of(&(chain[j].min(chain[j + 1]), chain[j].max(chain[j + 1])))).or_insert((u, v));
        }
    }
    let runs = runs_of(ways);
    let groups = groups(ways, &runs);
    // The move of each run onto its lane, across its axis.
    let mut moves: HashMap<(usize, usize), V3> = HashMap::new();
    for members in groups.values().filter(|m| m.len() >= 2) {
        let axis = runs[members[0]].2;
        let middle = scale(members.iter().fold([0.0; 3], |m, &r| add(m, runs[r].5)), 1.0 / members.len() as f32);
        let line = (axis, {
            let mut k = key_of(middle, cell);
            k[axis] = 0;
            k
        });
        // In a duct along it: packed into it.
        let inside = |c: &&Channel| (0..3).all(|k| middle[k] >= c.inside.0[k] - 1.0 && middle[k] <= c.inside.1[k] + 1.0);
        let along = |c: &&Channel| (0..3).all(|k| c.inside.1[axis] - c.inside.0[axis] >= c.inside.1[k] - c.inside.0[k]);
        if let Some(c) = channels.iter().find(|c| inside(c) && along(c)) {
            let at: Vec<(V3, f32)> = members.iter().map(|&r| (runs[r].5, ways[runs[r].0].diameter)).collect();
            // In the order of the sides the ways come into the channel from (or leave it
            // to), so that they do not cross each other there.
            let up = (0..3).find(|&k| k != axis && c.floor[k].abs() > 0.5).unwrap_or((axis + 1) % 3);
            let across_axis = 3 - axis - up;
            let approach = |m: usize| -> f32 {
                let (i, j, ..) = runs[members[m]];
                let pts = ways[i].points;
                let at = pts[j];
                [j.checked_sub(1).map(|k| pts[k]), pts.get(j + 2).copied()].into_iter().flatten().map(|q| q[across_axis] - at[across_axis]).find(|v| v.abs() > 1e-3).unwrap_or(0.0)
            };
            let mut order: Vec<usize> = (0..members.len()).collect();
            order.sort_by(|&a, &b| approach(a).total_cmp(&approach(b)).then(a.cmp(&b)));
            for (&r, to) in members.iter().zip(packed(c, axis, &at, &order)) {
                moves.insert((runs[r].0, runs[r].1), to);
            }
            continue;
        }
        // Across a duct (the short way from one duct into the next at a junction): the
        // runs stay where they are, between their lanes in the ducts.
        if channels.iter().any(|c| inside(&c)) {
            continue;
        }
        let gap = members.iter().map(|&r| ways[runs[r].0].diameter).fold(0.0, f32::max) + 1.0;
        // The surface the ways run along, across their axis.
        let rest = members.iter().map(|&r| ways[runs[r].0].rest).find(|n| *n != [0.0; 3]).map(|mut n| {
            n[axis] = 0.0;
            n
        });
        let rest = rest.filter(|n| dot(*n, *n) > 0.25).map(norm);
        let (u, v, cols, rows) = match rest {
            Some(n) => {
                let below = members.iter().map(|&r| ways[runs[r].0].below).fold(f32::INFINITY, f32::min).max(0.0);
                let rows = ((2.0 * below / gap).floor() as usize + 1).min(members.len());
                (norm(cross(unit(axis), n)), n, members.len().div_ceil(rows), rows)
            }
            None => {
                let (u, v) = across.get(&line).copied().unwrap_or((unit((axis + 1) % 3), unit((axis + 2) % 3)));
                let cols = (members.len() as f32).sqrt().ceil() as usize;
                (u, v, cols, members.len().div_ceil(cols))
            }
        };
        // The moves of the members onto lanes of `cols` × `rows` around the middle,
        // shifted by `aside` lanes along `u` and `v`, away from the surface where they
        // come too near to it.
        let layout = |cols: usize, rows: usize, aside: (f32, f32)| -> Vec<V3> {
            let mut tos: Vec<V3> = members
                .iter()
                .enumerate()
                .map(|(k, &r)| {
                    let (c, row) = ((k % cols) as f32 - (cols - 1) as f32 / 2.0 + aside.0, (k / cols) as f32 - (rows - 1) as f32 / 2.0 + aside.1);
                    let mut to = sub(add(middle, add(scale(u, c * gap), scale(v, row * gap))), runs[r].5);
                    to[axis] = 0.0;
                    to
                })
                .collect();
            if let Some(n) = rest {
                let shift = members.iter().zip(&tos).map(|(&r, to)| -ways[runs[r].0].below - dot(*to, n)).fold(0.0, f32::max);
                for to in &mut tos {
                    *to = add(*to, scale(n, shift));
                }
            }
            tos
        };
        // Moved, no run enters an obstacle it did not run through before or comes further
        // out of an inside it ran in.
        let clear = |tos: &[V3]| {
            members.iter().zip(tos).all(|(&r, to)| {
                let (i, _, axis, from, till, at) = runs[r];
                let (mut a, mut b) = (at, at);
                (a[axis], b[axis]) = (from, till);
                let half = ways[i].diameter / 2.0;
                let kept = |o: V3, (lo, hi): (V3, V3)| {
                    let q = add(o, *to);
                    !within(o, (lo, hi)) || (0..3).all(|k| q[k] >= (lo[k] + half).min(o[k]) - 1e-3 && q[k] <= (hi[k] - half).max(o[k]) + 1e-3)
                };
                insides.iter().all(|inside| kept(a, *inside) && kept(b, *inside))
                    && obstacles.iter().all(|(lo, hi)| {
                        let bx = (lo.map(|x| x - half), hi.map(|x| x + half));
                        segment_in_box(add(a, *to), add(b, *to), bx).is_none() || segment_in_box(a, b, bx).is_some()
                    })
            })
        };
        let (su, sv) = ((cols - 1) as f32 / 2.0, (rows - 1) as f32 / 2.0);
        let mut tries = vec![(cols, rows, (0.0, 0.0))];
        for (a, b) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
            let aside = (a * su, b * sv);
            if aside != (0.0, 0.0) && !tries.iter().any(|t| t.2 == aside) {
                tries.push((cols, rows, aside));
            }
        }
        if cols > 1 && rest.is_some() {
            tries.push((1, members.len(), (0.0, 0.0)));
        }
        if let Some(tos) = tries.into_iter().map(|(c, r, aside)| layout(c, r, aside)).find(|tos| clear(tos)) {
            for (&r, to) in members.iter().zip(tos) {
                moves.insert((runs[r].0, runs[r].1), to);
            }
        }
    }
    ways.iter()
        .enumerate()
        .map(|(i, way)| {
            if way.held {
                return (way.points.to_vec(), way.radii.to_vec());
            }
            let offsets: Vec<V3> = (0..way.points.len().saturating_sub(1)).map(|j| moves.get(&(i, j)).copied().unwrap_or([0.0; 3])).collect();
            let (points, radii) = lay(way.points, way.radii, &offsets, way.leave);
            in_ducts(points, radii, way.diameter / 2.0, channels)
        })
        .collect()
}

/// The axis away from a channel's floor, which way along it, and where the floor is.
fn floor_of(c: &Channel) -> (usize, f32, f32) {
    let up = (0..3).max_by(|&a, &b| c.floor[a].abs().total_cmp(&c.floor[b].abs())).unwrap_or(2);
    let sign = c.floor[up].signum();
    (up, sign, if sign > 0.0 { c.inside.0[up] } else { c.inside.1[up] })
}

/// The polyline with its ways into, out of and through a channel lifted over the closed
/// rim of the channel's walls: where it would cross a wall lower than the rim and half
/// its thickness (`half`), it rises to that height inside the channel at the corner
/// before the wall (or just before the wall, coming through the channel from wall to
/// wall) and runs across at it, down again behind the wall.
fn in_ducts(points: Vec<V3>, radii: Vec<f32>, half: f32, channels: &[Channel]) -> (Vec<V3>, Vec<f32>) {
    let rimmed: Vec<&Channel> = channels.iter().filter(|c| c.rim > 0.0).collect();
    if rimmed.is_empty() || points.len() < 2 {
        return (points, radii);
    }
    let inside = |c: &Channel, x: V3| (0..3).all(|k| x[k] >= c.inside.0[k] - 1.0 && x[k] <= c.inside.1[k] + 1.0);
    // The band of heights above a channel's floor a way lies in: over the rim of its
    // walls (where it can) and under its cover, a millimetre clear of both.
    let band = |c: &Channel| {
        let (up, ..) = floor_of(c);
        let depth = (c.inside.1[up] - c.inside.0[up]).abs();
        let hi = depth - half - 1.0;
        ((c.rim + half + 1.0).min(hi), hi.max(half))
    };
    let at_height = |c: &Channel, x: V3, h: f32| {
        let (up, sign, base) = floor_of(c);
        add(x, scale(c.floor, h - (x[up] - base) * sign))
    };
    let held = |c: &Channel, x: V3| {
        let (up, sign, base) = floor_of(c);
        let (lo, hi) = band(c);
        let h = ((x[up] - base) * sign).clamp(lo, hi);
        add(x, scale(c.floor, h - (x[up] - base) * sign))
    };
    // The corners inside a channel (not the ends of the way) held in its band.
    let n = points.len();
    let points: Vec<V3> = points
        .iter()
        .enumerate()
        .map(|(i, &q)| if i == 0 || i + 1 == n { q } else { rimmed.iter().find(|c| inside(c, q)).map_or(q, |c| held(c, q)) })
        .collect();
    let mut out: Vec<(V3, f32)> = Vec::with_capacity(points.len() + 4);
    for (&q, &r) in points.iter().zip(&radii) {
        let Some(&(p, rp)) = out.last() else {
            out.push((q, r));
            continue;
        };
        let (along, len) = (sub(q, p), dist(p, q));
        let mut q_held: Option<V3> = None;
        if len > 1e-3 {
            let step = scale(along, ((half + 1.0) / len).min(0.5));
            for c in &rimmed {
                let (up, sign, base) = floor_of(c);
                let height = |x: V3| (x[up] - base) * sign;
                let (lo, hi) = band(c);
                let in_band = |h: f32| h >= lo - 1e-3 && h <= hi + 1e-3;
                match (inside(c, p), inside(c, q)) {
                    (true, false) | (false, true) => {
                        // Where it crosses the wall: the end of the part inside, at the
                        // height of the corner inside (held in the band). Outside, the
                        // way runs on at that height for the bend radius of its corner
                        // there, so that its arc stays over the rim and under the cover.
                        let leaving = inside(c, p);
                        let t = segment_in_box(p, q, c.inside).map_or(0.5, |(t0, t1)| if leaving { t1 } else { t0 });
                        let cross = add(p, scale(along, t));
                        let radius = if leaving { rp } else { r };
                        // The corner outside: its arc stays off the wall, and the corner
                        // inside keeps two radii from it (the arcs of both fit between).
                        let from_inside = if leaving { dist(cross, p) } else { dist(cross, q) };
                        let beyond_len = (half + 1.0 + radius).max(2.0 * radius - from_inside);
                        let beyond = scale(along, (beyond_len / len).min(0.5));
                        // In its band there: the way may still leave or enter another
                        // channel on this stretch (from one duct into the next at a
                        // junction), whose band it must keep as well.
                        if in_band(height(cross)) {
                            continue;
                        }
                        // Without room for that corner between the wall and the corner
                        // outside, the corner outside itself is held at the height.
                        let room = beyond_len + from_inside < len;
                        if leaving {
                            let h = height(p).clamp(lo, hi);
                            if let Some(last) = out.last_mut() {
                                last.0 = at_height(c, p, h);
                            }
                            if room {
                                out.push((at_height(c, add(cross, beyond), h), rp));
                            } else {
                                q_held = Some(at_height(c, q, h));
                            }
                        } else {
                            let h = height(q).clamp(lo, hi);
                            if room {
                                out.push((at_height(c, sub(cross, beyond), h), r));
                            } else if let Some(last) = out.last_mut() {
                                last.0 = at_height(c, p, h);
                            }
                        }
                        break;
                    }
                    (false, false) => {
                        let Some((t0, t1)) = segment_in_box(p, q, c.inside).filter(|(t0, t1)| t1 - t0 > 1e-3) else { continue };
                        let (cross_in, cross_out) = (add(p, scale(along, t0)), add(p, scale(along, t1)));
                        if !in_band(height(cross_in)) || !in_band(height(cross_out)) {
                            let h = (height(cross_in).min(height(cross_out))).clamp(lo, hi);
                            out.push((at_height(c, sub(cross_in, step), h), rp));
                            out.push((at_height(c, add(cross_out, step), h), rp));
                        }
                        break;
                    }
                    (true, true) => {}
                }
            }
        }
        out.push((q_held.unwrap_or(q), r));
    }
    out.dedup_by(|a, b| dist(a.0, b.0) < 0.01);
    out.into_iter().unzip()
}

/// Least distance between two segments (for the tests).
#[cfg(test)]
pub(super) fn segment_gap_for_test(a: V3, b: V3, c: V3, d: V3) -> f32 {
    let (u, t) = closest(a, sub(b, a), c, sub(d, c));
    dist(add(a, scale(sub(b, a), u)), add(c, scale(sub(d, c), t)))
}

/// A straight stretch of a way along an axis: way, stretch, axis, from, to, a point.
type Run = (usize, usize, usize, f32, f32, V3);

/// The straight stretches along an axis: way, stretch, axis, from, to, the point. The
/// stretches at the ends (from the connection points) too: they run from their lanes
/// to the end points, but where the way leaves its end in a direction of its own
/// (out of a nozzle, a terminal, a gland): there it stays in line with it.
fn runs_of(ways: &[Way]) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for (i, way) in ways.iter().enumerate().filter(|(_, w)| !w.held) {
        let n = way.points.len();
        // (The way out is the stretch at the end, with the one after it where it only
        // leads from the end point onto the grid of the routing.)
        let short = |j: usize| dist(way.points[j], way.points[j + 1]) <= DOGLEG && (0..3).filter(|&k| (way.points[j][k] - way.points[j + 1][k]).abs() > 1e-3).count() > 1;
        let out = usize::from(way.leave[0] != [0.0; 3]) + usize::from(way.leave[0] != [0.0; 3] && n > 2 && short(0));
        let back = usize::from(way.leave[1] != [0.0; 3]) + usize::from(way.leave[1] != [0.0; 3] && n > 2 && short(n - 2));
        for (j, s) in way.points.windows(2).enumerate() {
            if j < out || j + 1 + back >= n {
                continue;
            }
            let moving: Vec<usize> = (0..3).filter(|&k| (s[0][k] - s[1][k]).abs() > 1e-3).collect();
            if let [axis] = moving[..] {
                runs.push((i, j, axis, s[0][axis].min(s[1][axis]), s[0][axis].max(s[1][axis]), s[0]));
            }
        }
    }
    runs
}

/// Groups of runs that lie inside each other (joined, through the ones in between).
fn groups(ways: &[Way], runs: &[Run]) -> BTreeMap<usize, Vec<usize>> {
    let mut root: Vec<usize> = (0..runs.len()).collect();
    fn find(root: &mut [usize], mut i: usize) -> usize {
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    let thickest = ways.iter().map(|w| w.diameter).fold(0.0, f32::max);
    let mut order: Vec<usize> = (0..runs.len()).collect();
    let side = |r: &Run| r.5[(r.2 + 1) % 3];
    order.sort_by(|&a, &b| runs[a].2.cmp(&runs[b].2).then(side(&runs[a]).total_cmp(&side(&runs[b]))));
    for (n, &a) in order.iter().enumerate() {
        let ra = runs[a];
        for &b in &order[n + 1..] {
            let rb = runs[b];
            if rb.2 != ra.2 || side(&rb) - side(&ra) > thickest + 1.0 {
                break;
            }
            let reach = (ways[ra.0].diameter + ways[rb.0].diameter) / 2.0;
            let apart = (0..3).filter(|&k| k != ra.2).map(|k| (ra.5[k] - rb.5[k]).powi(2)).sum::<f32>().sqrt();
            if ra.0 != rb.0 && apart < reach && ra.4.min(rb.4) - ra.3.max(rb.3) > 1.0 {
                let (x, y) = (find(&mut root, a), find(&mut root, b));
                root[x] = y;
            }
        }
    }
    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for r in 0..runs.len() {
        let g = find(&mut root, r);
        groups.entry(g).or_default().push(r);
    }
    groups
}

/// The moves of ways along `axis` (where they are, their diameter) into the lanes of
/// the channel `c` (across it), in the given `order` across it.
fn packed(c: &Channel, axis: usize, ways: &[(V3, f32)], order: &[usize]) -> Vec<V3> {
    let up = (0..3).find(|&k| k != axis && c.floor[k].abs() > 0.5).unwrap_or((axis + 1) % 3);
    let across = 3 - axis - up;
    let (lo, hi) = c.inside;
    let diameters: Vec<f32> = ways.iter().map(|w| w.1).collect();
    let (lanes, _) = crate::panel::duct_lanes_in_order(hi[across] - lo[across], hi[up] - lo[up], &diameters, order);
    // Rows stacked up from the height the ways were routed at (where they cross the
    // walls of the duct), over the rim of its walls and under its cover.
    let (sign, base) = if c.floor[up] > 0.0 { (1.0, lo[up]) } else { (-1.0, hi[up]) };
    let depth = hi[up] - lo[up];
    let thickest = diameters.iter().fold(0.0f32, |m, d| m.max(*d));
    let (lowest, highest) = lanes.iter().fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), l| (a.min(l[1]), b.max(l[1])));
    let routed = ways.iter().map(|w| (w.0[up] - base) * sign).fold(f32::INFINITY, f32::min);
    let floor_row = routed.max(c.rim + thickest / 2.0).min((depth - thickest / 2.0 - (highest - lowest)).max(thickest / 2.0));
    ways.iter()
        .zip(lanes)
        .map(|(&(at, _), [x, y])| {
            let mut lane = at;
            lane[across] = (lo[across] + hi[across]) / 2.0 + x;
            lane[up] = base + sign * (floor_row + y - lowest);
            let mut to = sub(lane, at);
            to[axis] = 0.0;
            to
        })
        .collect()
}

/// A line of the grid: its axis and the cell across it (the axis left at 0).
type Line = (usize, Key);

fn unit(k: usize) -> V3 {
    [0, 1, 2].map(|i| if i == k { 1.0 } else { 0.0 })
}

fn scale(a: V3, f: f32) -> V3 {
    [a[0] * f, a[1] * f, a[2] * f]
}

/// Longest slanting stretch (mm) that only joins the stretches before and after it
/// ([`lay`]): from an end point onto the grid of the routing (the cells of a room are
/// 50 mm wide), through a cable gland beside the others in it.
const DOGLEG: f32 = 90.0;

/// Length (per mm aside) of the slanting piece on which a way comes onto its lane
/// ([`lay`]): slanting by less than a corner must turn to be bent round
/// ([`crate::parts::vec3::fillet`]).
const LANE_CHANGE: f32 = 5.5;

/// How far a stretch may lean over (aside per length) between a corner moved with an
/// end point and one that is not ([`lay`]).
const TILT: f32 = 0.25;

/// Corners of a way moved with an end point at most ([`lay`]).
const MOVED: usize = 3;

/// How far (mm) beside the line of its first stretch an end point may lie to run
/// straight out along a line of its own ([`lay`]), where the cells of the routing are
/// not known: within them along that stretch.
const IN_LINE: f32 = 12.5;

/// The polyline with its straight stretches moved by `offsets` (one per stretch) onto
/// their lanes, the ends kept. A corner lies where the moved stretches meet, where they
/// do not meet (moved differently across both) midway between them: never a step at
/// the corner. From there a way comes onto its lane on a slanting piece
/// ([`LANE_CHANGE`] times as long as it lies aside, behind the bend of the corner); a
/// stretch too short for that runs straight from corner to corner, the stretches at the
/// ends from the end point to their corner (which lies on the lane of the stretch after
/// it).
///
/// A short slanting stretch ([`DOGLEG`]) has no lane: it joins the stretches before and
/// after it where they end. At an end (from the connection point onto the grid of the
/// routing) it is no stretch of its own: the way leaves its end straight on in
/// the direction `leave` (for the first and the last point; unknown: zero, then that of
/// the stretch after it where that runs on the same way). Its corners from there are
/// moved with it, as the end point lies beside the grid, up to the first stretch that
/// takes that up (it comes back onto its lane or leans over by no more than [`TILT`],
/// the bends of its corners fit on it; [`MOVED`] corners at most); between the corners
/// moved with one end and those moved with the other the longest stretch runs straight. Where the end point lies
/// further aside (the way straight out of it was barred, [`IN_LINE`]), the way comes
/// onto the line of its first stretch as the routing left room for ([`onto_lane`]).
#[allow(clippy::too_many_lines)]
fn lay(points: &[V3], radii: &[f32], offsets: &[V3], leave: [V3; 2]) -> (Vec<V3>, Vec<f32>) {
    let n = points.len();
    let radius = |j: usize| radii.get(j).copied().unwrap_or(0.0);
    let dir = |j: usize| norm(sub(points[j + 1], points[j]));
    let axial = |j: usize| (0..3).filter(|&k| (points[j][k] - points[j + 1][k]).abs() > 1e-3).count() == 1;
    if n < 3 {
        return (points.to_vec(), radii.to_vec());
    }
    // The slanting stretch `end` at an end (`next`: the stretch after it, where there is
    // one): the direction the way runs along it (`out`: where known), and whether the
    // stretch after it is part of that way out of the end.
    let lead = |end: usize, next: Option<usize>, out: V3| -> Option<(V3, bool)> {
        if axial(end) || dist(points[end], points[end + 1]) > DOGLEG {
            return None;
        }
        let on = |j: usize, d: V3| axial(j) && dot(dir(j), d) > 0.99;
        if dot(out, out) > 0.25 {
            let out = norm(out);
            return (dot(dir(end), out) > 0.0).then(|| (out, next.is_some_and(|j| on(j, out))));
        }
        let j = next.filter(|&j| axial(j) && dot(dir(end), dir(j)) > 0.5 && dist(points[end], points[end + 1]) <= dist(points[j], points[j + 1]))?;
        Some((dir(j), true))
    };
    let start = lead(0, (n > 2).then_some(1), leave[0]);
    let first = usize::from(start.is_some_and(|l| l.1));
    let end = lead(n - 2, (n >= 4 + first).then(|| n - 3), scale(leave[1], -1.0));
    let last = n - 1 - usize::from(end.is_some_and(|l| l.1));
    if start.is_none() && end.is_none() && offsets.iter().all(|o| *o == [0.0; 3]) {
        return (points.to_vec(), radii.to_vec());
    }
    // The lanes of the stretches: a point on each and its direction; the way out of an
    // end that is no stretch of its own along the cells of the routing.
    let mut lanes: Vec<(V3, V3)> = (first..last).map(|j| (add(points[j], offsets.get(j).copied().unwrap_or([0.0; 3])), dir(j))).collect();
    if let Some((d, false)) = start {
        lanes[0] = (points[1], d);
    }
    if let (Some((d, false)), Some(lane)) = (end, lanes.last_mut()) {
        *lane = (points[n - 2], d);
    }
    let m = lanes.len();
    let bend = radii.iter().copied().fold(0.0, f32::max);
    // The stretches that only join their neighbours.
    let mut loose: Vec<bool> = (first..last).map(|j| !axial(j) && dist(points[j], points[j + 1]) <= DOGLEG).collect();
    if matches!(start, Some((_, false))) {
        loose[0] = false;
    }
    if matches!(end, Some((_, false))) {
        loose[m - 1] = false;
    }
    let moved = |j: usize| offsets.get(j).copied().unwrap_or([0.0; 3]);
    // The nodes: the end points and the corners between the lanes.
    let mut nodes: Vec<(V3, f32)> = vec![(points[0], 0.0)];
    for (k, pair) in lanes.windows(2).enumerate() {
        let ((a, d1), (b, d2)) = (pair[0], pair[1]);
        if loose[k] || loose[k + 1] {
            let j = first + k + 1;
            nodes.push((add(points[j], add(moved(j - 1), moved(j))), radius(j)));
            continue;
        }
        let (w, along) = (sub(a, b), dot(d1, d2));
        let free = 1.0 - along * along;
        // (Lanes running on nearly the same way meet where the second one begins.)
        let (p, q) = if free > 0.03 {
            let (d, e) = (dot(d1, w), dot(d2, w));
            let (t, s) = ((along * e - d) / free, (e - along * d) / free);
            (add(a, scale(d1, t)), add(b, scale(d2, s)))
        } else {
            (add(a, scale(d1, dot(sub(b, a), d1))), b)
        };
        // The stretch at an end takes all of it: the corner lies on the other lane.
        let share = match (k == 0, k + 2 == m) {
            (true, false) => 1.0,
            (false, true) => 0.0,
            _ => 0.5,
        };
        nodes.push((add(scale(p, 1.0 - share), scale(q, share)), radius(first + k + 1)));
    }
    nodes.push((points[n - 1], 0.0));
    // How far the end points lie beside the lines out of them.
    let beside = |at: V3, (a, d): (V3, V3)| {
        let off = sub(at, a);
        sub(off, scale(d, dot(off, d)))
    };
    // (In the cells along that line: no further beside it than half a cell.)
    let in_line = |v: V3, out: V3| {
        let half = if dot(out, out) > 0.25 { dot(out, out).sqrt() / 2.0 } else { IN_LINE };
        dot(v, v) > 0.0025 && v.iter().all(|c| c.abs() <= half + 0.01)
    };
    let from_start = start.map(|_| beside(points[0], lanes[0])).filter(|v| in_line(*v, leave[0]));
    let from_end = end.map(|_| beside(points[n - 1], lanes[m - 1])).filter(|v| in_line(*v, leave[1]));
    // How far the bend of each node reaches along its stretches: its radius at a right
    // angle, nothing where the way runs straight on.
    let reach: Vec<f32> = (0..=m)
        .map(|k| {
            if k == 0 || k == m {
                return 0.0;
            }
            let turn = dot(lanes[k - 1].1, lanes[k].1).clamp(-1.0, 1.0).acos();
            nodes[k].1 * (turn / 2.0).tan().min(1.0)
        })
        .collect();
    // The stretch `k` comes onto its lane between its nodes at `from` and `to`.
    let onto = |lanes: &[(V3, V3)], k: usize, from: V3, to: V3| {
        let (a, d) = lanes[k];
        let (t0, t1) = (dot(sub(from, a), d), dot(sub(to, a), d));
        let aside = |q: V3, t: f32| dist(q, add(a, scale(d, t)));
        (t0 + reach[k] + LANE_CHANGE * aside(from, t0), t1 - reach[k + 1] - LANE_CHANGE * aside(to, t1))
    };
    // The corners moved with the start (up to `with_start`) and with the end (from
    // `with_end`).
    // The stretch `k` takes it up that one of its nodes is moved (`from` or `to`: where
    // they are then): the bends of its corners still fit on it, and it comes onto its
    // lane between them or leans over by no more than [`TILT`].
    let takes = |k: usize, from: V3, to: V3| {
        let length = dist(from, to);
        let (a, d) = lanes[k];
        let aside = |q: V3| dist(q, add(a, scale(d, dot(sub(q, a), d))));
        let (on, off) = onto(&lanes, k, from, to);
        !loose[k] && length >= reach[k] + reach[k + 1] + 1.0 && (on <= off || (aside(from) - aside(to)).abs() <= TILT * length)
    };
    // The corners moved with the start (up to `with_start`) and with the end (from
    // `with_end`): [`MOVED`] at most.
    let (mut with_start, mut with_end) = (0, m);
    if let Some(v) = from_start.filter(|_| m > 1) {
        with_start = 1;
        while with_start < (m - 1).min(MOVED) && !takes(with_start, add(nodes[with_start].0, v), nodes[with_start + 1].0) {
            with_start += 1;
        }
    }
    if let Some(v) = from_end.filter(|_| m > 1) {
        with_end = m - 1;
        while with_end > 1.max(m.saturating_sub(MOVED)) && !takes(with_end - 1, nodes[with_end - 1].0, add(nodes[with_end].0, v)) {
            with_end -= 1;
        }
    }
    if with_start >= with_end {
        // The longest stretch between them (not the way out of an end, where there is
        // another) runs from the corners of one to the other's.
        let length = |k: usize| dist(nodes[k].0, nodes[k + 1].0);
        let between = with_end.saturating_sub(1)..=with_start.min(m - 1);
        let inner = |k: &usize| *k > 0 && *k + 1 < m;
        let longest = |it: &mut dyn Iterator<Item = usize>| it.max_by(|&a, &b| length(a).total_cmp(&length(b)));
        let split = longest(&mut between.clone().filter(inner)).or_else(|| longest(&mut between.clone())).unwrap_or(with_start);
        (with_start, with_end) = (split, split + 1);
    }
    for (k, node) in nodes.iter_mut().enumerate().take(m).skip(1) {
        if let Some(v) = from_start.filter(|_| k <= with_start) {
            node.0 = add(node.0, v);
        } else if let Some(v) = from_end.filter(|_| k >= with_end) {
            node.0 = add(node.0, v);
        }
    }
    for (k, lane) in lanes.iter_mut().enumerate() {
        if let Some(v) = from_start.filter(|_| k < with_start) {
            lane.0 = add(lane.0, v);
        } else if let Some(v) = from_end.filter(|_| k >= with_end) {
            lane.0 = add(lane.0, v);
        }
    }
    let mut out: Vec<(V3, f32)> = Vec::with_capacity(3 * nodes.len());
    for (k, &(a, d)) in lanes.iter().enumerate() {
        let ((from, r0), (to, r1)) = (nodes[k], nodes[k + 1]);
        out.push((from, r0));
        // From an end point beside its line onto it.
        if k == 0 && start.is_some() && from_start.is_none() {
            out.extend(onto_lane(from, (a, d), to, bend, r1).map(|q| (q, bend)));
        }
        if k + 1 == m && end.is_some() && from_end.is_none() {
            let [p, q] = onto_lane(to, (a, d), from, bend, r0);
            out.extend([(q, bend), (p, bend)]);
        }
        // Onto its lane behind the bends of its corners (their radius: a right angle);
        // the stretches at the ends run straight from the end point to their corner.
        let (on, off) = onto(&lanes, k, from, to);
        let aside = |q: V3| dist(q, add(a, scale(d, dot(sub(q, a), d))));
        if on <= off && k > 0 && k + 1 < m && !loose[k] {
            let r = r0.max(r1);
            if aside(from) > 0.05 {
                out.push((add(a, scale(d, on)), r));
            }
            if aside(to) > 0.05 {
                out.push((add(a, scale(d, off)), r));
            }
        }
    }
    out.push(nodes[nodes.len() - 1]);
    out.dedup_by(|a, b| dot(sub(a.0, b.0), sub(a.0, b.0)) < 0.0025);
    out.into_iter().unzip()
}

/// The corners of the S-bend on which a way with the bend radius `bend` comes from its
/// end point `at` onto the line `lane` (a point and its direction) it runs along to
/// `to`, a corner with a bend of `corner` (0: none): out of the end along the line,
/// across onto it and on. Where leaning over to the line by [`super::grid::LEAN`] takes
/// no longer than that, the point it comes onto the line at (twice). Twice `at` (no
/// corners) where the line passes through it, the way reaches `to` before, or there is
/// no room for the S-bend before the bend at `to`: the way then runs straight to `to`.
fn onto_lane(at: V3, lane: (V3, V3), to: V3, bend: f32, corner: f32) -> [V3; 2] {
    let d = scale(lane.1, dot(sub(to, at), lane.1).signum());
    let off = sub(lane.0, at);
    let across = sub(off, scale(d, dot(off, d)));
    let (aside, length) = (dot(across, across).sqrt(), dot(sub(to, at), d));
    let (run, lean) = (super::grid::onto_line(aside, bend), aside / super::grid::LEAN);
    if aside < 0.5 || bend <= 0.0 {
        return [at; 2];
    }
    // Leaning over to it where that takes no longer than the S-bend and its bend.
    if lean <= run + bend {
        return if lean < length - 1.0 { [add(add(at, scale(d, lean)), across); 2] } else { [at; 2] };
    }
    if run + corner > length + 1.0 {
        return [at; 2];
    }
    let turn = add(at, scale(d, run / 2.0));
    [turn, add(turn, across)]
}

fn same(a: V3, b: V3) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 1e-3)
}

/// A wire, cable or tube drawn apart from the others by [`separate`]: its points (bent
/// round its corners), its outer diameter, the surface it runs along (as [`Way`]) and
/// how long the stretches at its ends are (with their bends): where its end point lies
/// as close to another one's end as their thicknesses (wires on one terminal, terminals
/// closer than the wires are thick) they converge, and the stretch stays as it is.
pub struct Strand<'a> {
    pub points: &'a [V3],
    pub diameter: f32,
    pub rest: V3,
    pub below: f32,
    pub tails: [f32; 2],
    /// Stays as it is: the others keep off it.
    pub held: bool,
}

/// Overlap (mm) of two strands [`separate`] leaves (lanes touch).
pub const STRAND_GAP: f32 = 0.05;

/// Rounds of [`separate`] at most.
pub const ROUNDS: usize = 250;

/// How far (mm) the drawn bends of the ways lie inside their arcs at most ([`lay_out`]).
const ARC_SAG: f32 = 0.25;

/// Overlap (mm) [`separate`] leaves more with every round.
const SLACK_GROWTH: f32 = 0.002;

/// Parts the segments of a round of [`separate`] are searched in (on the threads there are).
const PUSH_PARTS: usize = 16;

/// How far a strand is pushed from where it was routed at most ([`separate`]): this many
/// times its thickness and [`MOST_MOVE_EXTRA`] mm more.
const MOST_MOVE: f32 = 4.0;
const MOST_MOVE_EXTRA: f32 = 20.0;

/// The ways as the 3D view draws them: laid side by side ([`spread_ways`] on a grid of
/// `cell`), bent round their corners ([`Way::steps`] points per quarter turn) and pushed
/// apart ([`separate`]); `progress` as for [`separate_with`].
pub fn lay_out(ways: &[Way], cell: f32, obstacles: &[(V3, V3)], channels: &[Channel], insides: &[(V3, V3)], progress: &dyn Fn(usize, usize) -> bool) -> Option<Vec<Vec<V3>>> {
    let laid = spread_ways(ways, cell, obstacles, channels, insides);
    let bent: Vec<Vec<V3>> = ways
        .iter()
        .zip(&laid)
        .map(|(w, (points, radii))| {
            // Bends fine enough to lie no further than [`ARC_SAG`] inside their arcs: ways
            // side by side round a corner do not meet where their arcs are drawn.
            let widest = radii.iter().copied().fold(0.0, f32::max);
            let steps = if widest > ARC_SAG { (std::f32::consts::FRAC_PI_4 / (1.0 - ARC_SAG / widest).acos()).ceil() as usize } else { 0 };
            let mut bent = fillet_each(points, radii, steps.clamp(w.steps.max(4), 24));
            // Bent, it comes no nearer to the surface it runs along than its lowest corner.
            if w.rest != [0.0; 3] {
                let least = points.iter().map(|q| dot(*q, w.rest)).fold(f32::INFINITY, f32::min);
                for q in &mut bent {
                    let h = dot(*q, w.rest);
                    if h < least {
                        *q = add(*q, scale(w.rest, least - h));
                    }
                }
            }
            bent
        })
        .collect();
    let tails = |points: &[V3], radii: &[f32]| -> [f32; 2] {
        let n = points.len();
        let r = |i: usize| radii.get(i).copied().unwrap_or(0.0);
        if n < 2 {
            return [0.0; 2];
        }
        [dist(points[0], points[1]) + r(1), dist(points[n - 2], points[n - 1]) + r(n - 2)]
    };
    let strands: Vec<Strand> = ways.iter().zip(&laid).zip(&bent).map(|((w, (points, radii)), bent)| Strand { points: bent, diameter: w.diameter, rest: w.rest, below: w.below, tails: tails(points, radii), held: w.held }).collect();
    separate_with(&strands, obstacles, channels, insides, progress)
}

/// Where a drawn way bends tighter than with `radius`: the places (the middle of each,
/// one per bend) and the radius it bends with there. Along no stretch of it does it
/// turn by more than its length over the radius; ripples smaller than a fiftieth of
/// the radius (half a millimetre at most) are not counted, nor is what a bend drawn
/// that roughly may turn in one point.
pub fn tight_bends(points: &[V3], radius: f32) -> Vec<(V3, f32)> {
    if radius <= 0.0 {
        return Vec::new();
    }
    let rough = (radius / 50.0).clamp(0.05, 0.5);
    let pts = straighten(points, rough);
    if pts.len() < 3 {
        return Vec::new();
    }
    let turn: Vec<f32> = (1..pts.len() - 1).map(|i| dot(norm(sub(pts[i], pts[i - 1])), norm(sub(pts[i + 1], pts[i]))).clamp(-1.0, 1.0).acos()).collect();
    let allow = 2.2 * (1.0 - rough / radius).acos() + 0.1;
    let mut out: Vec<(V3, f32)> = Vec::new();
    let mut i = 0;
    while i < turn.len() {
        let (mut turned, mut along, mut worst) = (0.0, 0.0, None::<(usize, f32)>);
        for j in i..turn.len() {
            if j > i {
                along += dist(pts[j], pts[j + 1]);
            }
            if along > 3.0 * radius {
                break;
            }
            turned += turn[j];
            let over = turned - allow;
            if over > along / (0.9 * radius) + 1e-3 {
                let r = along / over;
                if worst.is_none_or(|w| r < w.1) {
                    worst = Some((j, r));
                }
            }
        }
        match worst {
            Some((j, r)) => {
                out.push((scale(add(pts[i + 1], pts[j + 1]), 0.5), r));
                i = j + 1;
            }
            None => i += 1,
        }
    }
    out
}

/// The strands pushed apart where they still run inside or through each other: sampled
/// along their length, each piece moved off the others by its share of their overlap
/// (one over the other where they cross) and the move spread along the strand, round
/// after round until none overlap (or [`ROUNDS`]). Their ends (twice their thickness
/// from the connection points, the stretches at the ends where they converge with
/// others: [`Strand::tails`]) and what lies inside one of the `obstacles` stay; none
/// moves further than [`MOST_MOVE`] times its thickness from where it was, no sample enters an obstacle it kept out of, comes further out of a channel's inside or
/// one of the `insides` (the cabinets) it was in, or nearer to the surface it runs along
/// than `below`. Straight where they did not move.
pub fn separate(strands: &[Strand], obstacles: &[(V3, V3)], channels: &[Channel], insides: &[(V3, V3)]) -> Vec<Vec<V3>> {
    separate_with(strands, obstacles, channels, insides, &|_, _| true).unwrap_or_default()
}

/// Like [`separate`], telling `progress` how many of at most [`ROUNDS`] rounds are done
/// before each; `None` when it answers false (stopped).
#[allow(clippy::too_many_lines)]
pub fn separate_with(strands: &[Strand], obstacles: &[(V3, V3)], channels: &[Channel], insides: &[(V3, V3)], progress: &dyn Fn(usize, usize) -> bool) -> Option<Vec<Vec<V3>>> {
    let radius: Vec<f32> = strands.iter().map(|s| s.diameter.max(0.0) / 2.0).collect();
    let Samples { mut at, owner, mut fixed, arc, spans } = samples(strands, obstacles);
    let was = at.clone();
    // The boxes a sample stays in (where it was in one) and those it keeps out of, by the
    // cells of a grid they reach into (the obstacles grown by the thickest strand).
    let walls: Vec<(V3, V3)> = channels.iter().map(|c| c.inside).chain(insides.iter().copied()).collect();
    let thickest = radius.iter().fold(0.0f32, |m, r| m.max(*r));
    let grown: Vec<(V3, V3)> = obstacles.iter().map(|(lo, hi)| (lo.map(|v| v - thickest), hi.map(|v| v + thickest))).collect();
    // Over the closed rim of a channel's walls a sample stays, in the channel and up to
    // its thickness outside it: the axis away from the floor, which way, the floor, the
    // rim.
    let rimmed: Vec<&Channel> = channels.iter().filter(|c| c.rim > 0.0).collect();
    // The side walls of the channels (along their length, from the floor to the top of
    // the inside): a sample keeps out of them, unless it was in one (crossing it).
    let fences: Vec<(V3, V3)> = channels
        .iter()
        .filter(|c| c.wall > 0.0)
        .flat_map(|c| {
            let (up, ..) = floor_of(c);
            let (lo, hi) = c.inside;
            let along = (0..3).filter(|&k| k != up).max_by(|&a, &b| (hi[a] - lo[a]).abs().total_cmp(&(hi[b] - lo[b]).abs())).unwrap_or(0);
            let across = 3 - up - along;
            let side = |a: f32, b: f32| {
                let (mut l, mut h) = (lo, hi);
                (l[across], h[across]) = (a.min(b), a.max(b));
                (l, h)
            };
            [side(lo[across] - c.wall, lo[across]), side(hi[across], hi[across] + c.wall)]
        })
        .collect();
    let rims: Vec<(usize, f32, f32, f32, f32)> = rimmed
        .iter()
        .map(|c| {
            let (up, sign, floor) = floor_of(c);
            (up, sign, floor, c.rim, (c.inside.1[up] - c.inside.0[up]).abs())
        })
        .collect();
    // Reaching beyond the ends of the channel too, so that a strand leaving it at a
    // junction rises only two thicknesses on.
    let rim_boxes: Vec<(V3, V3)> = rimmed.iter().map(|c| (c.inside.0.map(|v| v - 3.0 * thickest - 1.0), c.inside.1.map(|v| v + 3.0 * thickest + 1.0))).collect();
    let fence_grown: Vec<(V3, V3)> = fences.iter().map(|(lo, hi)| (lo.map(|v| v - thickest), hi.map(|v| v + thickest))).collect();
    let (wall_cells, obstacle_cells, rim_cells, fence_cells) = (BoxGrid::new(&walls), BoxGrid::new(&grown), BoxGrid::new(&rim_boxes), BoxGrid::new(&fence_grown));
    let keep = |i: usize, mut p: V3| -> V3 {
        let (o, r, st) = (was[i], radius[owner[i]], &strands[owner[i]]);
        // Never further than a few times its thickness from where it was routed.
        let (moved, most) = (sub(p, o), MOST_MOVE * st.diameter + MOST_MOVE_EXTRA);
        let far = dot(moved, moved).sqrt();
        if far > most {
            p = add(o, scale(moved, most / far));
        }
        if st.rest != [0.0; 3] {
            let (h, least) = (dot(sub(p, o), st.rest), -st.below.max(0.0));
            if h < least {
                p = add(p, scale(st.rest, least - h));
            }
        }
        for b in wall_cells.at(o).iter().map(|&w| walls[w as usize]).filter(|b| within(o, *b)) {
            for k in 0..3 {
                p[k] = p[k].clamp((b.0[k] + r).min(o[k]), (b.1[k] - r).max(o[k]));
            }
        }
        for w in rim_cells.at(o).iter().map(|&w| w as usize).filter(|&w| within(o, rim_boxes[w])) {
            let (up, sign, floor, rim, depth) = rims[w];
            let least = floor + sign * (rim + r);
            if (o[up] - least) * sign >= -0.01 && (p[up] - least) * sign < 0.0 {
                p[up] = least;
            }
            // And under its cover.
            let most = floor + sign * (depth - r - 1.0);
            if (o[up] - most) * sign <= 0.01 && (p[up] - most) * sign > 0.0 {
                p[up] = most;
            }
        }
        let out_of = |p: &mut V3, lo: V3, hi: V3| {
            // Out the nearest side.
            let (k, low) = (0..3).flat_map(|k| [(k, true), (k, false)]).min_by(|a, b| {
                let depth = |(k, low): (usize, bool)| if low { p[k] - lo[k] } else { hi[k] - p[k] };
                depth(*a).total_cmp(&depth(*b))
            }).unwrap_or((0, true));
            p[k] = if low { lo[k] - 0.01 } else { hi[k] + 0.01 };
        };
        for &(lo, hi) in fence_cells.at(p).iter().map(|&b| &fences[b as usize]) {
            let (glo, ghi) = (lo.map(|v| v - r), hi.map(|v| v + r));
            if within(p, (glo, ghi)) && !within(o, (lo, hi)) {
                out_of(&mut p, glo, ghi);
            }
        }
        for &(lo, hi) in obstacle_cells.at(p).iter().map(|&b| &obstacles[b as usize]) {
            let (lo, hi) = (lo.map(|v| v - r), hi.map(|v| v + r));
            if within(p, (lo, hi)) && !within(o, (lo, hi)) {
                out_of(&mut p, lo, hi);
            }
        }
        p
    };
    // Segments (by their first sample) in every cell of a grid they come within their
    // thickness of: the thin ones in small cells, the others in cells as large as the
    // thickest.
    let thin = 4.0f32;
    let coarse = (2.0 * thickest).max(2.0 * thin);
    let level = |r: f32| usize::from(r > thin);
    let size = [2.0 * thin, coarse];
    let reach = |l: usize, a: V3, b: V3, r: f32| -> ([i32; 3], [i32; 3]) {
        let key = |v: f32| (v / size[l]).floor() as i32;
        ([0, 1, 2].map(|k| key(a[k].min(b[k]) - r)), [0, 1, 2].map(|k| key(a[k].max(b[k]) + r)))
    };
    let mut grids: [HashMap<[i32; 3], Vec<u32>>; 2] = [HashMap::new(), HashMap::new()];
    let segment = |i: usize| i + 1 < owner.len() && owner[i] == owner[i + 1];
    let mut placed: Vec<([i32; 3], [i32; 3])> = vec![([0; 3], [-1; 3]); at.len()];
    for i in (0..at.len()).filter(|&i| segment(i)) {
        let r = radius[owner[i]];
        placed[i] = reach(level(r), at[i], at[i + 1], r);
        for c in cells(placed[i]) {
            grids[level(r)].entry(c).or_default().push(i as u32);
        }
    }
    // Sums of the pushes and moves of each sample in a round (and which have one).
    let mut pushes: Vec<(V3, f32)> = vec![([0.0; 3], 0.0); at.len()];
    let mut spread: Vec<(V3, f32)> = vec![([0.0; 3], 0.0); at.len()];
    let (mut pushed, mut moved): (Vec<usize>, Vec<usize>) = (Vec::new(), Vec::new());
    let add_to = |sums: &mut [(V3, f32)], touched: &mut Vec<usize>, k: usize, v: V3, w: f32| {
        if sums[k].1 == 0.0 {
            touched.push(k);
        }
        sums[k] = (add(sums[k].0, scale(v, w)), sums[k].1 + w);
    };
    // A move is refused where the segment to a neighbouring sample would come to cut
    // an obstacle (grown by the strand's radius) it is clear of: the samples are
    // apart, a corner of a box fits between them.
    let cuts = |at: &[V3], k: usize, to: V3| -> bool {
        let (span, r) = (&spans[owner[k]], radius[owner[k]]);
        [k.wrapping_sub(1), k + 1].into_iter().filter(|&n| n != usize::MAX && span.contains(&n)).any(|n| {
            let q = at[n];
            let boxes = obstacle_cells.at(to).iter().chain(obstacle_cells.at(q)).map(|&b| obstacles[b as usize]);
            let fenced = fence_cells.at(to).iter().chain(fence_cells.at(q)).map(|&b| fences[b as usize]);
            boxes.chain(fenced).any(|(lo, hi)| {
                let (lo, hi) = (lo.map(|v| v - r), hi.map(|v| v + r));
                !segment_meets_box(q, at[k], lo, hi) && segment_meets_box(q, to, lo, hi)
            })
        })
    };
    // How often a sample was kept from moving: six times, and it stays (the others give
    // way).
    let mut stuck = vec![0u8; at.len()];
    let mut active: Vec<usize> = (0..at.len()).filter(|&i| !fixed[i]).collect();
    for round in 0..ROUNDS {
        if !progress(round, ROUNDS) {
            return None;
        }
        // Each segment off the segments of the others it overlaps: by half the overlap
        // (all of it off one that stays), its ends as far as they are near.
        let mut segments: Vec<usize> = active.iter().flat_map(|&i| [i.wrapping_sub(1), i]).filter(|&i| i != usize::MAX && segment(i)).collect();
        segments.dedup();
        let mut searched = vec![false; at.len()];
        for &i in &segments {
            searched[i] = true;
        }
        // The overlap left grows a little with every round, so that they come to rest.
        let slack = STRAND_GAP + SLACK_GROWTH * round as f32;
        // Searched in parts on the threads there are, added up in order.
        let parts: Vec<&[usize]> = segments.chunks(segments.len().div_ceil(PUSH_PARTS).max(64)).collect();
        let found = crate::parallel::par_map(&parts, |part| {
            let mut out: Vec<(usize, V3, f32)> = Vec::new();
            let mut seen: Vec<usize> = Vec::new();
            for &i in *part {
                let (a, b, s) = (at[i], at[i + 1], owner[i]);
                seen.clear();
                for (l, grid) in grids.iter().enumerate() {
                    for c in cells(reach(l, a, b, radius[s])) {
                        for &j in grid.get(&c).map(Vec::as_slice).unwrap_or(&[]) {
                            let j = j as usize;
                            if owner[j] == s || seen.contains(&j) {
                                continue;
                            }
                            seen.push(j);
                            let need = radius[s] + radius[owner[j]];
                            let (own, other) = (sub(b, a), sub(at[j + 1], at[j]));
                            let (u, t) = closest(a, own, at[j], other);
                            let (p, q) = (add(a, scale(own, u)), add(at[j], scale(other, t)));
                            let d = dist(p, q);
                            if d >= need - slack {
                                continue;
                            }
                            // Across each other: one over the other (along both), otherwise
                            // apart.
                            let over = cross(own, other);
                            let crossing = dot(over, over) > 0.2 * dot(own, own) * dot(other, other);
                            let (away, gap) = if crossing {
                                let n = norm(over);
                                let h = dot(sub(p, q), n);
                                // Neither over the other yet: the strand of lower index goes
                                // along the rest direction, or across it towards positive
                                // coordinates (both strands must agree on the way).
                                let up = if h.abs() > 1e-3 {
                                    h > 0.0
                                } else {
                                    let lean = dot(n, strands[s.min(owner[j])].rest);
                                    let along = if lean.abs() > 1e-3 { lean > 0.0 } else { n.into_iter().find(|v| v.abs() > 1e-3).is_some_and(|v| v > 0.0) };
                                    along == (s < owner[j])
                                };
                                (scale(n, if up { 1.0 } else { -1.0 }), h.abs())
                            } else if d > 1e-4 {
                                (scale(sub(p, q), 1.0 / d), d)
                            } else {
                                (aside(other, strands[s].rest, s < owner[j]), 0.0)
                            };
                            let share = if fixed[j] && fixed[j + 1] { 1.0 } else { 0.5 };
                            let push = scale(away, (need - gap).max(0.0) * share);
                            // A segment at rest is pushed back by this one here (it does
                            // not look for overlaps itself any more).
                            if !searched[j] {
                                let back = scale(away, -(need - gap).max(0.0) * if fixed[i] && fixed[i + 1] { 1.0 } else { 0.5 });
                                out.extend([(j, back, 1.0 - t), (j + 1, back, t)]);
                            }
                            out.extend([(i, push, 1.0 - u), (i + 1, push, u)]);
                        }
                    }
                }
            }
            out
        });
        for (k, push, w) in found.into_iter().flatten() {
            add_to(&mut pushes, &mut pushed, k, push, w);
        }
        pushed.sort_unstable();
        pushed.dedup();
        let moves: Vec<(usize, V3)> = pushed.iter().map(|&k| (k, pushes[k])).filter(|(k, (_, w))| !fixed[*k] && *w > 1e-3).map(|(k, (sum, w))| (k, scale(sum, 1.0 / w.max(0.5)))).collect();
        for k in pushed.drain(..) {
            pushes[k] = ([0.0; 3], 0.0);
        }
        if moves.is_empty() {
            break;
        }
        // Each move spread along its strand, fading out over three times its thickness
        // (so that it bends away); where moves meet, their mean.
        for (i, m) in moves {
            let (span, reach) = (spans[owner[i]].clone(), 3.0 * strands[owner[i]].diameter + 10.0);
            let near = |k: &usize| (arc[*k] - arc[i]).abs() < reach;
            for k in (span.start..i).rev().take_while(near).chain((i..span.end).take_while(near)).filter(|&k| !fixed[k]) {
                add_to(&mut spread, &mut moved, k, m, 1.0 - (arc[k] - arc[i]).abs() / reach);
            }
        }
        moved.sort_unstable();
        moved.dedup();
        let mut changed: Vec<usize> = Vec::new();
        for k in moved.drain(..) {
            let (sum, w) = std::mem::replace(&mut spread[k], ([0.0; 3], 0.0));
            let want = scale(sum, 1.0 / w.max(1.0));
            let to = keep(k, add(at[k], want));
            let to = if cuts(&at, k, to) { at[k] } else { to };
            if dist(to, at[k]) < 0.5 * dot(want, want).sqrt() {
                stuck[k] += 1;
                fixed[k] |= stuck[k] >= 6;
            }
            if dist(to, at[k]) > 0.01 {
                at[k] = to;
                changed.push(k);
            }
        }
        // The grid follows the segments that moved; their samples and those next to
        // them are checked again.
        let mut next: Vec<usize> = Vec::new();
        for &i in &changed {
            for j in [i.wrapping_sub(1), i] {
                if j == usize::MAX || !segment(j) {
                    continue;
                }
                let r = radius[owner[j]];
                let l = level(r);
                let now = reach(l, at[j], at[j + 1], r);
                if now != placed[j] {
                    for c in cells(placed[j]) {
                        if let Some(list) = grids[l].get_mut(&c) {
                            list.retain(|&x| x as usize != j);
                        }
                    }
                    for c in cells(now) {
                        grids[l].entry(c).or_default().push(j as u32);
                    }
                    placed[j] = now;
                }
            }
            next.extend([i.wrapping_sub(1), i, i + 1].into_iter().filter(|&m| m < at.len() && owner[m] == owner[i] && !fixed[m]));
        }
        next.sort_unstable();
        next.dedup();
        active = next;
    }
    Some(spans.into_iter().map(|span| straighten(&at[span], 0.05)).collect())
}

/// Boxes by the cells of a grid they reach into, to find those around a point.
struct BoxGrid {
    cells: HashMap<[i32; 3], Vec<u32>>,
}

/// Edge (mm) of the cells of a [`BoxGrid`].
const BOX_CELL: f32 = 100.0;

impl BoxGrid {
    fn new(boxes: &[(V3, V3)]) -> BoxGrid {
        let mut cells: HashMap<[i32; 3], Vec<u32>> = HashMap::new();
        for (i, (lo, hi)) in boxes.iter().enumerate() {
            for c in cells_of(*lo, *hi) {
                cells.entry(c).or_default().push(i as u32);
            }
        }
        BoxGrid { cells }
    }

    /// The boxes reaching into the cell of `p` (all that may contain it).
    fn at(&self, p: V3) -> &[u32] {
        self.cells.get(&p.map(|v| (v / BOX_CELL).floor() as i32)).map_or(&[], Vec::as_slice)
    }
}

/// The cells of a [`BoxGrid`] a box reaches into.
fn cells_of(lo: V3, hi: V3) -> impl Iterator<Item = [i32; 3]> {
    let key = |v: f32| (v / BOX_CELL).floor() as i32;
    cells((lo.map(key), hi.map(key)))
}

/// The points of the strands for [`separate`], every three quarters of their thickness
/// (at least 3 mm): where they are, their strand, whether they stay (near the ends, at
/// an end converging with others, or inside an obstacle), how far along their strand,
/// the samples of each strand.
struct Samples {
    at: Vec<V3>,
    owner: Vec<usize>,
    fixed: Vec<bool>,
    arc: Vec<f32>,
    spans: Vec<std::ops::Range<usize>>,
}

fn samples(strands: &[Strand], obstacles: &[(V3, V3)]) -> Samples {
    let mut out = Samples { at: Vec::new(), owner: Vec::new(), fixed: Vec::new(), arc: Vec::new(), spans: Vec::new() };
    // The ends that lie as close to an end of another strand as their thicknesses.
    let ends: Vec<(usize, V3)> = strands.iter().enumerate().flat_map(|(s, st)| st.points.first().into_iter().chain(st.points.last()).map(move |q| (s, *q))).collect();
    let crowded = |s: usize, q: V3| ends.iter().any(|&(t, e)| t != s && !strands[t].held && dist(q, e) < (strands[s].diameter + strands[t].diameter) / 2.0);
    let grid = BoxGrid::new(obstacles);
    for (s, st) in strands.iter().enumerate() {
        let start = out.at.len();
        let step = (st.diameter * 0.75).max(3.0);
        let ends = 2.0 * st.diameter + 10.0;
        let total: f32 = st.points.windows(2).map(|w| dist(w[0], w[1])).sum();
        let held = [st.points.first(), st.points.last()].map(|q| q.is_some_and(|q| crowded(s, *q)));
        let hold = [0, 1].map(|e| if held[e] { st.tails[e].max(ends) } else { ends });
        let mut run = 0.0;
        for (k, &q) in st.points.iter().enumerate() {
            let a = if k == 0 { q } else { st.points[k - 1] };
            let len = dist(a, q);
            let n = if k == 0 { 1 } else { (len / step).ceil().max(1.0) as usize };
            for i in 1..=n {
                let t = i as f32 / n as f32;
                let p = add(a, scale(sub(q, a), t));
                let along = run + len * t;
                let inside = grid.at(p).iter().any(|&b| within(p, obstacles[b as usize]));
                out.at.push(p);
                out.owner.push(s);
                out.arc.push(along);
                out.fixed.push(st.held || along < hold[0] || total - along < hold[1] || inside);
            }
            run += len;
        }
        out.spans.push(start..out.at.len());
    }
    out
}

/// Where the segments from `a` along `da` and from `b` along `db` come nearest: the
/// share of each.
fn closest(a: V3, da: V3, b: V3, db: V3) -> (f32, f32) {
    let r = sub(a, b);
    let (la, lb, f) = (dot(da, da), dot(db, db), dot(db, r));
    if la <= 1e-9 || lb <= 1e-9 {
        return match (la <= 1e-9, lb <= 1e-9) {
            (true, true) => (0.0, 0.0),
            (true, false) => (0.0, (f / lb).clamp(0.0, 1.0)),
            _ => ((-dot(da, r) / la).clamp(0.0, 1.0), 0.0),
        };
    }
    let (c, k) = (dot(da, r), dot(da, db));
    let denom = la * lb - k * k;
    let s = if denom > 1e-9 { ((k * f - c * lb) / denom).clamp(0.0, 1.0) } else { 0.0 };
    let t = (k * s + f) / lb;
    if t < 0.0 {
        ((-c / la).clamp(0.0, 1.0), 0.0)
    } else if t > 1.0 {
        (((k - c) / la).clamp(0.0, 1.0), 1.0)
    } else {
        (s, t)
    }
}

/// The cells of a grid from `lo` to `hi`.
fn cells((lo, hi): ([i32; 3], [i32; 3])) -> impl Iterator<Item = [i32; 3]> {
    (lo[0]..=hi[0]).flat_map(move |x| (lo[1]..=hi[1]).flat_map(move |y| (lo[2]..=hi[2]).map(move |z| [x, y, z])))
}

/// Inside the box `b`.
/// The segment `a`-`b` meets the box `lo`-`hi` (its ends included).
fn segment_meets_box(a: V3, b: V3, lo: V3, hi: V3) -> bool {
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        let d = b[k] - a[k];
        if d.abs() < 1e-9 {
            if a[k] < lo[k] || a[k] > hi[k] {
                return false;
            }
            continue;
        }
        let (ta, tb) = ((lo[k] - a[k]) / d, (hi[k] - a[k]) / d);
        t0 = t0.max(ta.min(tb));
        t1 = t1.min(ta.max(tb));
        if t0 > t1 {
            return false;
        }
    }
    true
}

fn within(p: V3, b: (V3, V3)) -> bool {
    (0..3).all(|k| p[k] >= b.0[k] && p[k] <= b.1[k])
}

/// Across the segment `along` lying on another one: along the surface the strand runs
/// on (or across the axis it runs least along), to one side or the other.
fn aside(along: V3, rest: V3, first: bool) -> V3 {
    let least = (0..3).min_by(|&a, &b| along[a].abs().total_cmp(&along[b].abs())).unwrap_or(0);
    let mut d = cross(along, if rest != [0.0; 3] { rest } else { unit(least) });
    if dot(d, d) < 1e-8 {
        d = cross(along, unit((least + 1) % 3));
    }
    scale(norm(d), if first { 1.0 } else { -1.0 })
}

/// The points without those that lie within `tolerance` of the line through their
/// neighbours kept (Douglas–Peucker).
fn straighten(points: &[V3], tolerance: f32) -> Vec<V3> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    let (first, last) = (0, points.len() - 1);
    keep[first] = true;
    keep[last] = true;
    let mut stack = vec![(first, last)];
    while let Some((a, b)) = stack.pop() {
        let ab = sub(points[b], points[a]);
        let l2 = dot(ab, ab);
        let off = |q: V3| {
            let t = if l2 > 0.0 { (dot(sub(q, points[a]), ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
            dist(q, add(points[a], scale(ab, t)))
        };
        if let Some((m, d)) = (a + 1..b).map(|m| (m, off(points[m]))).max_by(|x, y| x.1.total_cmp(&y.1)) {
            if d > tolerance {
                keep[m] = true;
                stack.extend([(a, m), (m, b)]);
            }
        }
    }
    points.iter().zip(keep).filter(|(_, k)| *k).map(|(p, _)| *p).collect()
}

#[cfg(test)]
mod tests;
