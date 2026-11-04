//! Diagram canvas: drawing on screen, selection, moving, resizing, creating and inline editing.

use crate::app::{App, Drag, EditKind, Editing};
use crate::model::Eid;
use crate::render::{self, Canvas, Rgba};
use crate::views::{self, is_edge, is_node, node_rect, points_of, Measure, P, R};
use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Sense, Shape, Stroke, Vec2};
use serde_json::json;

pub const MINI_W: f32 = 130.0;
pub const MINI_H: f32 = 100.0;
pub const MINI_MARGIN: f32 = 15.0;
pub const AREA_COLOR: Color32 = Color32::from_rgb(0xcc, 0xcc, 0xcc);

pub fn c32(c: Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3)
}

pub fn diagram_family(style: u8) -> FontFamily {
    FontFamily::Name(match style & 3 {
        1 => "arial-b".into(),
        2 => "arial-i".into(),
        3 => "arial-bi".into(),
        _ => "arial".into(),
    })
}

/// Measures diagram text with the Arial-compatible fonts at 13px.
pub struct EguiMeasure {
    pub ctx: egui::Context,
}
impl Measure for EguiMeasure {
    fn width(&self, text: &str, style: u8) -> f64 {
        if text.is_empty() {
            return 0.0;
        }
        self.ctx.fonts_mut(|f| f.layout_no_wrap(text.to_string(), FontId::new(13.0, diagram_family(style)), Color32::BLACK).size().x as f64)
    }
}

pub struct ScreenCanvas<'a> {
    pub painter: &'a Painter,
    pub origin: Pos2,
    pub scroll: P,
    pub zoom: f64,
    pub measure: EguiMeasure,
}

impl ScreenCanvas<'_> {
    pub fn pos(&self, p: P) -> Pos2 {
        Pos2::new(self.origin.x + ((p.x - self.scroll.x) * self.zoom) as f32, self.origin.y + ((p.y - self.scroll.y) * self.zoom) as f32)
    }
    fn stroke(&self, c: Rgba) -> Stroke {
        Stroke::new((self.zoom as f32).clamp(0.6, 2.0), c32(c))
    }
}

impl Measure for ScreenCanvas<'_> {
    fn width(&self, text: &str, style: u8) -> f64 {
        self.measure.width(text, style)
    }
}

impl Canvas for ScreenCanvas<'_> {
    fn polyline(&mut self, pts: &[P], color: Rgba, dashed: bool) {
        let ps: Vec<Pos2> = pts.iter().map(|p| self.pos(*p)).collect();
        let st = self.stroke(color);
        if dashed {
            let d = (render::DASH * self.zoom) as f32;
            for s in Shape::dashed_line(&ps, st, d, d) {
                self.painter.add(s);
            }
        } else {
            self.painter.add(Shape::line(ps, st));
        }
    }
    fn polygon(&mut self, pts: &[P], fill: Option<Rgba>, stroke: Option<Rgba>) {
        let ps: Vec<Pos2> = pts.iter().map(|p| self.pos(*p)).collect();
        let st = stroke.map(|c| self.stroke(c)).unwrap_or(Stroke::NONE);
        self.painter.add(Shape::convex_polygon(ps, fill.map(c32).unwrap_or(Color32::TRANSPARENT), st));
    }
    fn ellipse(&mut self, c: P, rx: f64, ry: f64, fill: Option<Rgba>, stroke: Option<Rgba>) {
        let ctr = self.pos(c);
        let r = Vec2::new((rx * self.zoom) as f32, (ry * self.zoom) as f32);
        if let Some(f) = fill {
            self.painter.add(Shape::ellipse_filled(ctr, r, c32(f)));
        }
        if let Some(s) = stroke {
            let st = self.stroke(s);
            self.painter.add(Shape::ellipse_stroke(ctr, r, st));
        }
    }
    fn text(&mut self, t: &str, b: R, align: i64, style: u8, size: f64, color: Rgba, underline: bool) {
        let font = FontId::new((size * self.zoom) as f32, diagram_family(style));
        let (x, a) = match align {
            0 => (b.x, Align2::LEFT_CENTER),
            1 => (b.x + b.w, Align2::RIGHT_CENTER),
            _ => (b.x + b.w / 2.0, Align2::CENTER_CENTER),
        };
        let pos = self.pos(P { x, y: b.y + b.h / 2.0 });
        let rect = self.painter.text(pos, a, t, font, c32(color));
        if underline {
            let y = rect.bottom() - 1.0;
            self.painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], Stroke::new(1.0, c32(color)));
        }
    }
}

// ------------------------------------------------------------------ hit testing
fn rect_of(a: P, b: P) -> R {
    R { x: a.x.min(b.x), y: a.y.min(b.y), w: (a.x - b.x).abs(), h: (a.y - b.y).abs() }
}
fn dist_to_seg(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    ((p.x - (a.x + t * dx)).powi(2) + (p.y - (a.y + t * dy)).powi(2)).sqrt()
}
fn in_rect(p: P, r: &R) -> bool {
    p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h
}
fn overlaps(a: &R, b: &R) -> bool {
    a.x <= b.x + b.w && b.x <= a.x + a.w && a.y <= b.y + b.h && b.y <= a.y + a.h
}
fn contains(outer: &R, inner: &R) -> bool {
    inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.w <= outer.x + outer.w && inner.y + inner.h <= outer.y + outer.h
}

/// A short button text for an "add related element" action: "Add Sub-Class" → "+ Sub-Class".
fn short_label(l: &str) -> String {
    format!("+ {}", l.trim_start_matches("Add "))
}

/// Keys of the inline edit on a class name or a compartment item.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ItemKey {
    /// Ctrl+Enter: a new attribute (literal on an enumeration), or a sibling of the edited item
    Add,
    /// Ctrl+Shift+Enter: a new operation
    AddOperation,
    /// Ctrl+Delete: delete the edited item
    Delete,
    /// Ctrl+Up / Ctrl+Down: move the edited item
    Move(bool),
    /// Up / Down: edit the previous or next item
    Nav(bool),
    /// The class name edit's Add Template Parameter button
    AddTemplateParameter,
}

impl App {
    fn hit_edge(&self, e: Eid, p: P) -> bool {
        let pts = points_of(&self.repo, e);
        pts.windows(2).any(|w| dist_to_seg(p, w[0], w[1]) <= 5.0 / self.zoom)
    }

    /// The visible label of a line under `p`, topmost first: (label, line).
    fn hit_edge_label(&self, p: P) -> Option<(Eid, Eid)> {
        let d = self.diagram?;
        for e in self.repo.list(d, "ownedViews").into_iter().rev() {
            if !is_edge(&self.repo, e) || matches!(self.repo.raw(e, "visible"), Some(serde_json::Value::Bool(false))) {
                continue;
            }
            for l in self.repo.list(e, "subViews") {
                if matches!(self.repo.raw(l, "visible"), Some(serde_json::Value::Bool(false)) | Some(serde_json::Value::Null)) || self.repo.str(l, "text").is_empty() {
                    continue;
                }
                if in_rect(p, &node_rect(&self.repo, l)) {
                    return Some((l, e));
                }
            }
        }
        None
    }

    pub fn hit_view(&self, p: P, nodes_only: bool) -> Option<Eid> {
        let d = self.diagram?;
        let views = self.repo.list(d, "ownedViews");
        for v in views.into_iter().rev() {
            if !self.repo.alive(v) || matches!(self.repo.raw(v, "visible"), Some(serde_json::Value::Bool(false))) {
                continue;
            }
            if is_edge(&self.repo, v) {
                if !nodes_only && self.hit_edge(v, p) {
                    return Some(v);
                }
            } else if is_node(&self.repo, v) && (in_rect(p, &node_rect(&self.repo, v)) || self.in_template_box(v, p)) {
                return Some(v);
            }
        }
        None
    }

