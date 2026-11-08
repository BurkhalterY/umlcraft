//! SysML diagrams: requirement, block definition, internal block and parametric diagrams, and the
//! viewpoint elements. SysML builds on UML: its boxes are classifier views with compartments, its
//! relationships UML dependencies, generalizations and connectors with a keyword
//! (extensions/essential/sysml: metamodel.json, elements.js, sysml-factory.js, sysml-commands.js,
//! toolbox/sysml.json, menus/sysml.json, quickedits/sysml.json, preferences/sysml.json).

use crate::app::App;
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Comp, Info, Measure, Row, Show, P, R};
use serde_json::{json, Value};

type Attrs = Vec<(&'static str, Option<Value>)>;

/// The SysML kinds, in the order of sysml/metamodel.json, with their fields and defaults.
pub fn define(def: &mut dyn FnMut(&'static str, Option<&'static str>, Attrs)) {
    let n = || None;
    let d = |v: Value| Some(v);
    def("SysMLDiagram", Some("Diagram"), vec![]);
    for ty in ["SysMLRequirementDiagram", "SysMLBlockDefinitionDiagram", "SysMLInternalBlockDiagram"] {
        def(ty, Some("SysMLDiagram"), vec![]);
    }
    def("SysMLParametricDiagram", Some("SysMLInternalBlockDiagram"), vec![]);
    def("SysMLStakeholder", Some("UMLClassifier"), vec![("concern", d(json!("")))]);
    def("SysMLView", Some("UMLClass"), vec![]);
    def("SysMLViewpoint", Some("UMLClass"), vec![("stakeholders", n()), ("purpose", d(json!(""))), ("language", d(json!(""))), ("presentation", d(json!("")))]);
    def("SysMLConform", Some("UMLGeneralization"), vec![]);
    def("SysMLExpose", Some("UMLDependency"), vec![]);
    def("SysMLAbstractRequirement", Some("UMLClass"), vec![("id", d(json!(""))), ("text", d(json!("")))]);
    def("SysMLRequirement", Some("SysMLAbstractRequirement"), vec![]);
    def("SysMLTrace", Some("UMLAbstraction"), vec![]);
    for ty in ["SysMLCopy", "SysMLDeriveReqt", "SysMLVerify", "SysMLSatisfy"] {
        def(ty, Some("SysMLTrace"), vec![]);
    }
    def("SysMLRefine", Some("UMLAbstraction"), vec![]);
    def("SysMLProperty", Some("UMLAttribute"), vec![]);
    def("SysMLFlowProperty", Some("SysMLProperty"), vec![("direction", d(json!("inout")))]);
    def(
        "SysMLBlock",
        Some("UMLClass"),
        vec![("isEncapsulated", d(json!(false))), ("constraints", n()), ("parts", n()), ("ports", n()), ("references", n()), ("values", n()), ("properties", n()), ("flowProperties", n())],
    );
    def("SysMLValueType", Some("UMLDataType"), vec![("unit", n()), ("quantityKind", n())]);
    def("SysMLInterfaceBlock", Some("SysMLBlock"), vec![]);
    def("SysMLPort", Some("UMLPort"), vec![]);
    def("SysMLConnector", Some("UMLConnector"), vec![]);
    def("SysMLItemFlow", Some("UMLInformationFlow"), vec![("itemProperty", n())]);
    def("SysMLConstraintBlock", Some("SysMLBlock"), vec![("parameters", n())]);
    def("SysMLPropertyValueCompartmentItemView", Some("LabelView"), vec![("property", d(json!(""))), ("value", d(json!("")))]);
    def("SysMLPropertyValueCompartmentView", Some("UMLListCompartmentView"), vec![]);
    def("SysMLClassifierView", Some("UMLClassifierView"), vec![("suppressPropertyValues", d(json!(false))), ("propertyValueCompartment", n())]);
    for ty in ["SysMLStakeholderView", "SysMLViewView", "SysMLViewpointView", "SysMLRequirementView", "SysMLValueTypeView"] {
        def(ty, Some("SysMLClassifierView"), vec![]);
    }
    def("SysMLConformView", Some("UMLGeneralizationView"), vec![]);
    def("SysMLExposeView", Some("UMLDependencyView"), vec![]);
    def("SysMLAbstractionView", Some("UMLDependencyView"), vec![]);
    def("SysMLTraceView", Some("SysMLAbstractionView"), vec![]);
    for ty in ["SysMLCopyView", "SysMLDeriveReqtView", "SysMLVerifyView", "SysMLSatisfyView"] {
        def(ty, Some("SysMLTraceView"), vec![]);
    }
    def("SysMLRefineView", Some("SysMLAbstractionView"), vec![]);
    for kind in ["Constraint", "Part", "Port", "Reference", "Value", "Property", "FlowProperty", "Parameter"] {
        let item: &'static str = Box::leak(format!("SysML{kind}CompartmentItemView").into_boxed_str());
        let comp: &'static str = Box::leak(format!("SysML{kind}CompartmentView").into_boxed_str());
        def(item, Some("UMLAttributeView"), vec![]);
        def(comp, Some("UMLListCompartmentView"), vec![]);
    }
    let t = || d(json!(true));
    def(
        "SysMLBlockView",
        Some("UMLClassifierView"),
        vec![
            ("suppressConstraints", t()), ("suppressParts", t()), ("suppressPorts", t()), ("suppressReferences", t()), ("suppressValues", t()), ("suppressProperties", t()),
            ("suppressFlowProperties", t()), ("constraintCompartment", n()), ("partCompartment", n()), ("portCompartment", n()), ("referenceCompartment", n()), ("valueCompartment", n()),
            ("propertyCompartment", n()), ("flowPropertyCompartment", n()),
        ],
    );
    def("SysMLInterfaceBlockView", Some("SysMLBlockView"), vec![]);
    def("SysMLConstraintBlockView", Some("UMLClassifierView"), vec![("suppressConstraints", t()), ("suppressProperties", t()), ("constraintCompartment", n()), ("parameterCompartment", n())]);
    def("SysMLPartView", Some("UMLPartView"), vec![]);
    def("SysMLPortView", Some("UMLPortView"), vec![]);
    def("SysMLConnectorView", Some("UMLConnectorView"), vec![("itemFlowToHeadLabel", n()), ("itemFlowToTailLabel", n())]);
    def("SysMLConstraintPropertyView", Some("UMLPartView"), vec![]);
    def("SysMLConstraintParameterView", Some("UMLFloatingNodeView"), vec![("showVisibility", d(json!(false))), ("showType", t()), ("showMultiplicity", t())]);
}

// ------------------------------------------------------------------ kinds

/// The four SysML diagram kinds, with their Model › Add Diagram labels (menus/sysml.json).
pub const DIAGRAMS: [(&str, &str); 4] = [
    ("Requirement Diagram", "SysMLRequirementDiagram"),
    ("Block Definition Diagram", "SysMLBlockDefinitionDiagram"),
    ("Internal Block Diagram", "SysMLInternalBlockDiagram"),
    ("Parametric Diagram", "SysMLParametricDiagram"),
];

pub fn is_diagram(ty: &str) -> bool {
    DIAGRAMS.iter().any(|d| d.1 == ty)
}

/// The view of a SysML box element (the metamodel's `view`).
pub fn view_of(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "SysMLRequirement" => "SysMLRequirementView",
        "SysMLStakeholder" => "SysMLStakeholderView",
        "SysMLView" => "SysMLViewView",
        "SysMLViewpoint" => "SysMLViewpointView",
        "SysMLBlock" => "SysMLBlockView",
        "SysMLValueType" => "SysMLValueTypeView",
        "SysMLInterfaceBlock" => "SysMLInterfaceBlockView",
        "SysMLConstraintBlock" => "SysMLConstraintBlockView",
        _ => return None,
    })
}

