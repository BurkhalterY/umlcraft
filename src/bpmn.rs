//! BPMN diagrams: processes with activities, events, gateways and data, pools and lanes,
//! collaborations with messages and message flows, choreographies and conversations, all on one
//! BPMNDiagram (extensions/essential/bpmn: metamodel.json, elements.js, bpmn-factory.js,
//! bpmn-commands.js, quickedits/bpmn.json). This file holds the kinds, the factory and the
//! commands; `bpmn_draw.rs` lays the views out and draws them.

use crate::app::{App, Editing, EditKind};
use crate::meta::Kind;
use crate::model::{Eid, Repo};
use crate::views::{self, P, R};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

/// The padding around a name (LEFT_PADDING … BOTTOM_PADDING, elements.js:37-40).
pub const PAD: f64 = 5.0;
/// The corner radius of an activity (ACTIVITY_ROUND, elements.js:42).
pub const ACTIVITY_ROUND: f64 = 10.0;
/// Smallest sizes (elements.js:44-55).
pub const EVENT_MIN: f64 = 32.0;
pub const MESSAGE_MIN: (f64, f64) = (32.0, 24.0);
pub const GATEWAY_MIN: f64 = 32.0;
pub const DATAOBJECT_MIN: (f64, f64) = (32.0, 40.0);
pub const TEXTANNOTATION_MIN: f64 = 10.0;
pub const CONVERSATION_MIN: (f64, f64) = (38.0, 32.0);
/// A floating node's name label: 36 px from its centre, straight below (BPMNFloatingNodeView,
/// elements.js:900-912).
pub const LABEL_DISTANCE: f64 = 36.0;
pub const LABEL_ALPHA: f64 = -std::f64::consts::FRAC_PI_2;
/// The grey of an initiating participant's band and of a return message (elements.js:1405-1430,
/// bpmn-commands.js:166).
pub const INITIATING_FILL: &str = "#c6c6c6";
/// The text a new annotation from the quick edit starts with (bpmn-commands.js:196).
pub const ANNOTATION_TEXT: &str = "Text annotation";

