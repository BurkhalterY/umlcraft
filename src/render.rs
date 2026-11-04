//! Draws a diagram onto a canvas (the screen or an SVG file). Shapes and sizes: MECHANICS.md "Drawing".

use crate::model::{Eid, Repo};
use crate::views::{self, is_edge, is_node, node_rect, points_of, Info, Measure, P, R};
use std::collections::HashMap;

pub const SHADOW_OFFSET: f64 = 7.0; // elements.js:57
pub const SHADOW_ALPHA: f64 = 0.2; // elements.js:58
pub const SHADOW_COLOR: Rgba = Rgba(0xC0, 0xC0, 0xC0, 255);
// grid size 8 and colour #e0e0e0 are the defaults of diagramEditor.gridSize / gridColor (default.json:117-141),
// read through the preferences (prefs.rs); lines are drawn every 2 x grid size (screenshot)
pub const GRID_COLOR: Rgba = Rgba(0xe0, 0xe0, 0xe0, 255);
pub const SELECT_COLOR_HEX: &str = "#4f99ff"; // graphics.js:2723
pub const SELECT_COLOR: Rgba = Rgba(0x4f, 0x99, 0xff, 255); // = SELECT_COLOR_HEX
pub const HANDLE_SIZE: f64 = 8.0;
pub const ARROW_LENGTH: f64 = 11.0; // drawLineEnd c1, core.js:2748
pub const TRIANGLE_LENGTH: f64 = 22.0; // drawLineEnd c2, core.js:2749
pub const ARROW_HALF_ANGLE_DEG: f64 = 22.5; // pi/8, core.js:2738
pub const DASH: f64 = 3.0; // lineMode DOT

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);
pub const BLACK: Rgba = Rgba(0, 0, 0, 255);
pub const WHITE: Rgba = Rgba(255, 255, 255, 255);

/// A colour as StarUML writes it: `#rrggbb`, `#rrggbbaa` (the Color dialog's opacity), `#rgb`, or
/// CSS `rgb(r, g, b)` / `rgba(r, g, b, a)` with `a` from 0 to 1.
pub fn parse_color(s: &str, def: Rgba) -> Rgba {
    let t = s.trim();
    if let Some(h) = t.strip_prefix('#') {
        let n = |i: usize, w: usize| h.get(i..i + w).and_then(|x| u8::from_str_radix(x, 16).ok());
        return match h.len() {
            6 => (|| Some(Rgba(n(0, 2)?, n(2, 2)?, n(4, 2)?, 255)))(),
            8 => (|| Some(Rgba(n(0, 2)?, n(2, 2)?, n(4, 2)?, n(6, 2)?)))(),
            3 => (|| Some(Rgba(n(0, 1)? * 17, n(1, 1)? * 17, n(2, 1)? * 17, 255)))(),
            _ => None,
        }
        .unwrap_or(def);
    }
    let low = t.to_ascii_lowercase();
    if let Some(args) = low.strip_prefix("rgba(").or_else(|| low.strip_prefix("rgb(")).and_then(|x| x.strip_suffix(')')) {
        let v: Vec<f64> = args.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if v.len() == 3 || v.len() == 4 {
            let c = |x: f64| x.round().clamp(0.0, 255.0) as u8;
            return Rgba(c(v[0]), c(v[1]), c(v[2]), v.get(3).map(|a| c(a * 255.0)).unwrap_or(255));
        }
    }
    def
}

/// What the drawing code needs from a surface. Coordinates are diagram coordinates.
pub trait Canvas: Measure {
    fn polyline(&mut self, pts: &[P], color: Rgba, dashed: bool);
    fn polygon(&mut self, pts: &[P], fill: Option<Rgba>, stroke: Option<Rgba>);
    fn ellipse(&mut self, c: P, rx: f64, ry: f64, fill: Option<Rgba>, stroke: Option<Rgba>);
    /// align: 0 left, 1 right, 2 centre; text is centred vertically in the box
    fn text(&mut self, t: &str, b: R, align: i64, style: u8, size: f64, color: Rgba, underline: bool);
    fn rect(&mut self, r: R, fill: Option<Rgba>, stroke: Option<Rgba>) {
        let pts = [P { x: r.x, y: r.y }, P { x: r.x + r.w, y: r.y }, P { x: r.x + r.w, y: r.y + r.h }, P { x: r.x, y: r.y + r.h }];
        self.polygon(&pts, fill, stroke);
    }
}

fn color(r: &Repo, v: Eid, key: &str, def: Rgba) -> Rgba {
    parse_color(&r.str(v, key), def)
}

fn font_style(font: &str) -> u8 {
    font.split(';').nth(2).and_then(|s| s.parse().ok()).unwrap_or(0)
}
fn font_size(font: &str) -> f64 {
    font.split(';').nth(1).and_then(|s| s.parse().ok()).unwrap_or(13.0)
}

fn shown(r: &Repo, v: Eid) -> bool {
    !matches!(r.raw(v, "visible"), Some(serde_json::Value::Bool(false)) | Some(serde_json::Value::Null))
}

