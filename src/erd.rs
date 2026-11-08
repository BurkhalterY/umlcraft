//! Entity-relationship diagrams: entities with a compartment of columns (key, name and type in
//! three aligned columns) and relationships drawn with crow's feet by cardinality
//! (extensions/essential/erd: metamodel.json, elements.js, erd-factory.js, erd-commands.js).

use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Comp, Info, Measure, Row, P, R};
use serde_json::json;

const PAD: f64 = 5.0; // COMPARTMENT_*_PADDING (elements.js:203-206)
const GAP: f64 = 2.0; // COMPARTMENT_ITEM_INTERVAL

/// The cardinalities of a relationship end (quickedits/erd.json "erd.set-cardinality").
pub const CARDINALITIES: [(&str, &str); 4] = [("0..1", "Zero or One"), ("1", "One and Only One"), ("0..*", "Zero or Many"), ("1..*", "One or Many")];

/// A column's key text: PK, FK, then N (nullable) and U (unique) for non-key columns
/// (ERDColumn.getKeyString, elements.js:66-80).
pub fn key_text(r: &Repo, c: Eid) -> String {
    let mut k: Vec<&str> = vec![];
    let pk = r.bool(c, "primaryKey");
    if pk {
        k.push("PK");
    }
    if r.bool(c, "foreignKey") {
        k.push("FK");
    }
    if !pk && r.bool(c, "nullable") {
        k.push("N");
    }
    if !pk && r.bool(c, "unique") {
        k.push("U");
    }
    k.join(",")
}

fn length_of(r: &Repo, c: Eid) -> String {
    match r.raw(c, "length") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) if n.as_f64() != Some(0.0) => n.to_string(),
        _ => String::new(),
    }
}

/// A column's type text: `type(length)` (ERDColumn.getTypeString, elements.js:86-95).
pub fn type_text(r: &Repo, c: Eid) -> String {
    let len = length_of(r, c);
    let ty = r.str(c, "type");
    if len.is_empty() { ty } else { format!("{ty}({len})") }
}

/// What a column's inline edit shows: `name: type(length)` (ERDColumn.textualNotation,
/// elements.js:101-110).
pub fn column_text(r: &Repo, c: Eid) -> String {
    let name = r.name(c);
    let ty = r.str(c, "type");
    if ty.is_empty() {
        return name;
    }
    let len = length_of(r, c);
    if len.is_empty() { format!("{name}: {ty}") } else { format!("{name}: {ty}({len})") }
}

/// Apply a column expression `name : type(length)`: no type clears the type, and no length sets it
/// to 0, as the original does (handleSetColumnExpression, erd-commands.js:27-57).
pub fn apply_column(r: &mut Repo, c: Eid, text: &str) {
    let (name, rest) = match text.split_once(':') {
        Some((n, t)) => (n.trim(), Some(t.trim())),
        None => (text.trim(), None),
    };
    if !name.is_empty() {
        r.set(c, "name", json!(name));
    }
    let (ty, len) = match rest {
        Some(t) => match (t.find('('), t.rfind(')')) {
            (Some(a), Some(b)) if b > a => (t[..a].trim().to_string(), Some(t[a + 1..b].trim().to_string())),
            _ => (t.to_string(), None),
        },
        None => (String::new(), None),
    };
    r.set(c, "type", json!(ty));
    match len.filter(|l| !l.is_empty()) {
        Some(l) => r.set_raw(c, "length", json!(l)),
        None => r.set_raw(c, "length", json!(0)),
    }
}

/// An entity's view: its name label and its column compartment (ERDEntityView, elements.js:370-395).
pub fn create_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "ERDEntityView",
        &[
            ("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)),
            ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h)),
        ],
        Some((diagram, "ownedViews")),
    );
    let l = r.create("LabelView", &[("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(2))], Some((v, "subViews")));
    r.set_ref(v, "nameLabel", Some(l));
    let c = r.create("ERDColumnCompartmentView", &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true))], Some((v, "subViews")));
    r.set_ref(v, "columnCompartment", Some(c));
    v
}

/// The widths of the key, name and type columns of an entity's rows.
fn column_widths(ms: &dyn Measure, r: &Repo, cols: &[Eid]) -> (f64, f64, f64) {
    let mut w = (0.0f64, 0.0f64, 0.0f64);
    for &c in cols {
        w.0 = w.0.max(ms.width(&key_text(r, c), 0));
        w.1 = w.1.max(ms.width(&r.name(c), 0));
        w.2 = w.2.max(ms.width(&type_text(r, c), 0));
    }
    w
}

