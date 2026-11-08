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

/// Format › Suppress Property Values, Constraints, Parts, Ports, References, Values, Properties and
/// Flow Properties toggle these flags on the selected views that have them (sysml-commands.js:361-368).
pub const SUPPRESS_FLAGS: [&str; 8] = ["suppressPropertyValues", "suppressConstraints", "suppressParts", "suppressPorts", "suppressReferences", "suppressValues", "suppressProperties", "suppressFlowProperties"];

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
    specs(view_ty).is_some() || floating(view_ty) || matches!(view_ty, "SysMLPartView" | "SysMLConstraintPropertyView")
}

/// The tools that make a property of the block that owns the diagram, and its view: (field, view)
/// (sysml-factory.js:429-461).
pub fn property_tool(kind: &str) -> Option<(&'static str, &'static str)> {
    Some(match kind {
        "SysMLPart" => ("parts", "SysMLPartView"),
        "SysMLReference" => ("references", "SysMLPartView"),
        "SysMLValue" => ("values", "SysMLPartView"),
        "SysMLConstraintProperty" => ("constraints", "SysMLConstraintPropertyView"),
        _ => return None,
    })
}

/// The SysML views that sit on another box's border at a fixed size: ports and constraint
/// parameters (UMLPortView, SysMLConstraintParameterView, elements.js:1480, 1880).
pub fn floating(view_ty: &str) -> bool {
    matches!(view_ty, "SysMLPortView" | "SysMLConstraintParameterView")
}

/// The tools that attach their element to the box under the point (toolbox/sysml.json "parasitic").
pub fn parasitic(kind: &str) -> bool {
    matches!(kind, "SysMLPort" | "SysMLConstraintParameter")
}

/// What a block's model list holds: the kind, the name prefix and whether it is composite
/// (menus/sysml.json, quickedits/sysml.json).
pub fn item_kind(field: &str) -> Option<(&'static str, &'static str, bool)> {
    Some(match field {
        "parts" => ("SysMLProperty", "Part", true),
        "references" => ("SysMLProperty", "Reference", false),
        "values" => ("SysMLProperty", "Value", true),
        "properties" => ("SysMLProperty", "Property", false),
        "constraints" => ("SysMLProperty", "Constraint", false),
        "flowProperties" => ("SysMLFlowProperty", "FlowProperty", false),
        "parameters" => ("SysMLProperty", "Parameter", false),
        "ports" => ("SysMLPort", "Port", false),
        _ => return None,
    })
}

/// The explorer order of a SysML kind: the metamodel's `ordering` (sysml/metamodel.json).
pub fn order_rank(ty: &str) -> u32 {
    match ty {
        "SysMLBlock" => 1300,
        "SysMLValueType" => 1301,
        "SysMLTrace" | "SysMLCopy" | "SysMLDeriveReqt" | "SysMLVerify" | "SysMLSatisfy" | "SysMLRefine" => 1806,
        _ => 0,
    }
}

/// A compartment item's text for its inline edit: a flow property without its direction.
pub fn edit_text(r: &Repo, iv: Eid, item: Eid) -> String {
    let t = r.str(iv, "text");
    let dir = format!("{} ", r.str(item, "direction"));
    t.strip_prefix(&dir).map(String::from).unwrap_or(t)
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
        "SysMLConnector" => "SysMLConnectorView",
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
        "SysMLConnectorView" => "UMLConnectorView",
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

/// The SysML elements a diagram takes from the explorer (canAcceptModel of each diagram).
pub fn droppable(ty: &str) -> bool {
    view_of(ty).is_some() || matches!(ty, "SysMLPort" | "SysMLProperty" | "SysMLFlowProperty")
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
        // a connector joins two parts or ports; the original checks only the tail (connectorFn,
        // sysml-factory.js:179-188), UMLCraft both ends
        "SysMLConnector" => {
            let end = |m: Eid| r.kind_of(m, "SysMLProperty") || r.kind_of(m, "SysMLPort");
            Some(end(tm) && end(hm))
        }
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
    match ty.as_str() {
        "SysMLPortView" => views::layout_port(r, ms, v, m),
        "SysMLPartView" | "SysMLConstraintPropertyView" => layout_part(r, ms, v, m),
        "SysMLConstraintParameterView" => layout_parameter(r, ms, v, m),
        _ => {}
    }
    if specs(&ty).is_none() {
        return Info { shape: "sysml", ..Default::default() };
    }
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
    match ty {
        "SysMLPortView" => return draw_port(c, r, v, line, fill, fc),
        "SysMLConstraintParameterView" => {
            c.rect(b, Some(fill), Some(line));
            for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
                if let Some(lv) = r.refd(v, f) {
                    crate::render::draw_label(c, r, lv, fc);
                }
            }
            return;
        }
        // a part: a box, solid when composite and dashed otherwise, without a shadow (UMLPartView,
        // elements.js:6977-6984); a constraint property: a box rounded by 8 (elements.js:1765-1775)
        "SysMLPartView" | "SysMLConstraintPropertyView" => {
            if ty == "SysMLConstraintPropertyView" {
                c.polygon(&crate::render::round_rect(b, 8.0), Some(fill), Some(line));
            } else {
                c.rect(b, Some(fill), None);
                let pts = [P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }, P { x: b.x, y: b.y }];
                let composite = r.refd(v, "model").map(|m| r.str(m, "aggregation") == "composite").unwrap_or(false);
                if composite {
                    c.polyline(&pts, line, false);
                } else {
                    crate::render::dashes(c, &pts, line, 5.0, 5.0);
                }
            }
            crate::render::draw_name(c, r, v);
            return;
        }
        _ => {}
    }
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

/// The room around a part's or a constraint property's name (sizeAsCanonicalForm and
/// arrangeAsCanonicalForm, elements.js:1490-1505, 1790-1805).
const PART_MARGIN: f64 = 10.0;
/// A constraint parameter's square (CONSTRAINT_PARAMETER_MINWIDTH, elements.js:27-28).
const PARAMETER_SIZE: f64 = 14.0;

/// A part, reference, value or constraint property: its "name: Type[mult]" at the top, 10 px in
/// from every side (SysMLPartView, SysMLConstraintPropertyView, elements.js:1480-1810).
fn layout_part(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let text = views::feature_text(r, v, m);
    let min_w = ((ms.width(&text, 1) + 2.0 * views::PAD).ceil()).max(views::name_min_width(r, ms, v, m, None, false)) + 2.0 * PART_MARGIN;
    views::grow(r, v, min_w, views::NAME_MIN_H + 2.0 * PART_MARGIN);
    let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
    let nh = views::layout_name(r, ms, v, m, x + PART_MARGIN, y + PART_MARGIN, w - 2.0 * PART_MARGIN, None, false);
    views::grow(r, v, min_w, nh + 2.0 * PART_MARGIN);
    if let Some(nm) = r.refd(v, "nameCompartment").and_then(|nc| r.refd(nc, "nameLabel")) {
        r.set(nm, "text", json!(text));
        let underline = r.bool(m, "isStatic");
        r.set(nm, "underline", json!(underline));
    }
}