fn draw_label(c: &mut dyn Canvas, r: &Repo, lv: Eid, fc: Rgba) {
    if !shown(r, lv) {
        return;
    }
    let t = r.str(lv, "text");
    if t.is_empty() {
        return;
    }
    let font = r.str(lv, "font");
    let b = R { x: r.f(lv, "left"), y: r.f(lv, "top"), w: r.f(lv, "width"), h: r.f(lv, "height").max(13.0) };
    let align = r.val(lv, "horizontalAlignment").and_then(|v| v.as_i64()).unwrap_or(2);
    let (style, size) = (font_style(&font), font_size(&font));
    if r.bool(lv, "wordWrap") {
        // one line per row of the label, broken where the layout broke it
        let face = font.split(';').next().filter(|f| !f.is_empty()).unwrap_or("Arial").to_string();
        let lines = views::wrap_text(&views::Scaled { ms: &*c, face, size }, &t, b.w, style);
        for (i, l) in lines.iter().enumerate() {
            c.text(l, R { y: b.y + i as f64 * size, h: size, ..b }, align, style, size, fc, r.bool(lv, "underline"));
        }
        return;
    }
    c.text(&t, b, align, style, size, fc, r.bool(lv, "underline"));
}

fn draw_name(c: &mut dyn Canvas, r: &Repo, v: Eid) {
    let Some(nc) = r.refd(v, "nameCompartment") else { return };
    let fc = color(r, v, "fontColor", BLACK);
    for f in ["stereotypeLabel", "nameLabel", "namespaceLabel", "propertyLabel"] {
        if let Some(lv) = r.refd(nc, f) {
            draw_label(c, r, lv, fc);
        }
    }
}

fn shadow_rect(c: &mut dyn Canvas, r: &Repo, v: Eid, b: R) {
    if r.bool(v, "showShadow") {
        let s = Rgba(SHADOW_COLOR.0, SHADOW_COLOR.1, SHADOW_COLOR.2, (255.0 * SHADOW_ALPHA) as u8);
        c.rect(R { x: b.x + SHADOW_OFFSET, y: b.y + SHADOW_OFFSET, ..b }, Some(s), None);
    }
}
fn shadow_ellipse(c: &mut dyn Canvas, r: &Repo, v: Eid, ctr: P, rx: f64, ry: f64) {
    if r.bool(v, "showShadow") {
        let s = Rgba(SHADOW_COLOR.0, SHADOW_COLOR.1, SHADOW_COLOR.2, (255.0 * SHADOW_ALPHA) as u8);
        c.ellipse(P { x: ctr.x + SHADOW_OFFSET, y: ctr.y + SHADOW_OFFSET }, rx, ry, Some(s), None);
    }
}

pub fn wrap_lines(c: &dyn Measure, text: &str, width: f64) -> Vec<String> {
    views::wrap_text(c, text, width, 0)
}

/// A note or text in the view's font size, word-wrapped unless its wordWrap is off.
fn draw_wrapped(c: &mut dyn Canvas, text: &str, b: R, fc: Rgba, size: f64, wrap: bool, align: (i64, i64), overflow: f64) {
    let lh = (size * 1.15).round();
    let scaled = views::Scaled { ms: &*c, face: "Arial".into(), size };
    let lines = wrap_lines(&scaled, text, if wrap { b.w } else { f64::INFINITY });
    // horzAlign 0 left, 2 centre, 1 right; vertAlign 3 top, 5 middle, 4 bottom (style-editor-view.js:477-483)
    let total = lines.len() as f64 * lh;
    let mut y = match align.1 {
        5 => b.y + ((b.h - total) / 2.0).max(0.0),
        4 => b.y + (b.h - total).max(0.0),
        _ => b.y,
    };
    for l in lines {
        if y + lh > b.y + b.h + overflow + 1.0 {
            break;
        }
        c.text(&l, R { x: b.x, y, w: b.w, h: lh }, align.0, 0, size, fc, false);
        y += lh;
    }
}

/// A text view's horizontal and vertical alignment.
fn text_align(r: &Repo, v: Eid) -> (i64, i64) {
    let g = |k: &str, d: i64| r.val(v, k).and_then(|x| x.as_i64()).unwrap_or(d);
    (g("horzAlign", 0), g("vertAlign", 3))
}

