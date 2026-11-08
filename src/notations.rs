//! Data flow diagrams and mind maps: small notations whose nodes are boxes with a word-wrapped name
//! (extensions/essential/dfd and extensions/essential/mindmap: metamodel.json, elements.js,
//! *-factory.js, *-commands.js).

use crate::flowchart::{draw_label_in, round_box};
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;


/// Whether a model type is a node of a data flow diagram or a mind map.
pub fn is_node_type(ty: &str) -> bool {
    matches!(ty, "DFDExternalEntity" | "DFDProcess" | "DFDDataStore" | "MMNode")
}

/// Whether a view type is such a node's view.
pub fn is_node_view(ty: &str) -> bool {
    matches!(ty, "DFDExternalEntityView" | "DFDProcessView" | "DFDDataStoreView" | "MMNodeView")
}

/// The name prefix of a new element: DFD elements drop "DFD" (DFDElement.getDisplayClassName,
/// dfd/elements.js:42-45); mind map elements keep their whole class name ("MMNode1",
/// mindmap/elements.js:35-38).
pub fn name_base(ty: &str) -> &str {
    ty.strip_prefix("DFD").unwrap_or(ty)
}

/// A node's view: the box with its name label, and for data flow nodes an id label
/// (DFDGeneralNodeView, MMNodeView).
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
    let l = r.create("LabelView", &[("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(2)), ("wordWrap", json!(true))], Some((v, "subViews")));
    r.set_ref(v, "nameLabel", Some(l));
    if ty.starts_with("DFD") {
        let id = r.create("LabelView", &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(2))], Some((v, "subViews")));
        // only processes and data stores keep the field in the file (dfd/metamodel.json)
        if ty != "DFDExternalEntityView" {
            r.set_ref(v, "idLabel", Some(id));
        }
    }
    v
}

/// A data flow node's id label: the field, or the second sub-view of an external entity, whose
/// field the original does not save.
fn id_label(r: &Repo, v: Eid) -> Option<Eid> {
    r.refd(v, "idLabel").or_else(|| r.list(v, "subViews").into_iter().filter(|x| Some(*x) != r.refd(v, "nameLabel")).find(|x| r.ty(*x) == "LabelView"))
}

fn set_box(r: &mut Repo, l: Eid, x: f64, y: f64, w: f64, h: f64) {
    for (k, val) in [("left", x), ("top", y), ("width", w), ("height", h)] {
        r.set(l, k, json!(val));
    }
}

