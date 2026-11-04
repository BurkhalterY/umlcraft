//! Creating views (boxes, compartments, labels, lines) with the structure the original writes,
//! and laying them out: sizes, label texts and line end points. Numbers: MECHANICS.md "Drawing".

use crate::model::{num, Eid, Repo};
use serde_json::{json, Value};
use std::collections::HashMap;

pub const PAD: f64 = 5.0; // compartment padding, elements.js:65
pub const ITEM_GAP: f64 = 2.0; // compartment item interval, elements.js:64
pub const NAME_MIN_H: f64 = 25.0; // name compartment minimum height (sample.mdj)
pub const EMPTY_H: f64 = 10.0; // empty list compartment height (sample.mdj)
pub const LABEL_H: f64 = 13.0;
pub const PKG_TAB_H: f64 = 15.0; // package tab height, elements.js:87
pub const PKG_TAB_RATIO: f64 = 0.4; // package tab width ratio, elements.js:6760
pub const PKG_MIN_W: f64 = 50.0; // elements.js:85
pub const PKG_MIN_H: f64 = 40.0; // elements.js:86
pub const UC_MIN_W: f64 = 70.0; // use case min width, elements.js:96
pub const UC_MIN_H: f64 = 32.0; // use case min height, elements.js:97
pub const ACTOR_ICON_W: f64 = 26.0; // actor icon min width, elements.js:93
pub const ACTOR_ICON_H: f64 = 55.0; // actor icon min height, elements.js:92
/// The robustness icons (boundary, entity, control) of a class shown as an icon (estimated).
pub const ROBUST_ICON_W: f64 = 50.0;
pub const ROBUST_ICON_H: f64 = 36.0;
/// An artifact shown as an icon (estimated).
pub const ARTIFACT_ICON_W: f64 = 30.0;
pub const ARTIFACT_ICON_H: f64 = 40.0;

/// The robustness kind of a class shown as an icon: "boundary", "entity" or "control".
pub fn robustness_kind(r: &Repo, v: Eid, m: Eid) -> Option<&'static str> {
    let display = r.str(v, "stereotypeDisplay");
    if r.ty(m) != "UMLClass" || !(display == "icon" || display == "icon-label") {
        return None;
    }
    match type_name(r, m, "stereotype").to_lowercase().as_str() {
        "boundary" => Some("boundary"),
        "entity" => Some("entity"),
        "control" => Some("control"),
        _ => None,
    }
}
pub const NOTE_FOLD: f64 = 10.0; // note fold, elements.js:14261
pub const FONT: &str = "Arial;13;0"; // default font
pub const DEFAULT_LINE_STYLE: i64 = 1; // oblique, default.json:83

/// Text measuring, supplied by the window (font style: 0 normal, 1 bold, 2 italic, 3 bold italic).
pub trait Measure {
    /// Width of a text in the diagram font at 13 px (style: 1 bold, 2 italic).
    fn width(&self, text: &str, style: u8) -> f64;
    /// The font size the layout is working with.
    fn size(&self) -> f64 {
        13.0
    }
    /// The font face the layout is working with.
    fn face(&self) -> String {
        "Arial".into()
    }
}

/// A view's own font: widths scale with its size, and labels inside it take its face and size, as
/// labels with parentStyle copy their parent's font (core.js:1659-1666).
pub struct Scaled<'a> {
    pub ms: &'a dyn Measure,
    pub face: String,
    pub size: f64,
}
impl Measure for Scaled<'_> {
    fn width(&self, text: &str, style: u8) -> f64 {
        self.ms.width(text, style) * self.size / 13.0
    }
    fn size(&self) -> f64 {
        self.size
    }
    fn face(&self) -> String {
        self.face.clone()
    }
}

/// Face and size of a view's `font` ("Arial;13;0").
pub fn view_font(r: &Repo, v: Eid) -> (String, f64) {
    let f = r.str(v, "font");
    let mut it = f.split(';');
    let face = it.next().filter(|x| !x.is_empty()).unwrap_or("Arial").to_string();
    let size = it.next().and_then(|x| x.parse::<f64>().ok()).filter(|x| *x > 0.0).unwrap_or(13.0);
    (face, size)
}

/// A font string "face;size;style", the size without decimals when whole.
pub fn font_str(face: &str, size: f64, style: i64) -> String {
    if size.fract() == 0.0 { format!("{face};{};{style}", size as i64) } else { format!("{face};{size};{style}") }
}

fn font_style_of(r: &Repo, v: Eid) -> i64 {
    r.str(v, "font").split(';').nth(2).and_then(|x| x.parse().ok()).unwrap_or(0)
}

/// Break a text into lines no wider than `width`, between words; a word wider than the line keeps a
/// line of its own. Newlines always break.
pub fn wrap_text(ms: &dyn Measure, text: &str, width: f64, style: u8) -> Vec<String> {
    let mut out = vec![];
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let test = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if ms.width(&test, style) > width && !line.is_empty() {
                out.push(std::mem::take(&mut line));
                line = word.to_string();
            } else {
                line = test;
            }
        }
        out.push(line);
    }
    out
}

/// The width of the widest word: the narrowest a word-wrapped label can be (core.js:3769-3790).
pub fn widest_word(ms: &dyn Measure, text: &str, style: u8) -> f64 {
    text.split([' ', '\n']).map(|w| ms.width(w, style)).fold(0.0, f64::max)
}

/// How many lines a label takes at this width: one, or as many as its words need with word wrap.
fn label_lines(r: &Repo, ms: &dyn Measure, lv: Eid, width: f64) -> usize {
    if r.bool(lv, "wordWrap") {
        wrap_text(ms, &r.str(lv, "text"), width, font_style_of(r, lv) as u8).len().max(1)
    } else {
        1
    }
}

/// Give a label the face and size of the layout, keeping its own style.
fn set_label_font(r: &mut Repo, ms: &dyn Measure, lv: Eid, style: Option<i64>) {
    let st = style.unwrap_or_else(|| font_style_of(r, lv));
    r.set(lv, "font", json!(font_str(&ms.face(), ms.size(), st)));
}

pub fn vis_symbol(v: &str) -> &'static str {
    match v {
        "public" => "+",
        "protected" => "#",
        "private" => "-",
        "package" => "~",
        _ => "",
    }
}

pub struct NodeKind {
    pub view: &'static str,
    pub classifier: bool,
    pub suppress: bool,
    pub icon_interface: bool,
}