/// A constraint parameter: a 14 × 14 square against the inside of the nearest side of its constraint
/// property (or frame), with "name: Type[mult]" floating by it (SysMLConstraintParameterView.update and
/// arrange, elements.js:1840-1905).
fn layout_parameter(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let (mut x, mut y) = (r.f(v, "left"), r.f(v, "top"));
    if let Some(host) = r.refd(v, "containerView").filter(|h| r.alive(*h)) {
        let b = views::node_rect(r, host);
        let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
        let c = P { x: x + PARAMETER_SIZE / 2.0, y: y + PARAMETER_SIZE / 2.0 };
        let (td, bd, ld, rd) = ((y1 - c.y).abs(), (y2 - c.y).abs(), (x1 - c.x).abs(), (x2 - c.x).abs());
        if td < bd && td < ld && td < rd {
            y = y1;
        } else if bd < td && bd < ld && bd < rd {
            y = y2 - PARAMETER_SIZE + 1.0;
        } else if ld < td && ld < bd && ld < rd {
            x = x1;
        } else {
            x = x2 - PARAMETER_SIZE + 1.0;
        }
        x = x.max(x1).min(x2 - PARAMETER_SIZE + 1.0);
        y = y.max(y1).min(y2 - PARAMETER_SIZE + 1.0);
    }
    views::set_box(r, v, x, y, PARAMETER_SIZE, PARAMETER_SIZE);
    let show = Show { vis: r.bool(v, "showVisibility"), mult: r.bool(v, "showMultiplicity"), sig: true, ty: r.bool(v, "showType"), prop: false };
    let name = views::attribute_text(r, m, show);
    let st = views::stereotype_text(r, m);
    let name = if st.is_empty() { name } else { name.trim_start_matches(st.as_str()).trim_start().to_string() };
    for (field, text) in [("nameLabel", name), ("stereotypeLabel", st), ("propertyLabel", String::new())] {
        if let Some(lv) = r.refd(v, field) {
            views::place_floating_label(r, ms, v, lv, &text, true);
            if field == "nameLabel" {
                let underline = r.bool(m, "isStatic");
                r.set(lv, "underline", json!(underline));
            }
        }
    }
}

/// The direction a block's flow properties run: in, out, inout, or none (SysMLBlock.getFlowType,
/// elements.js:500-525).
pub fn flow_type(r: &Repo, block: Eid) -> Option<&'static str> {
    let (mut i, mut o) = (false, false);
    for fp in r.list(block, "flowProperties").into_iter().filter(|x| r.alive(*x)) {
        match r.str(fp, "direction").as_str() {
            "in" => i = true,
            "out" => o = true,
            _ => {
                i = true;
                o = true;
            }
        }
    }
    match (i, o) {
        (true, true) => Some("inout"),
        (true, false) => Some("in"),
        (false, true) => Some("out"),
        _ => None,
    }
}

/// A port: a filled square on its box's border with its labels floating by it (UMLPortView,
/// elements.js:6950-6953); a port typed by a block with flow properties shows their direction with
/// arrows across it, pointing in, out or both ways (SysMLPortView.drawObject, elements.js:1515-1600).
fn draw_port(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba, fill: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    c.rect(b, Some(fill), Some(line));
    let flow = r.refd(v, "model").and_then(|m| r.refd(m, "type")).filter(|t| r.kind_of(*t, "SysMLBlock")).and_then(|t| flow_type(r, t));
    let side = r.refd(v, "containerView").filter(|h| r.alive(*h)).map(|h| {
        let hb = views::node_rect(r, h);
        let p = views::port_junction(hb, P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 });
        if (p.x - hb.x).abs() < 2.0 {
            "left"
        } else if (p.x - (hb.x + hb.w)).abs() < 2.0 {
            "right"
        } else if (p.y - hb.y).abs() < 2.0 {
            "top"
        } else if (p.y - (hb.y + hb.h)).abs() < 2.0 {
            "bottom"
        } else {
            "else"
        }
    });
    if let (Some(flow), Some(side)) = (flow, side) {
        // outward is the way out of the box through this side
        let (inward, outward) = match side {
            "left" => ("right", "left"),
            "right" => ("left", "right"),
            "top" => ("down", "up"),
            "bottom" => ("up", "down"),
            _ => ("", ""),
        };
        match flow {
            "in" => port_arrow(c, b, inward, false, line),
            "out" => port_arrow(c, b, outward, false, line),
            _ => {
                port_arrow(c, b, outward, false, line);
                port_arrow(c, b, inward, true, line);
            }
        }
    }
    for f in ["stereotypeLabel", "nameLabel", "propertyLabel"] {
        if let Some(lv) = r.refd(v, f) {
            crate::render::draw_label(c, r, lv, fc);
        }
    }
}

/// An arrow across a port, 2 px in from its sides, with a 3 px head; only the head with `head_only`
/// (SysMLPortView.drawArrow, elements.js:1530-1565).
fn port_arrow(c: &mut dyn Canvas, b: R, dir: &str, head_only: bool, line: Rgba) {
    let m = 2.0;
    let ctr = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
    let (t, bt, l, rt) = (P { x: ctr.x, y: b.y + m }, P { x: ctr.x, y: b.y + b.h - m }, P { x: b.x + m, y: ctr.y }, P { x: b.x + b.w - m, y: ctr.y });
    let p = |x: f64, y: f64| P { x, y };
    let (from, to, h1, h2) = match dir {
        "up" => (bt, t, p(t.x - 3.0, t.y + 3.0), p(t.x + 3.0, t.y + 3.0)),
        "down" => (t, bt, p(bt.x - 3.0, bt.y - 3.0), p(bt.x + 3.0, bt.y - 3.0)),
        "left" => (rt, l, p(l.x + 3.0, l.y - 3.0), p(l.x + 3.0, l.y + 3.0)),
        "right" => (l, rt, p(rt.x - 3.0, rt.y - 3.0), p(rt.x - 3.0, rt.y + 3.0)),
        _ => return,
    };
    if !head_only {
        c.polyline(&[from, to], line, false);
    }
    c.polyline(&[h1, to, h2], line, false);
}

/// A connector's item flows towards its head (end2) or its tail: their item properties, or what they
/// convey, joined by commas (SysMLConnector.getItemFlows, SysMLConnectorView.update,
/// elements.js:620-650, 1640-1690).
pub fn item_flow_text(r: &Repo, connector: Eid, to_head: bool) -> String {
    let end = r.child(connector, if to_head { "end2" } else { "end1" });
    let target = end.and_then(|e| r.refd(e, "reference")).and_then(|p| r.refd(p, "type"));
    let Some(target) = target else { return String::new() };
    let mut texts = vec![];
    for f in r.list(connector, "ownedElements").into_iter().filter(|f| r.alive(*f) && r.ty(*f) == "SysMLItemFlow" && r.refd(*f, "target") == Some(target)) {
        let t = match r.refd(f, "itemProperty").filter(|p| r.alive(*p)) {
            Some(p) => views::attribute_text(r, p, Show::ALL),
            None => r.ref_list(f, "conveyed").iter().map(|x| r.name(*x)).collect::<Vec<_>>().join(", "),
        };
        if !t.is_empty() {
            texts.push(t);
        }
    }
    texts.join(", ")
}