/// Where the name and type columns start in a row: past the key column, and past the name column
/// (ERDColumnCompartmentView.size, elements.js:270-300).
pub fn offsets(ms: &dyn Measure, r: &Repo, cols: &[Eid]) -> (f64, f64, f64) {
    let (kw, nw, tw) = column_widths(ms, r, cols);
    let name_off = kw + PAD;
    let type_off = name_off + PAD + nw + PAD;
    (name_off, type_off, type_off + PAD + tw)
}

/// Size an entity to its name and columns and place them; the columns are rows of its
/// compartment (ERDEntityView.sizeObject and arrangeObject, ERDColumnCompartmentView.update, size
/// and arrange, elements.js:230-420).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) -> Info {
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return Info::default() };
    let lh = ms.size();
    let name = r.name(m);
    let font = r.str(v, "font");
    let font = if font.is_empty() { views::FONT.to_string() } else { font };
    let cols: Vec<Eid> = r.list(m, "columns").into_iter().filter(|c| r.alive(*c)).collect();
    let comp = r.refd(v, "columnCompartment");
    let show = !r.bool(v, "suppressColumns");
    // one column view per column, in the columns' order
    let mut rows = vec![];
    if let Some(cv) = comp {
        let old = r.list(cv, "subViews");
        let mut keep = vec![];
        for &c in &cols {
            let iv = match old.iter().copied().find(|x| r.refd(*x, "model") == Some(c)) {
                Some(iv) => iv,
                None => {
                    let rf = r.ref_to(c);
                    r.create("ERDColumnView", &[("model", rf), ("font", json!(font.clone())), ("parentStyle", json!(true)), ("horizontalAlignment", json!(0))], Some((cv, "subViews")))
                }
            };
            keep.push(iv);
        }
        for x in old {
            if !keep.contains(&x) {
                r.detach(x);
            }
        }
        r.set_list(cv, "subViews", keep.clone());
        rows = keep;
        r.set_raw(cv, "visible", json!(show));
        if show {
            r.remove(cv, "visible");
        }
    }
    let (_, _, row_w) = offsets(ms, r, &cols);
    let comp_h = if cols.is_empty() { 2.0 * PAD } else { cols.len() as f64 * lh + (cols.len() - 1) as f64 * GAP + 2.0 * PAD };
    let name_w = ms.width(&name, 1).ceil();
    let mut min_w = name_w + 2.0 * PAD;
    let mut min_h = PAD + lh + PAD;
    if show && comp.is_some() {
        min_h += comp_h;
        min_w = min_w.max(row_w.ceil() + 2.0 * PAD);
    }
    let auto = r.bool(v, "autoResize");
    let (w, h) = if auto { (min_w, min_h) } else { (r.f(v, "width").max(min_w), r.f(v, "height").max(min_h)) };
    r.set(v, "width", json!(w));
    r.set(v, "height", json!(h));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    if let Some(l) = r.refd(v, "nameLabel") {
        r.set(l, "text", json!(name));
        r.set(l, "font", json!(views::font_str(&ms.face(), ms.size(), 1)));
        for (k, val) in [("left", x), ("top", y + PAD), ("width", w), ("height", lh)] {
            r.set(l, k, json!(val));
        }
    }
    let cy = y + PAD + lh + PAD;
    let mut info = Info { shape: "erd-entity", ..Default::default() };
    if let Some(cv) = comp {
        for (k, val) in [("left", x), ("top", cy), ("width", w), ("height", comp_h)] {
            r.set(cv, k, json!(val));
        }
        let mut c = Comp { y: cy, rows: vec![] };
        for (i, iv) in rows.into_iter().enumerate() {
            for (k, val) in [("left", x + PAD), ("top", cy + PAD + i as f64 * (lh + GAP)), ("width", w - 2.0 * PAD), ("height", lh)] {
                r.set(iv, k, json!(val));
            }
            r.set(iv, "font", json!(font.clone()));
            if show {
                c.rows.push(Row { iv });
            }
        }
        info.compartments.push(c);
    }
    info
}