/// The BPMN kinds, fields and defaults, in the order of bpmn/metamodel.json. Fields with no
/// default there (a gateway's direction) are always written, as the original does.
pub fn defs(t: &mut HashMap<&'static str, Kind>) {
    let mut def = |name: &'static str, sup: &'static str, attrs: Vec<(&'static str, Option<Value>)>| {
        t.insert(name, Kind { sup: Some(sup), attrs });
    };
    let n = || None;
    let d = |v: Value| Some(v);
    let f = || Some(json!(false));
    def("BPMNDiagram", "Diagram", vec![]);
    def("BPMNBaseElement", "ExtensibleModel", vec![("id", d(json!("")))]);
    def("BPMNRootElement", "BPMNBaseElement", vec![]);
    def("BPMNCollaboration", "BPMNRootElement", vec![]);
    def("BPMNChoreography", "BPMNCollaboration", vec![]);
    def("BPMNGlobalConversation", "BPMNCollaboration", vec![]);
    def("BPMNParticipant", "BPMNBaseElement", vec![("minimum", d(json!(0))), ("maximum", d(json!(1)))]);
    def("BPMNLane", "BPMNBaseElement", vec![]);
    def("BPMNProcess", "BPMNRootElement", vec![("processType", d(json!("none"))), ("isClosed", f()), ("isExecutable", f())]);
    def("BPMNFlowElement", "BPMNBaseElement", vec![]);
    def(
        "BPMNActivity",
        "BPMNFlowElement",
        vec![
            ("isForCompensation", f()), ("startQuantity", d(json!(0))), ("completionQuantity", d(json!(0))), ("isMultiInstance", f()), ("multiInstanceBehavior", d(json!("none"))),
            ("isLoop", f()), ("isSequential", f()), ("testBefore", f()), ("loopExpression", d(json!(""))),
        ],
    );
    def("BPMNCallActivity", "BPMNActivity", vec![("calledRef", n())]);
    def("BPMNTask", "BPMNActivity", vec![("isGlobal", f())]);
    for ty in ["BPMNSendTask", "BPMNServiceTask", "BPMNUserTask", "BPMNBusinessRuleTask"] {
        def(ty, "BPMNTask", vec![("implementation", d(json!("")))]);
    }
    def("BPMNReceiveTask", "BPMNTask", vec![("implementation", d(json!(""))), ("instantiate", f())]);
    def("BPMNManualTask", "BPMNTask", vec![]);
    def("BPMNScriptTask", "BPMNTask", vec![("scriptFormat", d(json!(""))), ("script", d(json!("")))]);
    def("BPMNSubProcess", "BPMNActivity", vec![("triggeredByEvent", f())]);
    def("BPMNAdHocSubProcess", "BPMNSubProcess", vec![]);
    def("BPMNTransaction", "BPMNSubProcess", vec![]);
    def("BPMNChoreographyActivity", "BPMNFlowElement", vec![("loopType", d(json!("none"))), ("initiatingParticipant", n()), ("upperParticipants", n()), ("lowerParticipants", n())]);
    for ty in ["BPMNChoreographyTask", "BPMNSubChoreography", "BPMNCallChoreography"] {
        def(ty, "BPMNChoreographyActivity", vec![]);
    }
    def("BPMNEventDefinition", "BPMNRootElement", vec![]);
    for (_, ty, _) in EVENT_DEFINITIONS {
        def(ty, "BPMNEventDefinition", vec![]);
    }
    def("BPMNEvent", "BPMNFlowElement", vec![("eventDefinitions", n())]);
    def("BPMNThrowEvent", "BPMNEvent", vec![]);
    def("BPMNCatchEvent", "BPMNEvent", vec![("parallelMultiple", f())]);
    for ty in ["BPMNImplicitThrowEvent", "BPMNIntermediateThrowEvent", "BPMNEndEvent"] {
        def(ty, "BPMNThrowEvent", vec![]);
    }
    def("BPMNStartEvent", "BPMNCatchEvent", vec![("isInterrupting", f())]);
    def("BPMNIntermediateCatchEvent", "BPMNCatchEvent", vec![]);
    def("BPMNBoundaryEvent", "BPMNCatchEvent", vec![("cancelActivity", f())]);
    def("BPMNGateway", "BPMNFlowElement", vec![("gatewayDirection", n())]);
    for ty in ["BPMNExclusiveGateway", "BPMNInclusiveGateway"] {
        def(ty, "BPMNGateway", vec![("default", n())]);
    }
    def("BPMNParallelGateway", "BPMNGateway", vec![]);
    def("BPMNComplexGateway", "BPMNGateway", vec![("default", n())]);
    def("BPMNEventBasedGateway", "BPMNGateway", vec![("instantiate", f()), ("eventGatewayType", d(json!("exclusive")))]);
    def("BPMNItemAwareElement", "BPMNBaseElement", vec![]);
    def("BPMNDataObject", "BPMNItemAwareElement", vec![("isCollection", f())]);
    for ty in ["BPMNDataStore", "BPMNDataInput", "BPMNDataOutput"] {
        def(ty, "BPMNItemAwareElement", vec![]);
    }
    def("BPMNMessage", "BPMNRootElement", vec![]);
    def("BPMNConversationNode", "BPMNBaseElement", vec![]);
    def("BPMNConversation", "BPMNConversationNode", vec![]);
    def("BPMNSubConversation", "BPMNConversationNode", vec![]);
    def("BPMNCallConversation", "BPMNConversationNode", vec![("calledRef", n())]);
    def("BPMNConversationLink", "DirectedRelationship", vec![]);
    def("BPMNArtifact", "BPMNBaseElement", vec![]);
    def("BPMNTextAnnotation", "BPMNArtifact", vec![("text", d(json!("")))]);
    def("BPMNGroup", "BPMNArtifact", vec![]);
    def("BPMNAssociation", "DirectedRelationship", vec![("associationDirection", d(json!("none")))]);
    def("BPMNDataAssociation", "DirectedRelationship", vec![]);
    def("BPMNSequenceFlow", "DirectedRelationship", vec![("isImmediate", f()), ("condition", d(json!("")))]);
    def("BPMNMessageFlow", "DirectedRelationship", vec![("messageRef", n())]);
    def("BPMNMessageLink", "DirectedRelationship", vec![]);
    // views
    def("BPMNGeneralNodeView", "NodeView", vec![("nameLabel", n()), ("wordWrap", d(json!(true)))]);
    def("BPMNFloatingNodeView", "NodeView", vec![("nameLabel", n()), ("wordWrap", d(json!(true)))]);
    def("BPMNGeneralEdgeView", "EdgeView", vec![("nameLabel", n())]);
    def("BPMNActivityView", "BPMNGeneralNodeView", vec![]);
    for ty in ["BPMNCallActivityView", "BPMNTaskView", "BPMNSubProcessView", "BPMNAdHocSubProcessView", "BPMNTransactionView"] {
        def(ty, "BPMNActivityView", vec![]);
    }
    def("BPMNChoreographyActivityView", "BPMNGeneralNodeView", vec![]);
    def("BPMNChoreographyTaskView", "BPMNChoreographyActivityView", vec![]);
    def("BPMNSubChoreographyView", "BPMNChoreographyActivityView", vec![]);
    for ty in ["BPMNEventView", "BPMNMessageView", "BPMNConversationView", "BPMNGatewayView", "BPMNDataObjectView"] {
        def(ty, "BPMNFloatingNodeView", vec![]);
    }
    def("BPMNConversationLinkView", "EdgeView", vec![]);
    def("BPMNDataInputView", "BPMNDataObjectView", vec![]);
    def("BPMNDataOutputView", "BPMNDataObjectView", vec![]);
    def("BPMNDataStoreView", "BPMNGeneralNodeView", vec![]);
    def("BPMNTextAnnotationView", "NodeView", vec![("wordWrap", d(json!(true))), ("horzAlign", d(json!(0))), ("vertAlign", d(json!(3)))]);
    def("BPMNGroupView", "NodeView", vec![]);
    for ty in ["BPMNAssociationView", "BPMNDataAssociationView", "BPMNMessageLinkView"] {
        def(ty, "EdgeView", vec![]);
    }
    def("BPMNSequenceFlowView", "BPMNGeneralEdgeView", vec![]);
    def("BPMNMessageFlowView", "BPMNGeneralEdgeView", vec![]);
    def("BPMNPoolView", "NodeView", vec![("nameLabel", n()), ("isVertical", f())]);
    def("BPMNLaneView", "NodeView", vec![("nameLabel", n()), ("isVertical", f())]);
}

/// The preference that gives a box its fill colour (bpmn.activity.fillColor … in the view
/// constructors, elements.js:970-2500); pools, lanes, annotations and groups take the general one.
pub fn fill_pref(view_ty: &str) -> Option<&'static str> {
    Some(match view_ty {
        _ if is_activity_view(view_ty) => "bpmn.activity.fillColor",
        "BPMNChoreographyTaskView" | "BPMNSubChoreographyView" => "bpmn.activity.fillColor",
        "BPMNEventView" => "bpmn.event.fillColor",
        "BPMNMessageView" => "bpmn.message.fillColor",
        "BPMNGatewayView" => "bpmn.gateway.fillColor",
        "BPMNDataObjectView" | "BPMNDataInputView" | "BPMNDataOutputView" | "BPMNDataStoreView" => "bpmn.dataobject.fillColor",
        "BPMNConversationView" => "bpmn.conversation.fillColor",
        _ => return None,
    })
}