/// A filled arrow head on a connector where each item flow label hangs, pointing towards the end
/// the items flow to (SysMLConnectorView.drawFlow and drawObject, elements.js:1690-1770).
pub fn draw_item_flows(c: &mut dyn Canvas, r: &Repo, e: Eid, pts: &[P], line: Rgba) {
    if r.ty(e) != "SysMLConnectorView" || pts.len() < 2 {
        return;
    }
    for (field, to_head) in [("itemFlowToHeadLabel", true), ("itemFlowToTailLabel", false)] {
        let Some(l) = r.refd(e, field).filter(|l| shown(r, *l) && !r.str(*l, "text").is_empty()) else { continue };
        let lb = views::node_rect(r, l);
        let cp = P { x: lb.x + lb.w / 2.0, y: lb.y + lb.h / 2.0 };
        // the closest segment and the point on it
        let mut best = (0usize, pts[0], f64::INFINITY);
        for i in 0..pts.len() - 1 {
            let (a, b) = (pts[i], pts[i + 1]);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let l2 = dx * dx + dy * dy;
            let t = if l2 > 0.0 { (((cp.x - a.x) * dx + (cp.y - a.y) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
            let q = P { x: a.x + t * dx, y: a.y + t * dy };
            let d = ((cp.x - q.x).powi(2) + (cp.y - q.y).powi(2)).sqrt();
            if d < best.2 {
                best = (i, q, d);
            }
        }
        let (i, tip, _) = best;
        let back = if to_head { pts[i] } else { pts[i + 1] };
        let th = (back.y - tip.y).atan2(back.x - tip.x);
        let k = 11.0;
        let a1 = th - std::f64::consts::PI / 8.0;
        let a2 = th + std::f64::consts::PI / 8.0;
        let p1 = P { x: tip.x + k * a1.cos(), y: tip.y + k * a1.sin() };
        let p2 = P { x: tip.x + k * a2.cos(), y: tip.y + k * a2.sin() };
        c.polygon(&[p1, tip, p2], Some(line), Some(line));
    }
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
        // Part, Reference, Value and Constraint Property: a property of the block that owns the
        // diagram (blockPropertyPrecondition, sysml-factory.js:30-35, 429-461)
        if let Some((field, view_ty)) = property_tool(kind) {
            let Some(block) = self.repo.parent(d).filter(|p| self.repo.kind_of(*p, "SysMLBlock")) else {
                self.toast_error("SysMLProperty should be placed in a Block");
                return Some(None);
            };
            let m = self.new_sysml_item(block, field)?;
            if field == "constraints" {
                self.repo.set(m, "aggregation", json!("composite"));
            }
            let v = create_part_view(&mut self.repo, d, view_ty, m, rect);
            self.apply_view_defaults(v);
            return Some(Some(v));
        }
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

    /// A new item in one of a SysML element's lists, named after its kind ("Part1"…), a part or a
    /// value composite (factory:create-model with menus/sysml.json's arguments).
    pub fn new_sysml_item(&mut self, owner: Eid, field: &str) -> Option<Eid> {
        let (kind, prefix, composite) = item_kind(field)?;
        let names: Vec<String> = self.repo.list(owner, field).into_iter().map(|x| self.repo.name(x)).collect();
        let name = (1..).map(|i| format!("{prefix}{i}")).find(|n| !names.contains(n)).unwrap();
        let m = self.repo.create(kind, &[("name", json!(name))], Some((owner, field)));
        if composite {
            self.repo.set(m, "aggregation", json!("composite"));
        }
        Some(m)
    }

    /// Model › Add's SysML items ("sysml:parts"…): in a block, parameters in a constraint block
    /// (updateMenus, sysml-commands.js:333-344). None for the kinds UML adds.
    pub fn add_sysml_feature(&mut self, owner: Eid, key: &str) -> Option<Option<Eid>> {
        let field = key.strip_prefix("sysml:")?;
        let ok = if field == "parameters" { self.repo.kind_of(owner, "SysMLConstraintBlock") } else { self.repo.kind_of(owner, "SysMLBlock") };
        Some(if ok { self.new_sysml_item(owner, field) } else { None })
    }

    /// The Port and Constraint Parameter tools, and Add Port: the element goes in the block under
    /// the point, in the type of the part under it, or in the block that owns the diagram when the
    /// point is on its frame; its view sits on that box's border (portFn and parameterFn,
    /// sysml-factory.js:116-228).
    pub fn create_sysml_feature(&mut self, d: Eid, host: Eid, kind: &str, at: P) -> Result<Eid, String> {
        let r = &self.repo;
        let hm = r.refd(host, "model");
        let diagram_block = |m: Eid| (r.kind_of(m, "SysMLInternalBlockDiagram") && r.parent(m).map(|p| r.kind_of(p, "SysMLBlock")).unwrap_or(false)).then(|| r.parent(m)).flatten();
        let owner = if kind == "SysMLPort" {
            let refused = "Port should be placed in a block or a part".to_string();
            match hm {
                Some(m) if r.kind_of(m, "SysMLBlock") => m,
                Some(m) if r.kind_of(m, "SysMLProperty") => r.refd(m, "type").filter(|t| r.kind_of(*t, "SysMLBlock")).ok_or("Port should be placed in a typed part")?,
                Some(m) => diagram_block(m).ok_or(refused)?,
                None => return Err(refused),
            }
        } else {
            let refused = "Constraint Parameter should be placed in a block or a part".to_string();
            match hm {
                Some(m) if r.kind_of(m, "SysMLProperty") => r.refd(m, "type").filter(|t| r.kind_of(*t, "SysMLConstraintBlock")).ok_or("Constraint Parameter should be placed in a ConstraintProperty (typed with ConstraintBlock)")?,
                Some(m) => diagram_block(m).ok_or(refused)?,
                None => return Err(refused),
            }
        };
        let field = if kind == "SysMLPort" { "ports" } else { "parameters" };
        if field == "parameters" && !r.kind_of(owner, "SysMLConstraintBlock") {
            return Err(format!("{} should be placed in a Block", "SysMLProperty"));
        }
        let m = self.new_sysml_item(owner, field).ok_or_else(String::new)?;
        // a parameter made by the tool is composite (parameterFn's model-init)
        if field == "parameters" {
            self.repo.set(m, "aggregation", json!("composite"));
        }
        let ty = if kind == "SysMLPort" { "SysMLPortView" } else { "SysMLConstraintParameterView" };
        let v = create_floating_view(&mut self.repo, d, ty, m, host, at);
        self.apply_view_defaults(v);
        Ok(v)
    }

    /// The Connector tool: a SysMLConnector between two parts or ports, owned by the diagram's
    /// owner, with an end at each (connectorFn, sysml-factory.js:179-188).
    pub fn create_sysml_connector(&mut self, d: Eid, tail: Eid, head: Eid) -> Option<Eid> {
        let (tm, hm) = (self.repo.refd(tail, "model")?, self.repo.refd(head, "model")?);
        let owner = self.package_for(Some(d));
        let m = self.repo.create("SysMLConnector", &[], Some((owner, "ownedElements")));
        let (r1, r2) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
        self.repo.create("UMLConnectorEnd", &[("reference", r1)], Some((m, "end1")));
        self.repo.create("UMLConnectorEnd", &[("reference", r2)], Some((m, "end2")));
        views::create_edge_view(&mut self.repo, d, Some(m), tail, head)
    }

    /// A SysML element dragged from the explorer: a port onto its block's box on a block definition
    /// diagram (viewForBlockDefinitionDiagramFn, sysml-factory.js:234-261). None when UML places it.
    pub fn drop_sysml(&mut self, d: Eid, m: Eid, p: P) -> Option<Result<Vec<Eid>, String>> {
        let r = &self.repo;
        let under = |want: &dyn Fn(Eid) -> bool| {
            r.list(d, "ownedViews").into_iter().rev().find(|v| {
                !views::is_edge(r, *v) && r.raw(*v, "visible").and_then(|x| x.as_bool()) != Some(false) && want(*v) && {
                    let b = views::node_rect(r, *v);
                    p.x >= b.x && p.y >= b.y && p.x <= b.x + b.w && p.y <= b.y + b.h
                }
            })
        };
        let floating_on = |a: &mut App, host: Eid, ty: &str| {
            a.change(|a| {
                let v = create_floating_view(&mut a.repo, d, ty, m, host, p);
                a.apply_view_defaults(v);
                v
            })
        };
        if r.ty(d) == "SysMLBlockDefinitionDiagram" && r.ty(m) == "SysMLPort" {
            let owner = r.parent(m);
            let Some(host) = under(&|v| r.kind_of(v, "SysMLBlockView") && r.refd(v, "model") == owner) else {
                return Some(Err("Port should be dropped on it's parent Block".into()));
            };
            return Some(Ok(vec![floating_on(self, host, "SysMLPortView")]));
        }
        // an internal block or parametric diagram (viewForInternalBlockDiagramFn, sysml-factory.js:263-376)
        if !r.kind_of(d, "SysMLInternalBlockDiagram") {
            return None;
        }
        let block = r.parent(d);
        let owner = r.parent(m);
        if r.kind_of(m, "SysMLProperty") {
            // a property of the diagram's block: a part view, or a constraint property view for one
            // of its constraints
            if owner.is_some() && owner == block {
                let ty = if r.field_of(m).as_deref() == Some("constraints") { "SysMLConstraintPropertyView" } else { "SysMLPartView" };
                let v = self.change(|a| {
                    let v = create_part_view(&mut a.repo, d, ty, m, R { x: p.x, y: p.y, w: 1.0, h: 1.0 });
                    a.apply_view_defaults(v);
                    v
                });
                return Some(Ok(vec![v]));
            }
            // a constraint block's parameter: onto a constraint property typed by that block
            if r.field_of(m).as_deref() == Some("parameters") && owner.map(|o| r.kind_of(o, "SysMLConstraintBlock")).unwrap_or(false) {
                let Some(host) = under(&|v| r.ty(v) == "SysMLConstraintPropertyView" && r.refd(v, "model").and_then(|x| r.refd(x, "type")) == owner) else {
                    return Some(Err("Constraint Parameter should be dropped on a it's parent Constraint Parameter".into()));
                };
                return Some(Ok(vec![floating_on(self, host, "SysMLConstraintParameterView")]));
            }
            return Some(Err("Only part in the block context can be dropped".into()));
        }
        // a port: onto a part typed by its block, or onto the frame
        if r.ty(m) == "SysMLPort" {
            let Some(host) = under(&|v| (r.ty(v) == "SysMLPartView" && r.refd(v, "model").and_then(|x| r.refd(x, "type")) == owner) || r.ty(v) == "UMLFrameView") else {
                return Some(Err("Port should be dropped on a part whose type is the port's parent Block".into()));
            };
            return Some(Ok(vec![floating_on(self, host, "SysMLPortView")]));
        }
        // a block: a new composite part of the diagram's block, typed by it
        if r.kind_of(m, "SysMLBlock") {
            let block = block.filter(|b| r.kind_of(*b, "SysMLBlock"))?;
            let v = self.change(|a| {
                let part = a.new_sysml_item(block, "parts")?;
                a.repo.set_ref(part, "type", Some(m));
                let v = create_part_view(&mut a.repo, d, "SysMLPartView", part, R { x: p.x, y: p.y, w: 1.0, h: 1.0 });
                a.apply_view_defaults(v);
                Some(v)
            });
            return Some(Ok(v.into_iter().collect()));
        }
        None
    }

    /// Create Block, Create Value Type, Create ConstraintBlock: a new element of that kind named as
    /// typed, in the owner of the edited element (of its block, for a constraint property), set as
    /// its type (handleCreateBlock and the others, sysml-commands.js:96-170, 230-252).
    pub fn create_sysml_type(&mut self, e: Eid, kind: &str, text: &str) {
        let name = text.trim().to_string();
        if !self.repo.alive(e) || name.is_empty() {
            return;
        }
        let owner = if kind == "SysMLConstraintBlock" { self.repo.parent(e).and_then(|p| self.repo.parent(p)) } else { self.repo.parent(e) };
        let Some(owner) = owner else { return };
        let kind = kind.to_string();
        let t = self.change(|a| {
            let t = a.repo.create(&kind, &[("name", json!(name))], Some((owner, "ownedElements")));
            a.repo.set_ref(e, "type", Some(t));
            t
        });
        self.reveal(t);
    }

    /// Create Item Flow on a connector: both ends must be typed; then a dialog asks which way the
    /// items flow (handleCreateItemFlow, sysml-commands.js:175-215).
    pub fn ask_item_flow(&mut self, c: Eid) {
        let ty = |end: &str| self.repo.child(c, end).and_then(|x| self.repo.refd(x, "reference")).and_then(|p| self.repo.refd(p, "type")).filter(|t| self.repo.kind_of(*t, "UMLClassifier"));
        let (Some(t1), Some(t2)) = (ty("end1"), ty("end2")) else {
            self.toast_error("Connected Parts (or Ports) should be typed");
            return;
        };
        let (n1, n2) = (self.repo.name(t1), self.repo.name(t2));
        self.select_one = Some(crate::app::SelectOne {
            message: "Select direction of ItemFlow".into(),
            // the dialog's options are static texts; these two are made once per request
            options: vec![Box::leak(format!("From {n2} To {n1}").into_boxed_str()), Box::leak(format!("From {n1} To {n2}").into_boxed_str())],
            selected: 0,
            purpose: crate::app::SelectPurpose::ItemFlow(c),
        });
    }

    /// The item flow itself: from one end's type to the other's, in the connector, realized by it.
    pub fn add_item_flow(&mut self, c: Eid, to_end2: bool) {
        let ty = |end: &str| self.repo.child(c, end).and_then(|x| self.repo.refd(x, "reference")).and_then(|p| self.repo.refd(p, "type"));
        let (Some(t1), Some(t2)) = (ty("end1"), ty("end2")) else { return };
        let (s, t) = if to_end2 { (t1, t2) } else { (t2, t1) };
        let f = self.change(|a| {
            let (rs, rt) = (a.repo.ref_to(s), a.repo.ref_to(t));
            let f = a.repo.create("SysMLItemFlow", &[("source", rs), ("target", rt)], Some((c, "ownedElements")));
            a.repo.set_ref_list(f, "realizingConnectors", &[c]);
            f
        });
        self.reveal(f);
    }

    /// Add an item to one of a box's compartments, showing the compartment, and edit it
    /// (uml:add-compartment-item with quickedits/sysml.json's arguments, uml-commands.js:784-823).
    pub fn add_sysml_item(&mut self, host: Eid, owner: Eid, field: &str) {
        let flag = specs(self.repo.ty(host)).and_then(|sp| sp.iter().find(|s| s.items == field)).map(|s| s.flag);
        let new = self.change(|a| {
            if let Some(f) = flag.filter(|f| a.repo.bool(host, f)) {
                a.repo.set(host, f, json!(false));
            }
            a.new_sysml_item(owner, field)
        });
        if let Some(n) = new {
            self.focus = Some(n);
            self.edit_item(host, n);
        }
    }

    /// Add Port on a block or a part: a port on its right border, under the ones there (the
    /// original picks a random point on the box, handleAddPort, sysml-commands.js:64-82).
    fn add_sysml_port(&mut self, host: Eid) {
        let Some(d) = self.diagram else { return };
        let b = views::node_rect(&self.repo, host);
        let n = self.repo.list(d, "ownedViews").into_iter().filter(|v| self.repo.refd(*v, "containerView") == Some(host) && self.repo.ty(*v) == "SysMLPortView").count() as f64;
        let at = P { x: b.x + b.w, y: (b.y + 30.0 + 20.0 * n).min(b.y + b.h - views::PORT_SIZE) };
        let made = self.change(|a| a.create_sysml_feature(d, host, "SysMLPort", at).map_err(|e| a.toast_error(&e)).ok());
        if let Some(v) = made {
            self.select(vec![v]);
            self.pending_edit = Some(v);
        }
    }

    /// Run a SysML quick-edit button.
    fn run_quick(&mut self, host: Eid, m: Eid, q: Quick) {
        match q {
            Quick::AddItem(field) => self.add_sysml_item(host, m, field),
            Quick::AddPort => self.add_sysml_port(host),
            // a new block below, joined by an association whose end at the edited block is
            // composite (handleAddCompositedBlock, sysml-commands.js:39-62)
            Quick::CompositedBlock => {
                let rel = crate::related::Related { label: "Add Composited Block", kind: "SysMLBlock", rel: "UMLAssociation", init: crate::app::AssocInit::CompositeTail, from_new: false, place: crate::related::Place::Below };
                self.add_related(host, rel);
            }
            // the Element Picker on blocks, value types or constraint blocks (handleSelectBlock and
            // the others, sysml-commands.js:84-95)
            Quick::SelectType(kind) => self.open_typed_picker(m, "type", kind),
            Quick::CreateType(kind) => {
                let what = match kind {
                    "SysMLBlock" => "Enter name of Block to create",
                    "SysMLValueType" => "Enter name of ValueType to create",
                    _ => "Enter name of a ConstraintBlock to create",
                };
                self.input_dialog = Some(crate::input::InputDialog { message: what.into(), text: String::new(), action: crate::input::InputAction::CreateSysMLType(m, kind), fresh: true });
            }
            // Add Constraint Parameter: on the constraint property's right border, under the ones
            // there (the original picks a random point inside it, sysml-commands.js:254-272)
            Quick::ConstraintParameter => {
                let Some(d) = self.diagram else { return };
                let b = views::node_rect(&self.repo, host);
                let n = self.repo.list(d, "ownedViews").into_iter().filter(|v| self.repo.refd(*v, "containerView") == Some(host) && self.repo.ty(*v) == "SysMLConstraintParameterView").count() as f64;
                let at = P { x: b.x + b.w, y: (b.y + 30.0 + 20.0 * n).min(b.y + b.h - PARAMETER_SIZE) };
                let made = self.change(|a| a.create_sysml_feature(d, host, "SysMLConstraintParameter", at).map_err(|e| a.toast_error(&e)).ok());
                if let Some(v) = made {
                    self.select(vec![v]);
                    self.pending_edit = Some(v);
                }
            }
        }
    }
}

/// A part's, reference's, value's or constraint property's view (SysMLPartView,
/// SysMLConstraintPropertyView).
fn create_part_view(r: &mut Repo, diagram: Eid, ty: &str, model: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        ty,
        &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("showVisibility", json!(false)), ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h))],
        Some((diagram, "ownedViews")),
    );
    let nc = views::name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    v
}

