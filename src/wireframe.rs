//! Wireframes: frames and user interface widgets drawn as plain outlines. A wireframe owns its
//! diagrams and its widgets; frames and panels hold other widgets, and a tab list lines up its tabs
//! (extensions/essential/wireframe: metamodel.json, elements.js, wireframe-factory.js,
//! wireframe-commands.js).

use crate::flowchart::{cubic, round_box};
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;

/// Padding around a widget's name (elements.js:28-31).
const PAD_X: f64 = 8.0;
const PAD_Y: f64 = 6.0;
/// A frame's corner radius and smallest size, and a widget's corner radius (elements.js:32-36).
const ROUND: f64 = 14.0;
const FRAME_MIN: f64 = 100.0;
const WIDGET_ROUND: f64 = 4.0;
/// A link's font colour (WFLinkView, elements.js:530-535).
pub const LINK_COLOR: &str = "#2b71ff";

/// The toolbox, in its order: (label, model type) (toolbox/wireframe.json).
pub const FRAMES: [(&str, &str); 4] = [("Frame", "WFFrame"), ("Frame (Mobile)", "WFMobileFrame"), ("Frame (Web)", "WFWebFrame"), ("Frame (Desktop)", "WFDesktopFrame")];
pub const COMPONENTS: [(&str, &str); 15] = [
    ("Button", "WFButton"), ("Text", "WFText"), ("Radio", "WFRadio"), ("Checkbox", "WFCheckbox"), ("Switch", "WFSwitch"), ("Link", "WFLink"),
    ("Tab List", "WFTabList"), ("Tab", "WFTab"), ("Input", "WFInput"), ("Dropdown", "WFDropdown"), ("Panel", "WFPanel"), ("Image", "WFImage"),
    ("Separator", "WFSeparator"), ("Avatar", "WFAvatar"), ("Slider", "WFSlider"),
];

/// Whether a model type is a frame or widget (has a view on a wireframe diagram).
pub fn is_node_type(ty: &str) -> bool {
    FRAMES.iter().chain(COMPONENTS.iter()).any(|(_, t)| *t == ty)
}

/// Whether a view type is such a view.
pub fn is_node_view(ty: &str) -> bool {
    ty.strip_suffix("View").map(is_node_type).unwrap_or(false)
}

/// Whether a view has a name label (WFGeneralNodeView and its subclasses, elements.js:270-300).
pub fn has_name(view_ty: &str) -> bool {
    matches!(view_ty, "WFButtonView" | "WFTextView" | "WFRadioView" | "WFCheckboxView" | "WFLinkView" | "WFTabView" | "WFInputView" | "WFDropdownView")
}

/// Whether a view can hold other widgets: a frame or a panel holds every widget, a tab list holds
/// tabs (canContainViewKind, elements.js:320-332, 640-642, 720-732).
pub fn can_contain(container_ty: &str, view_ty: &str) -> bool {
    match container_ty {
        "WFFrameView" | "WFMobileFrameView" | "WFWebFrameView" | "WFDesktopFrameView" | "WFPanelView" => is_node_view(view_ty) && !view_ty.contains("Frame"),
        "WFTabListView" => view_ty == "WFTabView",
        _ => false,
    }
}

/// A widget's view; named widgets get a word-wrapped name label, left-aligned but in a button,
/// where it is centred (WFGeneralNodeView, WFButtonView constructors).
pub fn create_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let ty = format!("{}View", r.ty(model));
    let m = r.ref_to(model);
    let props = vec![
        ("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)),
        ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h)),
    ];
    let v = r.create(&ty, &props, Some((diagram, "ownedViews")));
    link_color(r, v);
    if has_name(&ty) {
        r.set(v, "wordWrap", json!(true));
        let align = if ty == "WFButtonView" { 2 } else { 0 };
        let l = r.create("LabelView", &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(align)), ("wordWrap", json!(true))], Some((v, "subViews")));
        r.set_ref(v, "nameLabel", Some(l));
    }
    v
}

/// A link is blue, whatever the default font colour (WFLinkView constructor).
pub fn link_color(r: &mut Repo, v: Eid) {
    if r.ty(v) == "WFLinkView" {
        r.set(v, "fontColor", json!(LINK_COLOR));
    }
}

fn set_box(r: &mut Repo, l: Eid, b: R) {
    for (k, val) in [("left", b.x), ("top", b.y), ("width", b.w), ("height", b.h)] {
        r.set(l, k, json!(val));
    }
}