/// Whether a view is one of the SysML boxes this module lays out and draws.
pub fn is_view(view_ty: &str) -> bool {
    specs(view_ty).is_some()
}

/// The line view of a SysML relationship.
pub fn edge_view_of(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "SysMLTrace" => "SysMLTraceView",
        "SysMLCopy" => "SysMLCopyView",
        "SysMLDeriveReqt" => "SysMLDeriveReqtView",
        "SysMLVerify" => "SysMLVerifyView",
        "SysMLSatisfy" => "SysMLSatisfyView",
        "SysMLRefine" => "SysMLRefineView",
        "SysMLConform" => "SysMLConformView",
        "SysMLExpose" => "SysMLExposeView",
        _ => return None,
    })
}

pub fn is_edge_view(view_ty: &str) -> bool {
    uml_edge(view_ty).is_some()
}

/// The UML line a SysML line is drawn as: the traces, refine and expose are dependencies (a dashed
/// line with an open arrow), conform is a generalization (elements.js:780-960).
pub fn uml_edge(view_ty: &str) -> Option<&'static str> {
    Some(match view_ty {
        "SysMLAbstractionView" | "SysMLTraceView" | "SysMLCopyView" | "SysMLDeriveReqtView" | "SysMLVerifyView" | "SysMLSatisfyView" | "SysMLRefineView" | "SysMLExposeView" => "UMLDependencyView",
        "SysMLConformView" => "UMLGeneralizationView",
        _ => return None,
    })
}