/// Draw an entity: a box with a shadow, its bold name, and when it shows columns, a line over them
/// and lines between the key, name and type columns (ERDEntityView.drawObject, elements.js:420-440;
/// ERDColumnView.draw, elements.js:230-240).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let pts = |b: R| vec![P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }];
    if r.bool(v, "showShadow") {
        let s = Rgba(crate::render::SHADOW_COLOR.0, crate::render::SHADOW_COLOR.1, crate::render::SHADOW_COLOR.2, (255.0 * crate::render::SHADOW_ALPHA) as u8);
        let o = crate::render::SHADOW_OFFSET;
        c.polygon(&pts(R { x: b.x + o, y: b.y + o, ..b }), Some(s), None);
    }
    c.polygon(&pts(b), Some(fill), Some(line));
    let (face, size) = views::view_font(r, v);
    if let Some(l) = r.refd(v, "nameLabel") {
        let lb = views::node_rect(r, l);
        c.text(&r.str(l, "text"), lb, 2, 1, size, fc, false);
    }
    let Some(m) = r.refd(v, "model") else { return };
    let Some(cv) = r.refd(v, "columnCompartment") else { return };
    if r.raw(cv, "visible").and_then(|x| x.as_bool()) == Some(false) {
        return;
    }
    let rows: Vec<Eid> = r.list(cv, "subViews");
    if rows.is_empty() {
        return;
    }
    let cols: Vec<Eid> = r.list(m, "columns").into_iter().filter(|x| r.alive(*x)).collect();
    let sc = views::Scaled { ms: &*c, face, size };
    let (name_off, type_off, _) = offsets(&sc, r, &cols);
    let top = r.f(cv, "top");
    let (x2, y2) = (b.x + b.w, b.y + b.h);
    c.polyline(&[P { x: b.x, y: top }, P { x: x2, y: top }], line, false);
    for off in [name_off, type_off] {
        let x = b.x + PAD + off;
        c.polyline(&[P { x, y: top }, P { x, y: y2 }], line, false);
    }
    for iv in rows {
        let Some(col) = r.refd(iv, "model") else { continue };
        let rb = views::node_rect(r, iv);
        let cell = |x: f64, w: f64| R { x, y: rb.y, w, h: size };
        c.text(&key_text(r, col), cell(rb.x, name_off), 0, 0, size, fc, false);
        c.text(&r.name(col), cell(rb.x + name_off + PAD, type_off - name_off), 0, 0, size, fc, false);
        c.text(&type_text(r, col), cell(rb.x + type_off + PAD, (rb.w - type_off).max(0.0)), 0, 0, size, fc, false);
    }
}

/// The end of a relationship line by cardinality: a bar for "one" and a crow's foot for "many", a
/// circle for the optional ones (ERDRelationshipView.update, elements.js:520-560; EdgeView's
/// ES_CROWFOOT_* ends, core.js:2730-2915).
pub fn draw_end(c: &mut dyn Canvas, p0: P, p1: P, cardinality: &str, line: Rgba) {
    let th = (p1.y - p0.y).atan2(p1.x - p0.x);
    let (c2, c3) = (22.0, 7.0);
    let at = |o: P, len: f64, a: f64| P { x: o.x + len * a.cos(), y: o.y + len * a.sin() };
    let half = std::f64::consts::FRAC_PI_2;
    let p3 = at(p0, c2, th);
    let p8 = at(p0, c3, th);
    let (p9, p10) = (at(p8, c3, th - half), at(p8, c3, th + half));
    let p11 = at(p0, 2.0 * c3, th);
    let (p12, p13) = (at(p0, c3, th - half), at(p0, c3, th + half));
    let (p14, p15) = (at(p11, c3, th - half), at(p11, c3, th + half));
    let seg = |c: &mut dyn Canvas, a: P, b: P| c.polyline(&[a, b], line, false);
    match cardinality {
        "0..1" => {
            c.ellipse(p3, c3, c3, Some(crate::render::WHITE), Some(line));
            seg(c, p9, p10);
        }
        "0..*" => {
            seg(c, p11, p12);
            seg(c, p11, p13);
            c.ellipse(p3, c3, c3, Some(crate::render::WHITE), Some(line));
        }
        "1..*" => {
            seg(c, p11, p12);
            seg(c, p11, p13);
            seg(c, p14, p15);
        }
        _ => {
            seg(c, p9, p10);
            seg(c, p14, p15);
        }
    }
}