/// The views a tab list holds that are tabs.
fn tabs(r: &Repo, v: Eid) -> Vec<Eid> {
    let d = r.parent(v);
    d.map(|d| r.list(d, "ownedViews")).unwrap_or_default().into_iter().filter(|t| r.ty(*t) == "WFTabView" && r.refd(*t, "containerView") == Some(v)).collect()
}

fn vertical(position: &str) -> bool {
    matches!(position, "left" | "right")
}

/// Size a widget and place its name (sizeObject and arrangeObject of each view, elements.js:270-860).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let ty = r.ty(v).to_string();
    let m = r.refd(v, "model");
    let (x, y, w0, h0) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
    let fit = |r: &mut Repo, min_w: f64, min_h: f64| {
        let (w, h) = (w0.max(min_w), h0.max(min_h));
        r.set(v, "width", json!(w));
        r.set(v, "height", json!(h));
        (w, h)
    };
    match ty.as_str() {
        "WFFrameView" | "WFMobileFrameView" | "WFWebFrameView" | "WFDesktopFrameView" => {
            fit(r, FRAME_MIN, FRAME_MIN);
        }
        // a switch is always 26 × 16
        "WFSwitchView" => {
            r.set(v, "width", json!(26.0));
            r.set(v, "height", json!(16.0));
        }
        "WFPanelView" | "WFImageView" => {
            fit(r, 16.0, 16.0);
        }
        // a separator is 8 thick and stretches one way only
        "WFSeparatorView" => {
            if m.map(|m| r.bool(m, "isVertical")).unwrap_or(false) {
                fit(r, 8.0, 16.0);
                r.set(v, "width", json!(8.0));
            } else {
                fit(r, 16.0, 8.0);
                r.set(v, "height", json!(8.0));
            }
        }
        // an avatar stays round: its shorter side wins
        "WFAvatarView" => {
            let (w, h) = fit(r, 24.0, 24.0);
            let s = w.min(h);
            r.set(v, "width", json!(s));
            r.set(v, "height", json!(s));
        }
        "WFSliderView" => {
            fit(r, 16.0, 12.0);
            r.set(v, "height", json!(12.0));
        }
        // a tab list is as long as its tabs, which it lines up across (top, bottom) or down (left,
        // right) in their order, each as deep as the list; they take its position
        "WFTabListView" => {
            let pos = m.map(|m| r.str(m, "position")).unwrap_or_default();
            let vert = vertical(&pos);
            let mut ts = tabs(r, v);
            for t in &ts {
                layout(r, ms, *t);
            }
            let sum: f64 = ts.iter().map(|t| if vert { r.f(*t, "height") } else { r.f(*t, "width") }).sum();
            let (min_w, min_h) = match (ts.is_empty(), vert) {
                (true, _) => (24.0, if vert { 24.0 } else { 26.0 }),
                (false, false) => (sum, 26.0),
                (false, true) => (24.0, sum),
            };
            let (w, h) = fit(r, min_w, min_h);
            if vert {
                ts.sort_by(|a, b| r.f(*a, "top").total_cmp(&r.f(*b, "top")));
            } else {
                ts.sort_by(|a, b| r.f(*a, "left").total_cmp(&r.f(*b, "left")));
            }
            let mut at = if vert { y } else { x };
            for t in ts {
                if let Some(tm) = r.refd(t, "model") {
                    if r.str(tm, "position") != pos && !pos.is_empty() {
                        r.set(tm, "position", json!(pos));
                    }
                }
                if vert {
                    r.set(t, "left", json!(x));
                    r.set(t, "top", json!(at));
                    r.set(t, "width", json!(w));
                    at += r.f(t, "height");
                } else {
                    r.set(t, "left", json!(at));
                    r.set(t, "top", json!(y));
                    r.set(t, "height", json!(h));
                    at += r.f(t, "width");
                }
                layout(r, ms, t);
            }
        }
        _ if has_name(&ty) => {
            let name = m.map(|m| r.name(m)).unwrap_or_default();
            let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
            let lh = ms.size();
            let name_w = if wrap { views::widest_word(ms, &name, 0) } else { ms.width(&name, 0) }.ceil();
            let lines_at = |w: f64| if wrap { views::wrap_text(ms, &name, (w - 1.0).max(1.0), 0).len().max(1) } else { 1 } as f64 * lh;
            // how far the name sits in from the left and the right
            let (inl, inr) = match ty.as_str() {
                "WFRadioView" | "WFCheckboxView" => (18.0, 0.0),
                "WFTabView" | "WFInputView" | "WFDropdownView" => (PAD_X, PAD_X),
                _ => (0.0, 0.0),
            };
            let (min_w, min_h) = match ty.as_str() {
                "WFButtonView" | "WFTabView" | "WFInputView" => (name_w + 2.0 * PAD_X, lines_at(w0 - inl - inr) + 2.0 * PAD_Y),
                "WFDropdownView" => (name_w + 2.0 * PAD_X + 16.0, lines_at(w0 - inl - inr) + 2.0 * PAD_Y),
                "WFRadioView" | "WFCheckboxView" => (name_w + 18.0, lines_at(w0 - inl)),
                _ => (name_w, lines_at(w0)),
            };
            let (w, h) = fit(r, min_w, min_h);
            let font = r.str(v, "font");
            let font = if font.is_empty() { views::FONT.to_string() } else { font };
            if let Some(l) = r.refd(v, "nameLabel") {
                r.set(l, "text", json!(name));
                r.set(l, "wordWrap", json!(wrap));
                r.set(l, "font", json!(font));
                if ty == "WFLinkView" {
                    r.set(l, "underline", json!(true));
                }
                let lw = (w - inl - inr).max(0.0);
                let lhh = lines_at(lw);
                let top = match ty.as_str() {
                    "WFTextView" | "WFLinkView" => y,
                    "WFInputView" | "WFDropdownView" => y + PAD_Y,
                    // centred down the box, 2 px low
                    _ => (y + h / 2.0).round() - (lhh / 2.0).round() + 2.0,
                };
                set_box(r, l, R { x: x + inl, y: top, w: lw, h: lhh });
            }
        }
        _ => {}
    }
}