/// The keyword a relationship shows when it has no stereotype of its own (getStereotypeString,
/// elements.js:259-375).
pub fn keyword(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "SysMLCopy" => "«copy»",
        "SysMLDeriveReqt" => "«deriveReqt»",
        "SysMLVerify" => "«verify»",
        "SysMLSatisfy" => "«satisfy»",
        "SysMLRefine" => "«refine»",
        "SysMLConform" => "«conform»",
        "SysMLExpose" => "«expose»",
        _ => return None,
    })
}

/// The display class name of a SysML kind: its type without "SysML" (SysMLElementMixin, elements.js:40-50).
pub fn display_name(ty: &str) -> &str {
    ty.strip_prefix("SysML").unwrap_or(ty)
}

/// What a SysML diagram takes (each diagram's canAcceptModel, elements.js:70-125).
pub fn accepts(diagram_ty: &str) -> Option<Vec<&'static str>> {
    Some(match diagram_ty {
        "SysMLRequirementDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLPackage", "UMLClassifier", "UMLGeneralization", "UMLDependency", "UMLAssociation", "SysMLRequirement", "SysMLTrace"],
        "SysMLBlockDefinitionDiagram" => vec![
            "Hyperlink", "Diagram", "UMLConstraint", "UMLPackage", "UMLClassifier", "UMLInstance", "UMLGeneralization", "UMLDependency", "UMLAssociation", "SysMLRequirement", "SysMLTrace",
            "SysMLPort",
        ],
        "SysMLInternalBlockDiagram" | "SysMLParametricDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "SysMLProperty", "SysMLPort", "SysMLBlock", "UMLInstance"],
        _ => return None,
    })
}

/// Whether a line of this kind may join these two elements on a SysML diagram; None for the kinds
/// SysML leaves to UML.
pub fn can_link(a: &App, kind: &str, tm: Eid, hm: Eid) -> Option<bool> {
    let r = &a.repo;
    let element = |m: Eid| r.kind_of(m, "UMLModelElement");
    match kind {
        // modelElementLinkPrecondition (sysml-factory.js:22-28)
        "SysMLCopy" | "SysMLDeriveReqt" | "SysMLVerify" | "SysMLSatisfy" | "SysMLRefine" | "SysMLConform" | "SysMLExpose" => Some(element(tm) && element(hm)),
        // on a SysML diagram a containment joins any two model elements, the head not inside the
        // tail (containmentFn, uml-factory.js:847-857)
        "Containment" if a.diagram.map(|d| r.kind_of(d, "SysMLDiagram")).unwrap_or(false) => Some(element(tm) && element(hm) && tm != hm && !r.path(hm).contains(&tm)),
        _ => None,
    }
}

// ------------------------------------------------------------------ compartments

/// A compartment of a SysML box: the view's field, its view type, the model list it shows, the type
/// of its item views, its title, and the flag that hides it.
pub struct Spec {
    pub field: &'static str,
    pub comp_ty: &'static str,
    pub items: &'static str,
    pub item_ty: &'static str,
    pub title: Option<&'static str>,
    pub flag: &'static str,
}

const fn spec(field: &'static str, comp_ty: &'static str, items: &'static str, item_ty: &'static str, title: Option<&'static str>, flag: &'static str) -> Spec {
    Spec { field, comp_ty, items, item_ty, title, flag }
}

const ATTRIBUTES: Spec = spec("attributeCompartment", "UMLAttributeCompartmentView", "attributes", "UMLAttributeView", None, "suppressAttributes");
const OPERATIONS: Spec = spec("operationCompartment", "UMLOperationCompartmentView", "operations", "UMLOperationView", None, "suppressOperations");
const RECEPTIONS: Spec = spec("receptionCompartment", "UMLReceptionCompartmentView", "receptions", "UMLReceptionView", None, "suppressReceptions");
const PROPERTY_VALUES: Spec = spec("propertyValueCompartment", "SysMLPropertyValueCompartmentView", "", "SysMLPropertyValueCompartmentItemView", None, "suppressPropertyValues");

/// A requirement, value type, stakeholder, view or viewpoint: the name, attributes, operations,
/// receptions and property values (SysMLClassifierView.getAllCompartments, elements.js:745-755).
const CLASSIFIER: [Spec; 4] = [ATTRIBUTES, OPERATIONS, RECEPTIONS, PROPERTY_VALUES];