fn draw_node(c: &mut dyn Canvas, r: &Repo, v: Eid, info: Option<&Info>) {
    let b = node_rect(r, v);
    let line = color(r, v, "lineColor", BLACK);
    let fill = color(r, v, "fillColor", WHITE);
    let fc = color(r, v, "fontColor", BLACK);
    let ty = r.ty(v).to_string();
    let shape = info.map(|i| i.shape).unwrap_or("");
    match shape {
        "interface" => {
            let ih = b.h - views::NAME_MIN_H;
            let d = b.w.min(ih);
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + ih / 2.0 };
            shadow_ellipse(c, r, v, ctr, d / 2.0, d / 2.0);
            c.ellipse(ctr, d / 2.0, d / 2.0, Some(fill), Some(line));
            draw_name(c, r, v);
            return;
        }
        "actor" => {
            let ih = b.h - views::NAME_MIN_H;
            let iw = b.w.min(ih * views::ACTOR_ICON_W / views::ACTOR_ICON_H);
            let (ix, iy) = (b.x + (b.w - iw) / 2.0, b.y);
            let head_h = ih * 0.34 - 1.0;
            c.ellipse(P { x: ix + iw / 2.0, y: iy + 1.0 + head_h / 2.0 }, iw * 0.34, head_h / 2.0, Some(fill), Some(line));
            let p = |fx: f64, fy: f64| P { x: ix + iw * fx, y: iy + ih * fy };
            c.polyline(&[p(0.5, 0.34), p(0.5, 0.66)], line, false);
            c.polyline(&[p(0.14, 0.46), p(0.86, 0.46)], line, false);
            c.polyline(&[p(0.0, 1.0), p(0.5, 0.66), p(1.0, 1.0)], line, false);
            draw_name(c, r, v);
            return;
        }
        "artifact" => {
            // a page with a folded corner above the name
            let ih = b.h - views::NAME_MIN_H;
            let pw = (ih * views::ARTIFACT_ICON_W / views::ARTIFACT_ICON_H).min(b.w);
            let (px, py, f) = (b.x + (b.w - pw) / 2.0, b.y, pw * 0.3);
            let page = [P { x: px, y: py }, P { x: px + pw - f, y: py }, P { x: px + pw, y: py + f }, P { x: px + pw, y: py + ih }, P { x: px, y: py + ih }];
            if r.bool(v, "showShadow") {
                let s = Rgba(SHADOW_COLOR.0, SHADOW_COLOR.1, SHADOW_COLOR.2, (255.0 * SHADOW_ALPHA) as u8);
                let sp: Vec<P> = page.iter().map(|p| P { x: p.x + SHADOW_OFFSET, y: p.y + SHADOW_OFFSET }).collect();
                c.polygon(&sp, Some(s), None);
            }
            c.polygon(&page, Some(fill), Some(line));
            c.polyline(&[P { x: px + pw - f, y: py }, P { x: px + pw - f, y: py + f }, P { x: px + pw, y: py + f }], line, false);
            draw_name(c, r, v);
            return;
        }
        "boundary" | "entity" | "control" => {
            // the UML robustness icons above the name (estimated proportions)
            let ih = b.h - views::NAME_MIN_H;
            let rr = (ih / 2.0 - 1.0).min(b.w / 2.0 - 1.0).max(4.0);
            let ctr = P { x: b.x + b.w / 2.0 + if info.map(|i| i.shape) == Some("boundary") { rr * 0.35 } else { 0.0 }, y: b.y + ih / 2.0 };
            shadow_ellipse(c, r, v, ctr, rr, rr);
            c.ellipse(ctr, rr, rr, Some(fill), Some(line));
            match info.map(|i| i.shape).unwrap_or("") {
                "boundary" => {
                    let x0 = ctr.x - rr * 1.7;
                    c.polyline(&[P { x: x0, y: ctr.y - rr }, P { x: x0, y: ctr.y + rr }], line, false);
                    c.polyline(&[P { x: x0, y: ctr.y }, P { x: ctr.x - rr, y: ctr.y }], line, false);
                }
                "entity" => c.polyline(&[P { x: ctr.x - rr, y: ctr.y + rr }, P { x: ctr.x + rr, y: ctr.y + rr }], line, false),
                _ => {
                    let (tx, ty) = (ctr.x, ctr.y - rr);
                    let a = rr * 0.35;
                    c.polyline(&[P { x: tx + a, y: ty - a }, P { x: tx, y: ty }, P { x: tx + a, y: ty + a }], line, false);
                }
            }
            draw_name(c, r, v);
            return;
        }
        "usecase" => {
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            shadow_ellipse(c, r, v, ctr, b.w / 2.0, b.h / 2.0);
            c.ellipse(ctr, b.w / 2.0, b.h / 2.0, Some(fill), Some(line));
            draw_name(c, r, v);
            // extension points: a chord under the name, then one row each
            if let Some(i) = info {
                let (rx, ry) = (b.w / 2.0, b.h / 2.0);
                for comp in &i.compartments {
                    let k = ((comp.y - ctr.y) / ry).clamp(-1.0, 1.0);
                    let dx = rx * (1.0 - k * k).sqrt();
                    c.polyline(&[P { x: ctr.x - dx, y: comp.y }, P { x: ctr.x + dx, y: comp.y }], line, false);
                    for row in &comp.rows {
                        draw_label(c, r, row.iv, fc);
                    }
                }
            }
            return;
        }
        "package" => {
            let tab_w = (b.w * views::PKG_TAB_RATIO).round();
            let th = views::PKG_TAB_H;
            let body = R { x: b.x, y: b.y + th - 1.0, w: b.w, h: b.h - th + 1.0 };
            shadow_rect(c, r, v, body);
            c.rect(R { x: b.x, y: b.y, w: tab_w, h: th }, Some(fill), Some(line));
            c.rect(body, Some(fill), Some(line));
            draw_name(c, r, v);
            return;
        }
        _ => {}
    }
    if ty == "UMLNoteView" || ty == "UMLCustomNoteView" {
        let f = views::NOTE_FOLD;
        let pts = [
            P { x: b.x, y: b.y }, P { x: b.x + b.w - f, y: b.y }, P { x: b.x + b.w, y: b.y + f },
            P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h },
        ];
        if r.bool(v, "showShadow") {
            let s = Rgba(SHADOW_COLOR.0, SHADOW_COLOR.1, SHADOW_COLOR.2, (255.0 * SHADOW_ALPHA) as u8);
            let sp: Vec<P> = pts.iter().map(|p| P { x: p.x + SHADOW_OFFSET, y: p.y + SHADOW_OFFSET }).collect();
            c.polygon(&sp, Some(s), None);
        }
        c.polygon(&pts, Some(fill), Some(line));
        c.polyline(&[P { x: b.x + b.w - f, y: b.y }, P { x: b.x + b.w - f, y: b.y + f }, P { x: b.x + b.w, y: b.y + f }], line, false);
        draw_wrapped(c, &r.str(v, "text"), R { x: b.x + 5.0, y: b.y + 5.0, w: b.w - 20.0, h: b.h - 10.0 }, fc, views::view_font(r, v).1, r.bool(v, "wordWrap"), text_align(r, v), 0.0);
        return;
    }
    if ty == "UMLTextBoxView" {
        // a text with a border and a fill
        shadow_rect(c, r, v, b);
        c.rect(b, Some(fill), Some(line));
        draw_wrapped(c, &r.str(v, "text"), R { x: b.x + 5.0, y: b.y + 5.0, w: (b.w - 10.0).max(10.0), h: (b.h - 10.0).max(10.0) }, fc, views::view_font(r, v).1, r.bool(v, "wordWrap"), text_align(r, v), 0.0);
        return;
    }
    if ty == "UMLTextView" || ty == "UMLCustomTextView" || ty == "UMLConstraintView" {
        draw_wrapped(c, &r.str(v, "text"), R { x: b.x, y: b.y, w: b.w.max(10.0), h: b.h.max(15.0) }, fc, views::view_font(r, v).1, r.bool(v, "wordWrap"), text_align(r, v), 100.0);
        return;
    }
    // a box: classifier, subject, or any kind UMLCraft does not draw itself
    shadow_rect(c, r, v, b);
    if ty == "UMLNodeView" {
        // a node is a box with depth: top and right faces behind the front one (estimated depth)
        let d = NODE_DEPTH;
        let top = [P { x: b.x, y: b.y }, P { x: b.x + d, y: b.y - d }, P { x: b.x + b.w + d, y: b.y - d }, P { x: b.x + b.w, y: b.y }];
        let right = [P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w + d, y: b.y - d }, P { x: b.x + b.w + d, y: b.y + b.h - d }, P { x: b.x + b.w, y: b.y + b.h }];
        c.polygon(&top, Some(fill), Some(line));
        c.polygon(&right, Some(fill), Some(line));
    }
    c.rect(b, Some(fill), Some(line));
    if ty == "UMLComponentView" {
        // the component icon at the top right: a box with two tabs on its left side
        let (ix, iy) = (b.x + b.w - 22.0, b.y + 6.0);
        c.rect(R { x: ix, y: iy, w: 14.0, h: 16.0 }, Some(fill), Some(line));
        c.rect(R { x: ix - 4.0, y: iy + 3.0, w: 8.0, h: 4.0 }, Some(fill), Some(line));
        c.rect(R { x: ix - 4.0, y: iy + 9.0, w: 8.0, h: 4.0 }, Some(fill), Some(line));
    }
    // stereotype display Decoration (with or without the label): a small icon at the top right, for
    // the stereotypes UMLCraft has icons for (interfaces and the robustness stereotypes)
    let display = r.str(v, "stereotypeDisplay");
    if display == "decoration" || display == "decoration-label" {
        if let Some(m) = r.refd(v, "model") {
            let st = views::type_name(r, m, "stereotype").to_lowercase();
            let c0 = P { x: b.x + b.w - 14.0, y: b.y + 12.0 };
            let rr = 5.0;
            let kind = if r.ty(m) == "UMLInterface" { "interface" } else { st.as_str() };
            match kind {
                "interface" => c.ellipse(c0, rr, rr, Some(fill), Some(line)),
                "boundary" => {
                    c.ellipse(P { x: c0.x + 2.0, y: c0.y }, rr, rr, Some(fill), Some(line));
                    c.polyline(&[P { x: c0.x - 6.0, y: c0.y - rr }, P { x: c0.x - 6.0, y: c0.y + rr }], line, false);
                    c.polyline(&[P { x: c0.x - 6.0, y: c0.y }, P { x: c0.x - 3.0, y: c0.y }], line, false);
                }
                "entity" => {
                    c.ellipse(c0, rr, rr, Some(fill), Some(line));
                    c.polyline(&[P { x: c0.x - rr, y: c0.y + rr }, P { x: c0.x + rr, y: c0.y + rr }], line, false);
                }
                "control" => {
                    c.ellipse(c0, rr, rr, Some(fill), Some(line));
                    c.polyline(&[P { x: c0.x + 2.0, y: c0.y - rr - 2.0 }, P { x: c0.x, y: c0.y - rr }, P { x: c0.x + 2.0, y: c0.y - rr + 2.0 }], line, false);
                }
                _ => {}
            }
        }
    }
    if ty == "UMLArtifactView" {
        // the artifact icon at the top right: a page with a folded corner
        let (ix, iy, f) = (b.x + b.w - 20.0, b.y + 6.0, 4.0);
        let page = [P { x: ix, y: iy }, P { x: ix + 12.0 - f, y: iy }, P { x: ix + 12.0, y: iy + f }, P { x: ix + 12.0, y: iy + 15.0 }, P { x: ix, y: iy + 15.0 }];
        c.polygon(&page, Some(fill), Some(line));
        c.polyline(&[P { x: ix + 12.0 - f, y: iy }, P { x: ix + 12.0 - f, y: iy + f }, P { x: ix + 12.0, y: iy + f }], line, false);
    }
    if let Some(i) = info {
        for comp in &i.compartments {
            c.polyline(&[P { x: b.x, y: comp.y }, P { x: b.x + b.w, y: comp.y }], line, false);
        }
        for comp in &i.compartments {
            for row in &comp.rows {
                draw_label(c, r, row.iv, fc);
            }
        }
        if i.is_active {
            c.polyline(&[P { x: b.x + 3.0, y: b.y }, P { x: b.x + 3.0, y: b.y + b.h }], line, false);
            c.polyline(&[P { x: b.x + b.w - 3.0, y: b.y }, P { x: b.x + b.w - 3.0, y: b.y + b.h }], line, false);
        }
    }
    // template parameters: a dashed box over the top-right corner
    if let (Some(i), Some(tc)) = (info.filter(|i| i.templ.is_some()), r.refd(v, "templateParameterCompartment")) {
        let t = node_rect(r, tc);
        let pts = [P { x: t.x, y: t.y }, P { x: t.x + t.w, y: t.y }, P { x: t.x + t.w, y: t.y + t.h }, P { x: t.x, y: t.y + t.h }, P { x: t.x, y: t.y }];
        c.polygon(&pts[..4], Some(fill), None);
        c.polyline(&pts, line, true);
        for row in &i.templ.as_ref().unwrap().rows {
            draw_label(c, r, row.iv, fc);
        }
    }
    if r.has(v, "nameCompartment") {
        draw_name(c, r, v);
    } else if let Some(m) = r.refd(v, "model") {
        c.text(&r.name(m), R { x: b.x, y: b.y + 5.0, w: b.w, h: 15.0 }, 2, 1, 13.0, fc, false);
    }
}

