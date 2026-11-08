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
    /// Text turned a quarter left, reading upwards, centred in the box (a horizontal swimlane's
    /// name, LabelView DK_VERT); surfaces that cannot turn text write it level.
    fn text_up(&mut self, t: &str, b: R, style: u8, size: f64, color: Rgba) {
        self.text(t, b, 2, style, size, color, false);
    }
    /// A picture given as a data URL, stretched over the box; surfaces that cannot show one skip it.
    fn image(&mut self, _b: R, _data_url: &str) {}
    /// A polyline `width` units wide (the original's lineWidth 3 borders); surfaces that cannot set
    /// a width draw it as lines side by side.
    fn thick_polyline(&mut self, pts: &[P], color: Rgba, width: f64) {
        let k = ((width - 1.0) / 2.0).max(0.0);
        for (dx, dy) in [(0.0, 0.0), (-k, 0.0), (k, 0.0), (0.0, -k), (0.0, k)] {
            let s: Vec<P> = pts.iter().map(|p| P { x: p.x + dx, y: p.y + dy }).collect();
            self.polyline(&s, color, false);
        }
    }
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

pub(crate) fn draw_label(c: &mut dyn Canvas, r: &Repo, lv: Eid, fc: Rgba) {
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

/// A stereotype's icon in its box: the script drawn, or the picture fitted to its own ratio and
/// centred (drawStereotypeIcon, elements.js:270-330).
pub fn draw_stereotype_icon(c: &mut dyn Canvas, b: R, icon: &views::StereotypeIcon, line: Rgba, fill: Rgba) {
    match icon {
        views::StereotypeIcon::Script(s) => draw_icon_script(c, b, s, line, fill),
        views::StereotypeIcon::Picture(data, w, h) => {
            let f = fit_ratio(b, w.max(1.0), h.max(1.0));
            c.image(R { x: b.x + ((b.w - f.w) / 2.0).round(), y: b.y + ((b.h - f.h) / 2.0).round(), ..f }, data);
        }
    }
}

/// The largest box of the ratio `w`:`h` that fits in `b`, centred (computeIconRect, elements.js:4702-4721).
pub fn fit_ratio(b: R, w: f64, h: f64) -> R {
    if w <= 0.0 || h <= 0.0 {
        return b;
    }
    let (rr, ir) = (b.w / b.h.max(0.0001), w / h);
    if rr >= ir {
        let fw = b.h * ir;
        R { x: b.x + (b.w - fw) / 2.0, y: b.y, w: fw, h: b.h }
    } else {
        let fh = b.w / ir;
        R { x: b.x, y: b.y + (b.h - fh) / 2.0, w: b.w, h: fh }
    }
}

/// Draw an icon script into `b`: `!icon w h` sets its coordinate space, fitted and centred; then
/// l, r, rr, e, pl, pg and a draw lines, rectangles, rounded rectangles, ellipses, polylines,
/// polygons and arcs, fr, frr, fe, fp and fa fill them, and %l, %f, %a and %p set the line colour,
/// the fill colour, the opacity and a dash pattern (graphics.js:2384-2654). %w (line width) is read
/// but UMLCraft draws one-pixel lines.
pub fn draw_icon_script(c: &mut dyn Canvas, b: R, script: &str, line0: Rgba, fill0: Rgba) {
    let (mut sw, mut sh) = (100.0, 100.0);
    let mut cmds: Vec<Vec<String>> = vec![];
    for ln in script.trim().lines() {
        let t: Vec<String> = ln.split(' ').filter(|x| !x.is_empty()).map(String::from).collect();
        if t.is_empty() {
            continue;
        }
        if t[0] == "!icon" {
            sw = t.get(1).and_then(|x| x.parse().ok()).unwrap_or(100.0);
            sh = t.get(2).and_then(|x| x.parse().ok()).unwrap_or(100.0);
        } else {
            cmds.push(t);
        }
    }
    // fit to the box, keeping the script's ratio (graphics.js:2514-2527)
    let (mut w, mut h) = (b.w, b.h);
    if b.h / b.w.max(0.0001) < sh / sw {
        w = h / (sh / sw);
    } else {
        h = w * (sh / sw);
    }
    let (ox, oy, s) = (b.x + ((b.w - w) / 2.0).round(), b.y + ((b.h - h) / 2.0).round(), w / sw);
    let (mut line, mut fill, mut alpha, mut pattern): (Rgba, Rgba, f64, Option<(f64, f64)>) = (line0, fill0, 1.0, None);
    let num = |t: &[String], i: usize| t.get(i).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
    let pt = |x: f64, y: f64| P { x: ox + x * s, y: oy + y * s };
    let with_alpha = |c: Rgba, a: f64| Rgba(c.0, c.1, c.2, (c.3 as f64 * a.clamp(0.0, 1.0)).round() as u8);
    for t in cmds {
        let (l, f) = (with_alpha(line, alpha), with_alpha(fill, alpha));
        let rect = |t: &[String]| {
            let (a, z) = (pt(num(t, 1), num(t, 2)), pt(num(t, 3), num(t, 4)));
            R { x: a.x.min(z.x), y: a.y.min(z.y), w: (z.x - a.x).abs(), h: (z.y - a.y).abs() }
        };
        let pts = |t: &[String]| -> Vec<P> { (0..(t.len() - 1) / 2).map(|i| pt(num(t, 2 * i + 1), num(t, 2 * i + 2))).collect() };
        let arc = |t: &[String]| -> Vec<P> {
            let (cx, cy, rad) = (num(t, 1), num(t, 2), num(t, 3));
            let (a1, a2) = (num(t, 4).to_radians(), num(t, 5).to_radians());
            (0..=24).map(|i| { let a = a1 + (a2 - a1) * i as f64 / 24.0; pt(cx + rad * a.cos(), cy + rad * a.sin()) }).collect()
        };
        let ellipse = |rr: R| -> Vec<P> { (0..=48).map(|i| { let a = std::f64::consts::TAU * i as f64 / 48.0; P { x: rr.x + rr.w / 2.0 * (1.0 + a.cos()), y: rr.y + rr.h / 2.0 * (1.0 + a.sin()) } }).collect() };
        let stroke = |c: &mut dyn Canvas, p: &[P]| match pattern {
            Some((on, off)) => dashes(c, p, l, on, off),
            None => c.polyline(p, l, false),
        };
        let closed = |p: Vec<P>| -> Vec<P> { let mut p = p; if let Some(&f0) = p.first() { p.push(f0); } p };
        let rect_pts = |rr: R| vec![P { x: rr.x, y: rr.y }, P { x: rr.x + rr.w, y: rr.y }, P { x: rr.x + rr.w, y: rr.y + rr.h }, P { x: rr.x, y: rr.y + rr.h }];
        match t[0].as_str() {
            "l" => stroke(c, &[pt(num(&t, 1), num(&t, 2)), pt(num(&t, 3), num(&t, 4))]),
            "r" => stroke(c, &closed(rect_pts(rect(&t)))),
            "rr" => stroke(c, &closed(round_rect(rect(&t), num(&t, 5) * s))),
            "e" => stroke(c, &ellipse(rect(&t))),
            "pl" => stroke(c, &pts(&t)),
            "pg" => stroke(c, &closed(pts(&t))),
            "a" => stroke(c, &arc(&t)),
            "fr" => c.polygon(&rect_pts(rect(&t)), Some(f), None),
            "frr" => c.polygon(&round_rect(rect(&t), num(&t, 5) * s), Some(f), None),
            "fe" => { let rr = rect(&t); c.ellipse(P { x: rr.x + rr.w / 2.0, y: rr.y + rr.h / 2.0 }, rr.w / 2.0, rr.h / 2.0, Some(f), None) }
            "fp" => c.polygon(&pts(&t), Some(f), None),
            "fa" => { let mut p = arc(&t); p.push(pt(num(&t, 1), num(&t, 2))); c.polygon(&p, Some(f), None) }
            "%l" => line = t.get(1).map(|x| parse_color(x, line0)).unwrap_or(line0),
            "%f" => fill = t.get(1).map(|x| parse_color(x, BLACK)).unwrap_or(fill0),
            "%a" => alpha = t.get(1).and_then(|x| x.parse().ok()).unwrap_or(1.0),
            "%p" => pattern = (t.len() > 1).then(|| (num(&t, 1), if t.len() > 2 { num(&t, 2) } else { num(&t, 1) })),
            _ => {}
        }
    }
}

/// A rectangle with corners rounded to radius `rad`, as a polygon (8 steps per corner).
fn round_rect(b: R, rad: f64) -> Vec<P> {
    let mut pts = vec![];
    let corners = [(b.x + b.w - rad, b.y + rad, -90.0), (b.x + b.w - rad, b.y + b.h - rad, 0.0), (b.x + rad, b.y + b.h - rad, 90.0), (b.x + rad, b.y + rad, 180.0)];
    for (cx, cy, start) in corners {
        for i in 0..=8 {
            let a = (start + 90.0 * i as f64 / 8.0_f64).to_radians();
            pts.push(P { x: cx + rad * a.cos(), y: cy + rad * a.sin() });
        }
    }
    pts
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
        "frame" => {
            // the outline, unfilled, and a filled tab at the top left, cut at the corner (elements.js:12214-12245)
            let (Some(nl), Some(tl)) = (r.refd(v, "nameLabel"), r.refd(v, "frameTypeLabel")) else { return };
            let bottom = |l: Eid| r.f(l, "top") + r.f(l, "height");
            let y = bottom(tl).max(bottom(nl)) + views::PAD;
            let h = y - b.y;
            let x = r.f(nl, "left") + r.f(nl, "width") + views::PAD + h / 2.0;
            let tab = [P { x: b.x, y: b.y }, P { x: b.x, y }, P { x: x - h / 2.0, y }, P { x, y: b.y + h / 2.0 }, P { x, y: b.y }];
            c.polygon(&tab, Some(fill), Some(line));
            let outline = [P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }, P { x: b.x, y: b.y }];
            c.polyline(&outline, line, false);
            draw_label(c, r, tl, fc);
            draw_label(c, r, nl, fc);
            return;
        }
        "stereotype-icon" => {
            // the stereotype's icon in the square above the name, then the compartments' rows
            // without a box (drawAsIconicForm and drawIcon, elements.js:4723-4740, 5107-5120)
            if let (Some((icon, iw, ih)), Some(nc)) = (r.refd(v, "model").and_then(|m| views::stereotype_icon(r, m)), r.refd(v, "nameCompartment")) {
                let area = fit_ratio(R { x: b.x, y: b.y, w: b.w, h: (r.f(nc, "top") - b.y).max(0.0) }, 100.0, 100.0);
                let square = R { x: b.x + (b.w - area.w) / 2.0, ..area };
                draw_stereotype_icon(c, fit_ratio(square, iw.max(1.0), ih.max(1.0)), &icon, line, fill);
            }
            draw_name(c, r, v);
            if let Some(i) = info {
                for comp in &i.compartments {
                    for row in &comp.rows {
                        draw_label(c, r, row.iv, fc);
                    }
                }
            }
            return;
        }
        "state" => {
            // a rounded box, the name on top and the activities under a line (estimated)
            let pts = round_rect(b, 10.0);
            c.polygon(&pts, Some(fill), Some(line));
            draw_name(c, r, v);
            // a submachine state: two small circles joined by a line at the bottom right
            // (UMLStateView.drawObject, elements.js:9581-9600)
            if r.refd(v, "model").and_then(|m| r.refd(m, "submachine")).is_some() {
                let (x2, y2) = (b.x + b.w, b.y + b.h);
                c.ellipse(P { x: x2 - 23.0, y: y2 - 13.0 }, 3.0, 3.0, None, Some(line));
                c.polyline(&[P { x: x2 - 20.0, y: y2 - 13.0 }, P { x: x2 - 14.0, y: y2 - 13.0 }], line, false);
                c.ellipse(P { x: x2 - 11.0, y: y2 - 13.0 }, 3.0, 3.0, None, Some(line));
            }
            if let Some(m) = r.refd(v, "model") {
                let rows = views::state_activity_rows(r, m);
                // a composite state: its regions under a line below the name and activities (estimated)
                if r.list(m, "regions").iter().any(|x| r.alive(*x)) {
                    let nh = r.refd(v, "nameCompartment").map(|nc| r.f(nc, "height")).unwrap_or(views::NAME_MIN_H);
                    let size = views::view_font(r, v).1;
                    let y = b.y + nh + if rows.is_empty() { 0.0 } else { rows.len() as f64 * size + 2.0 * views::PAD };
                    c.polyline(&[P { x: b.x, y }, P { x: b.x + b.w, y }], line, false);
                    // further regions: dashed lines splitting the state's height evenly (estimated)
                    let n = r.list(m, "regions").iter().filter(|x| r.alive(**x)).count();
                    for i in 1..n {
                        let ry = b.y + b.h * i as f64 / n as f64;
                        c.polyline(&[P { x: b.x, y: ry }, P { x: b.x + b.w, y: ry }], line, true);
                    }
                }
                if !rows.is_empty() {
                    let nh = r.refd(v, "nameCompartment").map(|nc| r.f(nc, "height")).unwrap_or(views::NAME_MIN_H);
                    let y = b.y + nh;
                    c.polyline(&[P { x: b.x, y }, P { x: b.x + b.w, y }], line, false);
                    let size = views::view_font(r, v).1;
                    let fc = color(r, v, "fontColor", BLACK);
                    for (i, t) in rows.iter().enumerate() {
                        c.text(t, R { x: b.x + views::PAD, y: y + views::PAD + i as f64 * size, w: b.w - 2.0 * views::PAD, h: size }, 0, 0, size, fc, false);
                    }
                }
            }
            return;
        }
        "connection-point" => {
            // an entry point's circle, or an exit point's with an X when it refers to exit points
            // (UMLConnectionPointReferenceView.drawObject, elements.js:9207-9230)
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            c.ellipse(ctr, b.w / 2.0, b.h / 2.0, Some(fill), Some(line));
            if r.refd(v, "model").map(|m| !r.ref_list(m, "exit").is_empty()).unwrap_or(false) {
                let d = (std::f64::consts::SQRT_2 * b.w / 4.0).round();
                c.polyline(&[P { x: ctr.x - d, y: ctr.y - d }, P { x: ctr.x + d, y: ctr.y + d }], line, false);
                c.polyline(&[P { x: ctr.x + d, y: ctr.y - d }, P { x: ctr.x - d, y: ctr.y + d }], line, false);
            }
            for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
                if let Some(lv) = r.refd(v, f) {
                    draw_label(c, r, lv, fc);
                }
            }
            return;
        }
        "final" => {
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            let rad = b.w.min(b.h) / 2.0;
            c.ellipse(ctr, rad, rad, Some(fill), Some(line));
            c.ellipse(ctr, rad - 5.0, rad - 5.0, Some(line), None);
            return;
        }
        "pseudostate" => {
            let kind = r.refd(v, "model").map(|m| r.str(m, "kind")).unwrap_or_default();
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            let rad = b.w.min(b.h) / 2.0;
            match kind.as_str() {
                "choice" => c.polygon(&[P { x: ctr.x, y: b.y }, P { x: b.x + b.w, y: ctr.y }, P { x: ctr.x, y: b.y + b.h }, P { x: b.x, y: ctr.y }], Some(fill), Some(line)),
                "fork" | "join" => {
                    let bar = if b.w >= b.h { R { h: views::BAR_THICK, ..b } } else { R { w: views::BAR_THICK, ..b } };
                    c.polygon(&round_rect(bar, 3.0), Some(line), None);
                }
                "junction" => c.ellipse(ctr, b.w / 2.0, b.h / 2.0, Some(line), Some(line)),
                "shallowHistory" | "deepHistory" => {
                    c.ellipse(ctr, rad, rad, Some(fill), Some(line));
                    let t = if kind == "deepHistory" { "H*" } else { "H" };
                    let size = views::view_font(r, v).1;
                    c.text(t, R { x: b.x, y: ctr.y - size / 2.0, w: b.w, h: size }, 2, 0, size, color(r, v, "fontColor", BLACK), false);
                }
                "entryPoint" => c.ellipse(ctr, rad, rad, Some(fill), Some(line)),
                "exitPoint" | "terminate" => {
                    if kind == "exitPoint" {
                        c.ellipse(ctr, rad, rad, Some(fill), Some(line));
                    }
                    let d = rad * std::f64::consts::FRAC_1_SQRT_2;
                    c.polyline(&[P { x: ctr.x - d, y: ctr.y - d }, P { x: ctr.x + d, y: ctr.y + d }], line, false);
                    c.polyline(&[P { x: ctr.x - d, y: ctr.y + d }, P { x: ctr.x + d, y: ctr.y - d }], line, false);
                }
                // initial: a filled circle
                _ => c.ellipse(ctr, rad, rad, Some(line), Some(line)),
            }
            return;
        }
        "action" => {
            // an action by its kind (UMLActionView.drawObject, elements.js:10100-10215)
            let m = r.refd(v, "model");
            let kind = m.map(|m| r.str(m, "kind")).unwrap_or_default();
            let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
            match kind.as_str() {
                "sendSignal" => {
                    let q = b.h / 4.0;
                    c.polygon(&[P { x: x1, y: y1 }, P { x: x1, y: y2 }, P { x: x2 - q, y: y2 }, P { x: x2, y: y1 + b.h / 2.0 }, P { x: x2 - q, y: y1 }], Some(fill), Some(line));
                }
                "acceptSignal" | "acceptEvent" => {
                    let q = b.h / 4.0;
                    c.polygon(&[P { x: x1, y: y1 }, P { x: x2, y: y1 }, P { x: x2, y: y2 }, P { x: x1, y: y2 }, P { x: x1 + q, y: y1 + b.h / 2.0 }], Some(fill), Some(line));
                }
                "timeEvent" => {
                    // an hourglass 24 px across, centred above the name
                    let cx = (x1 + x2) / 2.0;
                    let (l, rr, t, bt) = (cx - 12.0, cx + 12.0, y1, y1 + 24.0);
                    c.polygon(&[P { x: l, y: t }, P { x: rr, y: t }, P { x: l, y: bt }, P { x: rr, y: bt }], Some(fill), Some(line));
                }
                _ => {
                    c.polygon(&round_rect(b, 10.0), Some(fill), Some(line));
                    let m = m.unwrap_or(v);
                    if kind == "callBehavior" {
                        // calling an activity: a rake at the top right
                        if r.refd(m, "target").map(|t| r.ty(t) == "UMLActivity").unwrap_or(false) {
                            for (a, bb) in [((x2 - 10.0, y1 + 5.0), (x2 - 10.0, y1 + 20.0)), ((x2 - 15.0, y1 + 13.0), (x2 - 5.0, y1 + 13.0)), ((x2 - 15.0, y1 + 13.0), (x2 - 15.0, y1 + 20.0)), ((x2 - 5.0, y1 + 13.0), (x2 - 5.0, y1 + 20.0))] {
                                c.polyline(&[P { x: a.0, y: a.1 }, P { x: bb.0, y: bb.1 }], line, false);
                            }
                        }
                    } else if r.refd(m, "subactivity").is_some() {
                        // a subactivity: two small circles joined by a line at the bottom right
                        c.ellipse(P { x: x2 - 23.0, y: y2 - 13.0 }, 3.0, 3.0, None, Some(line));
                        c.polyline(&[P { x: x2 - 20.0, y: y2 - 13.0 }, P { x: x2 - 14.0, y: y2 - 13.0 }], line, false);
                        c.ellipse(P { x: x2 - 11.0, y: y2 - 13.0 }, 3.0, 3.0, None, Some(line));
                    }
                }
            }
            draw_name(c, r, v);
            return;
        }
        "swimlane" => {
            // a lane open at the far end, with a filled header holding its name (elements.js:10652-10690;
            // the 2 px pen is drawn 1 px wide)
            let vertical = r.bool(v, "isVertical");
            let lv = r.refd(v, "nameLabel");
            let th = lv.map(|l| if vertical { r.f(l, "height") } else { r.f(l, "width") }).unwrap_or(0.0);
            let (x2, y2) = (b.x + b.w, b.y + b.h);
            let (hdr, open) = if vertical {
                (R { x: b.x, y: b.y, w: b.w, h: th + 8.0 }, vec![P { x: b.x, y: y2 }, P { x: b.x, y: b.y }, P { x: x2, y: b.y }, P { x: x2, y: y2 }])
            } else {
                (R { x: b.x, y: b.y, w: th + 8.0, h: b.h }, vec![P { x: x2, y: b.y }, P { x: b.x, y: b.y }, P { x: b.x, y: y2 }, P { x: x2, y: y2 }])
            };
            c.rect(hdr, Some(fill), Some(line));
            c.polyline(&open, line, false);
            if let Some(l) = lv.filter(|l| shown(r, *l)) {
                let lb = R { x: r.f(l, "left"), y: r.f(l, "top"), w: r.f(l, "width"), h: r.f(l, "height") };
                let size = views::view_font(r, v).1;
                let t = r.str(l, "text");
                if vertical {
                    c.text(&t, lb, 2, 0, size, fc, false);
                } else {
                    c.text_up(&t, lb, 0, size, fc);
                }
            }
            return;
        }
        "flowchart" => {
            crate::flowchart::draw(c, r, v, fill, line, fc);
            return;
        }
        "timing-lifeline" => {
            crate::timing::draw_lifeline(c, r, v, line, fc);
            return;
        }
        "timing-state" => {
            crate::timing::draw_state(c, r, v, fc);
            return;
        }
        "time-segment" => {
            crate::timing::draw_segment(c, r, v, line);
            return;
        }
        "time-tick" => {
            crate::timing::draw_tick(c, r, v, line, fc);
            return;
        }
        "time-constraint" => {
            crate::sequence::draw_time(c, r, v, fc);
            return;
        }
        "duration-constraint" => {
            crate::sequence::draw_duration(c, r, v, line, fc);
            return;
        }
        "comm-lifeline" => {
            crate::communication::draw_lifeline(c, r, v, fill, line);
            draw_name(c, r, v);
            return;
        }
        "comm-message" => {
            crate::communication::draw_message(c, r, v, line, fc);
            return;
        }
        "lifeline" => {
            crate::sequence::draw_lifeline(c, r, v, fill, line);
            draw_name(c, r, v);
            crate::sequence::draw_invariants(c, r, v, fill, line, fc);
            return;
        }
        "fragment" => {
            crate::sequence::draw_fragment(c, r, v, fill, line, fc);
            return;
        }
        "endpoint" => {
            crate::sequence::draw_endpoint(c, r, v, fill, line);
            return;
        }
        "c4" => {
            crate::c4::draw(c, r, v, fill, line, fc);
            return;
        }
        "bpmn" => {
            crate::bpmn_draw::draw(c, r, v, fill, line, fc);
            return;
        }
        "erd-entity" => {
            crate::erd::draw(c, r, v, fill, line, fc);
            return;
        }
        "notation" => {
            crate::notations::draw(c, r, v, fill, line, fc);
            return;
        }
        "wireframe" => {
            crate::wireframe::draw(c, r, v, fill, line, fc);
            return;
        }
        "structured" => {
            // a filled rounded box with a dashed border (UMLStructuredActivityNodeView.drawObject,
            // elements.js:10891-10925)
            let mut pts = round_rect(b, 10.0);
            c.polygon(&pts, Some(fill), None);
            pts.push(pts[0]);
            dashes(c, &pts, line, 3.0, 3.0);
            if r.refd(v, "model").and_then(|m| r.refd(m, "subactivity")).is_some() {
                let (x2, y2) = (b.x + b.w, b.y + b.h);
                c.ellipse(P { x: x2 - 23.0, y: y2 - 13.0 }, 3.0, 3.0, None, Some(line));
                c.polyline(&[P { x: x2 - 20.0, y: y2 - 13.0 }, P { x: x2 - 14.0, y: y2 - 13.0 }], line, false);
                c.ellipse(P { x: x2 - 11.0, y: y2 - 13.0 }, 3.0, 3.0, None, Some(line));
            }
            draw_name(c, r, v);
            return;
        }
        "interruptible" => {
            // a dashed rounded box, not filled (UMLInterruptibleActivityRegionView.drawObject,
            // elements.js:10799-10808)
            let mut pts = round_rect(b, 10.0);
            pts.push(pts[0]);
            dashes(c, &pts, line, 3.0, 3.0);
            return;
        }
        "edge-connector" => {
            // a circle with its name in the middle (UMLActivityEdgeConnectorView.drawObject, elements.js:10820-10828)
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            c.ellipse(ctr, b.w / 2.0, b.h / 2.0, Some(fill), Some(line));
            let size = views::view_font(r, v).1;
            let t = r.refd(v, "model").map(|m| r.name(m)).unwrap_or_default();
            c.text(&t, R { x: b.x, y: ctr.y - size / 2.0, w: b.w, h: size }, 2, 0, size, fc, false);
            return;
        }
        "object-node" => {
            // a plain box with its name (elements.js:10244-10247)
            c.rect(b, Some(fill), Some(line));
            draw_name(c, r, v);
            return;
        }
        "control-node" => {
            // the mark of each control node (UMLControlNodeView.drawObject, elements.js:10393-10485)
            let ty = r.refd(v, "model").map(|m| r.ty(m).to_string()).unwrap_or_default();
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            let (rx, ry) = (b.w / 2.0, b.h / 2.0);
            match ty.as_str() {
                "UMLInitialNode" => c.ellipse(ctr, rx, ry, Some(line), None),
                "UMLActivityFinalNode" => {
                    c.ellipse(ctr, rx, ry, None, Some(line));
                    c.ellipse(ctr, (rx - 5.0).max(0.0), (ry - 5.0).max(0.0), Some(line), None);
                }
                "UMLFlowFinalNode" => {
                    c.ellipse(ctr, rx, ry, Some(fill), Some(line));
                    let d = (std::f64::consts::SQRT_2 * b.w / 4.0).round();
                    c.polyline(&[P { x: ctr.x - d, y: ctr.y - d }, P { x: ctr.x + d, y: ctr.y + d }], line, false);
                    c.polyline(&[P { x: ctr.x + d, y: ctr.y - d }, P { x: ctr.x - d, y: ctr.y + d }], line, false);
                }
                "UMLForkNode" | "UMLJoinNode" => {
                    let bar = if b.w >= b.h { R { h: views::BAR_THICK, ..b } } else { R { w: views::BAR_THICK, ..b } };
                    c.polygon(&round_rect(bar, 3.0), Some(line), None);
                }
                _ => c.polygon(&[P { x: b.x, y: ctr.y }, P { x: ctr.x, y: b.y }, P { x: b.x + b.w, y: ctr.y }, P { x: ctr.x, y: b.y + b.h }], Some(fill), Some(line)),
            }
            return;
        }
        "collaboration" => {
            // a dashed ellipse (elements.js:7214-7227, 7275-7279)
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            shadow_ellipse(c, r, v, ctr, b.w / 2.0, b.h / 2.0);
            c.ellipse(ctr, b.w / 2.0, b.h / 2.0, Some(fill), None);
            let pts: Vec<P> = (0..=72).map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 72.0;
                P { x: ctr.x + b.w / 2.0 * a.cos(), y: ctr.y + b.h / 2.0 * a.sin() }
            }).collect();
            dashes(c, &pts, line, 5.0, 5.0);
            draw_name(c, r, v);
            return;
        }
        "pin" => {
            // a square on the action's border with an arrow pointing into the action for an input
            // pin and out of it for an output pin (elements.js:9790-9840)
            c.rect(b, Some(fill), Some(line));
            if r.ty(v) == "UMLExpansionNodeView" {
                // an expansion node: a box split into four cells along its border (elements.js:9848-9895)
                let s = views::PIN_SIZE;
                for i in 1..4 {
                    let k = i as f64 * s;
                    match views::pin_side(r, v) {
                        "top" | "bottom" => c.polyline(&[P { x: b.x + k, y: b.y }, P { x: b.x + k, y: b.y + b.h }], line, false),
                        "left" | "right" => c.polyline(&[P { x: b.x, y: b.y + k }, P { x: b.x + b.w, y: b.y + k }], line, false),
                        _ => {}
                    }
                }
                for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
                    if let Some(lv) = r.refd(v, f) {
                        draw_label(c, r, lv, fc);
                    }
                }
                return;
            }
            let input = r.ty(v) == "UMLInputPinView";
            let dir = match (views::pin_side(r, v), input) {
                ("top", true) | ("bottom", false) => Some("down"),
                ("bottom", true) | ("top", false) => Some("up"),
                ("left", true) | ("right", false) => Some("right"),
                ("right", true) | ("left", false) => Some("left"),
                _ => None,
            };
            let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
            let (t, bt, l, rt) = (P { x: ctr.x, y: b.y + 2.0 }, P { x: ctr.x, y: b.y + b.h - 2.0 }, P { x: b.x + 2.0, y: ctr.y }, P { x: b.x + b.w - 2.0, y: ctr.y });
            let (from, to) = match dir {
                Some("down") => (t, bt),
                Some("up") => (bt, t),
                Some("right") => (l, rt),
                Some("left") => (rt, l),
                _ => (ctr, ctr),
            };
            if dir.is_some() {
                c.polyline(&[from, to], line, false);
                let (dx, dy) = (((to.x - from.x).signum()), ((to.y - from.y).signum()));
                let (a, bb) = if dx != 0.0 { (P { x: to.x - 3.0 * dx, y: to.y - 3.0 }, P { x: to.x - 3.0 * dx, y: to.y + 3.0 }) } else { (P { x: to.x - 3.0, y: to.y - 3.0 * dy }, P { x: to.x + 3.0, y: to.y - 3.0 * dy }) };
                c.polyline(&[a, to, bb], line, false);
            }
            for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
                if let Some(lv) = r.refd(v, f) {
                    draw_label(c, r, lv, fc);
                }
            }
            return;
        }
        "port" => {
            // a filled square on the border, its labels floating by it (elements.js:6950-6953)
            c.rect(b, Some(fill), Some(line));
            for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
                if let Some(lv) = r.refd(v, f) {
                    draw_label(c, r, lv, fc);
                }
            }
            return;
        }
        "part" => {
            // a box, solid for a composite part and dashed otherwise, no shadow (elements.js:6977-6984)
            let composite = r.refd(v, "model").map(|m| r.str(m, "aggregation") == "composite").unwrap_or(false);
            c.rect(b, Some(fill), None);
            let pts = [P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }, P { x: b.x, y: b.y }];
            if composite {
                c.polyline(&pts, line, false);
            } else {
                dashes(c, &pts, line, 5.0, 5.0);
            }
            draw_name(c, r, v);
            return;
        }
        "nary" => {
            // a diamond touching the middle of each side, with its shadow (elements.js:6619-6652)
            let (cx, cy) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
            let diamond = [P { x: b.x, y: cy }, P { x: cx, y: b.y }, P { x: b.x + b.w, y: cy }, P { x: cx, y: b.y + b.h }];
            if r.bool(v, "showShadow") {
                let s = Rgba(SHADOW_COLOR.0, SHADOW_COLOR.1, SHADOW_COLOR.2, (255.0 * SHADOW_ALPHA) as u8);
                let sp: Vec<P> = diamond.iter().map(|p| P { x: p.x + SHADOW_OFFSET, y: p.y + SHADOW_OFFSET }).collect();
                c.polygon(&sp, Some(s), None);
            }
            c.polygon(&diamond, Some(fill), Some(line));
            for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
                if let Some(lv) = r.refd(v, f) {
                    draw_label(c, r, lv, fc);
                }
            }
            return;
        }
        "hyperlink" => {
            // a filled box split after "link" (elements.js:14102-14107)
            let (Some(nl), Some(tl)) = (r.refd(v, "nameLabel"), r.refd(v, "typeLabel")) else { return };
            c.rect(b, Some(fill), Some(line));
            let x = r.f(tl, "left") + r.f(tl, "width") + views::PAD;
            c.polyline(&[P { x, y: b.y }, P { x, y: b.y + b.h }], line, false);
            draw_label(c, r, tl, fc);
            draw_label(c, r, nl, fc);
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
    // a picture over its box; with none yet, a crossed-out box (ImageView.drawObject, elements.js)
    if ty == "ImageView" {
        let data = r.str(v, "imageData");
        if data.is_empty() {
            c.polyline(&[P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }], line, false);
            c.polyline(&[P { x: b.x + b.w, y: b.y }, P { x: b.x, y: b.y + b.h }], line, false);
            c.rect(b, None, Some(line));
        } else {
            c.image(b, &data);
        }
        return;
    }
    // plain shapes, filled and outlined, with no shadow (elements.js:14337-14378)
    if ty == "RectangleView" {
        c.rect(b, Some(fill), Some(line));
        return;
    }
    if ty == "RoundRectView" {
        c.polygon(&round_rect(b, b.w.min(b.h) / 8.0), Some(fill), Some(line));
        return;
    }
    if ty == "EllipseView" {
        c.ellipse(P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 }, b.w / 2.0, b.h / 2.0, Some(fill), Some(line));
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
    if ty == "UMLNodeView" || ty == "UMLNodeInstanceView" {
        // a node is a box with depth: top and right faces behind the front one (estimated depth)
        let d = NODE_DEPTH;
        let top = [P { x: b.x, y: b.y }, P { x: b.x + d, y: b.y - d }, P { x: b.x + b.w + d, y: b.y - d }, P { x: b.x + b.w, y: b.y }];
        let right = [P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w + d, y: b.y - d }, P { x: b.x + b.w + d, y: b.y + b.h - d }, P { x: b.x + b.w, y: b.y + b.h }];
        c.polygon(&top, Some(fill), Some(line));
        c.polygon(&right, Some(fill), Some(line));
    }
    c.rect(b, Some(fill), Some(line));
    if ty == "UMLComponentView" || ty == "UMLComponentInstanceView" {
        // the component icon at the top right: a box with two tabs on its left side
        let (ix, iy) = (b.x + b.w - 22.0, b.y + 6.0);
        c.rect(R { x: ix, y: iy, w: 14.0, h: 16.0 }, Some(fill), Some(line));
        c.rect(R { x: ix - 4.0, y: iy + 3.0, w: 8.0, h: 4.0 }, Some(fill), Some(line));
        c.rect(R { x: ix - 4.0, y: iy + 9.0, w: 8.0, h: 4.0 }, Some(fill), Some(line));
    }
    // stereotype display Decoration (with or without the label): a small icon at the top right, for
    // the stereotypes UMLCraft has icons for (interfaces and the robustness stereotypes)
    let display = r.str(v, "stereotypeDisplay");
    let profile_icon = r.refd(v, "model").and_then(|m| views::stereotype_icon(r, m)).filter(|_| r.refd(v, "model").map(|m| r.ty(m) != "UMLInterface").unwrap_or(false));
    if let (Some((icon, iw, ih)), true) = (&profile_icon, display == "decoration" || display == "decoration-label") {
        // a profile stereotype's icon in the 24 × 24 box at the top right (drawAsDecorationForm, elements.js:5086-5105)
        let bx = R { x: b.x + b.w - views::PAD - views::DECORATION_ICON, y: b.y + views::PAD, w: views::DECORATION_ICON, h: views::DECORATION_ICON };
        draw_stereotype_icon(c, fit_ratio(bx, iw.max(1.0), ih.max(1.0)), icon, line, fill);
    } else if display == "decoration" || display == "decoration-label" {
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
    if ty == "UMLArtifactView" || ty == "UMLArtifactInstanceView" {
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
    /// A filled triangle, at the metaclass end of an extension.
    FilledTriangle,
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
        End::FilledTriangle => c.polygon(&[t1, p0, t2], Some(line), Some(line)),
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
pub(crate) fn dashes(c: &mut dyn Canvas, pts: &[P], color: Rgba, on: f64, off: f64) {
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
    // BPMN lines draw their own patterns and ends (bpmn/elements.js)
    if crate::bpmn::is_edge_view(r.ty(e)) {
        return crate::bpmn_draw::draw_edge(c, r, e, &pts);
    }
    let line = color(r, e, "lineColor", BLACK);
    let ty = r.ty(e).to_string();
    let mut dashed = matches!(
        ty.as_str(),
        "UMLDependencyView" | "UMLRealizationView" | "UMLInterfaceRealizationView" | "UMLIncludeView" | "UMLExtendView" | "UMLNoteLinkView" | "UMLComponentRealizationView" | "UMLDeploymentView" | "UMLAssociationClassLinkView" | "UMLTemplateBindingView" | "UMLConstraintLinkView" | "UMLInformationFlowView" | "UMLLinkObjectLinkView" | "UMLRoleBindingView"
    );
    let (mut head, mut tail) = (None, None);
    // a message: dotted for replies and create messages, a filled arrow for calls and deletes, an
    // open one for the others (UMLSeqMessageView.update, elements.js:11700-11725)
    let seq_sort = (ty == "UMLSeqMessageView" || ty == "UMLTimingMessageView").then(|| r.refd(e, "model").map(|m| r.str(m, "messageSort"))).flatten();
    if let Some(sort) = &seq_sort {
        dashed = matches!(sort.as_str(), "reply" | "createMessage");
        head = Some(if matches!(sort.as_str(), "asynchCall" | "asynchSignal" | "createMessage" | "reply") { End::Arrow } else { End::SolidArrow });
    }
    // a non-identifying ERD relationship is dotted (ERDRelationshipView.update, erd/elements.js:555-560)
    if ty == "ERDRelationshipView" && r.refd(e, "model").map(|m| !r.bool(m, "identifying")).unwrap_or(false) {
        dashed = true;
    }
    match ty.as_str() {
        "UMLGeneralizationView" | "UMLRealizationView" | "UMLInterfaceRealizationView" | "UMLComponentRealizationView" => head = Some(End::Triangle),
        // an extension: a solid line with a filled triangle at the metaclass
        "UMLExtensionView" => head = Some(End::FilledTriangle),
        "UMLDependencyView" | "UMLIncludeView" | "UMLExtendView" | "UMLDeploymentView" | "UMLTemplateBindingView" | "UMLInformationFlowView" | "UMLRoleBindingView" | "UMLTransitionView" | "UMLControlFlowView" | "UMLObjectFlowView" | "UMLExceptionHandlerView" | "UMLActivityInterruptView" => head = Some(End::Arrow),
        "UMLSeqMessageView" | "UMLTimingMessageView" => {}
        // a flowchart flow ends in a filled arrow (FCGeneralEdgeView, flowchart/elements.js:200)
        "FCFlowView" | "DFDDataFlowView" => head = Some(End::SolidArrow),
        // a C4 relationship is dotted with a filled arrow (C4RelationshipView, c4/elements.js:590-597)
        "C4RelationshipView" => {
            head = Some(End::SolidArrow);
            dashed = true;
        }
        "UMLContainmentView" => head = Some(End::CirclePlus),
        "UMLAssociationView" | "UMLCommunicationPathView" | "UMLLinkView" => {
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
    // an association end's qualifiers: a box of rows against the end's box
    for field in ["tailQualifiersCompartment", "headQualifiersCompartment"] {
        let Some(q) = r.refd(e, field).filter(|q| r.val(*q, "visible").and_then(|v| v.as_bool()) != Some(false)) else { continue };
        let rows = views::qualifier_rows(r, q);
        if rows.is_empty() {
            continue;
        }
        let b = node_rect(r, q);
        c.rect(b, Some(color(r, e, "fillColor", WHITE)), Some(line));
        let size = views::view_font(r, e).1;
        let fc = color(r, e, "fontColor", BLACK);
        for (i, t) in rows.iter().enumerate() {
            c.text(t, R { x: b.x + views::PAD, y: b.y + views::PAD + i as f64 * size, w: b.w - 2.0 * views::PAD, h: size }, 0, 0, size, fc, false);
        }
    }
    let n = pts.len();
    // a message's activation, drawn over the line's end (UMLActivationView.drawObject, elements.js:11595)
    if let Some(a) = r.refd(e, "activation").filter(|a| r.raw(*a, "visible").and_then(|v| v.as_bool()) != Some(false)) {
        c.rect(node_rect(r, a), Some(color(r, e, "fillColor", WHITE)), Some(line));
    }
    // a delete message crosses out the end of its lifeline (elements.js:11940-11948)
    if ty == "UMLSeqMessageView" && seq_sort.as_deref() == Some("deleteMessage") {
        if let Some(h) = r.refd(e, "head") {
            let b = node_rect(r, h);
            let (cx, by) = (b.x + b.w / 2.0, b.y + b.h);
            c.polyline(&[P { x: cx - 10.0, y: by - 10.0 }, P { x: cx + 10.0, y: by + 10.0 }], line, false);
            c.polyline(&[P { x: cx + 10.0, y: by - 10.0 }, P { x: cx - 10.0, y: by + 10.0 }], line, false);
        }
    }
    end_shape(c, pts[n - 1], pts[n - 2], head, line);
    end_shape(c, pts[0], pts[1], tail, line);
    // an ERD relationship: crow's feet by each end's cardinality
    if ty == "ERDRelationshipView" {
        if let Some(m) = r.refd(e, "model") {
            let card = |k: &str| r.child(m, k).map(|x| r.str(x, "cardinality")).filter(|c| !c.is_empty()).unwrap_or_else(|| "1".into());
            crate::erd::draw_end(c, pts[0], pts[1], &card("end1"), line);
            crate::erd::draw_end(c, pts[n - 1], pts[n - 2], &card("end2"), line);
        }
    }
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
        // the zig-zag of an exception handler or interrupt (UMLZigZagAdornmentView.drawObject, elements.js:10576-10596)
        if r.ty(sv) == "UMLZigZagAdornmentView" && r.val(sv, "visible").and_then(|x| x.as_bool()) != Some(false) {
            let z = node_rect(r, sv);
            let (x1, y1, x2, y2) = (z.x, z.y, z.x + z.w, z.y + z.h);
            c.polyline(&[P { x: x1, y: y1 }, P { x: x2, y: y1 }, P { x: x1, y: y2 - 5.0 }, P { x: x2, y: y2 - 5.0 }], line, false);
            c.polyline(&[P { x: x2 - 5.0, y: y2 }, P { x: x2, y: y2 - 5.0 }, P { x: x2 - 5.0, y: y2 - 10.0 }], line, false);
        }
    }
    // the order of the ends: a filled triangle beside the name, pointing from end1 to end2 (forward)
    // or back, on the side that end lies (UMLUndirectedRelationshipView.drawEndOrder, elements.js:5837-5868)
    let order = r.str(e, "showEndOrder");
    if let (true, Some(nl)) = (order == "forward" || order == "backward", r.refd(e, "nameLabel")) {
        let (x1, x2) = (pts[0].x, pts[pts.len() - 1].x);
        let right = if order == "forward" { x1 < x2 } else { x1 > x2 };
        let (l, t, w, h) = (r.f(nl, "left"), r.f(nl, "top"), r.f(nl, "width"), r.f(nl, "height"));
        let y = t + h / 2.0;
        let tri = if right {
            let x = l + w + 5.0;
            [P { x, y: y - 5.0 }, P { x: x + 10.0, y }, P { x, y: y + 5.0 }]
        } else {
            let x = l - 5.0;
            [P { x, y: y - 5.0 }, P { x: x - 10.0, y }, P { x, y: y + 5.0 }]
        };
        c.polygon(&tri, Some(line), None);
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
            if r.ty(v) == "UMLInteractionInlineView" {
                crate::overview::draw_inline(c, r, v, info);
            }
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
        if !shown(r, v) {
            continue;
        }
        if is_edge(r, v) {
            for p in points_of(r, v) {
                x1 = x1.min(p.x);
                y1 = y1.min(p.y);
                x2 = x2.max(p.x);
                y2 = y2.max(p.y);
            }
            // its labels, which can sit away from the line (the original counts children too)
            for l in r.list(v, "subViews") {
                if shown(r, l) && !r.str(l, "text").is_empty() && r.has(l, "width") {
                    let b = node_rect(r, l);
                    x1 = x1.min(b.x);
                    y1 = y1.min(b.y);
                    x2 = x2.max(b.x + b.w);
                    y2 = y2.max(b.y + b.h);
                }
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
    fn image(&mut self, b: R, data_url: &str) {
        self.out += &format!(
            "<image x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" preserveAspectRatio=\"none\" href=\"{}\"/>\n",
            b.x, b.y, b.w, b.h, esc(data_url)
        );
    }
    fn polyline(&mut self, pts: &[P], color: Rgba, dashed: bool) {
        let dash = if dashed { format!(" stroke-dasharray=\"{DASH}\"") } else { String::new() };
        self.out += &format!("<polyline points=\"{}\" fill=\"none\"{}{dash}/>\n", pts_attr(pts), paint("stroke", Some(color)));
    }
    fn thick_polyline(&mut self, pts: &[P], color: Rgba, width: f64) {
        self.out += &format!("<polyline points=\"{}\" fill=\"none\" stroke-width=\"{width}\"{}/>\n", pts_attr(pts), paint("stroke", Some(color)));
    }
    fn polygon(&mut self, pts: &[P], fill: Option<Rgba>, stroke: Option<Rgba>) {
        self.out += &format!("<polygon points=\"{}\"{}{}/>\n", pts_attr(pts), paint("fill", fill), paint("stroke", stroke));
    }
    fn ellipse(&mut self, c: P, rx: f64, ry: f64, fill: Option<Rgba>, stroke: Option<Rgba>) {
        self.out += &format!("<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\"{}{}/>\n", c.x, c.y, rx, ry, paint("fill", fill), paint("stroke", stroke));
    }
    fn text_up(&mut self, t: &str, b: R, style: u8, size: f64, color: Rgba) {
        let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
        let weight = if style & 1 != 0 { " font-weight=\"bold\"" } else { "" };
        self.out += &format!(
            "<text x=\"{x:.2}\" y=\"{y:.2}\" transform=\"rotate(-90 {x:.2} {y:.2})\" font-family=\"Arial, Liberation Sans, Arimo, sans-serif\" font-size=\"{size}\" text-anchor=\"middle\" dominant-baseline=\"central\"{weight}{}>{}</text>\n",
            paint("fill", Some(color)),
            esc(t)
        );
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