/// A port's or a constraint parameter's view: a 14 × 14 square at the point, its labels floating by
/// it, contained in `host` (UMLPortView, SysMLConstraintParameterView, elements.js:1480, 1880).
fn create_floating_view(r: &mut Repo, diagram: Eid, ty: &str, model: Eid, host: Eid, at: P) -> Eid {
    let h = views::PORT_SIZE / 2.0;
    let m = r.ref_to(model);
    let v = r.create(
        ty,
        &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(at.x - h)), ("top", json!(at.y - h)), ("width", json!(views::PORT_SIZE)), ("height", json!(views::PORT_SIZE))],
        Some((diagram, "ownedViews")),
    );
    views::floating_labels(r, v, model);
    views::contain(r, host, v);
    v
}

/// Whether Model › Add Diagram offers this SysML kind on the selected element: requirement and
/// block definition diagrams on nothing, the project or a package, an internal block diagram on a
/// block and a parametric diagram on a constraint block (updateMenus, sysml-commands.js:318-322).
/// None for other kinds.
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
        v.extend([
            ("Requirement", "SysMLRequirement", false), ("Block", "SysMLBlock", false), ("Value Type", "SysMLValueType", false), ("Interface Block", "SysMLInterfaceBlock", false),
            ("Constraint Block", "SysMLConstraintBlock", false),
        ]);
    }
    if r.kind_of(e, "SysMLBlock") {
        v.extend([
            ("Part", "sysml:parts", true), ("Reference", "sysml:references", true), ("Value", "sysml:values", true), ("Port", "sysml:ports", true), ("Property", "sysml:properties", true),
            ("Constraint Property", "sysml:constraints", true), ("Flow Property", "sysml:flowProperties", true),
        ]);
    }
    if r.kind_of(e, "SysMLConstraintBlock") {
        v.push(("Parameter", "sysml:parameters", true));
    }
    v
}