/// The ten event definitions: menu and quick edit label, type, and short button text, in the quick
/// edit's order (quickedits/bpmn.json "bpmn.event"; menus/bpmn.json).
pub const EVENT_DEFINITIONS: [(&str, &str, &str); 10] = [
    ("Compensate Event Definition", "BPMNCompensateEventDefinition", "Compensate"),
    ("Cancel Event Definition", "BPMNCancelEventDefinition", "Cancel"),
    ("Error Event Definition", "BPMNErrorEventDefinition", "Error"),
    ("Link Event Definition", "BPMNLinkEventDefinition", "Link"),
    ("Signal Event Definition", "BPMNSignalEventDefinition", "Signal"),
    ("Timer Event Definition", "BPMNTimerEventDefinition", "Timer"),
    ("Escalation Event Definition", "BPMNEscalationEventDefinition", "Escalation"),
    ("Message Event Definition", "BPMNMessageEventDefinition", "Message"),
    ("Terminate Event Definition", "BPMNTerminateEventDefinition", "Terminate"),
    ("Conditional Event Definition", "BPMNConditionalEventDefinition", "Conditional"),
];
/// The quick edit's hints for the ten event definitions, in the same order.
const EVENT_DEFINITION_HINTS: [&str; 10] = [
    "Add Compensate Event Definition", "Add Cancel Event Definition", "Add Error Event Definition", "Add Link Event Definition", "Add Signal Event Definition",
    "Add Timer Event Definition", "Add Escalation Event Definition", "Add Message Event Definition", "Add Terminate Event Definition", "Add Conditional Event Definition",
];

/// The view type of a model type (the `view` of each kind in metamodel.json).
pub fn view_of(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "BPMNParticipant" => "BPMNPoolView",
        "BPMNLane" => "BPMNLaneView",
        "BPMNCallActivity" => "BPMNCallActivityView",
        "BPMNTask" | "BPMNSendTask" | "BPMNReceiveTask" | "BPMNServiceTask" | "BPMNUserTask" | "BPMNManualTask" | "BPMNBusinessRuleTask" | "BPMNScriptTask" => "BPMNTaskView",
        "BPMNSubProcess" => "BPMNSubProcessView",
        "BPMNAdHocSubProcess" => "BPMNAdHocSubProcessView",
        "BPMNTransaction" => "BPMNTransactionView",
        "BPMNChoreographyTask" => "BPMNChoreographyTaskView",
        "BPMNSubChoreography" => "BPMNSubChoreographyView",
        "BPMNEvent" | "BPMNThrowEvent" | "BPMNCatchEvent" | "BPMNImplicitThrowEvent" | "BPMNIntermediateThrowEvent" | "BPMNEndEvent" | "BPMNStartEvent" | "BPMNIntermediateCatchEvent" | "BPMNBoundaryEvent" => "BPMNEventView",
        "BPMNGateway" | "BPMNExclusiveGateway" | "BPMNInclusiveGateway" | "BPMNParallelGateway" | "BPMNComplexGateway" | "BPMNEventBasedGateway" => "BPMNGatewayView",
        "BPMNDataObject" => "BPMNDataObjectView",
        "BPMNDataStore" => "BPMNDataStoreView",
        "BPMNDataInput" => "BPMNDataInputView",
        "BPMNDataOutput" => "BPMNDataOutputView",
        "BPMNMessage" => "BPMNMessageView",
        "BPMNConversation" | "BPMNSubConversation" | "BPMNCallConversation" => "BPMNConversationView",
        "BPMNTextAnnotation" => "BPMNTextAnnotationView",
        "BPMNGroup" => "BPMNGroupView",
        _ => return None,
    })
}

/// The line view of a relationship type.
pub fn edge_view_of(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "BPMNSequenceFlow" => "BPMNSequenceFlowView",
        "BPMNMessageFlow" => "BPMNMessageFlowView",
        "BPMNAssociation" => "BPMNAssociationView",
        "BPMNDataAssociation" => "BPMNDataAssociationView",
        "BPMNMessageLink" => "BPMNMessageLinkView",
        "BPMNConversationLink" => "BPMNConversationLinkView",
        _ => return None,
    })
}

/// Whether a model type has a box on a BPMN diagram.
pub fn is_node_type(ty: &str) -> bool {
    view_of(ty).is_some()
}

/// Whether a view type is a BPMN box.
pub fn is_node_view(ty: &str) -> bool {
    matches!(
        ty,
        "BPMNPoolView" | "BPMNLaneView" | "BPMNCallActivityView" | "BPMNTaskView" | "BPMNSubProcessView" | "BPMNAdHocSubProcessView" | "BPMNTransactionView" | "BPMNChoreographyTaskView"
            | "BPMNSubChoreographyView" | "BPMNEventView" | "BPMNGatewayView" | "BPMNDataObjectView" | "BPMNDataStoreView" | "BPMNDataInputView" | "BPMNDataOutputView" | "BPMNMessageView"
            | "BPMNConversationView" | "BPMNTextAnnotationView" | "BPMNGroupView"
    )
}

/// Whether a view type is a BPMN line.
pub fn is_edge_view(ty: &str) -> bool {
    matches!(ty, "BPMNSequenceFlowView" | "BPMNMessageFlowView" | "BPMNAssociationView" | "BPMNDataAssociationView" | "BPMNMessageLinkView" | "BPMNConversationLinkView")
}

/// Whether a view is a box with a floating name below it (BPMNFloatingNodeView and its kinds).
pub fn is_floating(view_ty: &str) -> bool {
    matches!(view_ty, "BPMNEventView" | "BPMNMessageView" | "BPMNConversationView" | "BPMNGatewayView" | "BPMNDataObjectView" | "BPMNDataInputView" | "BPMNDataOutputView")
}

/// Whether a view is an activity's box (BPMNActivityView and its kinds).
pub fn is_activity_view(view_ty: &str) -> bool {
    matches!(view_ty, "BPMNCallActivityView" | "BPMNTaskView" | "BPMNSubProcessView" | "BPMNAdHocSubProcessView" | "BPMNTransactionView")
}