    /// Cursor over the single selected line: a hand on its ends, move on a bend point, and on a segment
    /// ns/ew-resize for rectilinear lines or a crosshair for the others (diagram-editor.js:344-411).
    fn edge_cursor(&self, p: P) -> Option<egui::CursorIcon> {
        if self.selection.len() != 1 || !is_edge(&self.repo, self.selection[0]) {
            return None;
        }
        let e = self.selection[0];
        let pts = points_of(&self.repo, e);
        let n = pts.len();
        let near = |q: P| ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt() <= 6.0 / self.zoom;
        if n >= 2 && (near(pts[0]) || near(pts[n - 1])) {
            return Some(egui::CursorIcon::PointingHand);
        }
        if (1..n.saturating_sub(1)).any(|i| near(pts[i])) {
            return Some(egui::CursorIcon::Move);
        }
        let style = self.repo.val(e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
        let seg = (0..n.saturating_sub(1)).find(|&i| dist_to_seg(p, pts[i], pts[i + 1]) <= 4.0 / self.zoom)?;
        Some(if !views::rectilinear(style) {
            egui::CursorIcon::Crosshair
        } else if (pts[seg].y - pts[seg + 1].y).abs() < 0.5 {
            egui::CursorIcon::ResizeVertical
        } else {
            egui::CursorIcon::ResizeHorizontal
        })
    }

    /// Whether `p` is on the template parameter box of a view.
    fn in_template_box(&self, v: Eid, p: P) -> bool {
        self.repo.refd(v, "templateParameterCompartment").filter(|_| self.info.get(&v).map(|i| i.templ.is_some()).unwrap_or(false)).map(|tc| in_rect(p, &node_rect(&self.repo, tc))).unwrap_or(false)
    }

    fn hit_handle(&self, p: P) -> Option<usize> {
        if self.selection.len() != 1 {
            return None;
        }
        let v = self.selection[0];
        // an auto-resized box has no resize handles (diagram-editor.js:195, 1365-1369)
        if is_edge(&self.repo, v) || self.repo.bool(v, "autoResize") {
            return None;
        }
        let hs = 6.0 / self.zoom;
        let o = 4.0 / self.zoom;
        let b = node_rect(&self.repo, v);
        render::handle_points(&R { x: b.x - o, y: b.y - o, w: b.w + 2.0 * o, h: b.h + 2.0 * o }).iter().position(|h| (p.x - h.x).abs() <= hs && (p.y - h.y).abs() <= hs)
    }

    fn snap_v(&self, n: f64) -> f64 {
        let g = self.grid_size();
        if self.snap { (n / g).round() * g } else { n.round() }
    }

    /// Snap a dragged coordinate that started at `orig`: while it stays in the grid cell it started in,
    /// it keeps its original value, so an off-grid box does not jump on a tiny drag; once it crosses
    /// into another cell it snaps to the grid (diagram-editor.js:1201-1323, 1892-1915).
    pub(crate) fn snap_from(&self, orig: f64, n: f64) -> f64 {
        let g = self.grid_size();
        if self.snap && (n / g).floor() == (orig / g).floor() {
            orig
        } else {
            self.snap_v(n)
        }
    }

    fn to_diagram(&self, origin: Pos2, pos: Pos2) -> P {
        P { x: (pos.x - origin.x) as f64 / self.zoom + self.scroll.x, y: (pos.y - origin.y) as f64 / self.zoom + self.scroll.y }
    }

    fn mini_rect(rect: Rect) -> Rect {
        Rect::from_min_size(Pos2::new(rect.right() - MINI_MARGIN - MINI_W, rect.bottom() - MINI_MARGIN - MINI_H), Vec2::new(MINI_W, MINI_H))
    }

    fn world_box(&self, d: Eid, rect: Rect) -> R {
        let b = render::bounds(&self.repo, d).unwrap_or_default();
        let (vw, vh) = (rect.width() as f64 / self.zoom, rect.height() as f64 / self.zoom);
        let x1 = 0f64.min(b.x).min(self.scroll.x);
        let y1 = 0f64.min(b.y).min(self.scroll.y);
        let x2 = (b.x + b.w).max(self.scroll.x + vw);
        let y2 = (b.y + b.h).max(self.scroll.y + vh);
        R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
    }

    fn mini_scroll(&mut self, rect: Rect, sp: Pos2) {
        let Some(d) = self.diagram else { return };
        let m = Self::mini_rect(rect).shrink(5.0);
        let wb = self.world_box(d, rect);
        let s = (m.width() as f64 / wb.w.max(1.0)).min(m.height() as f64 / wb.h.max(1.0));
        let x = (sp.x - m.left()) as f64 / s + wb.x;
        let y = (sp.y - m.top()) as f64 / s + wb.y;
        self.scroll = P { x: (x - rect.width() as f64 / self.zoom / 2.0).max(0.0), y: (y - rect.height() as f64 / self.zoom / 2.0).max(0.0) };
    }

    // ------------------------------------------------------------------ the canvas
    pub fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        self.canvas_rect = Some(rect);
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        let ctx = ui.ctx().clone();
        let measure = EguiMeasure { ctx: ctx.clone() };

        let Some(d) = self.diagram.filter(|d| self.repo.alive(*d)) else {
            painter.rect_filled(rect, 0.0, AREA_COLOR);
            let c = rect.center();
            let st = Stroke::new(5.0, Color32::from_rgb(0xbd, 0xbd, 0xbd));
            painter.rect_stroke(Rect::from_min_size(c + Vec2::new(-45.0, -45.0), Vec2::new(56.0, 44.0)), 0.0, st, egui::StrokeKind::Middle);
            painter.rect_stroke(Rect::from_min_size(c + Vec2::new(-11.0, 1.0), Vec2::new(56.0, 44.0)), 0.0, st, egui::StrokeKind::Middle);
            return;
        };

        if let Some(v) = self.centre_pending.take() {
            if self.repo.alive(v) && self.repo.parent(v) == Some(d) {
                let b = self.view_bounds(v);
                self.centre_on(P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 });
            }
        }
        // a changed selection gives the canvas the keyboard, unless a text field is being typed in
        // (diagram-manager.js:657-660)
        if self.selection != self.last_selection {
            self.last_selection = self.selection.clone();
            // the explorer keeps the keyboard when its own clicks select views
            if !self.selection.is_empty() && self.editing.is_none() && self.last_area != crate::app::Area::Explorer && !crate::ui::text_field_focused(ui.ctx()) {
                self.refocus_canvas = true;
            }
        }
        if std::mem::take(&mut self.refocus_canvas) {
            resp.request_focus();
        }
        self.handle_input(ui, &resp, rect, d);
        if !self.repo.alive(d) || self.diagram != Some(d) {
            return;
        }