// ---- line ends (core.js:2715 drawLineEnd)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum End {
    Arrow,
    Triangle,
    Diamond,
    FilledDiamond,
    ArrowDiamond,
    ArrowFilledDiamond,
    Cross,
    /// A circle with a plus at the container's end of a containment line (radius estimated).
    CirclePlus,
    /// A filled arrow head, a free line's "Solid Arrow" end.
    SolidArrow,
}

const CIRCLE_PLUS_R: f64 = 7.0;
/// The depth of a node's box (estimated).
const NODE_DEPTH: f64 = 10.0;
/// The space between an interface ball and the socket around it (estimated).
const SOCKET_GAP: f64 = 5.0;

fn end_shape(c: &mut dyn Canvas, p0: P, p1: P, end: Option<End>, line: Rgba) {
    let Some(end) = end else { return };
    let th = (p1.y - p0.y).atan2(p1.x - p0.x);
    let half = ARROW_HALF_ANGLE_DEG.to_radians();
    let at = |len: f64, ang: f64| P { x: p0.x + len * ang.cos(), y: p0.y + len * ang.sin() };
    let (a1, a2) = (at(ARROW_LENGTH, th - half), at(ARROW_LENGTH, th + half));
    let tip3 = at(TRIANGLE_LENGTH, th);
    let (t1, t2) = (at(TRIANGLE_LENGTH, th - half), at(TRIANGLE_LENGTH, th + half));
    match end {
        End::Arrow => c.polyline(&[a1, p0, a2], line, false),
        End::SolidArrow => c.polygon(&[a1, p0, a2], Some(line), Some(line)),
        End::Triangle => c.polygon(&[t1, p0, t2], Some(WHITE), Some(line)),
        End::Diamond => c.polygon(&[a1, p0, a2, tip3], Some(WHITE), Some(line)),
        End::FilledDiamond => c.polygon(&[a1, p0, a2, tip3], Some(line), Some(line)),
        End::ArrowDiamond | End::ArrowFilledDiamond => {
            let f = if end == End::ArrowFilledDiamond { line } else { WHITE };
            c.polygon(&[a1, p0, a2, tip3], Some(f), Some(line));
            let b1 = P { x: tip3.x + ARROW_LENGTH * (th - half).cos(), y: tip3.y + ARROW_LENGTH * (th - half).sin() };
            let b2 = P { x: tip3.x + ARROW_LENGTH * (th + half).cos(), y: tip3.y + ARROW_LENGTH * (th + half).sin() };
            c.polyline(&[b1, tip3, b2], line, false);
        }
        End::CirclePlus => {
            let m = at(CIRCLE_PLUS_R, th);
            c.ellipse(m, CIRCLE_PLUS_R, CIRCLE_PLUS_R, Some(WHITE), Some(line));
            c.polyline(&[P { x: m.x - CIRCLE_PLUS_R, y: m.y }, P { x: m.x + CIRCLE_PLUS_R, y: m.y }], line, false);
            c.polyline(&[P { x: m.x, y: m.y - CIRCLE_PLUS_R }, P { x: m.x, y: m.y + CIRCLE_PLUS_R }], line, false);
        }
        End::Cross => {
            let m = at(7.0, th);
            let q = std::f64::consts::FRAC_PI_4;
            let k = |a: f64, s: f64| P { x: m.x + s * 7.0 * (th + a).cos(), y: m.y + s * 7.0 * (th + a).sin() };
            c.polyline(&[k(-q, 1.0), k(-q, -1.0)], line, false);
            c.polyline(&[k(q, 1.0), k(q, -1.0)], line, false);
        }
    }
}