/// The SysML elements' visible properties (sysml/metamodel.json `visible` attributes after the UML
/// ones of their super-kinds).
pub fn fields(r: &Repo, e: Eid) -> Option<Vec<(&'static str, &'static str)>> {
    let classifier = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isAbstract", "bool"), ("isFinalSpecialization", "bool"), ("isLeaf", "bool")];
    let class = |more: &[(&'static str, &'static str)]| {
        let mut f = classifier.clone();
        f.push(("isActive", "bool"));
        f.extend_from_slice(more);
        f
    };
    let viewpoint = |more: &[(&'static str, &'static str)]| {
        let mut f = classifier.clone();
        f.push(("isActive", "bool"));
        f.extend_from_slice(more);
        f
    };
    let attribute = vec![
        ("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isStatic", "bool"), ("isLeaf", "bool"), ("type", "type"), ("multiplicity", "text"), ("isReadOnly", "bool"),
        ("isOrdered", "bool"), ("isUnique", "bool"), ("defaultValue", "text"), ("isDerived", "bool"), ("aggregation", "enum"), ("isID", "bool"),
    ];
    Some(match r.ty(e) {
        "SysMLRequirement" => class(&[("id", "text"), ("text", "text")]),
        "SysMLBlock" | "SysMLInterfaceBlock" | "SysMLConstraintBlock" => class(&[("isEncapsulated", "bool")]),
        "SysMLStakeholder" => {
            let mut f = classifier.clone();
            f.push(("concern", "multiline"));
            f
        }
        "SysMLView" => viewpoint(&[]),
        "SysMLViewpoint" => viewpoint(&[("stakeholders", "reflist"), ("purpose", "text"), ("language", "text"), ("presentation", "text")]),
        "SysMLValueType" => {
            let mut f = classifier;
            f.extend([("unit", "ref"), ("quantityKind", "ref")]);
            f
        }
        "SysMLProperty" | "SysMLPort" => attribute,
        "SysMLFlowProperty" => {
            let mut f = attribute;
            f.push(("direction", "enum"));
            f
        }
        _ => return None,
    })
}

