//! Flowcharts: a flowchart owns its diagrams and its nodes; every node is a box with a word-wrapped
//! name label laid out inside its shape, and nodes are joined by flows with a filled arrow
//! (extensions/essential/flowchart: metamodel.json, elements.js, flowchart-factory.js).

use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;

/// Padding around a node's name label (elements.js:30-33).
const PAD: f64 = 10.0;

/// The flowchart tools in toolbox order: label, model type, and the name a new node gets
/// (toolbox/flowchart.json: "generate-name", or an empty "model-init" name; None: the type's own
/// name without "FC", numbered).
pub const NODES: [(&str, &str, Option<&str>); 28] = [
    ("Process", "FCProcess", None),
    ("Terminator", "FCTerminator", None),
    ("Decision", "FCDecision", None),
    ("Delay", "FCDelay", None),
    ("Predefined Process", "FCPredefinedProcess", None),
    ("Alternate Process", "FCAlternateProcess", None),
    ("Data", "FCData", None),
    ("Document", "FCDocument", None),
    ("Multi-Document", "FCMultiDocument", None),
    ("Preparation", "FCPreparation", None),
    ("Display", "FCDisplay", None),
    ("Manual Input", "FCManualInput", Some("Input")),
    ("Manual Operation", "FCManualOperation", Some("Operation")),
    ("Card", "FCCard", None),
    ("Punched Tape", "FCPunchedTape", None),
    ("Connector", "FCConnector", Some("C")),
    ("Off-Page Connector", "FCOffPageConnector", Some("P")),
    ("Or", "FCOr", Some("")),
    ("Summing Junction", "FCSummingJunction", Some("")),
    ("Collate", "FCCollate", Some("")),
    ("Sort", "FCSort", Some("")),
    ("Merge", "FCMerge", Some("")),
    ("Extract", "FCExtract", Some("")),
    ("Stored Data", "FCStoredData", None),
    ("Database", "FCDatabase", None),
    ("Direct Access Storage", "FCDirectAccessStorage", None),
    ("Internal Storage", "FCInternalStorage", None),
    ("Flow", "FCFlow", None),
];

/// Whether a model type is a flowchart node.
pub fn is_node_type(ty: &str) -> bool {
    NODES.iter().any(|n| n.1 == ty && ty != "FCFlow")
}

/// Whether a view type is a flowchart node's view.
pub fn is_node_view(ty: &str) -> bool {
    ty.starts_with("FC") && ty.ends_with("View") && ty != "FCFlowView" && is_node_type(&ty[..ty.len() - 4])
}

/// The name a new node gets: "Input", "Operation", "C" and "P" for some (numbered as usual), none
/// for the marks, else its type's name without "FC" (Process1, Decision1…).
pub fn name_base(ty: &str) -> Option<&'static str> {
    NODES.iter().find(|n| n.1 == ty).and_then(|n| n.2)
}

/// A node's view: the box with its name label (FCGeneralNodeView, elements.js:150-195).
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
    let l = r.create("LabelView", &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(2)), ("wordWrap", json!(true))], Some((v, "subViews")));
    r.set_ref(v, "nameLabel", Some(l));
    v
}

fn set(r: &mut Repo, v: Eid, k: &str, x: f64) {
    r.set(v, k, json!(x));
}