fn assoc_end(r: &Repo, end: Option<Eid>) -> Option<End> {
    let end = end?;
    let nav = r.str(end, "navigable");
    let agg = r.str(end, "aggregation");
    match (agg.as_str(), nav == "navigable") {
        ("shared", true) => Some(End::ArrowDiamond),
        ("shared", false) => Some(End::Diamond),
        ("composite", true) => Some(End::ArrowFilledDiamond),
        ("composite", false) => Some(End::FilledDiamond),
        (_, true) => Some(End::Arrow),
        _ if nav == "notNavigable" => Some(End::Cross),
        _ => None,
    }
}

/// The length of a dash and of the gap after it, for a free line's "Dashed" mode (estimated).
const LONG_DASH: (f64, f64) = (8.0, 4.0);

/// Draw a polyline as dashes of `on` units with `off` gaps.
fn dashes(c: &mut dyn Canvas, pts: &[P], color: Rgba, on: f64, off: f64) {
    let (mut in_dash, mut left) = (true, on);
    let mut cur = vec![pts[0]];
    for w in pts.windows(2) {
        let (mut a, b) = (w[0], w[1]);
        let mut seg = ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        while seg > 0.0 {
            let step = left.min(seg);
            let t = step / seg;
            let q = P { x: a.x + (b.x - a.x) * t, y: a.y + (b.y - a.y) * t };
            if in_dash {
                cur.push(q);
            }
            seg -= step;
            left -= step;
            a = q;
            if left <= 0.0 {
                if in_dash && cur.len() > 1 {
                    c.polyline(&cur, color, false);
                }
                in_dash = !in_dash;
                left = if in_dash { on } else { off };
                cur = vec![a];
            }
        }
    }
    if in_dash && cur.len() > 1 {
        c.polyline(&cur, color, false);
    }
}

