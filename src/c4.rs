//! C4 model diagrams: people, software systems, containers and components as rounded boxes with
//! their name, a "[Kind: technology]" line and a description, and dotted relationships
//! (extensions/essential/c4: metamodel.json, elements.js, c4-factory.js, c4-commands.js).

use crate::flowchart::{cubic, round_box};
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;

const PAD: f64 = 8.0; // COMPARTMENT_*_PADDING (elements.js:32-35)
const GAP: f64 = 2.0; // COMPARTMENT_ITEM_INTERVAL
const HEAD_BODY_GAP: f64 = 8.0;
const DESCRIPTION_MARGIN: f64 = 8.0;
const ROUND: f64 = 6.0;

/// The C4 tools: label, tool id, model type and container kind (toolbox/c4.json).
pub const TOOLS: [(&str, &str, &str, Option<&str>); 9] = [
    ("Person", "C4Person", "C4Person", None),
    ("Software System", "C4SoftwareSystem", "C4SoftwareSystem", None),
    ("Container", "C4Container", "C4Container", None),
    ("Container (Database)", "C4ContainerDatabase", "C4Container", Some("database")),
    ("Container (Web App)", "C4ContainerWebApp", "C4Container", Some("client-webapp")),
    ("Container (Desktop App)", "C4ContainerDesktopApp", "C4Container", Some("desktop-app")),
    ("Container (Mobile App)", "C4ContainerMobileApp", "C4Container", Some("mobile-app")),
    ("Component", "C4Component", "C4Component", None),
    ("Element", "C4Element", "C4Element", None),
];

/// The model type and container kind a C4 tool makes.
pub fn tool(kind: &str) -> Option<(&'static str, Option<&'static str>)> {
    TOOLS.iter().find(|t| t.1 == kind).map(|t| (t.2, t.3))
}

pub fn is_node_type(ty: &str) -> bool {
    matches!(ty, "C4Person" | "C4SoftwareSystem" | "C4Container" | "C4Component" | "C4Element")
}

pub fn is_node_view(ty: &str) -> bool {
    matches!(ty, "C4PersonView" | "C4SoftwareSystemView" | "C4ContainerView" | "C4ComponentView" | "C4ElementView")
}