/// Size a node to its name and place the name label inside its shape (sizeObject and
/// arrangeObject of each FC*View, elements.js:150-1340).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let ty = r.ty(v).to_string();
    let name = r.refd(v, "model").map(|m| r.name(m)).unwrap_or_default();
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let lh = ms.size();
    // the label's smallest width and height: its widest word (or the whole text without word wrap),
    // and its height once wrapped to the width it gets
    let text_w = if wrap { views::widest_word(ms, &name, 0) } else { ms.width(&name, 0) }.ceil();
    let lines_at = |w: f64| if wrap { views::wrap_text(ms, &name, (w - 1.0).max(1.0), 0).len().max(1) } else { 1 } as f64 * lh;
    let marks = matches!(ty.as_str(), "FCOrView" | "FCSummingJunctionView" | "FCCollateView" | "FCSortView");
    let (w0, h0) = (r.f(v, "width"), r.f(v, "height"));
    let lmin_h = lines_at((w0 - 2.0 * PAD).max(text_w));
    let (min_w, min_h) = match ty.as_str() {
        "FCDecisionView" => ((text_w * 2.0).max(50.0), (lmin_h * 2.0).max(30.0)),
        "FCMergeView" | "FCExtractView" => ((text_w * 2.0).max(30.0), (lmin_h * 2.0).max(25.0)),
        "FCConnectorView" => (text_w.max(30.0), lmin_h.max(30.0)),
        _ if marks => (30.0, 30.0),
        _ => (text_w + 2.0 * PAD, lmin_h + 2.0 * PAD),
    };
    let auto = r.bool(v, "autoResize");
    let (mut w, mut h) = if auto { (min_w, min_h) } else { (w0.max(min_w), h0.max(min_h)) };
    // an or and a summing junction stay round
    if matches!(ty.as_str(), "FCOrView" | "FCSummingJunctionView") {
        let s = w.min(h);
        w = s;
        h = s;
    }
    set(r, v, "width", w);
    set(r, v, "height", h);
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let Some(l) = r.refd(v, "nameLabel") else { return };
    r.set(l, "text", json!(name));
    r.set(l, "wordWrap", json!(wrap));
    let font = r.str(v, "font");
    r.set(l, "font", json!(if font.is_empty() { views::FONT.to_string() } else { font }));
    if marks {
        r.set_raw(l, "visible", json!(false));
    } else {
        r.remove(l, "visible");
    }
    // the general box: 10 px in from each side, as high as its text
    let (mut lx, mut ly, mut lw) = (x + PAD, y + PAD, w - 2.0 * PAD);
    let mut lhh = lines_at(lw);
    let g6w = (w / 6.0).floor();
    let g8w = (w / 8.0).floor();
    match ty.as_str() {
        "FCDecisionView" | "FCConnectorView" => {
            (lx, ly, lw, lhh) = (x + w / 4.0, y + h / 4.0, w / 2.0, h / 2.0);
        }
        "FCMergeView" => (lx, ly, lw, lhh) = (x + w / 4.0, y + PAD, w / 2.0, h / 2.0),
        "FCExtractView" => (lx, ly, lw, lhh) = (x + w / 4.0, y + h / 2.0 - PAD, w / 2.0, h / 2.0),
        "FCDelayView" => lw = w * 0.75,
        "FCPredefinedProcessView" => (lx, lw) = (x + PAD + 5.0, w - 2.0 * PAD - 5.0),
        "FCDataView" | "FCPreparationView" | "FCDisplayView" | "FCManualOperationView" => (lx, lw) = (x + PAD + g6w, w - 2.0 * PAD - 2.0 * g6w),
        "FCDocumentView" => lhh = h - 2.0 * PAD - (h / 6.0).floor(),
        "FCMultiDocumentView" => (lx, ly, lw, lhh) = (x + PAD + 10.0, y + PAD + 10.0, w - 2.0 * PAD - 10.0, h - 2.0 * PAD - (h / 6.0).floor() - 10.0),
        "FCManualInputView" => {
            let g = (h / 6.0).floor();
            (ly, lhh) = (y + PAD + g, h - 2.0 * PAD - g);
        }
        "FCPunchedTapeView" => {
            let g = (h / 6.0).floor();
            (ly, lhh) = (y + PAD + g, h - 2.0 * PAD - 2.0 * g);
        }
        "FCOffPageConnectorView" => lhh = h - 2.0 * PAD - (h / 4.0).floor(),
        "FCStoredDataView" => (lx, lw) = (x + PAD + g8w, w - 2.0 * PAD - 2.0 * g8w),
        "FCDirectAccessStorageView" => (lx, lw) = (x + PAD + g8w, w - 2.0 * PAD - 3.0 * g8w),
        "FCDatabaseView" => {
            let g = (h / 8.0).floor();
            (ly, lhh) = (y + PAD + 2.0 * g, h - 2.0 * PAD - 3.0 * g);
        }
        "FCInternalStorageView" => (lx, ly, lw, lhh) = (x + 2.0 * PAD, y + 2.0 * PAD, w - 3.0 * PAD, h - 3.0 * PAD),
        _ => {}
    }
    set(r, l, "left", lx);
    set(r, l, "top", ly);
    set(r, l, "width", lw);
    set(r, l, "height", lhh);
}