/// A free line: its points with the chosen pattern and end shapes (style-editor-view.js:296-319).
fn draw_freeline(c: &mut dyn Canvas, r: &Repo, e: Eid, pts: &[P]) {
    let line = color(r, e, "lineColor", BLACK);
    let style = r.val(e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(1);
    let drawn = views::drawn_points(pts, style);
    match r.val(e, "lineMode").and_then(|v| v.as_i64()).unwrap_or(0) {
        1 => c.polyline(&drawn, line, true),
        2 => dashes(c, &drawn, line, LONG_DASH.0, LONG_DASH.1),
        _ => c.polyline(&drawn, line, false),
    }
    let end = |f: &str| match r.val(e, f).and_then(|v| v.as_i64()).unwrap_or(0) {
        1 => Some(End::Arrow),
        2 => Some(End::SolidArrow),
        _ => None,
    };
    let n = pts.len();
    end_shape(c, pts[n - 1], pts[n - 2], end("headEndStyle"), line);
    end_shape(c, pts[0], pts[1], end("tailEndStyle"), line);
}

fn draw_edge(c: &mut dyn Canvas, r: &Repo, e: Eid, info: &HashMap<Eid, Info>) {
    let pts = points_of(r, e);
    if pts.len() < 2 {
        return;
    }
    if views::is_freeline(r, e) {
        return draw_freeline(c, r, e, &pts);
    }
    let line = color(r, e, "lineColor", BLACK);
    let ty = r.ty(e).to_string();
    let mut dashed = matches!(
        ty.as_str(),
        "UMLDependencyView" | "UMLRealizationView" | "UMLInterfaceRealizationView" | "UMLIncludeView" | "UMLExtendView" | "UMLNoteLinkView" | "UMLComponentRealizationView" | "UMLDeploymentView" | "UMLAssociationClassLinkView" | "UMLTemplateBindingView" | "UMLConstraintLinkView"
    );
    let (mut head, mut tail) = (None, None);
    match ty.as_str() {
        "UMLGeneralizationView" | "UMLRealizationView" | "UMLInterfaceRealizationView" | "UMLComponentRealizationView" => head = Some(End::Triangle),
        "UMLDependencyView" | "UMLIncludeView" | "UMLExtendView" | "UMLDeploymentView" | "UMLTemplateBindingView" => head = Some(End::Arrow),
        "UMLContainmentView" => head = Some(End::CirclePlus),
        "UMLAssociationView" | "UMLCommunicationPathView" => {
            if let Some(m) = r.refd(e, "model") {
                tail = assoc_end(r, r.child(m, "end1"));
                head = assoc_end(r, r.child(m, "end2"));
            }
        }
        _ => {}
    }
    let head_ball = r.refd(e, "head").and_then(|h| info.get(&h)).map(|i| i.shape == "interface").unwrap_or(false);
    if head_ball && (ty == "UMLInterfaceRealizationView" || ty == "UMLRealizationView") {
        dashed = false;
        head = None;
    }
    let style = r.val(e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
    let mut pts = pts;
    // a dependency on an interface shown as a ball is a socket: a solid line ending in a half circle
    // around the ball (ball-and-socket notation; the gap is estimated)
    if head_ball && ty == "UMLDependencyView" {
        if let Some(hv) = r.refd(e, "head") {
            let b = node_rect(r, hv);
            let ih = b.h - views::NAME_MIN_H;
            let d = b.w.min(ih);
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + ih / 2.0 };
            let rad = d / 2.0 + SOCKET_GAP;
            let n = pts.len();
            let prev = pts[n - 2];
            let th = (prev.y - ctr.y).atan2(prev.x - ctr.x);
            pts[n - 1] = P { x: ctr.x + rad * th.cos(), y: ctr.y + rad * th.sin() };
            let arc: Vec<P> = (0..=16).map(|i| th - std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * i as f64 / 16.0).map(|a| P { x: ctr.x + rad * a.cos(), y: ctr.y + rad * a.sin() }).collect();
            c.polyline(&arc, line, false);
            dashed = false;
            head = None;
        }
    }
    c.polyline(&views::drawn_points(&pts, style), line, dashed);
    let n = pts.len();
    end_shape(c, pts[n - 1], pts[n - 2], head, line);
    end_shape(c, pts[0], pts[1], tail, line);
    // the ownership dot: an end owned by the classifier at the other end (its ownerAttribute) gets a
    // small filled circle at its tip (UML 2.5; size estimated)
    if let Some(m) = r.refd(e, "model").filter(|m| r.has(*m, "end1")) {
        for (k, tip, next) in [("end1", pts[0], pts[1]), ("end2", pts[n - 1], pts[n - 2])] {
            if r.child(m, k).and_then(|end| r.refd(end, "ownerAttribute")).is_some() {
                let len = ((next.x - tip.x).powi(2) + (next.y - tip.y).powi(2)).sqrt().max(1.0);
                let at = P { x: tip.x + (next.x - tip.x) / len * 6.0, y: tip.y + (next.y - tip.y) / len * 6.0 };
                c.ellipse(at, 3.0, 3.0, Some(line), Some(line));
            }
        }
    }
    let fc = color(r, e, "fontColor", BLACK);
    for sv in r.list(e, "subViews") {
        if r.kind_of(sv, "LabelView") || r.ty(sv) == "EdgeLabelView" {
            draw_label(c, r, sv, fc);
        }
    }
}

pub fn draw_diagram(c: &mut dyn Canvas, r: &Repo, d: Eid, info: &HashMap<Eid, Info>) {
    for v in r.list(d, "ownedViews") {
        if !r.alive(v) || matches!(r.raw(v, "visible"), Some(serde_json::Value::Bool(false))) {
            continue;
        }
        if is_edge(r, v) {
            draw_edge(c, r, v, info);
        } else if is_node(r, v) {
            draw_node(c, r, v, info.get(&v));
        }
    }
}

pub fn handle_points(b: &R) -> [P; 8] {
    let (cx, cy, x2, y2) = (b.x + b.w / 2.0, b.y + b.h / 2.0, b.x + b.w, b.y + b.h);
    [
        P { x: b.x, y: b.y }, P { x: cx, y: b.y }, P { x: x2, y: b.y }, P { x: x2, y: cy },
        P { x: x2, y: y2 }, P { x: cx, y: y2 }, P { x: b.x, y: y2 }, P { x: b.x, y: cy },
    ]
}

pub fn bounds(r: &Repo, d: Eid) -> Option<R> {
    let (mut x1, mut y1, mut x2, mut y2) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    for v in r.list(d, "ownedViews") {
        if !r.alive(v) {
            continue;
        }
        if is_edge(r, v) {
            for p in points_of(r, v) {
                x1 = x1.min(p.x);
                y1 = y1.min(p.y);
                x2 = x2.max(p.x);
                y2 = y2.max(p.y);
            }
        } else if is_node(r, v) {
            let mut boxes = vec![node_rect(r, v)];
            // a template parameter box sticks out of its class
            if let Some(tc) = r.refd(v, "templateParameterCompartment").filter(|tc| shown(r, *tc) && !r.list(*tc, "subViews").is_empty()) {
                boxes.push(node_rect(r, tc));
            }
            for b in boxes {
                x1 = x1.min(b.x);
                y1 = y1.min(b.y);
                x2 = x2.max(b.x + b.w);
                y2 = y2.max(b.y + b.h);
            }
        }
    }
    if x1.is_finite() { Some(R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }) } else { None }
}

