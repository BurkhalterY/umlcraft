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
/// A stereotype decoration's box, and the smallest icon of the iconic form (elements.js:70-74).
pub const DECORATION_ICON: f64 = 24.0;
const ICONIC_MIN: f64 = 24.0;
pub const ROBUST_ICON_W: f64 = 50.0;
pub const ROBUST_ICON_H: f64 = 36.0;
/// An artifact shown as an icon (estimated).
pub const ARTIFACT_ICON_W: f64 = 30.0;
pub const ARTIFACT_ICON_H: f64 = 40.0;

/// The robustness kind of a class shown as an icon: "boundary", "entity" or "control".
/// A stereotype icon's script, if the stereotype of the view's element has one (`!icon w h` in its
/// icon's content), with the icon's minimum size (elements.js:270-275, 4693-4700).
pub fn stereotype_icon(r: &Repo, m: Eid) -> Option<(StereotypeIcon, f64, f64)> {
    let st = r.refd(m, "stereotype")?;
    let icon = r.child(st, "icon")?;
    let content = r.str(icon, "content");
    let (w, h) = (r.f(icon, "width"), r.f(icon, "height"));
    if content.trim_start().starts_with("!icon") {
        return Some((StereotypeIcon::Script(content), w, h));
    }
    // a picture: JSON {width, height, data} in `image` (property-editor-view.js:385-398)
    let img: Value = serde_json::from_str(&r.str(icon, "image")).ok()?;
    let data = img.get("data")?.as_str()?.to_string();
    let (iw, ih) = (img.get("width")?.as_f64()?, img.get("height")?.as_f64()?);
    Some((StereotypeIcon::Picture(data, iw, ih), w, h))
}

/// How a stereotype's icon is given: an icon script, or a picture with its size.
#[derive(Clone, Debug, PartialEq)]
pub enum StereotypeIcon {
    Script(String),
    Picture(String, f64, f64),
}

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
pub const FRAME_MIN_W: f64 = 80.0; // elements.js:218
pub const FRAME_MIN_H: f64 = 30.0; // elements.js:219
const FRAME_CONTENT_MIN_W: f64 = 10.0; // elements.js:220
const FRAME_CONTENT_MIN_H: f64 = 30.0; // elements.js:221
const LABEL_INTERVAL: f64 = 3.0; // elements.js:62
pub const PORT_SIZE: f64 = 14.0; // elements.js:89-90
pub const PIN_SIZE: f64 = 18.0; // elements.js:159-160
pub const NARY_MIN_W: f64 = 30.0; // elements.js:82
pub const NARY_MIN_H: f64 = 22.0; // elements.js:83
pub const HYPERLINK_MIN_W: f64 = 100.0; // elements.js:231
pub const HYPERLINK_MIN_H: f64 = 20.0; // elements.js:232
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
        // a subsystem is drawn like a package (estimated)
        "UMLSubsystem" => k("UMLSubsystemView", false, false, false),
        // component and deployment diagrams (uml.json "uml.component", "uml.deployment")
        "UMLComponent" => k("UMLComponentView", true, true, false),
        "UMLArtifact" => k("UMLArtifactView", true, true, false),
        "UMLNode" => k("UMLNodeView", true, true, false),
        // an object: its name and classifier underlined, then its slots
        "UMLObject" => k("UMLObjectView", false, false, false),
        "UMLInformationItem" => k("UMLInformationItemView", true, true, false),
        "UMLStereotype" => k("UMLStereotypeView", true, false, false),
        "UMLMetaClass" => k("UMLMetaClassView", false, false, false),
        // dashed ellipses with their name (elements.js:7022-7280)
        "UMLCollaboration" => k("UMLCollaborationView", false, false, false),
        "UMLCollaborationUse" => k("UMLCollaborationUseView", false, false, false),
        "UMLState" => k("UMLStateView", false, false, false),
        "UMLPseudostate" => k("UMLPseudostateView", false, false, false),
        "UMLConnectionPointReference" => k("UMLConnectionPointReferenceView", false, false, false),
        "UMLFinalState" => k("UMLFinalStateView", false, false, false),
        // activities (elements.js:9903-10300)
        "UMLAction" => k("UMLActionView", false, false, false),
        "UMLObjectNode" => k("UMLObjectNodeView", false, false, false),
        "UMLCentralBufferNode" => k("UMLCentralBufferNodeView", false, false, false),
        "UMLDataStoreNode" => k("UMLDataStoreNodeView", false, false, false),
        "UMLActivityParameterNode" => k("UMLActivityParameterNodeView", false, false, false),
        "UMLStructuredActivityNode" => k("UMLStructuredActivityNodeView", false, false, false),
        "UMLExpansionRegion" => k("UMLExpansionRegionView", false, false, false),
        // plain NodeViews without a name compartment (elements.js:10782-10830)
        "UMLInterruptibleActivityRegion" => k("UMLInterruptibleActivityRegionView", false, false, false),
        "UMLActivityEdgeConnector" => k("UMLActivityEdgeConnectorView", false, false, false),
        "UMLInitialNode" | "UMLActivityFinalNode" | "UMLFlowFinalNode" | "UMLForkNode" | "UMLJoinNode" | "UMLMergeNode" | "UMLDecisionNode" => k("UMLControlNodeView", false, false, false),
        "UMLArtifactInstance" => k("UMLArtifactInstanceView", false, false, false),
        "UMLComponentInstance" => k("UMLComponentInstanceView", false, false, false),
        "UMLNodeInstance" => k("UMLNodeInstanceView", false, false, false),
        _ => None,
    }
}

pub fn edge_view_of(model_ty: &str) -> Option<&'static str> {
    Some(match model_ty {
        "UMLAssociation" => "UMLAssociationView",
        "UMLConnector" => "UMLConnectorView",
        "UMLGeneralization" => "UMLGeneralizationView",
        "UMLDependency" => "UMLDependencyView",
        "UMLRoleBinding" => "UMLRoleBindingView",
        "UMLRealization" => "UMLRealizationView",
        "UMLInterfaceRealization" => "UMLInterfaceRealizationView",
        "UMLInclude" => "UMLIncludeView",
        "UMLExtend" => "UMLExtendView",
        "UMLComponentRealization" => "UMLComponentRealizationView",
        "UMLTemplateBinding" => "UMLTemplateBindingView",
        "UMLDeployment" => "UMLDeploymentView",
        "UMLCommunicationPath" => "UMLCommunicationPathView",
        "UMLLink" => "UMLLinkView",
        "UMLInformationFlow" => "UMLInformationFlowView",
        "UMLExtension" => "UMLExtensionView",
        "UMLTransition" => "UMLTransitionView",
        "UMLControlFlow" => "UMLControlFlowView",
        "UMLObjectFlow" => "UMLObjectFlowView",
        "UMLExceptionHandler" => "UMLExceptionHandlerView",
        "UMLActivityInterrupt" => "UMLActivityInterruptView",
        "FCFlow" => "FCFlowView",
        "DFDDataFlow" => "DFDDataFlowView",
        "MMEdge" => "MMEdgeView",
        "ERDRelationship" => "ERDRelationshipView",
        "C4Relationship" => "C4RelationshipView",
        t if t.starts_with("BPMN") => return crate::bpmn::edge_view_of(t),
        // the cloud notations' lines (aws, azure, gcp metamodel.json)
        t if crate::cloud::edge_view_of(t).is_some() => return crate::cloud::edge_view_of(t),
        t => return crate::sysml::edge_view_of(t),
    })
}

const KNOWN_EDGE_VIEWS: [&str; 35] = [
    "AWSArrowView", "AzureConnectorView", "GCPPathView", "C4RelationshipView", "ERDRelationshipView", "FCFlowView", "DFDDataFlowView", "MMEdgeView", "UMLExceptionHandlerView", "UMLActivityInterruptView", "UMLConnectorView", "UMLTimeConstraintLinkView", "UMLRoleBindingView", "UMLTransitionView", "UMLControlFlowView", "UMLObjectFlowView",
    "UMLAssociationView", "UMLGeneralizationView", "UMLDependencyView", "UMLRealizationView",
    "UMLInterfaceRealizationView", "UMLIncludeView", "UMLExtendView", "UMLNoteLinkView", "UMLContainmentView",
    "UMLComponentRealizationView", "UMLDeploymentView", "UMLCommunicationPathView", "UMLAssociationClassLinkView",
    "UMLTemplateBindingView", "UMLConstraintLinkView", "UMLLinkView", "UMLInformationFlowView", "UMLExtensionView", "UMLLinkObjectLinkView",
];

/// Size used when the person clicks instead of dragging (estimated).
/// Statechart marks: an initial pseudostate is at least 20 px, a final state 26 px (elements.js:131-150).
pub const PSEUDO_SIZE: f64 = 20.0;
pub const FINAL_SIZE: f64 = 26.0;
/// A connection point reference: a 14 px circle on its state's border (elements.js:151-152).
pub const CONNECTION_POINT_SIZE: f64 = 14.0;

/// The smallest a pseudostate may be, by kind (elements.js:131-150): history and terminate marks
/// 26 px, junctions and entry and exit points 15 px, a choice 23 × 19, an initial state 20 px.
pub fn pseudostate_min(kind: &str) -> (f64, f64) {
    match kind {
        "shallowHistory" | "deepHistory" | "terminate" => (26.0, 26.0),
        "junction" | "entryPoint" | "exitPoint" => (15.0, 15.0),
        "choice" => (DECISION_W, DECISION_H),
        "fork" | "join" => (BAR_LENGTH, BAR_THICK),
        _ => (PSEUDO_SIZE, PSEUDO_SIZE),
    }
}

/// An activity edge connector is a circle at least 26 across (elements.js:176); an interruptible
/// region at least 30 × 30 (elements.js:10794-10797).
pub const EDGE_CONNECTOR_SIZE: f64 = 26.0;
pub const REGION_MIN: f64 = 30.0;

/// Swimlanes (elements.js:179-187): a vertical lane is at least 50 wide and 300 high (a horizontal
/// one the other way round), its name 4 px from the top (left) and 10 px from the sides.
pub const LANE_THIN: f64 = 50.0;
pub const LANE_LONG: f64 = 300.0;

/// A swimlane: a lane with its name in a header (UMLSwimlaneView, elements.js:10560-10680).
pub fn create_swimlane_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R, vertical: bool) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "UMLSwimlaneView",
        &[("model", m.clone()), ("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))],
        Some((diagram, "ownedViews")),
    );
    if !vertical {
        r.set(v, "isVertical", json!(false));
    }
    let l = r.create("LabelView", &[("model", m), ("font", json!(FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(2)), ("verticalAlignment", json!(4))], Some((v, "subViews")));
    r.set_ref(v, "nameLabel", Some(l));
    v
}

/// A swimlane's size and its name's box: the header is the name plus 4 + 4 px across the lane
/// (UMLSwimlaneView.sizeObject and arrangeObject, elements.js:10596-10650).
fn layout_swimlane(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let name = r.name(m);
    let (tw, th) = (ms.width(&name, 0).ceil(), ms.size());
    let vertical = r.bool(v, "isVertical");
    // not affected by auto resize (elements.js:10627)
    let (min_w, min_h) = if vertical { ((tw + 20.0).max(LANE_THIN), (th + 8.0).max(LANE_LONG)) } else { ((th + 8.0).max(LANE_LONG), (tw + 20.0).max(LANE_THIN)) };
    let (w, h) = (r.f(v, "width").max(min_w), r.f(v, "height").max(min_h));
    r.set(v, "width", num(w));
    r.set(v, "height", num(h));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    if let Some(l) = r.refd(v, "nameLabel") {
        r.set(l, "text", json!(name));
        r.set(l, "visible", json!(!name.is_empty()));
        if vertical {
            set_box(r, l, x, y + 4.0, w, th);
        } else {
            set_box(r, l, x + 4.0, y, th, h);
        }
    }
}

/// Activity sizes (elements.js:155-175): an action is at least 60 × 40 with corners rounded by 10, a
/// fork or join bar 70 long and 6 thick, a decision or merge diamond 23 × 19.
pub const ACTION_MIN_W: f64 = 60.0;
pub const ACTION_MIN_H: f64 = 40.0;
pub const BAR_LENGTH: f64 = 70.0;
pub const BAR_THICK: f64 = 6.0;
pub const DECISION_W: f64 = 23.0;
pub const DECISION_H: f64 = 19.0;

/// The action kind an activity tool makes (uml.json "uml.activity-adv", "model-init" kind).
pub fn action_kind(tool: &str) -> Option<&'static str> {
    Some(match tool {
        "UMLAction" => "opaque",
        "SendSignal" => "sendSignal",
        "AcceptSignal" => "acceptSignal",
        "AcceptTimeEvent" => "timeEvent",
        _ => return None,
    })
}

/// The pseudostate kind a statechart tool makes (uml.json "uml.statechart", "model-init" kind).
pub fn pseudostate_kind(tool: &str) -> Option<&'static str> {
    Some(match tool {
        "Initial" => "initial",
        "Choice" => "choice",
        "Join" => "join",
        "Fork" => "fork",
        "Junction" => "junction",
        "ShallowHistory" => "shallowHistory",
        "DeepHistory" => "deepHistory",
        "EntryPoint" => "entryPoint",
        "ExitPoint" => "exitPoint",
        "Terminate" => "terminate",
        _ => return None,
    })
}