pub(crate) fn cubic(out: &mut Vec<P>, p0: P, c1: P, c2: P, p3: P) {
    for i in 1..=16 {
        let t = i as f64 / 16.0;
        let u = 1.0 - t;
        let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
        out.push(P { x: a * p0.x + b * c1.x + c * c2.x + d * p3.x, y: a * p0.y + b * c1.y + c * c2.y + d * p3.y });
    }
}

fn quad(out: &mut Vec<P>, p0: P, c: P, p2: P) {
    for i in 1..=12 {
        let t = i as f64 / 12.0;
        let u = 1.0 - t;
        out.push(P { x: u * u * p0.x + 2.0 * u * t * c.x + t * t * p2.x, y: u * u * p0.y + 2.0 * u * t * c.y + t * t * p2.y });
    }
}

fn p(x: f64, y: f64) -> P {
    P { x, y }
}

/// A document's outline: a box whose bottom edge is a wave (FCDocumentView.drawDocument,
/// elements.js:540-565).
fn document(b: R) -> Vec<P> {
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let h = (b.h / 6.0).floor();
    let g = (b.w / 3.0).floor();
    let mut pts = vec![p(x1, y1), p(x2, y1), p(x2, y2)];
    cubic(&mut pts, p(x2, y2), p(x2 - g, y2 - h * 2.0), p(x1 + g, y2 + h * 1.5), p(x1, y2 - h));
    pts
}

/// A rectangle with rounded corners as a polygon.
pub(crate) fn round_box(b: R, rad: f64) -> Vec<P> {
    let rad = rad.min(b.w / 2.0).min(b.h / 2.0).max(0.0);
    let mut pts = vec![];
    let corners = [(b.x + b.w - rad, b.y + rad, -90.0f64), (b.x + b.w - rad, b.y + b.h - rad, 0.0), (b.x + rad, b.y + b.h - rad, 90.0), (b.x + rad, b.y + rad, 180.0)];
    for (cx, cy, start) in corners {
        for i in 0..=8 {
            let a = (start + 90.0 * i as f64 / 8.0).to_radians();
            pts.push(p(cx + rad * a.cos(), cy + rad * a.sin()));
        }
    }
    pts
}