/// What a node tool makes: the model type, and whether its pool or lane is vertical
/// (toolbox/bpmn.json: "BPMNParticipantVert" and "BPMNLaneVert" make the same kinds with
/// `view-init` isVertical).
pub fn tool(kind: &str) -> Option<(&'static str, bool)> {
    match kind {
        "BPMNParticipantVert" => Some(("BPMNParticipant", true)),
        "BPMNLaneVert" => Some(("BPMNLane", true)),
        _ if is_node_type(kind) && kind != "BPMNEvent" && kind != "BPMNGateway" => Some((crate::meta::static_name(kind), false)),
        _ => None,
    }
}

/// The size a plain click gives: the smallest one, as the original does, but pools, lanes and
/// groups take a roomier box (estimated; the original's would be just their name).
pub fn click_size(kind: &str) -> Option<(f64, f64)> {
    match kind {
        "BPMNParticipant" | "BPMNLane" => Some((600.0, 150.0)),
        "BPMNParticipantVert" | "BPMNLaneVert" => Some((150.0, 600.0)),
        "BPMNGroup" => Some((200.0, 150.0)),
        _ if tool(kind).is_some() => Some((1.0, 1.0)),
        _ => None,
    }
}

/// Whether a tool's element goes in the lane or pool it is drawn in: the toolbox items with
/// `container-views` [BPMNLaneView, BPMNPoolView], and a lane in a pool (toolbox/bpmn.json;
/// factory.js:241-248).
fn tool_containers(model_ty: &str) -> &'static [&'static str] {
    match model_ty {
        "BPMNLane" => &["BPMNPoolView"],
        "BPMNParticipant" | "BPMNChoreographyTask" | "BPMNSubChoreography" | "BPMNConversation" | "BPMNSubConversation" | "BPMNCallConversation" | "BPMNTextAnnotation" | "BPMNGroup" | "BPMNBoundaryEvent" => &[],
        _ => &["BPMNLaneView", "BPMNPoolView"],
    }
}

/// Whether a box can hold another when it is moved in (canContainViewKind of BPMNPoolView and
/// BPMNLaneView, elements.js:2898-2910, 3007-3016): a pool holds lanes, sub-processes, tasks,
/// events, gateways, data objects and data stores; a lane the same but lanes.
pub fn can_contain(container_ty: &str, view_ty: &str) -> bool {
    let flow = matches!(view_ty, "BPMNSubProcessView" | "BPMNTaskView" | "BPMNEventView" | "BPMNGatewayView" | "BPMNDataObjectView" | "BPMNDataInputView" | "BPMNDataOutputView" | "BPMNDataStoreView");
    match container_ty {
        "BPMNPoolView" => flow || view_ty == "BPMNLaneView",
        "BPMNLaneView" => flow,
        _ => false,
    }
}

/// A new element's name: its class name and a number, unique in the field it goes in
/// (BPMNBaseElement.getDisplayClassName, elements.js:93-95; Element.getNewName).
pub fn new_name(r: &Repo, parent: Eid, field: &str, base: &str) -> String {
    let names: HashSet<String> = r.children(parent).into_iter().filter(|(k, _)| k == field).map(|(_, c)| r.name(c)).collect();
    (1..).map(|i| format!("{base}{i}")).find(|n| !names.contains(n)).unwrap()
}

fn num(x: f64) -> Value {
    json!(x)
}