/// A box with its own radius at each corner: top left, top right, bottom right, bottom left
/// (Canvas.roundRect2).
fn corners(b: R, rad: [f64; 4]) -> Vec<P> {
    let k = 0.5523;
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let mut out = vec![];
    let [tl, tr, br, bl] = rad.map(|r| r.min(b.w / 2.0).min(b.h / 2.0).max(0.0));
    out.push(P { x: x1 + tl, y: y1 });
    out.push(P { x: x2 - tr, y: y1 });
    if tr > 0.0 {
        cubic(&mut out, P { x: x2 - tr, y: y1 }, P { x: x2 - tr + tr * k, y: y1 }, P { x: x2, y: y1 + tr - tr * k }, P { x: x2, y: y1 + tr });
    }
    out.push(P { x: x2, y: y2 - br });
    if br > 0.0 {
        cubic(&mut out, P { x: x2, y: y2 - br }, P { x: x2, y: y2 - br + br * k }, P { x: x2 - br + br * k, y: y2 }, P { x: x2 - br, y: y2 });
    }
    out.push(P { x: x1 + bl, y: y2 });
    if bl > 0.0 {
        cubic(&mut out, P { x: x1 + bl, y: y2 }, P { x: x1 + bl - bl * k, y: y2 }, P { x: x1, y: y2 - bl + bl * k }, P { x: x1, y: y2 - bl });
    }
    out.push(P { x: x1, y: y1 + tl });
    if tl > 0.0 {
        cubic(&mut out, P { x: x1, y: y1 + tl }, P { x: x1, y: y1 + tl - tl * k }, P { x: x1 + tl - tl * k, y: y1 }, P { x: x1 + tl, y: y1 });
    }
    out
}

fn line(c: &mut dyn Canvas, a: (f64, f64), b: (f64, f64), color: Rgba) {
    c.polyline(&[P { x: a.0, y: a.1 }, P { x: b.0, y: b.1 }], color, false);
}

/// An ellipse in the box (x1, y1)–(x2, y2).
fn oval(c: &mut dyn Canvas, x1: f64, y1: f64, x2: f64, y2: f64, fill: Option<Rgba>, stroke: Option<Rgba>) {
    c.ellipse(P { x: (x1 + x2) / 2.0, y: (y1 + y2) / 2.0 }, (x2 - x1) / 2.0, (y2 - y1) / 2.0, fill, stroke);
}

fn bx(x1: f64, y1: f64, x2: f64, y2: f64) -> R {
    R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
}