/// A block: its SysML compartments, each titled, then the titled operations and receptions; the
/// attribute compartment stays hidden (SysMLBlockView.getAllCompartments, elements.js:1130-1145).
const BLOCK: [Spec; 9] = [
    spec("constraintCompartment", "SysMLConstraintCompartmentView", "constraints", "SysMLConstraintCompartmentItemView", Some("constraints"), "suppressConstraints"),
    spec("partCompartment", "SysMLPartCompartmentView", "parts", "SysMLPartCompartmentItemView", Some("parts"), "suppressParts"),
    spec("portCompartment", "SysMLPortCompartmentView", "ports", "SysMLPortCompartmentItemView", Some("ports"), "suppressPorts"),
    spec("referenceCompartment", "SysMLReferenceCompartmentView", "references", "SysMLReferenceCompartmentItemView", Some("references"), "suppressReferences"),
    spec("valueCompartment", "SysMLValueCompartmentView", "values", "SysMLValueCompartmentItemView", Some("values"), "suppressValues"),
    spec("propertyCompartment", "SysMLPropertyCompartmentView", "properties", "SysMLPropertyCompartmentItemView", Some("properties"), "suppressProperties"),
    spec("flowPropertyCompartment", "SysMLFlowPropertyCompartmentView", "flowProperties", "SysMLFlowPropertyCompartmentItemView", Some("flow properties"), "suppressFlowProperties"),
    spec("operationCompartment", "UMLOperationCompartmentView", "operations", "UMLOperationView", Some("operations"), "suppressOperations"),
    spec("receptionCompartment", "UMLReceptionCompartmentView", "receptions", "UMLReceptionView", Some("receptions"), "suppressReceptions"),
];

/// A constraint block: its constraints and its parameters, which Suppress Properties hides
/// (SysMLConstraintBlockView, elements.js:1385-1455).
const CONSTRAINT_BLOCK: [Spec; 2] = [
    spec("constraintCompartment", "SysMLConstraintCompartmentView", "constraints", "SysMLConstraintCompartmentItemView", Some("constraints"), "suppressConstraints"),
    spec("parameterCompartment", "SysMLParameterCompartmentView", "parameters", "SysMLParameterCompartmentItemView", Some("parameters"), "suppressProperties"),
];

/// The UML classifier compartments every box has (UMLClassifierView's constructor).
const UML_COMPARTMENTS: [(&str, &str); 4] = [
    ("attributeCompartment", "UMLAttributeCompartmentView"),
    ("operationCompartment", "UMLOperationCompartmentView"),
    ("receptionCompartment", "UMLReceptionCompartmentView"),
    ("templateParameterCompartment", "UMLTemplateParameterCompartmentView"),
];

pub fn specs(view_ty: &str) -> Option<&'static [Spec]> {
    match view_ty {
        "SysMLRequirementView" | "SysMLStakeholderView" | "SysMLViewView" | "SysMLViewpointView" | "SysMLValueTypeView" => Some(&CLASSIFIER),
        "SysMLBlockView" | "SysMLInterfaceBlockView" => Some(&BLOCK),
        "SysMLConstraintBlockView" => Some(&CONSTRAINT_BLOCK),
        _ => None,
    }
}

/// The keyword over a box's name, and whether it replaces the element's own stereotype (the
/// block kinds and value types always show theirs; getStereotypeLabelText, elements.js:800-1430).
fn name_keyword(view_ty: &str) -> Option<(&'static str, bool)> {
    Some(match view_ty {
        "SysMLRequirementView" => ("requirement", false),
        "SysMLStakeholderView" => ("stakeholder", false),
        "SysMLViewView" => ("view", false),
        "SysMLViewpointView" => ("viewpoint", false),
        "SysMLValueTypeView" => ("valueType", true),
        "SysMLBlockView" => ("block", true),
        "SysMLInterfaceBlockView" => ("interfaceBlock", true),
        "SysMLConstraintBlockView" => ("constraint", true),
        _ => return None,
    })
}