/// The tools whose toolbox rubber band is a point: the element takes its own size, not the drag's
/// (the "point" items of extensions/essential/uml/toolbox/uml.json).
pub fn is_point_tool(kind: &str) -> bool {
    matches!(
        kind,
        "UMLEndpoint" | "UMLGate" | "UMLStateInvariant" | "UMLTimeConstraint" | "UMLTimingState" | "UMLTimeTick" | "UMLInitialNode" | "UMLActivityFinalNode" | "UMLMergeNode" | "UMLDecisionNode"
            | "UMLFinalState" | "UMLFlowFinalNode" | "UMLActivityEdgeConnector" | "Initial" | "Choice" | "Junction" | "ShallowHistory" | "DeepHistory" | "EntryPoint" | "ExitPoint" | "Terminate"
    )
}

pub fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        // a click makes a UML box at its smallest size: the click's empty rectangle, grown by the
        // view's minimum (NodeView.initialize, core.js:2105-2111; diagram-editor.js:1025-1036)
        "UMLClass" | "UMLInterface" | "UMLEnumeration" | "UMLPackage" | "UMLModel" | "UMLSubsystem" | "UMLDataType" | "UMLPrimitiveType" | "UMLSignal" | "UMLUseCase" | "UMLCollaboration"
        | "UMLCollaborationUse" | "UMLActor" | "UMLComponent" | "UMLArtifact" | "UMLNode" | "UMLUseCaseSubject" | "UMLPart" | "UMLState" | "SubmachineState" => (1.0, 1.0),
        "Boundary" | "Entity" | "Control" => (ROBUST_ICON_W, ROBUST_ICON_H + NAME_MIN_H),
        // the common tools' min-width and min-height (common/toolbox/common.json)
        "Note" => (70.0, 40.0),
        "Text" | "TextBox" => (60.0, 15.0),
        "Frame" => (FRAME_MIN_W, FRAME_MIN_H),
        "Hyperlink" => (HYPERLINK_MIN_W, HYPERLINK_MIN_H),
        "Rect" | "RoundRect" | "Ellipse" => (50.0, 50.0),
        "Image" => (100.0, 100.0),
        "UMLPort" => (PORT_SIZE, PORT_SIZE),
        // activities: the minimum sizes, which a plain click makes (elements.js:155-175)
        "UMLAction" | "SendSignal" | "AcceptSignal" => (ACTION_MIN_W, ACTION_MIN_H),
        "AcceptTimeEvent" => (24.0, 24.0 + NAME_MIN_H),
        "UMLObjectNode" | "UMLCentralBufferNode" | "UMLDataStoreNode" | "UMLActivityParameterNode" => (ACTION_MIN_W, ACTION_MIN_H),
        // the regions take a roomier box on a plain click (estimated)
        "UMLStructuredActivityNode" | "UMLExpansionRegion" | "UMLInterruptibleActivityRegion" => (200.0, 150.0),
        "UMLActivityEdgeConnector" => (EDGE_CONNECTOR_SIZE, EDGE_CONNECTOR_SIZE),
        "UMLInitialNode" => (PSEUDO_SIZE, PSEUDO_SIZE),
        "UMLActivityFinalNode" | "UMLFlowFinalNode" => (FINAL_SIZE, FINAL_SIZE),
        "UMLForkNode" | "UMLJoinNode" => (BAR_LENGTH, BAR_THICK),
        "SwimlaneVert" => (LANE_THIN, LANE_LONG),
        "SwimlaneHorz" => (LANE_LONG, LANE_THIN),
        "UMLMergeNode" | "UMLDecisionNode" => (DECISION_W, DECISION_H),
        "CompositeState" | "OrthogonalState" => (200.0, 150.0),
        "UMLFinalState" => (FINAL_SIZE, FINAL_SIZE),
        _ if pseudostate_kind(kind).is_some() => pseudostate_min(pseudostate_kind(kind).unwrap_or("initial")),
        "UMLConnectionPointReference" => (CONNECTION_POINT_SIZE, CONNECTION_POINT_SIZE),
        "UMLNaryAssociationNode" => (NARY_MIN_W, NARY_MIN_H),
        // sequence fragments take their smallest size on a click (elements.js:212-229)
        "UMLCombinedFragment" => (crate::sequence::FRAGMENT_MIN_W * 2.0, crate::sequence::FRAGMENT_MIN_H * 3.0),
        "UMLInteractionUse" => (crate::sequence::FRAGMENT_MIN_W * 2.0, crate::sequence::FRAGMENT_MIN_H),
        "UMLContinuation" | "UMLStateInvariant" => crate::sequence::INVARIANT_MIN,
        "UMLTimeConstraint" => (1.0, 1.0),
        "UMLDurationConstraint" => (crate::sequence::DURATION_MIN_W, crate::sequence::DURATION_MIN_H),
        // a BPMN element at its smallest size, but pools, lanes and groups (bpmn/elements.js:44-55)
        _ if crate::bpmn::click_size(kind).is_some() => crate::bpmn::click_size(kind).unwrap_or((1.0, 1.0)),
        // a click makes a flowchart node at its smallest size, as in the original (factory.js:279-292)
        _ if crate::flowchart::is_node_type(kind) || crate::notations::is_node_type(kind) || kind == "ERDEntity" || crate::c4::tool(kind).is_some() || crate::wireframe::is_node_type(kind) || crate::cloud::is_node_type(kind) || crate::sysml::view_of(kind).is_some() || crate::sysml::property_tool(kind).is_some() => (1.0, 1.0),
        _ => (100.0, 60.0),
    }
}