pub fn node_kind(model_ty: &str) -> Option<NodeKind> {
    let k = |view, classifier, suppress, icon_interface| Some(NodeKind { view, classifier, suppress, icon_interface });
    match model_ty {
        "UMLClass" => k("UMLClassView", true, false, false),
        "UMLInterface" => k("UMLInterfaceView", true, true, true),
        "UMLDataType" => k("UMLDataTypeView", true, false, false),
        "UMLPrimitiveType" => k("UMLPrimitiveTypeView", true, false, false),
        "UMLSignal" => k("UMLSignalView", true, false, false),
        "UMLEnumeration" => k("UMLEnumerationView", true, true, false),
        "UMLPackage" | "UMLModel" => k("UMLPackageView", false, false, false),
        "UMLUseCase" => k("UMLUseCaseView", true, true, false),
        "UMLActor" => k("UMLActorView", true, true, false),
        "UMLUseCaseSubject" => k("UMLUseCaseSubjectView", false, false, false),
        // component and deployment diagrams (uml.json "uml.component", "uml.deployment")
        "UMLComponent" => k("UMLComponentView", true, true, false),
        "UMLArtifact" => k("UMLArtifactView", true, true, false),
        "UMLNode" => k("UMLNodeView", true, true, false),
        _ => None,
    }
}

pub fn edge_view_of(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "UMLAssociation" => "UMLAssociationView",
        "UMLGeneralization" => "UMLGeneralizationView",
        "UMLDependency" => "UMLDependencyView",
        "UMLRealization" => "UMLRealizationView",
        "UMLInterfaceRealization" => "UMLInterfaceRealizationView",
        "UMLInclude" => "UMLIncludeView",
        "UMLExtend" => "UMLExtendView",
        "UMLComponentRealization" => "UMLComponentRealizationView",
        "UMLDeployment" => "UMLDeploymentView",
        "UMLCommunicationPath" => "UMLCommunicationPathView",
        _ => return None,
    })
}

const KNOWN_EDGE_VIEWS: [&str; 12] = [
    "UMLAssociationView", "UMLGeneralizationView", "UMLDependencyView", "UMLRealizationView",
    "UMLInterfaceRealizationView", "UMLIncludeView", "UMLExtendView", "UMLNoteLinkView", "UMLContainmentView",
    "UMLComponentRealizationView", "UMLDeploymentView", "UMLCommunicationPathView",
];

/// Size used when the person clicks instead of dragging (estimated).
pub fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "UMLClass" => (160.0, 100.0),
        "UMLInterface" => (140.0, 80.0),
        "UMLEnumeration" | "UMLPackage" | "UMLModel" => (140.0, 100.0),
        "UMLDataType" | "UMLPrimitiveType" | "UMLSignal" => (160.0, 80.0),
        "UMLUseCase" => (120.0, 40.0),
        "UMLActor" => (40.0, 80.0),
        "UMLComponent" => (160.0, 60.0),
        "UMLArtifact" => (80.0, 70.0),
        "UMLNode" => (160.0, 80.0),
        "Boundary" | "Entity" | "Control" => (ROBUST_ICON_W, ROBUST_ICON_H + NAME_MIN_H),
        "UMLUseCaseSubject" => (240.0, 250.0),
        "Note" => (140.0, 60.0),
        "Text" => (60.0, 15.0),
        _ => (100.0, 60.0),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct R {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct P {
    pub x: f64,
    pub y: f64,
}

pub fn node_rect(r: &Repo, v: Eid) -> R {
    R { x: r.f(v, "left"), y: r.f(v, "top"), w: r.f(v, "width"), h: r.f(v, "height") }
}

pub fn is_edge(r: &Repo, v: Eid) -> bool {
    r.has(v, "points") || (r.has(v, "head") && r.has(v, "tail"))
}
pub fn is_node(r: &Repo, v: Eid) -> bool {
    !is_edge(r, v) && (r.has(v, "left") || r.has(v, "width"))
}

pub fn parse_points(s: &str) -> Vec<P> {
    s.split(';')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut it = p.split(':').map(|x| x.trim().parse::<f64>().unwrap_or(0.0));
            P { x: it.next().unwrap_or(0.0), y: it.next().unwrap_or(0.0) }
        })
        .collect()
}
pub fn points_of(r: &Repo, e: Eid) -> Vec<P> {
    parse_points(&r.str(e, "points"))
}
fn fmt_num(f: f64) -> String {
    match num(f) {
        Value::Number(n) => n.to_string(),
        _ => "0".into(),
    }
}
pub fn format_points(pts: &[P]) -> String {
    pts.iter().map(|p| format!("{}:{}", fmt_num(p.x), fmt_num(p.y))).collect::<Vec<_>>().join(";")
}

// ------------------------------------------------------------------ creation
fn label(r: &mut Repo, parent: Eid, props: &[(&str, Value)]) -> Eid {
    let mut all: Vec<(&str, Value)> = vec![("font", json!(FONT)), ("parentStyle", json!(true))];
    for (k, v) in props {
        if let Some(slot) = all.iter_mut().find(|(n, _)| n == k) {
            slot.1 = v.clone();
        } else {
            all.push((k, v.clone()));
        }
    }
    r.create("LabelView", &all, Some((parent, "subViews")))
}

fn set_false(r: &mut Repo, e: Eid, key: &str) {
    r.set_raw(e, key, json!(false));
}

fn name_compartment(r: &mut Repo, view: Eid, model: Eid) -> Eid {
    let m = r.ref_to(model);
    let nc = r.create("UMLNameCompartmentView", &[("model", m), ("font", json!(FONT)), ("parentStyle", json!(true))], Some((view, "subViews")));
    let st = label(r, nc, &[("height", json!(LABEL_H))]);
    set_false(r, st, "visible");
    let nm = label(r, nc, &[("font", json!("Arial;13;1")), ("height", json!(LABEL_H))]);
    let ns = label(r, nc, &[("height", json!(LABEL_H))]);
    set_false(r, ns, "visible");
    let pr = label(r, nc, &[("height", json!(LABEL_H)), ("horizontalAlignment", json!(1))]);
    set_false(r, pr, "visible");
    r.set_ref(nc, "stereotypeLabel", Some(st));
    r.set_ref(nc, "nameLabel", Some(nm));
    r.set_ref(nc, "namespaceLabel", Some(ns));
    r.set_ref(nc, "propertyLabel", Some(pr));
    nc
}

fn list_compartment(r: &mut Repo, view: Eid, model: Eid, ty: &str, visible: bool) -> Eid {
    let m = r.ref_to(model);
    let c = r.create(ty, &[("model", m), ("font", json!(FONT)), ("parentStyle", json!(true)), ("width", json!(10)), ("height", json!(10))], Some((view, "subViews")));
    if !visible {
        set_false(r, c, "visible");
    }
    c
}