/// The property values a box lists, "name = value" (getPropertyValues of each kind,
/// elements.js:150-575): strings are quoted, references named.
pub fn property_values(r: &Repo, m: Eid) -> Vec<(String, String)> {
    let mut out = vec![];
    let quoted = |out: &mut Vec<(String, String)>, field: &str| {
        let s = r.str(m, field);
        if !s.is_empty() {
            out.push((field.to_string(), format!("\"{s}\"")));
        }
    };
    match r.ty(m) {
        "SysMLRequirement" => {
            quoted(&mut out, "id");
            quoted(&mut out, "text");
        }
        "SysMLStakeholder" => quoted(&mut out, "concern"),
        "SysMLValueType" => {
            for f in ["unit", "quantityKind"] {
                if let Some(x) = r.refd(m, f).filter(|x| r.alive(*x)) {
                    out.push((f.to_string(), r.name(x)));
                }
            }
        }
        "SysMLViewpoint" => {
            let st: Vec<Eid> = r.ref_list(m, "stakeholders").into_iter().filter(|x| r.alive(*x)).collect();
            if !st.is_empty() {
                out.push(("/stakeholder".into(), st.iter().map(|x| r.name(*x)).collect::<Vec<_>>().join(", ")));
                let concerns: Vec<String> = st.iter().map(|x| r.str(*x, "concern")).filter(|c| !c.is_empty()).map(|c| format!("\"{c}\"")).collect();
                out.push(("/concern".into(), concerns.join(", ")));
            }
            for f in ["purpose", "language", "presentation"] {
                quoted(&mut out, f);
            }
        }
        // a view: the viewpoints it conforms to and their stakeholders (SysMLView.getConforms)
        "SysMLView" => {
            let viewpoints: Vec<Eid> = r.all_of("SysMLConform").into_iter().filter(|c| r.refd(*c, "source") == Some(m)).filter_map(|c| r.refd(c, "target")).filter(|t| r.alive(*t)).collect();
            if !viewpoints.is_empty() {
                out.push(("/viewpoint".into(), viewpoints.iter().map(|x| r.name(*x)).collect::<Vec<_>>().join(", ")));
                let mut st: Vec<Eid> = vec![];
                for vp in &viewpoints {
                    for s in r.ref_list(*vp, "stakeholders") {
                        if r.alive(s) && !st.contains(&s) {
                            st.push(s);
                        }
                    }
                }
                out.push(("/stakeholder".into(), st.iter().map(|x| r.name(*x)).collect::<Vec<_>>().join(", ")));
            }
        }
        _ => {}
    }
    out
}

/// A compartment row's text: an attribute's text, a flow property's with its direction first
/// (SysMLFlowProperty.getString, elements.js:420-425), an operation's or a reception's.
pub fn item_text(r: &Repo, item: Eid, show: Show) -> String {
    match r.ty(item) {
        "UMLOperation" => views::operation_text(r, item, show),
        "UMLReception" => views::reception_text(r, item, show),
        "SysMLFlowProperty" => format!("{} {}", r.str(item, "direction"), views::attribute_text(r, item, show)),
        _ => views::attribute_text(r, item, show),
    }
}

// ------------------------------------------------------------------ boxes

/// A SysML box's view: the classifier compartments, then its own (each view's constructor,
/// elements.js:740-1430). `tool`: made by a toolbox tool, whose `view-init` suppresses a block's and a
/// value type's attributes and operations (toolbox/sysml.json).
pub fn create_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R, tool: bool) -> Option<Eid> {
    let ty = view_of(r.ty(model))?;
    let block = matches!(ty, "SysMLBlockView" | "SysMLInterfaceBlockView" | "SysMLConstraintBlockView");
    let m = r.ref_to(model);
    let v = r.create(
        ty,
        &[
            ("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("containerChangeable", json!(block)),
            ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h)),
        ],
        Some((diagram, "ownedViews")),
    );
    // a SysMLClassifierView hides its attributes and operations; a block view its operations
    // (elements.js:745-760, 1110-1125); the Block and Value Type tools hide both
    if !block || (tool && ty == "SysMLBlockView") {
        r.set(v, "suppressAttributes", json!(true));
    }
    r.set(v, "suppressOperations", json!(true));
    let nc = views::name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    let specs = specs(ty).unwrap_or(&[]);
    // the classifier's compartments, then the box's own; the ones it does not list stay hidden
    let all = UML_COMPARTMENTS.iter().copied().chain(specs.iter().filter(|s| !UML_COMPARTMENTS.iter().any(|u| u.0 == s.field)).map(|s| (s.field, s.comp_ty)));
    for (field, comp_ty) in all.collect::<Vec<_>>() {
        let shown = specs.iter().find(|s| s.field == field).map(|s| !r.bool(v, s.flag)).unwrap_or(false);
        let c = views::list_compartment(r, v, model, comp_ty, shown);
        r.set_ref(v, field, Some(c));
    }
    Some(v)
}