/// The choices of a SysML enumeration (SysMLFlowDirectionKind, sysml/metamodel.json).
pub fn enum_options(ty: &str, field: &str) -> Option<&'static [&'static str]> {
    match (ty, field) {
        ("SysMLFlowProperty", "direction") => Some(&["in", "out", "inout"]),
        _ => None,
    }
}

/// The icon of a SysML element in the explorer.
pub fn icon_key(ty: &str) -> &'static str {
    match ty {
        "SysMLRequirement" => "SysMLRequirement",
        "SysMLBlock" | "SysMLInterfaceBlock" => "SysMLBlock",
        "SysMLConstraintBlock" => "SysMLConstraintBlock",
        "SysMLValueType" => "datatype",
        "SysMLProperty" | "SysMLFlowProperty" => "attribute",
        "SysMLPort" => "port",
        "SysMLConnector" => "association",
        "SysMLCopy" | "SysMLDeriveReqt" | "SysMLVerify" | "SysMLSatisfy" | "SysMLRefine" | "SysMLTrace" | "SysMLExpose" | "SysMLItemFlow" => "dependency",
        "SysMLConform" => "generalization",
        "SysMLRequirementDiagram" => "SysMLRequirementDiagram",
        "SysMLBlockDefinitionDiagram" => "diagram-class",
        "SysMLInternalBlockDiagram" | "SysMLParametricDiagram" => "SysMLInternalBlockDiagram",
        _ if ty.ends_with("Diagram") => "diagram",
        _ => "class",
    }
}

// ------------------------------------------------------------------ quick edits