pub fn create_node_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Option<Eid> {
    let kind = node_kind(r.ty(model))?;
    let m = r.ref_to(model);
    let changeable = r.ty(model) != "UMLUseCaseSubject";
    let v = r.create(
        kind.view,
        &[
            ("model", m), ("font", json!(FONT)), ("parentStyle", json!(false)), ("containerChangeable", json!(changeable)),
            ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h)),
        ],
        Some((diagram, "ownedViews")),
    );
    if kind.icon_interface {
        r.set(v, "stereotypeDisplay", json!("icon"));
    }
    if kind.suppress {
        r.set(v, "suppressAttributes", json!(true));
        r.set(v, "suppressOperations", json!(true));
    }
    let nc = name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    if kind.classifier {
        let a = list_compartment(r, v, model, "UMLAttributeCompartmentView", !kind.suppress);
        r.set_ref(v, "attributeCompartment", Some(a));
        let o = list_compartment(r, v, model, "UMLOperationCompartmentView", !kind.suppress);
        r.set_ref(v, "operationCompartment", Some(o));
        let rc = list_compartment(r, v, model, "UMLReceptionCompartmentView", false);
        r.set_ref(v, "receptionCompartment", Some(rc));
        let tp = list_compartment(r, v, model, "UMLTemplateParameterCompartmentView", false);
        r.set_ref(v, "templateParameterCompartment", Some(tp));
        if r.ty(model) == "UMLEnumeration" {
            let l = list_compartment(r, v, model, "UMLEnumerationLiteralCompartmentView", true);
            r.set_ref(v, "enumerationLiteralCompartment", Some(l));
        }
        if r.ty(model) == "UMLUseCase" {
            let x = list_compartment(r, v, model, "UMLExtensionPointCompartmentView", false);
            r.set_ref(v, "extensionPointCompartment", Some(x));
        }
    }
    Some(v)
}