// ------------------------------------------------------------------ SVG export
pub struct Svg<'a> {
    pub out: String,
    pub measure: &'a dyn Measure,
}

fn hex(c: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}
fn paint(attr: &str, c: Option<Rgba>) -> String {
    match c {
        None => format!(" {attr}=\"none\""),
        Some(c) if c.3 < 255 => format!(" {attr}=\"{}\" {attr}-opacity=\"{:.2}\"", hex(c), c.3 as f64 / 255.0),
        Some(c) => format!(" {attr}=\"{}\"", hex(c)),
    }
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
fn pts_attr(pts: &[P]) -> String {
    pts.iter().map(|p| format!("{:.2},{:.2}", p.x, p.y)).collect::<Vec<_>>().join(" ")
}

impl Measure for Svg<'_> {
    fn width(&self, text: &str, style: u8) -> f64 {
        self.measure.width(text, style)
    }
}

impl Canvas for Svg<'_> {
    fn polyline(&mut self, pts: &[P], color: Rgba, dashed: bool) {
        let dash = if dashed { format!(" stroke-dasharray=\"{DASH}\"") } else { String::new() };
        self.out += &format!("<polyline points=\"{}\" fill=\"none\"{}{dash}/>\n", pts_attr(pts), paint("stroke", Some(color)));
    }
    fn polygon(&mut self, pts: &[P], fill: Option<Rgba>, stroke: Option<Rgba>) {
        self.out += &format!("<polygon points=\"{}\"{}{}/>\n", pts_attr(pts), paint("fill", fill), paint("stroke", stroke));
    }
    fn ellipse(&mut self, c: P, rx: f64, ry: f64, fill: Option<Rgba>, stroke: Option<Rgba>) {
        self.out += &format!("<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\"{}{}/>\n", c.x, c.y, rx, ry, paint("fill", fill), paint("stroke", stroke));
    }
    fn text(&mut self, t: &str, b: R, align: i64, style: u8, size: f64, color: Rgba, underline: bool) {
        let (x, anchor) = match align {
            0 => (b.x, "start"),
            1 => (b.x + b.w, "end"),
            _ => (b.x + b.w / 2.0, "middle"),
        };
        let weight = if style & 1 != 0 { " font-weight=\"bold\"" } else { "" };
        let italic = if style & 2 != 0 { " font-style=\"italic\"" } else { "" };
        let deco = if underline { " text-decoration=\"underline\"" } else { "" };
        self.out += &format!(
            "<text x=\"{x:.2}\" y=\"{:.2}\" font-family=\"Arial, Liberation Sans, Arimo, sans-serif\" font-size=\"{size}\" text-anchor=\"{anchor}\" dominant-baseline=\"central\"{weight}{italic}{deco}{}>{}</text>\n",
            b.y + b.h / 2.0,
            paint("fill", Some(color)),
            esc(t)
        );
    }
}

