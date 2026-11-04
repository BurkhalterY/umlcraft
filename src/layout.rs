//! Format > Layout: lay a diagram out in ranks, as StarUML's diagram-layout extension does with dagre
//! (extensions/default/diagram-layout/main.js, src/engine/engine.js:1513-1549).
//!
//! UMLCraft has its own layered layout in the same spirit: cycles are broken, boxes get ranks along the
//! lines (the head of a line above its tail, so super-classes sit above sub-classes), long lines pass
//! through invisible points on the ranks they cross, the order within each rank is improved by
//! barycentre sweeps, and each box is then pulled towards its neighbours without overlapping.
//! The separations are estimated.

use crate::app::App;
use crate::model::{num, Eid};
use crate::views::{self, P, R};
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Dir {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl Dir {
    /// Format > Layout, in menu order; Auto lays out top to bottom.
    pub const MENU: [(Dir, &'static str); 5] = [
        (Dir::TopBottom, "Auto"),
        (Dir::TopBottom, "Top to Bottom"),
        (Dir::BottomTop, "Bottom to Top"),
        (Dir::LeftRight, "Left to Right"),
        (Dir::RightLeft, "Right to Left"),
    ];
}

/// Space between two boxes of a rank, between a line and anything else in a rank, and between ranks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seps {
    pub node: f64,
    pub edge: f64,
    pub rank: f64,
}

/// The separations of Format › Layout (estimated).
pub const SEPS: Seps = Seps { node: 50.0, edge: 10.0, rank: 50.0 };

/// The result: each box's top-left corner, and for each line the bend points it passes through
/// (from its first end to its second), all relative to (0, 0).
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub boxes: Vec<P>,
    pub bends: Vec<Vec<P>>,
}