/// A C4 element's view: its name, technology and description labels (C4GeneralNodeView,
/// elements.js:120-150).
pub fn create_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let ty = format!("{}View", r.ty(model));
    let m = r.ref_to(model);
    let v = r.create(
        &ty,
        &[
            ("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)),
            ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h)),
        ],
        Some((diagram, "ownedViews")),
    );
    for f in ["nameLabel", "technologyLabel", "descriptionLabel"] {
        let l = r.create("LabelView", &[("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(2)), ("wordWrap", json!(true))], Some((v, "subViews")));
        r.set_ref(v, f, Some(l));
    }
    v
}

/// Whether the element is the diagram's owner or one of its owners: then the view is the boundary
/// the diagram details, drawn dashed with its name at the bottom left and no description
/// (C4GeneralNodeView.update with isOneOfTheContainers, elements.js:175-180; core.js:843-851).
pub fn is_boundary(r: &Repo, v: Eid) -> bool {
    let (Some(m), Some(d)) = (r.refd(v, "model"), r.parent(v)) else { return false };
    let mut p = r.parent(d);
    while let Some(x) = p {
        if x == m {
            return true;
        }
        p = r.parent(x);
    }
    false
}

/// The "[Kind: technology]" line (the update of each view, elements.js:330-500).
pub fn technology_text(r: &Repo, m: Eid) -> String {
    let tech = r.str(m, "technology");
    let kind = match r.ty(m) {
        "C4Person" => "Person",
        "C4SoftwareSystem" => "Software System",
        "C4Container" => "Container",
        "C4Component" => "Component",
        _ => return if tech.is_empty() { String::new() } else { format!("[{tech}]") },
    };
    if tech.is_empty() { format!("[{kind}]") } else { format!("[{kind}: {tech}]") }
}

/// The padding above and below the labels: a person leaves room for its head, containers for their
/// marks (C4PersonView.update, C4ContainerView.update, elements.js:330-420).
fn paddings(r: &Repo, v: Eid, m: Eid) -> (f64, f64) {
    let (w, h) = (r.f(v, "width"), r.f(v, "height"));
    match r.ty(m) {
        "C4Person" => ((w / 3.0).round() + PAD + HEAD_BODY_GAP, PAD),
        "C4Container" => match r.str(m, "kind").as_str() {
            "client-webapp" | "console-app" | "shell-script" | "serverless-function" => (30.0, PAD),
            "mobile-app" => (16.0, 20.0),
            "desktop-app" => (20.0, PAD),
            "database" => (PAD + (h / 8.0).floor() * 2.0, PAD),
            _ => (PAD, PAD),
        },
        _ => (PAD, PAD),
    }
}

/// Size the box to its labels and stack them: from the top, centred, or from the bottom on the
/// left for a generic element and a boundary (C4GeneralNodeView.size and arrange, elements.js:150-245).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return };
    let boundary = is_boundary(r, v);
    let bottom = boundary || r.ty(v) == "C4ElementView";
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let (face, size) = (ms.face(), ms.size());
    let texts = [(r.name(m), 1u8, size), (technology_text(r, m), 0u8, size * 0.9), (r.str(m, "description"), 0u8, size)];
    let (top_pad, bottom_pad) = paddings(r, v, m);
    let w0 = r.f(v, "width");
    let scaled = |s: f64| views::Scaled { ms, face: face.clone(), size: s };
    let visible = |i: usize| !(i == 2 && boundary);
    // the narrowest the box may be: its widest word (or line, without word wrap); the labels then
    // wrap to the width the box gets
    let mut min_w = 0.0f64;
    for (i, (t, style, s)) in texts.iter().enumerate().filter(|(i, _)| visible(*i)) {
        let sm = scaled(*s);
        let words = if wrap { views::widest_word(&sm, t, *style) } else { sm.width(t, *style) };
        let _ = i;
        min_w = min_w.max(words.ceil());
    }
    let lw = (w0.max(min_w + 2.0 * PAD) - 2.0 * PAD).max(1.0);
    let mut heights = [0.0; 3];
    for (i, (t, style, s)) in texts.iter().enumerate().filter(|(i, _)| visible(*i)) {
        let sm = scaled(*s);
        let lines = if t.is_empty() { 0 } else if wrap { views::wrap_text(&sm, t, lw - 1.0, *style).len() } else { 1 };
        heights[i] = lines as f64 * s;
    }
    let shown = if boundary { 2 } else { 3 };
    let min_h = top_pad + heights.iter().take(shown).sum::<f64>() + GAP * (shown - 1) as f64 + bottom_pad + DESCRIPTION_MARGIN;
    let min_w = min_w + 2.0 * PAD;
    let auto = r.bool(v, "autoResize");
    let (w, h) = if auto { (min_w, min_h) } else { (w0.max(min_w), r.f(v, "height").max(min_h)) };
    r.set(v, "width", json!(w));
    r.set(v, "height", json!(h));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let labels: Vec<Option<Eid>> = ["nameLabel", "technologyLabel", "descriptionLabel"].iter().map(|f| r.refd(v, f)).collect();
    let mut tops = [0.0; 3];
    if bottom {
        let mut acc = bottom_pad;
        for i in (0..3).rev() {
            if i == 2 && boundary {
                continue;
            }
            acc += heights[i];
            tops[i] = y + h - acc;
            if i < 2 {
                acc += GAP;
            }
        }
    } else {
        let mut acc = top_pad;
        for i in 0..3 {
            if i > 0 {
                acc += GAP;
            }
            tops[i] = y + acc;
            acc += heights[i];
        }
    }
    tops[2] += DESCRIPTION_MARGIN;
    for (i, l) in labels.into_iter().enumerate() {
        let Some(l) = l else { continue };
        let (t, style, s) = &texts[i];
        r.set(l, "text", json!(t));
        r.set(l, "font", json!(views::font_str(&face, *s, *style as i64)));
        r.set(l, "wordWrap", json!(wrap));
        r.set(l, "horizontalAlignment", json!(if bottom { 0 } else { 2 }));
        if i == 2 && boundary {
            r.set_raw(l, "visible", json!(false));
        } else {
            r.remove(l, "visible");
        }
        for (k, val) in [("left", x + PAD), ("top", tops[i]), ("width", w - 2.0 * PAD), ("height", heights[i])] {
            r.set(l, k, json!(val));
        }
    }
}

fn database(b: R) -> (Vec<P>, Vec<P>) {
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let xm = (x1 + x2) / 2.0;
    let g = (b.h / 8.0).floor();
    let k = 0.5522848;
    let (ox, oy) = (b.w / 2.0 * k, g * k);
    let p = |x: f64, y: f64| P { x, y };
    let mut pts = vec![p(x1, y1 + g)];
    cubic(&mut pts, p(x1, y1 + g), p(x1, y1 + g - oy), p(xm - ox, y1), p(xm, y1));
    cubic(&mut pts, p(xm, y1), p(xm + ox, y1), p(x2, y1 + g - oy), p(x2, y1 + g));
    pts.push(p(x2, y2 - g));
    cubic(&mut pts, p(x2, y2 - g), p(x2, y2 - g + oy), p(xm + ox, y2), p(xm, y2));
    cubic(&mut pts, p(xm, y2), p(xm - ox, y2), p(x1, y2 - g + oy), p(x1, y2 - g));
    let mut arc = vec![p(x1, y1 + g)];
    cubic(&mut arc, p(x1, y1 + g), p(x1, y1 + g + oy), p(xm - ox, y1 + 2.0 * g), p(xm, y1 + 2.0 * g));
    cubic(&mut arc, p(xm, y1 + 2.0 * g), p(xm + ox, y1 + 2.0 * g), p(x2, y1 + g + oy), p(x2, y1 + g));
    (pts, arc)
}

/// Draw an element: a person's head over its body, a container's mark by its kind, else a box
/// rounded by 6; a boundary is dashed and not filled (drawObject and drawShadow of each view,
/// elements.js:245-500).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let Some(m) = r.refd(v, "model") else { return };
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let shadow = r.bool(v, "showShadow");
    let sc = Rgba(crate::render::SHADOW_COLOR.0, crate::render::SHADOW_COLOR.1, crate::render::SHADOW_COLOR.2, (255.0 * crate::render::SHADOW_ALPHA) as u8);
    let off = crate::render::SHADOW_OFFSET;
    let shift = |pts: &[P]| pts.iter().map(|p| P { x: p.x + off, y: p.y + off }).collect::<Vec<P>>();
    let seg = |c: &mut dyn Canvas, a: P, b2: P| c.polyline(&[a, b2], line, false);
    let p = |x: f64, y: f64| P { x, y };
    let kind = r.str(m, "kind");
    if is_boundary(r, v) {
        let mut pts = round_box(b, ROUND);
        pts.push(pts[0]);
        crate::render::dashes(c, &pts, line, 4.0, 4.0);
    } else if r.ty(m) == "C4Person" {
        let head = (b.w / 3.0).round();
        let margin = (b.w - head) / 2.0;
        let hc = p(x1 + margin + head / 2.0, y1 + head / 2.0);
        let body = R { x: x1, y: y1 + head + HEAD_BODY_GAP, w: b.w, h: (b.h - head - HEAD_BODY_GAP).max(0.0) };
        if shadow {
            c.ellipse(p(hc.x + off, hc.y + off), head / 2.0, head / 2.0, Some(sc), None);
            c.polygon(&shift(&round_box(body, ROUND)), Some(sc), None);
        }
        c.ellipse(hc, head / 2.0, head / 2.0, Some(fill), Some(line));
        c.polygon(&round_box(body, ROUND), Some(fill), Some(line));
    } else if r.ty(m) == "C4Container" && kind == "database" {
        let (pts, arc) = database(b);
        if shadow {
            c.polygon(&shift(&pts), Some(sc), None);
        }
        c.polygon(&pts, Some(fill), Some(line));
        c.polyline(&arc, line, false);
    } else {
        let pts = round_box(b, ROUND);
        if shadow {
            c.polygon(&shift(&pts), Some(sc), None);
        }
        c.polygon(&pts, Some(fill), Some(line));
        if r.ty(m) == "C4Container" {
            let size = views::view_font(r, v).1;
            match kind.as_str() {
                "client-webapp" => {
                    c.ellipse(p(x1 + 13.0, y1 + 11.0), 5.0, 5.0, None, Some(line));
                    c.ellipse(p(x1 + 27.0, y1 + 11.0), 5.0, 5.0, None, Some(line));
                    c.polygon(&round_box(R { x: x1 + 36.0, y: y1 + 6.0, w: (b.w - 44.0).max(0.0), h: 10.0 }, 5.0), None, Some(line));
                    seg(c, p(x1, y1 + 24.0), p(x2, y1 + 24.0));
                }
                "mobile-app" => {
                    let cx = ((x1 + x2) / 2.0).round();
                    seg(c, p(x1, y1 + 12.0), p(x2, y1 + 12.0));
                    seg(c, p(x1, y2 - 16.0), p(x2, y2 - 16.0));
                    c.ellipse(p(cx, y2 - 8.0), 4.0, 4.0, None, Some(line));
                }
                "desktop-app" => {
                    seg(c, p(x1, y1 + 15.0), p(x2, y1 + 15.0));
                    seg(c, p(x2 - 11.0, y1 + 4.0), p(x2 - 4.0, y1 + 11.0));
                    seg(c, p(x2 - 4.0, y1 + 4.0), p(x2 - 11.0, y1 + 11.0));
                }
                "serverless-function" | "console-app" | "shell-script" => {
                    let mark = match kind.as_str() {
                        "serverless-function" => "λ",
                        "console-app" => ">_",
                        _ => "#_",
                    };
                    c.text(mark, R { x: x1 + 6.0, y: y1 + 6.0, w: 30.0, h: size }, 0, 0, size, fc, false);
                    seg(c, p(x1, y1 + 24.0), p(x2, y1 + 24.0));
                }
                _ => {}
            }
        }
    }
    for f in ["nameLabel", "technologyLabel", "descriptionLabel"] {
        let Some(l) = r.refd(v, f) else { continue };
        if r.raw(l, "visible").and_then(|x| x.as_bool()) == Some(false) {
            continue;
        }
        let t = r.str(l, "text");
        if t.is_empty() {
            continue;
        }
        let font = r.str(l, "font");
        let mut it = font.split(';');
        let face = it.next().unwrap_or("Arial").to_string();
        let size = it.next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(13.0);
        let style = it.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
        let lb = views::node_rect(r, l);
        let align = r.val(l, "horizontalAlignment").and_then(|x| x.as_i64()).unwrap_or(2);
        let lines = if r.bool(l, "wordWrap") { views::wrap_text(&views::Scaled { ms: &*c, face, size }, &t, lb.w, style) } else { vec![t] };
        for (i, s) in lines.iter().enumerate() {
            c.text(s, R { x: lb.x, y: lb.y + i as f64 * size, w: lb.w, h: size }, align, style, size, fc, false);
        }
    }
}