/// Draw a frame or widget as its outline (drawObject of each view, elements.js:310-860).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, ln: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let ty = r.ty(v).to_string();
    let m = r.refd(v, "model");
    let checked = m.map(|m| r.bool(m, "checked")).unwrap_or(false);
    let yc = ((y1 + y2) / 2.0).round();
    match ty.as_str() {
        "WFFrameView" => c.polygon(&round_box(b, ROUND), Some(fill), Some(ln)),
        // a phone: a rounded body around a screen inset a tenth of the width at the ends and a
        // fifth of that at the sides (turned for landscape)
        "WFMobileFrameView" => {
            let yg = (b.w * 0.1).round();
            let xg = yg * 0.2;
            c.polygon(&round_box(b, yg * 0.5), Some(fill), Some(ln));
            let land = m.map(|m| r.str(m, "orientation") == "landscape").unwrap_or(false);
            let s = if land { bx(x1 + yg, y1 + xg, x2 - yg, y2 - xg) } else { bx(x1 + xg, y1 + yg, x2 - xg, y2 - yg) };
            c.rect(s, None, Some(ln));
        }
        // a browser: a bar 20 high with an address field and two round buttons
        "WFWebFrameView" => {
            c.polygon(&round_box(b, WIDGET_ROUND), Some(fill), Some(ln));
            line(c, (x1, y1 + 20.0), (x2, y1 + 20.0), ln);
            oval(c, x2 - 16.0, y1 + 4.0, x2 - 4.0, y1 + 16.0, None, Some(ln));
            oval(c, x2 - 32.0, y1 + 4.0, x2 - 20.0, y1 + 16.0, None, Some(ln));
            c.polygon(&round_box(bx(x1 + 4.0, y1 + 4.0, x2 - 38.0, y1 + 16.0), 6.0), None, Some(ln));
        }
        // a window: a title bar 20 high with a close box
        "WFDesktopFrameView" => {
            c.polygon(&round_box(b, WIDGET_ROUND), Some(fill), Some(ln));
            line(c, (x1, y1 + 20.0), (x2, y1 + 20.0), ln);
            c.rect(bx(x2 - 16.0, y1 + 4.0, x2 - 4.0, y1 + 16.0), None, Some(ln));
            line(c, (x2 - 16.0, y1 + 4.0), (x2 - 4.0, y1 + 16.0), ln);
            line(c, (x2 - 4.0, y1 + 4.0), (x2 - 16.0, y1 + 16.0), ln);
        }
        "WFButtonView" | "WFInputView" => c.polygon(&round_box(b, WIDGET_ROUND), Some(fill), Some(ln)),
        // a box with a small chevron on the right
        "WFDropdownView" => {
            c.polygon(&round_box(b, WIDGET_ROUND), Some(fill), Some(ln));
            let xr = x2 - PAD_X;
            c.polyline(&[P { x: xr, y: yc - 2.0 }, P { x: xr - 3.0, y: yc + 2.0 }, P { x: xr - 6.0, y: yc - 2.0 }], ln, false);
        }
        // a circle 14 across on the left, with a dot when checked
        "WFRadioView" => {
            oval(c, x1, yc - 7.0, x1 + 14.0, yc + 7.0, None, Some(ln));
            if checked {
                oval(c, x1 + 2.0, yc - 5.0, x1 + 12.0, yc + 5.0, Some(ln), None);
            }
        }
        // a rounded square 14 across on the left, with a tick when checked
        "WFCheckboxView" => {
            c.polygon(&round_box(bx(x1, yc - 7.0, x1 + 14.0, yc + 7.0), 3.0), None, Some(ln));
            if checked {
                c.polyline(&[P { x: x1 + 2.0, y: y1 + 2.0 }, P { x: x1 + 6.0, y: yc + 3.0 }, P { x: x1 + 14.0, y: y1 }], ln, false);
            }
        }
        // a pill with a knob: filled on the right when on, an outline on the left when off
        "WFSwitchView" => {
            c.polygon(&round_box(b, (b.h / 2.0).round()), Some(fill), Some(ln));
            let g = 2.0;
            if checked {
                oval(c, x2 - 15.0 + g, y1 + g, x2 - g, y1 + 15.0 - g, Some(ln), None);
            } else {
                oval(c, x1 + g, y1 + g, x1 + 15.0 - g, y1 + 15.0 - g, None, Some(ln));
            }
        }
        // a tab is rounded on the side away from its list's line
        "WFTabView" => {
            let w = WIDGET_ROUND;
            let rad = match m.map(|m| r.str(m, "position")).unwrap_or_default().as_str() {
                "bottom" => [0.0, 0.0, w, w],
                "left" => [w, 0.0, 0.0, w],
                "right" => [0.0, w, w, 0.0],
                _ => [w, w, 0.0, 0.0],
            };
            c.polygon(&corners(b, rad), Some(fill), Some(ln));
        }
        // a tab list: filled, with a line along the side its tabs sit on
        "WFTabListView" => {
            c.rect(b, Some(fill), None);
            match m.map(|m| r.str(m, "position")).unwrap_or_default().as_str() {
                "bottom" => line(c, (x1, y1), (x2, y1), ln),
                "left" => line(c, (x2, y1), (x2, y2), ln),
                "right" => line(c, (x1, y1), (x1, y2), ln),
                _ => line(c, (x1, y2), (x2, y2), ln),
            }
        }
        "WFPanelView" => c.rect(b, Some(fill), Some(ln)),
        // a crossed box
        "WFImageView" => {
            c.rect(b, Some(fill), Some(ln));
            line(c, (x1, y1), (x2, y2), ln);
            line(c, (x2, y1), (x1, y2), ln);
        }
        "WFSeparatorView" => {
            if m.map(|m| r.bool(m, "isVertical")).unwrap_or(false) {
                line(c, (x1 + 4.0, y1), (x1 + 4.0, y2), ln);
            } else {
                line(c, (x1, y1 + 4.0), (x2, y1 + 4.0), ln);
            }
        }
        // a circle with a head and shoulders
        "WFAvatarView" => {
            oval(c, x1, y1, x2, y2, Some(fill), Some(ln));
            let g = b.w * 0.15;
            let xc = ((x1 + x2) / 2.0).round();
            oval(c, xc - g, yc - g * 2.0, xc + g, yc, None, Some(ln));
            c.polygon(&corners(bx(xc - g * 1.5, yc, xc + g * 1.5, yc + g * 2.0), [WIDGET_ROUND, WIDGET_ROUND, 0.0, 0.0]), None, Some(ln));
        }
        // a track 4 high with a knob at the value, 0 to 100
        "WFSliderView" => {
            c.polygon(&round_box(bx(x1, y1 + 4.0, x2, y1 + 8.0), 2.0), None, Some(ln));
            let value = m.map(|m| r.f(m, "value")).unwrap_or(0.0).clamp(0.0, 100.0);
            let x = (b.w - 12.0) * (value / 100.0) + x1 + 6.0;
            oval(c, x - 6.0, y1, x + 6.0, y2, Some(fill), Some(ln));
        }
        _ => {}
    }
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_label(c, r, v, l, fc);
    }
}