/// Lay out boxes of these sizes; each edge (a, b) puts a above b (before b in the direction).
pub fn layered(sizes: &[(f64, f64)], edges: &[(usize, usize)], dir: Dir, seps: Seps) -> Placed {
    let n = sizes.len();
    let across = matches!(dir, Dir::LeftRight | Dir::RightLeft);
    // work top to bottom; for left-right layouts the boxes are turned
    let sz: Vec<(f64, f64)> = sizes.iter().map(|&(w, h)| if across { (h, w) } else { (w, h) }).collect();

    // 1. break cycles: a line back to a box still being visited is turned around
    let mut out: Vec<Vec<usize>> = vec![vec![]; n];
    for (i, &(a, b)) in edges.iter().enumerate() {
        if a != b {
            out[a].push(i);
        }
    }
    let mut flipped = vec![false; edges.len()];
    let mut state = vec![0u8; n]; // 0 new, 1 on the stack, 2 done
    for s in 0..n {
        if state[s] != 0 {
            continue;
        }
        let mut stack = vec![(s, 0usize)];
        state[s] = 1;
        while let Some(&mut (v, ref mut k)) = stack.last_mut() {
            if *k < out[v].len() {
                let ei = out[v][*k];
                *k += 1;
                let w = edges[ei].1;
                match state[w] {
                    0 => {
                        state[w] = 1;
                        stack.push((w, 0));
                    }
                    1 => flipped[ei] = true,
                    _ => {}
                }
            } else {
                state[v] = 2;
                stack.pop();
            }
        }
    }
    let dag: Vec<(usize, usize)> = edges
        .iter()
        .enumerate()
        .filter(|(_, e)| e.0 != e.1)
        .map(|(i, &(a, b))| if flipped[i] { (b, a) } else { (a, b) })
        .collect();

    // 2. ranks: longest path from the top, then boxes with only lines below are pulled down to them
    let mut indeg = vec![0usize; n];
    let mut succ: Vec<Vec<usize>> = vec![vec![]; n];
    let mut pred: Vec<Vec<usize>> = vec![vec![]; n];
    for &(a, b) in &dag {
        succ[a].push(b);
        pred[b].push(a);
        indeg[b] += 1;
    }
    let mut topo = vec![];
    let mut queue: Vec<usize> = (0..n).filter(|v| indeg[*v] == 0).collect();
    queue.reverse();
    while let Some(v) = queue.pop() {
        topo.push(v);
        for &w in succ[v].iter().rev() {
            indeg[w] -= 1;
            if indeg[w] == 0 {
                queue.push(w);
            }
        }
    }
    let mut rank = vec![0usize; n];
    for &v in &topo {
        for &w in &succ[v] {
            rank[w] = rank[w].max(rank[v] + 1);
        }
    }
    for &v in topo.iter().rev() {
        if pred[v].is_empty() {
            if let Some(m) = succ[v].iter().map(|w| rank[*w]).min() {
                rank[v] = rank[v].max(m.saturating_sub(1));
            }
        }
    }

    // 3. invisible points where a line crosses a rank
    let mut node_sz = sz.clone();
    let mut is_dummy = vec![false; n];
    let mut node_rank = rank.clone();
    let mut chains: Vec<Vec<usize>> = vec![]; // per dag edge: the dummies, top to bottom
    let mut links: Vec<(usize, usize)> = vec![];
    for &(a, b) in &dag {
        let mut chain = vec![];
        let mut prev = a;
        for r in rank[a] + 1..rank[b] {
            let d = node_sz.len();
            node_sz.push((seps.edge, 0.0));
            is_dummy.push(true);
            node_rank.push(r);
            links.push((prev, d));
            chain.push(d);
            prev = d;
        }
        links.push((prev, b));
        chains.push(chain);
    }
    let total = node_sz.len();
    let mut up: Vec<Vec<usize>> = vec![vec![]; total];
    let mut down: Vec<Vec<usize>> = vec![vec![]; total];
    for &(a, b) in &links {
        down[a].push(b);
        up[b].push(a);
    }

    // 4. order within the ranks: start in breadth-first order, then barycentre sweeps
    let ranks = node_rank.iter().max().map(|m| m + 1).unwrap_or(0);
    let mut layers: Vec<Vec<usize>> = vec![vec![]; ranks];
    let mut seen = vec![false; total];
    let mut starts: Vec<usize> = (0..total).filter(|v| up[*v].is_empty()).collect();
    starts.sort_by_key(|v| (node_rank[*v], *v));
    for s in starts {
        let mut q = std::collections::VecDeque::from([s]);
        seen[s] = true;
        while let Some(v) = q.pop_front() {
            layers[node_rank[v]].push(v);
            for &w in &down[v] {
                if !seen[w] {
                    seen[w] = true;
                    q.push_back(w);
                }
            }
        }
    }
    for v in 0..total {
        if !seen[v] {
            layers[node_rank[v]].push(v);
        }
    }
    let crossings = |layers: &Vec<Vec<usize>>| -> usize {
        let mut pos = vec![0usize; total];
        for l in layers {
            for (i, v) in l.iter().enumerate() {
                pos[*v] = i;
            }
        }
        let mut c = 0;
        for l in layers {
            let es: Vec<(usize, usize)> = l.iter().flat_map(|v| down[*v].iter().map(|w| (pos[*v], pos[*w]))).collect();
            for i in 0..es.len() {
                for j in i + 1..es.len() {
                    if (es[i].0 < es[j].0 && es[i].1 > es[j].1) || (es[i].0 > es[j].0 && es[i].1 < es[j].1) {
                        c += 1;
                    }
                }
            }
        }
        c
    };
    let mut best = layers.clone();
    let mut best_c = crossings(&layers);
    for it in 0..12 {
        let mut pos = vec![0.0; total];
        for l in &layers {
            for (i, v) in l.iter().enumerate() {
                pos[*v] = i as f64;
            }
        }
        let downward = it % 2 == 0;
        let order: Vec<usize> = if downward { (1..ranks).collect() } else { (0..ranks.saturating_sub(1)).rev().collect() };
        for r in order {
            let l = &layers[r];
            let mut keyed: Vec<(f64, usize)> = l
                .iter()
                .map(|v| {
                    let nb = if downward { &up[*v] } else { &down[*v] };
                    let k = if nb.is_empty() { pos[*v] } else { nb.iter().map(|w| pos[*w]).sum::<f64>() / nb.len() as f64 };
                    (k, *v)
                })
                .collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0));
            layers[r] = keyed.into_iter().map(|(_, v)| v).collect();
            for (i, v) in layers[r].iter().enumerate() {
                pos[*v] = i as f64;
            }
        }
        let c = crossings(&layers);
        if c < best_c {
            best_c = c;
            best = layers.clone();
        }
    }
    let layers = best;

    // 5. positions across the ranks: packed, then pulled towards the neighbours without overlapping
    let gap = |a: usize, b: usize| -> f64 {
        let sep = if is_dummy[a] || is_dummy[b] { seps.edge } else { seps.node };
        node_sz[a].0 / 2.0 + sep + node_sz[b].0 / 2.0
    };
    let mut x = vec![0.0; total];
    for l in &layers {
        let mut cx = 0.0;
        for (i, v) in l.iter().enumerate() {
            if i > 0 {
                cx += gap(l[i - 1], *v);
            }
            x[*v] = cx;
        }
    }
    for it in 0..16 {
        let order: Vec<usize> = if it % 2 == 0 { (0..ranks).collect() } else { (0..ranks).rev().collect() };
        for r in order {
            let l = &layers[r];
            if l.is_empty() {
                continue;
            }
            // a box without lines follows the one before it, so loose boxes stay packed
            let want: Vec<f64> = l
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let nb: Vec<usize> = up[*v].iter().chain(down[*v].iter()).copied().collect();
                    if !nb.is_empty() {
                        nb.iter().map(|w| x[*w]).sum::<f64>() / nb.len() as f64
                    } else if i > 0 {
                        x[l[i - 1]] + gap(l[i - 1], *v)
                    } else {
                        x[*v]
                    }
                })
                .collect();
            // keep the order and the gaps: least squares under order constraints (pool adjacent violators)
            let mut off = vec![0.0; l.len()];
            for i in 1..l.len() {
                off[i] = off[i - 1] + gap(l[i - 1], l[i]);
            }
            let mut blocks: Vec<(f64, usize)> = vec![]; // (mean, count)
            for (i, w) in want.iter().enumerate() {
                blocks.push((w - off[i], 1));
                while blocks.len() > 1 && blocks[blocks.len() - 2].0 > blocks[blocks.len() - 1].0 {
                    let (m2, c2) = blocks.pop().unwrap();
                    let (m1, c1) = blocks.pop().unwrap();
                    blocks.push(((m1 * c1 as f64 + m2 * c2 as f64) / (c1 + c2) as f64, c1 + c2));
                }
            }
            let mut i = 0;
            for (m, c) in blocks {
                for _ in 0..c {
                    x[l[i]] = m + off[i];
                    i += 1;
                }
            }
        }
    }
    let min_x = (0..total).map(|v| x[v] - node_sz[v].0 / 2.0).fold(f64::INFINITY, f64::min);
    let min_x = if min_x.is_finite() { min_x } else { 0.0 };

    // 6. ranks one under the other, each box centred on its rank
    let heights: Vec<f64> = layers.iter().map(|l| l.iter().map(|v| node_sz[*v].1).fold(0.0, f64::max)).collect();
    let mut rank_y = vec![0.0; ranks];
    for r in 1..ranks {
        rank_y[r] = rank_y[r - 1] + heights[r - 1] + seps.rank;
    }
    let depth = if ranks == 0 { 0.0 } else { rank_y[ranks - 1] + heights[ranks - 1] };
    let width = (0..total).map(|v| x[v] + node_sz[v].0 / 2.0 - min_x).fold(0.0, f64::max);
    // centre of node v, top-to-bottom coordinates
    let centre = |v: usize| P { x: x[v] - min_x, y: rank_y[node_rank[v]] + heights[node_rank[v]] / 2.0 };
    let turn = |p: P| -> P {
        match dir {
            Dir::TopBottom => p,
            Dir::BottomTop => P { x: p.x, y: depth - p.y },
            Dir::LeftRight => P { x: p.y, y: p.x },
            Dir::RightLeft => P { x: depth - p.y, y: p.x },
        }
    };
    let _ = width;
    let round = |p: P| P { x: p.x.round(), y: p.y.round() };
    let boxes: Vec<P> = (0..n)
        .map(|v| {
            let c = turn(centre(v));
            let (w, h) = sizes[v];
            round(P { x: c.x - w / 2.0, y: c.y - h / 2.0 })
        })
        .collect();
    // bend points of each original line, from its first end to its second
    let mut bends = vec![vec![]; edges.len()];
    let mut k = 0;
    for (i, e) in edges.iter().enumerate() {
        if e.0 == e.1 {
            continue;
        }
        let mut pts: Vec<P> = chains[k].iter().map(|d| round(turn(centre(*d)))).collect();
        if flipped[i] {
            pts.reverse();
        }
        bends[i] = pts;
        k += 1;
    }
    Placed { boxes, bends }
}