/// Draw a node's shape and its name (drawObject of each FC*View, elements.js:330-1340).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let (xm, ym) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    let ty = r.ty(v).to_string();
    let shape = |c: &mut dyn Canvas, pts: &[P]| c.polygon(pts, Some(fill), Some(line));
    let seg = |c: &mut dyn Canvas, a: P, b2: P| c.polyline(&[a, b2], line, false);
    match ty.as_str() {
        "FCTerminatorView" => shape(c, &round_box(b, b.h.min(b.w) / 2.0)),
        "FCAlternateProcessView" => shape(c, &round_box(b, 10.0)),
        "FCDecisionView" | "FCSortView" => {
            shape(c, &[p(x1, ym), p(xm, y1), p(x2, ym), p(xm, y2)]);
            if ty == "FCSortView" {
                seg(c, p(x1, ym), p(x2, ym));
            }
        }
        "FCDelayView" => {
            let rad = b.h / 2.0;
            let mut pts = vec![p(x1, y1), p(x2 - rad, y1)];
            for i in 0..=16 {
                let a = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * i as f64 / 16.0;
                pts.push(p(x2 - rad + rad * a.cos(), y1 + rad + rad * a.sin()));
            }
            pts.push(p(x1, y2));
            shape(c, &pts);
        }
        "FCPredefinedProcessView" => {
            shape(c, &[p(x1, y1), p(x2, y1), p(x2, y2), p(x1, y2)]);
            seg(c, p(x1 + 5.0, y1), p(x1 + 5.0, y2));
            seg(c, p(x2 - 5.0, y1), p(x2 - 5.0, y2));
        }
        "FCDataView" => {
            let g = (b.w / 6.0).floor();
            shape(c, &[p(x1 + g, y1), p(x2, y1), p(x2 - g, y2), p(x1, y2)]);
        }
        "FCDocumentView" => shape(c, &document(b)),
        "FCMultiDocumentView" => {
            let g = 5.0;
            for (dx1, dy1, dx2, dy2) in [(0.0, 0.0, 2.0 * g, 2.0 * g), (g, g, g, g), (2.0 * g, 2.0 * g, 0.0, 0.0)] {
                shape(c, &document(R { x: x1 + dx1, y: y1 + dy1, w: b.w - dx1 - dx2, h: b.h - dy1 - dy2 }));
            }
        }
        "FCPreparationView" => {
            let g = (b.w / 6.0).floor();
            shape(c, &[p(x1, ym), p(x1 + g, y1), p(x2 - g, y1), p(x2, ym), p(x2 - g, y2), p(x1 + g, y2)]);
        }
        "FCDisplayView" => {
            let g = (b.w / 6.0).floor();
            let mut pts = vec![p(x1, ym), p(x1 + g, y1), p(x2 - g, y1)];
            quad(&mut pts, p(x2 - g, y1), p(x2, y1), p(x2, ym));
            quad(&mut pts, p(x2, ym), p(x2, y2), p(x2 - g, y2));
            pts.push(p(x1 + g, y2));
            shape(c, &pts);
        }
        "FCManualInputView" => {
            let g = (b.h / 6.0).floor();
            shape(c, &[p(x1, y1 + g), p(x2, y1), p(x2, y2), p(x1, y2)]);
        }
        "FCManualOperationView" => {
            let g = (b.w / 6.0).floor();
            shape(c, &[p(x1, y1), p(x2, y1), p(x2 - g, y2), p(x1 + g, y2)]);
        }
        "FCCardView" => {
            let g = (b.w / 8.0).floor();
            shape(c, &[p(x1 + g, y1), p(x2, y1), p(x2, y2), p(x1, y2), p(x1, y1 + g)]);
        }
        "FCPunchedTapeView" => {
            let h = (b.h / 6.0).floor();
            let g = (b.w / 3.0).floor();
            let mut pts = vec![p(x1, y1)];
            cubic(&mut pts, p(x1, y1), p(x1 + g, y1 + h * 2.0), p(x2 - g, y1 - h * 1.5), p(x2, y1 + h));
            pts.push(p(x2, y2));
            cubic(&mut pts, p(x2, y2), p(x2 - g, y2 - h * 2.0), p(x1 + g, y2 + h * 1.5), p(x1, y2 - h));
            shape(c, &pts);
        }
        "FCConnectorView" | "FCOrView" | "FCSummingJunctionView" => {
            c.ellipse(p(xm, ym), b.w / 2.0, b.h / 2.0, Some(fill), Some(line));
            if ty == "FCOrView" {
                seg(c, p(xm, y1), p(xm, y2));
                seg(c, p(x1, ym), p(x2, ym));
            } else if ty == "FCSummingJunctionView" {
                let d = (std::f64::consts::SQRT_2 * b.w / 4.0).round();
                seg(c, p(xm - d, ym - d), p(xm + d, ym + d));
                seg(c, p(xm + d, ym - d), p(xm - d, ym + d));
            }
        }
        "FCOffPageConnectorView" => {
            let g = (b.h / 4.0).floor();
            shape(c, &[p(x1, y1), p(x2, y1), p(x2, y2 - g), p(xm, y2), p(x1, y2 - g)]);
        }
        "FCCollateView" => shape(c, &[p(x1, y1), p(x2, y1), p(x1, y2), p(x2, y2)]),
        "FCMergeView" => shape(c, &[p(x1, y1), p(x2, y1), p(xm, y2)]),
        "FCExtractView" => shape(c, &[p(xm, y1), p(x2, y2), p(x1, y2)]),
        "FCStoredDataView" | "FCDirectAccessStorageView" => {
            // a drum on its side: curved ends made of quarter ellipses (kappa curves)
            let g = (b.w / 8.0).floor();
            let k = 0.5522848;
            let (ox, oy) = (g * k, b.h / 2.0 * k);
            let mut pts = vec![];
            if ty == "FCStoredDataView" {
                pts.push(p(x2, y1));
                cubic(&mut pts, p(x2, y1), p(x2 - ox, y1), p(x2 - g, ym - oy), p(x2 - g, ym));
                cubic(&mut pts, p(x2 - g, ym), p(x2 - g, ym + oy), p(x2 - ox, y2), p(x2, y2));
            } else {
                pts.push(p(x2 - g, y1));
                cubic(&mut pts, p(x2 - g, y1), p(x2 - g + ox, y1), p(x2, ym - oy), p(x2, ym));
                cubic(&mut pts, p(x2, ym), p(x2, ym + oy), p(x2 - g + ox, y2), p(x2 - g, y2));
            }
            pts.push(p(x1 + g, y2));
            cubic(&mut pts, p(x1 + g, y2), p(x1 + g - ox, y2), p(x1, ym + oy), p(x1, ym));
            cubic(&mut pts, p(x1, ym), p(x1, ym - oy), p(x1 + g - ox, y1), p(x1 + g, y1));
            shape(c, &pts);
            if ty == "FCDirectAccessStorageView" {
                let mut arc = vec![p(x2 - g, y1)];
                cubic(&mut arc, p(x2 - g, y1), p(x2 - g - ox, y1), p(x2 - 2.0 * g, ym - oy), p(x2 - 2.0 * g, ym));
                cubic(&mut arc, p(x2 - 2.0 * g, ym), p(x2 - 2.0 * g, ym + oy), p(x2 - g - ox, y2), p(x2 - g, y2));
                c.polyline(&arc, line, false);
            }
        }
        "FCDatabaseView" => {
            // a cylinder standing up
            let g = (b.h / 8.0).floor();
            let k = 0.5522848;
            let (ox, oy) = (b.w / 2.0 * k, g * k);
            let mut pts = vec![p(x1, y1 + g)];
            cubic(&mut pts, p(x1, y1 + g), p(x1, y1 + g - oy), p(xm - ox, y1), p(xm, y1));
            cubic(&mut pts, p(xm, y1), p(xm + ox, y1), p(x2, y1 + g - oy), p(x2, y1 + g));
            pts.push(p(x2, y2 - g));
            cubic(&mut pts, p(x2, y2 - g), p(x2, y2 - g + oy), p(xm + ox, y2), p(xm, y2));
            cubic(&mut pts, p(xm, y2), p(xm - ox, y2), p(x1, y2 - g + oy), p(x1, y2 - g));
            shape(c, &pts);
            let mut arc = vec![p(x1, y1 + g)];
            cubic(&mut arc, p(x1, y1 + g), p(x1, y1 + g + oy), p(xm - ox, y1 + 2.0 * g), p(xm, y1 + 2.0 * g));
            cubic(&mut arc, p(xm, y1 + 2.0 * g), p(xm + ox, y1 + 2.0 * g), p(x2, y1 + g + oy), p(x2, y1 + g));
            c.polyline(&arc, line, false);
        }
        "FCInternalStorageView" => {
            shape(c, &[p(x1, y1), p(x2, y1), p(x2, y2), p(x1, y2)]);
            seg(c, p(x1 + 10.0, y1), p(x1 + 10.0, y2));
            seg(c, p(x1, y1 + 10.0), p(x2, y1 + 10.0));
        }
        // a process, and any other node: a plain box
        _ => shape(c, &[p(x1, y1), p(x2, y1), p(x2, y2), p(x1, y2)]),
    }
    draw_name(c, r, v, fc);
}