/// A box's view with its name label: inside for general nodes (centred, wrapped), floating 36 px
/// below the centre for events, gateways, messages, conversations and data objects, at the top or
/// left of a pool or lane (the view constructors, elements.js:836-930, 2775-2935).
pub fn create_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R, vertical: bool) -> Option<Eid> {
    let ty = view_of(r.ty(model))?;
    let m = r.ref_to(model);
    let mut props = vec![("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false))];
    // the views that may change container (containerChangeable in each constructor); a boundary
    // event stays on its activity (BPMNEventView.update)
    if !matches!(ty, "BPMNTextAnnotationView" | "BPMNGroupView") && r.ty(model) != "BPMNBoundaryEvent" {
        props.push(("containerChangeable", json!(true)));
    }
    props.extend([("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))]);
    let v = r.create(ty, &props, Some((diagram, "ownedViews")));
    match ty {
        "BPMNTextAnnotationView" | "BPMNGroupView" => {}
        "BPMNPoolView" | "BPMNLaneView" => {
            if vertical {
                r.set(v, "isVertical", json!(true));
            }
            let l = r.create("LabelView", &[("font", json!(views::FONT)), ("parentStyle", json!(true)), ("verticalAlignment", json!(3))], Some((v, "subViews")));
            r.set_ref(v, "nameLabel", Some(l));
        }
        _ if is_floating(ty) => {
            let l = r.create("NodeLabelView", &[("model", m), ("font", json!(views::FONT)), ("alpha", json!(LABEL_ALPHA)), ("distance", json!(LABEL_DISTANCE)), ("height", json!(views::LABEL_H))], Some((v, "subViews")));
            r.set_ref(v, "nameLabel", Some(l));
        }
        _ => {
            let l = r.create("LabelView", &[("font", json!(views::FONT)), ("parentStyle", json!(true)), ("wordWrap", json!(true))], Some((v, "subViews")));
            r.set_ref(v, "nameLabel", Some(l));
        }
    }
    Some(v)
}

/// A line's name label, 15 px off its middle, on sequence and message flows (BPMNGeneralEdgeView,
/// elements.js:935-968); the other BPMN lines have no label.
pub fn edge_labels(r: &mut Repo, e: Eid, m: Eid) {
    if matches!(r.ty(e), "BPMNSequenceFlowView" | "BPMNMessageFlowView") {
        let l = views::edge_label(r, e, Some(m), std::f64::consts::FRAC_PI_2, 15.0, 1, false);
        r.set_ref(e, "nameLabel", Some(l));
    }
}

/// The texts of a line's labels.
pub fn edge_label_texts(r: &Repo, m: Eid) -> Vec<(&'static str, String)> {
    if matches!(r.ty(m), "BPMNSequenceFlow" | "BPMNMessageFlow") {
        vec![("nameLabel", r.name(m))]
    } else {
        vec![]
    }
}

/// Whether a BPMN line may join two views: sequence and message flows join two BPMN elements
/// (linkPrecondition, bpmn-factory.js:20-25); an association may also end on a line (a text
/// annotation on a flow); the other lines join any two boxes. None for other kinds.
pub fn can_link(r: &Repo, kind: &str, tail: Eid, head: Eid) -> Option<bool> {
    if !kind.starts_with("BPMN") {
        return None;
    }
    let (Some(tm), Some(hm)) = (r.refd(tail, "model"), r.refd(head, "model")) else { return Some(false) };
    let edge = views::is_edge(r, tail) || views::is_edge(r, head);
    Some(match kind {
        "BPMNSequenceFlow" | "BPMNMessageFlow" => !edge && r.kind_of(tm, "BPMNBaseElement") && r.kind_of(hm, "BPMNBaseElement"),
        "BPMNAssociation" => !views::is_edge(r, tail) || !views::is_edge(r, head),
        _ => !edge,
    })
}

/// The editable fields of a BPMN element, in metamodel order (the `visible` attributes of
/// metamodel.json).
pub fn fields(r: &Repo, e: Eid) -> Vec<(&'static str, &'static str)> {
    let t = r.ty(e);
    if r.kind_of(e, "DirectedRelationship") {
        let mut f = vec![("name", "text"), ("source", "ref"), ("target", "ref")];
        match t {
            "BPMNAssociation" => f.push(("associationDirection", "enum")),
            "BPMNSequenceFlow" => f.extend([("isImmediate", "bool"), ("condition", "text")]),
            "BPMNMessageFlow" => f.push(("messageRef", "ref")),
            _ => {}
        }
        return f;
    }
    let mut f = vec![("name", "text"), ("id", "text")];
    let k = |b: &str| r.kind_of(e, b);
    if k("BPMNProcess") {
        f.extend([("processType", "enum"), ("isClosed", "bool"), ("isExecutable", "bool")]);
    }
    if k("BPMNParticipant") {
        f.extend([("minimum", "int"), ("maximum", "int")]);
    }
    if k("BPMNActivity") {
        f.extend([
            ("isForCompensation", "bool"), ("startQuantity", "int"), ("completionQuantity", "int"), ("isMultiInstance", "bool"), ("multiInstanceBehavior", "enum"), ("isLoop", "bool"),
            ("isSequential", "bool"), ("testBefore", "bool"), ("loopExpression", "text"),
        ]);
    }
    if k("BPMNCallActivity") || k("BPMNCallConversation") {
        f.push(("calledRef", "ref"));
    }
    if k("BPMNTask") {
        f.push(("isGlobal", "bool"));
    }
    if matches!(t, "BPMNSendTask" | "BPMNReceiveTask" | "BPMNServiceTask" | "BPMNUserTask" | "BPMNBusinessRuleTask") {
        f.push(("implementation", "text"));
    }
    if matches!(t, "BPMNReceiveTask" | "BPMNEventBasedGateway") {
        f.push(("instantiate", "bool"));
    }
    if t == "BPMNScriptTask" {
        f.extend([("scriptFormat", "text"), ("script", "multiline")]);
    }
    if k("BPMNSubProcess") {
        f.push(("triggeredByEvent", "bool"));
    }
    if k("BPMNChoreographyActivity") {
        f.extend([("loopType", "enum"), ("initiatingParticipant", "ref"), ("upperParticipants", "reflist"), ("lowerParticipants", "reflist")]);
    }
    if k("BPMNCatchEvent") {
        f.push(("parallelMultiple", "bool"));
    }
    if t == "BPMNStartEvent" {
        f.push(("isInterrupting", "bool"));
    }
    if t == "BPMNBoundaryEvent" {
        f.push(("cancelActivity", "bool"));
    }
    if k("BPMNGateway") {
        f.push(("gatewayDirection", "enum"));
    }
    if matches!(t, "BPMNExclusiveGateway" | "BPMNInclusiveGateway" | "BPMNComplexGateway") {
        f.push(("default", "ref"));
    }
    if t == "BPMNEventBasedGateway" {
        f.push(("eventGatewayType", "enum"));
    }
    if t == "BPMNDataObject" {
        f.push(("isCollection", "bool"));
    }
    if t == "BPMNTextAnnotation" {
        f.push(("text", "multiline"));
    }
    f
}

/// The values of a BPMN enumeration field (the enums of metamodel.json).
pub fn enum_options(field: &str) -> Option<&'static [&'static str]> {
    Some(match field {
        "processType" => &["none", "public", "private"],
        "gatewayDirection" => &["unspecified", "converging", "diverging", "mixed"],
        "eventGatewayType" => &["parallel", "exclusive"],
        "associationDirection" => &["none", "one", "both"],
        "multiInstanceBehavior" => &["none", "one", "all", "complex"],
        "loopType" => &["none", "standard", "multiInstanceSequential", "multiInstanceParallel"],
        _ => return None,
    })
}

/// Model › Add under an element (menus/bpmn.json, with the visible states of updateMenus,
/// bpmn-commands.js:216-263): a business process under any element that is not a BPMN one;
/// collaborations, choreographies, global conversations, tasks, sub-processes and gateways under BPMN
/// elements; events under a process or sub-process; event definitions under an event.
pub fn add_kinds(r: &Repo, e: Eid) -> Vec<(&'static str, &'static str, bool)> {
    let mut v = vec![];
    let bpmn = r.kind_of(e, "BPMNBaseElement");
    if (r.ty(e) == "Project" || r.kind_of(e, "ExtensibleModel")) && !bpmn && !r.kind_of(e, "Diagram") {
        v.push(("Business Process", "BPMNProcess", false));
    }
    if bpmn {
        v.extend([
            ("Collaboration", "BPMNCollaboration", false), ("Choreography", "BPMNChoreography", false), ("Global Conversation", "BPMNGlobalConversation", false),
            ("Task", "BPMNTask", false), ("Sub Process", "BPMNSubProcess", false), ("Gateway", "BPMNGateway", false),
        ]);
    }
    if r.kind_of(e, "BPMNProcess") || r.kind_of(e, "BPMNSubProcess") {
        v.extend([
            ("Start Event", "BPMNStartEvent", false), ("Intermediate Event (Throw)", "BPMNIntermediateThrowEvent", false), ("Intermediate Event (Catch)", "BPMNIntermediateCatchEvent", false),
            ("End Event", "BPMNEndEvent", false),
        ]);
    }
    if r.kind_of(e, "BPMNEvent") {
        v.extend(EVENT_DEFINITIONS.iter().map(|(l, t, _)| (*l, *t, false)));
    }
    v
}

/// Make a model element as the factory does: named after its class in the field it goes in, a
/// gateway's direction set, an event definition in the event's definitions (bpmn-factory.js:78-96,
/// factory.js defaultModelFn; elements.js:571-580).
pub fn create_model(r: &mut Repo, parent: Eid, ty: &str) -> Eid {
    let field = if ty.ends_with("EventDefinition") { "eventDefinitions" } else { "ownedElements" };
    let name = new_name(r, parent, field, ty);
    let m = r.create(ty, &[("name", json!(name))], Some((parent, field)));
    if r.kind_of(m, "BPMNGateway") {
        r.set(m, "gatewayDirection", json!("unspecified"));
    }
    m
}

/// Put pools first, then lanes, then groups, behind the other views, keeping each one's order: what
/// they hold stays above them and is the one a click finds.
pub fn restack(r: &mut Repo, d: Eid) {
    let rank = |r: &Repo, v: Eid| match r.ty(v) {
        "BPMNPoolView" => 0,
        "BPMNLaneView" => 1,
        "BPMNGroupView" => 2,
        _ => 3,
    };
    let list = r.list(d, "ownedViews");
    let mut sorted = list.clone();
    sorted.sort_by_key(|v| rank(r, *v));
    if sorted != list {
        r.set_list(d, "ownedViews", sorted);
    }
}

/// The quick edit buttons right of a name (quickedits/bpmn.json).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Quick {
    /// A pool's Add Lane
    AddLane,
    /// An activity's Add Boundary Event
    AddBoundaryEvent,
    /// An event's Add … Event Definition
    AddEventDefinition(&'static str),
    /// A choreography's Create Participant (Upper when true, Lower otherwise)
    CreateParticipant(bool),
    /// A choreography's Assign Participant (Upper / Lower)
    AssignParticipant(bool),
    /// A choreography's Initiating Participant
    InitiatingParticipant,
    /// A choreography's Create Initiating Message (true) or Create Return Message (false)
    Message(bool),
}

/// The buttons of a BPMN element's name edit: (short text, hint, key, action).
pub fn buttons(r: &Repo, m: Eid) -> Vec<(&'static str, &'static str, &'static str, Quick)> {
    if r.ty(m) == "BPMNParticipant" {
        return vec![("+ Lane", "Add Lane", "", Quick::AddLane)];
    }
    if r.kind_of(m, "BPMNActivity") {
        return vec![("+ Boundary", "Add Boundary Event", "", Quick::AddBoundaryEvent)];
    }
    if r.kind_of(m, "BPMNEvent") {
        return EVENT_DEFINITIONS.iter().zip(EVENT_DEFINITION_HINTS).map(|((_, t, s), h)| (*s, h, "", Quick::AddEventDefinition(t))).collect();
    }
    if r.kind_of(m, "BPMNChoreographyActivity") {
        return vec![
            ("Upper…", "Assign Participant (Upper)", "", Quick::AssignParticipant(true)),
            ("+ Upper", "Create Participant (Upper)", "", Quick::CreateParticipant(true)),
            ("+ Init Msg", "Create Initiating Message", "", Quick::Message(true)),
            ("Lower…", "Assign Participant (Lower)", "", Quick::AssignParticipant(false)),
            ("+ Lower", "Create Participant (Lower)", "", Quick::CreateParticipant(false)),
            ("+ Return Msg", "Create Return Message", "", Quick::Message(false)),
            ("Initiating…", "Initiating Participant", "", Quick::InitiatingParticipant),
        ];
    }
    vec![]
}

/// The left-hand "Add Text Annotation" of every BPMN quick edit but a text annotation's
/// (quickedits/bpmn.json): an annotation below, joined to the element by an association from it
/// (bpmn-commands.js:178-214).
pub fn related(model_ty: &str) -> Vec<crate::related::Related> {
    if matches!(model_ty, "BPMNTextAnnotation" | "BPMNGroup") || model_ty.ends_with("EventDefinition") {
        return vec![];
    }
    vec![ANNOTATION]
}

pub const ANNOTATION: crate::related::Related =
    crate::related::Related { label: "Add Text Annotation", kind: "BPMNTextAnnotation", rel: "BPMNAssociation", init: crate::app::AssocInit::Plain, from_new: true, place: crate::related::Place::Below };

/// A deterministic stand-in for Math.random, for the boundary event's position.
fn random_unit(seed: u64) -> f64 {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0) ^ seed.wrapping_mul(0x9e3779b97f4a7c15);
    let mut x = t.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

impl App {
    /// The topmost visible box under a point.
    fn bpmn_view_at(&self, d: Eid, p: P, ok: impl Fn(&Repo, Eid) -> bool) -> Option<Eid> {
        self.repo.list(d, "ownedViews").into_iter().rev().find(|c| {
            self.repo.alive(*c) && views::is_node(&self.repo, *c) && !matches!(self.repo.raw(*c, "visible"), Some(Value::Bool(false))) && ok(&self.repo, *c) && {
                let b = views::node_rect(&self.repo, *c);
                p.x >= b.x && p.x <= b.x + b.w && p.y >= b.y && p.y <= b.y + b.h
            }
        })
    }

    /// The BPMN node tools (bpmn-factory.js:98-145, factory.js defaultModelAndViewFn): the element
    /// goes in the diagram's owner, or in the lane or pool it is drawn in (`container-views`); a
    /// boundary event goes on the box under the point, whose element owns it (`parasitic`). None when
    /// the tool is not a BPMN one; Some(None) when it cannot be made here.
    pub fn create_bpmn_node(&mut self, d: Eid, kind: &str, rect: R) -> Option<Option<Eid>> {
        let (ty, vertical) = tool(kind)?;
        let owner = self.repo.parent(d)?;
        let at = P { x: rect.x, y: rect.y };
        let containers = tool_containers(ty);
        let holder = if ty == "BPMNBoundaryEvent" {
            match self.bpmn_view_at(d, at, |r, c| r.refd(c, "model").is_some()) {
                Some(h) => Some(h),
                None => return Some(None),
            }
        } else {
            self.bpmn_view_at(d, at, |r, c| containers.contains(&r.ty(c)))
        };
        let parent = holder.and_then(|h| self.repo.refd(h, "model")).unwrap_or(owner);
        let m = create_model(&mut self.repo, parent, ty);
        let v = create_view(&mut self.repo, d, m, rect, vertical)?;
        self.apply_view_defaults(v);
        if let Some(h) = holder {
            self.repo.set_ref(v, "containerView", Some(h));
            let mut list = match self.repo.raw(h, "containedViews") {
                Some(Value::Array(a)) => a.clone(),
                _ => vec![],
            };
            list.push(self.repo.ref_to(v));
            self.repo.set_raw(h, "containedViews", json!(list));
        }
        restack(&mut self.repo, d);
        Some(Some(v))
    }

    /// Model › Add of a BPMN kind (bpmn-factory.js:78-96).
    pub fn add_bpmn_model(&mut self, parent: Eid, ty: &str) -> Eid {
        create_model(&mut self.repo, parent, ty)
    }

    /// Model › Add Diagram › BPMN Diagram: in the BPMN root element or activity it is added to,
    /// elsewhere in a new "BPMNProcess1" (diagramFn, bpmn-factory.js:28-57).
    pub fn add_bpmn_diagram(&mut self, parent: Option<Eid>) -> Eid {
        let p = parent.unwrap_or(self.repo.root);
        let target = if self.repo.kind_of(p, "BPMNRootElement") || self.repo.kind_of(p, "BPMNActivity") {
            p
        } else {
            let name = new_name(&self.repo, p, "ownedElements", "BPMNProcess");
            self.repo.create("BPMNProcess", &[("name", json!(name))], Some((p, "ownedElements")))
        };
        let name = new_name(&self.repo, target, "ownedElements", "BPMNDiagram");
        self.repo.create("BPMNDiagram", &[("name", json!(name))], Some((target, "ownedElements")))
    }

    /// The inline edit of a BPMN box (quickedits/bpmn.json): a text annotation's text in a text area,
    /// the name over a pool's or lane's name, over the name label otherwise. Some(None): no edit (a
    /// group); None: not a BPMN box.
    pub fn bpmn_edit(&self, v: Eid) -> Option<Option<Editing>> {
        let t = self.repo.ty(v);
        if !is_node_view(t) {
            return None;
        }
        let Some(m) = self.repo.refd(v, "model") else { return Some(None) };
        let b = views::node_rect(&self.repo, v);
        if t == "BPMNGroupView" {
            return Some(None);
        }
        if t == "BPMNTextAnnotationView" {
            let text = self.repo.str(m, "text");
            return Some(Some(Editing { kind: EditKind::Name(m), text, rect: b, multiline: true, center: false, fresh: true, host: Some(v), orig: None }));
        }
        let l = self.repo.refd(v, "nameLabel");
        let rect = match (t, l) {
            // the area from the left at the name's top, 150 wide (quickedits/bpmn.json "bpmn.pool")
            ("BPMNPoolView" | "BPMNLaneView", Some(l)) => R { x: b.x, y: self.repo.f(l, "top"), w: 150.0, h: 19.0 },
            (_, Some(l)) if self.repo.f(l, "width") > 0.0 => R { x: self.repo.f(l, "left") - 2.0, y: self.repo.f(l, "top") - 3.0, w: self.repo.f(l, "width").max(80.0) + 4.0, h: 19.0 },
            _ => R { x: b.x + b.w / 2.0 - 50.0, y: b.y + b.h + 10.0, w: 100.0, h: 19.0 },
        };
        let text = self.repo.name(m);
        Some(Some(Editing { kind: EditKind::Name(m), text, rect, multiline: false, center: true, fresh: true, host: Some(v), orig: None }))
    }

    /// Run a quick edit button on the box `v`.
    pub fn bpmn_quick(&mut self, v: Eid, q: Quick) {
        let Some(m) = self.repo.refd(v, "model").filter(|m| self.repo.alive(*m)) else { return };
        match q {
            Quick::AddLane => {
                if let Some(l) = self.change(|a| a.add_lane(v)) {
                    self.select(vec![l]);
                }
            }
            Quick::AddBoundaryEvent => {
                if let Some(e) = self.change(|a| a.add_boundary_event(v)) {
                    self.select(vec![e]);
                }
            }
            Quick::AddEventDefinition(ty) => {
                let e = self.change(|a| a.add_bpmn_model(m, ty));
                self.reveal(e);
            }
            Quick::CreateParticipant(upper) => {
                let p = self.change(|a| a.create_choreography_participant(m, upper));
                if let Some(p) = p {
                    self.reveal(p);
                }
            }
            Quick::AssignParticipant(upper) => self.open_typed_picker(m, if upper { "upperParticipants" } else { "lowerParticipants" }, "BPMNParticipant"),
            Quick::InitiatingParticipant => self.open_typed_picker(m, "initiatingParticipant", "BPMNParticipant"),
            Quick::Message(initiating) => {
                if let Some(n) = self.change(|a| a.add_choreography_message(v, initiating)) {
                    self.select(vec![n]);
                    if let Some(nm) = self.repo.refd(n, "model") {
                        self.reveal(nm);
                    }
                    self.pending_edit = Some(n);
                }
            }
        }
    }

    /// Add Lane: a new lane in the pool, at its bottom left; the pool then lines it up under the
    /// others (handleAddLane, bpmn-commands.js:42-58).
    pub fn add_lane(&mut self, pool: Eid) -> Option<Eid> {
        let d = self.repo.parent(pool)?;
        let pm = self.repo.refd(pool, "model")?;
        let (x, y) = (self.repo.f(pool, "left"), self.repo.f(pool, "top") + self.repo.f(pool, "height"));
        let m = create_model(&mut self.repo, pm, "BPMNLane");
        let v = create_view(&mut self.repo, d, m, R { x, y, w: 1.0, h: 1.0 }, false)?;
        self.apply_view_defaults(v);
        self.contain_in(pool, v);
        restack(&mut self.repo, d);
        Some(v)
    }

    /// Add Boundary Event: a boundary event of the activity at a random point of its box, which its
    /// layout then moves onto the border (handleAddBoundaryEvent, bpmn-commands.js:16-40).
    pub fn add_boundary_event(&mut self, act: Eid) -> Option<Eid> {
        let d = self.repo.parent(act)?;
        let am = self.repo.refd(act, "model")?;
        let b = views::node_rect(&self.repo, act);
        let x = b.x + (random_unit(1) * b.w).round();
        let y = b.y + (random_unit(2) * b.h).round();
        let m = create_model(&mut self.repo, am, "BPMNBoundaryEvent");
        let v = create_view(&mut self.repo, d, m, R { x, y, w: 1.0, h: 1.0 }, false)?;
        self.apply_view_defaults(v);
        self.contain_in(act, v);
        Some(v)
    }

    fn contain_in(&mut self, host: Eid, v: Eid) {
        self.repo.set_ref(v, "containerView", Some(host));
        let mut list = match self.repo.raw(host, "containedViews") {
            Some(Value::Array(a)) => a.clone(),
            _ => vec![],
        };
        list.push(self.repo.ref_to(v));
        self.repo.set_raw(host, "containedViews", json!(list));
    }

    /// Create Participant (Upper / Lower): a participant "Participant" next to the choreography, in
    /// its upper or lower participants (handleCreateChoreographyParticipant, bpmn-commands.js:72-88).
    pub fn create_choreography_participant(&mut self, m: Eid, upper: bool) -> Option<Eid> {
        let parent = self.repo.parent(m)?;
        let p = self.repo.create("BPMNParticipant", &[("name", json!("Participant"))], Some((parent, "ownedElements")));
        let field = if upper { "upperParticipants" } else { "lowerParticipants" };
        let mut list = self.repo.ref_list(m, field);
        list.push(p);
        self.repo.set_ref_list(m, field, &list);
        Some(p)
    }

    /// Create Initiating Message (above) or Return Message (below, grey), in the diagram's owner,
    /// joined to the choreography by a rectilinear message link from the message
    /// (handleAddChoreographyInitiatingMessage and …ReturnMessage, bpmn-commands.js:117-176).
    pub fn add_choreography_message(&mut self, v: Eid, initiating: bool) -> Option<Eid> {
        let d = self.repo.parent(v)?;
        let owner = self.repo.parent(d)?;
        let b = views::node_rect(&self.repo, v);
        // above or below the box, 10 px past the messages already linked on that side (estimated,
        // as the related-element buttons place theirs)
        let linked: Vec<R> = self
            .repo
            .list(d, "ownedViews")
            .into_iter()
            .filter(|e| self.repo.ty(*e) == "BPMNMessageLinkView" && self.repo.refd(*e, "head") == Some(v))
            .filter_map(|e| self.repo.refd(e, "tail"))
            .map(|t| views::node_rect(&self.repo, t))
            .filter(|o| if initiating { o.y + o.h <= b.y } else { o.y >= b.y + b.h })
            .collect();
        let (w, h) = MESSAGE_MIN;
        let mut x = (b.x + (b.w - w) / 2.0).round();
        let mut y = if initiating { b.y - 100.0 - h } else { b.y + b.h + 70.0 };
        if let Some(o) = linked.iter().max_by(|p, q| (p.x + p.w).total_cmp(&(q.x + q.w))) {
            x = o.x + o.w + 10.0;
            y = o.y;
        }
        let m = create_model(&mut self.repo, owner, "BPMNMessage");
        let n = create_view(&mut self.repo, d, m, R { x: x.max(0.0), y: y.max(0.0), w, h }, false)?;
        self.apply_view_defaults(n);
        if !initiating {
            self.repo.set(n, "fillColor", json!(INITIATING_FILL));
        }
        let tool = crate::app::Tool { label: "", kind: "BPMNMessageLink", icon: "", line: true, init: crate::app::AssocInit::Plain };
        let e = self.create_edge(&tool, n, v)?;
        self.repo.set(e, "lineStyle", json!(0));
        Some(n)
    }

    /// Add Text Annotation on a line: the annotation below the line's middle, joined to it by an
    /// association (handleAddTextAnnotation, bpmn-commands.js:178-214).
    pub fn add_annotation_to_edge(&mut self, e: Eid) -> Option<Eid> {
        let pts = views::points_of(&self.repo, e);
        let (x1, x2) = pts.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.x), b.max(p.x)));
        let y2 = pts.iter().fold(f64::MIN, |a, p| a.max(p.y));
        let rect = R { x: ((x1 + x2) / 2.0).round().max(0.0), y: (y2 + 70.0).round(), w: 1.0, h: 1.0 };
        let n = self.change(|a| {
            let n = a.create_node("BPMNTextAnnotation", rect)?;
            let m = a.repo.refd(n, "model")?;
            a.repo.set(m, "text", json!(ANNOTATION_TEXT));
            let tool = crate::app::Tool { label: "", kind: "BPMNAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
            a.create_edge(&tool, n, e);
            Some(n)
        })?;
        self.select(vec![n]);
        self.pending_edit = Some(n);
        Some(n)
    }
}