impl App {
    /// Format > Layout: lay out the current diagram's top-level boxes in one undo step. Boxes inside a
    /// package or subject, contained or only lying within its bounds, move with it; lines get the bend
    /// points of the layout (none when rectilinear, which routes itself).
    pub fn layout_diagram(&mut self, dir: Dir) {
        self.layout_diagram_with(dir, SEPS);
    }

    /// Lay out the current diagram with these separations.
    pub fn layout_diagram_with(&mut self, dir: Dir, seps: Seps) {
        let Some(d) = self.diagram else { return };
        let all = self.repo.list(d, "ownedViews");
        let free: Vec<Eid> = all.iter().copied().filter(|v| views::is_node(&self.repo, *v) && !self.repo.has(*v, "containerView") && self.repo.refd(*v, "hostEdge").is_none()).collect();
        // a box lying inside a package or subject moves with it, even when it is not contained in it
        let holds = |a: &App, c: Eid, v: Eid| -> bool {
            let ct = a.repo.ty(c);
            let (o, i) = (views::node_rect(&a.repo, c), views::node_rect(&a.repo, v));
            c != v
                && matches!(ct, "UMLPackageView" | "UMLModelView" | "UMLSubsystemView" | "UMLUseCaseSubjectView")
                && i.x >= o.x && i.y >= o.y && i.x + i.w <= o.x + o.w && i.y + i.h <= o.y + o.h
                && o.w * o.h > i.w * i.h
        };
        let mut lying_in: HashMap<Eid, Eid> = HashMap::new();
        for &v in &free {
            // the smallest box around it
            if let Some(c) = free.iter().copied().filter(|c| holds(self, *c, v)).min_by(|p, q| {
                let (p, q) = (views::node_rect(&self.repo, *p), views::node_rect(&self.repo, *q));
                (p.w * p.h).total_cmp(&(q.w * q.h))
            }) {
                lying_in.insert(v, c);
            }
        }
        let top: Vec<Eid> = free.iter().copied().filter(|v| !lying_in.contains_key(v)).collect();
        if top.is_empty() {
            return;
        }
        let index: HashMap<Eid, usize> = top.iter().enumerate().map(|(i, v)| (*v, i)).collect();
        // a line's end counts as the top-level box that holds it
        let owner = |a: &App, mut v: Eid| -> Option<usize> {
            loop {
                if let Some(i) = index.get(&v) {
                    return Some(*i);
                }
                v = match a.repo.refd(v, "containerView") {
                    Some(c) => c,
                    None => *lying_in.get(&v)?,
                };
            }
        };
        let lines: Vec<(Eid, usize, usize)> = all
            .iter()
            .copied()
            .filter(|e| views::is_edge(&self.repo, *e))
            .filter_map(|e| {
                let (t, h) = (self.repo.refd(e, "tail")?, self.repo.refd(e, "head")?);
                Some((e, owner(self, t)?, owner(self, h)?))
            })
            .collect();
        let sizes: Vec<(f64, f64)> = top.iter().map(|v| (self.repo.f(*v, "width"), self.repo.f(*v, "height"))).collect();
        // the head of a line goes above its tail
        let edges: Vec<(usize, usize)> = lines.iter().map(|(_, t, h)| (*h, *t)).collect();
        let placed = layered(&sizes, &edges, dir, seps);
        // keep the diagram where it was: the layout starts at the old top-left corner
        let rects: Vec<R> = top.iter().map(|v| views::node_rect(&self.repo, *v)).collect();
        let ox = rects.iter().map(|b| b.x).fold(f64::INFINITY, f64::min);
        let oy = rects.iter().map(|b| b.y).fold(f64::INFINITY, f64::min);
        self.change(|a| {
            for (i, v) in top.iter().enumerate() {
                let (dx, dy) = (placed.boxes[i].x + ox - rects[i].x, placed.boxes[i].y + oy - rects[i].y);
                a.shift_view(*v, dx, dy);
                // and the boxes lying in it, at any depth
                let mut inner: Vec<Eid> = lying_in.iter().filter(|(_, c)| **c == *v).map(|(x, _)| *x).collect();
                while let Some(x) = inner.pop() {
                    a.shift_view(x, dx, dy);
                    inner.extend(lying_in.iter().filter(|(_, c)| **c == x).map(|(y, _)| *y));
                }
            }
            for (k, (e, t, h)) in lines.iter().enumerate() {
                let style = a.repo.val(*e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
                let tail = views::node_rect(&a.repo, a.repo.refd(*e, "tail").unwrap());
                let head = views::node_rect(&a.repo, a.repo.refd(*e, "head").unwrap());
                let c = |b: R| P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
                // bend points run from the head (above) to the tail; the line runs tail to head
                let mut bends: Vec<P> = placed.bends[k].iter().map(|p| P { x: p.x + ox, y: p.y + oy }).collect();
                bends.reverse();
                if t == h || style == 0 || bends.is_empty() {
                    bends.clear();
                }
                let mut pts = vec![c(tail)];
                pts.extend(bends);
                pts.push(c(head));
                a.repo.set(*e, "points", json!(views::format_points(&pts)));
            }
        });
    }

    /// Move a box, the boxes it contains and its labels by (dx, dy).
    pub(crate) fn shift_view(&mut self, v: Eid, dx: f64, dy: f64) {
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        let x = self.repo.f(v, "left");
        let y = self.repo.f(v, "top");
        self.repo.set(v, "left", num(x + dx));
        self.repo.set(v, "top", num(y + dy));
        let inner: Vec<Eid> = match self.repo.raw(v, "containedViews") {
            Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| self.repo.by_ref(x)).collect(),
            _ => vec![],
        };
        for c in inner {
            self.shift_view(c, dx, dy);
        }
    }
}