/// A SysML quick-edit button (quickedits/sysml.json components).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Quick {
    /// Add Part, Reference, Value, Property, Flow Property, Constraint Property or Parameter: an item
    /// in this list of the element
    AddItem(&'static str),
    /// Add Port on a block or a part
    AddPort,
    /// Add Composited Block on a block
    CompositedBlock,
    /// Select Block, Value Type or ConstraintBlock as the type of a part, port, constraint property
    /// or parameter
    SelectType(&'static str),
    /// Create Block, Value Type or ConstraintBlock as its type
    CreateType(&'static str),
    /// Add Constraint Parameter on a constraint property
    ConstraintParameter,
}

type Button = (&'static str, &'static str, &'static str, crate::canvas::ItemKey);

/// The buttons of a SysML element's name edit, besides the related-element ones
/// (quickedits/sysml.json). None for the elements SysML leaves to UML.
pub fn item_buttons(r: &Repo, host: Option<Eid>, m: Eid) -> Option<Vec<Button>> {
    use crate::canvas::ItemKey;
    let add = |label: &'static str, hint: &'static str, field: &'static str| -> Button { (label, hint, "", ItemKey::SysML(Quick::AddItem(field))) };
    let q = |label: &'static str, hint: &'static str, q: Quick| -> Button { (label, hint, "", ItemKey::SysML(q)) };
    let block_type = [q("Block…", "Select Block", Quick::SelectType("SysMLBlock")), q("+ Block", "Create Block", Quick::CreateType("SysMLBlock"))];
    // a part, port, constraint property or parameter, by the view edited ("sysml.part", "sysml.port",
    // "sysml.constraint-property", "sysml.constraint-parameter")
    match host.map(|h| r.ty(h)) {
        Some("SysMLPartView") => {
            let mut v = block_type.to_vec();
            v.extend([q("ValueType…", "Select Value Type", Quick::SelectType("SysMLValueType")), q("+ ValueType", "Create Value Type", Quick::CreateType("SysMLValueType")), q("+ Port", "Add Port", Quick::AddPort)]);
            return Some(v);
        }
        Some("SysMLPortView") => {
            let mut v = block_type.to_vec();
            v.extend([("+ Prov", "Add Provided Interface", "", ItemKey::PortInterface(true)), ("+ Req", "Add Required Interface", "", ItemKey::PortInterface(false))]);
            return Some(v);
        }
        Some("SysMLConstraintPropertyView") => {
            return Some(vec![
                q("Constraint…", "Select ConstraintBlock", Quick::SelectType("SysMLConstraintBlock")),
                q("+ Constraint", "Create ConstraintBlock", Quick::CreateType("SysMLConstraintBlock")),
                q("+ Param", "Add Constraint Parameter", Quick::ConstraintParameter),
            ]);
        }
        Some("SysMLConstraintParameterView") => return Some(block_type.to_vec()),
        _ => {}
    }
    Some(match r.ty(m) {
        // a requirement only adds a sub-requirement, a related element ("sysml.requirement")
        "SysMLRequirement" => vec![],
        // "sysml.block": Add Property takes Ctrl+Enter
        "SysMLBlock" | "SysMLInterfaceBlock" => vec![
            add("+ Part", "Add Part", "parts"),
            add("+ Ref", "Add Reference", "references"),
            add("+ Value", "Add Value", "values"),
            add("+ Flow", "Add Flow Property", "flowProperties"),
            add("+ Constraint", "Add Constraint Property", "constraints"),
            ("+ Prop", "Add Property", "Ctrl+Enter", ItemKey::SysML(Quick::AddItem("properties"))),
            ("+ Op", "Add Operation", "Ctrl+Shift+Enter", ItemKey::AddOperation),
            ("+ Rec", "Add Reception", "", ItemKey::AddReception),
            ("+ Port", "Add Port", "", ItemKey::SysML(Quick::AddPort)),
            ("+ Composited", "Add Composited Block", "", ItemKey::SysML(Quick::CompositedBlock)),
        ],
        // "sysml.constraint-block": its "sysml.add-constraint" names no component, so only Add
        // Parameter shows
        "SysMLConstraintBlock" => vec![add("+ Param", "Add Parameter", "parameters")],
        // "sysml.classifier": attributes and operations, no receptions
        "SysMLStakeholder" | "SysMLView" | "SysMLViewpoint" => vec![("+ Attr", "Add Attribute", "Ctrl+Enter", ItemKey::Add), ("+ Op", "Add Operation", "Ctrl+Shift+Enter", ItemKey::AddOperation)],
        _ => return None,
    })
}

impl App {
    /// The keys and buttons of a SysML element's inline edit; false when UML handles them.
    pub(crate) fn sysml_item_key(&mut self, host: Eid, m: Eid, item: Option<Eid>, key: crate::canvas::ItemKey) -> bool {
        use crate::canvas::ItemKey;
        let t = self.repo.ty(m).to_string();
        if let ItemKey::SysML(q) = key {
            self.run_quick(host, m, q);
            return true;
        }
        match (key, item) {
            // Ctrl+Enter on a block's name adds a property; a requirement's and a constraint block's
            // name edits have no attribute or operation key
            (ItemKey::Add, None) if matches!(t.as_str(), "SysMLBlock" | "SysMLInterfaceBlock") => {
                self.add_sysml_item(host, m, "properties");
                true
            }
            (ItemKey::Add | ItemKey::AddOperation, None) => matches!(t.as_str(), "SysMLRequirement" | "SysMLConstraintBlock"),
            // Add on a SysML compartment item: another item in the same list (the original's buttons
            // carry no arguments here)
            (ItemKey::Add, Some(f)) => match (self.repo.parent(f), self.repo.field_of(f)) {
                (Some(owner), Some(field)) if self.repo.ty(f).starts_with("SysML") => {
                    if let Some(field) = ["parts", "references", "values", "properties", "constraints", "flowProperties", "parameters", "ports"].into_iter().find(|x| *x == field) {
                        self.add_sysml_item(host, owner, field);
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }
}

/// Preferences › SysML, in the original's order (preferences/sysml.json).
pub fn preferences() -> Vec<(&'static str, &'static str, &'static str, crate::prefs::Kind, Value)> {
    use crate::prefs::Kind;
    let mut v = vec![];
    let mut fill = |label: &'static str, key: &'static str, extra: &[(&'static str, &'static str)]| {
        v.push(("SysML", "", label, Kind::Header, Value::Null));
        v.push(("SysML", key, "Fill Color", Kind::Color, Value::Null));
        for (k, l) in extra {
            v.push(("SysML", *k, *l, Kind::Check, json!(true)));
        }
    };
    fill("Requirement", "sysml.requirement.fillColor", &[]);
    fill(
        "Block",
        "sysml.block.fillColor",
        &[
            ("sysml.block.suppressConstraints", "Suppress Constraints"), ("sysml.block.suppressParts", "Suppress Parts"), ("sysml.block.suppressPorts", "Suppress Ports"),
            ("sysml.block.suppressReferences", "Suppress References"), ("sysml.block.suppressValues", "Suppress Values"), ("sysml.block.suppressFlowProperties", "Suppress FlowProperties"),
        ],
    );
    fill("InterfaceBlock", "sysml.interface-block.fillColor", &[]);
    fill("ConstraintBlock", "sysml.constraint-block.fillColor", &[]);
    fill("ValueType", "sysml.value-type.fillColor", &[]);
    fill("Part", "sysml.part.fillColor", &[]);
    fill("Port", "sysml.port.fillColor", &[]);
    fill("ConstraintProperty", "sysml.constraint-property.fillColor", &[]);
    fill("ConstraintParameter", "sysml.constraint-parameter.fillColor", &[]);
    v
}

/// The fill colour preference of a SysML view (the views' constructors, elements.js:845-1700).
pub fn fill_pref(view_ty: &str) -> Option<&'static str> {
    Some(match view_ty {
        "SysMLRequirementView" => "sysml.requirement.fillColor",
        "SysMLBlockView" => "sysml.block.fillColor",
        "SysMLInterfaceBlockView" => "sysml.interface-block.fillColor",
        "SysMLConstraintBlockView" => "sysml.constraint-block.fillColor",
        "SysMLValueTypeView" => "sysml.value-type.fillColor",
        "SysMLPortView" => "sysml.port.fillColor",
        "SysMLPartView" => "sysml.part.fillColor",
        "SysMLConstraintPropertyView" => "sysml.constraint-property.fillColor",
        "SysMLConstraintParameterView" => "sysml.constraint-parameter.fillColor",
        _ => return None,
    })
}

/// The compartments a new block view suppresses by preference: an interface block's too, and a
/// constraint block's constraints (SysMLBlockView and SysMLConstraintBlockView constructors,
/// elements.js:1080-1110, 1390-1396).
pub fn suppress_prefs(view_ty: &str) -> &'static [(&'static str, &'static str)] {
    match view_ty {
        "SysMLBlockView" | "SysMLInterfaceBlockView" => &[
            ("suppressConstraints", "sysml.block.suppressConstraints"), ("suppressParts", "sysml.block.suppressParts"), ("suppressPorts", "sysml.block.suppressPorts"),
            ("suppressReferences", "sysml.block.suppressReferences"), ("suppressValues", "sysml.block.suppressValues"), ("suppressFlowProperties", "sysml.block.suppressFlowProperties"),
        ],
        "SysMLConstraintBlockView" => &[("suppressConstraints", "sysml.block.suppressConstraints")],
        _ => &[],
    }
}

// ------------------------------------------------------------------ Mermaid requirement diagrams

/// The Mermaid dialog's requirement example (UMLCraft's own).
pub const MERMAID_SAMPLE: &str = "requirementDiagram
    requirement brakes {
        id: 1
        text: \"The car stops within 40 m from 100 km/h.\"
        risk: high
        verifymethod: test
    }
    functionalRequirement abs {
        id: 1.1
        text: \"The wheels do not lock.\"
        risk: medium
        verifymethod: analysis
    }
    element brake_test {
        type: simulation
        docref: tests/brakes.md
    }
    brakes - contains -> abs
    brake_test - verifies -> brakes
";

/// A Mermaid requirement or element block: its name, kind and fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MBlock {
    pub name: String,
    /// the requirement type ("requirement", "functionalRequirement"…), or "element"
    pub kind: String,
    pub id: String,
    pub text: String,
    pub risk: String,
    pub verify: String,
    pub ty: String,
    pub docref: String,
}

/// A Mermaid requirement diagram: its blocks, its relations (kind, source, destination) and its
/// direction.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MRequirements {
    pub blocks: Vec<MBlock>,
    pub rels: Vec<(String, String, String)>,
    pub direction: String,
}

const REQUIREMENT_TYPES: [&str; 6] = ["requirement", "functionalRequirement", "interfaceRequirement", "performanceRequirement", "physicalRequirement", "designConstraint"];
const RELATIONS: [&str; 7] = ["contains", "copies", "derives", "satisfies", "verifies", "refines", "traces"];

/// Read Mermaid's requirement diagram syntax (requirementDiagram.jison, requirementDb.js): blocks of
/// `id`, `text`, `risk` and `verifymethod` (an element's `type` and `docref`), relations written
/// `a - satisfies -> b` or `b <- satisfies - a`, and `direction`; styles and classes are ignored.
pub fn parse_mermaid(code: &str) -> Result<MRequirements, String> {
    let fail = || "Failed to parse requirement diagram".to_string();
    let mut out = MRequirements { direction: "TB".into(), ..Default::default() };
    let lines = crate::mermaid_kinds::lines(code);
    let mut it = lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty());
    if !it.next().map(|l| l.starts_with("requirementDiagram")).unwrap_or(false) {
        return Err(fail());
    }
    let mut cur: Option<MBlock> = None;
    let unq = crate::mermaid_kinds::unquote;
    for l in it {
        if let Some(b) = cur.as_mut() {
            if l == "}" {
                let b = cur.take().unwrap_or_default();
                if !out.blocks.iter().any(|x| x.name == b.name) {
                    out.blocks.push(b);
                }
                continue;
            }
            let (k, v) = l.split_once(':').ok_or_else(fail)?;
            let v = unq(v);
            // the lexer's keywords, matched without regard to case
            let cap = |v: &str, words: &[&str]| words.iter().find(|w| w.eq_ignore_ascii_case(v)).map(|w| w.to_string()).unwrap_or_else(|| v.to_string());
            match k.trim().to_ascii_lowercase().as_str() {
                "id" => b.id = v,
                "text" => b.text = v,
                "risk" => b.risk = cap(&v, &["Low", "Medium", "High"]),
                "verifymethod" => b.verify = cap(&v, &["Analysis", "Demonstration", "Inspection", "Test"]),
                "type" => b.ty = v,
                "docref" => b.docref = v,
                _ => return Err(fail()),
            }
            continue;
        }
        if let Some(d) = l.strip_prefix("direction") {
            out.direction = d.trim().to_string();
            continue;
        }
        if ["style ", "classDef ", "class ", "accTitle", "accDescr", "title "].iter().any(|p| l.starts_with(p)) {
            continue;
        }
        // a block: `<kind> <name> {`
        if let Some(head) = l.strip_suffix('{') {
            let (kind, name) = head.trim().split_once(char::is_whitespace).ok_or_else(fail)?;
            let kind = REQUIREMENT_TYPES.iter().chain(["element"].iter()).find(|k| k.eq_ignore_ascii_case(kind)).ok_or_else(fail)?;
            cur = Some(MBlock { name: unq(name), kind: kind.to_string(), ..Default::default() });
            continue;
        }
        // a relation
        let rel = |s: &str| RELATIONS.iter().find(|r| r.eq_ignore_ascii_case(s.trim())).map(|r| r.to_string());
        if let Some((a, rest)) = l.split_once("<-") {
            let (kind, b) = rest.rsplit_once('-').ok_or_else(fail)?;
            out.rels.push((rel(kind).ok_or_else(fail)?, unq(b), unq(a)));
            continue;
        }
        if let Some((left, b)) = l.split_once("->") {
            let (a, kind) = left.split_once('-').ok_or_else(fail)?;
            out.rels.push((rel(kind).ok_or_else(fail)?, unq(a), unq(b)));
            continue;
        }
        return Err(fail());
    }
    if cur.is_some() {
        return Err(fail());
    }
    Ok(out)
}

impl App {
    /// A Mermaid requirement diagram: "Requirement Diagram by Mermaid" in the selected package or
    /// a new "Package by Mermaid", without a frame; requirements with their id and text, their risk
    /// and verify method in the documentation and their type as a stereotype; elements as classes
    /// «element» with Type and DocRef attributes; `contains` moves the element into the container
    /// with a containment line, `traces` is a «trace» dependency, the others the SysML relationships
    /// (createRequirementDiagram, requirement-factory.js:15-300).
    pub fn generate_requirement(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let md = parse_mermaid(code)?;
        // the base package, else a new "Package by Mermaid" in the project
        let base = base.filter(|f| self.repo.alive(*f) && *f != self.repo.root && self.repo.kind_of(*f, "UMLPackage"));
        let undo = self.undo.len();
        let d = self.change(|a| {
            let root = a.repo.root;
            let pkg = base.unwrap_or_else(|| a.repo.create("UMLPackage", &[("name", json!("Package by Mermaid"))], Some((root, "ownedElements"))));
            let d = a.repo.create("SysMLRequirementDiagram", &[("name", json!("Requirement Diagram by Mermaid"))], Some((pkg, "ownedElements")));
            let mut made: Vec<(String, Eid, Eid)> = vec![];
            // requirements first, then elements (requirement-factory.js:52-62)
            let blocks = md.blocks.iter().filter(|b| b.kind != "element").chain(md.blocks.iter().filter(|b| b.kind == "element"));
            for (i, b) in blocks.enumerate() {
                let at = R { x: 10.0 + i as f64, y: 10.0, w: 1.0, h: 1.0 };
                let (m, v) = if b.kind == "element" {
                    let m = a.repo.create("UMLClass", &[("name", json!(b.name)), ("stereotype", json!("element"))], Some((pkg, "ownedElements")));
                    for (n, val) in [("Type", &b.ty), ("DocRef", &b.docref)] {
                        a.repo.create("UMLAttribute", &[("name", json!(n)), ("defaultValue", json!(val))], Some((m, "attributes")));
                    }
                    let v = views::create_node_view(&mut a.repo, d, m, at);
                    if let Some(v) = v {
                        a.repo.set(v, "suppressOperations", json!(true));
                    }
                    (m, v)
                } else {
                    let m = a.repo.create("SysMLRequirement", &[("name", json!(b.name)), ("id", json!(b.id)), ("text", json!(b.text))], Some((pkg, "ownedElements")));
                    let mut doc = String::new();
                    if !b.risk.is_empty() {
                        doc += &format!("Risk: {}\n", b.risk);
                    }
                    if !b.verify.is_empty() {
                        doc += &format!("VerifyMethod: {}\n", b.verify);
                    }
                    a.repo.set(m, "documentation", json!(doc));
                    if b.kind != "requirement" {
                        a.repo.set(m, "stereotype", json!(b.kind));
                    }
                    (m, create_view(&mut a.repo, d, m, at, false))
                };
                if let Some(v) = v {
                    a.apply_view_defaults(v);
                    made.push((b.name.clone(), m, v));
                }
            }
            for (kind, src, dst) in &md.rels {
                let (Some(s), Some(t)) = (made.iter().find(|x| x.0 == *src).cloned(), made.iter().find(|x| x.0 == *dst).cloned()) else { continue };
                if kind == "contains" {
                    if a.repo.parent(t.1) != Some(s.1) {
                        a.repo.relocate(t.1, s.1, "ownedElements");
                    }
                    let props = [("font", json!(views::FONT)), ("head", a.repo.ref_to(s.2)), ("tail", a.repo.ref_to(t.2)), ("lineStyle", json!(views::DEFAULT_LINE_STYLE)), ("points", json!("0:0;0:0"))];
                    let e = a.repo.create("UMLContainmentView", &props, Some((d, "ownedViews")));
                    a.apply_view_defaults(e);
                    continue;
                }
                let ty = match kind.as_str() {
                    "satisfies" => "SysMLSatisfy",
                    "copies" => "SysMLCopy",
                    "derives" => "SysMLDeriveReqt",
                    "verifies" => "SysMLVerify",
                    "refines" => "SysMLRefine",
                    _ => "UMLDependency",
                };
                let (rs, rt) = (a.repo.ref_to(s.1), a.repo.ref_to(t.1));
                let m = a.repo.create(ty, &[("source", rs), ("target", rt)], Some((s.1, "ownedElements")));
                if kind == "traces" {
                    a.repo.set(m, "stereotype", json!("trace"));
                }
                if let Some(e) = views::create_edge_view(&mut a.repo, d, Some(m), s.2, t.2) {
                    a.apply_view_defaults(e);
                }
            }
            d
        });
        Ok(self.finish_mermaid(d, undo, crate::mermaid_kinds::dir(&md.direction, true), ms))
    }
}