/// Size a node to its labels and place them (sizeObject and arrangeObject of the DFD views,
/// dfd/elements.js:130-330, and of MMNodeView, mindmap/elements.js:52-100).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let ty = r.ty(v).to_string();
    let m = r.refd(v, "model");
    let name = m.map(|m| r.name(m)).unwrap_or_default();
    let id = m.map(|m| r.str(m, "id")).unwrap_or_default();
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let lh = ms.size();
    let name_w = if wrap { views::widest_word(ms, &name, 0) } else { ms.width(&name, 0) }.ceil();
    let lines_at = |w: f64| if wrap { views::wrap_text(ms, &name, (w - 1.0).max(1.0), 0).len().max(1) } else { 1 } as f64 * lh;
    let id_w = ms.width(&id, 0).ceil();
    let (w0, h0) = (r.f(v, "width"), r.f(v, "height"));
    // the padding around the name: 10 px in a data flow node, 8 across and 6 down in a mind map node
    let (px, py) = if ty == "MMNodeView" { (8.0, 6.0) } else { (10.0, 10.0) };
    let id_h = if id.is_empty() { 0.0 } else { lh };
    let (min_w, min_h) = match ty.as_str() {
        "DFDExternalEntityView" => (name_w.max(id_w) + 2.0 * px, 2.0 * py + id_h + lines_at(w0 - 2.0 * px)),
        // the id compartment is 5 + id + 5 high
        "DFDProcessView" => (name_w.max(id_w) + 2.0 * px, 5.0 + lh + 5.0 + 2.0 * py + lines_at(w0 - 2.0 * px)),
        // the id sits in a compartment on the left: 5 + id + 5 wide
        "DFDDataStoreView" => (5.0 + id_w + 5.0 + name_w + 2.0 * px, 2.0 * py + lines_at(w0 - 20.0 - id_w - 2.0 * px)),
        _ => (name_w + 2.0 * px, lines_at(w0 - 2.0 * px) + 2.0 * py),
    };
    let auto = r.bool(v, "autoResize");
    let (w, h) = if auto { (min_w, min_h) } else { (w0.max(min_w), h0.max(min_h)) };
    r.set(v, "width", json!(w));
    r.set(v, "height", json!(h));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let font = r.str(v, "font");
    let font = if font.is_empty() { views::FONT.to_string() } else { font };
    let nl = r.refd(v, "nameLabel");
    if let Some(l) = nl {
        r.set(l, "text", json!(name));
        r.set(l, "wordWrap", json!(wrap));
        r.set(l, "font", json!(font));
    }
    let idl = id_label(r, v);
    if let Some(l) = idl {
        r.set(l, "text", json!(id));
        r.set(l, "font", json!(font));
    }
    match ty.as_str() {
        "DFDExternalEntityView" => {
            if let Some(l) = idl {
                set_box(r, l, x + px, y + 5.0, w - 2.0 * px, lh);
            }
            if let Some(l) = nl {
                set_box(r, l, x + px, y + py + id_h, w - 2.0 * px, lines_at(w - 2.0 * px));
            }
        }
        "DFDProcessView" => {
            if let Some(l) = idl {
                set_box(r, l, x + px, y + 5.0, w - 2.0 * px, lh);
            }
            if let Some(l) = nl {
                set_box(r, l, x + px, y + 5.0 + lh + 5.0 + py, w - 2.0 * px, lines_at(w - 2.0 * px));
            }
        }
        "DFDDataStoreView" => {
            if let Some(l) = idl {
                set_box(r, l, x + 5.0, y + py, id_w, lh);
            }
            let nx = 5.0 + id_w + 5.0 + px;
            if let Some(l) = nl {
                set_box(r, l, x + nx, y + py, w - nx - px, lines_at(w - nx - px));
            }
        }
        _ => {
            // a mind map node: the name across the whole box, centred down it, 2 px low
            if let Some(l) = nl {
                let lhh = lines_at(w - 2.0 * px);
                set_box(r, l, x, (y + h / 2.0).round() - (lhh / 2.0).round() + 2.0, w, lhh);
            }
        }
    }
}

/// Draw a node: an external entity is a box, a process a rounded box split under its id, a data
/// store a box open on the right with its id in a compartment on the left, a mind map node a box
/// rounded by 14 (drawObject and drawShadow of each view).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let ty = r.ty(v).to_string();
    let shadow = r.bool(v, "showShadow");
    let sc = Rgba(crate::render::SHADOW_COLOR.0, crate::render::SHADOW_COLOR.1, crate::render::SHADOW_COLOR.2, (255.0 * crate::render::SHADOW_ALPHA) as u8);
    let off = crate::render::SHADOW_OFFSET;
    let sb = R { x: b.x + off, y: b.y + off, ..b };
    let rect = |b: R| vec![P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }];
    match ty.as_str() {
        "DFDProcessView" | "MMNodeView" => {
            let rad = if ty == "MMNodeView" { 14.0 } else { 10.0 };
            if shadow {
                c.polygon(&round_box(sb, rad), Some(sc), None);
            }
            c.polygon(&round_box(b, rad), Some(fill), Some(line));
            if ty == "DFDProcessView" {
                let ih = id_label(r, v).map(|l| r.f(l, "height")).unwrap_or(0.0);
                let y = y1 + 5.0 + ih + 5.0;
                c.polyline(&[P { x: x1, y }, P { x: x2, y }], line, false);
            }
        }
        "DFDDataStoreView" => {
            if shadow {
                c.polygon(&rect(sb), Some(sc), None);
            }
            c.polygon(&rect(b), Some(fill), None);
            c.polyline(&[P { x: x2, y: y1 }, P { x: x1, y: y1 }, P { x: x1, y: y2 }, P { x: x2, y: y2 }], line, false);
            let iw = id_label(r, v).map(|l| r.f(l, "width")).unwrap_or(0.0);
            let x = x1 + 5.0 + iw + 5.0;
            c.polyline(&[P { x, y: y1 }, P { x, y: y2 }], line, false);
        }
        _ => {
            if shadow {
                c.polygon(&rect(sb), Some(sc), None);
            }
            c.polygon(&rect(b), Some(fill), Some(line));
        }
    }
    if let Some(l) = id_label(r, v) {
        draw_label_in(c, r, v, l, fc);
    }
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_label_in(c, r, v, l, fc);
    }
}