/// Keep the property value rows in step with the element's values, one row per property
/// (SysMLPropertyValueCompartmentView.update, elements.js:700-735).
fn sync_values(r: &mut Repo, ms: &dyn Measure, comp: Eid, m: Eid, wrap: bool) -> (f64, f64, Vec<Row>) {
    let values = property_values(r, m);
    let existing = r.list(comp, "subViews");
    let mut keep = vec![];
    for (prop, value) in &values {
        let iv = match existing.iter().copied().find(|x| r.str(*x, "property") == *prop && !keep.contains(x)) {
            Some(x) => x,
            None => {
                let mref = r.ref_to(m);
                r.create("SysMLPropertyValueCompartmentItemView", &[("model", mref), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(0))], Some((comp, "subViews")))
            }
        };
        r.set(iv, "property", json!(prop));
        r.set(iv, "value", json!(value));
        r.set(iv, "text", json!(format!("{prop} = {}", if value.is_empty() { "<none>" } else { value })));
        r.set(iv, "wordWrap", json!(wrap));
        keep.push(iv);
    }
    for x in existing {
        if !keep.contains(&x) && r.alive(x) {
            r.detach(x);
        }
    }
    r.set_list(comp, "subViews", keep.clone());
    let mut min_w: f64 = 0.0;
    for iv in &keep {
        let t = r.str(*iv, "text");
        let tw = if wrap { views::widest_word(ms, &t, 0) } else { ms.width(&t, 0) };
        min_w = min_w.max(tw.ceil() + 2.0 * views::PAD);
    }
    let rows: Vec<Row> = keep.into_iter().map(|iv| Row { iv }).collect();
    let h = views::list_height(r, ms, &rows, f64::INFINITY);
    (min_w, h, rows)
}

/// The height of a compartment's title, "parts" in small italics over its rows: 0.8 of the text
/// height plus the bottom padding (UMLListCompartmentView.size, elements.js:4333-4340).
fn title_height(ms: &dyn Measure) -> f64 {
    ms.size() * 0.8 + views::PAD
}

/// Lay out a SysML box like a class: the name with its keyword, then each compartment that is not
/// suppressed, titled where the view titles them (UMLClassifierView with the SysML compartments,
/// elements.js:740-1455).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) -> Info {
    let ty = r.ty(v).to_string();
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return Info::default() };
    let specs = specs(&ty).unwrap_or(&[]);
    let (kw, forced) = name_keyword(&ty).unwrap_or(("", false));
    let keyword = (!kw.is_empty()).then_some(kw);
    let show = Show::of(r, v);
    let wrap = r.bool(v, "wordWrap");
    // the classifier compartments a SysML box does not list stay hidden
    for (field, _) in UML_COMPARTMENTS {
        if !specs.iter().any(|s| s.field == field) {
            let c = r.refd(v, field);
            views::hide_comp(r, c);
        }
    }
    let th = title_height(ms);
    let mut comps: Vec<(Eid, f64, f64, Vec<Row>, f64, bool)> = vec![];
    for s in specs {
        let Some(c) = r.refd(v, s.field) else { continue };
        if r.bool(v, s.flag) {
            views::hide_comp(r, Some(c));
            continue;
        }
        r.remove(c, "visible");
        let (mut w, h, rows) = if s.field == "propertyValueCompartment" {
            sync_values(r, ms, c, m, wrap)
        } else {
            let items: Vec<Eid> = r.list(m, s.items).into_iter().filter(|x| r.alive(*x)).collect();
            let texts: Vec<String> = items.iter().map(|x| item_text(r, *x, show)).collect();
            views::sync_list(r, ms, c, &items, s.item_ty, &texts, wrap)
        };
        let t = if let Some(title) = s.title {
            w = w.max((ms.width(title, 2) * 0.8).ceil() + 2.0 * views::PAD);
            th
        } else {
            0.0
        };
        comps.push((c, w, h + t, rows, t, s.field != "propertyValueCompartment"));
    }
    let mut min_w = views::name_min_width(r, ms, v, m, keyword, false);
    for c in &comps {
        min_w = min_w.max(c.1);
    }
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let w0 = if r.bool(v, "autoResize") { min_w } else { r.f(v, "width").max(min_w) };
    let name_h = views::layout_name(r, ms, v, m, x, y, w0, keyword, false);
    // a block's keyword replaces the element's own stereotype
    if forced {
        if let Some(st) = r.refd(v, "nameCompartment").and_then(|nc| r.refd(nc, "stereotypeLabel")) {
            r.set(st, "text", json!(format!("«{kw}»")));
        }
    }
    if wrap {
        for c in comps.iter_mut() {
            c.2 = views::list_height(r, ms, &c.3, w0) + c.4;
        }
    }
    let total = name_h + comps.iter().map(|c| c.2).sum::<f64>();
    views::grow(r, v, min_w, total);
    let w = r.f(v, "width");
    let mut cy = y + name_h;
    let mut out = vec![];
    for (c, _, h, rows, t, editable) in comps {
        views::place_list(r, ms, c, x, cy + t, w, h - t, &rows);
        views::set_box(r, c, x, cy, w + 1.0, h);
        // property values are not edited in place, so they are left out of the rows
        out.push(Comp { y: cy, rows: if editable { rows } else { vec![] } });
        cy += h;
    }
    Info { shape: "sysml", compartments: out, is_active: false, templ: None }
}

