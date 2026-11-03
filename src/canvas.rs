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
fn contains(outer: &R, inner: &R) -> bool {
    inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.w <= outer.x + outer.w && inner.y + inner.h <= outer.y + outer.h
}

impl App {
    fn hit_edge(&self, e: Eid, p: P) -> bool {
        let pts = points_of(&self.repo, e);
        pts.windows(2).any(|w| dist_to_seg(p, w[0], w[1]) <= 5.0 / self.zoom)
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
            } else if is_node(&self.repo, v) && in_rect(p, &node_rect(&self.repo, v)) {
                return Some(v);
            }
        }
        None
    }

    fn hit_handle(&self, p: P) -> Option<usize> {
        if self.selection.len() != 1 {
            return None;
        }
        let v = self.selection[0];
        if is_edge(&self.repo, v) {
            return None;
        }
        let hs = 6.0 / self.zoom;
        render::handle_points(&node_rect(&self.repo, v)).iter().position(|h| (p.x - h.x).abs() <= hs && (p.y - h.y).abs() <= hs)
    }

    fn snap_v(&self, n: f64) -> f64 {
        if self.snap { (n / render::GRID_SIZE).round() * render::GRID_SIZE } else { n.round() }
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
        self.handle_input(ui, &resp, rect, d);
        if !self.repo.alive(d) || self.diagram != Some(d) {
            return;
        }

        self.info = views::layout_diagram(&mut self.repo, &measure, d);
        painter.rect_filled(rect, 0.0, Color32::WHITE);
        let mut c = ScreenCanvas { painter: &painter, origin: rect.min, scroll: self.scroll, zoom: self.zoom, measure: EguiMeasure { ctx: ctx.clone() } };
        if self.show_grid {
            let grid = Stroke::new(1.0, c32(render::parse_color(render::GRID_COLOR_HEX, render::GRID_COLOR)));
            let step = render::GRID_STEP;
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
                for p in render::handle_points(&b) {
                    handle(c, p);
                }
            }
        }
    }

    fn draw_drag(&self, c: &mut ScreenCanvas) {
        let sel = c32(render::SELECT_COLOR);
        match &self.drag {
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
            _ => {}
        }
    }

    fn draw_minimap(&self, painter: &Painter, rect: Rect, d: Eid, ctx: &egui::Context) {
        let m = Self::mini_rect(rect);
        if m.left() < rect.left() + 10.0 || m.top() < rect.top() + 10.0 {
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

        // drop from the explorer
        if let Some(eid) = resp.dnd_release_payload::<Eid>() {
            if let Some(pos) = latest {
                let p = self.to_diagram(origin, pos);
                let m = *eid;
                if self.repo.alive(m) && views::node_kind(self.repo.ty(m)).is_some() {
                    let (w, h) = views::default_size(self.repo.ty(m));
                    let r = R { x: self.snap_v(p.x), y: self.snap_v(p.y), w, h };
                    if let Some(v) = self.change(|a| a.create_view_of(m, r)) {
                        self.select(vec![v]);
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
                        _ => egui::CursorIcon::Default,
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
                }
            }
        }

        if resp.secondary_clicked() {
            if let Some(pos) = pointer.interact_pos() {
                let p = self.to_diagram(origin, pos);
                self.ctx_pos = p;
                self.ctx_target = self.hit_view(p, false);
                if let Some(v) = self.ctx_target {
                    if !self.selection.contains(&v) {
                        self.select(vec![v]);
                    }
                }
            }
        }
        resp.context_menu(|ui| self.canvas_menu(ui));

        // keyboard on the canvas
        if resp.has_focus() && self.editing.is_none() {
            let (del, esc, enter, arrows) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::Delete) && !i.modifiers.command,
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
            if enter && self.selection.len() == 1 {
                let v = self.selection[0];
                self.edit_view(v, None);
            }
            let (l, r, u, dn, shift) = arrows;
            if (l || r || u || dn) && !self.selection.is_empty() {
                let step = if shift { render::GRID_SIZE } else { 1.0 };
                let dx = if l { -step } else if r { step } else { 0.0 };
                let dy = if u { -step } else if dn { step } else { 0.0 };
                let sel = self.selection.clone();
                self.change(|a| {
                    for v in sel {
                        if !is_edge(&a.repo, v) {
                            let (x, y) = (a.repo.f(v, "left"), a.repo.f(v, "top"));
                            a.repo.set(v, "left", crate::model::num(x + dx));
                            a.repo.set(v, "top", crate::model::num(y + dy));
                        }
                    }
                });
            }
        }
        let _ = d;
    }

    fn on_press(&mut self, rect: Rect, pos: Pos2, mods: egui::Modifiers) {
        let p = self.to_diagram(rect.min, pos);
        if Self::mini_rect(rect).contains(pos) && rect.width() > MINI_W + 40.0 {
            self.drag = Some(Drag::Mini);
            self.mini_scroll(rect, pos);
            return;
        }
        if let Some(tool) = self.tool {
            if !tool.line {
                self.drag = Some(Drag::CreateNode { start: p, now: p });
            } else if let Some(tail) = self.hit_view(p, tool.kind != "NoteLink") {
                self.drag = Some(Drag::CreateEdge { start: p, now: p, tail });
            }
            return;
        }
        if let Some(h) = self.hit_handle(p) {
            let v = self.selection[0];
            self.drag = Some(Drag::Resize { handle: h, start: p, orig: node_rect(&self.repo, v), view: v, changed: false });
            return;
        }
        // bend points of a selected line
        if self.selection.len() == 1 && is_edge(&self.repo, self.selection[0]) {
            let e = self.selection[0];
            let pts = points_of(&self.repo, e);
            for i in 1..pts.len().saturating_sub(1) {
                if ((pts[i].x - p.x).powi(2) + (pts[i].y - p.y).powi(2)).sqrt() <= 6.0 / self.zoom {
                    self.drag = Some(Drag::Point { edge: e, index: i, insert: None, changed: false });
                    return;
                }
            }
            for i in 0..pts.len().saturating_sub(1) {
                if dist_to_seg(p, pts[i], pts[i + 1]) <= 5.0 / self.zoom {
                    self.drag = Some(Drag::Point { edge: e, index: i + 1, insert: Some(p), changed: false });
                    return;
                }
            }
        }
        let add = mods.shift || mods.command;
        match self.hit_view(p, false) {
            None => {
                if !add {
                    self.select(vec![]);
                }
                self.drag = Some(Drag::Rubber { start: p, now: p, add, base: self.selection.clone() });
            }
            Some(v) => {
                if add {
                    let mut sel = self.selection.clone();
                    if let Some(i) = sel.iter().position(|x| *x == v) {
                        sel.remove(i);
                    } else {
                        sel.push(v);
                    }
                    self.select(sel);
                } else if !self.selection.contains(&v) {
                    self.select(vec![v]);
                } else {
                    self.focus = Some(self.repo.refd(v, "model").unwrap_or(v));
                }
                self.start_move(p);
            }
        }
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
            if let Some(serde_json::Value::Array(cs)) = self.repo.raw(v, "containedViews") {
                for c in cs.clone() {
                    if let Some(cv) = self.repo.by_ref(&c) {
                        push(&mut nodes, cv);
                    }
                }
            }
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
            }
        }
        self.drag = Some(Drag::Move { start: p, orig, edges, lead: nodes.first().copied(), changed: false });
    }

    fn on_move(&mut self, rect: Rect, pos: Pos2) {
        let p = self.to_diagram(rect.min, pos);
        let Some(mut drag) = self.drag.take() else { return };
        match &mut drag {
            Drag::Pan { start, scroll } => {
                self.scroll.x = (scroll.x - (pos.x - start.x) as f64 / self.zoom).max(0.0);
                self.scroll.y = (scroll.y - (pos.y - start.y) as f64 / self.zoom).max(0.0);
            }
            Drag::Mini => self.mini_scroll(rect, pos),
            Drag::Rubber { start, now, add, base } => {
                *now = p;
                let r = rect_of(*start, p);
                let mut inside = vec![];
                if let Some(d) = self.diagram {
                    for v in self.repo.list(d, "ownedViews") {
                        let hit = if is_edge(&self.repo, v) {
                            let pts = points_of(&self.repo, v);
                            !pts.is_empty() && pts.iter().all(|q| in_rect(*q, &r))
                        } else {
                            is_node(&self.repo, v) && contains(&r, &node_rect(&self.repo, v))
                        };
                        if hit {
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
                let nodes_only = self.tool.map(|t| t.kind != "NoteLink").unwrap_or(true);
                self.hover = self.hit_view(p, nodes_only);
            }
            Drag::Move { start, orig, edges, lead, changed } => {
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
                        dx = self.snap_v(o.x + dx) - o.x;
                        dy = self.snap_v(o.y + dy) - o.y;
                    }
                }
                for (n, o) in orig.iter() {
                    self.repo.set(*n, "left", crate::model::num(o.x + dx));
                    self.repo.set(*n, "top", crate::model::num(o.y + dy));
                }
                for (e, pts) in edges.iter() {
                    if pts.len() > 2 {
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
                    x1 = self.snap_v(o.x + dx);
                }
                if [2, 3, 4].contains(&h) {
                    x2 = self.snap_v(o.x + o.w + dx);
                }
                if [0, 1, 2].contains(&h) {
                    y1 = self.snap_v(o.y + dy);
                }
                if [4, 5, 6].contains(&h) {
                    y2 = self.snap_v(o.y + o.h + dy);
                }
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
                    pts[*index] = P { x: self.snap_v(p.x), y: self.snap_v(p.y) };
                    self.repo.set(*edge, "points", json!(views::format_points(&pts)));
                }
            }
        }
        self.drag = Some(drag);
    }

    fn on_release(&mut self, origin: Pos2, pos: Pos2) {
        let Some(drag) = self.drag.take() else { return };
        self.hover = None;
        let p = self.to_diagram(origin, pos);
        match drag {
            Drag::Rubber { .. } => {
                let s = self.selection.clone();
                self.select(s);
            }
            Drag::CreateNode { start, .. } => {
                let Some(tool) = self.tool else { return };
                let mut r = rect_of(start, p);
                if r.w < 10.0 || r.h < 10.0 {
                    let (w, h) = views::default_size(tool.kind);
                    r = R { x: start.x, y: start.y, w, h };
                }
                let r = R { x: self.snap_v(r.x), y: self.snap_v(r.y), w: r.w.round(), h: r.h.round() };
                let view = self.change(|a| {
                    let v = a.create_node(tool.kind, r);
                    if let (Some(v), Some(d)) = (v, a.diagram) {
                        let t = a.repo.ty(v);
                        if t == "UMLUseCaseSubjectView" || t == "UMLPackageView" {
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
                    self.pending_edit = Some(v);
                }
            }
            Drag::CreateEdge { tail, .. } => {
                let Some(tool) = self.tool else { return };
                if let Some(head) = self.hit_view(p, tool.kind != "NoteLink") {
                    if self.can_connect(&tool, tail, head) {
                        if let Some(v) = self.change(|a| a.create_edge(&tool, tail, head)) {
                            self.select(vec![v]);
                        }
                    }
                }
                if !self.tool_sticky {
                    self.tool = None;
                }
            }
            Drag::Move { changed, .. } | Drag::Resize { changed, .. } | Drag::Point { changed, .. } => {
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
        if t == "UMLNoteView" || t == "UMLTextView" || t == "UMLCustomTextView" {
            let text = self.repo.str(v, "text");
            self.editing = Some(Editing { kind: EditKind::Text(v), text, rect: node_rect(&self.repo, v), multiline: true, center: false, fresh: true });
            return;
        }
        let Some(m) = self.repo.refd(v, "model") else { return };
        if is_edge(&self.repo, v) {
            let pts = points_of(&self.repo, v);
            let mid = pts.get(pts.len() / 2).copied().unwrap_or_default();
            let lv = self.repo.refd(v, "nameLabel");
            let rect = match lv {
                Some(l) if !self.repo.str(l, "text").is_empty() => R { x: self.repo.f(l, "left") - 20.0, y: self.repo.f(l, "top") - 3.0, w: self.repo.f(l, "width") + 40.0, h: 19.0 },
                _ => R { x: mid.x - 50.0, y: mid.y - 10.0, w: 100.0, h: 19.0 },
            };
            self.editing = Some(Editing { kind: EditKind::Name(m), text: self.repo.name(m), rect, multiline: false, center: true, fresh: true });
            return;
        }
        if let Some(iv) = p.and_then(|p| self.row_at(v, p)) {
            if let Some(item) = self.repo.refd(iv, "model") {
                let b = node_rect(&self.repo, iv);
                self.editing = Some(Editing {
                    kind: EditKind::Feature(item),
                    text: self.repo.str(iv, "text"),
                    rect: R { x: b.x - 2.0, y: b.y - 3.0, w: b.w.max(120.0) + 4.0, h: 19.0 },
                    multiline: false,
                    center: false,
                    fresh: true,
                });
                return;
            }
        }
        let nl = self.repo.refd(v, "nameCompartment").and_then(|nc| self.repo.refd(nc, "nameLabel"));
        let rect = match nl {
            Some(l) => R { x: self.repo.f(l, "left") - 2.0, y: self.repo.f(l, "top") - 3.0, w: self.repo.f(l, "width").max(80.0) + 4.0, h: 19.0 },
            None => node_rect(&self.repo, v),
        };
        self.editing = Some(Editing { kind: EditKind::Name(m), text: self.repo.name(m), rect, multiline: false, center: true, fresh: true });
    }

    pub fn close_inline(&mut self, commit: bool) {
        let Some(e) = self.editing.take() else { return };
        if !commit {
            return;
        }
        match e.kind {
            EditKind::Text(v) => {
                if self.repo.alive(v) && self.repo.str(v, "text") != e.text {
                    self.change(|a| a.repo.set(v, "text", json!(e.text)));
                }
            }
            EditKind::Name(m) => {
                if self.repo.alive(m) && self.repo.name(m) != e.text {
                    self.change(|a| a.repo.set(m, "name", json!(e.text)));
                }
            }
            EditKind::Feature(f) => {
                if self.repo.alive(f) {
                    let ty = self.repo.ty(f).to_string();
                    self.change(|a| {
                        if ty == "UMLOperation" {
                            crate::parse::apply_operation(&mut a.repo, f, &e.text);
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
        egui::Area::new(egui::Id::new("inline-editor")).fixed_pos(pos).order(egui::Order::Foreground).show(ui.ctx(), |ui| {
            egui::Frame::NONE.fill(Color32::WHITE).stroke(Stroke::new(1.0, c32(render::SELECT_COLOR))).show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(Color32::BLACK);
                ui.visuals_mut().extreme_bg_color = Color32::WHITE;
                let font = FontId::new((13.0 * self.zoom) as f32, diagram_family(0));
                let te = if ed.multiline {
                    egui::TextEdit::multiline(&mut ed.text).desired_width(w).desired_rows(((h / 16.0) as usize).max(2))
                } else {
                    egui::TextEdit::singleline(&mut ed.text).desired_width(w)
                };
                let te = te.font(font).frame(egui::Frame::NONE).horizontal_align(if ed.center { egui::Align::Center } else { egui::Align::Min });
                let r = ui.add(te);
                if ed.fresh {
                    r.request_focus();
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), r.id) {
                        let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(ed.text.chars().count()));
                        state.cursor.set_char_range(Some(all));
                        state.store(ui.ctx(), r.id);
                    }
                    ed.fresh = false;
                }
                let (esc, enter, ctrl_enter) = ui.input(|i| (i.key_pressed(egui::Key::Escape), i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Enter) && i.modifiers.command));
                if esc {
                    commit = Some(false);
                } else if (!ed.multiline && enter) || (ed.multiline && ctrl_enter) {
                    commit = Some(true);
                } else if r.lost_focus() {
                    commit = Some(true);
                }
            });
        });
        self.editing = Some(ed);
        if let Some(c) = commit {
            self.close_inline(c);
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
        let Some(v) = self.ctx_target.filter(|v| self.repo.alive(*v)) else {
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
            ui.menu_button("Line Style", |ui| {
                if ui.button("Rectilinear").clicked() {
                    self.set_line_style(0);
                    ui.close();
                }
                if ui.button("Oblique").clicked() {
                    self.set_line_style(1);
                    ui.close();
                }
            });
        }
        ui.menu_button("Format", |ui| self.alignment_menu(ui));
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

    pub fn select_all(&mut self) {
        if let Some(d) = self.diagram {
            let all = self.repo.list(d, "ownedViews");
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
