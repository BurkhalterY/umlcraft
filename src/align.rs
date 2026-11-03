//! Format > Alignment: align, space and size the selected boxes equally, and change the drawing order.
//!
//! The rules follow StarUML's alignment extension (extensions/default/alignment/main.js). Only node views
//! take part in alignment and sizing; lines and labels are ignored. Every command is one undo step.

use crate::app::App;
use crate::model::{num, Eid};
use crate::views::{is_node, node_rect, R};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Align {
    Left,
    Right,
    Center,
    Top,
    Bottom,
    Middle,
    SpaceH,
    SpaceV,
    Width,
    Height,
    Size,
}

impl Align {
    /// Menu order of Format > Alignment after the z-order items (alignment.json).
    pub const ALL: [(Align, &'static str); 11] = [
        (Align::Left, "Align Left"),
        (Align::Right, "Align Right"),
        (Align::Center, "Align Center"),
        (Align::Top, "Align Top"),
        (Align::Bottom, "Align Bottom"),
        (Align::Middle, "Align Middle"),
        (Align::SpaceH, "Space Equally, Horizontally"),
        (Align::SpaceV, "Space Equally, Vertically"),
        (Align::Width, "Set Width Equally"),
        (Align::Height, "Set Height Equally"),
        (Align::Size, "Set Size Equally"),
    ];
}

/// New left/top/width/height for each box, or nothing when the command does not apply.
pub fn arrange(how: Align, boxes: &[R]) -> Vec<R> {
    let mut out = boxes.to_vec();
    if boxes.is_empty() {
        return out;
    }
    let min_x = boxes.iter().map(|b| b.x).fold(f64::INFINITY, f64::min);
    let min_y = boxes.iter().map(|b| b.y).fold(f64::INFINITY, f64::min);
    let max_x = boxes.iter().map(|b| b.x + b.w).fold(f64::NEG_INFINITY, f64::max);
    let max_y = boxes.iter().map(|b| b.y + b.h).fold(f64::NEG_INFINITY, f64::max);
    let max_w = boxes.iter().map(|b| b.w).fold(0.0, f64::max);
    let max_h = boxes.iter().map(|b| b.h).fold(0.0, f64::max);
    match how {
        Align::Left => out.iter_mut().for_each(|b| b.x = min_x),
        Align::Top => out.iter_mut().for_each(|b| b.y = min_y),
        // the original measures right = left + width - 1 and sets left = max - width + 1: the same right edge
        Align::Right => out.iter_mut().for_each(|b| b.x = max_x - b.w),
        Align::Bottom => out.iter_mut().for_each(|b| b.y = max_y - b.h),
        Align::Center => {
            let c = (min_x + max_x) / 2.0;
            out.iter_mut().for_each(|b| b.x = (c - b.w / 2.0).round());
        }
        Align::Middle => {
            let c = (min_y + max_y) / 2.0;
            out.iter_mut().for_each(|b| b.y = (c - b.h / 2.0).round());
        }
        Align::SpaceH | Align::SpaceV => {
            let horiz = how == Align::SpaceH;
            let (lo, hi) = if horiz { (min_x, max_x) } else { (min_y, max_y) };
            let total: f64 = boxes.iter().map(|b| if horiz { b.w } else { b.h }).sum();
            if boxes.len() < 2 || total >= hi - lo {
                return out;
            }
            let gap = (hi - lo - total) / (boxes.len() - 1) as f64;
            let mut order: Vec<usize> = (0..boxes.len()).collect();
            order.sort_by(|&a, &b| {
                let (ka, kb) = if horiz { (boxes[a].x, boxes[b].x) } else { (boxes[a].y, boxes[b].y) };
                ka.total_cmp(&kb)
            });
            let mut at = lo;
            for i in order {
                if horiz {
                    out[i].x = at.round();
                    at += boxes[i].w + gap;
                } else {
                    out[i].y = at.round();
                    at += boxes[i].h + gap;
                }
            }
        }
        Align::Width => out.iter_mut().for_each(|b| b.w = max_w),
        Align::Height => out.iter_mut().for_each(|b| b.h = max_h),
        Align::Size => out.iter_mut().for_each(|b| {
            b.w = max_w;
            b.h = max_h;
        }),
    }
    out
}

impl App {
    /// The selected boxes that alignment acts on: node views, no lines, no labels.
    fn aligned_nodes(&self) -> Vec<Eid> {
        self.selection
            .iter()
            .copied()
            .filter(|v| self.repo.alive(*v) && is_node(&self.repo, *v) && !self.repo.ty(*v).ends_with("LabelView"))
            .collect()
    }

    pub fn can_align(&self) -> bool {
        !self.selection.is_empty()
    }

    pub fn align(&mut self, how: Align) {
        let nodes = self.aligned_nodes();
        if nodes.is_empty() {
            return;
        }
        let before: Vec<R> = nodes.iter().map(|v| node_rect(&self.repo, *v)).collect();
        let after = arrange(how, &before);
        if after == before {
            return;
        }
        self.change(|a| {
            for ((v, b), n) in nodes.iter().zip(&before).zip(&after) {
                if n.x != b.x {
                    a.repo.set(*v, "left", num(n.x));
                }
                if n.y != b.y {
                    a.repo.set(*v, "top", num(n.y));
                }
                if n.w != b.w {
                    a.repo.set_raw(*v, "width", num(n.w));
                }
                if n.h != b.h {
                    a.repo.set_raw(*v, "height", num(n.h));
                }
            }
        });
    }

    /// Bring to Front (`front`) or Send to Back: move the selected top-level views to the end or the start
    /// of the diagram's ownedViews, which is the drawing and hit-testing order. Their own order is kept.
    pub fn reorder(&mut self, front: bool) {
        let Some(d) = self.diagram else { return };
        let list = self.repo.list(d, "ownedViews");
        let (picked, rest): (Vec<Eid>, Vec<Eid>) = list.iter().partition(|v| self.selection.contains(v));
        if picked.is_empty() {
            return;
        }
        let new: Vec<Eid> = if front { rest.into_iter().chain(picked).collect() } else { picked.into_iter().chain(rest).collect() };
        if new == list {
            return;
        }
        self.change(|a| a.repo.set_list(d, "ownedViews", new));
    }
}