fn shown(r: &Repo, v: Eid) -> bool {
    r.raw(v, "visible").and_then(|x| x.as_bool()) != Some(false)
}

/// Draw a SysML box: a class box with a line over each compartment, the titles in italics at 0.8
/// of the font size, centred (drawObject of each view; UMLListCompartmentView.drawObject,
/// elements.js:4344-4358).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let ty = r.ty(v);
    let b = views::node_rect(r, v);
    let Some(specs) = specs(ty) else { return };
    crate::render::shadow_rect(c, r, v, b);
    c.rect(b, Some(fill), Some(line));
    let size = views::view_font(r, v).1;
    for s in specs {
        let Some(comp) = r.refd(v, s.field).filter(|x| shown(r, *x)) else { continue };
        let top = r.f(comp, "top");
        c.polyline(&[P { x: b.x, y: top }, P { x: b.x + b.w, y: top }], line, false);
        if let Some(title) = s.title {
            c.text(title, R { x: b.x, y: top + views::PAD, w: b.w, h: size * 0.8 }, 2, 2, size * 0.8, fc, false);
        }
        for row in r.list(comp, "subViews") {
            crate::render::draw_label(c, r, row, fc);
        }
    }
    crate::render::draw_name(c, r, v);
}

// ------------------------------------------------------------------ diagrams and elements

impl App {
    /// A SysML diagram: in the selected element, or under the project in a new model "Model1"; it
    /// starts with a frame showing it, with its type and diagram name, at 8, 8 to 700, 600
    /// (diagramFn and _addFrame, sysml-factory.js:39-86).
    pub fn add_sysml_diagram(&mut self, parent: Option<Eid>, ty: &str) -> Eid {
        let parent = parent.filter(|p| self.repo.alive(*p) && !self.repo.kind_of(*p, "Diagram") && !self.is_view(*p)).unwrap_or(self.repo.root);
        let name = self.new_name(parent, ty);
        let target = if self.repo.ty(parent) == "Project" {
            let mname = (1..).map(|i| format!("Model{i}")).find(|n| !self.repo.children(parent).iter().any(|(_, c)| self.repo.name(*c) == *n)).unwrap();
            self.repo.create("UMLModel", &[("name", json!(mname))], Some((parent, "ownedElements")))
        } else {
            parent
        };
        let d = self.repo.create(ty, &[("name", json!(name))], Some((target, "ownedElements")));
        let f = views::create_frame_view(&mut self.repo, d, Some(d), R { x: 8.0, y: 8.0, w: 692.0, h: 592.0 });
        self.repo.set(f, "showType", json!(true));
        self.repo.set(f, "showDiagramName", json!(true));
        self.apply_view_defaults(f);
        self.open_diagram(d);
        self.reveal(d);
        d
    }

    /// The SysML box tools: the element goes in the diagram's owner, named after its kind
    /// ("Requirement1"…), its view made at the drawn box (_modelAndViewFn, sysml-factory.js:92-94).
    /// None for the tools SysML leaves to UML.
    pub fn create_sysml_node(&mut self, d: Eid, kind: &str, rect: R) -> Option<Option<Eid>> {
        view_of(kind)?;
        let parent = self.package_for(Some(d));
        let name = self.new_name(parent, kind);
        let m = self.repo.create(kind, &[("name", json!(name))], Some((parent, "ownedElements")));
        let v = create_view(&mut self.repo, d, m, rect, true);
        if let Some(v) = v {
            self.apply_view_defaults(v);
        }
        Some(v)
    }
}