/// The smallest box an annotation tool makes: a drag smaller than this is enlarged (the toolbox
/// items' `min-width` and `min-height`, common/toolbox/common.json; factory.js:279-292).
pub fn tool_min_size(kind: &str) -> Option<(f64, f64)> {
    match kind {
        "Text" | "TextBox" => Some((60.0, 15.0)),
        "Note" => Some((70.0, 40.0)),
        "Hyperlink" => Some((70.0, 20.0)),
        "Rect" | "RoundRect" | "Ellipse" => Some((50.0, 50.0)),
        "Image" => Some((100.0, 100.0)),
        _ => None,
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
/// A free line: a line drawn on its own, joined to no box.
pub fn is_freeline(r: &Repo, v: Eid) -> bool {
    r.ty(v) == "FreelineEdgeView"
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

pub(crate) fn name_compartment(r: &mut Repo, view: Eid, model: Eid) -> Eid {
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

pub(crate) fn list_compartment(r: &mut Repo, view: Eid, model: Eid, ty: &str, visible: bool) -> Eid {
    let m = r.ref_to(model);
    let c = r.create(ty, &[("model", m), ("font", json!(FONT)), ("parentStyle", json!(true)), ("width", json!(10)), ("height", json!(10))], Some((view, "subViews")));
    if !visible {
        set_false(r, c, "visible");
    }
    c
}

pub fn create_node_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Option<Eid> {
    if crate::flowchart::is_node_type(r.ty(model)) {
        return Some(crate::flowchart::create_view(r, diagram, model, rect));
    }
    if crate::notations::is_node_type(r.ty(model)) {
        return Some(crate::notations::create_view(r, diagram, model, rect));
    }
    if crate::wireframe::is_node_type(r.ty(model)) {
        return Some(crate::wireframe::create_view(r, diagram, model, rect));
    }
    if r.ty(model) == "ERDEntity" {
        return Some(crate::erd::create_view(r, diagram, model, rect));
    }
    if crate::bpmn::is_node_type(r.ty(model)) {
        return crate::bpmn::create_view(r, diagram, model, rect, false);
    }
    if crate::c4::is_node_type(r.ty(model)) {
        return Some(crate::c4::create_view(r, diagram, model, rect));
    }
    if crate::cloud::is_node_type(r.ty(model)) {
        return Some(crate::cloud::create_view(r, diagram, model, rect));
    }
    if crate::sysml::view_of(r.ty(model)).is_some() {
        return crate::sysml::create_view(r, diagram, model, rect, false);
    }
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
    // a control node is a NodeView without a name (elements.js:10288), and so are an interruptible
    // region and an edge connector (elements.js:10782-10830)
    if matches!(kind.view, "UMLControlNodeView" | "UMLInterruptibleActivityRegionView" | "UMLActivityEdgeConnectorView" | "UMLFinalStateView") {
        return Some(v);
    }
    // a pseudostate is a floating node whose labels stay hidden (UMLPseudostateView, elements.js:8835)
    if kind.view == "UMLPseudostateView" {
        floating_labels(r, v, model);
        return Some(v);
    }
    let nc = name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    // a collaboration keeps a template parameter compartment, hidden while it has none (elements.js:7022-7050)
    if r.ty(model) == "UMLCollaboration" {
        let tp = list_compartment(r, v, model, "UMLTemplateParameterCompartmentView", false);
        r.set_ref(v, "templateParameterCompartment", Some(tp));
    }
    if r.kind_of(model, "UMLInstance") {
        let s = list_compartment(r, v, model, "UMLSlotCompartmentView", true);
        r.set_ref(v, "slotCompartment", Some(s));
    }
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

/// A frame showing `model` (or nothing), with its name label and its bold diagram-kind label
/// (UMLCustomFrameView, elements.js:12167-12180; uml-commands.js:244-262).
pub fn create_frame_view(r: &mut Repo, diagram: Eid, model: Option<Eid>, rect: R) -> Eid {
    create_frame_view_of(r, "UMLFrameView", diagram, model, rect)
}

/// A frame-like view of the given type (a frame, or an interaction overview's inline interaction).
pub fn create_frame_view_of(r: &mut Repo, ty: &str, diagram: Eid, model: Option<Eid>, rect: R) -> Eid {
    let mut props = vec![("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))];
    if let Some(m) = model {
        props.insert(0, ("model", r.ref_to(m)));
    }
    let v = r.create(ty, &props, Some((diagram, "ownedViews")));
    let nm = label(r, v, &[("height", json!(LABEL_H)), ("horizontalAlignment", json!(0))]);
    let ty = label(r, v, &[("font", json!("Arial;13;1")), ("height", json!(LABEL_H)), ("horizontalAlignment", json!(0))]);
    r.set_ref(v, "nameLabel", Some(nm));
    r.set_ref(v, "frameTypeLabel", Some(ty));
    v
}

/// A collaboration or collaboration use: at least √2 times its name (the ellipse around it) and the
/// use case minimum, the name centred; a use shows "name: Collaboration" once typed
/// (elements.js:7076-7134, 7238-7279).
/// A state's internal activities, one row each: `entry/ name`, `do/ name`, `exit/ name`.
pub fn state_activity_rows(r: &Repo, m: Eid) -> Vec<String> {
    let mut out = vec![];
    for (field, label) in [("entryActivities", "entry"), ("doActivities", "do"), ("exitActivities", "exit")] {
        for b in r.list(m, field).into_iter().filter(|b| r.alive(*b)) {
            out.push(format!("{label}/ {}", r.name(b)));
        }
    }
    out
}

/// A state: a rounded box with its name on top and its activities under a line (sizes estimated).
fn layout_state(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let rows = state_activity_rows(r, m);
    let lh = ms.size();
    let rows_w = rows.iter().map(|t| ms.width(t, 0)).fold(0.0, f64::max) + 2.0 * PAD;
    // a submachine state keeps 16 px under its compartments for the submachine mark
    // (UMLStateView.sizeObject, elements.js:9509-9519)
    let mark = if r.refd(m, "submachine").is_some() { 16.0 } else { 0.0 };
    let rows_h = if rows.is_empty() { 0.0 } else { rows.len() as f64 * lh + 2.0 * PAD } + mark;
    let min_w = name_min_width(r, ms, v, m, None, false).max(rows_w.ceil()).max(40.0);
    grow(r, v, min_w, NAME_MIN_H + rows_h);
    let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
    let nh = layout_name(r, ms, v, m, x, y, w, None, false);
    grow(r, v, min_w, nh + rows_h);
}

/// An action or an object node: its name on top, at least 60 × 40 for an action. An accept time
/// event action is an hourglass of 24 × 24 with its name under it (UMLActionView.sizeObject,
/// arrangeObject, elements.js:9937-9978).
fn layout_activity_box(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let kw = keyword_of(r.ty(m));
    let min_w = name_min_width(r, ms, v, m, kw, false);
    if r.str(m, "kind") == "timeEvent" {
        grow(r, v, min_w.max(24.0), 24.0 + NAME_MIN_H);
        let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
        let nh = layout_name(r, ms, v, m, x, y + 24.0, w, kw, false);
        grow(r, v, min_w.max(24.0), 24.0 + nh);
        return;
    }
    let (min_w, min_h) = if r.ty(m) == "UMLAction" {
        let extra = if r.str(m, "kind") == "callBehavior" { 15.0 } else { 0.0 };
        ((min_w + extra).max(ACTION_MIN_W), ACTION_MIN_H)
    } else {
        (min_w, NAME_MIN_H)
    };
    grow(r, v, min_w, min_h);
    let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
    let nh = layout_name(r, ms, v, m, x, y, w, kw, false);
    grow(r, v, min_w, min_h.max(nh));
}

/// The keyword over a structured activity's name: «structured», or an expansion region's mode
/// (UMLStructuredActivityNodeView.getStereotypeLabelText, UMLExpansionRegionView, elements.js:10843,
/// 10927-10940).
pub fn structured_keyword(r: &Repo, m: Eid) -> &'static str {
    if r.ty(m) != "UMLExpansionRegion" {
        return "structured";
    }
    match r.str(m, "mode").as_str() {
        "parallel" => "parallel",
        "stream" => "stream",
        _ => "iterative",
    }
}

/// A structured activity or an expansion region: its keyword, left-aligned, and name on top of a
/// box that holds activity nodes (UMLStructuredActivityNodeView.update, elements.js:10857-10866).
fn layout_structured(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let kw = Some(structured_keyword(r, m));
    if let Some(st) = r.refd(v, "nameCompartment").and_then(|nc| r.refd(nc, "stereotypeLabel")) {
        r.set(st, "horizontalAlignment", json!(0));
    }
    let min_w = name_min_width(r, ms, v, m, kw, false).max(ACTION_MIN_W);
    grow(r, v, min_w, ACTION_MIN_H);
    let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
    let nh = layout_name(r, ms, v, m, x, y, w, kw, false);
    grow(r, v, min_w, ACTION_MIN_H.max(nh));
}

/// A control node keeps at least its mark's size; a fork or join bar is horizontal unless it is
/// taller than wide, and keeps its thickness (UMLControlNodeView.sizeObject, elements.js:10296-10343).
fn layout_control_node(r: &mut Repo, v: Eid, m: Eid) {
    let (w, h) = (r.f(v, "width"), r.f(v, "height"));
    let (min_w, min_h) = match r.ty(m) {
        "UMLInitialNode" => (PSEUDO_SIZE, PSEUDO_SIZE),
        "UMLActivityFinalNode" | "UMLFlowFinalNode" => (FINAL_SIZE, FINAL_SIZE),
        "UMLForkNode" | "UMLJoinNode" => {
            if h > w {
                r.set(v, "width", num(BAR_THICK));
                (BAR_THICK, BAR_LENGTH)
            } else {
                r.set(v, "height", num(BAR_THICK));
                (BAR_LENGTH, BAR_THICK)
            }
        }
        _ => (DECISION_W, DECISION_H),
    };
    grow(r, v, min_w, min_h);
}

fn layout_collaboration(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let typed = (r.ty(m) == "UMLCollaborationUse").then(|| r.refd(m, "type").map(|t| r.name(t))).flatten().filter(|n| !n.is_empty());
    let mut min_w = name_min_width(r, ms, v, m, None, false);
    if let Some(t) = &typed {
        min_w = min_w.max((ms.width(&format!("{}: {t}", r.name(m)), name_style(r, m)) + 2.0 * PAD).ceil());
    }
    grow(r, v, (std::f64::consts::SQRT_2 * min_w).floor().max(UC_MIN_W), (std::f64::consts::SQRT_2 * NAME_MIN_H).floor().max(UC_MIN_H));
    let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
    // lay the name out once to learn its height, then centre it
    let nh = layout_name(r, ms, v, m, x, y, w, None, false);
    layout_name(r, ms, v, m, x, y + (h - nh) / 2.0, w, None, false);
    if let (Some(t), Some(nm)) = (typed, r.refd(v, "nameCompartment").and_then(|nc| r.refd(nc, "nameLabel"))) {
        let text = format!("{}: {t}", r.name(m));
        r.set(nm, "text", json!(text));
    }
}

/// An n-ary association node's diamond, with its name, stereotype and property labels floating
/// around it (UMLFloatingNodeView, elements.js:5162-5188).
pub fn create_nary_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "UMLNaryAssociationNodeView",
        &[("model", m.clone()), ("font", json!(FONT)), ("parentStyle", json!(false)), ("containerChangeable", json!(false)), ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))],
        Some((diagram, "ownedViews")),
    );
    let pi = std::f64::consts::PI;
    for (field, alpha, distance) in [("nameLabel", 3.0 * pi / 4.0, 20.0), ("stereotypeLabel", 3.0 * pi / 4.0, 35.0), ("propertyLabel", -3.0 * pi / 4.0, 20.0)] {
        let l = r.create("NodeLabelView", &[("model", m.clone()), ("font", json!(FONT)), ("alpha", json!(alpha)), ("distance", json!(distance)), ("height", json!(LABEL_H))], Some((v, "subViews")));
        r.set_ref(v, field, Some(l));
    }
    v
}

/// A port on the border of its classifier's box: a 14 × 14 square with floating labels
/// (UMLPortView, elements.js:6899-6950), contained in `host`.
pub fn create_port_view(r: &mut Repo, diagram: Eid, model: Eid, host: Eid, at: P) -> Eid {
    let m = r.ref_to(model);
    let h = PORT_SIZE / 2.0;
    let v = r.create(
        "UMLPortView",
        &[("model", m.clone()), ("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(at.x - h)), ("top", num(at.y - h)), ("width", num(PORT_SIZE)), ("height", num(PORT_SIZE))],
        Some((diagram, "ownedViews")),
    );
    let pi = std::f64::consts::PI;
    for (field, alpha, distance) in [("nameLabel", 3.0 * pi / 4.0, 20.0), ("stereotypeLabel", 3.0 * pi / 4.0, 35.0), ("propertyLabel", -3.0 * pi / 4.0, 20.0)] {
        let l = r.create("NodeLabelView", &[("model", m.clone()), ("font", json!(FONT)), ("alpha", json!(alpha)), ("distance", json!(distance)), ("height", json!(LABEL_H))], Some((v, "subViews")));
        r.set_ref(v, field, Some(l));
    }
    contain(r, host, v);
    v
}

/// The name, stereotype and property labels floating around a node (UMLFloatingNodeView,
/// elements.js:5162-5188).
pub(crate) fn floating_labels(r: &mut Repo, v: Eid, model: Eid) {
    let m = r.ref_to(model);
    let pi = std::f64::consts::PI;
    for (field, alpha, distance) in [("nameLabel", 3.0 * pi / 4.0, 20.0), ("stereotypeLabel", 3.0 * pi / 4.0, 35.0), ("propertyLabel", -3.0 * pi / 4.0, 20.0)] {
        let l = r.create("NodeLabelView", &[("model", m.clone()), ("font", json!(FONT)), ("alpha", json!(alpha)), ("distance", json!(distance)), ("height", json!(LABEL_H))], Some((v, "subViews")));
        r.set_ref(v, field, Some(l));
    }
}

/// A connection point reference on its state's border: a 14 px circle (an exit point's with an
/// X), labelled with the names of the entry and exit points it refers to, contained in `host`
/// (UMLConnectionPointReferenceView, elements.js:9154-9230).
pub fn create_connection_point_view(r: &mut Repo, diagram: Eid, model: Eid, host: Eid, at: P) -> Eid {
    let h = CONNECTION_POINT_SIZE / 2.0;
    let v = r.create(
        "UMLConnectionPointReferenceView",
        &[("model", r.ref_to(model)), ("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(at.x - h)), ("top", num(at.y - h)), ("width", num(CONNECTION_POINT_SIZE)), ("height", num(CONNECTION_POINT_SIZE))],
        Some((diagram, "ownedViews")),
    );
    floating_labels(r, v, model);
    contain(r, host, v);
    v
}

/// The text a connection point reference shows: its entry and exit points' names, or its own name.
pub fn connection_point_text(r: &Repo, m: Eid) -> String {
    let names: Vec<String> = r.ref_list(m, "entry").into_iter().chain(r.ref_list(m, "exit")).map(|p| r.name(p)).filter(|n| !n.is_empty()).collect();
    if names.is_empty() {
        r.name(m)
    } else {
        names.join(", ")
    }
}

/// Centred on its state's border where the line from the state's centre through it meets it
/// (UMLConnectionPointReferenceView.arrange, elements.js:9192-9205).
fn layout_connection_point(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    if let Some(host) = r.refd(v, "containerView").filter(|h| r.alive(*h)) {
        let c = node_rect(r, host);
        let p = port_junction(c, center(&node_rect(r, v)));
        let h = CONNECTION_POINT_SIZE / 2.0;
        r.set(v, "left", num(p.x - h));
        r.set(v, "top", num(p.y - h));
    }
    r.set(v, "width", num(CONNECTION_POINT_SIZE));
    r.set(v, "height", num(CONNECTION_POINT_SIZE));
    let text = connection_point_text(r, m);
    let st = stereotype_text(r, m);
    for (field, t) in [("nameLabel", text), ("stereotypeLabel", st), ("propertyLabel", String::new())] {
        if let Some(lv) = r.refd(v, field) {
            place_floating_label(r, ms, v, lv, &t, true);
        }
    }
}

/// A pin on its action's border: an 18 × 18 square with its labels floating by it, contained in
/// `host` (UMLInputPinView, UMLOutputPinView, elements.js:9662-9800).
pub fn create_pin_view(r: &mut Repo, diagram: Eid, model: Eid, host: Eid, at: P) -> Eid {
    let m = r.ref_to(model);
    let ty = match r.ty(model) {
        "UMLInputPin" => "UMLInputPinView",
        "UMLExpansionNode" => "UMLExpansionNodeView",
        _ => "UMLOutputPinView",
    };
    let h = PIN_SIZE / 2.0;
    let v = r.create(
        ty,
        &[("model", m.clone()), ("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(at.x - h)), ("top", num(at.y - h)), ("width", num(PIN_SIZE)), ("height", num(PIN_SIZE))],
        Some((diagram, "ownedViews")),
    );
    let pi = std::f64::consts::PI;
    for (field, alpha, distance) in [("nameLabel", 3.0 * pi / 4.0, 20.0), ("stereotypeLabel", 3.0 * pi / 4.0, 35.0), ("propertyLabel", -3.0 * pi / 4.0, 20.0)] {
        let l = r.create("NodeLabelView", &[("model", m.clone()), ("font", json!(FONT)), ("alpha", json!(alpha)), ("distance", json!(distance)), ("height", json!(LABEL_H))], Some((v, "subViews")));
        r.set_ref(v, field, Some(l));
    }
    contain(r, host, v);
    v
}

/// Which side of its action a pin sits on: within half a pin of the top, bottom, left or right
/// edge (UMLPinView.getPosition, elements.js:9687-9703).
pub fn pin_side(r: &Repo, v: Eid) -> &'static str {
    let Some(host) = r.refd(v, "containerView") else { return "else" };
    let (c, b) = (node_rect(r, host), node_rect(r, v));
    let range = (PIN_SIZE / 2.0).round();
    let near = |a: f64, x: f64| a - range <= x && x <= a + range;
    if near(c.y, b.y + b.h) {
        "top"
    } else if near(c.y + c.h, b.y) {
        "bottom"
    } else if near(c.x, b.x + b.w) {
        "left"
    } else if near(c.x + c.w, b.x) {
        "right"
    } else {
        "else"
    }
}

/// A pin's size: 18 × 18; an expansion node is four cells long along the border it sits on
/// (UMLExpansionNodeView.sizeObject, elements.js:9832-9846).
fn pin_size(r: &Repo, v: Eid) -> (f64, f64) {
    if r.ty(v) != "UMLExpansionNodeView" {
        return (PIN_SIZE, PIN_SIZE);
    }
    match pin_side(r, v) {
        "top" | "bottom" => (PIN_SIZE * 4.0, PIN_SIZE),
        "left" | "right" => (PIN_SIZE, PIN_SIZE * 4.0),
        _ => (r.f(v, "width").max(PIN_SIZE), r.f(v, "height").max(PIN_SIZE)),
    }
}

/// A pin: just outside its action's border, where the line from the action's centre through the
/// pin meets it; the text `name: Type[mult]` floats by it (UMLPinView.arrange and update,
/// elements.js:9668-9680, 9741-9765).
fn layout_pin(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    if let Some(host) = r.refd(v, "containerView").filter(|h| r.alive(*h)) {
        let c = node_rect(r, host);
        let b = node_rect(r, v);
        let p = port_junction(c, center(&b));
        let inside = b.x <= p.x && p.x <= b.x + b.w && b.y <= p.y && p.y <= b.y + b.h;
        let (w, h) = pin_size(r, v);
        if !inside {
            let x = if c.x < p.x { p.x } else { p.x - w + 1.0 };
            let y = if c.y < p.y { p.y } else { p.y - h + 1.0 };
            r.set(v, "left", num(x));
            r.set(v, "top", num(y));
        }
    }
    let (w, h) = pin_size(r, v);
    r.set(v, "width", num(w));
    r.set(v, "height", num(h));
    let show = Show { vis: false, mult: true, sig: true, ty: true, prop: true };
    let text = attribute_text(r, m, show);
    let st = stereotype_text(r, m);
    let text = if st.is_empty() { text } else { text.trim_start_matches(st.as_str()).trim_start().to_string() };
    for (field, t) in [("nameLabel", text), ("stereotypeLabel", st), ("propertyLabel", String::new())] {
        if let Some(lv) = r.refd(v, field) {
            place_floating_label(r, ms, v, lv, &t, true);
            if field == "nameLabel" {
                let underline = r.bool(m, "isStatic");
                r.set(lv, "underline", json!(underline));
            }
        }
    }
}

/// A part inside its classifier's box: a box with the part's "name: Type[mult]" (UMLPartView,
/// elements.js:6955-6985), contained in `host`.
pub fn create_part_view(r: &mut Repo, diagram: Eid, model: Eid, host: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "UMLPartView",
        &[("model", m), ("font", json!(FONT)), ("parentStyle", json!(false)), ("showVisibility", json!(false)), ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))],
        Some((diagram, "ownedViews")),
    );
    let nc = name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    contain(r, host, v);
    v
}

/// Record `v` as contained in `host` (containerView, containedViews).
pub(crate) fn contain(r: &mut Repo, host: Eid, v: Eid) {
    r.set_ref(v, "containerView", Some(host));
    let mut a = match r.raw(host, "containedViews") {
        Some(Value::Array(a)) => a.clone(),
        _ => vec![],
    };
    a.push(r.ref_to(v));
    r.set_raw(host, "containedViews", json!(a));
}

/// Where a port sits on its box `b`: where the line from the box's centre through the port's centre
/// `p` meets the border; a port inside goes to the nearest of those border points
/// (UMLFloatingNodeView._junction2, elements.js:5232-5290).
pub fn port_junction(b: R, p: P) -> P {
    let c = center(&b);
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    if c.x == p.x || c.y == p.y {
        // straight above, below or beside the centre (Coord.orthoJunction)
        if c.x == p.x {
            return P { x: p.x, y: if p.y < c.y { y1 } else { y2 } };
        }
        return P { x: if p.x < c.x { x1 } else { x2 }, y: p.y };
    }
    let lean = (p.y - c.y) / (p.x - c.x);
    let cp = [
        P { x: x1, y: (lean * (x1 - c.x) + c.y).round() },
        P { x: x2, y: (lean * (x2 - c.x) + c.y).round() },
        P { x: ((y1 - c.y) / lean + c.x).round(), y: y1 },
        P { x: ((y2 - c.y) / lean + c.x).round(), y: y2 },
    ];
    if x1 <= p.x && p.x <= x2 && y1 <= p.y && p.y <= y2 {
        let d = |q: &P| (q.x - p.x).powi(2) + (q.y - p.y).powi(2);
        return *cp.iter().min_by(|a, b| d(a).total_cmp(&d(b))).unwrap();
    }
    let (lx, hx, ly, hy) = (c.x.min(p.x), c.x.max(p.x), c.y.min(p.y), c.y.max(p.y));
    cp.into_iter().find(|q| x1 <= q.x && q.x <= x2 && y1 <= q.y && q.y <= y2 && lx <= q.x && q.x <= hx && ly <= q.y && q.y <= hy).unwrap_or(c)
}

/// An attribute's text without its leading «stereotype», which a port or a part shows apart.
pub(crate) fn feature_text(r: &Repo, v: Eid, m: Eid) -> String {
    let show = Show { vis: r.bool(v, "showVisibility"), mult: r.bool(v, "showMultiplicity"), sig: true, ty: r.bool(v, "showType"), prop: false };
    let text = attribute_text(r, m, show);
    let st = stereotype_text(r, m);
    if st.is_empty() { text } else { text.trim_start_matches(st.as_str()).trim_start().to_string() }
}

/// A port: snapped onto its container's border, 14 × 14, with "name: Type[mult]" floating by it
/// (UMLPortView.update and arrange, elements.js:6914-6948).
pub(crate) fn layout_port(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    if let Some(host) = r.refd(v, "containerView").filter(|h| r.alive(*h)) {
        let p = port_junction(node_rect(r, host), center(&node_rect(r, v)));
        set_box(r, v, p.x - PORT_SIZE / 2.0, p.y - PORT_SIZE / 2.0, PORT_SIZE, PORT_SIZE);
    } else {
        grow(r, v, PORT_SIZE, PORT_SIZE);
    }
    let name = feature_text(r, v, m);
    let st = stereotype_text(r, m);
    for (field, text) in [("nameLabel", name), ("stereotypeLabel", st), ("propertyLabel", String::new())] {
        if let Some(lv) = r.refd(v, field) {
            place_floating_label(r, ms, v, lv, &text, true);
            if field == "nameLabel" {
                let underline = r.bool(m, "isStatic");
                r.set(lv, "underline", json!(underline));
            }
        }
    }
}

/// A part: the box around its "name: Type[mult]" (no visibility unless shown), underlined when
/// static (UMLPartView.update, elements.js:6967-6975).
fn layout_part(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    let text = feature_text(r, v, m);
    let min_w = ((ms.width(&text, 1) + 2.0 * PAD).ceil()).max(name_min_width(r, ms, v, m, None, false));
    grow(r, v, min_w, NAME_MIN_H);
    let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
    let nh = layout_name(r, ms, v, m, x, y, w, None, false);
    layout_name(r, ms, v, m, x, y + ((h - nh) / 2.0).max(0.0), w, None, false);
    if let Some(nm) = r.refd(v, "nameCompartment").and_then(|nc| r.refd(nc, "nameLabel")) {
        r.set(nm, "text", json!(text));
        let underline = r.bool(m, "isStatic");
        r.set(nm, "underline", json!(underline));
    }
}

/// A floating label's box: its text's size, centred on the point at its angle and distance from
/// the host's centre (NodeParasiticView.arrange, core.js:3600-3616).
pub(crate) fn place_floating_label(r: &mut Repo, ms: &dyn Measure, host: Eid, lv: Eid, text: &str, shown: bool) {
    r.set(lv, "text", json!(text));
    r.set(lv, "visible", json!(shown && !text.is_empty()));
    // as wide as its text (its widest word with word wrap) or as it was widened (NodeLabelView.sizeObject,
    // core.js:3705-3727)
    let wrap = r.bool(lv, "wordWrap");
    let min_w = if wrap { widest_word(ms, text, 0) } else { ms.width(text, 0) }.ceil();
    let w = if text.is_empty() { min_w } else { r.f(lv, "width").max(min_w) };
    let h = if wrap && !text.is_empty() { wrap_text(ms, text, w, 0).len().max(1) as f64 * ms.size() } else { ms.size() };
    let b = node_rect(r, host);
    let c = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
    let off = point_away_line(c, c, r.f(lv, "alpha"), r.f(lv, "distance"));
    set_box(r, lv, off.x + c.x - w / 2.0, off.y + c.y - h / 2.0, w, h);
}

/// An n-ary association node: at least 30×22, with its name (and visibility when shown), its
/// stereotype and its properties around it (UMLNaryAssociationNodeView, elements.js:6596-6617).
fn layout_nary(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) {
    grow(r, v, NARY_MIN_W, NARY_MIN_H);
    let mut name = r.name(m);
    if r.bool(v, "showVisibility") {
        name = format!("{}{name}", vis_symbol(&r.str(m, "visibility")));
    }
    let st = stereotype_text(r, m);
    let prop = property_text(r, m);
    let show_prop = r.bool(v, "showProperty");
    for (field, text, shown) in [("nameLabel", name, true), ("stereotypeLabel", st, true), ("propertyLabel", prop, show_prop)] {
        if let Some(lv) = r.refd(v, field) {
            place_floating_label(r, ms, v, lv, &text, shown);
        }
    }
}

/// A hyperlink's view: a bold "link" label and the target's name (HyperlinkView, elements.js:14060-14073).
pub fn create_hyperlink_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "HyperlinkView",
        &[("model", m), ("font", json!(FONT)), ("parentStyle", json!(false)), ("left", num(rect.x)), ("top", num(rect.y)), ("width", num(rect.w)), ("height", num(rect.h))],
        Some((diagram, "ownedViews")),
    );
    let nm = label(r, v, &[("height", json!(LABEL_H)), ("horizontalAlignment", json!(0))]);
    let ty = label(r, v, &[("font", json!("Arial;13;1")), ("height", json!(LABEL_H)), ("horizontalAlignment", json!(0))]);
    r.set_ref(v, "nameLabel", Some(nm));
    r.set_ref(v, "typeLabel", Some(ty));
    v
}

/// What a hyperlink's view shows after "link": its name with showName, else the referenced
/// element's name, else its URL (HyperlinkView.update, elements.js:14108-14122).
pub fn hyperlink_text(r: &Repo, m: Eid) -> String {
    if r.bool(m, "showName") {
        r.name(m)
    } else if let Some(x) = r.refd(m, "reference") {
        r.name(x)
    } else {
        r.str(m, "url")
    }
}

/// A hyperlink's labels and minimum size (HyperlinkView.sizeObject and arrangeObject, elements.js:14075-14100).
fn layout_hyperlink(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let (Some(nl), Some(tl)) = (r.refd(v, "nameLabel"), r.refd(v, "typeLabel")) else { return };
    let text = r.refd(v, "model").map(|m| hyperlink_text(r, m)).unwrap_or_default();
    r.set(nl, "text", json!(text));
    r.set(tl, "text", json!("link"));
    let (size, face, style) = (ms.size(), ms.face(), font_style_of(r, v));
    r.set(nl, "font", json!(font_str(&face, size, style)));
    r.set(tl, "font", json!(font_str(&face, size, 1)));
    let tw = ms.width("link", 1).ceil();
    let nw = ms.width(&text, style as u8).ceil();
    grow(r, v, (tw + nw + (PAD + PAD) * 2.0).max(HYPERLINK_MIN_W), (size + PAD + PAD).max(HYPERLINK_MIN_H));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    set_box(r, tl, x + PAD, y + PAD, tw, size);
    set_box(r, nl, x + PAD + tw + PAD + PAD, y + PAD, nw, size);
}

/// The short diagram kind a frame's tab shows for an element or a diagram ("class", "uc"…), or ""
/// (UMLFrameView.getDiagramTypeName, elements.js:12266-12316).
pub fn frame_type_name(r: &Repo, e: Eid) -> &'static str {
    let k = |t: &str| r.kind_of(e, t);
    if k("UMLClass") || k("UMLClassDiagram") {
        "class"
    } else if k("UMLComponent") || k("UMLComponentDiagram") {
        "cmp"
    } else if k("UMLDeploymentDiagram") {
        "dep"
    } else if k("UMLInteraction") || k("UMLSequenceDiagram") || k("UMLCommunicationDiagram") || k("UMLTimingDiagram") || k("UMLInteractionOverviewDiagram") {
        "sd"
    } else if k("UMLStateMachine") || k("UMLStatechartDiagram") {
        "stm"
    } else if k("UMLActivity") || k("UMLActivityDiagram") {
        "act"
    } else if k("UMLUseCase") || k("UMLUseCaseDiagram") {
        "uc"
    } else if k("UMLPackage") || k("UMLPackageDiagram") {
        "pkg"
    } else if k("SysMLRequirementDiagram") {
        "req"
    } else if k("SysMLBlockDefinitionDiagram") {
        "bdd"
    } else if k("SysMLParametricDiagram") {
        "par"
    } else if k("SysMLInternalBlockDiagram") {
        "ibd"
    } else {
        ""
    }
}

/// The text of a frame's name label: `[Type]`, the name (with `Owner::` when Show Namespace is on),
/// then `[Diagram]` with the name of the diagram the frame is drawn in. A frame showing a diagram
/// names the diagram's owner (UMLFrameView.update and getTarget, elements.js:12318-12374).
pub fn frame_name_text(r: &Repo, v: Eid, m: Eid) -> String {
    let target = match r.parent(m) {
        Some(p) if r.kind_of(m, "Diagram") && r.kind_of(p, "ExtensibleModel") => p,
        _ => m,
    };
    let mut tokens = vec![];
    if r.bool(v, "showType") {
        tokens.push(format!("[{}]", crate::sysml::display_name(r.ty(target).trim_start_matches("UML"))));
    }
    match r.parent(target) {
        Some(p) if r.bool(v, "showNamespace") && r.kind_of(p, "ExtensibleModel") => tokens.push(format!("{}::{}", r.name(p), r.name(target))),
        _ => tokens.push(r.name(target)),
    }
    if r.bool(v, "showDiagramName") {
        if let Some(d) = r.parent(v) {
            tokens.push(format!("[{}]", r.name(d)));
        }
    }
    tokens.join(" ")
}

/// A frame's labels and minimum size (UMLCustomFrameView.sizeObject and arrangeObject, elements.js:12182-12212).
fn layout_frame(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let (Some(nl), Some(tl)) = (r.refd(v, "nameLabel"), r.refd(v, "frameTypeLabel")) else { return };
    // an inline interaction shows the diagram its action calls (UMLInteractionInlineView.update)
    let model = r.refd(v, "model").map(|m| if r.ty(m) == "UMLAction" { r.refd(m, "target").unwrap_or(m) } else { m });
    if let Some(m) = model {
        let mut kind = frame_type_name(r, m);
        if kind.is_empty() {
            if let Some(d) = r.parent(v) {
                kind = frame_type_name(r, d);
            }
        }
        r.set(tl, "text", json!(kind));
        let text = frame_name_text(r, v, m);
        r.set(nl, "text", json!(text));
    }
    let size = ms.size();
    let face = ms.face();
    // the labels take the frame's font; the kind label is always bold
    let style = font_style_of(r, v);
    for (lv, st) in [(nl, style), (tl, 1)] {
        r.set(lv, "font", json!(font_str(&face, size, st)));
    }
    let tw = ms.width(&r.str(tl, "text"), 1).ceil();
    let nw = ms.width(&r.str(nl, "text"), style as u8).ceil();
    let h = size + PAD + PAD;
    // the label gap counts whenever the kind label is visible (the original tests it twice)
    let w = tw + nw + LABEL_INTERVAL + PAD + PAD + h / 2.0;
    grow(r, v, (w + FRAME_CONTENT_MIN_W).max(FRAME_MIN_W), (h + FRAME_CONTENT_MIN_H).max(FRAME_MIN_H));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    set_box(r, tl, x + PAD, y + PAD, tw, size);
    let nx = if tw > 0.0 { x + PAD + tw + LABEL_INTERVAL * 2.0 } else { x + PAD };
    set_box(r, nl, nx, y + PAD, nw, size);
}

const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;

pub(crate) fn edge_label(r: &mut Repo, edge: Eid, model: Option<Eid>, alpha: f64, distance: f64, pos: i64, visible_null: bool) -> Eid {
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
    // a BPMN flow's name 15 px off its middle; the other BPMN lines have no label (elements.js:935-968)
    if ty.starts_with("BPMN") {
        crate::bpmn::edge_labels(r, e, m);
        return Some(e);
    }
    // a flow has only its name, 15 px off its middle (FCGeneralEdgeView, flowchart/elements.js:197-215)
    // and so does a data flow (DFDGeneralEdgeView, dfd/elements.js:335-350); a mind map edge has no
    // label (MMEdgeView, mindmap/elements.js:103-115)
    if ty == "FCFlowView" || ty == "DFDDataFlowView" || crate::cloud::is_edge_view(ty) {
        let nl = edge_label(r, e, Some(m), HALF_PI, 15.0, 1, false);
        r.set_ref(e, "nameLabel", Some(nl));
        return Some(e);
    }
    if ty == "MMEdgeView" {
        return Some(e);
    }
    // a C4 relationship: its description 15 px off its middle on one side, its "[technology]" on the
    // other (C4GeneralEdgeView, c4/elements.js:250-275)
    if ty == "C4RelationshipView" {
        for (field, a) in [("descriptionLabel", HALF_PI), ("technologyLabel", -HALF_PI)] {
            let l = edge_label(r, e, Some(m), a, 15.0, 1, false);
            r.set(l, "wordWrap", json!(true));
            r.set_ref(e, field, Some(l));
        }
        return Some(e);
    }
    // a relationship's name by its middle, and its ends' names by its ends (ERDRelationshipView,
    // erd/elements.js:470-500)
    if ty == "ERDRelationshipView" {
        let p6 = std::f64::consts::FRAC_PI_6;
        for (field, a, dist, pos) in [("nameLabel", HALF_PI, 15.0, 1), ("tailNameLabel", p6, 30.0, 2), ("headNameLabel", -p6, 30.0, 0)] {
            let l = edge_label(r, e, Some(m), a, dist, pos, false);
            r.set_ref(e, field, Some(l));
        }
        return Some(e);
    }
    r.set(e, "showVisibility", json!(true));
    let nl = edge_label(r, e, Some(m), HALF_PI, 15.0, 1, false);
    let sl = edge_label(r, e, Some(m), HALF_PI, 30.0, 1, true);
    let pl = edge_label(r, e, Some(m), -HALF_PI, 15.0, 1, false);
    r.set_ref(e, "nameLabel", Some(nl));
    r.set_ref(e, "stereotypeLabel", Some(sl));
    r.set_ref(e, "propertyLabel", Some(pl));
    // a role binding names the role near its head (UMLRoleBindingView, elements.js:7289-7300)
    if r.ty(m) == "UMLRoleBinding" {
        let l = edge_label(r, e, Some(m), -std::f64::consts::FRAC_PI_6, 30.0, 0, false);
        r.set_ref(e, "roleNameLabel", Some(l));
    }
    // an exception handler or activity interrupt: the zig-zag mark 20 px off the middle of the line
    // (UMLZigZagAdornmentView, elements.js:10537-10560)
    if matches!(ty, "UMLExceptionHandlerView" | "UMLActivityInterruptView") {
        let z = r.create("UMLZigZagAdornmentView", &[("model", r.ref_to(m)), ("font", json!(FONT)), ("parentStyle", json!(true))], Some((e, "subViews")));
        r.set(z, "width", json!(ZIGZAG_W));
        r.set(z, "height", json!(ZIGZAG_H));
        r.set(z, "alpha", json!(HALF_PI));
        r.set(z, "distance", json!(20));
        r.set_ref(z, "hostEdge", Some(e));
        r.set(z, "edgePosition", json!(1));
        r.set_ref(e, "adornment", Some(z));
    }
    // associations and links have end labels; only associations have qualifiers
    if r.kind_of(m, "UMLAssociation") || r.ty(m) == "UMLLink" || r.kind_of(m, "UMLConnector") {
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
        let quals = if r.kind_of(m, "UMLAssociation") { vec![("tailQualifiersCompartment", end1), ("headQualifiersCompartment", end2)] } else { vec![] };
        for (field, end) in quals {
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
    // a SysML connector's item flow labels, 15 px off its middle on each side (SysMLConnectorView,
    // sysml/elements.js:1610-1630)
    if ty == "SysMLConnectorView" {
        for (field, a) in [("itemFlowToHeadLabel", HALF_PI), ("itemFlowToTailLabel", -HALF_PI)] {
            let l = edge_label(r, e, Some(m), a, 15.0, 1, false);
            r.set_ref(e, field, Some(l));
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

/// An extension point's line: its name, then ": location" when it has one (estimated).
pub fn extension_point_text(r: &Repo, x: Eid) -> String {
    let loc = r.str(x, "location");
    if loc.is_empty() { r.name(x) } else { format!("{}: {loc}", r.name(x)) }
}

/// A reception's line: its stereotype (or «signal»), visibility and name (UML 2; the exact form is
/// estimated for StarUML).
pub fn reception_text(r: &Repo, x: Eid, show: Show) -> String {
    let st = stereotype_text(r, x);
    let mut s = if st.is_empty() { "«signal»".to_string() } else { st };
    s.push(' ');
    if show.vis {
        s += vis_symbol(&r.str(x, "visibility"));
    }
    s + &r.name(x)
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
    /// "interface", "actor", "usecase", "package", "subject", "frame", "hyperlink", "nary", "collaboration" or ""
    pub shape: &'static str,
    pub compartments: Vec<Comp>,
    pub is_active: bool,
}

pub(crate) fn set_box(r: &mut Repo, v: Eid, x: f64, y: f64, w: f64, h: f64) {
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
    /// shown whatever showNamespace says: an action's called operation or an object node's states
    force_ns: bool,
}

/// The namespace line an activity box shows in place of "(from …)": a named action calling an
/// operation shows "(Class::operation)", an object node in states shows "[s1, s2]"
/// (UMLActionView.update, UMLObjectNodeView.update, elements.js:9917-9925, 10232-10241).
fn activity_ns(r: &Repo, m: Eid) -> Option<String> {
    if r.kind_of(m, "UMLAction") && r.str(m, "kind") == "callOperation" && !r.name(m).trim().is_empty() {
        let op = r.refd(m, "target").filter(|t| r.ty(*t) == "UMLOperation")?;
        let owner = r.parent(op).map(|p| r.name(p)).unwrap_or_default();
        return Some(format!("({owner}::{})", r.name(op)));
    }
    if r.kind_of(m, "UMLObjectNode") {
        let states: Vec<String> = r.ref_list(m, "inStates").into_iter().map(|x| r.name(x)).collect();
        if !states.is_empty() {
            return Some(format!("[{}]", states.join(", ")));
        }
    }
    None
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
    // the label forms show the stereotype: Label, Decoration with Label, Icon with Label
    // (UMLGeneralNodeView.sizeObject, elements.js:4860-4883)
    let show_st = !st_text.is_empty() && (display == "label" || display == "decoration-label" || display == "icon-label") && !hide_st;
    let parent = r.parent(m).map(|p| r.name(p)).unwrap_or_default();
    let forced = activity_ns(r, m);
    Some(NameParts {
        nc,
        st: r.refd(nc, "stereotypeLabel"),
        nm: r.refd(nc, "nameLabel"),
        ns: r.refd(nc, "namespaceLabel"),
        st_text,
        show_st,
        force_ns: forced.is_some(),
        ns_text: forced.unwrap_or_else(|| format!("(from {parent})")),
    })
}

/// The text of a box's name: an object (an instance) also shows its classifier, "name : Class".
pub fn display_name(r: &Repo, m: Eid) -> String {
    let name = r.name(m);
    if !r.kind_of(m, "UMLInstance") {
        return name;
    }
    match r.refd(m, "classifier") {
        Some(c) => format!("{name} : {}", r.name(c)),
        None => name,
    }
}

/// The name an activity box shows (UMLActionView.update, UMLObjectNodeView.update,
/// elements.js:9912-9934, 10232-10241): an action that calls an activity shows "name:Activity", one
/// calling a behaviour with no name of its own shows the behaviour's name, and an object node shows
/// "name: Type". Other boxes show their display name.
pub(crate) fn box_name(r: &Repo, m: Eid) -> String {
    let name = r.name(m);
    if r.kind_of(m, "UMLAction") {
        if let Some(sub) = r.refd(m, "subactivity").filter(|s| r.ty(*s) == "UMLActivity") {
            return format!("{name}:{}", r.name(sub));
        }
        // calling an operation without a name of its own: the operation's name
        if r.str(m, "kind") == "callOperation" && name.trim().is_empty() {
            if let Some(t) = r.refd(m, "target").filter(|t| r.ty(*t) == "UMLOperation") {
                return r.name(t);
            }
        }
        if r.str(m, "kind") == "callBehavior" && name.trim().is_empty() {
            if let Some(t) = r.refd(m, "target").filter(|t| r.kind_of(*t, "UMLBehavior")) {
                return r.name(t);
            }
        }
        return name;
    }
    // an unnamed activity parameter node shows its parameter (UMLActivityParameterNodeView.update,
    // elements.js:10272-10283)
    if r.ty(m) == "UMLActivityParameterNode" && name.trim().is_empty() {
        if let Some(p) = r.refd(m, "parameter").filter(|p| r.alive(*p)) {
            return attribute_text(r, p, Show { vis: false, mult: true, sig: true, ty: true, prop: true });
        }
    }
    // a lifeline: "name[selector]: Type", its type that of the role it represents
    // (UMLLifeline.getString, elements.js:3790-3800)
    if r.ty(m) == "UMLLifeline" {
        let sel = r.str(m, "selector");
        let mut s = if sel.is_empty() { name } else { format!("{name}[{sel}]") };
        if let Some(role) = r.refd(m, "represent") {
            let ty = type_name(r, role, "type");
            if !ty.is_empty() {
                s += &format!(": {ty}");
            }
        }
        return s;
    }
    // a submachine state: "name: Machine" (UMLStateView.update, elements.js:9502-9505)
    if r.ty(m) == "UMLState" {
        if let Some(sm) = r.refd(m, "submachine").filter(|s| r.alive(*s)) {
            return format!("{name}: {}", r.name(sm));
        }
    }
    if r.kind_of(m, "UMLObjectNode") {
        let ty = match r.refd(m, "type") {
            Some(t) => r.name(t),
            None => r.str(m, "type"),
        };
        return if ty.is_empty() { name } else { format!("{name}: {ty}") };
    }
    display_name(r, m)
}

fn name_style(r: &Repo, m: Eid) -> u8 {
    if r.bool(m, "isAbstract") { 3 } else { 1 }
}

pub(crate) fn layout_name(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid, x: f64, y: f64, w: f64, keyword: Option<&str>, hide_st: bool) -> f64 {
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
        let name = box_name(r, m);
        r.set(nm, "text", json!(name));
        set_label_font(r, ms, nm, Some(style as i64));
        // an object's name is underlined (UML instance notation)
        if r.kind_of(m, "UMLInstance") {
            r.set(nm, "underline", json!(true));
        }
        items.push(nm);
    }
    if let Some(ns) = p.ns {
        set_label_font(r, ms, ns, None);
        let show = r.bool(v, "showNamespace") || p.force_ns;
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

pub(crate) fn name_min_width(r: &Repo, ms: &dyn Measure, v: Eid, m: Eid, keyword: Option<&str>, hide_st: bool) -> f64 {
    let wrap = r.bool(v, "wordWrap");
    let width = |t: &str, style: u8| if wrap { widest_word(ms, t, style) } else { ms.width(t, style) };
    let mut w = width(&box_name(r, m), name_style(r, m));
    if let Some(p) = name_parts(r, v, m, keyword, hide_st) {
        if p.show_st {
            w = w.max(width(&p.st_text, 0));
        }
        if p.force_ns {
            w = w.max(width(&p.ns_text, 0));
        }
    }
    w.ceil() + 2.0 * PAD
}

/// Keep a list compartment's item views in step with the model list; returns (min width, height, rows).
pub(crate) fn sync_list(r: &mut Repo, ms: &dyn Measure, comp: Eid, items: &[Eid], item_ty: &str, texts: &[String], wrap: bool) -> (f64, f64, Vec<Row>) {
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
    let mut shown = vec![];
    for (i, iv) in keep.iter().enumerate() {
        // an item hidden by Delete keeps its view, which neither shows nor takes room
        // (UMLAttributeView.canHide and the compartments' size, elements.js:4426, 4700-4720)
        if r.raw(*iv, "visible") == Some(&json!(false)) {
            continue;
        }
        shown.push(*iv);
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
    let rows: Vec<Row> = shown.into_iter().map(|iv| Row { iv }).collect();
    let h = list_height(r, ms, &rows, f64::INFINITY);
    (min_w, h, rows)
}

/// The height of a list compartment `w` wide: its rows, with word-wrapped rows on several lines.
pub(crate) fn list_height(r: &Repo, ms: &dyn Measure, rows: &[Row], w: f64) -> f64 {
    if rows.is_empty() {
        return EMPTY_H;
    }
    let lines: usize = rows.iter().map(|row| label_lines(r, ms, row.iv, w - 2.0 * PAD + 1.0)).sum();
    2.0 * PAD + lines as f64 * ms.size() + ITEM_GAP * (rows.len() as f64 - 1.0)
}

pub(crate) fn place_list(r: &mut Repo, ms: &dyn Measure, comp: Eid, x: f64, y: f64, w: f64, h: f64, rows: &[Row]) {
    set_box(r, comp, x, y, w + 1.0, h);
    let mut cy = y + PAD;
    for row in rows {
        let lh = label_lines(r, ms, row.iv, w - 2.0 * PAD + 1.0) as f64 * ms.size();
        set_box(r, row.iv, x + PAD, cy, w - 2.0 * PAD + 1.0, lh);
        cy += lh + ITEM_GAP;
    }
}

pub(crate) fn hide_comp(r: &mut Repo, comp: Option<Eid>) {
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
        "UMLInformationItem" => Some("information"),
        "UMLStereotype" => Some("stereotype"),
        "UMLMetaClass" => Some("metaclass"),
        // central buffers and data stores (elements.js:10254-10272)
        "UMLCentralBufferNode" => Some("centralBuffer"),
        "UMLDataStoreNode" => Some("datastore"),
        _ => None,
    }
}

/// Grow a box to its content; with autoResize on it takes exactly its content size (core.js:2115-2125).
pub(crate) fn grow(r: &mut Repo, v: Eid, min_w: f64, min_h: f64) {
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
        // the attributes and operations, when not suppressed, follow under the name without lines
        // (UMLActorView.sizeAsIconicForm, elements.js:8628-8636; suppressed by default, 8614-8621)
        let show = Show::of(r, v);
        let mut comps: Vec<(Eid, f64, f64, Vec<Row>)> = vec![];
        for (field, flag, list) in [("attributeCompartment", "suppressAttributes", "attributes"), ("operationCompartment", "suppressOperations", "operations")] {
            let Some(c) = r.refd(v, field).filter(|_| !r.bool(v, flag)) else { continue };
            r.remove(c, "visible");
            let items = r.list(m, list);
            let texts: Vec<String> = items.iter().map(|a| if list == "attributes" { attribute_text(r, *a, show) } else { operation_text(r, *a, show) }).collect();
            let (cw, ch, rows) = sync_list(r, ms, c, &items, if list == "attributes" { "UMLAttributeView" } else { "UMLOperationView" }, &texts, false);
            comps.push((c, cw, ch, rows));
        }
        let comps_h: f64 = comps.iter().map(|c| c.2).sum();
        let min_w = comps.iter().map(|c| c.1).fold(name_min_width(r, ms, v, m, None, false).max(ACTOR_ICON_W), f64::max);
        grow(r, v, min_w, NAME_MIN_H + ACTOR_ICON_H + comps_h);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        let ny = y + h - NAME_MIN_H - comps_h;
        layout_name(r, ms, v, m, x, ny, w, None, false);
        let mut cy = ny + NAME_MIN_H;
        let mut out = vec![];
        for (c, _, ch, rows) in comps {
            place_list(r, ms, c, x, cy, w, ch, &rows);
            out.push(Comp { y: cy, rows });
            cy += ch;
        }
        return Info { shape: "actor", compartments: out, ..Default::default() };
    }
    if ty == "UMLUseCase" {
        hide_all(r);
        // extension points, listed under the name inside the ellipse (layout estimated)
        let items = r.list(m, "extensionPoints");
        let ext = match r.refd(v, "extensionPointCompartment") {
            Some(xc) if !items.is_empty() => {
                r.remove(xc, "visible");
                let texts: Vec<String> = items.iter().map(|x| extension_point_text(r, *x)).collect();
                let (w, h, rows) = sync_list(r, ms, xc, &items, "UMLExtensionPointView", &texts, false);
                Some((xc, w, h, rows))
            }
            _ => None,
        };
        let nw = name_min_width(r, ms, v, m, None, false).max(ext.as_ref().map(|e| e.1).unwrap_or(0.0));
        let inner_h = NAME_MIN_H + ext.as_ref().map(|e| e.2).unwrap_or(0.0);
        let min_w = (std::f64::consts::SQRT_2 * nw).floor().max(UC_MIN_W);
        let min_h = (std::f64::consts::SQRT_2 * inner_h).floor().max(UC_MIN_H);
        grow(r, v, min_w, min_h);
        let (x, y, w, h) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"), r.f(v, "height"));
        let cw = (w / std::f64::consts::SQRT_2).floor();
        let ny = y + (h - inner_h) / 2.0;
        layout_name(r, ms, v, m, x + (w - cw) / 2.0, ny, cw, None, false);
        if let Some(nc) = r.refd(v, "nameCompartment") {
            r.set(nc, "width", num(cw));
        }
        let mut compartments = vec![];
        if let Some((xc, _, eh, rows)) = ext {
            let cy = ny + NAME_MIN_H;
            place_list(r, ms, xc, x + (w - cw) / 2.0, cy, cw, eh, &rows);
            compartments.push(Comp { y: cy, rows });
        }
        return Info { shape: "usecase", compartments, ..Default::default() };
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
    // receptions, under the operations; shown only when not suppressed, which is the default
    if let Some(rc) = r.refd(v, "receptionCompartment") {
        if r.bool(v, "suppressReceptions") {
            hide_comp(r, Some(rc));
        } else {
            r.remove(rc, "visible");
            let items = r.list(m, "receptions");
            let texts: Vec<String> = items.iter().map(|x| reception_text(r, *x, show)).collect();
            let (w, h, rows) = sync_list(r, ms, rc, &items, "UMLReceptionView", &texts, wrap);
            comps.push((rc, w, h, rows));
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
    // a profile stereotype's icon: above the compartments (Icon, Icon with Label), or a 24 × 24
    // decoration at the top right (elements.js:4894-4917, 4955-4980)
    let icon = stereotype_icon(r, m).filter(|_| ty != "UMLInterface");
    let iconic = icon.is_some() && (display == "icon" || display == "icon-label");
    let decorated = icon.is_some() && (display == "decoration" || display == "decoration-label");
    let hide_st = iconic && display == "icon";
    let mut min_w = name_min_width(r, ms, v, m, keyword, hide_st) + icon_room(r.ty(v));
    if decorated {
        min_w += DECORATION_ICON + 2.0 * PAD;
    }
    if let (true, Some((_, iw, _))) = (iconic, &icon) {
        min_w = min_w.max(*iw);
    }
    for c in &comps {
        min_w = min_w.max(c.1);
    }
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let w0 = if r.bool(v, "autoResize") { min_w } else { r.f(v, "width").max(min_w) };
    // a decoration narrows the name by its box (arrangeAsDecorationForm, elements.js:4955-4963)
    let name_w = if decorated { w0 - DECORATION_ICON } else { w0 };
    let name_h = layout_name(r, ms, v, m, x, y, name_w, keyword, hide_st);
    if wrap {
        for c in comps.iter_mut() {
            c.2 = list_height(r, ms, &c.3, w0);
        }
    }
    let comps_h = name_h + comps.iter().map(|c| c.2).sum::<f64>();
    let icon_min_h = match (&icon, iconic) {
        (Some((_, _, ih)), true) => ih.max(ICONIC_MIN),
        _ => 0.0,
    };
    grow(r, v, min_w, comps_h + icon_min_h);
    let w = r.f(v, "width");
    // the icon takes the square that fits above the compartments; they follow under it
    let icon_h = if iconic {
        let area = R { x, y, w, h: r.f(v, "height") - comps_h };
        let ih = crate::render::fit_ratio(area, 100.0, 100.0).h;
        layout_name(r, ms, v, m, x, y + ih, w0, keyword, hide_st);
        ih
    } else {
        0.0
    };
    let mut cy = y + icon_h + name_h;
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
    Info { shape: if iconic { "stereotype-icon" } else { "" }, compartments: out, is_active: r.bool(m, "isActive"), templ }
}

/// A slot's line: "name: type = value" (the defining feature's name when the slot has none; estimated).
pub fn slot_text(r: &Repo, s: Eid) -> String {
    let mut name = r.name(s);
    if name.is_empty() {
        name = r.refd(s, "definingFeature").map(|f| r.name(f)).unwrap_or_default();
    }
    let ty = type_name(r, s, "type");
    if !ty.is_empty() {
        name = format!("{name}: {ty}");
    }
    let value = r.str(s, "value");
    if value.is_empty() { name } else { format!("{name} = {value}") }
}

/// The width kept free on each side of the name for the icon drawn at a box's top right (components
/// and artifacts, and their instances), so a long name does not run under it (estimated).
pub const ICON_ROOM: f64 = 26.0;

fn icon_room(view_ty: &str) -> f64 {
    match view_ty {
        "UMLComponentView" | "UMLArtifactView" | "UMLComponentInstanceView" | "UMLArtifactInstanceView" => 2.0 * ICON_ROOM,
        _ => 0.0,
    }
}

/// An object: the name compartment, then the slots (box form, like a class).
fn layout_object(r: &mut Repo, ms: &dyn Measure, v: Eid, m: Eid) -> Info {
    let wrap = r.bool(v, "wordWrap");
    let mut comps: Vec<(Eid, f64, f64, Vec<Row>)> = vec![];
    if let Some(sc) = r.refd(v, "slotCompartment") {
        r.remove(sc, "visible");
        let items = r.list(m, "slots");
        let texts: Vec<String> = items.iter().map(|s| slot_text(r, *s)).collect();
        let (w, h, rows) = sync_list(r, ms, sc, &items, "UMLSlotView", &texts, wrap);
        comps.push((sc, w, h, rows));
    }
    let mut min_w = name_min_width(r, ms, v, m, None, false) + icon_room(r.ty(v));
    for c in &comps {
        min_w = min_w.max(c.1);
    }
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let w0 = if r.bool(v, "autoResize") { min_w } else { r.f(v, "width").max(min_w) };
    let name_h = layout_name(r, ms, v, m, x, y, w0, None, false);
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
    Info { shape: "", compartments: out, ..Default::default() }
}

fn layout_node(r: &mut Repo, ms: &dyn Measure, v: Eid) -> Info {
    let (face, size) = view_font(r, v);
    let scaled = Scaled { ms, face, size };
    let ms: &dyn Measure = &scaled;
    let t = r.ty(v).to_string();
    let m = r.refd(v, "model");
    if crate::flowchart::is_node_view(&t) {
        crate::flowchart::layout(r, ms, v);
        return Info { shape: "flowchart", ..Default::default() };
    }
    if t == "ERDEntityView" {
        return crate::erd::layout(r, ms, v);
    }
    if crate::bpmn::is_node_view(&t) {
        crate::bpmn_draw::layout(r, ms, v);
        return Info { shape: "bpmn", ..Default::default() };
    }
    if matches!(t.as_str(), "UMLCombinedFragmentView" | "UMLContinuationView" | "UMLInteractionUseView") {
        crate::sequence::layout_fragment(r, ms, v);
        return Info { shape: "fragment", ..Default::default() };
    }
    if t == "UMLTimeConstraintView" {
        crate::sequence::layout_time(r, ms, v);
        return Info { shape: "time-constraint", ..Default::default() };
    }
    if t == "UMLDurationConstraintView" {
        crate::sequence::layout_duration(r, ms, v);
        return Info { shape: "duration-constraint", ..Default::default() };
    }
    if t == "UMLCommLifelineView" {
        crate::communication::layout_lifeline(r, ms, v);
        return Info { shape: "comm-lifeline", ..Default::default() };
    }
    // a communication message is placed by its connector, once the lines are laid out
    if t == "UMLCommMessageView" {
        return Info { shape: "comm-message", ..Default::default() };
    }
    if t == "UMLSeqLifelineView" {
        let invariants: Vec<Eid> = r.list(v, "subViews").into_iter().filter(|x| r.ty(*x) == "UMLStateInvariantView").collect();
        for iv in invariants {
            crate::sequence::layout_invariant(r, ms, v, iv);
        }
        crate::sequence::layout_lifeline(r, ms, v);
        return Info { shape: "lifeline", ..Default::default() };
    }
    if t == "UMLEndpointView" || t == "UMLGateView" {
        r.set(v, "width", json!(crate::sequence::ENDPOINT_SIZE));
        r.set(v, "height", json!(crate::sequence::ENDPOINT_SIZE));
        return Info { shape: "endpoint", ..Default::default() };
    }
    if crate::c4::is_node_view(&t) {
        crate::c4::layout(r, ms, v);
        return Info { shape: "c4", ..Default::default() };
    }
    if crate::sysml::is_view(&t) {
        return crate::sysml::layout(r, ms, v);
    }
    if crate::notations::is_node_view(&t) {
        crate::notations::layout(r, ms, v);
        return Info { shape: "notation", ..Default::default() };
    }
    if crate::wireframe::is_node_view(&t) {
        crate::wireframe::layout(r, ms, v);
        return Info { shape: "wireframe", ..Default::default() };
    }
    if crate::cloud::is_node_view(&t) {
        crate::cloud::layout(r, ms, v);
        return Info { shape: "cloud", ..Default::default() };
    }
    if let Some(m) = m {
        if let Some(k) = node_kind(r.ty(m)) {
            if k.classifier && k.view == t {
                return layout_classifier(r, ms, v, m);
            }
        }
    }
    // a metaclass: a box with «metaclass» and its name (estimated)
    if t == "UMLMetaClassView" {
        if let Some(m) = m {
            let min_w = name_min_width(r, ms, v, m, Some("metaclass"), false);
            grow(r, v, min_w, NAME_MIN_H);
            let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
            let nh = layout_name(r, ms, v, m, x, y, w, Some("metaclass"), false);
            grow(r, v, min_w, nh);
        }
        return Info::default();
    }
    if t == "UMLObjectView" || t.ends_with("InstanceView") {
        if let Some(m) = m {
            return layout_object(r, ms, v, m);
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
    // a constraint: "{specification}", the box at least as wide as the text (estimated)
    if t == "UMLConstraintView" {
        if let Some(m) = m {
            let text = format!("{{{}}}", r.str(m, "specification"));
            let w = ms.width(&text, 0) + 10.0;
            r.set(v, "text", json!(text));
            grow(r, v, w.ceil(), 20.0);
        }
        return Info::default();
    }
    if t == "UMLInputPinView" || t == "UMLOutputPinView" || t == "UMLExpansionNodeView" {
        if let Some(m) = m {
            layout_pin(r, ms, v, m);
        }
        return Info { shape: "pin", ..Default::default() };
    }
    if t == "UMLPortView" {
        if let Some(m) = m {
            layout_port(r, ms, v, m);
        }
        return Info { shape: "port", ..Default::default() };
    }
    if t == "UMLPartView" {
        if let Some(m) = m {
            layout_part(r, ms, v, m);
        }
        return Info { shape: "part", ..Default::default() };
    }
    if t == "UMLNaryAssociationNodeView" {
        if let Some(m) = m {
            layout_nary(r, ms, v, m);
        }
        return Info { shape: "nary", ..Default::default() };
    }
    if t == "HyperlinkView" {
        layout_hyperlink(r, ms, v);
        return Info { shape: "hyperlink", ..Default::default() };
    }
    if t == "UMLTimingLifelineView" || t == "UMLTimingStateView" || t == "UMLTimeSegmentView" || t == "UMLTimeTickView" {
        // placed by their frame (crate::timing::layout)
        let shape = match t.as_str() {
            "UMLTimingLifelineView" => "timing-lifeline",
            "UMLTimingStateView" => "timing-state",
            "UMLTimeSegmentView" => "time-segment",
            _ => "time-tick",
        };
        return Info { shape, ..Default::default() };
    }
    if t == "UMLFrameView" || t == "UMLInteractionInlineView" || t == "UMLTimingFrameView" {
        layout_frame(r, ms, v);
        return Info { shape: "frame", ..Default::default() };
    }
    if t == "UMLStateView" {
        if let Some(m) = m {
            layout_state(r, ms, v, m);
        }
        return Info { shape: "state", ..Default::default() };
    }
    if t == "UMLPseudostateView" || t == "UMLFinalStateView" {
        // freely sizable marks with a minimum size by kind; fork and join bars keep their thickness
        // (UMLPseudostateView.sizeObject, UMLFinalStateView.sizeObject, elements.js:8842-8910, 9118-9123)
        let kind = m.map(|m| r.str(m, "kind")).unwrap_or_default();
        if t == "UMLFinalStateView" {
            grow(r, v, FINAL_SIZE, FINAL_SIZE);
        } else if kind == "fork" || kind == "join" {
            let (w, h) = (r.f(v, "width"), r.f(v, "height"));
            if h > w {
                r.set(v, "width", num(BAR_THICK));
                grow(r, v, BAR_THICK, BAR_LENGTH);
            } else {
                r.set(v, "height", num(BAR_THICK));
                grow(r, v, BAR_LENGTH, BAR_THICK);
            }
        } else {
            let (w, h) = pseudostate_min(&kind);
            grow(r, v, w, h);
        }
        // its floating labels stay hidden (UMLPseudostateView.arrange, elements.js:8912-8916); files
        // from older UMLCraft versions have a name compartment instead
        for f in ["nameLabel", "stereotypeLabel"] {
            if let Some(l) = r.refd(v, f) {
                r.set_raw(l, "visible", json!(false));
            }
        }
        if let Some(nc) = r.refd(v, "nameCompartment") {
            r.set_raw(nc, "visible", json!(false));
        }
        return Info { shape: if t == "UMLFinalStateView" { "final" } else { "pseudostate" }, ..Default::default() };
    }
    if t == "UMLConnectionPointReferenceView" {
        if let Some(m) = m {
            layout_connection_point(r, ms, v, m);
        }
        return Info { shape: "connection-point", ..Default::default() };
    }
    if t == "UMLStructuredActivityNodeView" || t == "UMLExpansionRegionView" {
        if let Some(m) = m {
            layout_structured(r, ms, v, m);
        }
        return Info { shape: "structured", ..Default::default() };
    }
    if t == "UMLInterruptibleActivityRegionView" {
        grow(r, v, REGION_MIN, REGION_MIN);
        return Info { shape: "interruptible", ..Default::default() };
    }
    if t == "UMLActivityEdgeConnectorView" {
        grow(r, v, EDGE_CONNECTOR_SIZE, EDGE_CONNECTOR_SIZE);
        return Info { shape: "edge-connector", ..Default::default() };
    }
    if t == "UMLActionView" || t == "UMLObjectNodeView" || t == "UMLCentralBufferNodeView" || t == "UMLDataStoreNodeView" || t == "UMLActivityParameterNodeView" {
        if let Some(m) = m {
            layout_activity_box(r, ms, v, m);
        }
        return Info { shape: if t == "UMLActionView" { "action" } else { "object-node" }, ..Default::default() };
    }
    if t == "UMLSwimlaneView" {
        if let Some(m) = m {
            layout_swimlane(r, ms, v, m);
        }
        return Info { shape: "swimlane", ..Default::default() };
    }
    if t == "UMLControlNodeView" {
        if let Some(m) = m {
            layout_control_node(r, v, m);
        }
        return Info { shape: "control-node", ..Default::default() };
    }
    if t == "UMLCollaborationView" || t == "UMLCollaborationUseView" {
        if let Some(m) = m {
            layout_collaboration(r, ms, v, m);
        }
        return Info { shape: "collaboration", ..Default::default() };
    }
    if t == "UMLUseCaseSubjectView" {
        if let Some(m) = m {
            // at least as big as its name (UMLGeneralNodeView.sizeObject)
            let min_w = name_min_width(r, ms, v, m, None, false);
            let (x, y, w) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width").max(min_w));
            let nh = layout_name(r, ms, v, m, x, y, w, None, false);
            grow(r, v, min_w, nh);
        }
        return Info { shape: "subject", ..Default::default() };
    }
    Info::default()
}

// ------------------------------------------------------------------ edges
pub(crate) fn center(r: &R) -> P {
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

/// The middle of a line: its middle point, or the middle of its central segment when it has an even
/// number of points (as edge labels are anchored, diagram-editor.js:2071-2097).
pub fn edge_middle(pts: &[P]) -> P {
    match pts.len() {
        0 => P::default(),
        n if n % 2 == 1 => pts[n / 2],
        n => P { x: (pts[n / 2 - 1].x + pts[n / 2].x) / 2.0, y: (pts[n / 2 - 1].y + pts[n / 2].y) / 2.0 },
    }
}

fn end_rect(r: &Repo, v: Eid, info: &HashMap<Eid, Info>) -> R {
    if is_edge(r, v) {
        let mid = edge_middle(&points_of(r, v));
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
/// Move point `i` of a rectilinear path to `to`; each neighbour keeps the segment it shares with the
/// point vertical or horizontal by following on that axis (diagram-editor.js:1887-1922). A neighbour
/// on neither axis is left where it is.
pub fn move_orthogonal(pts: &mut [P], i: usize, to: P) {
    let from = pts[i];
    for j in [i.wrapping_sub(1), i + 1] {
        let Some(q) = pts.get_mut(j) else { continue };
        if (q.x - from.x).abs() < 0.5 && (q.y - from.y).abs() >= 0.5 {
            q.x = to.x;
        } else if (q.y - from.y).abs() < 0.5 && (q.x - from.x).abs() >= 0.5 {
            q.y = to.y;
        }
    }
    pts[i] = to;
}

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
    layout_qualifiers(r, ms, e, &mut pts, tr, hr);
    r.set(e, "points", json!(format_points(&pts)));
    layout_edge_labels(r, ms, e, &pts);
}

/// The qualifiers of an association end: a box of their rows against the end's box, on the side the
/// line arrives at, and the line then ends on the box's outer side. Shown only when the end has
/// qualifiers (UMLQualifierCompartmentView; the placement is estimated).
pub fn qualifier_rows(r: &Repo, q: Eid) -> Vec<String> {
    let Some(end) = r.refd(q, "model") else { return vec![] };
    r.list(end, "qualifiers").into_iter().filter(|a| r.alive(*a)).map(|a| attribute_text(r, a, Show::ALL)).collect()
}

fn layout_qualifiers(r: &mut Repo, ms: &dyn Measure, e: Eid, pts: &mut [P], tr: R, hr: R) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let (face, size) = view_font(r, e);
    let scaled = Scaled { ms, face, size };
    for (field, i, b) in [("tailQualifiersCompartment", 0, tr), ("headQualifiersCompartment", n - 1, hr)] {
        let Some(q) = r.refd(e, field) else { continue };
        let rows = qualifier_rows(r, q);
        if rows.is_empty() {
            r.set_raw(q, "visible", json!(false));
            continue;
        }
        let lh = scaled.size();
        let w = (rows.iter().map(|t| scaled.width(t, 0)).fold(0.0, f64::max) + 2.0 * PAD).ceil();
        let h = (rows.len() as f64 * lh + 2.0 * PAD).ceil();
        let p = pts[i];
        let sides = [(p.x - b.x).abs(), (p.x - b.x - b.w).abs(), (p.y - b.y).abs(), (p.y - b.y - b.h).abs()];
        let side = (0..4).min_by(|a, c| sides[*a].total_cmp(&sides[*c])).unwrap_or(0);
        let (cy, cx) = ((p.y - h / 2.0).clamp(b.y, (b.y + b.h - h).max(b.y)), (p.x - w / 2.0).clamp(b.x, (b.x + b.w - w).max(b.x)));
        let (bx, by, end) = match side {
            0 => (b.x - w, cy, P { x: b.x - w, y: p.y }),
            1 => (b.x + b.w, cy, P { x: b.x + b.w + w, y: p.y }),
            2 => (cx, b.y - h, P { x: p.x, y: b.y - h }),
            _ => (cx, b.y + b.h, P { x: p.x, y: b.y + b.h + h }),
        };
        r.remove(q, "visible");
        r.set(q, "left", num(bx));
        r.set(q, "top", num(by));
        r.set(q, "width", num(w));
        r.set(q, "height", num(h));
        pts[i] = end;
    }
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

/// A transition's label: `trigger1, trigger2 [guard] / effect` (uml-transition.pegjs), each part
/// only when present.
pub fn transition_text(r: &Repo, m: Eid) -> String {
    let triggers: Vec<String> = r.list(m, "triggers").into_iter().filter(|t| r.alive(*t)).map(|t| r.name(t)).collect();
    let mut s = triggers.join(", ");
    let guard = r.str(m, "guard");
    if !guard.is_empty() {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&format!("[{guard}]"));
    }
    if let Some(e) = r.list(m, "effects").into_iter().find(|e| r.alive(*e)) {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&format!("/ {}", r.name(e)));
    }
    s
}

/// A control or object flow's label: `name [guard] {weight=w}`, each part only when present
/// (UMLActivityEdge.getString, elements.js:3555-3567).
pub fn activity_edge_text(r: &Repo, m: Eid) -> String {
    let mut s = r.name(m);
    let guard = r.str(m, "guard");
    if !guard.is_empty() {
        s.push_str(&format!(" [{guard}]"));
    }
    let weight = r.str(m, "weight");
    if !weight.is_empty() {
        s.push_str(&format!(" {{weight={weight}}}"));
    }
    s
}

pub(crate) fn edge_label_texts(r: &Repo, e: Eid) -> Vec<(&'static str, String)> {
    let Some(m) = r.refd(e, "model") else { return vec![] };
    let show_vis = r.bool(e, "showVisibility");
    let vis = |x: Eid| if show_vis { vis_symbol(&r.str(x, "visibility")).to_string() } else { String::new() };
    let mut out = vec![];
    let name = r.name(m);
    // a derived association shows "/" before its name (StarUML 5.x)
    let derived = if r.kind_of(m, "UMLAssociation") && r.bool(m, "isDerived") { "/" } else { "" };
    if r.ty(m) == "FCFlow" || r.ty(m) == "DFDDataFlow" || crate::cloud::edge_view_of(r.ty(m)).is_some() {
        return vec![("nameLabel", name)];
    }
    if r.ty(m) == "MMEdge" {
        return vec![];
    }
    if r.ty(m).starts_with("BPMN") {
        return crate::bpmn::edge_label_texts(r, m);
    }
    if r.ty(m) == "UMLMessage" {
        let show_type = r.raw(e, "showType").and_then(|v| v.as_bool()).unwrap_or(true);
        let text = r.parent(e).map(|d| crate::sequence::message_text(r, d, m, show_type)).unwrap_or_default();
        let st = match r.str(m, "messageSort").as_str() {
            "createMessage" => "«create»".to_string(),
            "deleteMessage" => "«destroy»".to_string(),
            _ => stereotype_text(r, m),
        };
        return vec![("nameLabel", text), ("stereotypeLabel", st), ("propertyLabel", String::new())];
    }
    if r.ty(m) == "C4Relationship" {
        let tech = r.str(m, "technology");
        return vec![("descriptionLabel", r.str(m, "description")), ("technologyLabel", if tech.is_empty() { tech } else { format!("[{tech}]") })];
    }
    if r.ty(m) == "ERDRelationship" {
        let end = |k: &str| r.child(m, k).map(|e| r.name(e)).unwrap_or_default();
        return vec![("nameLabel", name), ("tailNameLabel", end("end1")), ("headNameLabel", end("end2"))];
    }
    if r.ty(m) == "UMLTransition" {
        out.push(("nameLabel", transition_text(r, m)));
    } else if r.kind_of(m, "UMLActivityEdge") {
        out.push(("nameLabel", activity_edge_text(r, m)));
    } else {
        out.push(("nameLabel", if name.is_empty() && derived.is_empty() { name } else { vis(m) + derived + &name }));
    }
    let st = match r.ty(m) {
        "UMLInclude" => "«include»".to_string(),
        // a deployment is a «deploy» dependency (UML 2; the keyword is estimated for StarUML)
        "UMLDeployment" if stereotype_text(r, m).is_empty() => "«deploy»".to_string(),
        "UMLExtend" => "«extend»".to_string(),
        // a template binding shows the «bind» keyword (UML 2; estimated for StarUML)
        "UMLTemplateBinding" if stereotype_text(r, m).is_empty() => "«bind»".to_string(),
        // an information flow shows «flow» (UML 2; estimated for StarUML)
        "UMLInformationFlow" if stereotype_text(r, m).is_empty() => "«flow»".to_string(),
        // a SysML relationship shows its keyword (getStereotypeString, sysml/elements.js:259-375)
        t if crate::sysml::keyword(t).is_some() && stereotype_text(r, m).is_empty() => crate::sysml::keyword(t).unwrap_or_default().to_string(),
        _ => stereotype_text(r, m),
    };
    out.push(("stereotypeLabel", st));
    out.push(("propertyLabel", String::new()));
    if r.ty(m) == "UMLRoleBinding" {
        out.push(("roleNameLabel", r.str(m, "roleName")));
    }
    if r.kind_of(m, "UMLAssociation") || r.ty(m) == "UMLLink" || r.kind_of(m, "UMLConnector") {
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
    if r.ty(e) == "SysMLConnectorView" {
        out.push(("itemFlowToHeadLabel", crate::sysml::item_flow_text(r, m, true)));
        out.push(("itemFlowToTailLabel", crate::sysml::item_flow_text(r, m, false)));
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

pub const ZIGZAG_W: f64 = 25.0;
pub const ZIGZAG_H: f64 = 20.0;

pub(crate) fn layout_edge_labels(r: &mut Repo, ms: &dyn Measure, e: Eid, pts: &[P]) {
    if pts.len() < 2 {
        return;
    }
    if let Some(z) = r.refd(e, "adornment").filter(|z| r.alive(*z)) {
        r.set(z, "width", json!(ZIGZAG_W));
        r.set(z, "height", json!(ZIGZAG_H));
        let (p1, p2) = label_anchor(r, z, pts);
        let d = point_away_line(p1, p2, r.f(z, "alpha"), r.f(z, "distance"));
        r.set(z, "left", num(d.x + p1.x - (ZIGZAG_W / 2.0).floor()));
        r.set(z, "top", num(d.y + p1.y - (ZIGZAG_H / 2.0).floor()));
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
        // at least as wide as its text, or its widest word with word wrap; a label widened by hand
        // keeps its width, and wraps to it (EdgeLabelView.sizeObject, core.js:3779-3800)
        let wrap = r.bool(lv, "wordWrap");
        let min_w = if t.is_empty() { 0.0 } else if wrap { widest_word(ms, &t, 0) } else { ms.width(&t, 0) };
        let w = if t.is_empty() { 0.0 } else { r.f(lv, "width").max(min_w) };
        let lh = if wrap && !t.is_empty() { wrap_text(ms, &t, w, 0).len().max(1) as f64 * ms.size() } else { ms.size() };
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
    if r.ty(d) == "UMLTimingDiagram" {
        crate::timing::layout(r, ms, d);
    }
    // a UML box grows over the contained views that extend their container, with the compartment
    // padding around them (UMLGeneralNodeView.delimitContainingBoundary, elements.js:4789-4808)
    for v in &views {
        if r.alive(*v) && is_node(r, *v) && r.ty(*v).starts_with("UML") {
            let kids: Vec<Eid> = views.iter().copied().filter(|c| r.alive(*c) && r.refd(*c, "containerView") == Some(*v) && r.bool(*c, "containerExtending")).collect();
            if kids.is_empty() {
                continue;
            }
            let b = node_rect(r, *v);
            let (mut x1, mut y1, mut x2, mut y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
            for c in kids {
                let k = node_rect(r, c);
                x1 = x1.min(k.x - PAD);
                y1 = y1.min(k.y - PAD);
                x2 = x2.max(k.x + k.w + PAD);
                y2 = y2.max(k.y + k.h + PAD);
            }
            set_box(r, *v, x1, y1, x2 - x1, y2 - y1);
        }
    }
    for v in &views {
        // only line kinds UMLCraft knows are re-routed; others keep the points they were saved with
        if r.alive(*v) && (KNOWN_EDGE_VIEWS.contains(&r.ty(*v)) || crate::bpmn::is_edge_view(r.ty(*v)) || crate::sysml::is_edge_view(r.ty(*v))) && is_edge(r, *v) {
            layout_edge(r, ms, *v, &info);
            // what a cloud line's update sets on its view (its colour by kind)
            if crate::cloud::is_edge_view(r.ty(*v)) {
                crate::cloud::update_edge(r, *v);
            }
        }
    }
    if r.ty(d) == "UMLSequenceDiagram" {
        crate::sequence::layout(r, ms, d);
        // a time constraint's link follows its message, placed by the sequence layout
        for v in &views {
            if r.alive(*v) && r.ty(*v) == "UMLTimeConstraintLinkView" {
                layout_edge(r, ms, *v, &info);
            }
        }
    }
    if r.ty(d) == "UMLInteractionOverviewDiagram" {
        crate::overview::layout_inlines(r, ms, d, &mut info);
    }
    if r.ty(d) == "UMLTimingDiagram" {
        crate::timing::layout_messages(r, ms, d);
        for v in &views {
            if r.alive(*v) && r.ty(*v) == "UMLTimeConstraintLinkView" {
                layout_edge(r, ms, *v, &info);
                // from a time segment, the link starts at its middle height (UMLTimeConstraintLinkView.arrangeObject)
                if let Some(t) = r.refd(*v, "tail").filter(|t| r.ty(*t) == "UMLTimeSegmentView") {
                    let mut pts = points_of(r, *v);
                    if let Some(p) = pts.first_mut() {
                        p.y = (r.f(t, "top") + r.f(t, "height") / 2.0).round();
                    }
                    r.set(*v, "points", json!(format_points(&pts)));
                }
            }
        }
    }
    for v in &views {
        if r.alive(*v) && r.ty(*v) == "UMLCommMessageView" {
            crate::communication::layout_message(r, ms, *v);
        }
    }
    info
}

/// How a box may be resized: every handle, only left and right, only top and bottom, or not at all
/// (NodeView.sizable: SZ_FREE and SZ_RATIO, SZ_HORZ, SZ_VERT, SZ_NONE; core.js NodeView and each
/// view's constructor).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sizable {
    Free,
    Horz,
    Vert,
    None,
}

/// The sizable mode of a view (the constructors and update() of the views that set one).
/// The compartment items a click selects on their own (UMLAttributeView… set selectable to SK_YES,
/// sizable to SZ_NONE and movable to MM_NONE, elements.js:4386-4392 and the like).
pub const ITEM_VIEWS: [&str; 7] = ["UMLAttributeView", "UMLOperationView", "UMLReceptionView", "UMLTemplateParameterView", "UMLEnumerationLiteralView", "UMLSlotView", "UMLExtensionPointView"];

pub fn is_item_view(ty: &str) -> bool {
    ITEM_VIEWS.contains(&ty)
}

/// Whether Delete may remove the view alone: not labels and parasitic views (core.js:3537, 3584),
/// compartment items (elements.js:4435 and the like) or the interaction views that ask to go with
/// their models (canDelete in elements.js:10593-13884).
pub fn can_delete(r: &Repo, v: Eid) -> bool {
    let t = r.ty(v);
    !(is_item_view(t)
        || r.kind_of(v, "LabelView")
        || matches!(
            t,
            "NodeLabelView" | "EdgeLabelView" | "UMLZigZagAdornmentView" | "UMLSeqLifelineView" | "UMLMessageEndpointView" | "UMLActivationView" | "UMLSeqMessageView" | "UMLStateInvariantView" | "UMLContinuationView"
                | "UMLInteractionOperandView" | "UMLCombinedFragmentView" | "UMLInteractionUseView" | "UMLTimeConstraintLinkView" | "UMLDurationConstraintView" | "UMLCommLifelineView" | "UMLCommMessageView"
                | "UMLTimingFrameView" | "UMLTimingStateView" | "UMLTimingLifelineView" | "UMLTimeSegmentView" | "UMLTimingMessageView" | "UMLInteractionInlineView"
        ))
}

/// Whether the view can be copied: not labels, parasitic views, compartment items, nor the
/// interaction parts that only make sense with their models (canCopy, core.js:3527, 3574;
/// elements.js:10593-13884).
pub fn can_copy(r: &Repo, v: Eid) -> bool {
    let t = r.ty(v);
    !(is_item_view(t)
        || r.kind_of(v, "LabelView")
        || matches!(
            t,
            "NodeLabelView" | "EdgeLabelView" | "UMLZigZagAdornmentView" | "UMLSeqLifelineView" | "UMLMessageEndpointView" | "UMLActivationView" | "UMLSeqMessageView" | "UMLStateInvariantView" | "UMLContinuationView"
                | "UMLInteractionOperandView" | "UMLCombinedFragmentView" | "UMLInteractionUseView" | "UMLTimeConstraintLinkView" | "UMLDurationConstraintView" | "UMLCommLifelineView" | "UMLCommMessageView"
                | "UMLTimingStateView" | "UMLTimingLifelineView" | "UMLTimeSegmentView" | "UMLTimeTickView" | "UMLTimingMessageView" | "UMLInteractionInlineView"
        ))
}

/// Whether Delete hides the view instead (the compartment items' canHide, elements.js:4426, 4499,
/// 4565, 4621, 6179, 7393, 8319).
pub fn can_hide(r: &Repo, v: Eid) -> bool {
    is_item_view(r.ty(v))
}

pub fn sizable(r: &Repo, v: Eid) -> Sizable {
    let vertical = || r.refd(v, "model").map(|m| r.bool(m, "isVertical")).unwrap_or(false);
    let timing = || r.parent(v).map(|d| r.ty(d) == "UMLTimingDiagram").unwrap_or(false);
    if is_item_view(r.ty(v)) {
        return Sizable::None;
    }
    match r.ty(v) {
        "UMLPortView" | "UMLPinView" | "UMLInputPinView" | "UMLOutputPinView" | "UMLConnectionPointReferenceView" | "UMLEndpointView" | "UMLGateView" | "UMLCommMessageView" | "UMLTimeConstraintView"
        | "UMLTimingStateView" | "UMLZigZagAdornmentView" | "WFSwitchView" | "ERDColumnView" => Sizable::None,
        t if crate::sysml::floating(t) => Sizable::None,
        "UMLTimeSegmentView" | "UMLTimeTickView" | "WFSliderView" | "EdgeLabelView" | "NodeLabelView" => Sizable::Horz,
        "UMLActivationView" | "UMLLinePartView" | "UMLRegionView" | "UMLInteractionOperandView" => Sizable::Vert,
        "UMLDurationConstraintView" => if timing() { Sizable::Horz } else { Sizable::Vert },
        "WFSeparatorView" => if vertical() { Sizable::Vert } else { Sizable::Horz },
        _ => Sizable::Free,
    }
}

/// Whether a view moves across and down (NodeView.movable: MM_FREE, MM_HORZ, MM_VERT, MM_NONE).
pub fn movable(r: &Repo, v: Eid) -> (bool, bool) {
    if is_item_view(r.ty(v)) {
        return (false, false);
    }
    match r.ty(v) {
        "UMLSeqLifelineView" | "UMLTimeTickView" => (true, false),
        "UMLTimingLifelineView" | "UMLTimingStateView" | "UMLStateInvariantView" => (false, true),
        "ERDColumnView" | "UMLActivationView" | "UMLLinePartView" | "UMLRegionView" | "UMLInteractionOperandView" => (false, false),
        _ => (true, true),
    }
}

/// Whether a box shows resize handle `i` (0 top left, then clockwise to 7 left) in its sizable
/// mode (diagram-editor.js:1411-1454).
pub fn handle_shown(s: Sizable, i: usize) -> bool {
    match s {
        Sizable::Free => true,
        Sizable::Horz => i == 3 || i == 7,
        Sizable::Vert => i == 1 || i == 5,
        Sizable::None => false,
    }
}