pub fn create_note_view(r: &mut Repo, diagram: Eid, rect: R, ty: &str) -> Eid {
    r.create(
        ty,
        &[("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))],
        Some((diagram, "ownedViews")),
    )
}

const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;

fn edge_label(r: &mut Repo, edge: Eid, model: Option<Eid>, alpha: f64, distance: f64, pos: i64, visible_null: bool) -> Eid {
    let mut props: Vec<(&str, Value)> = vec![];
    if let Some(m) = model {
        props.push(("model", r.ref_to(m)));
    }
    let l = r.create("EdgeLabelView", &props, Some((edge, "subViews")));
    if visible_null {
        r.set_raw(l, "visible", Value::Null);
    } else {
        r.set_raw(l, "visible", json!(false));
    }
    r.set(l, "font", json!(FONT));
    r.set(l, "parentStyle", json!(false));
    r.set(l, "height", json!(LABEL_H));
    r.set(l, "alpha", json!(alpha));
    r.set(l, "distance", num(distance));
    r.set_ref(l, "hostEdge", Some(edge));
    r.set(l, "edgePosition", json!(pos));
    l
}

pub fn create_edge_view(r: &mut Repo, diagram: Eid, model: Option<Eid>, tail: Eid, head: Eid) -> Option<Eid> {
    let ty = match model {
        Some(m) => edge_view_of(r.ty(m))?,
        None => "UMLNoteLinkView",
    };
    let mut props: Vec<(&str, Value)> = vec![];
    if let Some(m) = model {
        props.push(("model", r.ref_to(m)));
    }
    props.push(("font", json!(FONT)));
    props.push(("parentStyle", json!(false)));
    props.push(("head", r.ref_to(head)));
    props.push(("tail", r.ref_to(tail)));
    props.push(("lineStyle", json!(DEFAULT_LINE_STYLE)));
    props.push(("points", json!("0:0;0:0")));
    let e = r.create(ty, &props, Some((diagram, "ownedViews")));
    let Some(m) = model else { return Some(e) };
    r.set(e, "showVisibility", json!(true));
    let nl = edge_label(r, e, Some(m), HALF_PI, 15.0, 1, false);
    let sl = edge_label(r, e, Some(m), HALF_PI, 30.0, 1, true);
    let pl = edge_label(r, e, Some(m), -HALF_PI, 15.0, 1, false);
    r.set_ref(e, "nameLabel", Some(nl));
    r.set_ref(e, "stereotypeLabel", Some(sl));
    r.set_ref(e, "propertyLabel", Some(pl));
    if r.kind_of(m, "UMLAssociation") {
        r.set_raw(e, "showEndOrder", json!("hide")); // as the original writes it (sample.mdj)
        let end1 = r.child(m, "end1");
        let end2 = r.child(m, "end2");
        let p6 = std::f64::consts::FRAC_PI_6;
        let p4 = std::f64::consts::FRAC_PI_4;
        let specs = [
            ("tailRoleNameLabel", end1, p6, 30.0, 2),
            ("tailPropertyLabel", end1, p4, 40.0, 2),
            ("tailMultiplicityLabel", end1, -p6, 25.0, 2),
            ("headRoleNameLabel", end2, -p6, 30.0, 0),
            ("headPropertyLabel", end2, -p4, 40.0, 0),
            ("headMultiplicityLabel", end2, p6, 25.0, 0),
        ];
        for (field, end, a, d, pos) in specs {
            let l = edge_label(r, e, end, a, d, pos, false);
            r.set_ref(e, field, Some(l));
        }
        for (field, end) in [("tailQualifiersCompartment", end1), ("headQualifiersCompartment", end2)] {
            let mut p: Vec<(&str, Value)> = vec![];
            if let Some(x) = end {
                p.push(("model", r.ref_to(x)));
            }
            let q = r.create("UMLQualifierCompartmentView", &p, Some((e, "subViews")));
            r.set_raw(q, "visible", json!(false));
            r.set(q, "font", json!(FONT));
            r.set(q, "parentStyle", json!(true));
            r.set(q, "width", json!(10));
            r.set(q, "height", json!(10));
            r.set_ref(e, field, Some(q));
        }
    }
    Some(e)
}

// ------------------------------------------------------------------ text formats (elements.js:375, 870, 957)
pub fn type_name(r: &Repo, e: Eid, key: &str) -> String {
    match r.raw(e, key) {
        Some(Value::String(s)) => s.clone(),
        Some(v) => r.by_ref(v).map(|t| r.name(t)).unwrap_or_default(),
        None => String::new(),
    }
}
pub fn stereotype_text(r: &Repo, e: Eid) -> String {
    let n = type_name(r, e, "stereotype");
    if n.is_empty() { n } else { format!("«{n}»") }
}
/// Which parts of a feature's text a view shows (its showVisibility, showMultiplicity,
/// showOperationSignature, showType and showProperty flags).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Show {
    pub vis: bool,
    pub mult: bool,
    pub sig: bool,
    pub ty: bool,
    pub prop: bool,
}

impl Show {
    pub const ALL: Show = Show { vis: true, mult: true, sig: true, ty: true, prop: true };

    /// The flags of a view; a flag the view does not have counts as on.
    pub fn of(r: &Repo, v: Eid) -> Show {
        let f = |k: &str| r.val(v, k).and_then(|x| x.as_bool()).unwrap_or(true);
        Show { vis: f("showVisibility"), mult: f("showMultiplicity"), sig: f("showOperationSignature"), ty: f("showType"), prop: f("showProperty") }
    }
}

/// The property string of an attribute or association end: `{id, readOnly, ordered, unique}`, or "".
pub fn property_text(r: &Repo, a: Eid) -> String {
    let props: Vec<&str> = [("isID", "id"), ("isReadOnly", "readOnly"), ("isOrdered", "ordered"), ("isUnique", "unique")].into_iter().filter(|(k, _)| r.bool(a, k)).map(|(_, l)| l).collect();
    if props.is_empty() { String::new() } else { format!("{{{}}}", props.join(", ")) }
}

pub fn attribute_text(r: &Repo, a: Eid, show: Show) -> String {
    let mut s = stereotype_text(r, a);
    if !s.is_empty() {
        s.push(' ');
    }
    if show.vis {
        s += vis_symbol(&r.str(a, "visibility"));
    }
    if r.bool(a, "isDerived") {
        s.push('/');
    }
    s += &r.name(a);
    let t = type_name(r, a, "type");
    if show.ty && !t.is_empty() {
        s += &format!(": {t}");
    }
    let m = r.str(a, "multiplicity");
    if show.mult && !m.is_empty() {
        s += &format!("[{m}]");
    }
    let d = r.str(a, "defaultValue");
    if !d.is_empty() {
        s += &format!(" = {d}");
    }
    let p = property_text(r, a);
    if show.prop && !p.is_empty() {
        s += &format!(" {p}");
    }
    s
}
/// How far the template parameter box reaches over the class's right and top edges (estimated).
pub const TEMPLATE_OUT: f64 = 10.0;

/// A template parameter's text: «stereotype» name: type = default.
pub fn template_parameter_text(r: &Repo, t: Eid) -> String {
    let mut s = stereotype_text(r, t);
    if !s.is_empty() {
        s.push(' ');
    }
    s += &r.name(t);
    let ty = r.str(t, "parameterType");
    if !ty.is_empty() {
        s += &format!(": {ty}");
    }
    let d = r.str(t, "defaultValue");
    if !d.is_empty() {
        s += &format!(" = {d}");
    }
    s
}

pub fn operation_text(r: &Repo, o: Eid, show: Show) -> String {
    let mut s = stereotype_text(r, o);
    if !s.is_empty() {
        s.push(' ');
    }
    if show.vis {
        s += vis_symbol(&r.str(o, "visibility"));
    }
    s += &r.name(o);
    if !show.sig {
        return s + "()";
    }
    let params = r.list(o, "parameters");
    let ret = params.iter().copied().find(|p| r.str(*p, "direction") == "return");
    let list: Vec<String> = params
        .iter()
        .copied()
        .filter(|p| r.str(*p, "direction") != "return")
        .map(|p| {
            let mut t = String::new();
            let dir = r.str(p, "direction");
            if dir == "inout" || dir == "out" {
                t += &format!("{dir} ");
            }
            t += &r.name(p);
            let ty = type_name(r, p, "type");
            if show.ty && !ty.is_empty() {
                t += &format!(": {ty}");
            }
            let m = r.str(p, "multiplicity");
            if show.mult && !m.is_empty() {
                t += &format!("[{m}]");
            }
            let d = r.str(p, "defaultValue");
            if !d.is_empty() {
                t += &format!(" = {d}");
            }
            t
        })
        .collect();
    s += &format!("({})", list.join(", "));
    if let Some(rp) = ret.filter(|_| show.ty) {
        let ty = type_name(r, rp, "type");
        if !ty.is_empty() {
            s += &format!(": {ty}");
        }
        let m = r.str(rp, "multiplicity");
        if show.mult && !m.is_empty() {
            s += &format!("[{m}]");
        }
    }
    s
}

// ------------------------------------------------------------------ layout
#[derive(Clone, Debug, Default)]
pub struct Row {
    pub iv: Eid,
}
#[derive(Clone, Debug, Default)]
pub struct Comp {
    pub y: f64,
    pub rows: Vec<Row>,
}
#[derive(Clone, Debug, Default)]
pub struct Info {
    /// The template parameter box at the top-right corner, when there are template parameters.
    pub templ: Option<Comp>,
    /// "interface", "actor", "usecase", "package", "subject" or ""
    pub shape: &'static str,
    pub compartments: Vec<Comp>,
    pub is_active: bool,
}

fn set_box(r: &mut Repo, v: Eid, x: f64, y: f64, w: f64, h: f64) {
    r.set(v, "left", num(x));
    r.set(v, "top", num(y));
    r.set(v, "width", num(w));
    r.set(v, "height", num(h));
}

struct NameParts {
    nc: Eid,
    st: Option<Eid>,
    nm: Option<Eid>,
    ns: Option<Eid>,
    st_text: String,
    show_st: bool,
    ns_text: String,
}

fn name_parts(r: &Repo, v: Eid, m: Eid, keyword: Option<&str>, hide_st: bool) -> Option<NameParts> {
    let nc = r.refd(v, "nameCompartment")?;
    let display = r.str(v, "stereotypeDisplay");
    let mut st_text = stereotype_text(r, m);
    if st_text.is_empty() {
        if let Some(k) = keyword {
            st_text = format!("«{k}»");
        }
    }
    let show_st = !st_text.is_empty() && (display == "label" || display == "decoration-label") && !hide_st;
    let parent = r.parent(m).map(|p| r.name(p)).unwrap_or_default();
    Some(NameParts {
        nc,
        st: r.refd(nc, "stereotypeLabel"),
        nm: r.refd(nc, "nameLabel"),
        ns: r.refd(nc, "namespaceLabel"),
        st_text,
        show_st,
        ns_text: format!("(from {parent})"),
    })
}

fn name_style(r: &Repo, m: Eid) -> u8 {
    if r.bool(m, "isAbstract") { 3 } else { 1 }
}

fn layout_name(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid, x: f64, y: f64, w: f64, keyword: Option<&str>, hide_st: bool) -> f64 {
    let Some(p) = name_parts(r, v, m, keyword, hide_st) else { return NAME_MIN_H };
    let style = name_style(r, m);
    let wrap = r.bool(v, "wordWrap");
    let mut items: Vec<Eid> = vec![];
    let lh = ms.size();
    if let Some(st) = p.st {
        set_label_font(r, ms, st, None);
        r.set(st, "text", json!(p.st_text));
        r.set_raw(st, "visible", json!(p.show_st));
        if p.show_st {
            r.remove(st, "visible");
            items.push(st);
        }
    }
    if let Some(nm) = p.nm {
        let name = r.name(m);
        r.set(nm, "text", json!(name));
        set_label_font(r, ms, nm, Some(style as i64));
        items.push(nm);
    }
    if let Some(ns) = p.ns {
        set_label_font(r, ms, ns, None);
        let show = r.bool(v, "showNamespace");
        r.set(ns, "text", json!(p.ns_text));
        if show {
            r.remove(ns, "visible");
            items.push(ns);
        } else {
            r.set_raw(ns, "visible", json!(false));
        }
    }
    for lv in &items {
        r.set(*lv, "wordWrap", json!(wrap));
    }
    let heights: Vec<f64> = items.iter().map(|lv| label_lines(r, ms, *lv, w - 2.0 * PAD) as f64 * lh).collect();
    let n = items.len() as f64;
    let content = heights.iter().sum::<f64>() + ITEM_GAP * (n - 1.0).max(0.0);
    let h = NAME_MIN_H.max(content + 2.0 * PAD);
    let mut cy = y + ((h - content) / 2.0).ceil();
    for (lv, lh) in items.into_iter().zip(heights) {
        set_box(r, lv, x + PAD, cy, w - 2.0 * PAD, lh);
        cy += lh + ITEM_GAP;
    }
    set_box(r, p.nc, x, y, w + 1.0, h);
    h
}

fn name_min_width(r: &Repo, ms: &dyn Measure, v: Eid, m: Eid, keyword: Option<&str>, hide_st: bool) -> f64 {
    let wrap = r.bool(v, "wordWrap");
    let width = |t: &str, style: u8| if wrap { widest_word(ms, t, style) } else { ms.width(t, style) };
    let mut w = width(&r.name(m), name_style(r, m));
    if let Some(p) = name_parts(r, v, m, keyword, hide_st) {
        if p.show_st {
            w = w.max(width(&p.st_text, 0));
        }
    }
    w.ceil() + 2.0 * PAD
}

/// Keep a list compartment's item views in step with the model list; returns (min width, height, rows).
fn sync_list(r: &mut Repo, ms: &dyn Measure, comp: Eid, items: &[Eid], item_ty: &str, texts: &[String], wrap: bool) -> (f64, f64, Vec<Row>) {
    let existing = r.list(comp, "subViews");
    let mut keep = vec![];
    for it in items {
        let id = r.id(*it);
        let found = existing.iter().copied().find(|x| r.raw(*x, "model").and_then(|m| m.get("$ref")).and_then(|s| s.as_str()) == Some(id.as_str()));
        let iv = match found {
            Some(x) => x,
            None => {
                let mref = r.ref_to(*it);
                r.create(item_ty, &[("model", mref), ("font", json!(FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(0))], Some((comp, "subViews")))
            }
        };
        keep.push(iv);
    }
    for x in existing {
        if !keep.contains(&x) && r.alive(x) {
            r.detach(x);
        }
    }
    if !keep.is_empty() {
        r.set_list(comp, "subViews", keep.clone());
    }
    let mut min_w: f64 = 0.0;
    for (i, iv) in keep.iter().enumerate() {
        let t = &texts[i];
        r.set(*iv, "text", json!(t));
        let st = r.bool(items[i], "isStatic");
        r.set(*iv, "underline", json!(st));
        let italic = if r.bool(items[i], "isAbstract") { 2 } else { 0 };
        set_label_font(r, ms, *iv, None);
        r.set(*iv, "wordWrap", json!(wrap));
        let tw = if wrap { widest_word(ms, t, italic) } else { ms.width(t, italic) };
        min_w = min_w.max(tw.ceil() + 2.0 * PAD);
    }
    let rows: Vec<Row> = keep.into_iter().map(|iv| Row { iv }).collect();
    let h = list_height(r, ms, &rows, f64::INFINITY);
    (min_w, h, rows)
}

/// The height of a list compartment `w` wide: its rows, with word-wrapped rows on several lines.
fn list_height(r: &Repo, ms: &dyn Measure, rows: &[Row], w: f64) -> f64 {
    if rows.is_empty() {
        return EMPTY_H;
    }
    let lines: usize = rows.iter().map(|row| label_lines(r, ms, row.iv, w - 2.0 * PAD + 1.0)).sum();
    2.0 * PAD + lines as f64 * ms.size() + ITEM_GAP * (rows.len() as f64 - 1.0)
}

fn place_list(r: &mut Repo, ms: &dyn Measure, comp: Eid, x: f64, y: f64, w: f64, h: f64, rows: &[Row]) {
    set_box(r, comp, x, y, w + 1.0, h);
    let mut cy = y + PAD;
    for row in rows {
        let lh = label_lines(r, ms, row.iv, w - 2.0 * PAD + 1.0) as f64 * ms.size();
        set_box(r, row.iv, x + PAD, cy, w - 2.0 * PAD + 1.0, lh);
        cy += lh + ITEM_GAP;
    }
}

fn hide_comp(r: &mut Repo, comp: Option<Eid>) {
    let Some(c) = comp else { return };
    r.set_raw(c, "visible", json!(false));
    r.remove(c, "left");
    r.remove(c, "top");
    r.set(c, "width", json!(10));
    r.set(c, "height", json!(10));
    for s in r.list(c, "subViews") {
        r.detach(s);
    }
}

fn keyword_of(ty: &str) -> Option<&'static str> {
    match ty {
        "UMLInterface" => Some("interface"),
        "UMLEnumeration" => Some("enumeration"),
        "UMLDataType" => Some("dataType"),
        "UMLPrimitiveType" => Some("primitive"),
        "UMLSignal" => Some("signal"),
        _ => None,
    }
}

/// Grow a box to its content; with autoResize on it takes exactly its content size (core.js:2115-2125).
fn grow(r: &mut Repo, v: Eid, min_w: f64, min_h: f64) {
    let auto = r.bool(v, "autoResize");
    let w = if auto { min_w } else { r.f(v, "width").max(min_w) };
    let h = if auto { min_h } else { r.f(v, "height").max(min_h) };
    r.set(v, "width", num(w));
    r.set(v, "height", num(h));
}

fn layout_classifier(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) -> Info {
    let ty = r.ty(m).to_string();
    let display = r.str(v, "stereotypeDisplay");
    let keyword = keyword_of(&ty);
    let comps_all = ["attributeCompartment", "operationCompartment", "receptionCompartment", "templateParameterCompartment", "extensionPointCompartment"];
    let hide_all = |r: &mut Repo| {
        for f in comps_all {
            let c = r.refd(v, f);
            hide_comp(r, c);
        }
    };

    if ty == "UMLInterface" && (display == "icon" || display == "icon-label") {
        hide_all(r);
        let min_w = name_min_width(r, ms, v, m, keyword, true).max(24.0);
        grow(r, v, min_w, NAME_MIN_H + 24.0);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        layout_name(r, ms, v, m, x, y + h - NAME_MIN_H, w, keyword, true);
        return Info { shape: "interface", ..Default::default() };
    }
    // an artifact in icon display: a page with a folded corner, the name under it (size estimated)
    if ty == "UMLArtifact" && (display == "icon" || display == "icon-label") {
        hide_all(r);
        let hide_st = display == "icon";
        let min_w = name_min_width(r, ms, v, m, None, hide_st).max(ARTIFACT_ICON_W);
        grow(r, v, min_w, NAME_MIN_H + ARTIFACT_ICON_H);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        layout_name(r, ms, v, m, x, y + h - NAME_MIN_H, w, None, hide_st);
        return Info { shape: "artifact", ..Default::default() };
    }
    // a boundary, entity or control class as its robustness icon, the name under it
    if let Some(kind) = robustness_kind(r, v, m) {
        hide_all(r);
        let hide_st = display == "icon";
        let min_w = name_min_width(r, ms, v, m, None, hide_st).max(ROBUST_ICON_W);
        grow(r, v, min_w, NAME_MIN_H + ROBUST_ICON_H);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        layout_name(r, ms, v, m, x, y + h - NAME_MIN_H, w, None, hide_st);
        return Info { shape: kind, ..Default::default() };
    }
    if ty == "UMLActor" {
        hide_all(r);
        let min_w = name_min_width(r, ms, v, m, None, false).max(ACTOR_ICON_W);
        grow(r, v, min_w, NAME_MIN_H + ACTOR_ICON_H);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        layout_name(r, ms, v, m, x, y + h - NAME_MIN_H, w, None, false);
        return Info { shape: "actor", ..Default::default() };
    }
    if ty == "UMLUseCase" {
        hide_all(r);
        let nw = name_min_width(r, ms, v, m, None, false);
        let min_w = (std::f64::consts::SQRT_2 * nw).floor().max(UC_MIN_W);
        let min_h = (std::f64::consts::SQRT_2 * NAME_MIN_H).floor().max(UC_MIN_H);
        grow(r, v, min_w, min_h);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        let cw = (w / std::f64::consts::SQRT_2).floor();
        layout_name(r, ms, v, m, x + (w - cw) / 2.0, y + (h - NAME_MIN_H) / 2.0, cw, None, false);
        if let Some(nc) = r.refd(v, "nameCompartment") {
            r.set(nc, "width", num(cw));
        }
        return Info { shape: "usecase", ..Default::default() };
    }

    // box form: name, [literals], attributes, operations
    let show = Show::of(r, v);
    let wrap = r.bool(v, "wordWrap");
    let mut comps: Vec<(Eid, f64, f64, Vec<Row>)> = vec![];
    if let Some(lc) = r.refd(v, "enumerationLiteralCompartment") {
        if r.bool(v, "suppressLiterals") {
            hide_comp(r, Some(lc));
        } else {
            r.remove(lc, "visible");
            let items = r.list(m, "literals");
            let texts: Vec<String> = items
                .iter()
                .map(|l| {
                    let st = stereotype_text(r, *l);
                    if st.is_empty() { r.name(*l) } else { format!("{st} {}", r.name(*l)) }
                })
                .collect();
            let (w, h, rows) = sync_list(r, ms, lc, &items, "UMLEnumerationLiteralView", &texts, wrap);
            comps.push((lc, w, h, rows));
        }
    }
    if let Some(ac) = r.refd(v, "attributeCompartment") {
        if r.bool(v, "suppressAttributes") {
            hide_comp(r, Some(ac));
        } else {
            r.remove(ac, "visible");
            let items = r.list(m, "attributes");
            let texts: Vec<String> = items
                .iter()
                .map(|a| attribute_text(r, *a, show))
                .collect();
            let (w, h, rows) = sync_list(r, ms, ac, &items, "UMLAttributeView", &texts, wrap);
            comps.push((ac, w, h, rows));
        }
    }
    if let Some(oc) = r.refd(v, "operationCompartment") {
        if r.bool(v, "suppressOperations") {
            hide_comp(r, Some(oc));
        } else {
            r.remove(oc, "visible");
            let items = r.list(m, "operations");
            let texts: Vec<String> = items.iter().map(|o| operation_text(r, *o, show)).collect();
            let (w, h, rows) = sync_list(r, ms, oc, &items, "UMLOperationView", &texts, wrap);
            comps.push((oc, w, h, rows));
        }
    }
    // template parameters: a dashed box over the top-right corner (estimated placement)
    let mut templ = None;
    if let Some(tc) = r.refd(v, "templateParameterCompartment") {
        let items = r.list(m, "templateParameters");
        if items.is_empty() {
            hide_comp(r, Some(tc));
        } else {
            r.remove(tc, "visible");
            let texts: Vec<String> = items.iter().map(|t| template_parameter_text(r, *t)).collect();
            templ = Some((tc, sync_list(r, ms, tc, &items, "UMLTemplateParameterView", &texts, false)));
        }
    }
    let mut min_w = name_min_width(r, ms, v, m, keyword, false);
    for c in &comps {
        min_w = min_w.max(c.1);
    }
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let w0 = if r.bool(v, "autoResize") { min_w } else { r.f(v, "width").max(min_w) };
    let name_h = layout_name(r, ms, v, m, x, y, w0, keyword, false);
    if wrap {
        for c in comps.iter_mut() {
            c.2 = list_height(r, ms, &c.3, w0);
        }
    }
    let min_h = name_h + comps.iter().map(|c| c.2).sum::<f64>();
    grow(r, v, min_w, min_h);
    let w = r.f(v, "width");
    let mut cy = y + name_h;
    let mut out = vec![];
    for (c, _, h, rows) in comps {
        place_list(r, ms, c, x, cy, w, h, &rows);
        out.push(Comp { y: cy, rows });
        cy += h;
    }
    let templ = templ.map(|(tc, (tw, th, rows))| {
        let (tx, ty) = (x + w - tw + TEMPLATE_OUT, y - th + TEMPLATE_OUT);
        place_list(r, ms, tc, tx, ty, tw, th, &rows);
        Comp { y: ty, rows }
    });
    Info { shape: "", compartments: out, is_active: r.bool(m, "isActive"), templ }
}

fn layout_node(r: &mut Repo, ms: &dyn Measure, v: Eid) -> Info {
    let (face, size) = view_font(r, v);
    let scaled = Scaled { ms, face, size };
    let ms: &dyn Measure = &scaled;
    let t = r.ty(v).to_string();
    let m = r.refd(v, "model");
    if let Some(m) = m {
        if let Some(k) = node_kind(r.ty(m)) {
            if k.classifier && k.view == t {
                return layout_classifier(r, ms, v, m);
            }
        }
    }
    if t == "UMLPackageView" || t == "UMLSubsystemView" {
        let min_w = match m {
            Some(m) => PKG_MIN_W.max(name_min_width(r, ms, v, m, None, false)),
            None => PKG_MIN_W,
        };
        grow(r, v, min_w, PKG_MIN_H);
        if let Some(m) = m {
            let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
            layout_name(r, ms, v, m, x, y + PKG_TAB_H, w, None, false);
        }
        return Info { shape: "package", ..Default::default() };
    }
    if t == "UMLUseCaseSubjectView" {
        if let Some(m) = m {
            let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
            layout_name(r, ms, v, m, x, y, w, None, false);
        }
        return Info { shape: "subject", ..Default::default() };
    }
    Info::default()
}

// ------------------------------------------------------------------ edges
fn center(r: &R) -> P {
    P { x: r.x + r.w / 2.0, y: r.y + r.h / 2.0 }
}

/// Where the line from the box centre towards p leaves the box grown by 1 (core.js:2700).
pub fn junction(r: &R, p: P) -> P {
    let (x1, y1, x2, y2) = (r.x - 1.0, r.y - 1.0, r.x + r.w + 1.0, r.y + r.h + 1.0);
    let c = P { x: (x1 + x2) / 2.0, y: (y1 + y2) / 2.0 };
    let (dx, dy) = (p.x - c.x, p.y - c.y);
    if dx == 0.0 && dy == 0.0 {
        return c;
    }
    let tx = if dx != 0.0 { (if dx > 0.0 { x2 - c.x } else { x1 - c.x }) / dx } else { f64::INFINITY };
    let ty = if dy != 0.0 { (if dy > 0.0 { y2 - c.y } else { y1 - c.y }) / dy } else { f64::INFINITY };
    let t = tx.min(ty);
    P { x: (c.x + dx * t).round(), y: (c.y + dy * t).round() }
}

fn end_rect(r: &Repo, v: Eid, info: &HashMap<Eid, Info>) -> R {
    if is_edge(r, v) {
        let pts = points_of(r, v);
        let mid = pts.get(pts.len() / 2).copied().unwrap_or_default();
        return R { x: mid.x, y: mid.y, w: 0.0, h: 0.0 };
    }
    let nr = node_rect(r, v);
    let shape = info.get(&v).map(|i| i.shape).unwrap_or("");
    if shape == "interface" {
        let ih = nr.h - NAME_MIN_H;
        let d = nr.w.min(ih);
        return R { x: nr.x + (nr.w - d) / 2.0, y: nr.y + (ih - d) / 2.0, w: d, h: d };
    }
    // lines meet the icon of an artifact or a robustness class, not its whole box
    if shape == "artifact" || shape == "boundary" || shape == "entity" || shape == "control" {
        let ih = nr.h - NAME_MIN_H;
        let iw = if shape == "artifact" { (ih * ARTIFACT_ICON_W / ARTIFACT_ICON_H).min(nr.w) } else { nr.w.min(ROBUST_ICON_W) };
        return R { x: nr.x + (nr.w - iw) / 2.0, y: nr.y, w: iw, h: ih };
    }
    nr
}

/// Line styles: 0 rectilinear, 1 oblique, 2 rounded rectilinear, 3 curve (core.js LS_*).
pub fn rectilinear(style: i64) -> bool {
    style == 0 || style == 2
}

/// The polyline actually drawn for a line's points: rounded corners for style 2 and a smooth curve
/// for style 3 (both estimated: the original's drawing code was not read).
pub fn drawn_points(pts: &[P], style: i64) -> Vec<P> {
    if pts.len() < 3 || !(style == 2 || style == 3) {
        return pts.to_vec();
    }
    let quad = |out: &mut Vec<P>, a: P, c: P, b: P| {
        for k in 1..=8 {
            let t = k as f64 / 8.0;
            let u = 1.0 - t;
            out.push(P { x: u * u * a.x + 2.0 * u * t * c.x + t * t * b.x, y: u * u * a.y + 2.0 * u * t * c.y + t * t * b.y });
        }
    };
    let mid = |a: P, b: P| P { x: (a.x + b.x) / 2.0, y: (a.y + b.y) / 2.0 };
    let mut out = vec![pts[0]];
    let n = pts.len();
    if style == 3 {
        // through the middle of every segment, bending towards each point
        let mut from = pts[0];
        for i in 1..n - 1 {
            let to = if i == n - 2 { pts[n - 1] } else { mid(pts[i], pts[i + 1]) };
            quad(&mut out, from, pts[i], to);
            from = to;
        }
        return out;
    }
    for i in 1..n - 1 {
        let (a, c, b) = (pts[i - 1], pts[i], pts[i + 1]);
        let len = |p: P, q: P| ((p.x - q.x).powi(2) + (p.y - q.y).powi(2)).sqrt();
        let rad = ROUND_RADIUS.min(len(a, c) / 2.0).min(len(c, b) / 2.0);
        let towards = |p: P, q: P, d: f64| {
            let l = len(p, q).max(1e-9);
            P { x: p.x + (q.x - p.x) * d / l, y: p.y + (q.y - p.y) * d / l }
        };
        let (s, e) = (towards(c, a, rad), towards(c, b, rad));
        out.push(s);
        quad(&mut out, s, c, e);
    }
    out.push(pts[n - 1]);
    out
}

/// Bend points left after a drag: points within 5 px of a neighbour (oblique and curve lines only) and
/// points in a straight run are dropped; the ends always stay (graphics.js:737-773, 1000-1044).
pub fn clean_points(pts: &[P], rectilinear: bool) -> Vec<P> {
    let mut out: Vec<P> = pts.to_vec();
    let dist = |a: P, b: P| ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
    loop {
        let n = out.len();
        if n <= 2 {
            return out;
        }
        let drop = (1..n - 1).find(|&i| {
            let (a, c, b) = (out[i - 1], out[i], out[i + 1]);
            let close = if rectilinear { dist(a, c) < 0.5 || dist(c, b) < 0.5 } else { dist(a, c) <= 5.0 || dist(c, b) <= 5.0 };
            // collinear: c lies on the line through a and b
            let cross = ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs() / dist(a, b).max(1e-9);
            let between = (c.x - a.x) * (c.x - b.x) <= 0.5 && (c.y - a.y) * (c.y - b.y) <= 0.5;
            close || (cross < 1.0 && between)
        });
        match drop {
            Some(i) => {
                out.remove(i);
            }
            None => return out,
        }
    }
}

/// Corner radius of rounded rectilinear lines (estimated).
pub const ROUND_RADIUS: f64 = 10.0;

fn layout_edge(r: &mut Repo, ms: &dyn Measure, e: Eid, info: &HashMap<Eid, Info>) {
    let (Some(tail), Some(head)) = (r.refd(e, "tail"), r.refd(e, "head")) else { return };
    let tr = end_rect(r, tail, info);
    let hr = end_rect(r, head, info);
    let mut pts = points_of(r, e);
    let style = r.val(e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
    if tail == head {
        let (x2, cy, cx) = (tr.x + tr.w, tr.y + tr.h / 2.0, tr.x + tr.w / 2.0);
        pts = vec![
            P { x: cx, y: tr.y - 1.0 }, P { x: cx, y: tr.y - 20.0 }, P { x: x2 + 30.0, y: tr.y - 20.0 },
            P { x: x2 + 30.0, y: cy }, P { x: x2 + 1.0, y: cy },
        ];
    } else if pts.len() <= 2 || (rectilinear(style) && pts.len() <= 3) {
        let (tc, hc) = (center(&tr), center(&hr));
        if rectilinear(style) {
            // rectilinear: straight when the boxes overlap on one axis, else one elbow (estimated)
            let (ox1, ox2) = (tr.x.max(hr.x), (tr.x + tr.w).min(hr.x + hr.w));
            let (oy1, oy2) = (tr.y.max(hr.y), (tr.y + tr.h).min(hr.y + hr.h));
            if ox2 > ox1 {
                let x = ((ox1 + ox2) / 2.0).round();
                let mut a = junction(&tr, P { x, y: hc.y });
                let mut b = junction(&hr, P { x, y: tc.y });
                a.x = x;
                b.x = x;
                pts = vec![a, b];
            } else if oy2 > oy1 {
                let y = ((oy1 + oy2) / 2.0).round();
                let mut a = junction(&tr, P { x: hc.x, y });
                let mut b = junction(&hr, P { x: tc.x, y });
                a.y = y;
                b.y = y;
                pts = vec![a, b];
            } else {
                let elbow = P { x: hc.x.round(), y: tc.y.round() };
                let mut a = junction(&tr, elbow);
                let mut b = junction(&hr, elbow);
                a.y = tc.y.round();
                b.x = hc.x.round();
                pts = vec![a, elbow, b];
            }
        } else {
            pts = vec![junction(&tr, hc), junction(&hr, tc)];
        }
    } else {
        let n = pts.len();
        pts[0] = junction(&tr, pts[1]);
        pts[n - 1] = junction(&hr, pts[n - 2]);
    }
    r.set(e, "points", json!(format_points(&pts)));
    layout_edge_labels(r, ms, e, &pts);
}

/// Coord.getPointAwayLine (graphics.js:1207): offset `distance` from p1 at the segment angle + alpha.
pub fn point_away_line(p1: P, p2: P, alpha: f64, distance: f64) -> P {
    let a = p1.x - p2.x;
    let b = p1.y - p2.y;
    let mut th = (b.abs() / (a.abs() + 0.000001)).atan();
    if a > 0.0 {
        th = if b > 0.0 { std::f64::consts::PI - th } else { th + std::f64::consts::PI };
    } else if b < 0.0 {
        th = 2.0 * std::f64::consts::PI - th;
    }
    P { x: (distance * (th + alpha).cos()).floor(), y: (-distance * (th + alpha).sin()).floor() }
}

fn edge_label_texts(r: &Repo, e: Eid) -> Vec<(&'static str, String)> {
    let Some(m) = r.refd(e, "model") else { return vec![] };
    let show_vis = r.bool(e, "showVisibility");
    let vis = |x: Eid| if show_vis { vis_symbol(&r.str(x, "visibility")).to_string() } else { String::new() };
    let mut out = vec![];
    let name = r.name(m);
    out.push(("nameLabel", if name.is_empty() { name } else { vis(m) + &name }));
    let st = match r.ty(m) {
        "UMLInclude" => "«include»".to_string(),
        // a deployment is a «deploy» dependency (UML 2; the keyword is estimated for StarUML)
        "UMLDeployment" if stereotype_text(r, m).is_empty() => "«deploy»".to_string(),
        "UMLExtend" => "«extend»".to_string(),
        _ => stereotype_text(r, m),
    };
    out.push(("stereotypeLabel", st));
    out.push(("propertyLabel", String::new()));
    if r.kind_of(m, "UMLAssociation") {
        let role = |end: Option<Eid>| end.map(|x| { let n = r.name(x); if n.is_empty() { n } else { vis(x) + &n } }).unwrap_or_default();
        let (e1, e2) = (r.child(m, "end1"), r.child(m, "end2"));
        let show_mult = r.bool(e, "showMultiplicity");
        let mult = |end: Option<Eid>| if show_mult { end.map(|x| r.str(x, "multiplicity")).unwrap_or_default() } else { String::new() };
        out.push(("tailRoleNameLabel", role(e1)));
        out.push(("headRoleNameLabel", role(e2)));
        out.push(("tailMultiplicityLabel", mult(e1)));
        out.push(("headMultiplicityLabel", mult(e2)));
        let show_prop = r.bool(e, "showProperty");
        let prop = |end: Option<Eid>| if show_prop { end.map(|x| property_text(r, x)).unwrap_or_default() } else { String::new() };
        out.push(("tailPropertyLabel", prop(e1)));
        out.push(("headPropertyLabel", prop(e2)));
    }
    out
}

/// The segment an edge label hangs on: head, tail or middle by its edgePosition; with an even number of
/// points the middle anchor is the middle of the central segment (diagram-editor.js:2071-2097).
pub fn label_anchor(r: &Repo, lv: Eid, pts: &[P]) -> (P, P) {
    let n = pts.len();
    match r.val(lv, "edgePosition").and_then(|v| v.as_i64()).unwrap_or(0) {
        0 => (pts[n - 1], pts[n - 2]),
        2 => (pts[0], pts[1]),
        _ => {
            let mut mi = n / 2;
            if n % 2 == 0 {
                mi -= 1;
            }
            let mut p1 = pts[mi];
            let p2 = pts[mi + 1];
            if n % 2 == 0 {
                p1.x = ((p1.x + p2.x) / 2.0).floor();
                p1.y = ((p1.y + p2.y) / 2.0).floor();
            }
            (p1, p2)
        }
    }
}

/// The alpha and distance that put a label's centre at `c`: the inverse of point_away_line
/// (engine.js:754-795).
pub fn label_polar(p1: P, p2: P, c: P) -> (f64, f64) {
    let (vx, vy) = (c.x - p1.x, c.y - p1.y);
    let dist = (vx * vx + vy * vy).sqrt();
    let a = p1.x - p2.x;
    let b = p1.y - p2.y;
    let mut th = (b.abs() / (a.abs() + 0.000001)).atan();
    if a > 0.0 {
        th = if b > 0.0 { std::f64::consts::PI - th } else { th + std::f64::consts::PI };
    } else if b < 0.0 {
        th = 2.0 * std::f64::consts::PI - th;
    }
    let phi = (-vy).atan2(vx);
    (phi - th, dist)
}

fn layout_edge_labels(r: &mut Repo, ms: &dyn Measure, e: Eid, pts: &[P]) {
    if pts.len() < 2 {
        return;
    }
    let (face, size) = view_font(r, e);
    let scaled = Scaled { ms, face, size };
    let ms: &dyn Measure = &scaled;
    for (field, t) in edge_label_texts(r, e) {
        let Some(lv) = r.refd(e, field) else { continue };
        set_label_font(r, ms, lv, None);
        r.set(lv, "text", json!(t));
        if field == "stereotypeLabel" {
            if t.is_empty() {
                r.set_raw(lv, "visible", Value::Null);
            } else {
                r.remove(lv, "visible");
            }
        } else if t.is_empty() {
            r.set_raw(lv, "visible", json!(false));
        } else {
            r.remove(lv, "visible");
        }
        let w = if t.is_empty() { 0.0 } else { ms.width(&t, 0) };
        let lh = ms.size();
        r.set(lv, "width", num(w));
        r.set(lv, "height", num(lh));
        let (p1, p2) = label_anchor(r, lv, pts);
        let d = point_away_line(p1, p2, r.f(lv, "alpha"), r.f(lv, "distance"));
        r.set(lv, "left", num(d.x + p1.x - (w / 2.0).floor()));
        r.set(lv, "top", num(d.y + p1.y - (lh / 2.0).floor()));
    }
}

/// Lay out a whole diagram; returns drawing info per node view.
pub fn layout_diagram(r: &mut Repo, ms: &dyn Measure, d: Eid) -> HashMap<Eid, Info> {
    let mut info = HashMap::new();
    let views = r.list(d, "ownedViews");
    for v in &views {
        if r.alive(*v) && is_node(r, *v) {
            let i = layout_node(r, ms, *v);
            info.insert(*v, i);
        }
    }
    for v in &views {
        // only line kinds UMLCraft knows are re-routed; others keep the points they were saved with
        if r.alive(*v) && KNOWN_EDGE_VIEWS.contains(&r.ty(*v)) && is_edge(r, *v) {
            layout_edge(r, ms, *v, &info);
        }
    }
    info
}