/// Whether Model › Add Diagram offers this SysML kind on the selected element: requirement and
/// block definition diagrams on nothing, the project or a package (updateMenus,
/// sysml-commands.js:318-322). None for other kinds.
pub fn diagram_offered(r: &Repo, e: Option<Eid>, ty: &str) -> Option<bool> {
    if !is_diagram(ty) {
        return None;
    }
    let none = e.is_none();
    let project = e.map(|e| r.ty(e) == "Project").unwrap_or(false);
    let package = e.map(|e| r.kind_of(e, "UMLPackage") || r.kind_of(e, "UMLSubsystem")).unwrap_or(false);
    let block = e.map(|e| r.kind_of(e, "SysMLBlock")).unwrap_or(false);
    let constraint_block = e.map(|e| r.kind_of(e, "SysMLConstraintBlock")).unwrap_or(false);
    Some(match ty {
        "SysMLRequirementDiagram" | "SysMLBlockDefinitionDiagram" => none || project || package,
        "SysMLInternalBlockDiagram" => block,
        _ => constraint_block,
    })
}

/// Model › Add's SysML entries for an element: (label, kind, made as a feature)
/// (menus/sysml.json; updateMenus, sysml-commands.js:296-345).
pub fn add_kinds(r: &Repo, e: Eid) -> Vec<(&'static str, &'static str, bool)> {
    let t = r.ty(e);
    let holder = t == "Project" || r.kind_of(e, "UMLPackage") || r.kind_of(e, "UMLSubsystem") || r.kind_of(e, "UMLClassifier");
    let mut v = vec![];
    if holder {
        v.push(("Requirement", "SysMLRequirement", false));
    }
    v
}

/// The SysML elements' visible properties (sysml/metamodel.json `visible` attributes after the UML
/// ones of their super-kinds).
pub fn fields(r: &Repo, e: Eid) -> Option<Vec<(&'static str, &'static str)>> {
    let base = |more: &[(&'static str, &'static str)]| {
        let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isAbstract", "bool"), ("isFinalSpecialization", "bool"), ("isLeaf", "bool"), ("isActive", "bool")];
        f.extend_from_slice(more);
        f
    };
    Some(match r.ty(e) {
        "SysMLRequirement" => base(&[("id", "text"), ("text", "text")]),
        _ => return None,
    })
}

/// The icon of a SysML element in the explorer.
pub fn icon_key(ty: &str) -> &'static str {
    match ty {
        "SysMLRequirement" => "SysMLRequirement",
        "SysMLCopy" | "SysMLDeriveReqt" | "SysMLVerify" | "SysMLSatisfy" | "SysMLRefine" | "SysMLTrace" | "SysMLExpose" | "SysMLItemFlow" => "dependency",
        "SysMLConform" => "generalization",
        "SysMLRequirementDiagram" => "SysMLRequirementDiagram",
        _ if ty.ends_with("Diagram") => "diagram",
        _ => "class",
    }
}

// ------------------------------------------------------------------ quick edits

type Button = (&'static str, &'static str, &'static str, crate::canvas::ItemKey);

/// The buttons of a SysML element's name edit, besides the related-element ones
/// (quickedits/sysml.json). None for the elements SysML leaves to UML.
pub fn item_buttons(r: &Repo, _host: Option<Eid>, m: Eid) -> Option<Vec<Button>> {
    match r.ty(m) {
        // a requirement only adds a sub-requirement, a related element ("sysml.requirement")
        "SysMLRequirement" => Some(vec![]),
        _ => None,
    }
}

impl App {
    /// The keys and buttons of a SysML element's inline edit; false when UML handles them.
    pub(crate) fn sysml_item_key(&mut self, _host: Eid, m: Eid, item: Option<Eid>, key: crate::canvas::ItemKey) -> bool {
        use crate::canvas::ItemKey;
        // a requirement's name edit has no Add Attribute or Add Operation key ("sysml.requirement")
        self.repo.ty(m) == "SysMLRequirement" && item.is_none() && matches!(key, ItemKey::Add | ItemKey::AddOperation)
    }
}

/// Preferences › SysML, in the original's order (preferences/sysml.json).
pub fn preferences() -> Vec<(&'static str, &'static str, &'static str, crate::prefs::Kind, Value)> {
    use crate::prefs::Kind;
    vec![("SysML", "", "Requirement", Kind::Header, Value::Null), ("SysML", "sysml.requirement.fillColor", "Fill Color", Kind::Color, Value::Null)]
}

/// The fill colour preference of a SysML view (the views' constructors, elements.js:845-1700).
pub fn fill_pref(view_ty: &str) -> Option<&'static str> {
    Some(match view_ty {
        "SysMLRequirementView" => "sysml.requirement.fillColor",
        _ => return None,
    })
}