/// Margin around the drawing in exported pictures (diagram-export.js:33 BOUNDING_BOX_EXPAND).
pub const BOUNDING_BOX_EXPAND: f64 = 10.0;
/// Extra width and height of exported PNG pictures (diagram-export.js:116-117 getImageData).
pub const PNG_EXTRA: f64 = 30.0;

/// The diagram as SVG: its bounds grown by BOUNDING_BOX_EXPAND on every side, no background, at
/// zoom 1, with `extra` more width and height (0 for SVG export, PNG_EXTRA for PNG; diagram-export.js:178).
pub fn to_svg_sized(r: &Repo, d: Eid, info: &HashMap<Eid, Info>, measure: &dyn Measure, extra: f64) -> Option<String> {
    let b = bounds(r, d)?;
    let m = BOUNDING_BOX_EXPAND;
    let (x, y) = (b.x - m, b.y - m);
    let (w, h) = (b.w + 2.0 * m + extra, b.h + 2.0 * m + extra);
    let mut svg = Svg { out: String::new(), measure };
    draw_diagram(&mut svg, r, d, info);
    Some(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h:.0}\" viewBox=\"{x:.2} {y:.2} {w:.2} {h:.2}\">\n{}</svg>\n",
        svg.out
    ))
}

pub fn to_svg(r: &Repo, d: Eid, info: &HashMap<Eid, Info>, measure: &dyn Measure) -> Option<String> {
    to_svg_sized(r, d, info, measure, 0.0)
}

/// The diagram as a PNG with a transparent background, `scale` pixels per point (the original
/// scales by the screen's devicePixelRatio; diagram-export.js:120-127). Drawn off-screen, so the
/// grid and whatever is scrolled out of view do not matter.
pub fn to_png(r: &Repo, d: Eid, info: &HashMap<Eid, Info>, measure: &dyn Measure, scale: f32) -> Result<Vec<u8>, String> {
    use resvg::{tiny_skia, usvg};
    let svg = to_svg_sized(r, d, info, measure, PNG_EXTRA).ok_or("The diagram is empty.")?;
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(&svg, &opt).map_err(|e| e.to_string())?;
    let size = tree.size();
    let (w, h) = ((size.width() * scale).ceil() as u32, (size.height() * scale).ceil() as u32);
    let mut pixmap = tiny_skia::Pixmap::new(w.max(1), h.max(1)).ok_or("The picture is too large.")?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    pixmap.encode_png().map_err(|e| e.to_string())
}