/// The name label: its lines centred across the label's box and down it (LabelView with the
/// middle vertical alignment).
pub(crate) fn draw_name(c: &mut dyn Canvas, r: &Repo, v: Eid, fc: Rgba) {
    let Some(l) = r.refd(v, "nameLabel") else { return };
    draw_label_in(c, r, v, l, fc);
}

/// A label of a node: its lines centred across the label's box and down it.
pub(crate) fn draw_label_in(c: &mut dyn Canvas, r: &Repo, v: Eid, l: Eid, fc: Rgba) {
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
    let lines = if r.bool(l, "wordWrap") { views::wrap_text(&views::Scaled { ms: &*c, face, size }, &t, b.w, style) } else { vec![t] };
    let top = b.y + ((b.h - lines.len() as f64 * size) / 2.0).max(0.0);
    for (i, s) in lines.iter().enumerate() {
        c.text(s, R { x: b.x, y: top + i as f64 * size, w: b.w, h: size }, 2, style, size, fc, false);
    }
}

/// A node's preference id, as in `flowchart.<id>.fillColor`: its type's name in kebab case
/// ("predefined-process"), but "multidocument" (preferences/flowchart.json).
pub fn pref_id(ty: &str) -> String {
    let name = ty.strip_prefix("FC").unwrap_or(ty);
    if name == "MultiDocument" {
        return "multidocument".into();
    }
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('-');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}