/// A name label, aligned as it says and underlined in a link.
fn draw_label(c: &mut dyn Canvas, r: &Repo, v: Eid, l: Eid, fc: Rgba) {
    if r.raw(l, "visible").and_then(|x| x.as_bool()) == Some(false) {
        return;
    }
    let t = r.str(l, "text");
    if t.is_empty() {
        return;
    }
    let (face, size) = views::view_font(r, v);
    let style = r.str(v, "font").split(';').nth(2).and_then(|x| x.parse::<u8>().ok()).unwrap_or(0);
    let b = views::node_rect(r, l);
    let align = r.raw(l, "horizontalAlignment").and_then(|x| x.as_i64()).unwrap_or(2);
    let underline = r.bool(l, "underline");
    let lines = if r.bool(l, "wordWrap") { views::wrap_text(&views::Scaled { ms: &*c, face, size }, &t, b.w, style) } else { vec![t] };
    for (i, s) in lines.iter().enumerate() {
        c.text(s, R { x: b.x, y: b.y + i as f64 * size, w: b.w, h: size }, align, style, size, fc, underline);
    }
}

impl crate::app::App {
    /// A tab list's Add Tab: a new tab in the list's model, at its left end and bottom edge, which the
    /// list then lines up after its other tabs (handleAddTab, wireframe-commands.js:30-45).
    pub fn add_tab(&mut self, list: Eid) -> Option<Eid> {
        let d = self.diagram?;
        let lm = self.repo.refd(list, "model")?;
        let b = views::node_rect(&self.repo, list);
        let name = self.new_name(lm, "WFTab");
        let m = self.repo.create("WFTab", &[("name", json!(name))], Some((lm, "ownedElements")));
        // after the last tab, so it comes last in the line
        let x = tabs(&self.repo, list).iter().map(|t| self.repo.f(*t, "left") + self.repo.f(*t, "width")).fold(b.x, f64::max);
        let v = create_view(&mut self.repo, d, m, R { x, y: b.y + b.h, w: 1.0, h: 1.0 });
        self.apply_view_defaults(v);
        self.set_container(&[v], Some(list));
        Some(v)
    }
}