        self.info = views::layout_diagram(&mut self.repo, &measure, d);
        // a left or top resize below the content's minimum keeps the opposite edge where it was
        // (diagram-editor.js:1455-1493): the layout grew the box to the right or bottom, so shift it back
        if let Some(Drag::Resize { handle, orig, view, changed: true, .. }) = self.drag.as_ref() {
            let (h, o, v) = (*handle, *orig, *view);
            let b = node_rect(&self.repo, v);
            let mut moved = false;
            if [0, 6, 7].contains(&h) && b.x + b.w > o.x + o.w + 0.5 {
                self.repo.set(v, "left", crate::model::num((o.x + o.w - b.w).max(0.0)));
                moved = true;
            }
            if [0, 1, 2].contains(&h) && b.y + b.h > o.y + o.h + 0.5 {
                self.repo.set(v, "top", crate::model::num((o.y + o.h - b.h).max(0.0)));
                moved = true;
            }
            if moved {
                self.info = views::layout_diagram(&mut self.repo, &measure, d);
            }
        }
        // diagramEditor.backgroundColor and gridColor (default.json:135-141)
        let pref_color = |a: &App, key: &str, def: render::Rgba| a.pref(key).as_str().map(|s| render::parse_color(s, def)).unwrap_or(def);
        painter.rect_filled(rect, 0.0, c32(pref_color(self, "diagramEditor.backgroundColor", render::Rgba(255, 255, 255, 255))));
        let mut c = ScreenCanvas { painter: &painter, origin: rect.min, scroll: self.scroll, zoom: self.zoom, measure: EguiMeasure { ctx: ctx.clone() } };
        if self.show_grid {
            let grid = Stroke::new(1.0, c32(pref_color(self, "diagramEditor.gridColor", render::GRID_COLOR)));
            // lines every two grid cells
            let step = 2.0 * self.grid_size();
            let (vw, vh) = (rect.width() as f64 / self.zoom, rect.height() as f64 / self.zoom);
            let mut x = (self.scroll.x / step).floor() * step;
            while x <= self.scroll.x + vw {
                let sx = c.pos(P { x, y: 0.0 }).x.round();
                painter.line_segment([Pos2::new(sx, rect.top()), Pos2::new(sx, rect.bottom())], grid);
                x += step;
            }
            let mut y = (self.scroll.y / step).floor() * step;
            while y <= self.scroll.y + vh {
                let sy = c.pos(P { x: 0.0, y }).y.round();
                painter.line_segment([Pos2::new(rect.left(), sy), Pos2::new(rect.right(), sy)], grid);
                y += step;
            }
        }
        render::draw_diagram(&mut c, &self.repo, d, &self.info);
        self.draw_selection(&mut c);
        self.draw_drag(&mut c);
        self.draw_minimap(&painter, rect, d, &ctx);
        self.inline_editor(ui, rect);
    }

    fn draw_selection(&self, c: &mut ScreenCanvas) {
        let sel = c32(render::parse_color(render::SELECT_COLOR_HEX, render::SELECT_COLOR));
        let hs = render::HANDLE_SIZE as f32;
        let handle = |c: &ScreenCanvas, p: P| {
            let r = Rect::from_center_size(c.pos(p), Vec2::splat(hs));
            c.painter.rect_filled(r, 0.0, Color32::WHITE);
            c.painter.rect_stroke(r, 0.0, Stroke::new(1.0, sel), egui::StrokeKind::Inside);
        };
        for &v in &self.selection {
            if !self.repo.alive(v) {
                continue;
            }
            if is_edge(&self.repo, v) {
                let pts = points_of(&self.repo, v);
                let ps: Vec<Pos2> = pts.iter().map(|p| c.pos(*p)).collect();
                for s in Shape::dashed_line(&ps, Stroke::new(1.0, sel), 2.0, 2.0) {
                    c.painter.add(s);
                }
                for p in pts {
                    handle(c, p);
                }
            } else {
                let b = node_rect(&self.repo, v);
                let r = Rect::from_two_pos(c.pos(P { x: b.x, y: b.y }), c.pos(P { x: b.x + b.w, y: b.y + b.h }));
                c.painter.rect_stroke(r, 0.0, Stroke::new(1.0, sel), egui::StrokeKind::Middle);
                // centred 4 screen px outside the corners and edge midpoints (graphics.js:2726-2727)
                let o = 4.0 / c.zoom;
                for p in render::handle_points(&R { x: b.x - o, y: b.y - o, w: b.w + 2.0 * o, h: b.h + 2.0 * o }) {
                    handle(c, p);
                }
            }
        }
    }

    fn draw_drag(&self, c: &mut ScreenCanvas) {
        let sel = c32(render::SELECT_COLOR);
        match &self.drag {
            Some(Drag::CreateNode { start, now }) if self.tool.map(|t| t.kind == "FreeLine").unwrap_or(false) => {
                for s in Shape::dashed_line(&[c.pos(*start), c.pos(*now)], Stroke::new(1.0, sel), 3.0, 3.0) {
                    c.painter.add(s);
                }
            }
            Some(Drag::Rubber { start, now, .. }) | Some(Drag::CreateNode { start, now }) => {
                let r = rect_of(*start, *now);
                let pts = [
                    c.pos(P { x: r.x, y: r.y }), c.pos(P { x: r.x + r.w, y: r.y }), c.pos(P { x: r.x + r.w, y: r.y + r.h }),
                    c.pos(P { x: r.x, y: r.y + r.h }), c.pos(P { x: r.x, y: r.y }),
                ];
                for s in Shape::dashed_line(&pts, Stroke::new(1.0, sel), 3.0, 3.0) {
                    c.painter.add(s);
                }
            }
            Some(Drag::CreateEdge { start, now, tail }) => {
                for s in Shape::dashed_line(&[c.pos(*start), c.pos(*now)], Stroke::new(1.0, sel), 3.0, 3.0) {
                    c.painter.add(s);
                }
                if let (Some(h), Some(tool)) = (self.hover, self.tool) {
                    if h != *tail {
                        let ok = self.can_connect(&tool, *tail, h);
                        let b = node_rect(&self.repo, h);
                        let r = Rect::from_two_pos(c.pos(P { x: b.x, y: b.y }), c.pos(P { x: b.x + b.w, y: b.y + b.h }));
                        let col = if ok { sel } else { Color32::from_rgb(0xff, 0x5a, 0x4f) };
                        c.painter.rect_stroke(r, 0.0, Stroke::new(2.0, col), egui::StrokeKind::Middle);
                    }
                }
            }
            Some(Drag::Move { changed: true, .. }) => {
                // the prospective container, else the current one (diagram-editor.js:429-474)
                let current = self.container_group().and_then(|g| g.1);
                if let Some(cv) = self.hover.or(current) {
                    let b = node_rect(&self.repo, cv);
                    let r = Rect::from_two_pos(c.pos(P { x: b.x, y: b.y }), c.pos(P { x: b.x + b.w, y: b.y + b.h }));
                    c.painter.rect_stroke(r, 0.0, Stroke::new(2.0, sel), egui::StrokeKind::Middle);
                }
            }
            Some(Drag::Reconnect { edge, tail, now }) => {
                // rubber line from the end that stays, and the drop target outlined
                let pts = points_of(&self.repo, *edge);
                if pts.len() >= 2 {
                    let fixed = if *tail { pts[1] } else { pts[pts.len() - 2] };
                    for s in Shape::dashed_line(&[c.pos(fixed), c.pos(*now)], Stroke::new(1.0, sel), 3.0, 3.0) {
                        c.painter.add(s);
                    }
                }
                let current = self.repo.refd(*edge, if *tail { "tail" } else { "head" });
                if let Some(h) = self.hover.filter(|h| Some(*h) != current) {
                    let (t, hd) = (self.repo.refd(*edge, "tail"), self.repo.refd(*edge, "head"));
                    let ok = match (t, hd) {
                        (Some(t), Some(hd)) => self.can_link(&self.edge_kind(*edge), if *tail { h } else { t }, if *tail { hd } else { h }),
                        _ => false,
                    };
                    let b = node_rect(&self.repo, h);
                    let r = Rect::from_two_pos(c.pos(P { x: b.x, y: b.y }), c.pos(P { x: b.x + b.w, y: b.y + b.h }));
                    let col = if ok { sel } else { Color32::from_rgb(0xff, 0x5a, 0x4f) };
                    c.painter.rect_stroke(r, 0.0, Stroke::new(2.0, col), egui::StrokeKind::Middle);
                }
            }
            _ => {}
        }
    }

    fn draw_minimap(&self, painter: &Painter, rect: Rect, d: Eid, ctx: &egui::Context) {
        let m = Self::mini_rect(rect);
        if !self.show_minimap || m.left() < rect.left() + 10.0 || m.top() < rect.top() + 10.0 {
            return;
        }
        painter.rect_filled(m.translate(Vec2::new(0.0, 2.0)).expand(2.0), 2.0, Color32::from_black_alpha(40));
        painter.rect_filled(m, 0.0, Color32::from_rgb(0xf4, 0xf4, 0xf4));
        let inner = m.shrink(5.0);
        painter.rect_filled(inner, 0.0, Color32::WHITE);
        let wb = self.world_box(d, rect);
        let s = (inner.width() as f64 / wb.w.max(1.0)).min(inner.height() as f64 / wb.h.max(1.0));
        let mp = painter.with_clip_rect(inner);
        let mut mc = ScreenCanvas {
            painter: &mp,
            origin: inner.min,
            scroll: P { x: wb.x, y: wb.y },
            zoom: s,
            measure: EguiMeasure { ctx: ctx.clone() },
        };
        render::draw_diagram(&mut mc, &self.repo, d, &self.info);
        let vx = inner.left() + ((self.scroll.x - wb.x) * s) as f32;
        let vy = inner.top() + ((self.scroll.y - wb.y) * s) as f32;
        let vw = (rect.width() as f64 / self.zoom * s) as f32;
        let vh = (rect.height() as f64 / self.zoom * s) as f32;
        painter.rect_stroke(Rect::from_min_size(Pos2::new(vx, vy), Vec2::new(vw - 1.0, vh - 1.0)), 0.0, Stroke::new(1.0, c32(render::SELECT_COLOR)), egui::StrokeKind::Inside);
    }

    // ------------------------------------------------------------------ input
    fn handle_input(&mut self, ui: &mut egui::Ui, resp: &egui::Response, rect: Rect, d: Eid) {
        let (pointer, mods, scroll_delta, wants_kb) = ui.input(|i| (i.pointer.clone(), i.modifiers, i.smooth_scroll_delta, i.focused.then_some(()).is_some()));
        let _ = wants_kb;
        let origin = rect.min;
        let latest = pointer.latest_pos();

        // drop from the explorer: the cursor says whether the diagram takes it (model-explorer-view.js:161-224)
        if let Some(eid) = resp.dnd_hover_payload::<Eid>() {
            ui.ctx().set_cursor_icon(if self.can_drop_model(*eid) { egui::CursorIcon::Copy } else { egui::CursorIcon::NotAllowed });
        }
        if let Some(eid) = resp.dnd_release_payload::<Eid>() {
            if let Some(pos) = latest {
                let p = self.to_diagram(origin, pos);
                let m = *eid;
                if self.repo.alive(m) {
                    match self.drop_model(m, P { x: self.snap_v(p.x), y: self.snap_v(p.y) }) {
                        Ok(vs) if !vs.is_empty() => self.select(vs),
                        Ok(_) => {}
                        Err(msg) => self.message = Some(("UMLCraft".into(), msg)),
                    }
                }
            }
        }

        if resp.hovered() {
            if mods.command && scroll_delta.y != 0.0 {
                let z = self.zoom + if scroll_delta.y > 0.0 { 0.1 } else { -0.1 };
                self.set_zoom(z);
            } else if scroll_delta != Vec2::ZERO {
                let (dx, dy) = if mods.shift { (-scroll_delta.y, 0.0) } else { (-scroll_delta.x, -scroll_delta.y) };
                self.scroll.x = (self.scroll.x + dx as f64 / self.zoom).max(0.0);
                self.scroll.y = (self.scroll.y + dy as f64 / self.zoom).max(0.0);
            }
        }

        if resp.hovered() && pointer.primary_pressed() {
            if let Some(pos) = pointer.interact_pos() {
                resp.request_focus();
                self.last_area = crate::app::Area::Canvas;
                self.close_inline(true);
                self.on_press(rect, pos, mods);
            }
        }
        if resp.hovered() && pointer.button_pressed(egui::PointerButton::Middle) {
            if let Some(pos) = pointer.interact_pos() {
                self.drag = Some(Drag::Pan { start: crate::app::egui_pos::Pos { x: pos.x, y: pos.y }, scroll: self.scroll });
            }
        }
        if let (Some(pos), true) = (latest, self.drag.is_some()) {
            if pointer.any_down() {
                self.on_move(rect, pos);
            }
        }
        if pointer.any_released() && self.drag.is_some() {
            let pos = latest.unwrap_or(rect.center());
            self.on_release(origin, pos);
        }
        if matches!(self.drag, Some(Drag::Pan { .. })) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
        }
        if !matches!(self.drag, Some(_)) {
            if let Some(pos) = resp.hover_pos() {
                let p = self.to_diagram(origin, pos);
                let icon = if self.tool.is_some() {
                    egui::CursorIcon::Crosshair
                } else {
                    match self.hit_handle(p) {
                        Some(0) | Some(4) => egui::CursorIcon::ResizeNwSe,
                        Some(2) | Some(6) => egui::CursorIcon::ResizeNeSw,
                        Some(1) | Some(5) => egui::CursorIcon::ResizeVertical,
                        Some(3) | Some(7) => egui::CursorIcon::ResizeHorizontal,
                        _ => self.edge_cursor(p).unwrap_or(egui::CursorIcon::Default),
                    }
                };
                ui.ctx().set_cursor_icon(icon);
            }
        }

        if resp.double_clicked() && self.tool.is_none() {
            if let Some(pos) = pointer.interact_pos() {
                let p = self.to_diagram(origin, pos);
                if let Some(v) = self.hit_view(p, false) {
                    self.edit_view(v, Some(p));
                } else {
                    self.create_text_at(p);
                }
            }
        }

        if resp.secondary_clicked() {
            if let Some(pos) = pointer.interact_pos() {
                let p = self.to_diagram(origin, pos);
                self.ctx_pos = p;
                self.ctx_target = self.hit_view(p, false);
                match self.ctx_target {
                    Some(v) if !self.selection.contains(&v) => self.select(vec![v]),
                    Some(_) => {}
                    // a right-click on empty canvas deselects everything (diagram-editor.js:701-704)
                    None => self.select(vec![]),
                }
            }
        }
        resp.context_menu(|ui| self.canvas_menu(ui));

        // keyboard on the canvas
        if resp.has_focus() && self.editing.is_none() && self.last_area != crate::app::Area::Explorer && !self.modal_open() {
            // keep Tab and the arrows on the canvas instead of moving the keyboard focus
            ui.memory_mut(|m| m.set_focus_lock_filter(resp.id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true }));
            let tab = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
            if tab {
                self.select_next_view();
            }
            let (del, esc, enter, arrows) = ui.input(|i| {
                (
                    (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) && !i.modifiers.command,
                    i.key_pressed(egui::Key::Escape),
                    i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::F2),
                    (
                        i.key_pressed(egui::Key::ArrowLeft),
                        i.key_pressed(egui::Key::ArrowRight),
                        i.key_pressed(egui::Key::ArrowUp),
                        i.key_pressed(egui::Key::ArrowDown),
                        i.modifiers.shift,
                    ),
                )
            });
            if del {
                self.delete_selection(false);
            }
            if esc {
                self.tool = None;
                self.select(vec![]);
            }
            // a keyboard double-click on the first selected view (diagram-manager.js:712-725)
            if let Some(v) = self.selection.first().copied().filter(|v| enter && self.repo.alive(*v)) {
                self.edit_view(v, None);
            }
            let (l, r, u, dn, shift) = arrows;
            if (l || r || u || dn) && !self.selection.is_empty() {
                // Shift moves by the grid factor: the grid size, or 1 with snap off (diagram-manager.js:761-765)
                let step = if shift && self.snap { self.grid_size() } else { 1.0 };
                let dx = if l { -step } else if r { step } else { 0.0 };
                let dy = if u { -step } else if dn { step } else { 0.0 };
                self.nudge_selection(dx, dy);
            }
        }
        let _ = d;
    }

    /// Move the selected boxes by (dx, dy) as one undo step; contained views follow their box, and a
    /// box inside a selected one moves only once (diagram-manager.js:755-814).
    pub fn nudge_selection(&mut self, dx: f64, dy: f64) {
        let sel = self.selection.clone();
        // contained views follow their box; a box inside a selected one moves only once
        let inside_selected = |a: &App, v: Eid| {
            let mut c = a.repo.refd(v, "containerView");
            while let Some(x) = c {
                if sel.contains(&x) {
                    return true;
                }
                c = a.repo.refd(x, "containerView");
            }
            false
        };
        let movers: Vec<Eid> = sel.iter().copied().filter(|v| !is_edge(&self.repo, *v) && !inside_selected(self, *v)).collect();
        let lines: Vec<Eid> = sel.iter().copied().filter(|v| views::is_freeline(&self.repo, *v)).collect();
        self.change(|a| {
            for v in movers {
                a.shift_view(v, dx, dy);
            }
            // free lines move by their points
            for e in lines {
                let pts: Vec<P> = points_of(&a.repo, e).into_iter().map(|q| P { x: (q.x + dx).max(0.0), y: (q.y + dy).max(0.0) }).collect();
                a.repo.set(e, "points", json!(views::format_points(&pts)));
            }
        });
    }

    pub(crate) fn on_press(&mut self, rect: Rect, pos: Pos2, mods: egui::Modifiers) {
        let p = self.to_diagram(rect.min, pos);
        if self.show_minimap && Self::mini_rect(rect).contains(pos) && rect.width() > MINI_W + 40.0 {
            self.drag = Some(Drag::Mini);
            self.mini_scroll(rect, pos);
            return;
        }
        // Ctrl/Cmd + left-drag pans, and while Ctrl/Cmd is held nothing is edited or selected
        // (diagram-editor.js:2178-2193, 2236-2244)
        if mods.command {
            self.drag = Some(Drag::Pan { start: crate::app::egui_pos::Pos { x: pos.x, y: pos.y }, scroll: self.scroll });
            return;
        }
        if let Some(tool) = self.tool {
            if !tool.line || tool.kind == "FreeLine" {
                self.drag = Some(Drag::CreateNode { start: p, now: p });
            } else if let Some(tail) = self.hit_view(p, tool.kind != "NoteLink" && tool.kind != "AssociationClass") {
                self.drag = Some(Drag::CreateEdge { start: p, now: p, tail });
            }
            return;
        }
        if let Some(h) = self.hit_handle(p) {
            let v = self.selection[0];
            self.drag = Some(Drag::Resize { handle: h, start: p, orig: node_rect(&self.repo, v), view: v, changed: false });
            return;
        }
        // ends and bend points of a selected line
        if self.selection.len() == 1 && is_edge(&self.repo, self.selection[0]) {
            let e = self.selection[0];
            let pts = points_of(&self.repo, e);
            let near = |q: P| ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt() <= 6.0 / self.zoom;
            let n = pts.len();
            if n >= 2 {
                for (tail, q) in [(true, pts[0]), (false, pts[n - 1])] {
                    if near(q) {
                        // a free line's end only moves (diagram-editor.js:1795-1797)
                        self.drag = Some(if views::is_freeline(&self.repo, e) {
                            Drag::Point { edge: e, index: if tail { 0 } else { n - 1 }, insert: None, changed: false }
                        } else {
                            Drag::Reconnect { edge: e, tail, now: p }
                        });
                        return;
                    }
                }
            }
            for i in 1..n.saturating_sub(1) {
                if near(pts[i]) {
                    self.drag = Some(Drag::Point { edge: e, index: i, insert: None, changed: false });
                    return;
                }
            }
            let style = self.repo.val(e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
            for i in 0..n.saturating_sub(1) {
                if dist_to_seg(p, pts[i], pts[i + 1]) <= 5.0 / self.zoom {
                    let (a, b) = (pts[i], pts[i + 1]);
                    let straight = (a.y - b.y).abs() < 0.5 || (a.x - b.x).abs() < 0.5;
                    if views::rectilinear(style) && straight {
                        // move the segment across, keeping the path orthogonal (diagram-editor.js:1923-1979)
                        self.drag = Some(Drag::Segment { edge: e, index: i, start: p, orig: pts.clone(), horizontal: (a.y - b.y).abs() < 0.5, changed: false });
                    } else {
                        self.drag = Some(Drag::Point { edge: e, index: i + 1, insert: Some(p), changed: false });
                    }
                    return;
                }
            }
        }
        // a line's label can be dragged away from its line (diagram-editor.js:2051-2106)
        if !mods.shift {
            if let Some((label, edge)) = self.hit_edge_label(p) {
                if !self.selection.contains(&edge) {
                    self.select(vec![edge]);
                }
                let b = node_rect(&self.repo, label);
                let orig = P { x: b.x + (b.w / 2.0).floor(), y: b.y + (views::LABEL_H / 2.0).floor() };
                self.drag = Some(Drag::Label { label, edge, start: p, orig, changed: false });
                return;
            }
        }
        let add = mods.shift;
        match self.hit_view(p, false) {
            None => {
                if !add {
                    self.select(vec![]);
                }
                self.drag = Some(Drag::Rubber { start: p, now: p, add, base: self.selection.clone() });
            }
            Some(v) => {
                if add {
                    // Shift+click toggles, and a Shift press never drags (diagram-editor.js:645-660)
                    let mut sel = self.selection.clone();
                    if let Some(i) = sel.iter().position(|x| *x == v) {
                        sel.remove(i);
                    } else {
                        sel.push(v);
                    }
                    self.select(sel);
                    return;
                }
                let group = self.selection.len() > 1 && self.selection.contains(&v);
                if !self.selection.contains(&v) {
                    self.select(vec![v]);
                } else {
                    self.focus = Some(self.repo.refd(v, "model").unwrap_or(v));
                }
                self.start_move(p);
                if let Some(Drag::Move { collapse, .. }) = &mut self.drag {
                    *collapse = group.then_some(v);
                }
            }
        }
    }

    /// The selected boxes that may change container, if they all can and share one (the current one).
    fn container_group(&self) -> Option<(Vec<Eid>, Option<Eid>)> {
        let vs: Vec<Eid> = self.selection.iter().copied().filter(|v| self.repo.alive(*v) && is_node(&self.repo, *v)).collect();
        if vs.is_empty() || !vs.iter().all(|v| self.repo.bool(*v, "containerChangeable")) {
            return None;
        }
        let current = self.repo.refd(vs[0], "containerView");
        vs.iter().all(|v| self.repo.refd(*v, "containerView") == current).then_some((vs, current))
    }

    /// The topmost package or subject under `p` that can hold the selected boxes, other than the moved ones.
    fn container_at(&self, p: P, moved: &[Eid]) -> Option<Eid> {
        let (vs, _) = self.container_group()?;
        let d = self.diagram?;
        self.repo.list(d, "ownedViews").into_iter().rev().find(|c| {
            !moved.contains(c)
                && !matches!(self.repo.raw(*c, "visible"), Some(serde_json::Value::Bool(false)))
                && in_rect(p, &node_rect(&self.repo, *c))
                && vs.iter().all(|v| self.can_contain_view(*c, *v))
        })
    }

    fn start_move(&mut self, p: P) {
        let Some(d) = self.diagram else { return };
        let mut nodes: Vec<Eid> = vec![];
        let push = |nodes: &mut Vec<Eid>, v: Eid| {
            if !nodes.contains(&v) {
                nodes.push(v);
            }
        };
        for &v in &self.selection {
            if is_edge(&self.repo, v) {
                continue;
            }
            push(&mut nodes, v);
        }
        // contained views follow their container, recursively (engine.js:633-654)
        let mut i = 0;
        while i < nodes.len() {
            if let Some(serde_json::Value::Array(cs)) = self.repo.raw(nodes[i], "containedViews").cloned() {
                for c in cs {
                    if let Some(cv) = self.repo.by_ref(&c) {
                        push(&mut nodes, cv);
                    }
                }
            }
            i += 1;
        }
        // things drawn inside a package or subject move with it
        for v in nodes.clone() {
            let t = self.repo.ty(v);
            if t == "UMLPackageView" || t == "UMLUseCaseSubjectView" {
                let b = node_rect(&self.repo, v);
                for o in self.repo.list(d, "ownedViews") {
                    if o != v && is_node(&self.repo, o) && contains(&b, &node_rect(&self.repo, o)) {
                        push(&mut nodes, o);
                    }
                }
            }
        }
        let orig: Vec<(Eid, P)> = nodes.iter().map(|n| (*n, P { x: self.repo.f(*n, "left"), y: self.repo.f(*n, "top") })).collect();
        let mut edges = vec![];
        for e in self.repo.list(d, "ownedViews") {
            if !is_edge(&self.repo, e) {
                continue;
            }
            if let (Some(t), Some(h)) = (self.repo.refd(e, "tail"), self.repo.refd(e, "head")) {
                if nodes.contains(&t) && nodes.contains(&h) {
                    edges.push((e, points_of(&self.repo, e)));
                }
            } else if views::is_freeline(&self.repo, e) && self.selection.contains(&e) {
                // a selected free line moves with the group
                edges.push((e, points_of(&self.repo, e)));
            }
        }
        self.drag = Some(Drag::Move { start: p, orig, edges, lead: nodes.first().copied(), changed: false, collapse: None });
    }

    pub(crate) fn on_move(&mut self, rect: Rect, pos: Pos2) {
        let p = self.to_diagram(rect.min, pos);
        let Some(mut drag) = self.drag.take() else { return };
        match &mut drag {
            Drag::Pan { start, scroll } => {
                self.scroll.x = (scroll.x - (pos.x - start.x) as f64 / self.zoom).max(0.0);
                self.scroll.y = (scroll.y - (pos.y - start.y) as f64 / self.zoom).max(0.0);
            }
            Drag::Mini => self.mini_scroll(rect, pos),
            Drag::Rubber { start, now, add, base } => {
                // the band stops at the left and top edge (diagram-editor.js:747-765)
                let p = P { x: p.x.max(0.0), y: p.y.max(0.0) };
                *now = p;
                let r = rect_of(*start, p);
                let mut inside = vec![];
                if let Some(d) = self.diagram {
                    // views whose box overlaps the band; a selected one counts 5 px larger (core.js:4162-4176)
                    for v in self.repo.list(d, "ownedViews") {
                        if matches!(self.repo.raw(v, "visible"), Some(serde_json::Value::Bool(false))) || !(is_edge(&self.repo, v) || is_node(&self.repo, v)) {
                            continue;
                        }
                        let mut b = self.view_bounds(v);
                        if base.contains(&v) {
                            b = R { x: b.x - 5.0, y: b.y - 5.0, w: b.w + 10.0, h: b.h + 10.0 };
                        }
                        if overlaps(&r, &b) {
                            inside.push(v);
                        }
                    }
                }
                if *add {
                    let mut s = base.clone();
                    for v in inside {
                        if !s.contains(&v) {
                            s.push(v);
                        }
                    }
                    self.selection = s;
                } else {
                    self.selection = inside;
                }
            }
            Drag::CreateNode { now, .. } => *now = p,
            Drag::CreateEdge { now, .. } => {
                *now = p;
                let nodes_only = self.tool.map(|t| t.kind != "NoteLink" && t.kind != "AssociationClass").unwrap_or(true);
                self.hover = self.hit_view(p, nodes_only);
            }
            Drag::Move { start, orig, edges, lead, changed, .. } => {
                let (mut dx, mut dy) = (p.x - start.x, p.y - start.y);
                if !*changed && (dx * dx + dy * dy).sqrt() * self.zoom < 3.0 {
                    self.drag = Some(drag);
                    return;
                }
                if !*changed {
                    self.checkpoint();
                    *changed = true;
                }
                if let Some(l) = lead {
                    if let Some((_, o)) = orig.iter().find(|(n, _)| n == l) {
                        dx = self.snap_from(o.x, o.x + dx) - o.x;
                        dy = self.snap_from(o.y, o.y + dy) - o.y;
                    }
                }
                // the group stops at the canvas origin (diagram-editor.js:1287-1323)
                let min_x = orig.iter().map(|(_, o)| o.x).fold(f64::INFINITY, f64::min);
                let min_y = orig.iter().map(|(_, o)| o.y).fold(f64::INFINITY, f64::min);
                if min_x.is_finite() {
                    dx = dx.max(-min_x.max(0.0));
                    dy = dy.max(-min_y.max(0.0));
                }
                for (n, o) in orig.iter() {
                    self.repo.set(*n, "left", crate::model::num(o.x + dx));
                    self.repo.set(*n, "top", crate::model::num(o.y + dy));
                }
                let moved: Vec<Eid> = orig.iter().map(|(n, _)| *n).collect();
                self.hover = self.container_at(p, &moved);
                for (e, pts) in edges.iter() {
                    if pts.len() > 2 || views::is_freeline(&self.repo, *e) {
                        let moved: Vec<P> = pts.iter().map(|q| P { x: q.x + dx, y: q.y + dy }).collect();
                        self.repo.set(*e, "points", json!(views::format_points(&moved)));
                    }
                }
            }
            Drag::Resize { handle, start, orig, view, changed } => {
                let (dx, dy) = (p.x - start.x, p.y - start.y);
                let o = *orig;
                let (mut x1, mut y1, mut x2, mut y2) = (o.x, o.y, o.x + o.w, o.y + o.h);
                let h = *handle;
                if [0, 6, 7].contains(&h) {
                    x1 = self.snap_from(o.x, o.x + dx);
                }
                if [2, 3, 4].contains(&h) {
                    x2 = self.snap_from(o.x + o.w, o.x + o.w + dx);
                }
                if [0, 1, 2].contains(&h) {
                    y1 = self.snap_from(o.y, o.y + dy);
                }
                if [4, 5, 6].contains(&h) {
                    y2 = self.snap_from(o.y + o.h, o.y + o.h + dy);
                }
                // edges stop at the canvas origin (diagram-editor.js:1201-1285)
                x1 = x1.max(0.0);
                y1 = y1.max(0.0);
                if x2 - x1 < 20.0 {
                    if x1 != o.x { x1 = x2 - 20.0 } else { x2 = x1 + 20.0 }
                }
                if y2 - y1 < 20.0 {
                    if y1 != o.y { y1 = y2 - 20.0 } else { y2 = y1 + 20.0 }
                }
                if !*changed {
                    self.checkpoint();
                    *changed = true;
                }
                let v = *view;
                self.repo.set(v, "left", crate::model::num(x1));
                self.repo.set(v, "top", crate::model::num(y1));
                // shrinking below the content is undone by the layout, which only grows boxes
                self.repo.set_raw(v, "width", crate::model::num(x2 - x1));
                self.repo.set_raw(v, "height", crate::model::num(y2 - y1));
            }
            Drag::Point { edge, index, insert, changed } => {
                if !*changed {
                    self.checkpoint();
                    *changed = true;
                    if let Some(at) = insert.take() {
                        let mut pts = points_of(&self.repo, *edge);
                        pts.insert(*index, at);
                        self.repo.set(*edge, "points", json!(views::format_points(&pts)));
                    }
                }
                let mut pts = points_of(&self.repo, *edge);
                if *index < pts.len() {
                    pts[*index] = P { x: self.snap_v(p.x).max(0.0), y: self.snap_v(p.y).max(0.0) };
                    self.repo.set(*edge, "points", json!(views::format_points(&pts)));
                }
            }
            Drag::Label { label, edge, start, orig, changed } => {
                let (dx, dy) = (p.x - start.x, p.y - start.y);
                if !*changed && (dx * dx + dy * dy).sqrt() * self.zoom < 3.0 {
                    self.drag = Some(drag);
                    return;
                }
                if !*changed {
                    self.checkpoint();
                    *changed = true;
                }
                let pts = points_of(&self.repo, *edge);
                if pts.len() >= 2 {
                    let (p1, p2) = views::label_anchor(&self.repo, *label, &pts);
                    let (alpha, distance) = views::label_polar(p1, p2, P { x: orig.x + dx, y: orig.y + dy });
                    self.repo.set(*label, "alpha", json!(alpha));
                    self.repo.set(*label, "distance", json!(distance));
                }
            }
            Drag::Reconnect { now, .. } => {
                *now = p;
                self.hover = self.hit_view(p, true);
            }
            Drag::Segment { edge, index, start, orig, horizontal, changed } => {
                let d = if *horizontal { p.y - start.y } else { p.x - start.x };
                if !*changed && d.abs() * self.zoom < 3.0 {
                    self.drag = Some(drag);
                    return;
                }
                if !*changed {
                    self.checkpoint();
                    *changed = true;
                    // an end segment gets an extra point so the end stays on its box
                    let n = orig.len();
                    if *index + 1 == n - 1 {
                        orig.insert(n - 1, orig[n - 1]);
                    }
                    if *index == 0 {
                        orig.insert(0, orig[0]);
                        *index = 1;
                    }
                }
                let mut pts = orig.clone();
                for k in [*index, *index + 1] {
                    if *horizontal {
                        pts[k].y = self.snap_v(orig[k].y + d).max(0.0);
                    } else {
                        pts[k].x = self.snap_v(orig[k].x + d).max(0.0);
                    }
                }
                self.repo.set(*edge, "points", json!(views::format_points(&pts)));
            }
        }
        self.drag = Some(drag);
    }

    pub(crate) fn on_release(&mut self, origin: Pos2, pos: Pos2) {
        let Some(drag) = self.drag.take() else { return };
        self.hover = None;
        let p = self.to_diagram(origin, pos);
        match drag {
            Drag::Rubber { .. } => {
                let s = self.selection.clone();
                self.select(s);
            }
            Drag::CreateNode { start, .. } if self.tool.map(|t| t.kind == "FreeLine").unwrap_or(false) => {
                // a free line goes from the press to the release; a click gives a 100 px line (estimated)
                let a = P { x: self.snap_v(start.x).max(0.0), y: self.snap_v(start.y).max(0.0) };
                let mut b = P { x: self.snap_v(p.x).max(0.0), y: self.snap_v(p.y).max(0.0) };
                if ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt() < 10.0 {
                    b = P { x: a.x + 100.0, y: a.y };
                }
                let v = self.change(|app| app.create_freeline(a, b));
                if !self.tool_sticky {
                    self.tool = None;
                }
                if let Some(v) = v {
                    self.select(vec![v]);
                }
            }
            Drag::CreateNode { start, .. } => {
                let Some(tool) = self.tool else { return };
                let raw = rect_of(start, p);
                let r = if raw.w < 10.0 || raw.h < 10.0 {
                    let (w, h) = views::default_size(tool.kind);
                    R { x: self.snap_v(start.x), y: self.snap_v(start.y), w, h }
                } else {
                    // both corners snap to the grid (diagram-editor.js:979-1038)
                    let (x1, y1) = (self.snap_v(raw.x), self.snap_v(raw.y));
                    let (x2, y2) = (self.snap_v(raw.x + raw.w), self.snap_v(raw.y + raw.h));
                    R { x: x1, y: y1, w: (x2 - x1).max(1.0), h: (y2 - y1).max(1.0) }
                };
                let view = self.change(|a| {
                    let v = a.create_node(tool.kind, r);
                    if let (Some(v), Some(d)) = (v, a.diagram) {
                        let t = a.repo.ty(v);
                        if t == "UMLUseCaseSubjectView" || t == "UMLPackageView" || t == "UMLSubsystemView" {
                            // containers sit behind what they contain
                            let mut list = a.repo.list(d, "ownedViews");
                            list.retain(|x| *x != v);
                            list.insert(0, v);
                            a.repo.set_list(d, "ownedViews", list);
                        }
                    }
                    v
                });
                if !self.tool_sticky {
                    self.tool = None;
                }
                if let Some(v) = view {
                    self.select(vec![v]);
                    // the new element shows in the explorer, its owners expanded (app-context.js:837-848)
                    if let Some(m) = self.repo.refd(v, "model") {
                        self.reveal(m);
                    }
                    self.pending_edit = Some(v);
                }
            }
            Drag::CreateEdge { tail, .. } => {
                let Some(tool) = self.tool else { return };
                if let Some(head) = self.hit_view(p, tool.kind != "NoteLink" && tool.kind != "AssociationClass") {
                    if !self.can_connect(&tool, tail, head) {
                        // a refused link is reported (uml-factory.js:19-146, toolbox-view.js:209-216)
                        self.toast_error(&format!("Invalid connection ({})", tool.kind.trim_start_matches("UML")));
                    } else if let Some(v) = self.change(|a| a.create_edge(&tool, tail, head)) {
                        self.select(vec![v]);
                        if let Some(m) = self.repo.refd(v, "model") {
                            self.reveal(m);
                        }
                        // its name edit opens, once the line is laid out (toolbox-view.js:196-208)
                        if self.repo.has(v, "model") {
                            self.pending_edit = Some(v);
                        }
                    }
                }
                if !self.tool_sticky {
                    self.tool = None;
                }
            }
            Drag::Move { changed: true, orig, .. } => {
                // drop into a container, or out of one (diagram-editor.js:1640-1657, 476-493)
                let moved: Vec<Eid> = orig.iter().map(|(n, _)| *n).collect();
                if let Some((vs, current)) = self.container_group() {
                    let target = self.container_at(p, &moved);
                    if target != current && vs.iter().all(|v| target.map(|t| self.can_contain_view(t, *v)).unwrap_or(true)) {
                        self.set_container(&vs, target);
                    }
                }
                self.done();
            }
            Drag::Move { changed: false, collapse: Some(v), .. } => {
                // a click on a member of a group, without moving, keeps only that view (diagram-editor.js:901-912)
                if self.repo.alive(v) {
                    self.select(vec![v]);
                }
            }
            Drag::Reconnect { edge, tail, .. } => {
                if let Some(to) = self.hit_view(p, true) {
                    if let Err(msg) = self.reconnect(edge, tail, to) {
                        self.toast(&msg);
                    }
                }
            }
            Drag::Point { edge, changed: true, .. } | Drag::Segment { edge, changed: true, .. } => {
                // tidy the bend points (diagram-editor.js:1774-1781)
                let pts = points_of(&self.repo, edge);
                let style = self.repo.val(edge, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
                let clean = views::clean_points(&pts, views::rectilinear(style));
                if clean != pts {
                    self.repo.set(edge, "points", json!(views::format_points(&clean)));
                }
                self.done();
            }
            Drag::Move { changed, .. } | Drag::Resize { changed, .. } | Drag::Point { changed, .. } | Drag::Segment { changed, .. } | Drag::Label { changed, .. } => {
                if changed {
                    self.done();
                }
            }
            Drag::Pan { .. } | Drag::Mini => {}
        }
    }

    // ------------------------------------------------------------------ inline editing
    fn row_at(&self, v: Eid, p: P) -> Option<Eid> {
        let i = self.info.get(&v)?;
        // a template parameter row is found by both coordinates, as its box sits over the corner
        if let Some(t) = &i.templ {
            for row in &t.rows {
                let b = node_rect(&self.repo, row.iv);
                if in_rect(p, &R { x: b.x - 1.0, y: b.y - 1.0, w: b.w + 2.0, h: b.h + 2.0 }) {
                    return Some(row.iv);
                }
            }
        }
        for c in &i.compartments {
            for row in &c.rows {
                let b = node_rect(&self.repo, row.iv);
                if p.y >= b.y - 1.0 && p.y <= b.y + b.h + 1.0 {
                    return Some(row.iv);
                }
            }
        }
        None
    }

    pub fn edit_view(&mut self, v: Eid, p: Option<P>) {
        let t = self.repo.ty(v).to_string();
        // a constraint edits its specification (uml-commands.js:669)
        if t == "UMLConstraintView" {
            let Some(m) = self.repo.refd(v, "model") else { return };
            let text = self.repo.str(m, "specification");
            self.editing = Some(Editing { kind: EditKind::Text(v), text, rect: node_rect(&self.repo, v), multiline: true, center: false, fresh: true, host: None });
            return;
        }
        if t == "UMLNoteView" || t == "UMLTextView" || t == "UMLTextBoxView" || t == "UMLCustomTextView" {
            let text = self.repo.str(v, "text");
            self.editing = Some(Editing { kind: EditKind::Text(v), text, rect: node_rect(&self.repo, v), multiline: true, center: false, fresh: true, host: None });
            return;
        }
        let Some(m) = self.repo.refd(v, "model") else { return };
        if is_edge(&self.repo, v) {
            let pts = points_of(&self.repo, v);
            let mid = pts.get(pts.len() / 2).copied().unwrap_or_default();
            // on an association, a click nearer an end than the middle edits that end's role name
            // (quickedit-manager.js:151-156, 589-624)
            if let (Some(p), true, n) = (p, self.repo.has(m, "end1"), pts.len()) {
                if n >= 2 {
                    let mi = if n % 2 == 0 { n / 2 - 1 } else { n / 2 };
                    let centre = if n % 2 == 0 { P { x: (pts[mi].x + pts[mi + 1].x) / 2.0, y: (pts[mi].y + pts[mi + 1].y) / 2.0 } } else { pts[mi] };
                    let dist = |q: P| (q.x - p.x).powi(2) + (q.y - p.y).powi(2);
                    let (dt, dh, dm) = (dist(pts[0]), dist(pts[n - 1]), dist(centre));
                    let end = if dt < dh && dt < dm { Some(("end1", "tailRoleNameLabel", pts[0])) } else if dh < dt && dh < dm { Some(("end2", "headRoleNameLabel", pts[n - 1])) } else { None };
                    if let Some((field, label, at)) = end {
                        if let Some(e) = self.repo.child(m, field) {
                            let rect = match self.repo.refd(v, label) {
                                Some(l) if !self.repo.str(l, "text").is_empty() => R { x: self.repo.f(l, "left") - 20.0, y: self.repo.f(l, "top") - 3.0, w: self.repo.f(l, "width") + 40.0, h: 19.0 },
                                _ => R { x: at.x - 50.0, y: at.y - 25.0, w: 100.0, h: 19.0 },
                            };
                            self.editing = Some(Editing { kind: EditKind::Name(e), text: crate::parse::name_text(&self.repo, e), rect, multiline: false, center: true, fresh: true, host: None });
                            return;
                        }
                    }
                }
            }
            let lv = self.repo.refd(v, "nameLabel");
            let rect = match lv {
                Some(l) if !self.repo.str(l, "text").is_empty() => R { x: self.repo.f(l, "left") - 20.0, y: self.repo.f(l, "top") - 3.0, w: self.repo.f(l, "width") + 40.0, h: 19.0 },
                _ => R { x: mid.x - 50.0, y: mid.y - 10.0, w: 100.0, h: 19.0 },
            };
            self.editing = Some(Editing { kind: EditKind::Name(m), text: crate::parse::name_text(&self.repo, m), rect, multiline: false, center: true, fresh: true, host: None });
            return;
        }
        if let Some(iv) = p.and_then(|p| self.row_at(v, p)) {
            if self.edit_row(v, iv) {
                return;
            }
        }
        let nl = self.repo.refd(v, "nameCompartment").and_then(|nc| self.repo.refd(nc, "nameLabel"));
        let rect = match nl {
            Some(l) => R { x: self.repo.f(l, "left") - 2.0, y: self.repo.f(l, "top") - 3.0, w: self.repo.f(l, "width").max(80.0) + 4.0, h: 19.0 },
            None => node_rect(&self.repo, v),
        };
        self.editing = Some(Editing { kind: EditKind::Name(m), text: crate::parse::name_text(&self.repo, m), rect, multiline: false, center: true, fresh: true, host: Some(v) });
    }

    /// Open the inline edit on one compartment row (an attribute, operation or literal label).
    fn edit_row(&mut self, v: Eid, iv: Eid) -> bool {
        let Some(item) = self.repo.refd(iv, "model") else { return false };
        let b = node_rect(&self.repo, iv);
        self.editing = Some(Editing {
            kind: EditKind::Feature(item),
            // a reception is edited as a name expression, without its «signal» keyword
            text: if self.repo.ty(item) == "UMLReception" { crate::parse::name_text(&self.repo, item) } else { self.repo.str(iv, "text") },
            rect: R { x: b.x - 2.0, y: b.y - 3.0, w: b.w.max(120.0) + 4.0, h: 19.0 },
            multiline: false,
            center: false,
            fresh: true,
            host: Some(v),
        });
        true
    }

    /// The visible compartment rows of a box, top to bottom: (row view, item).
    fn item_rows(&self, v: Eid) -> Vec<(Eid, Eid)> {
        let Some(i) = self.info.get(&v) else { return vec![] };
        i.compartments.iter().flat_map(|c| c.rows.iter()).filter_map(|r| self.repo.refd(r.iv, "model").map(|m| (r.iv, m))).collect()
    }

    /// The compartment rows and the template parameter rows of a box.
    fn all_item_rows(&self, v: Eid) -> Vec<(Eid, Eid)> {
        let mut rows = self.item_rows(v);
        if let Some(t) = self.info.get(&v).and_then(|i| i.templ.as_ref()) {
            rows.extend(t.rows.iter().filter_map(|r| self.repo.refd(r.iv, "model").map(|m| (r.iv, m))));
        }
        rows
    }

    /// Open the queued item edit once its row is laid out, waiting a few frames at most.
    pub fn take_pending_item(&mut self) {
        let Some((v, item, left)) = self.pending_item.take() else { return };
        if !self.repo.alive(v) || !self.repo.alive(item) {
            return;
        }
        if self.item_shown(v, item) {
            self.edit_item(v, item);
        } else if left > 0 {
            self.pending_item = Some((v, item, left - 1));
        }
    }

    pub fn item_shown(&self, v: Eid, item: Eid) -> bool {
        self.all_item_rows(v).iter().any(|(_, m)| *m == item)
    }

    /// Edit this item inline: now if its row is laid out, else after the next layout.
    pub fn edit_item(&mut self, v: Eid, item: Eid) {
        match self.all_item_rows(v).into_iter().find(|(_, m)| *m == item) {
            Some((iv, _)) if self.repo.alive(iv) => {
                self.edit_row(v, iv);
            }
            _ => self.pending_item = Some((v, item, 3)),
        }
    }

    /// The suppress flag that hides a kind of item.
    fn suppress_flag(ty: &str) -> Option<&'static str> {
        match ty {
            "UMLOperation" => Some("suppressOperations"),
            "UMLReception" => Some("suppressReceptions"),
            "UMLExtensionPoint" => None,
            "UMLEnumerationLiteral" => Some("suppressLiterals"),
            "UMLTemplateParameter" => None,
            _ => Some("suppressAttributes"),
        }
    }

    /// Add an item of this kind to the box's model, show its compartment and edit it
    /// (uml-commands.js:784-823).
    fn add_item(&mut self, v: Eid, owner: Eid, ty: &str) {
        if !self.can_add_feature(owner, ty) {
            return;
        }
        let flag = Self::suppress_flag(ty);
        let new = self.change(|a| {
            if let Some(flag) = flag.filter(|f| a.repo.bool(v, f)) {
                a.repo.set(v, flag, json!(false));
            }
            a.add_feature(owner, ty)
        });
        if let Some(n) = new {
            self.focus = Some(n);
            self.edit_item(v, n);
        }
    }

    /// Keys of the inline edit on a class box or one of its items, applied after the text is committed
    /// (quickedits/uml.json; uml-commands.js:784-1024).
    pub(crate) fn item_key(&mut self, host: Eid, kind: EditKind, key: ItemKey) {
        let Some(m) = self.repo.refd(host, "model").filter(|m| self.repo.alive(*m)) else { return };
        let item = match kind {
            EditKind::Feature(f) if self.repo.alive(f) => Some(f),
            _ => None,
        };
        match (key, item) {
            (ItemKey::Add, None) => {
                // a literal on an enumeration, an extension point on a use case (quickedits/uml.json)
                let ty = match self.repo.ty(m) {
                    "UMLEnumeration" => "UMLEnumerationLiteral",
                    "UMLUseCase" => "UMLExtensionPoint",
                    _ => "UMLAttribute",
                };
                self.add_item(host, m, ty);
            }
            (ItemKey::Add, Some(f)) => {
                let (Some(owner), ty) = (self.repo.parent(f), self.repo.ty(f).to_string()) else { return };
                self.add_item(host, owner, &ty);
            }
            (ItemKey::AddOperation, _) => self.add_item(host, m, "UMLOperation"),
            (ItemKey::AddTemplateParameter, _) => self.add_item(host, m, "UMLTemplateParameter"),
            (ItemKey::Delete, Some(f)) => {
                let (Some(owner), Some(field)) = (self.repo.parent(f), self.repo.field_of(f)) else { return };
                let i = self.repo.list(owner, &field).iter().position(|x| *x == f).unwrap_or(0);
                self.change(|a| a.delete_models(&[f]));
                let rest = self.repo.list(owner, &field);
                match rest.get(i).or(rest.last()) {
                    Some(&n) => self.edit_item(host, n),
                    None => self.focus = Some(m),
                }
            }
            (ItemKey::Move(up), Some(f)) => {
                let (Some(owner), Some(field)) = (self.repo.parent(f), self.repo.field_of(f)) else { return };
                let mut list = self.repo.list(owner, &field);
                let i = list.iter().position(|x| *x == f).unwrap_or(0);
                let j = if up { i.checked_sub(1) } else { Some(i + 1).filter(|j| *j < list.len()) };
                if let Some(j) = j {
                    list.swap(i, j);
                    self.change(|a| a.repo.set_list(owner, &field, list));
                }
                self.edit_item(host, f);
            }
            (ItemKey::Nav(up), _) => {
                // template parameters, then the name, then the compartment items (uml-commands.js:896, 962)
                let templ: Vec<(Eid, Eid)> = self.info.get(&host).and_then(|i| i.templ.as_ref()).map(|t| t.rows.iter().filter_map(|r| self.repo.refd(r.iv, "model").map(|m| (r.iv, m))).collect()).unwrap_or_default();
                let seq: Vec<Option<(Eid, Eid)>> = templ.into_iter().map(Some).chain(std::iter::once(None)).chain(self.item_rows(host).into_iter().map(Some)).collect();
                let at = seq.iter().position(|x| x.map(|r| r.1) == item).unwrap_or(0);
                let next = if up { at.checked_sub(1) } else { Some(at + 1).filter(|i| *i < seq.len()) };
                match next.map(|i| seq[i]) {
                    Some(Some((iv, _))) => {
                        self.edit_row(host, iv);
                    }
                    Some(None) => self.edit_view(host, None),
                    // nothing further: stay on the same item or name
                    None => match item {
                        Some(f) => self.edit_item(host, f),
                        None => self.edit_view(host, None),
                    },
                }
            }
            _ => {}
        }
    }

    pub fn close_inline(&mut self, commit: bool) {
        let Some(e) = self.editing.take() else { return };
        // the canvas takes the keyboard back, so Enter can open the edit again (quickedit-manager.js:680-682)
        self.refocus_canvas = true;
        if !commit {
            return;
        }
        match e.kind {
            EditKind::Text(v) if self.repo.ty(v) == "UMLConstraintView" => {
                if let Some(m) = self.repo.refd(v, "model").filter(|m| self.repo.alive(*m) && self.repo.str(*m, "specification") != e.text) {
                    self.change(|a| a.repo.set(m, "specification", json!(e.text)));
                }
            }
            EditKind::Text(v) => {
                if self.repo.alive(v) && self.repo.str(v, "text") != e.text {
                    self.change(|a| a.repo.set(v, "text", json!(e.text)));
                }
            }
            EditKind::Name(m) => {
                if self.repo.alive(m) && crate::parse::name_text(&self.repo, m) != e.text {
                    self.change(|a| crate::parse::apply_name(&mut a.repo, m, &e.text));
                }
            }
            EditKind::Feature(f) => {
                if self.repo.alive(f) {
                    let ty = self.repo.ty(f).to_string();
                    self.change(|a| {
                        if ty == "UMLOperation" {
                            crate::parse::apply_operation(&mut a.repo, f, &e.text);
                        } else if ty == "UMLExtensionPoint" {
                            // "name: location"
                            let (n, loc) = e.text.split_once(':').unwrap_or((&e.text, ""));
                            a.repo.set(f, "name", json!(n.trim()));
                            a.repo.set(f, "location", json!(loc.trim()));
                        } else if ty == "UMLReception" {
                            crate::parse::apply_name(&mut a.repo, f, &e.text);
                        } else if ty == "UMLTemplateParameter" {
                            crate::parse::apply_template_parameter(&mut a.repo, f, &e.text);
                        } else {
                            crate::parse::apply_attribute(&mut a.repo, f, &e.text);
                        }
                    });
                }
            }
        }
    }

    fn inline_editor(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let Some(mut ed) = self.editing.take() else { return };
        let to = |p: P| Pos2::new(rect.min.x + ((p.x - self.scroll.x) * self.zoom) as f32, rect.min.y + ((p.y - self.scroll.y) * self.zoom) as f32);
        let pos = to(P { x: ed.rect.x, y: ed.rect.y });
        let w = ((ed.rect.w * self.zoom) as f32).max(60.0);
        let h = ((ed.rect.h * self.zoom) as f32).max(if ed.multiline { 40.0 } else { 20.0 });
        let mut commit = None;
        let mut key: Option<ItemKey> = None;
        let mut related: Option<crate::related::Related> = None;
        let mut edge_note: Option<Eid> = None;
        egui::Area::new(egui::Id::new("inline-editor")).fixed_pos(pos).order(egui::Order::Foreground).show(ui.ctx(), |ui| {
            egui::Frame::NONE.fill(Color32::WHITE).stroke(Stroke::new(1.0, c32(render::SELECT_COLOR))).show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(Color32::BLACK);
                ui.visuals_mut().extreme_bg_color = Color32::WHITE;
                // the editor uses the edited view's font size (quickedit-manager.js:365-381)
                let size = match (&ed.kind, ed.host) {
                    (EditKind::Text(v), _) => views::view_font(&self.repo, *v).1,
                    (_, Some(h)) => views::view_font(&self.repo, h).1,
                    _ => 13.0,
                };
                let font = FontId::new((size * self.zoom) as f32, diagram_family(0));
                let te = if ed.multiline {
                    egui::TextEdit::multiline(&mut ed.text).desired_width(w).desired_rows(((h / 16.0) as usize).max(2))
                } else {
                    egui::TextEdit::singleline(&mut ed.text).desired_width(w)
                };
                let te = te.font(font).frame(egui::Frame::NONE).horizontal_align(if ed.center { egui::Align::Center } else { egui::Align::Min });
                // Enter commits a note too; Shift+Enter is the new line (quickedit-manager.js:326-332)
                let mut plain_enter = false;
                if ed.multiline {
                    ui.input_mut(|i| {
                        i.events.retain(|e| match e {
                            egui::Event::Key { key: egui::Key::Enter, pressed: true, modifiers, .. } if !modifiers.shift => {
                                plain_enter = true;
                                false
                            }
                            _ => true,
                        })
                    });
                }
                // item keys on a class name or a compartment item, taken before the text field sees them
                if let (Some(_), false) = (ed.host, ed.multiline) {
                    let item = matches!(ed.kind, EditKind::Feature(_));
                    let k = ui.input_mut(|i| {
                        use egui::{Key, Modifiers};
                        let cs = Modifiers::COMMAND | Modifiers::SHIFT;
                        if i.consume_key(cs, Key::Enter) {
                            Some(ItemKey::AddOperation)
                        } else if i.consume_key(Modifiers::COMMAND, Key::Enter) {
                            Some(ItemKey::Add)
                        } else if item && i.consume_key(Modifiers::COMMAND, Key::Delete) {
                            Some(ItemKey::Delete)
                        } else if item && i.consume_key(Modifiers::COMMAND, Key::ArrowUp) {
                            Some(ItemKey::Move(true))
                        } else if item && i.consume_key(Modifiers::COMMAND, Key::ArrowDown) {
                            Some(ItemKey::Move(false))
                        } else if i.consume_key(Modifiers::NONE, Key::ArrowUp) {
                            Some(ItemKey::Nav(true))
                        } else if i.consume_key(Modifiers::NONE, Key::ArrowDown) {
                            Some(ItemKey::Nav(false))
                        } else {
                            None
                        }
                    });
                    if k.is_some() {
                        key = k;
                    }
                }
                // a visibility dropdown left of the field (quickedits/uml.json "uml.visibility")
                let vis_target = match &ed.kind {
                    EditKind::Name(m) | EditKind::Feature(m) if crate::meta::order_of(self.repo.ty(*m), "visibility").is_some() => Some(*m),
                    _ => None,
                };
                let mut picked_vis: Option<&'static str> = None;
                let r = ui
                    .horizontal(|ui| {
                        if let Some(m) = vis_target {
                            let cur = views::vis_symbol(&self.repo.str(m, "visibility"));
                            ui.menu_button(egui::RichText::new(if cur.is_empty() { "+" } else { cur }).monospace().color(Color32::WHITE), |ui| {
                                for (sym, v) in [("+", "public"), ("#", "protected"), ("-", "private"), ("~", "package")] {
                                    if ui.button(format!("{sym}  {v}")).clicked() {
                                        picked_vis = Some(v);
                                        ui.close();
                                    }
                                }
                            });
                        }
                        ui.add(te)
                    })
                    .inner;
                if let (Some(v), Some(m)) = (picked_vis, vis_target) {
                    self.change(|a| a.repo.set(m, "visibility", serde_json::json!(v)));
                    // keep the typed text in step: a leading symbol is replaced by the new one
                    if let EditKind::Feature(_) = ed.kind {
                        let (st, rest) = crate::parse::split_stereotype(&ed.text);
                        let rest = rest.trim_start_matches(['+', '-', '#', '~']).trim_start();
                        let sym = views::vis_symbol(v);
                        ed.text = match st {
                            Some(st) => format!("<<{st}>> {sym}{rest}"),
                            None => format!("{sym}{rest}"),
                        };
                    }
                    ed.fresh = true;
                }
                // an association end: aggregation, navigability and multiplicity dropdowns
                // (quickedits/uml.json "uml.association-end1/2")
                let mut picked_end: Option<(&'static str, String)> = None;
                if let EditKind::Name(end) = ed.kind {
                    if self.repo.ty(end) == "UMLAssociationEnd" {
                        ui.horizontal(|ui| {
                            let fields: [(&'static str, &str, &[&str]); 3] = [
                                ("aggregation", "Aggregation Kind", &["none", "shared", "composite"]),
                                ("navigable", "Navigability", &["unspecified", "navigable", "notNavigable"]),
                                ("multiplicity", "Multiplicity", &crate::ui::MULTIPLICITIES),
                            ];
                            for (field, hint, options) in fields {
                                let cur = self.repo.str(end, field);
                                let shown = if cur.is_empty() { "—".to_string() } else { cur.clone() };
                                ui.menu_button(egui::RichText::new(format!("{shown} ▾")).size(11.0).color(Color32::WHITE), |ui| {
                                    for o in options {
                                        if ui.selectable_label(cur == *o, *o).clicked() {
                                            picked_end = Some((field, o.to_string()));
                                            ui.close();
                                        }
                                    }
                                })
                                .response
                                .on_hover_text(hint);
                            }
                        });
                    }
                }
                if let (Some((field, value)), EditKind::Name(end)) = (picked_end.clone(), &ed.kind) {
                    let end = *end;
                    self.change(|a| a.repo.set(end, field, serde_json::json!(value)));
                    ed.fresh = true;
                }
                if ed.fresh {
                    r.request_focus();
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), r.id) {
                        let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(ed.text.chars().count()));
                        state.cursor.set_char_range(Some(all));
                        state.store(ui.ctx(), r.id);
                    }
                    ed.fresh = false;
                }
                // the "add related element" buttons of a box's name edit (quickedits/uml.json)
                if let (Some(h), EditKind::Name(m)) = (ed.host, &ed.kind) {
                    let acts = crate::related::actions(self.repo.ty(*m));
                    if self.repo.refd(h, "model") == Some(*m) && !acts.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            ui.set_max_width(w.max(220.0));
                            // a class also gets Add Template Parameter (quickedits/uml.json "uml.class")
                            if self.repo.ty(*m) == "UMLClass" {
                                let b = ui.add(egui::Button::new(egui::RichText::new("T").size(11.0).color(Color32::BLACK)).fill(Color32::from_rgb(0xee, 0xee, 0xee)).small()).on_hover_text("Add Template Parameter");
                                if b.is_pointer_button_down_on() {
                                    key = Some(ItemKey::AddTemplateParameter);
                                }
                            }
                            for a in acts {
                                let b = ui.add(egui::Button::new(egui::RichText::new(short_label(a.label)).size(11.0).color(Color32::BLACK)).fill(Color32::from_rgb(0xee, 0xee, 0xee)).small()).on_hover_text(a.label);
                                // taken on press: the text field loses focus (and the edit closes) before a release
                                if b.is_pointer_button_down_on() {
                                    related = Some(a);
                                }
                            }
                        });
                    }
                }
                // a line's name edit has Add Note too (quickedits/uml.json, uml-commands.js:684)
                if let (None, EditKind::Name(m)) = (ed.host, &ed.kind) {
                    let edge = self.diagram.and_then(|d| self.repo.list(d, "ownedViews").into_iter().find(|v| is_edge(&self.repo, *v) && self.repo.refd(*v, "model") == Some(*m)));
                    if let Some(e) = edge {
                        let b = ui.add(egui::Button::new(egui::RichText::new("+ Note").size(11.0).color(Color32::BLACK)).fill(Color32::from_rgb(0xee, 0xee, 0xee)).small()).on_hover_text("Add Note");
                        if b.is_pointer_button_down_on() {
                            edge_note = Some(e);
                        }
                    }
                }
                let (esc, enter, ctrl_enter) = ui.input(|i| (i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Enter) && i.modifiers.command));
                if key.is_some() || related.is_some() || edge_note.is_some() {
                    commit = Some(true);
                } else if esc {
                    commit = Some(false);
                } else if (!ed.multiline && enter) || (ed.multiline && (ctrl_enter || plain_enter)) {
                    commit = Some(true);
                } else if r.lost_focus() && !egui::Popup::is_any_open(ui.ctx()) && picked_vis.is_none() && picked_end.is_none() {
                    commit = Some(true);
                }
            });
        });
        let (host, kind) = (ed.host, ed.kind.clone());
        self.editing = Some(ed);
        if let Some(c) = commit {
            self.close_inline(c);
        }
        if let (Some(k), Some(h)) = (key, host) {
            self.item_key(h, kind, k);
        }
        if let (Some(a), Some(h)) = (related, host) {
            self.add_related(h, a);
        }
        if let Some(e) = edge_note.filter(|e| self.repo.alive(*e)) {
            self.add_note_to_edge(e);
        }
    }

    // ------------------------------------------------------------------ deleting and menus
    pub fn delete_selection(&mut self, from_model: bool) {
        let sel: Vec<Eid> = self.selection.iter().copied().filter(|v| self.repo.alive(*v)).collect();
        if sel.is_empty() {
            if from_model {
                if let Some(f) = self.focus.filter(|f| self.repo.ty(*f) != "Project") {
                    self.change(|a| a.delete_models(&[f]));
                }
            }
            return;
        }
        self.change(|a| {
            if from_model {
                let models: Vec<Eid> = sel.iter().filter_map(|v| a.repo.refd(*v, "model")).collect();
                let bare: Vec<Eid> = sel.iter().copied().filter(|v| !a.repo.has(*v, "model")).collect();
                if !bare.is_empty() {
                    a.remove_views(&bare);
                }
                if !models.is_empty() {
                    a.delete_models(&models);
                }
            } else {
                a.remove_views(&sel);
            }
            a.selection.clear();
            a.focus = None;
        });
    }

    fn canvas_menu(&mut self, ui: &mut egui::Ui) {
        self.last_area = crate::app::Area::Canvas;
        let Some(v) = self.ctx_target.filter(|v| self.repo.alive(*v)) else {
            if ui.button("Paste").clicked() {
                self.cmd_paste();
                ui.close();
            }
            if ui.button("Select All").clicked() {
                self.select_all();
                ui.close();
            }
            if ui.button("Fit To Window").clicked() {
                self.fit();
                ui.close();
            }
            return;
        };
        let m = self.repo.refd(v, "model");
        self.clipboard_items(ui);
        ui.separator();
        if ui.button("Edit").clicked() {
            let p = self.ctx_pos;
            self.edit_view(v, Some(p));
            ui.close();
        }
        if let Some(m) = m {
            for (label, ty) in [("Add Literal", "UMLEnumerationLiteral"), ("Add Attribute", "UMLAttribute"), ("Add Operation", "UMLOperation")] {
                if self.can_add_feature(m, ty) && ui.button(label).clicked() {
                    if let Some(c) = self.change(|a| a.add_feature(m, ty)) {
                        self.focus = Some(c);
                    }
                    ui.close();
                }
            }
        }
        if is_edge(&self.repo, v) {
            self.line_style_menu(ui);
        }
        ui.menu_button("Format", |ui| {
            if ui.button("Font...").clicked() {
                self.open_font_dialog();
                ui.close();
            }
            self.color_menus(ui);
            self.stereotype_display_menu(ui);
            ui.separator();
            self.format_toggles(ui);
            ui.separator();
            self.alignment_menu(ui);
        });
        ui.separator();
        if let Some(m) = m {
            if ui.button("Select In Explorer").clicked() {
                self.reveal(m);
                ui.close();
            }
            if ui.button("Select In Diagram").clicked() {
                self.focus = Some(m);
                self.select_in_diagram();
                ui.close();
            }
            if ui.button("Open Sub-Diagram").clicked() {
                self.focus = Some(m);
                self.open_sub_diagram();
                ui.close();
            }
            if ui.button("Tag Editor...").clicked() {
                self.focus = Some(m);
                self.open_tag_editor();
                ui.close();
            }
        }
        if ui.button("Delete").clicked() {
            self.delete_selection(false);
            ui.close();
        }
        if m.is_some() && ui.button("Delete from Model").clicked() {
            self.delete_selection(true);
            ui.close();
        }
    }

    /// A double-click on empty canvas adds a text "Text", 100 wide, there and edits it (app-context.js:1397-1402).
    pub fn create_text_at(&mut self, p: P) {
        let (_, h) = views::default_size("Text");
        let r = R { x: self.snap_v(p.x).max(0.0), y: self.snap_v(p.y).max(0.0), w: 100.0, h };
        let t = self.change(|a| {
            let t = a.create_node("Text", r)?;
            a.repo.set(t, "text", json!("Text"));
            Some(t)
        });
        if let Some(t) = t {
            self.select(vec![t]);
            self.edit_view(t, None);
        }
    }

    /// Tab: select the top-level view after the first selected one in ownedViews order, wrapping
    /// around, or the first view when nothing is selected (diagram-manager.js:726-754).
    pub fn select_next_view(&mut self) {
        let Some(d) = self.diagram else { return };
        let list = self.repo.list(d, "ownedViews");
        if list.is_empty() {
            return;
        }
        let next = match self.selection.first().and_then(|v| list.iter().position(|x| x == v)) {
            Some(i) => list[(i + 1) % list.len()],
            None => list[0],
        };
        self.select(vec![next]);
    }

    /// Select All: every visible, enabled top-level view (core.js:4180-4196).
    pub fn select_all(&mut self) {
        if let Some(d) = self.diagram {
            let all = self.repo.list(d, "ownedViews").into_iter().filter(|v| !matches!(self.repo.raw(*v, "visible"), Some(serde_json::Value::Bool(false))) && !matches!(self.repo.raw(*v, "enabled"), Some(serde_json::Value::Bool(false)))).collect();
            self.select(all);
        }
    }

    /// Size of the visible diagram area, in diagram units at the current zoom.
    fn view_size(&self) -> Option<(f64, f64)> {
        let r = self.canvas_rect?;
        Some((r.width() as f64 / self.zoom, r.height() as f64 / self.zoom))
    }

    /// Scroll so that diagram point `c` is at the centre of the canvas, never into negative space
    /// (diagram-manager.js:340-348, diagram-editor.js:2380-2383).
    pub fn centre_on(&mut self, c: P) {
        let Some((w, h)) = self.view_size() else { return };
        self.scroll = P { x: (c.x - w / 2.0).max(0.0), y: (c.y - h / 2.0).max(0.0) };
    }

    /// Zoom In, Zoom Out and Actual Size: change the zoom and keep the point at the screen centre
    /// where it was (default-commands.js:1063-1084).
    pub fn zoom_keep_centre(&mut self, z: f64) {
        let Some((w, h)) = self.view_size() else {
            self.set_zoom(z);
            return;
        };
        let c = P { x: self.scroll.x + w / 2.0, y: self.scroll.y + h / 2.0 };
        self.set_zoom(z);
        self.centre_on(c);
    }

    /// Fit to Window: the largest zoom (at most 1.0) showing the whole diagram, centred on its
    /// bounding box (default-commands.js:1086-1101).
    pub fn fit(&mut self) {
        let (Some(d), Some(rect)) = (self.diagram, self.canvas_rect) else { return };
        match render::bounds(&self.repo, d) {
            None => self.set_zoom(1.0),
            Some(b) => {
                let z = (rect.width() as f64 / b.w.max(1.0)).min(rect.height() as f64 / b.h.max(1.0));
                // whole tenths, rounded down so the diagram still fits
                self.set_zoom(1f64.min(((z * 10.0).floor() / 10.0).max(crate::app::ZOOM_MIN)));
                self.centre_on(P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 });
            }
        }
    }

    pub fn reveal(&mut self, e: Eid) {
        let mut p = self.repo.parent(e);
        while let Some(x) = p {
            self.expanded.insert(x);
            p = self.repo.parent(x);
        }
        self.focus = Some(e);
        self.reveal_pending = Some(e);
    }
}
