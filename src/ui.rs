//! The window: menu bar, Working Diagrams, Toolbox, canvas, Model Explorer, Editors and status bar.
//! Colours and sizes measured from the original's window (MECHANICS.md "Window").

use crate::app::{App, Area, AssocInit, Tool};
use crate::canvas::EguiMeasure;
use crate::dialogs::dialog;
use crate::icons;
use crate::model::Eid;
use crate::render;
use crate::views::{self, R};
use egui::{Align2, Color32, CornerRadius, FontId, Frame, Margin, Panel, Pos2, Rect, Sense, Stroke, Vec2};
use serde_json::{json, Value};

pub const WINDOW_WIDTH: f32 = 1200.0; // window.js:38
pub const WINDOW_HEIGHT: f32 = 900.0; // window.js:39
pub const SIDEBAR_WIDTH: f32 = 200.0;
pub const NAVIGATOR_WIDTH: f32 = 300.0;
pub const TOOLBAR_WIDTH: f32 = 30.0;
pub const STATUSBAR_HEIGHT: f32 = 27.0;
pub const TOOLBOX_HEIGHT: f32 = 400.0;
pub const EDITORS_HEIGHT: f32 = 400.0;
pub const HEADER_HEIGHT: f32 = 34.0;
pub const SCREENSHOT_FRAMES: u32 = 120;
/// Help › Documentation and Release Notes open these pages.
pub const DOCS_URL: &str = "https://github.com/BurkhalterY/umlcraft#readme";
pub const RELEASES_URL: &str = "https://github.com/BurkhalterY/umlcraft/releases";
/// Help > Forum: UMLCraft has no forum, so its issues on GitHub (help:forum, default-commands.js:1164-1166).
pub const FORUM_URL: &str = "https://github.com/BurkhalterY/umlcraft/issues";
/// The grab handle a hidden sidebar or navigator leaves at the window edge (resizer.js:238-258).
const GRIP_HIDDEN: f32 = 4.0;

static LIGHT_THEME: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Switch the interface colours between the dark theme (the default) and the light one.
pub fn set_light_theme(on: bool) {
    LIGHT_THEME.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn light_theme() -> bool {
    LIGHT_THEME.load(std::sync::atomic::Ordering::Relaxed)
}

/// An interface colour: its dark-theme and light-theme values.
macro_rules! themed {
    ($name:ident, $dark:expr, $light:expr) => {
        #[allow(non_snake_case)]
        pub fn $name() -> Color32 {
            if light_theme() { $light } else { $dark }
        }
    };
}
themed!(LAYER0, Color32::from_rgb(0x3a, 0x3f, 0x41), Color32::from_rgb(0xf2, 0xf2, 0xf2));
themed!(LAYER1, Color32::from_rgb(0x4a, 0x4c, 0x4e), Color32::from_rgb(0xe6, 0xe6, 0xe6));
themed!(TOOLBAR, Color32::from_rgb(0x5d, 0x5f, 0x60), Color32::from_rgb(0xd9, 0xd9, 0xd9));
themed!(TEXT, Color32::from_rgb(0xe3, 0xe3, 0xe3), Color32::from_rgb(0x24, 0x24, 0x24));
themed!(DIM, Color32::from_rgb(0xb4, 0xb6, 0xb7), Color32::from_rgb(0x68, 0x6a, 0x6c));
themed!(BORDER, Color32::from_rgb(0x56, 0x59, 0x5b), Color32::from_rgb(0xcc, 0xcc, 0xcc));
themed!(INPUT, Color32::from_rgb(0x40, 0x42, 0x44), Color32::from_rgb(0xff, 0xff, 0xff));
themed!(INPUT_BORDER, Color32::from_rgb(0x63, 0x66, 0x68), Color32::from_rgb(0xb8, 0xb8, 0xb8));
themed!(ACCENT, Color32::from_rgb(0x1f, 0x7a, 0xe8), Color32::from_rgb(0x1f, 0x7a, 0xe8));
themed!(ACCENT_TEXT, Color32::from_rgb(0x4f, 0xa6, 0xff), Color32::from_rgb(0x0b, 0x5f, 0xc4));
themed!(CURRENT_ROW, Color32::from_rgb(0x33, 0x35, 0x3c), Color32::from_rgb(0xd4, 0xe3, 0xf7));
themed!(SELECTED_ROW, Color32::from_rgb(0x2d, 0x32, 0x38), Color32::from_rgb(0xd4, 0xe3, 0xf7));
themed!(ROW_HOVER, Color32::from_rgb(0x33, 0x36, 0x3a), Color32::from_rgb(0xe2, 0xe2, 0xe2));
themed!(TOOL_HOVER, Color32::from_rgb(0x6d, 0x70, 0x71), Color32::from_rgb(0xc4, 0xc4, 0xc4));
themed!(HOVER_ROW, Color32::from_rgb(0x55, 0x58, 0x5a), Color32::from_rgb(0xdc, 0xdc, 0xdc));
pub const OK_GREEN: Color32 = Color32::from_rgb(0x8d, 0xc6, 0x3f);

const fn node(label: &'static str, kind: &'static str, icon: &'static str) -> Tool {
    Tool { label, kind, icon, line: false, init: AssocInit::Plain }
}
const fn line(label: &'static str, kind: &'static str, icon: &'static str, init: AssocInit) -> Tool {
    Tool { label, kind, icon, line: true, init }
}

#[derive(Clone, Copy)]
pub struct Group {
    pub id: &'static str,
    pub label: &'static str,
    pub items: &'static [Tool],
}

// toolbox groups and tools: extensions/essential/uml/toolbox/uml.json
const ANNOTATIONS: Group = Group {
    id: "annotations",
    label: "Annotations",
    items: &[
        node("Text", "Text", "text"),
        node("Text Box", "TextBox", "textbox"),
        line("Free Line", "FreeLine", "freeline", AssocInit::Plain),
        node("Note", "Note", "note"),
        line("Note Link", "NoteLink", "notelink", AssocInit::Plain),
        node("Hyperlink", "Hyperlink", "hyperlink"),
        node("Rectangle", "Rect", "rect"),
        node("Rounded Rectangle", "RoundRect", "roundrect"),
        node("Ellipse", "Ellipse", "ellipse"),
        node("Image", "Image", "picture"),
    ],
};
// the Composite Structure group (uml.json "uml.compositestructure": class, composite structure and
// component diagrams)
const COMPOSITE: Group = Group {
    id: "compositestructure",
    label: "Composite Structure",
    items: &[
        node("Collaboration", "UMLCollaboration", "collaboration"),
        node("Port", "UMLPort", "port"),
        node("Part", "UMLPart", "part"),
        line("Connector", "UMLConnector", "association", AssocInit::Plain),
        node("Collaboration Use", "UMLCollaborationUse", "collaboration"),
        line("Role Binding", "UMLRoleBinding", "dependency", AssocInit::Plain),
        line("Realization", "UMLRealization", "realization", AssocInit::Plain),
    ],
};
const CLASS_GROUPS: &[Group] = &[
    Group {
        id: "class",
        label: "Classes (Basic)",
        items: &[
            node("Class", "UMLClass", "class"),
            node("Interface", "UMLInterface", "interface"),
            line("Association", "UMLAssociation", "association", AssocInit::Plain),
            line("Directed Association", "UMLAssociation", "directed", AssocInit::Directed),
            line("Aggregation", "UMLAssociation", "aggregation", AssocInit::Shared),
            line("Composition", "UMLAssociation", "composition", AssocInit::Composite),
            line("Dependency", "UMLDependency", "dependency", AssocInit::Plain),
            line("Generalization", "UMLGeneralization", "generalization", AssocInit::Plain),
            line("Interface Realization", "UMLInterfaceRealization", "realization", AssocInit::Plain),
        ],
    },
    Group {
        id: "class-adv",
        label: "Classes (Advanced)",
        items: &[
            node("Signal", "UMLSignal", "datatype"),
            node("DataType", "UMLDataType", "datatype"),
            node("PrimitiveType", "UMLPrimitiveType", "datatype"),
            node("Enumeration", "UMLEnumeration", "enumeration"),
            node("Frame", "Frame", "frame"),
            node("N-ary Association Node", "UMLNaryAssociationNode", "nary"),
            line("Association Class", "AssociationClass", "associationclass", AssocInit::Plain),
            line("Template Binding", "UMLTemplateBinding", "dependency", AssocInit::Plain),
            line("Realization", "UMLRealization", "realization", AssocInit::Plain),
        ],
    },
    Group {
        id: "package",
        label: "Packages",
        items: &[node("Package", "UMLPackage", "package"), node("Model", "UMLModel", "model"), node("Subsystem", "UMLSubsystem", "package"), line("Dependency", "UMLDependency", "dependency", AssocInit::Plain), line("Containment", "Containment", "containment", AssocInit::Plain)],
    },
    COMPOSITE,
    INSTANCES,
    // the robustness extension's group (robustness/toolbox/robustness.json)
    Group { id: "robustness", label: "Robustness", items: &[node("Boundary", "Boundary", "boundary"), node("Entity", "Entity", "entity"), node("Control", "Control", "control")] },
    ANNOTATIONS,
];
const USECASE_GROUPS: &[Group] = &[
    Group {
        id: "usecase",
        label: "Use Cases",
        items: &[
            node("Package", "UMLPackage", "package"),
            node("Use Case Subject", "UMLUseCaseSubject", "subject"),
            node("Use Case", "UMLUseCase", "usecase"),
            node("Actor", "UMLActor", "actor"),
            node("Frame", "Frame", "frame"),
            line("Association", "UMLAssociation", "association", AssocInit::Plain),
            line("Directed Association", "UMLAssociation", "directed", AssocInit::Directed),
            line("Generalization", "UMLGeneralization", "generalization", AssocInit::Plain),
            line("Dependency", "UMLDependency", "dependency", AssocInit::Plain),
            line("Include", "UMLInclude", "include", AssocInit::Plain),
            line("Extend", "UMLExtend", "include", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
const OTHER_GROUPS: &[Group] = &[ANNOTATIONS];
// a data flow diagram (dfd/toolbox/dfd.json) and a mind map (mindmap/toolbox/mindmap.json)
const DFD_GROUPS: &[Group] = &[
    Group {
        id: "dfd",
        label: "Data Flow",
        items: &[
            node("External Entity", "DFDExternalEntity", "DFDExternalEntity"),
            node("Process", "DFDProcess", "DFDProcess"),
            node("Data Store", "DFDDataStore", "DFDDataStore"),
            line("Data Flow", "DFDDataFlow", "DFDDataFlow", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
const MINDMAP_GROUPS: &[Group] = &[
    Group { id: "mindmap", label: "Mindmap", items: &[node("Node", "MMNode", "MMNode"), line("Edge", "MMEdge", "MMEdge", AssocInit::Plain)] },
    ANNOTATIONS,
];
// a sequence diagram (uml.json "uml.sequence", "uml.sequence-adv"): the tools UMLCraft has
const SEQUENCE_GROUPS: &[Group] = &[
    Group {
        id: "sequence",
        label: "Interactions (Basic)",
        items: &[
            node("Lifeline", "UMLLifeline", "lifeline"),
            line("Message", "UMLMessage", "message", AssocInit::Plain),
            line("Self Message", "UMLSelfMessage", "self-message", AssocInit::Plain),
            line("Async Message", "UMLAsyncMessage", "async-message", AssocInit::Plain),
            line("Reply Message", "UMLReplyMessage", "reply-message", AssocInit::Plain),
            line("Create Message", "UMLCreateMessage", "reply-message", AssocInit::Plain),
            line("Delete Message", "UMLDeleteMessage", "message", AssocInit::Plain),
            line("Async Signal Message", "UMLAsyncSignalMessage", "async-message", AssocInit::Plain),
        ],
    },
    Group {
        id: "sequence-adv",
        label: "Interactions (Advanced)",
        items: &[
            line("Found Message", "UMLFoundMessage", "found-message", AssocInit::Plain),
            line("Lost Message", "UMLLostMessage", "lost-message", AssocInit::Plain),
            node("Endpoint", "UMLEndpoint", "endpoint"),
            node("Gate", "UMLGate", "gate"),
            node("State Invariant", "UMLStateInvariant", "state"),
            node("Continuation", "UMLContinuation", "state"),
            node("Combined Fragment", "UMLCombinedFragment", "frame"),
            node("Interaction Use", "UMLInteractionUse", "frame"),
            node("Time Constraint", "UMLTimeConstraint", "time-constraint"),
            node("Duration Constraint", "UMLDurationConstraint", "duration-constraint"),
            node("Frame", "Frame", "frame"),
        ],
    },
    ANNOTATIONS,
];
// a wireframe diagram (wireframe/toolbox/wireframe.json)
const WIREFRAME_GROUPS: &[Group] = &[
    Group {
        id: "wireframe.frames",
        label: "Wireframe Frames",
        items: &[
            node("Frame", "WFFrame", "WFFrame"),
            node("Frame (Mobile)", "WFMobileFrame", "WFMobileFrame"),
            node("Frame (Web)", "WFWebFrame", "WFWebFrame"),
            node("Frame (Desktop)", "WFDesktopFrame", "WFDesktopFrame"),
        ],
    },
    Group {
        id: "wireframe.components",
        label: "Wireframe Components",
        items: &[
            node("Button", "WFButton", "WFButton"),
            node("Text", "WFText", "WFText"),
            node("Radio", "WFRadio", "WFRadio"),
            node("Checkbox", "WFCheckbox", "WFCheckbox"),
            node("Switch", "WFSwitch", "WFSwitch"),
            node("Link", "WFLink", "WFLink"),
            node("Tab List", "WFTabList", "WFTabList"),
            node("Tab", "WFTab", "WFTab"),
            node("Input", "WFInput", "WFInput"),
            node("Dropdown", "WFDropdown", "WFDropdown"),
            node("Panel", "WFPanel", "WFPanel"),
            node("Image", "WFImage", "WFImage"),
            node("Separator", "WFSeparator", "WFSeparator"),
            node("Avatar", "WFAvatar", "WFAvatar"),
            node("Slider", "WFSlider", "WFSlider"),
        ],
    },
    ANNOTATIONS,
];
// a communication diagram (uml.json "uml.communication")
const COMMUNICATION_GROUPS: &[Group] = &[
    Group {
        id: "communication",
        label: "Communication",
        items: &[
            node("Lifeline", "UMLLifeline", "object"),
            line("Connector", "UMLConnector", "association", AssocInit::Plain),
            line("Self Connector", "UMLSelfConnector", "self-connector", AssocInit::Plain),
            line("Forward Message", "UMLForwardMessage", "forward-message", AssocInit::Plain),
            line("Reverse Message", "UMLReverseMessage", "reverse-message", AssocInit::Plain),
            node("Frame", "Frame", "frame"),
        ],
    },
    ANNOTATIONS,
];
// a timing diagram (uml.json "uml.timing")
const TIMING_GROUPS: &[Group] = &[
    Group {
        id: "timing",
        label: "Timing",
        items: &[
            node("Lifeline", "UMLLifeline", "timing-lifeline"),
            node("State/Condition", "UMLTimingState", "state-condition"),
            node("Time Segment", "UMLTimeSegment", "time-segment"),
            node("Time Tick", "UMLTimeTick", "time-tick"),
            node("Time Constraint", "UMLTimeConstraint", "time-constraint"),
            node("Duration Constraint", "UMLDurationConstraint", "duration-constraint"),
            line("Message", "UMLMessage", "message", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// an interaction overview diagram (uml.json "uml.interaction-overview")
const OVERVIEW_GROUPS: &[Group] = &[
    Group {
        id: "interaction-overview",
        label: "Interaction Overview",
        items: &[
            node("Interaction Use", "UMLInteractionUseInOverview", "frame"),
            node("Interaction (Inline)", "UMLInteractionInOverview", "frame"),
            node("Initial", "UMLInitialNode", "initial"),
            node("Final", "UMLActivityFinalNode", "final"),
            node("Fork", "UMLForkNode", "fork"),
            node("Join", "UMLJoinNode", "fork"),
            node("Merge", "UMLMergeNode", "choice"),
            node("Decision", "UMLDecisionNode", "choice"),
            line("Control Flow", "UMLControlFlow", "dependency", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// a BPMN diagram (bpmn/toolbox/bpmn.json); the node tools take the model type as icon
const BPMN_GROUPS: &[Group] = &[
    Group {
        id: "bpmn.basic",
        label: "BPMN Basic",
        items: &[
            node("Task", "BPMNTask", "BPMNTask"),
            node("Gateway", "BPMNExclusiveGateway", "BPMNExclusiveGateway"),
            node("Start Event", "BPMNStartEvent", "BPMNStartEvent"),
            node("End Event", "BPMNEndEvent", "BPMNEndEvent"),
            line("Sequence Flow", "BPMNSequenceFlow", "BPMNSequenceFlow", AssocInit::Plain),
        ],
    },
    Group {
        id: "bpmn.artifact",
        label: "BPMN Artifact",
        items: &[node("Text Annotation", "BPMNTextAnnotation", "BPMNTextAnnotation"), node("Group", "BPMNGroup", "BPMNGroup"), line("Association", "BPMNAssociation", "BPMNAssociation", AssocInit::Plain)],
    },
    Group {
        id: "bpmn.swimlanes",
        label: "BPMN Swimlanes",
        items: &[node("Pool", "BPMNParticipant", "BPMNParticipant"), node("Pool (Vert)", "BPMNParticipantVert", "BPMNParticipantVert"), node("Lane", "BPMNLane", "BPMNLane"), node("Lane (Vert)", "BPMNLaneVert", "BPMNLaneVert")],
    },
    Group {
        id: "bpmn.activities",
        label: "BPMN Activities",
        items: &[
            node("Task", "BPMNTask", "BPMNTask"),
            node("Send Task", "BPMNSendTask", "BPMNSendTask"),
            node("Receive Task", "BPMNReceiveTask", "BPMNReceiveTask"),
            node("Service Task", "BPMNServiceTask", "BPMNServiceTask"),
            node("User Task", "BPMNUserTask", "BPMNUserTask"),
            node("Manual Task", "BPMNManualTask", "BPMNManualTask"),
            node("Business Rule Task", "BPMNBusinessRuleTask", "BPMNBusinessRuleTask"),
            node("Script Task", "BPMNScriptTask", "BPMNScriptTask"),
            node("Call Activity", "BPMNCallActivity", "BPMNCallActivity"),
            node("Sub Process", "BPMNSubProcess", "BPMNSubProcess"),
            node("Ad-Hoc Sub Process", "BPMNAdHocSubProcess", "BPMNAdHocSubProcess"),
            node("Transaction", "BPMNTransaction", "BPMNTransaction"),
        ],
    },
    Group {
        id: "bpmn.gateways",
        label: "BPMN Gateways",
        items: &[
            node("Gateway (Exclusive)", "BPMNExclusiveGateway", "BPMNExclusiveGateway"),
            node("Gateway (Event-Based)", "BPMNEventBasedGateway", "BPMNEventBasedGateway"),
            node("Gateway (Inclusive)", "BPMNInclusiveGateway", "BPMNInclusiveGateway"),
            node("Gateway (Complex)", "BPMNComplexGateway", "BPMNComplexGateway"),
            node("Gateway (Parallel)", "BPMNParallelGateway", "BPMNParallelGateway"),
        ],
    },
    Group {
        id: "bpmn.events",
        label: "BPMN Events",
        items: &[
            node("Start Event", "BPMNStartEvent", "BPMNStartEvent"),
            node("Intermediate Event (Throw)", "BPMNIntermediateThrowEvent", "BPMNIntermediateEvent"),
            node("Intermediate Event (Catch)", "BPMNIntermediateCatchEvent", "BPMNIntermediateEvent"),
            node("Boundary Event", "BPMNBoundaryEvent", "BPMNBoundaryEvent"),
            node("End Event", "BPMNEndEvent", "BPMNEndEvent"),
        ],
    },
    Group {
        id: "bpmn.data",
        label: "BPMN Data",
        items: &[
            node("Data Object", "BPMNDataObject", "BPMNDataObject"),
            node("Data Store", "BPMNDataStore", "BPMNDataStore"),
            node("Data Input", "BPMNDataInput", "BPMNDataInput"),
            node("Data Output", "BPMNDataOutput", "BPMNDataOutput"),
            line("Data Association", "BPMNDataAssociation", "BPMNDataAssociation", AssocInit::Plain),
        ],
    },
    Group { id: "bpmn.message", label: "BPMN Message", items: &[node("Message", "BPMNMessage", "BPMNMessage"), line("Message Flow", "BPMNMessageFlow", "BPMNMessageFlow", AssocInit::Plain)] },
    Group {
        id: "bpmn.choreography",
        label: "BPMN Choreography",
        items: &[node("Choreography Task", "BPMNChoreographyTask", "BPMNChoreographyTask"), node("Sub Choreography", "BPMNSubChoreography", "BPMNSubChoreography"), line("Message Link", "BPMNMessageLink", "BPMNMessageLink", AssocInit::Plain)],
    },
    Group {
        id: "bpmn.conversation",
        label: "BPMN Conversation",
        items: &[
            node("Conversation", "BPMNConversation", "BPMNConversation"),
            node("Sub Conversation", "BPMNSubConversation", "BPMNSubConversation"),
            node("Call Conversation", "BPMNCallConversation", "BPMNCallConversation"),
            line("Conversation Link", "BPMNConversationLink", "BPMNConversationLink", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// an AWS architecture diagram (aws/toolbox/aws.json)
const AWS_GROUPS: &[Group] = &[
    Group {
        id: "aws.elements",
        label: "AWS",
        items: &[
            node("AWS Group", "AWSGroup", "AWSGroup"),
            node("AWS Generic Group", "AWSGenericGroup", "AWSGenericGroup"),
            node("AWS Availability Zone", "AWSAvailabilityZone", "AWSAvailabilityZone"),
            node("AWS Security Group", "AWSSecurityGroup", "AWSSecurityGroup"),
            node("AWS Service", "AWSService", "AWSService"),
            node("AWS Resource", "AWSResource", "AWSResource"),
            node("AWS General Resource", "AWSGeneralResource", "AWSGeneralResource"),
            node("AWS Callout", "AWSCallout", "AWSCallout"),
            line("AWS Arrow", "AWSArrow", "AWSArrow", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// an Azure architecture diagram (azure/toolbox/azure.json)
const AZURE_GROUPS: &[Group] = &[
    Group {
        id: "azure.elements",
        label: "Azure",
        items: &[
            node("Azure Group", "AzureGroup", "AzureGroup"),
            node("Azure Service", "AzureService", "AzureService"),
            node("Azure Callout", "AzureCallout", "AzureCallout"),
            line("Azure Connector", "AzureConnector", "AzureConnector", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// a GCP architecture diagram (gcp/toolbox/gcp.json)
const GCP_GROUPS: &[Group] = &[
    Group {
        id: "gcp.elements",
        label: "GCP",
        items: &[
            node("Zone", "GCPZone", "GCPZone"),
            node("User", "GCPUser", "GCPUser"),
            node("Product", "GCPProduct", "GCPProduct"),
            node("Service", "GCPService", "GCPService"),
            line("Path", "GCPPath", "GCPPath", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// a C4 diagram (c4/toolbox/c4.json)
const C4_GROUPS: &[Group] = &[
    Group {
        id: "c4",
        label: "C4 Model",
        items: &[
            node("Person", "C4Person", "C4Person"),
            node("Software System", "C4SoftwareSystem", "C4SoftwareSystem"),
            node("Container", "C4Container", "C4Container"),
            node("Container (Database)", "C4ContainerDatabase", "C4ContainerDatabase"),
            node("Container (Web App)", "C4ContainerWebApp", "C4Container"),
            node("Container (Desktop App)", "C4ContainerDesktopApp", "C4Container"),
            node("Container (Mobile App)", "C4ContainerMobileApp", "C4Container"),
            node("Component", "C4Component", "C4Component"),
            node("Element", "C4Element", "C4Element"),
            line("Relationship", "C4Relationship", "C4Relationship", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// an ER diagram (erd/toolbox/erd.json)
const ERD_GROUPS: &[Group] = &[
    Group {
        id: "erd",
        label: "Entity-Relationship",
        items: &[
            node("Entity", "ERDEntity", "ERDEntity"),
            line("One to One Relationship", "ERDRelationship", "ERDRelationship", AssocInit::Plain),
            line("One to Many Relationship", "ERDRelationshipOneToMany", "ERDRelationshipOneToMany", AssocInit::Plain),
            line("Many to Many Relationship", "ERDRelationshipManyToMany", "ERDRelationshipManyToMany", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// a flowchart diagram (flowchart/toolbox/flowchart.json)
const FLOWCHART_GROUPS: &[Group] = &[
    Group {
        id: "flowchart",
        label: "Flowchart",
        items: &[
            line("Flow", "FCFlow", "FCFlow", AssocInit::Plain),
            node("Process", "FCProcess", "FCProcess"),
            node("Terminator", "FCTerminator", "FCTerminator"),
            node("Decision", "FCDecision", "FCDecision"),
            node("Delay", "FCDelay", "FCDelay"),
            node("Predefined Process", "FCPredefinedProcess", "FCPredefinedProcess"),
            node("Alternate Process", "FCAlternateProcess", "FCAlternateProcess"),
            node("Data", "FCData", "FCData"),
            node("Document", "FCDocument", "FCDocument"),
            node("Multi-Document", "FCMultiDocument", "FCMultiDocument"),
            node("Preparation", "FCPreparation", "FCPreparation"),
            node("Display", "FCDisplay", "FCDisplay"),
            node("Manual Input", "FCManualInput", "FCManualInput"),
            node("Manual Operation", "FCManualOperation", "FCManualOperation"),
            node("Card", "FCCard", "FCCard"),
            node("Punched Tape", "FCPunchedTape", "FCPunchedTape"),
            node("Connector", "FCConnector", "FCConnector"),
            node("Off-Page Connector", "FCOffPageConnector", "FCOffPageConnector"),
            node("Or", "FCOr", "FCOr"),
            node("Summing Junction", "FCSummingJunction", "FCSummingJunction"),
            node("Collate", "FCCollate", "FCCollate"),
            node("Sort", "FCSort", "FCSort"),
            node("Merge", "FCMerge", "FCMerge"),
            node("Extract", "FCExtract", "FCExtract"),
            node("Stored Data", "FCStoredData", "FCStoredData"),
            node("Database", "FCDatabase", "FCDatabase"),
            node("Direct Access Storage", "FCDirectAccessStorage", "FCDirectAccessStorage"),
            node("Internal Storage", "FCInternalStorage", "FCInternalStorage"),
        ],
    },
    ANNOTATIONS,
];
// component and deployment diagrams share the Component and Deployment groups (uml.json "uml.component",
// "uml.deployment")
const COMPONENT_GROUPS: &[Group] = &[
    Group {
        id: "component",
        label: "Component",
        items: &[
            node("Component", "UMLComponent", "component"),
            node("Artifact", "UMLArtifact", "artifact"),
            node("Interface", "UMLInterface", "interface"),
            node("Frame", "Frame", "frame"),
            line("Dependency", "UMLDependency", "dependency", AssocInit::Plain),
            line("Interface Realization", "UMLInterfaceRealization", "realization", AssocInit::Plain),
            line("Component Realization", "UMLComponentRealization", "realization", AssocInit::Plain),
        ],
    },
    Group {
        id: "deployment",
        label: "Deployment",
        items: &[
            node("Node", "UMLNode", "node"),
            line("Deployment", "UMLDeployment", "dependency", AssocInit::Plain),
            line("Communication Path", "UMLCommunicationPath", "association", AssocInit::Plain),
        ],
    },
    COMPOSITE,
    INSTANCES,
    ANNOTATIONS,
];
// objects and links (uml.json "uml.instance": UMLClassDiagram, UMLObjectDiagram, …)
const INSTANCES: Group = Group {
    id: "instance",
    label: "Instances",
    items: &[
        node("Object", "UMLObject", "object"),
        node("Artifact Instance", "UMLArtifactInstance", "artifact"),
        node("Component Instance", "UMLComponentInstance", "component"),
        node("Node Instance", "UMLNodeInstance", "node"),
        line("Link", "UMLLink", "association", AssocInit::Plain),
        line("Directed Link", "UMLLink", "directed", AssocInit::Directed),
        line("Link Object", "LinkObject", "associationclass", AssocInit::Plain),
    ],
};
const OBJECT_GROUPS: &[Group] = &[INSTANCES, ANNOTATIONS];
// profile diagrams (uml.json "uml.profile")
const PROFILE_GROUPS: &[Group] = &[
    Group {
        id: "profile",
        label: "Profiles",
        items: &[
            node("MetaClass", "UMLMetaClass", "class"),
            node("Stereotype", "UMLStereotype", "class"),
            node("Enumeration", "UMLEnumeration", "enumeration"),
            line("Extension", "UMLExtension", "generalization", AssocInit::Plain),
            line("Generalization", "UMLGeneralization", "generalization", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
// information flow diagrams: their own group, then the use case and basic class tools (uml.json)
const INFO_FLOW_GROUPS: &[Group] = &[
    Group {
        id: "information-flow",
        label: "Information Flows",
        items: &[node("Information Item", "UMLInformationItem", "class"), line("Information Flow", "UMLInformationFlow", "dependency", AssocInit::Plain)],
    },
    USECASE_GROUPS[0],
    CLASS_GROUPS[0],
    ANNOTATIONS,
];
// statechart diagrams (uml.json "uml.statechart", "uml.statechart-adv")
const STATECHART_GROUPS: &[Group] = &[
    Group {
        id: "statechart",
        label: "Statechart (Basic)",
        items: &[
            node("Simple State", "UMLState", "state"),
            node("Initial State", "Initial", "initial"),
            node("Choice", "Choice", "choice"),
            node("Join", "Join", "fork"),
            node("Fork", "Fork", "fork"),
            node("Final State", "UMLFinalState", "final"),
            line("Transition", "UMLTransition", "dependency", AssocInit::Plain),
            // a click on a state makes a transition to itself (uml.json "self-connection")
            line("Self Transition", "UMLTransition", "dependency", AssocInit::Plain),
        ],
    },
    Group {
        id: "statechart-adv",
        label: "Statechart (Advanced)",
        items: &[
            node("Composite State", "CompositeState", "state"),
            node("Submachine State", "SubmachineState", "state"),
            node("Orthogonal State", "OrthogonalState", "state"),
            node("Junction", "Junction", "initial"),
            node("Shallow History", "ShallowHistory", "history"),
            node("Deep History", "DeepHistory", "history"),
            node("Entry Point", "EntryPoint", "final"),
            node("Exit Point", "ExitPoint", "final"),
            node("Terminate", "Terminate", "terminate"),
            node("Connection Point Reference", "UMLConnectionPointReference", "connection-point"),
            node("Frame", "Frame", "frame"),
        ],
    },
    ANNOTATIONS,
];
// activity diagrams (uml.json "uml.activity", "uml.activity-adv")
const ACTIVITY_GROUPS: &[Group] = &[
    Group {
        id: "activity",
        label: "Activities (Basic)",
        items: &[
            node("Action", "UMLAction", "action"),
            node("Initial", "UMLInitialNode", "initial"),
            node("Final", "UMLActivityFinalNode", "final"),
            node("Fork", "UMLForkNode", "fork"),
            node("Join", "UMLJoinNode", "fork"),
            node("Merge", "UMLMergeNode", "choice"),
            node("Decision", "UMLDecisionNode", "choice"),
            line("Control Flow", "UMLControlFlow", "dependency", AssocInit::Plain),
        ],
    },
    Group {
        id: "activity-adv",
        label: "Activities (Advanced)",
        items: &[
            node("Swimlane (Vertical)", "SwimlaneVert", "swimlane-vert"),
            node("Swimlane (Horizontal)", "SwimlaneHorz", "swimlane-horz"),
            node("Interruptible Activity Region", "UMLInterruptibleActivityRegion", "interruptible"),
            node("Structured Activity", "UMLStructuredActivityNode", "structured"),
            node("Input Pin", "UMLInputPin", "pin"),
            node("Output Pin", "UMLOutputPin", "pin"),
            node("Send Signal", "SendSignal", "send-signal"),
            node("Accept Signal", "AcceptSignal", "accept-signal"),
            node("Accept Time Event", "AcceptTimeEvent", "time-event"),
            node("Flow Final", "UMLFlowFinalNode", "flow-final"),
            node("Object Node", "UMLObjectNode", "object-node"),
            node("Central Buffer", "UMLCentralBufferNode", "object-node"),
            node("Datastore", "UMLDataStoreNode", "object-node"),
            node("Activity Parameter Node", "UMLActivityParameterNode", "param-node"),
            node("Expansion Region", "UMLExpansionRegion", "expansion-region"),
            node("Input Expansion Node", "UMLInputExpansionNode", "expansion-node"),
            node("Output Expansion Node", "UMLOutputExpansionNode", "expansion-node"),
            node("Frame", "Frame", "frame"),
            line("Object Flow", "UMLObjectFlow", "dependency", AssocInit::Plain),
            line("Exception Handler", "UMLExceptionHandler", "interrupt", AssocInit::Plain),
            line("Activity Interrupt", "UMLActivityInterrupt", "interrupt", AssocInit::Plain),
            node("Activity Edge Connector", "UMLActivityEdgeConnector", "edge-connector"),
        ],
    },
    ANNOTATIONS,
];
// a composite structure diagram: the two class groups and Composite Structure (uml.json)
const COMPOSITE_GROUPS: &[Group] = &[CLASS_GROUPS[0], CLASS_GROUPS[1], COMPOSITE, ANNOTATIONS];
// a package diagram has the Packages group (uml.json "uml.package": UMLClassDiagram, UMLPackageDiagram)
const PACKAGE_GROUPS: &[Group] = &[CLASS_GROUPS[2], SYSML_VIEWPOINTS, ANNOTATIONS];

// SysML diagrams (sysml/toolbox/sysml.json)
const SYSML_REQUIREMENTS: Group = Group {
    id: "sysml.requirement",
    label: "Requirements",
    items: &[
        node("Package", "UMLPackage", "package"),
        node("Requirement", "SysMLRequirement", "SysMLRequirement"),
        node("Frame", "Frame", "frame"),
        line("Copy", "SysMLCopy", "dependency", AssocInit::Plain),
        line("DeriveReqt", "SysMLDeriveReqt", "dependency", AssocInit::Plain),
        line("Verify", "SysMLVerify", "dependency", AssocInit::Plain),
        line("Satisfy", "SysMLSatisfy", "dependency", AssocInit::Plain),
        line("Refine", "SysMLRefine", "dependency", AssocInit::Plain),
        line("Containment", "Containment", "containment", AssocInit::Plain),
    ],
};
// the Viewpoints group, on requirement, block definition and package diagrams
const SYSML_VIEWPOINTS: Group = Group {
    id: "sysml.viewpoint",
    label: "Viewpoints",
    items: &[
        node("Stakeholder", "SysMLStakeholder", "class"),
        node("View", "SysMLView", "class"),
        node("Viewpoint", "SysMLViewpoint", "class"),
        line("Conform", "SysMLConform", "generalization", AssocInit::Plain),
        line("Expose", "SysMLExpose", "dependency", AssocInit::Plain),
    ],
};
const SYSML_REQUIREMENT_GROUPS: &[Group] = &[SYSML_REQUIREMENTS, SYSML_VIEWPOINTS, ANNOTATIONS];
const SYSML_BLOCK: Group = Group {
    id: "sysml.block",
    label: "Block",
    items: &[
        node("Block", "SysMLBlock", "SysMLBlock"),
        node("Value Type", "SysMLValueType", "datatype"),
        node("Interface Block", "SysMLInterfaceBlock", "SysMLBlock"),
        node("Constraint Block", "SysMLConstraintBlock", "SysMLConstraintBlock"),
        node("Enumeration", "UMLEnumeration", "enumeration"),
        node("Signal", "UMLSignal", "datatype"),
        node("Port", "SysMLPort", "port"),
        node("Object", "UMLObject", "object"),
        node("Frame", "Frame", "frame"),
        line("Association", "UMLAssociation", "association", AssocInit::Plain),
        line("Directed Association", "UMLAssociation", "directed", AssocInit::Directed),
        line("Aggregation", "UMLAssociation", "aggregation", AssocInit::Shared),
        line("Composition", "UMLAssociation", "composition", AssocInit::Composite),
        line("Dependency", "UMLDependency", "dependency", AssocInit::Plain),
        line("Generalization", "UMLGeneralization", "generalization", AssocInit::Plain),
        line("Interface Realization", "UMLInterfaceRealization", "realization", AssocInit::Plain),
        line("Connector", "SysMLConnector", "association", AssocInit::Plain),
        line("Containment", "Containment", "containment", AssocInit::Plain),
    ],
};
const SYSML_BLOCK_GROUPS: &[Group] = &[SYSML_BLOCK, SYSML_VIEWPOINTS, ANNOTATIONS];
const SYSML_INTERNAL_BLOCK: Group = Group {
    id: "sysml.internal-block",
    label: "Internal Block",
    items: &[
        node("Part", "SysMLPart", "part"),
        node("Reference", "SysMLReference", "part"),
        node("Value", "SysMLValue", "part"),
        node("Port", "SysMLPort", "port"),
        node("Frame", "Frame", "frame"),
        line("Connector", "SysMLConnector", "association", AssocInit::Plain),
    ],
};
const SYSML_PARAMETRIC: Group = Group {
    id: "sysml.parametric",
    label: "Parametric",
    items: &[node("Constraint Property", "SysMLConstraintProperty", "SysMLConstraintProperty"), node("Constraint Parameter", "SysMLConstraintParameter", "SysMLConstraintParameter")],
};
const SYSML_INTERNAL_BLOCK_GROUPS: &[Group] = &[SYSML_INTERNAL_BLOCK, ANNOTATIONS];
const SYSML_PARAMETRIC_GROUPS: &[Group] = &[SYSML_INTERNAL_BLOCK, SYSML_PARAMETRIC, ANNOTATIONS];

/// The toolbox groups of a diagram kind (the "diagram-types" of each group in toolbox/*.json).
pub(crate) fn tool_groups(diagram_type: Option<&str>) -> &'static [Group] {
    match diagram_type {
        Some("UMLClassDiagram") => CLASS_GROUPS,
        Some("UMLUseCaseDiagram") => USECASE_GROUPS,
        Some("UMLPackageDiagram") => PACKAGE_GROUPS,
        Some("UMLObjectDiagram") => OBJECT_GROUPS,
        Some("UMLInformationFlowDiagram") => INFO_FLOW_GROUPS,
        Some("UMLProfileDiagram") => PROFILE_GROUPS,
        Some("UMLCompositeStructureDiagram") => COMPOSITE_GROUPS,
        Some("UMLStatechartDiagram") => STATECHART_GROUPS,
        Some("UMLActivityDiagram") => ACTIVITY_GROUPS,
        Some("FCFlowchartDiagram") => FLOWCHART_GROUPS,
        Some("DFDDiagram") => DFD_GROUPS,
        Some("MMMindmapDiagram") => MINDMAP_GROUPS,
        Some("ERDDiagram") => ERD_GROUPS,
        Some("C4Diagram") => C4_GROUPS,
        Some("BPMNDiagram") => BPMN_GROUPS,
        Some("AWSDiagram") => AWS_GROUPS,
        Some("AzureDiagram") => AZURE_GROUPS,
        Some("GCPDiagram") => GCP_GROUPS,
        Some("SysMLRequirementDiagram") => SYSML_REQUIREMENT_GROUPS,
        Some("SysMLBlockDefinitionDiagram") => SYSML_BLOCK_GROUPS,
        Some("SysMLInternalBlockDiagram") => SYSML_INTERNAL_BLOCK_GROUPS,
        Some("SysMLParametricDiagram") => SYSML_PARAMETRIC_GROUPS,
        Some("WFWireframeDiagram") => WIREFRAME_GROUPS,
        Some("UMLSequenceDiagram") => SEQUENCE_GROUPS,
        Some("UMLCommunicationDiagram") => COMMUNICATION_GROUPS,
        Some("UMLInteractionOverviewDiagram") => OVERVIEW_GROUPS,
        Some("UMLTimingDiagram") => TIMING_GROUPS,
        Some("UMLComponentDiagram") | Some("UMLDeploymentDiagram") => COMPONENT_GROUPS,
        Some(_) => OTHER_GROUPS,
        None => &[],
    }
}

/// A toolbox tool by its factory id (the kind it creates), as the scriptable factory commands name
/// it: first among the tools of the diagram kind, the plain one when several tools share the kind.
pub(crate) fn tool_for(diagram_type: &str, id: &str) -> Option<Tool> {
    let find = |groups: &[Group]| {
        let all = || groups.iter().flat_map(|g| g.items.iter());
        all().find(|t| t.kind == id && matches!(t.init, AssocInit::Plain)).or_else(|| all().find(|t| t.kind == id)).copied()
    };
    find(tool_groups(Some(diagram_type))).or_else(|| DIAGRAM_KINDS.iter().find_map(|(_, k)| find(tool_groups(Some(k)))))
}

/// Model › Add Diagram, for the diagram kinds UMLCraft edits.
const DIAGRAM_KINDS: [(&str, &str); 29] = [
    ("Class Diagram", "UMLClassDiagram"),
    ("Package Diagram", "UMLPackageDiagram"),
    ("Object Diagram", "UMLObjectDiagram"),
    ("Composite Structure Diagram", "UMLCompositeStructureDiagram"),
    ("Component Diagram", "UMLComponentDiagram"),
    ("Deployment Diagram", "UMLDeploymentDiagram"),
    ("Use Case Diagram", "UMLUseCaseDiagram"),
    ("Information Flow Diagram", "UMLInformationFlowDiagram"),
    ("Profile Diagram", "UMLProfileDiagram"),
    ("Statechart Diagram", "UMLStatechartDiagram"),
    ("Activity Diagram", "UMLActivityDiagram"),
    ("Sequence Diagram", "UMLSequenceDiagram"),
    ("Communication Diagram", "UMLCommunicationDiagram"),
    ("Timing Diagram", "UMLTimingDiagram"),
    ("Interaction Overview Diagram", "UMLInteractionOverviewDiagram"),
    ("Flowchart Diagram", "FCFlowchartDiagram"),
    // the original's labels (dfd/menus/dfd.json, mindmap/menus/mindmap.json)
    ("DataFlowDiagram", "DFDDiagram"),
    ("MindmapDiagram", "MMMindmapDiagram"),
    ("ER Diagram", "ERDDiagram"),
    ("C4 Diagram", "C4Diagram"),
    ("Wireframe Diagram", "WFWireframeDiagram"),
    ("BPMN Diagram", "BPMNDiagram"),
    ("AWS Architecture Diagram", "AWSDiagram"),
    ("Azure Architecture Diagram", "AzureDiagram"),
    ("GCP Architecture Diagram", "GCPDiagram"),
    // the SysML kinds (sysml/menus/sysml.json)
    ("Requirement Diagram", "SysMLRequirementDiagram"),
    ("Block Definition Diagram", "SysMLBlockDefinitionDiagram"),
    ("Internal Block Diagram", "SysMLInternalBlockDiagram"),
    ("Parametric Diagram", "SysMLParametricDiagram"),
];

/// Explorer order: kind priority, then the order they were added (core.js:311 getOrdering;
/// priorities from the UML metamodel; kinds without one, such as diagrams, come first).
pub(crate) fn order_rank(t: &str) -> u32 {
    match t {
        "UMLModel" => 1000, "UMLPackage" => 1001, "UMLSubsystem" => 1002, "UMLUseCaseSubject" => 1003,
        "UMLTemplateParameter" => 1101, "UMLParameter" => 1102, "UMLEnumerationLiteral" => 1103, "UMLPort" => 1104,
        "UMLAttribute" => 1105, "UMLOperation" => 1106, "UMLReception" => 1107, "UMLExtensionPoint" => 1108, "UMLSlot" => 1109,
        "UMLClass" => 1201, "UMLInterface" => 1202, "UMLSignal" => 1203, "UMLDataType" => 1204, "UMLPrimitiveType" => 1205,
        "UMLEnumeration" => 1206, "UMLArtifact" => 1207, "UMLComponent" => 1208, "UMLNode" => 1209, "UMLUseCase" => 1210, "UMLActor" => 1211,
        "UMLInstance" => 1301, "UMLObject" => 1302, "UMLCollaboration" => 1401, "UMLStateMachine" => 1402, "UMLInteraction" => 1403, "UMLActivity" => 1404, "UMLRegion" => 1407, "UMLState" => 1409, "UMLTemplateParameterSubstitution" => 1102,
        "UMLAssociationEnd" => 1801, "UMLAssociation" => 1802, "UMLDependency" => 1806, "UMLGeneralization" => 1807, "UMLRealization" => 1808,
        "UMLInterfaceRealization" => 1809, "UMLComponentRealization" => 1810, "UMLAbstraction" => 1811, "UMLInclude" => 1816, "UMLExtend" => 1817,
        "UMLTransition" => 1819, "UMLMessage" => 1822, "UMLProfile" => 1901, "UMLStereotype" => 1903,
        t => crate::sysml::order_rank(t),
    }
}

fn text_galley_width(ui: &egui::Ui, t: &str, size: f32) -> f32 {
    ui.ctx().fonts_mut(|f| f.layout_no_wrap(t.to_string(), FontId::proportional(size), TEXT()).size().x)
}

impl App {
    // ------------------------------------------------------------ explorer
    pub fn explorer_children(&self, e: Eid) -> Vec<Eid> {
        let mut list: Vec<(u32, usize, Eid)> = self
            .repo
            .children(e)
            .into_iter()
            .filter(|(k, _)| k != "ownedViews" && k != "subViews")
            .enumerate()
            .map(|(i, (_, c))| (order_rank(self.repo.ty(c)), i, c))
            .collect();
        if self.sort_by_name {
            list.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| self.repo.name(a.2).cmp(&self.repo.name(b.2))));
        } else {
            list.sort_by_key(|x| (x.0, x.1));
        }
        list.into_iter().map(|x| x.2).collect()
    }

    pub fn label_of(&self, e: Eid) -> String {
        let r = &self.repo;
        let t = r.ty(e);
        let name = r.name(e);
        let nm = |x: Option<Eid>| x.map(|x| r.name(x)).unwrap_or_default();
        if r.has(e, "end1") && r.has(e, "end2") {
            let a = r.child(e, "end1").and_then(|x| r.refd(x, "reference"));
            let b = r.child(e, "end2").and_then(|x| r.refd(x, "reference"));
            let s = format!("({}—{})", nm(a), nm(b));
            return if name.is_empty() { s } else { format!("{name} {s}") };
        }
        if r.has(e, "source") && r.has(e, "target") {
            let s = format!("({}→{})", nm(r.refd(e, "source")), nm(r.refd(e, "target")));
            return if name.is_empty() { s } else { format!("{name} {s}") };
        }
        if r.kind_of(e, "RelationshipEnd") || t == "UMLAssociationEnd" {
            return if name.is_empty() { format!("({})", nm(r.refd(e, "reference"))) } else { name };
        }
        if t == "Tag" {
            return format!("{name} = {}", r.str(e, "value"));
        }
        // a lifeline: `name: Type`, the type of the role it represents (UMLLifeline.getNodeText and
        // getTypeString, elements.js:3787-3800, 3840-3855)
        if t == "UMLLifeline" {
            let ty = r.refd(e, "represent").map(|a| views::type_name(r, a, "type")).unwrap_or_default();
            let text = if ty.is_empty() { name } else { format!("{name}: {ty}") };
            return if text.is_empty() { "(Lifeline)".into() } else { text };
        }
        // a hyperlink: its name with showName, else what it links to (core.js:1322-1333)
        if t == "Hyperlink" {
            let url = r.str(e, "url");
            return if r.bool(e, "showName") {
                format!("(link: {})", if name.is_empty() { "(Hyperlink)" } else { &name })
            } else if let Some(x) = r.refd(e, "reference") {
                format!("(link to {})", r.name(x))
            } else if !url.is_empty() {
                format!("(link to {url})")
            } else {
                "(none)".to_string()
            };
        }
        if name.is_empty() { format!("({})", crate::sysml::display_name(t.trim_start_matches("UML"))) } else { name }
    }

    fn tree_row(&mut self, ui: &mut egui::Ui, e: Eid, depth: usize, has_kids: bool, open: bool, search_parent: Option<String>) {
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::click_and_drag());
        let x0 = rect.left() + 4.0 + depth as f32 * 16.0;
        let p = ui.painter();
        if has_kids {
            let c = Pos2::new(x0 + 8.0, rect.center().y);
            let tri = if open {
                vec![c + Vec2::new(-3.5, -1.5), c + Vec2::new(3.5, -1.5), c + Vec2::new(0.0, 3.0)]
            } else {
                vec![c + Vec2::new(-1.5, -3.5), c + Vec2::new(3.0, 0.0), c + Vec2::new(-1.5, 3.5)]
            };
            p.add(egui::Shape::convex_polygon(tri, TEXT(), Stroke::NONE));
        }
        let ir = Rect::from_min_size(Pos2::new(x0 + 16.0, rect.center().y - 8.0), Vec2::splat(16.0));
        icons::draw_for(p, ir, &self.repo, e);
        let mut label = self.label_of(e);
        if self.show_stereotype {
            // a SysML relationship shows its keyword (getNodeText, sysml/elements.js:259-375)
            let st = Some(views::stereotype_text(&self.repo, e)).filter(|s| !s.is_empty()).or_else(|| crate::sysml::keyword(self.repo.ty(e)).map(String::from)).unwrap_or_default();
            if !st.is_empty() {
                label = format!("{st} {label}");
            }
        }
        if let Some(parent) = search_parent {
            label = format!("{label} — {parent}");
        }
        let tx = ir.right() + 6.0;
        let tw = text_galley_width(ui, &label, 13.0);
        // renaming in place: Enter applies, Esc cancels (model-explorer-view.js)
        if let Some((re, mut text, fresh)) = self.tree_rename.clone().filter(|r| r.0 == e) {
            let er = Rect::from_min_max(Pos2::new(tx - 3.0, rect.top() + 1.0), Pos2::new(rect.right() - 4.0, rect.bottom() - 1.0));
            let out = ui.put(er, egui::TextEdit::singleline(&mut text).font(FontId::proportional(13.0)).margin(Margin::symmetric(3, 1)));
            if fresh {
                out.request_focus();
                if let Some(mut st) = egui::TextEdit::load_state(ui.ctx(), out.id) {
                    let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(text.chars().count()));
                    st.cursor.set_char_range(Some(all));
                    st.store(ui.ctx(), out.id);
                }
            }
            let (enter, esc) = ui.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
            if esc {
                self.tree_rename = None;
            } else if (out.lost_focus() || enter) && !fresh {
                // the Enter that applied the name does not start another rename
                ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                self.tree_rename = None;
                if self.repo.alive(re) && self.repo.name(re) != text {
                    self.change(|a| a.repo.set(re, "name", json!(text)));
                }
            } else {
                self.tree_rename = Some((re, text, false));
            }
            return;
        }
        if self.focus == Some(e) && !self.tree_cleared {
            let lr = Rect::from_min_size(Pos2::new(tx - 3.0, rect.top() + 1.0), Vec2::new(tw + 6.0, 20.0));
            p.rect_filled(lr, 0.0, SELECTED_ROW());
            p.rect_stroke(lr, 0.0, Stroke::new(1.0, Color32::from_rgb(0x4f, 0x99, 0xff)), egui::StrokeKind::Inside);
        }
        p.text(Pos2::new(tx, rect.center().y), Align2::LEFT_CENTER, &label, FontId::proportional(13.0), TEXT());

        let ty = self.repo.ty(e).to_string();
        if ty != "Project" {
            resp.dnd_set_drag_payload(e);
        }
        // dropping a row on another moves it there (model-explorer-view.js:182-241, app-context.js:1117-1144)
        if let Some(src) = resp.dnd_hover_payload::<Eid>() {
            let ok = self.can_relocate(*src, e);
            if ok {
                ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0x4f, 0x99, 0xff)), egui::StrokeKind::Inside);
            } else if *src != e {
                ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
            }
        }
        if let Some(src) = resp.dnd_release_payload::<Eid>() {
            let src = *src;
            if self.can_relocate(src, e) {
                let field = self.repo.field_of(src).unwrap_or_default();
                self.change(|a| a.repo.relocate(src, e, &field));
                self.expanded.insert(e);
                self.focus = Some(src);
                self.reveal(src);
            }
        }
        if resp.clicked() || resp.secondary_clicked() {
            self.last_area = Area::Explorer;
            self.tree_cleared = false;
            let on_toggle = resp.interact_pointer_pos().map(|q| q.x < x0 + 16.0).unwrap_or(false);
            if on_toggle && has_kids && resp.clicked() {
                if !self.expanded.remove(&e) {
                    self.expanded.insert(e);
                }
            } else {
                self.focus_element(e);
            }
        }
        if resp.double_clicked() {
            if self.is_diagram(e) {
                self.open_diagram(e);
            } else if has_kids && !self.expanded.remove(&e) {
                self.expanded.insert(e);
            }
        }
        resp.context_menu(|ui| self.model_menu(ui, e));
        if self.reveal_pending == Some(e) {
            resp.scroll_to_me(Some(egui::Align::Center));
            self.reveal_pending = None;
        }
    }

    fn tree(&mut self, ui: &mut egui::Ui, e: Eid, depth: usize) {
        let kids = self.explorer_children(e);
        let open = self.expanded.contains(&e);
        self.tree_row(ui, e, depth, !kids.is_empty(), open, None);
        if open {
            for k in kids {
                if self.repo.alive(k) {
                    self.tree(ui, k, depth + 1);
                }
            }
        }
    }

    /// What Model › Add offers under an element: (label, type, is a feature).
    pub(crate) fn add_kinds(&self, e: Eid) -> Vec<(&'static str, &'static str, bool)> {
        let t = self.repo.ty(e);
        let mut adds = vec![];
        if self.repo.kind_of(e, "UMLPackage") || t == "Project" {
            if t == "Project" {
                adds.push(("Model", "UMLModel", false));
            }
            for (l, k) in [
                ("Package", "UMLPackage"), ("Profile", "UMLProfile"), ("Class", "UMLClass"), ("Interface", "UMLInterface"), ("Signal", "UMLSignal"),
                ("DataType", "UMLDataType"), ("PrimitiveType", "UMLPrimitiveType"), ("Enumeration", "UMLEnumeration"), ("Stereotype", "UMLStereotype"),
                ("Actor", "UMLActor"), ("Use Case", "UMLUseCase"), ("Subsystem", "UMLSubsystem"), ("Component", "UMLComponent"), ("Artifact", "UMLArtifact"), ("Node", "UMLNode"),
                ("Object", "UMLObject"), ("Information Item", "UMLInformationItem"), ("Collaboration", "UMLCollaboration"),
            ] {
                adds.push((l, k, false));
            }
        }
        // a metaclass goes in a profile
        if t == "UMLProfile" {
            adds.push(("MetaClass", "UMLMetaClass", false));
        }
        // behaviours: an interaction and an opaque behaviour in a classifier, a state machine and an
        // activity in a classifier or a package (uml-commands.js:4272-4275)
        let classifier = self.repo.kind_of(e, "UMLClassifier");
        let package = self.repo.kind_of(e, "UMLPackage");
        for (l, k, ok) in [("Interaction", "UMLInteraction", classifier), ("State Machine", "UMLStateMachine", classifier || package), ("Activity", "UMLActivity", classifier || package), ("Opaque Behavior", "UMLOpaqueBehavior", classifier)] {
            if ok {
                adds.push((l, k, false));
            }
        }
        // data flow models in a project or package, and their elements in any DFD element; a mind map
        // in any element (dfd-commands.js:119-134, mindmap-commands.js:62-74)
        if t == "Project" || self.repo.kind_of(e, "UMLPackage") {
            adds.push(("DataFlowModel", "DFDDataFlowModel", false));
        }
        // a C4 model in the project, C4 elements in a C4 model or element (c4/menus/c4.json;
        // c4-commands.js:20-40)
        if t == "Project" {
            adds.push(("C4 Model", "C4Model", false));
        }
        if self.repo.kind_of(e, "C4Element") {
            adds.extend([("C4 Person", "C4Person", false), ("C4 Software System", "C4SoftwareSystem", false), ("C4 Container", "C4Container", false), ("C4 Component", "C4Component", false)]);
        }
        // ER data models in a project or package, entities in a data model, columns in an entity
        // (erd/menus/erd.json; erd-commands.js:160-175)
        if t == "Project" || self.repo.kind_of(e, "UMLPackage") {
            adds.push(("Data Model", "ERDDataModel", false));
        }
        if t == "ERDDataModel" {
            adds.push(("Entity", "ERDEntity", false));
        }
        if t == "ERDEntity" {
            adds.push(("Column", "ERDColumn", true));
        }
        // BPMN processes, their elements, events and event definitions (bpmn/menus/bpmn.json)
        adds.extend(crate::bpmn::add_kinds(&self.repo, e));
        if self.repo.kind_of(e, "DFDElement") {
            adds.extend([("ExternalEntity", "DFDExternalEntity", false), ("Process", "DFDProcess", false), ("DataStore", "DFDDataStore", false)]);
        }
        if self.repo.kind_of(e, "ExtensibleModel") && !self.repo.kind_of(e, "Diagram") {
            adds.push(("Mindmap", "MMMindmap", false));
            // and a wireframe (wireframe/menus/wireframe.json, wireframe-commands.js:50-60)
            adds.push(("Wireframe", "WFWireframe", false));
            // and the cloud models: their menus add one under any element (aws/menus/aws.json)
            for (_, model, _, label) in crate::cloud::DIAGRAMS {
                adds.push((label, model, false));
            }
        }
        // SysML elements (sysml/menus/sysml.json)
        adds.extend(crate::sysml::add_kinds(&self.repo, e));
        // Model › Add › Constraint on any model element (uml.json)
        if self.repo.kind_of(e, "UMLModelElement") {
            adds.push(("Constraint", "UMLConstraint", false));
        }
        for (l, k) in [
            ("Attribute", "UMLAttribute"), ("Port", "UMLPort"), ("Operation", "UMLOperation"), ("Reception", "UMLReception"), ("Extension Point", "UMLExtensionPoint"), ("Slot", "UMLSlot"), ("Literal", "UMLEnumerationLiteral"),
            ("Parameter", "UMLParameter"), ("Template Parameter", "UMLTemplateParameter"), ("Template Parameter Substitution", "UMLTemplateParameterSubstitution"),
            ("State", "UMLState"), ("Region", "UMLRegion"), ("Trigger", "UMLEvent"), ("Action", "UMLAction"), ("Interaction Operand", "UMLInteractionOperand"),
        ] {
            if self.can_add_feature(e, k) {
                adds.push((l, k, true));
            }
        }
        // a state's activities and a transition's effect ask which kind of behaviour (uml.json)
        if t == "UMLState" {
            adds.extend([("Entry Activity", "entryActivities", false), ("Do Activity", "doActivities", false), ("Exit Activity", "exitActivities", false)]);
        }
        if t == "UMLTransition" {
            adds.push(("Effect", "effects", false));
        }
        // a block takes no plain attributes (updateMenus, sysml-commands.js:308-309)
        if self.repo.kind_of(e, "SysMLBlock") {
            adds.retain(|a| a.1 != "UMLAttribute");
        }
        adds
    }

    /// Ask which kind of behaviour to create in a state's activities or a transition's effects.
    pub fn ask_behavior_kind(&mut self, e: Eid, field: &'static str) {
        let what = match field {
            "entryActivities" => "an entry Activity",
            "doActivities" => "an do Activity",
            "exitActivities" => "an exit Activity",
            _ => "an effect Behavior",
        };
        self.select_one = Some(crate::app::SelectOne {
            message: format!("Select one to create as {what}"),
            options: crate::app::BEHAVIOR_KINDS.iter().map(|x| x.0).collect(),
            selected: 0,
            purpose: crate::app::SelectPurpose::BehaviorKind(e, field),
        });
    }

    /// The items of an Add submenu for an element.
    fn add_items(&mut self, ui: &mut egui::Ui, e: Eid) {
        // Model › Add › Tag on any element that can hold tags (common/menus)
        if self.repo.kind_of(e, "ExtensibleModel") && ui.button("Tag").clicked() {
            let name = self.new_name(e, "Tag");
            let t = self.change(|a| a.repo.create("Tag", &[("name", json!(name)), ("kind", json!("string"))], Some((e, "tags"))));
            self.expanded.insert(e);
            self.focus = Some(t);
            ui.close();
            return;
        }
        for (l, k, feature) in self.add_kinds(e) {
            // a behaviour of a state or a transition: which kind, in a Select One dialog (uml-commands.js:109-190)
            if matches!(k, "entryActivities" | "doActivities" | "exitActivities" | "effects") {
                if ui.button(l).clicked() {
                    self.ask_behavior_kind(e, k);
                    ui.close();
                }
                continue;
            }
            // a constraint on an operation: which kind, in a Select One dialog (uml-commands.js:185)
            if k == "UMLConstraint" && self.repo.ty(e) == "UMLOperation" {
                if ui.button(l).clicked() {
                    self.select_one = Some(crate::app::SelectOne {
                        message: "Select the kind of constraint".into(),
                        options: crate::app::OPERATION_CONSTRAINTS.iter().map(|x| x.0).collect(),
                        selected: 0,
                        purpose: crate::app::SelectPurpose::ConstraintKind(e),
                    });
                    ui.close();
                }
                continue;
            }
            if ui.button(l).clicked() {
                let created = self.change(|a| if feature { a.add_feature(e, k) } else { Some(a.add_model(e, k)) });
                self.expanded.insert(e);
                self.focus = created;
                if let Some(c) = created {
                    self.reveal(c);
                }
                ui.close();
            }
        }
    }

    fn model_menu(&mut self, ui: &mut egui::Ui, e: Eid) {
        let t = self.repo.ty(e).to_string();
        let is_pkg = self.repo.kind_of(e, "UMLPackage") || t == "Project";
        let is_classifier = self.repo.kind_of(e, "UMLClassifier");
        let adds = self.add_kinds(e);
        if !adds.is_empty() || self.repo.kind_of(e, "ExtensibleModel") {
            ui.menu_button("Add", |ui| self.add_items(ui, e));
        }
        // a flowchart or a mind map can be added under any element, a data flow diagram under a DFD
        // element too (flowchart-commands.js:136, mindmap-commands.js:62-74, dfd-commands.js:119-134)
        let other = self.repo.kind_of(e, "ExtensibleModel") && !self.repo.kind_of(e, "Diagram");
        if is_pkg || is_classifier || other {
            let dfd = self.repo.kind_of(e, "DFDElement");
            ui.menu_button("Add Diagram", |ui| {
                let kinds: Vec<(&str, &str)> = DIAGRAM_KINDS.into_iter().filter(|(_, k)| crate::sysml::diagram_offered(&self.repo, Some(e), k).unwrap_or(true)).filter(|(_, k)| crate::sysml::is_diagram(k) || is_pkg || is_classifier || matches!(*k, "FCFlowchartDiagram" | "MMMindmapDiagram" | "WFWireframeDiagram" | "BPMNDiagram") || crate::cloud::is_diagram(k) || (dfd && *k == "DFDDiagram") || (*k == "ERDDiagram" && self.repo.ty(e) == "ERDDataModel") || (*k == "C4Diagram" && (self.repo.kind_of(e, "C4Element") || self.repo.ty(e) == "Project"))).collect();
                for (l, k) in kinds {
                    if ui.button(l).clicked() {
                        self.change(|a| a.add_diagram(Some(e), k));
                        self.expanded.insert(e);
                        ui.close();
                    }
                }
            });
        }
        if self.is_diagram(e) && ui.button("Open Diagram").clicked() {
            self.open_diagram(e);
            ui.close();
        }
        if self.repo.kind_of(e, "ExtensibleModel") && ui.add(self.key_button("Tag Editor...", "model:tag-editor")).clicked() {
            self.focus = Some(e);
            self.open_tag_editor();
            ui.close();
        }
        if !self.is_diagram(e) && t != "Project" && ui.add(self.key_button("Select In Diagram", "edit:select-in-diagram")).clicked() {
            self.focus = Some(e);
            self.select_in_diagram();
            ui.close();
        }
        ui.separator();
        self.last_area = Area::Explorer;
        self.clipboard_items(ui);
        self.move_items(ui);
        if ui.add_enabled(t != "Project", self.key_button("Delete from Model", "edit:delete-from-model")).clicked() {
            self.change(|a| a.delete_models(&[e]));
            ui.close();
        }
    }

    pub(crate) fn explorer_ui(&mut self, ui: &mut egui::Ui) {
        let mut search_rect: Option<Rect> = None;
        ui.horizontal(|ui| {
            ui.set_height(HEADER_HEIGHT);
            ui.add_space(13.0);
            ui.label(egui::RichText::new("MODEL EXPLORER").color(TEXT()).size(13.0));
            ui.add_space(8.0);
            let w = ui.available_width() - 30.0;
            let r = ui.add_sized(Vec2::new(w, 23.0), egui::TextEdit::singleline(&mut self.search).hint_text("🔍").margin(Margin::symmetric(10, 3)));
            search_rect = Some(r.rect);
            if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.search.clear();
            }
            let (r, resp) = ui.allocate_exact_size(Vec2::new(14.0, 20.0), Sense::click());
            let c = r.center();
            let p = ui.painter();
            p.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-4.0, -1.5), c + Vec2::new(4.0, -1.5), c + Vec2::new(0.0, -6.5)], TEXT(), Stroke::NONE));
            p.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-4.0, 1.5), c + Vec2::new(4.0, 1.5), c + Vec2::new(0.0, 6.5)], TEXT(), Stroke::NONE));
            // the settings button opens the model-explorer-settings menu (model-explorer-view.js:632-641)
            let resp = resp.on_hover_text("Settings");
            egui::Popup::menu(&resp).show(|ui| self.explorer_settings_menu(ui));
        });
        let out = egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let root = self.repo.root;
            self.tree(ui, root, 0);
            ui.add_space(8.0);
        });
        // a shadow along the top while the tree is scrolled (model-explorer-view.js:704), hidden while
        // a panel is being resized (navigator-view.js:92-105)
        if out.state.offset.y > 0.5 && self.split_drag.is_none() {
            scroll_shadow(ui.painter(), out.inner_rect);
        }
        // quick search: an autocomplete under the box, names containing the text, at most 99, each with
        // its icon, label and " — parent"; picking one selects it in the tree (model-explorer-view.js:644-686)
        let q = self.search.trim().to_lowercase();
        if let (false, Some(sr)) = (q.is_empty(), search_rect) {
            let hits: Vec<Eid> = self.repo.all().filter(|e| !self.is_view(*e) && self.repo.name(*e).to_lowercase().contains(&q)).take(99).collect();
            let mut picked = None;
            egui::Area::new(egui::Id::new("explorer-search")).order(egui::Order::Foreground).fixed_pos(sr.left_bottom() + Vec2::new(0.0, 2.0)).show(ui.ctx(), |ui| {
                Frame::NONE.fill(LAYER1()).stroke(Stroke::new(1.0, INPUT_BORDER())).inner_margin(Margin::same(4)).show(ui, |ui| {
                    ui.set_width(sr.width().max(220.0));
                    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                        if hits.is_empty() {
                            ui.label(egui::RichText::new("No results").color(DIM()).size(12.0));
                        }
                        for e in hits {
                            ui.horizontal(|ui| {
                                let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                                icons::draw_for(ui.painter(), ir, &self.repo, e);
                                let parent = self.repo.parent(e).map(|p| self.repo.name(p)).filter(|n| !n.is_empty());
                                let text = match parent {
                                    Some(p) => format!("{} — {p}", self.label_of(e)),
                                    None => self.label_of(e),
                                };
                                if ui.selectable_label(false, egui::RichText::new(text).size(12.0)).clicked() {
                                    picked = Some(e);
                                }
                            });
                        }
                    });
                });
            });
            if let Some(e) = picked {
                self.search.clear();
                self.reveal(e);
                self.last_area = Area::Explorer;
            }
        }
        // arrow keys walk the tree while it is the last area used
        if self.last_area == Area::Explorer && self.editing.is_none() && !text_field_focused(ui.ctx()) && self.search.trim().is_empty() {
            let keys = ui.input(|i| (i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
            self.tree_keys(keys);
            // Enter renames the selected element in place
            if self.tree_rename.is_none() && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                self.start_tree_rename();
            }
        }
        if ui.ui_contains_pointer() && ui.input(|i| i.key_pressed(egui::Key::Delete)) && self.last_area == Area::Explorer && !text_field_focused(ui.ctx()) {
            if let Some(f) = self.focus.filter(|f| self.repo.ty(*f) != "Project" && !self.is_view(*f)) {
                self.change(|a| a.delete_models(&[f]));
            }
        }
    }

    /// Set the text alignment of notes and texts (horzAlign 0/2/1, vertAlign 3/5/4), one undo step.
    pub fn set_text_align(&mut self, views: &[Eid], field: &str, value: i64) {
        let views = views.to_vec();
        let field = field.to_string();
        self.change(|a| {
            for v in views {
                a.repo.set(v, &field, json!(value));
            }
        });
    }

    /// Start renaming the selected element in the tree, its name selected.
    pub fn start_tree_rename(&mut self) {
        if let Some(f) = self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f) && self.repo.ty(*f) != "Project" && crate::meta::order_of(self.repo.ty(*f), "name").is_some()) {
            self.tree_rename = Some((f, self.repo.name(f), true));
            self.reveal(f);
        }
    }

    /// The model-explorer-settings menu: sort order, stereotype text, Expand All and Collapse All.
    fn explorer_settings_menu(&mut self, ui: &mut egui::Ui) {
        if ui.radio(!self.sort_by_name, "Sort By Added").clicked() {
            self.sort_by_name = false;
            ui.close();
        }
        if ui.radio(self.sort_by_name, "Sort By Name").clicked() {
            self.sort_by_name = true;
            ui.close();
        }
        ui.separator();
        if ui.checkbox(&mut self.show_stereotype, "Show Stereotype Text").clicked() {
            ui.close();
        }
        ui.separator();
        if ui.button("Expand All").clicked() {
            self.expand_all();
            ui.close();
        }
        if ui.button("Collapse All").clicked() {
            self.expanded.clear();
            ui.close();
        }
    }

    /// Expand All: open every element that has children in the tree (model-explorer-view.js:455-477).
    pub fn expand_all(&mut self) {
        let all: Vec<Eid> = self.repo.all().filter(|e| !self.is_view(*e)).collect();
        for e in all {
            if !self.explorer_children(e).is_empty() {
                self.expanded.insert(e);
            }
        }
    }

    /// The rows the tree shows, top to bottom.
    pub fn visible_rows(&self) -> Vec<Eid> {
        fn walk(a: &App, e: Eid, out: &mut Vec<Eid>) {
            out.push(e);
            if a.expanded.contains(&e) {
                for k in a.explorer_children(e) {
                    if a.repo.alive(k) {
                        walk(a, k, out);
                    }
                }
            }
        }
        let mut out = vec![];
        walk(self, self.repo.root, &mut out);
        out
    }

    /// Up/Down move to the previous or next row; Right opens a row or goes to its first child; Left
    /// closes it or goes to its parent (the tree widget's own keys).
    pub fn tree_keys(&mut self, (up, down, left, right): (bool, bool, bool, bool)) {
        if !(up || down || left || right) {
            return;
        }
        let rows = self.visible_rows();
        let Some(f) = self.focus.filter(|f| rows.contains(f)) else {
            if let Some(&first) = rows.first() {
                self.focus_element(first);
            }
            return;
        };
        let i = rows.iter().position(|x| *x == f).unwrap_or(0);
        let kids = !self.explorer_children(f).is_empty();
        let target = if up {
            i.checked_sub(1).map(|j| rows[j])
        } else if down {
            rows.get(i + 1).copied()
        } else if right {
            if kids && !self.expanded.contains(&f) {
                self.expanded.insert(f);
                None
            } else if kids {
                self.explorer_children(f).first().copied()
            } else {
                None
            }
        } else if kids && self.expanded.contains(&f) {
            self.expanded.remove(&f);
            None
        } else {
            self.repo.parent(f)
        };
        if let Some(t) = target {
            self.focus_element(t);
            self.reveal(t);
        }
    }

    // ------------------------------------------------------------ editors
    pub(crate) fn fields_for(&self, e: Eid) -> Vec<(&'static str, &'static str)> {
        let r = &self.repo;
        let t = r.ty(e);
        let end = |p: &str| -> Vec<(&'static str, &'static str)> {
            let names: [(&'static str, &'static str); 14] = [
                ("name", "text"), ("reference", "ref"), ("stereotype", "text"), ("visibility", "enum"), ("navigable", "enum"), ("aggregation", "enum"), ("multiplicity", "text"),
                ("defaultValue", "text"), ("isReadOnly", "bool"), ("isOrdered", "bool"), ("isUnique", "bool"), ("isDerived", "bool"), ("isID", "bool"), ("ownerAttribute", "owner"),
            ];
            names
                .iter()
                .map(|(n, k)| {
                    let full: &'static str = match p {
                        "end1" => Box::leak(format!("end1.{n}").into_boxed_str()),
                        "end2" => Box::leak(format!("end2.{n}").into_boxed_str()),
                        _ => n,
                    };
                    (full, *k)
                })
                .collect()
        };
        if t == "Project" {
            return vec![("name", "text"), ("author", "text"), ("company", "text"), ("copyright", "text"), ("version", "text")];
        }
        if self.is_diagram(e) {
            return vec![("name", "text"), ("defaultDiagram", "bool")];
        }
        if t == "Hyperlink" {
            return vec![("name", "text"), ("reference", "link"), ("url", "text"), ("showName", "bool")];
        }
        if let Some(f) = crate::sysml::fields(r, e) {
            return f;
        }
        // the other notations' visible attributes (their metamodel.json files)
        match t {
            "FCFlow" | "DFDDataFlow" | "MMEdge" => return vec![("name", "text"), ("source", "ref"), ("target", "ref")],
            "C4Relationship" => return vec![("name", "text"), ("source", "ref"), ("target", "ref"), ("technology", "text"), ("description", "multiline")],
            "ERDRelationship" => {
                return vec![("name", "text"), ("identifying", "bool"), ("-", ""), ("end1.name", "text"), ("end1.cardinality", "enum"), ("-", ""), ("end2.name", "text"), ("end2.cardinality", "enum")];
            }
            "ERDColumn" => {
                return vec![
                    ("name", "text"), ("type", "text"), ("length", "text"), ("primaryKey", "bool"), ("foreignKey", "bool"), ("referenceTo", "ref"), ("nullable", "bool"), ("unique", "bool"),
                ];
            }
            "C4Container" => return vec![("name", "text"), ("technology", "text"), ("description", "multiline"), ("kind", "enum")],
            _ if t.starts_with("BPMN") && !self.is_diagram(e) => return crate::bpmn::fields(r, e),
            _ if r.kind_of(e, "C4Element") => return vec![("name", "text"), ("technology", "text"), ("description", "multiline")],
            _ if r.kind_of(e, "DFDElement") => return vec![("name", "text"), ("id", "text")],
            "WFMobileFrame" => return vec![("name", "text"), ("id", "text"), ("orientation", "enum")],
            "WFRadio" | "WFCheckbox" | "WFSwitch" => return vec![("name", "text"), ("id", "text"), ("checked", "bool")],
            "WFTab" | "WFTabList" => return vec![("name", "text"), ("id", "text"), ("position", "enum")],
            "WFSeparator" => return vec![("name", "text"), ("id", "text"), ("isVertical", "bool")],
            "WFSlider" => return vec![("name", "text"), ("id", "text"), ("value", "int")],
            _ if r.kind_of(e, "WFElement") => return vec![("name", "text"), ("id", "text")],
            // the cloud elements' visible attributes (aws/metamodel.json); the icon is not shown
            _ if crate::cloud::element_kind(t).is_some() || crate::cloud::edge_view_of(t).is_some() => return crate::cloud::fields(t),
            _ => {}
        }
        if t == "UMLAssociation" {
            let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isDerived", "bool"), ("-", "")];
            f.extend(end("end1"));
            f.push(("-", ""));
            f.extend(end("end2"));
            return f;
        }
        // a link: its two ends, without the association-only owner attribute
        if t == "UMLLink" {
            let link_end = |p: &str| end(p).into_iter().filter(|x| !x.0.ends_with("ownerAttribute")).collect::<Vec<_>>();
            let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("-", "")];
            f.extend(link_end("end1"));
            f.push(("-", ""));
            f.extend(link_end("end2"));
            return f;
        }
        // a connector: its kind, assembly or delegation, then its two ends (uml/metamodel.json UMLConnector)
        if t == "UMLConnector" {
            let conn_end = |p: &str| end(p).into_iter().filter(|x| !x.0.ends_with("ownerAttribute")).collect::<Vec<_>>();
            let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("kind", "enum"), ("-", "")];
            f.extend(conn_end("end1"));
            f.push(("-", ""));
            f.extend(conn_end("end2"));
            return f;
        }
        if r.has(e, "source") && r.has(e, "target") {
            let mut f = vec![("name", "text"), ("source", "ref"), ("target", "ref"), ("stereotype", "text"), ("visibility", "enum")];
            if t == "UMLGeneralization" {
                f.push(("discriminator", "text"));
            }
            if t == "UMLExtend" {
                f.push(("condition", "text"));
            }
            if t == "UMLRoleBinding" {
                f.push(("roleName", "text"));
            }
            if t == "UMLTransition" {
                f.push(("kind", "enum"));
                f.push(("guard", "text"));
            }
            // the items an information flow conveys, edited in the Element List Editor
            if t == "UMLInformationFlow" {
                f.push(("conveyed", "reflist"));
            }
            return f;
        }
        match t {
            "UMLAttribute" => vec![
                ("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isStatic", "bool"), ("isLeaf", "bool"), ("type", "type"), ("multiplicity", "text"),
                ("isReadOnly", "bool"), ("isOrdered", "bool"), ("isUnique", "bool"), ("defaultValue", "text"), ("isDerived", "bool"), ("aggregation", "enum"), ("isID", "bool"),
            ],
            "UMLOperation" => vec![
                ("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isStatic", "bool"), ("isLeaf", "bool"), ("concurrency", "enum"), ("isQuery", "bool"),
                ("isAbstract", "bool"), ("specification", "multiline"),
            ],
            // behaviours (uml/metamodel.json: the visible attributes of UMLBehavior and UMLActivity)
            "UMLOpaqueBehavior" | "UMLInteraction" | "UMLStateMachine" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isReentrant", "bool")],
            "UMLActivity" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isReentrant", "bool"), ("isReadOnly", "bool"), ("isSingleExecution", "bool")],
            "UMLTemplateParameterSubstitution" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("formal", "ref"), ("actual", "type")],
            "UMLParameter" => vec![("name", "text"), ("stereotype", "text"), ("type", "type"), ("multiplicity", "text"), ("defaultValue", "text"), ("direction", "enum")],
            "UMLEnumerationLiteral" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum")],
            "UMLExtensionPoint" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("location", "text")],
            "UMLObject" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("classifier", "type")],
            // a stereotype's icon: its minimum size and its icon script (uml/metamodel.json UMLImage)
            "UMLImage" => vec![("width", "int"), ("height", "int"), ("image", "image"), ("content", "multiline")],
            "UMLCollaborationUse" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("type", "type")],
            // the visible attributes of states, events and actions (uml/metamodel.json)
            "UMLState" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("submachine", "ref")],
            "UMLInteractionOperand" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("guard", "text")],
            "UMLEvent" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("kind", "enum"), ("value", "text"), ("expression", "text"), ("targetOperation", "ref"), ("targetSignal", "ref")],
            "UMLAction" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("kind", "enum"), ("isLocallyReentrant", "bool"), ("isSynchronous", "bool"), ("language", "text")],
            "UMLTimeConstraint" | "UMLDurationConstraint" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("specification", "multiline"), ("constrainedElements", "reflist"), ("min", "text"), ("max", "text")],
            "UMLConstraint" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("specification", "multiline"), ("constrainedElements", "reflist")],
            "UMLSlot" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("type", "type"), ("value", "text")],
            // a reception names the signal it receives (uml-commands.js:1484)
            "UMLReception" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isStatic", "bool"), ("isLeaf", "bool"), ("signal", "type")],
            "UMLAssociationEnd" => end(""),
            "UMLLinkEnd" => end("").into_iter().filter(|x| x.0 != "ownerAttribute").collect(),
            _ if r.kind_of(e, "UMLClassifier") => {
                let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isAbstract", "bool"), ("isFinalSpecialization", "bool"), ("isLeaf", "bool")];
                if t == "UMLClass" {
                    f.push(("isActive", "bool"));
                }
                f
            }
            _ if r.kind_of(e, "UMLModelElement") => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum")],
            _ if r.has(e, "name") || r.kind_of(e, "Model") => vec![("name", "text")],
            _ => vec![],
        }
    }

    fn target(&self, e: Eid, path: &str) -> Option<(Eid, String)> {
        match path.split_once('.') {
            Some((a, b)) => self.repo.child(e, a).map(|c| (c, b.to_string())),
            None => Some((e, path.to_string())),
        }
    }

    fn display_value(&self, obj: Eid, field: &str, kind: &str) -> String {
        if kind == "type" || field == "stereotype" {
            return views::type_name(&self.repo, obj, field);
        }
        match self.repo.val(obj, field) {
            Some(Value::String(s)) => s,
            Some(Value::Null) | None => String::new(),
            Some(v) => v.to_string(),
        }
    }

    pub(crate) fn commit_text(&mut self, obj: Eid, field: &str, kind: &str, text: String) {
        if self.display_value(obj, field, kind) == text {
            return;
        }
        let field = field.to_string();
        let kind = kind.to_string();
        self.change(|a| {
            if kind == "type" || field == "stereotype" {
                let linked = a.repo.raw(obj, &field).and_then(|v| a.repo.by_ref(v)).map(|t| a.repo.name(t));
                if linked.as_deref() == Some(text.as_str()) && !text.is_empty() {
                    return;
                }
                if text.is_empty() && field == "type" {
                    a.repo.set_raw(obj, "type", json!(""));
                } else if text.is_empty() {
                    a.repo.remove(obj, &field);
                } else if let Some(x) = a.resolve_name(obj, &text, crate::picker::field_type(&field)) {
                    // a name found from the owner outward, then anywhere, becomes a reference
                    a.repo.set_ref(obj, &field, Some(x));
                } else {
                    a.repo.set(obj, &field, json!(text));
                }
            } else {
                a.repo.set(obj, &field, json!(text));
            }
        });
    }

    pub(crate) fn enum_options(ty: &str, field: &str) -> &'static [&'static str] {
        if let Some(o) = crate::sysml::enum_options(ty, field) {
            return o;
        }
        match (ty, field) {
            ("UMLTransition", "kind") => return &["external", "internal", "local"],
            // UMLConnectorKind (uml/metamodel.json)
            ("UMLConnector", "kind") => return &["assembly", "delegation"],
            (t, f) if t.starts_with("BPMN") && crate::bpmn::enum_options(f).is_some() => return crate::bpmn::enum_options(f).unwrap_or(&[]),
            ("ERDRelationshipEnd", "cardinality") => return &["0..1", "1", "0..*", "1..*"],
            // WFOrientationKind, WFPositionKind (wireframe/metamodel.json)
            ("WFMobileFrame", "orientation") => return &["portrait", "landscape"],
            ("WFTab" | "WFTabList", "position") => return &["top", "bottom", "left", "right"],
            // GCPZoneKind, GCPPathKind, GCPMachineType, GCPDiskType, GCPAdditionalModifierType (gcp/metamodel.json)
            ("GCPZone", "zoneType") => return &crate::gcp::ZONE_TYPES,
            ("GCPPath", "pathType") => return &crate::gcp::PATH_KINDS,
            ("GCPProduct", "machineType") => return &crate::gcp::MACHINE_TYPES,
            ("GCPProduct", "diskType") => return &crate::gcp::DISK_TYPES,
            ("GCPProduct", "additionalModifier") => return &crate::gcp::MODIFIERS,
            ("C4Container", "kind") => {
                return &["server-webapp", "client-webapp", "desktop-app", "mobile-app", "console-app", "serverless-function", "database", "blob-store", "filesystem", "shell-script", "etc"]
            }
            ("UMLEvent", "kind") => return &["signal", "call", "change", "time", "anyReceive"],
            ("UMLAction", "kind") => {
                return &["opaque", "create", "destroy", "read", "write", "insert", "delete", "sendSignal", "acceptSignal", "triggerEvent", "acceptEvent", "structured", "timeEvent", "callBehavior", "callOperation"]
            }
            _ => {}
        }
        match field {
            "visibility" => &["public", "protected", "private", "package"],
            "aggregation" => &["none", "shared", "composite"],
            "navigable" => &["unspecified", "navigable", "notNavigable"],
            "direction" => &["in", "inout", "out", "return"],
            "concurrency" => &["sequential", "guarded", "concurrent"],
            _ => &[],
        }
    }

    /// Apply a change to every target as one undo step.
    fn change_all(&mut self, targets: &[(Eid, String)], f: impl Fn(&mut App, Eid, &str)) {
        self.change(|a| {
            for (obj, field) in targets {
                f(a, *obj, field);
            }
        });
    }

    /// One property row for one element, or for several (merged: a value they share, or blank with
    /// "—"; a change applies to all of them) (property-editor-view.js:140-147, 185-204, 780-794).
    fn prop_row(&mut self, ui: &mut egui::Ui, es: &[Eid], path: &'static str, kind: &str) {
        let targets: Vec<(Eid, String)> = es.iter().filter_map(|e| self.target(*e, path)).collect();
        let Some((obj, field)) = targets.first().cloned() else { return };
        let single = targets.len() == 1;
        let values: Vec<String> = targets.iter().map(|(o, f)| self.display_value(*o, f, kind)).collect();
        let merged: Option<String> = values.iter().all(|v| *v == values[0]).then(|| values[0].clone());
        ui.horizontal(|ui| {
            ui.set_min_height(29.0);
            let (lr, _) = ui.allocate_exact_size(Vec2::new(116.0, 22.0), Sense::hover());
            let p = ui.painter();
            let label_right = lr.right() - 10.0;
            match path.split_once('.') {
                Some((pre, f)) => {
                    let fw = text_galley_width(ui, f, 12.0);
                    p.text(Pos2::new(label_right, lr.center().y), Align2::RIGHT_CENTER, f, FontId::proportional(12.0), TEXT());
                    p.text(Pos2::new(label_right - fw, lr.center().y), Align2::RIGHT_CENTER, format!("{pre}."), FontId::proportional(12.0), DIM());
                }
                None => {
                    p.text(Pos2::new(label_right, lr.center().y), Align2::RIGHT_CENTER, path, FontId::proportional(12.0), TEXT());
                }
            }
            let w = ui.available_width() - 2.0;
            match kind {
                "text" | "type" => {
                    // type and stereotype: the linked element's icon, then a picker button
                    let var = (kind == "type" || field == "stereotype") && single;
                    let linked = if var { self.repo.raw(obj, &field).and_then(|v| self.repo.by_ref(v)).filter(|t| self.repo.alive(*t)) } else { None };
                    let mut fw = w;
                    if let Some(t) = linked {
                        let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        icons::draw_for(ui.painter(), ir, &self.repo, t);
                        fw -= 16.0 + ui.spacing().item_spacing.x;
                    }
                    let combo = field == "multiplicity";
                    if var || combo {
                        fw -= 24.0 + ui.spacing().item_spacing.x;
                    }
                    let create = var && kind == "type" && field == "type";
                    if create {
                        fw -= 24.0 + ui.spacing().item_spacing.x;
                    }
                    let cur = merged.clone().unwrap_or_default();
                    let buf = self.prop_buf.entry(path.to_string()).or_insert(cur.clone());
                    let r = ui.add_sized(Vec2::new(fw, 23.0), egui::TextEdit::singleline(buf).hint_text("—").font(FontId::proportional(12.0)).margin(Margin::symmetric(9, 3)));
                    let typed = buf.clone();
                    // type and stereotype names autocomplete while typed (property-editor-view.js:575-653)
                    let over_list = egui::Id::new(("autocomplete-over", path));
                    let was_over = ui.ctx().data(|d| d.get_temp::<bool>(over_list)).unwrap_or(false);
                    if var && (ui.memory(|m| m.has_focus(r.id)) || was_over) && !typed.trim().is_empty() && typed != cur {
                        if let Some(e) = self.autocomplete_list(ui, &r, over_list, &field, &typed) {
                            self.apply_picked(obj, &field, Some(e));
                            r.surrender_focus();
                        }
                    } else {
                        ui.ctx().data_mut(|d| d.insert_temp(over_list, false));
                    }
                    // a click in the list is a pick, not the end of typing
                    if r.lost_focus() && !(var && was_over) {
                        let text = typed.clone();
                        if single {
                            self.commit_text(obj, &field, kind, text);
                        } else if text != cur {
                            self.commit_all(&targets, kind, text);
                        }
                    }
                    if var && ui.add_sized(Vec2::new(24.0, 23.0), egui::Button::new("…")).on_hover_text("Select Element").clicked() {
                        self.open_element_picker(obj, &field);
                    }
                    // Create Type: a new class, named in a dialog, becomes the type (uml-commands.js:1606)
                    if create && ui.add_sized(Vec2::new(24.0, 23.0), egui::Button::new("+")).on_hover_text("Create Type").clicked() {
                        self.input_dialog = Some(crate::input::InputDialog { message: "Enter type name".into(), text: String::new(), action: crate::input::InputAction::CreateType(obj), fresh: true });
                    }
                    if combo {
                        // an editable field plus the usual values (property-editor-view.js:445-479)
                        let mut picked = None;
                        ui.menu_button("▾", |ui| {
                            for o in MULTIPLICITIES {
                                if ui.button(o).clicked() {
                                    picked = Some(o.to_string());
                                    ui.close();
                                }
                            }
                        });
                        if let Some(o) = picked {
                            self.prop_buf.insert(path.to_string(), o.clone());
                            self.commit_all(&targets, kind, o);
                        }
                    }
                }
                "multiline" => {
                    // a 70 px text area, committed on blur (property-editor-view.js:215-234)
                    let cur = merged.clone().unwrap_or_default();
                    let buf = self.prop_buf.entry(path.to_string()).or_insert(cur.clone());
                    let r = ui.add_sized(Vec2::new(w, 70.0), egui::TextEdit::multiline(buf).hint_text("—").font(FontId::proportional(12.0)).margin(Margin::symmetric(9, 3)));
                    if r.lost_focus() && *buf != cur {
                        let text = buf.clone();
                        self.commit_all(&targets, "text", text);
                    }
                }
                "bool" => {
                    // mixed values show unchecked; a click sets them all
                    let mut v = targets.iter().all(|(o, f)| self.repo.bool(*o, f));
                    if ui.checkbox(&mut v, "").changed() {
                        self.change_all(&targets, |a, o, f| a.repo.set(o, f, json!(v)));
                    }
                }
                "enum" => {
                    let cur = merged.clone().unwrap_or_default();
                    let mut sel = cur.clone();
                    let shown = if merged.is_some() { sel.clone() } else { "—".to_string() };
                    egui::ComboBox::from_id_salt(path).selected_text(shown).width(w).show_ui(ui, |ui| {
                        for o in Self::enum_options(self.repo.ty(obj), &field) {
                            ui.selectable_value(&mut sel, o.to_string(), *o);
                        }
                    });
                    if sel != cur {
                        self.change_all(&targets, |a, o, f| a.repo.set(o, f, json!(sel)));
                    }
                }
                // a reference list: its elements, for one element only, and "…" for the Element List
                // Editor (property-editor-view.js:550-564)
                "reflist" => {
                    if single {
                        let items = self.repo.ref_list(obj, &field);
                        ui.vertical(|ui| {
                            for x in &items {
                                ui.horizontal(|ui| {
                                    let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                                    icons::draw_for(ui.painter(), ir, &self.repo, *x);
                                    ui.label(egui::RichText::new(self.repo.name(*x)).size(12.0).color(TEXT()));
                                });
                            }
                            if items.is_empty() {
                                ui.label(egui::RichText::new("—").size(12.0).color(DIM()));
                            }
                        });
                        if ui.small_button("…").on_hover_text("Edit the list").clicked() {
                            self.list_editor = Some((obj, path, None));
                        }
                    }
                }
                "owner" => {
                    // an attribute of the classifier at the other end (uml-commands.js:1465)
                    let attrs = self.owner_attribute_options(obj);
                    let cur = self.repo.refd(obj, &field);
                    let shown = cur.map(|c| self.repo.name(c)).unwrap_or_else(|| "—".to_string());
                    let mut pick: Option<Option<Eid>> = None;
                    ui.add_enabled_ui(single, |ui| {
                        egui::ComboBox::from_id_salt(path).selected_text(shown).width(w).show_ui(ui, |ui| {
                            if ui.selectable_label(cur.is_none(), "(none)").clicked() {
                                pick = Some(None);
                            }
                            for a in attrs {
                                if ui.selectable_label(cur == Some(a), self.repo.name(a)).clicked() {
                                    pick = Some(Some(a));
                                }
                            }
                        });
                    });
                    if let Some(p) = pick.filter(|p| *p != cur) {
                        let f = field.clone();
                        self.change(|a| a.repo.set_ref(obj, &f, p));
                    }
                }
                // a picture: "(Image)" or "—", a browse button and a delete button (property-editor-view.js:342-402)
                "image" => {
                    let set = !self.repo.str(obj, &field).is_empty();
                    ui.label(egui::RichText::new(if set { "(Image)" } else { "—" }).size(12.0).color(if set { TEXT() } else { DIM() }));
                    if single && ui.add_sized(Vec2::new(24.0, 23.0), egui::Button::new("…")).on_hover_text("Select an image").clicked() {
                        if let Some(path) = rfd::FileDialog::new().set_title("Select an image").add_filter("Images", &["jpg", "jpeg", "png", "gif"]).pick_file() {
                            match crate::picture::load_icon_picture(&path) {
                                Ok((data, w, h, ow, oh)) => {
                                    if ow > crate::picture::ICON_MAX || oh > crate::picture::ICON_MAX {
                                        self.message = Some(("Image".into(), "Image is too large, so it will be resized smaller.".into()));
                                    }
                                    let value = json!({ "width": w, "height": h, "data": data }).to_string();
                                    let f = field.clone();
                                    self.change(|a| a.repo.set(obj, &f, json!(value)));
                                }
                                Err(e) => self.toast_error(&e),
                            }
                        }
                    }
                    if set && single && ui.add_sized(Vec2::new(24.0, 23.0), egui::Button::new("×")).on_hover_text("Delete the image").clicked() {
                        // a confirmation box (property-editor-view.js:365)
                        if self.confirm("Do you want to delete this image?") {
                            let f = field.clone();
                            self.change(|a| a.repo.remove(obj, &f));
                        }
                    }
                }
                // an integer: a typed value that is not a whole number is ignored (property-editor-view.js:274-330)
                // a number read as parseInt or parseFloat would: the number the text starts with
                // ("12px" is 12); text with no number keeps the old value (property-editor-view.js:274-330)
                "int" | "real" => {
                    let cur = merged.clone().unwrap_or_default();
                    let buf = self.prop_buf.entry(path.to_string()).or_insert(cur.clone());
                    let r = ui.add_sized(Vec2::new(w, 23.0), egui::TextEdit::singleline(buf).hint_text("—").font(FontId::proportional(12.0)).margin(Margin::symmetric(9, 3)));
                    if r.lost_focus() && *buf != cur {
                        let v = if kind == "int" { parse_int(buf).map(|n| json!(n)) } else { parse_real(buf).map(|n| json!(n)) };
                        match v {
                            Some(v) => self.change_all(&targets, |a, o, f| a.repo.set(o, f, v.clone())),
                            None => *buf = cur,
                        }
                    }
                }
                // a hyperlink's element: its icon and name, and "…" for the Element Picker
                "link" => {
                    if let Some(t) = self.repo.refd(obj, &field).filter(|t| self.repo.alive(*t)) {
                        let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        icons::draw_for(ui.painter(), ir, &self.repo, t);
                        ui.label(egui::RichText::new(self.repo.name(t)).size(12.0).color(TEXT()));
                    } else {
                        ui.label(egui::RichText::new("—").size(12.0).color(DIM()));
                    }
                    if single && ui.add_sized(Vec2::new(24.0, 23.0), egui::Button::new("…")).on_hover_text("Select Element").clicked() {
                        self.open_element_picker(obj, &field);
                    }
                }
                // a reference: the element's icon and name, or "—"; a search button opens the Element
                // Picker on the attribute's type, except for the read-only source, target and end
                // reference (property-editor-view.js:490-539)
                "ref" => {
                    let shared = targets.iter().map(|(o, f)| self.repo.refd(*o, f)).collect::<Vec<_>>();
                    let read_only = matches!(field.as_str(), "source" | "target" | "reference");
                    let bw = if read_only || !single { 0.0 } else { 28.0 };
                    ui.allocate_ui_with_layout(Vec2::new(w - bw, 23.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.set_min_width(w - bw);
                        match shared[0].filter(|_| shared.iter().all(|x| *x == shared[0])) {
                            Some(t) => {
                                let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                                icons::draw_for(ui.painter(), ir, &self.repo, t);
                                ui.label(egui::RichText::new(self.repo.name(t)).size(12.0).color(TEXT()));
                            }
                            None => {
                                ui.label(egui::RichText::new("—").size(12.0).color(DIM()));
                            }
                        }
                    });
                    if bw > 0.0 && ui.add_sized(Vec2::new(24.0, 23.0), egui::Button::new("…")).on_hover_text("Select Element").clicked() {
                        let ty = crate::picker::field_type(&field);
                        self.open_typed_picker(obj, &field, ty);
                    }
                }
                _ => {}
            }
        });
    }

    /// The attributes an association end's ownerAttribute can be: those of the classifier at the
    /// other end (uml-commands.js:1465).
    pub(crate) fn owner_attribute_options(&self, end: Eid) -> Vec<Eid> {
        let other = self.repo.parent(end).and_then(|a| {
            let mine = self.repo.field_of(end);
            let k = if mine.as_deref() == Some("end1") { "end2" } else { "end1" };
            self.repo.child(a, k).and_then(|x| self.repo.refd(x, "reference"))
        });
        other.map(|c| self.repo.list(c, "attributes")).unwrap_or_default()
    }

    /// Commit a text to several elements as one undo step.
    pub(crate) fn commit_all(&mut self, targets: &[(Eid, String)], kind: &str, text: String) {
        let undo = self.undo.len();
        for (obj, field) in targets {
            self.commit_text(*obj, field, kind, text.clone());
        }
        // one step for the whole change
        if self.undo.len() > undo + 1 {
            self.undo.truncate(undo + 1);
        }
    }

    /// The elements the Properties section edits: the models of the selected views when there are
    /// several, otherwise the focused element.
    pub(crate) fn prop_elements(&self) -> Vec<Eid> {
        let mut models: Vec<Eid> = vec![];
        for v in &self.selection {
            if let Some(m) = self.repo.refd(*v, "model").filter(|m| self.repo.alive(*m)) {
                if !models.contains(&m) {
                    models.push(m);
                }
            }
        }
        if models.len() > 1 {
            return models;
        }
        self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f)).into_iter().collect()
    }

    /// The fields several elements share: those of the first, kept when every other has the same one.
    pub(crate) fn common_fields(&self, es: &[Eid]) -> Vec<(&'static str, &'static str)> {
        let Some(first) = es.first() else { return vec![] };
        let rest: Vec<Vec<(&'static str, &'static str)>> = es[1..].iter().map(|e| self.fields_for(*e)).collect();
        self.fields_for(*first).into_iter().filter(|f| f.0 != "-" || es.len() == 1).filter(|f| rest.iter().all(|r| r.contains(f))).collect()
    }

    fn editors_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(HEADER_HEIGHT);
            ui.add_space(13.0);
            ui.label(egui::RichText::new("EDITORS").color(TEXT()).size(13.0));
        });
        let focus = self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f));
        let es = self.prop_elements();
        if self.prop_for != focus || self.prop_set != es {
            self.prop_buf.clear();
            self.prop_for = focus;
            self.prop_set = es.clone();
            self.doc_buf = focus.map(|f| self.repo.str(f, "documentation")).unwrap_or_default();
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Frame::NONE.inner_margin(Margin { left: 13, right: 13, top: 0, bottom: 10 }).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                // Styles comes first and only shows for a view selection (style-editor-view.js:666-668)
                let views_selected = self.selection.iter().any(|v| self.repo.alive(*v));
                if views_selected {
                    if section_header(ui, "Styles", self.style_open) {
                        self.style_open = !self.style_open;
                    }
                    let mut fold = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), egui::Id::new("editor-styles"), self.style_open);
                    fold.set_open(self.style_open);
                    fold.show_body_unindented(ui, |ui| {
                        self.style_ui(ui);
                        ui.add_space(6.0);
                    });
                    ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), Stroke::new(1.0, BORDER()));
                }
                // Properties hides with nothing selected (property-editor-view.js:815-817)
                if !es.is_empty() && section_header(ui, "Properties", self.props_open) {
                    self.props_open = !self.props_open;
                }
                // the body folds and unfolds with a slide (editors-holder-view.js:61-87)
                if !es.is_empty() {
                    let open = self.props_open;
                    let mut fold = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), egui::Id::new("editor-properties"), open);
                    fold.set_open(open);
                    fold.show_body_unindented(ui, |ui| {
                    if es.len() > 1 {
                        // several elements: the fields they share, merged
                        ui.label(egui::RichText::new(format!("{} elements", es.len())).size(12.0).color(DIM()));
                        for (path, kind) in self.common_fields(&es) {
                            self.prop_row(ui, &es, path, kind);
                        }
                    } else if let Some(e) = focus {
                        for (path, kind) in self.fields_for(e) {
                            if path == "-" {
                                ui.add_space(8.0);
                                ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), Stroke::new(1.0, BORDER()));
                                ui.add_space(8.0);
                            } else {
                                self.prop_row(ui, &[e], path, kind);
                            }
                        }
                        let t = self.repo.ty(e).to_string();
                        let mut buttons: Vec<(&str, &str)> = vec![];
                        if t == "UMLEnumeration" {
                            buttons.push(("+ Literal", "UMLEnumerationLiteral"));
                        }
                        if self.can_add_feature(e, "UMLAttribute") {
                            buttons.push(("+ Attribute", "UMLAttribute"));
                            buttons.push(("+ Operation", "UMLOperation"));
                        }
                        if t == "UMLOperation" {
                            buttons.push(("+ Parameter", "UMLParameter"));
                        }
                        if !buttons.is_empty() {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.add_space(116.0);
                                for (label, ty) in buttons {
                                    if ui.add(egui::Button::new(egui::RichText::new(label).size(12.0)).fill(HOVER_ROW()).stroke(Stroke::new(1.0, INPUT_BORDER()))).clicked() {
                                        if let Some(c) = self.change(|a| a.add_feature(e, ty)) {
                                            self.focus = Some(c);
                                            self.expanded.insert(e);
                                        }
                                    }
                                }
                            });
                            ui.add_space(6.0);
                        }
                    }
                    });
                }
                if !es.is_empty() {
                    ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), Stroke::new(1.0, BORDER()));
                }
                // Documentation shows only for one element that can hold it (documentation-editor-view.js:41-50)
                let has_doc = es.len() <= 1 && focus.map(|f| self.repo.kind_of(f, "ExtensibleModel") || self.repo.has(f, "documentation") || !crate::meta::known(self.repo.ty(f))).unwrap_or(false);
                if has_doc && section_header(ui, "Documentation", self.doc_open) {
                    self.doc_open = !self.doc_open;
                }
                if let (Some(e), true) = (focus, has_doc) {
                    let mut fold = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), egui::Id::new("editor-documentation"), self.doc_open);
                    fold.set_open(self.doc_open);
                    fold.show_body_unindented(ui, |ui| {
                        let r = ui.add_sized(Vec2::new(ui.available_width(), 180.0), egui::TextEdit::multiline(&mut self.doc_buf).font(FontId::proportional(13.0)));
                        if r.lost_focus() && self.repo.str(e, "documentation") != self.doc_buf {
                            let text = self.doc_buf.clone();
                            self.change(|a| a.repo.set(e, "documentation", json!(text)));
                        }
                    });
                }
            });
        });
    }

    // ------------------------------------------------------------ sidebar
    fn working_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(HEADER_HEIGHT);
            ui.add_space(13.0);
            ui.label(egui::RichText::new("WORKING DIAGRAMS").color(TEXT()).size(13.0));
        });
        ui.spacing_mut().item_spacing.y = 0.0;
        for d in self.open.clone() {
            if !self.repo.alive(d) {
                continue;
            }
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click_and_drag());
            // rows can be dragged into a new order (working-diagrams-view.js:204-205)
            resp.dnd_set_drag_payload(WorkingRow(d));
            if let Some(src) = resp.dnd_hover_payload::<WorkingRow>() {
                if src.0 != d {
                    ui.painter().hline(rect.x_range(), rect.top(), Stroke::new(2.0, ACCENT()));
                }
            }
            if let Some(src) = resp.dnd_release_payload::<WorkingRow>() {
                self.move_working(src.0, d);
            }
            let current = self.diagram == Some(d);
            let p = ui.painter();
            if current {
                p.rect_filled(rect, 0.0, CURRENT_ROW());
            } else if resp.hovered() {
                p.rect_filled(rect, 0.0, ROW_HOVER());
            }
            icons::draw(p, Rect::from_min_size(Pos2::new(rect.left() + 28.0, rect.top() + 7.0), Vec2::splat(16.0)), icons::key_for(&self.repo, d));
            let name = self.repo.name(d);
            let nx = rect.left() + 46.0;
            p.text(Pos2::new(nx, rect.center().y), Align2::LEFT_CENTER, &name, FontId::proportional(13.0), if current { ACCENT_TEXT() } else { TEXT() });
            let nw = text_galley_width(ui, &name, 13.0);
            if let Some(owner) = self.repo.parent(d).map(|o| self.repo.name(o)).filter(|o| !o.is_empty()) {
                p.text(Pos2::new(nx + nw + 4.0, rect.center().y + 1.0), Align2::LEFT_CENTER, format!("— {owner}"), FontId::proportional(11.0), DIM());
            }
            let close = Rect::from_center_size(Pos2::new(rect.right() - 16.0, rect.center().y), Vec2::splat(16.0));
            if resp.hovered() {
                p.text(close.center(), Align2::CENTER_CENTER, "×", FontId::proportional(14.0), DIM());
            }
            // the menu acts on the current diagram; right-clicking a row does not make it current
            resp.context_menu(|ui| {
                self.diagram_close_items(ui);
                ui.separator();
                if ui.add_enabled(self.diagram.is_some(), egui::Button::new("Rename")).clicked() {
                    self.rename_diagram();
                    ui.close();
                }
            });
            if resp.clicked() {
                if resp.interact_pointer_pos().map(|q| close.contains(q)).unwrap_or(false) {
                    self.close_diagram(Some(d));
                } else {
                    self.open_diagram(d);
                    self.focus_element(d);
                }
            }
        }
    }

    /// Move a working diagram to the place of another (dropped on its row), keeping the current one.
    pub fn move_working(&mut self, d: Eid, before: Eid) {
        if d == before {
            return;
        }
        let Some(from) = self.open.iter().position(|x| *x == d) else { return };
        let Some(to) = self.open.iter().position(|x| *x == before) else { return };
        let item = self.open.remove(from);
        self.open.insert(to, item);
    }

    fn toolbox_ui(&mut self, ui: &mut egui::Ui) {
        let top = ui.cursor().top();
        ui.painter().text(Pos2::new(ui.max_rect().left() + 13.0, top + 17.0), Align2::LEFT_CENTER, "TOOLBOX", FontId::proportional(13.0), TEXT());
        ui.add_space(26.0);
        // without a diagram the toolbox is empty (toolbox-view.js:307-310)
        if self.diagram.is_none() {
            return;
        }
        // a group opens by itself when a diagram of its type becomes current (toolbox-view.js:298-305)
        let dty = self.diagram.map(|d| self.repo.ty(d).to_string());
        if dty != self.toolbox_for {
            match dty.as_deref() {
                Some("UMLClassDiagram") => self.open_groups.insert("class"),
                Some("UMLUseCaseDiagram") => self.open_groups.insert("usecase"),
                Some("UMLPackageDiagram") => self.open_groups.insert("package"),
                Some("UMLComponentDiagram") => self.open_groups.insert("component"),
                Some("UMLDeploymentDiagram") => self.open_groups.insert("deployment"),
                Some("UMLObjectDiagram") => self.open_groups.insert("instance"),
                Some("UMLInformationFlowDiagram") => self.open_groups.insert("information-flow"),
                Some("UMLProfileDiagram") => self.open_groups.insert("profile"),
                Some("UMLCompositeStructureDiagram") => self.open_groups.insert("compositestructure"),
                Some("UMLStatechartDiagram") => self.open_groups.insert("statechart"),
                Some("UMLActivityDiagram") => self.open_groups.insert("activity"),
                Some("FCFlowchartDiagram") => self.open_groups.insert("flowchart"),
                Some("DFDDiagram") => self.open_groups.insert("dfd"),
                Some("MMMindmapDiagram") => self.open_groups.insert("mindmap"),
                Some("ERDDiagram") => self.open_groups.insert("erd"),
                Some("C4Diagram") => self.open_groups.insert("c4"),
                // Basic, Artifact and Swimlanes open on a BPMN diagram ("auto-expands")
                Some("BPMNDiagram") => self.open_groups.insert("bpmn.basic") | self.open_groups.insert("bpmn.artifact") | self.open_groups.insert("bpmn.swimlanes"),
                Some("AWSDiagram") => self.open_groups.insert("aws.elements"),
                Some("AzureDiagram") => self.open_groups.insert("azure.elements"),
                Some("GCPDiagram") => self.open_groups.insert("gcp.elements"),
                Some("SysMLRequirementDiagram") => self.open_groups.insert("sysml.requirement"),
                Some("SysMLBlockDefinitionDiagram") => self.open_groups.insert("sysml.block"),
                Some("SysMLInternalBlockDiagram") => self.open_groups.insert("sysml.internal-block"),
                Some("SysMLParametricDiagram") => self.open_groups.insert("sysml.parametric"),
                // both groups open on a wireframe diagram ("auto-expands")
                Some("WFWireframeDiagram") => self.open_groups.insert("wireframe.frames") | self.open_groups.insert("wireframe.components"),
                Some("UMLSequenceDiagram") => self.open_groups.insert("sequence"),
                Some("UMLCommunicationDiagram") => self.open_groups.insert("communication"),
                Some("UMLInteractionOverviewDiagram") => self.open_groups.insert("interaction-overview"),
                Some("UMLTimingDiagram") => self.open_groups.insert("timing"),
                _ => false,
            };
            self.toolbox_for = dty;
        }
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 26.0) / 2.0);
            let (r, resp) = ui.allocate_exact_size(Vec2::new(26.0, 24.0), Sense::click());
            if self.tool.is_none() {
                ui.painter().rect_filled(r, CornerRadius::same(2), TOOLBAR());
            }
            icons::draw(ui.painter(), Rect::from_center_size(r.center(), Vec2::splat(16.0)), "select");
            if resp.clicked() {
                self.tool = None;
            }
        });
        ui.add_space(5.0);
        let groups: &[Group] = tool_groups(self.diagram.map(|d| self.repo.ty(d)));
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for g in groups {
                let open = self.open_groups.contains(g.id);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
                let p = ui.painter();
                p.text(Pos2::new(rect.left() + 13.0, rect.center().y), Align2::LEFT_CENTER, g.label, FontId::proportional(13.0), DIM());
                p.hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, BORDER()));
                let c = Pos2::new(rect.right() - 16.0, rect.center().y);
                let tri = if open {
                    vec![c + Vec2::new(-4.0, 3.0), c + Vec2::new(4.0, 3.0), c + Vec2::new(0.0, -3.0)]
                } else {
                    vec![c + Vec2::new(-4.0, -3.0), c + Vec2::new(4.0, -3.0), c + Vec2::new(0.0, 3.0)]
                };
                p.add(egui::Shape::convex_polygon(tri, TEXT(), Stroke::NONE));
                if resp.clicked() {
                    if open {
                        self.open_groups.remove(g.id);
                    } else {
                        self.open_groups.insert(g.id);
                    }
                    // opening or closing a group goes back to Select (toolbox-view.js:446-449)
                    self.tool = None;
                    self.tool_sticky = false;
                }
                if !open {
                    continue;
                }
                for it in g.items {
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.5), Sense::click());
                    let p = ui.painter();
                    let selected = self.tool == Some(*it);
                    if selected {
                        p.rect_filled(rect, 0.0, ACCENT());
                    } else if resp.hovered() {
                        p.rect_filled(rect, 0.0, HOVER_ROW());
                    }
                    icons::draw(p, Rect::from_min_size(Pos2::new(rect.left() + 14.0, rect.top() + 8.0), Vec2::splat(16.0)), it.icon);
                    p.text(Pos2::new(rect.left() + 34.0, rect.center().y), Align2::LEFT_CENTER, it.label, FontId::proportional(13.0), if selected && !light_theme() { Color32::WHITE } else { TEXT() });
                    if selected && self.tool_sticky {
                        draw_lock(p, Pos2::new(rect.right() - 18.0, rect.center().y));
                    }
                    if resp.clicked() {
                        self.toolbox_click(*it);
                    }
                }
            }
        });
    }

    /// A click on a toolbox item: select it; on the active item, lock it; on a locked item, unlock it and
    /// go back to Select. A double-click therefore locks (toolbox-view.js:451-457).
    pub(crate) fn toolbox_click(&mut self, it: crate::app::Tool) {
        if self.tool != Some(it) {
            self.tool = Some(it);
            self.tool_sticky = false;
        } else if !self.tool_sticky {
            self.tool_sticky = true;
        } else {
            self.tool = None;
            self.tool_sticky = false;
        }
    }

    fn toolbar_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            // the panel buttons in the order their extensions load (diagram-thumbnails, markdown,
            // minimap, relationship-view: diagram-thumbnails/main.js:44-46, markdown/main.js:42-44,
            // minimap/main.js:35, relationship-view/main.js:34)
            for (key, tip) in [("zoomIn", "Zoom In"), ("zoomOut", "Zoom Out"), ("fit", "Fit To Window"), ("grid", "Show Grid"), ("image", "Export Diagram as PNG"), ("thumbnails", "Diagram Thumbnails"), ("markdown", "Markdown Documentation"), ("minimap", "Minimap"), ("relationships", "Relationship View")] {
                let (r, resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                // a panel's button stays selected while its panel shows (markdown/main.js:66-80,
                // minimap/main.js:47, 60)
                let selected = match key {
                    "markdown" => self.show_markdown,
                    "thumbnails" => self.show_thumbnails,
                    "minimap" => self.show_minimap,
                    "relationships" => self.show_relationships,
                    _ => false,
                };
                if resp.hovered() || selected {
                    ui.painter().rect_filled(r, CornerRadius::same(2), TOOL_HOVER());
                }
                icons::draw(ui.painter(), r.shrink(3.0), key);
                if resp.on_hover_text(tip).clicked() {
                    match key {
                        "zoomIn" => self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP),
                        "zoomOut" => self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP),
                        "fit" => self.fit(),
                        "grid" => self.show_grid = !self.show_grid,
                        "markdown" => self.show_markdown = !self.show_markdown,
                        "thumbnails" => self.show_thumbnails = !self.show_thumbnails,
                        "minimap" => self.show_minimap = !self.show_minimap,
                        "relationships" => self.show_relationships = !self.show_relationships,
                        _ => self.export_current(&ui.ctx().clone(), crate::export::Format::Png),
                    }
                }
            }
        });
    }

    /// Add and Add Diagram, acting on the focused element (or the project): in the Model menu and at
    /// the top of the canvas menu (menus/linux.json "diagram-canvas").
    pub(crate) fn add_menus(&mut self, ui: &mut egui::Ui) {
        let target = self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f)).unwrap_or(self.repo.root);
        ui.add_enabled_ui(!self.add_kinds(target).is_empty() || self.repo.kind_of(target, "ExtensibleModel"), |ui| {
            ui.menu_button("Add", |ui| self.add_items(ui, target));
        });
        ui.menu_button("Add Diagram", |ui| {
            let sel = self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f));
            let kinds: Vec<(&str, &str)> = DIAGRAM_KINDS.into_iter().filter(|(_, k)| crate::sysml::diagram_offered(&self.repo, sel, k).unwrap_or(true)).collect();
            for (l, k) in kinds {
                if ui.button(l).clicked() {
                    let f = self.focus;
                    self.change(|a| a.add_diagram(f, k));
                    ui.close();
                }
            }
        });
    }

    fn status_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let p = ui.painter();
        p.hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, BORDER()));
        // right side: check mark and zoom
        let zr = Rect::from_min_max(Pos2::new(rect.right() - 57.0, rect.top()), rect.right_bottom());
        p.vline(zr.left(), rect.y_range(), Stroke::new(1.0, BORDER()));
        p.text(zr.center(), Align2::CENTER_CENTER, format!("{}%", (self.zoom * 100.0).round()), FontId::proportional(13.0), TEXT());
        let vr = Rect::from_min_max(Pos2::new(zr.left() - 33.0, rect.top()), Pos2::new(zr.left(), rect.bottom()));
        p.vline(vr.left(), rect.y_range(), Stroke::new(1.0, BORDER()));
        // validation indicator: green OK, or red when the last validation found errors; a click toggles
        // the Validation Results panel (statusbar-view.js:62-70)
        let c = vr.center();
        let errors = self.problems.iter().any(|q| self.repo.alive(q.elem));
        if errors {
            p.circle_filled(c, 7.5, Color32::from_rgb(0xe5, 0x48, 0x4d));
            let st = Stroke::new(2.0, LAYER0());
            p.line_segment([c + Vec2::new(-3.0, -3.0), c + Vec2::new(3.0, 3.0)], st);
            p.line_segment([c + Vec2::new(-3.0, 3.0), c + Vec2::new(3.0, -3.0)], st);
        } else {
            p.circle_filled(c, 7.5, OK_GREEN);
            p.add(egui::Shape::line(vec![c + Vec2::new(-3.5, 0.0), c + Vec2::new(-1.0, 2.5), c + Vec2::new(3.5, -2.5)], Stroke::new(2.0, LAYER0())));
        }
        let tip = if errors { "Validation errors" } else { "Validation OK" };
        if ui.interact(vr, egui::Id::new("validation-indicator"), Sense::click()).on_hover_text(tip).clicked() {
            self.show_validation = !self.show_validation;
        }
        // theme button: a moon in the dark theme, a sun in the light one; a click switches theme.default
        let tr = Rect::from_min_max(Pos2::new(vr.left() - 33.0, rect.top()), Pos2::new(vr.left(), rect.bottom()));
        p.vline(tr.left(), rect.y_range(), Stroke::new(1.0, BORDER()));
        let c = tr.center();
        if light_theme() {
            p.circle_filled(c, 4.0, TEXT());
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let d = Vec2::new(a.cos(), a.sin());
                p.line_segment([c + d * 6.0, c + d * 8.0], Stroke::new(1.5, TEXT()));
            }
        } else {
            p.circle_filled(c, 7.0, TEXT());
            p.circle_filled(c + Vec2::new(3.5, -3.0), 6.0, LAYER1());
        }
        let theme_tip = if light_theme() { "Dark Theme" } else { "Light Theme" };
        if ui.interact(tr, egui::Id::new("theme-button"), Sense::click()).on_hover_text(theme_tip).clicked() {
            let next = if light_theme() { "dark" } else { "light" };
            self.prefs.set("theme.default", json!(next));
            self.prefs.flush();
        }
        // left side: the focused element's ownership path, each owner with its icon and name (the project
        // with its icon only), then [Type]; blank with several views selected (statusbar-view.js:78-120)
        let Some(f) = self.focus.filter(|f| self.repo.alive(*f)) else { return };
        if self.selection.len() > 1 {
            return;
        }
        let chain = self.repo.path(f);
        let label = |e: Eid| if e == self.repo.root { String::new() } else { self.label_of(e) };
        let widths: Vec<f32> = chain.iter().map(|e| { let l = label(*e); if l.is_empty() { 28.0 } else { 36.0 + text_galley_width(ui, &l, 13.0) } }).collect();
        // the path drops its outer owners when it does not fit before the right-hand indicators
        let room = (rect.width() - 200.0).max(0.0);
        let mut first = 0;
        while first + 1 < chain.len() && widths[first..].iter().sum::<f32>() > room {
            first += 1;
        }
        let mut x = rect.left();
        for (i, e) in chain.iter().enumerate().skip(first) {
            let l = label(*e);
            let p = ui.painter();
            icons::draw(p, Rect::from_min_size(Pos2::new(x + 8.0, rect.center().y - 8.0), Vec2::splat(16.0)), icons::key_for(&self.repo, *e));
            if !l.is_empty() {
                p.text(Pos2::new(x + 28.0, rect.center().y), Align2::LEFT_CENTER, &l, FontId::proportional(13.0), TEXT());
            }
            x += widths[i];
            p.vline(x, rect.y_range(), Stroke::new(1.0, BORDER()));
        }
        ui.painter().text(Pos2::new(x + 8.0, rect.center().y), Align2::LEFT_CENTER, format!("[{}]", self.repo.ty(f)), FontId::proportional(11.0), DIM());
    }

    // ------------------------------------------------------------ menus and shortcuts
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let measure = EguiMeasure { ctx: ui.ctx().clone() };
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.add(self.key_button("New", "application:new")).clicked() {
                    self.cmd_new(&measure);
                    ui.close();
                }
                ui.menu_button("New From Template", |ui| {
                    for t in &crate::templates::TEMPLATES {
                        if ui.button(t.label).clicked() {
                            // a new window with the template (application.js:249-253)
                            self.new_window_from_template(t.label);
                            ui.close();
                        }
                    }
                });
                if ui.add(self.key_button("Open...", "project:open")).clicked() {
                    self.cmd_open(&measure);
                    ui.close();
                }
                let recent = self.recent_files();
                ui.add_enabled_ui(!recent.is_empty(), |ui| {
                    ui.menu_button("Open Recent", |ui| {
                        for p in recent {
                            if ui.button(p.display().to_string()).clicked() {
                                self.open_recent(&measure, p);
                                ui.close();
                            }
                        }
                    });
                });
                ui.separator();
                if ui.add(self.key_button("Save", "project:save")).clicked() {
                    self.save(&measure);
                    ui.close();
                }
                if ui.add(self.key_button("Save As...", "project:save-as")).clicked() {
                    self.save_as(&measure);
                    ui.close();
                }
                ui.separator();
                // project:close closes the window, after the save prompt (default-commands.js:345-347)
                if ui.button("Close").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
                ui.separator();
                if ui.add(self.key_button("Export Image...", "project:export-image")).clicked() {
                    self.open_export_image();
                    ui.close();
                }
                ui.menu_button("Export Diagram As", |ui| {
                    use crate::export::Format;
                    for f in [Format::Png, Format::Jpeg, Format::Svg] {
                        if ui.button(format!("{}...", f.label())).clicked() {
                            self.export_current(&ui.ctx().clone(), f);
                            ui.close();
                        }
                    }
                    ui.separator();
                    for f in [Format::Png, Format::Jpeg, Format::Svg] {
                        if ui.button(format!("All to {}s...", f.label())).clicked() {
                            self.export_all_cmd(&ui.ctx().clone(), f);
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Import", |ui| {
                    if ui.button("Fragment...").clicked() {
                        self.import_fragment_dialog();
                        ui.close();
                    }
                    if ui.button("XMI Import (v2.1)...").clicked() {
                        self.import_xmi_dialog();
                        ui.close();
                    }
                    if ui.button("StarUML 1 File (.uml)...").clicked() {
                        self.cmd_import_staruml_v1(&measure);
                        ui.close();
                    }
                });
                ui.menu_button("Export", |ui| {
                    if ui.button("Fragment...").clicked() {
                        self.export_fragment_dialog();
                        ui.close();
                    }
                    if ui.button("HTML Docs...").clicked() {
                        self.export_html_dialog(&measure);
                        ui.close();
                    }
                    if ui.button("XMI Export (v2.1)...").clicked() {
                        self.export_xmi_dialog();
                        ui.close();
                    }
                    if ui.button("StarUML V7...").clicked() {
                        self.export_v7_dialog(&measure);
                        ui.close();
                    }
                });
                if ui.add(self.key_button("Print to PDF...", "file:print-to-pdf")).clicked() {
                    self.print_open = true;
                    ui.close();
                }
                ui.separator();
                if ui.add(self.key_button("Preferences...", "application:preferences")).clicked() {
                    self.prefs_open = Some("General".into());
                    ui.close();
                }
                ui.separator();
                if ui.add(self.key_button("Exit", "application:quit")).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(!self.undo.is_empty(), self.key_button("Undo", "edit:undo")).clicked() {
                    self.do_undo();
                    ui.close();
                }
                if ui.add_enabled(!self.redo.is_empty(), self.key_button("Redo", "edit:redo")).clicked() {
                    self.do_redo();
                    ui.close();
                }
                ui.separator();
                self.clipboard_items(ui);
                if ui.add(self.key_button("Delete", "edit:delete")).clicked() {
                    self.delete_selection(false);
                    ui.close();
                }
                // enabled when the selected model can be deleted (default-commands.js:1319-1324)
                let sel: Vec<Eid> = self.selection.iter().copied().filter(|v| self.repo.alive(*v)).collect();
                let can = if sel.is_empty() { self.focus.is_some_and(|f| self.repo.alive(f) && self.repo.ty(f) != "Project") } else { sel.iter().any(|v| self.repo.refd(*v, "model").is_some()) };
                if ui.add_enabled(can, self.key_button("Delete From Model", "edit:delete-from-model")).clicked() {
                    self.delete_selection(true);
                    ui.close();
                }
                ui.separator();
                if ui.add(self.key_button("Select All", "edit:select-all")).clicked() {
                    self.select_all();
                    ui.close();
                }
                if ui.add(self.key_button("Select In Explorer", "edit:select-in-explorer")).clicked() {
                    if let Some(f) = self.focus {
                        self.reveal(f);
                    }
                    ui.close();
                }
                if ui.add_enabled(self.focus.is_some(), self.key_button("Select In Diagram", "edit:select-in-diagram")).clicked() {
                    self.select_in_diagram();
                    ui.close();
                }
                if ui.add_enabled(self.focus.is_some(), self.key_button("Open Sub-Diagram", "edit:open-sub-diagram")).clicked() {
                    self.open_sub_diagram();
                    ui.close();
                }
                ui.separator();
                self.move_items(ui);
            });
            ui.menu_button("Format", |ui| {
                if ui.add_enabled(!self.selection.is_empty(), self.key_button("Font...", "format:font")).clicked() {
                    self.open_font_dialog();
                    ui.close();
                }
                self.color_menus(ui);
                self.line_style_menu(ui);
                self.stereotype_display_menu(ui);
                self.format_toggles(ui);
                ui.separator();
                self.alignment_menu(ui);
                ui.add_enabled_ui(self.diagram.is_some(), |ui| {
                    ui.menu_button("Layout", |ui| {
                        for (dir, label) in crate::layout::Dir::MENU {
                            if ui.button(label).clicked() {
                                self.layout_diagram(dir);
                                ui.close();
                            }
                        }
                    });
                });
            });
            ui.menu_button("Model", |ui| {
                self.add_menus(ui);
                if ui.add(self.key_button("Find...", "find:find")).clicked() {
                    self.open_find();
                    ui.close();
                }
                ui.menu_button("Apply Profile", |ui| {
                    if ui.button("UML Standard Profile (v2)").clicked() {
                        self.apply_standard_profile();
                        ui.close();
                    }
                });
                let taggable = self.focus.map(|f| self.repo.alive(f) && self.repo.kind_of(f, "ExtensibleModel")).unwrap_or(false);
                if ui.add_enabled(taggable, self.key_button("Tag Editor...", "model:tag-editor")).clicked() {
                    self.open_tag_editor();
                    ui.close();
                }
                ui.separator();
                for (label, ty) in [("Add Attribute", "UMLAttribute"), ("Add Operation", "UMLOperation"), ("Add Literal", "UMLEnumerationLiteral")] {
                    let ok = self.focus.map(|f| self.can_add_feature(f, ty)).unwrap_or(false);
                    if ui.add_enabled(ok, egui::Button::new(label)).clicked() {
                        let f = self.focus.unwrap();
                        if let Some(c) = self.change(|a| a.add_feature(f, ty)) {
                            self.focus = Some(c);
                        }
                        ui.close();
                    }
                }
            });
            ui.menu_button("Tools", |ui| {
                // a dialog per language with Generate and Reverse tabs (docs.staruml.io language-support)
                for l in crate::codegen::Lang::ALL {
                    if ui.button(format!("{}...", l.label())).clicked() {
                        self.open_codegen(l);
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Generate Diagram by Mermaid...").clicked() {
                    self.mermaid_code = Some(crate::mermaid::SAMPLE.to_string());
                    ui.close();
                }
                ui.menu_button("Diagram Generator", |ui| {
                    for (g, label) in crate::generate::Generator::MENU {
                        if ui.button(label).clicked() {
                            self.generate_diagram(g, &measure);
                            ui.close();
                        }
                    }
                });
            });
            ui.menu_button("View", |ui| {
                if ui.add(self.key_button("Quick Find...", "view:quick-find")).clicked() {
                    self.open_palette(crate::palette::Mode::Find);
                    ui.close();
                }
                if ui.add(self.key_button("Command Palette...", "view:command-palette")).clicked() {
                    self.open_palette(crate::palette::Mode::Commands);
                    ui.close();
                }
                ui.separator();
                self.diagram_close_items(ui);
                ui.separator();
                let many = self.open.len() > 1;
                if ui.add_enabled(many, self.key_button("Next Diagram", "view:next-diagram")).clicked() {
                    self.cycle_diagram(1);
                    ui.close();
                }
                if ui.add_enabled(many, self.key_button("Previous Diagram", "view:previous-diagram")).clicked() {
                    self.cycle_diagram(-1);
                    ui.close();
                }
                ui.separator();
                if ui.add(self.key_button("Zoom In", "view:zoom-in")).clicked() {
                    self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP);
                }
                if ui.add(self.key_button("Zoom Out", "view:zoom-out")).clicked() {
                    self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP);
                }
                if ui.add(self.key_button("Actual Size", "view:actual-size")).clicked() {
                    self.zoom_keep_centre(1.0);
                }
                if ui.add(self.key_button("Fit To Window", "view:fit-to-window")).clicked() {
                    self.fit();
                    ui.close();
                }
                ui.separator();
                let t = self.key_text("Show Grid", "view:show-grid");
                ui.checkbox(&mut self.show_grid, t);
                let t = self.key_text("Snap to Grid", "view:snap-to-grid");
                ui.checkbox(&mut self.snap, t);
                ui.separator();
                let t = self.key_text("Minimap", "minimap:toggle");
                ui.checkbox(&mut self.show_minimap, t);
                let t = self.key_text("Markdown Documentation", "markdown-doc:toggle");
                ui.checkbox(&mut self.show_markdown, t);
                let t = self.key_text("Diagram Thumbnails", "diagram-thumbnails:toggle");
                ui.checkbox(&mut self.show_thumbnails, t);
                let t = self.key_text("Relationships", "relationship-view:toggle");
                ui.checkbox(&mut self.show_relationships, t);
                ui.separator();
                for (i, label) in PANELS.iter().enumerate() {
                    let mut on = *self.panel_flag(i);
                    let t = self.key_text(label, &format!("view:{}", label.to_lowercase()));
                    if ui.checkbox(&mut on, t).clicked() {
                        *self.panel_flag(i) = on;
                    }
                }
            });
            ui.menu_button("Window", |ui| {
                if ui.button("Minimize").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    ui.close();
                }
                if ui.button("Full Screen").clicked() {
                    let full = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!full));
                    ui.close();
                }
                ui.separator();
                if ui.button("Bring All to Front").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
                    ui.close();
                }
            });
            // Help: Documentation, Release Notes and About (default-commands.js:1152-1178)
            ui.menu_button("Help", |ui| {
                if ui.button("Documentation...").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(DOCS_URL));
                    ui.close();
                }
                // the place to ask questions: the project's issues, as there is no forum
                if ui.button("Forum...").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(FORUM_URL));
                    ui.close();
                }
                if ui.button("Release Notes...").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(RELEASES_URL));
                    ui.close();
                }
                ui.separator();
                if ui.button("About UMLCraft").clicked() {
                    self.about_open = true;
                    ui.close();
                }
            });
        });
    }

    /// The Style editor: colour swatches, line style, the show and suppress toggles, z-order and alignment.
    fn style_ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 4.0;
        // font face and size: merged, blank when the views differ (style-editor-view.js:180-227)
        let (face, size) = self.font_state();
        // while the selection inherits its parent's style, font and colours are disabled (style-editor-view.js:741-764)
        let inherit = self.flag_state("parentStyle") == Some(true);
        let disabled = ui.add_enabled_ui(!inherit, |ui| {
            ui.horizontal(|ui| {
                let mut f = face.clone().unwrap_or_default();
                let r = ui.add(egui::TextEdit::singleline(&mut f).hint_text("—").desired_width(90.0).font(FontId::proportional(12.0)));
                if r.lost_focus() && !f.trim().is_empty() && Some(f.trim().to_string()) != face {
                    self.set_font(Some(f.trim()), None);
                }
                // the list of faces: a pick is copied into the field and applied (style-editor-view.js:180-201)
                if let Some(f) = crate::style::font_face_menu(ui, "style-face") {
                    self.set_font(Some(&f), None);
                }
                let label = size.map(|s| format!("{s}")).unwrap_or_else(|| "—".into());
                egui::ComboBox::from_id_salt("style-size").selected_text(label).width(60.0).show_ui(ui, |ui| {
                    for s in crate::style::SIZES {
                        if ui.selectable_label(size == Some(s), format!("{s}")).clicked() {
                            self.set_font(None, Some(s));
                        }
                    }
                });
            });
            ui.horizontal(|ui| {
                // inheriting, the colour icons are drawn at 30% (egui's disabled 50%, times 0.6)
                if inherit {
                    ui.multiply_opacity(0.6);
                }
                for (field, label) in crate::style::COLORS {
                    if self.color_views(field).is_empty() {
                        continue;
                    }
                    self.color_swatch(ui, field, label);
                    ui.label(egui::RichText::new(label.trim_end_matches(" Color")).size(12.0).color(DIM()));
                    ui.add_space(6.0);
                }
            });
        });
        // and the pointer over them is "not allowed" (style-editor-view.js:741-764)
        if inherit && ui.rect_contains_pointer(disabled.response.rect) {
            ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
        }
        if !self.flag_views("stereotypeDisplay").is_empty() {
            // the stereotype display dropdown, empty when mixed (style-editor-view.js)
            let cur = self.stereotype_display_state();
            let shown = crate::style::STEREOTYPE_DISPLAYS.iter().find(|d| Some(d.0) == cur.as_deref()).map(|d| d.1).unwrap_or("");
            egui::ComboBox::from_id_salt("style-stereotype").selected_text(egui::RichText::new(shown).size(12.0)).width(150.0).show_ui(ui, |ui| {
                for (value, label) in crate::style::STEREOTYPE_DISPLAYS {
                    if ui.selectable_label(cur.as_deref() == Some(value), label).clicked() {
                        self.set_stereotype_display(value);
                    }
                }
            });
        }
        // a free line's pattern and end shapes, when only free lines are selected (style-editor-view.js:296-319)
        let free: Vec<Eid> = self.selection.iter().copied().filter(|v| self.repo.alive(*v)).collect();
        if !free.is_empty() && free.iter().all(|v| views::is_freeline(&self.repo, *v)) {
            for (field, label, options) in [
                ("lineMode", "Line", ["Solid", "Dotted", "Dashed"]),
                ("tailEndStyle", "Tail", ["Flat", "Stick Arrow", "Solid Arrow"]),
                ("headEndStyle", "Head", ["Flat", "Stick Arrow", "Solid Arrow"]),
            ] {
                let vals: Vec<i64> = free.iter().map(|v| self.repo.val(*v, field).and_then(|x| x.as_i64()).unwrap_or(0)).collect();
                let cur = vals.iter().all(|x| *x == vals[0]).then_some(vals[0]);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(label).size(12.0).color(DIM()));
                    let shown = cur.and_then(|c| options.get(c as usize)).copied().unwrap_or("");
                    egui::ComboBox::from_id_salt(("freeline", field)).selected_text(egui::RichText::new(shown).size(12.0)).width(110.0).show_ui(ui, |ui| {
                        for (i, o) in options.iter().enumerate() {
                            if ui.selectable_label(cur == Some(i as i64), *o).clicked() {
                                let vs = free.clone();
                                self.change(|a| {
                                    for v in vs {
                                        a.repo.set(v, field, json!(i));
                                    }
                                });
                            }
                        }
                    });
                });
            }
        }
        if self.selection.iter().any(|v| self.repo.alive(*v) && views::is_edge(&self.repo, *v)) {
            ui.horizontal(|ui| {
                let cur = self.line_style_state();
                for (style, label) in [(0, "Rectilinear"), (1, "Oblique"), (2, "Rounded"), (3, "Curve")] {
                    if ui.radio(cur == Some(style), egui::RichText::new(label).size(12.0)).clicked() {
                        self.set_line_style(style);
                    }
                }
            });
        }
        for (field, label, _) in FORMAT_TOGGLES {
            if self.flag_views(field).is_empty() || crate::sysml::SUPPRESS_FLAGS.contains(&field) {
                continue;
            }
            let mut on = self.flag_state(field) == Some(true);
            if ui.checkbox(&mut on, egui::RichText::new(label).size(12.0)).clicked() {
                self.toggle_flag(field);
            }
        }
        // text alignment, only when every selected view is a note or text (style-editor-view.js:477-483)
        let texts: Vec<Eid> = self.selection.iter().copied().filter(|v| self.repo.alive(*v)).collect();
        if !texts.is_empty() && texts.iter().all(|v| self.repo.kind_of(*v, "UMLCustomTextView")) {
            for (field, opts) in [("horzAlign", [(0, "Left"), (2, "Center"), (1, "Right")]), ("vertAlign", [(3, "Top"), (5, "Middle"), (4, "Bottom")])] {
                let vals: Vec<i64> = texts.iter().map(|v| self.repo.val(*v, field).and_then(|x| x.as_i64()).unwrap_or(-1)).collect();
                let shared = vals.iter().all(|x| *x == vals[0]).then_some(vals[0]);
                ui.horizontal(|ui| {
                    for (value, label) in opts {
                        if ui.selectable_label(shared == Some(value), egui::RichText::new(label).size(11.0)).clicked() {
                            self.set_text_align(&texts, field, value);
                        }
                    }
                });
            }
        }
        ui.horizontal_wrapped(|ui| {
            let small = |t: &str| egui::Button::new(egui::RichText::new(t).size(11.0)).fill(HOVER_ROW()).stroke(Stroke::new(1.0, INPUT_BORDER()));
            if ui.add(small("Back")).on_hover_text("Send To Back").clicked() {
                self.reorder(false);
            }
            if ui.add(small("Front")).on_hover_text("Bring To Front").clicked() {
                self.reorder(true);
            }
            for (how, label, short) in [
                (crate::align::Align::Left, "Align Left", "L"),
                (crate::align::Align::Center, "Align Center", "C"),
                (crate::align::Align::Right, "Align Right", "R"),
                (crate::align::Align::Top, "Align Top", "T"),
                (crate::align::Align::Middle, "Align Middle", "M"),
                (crate::align::Align::Bottom, "Align Bottom", "B"),
                (crate::align::Align::SpaceH, "Space Equally, Horizontally", "↔"),
                (crate::align::Align::SpaceV, "Space Equally, Vertically", "↕"),
                (crate::align::Align::Width, "Set Width Equally", "W"),
                (crate::align::Align::Height, "Set Height Equally", "H"),
                (crate::align::Align::Size, "Set Size Equally", "S"),
            ] {
                if ui.add(small(short)).on_hover_text(label).clicked() {
                    self.align(how);
                }
            }
        });
    }

    /// A colour swatch that opens the Color dialog; a mixed selection shows a transparent swatch
    /// (style-editor-view.js:229-278, 549-592).
    fn color_swatch(&mut self, ui: &mut egui::Ui, field: &'static str, label: &str) {
        let c = self.color_state(field).map(|c| Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3)).unwrap_or(Color32::TRANSPARENT);
        if swatch(ui, c, Vec2::new(28.0, 16.0)).on_hover_text(label).clicked() {
            self.open_color_dialog(field);
        }
    }

    /// Format > Fill Color and Line Color: a colour picker on the selected views (default-commands.js:966-991).
    pub(crate) fn color_menus(&mut self, ui: &mut egui::Ui) {
        for (field, label) in crate::style::COLORS.into_iter().take(2) {
            ui.add_enabled_ui(!self.color_views(field).is_empty(), |ui| {
                ui.menu_button(label, |ui| {
                    let cur = self.color_state(field).unwrap_or(crate::render::Rgba(255, 255, 255, 255));
                    let mut c = Color32::from_rgb(cur.0, cur.1, cur.2);
                    if egui::color_picker::color_picker_color32(ui, &mut c, egui::color_picker::Alpha::Opaque) {
                        self.set_color(field, crate::render::Rgba(c.r(), c.g(), c.b(), 255));
                    }
                });
            });
        }
    }

    /// Format > Line Style, with a check on the style the selected lines share (default-commands.js:993-1011, 1337-1342).
    pub(crate) fn line_style_menu(&mut self, ui: &mut egui::Ui) {
        let has = self.selection.iter().any(|v| self.repo.alive(*v) && views::is_edge(&self.repo, *v));
        ui.add_enabled_ui(has, |ui| {
            ui.menu_button("Line Style", |ui| {
                let cur = self.line_style_state();
                for (style, label) in [(0, self.key_text("Rectilinear", "format:linestyle-rectilinear")), (1, self.key_text("Oblique", "format:linestyle-oblique")), (2, self.key_text("Rounded Rectilinear", "format:linestyle-roundrect")), (3, self.key_text("Curve", "format:linestyle-curve"))] {
                    let mut on = cur == Some(style);
                    if ui.checkbox(&mut on, label).clicked() {
                        self.set_line_style(style);
                        ui.close();
                    }
                }
            });
        });
    }

    /// Move Up and Move Down (Edit menu and explorer menu).
    pub(crate) fn move_items(&mut self, ui: &mut egui::Ui) {
        let e = self.focus.filter(|f| self.repo.alive(*f) && self.repo.ty(*f) != "Project" && !self.is_view(*f));
        if ui.add_enabled(e.is_some(), self.key_button("Move Up", "edit:move-up")).clicked() {
            self.move_element(e.unwrap(), true);
            ui.close();
        }
        if ui.add_enabled(e.is_some(), self.key_button("Move Down", "edit:move-down")).clicked() {
            self.move_element(e.unwrap(), false);
            ui.close();
        }
    }

    /// Cut, Copy and Paste (Edit menu, canvas menu and explorer menu).
    pub(crate) fn clipboard_items(&mut self, ui: &mut egui::Ui) {
        let can = self.can_copy();
        if ui.add_enabled(can, self.key_button("Cut", "edit:cut")).clicked() {
            self.cmd_cut(Some(&ui.ctx().clone()));
            ui.close();
        }
        if ui.add_enabled(can, self.key_button("Copy", "edit:copy")).clicked() {
            self.cmd_copy(Some(&ui.ctx().clone()));
            ui.close();
        }
        if ui.add_enabled(self.diagram.is_some(), self.key_button("Copy Diagram As Image", "edit:copy-diagram-as-image")).clicked() {
            self.cmd_copy_diagram_image(&ui.ctx().clone());
            ui.close();
        }
        if ui.add(self.key_button("Paste", "edit:paste")).clicked() {
            self.cmd_paste();
            ui.close();
        }
    }

    /// Close Diagram, Close Other Diagrams and Close All Diagrams (View menu and Working Diagrams menu).
    fn diagram_close_items(&mut self, ui: &mut egui::Ui) {
        let has = self.diagram.is_some();
        if ui.add_enabled(has, self.key_button("Close Diagram", "view:close-diagram")).clicked() {
            self.close_diagram(None);
            ui.close();
        }
        if ui.add_enabled(has, self.key_button("Close Other Diagrams", "view:close-other-diagrams")).clicked() {
            self.close_other_diagrams();
            ui.close();
        }
        if ui.add_enabled(!self.open.is_empty(), self.key_button("Close All Diagrams", "view:close-all-diagrams")).clicked() {
            self.close_all_diagrams();
            ui.close();
        }
    }

    /// The Format menu's on/off items, checked when every selected view that has the flag is on,
    /// and enabled only when a selected view has it (uml.json order; default-commands.js:1319-1350).
    /// Format > Stereotype Display, checked on the value the selected views share.
    pub(crate) fn stereotype_display_menu(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(!self.flag_views("stereotypeDisplay").is_empty(), |ui| {
            ui.menu_button("Stereotype Display", |ui| {
                let cur = self.stereotype_display_state();
                for (value, label) in crate::style::STEREOTYPE_DISPLAYS {
                    let mut on = cur.as_deref() == Some(value);
                    if ui.checkbox(&mut on, self.key_text(label, &format!("format:stereotype-{value}"))).clicked() {
                        self.set_stereotype_display(value);
                        ui.close();
                    }
                }
            });
        });
    }

    pub(crate) fn format_toggles(&mut self, ui: &mut egui::Ui) {
        for (i, (field, label, id)) in FORMAT_TOGGLES.into_iter().enumerate() {
            // UML display toggles after Auto Resize, Show Shadow and Inherit Style, then the Suppress ones
            if i == 3 || field == "suppressAttributes" || field == "suppressPropertyValues" {
                ui.separator();
            }
            let has = !self.flag_views(field).is_empty();
            let mut on = self.flag_state(field) == Some(true);
            if ui.add_enabled(has, egui::Checkbox::new(&mut on, self.key_text(label, id))).clicked() {
                self.toggle_flag(field);
                ui.close();
            }
        }
    }

    /// Format > Alignment, in the main menu and the canvas menu (alignment.json order).
    pub(crate) fn alignment_menu(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.can_align(), |ui| {
            ui.menu_button("Alignment", |ui| {
                if ui.button("Send To Back").clicked() {
                    self.reorder(false);
                    ui.close();
                }
                if ui.button("Bring To Front").clicked() {
                    self.reorder(true);
                    ui.close();
                }
                for (i, (how, label)) in crate::align::Align::ALL.into_iter().enumerate() {
                    if i == 0 || i == 6 || i == 8 {
                        ui.separator();
                    }
                    if ui.button(label).clicked() {
                        self.align(how);
                        ui.close();
                    }
                }
            });
        });
    }

    pub(crate) fn shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, KeyboardShortcut, Modifiers};
        // an open dialog takes the keys: no shortcut fires behind it (dialog-manager.js:308-309)
        if self.modal_open() {
            return;
        }
        // while a text field has the keys, the edit keys (Ctrl+C/X/V/Z/Y, Ctrl+A, Delete and
        // Backspace, with any other modifier) are the field's; every other shortcut still runs
        // (keymap-manager.js:294-347)
        let typing = self.editing.is_some() || text_field_focused(ctx);
        let measure = EguiMeasure { ctx: ctx.clone() };
        let sc = |m: Modifiers, k: Key| KeyboardShortcut::new(m, k);
        let cs = Modifiers::COMMAND | Modifiers::SHIFT;
        let hit = |s: KeyboardShortcut| !(typing && crate::keymap::is_edit_key(s.modifiers, s.logical_key)) && ctx.input_mut(|i| i.consume_shortcut(&s));
        // the user's keymap.json wins over the built-in shortcuts
        self.user_shortcuts(ctx, typing);
        // Ctrl+Shift toggles first: a Ctrl shortcut also matches when Shift is held (Ctrl+N, Ctrl+A…)
        {
            if hit(sc(cs, Key::F)) {
                self.open_font_dialog();
            }
            if hit(sc(cs, Key::P)) {
                self.open_palette(crate::palette::Mode::Commands);
            }
            // Ctrl+Shift+0…5 before Ctrl+0 (Actual Size), which also matches with Shift held
            for (i, key) in [Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5].into_iter().enumerate() {
                if hit(sc(cs, key)) {
                    self.set_stereotype_display(crate::style::STEREOTYPE_DISPLAYS[i].0);
                }
            }
            for (key, field) in [(Key::I, "fillColor"), (Key::L, "lineColor")] {
                if hit(sc(cs, key)) {
                    self.open_color_dialog(field);
                }
            }
            for (key, field) in [(Key::R, "autoResize"), (Key::K, "parentStyle"), (Key::W, "wordWrap"), (Key::V, "showVisibility"), (Key::N, "showNamespace"), (Key::B, "showProperty"), (Key::Y, "showType"), (Key::M, "showMultiplicity"), (Key::J, "showOperationSignature"), (Key::A, "suppressAttributes"), (Key::O, "suppressOperations"), (Key::E, "suppressReceptions"), (Key::T, "suppressLiterals")] {
                if hit(sc(cs, key)) {
                    self.toggle_flag(field);
                }
            }
            // Ctrl+Shift+V reaches egui as a paste (when the clipboard holds text), not as a key
            let shift_paste = !typing && ctx.input_mut(|i| {
                let n = i.events.len();
                if i.modifiers.command && i.modifiers.shift {
                    i.events.retain(|e| !matches!(e, egui::Event::Paste(_)));
                }
                n != i.events.len()
            });
            if shift_paste {
                self.toggle_flag("showVisibility");
            }
        }
        if hit(sc(Modifiers::COMMAND, Key::N)) {
            self.cmd_new(&measure);
        }
        if hit(sc(Modifiers::COMMAND, Key::O)) {
            self.cmd_open(&measure);
        }
        if hit(sc(cs, Key::S)) {
            self.save_as(&measure);
        }
        if hit(sc(Modifiers::COMMAND, Key::S)) {
            self.save(&measure);
        }
        if hit(sc(Modifiers::COMMAND, Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // Esc anywhere but in a field goes back to the Select tool and clears the lock (toolbox-view.js:473-479)
        if !typing && ctx.input(|i| i.key_pressed(Key::Escape)) && self.drag.is_none() {
            self.tool = None;
            self.tool_sticky = false;
        }
        if hit(sc(Modifiers::COMMAND, Key::Y)) || hit(sc(cs, Key::Z)) {
            self.do_redo();
        }
        if hit(sc(Modifiers::COMMAND, Key::Z)) {
            self.do_undo();
        }
        // Ctrl+C, Ctrl+X and Ctrl+V reach egui as Copy, Cut and Paste events, not as keys
        let clip = if typing { None } else { ctx.input_mut(|i| {
            let mut out = None;
            i.events.retain(|e| match e {
                egui::Event::Copy | egui::Event::Cut | egui::Event::Paste(_) if out.is_none() => {
                    out = Some(e.clone());
                    false
                }
                _ => true,
            });
            out
        }) };
        let shift = ctx.input(|i| i.modifiers.shift);
        match clip {
            // Ctrl+Shift+C also arrives as a Copy event
            Some(egui::Event::Copy) if shift => self.cmd_copy_diagram_image(ctx),
            // Ctrl+Shift+X (Suppress Receptions, StarUML 8's keymap) arrives as a Cut event
            Some(egui::Event::Cut) if shift && ctx.input(|i| i.modifiers.command) => self.toggle_flag("suppressReceptions"),
            Some(egui::Event::Copy) => self.cmd_copy(Some(ctx)),
            Some(egui::Event::Cut) => self.cmd_cut(Some(ctx)),
            Some(egui::Event::Paste(t)) => self.paste_text(&t),
            _ => {}
        }
        if hit(sc(Modifiers::COMMAND, Key::Delete)) {
            self.delete_selection(true);
        }
        if hit(sc(Modifiers::COMMAND, Key::A)) {
            self.select_all();
        }
        if hit(sc(Modifiers::COMMAND, Key::E)) {
            if let Some(f) = self.focus {
                self.reveal(f);
            }
        }
        // Ctrl+Alt+D and Ctrl+Shift+D first: Ctrl+D also matches when Alt or Shift is held
        if hit(sc(Modifiers::COMMAND | Modifiers::ALT, Key::D)) {
            self.show_markdown = !self.show_markdown;
        }
        // StarUML's UML keymap gives Ctrl+Shift+D to Show Diagram Name over Open Sub-Diagram
        // (keymaps/uml.json); UMLCraft keeps Open Sub-Diagram and toggles the diagram name only when
        // the selection holds frames
        if hit(sc(cs, Key::D)) {
            if self.flag_views("showDiagramName").is_empty() {
                self.open_sub_diagram();
            } else {
                self.toggle_flag("showDiagramName");
            }
        }
        if hit(sc(Modifiers::COMMAND, Key::D)) {
            self.select_in_diagram();
        }
        for (key, up) in [(Key::ArrowUp, true), (Key::ArrowDown, false)] {
            if hit(sc(cs, key)) {
                if let Some(f) = self.focus.filter(|f| self.repo.alive(*f) && self.repo.ty(*f) != "Project") {
                    self.move_element(f, up);
                }
            }
        }
        // Ctrl+Alt first: Ctrl+L and Ctrl+B also match when Alt is held
        let ca = Modifiers::COMMAND | Modifiers::ALT;
        if hit(sc(ca, Key::L)) {
            self.set_line_style(2);
        }
        if hit(sc(ca, Key::B)) {
            self.set_line_style(3);
        }
        if hit(sc(ca, Key::R)) {
            self.show_relationships = !self.show_relationships;
        }
        if hit(sc(ca, Key::M)) {
            self.show_minimap = !self.show_minimap;
        }
        if hit(sc(ca, Key::T)) {
            self.show_thumbnails = !self.show_thumbnails;
        }
        if hit(sc(Modifiers::COMMAND, Key::T)) {
            self.open_tag_editor();
        }
        if hit(sc(Modifiers::COMMAND, Key::P)) {
            self.open_palette(crate::palette::Mode::Find);
        }
        if hit(sc(Modifiers::COMMAND, Key::F)) {
            self.open_find();
        }
        if hit(sc(Modifiers::COMMAND, Key::L)) {
            self.set_line_style(0);
        }
        if hit(sc(Modifiers::COMMAND, Key::B)) {
            self.set_line_style(1);
        }
        if hit(sc(cs, Key::H)) {
            self.toggle_shadow();
        }
        if hit(sc(Modifiers::COMMAND, Key::Equals)) || hit(sc(Modifiers::COMMAND, Key::Plus)) {
            self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP);
        }
        if hit(sc(Modifiers::COMMAND, Key::Minus)) {
            self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP);
        }
        // View > Sidebar, Navigator, Toolbar, Statusbar, Toolbox, Editors (keymaps/linux.json)
        for (i, key) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6].into_iter().enumerate() {
            if hit(sc(Modifiers::COMMAND, key)) {
                let f = self.panel_flag(i);
                *f = !*f;
            }
        }
        if hit(sc(Modifiers::COMMAND, Key::Num0)) {
            self.zoom_keep_centre(1.0);
        }
        if hit(sc(Modifiers::COMMAND, Key::Num9)) {
            self.fit();
        }
        // Ctrl+Shift+G first: Ctrl+G also matches when Shift is held
        if hit(sc(cs, Key::G)) {
            self.snap = !self.snap;
        }
        if hit(sc(Modifiers::COMMAND, Key::G)) {
            self.show_grid = !self.show_grid;
        }
        // Shift+F4 first: a plain F4 shortcut also matches when Shift is held
        if hit(sc(Modifiers::SHIFT, Key::F4)) {
            self.close_all_diagrams();
        }
        if hit(sc(Modifiers::COMMAND, Key::F4)) {
            self.close_other_diagrams();
        }
        if hit(sc(Modifiers::NONE, Key::F4)) {
            self.close_diagram(None);
        }
        // Ctrl+Shift+] and [: with Shift held most layouts report } and {
        if hit(sc(cs, Key::CloseBracket)) || hit(sc(cs, Key::CloseCurlyBracket)) {
            self.cycle_diagram(1);
        }
        if hit(sc(cs, Key::OpenBracket)) || hit(sc(cs, Key::OpenCurlyBracket)) {
            self.cycle_diagram(-1);
        }
    }

    /// File > New: a new window with a blank project (application:new, application.js:246-248).
    pub(crate) fn cmd_new(&mut self, _ms: &EguiMeasure) {
        self.new_window();
    }

    /// Open a recent file; a file that is gone is dropped from the list with the toast
    /// "File not found: <path>" (open-recent/main.js:59).
    fn open_recent(&mut self, _ms: &EguiMeasure, p: std::path::PathBuf) {
        if !p.exists() {
            self.toast_error(&format!("File not found: {}", p.display()));
            let mut list = self.recent_files();
            list.retain(|x| *x != p);
            self.set_recent(list);
            return;
        }
        self.open_file(p);
    }

    /// Each .mdj file dropped on the window is opened, here or in a new window (app-context.js:546-565);
    /// other files are ignored.
    fn dropped_files(&mut self, ctx: &egui::Context) {
        let files: Vec<std::path::PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).collect());
        for p in files.into_iter().filter(|p| p.extension().map(|e| e.eq_ignore_ascii_case("mdj")).unwrap_or(false)) {
            self.open_file(p);
        }
    }

    /// File › Export › StarUML V7...: a copy of the project for StarUML 7, colour variables replaced by
    /// their light-theme colours (https://docs.staruml.io/user-guide/managing-project/).
    fn export_v7_dialog(&mut self, ms: &EguiMeasure) {
        let stem = self.filename.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| self.repo.name(self.repo.root));
        let Some(mut p) = rfd::FileDialog::new().set_title("Export StarUML V7").set_file_name(format!("{stem}-v7.mdj")).add_filter("Models", &["mdj"]).save_file() else { return };
        if p.extension().map(|e| e != "mdj").unwrap_or(true) {
            p.set_extension("mdj");
        }
        let text = self.v7_json(ms);
        if let Err(e) = std::fs::write(&p, text) {
            self.message = Some(("Failed to export the file.".into(), e.to_string()));
        }
    }

    /// File > Open: the dialog, then the file here or in a new window (default-commands.js:221-234).
    fn cmd_open(&mut self, _ms: &EguiMeasure) {
        self.open_dialog();
    }

    /// The original replaces the project without asking; UMLCraft asks to save changes first.
    /// Dialog title and filters of extensions/default/staruml-v1/main.js:29, 101.
    fn cmd_import_staruml_v1(&mut self, ms: &EguiMeasure) {
        if !self.confirm_discard(ms) {
            return;
        }
        let dialog = rfd::FileDialog::new().set_title("Select a StarUML 1 file (.uml)").add_filter("UML model (StarUML V1)", &["uml"]).add_filter("All Files", &["*"]);
        if let Some(p) = dialog.pick_file() {
            self.import_staruml_v1(p);
        }
    }

    /// Export the current diagram as PNG, JPEG or SVG. The selection is cleared first; with no diagram
    /// open the alert "No diagram to export." appears (default-commands.js:360-445).
    pub(crate) fn export_current(&mut self, ctx: &egui::Context, f: crate::export::Format) {
        let Some(d) = self.diagram else {
            self.alert("No diagram to export.");
            return;
        };
        self.select(vec![]);
        let ms = EguiMeasure { ctx: ctx.clone() };
        let title = format!("Export Diagram as {}", f.label());
        let Some(p) = rfd::FileDialog::new().set_title(&title).set_file_name(self.export_name(d, f)).add_filter(f.label(), &[f.ext()]).save_file() else {
            return;
        };
        let result = crate::export::render(self, d, &ms, f, ctx.pixels_per_point()).and_then(|bytes| std::fs::write(&p, bytes).map_err(|e| e.to_string()));
        if let Err(e) = result {
            self.message = Some(("Failed to export the diagram.".into(), e));
        }
    }

    /// Export every diagram into a folder the person picks (default-commands.js:452-535).
    pub(crate) fn export_all_cmd(&mut self, ctx: &egui::Context, f: crate::export::Format) {
        if self.repo.all_of("Diagram").is_empty() {
            self.alert("No diagram to export.");
            return;
        }
        let Some(folder) = rfd::FileDialog::new().set_title("Select a folder where all diagrams to be exported").pick_folder() else { return };
        self.select(vec![]);
        let ms = EguiMeasure { ctx: ctx.clone() };
        match self.export_all(&folder, &ms, f, ctx.pixels_per_point()) {
            Ok(n) => self.toast(&format!("{n} diagrams exported.")),
            Err(e) => self.message = Some(("Failed to export the diagrams.".into(), e)),
        }
    }

    pub(crate) fn cmd_copy_diagram_image(&mut self, ctx: &egui::Context) {
        let Some(d) = self.diagram else { return };
        let ms = EguiMeasure { ctx: ctx.clone() };
        let info = views::layout_diagram(&mut self.repo, &ms, d);
        let png = render::to_png(&self.repo, d, &info, &ms, 1.0);
        self.copy_diagram_image(png);
    }

    /// Whether a dialog is open (the Command Palette, which handles its own keys, aside).
    pub fn modal_open(&self) -> bool {
        self.about_open
            || self.message.is_some()
            || self.list_picker.is_some()
            || !self.boxes.is_empty()
            || self.color_dialog.is_some()
            || self.list_editor.is_some()
            || self.select_one.is_some()
            || self.delete_prompt
            || self.font_dialog.is_some()
            || self.prefs_open.is_some()
            || self.tag_editor.is_some()
            || self.find_dialog.is_some()
            || self.element_picker.is_some()
            || self.icon_picker.is_some()
            || self.input_dialog.is_some()
            || self.mermaid_code.is_some()
            || self.codegen.is_some()
            || self.print_open
            || self.color_prompt
            || self.export_image_open
    }

    pub(crate) fn dialogs(&mut self, ctx: &egui::Context) {
        if self.about_open {
            // the app icon, name and version, and a link to the third-party software (about-dialog.js:31-47)
            let icon = app_icon(ctx);
            // a modal window: a backdrop keeps the rest of the window out of reach (dialog-manager.js:242-313)
            dialog(ctx, "about", "About UMLCraft", |ui| {
                ui.horizontal(|ui| {
                    if let Some(t) = &icon {
                        ui.add(egui::Image::new((t.id(), Vec2::splat(64.0))));
                    }
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("UMLCraft").size(20.0).strong());
                        ui.label(concat!("Version ", env!("CARGO_PKG_VERSION")));
                    });
                });
                ui.add_space(6.0);
                ui.label("A UML modeler that opens and saves StarUML .mdj model files.");
                ui.label("Written in Rust with egui, under the MIT licence.");
                ui.horizontal(|ui| {
                    ui.hyperlink_to("Website", "https://burkhaltery.github.io/umlcraft");
                    ui.hyperlink_to("Third-party software", "https://github.com/BurkhalterY/umlcraft/network/dependencies");
                });
                ui.add_space(6.0);
                // Enter presses OK and Esc cancels (dialog-manager.js:148-162); OK has the focus
                let ok = ui.button("OK");
                focus_primary(ui, &ok);
                if ok.clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape)) {
                    self.about_open = false;
                }
            });
        }
        self.list_picker_ui(ctx);
        self.color_dialog_ui(ctx);
        self.list_editor_ui(ctx);
        self.select_one_ui(ctx);
        self.delete_prompt_ui(ctx);
        self.print_dialog_ui(ctx);
        self.color_prompt_ui(ctx);
        self.export_image_ui(ctx);
        self.element_picker_ui(ctx);
        self.icon_picker_ui(ctx);
        self.input_dialog_ui(ctx);
        self.mermaid_dialog_ui(ctx);
        self.codegen_ui(ctx);
        self.preferences_ui(ctx);
        self.tag_editor_ui(ctx);
        self.palette_ui(ctx);
        self.find_dialog_ui(ctx);
        self.font_dialog_ui(ctx);
        self.message_boxes_ui(ctx);
    }

    pub(crate) fn open_font_dialog(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        let (face, size) = self.font_state();
        let size = size.map(|s| if s.fract() == 0.0 { format!("{}", s as i64) } else { s.to_string() });
        self.font_dialog = Some((face.unwrap_or_default(), size.unwrap_or_default(), self.color_state("fontColor")));
    }

    /// Format > Font (Ctrl+Shift+F): face, size and colour, pre-filled with the merged values (blank where the
    /// selected views differ). OK applies what was filled in; a size that is not a number is ignored
    /// (default-commands.js:940-964).
    fn font_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some((mut face, mut size, mut color)) = self.font_dialog.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        dialog(ctx, "font", "Font", |ui| {
            egui::Grid::new("font-grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label("Face");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut face).hint_text("—").desired_width(160.0));
                    // the loaded fonts (dialog-manager.js:538); a face that is not installed draws as Arial
                    egui::ComboBox::from_id_salt("font-faces").selected_text("").width(20.0).show_ui(ui, |ui| {
                        for f in crate::fonts::face_names() {
                            let ok = crate::fonts::available(&f);
                            let r = ui.add_enabled(true, egui::Button::selectable(face == f, egui::RichText::new(&f).color(if ok { TEXT() } else { DIM() })));
                            let r = if ok { r } else { r.on_hover_text("Not installed: drawn with the Arial-compatible font") };
                            if r.clicked() {
                                face = f.clone();
                            }
                        }
                    });
                });
                ui.end_row();
                ui.label("Size");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut size).hint_text("—").desired_width(60.0));
                    // the dialog's own sizes (dialog-manager.js:541)
                    egui::ComboBox::from_id_salt("font-sizes").selected_text("").width(20.0).show_ui(ui, |ui| {
                        for s in [8, 9, 10, 11, 12, 13, 15, 16, 18, 20, 22, 24, 26, 28, 32, 36, 42] {
                            if ui.selectable_label(false, format!("{s}")).clicked() {
                                size = format!("{s}");
                            }
                        }
                    });
                });
                ui.end_row();
                ui.label("Color");
                let mut c = color.map(|c| Color32::from_rgb(c.0, c.1, c.2)).unwrap_or(Color32::TRANSPARENT);
                if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                    color = Some(crate::render::Rgba(c.r(), c.g(), c.b(), 255));
                }
                ui.end_row();
                ui.label("");
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(2.0);
                    for row in crate::style::palette().chunks(16) {
                        ui.horizontal(|ui| {
                            for c in row {
                                if swatch(ui, Color32::from_rgb(c.0, c.1, c.2), Vec2::splat(12.0)).on_hover_text(crate::style::hex(*c)).clicked() {
                                    color = Some(*c);
                                }
                            }
                        });
                    }
                });
                ui.end_row();
            });
            // the live preview, on the diagram's white with the chosen size and colour
            let px = size.trim().parse::<f32>().ok().filter(|s| *s > 0.0).unwrap_or(13.0).min(42.0);
            let fg = color.map(|c| Color32::from_rgb(c.0, c.1, c.2)).unwrap_or(Color32::BLACK);
            ui.add_space(4.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 60.0), egui::Sense::hover());
            ui.painter().rect(rect, 0.0, Color32::WHITE, Stroke::new(1.0, BORDER()), egui::StrokeKind::Inside);
            let family = crate::fonts::family(ui.ctx(), if face.trim().is_empty() { "Arial" } else { face.trim() }, 0);
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, "AaBb123", egui::FontId::new(px, family), fg);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ok = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
            });
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                cancel = true;
            }
        });
        if ok {
            let f = face.trim();
            let sz = size.trim().parse::<f64>().ok().filter(|s| *s > 0.0);
            self.set_font(if f.is_empty() { None } else { Some(f) }, sz);
            if let Some(c) = color {
                if Some(c) != self.color_state("fontColor") {
                    self.style_edit = None;
                    self.set_color("fontColor", c);
                }
            }
            self.font_dialog = None;
        } else if cancel {
            self.font_dialog = None;
        } else {
            self.font_dialog = Some((face, size, color));
        }
    }

    /// The Color dialog (dialog-manager.js:484-527): a picker with opacity, a hex field, a 16-column
    /// preset palette and a Transparent swatch. OK (or Enter) applies the colour to the selected
    /// views as one undo step; Cancel (or Esc) leaves them.
    fn color_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.color_dialog.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        dialog(ctx, "color", "Color", |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    let mut c = Color32::from_rgba_unmultiplied(d.color.0, d.color.1, d.color.2, d.color.3);
                    if egui::color_picker::color_picker_color32(ui, &mut c, egui::color_picker::Alpha::OnlyBlend) {
                        let [r, g, b, a] = c.to_srgba_unmultiplied();
                        d.color = crate::render::Rgba(r, g, b, a);
                        d.text = crate::style::hex(d.color);
                        d.var = None;
                    }
                    let r = ui.add(egui::TextEdit::singleline(&mut d.text).desired_width(120.0).font(egui::TextStyle::Monospace)).on_hover_text("#rrggbb, #rrggbbaa or a color variable ($blue9)");
                    if r.changed() {
                        d.take_text();
                    }
                });
                // StarUML 8's palette: background, foreground and transparent, then 12 steps of each
                // hue, in the current theme's colours; hovering shows the variable's name
                // (https://docs.staruml.io/user-guide/formatting-diagram/)
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(2.0);
                    let names = crate::colorvars::names();
                    let (first, hues) = names.split_at(3);
                    for row in std::iter::once(first).chain(hues.chunks(12)) {
                        ui.horizontal(|ui| {
                            for n in row {
                                let c = crate::render::parse_color(n, crate::render::WHITE);
                                let r = swatch(ui, Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3), Vec2::splat(14.0));
                                if d.var.as_deref() == Some(n.as_str()) {
                                    ui.painter().rect_stroke(r.rect.expand(1.0), 0.0, Stroke::new(1.5, ACCENT()), egui::StrokeKind::Outside);
                                }
                                if r.on_hover_text(n.as_str()).clicked() {
                                    d.pick_var(n, self.color_vars);
                                }
                            }
                        });
                    }
                });
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(2.0);
                    for row in crate::style::palette().chunks(16) {
                        ui.horizontal(|ui| {
                            for c in row {
                                if swatch(ui, Color32::from_rgb(c.0, c.1, c.2), Vec2::splat(14.0)).on_hover_text(crate::style::hex(*c)).clicked() {
                                    d.color = *c;
                                    d.text = crate::style::hex(*c);
                                    d.var = None;
                                }
                            }
                        });
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if swatch(ui, Color32::TRANSPARENT, Vec2::splat(14.0)).on_hover_text("rgba(255, 255, 255, 0)").clicked() {
                            d.color = crate::render::Rgba(255, 255, 255, 0);
                            d.text = crate::style::hex(d.color);
                            d.var = None;
                        }
                        ui.label("Transparent");
                    });
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ok = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
            });
            ok |= ui.input(|i| i.key_pressed(egui::Key::Enter));
            cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if ok {
            self.style_edit = None;
            self.apply_color_dialog(&d);
            self.color_dialog = None;
        } else if cancel || self.color_views(d.field).is_empty() {
            self.color_dialog = None;
        } else {
            self.color_dialog = Some(d);
        }
    }

    /// The "Select One" dialog: the message, one radio button per option, OK (or Enter) and Cancel
    /// (or Esc) (dialog-manager.js:433-476, select-radio-dialog.html).
    fn select_one_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.select_one.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        dialog(ctx, "select-one", "Select One", |ui| {
            ui.label(&d.message);
            ui.add_space(4.0);
            for (i, o) in d.options.iter().enumerate() {
                ui.radio_value(&mut d.selected, i, *o);
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ok = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
            });
            ok |= ui.input(|i| i.key_pressed(egui::Key::Enter));
            cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if ok {
            self.select_one = None;
            self.select_one_ok(d);
        } else if cancel {
            self.select_one = None;
        } else {
            self.select_one = Some(d);
        }
    }

    /// "Delete Views Only": Delete on views that ask to go with their models (lifelines, messages,
    /// labels…) offers to delete the views only or the models too (default-commands.js:786-822).
    fn delete_prompt_ui(&mut self, ctx: &egui::Context) {
        if !self.delete_prompt {
            return;
        }
        let (mut views, mut models, mut cancel) = (false, false, false);
        dialog(ctx, "delete-views", "Delete Views Only", |ui| {
            ui.label("Do you want to delete views only?");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                views = ui.button("Delete Views Only").clicked();
                models = ui.button("Delete from Model").clicked();
                cancel = ui.button("Cancel").clicked();
            });
            models |= ui.input(|i| i.key_pressed(egui::Key::Enter));
            cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if views || models || cancel {
            self.delete_prompt = false;
        }
        if views {
            self.delete_views_only();
        } else if models {
            self.delete_selection(true);
        }
    }

    /// Toasts stack upward inside the diagram area and hide after 5 s (toast-manager.js:36-72).
    fn toasts_ui(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|(_, t, _)| t.elapsed().as_secs_f32() < 5.0);
        let Some(area) = self.canvas_rect else { return };
        let mut y = area.bottom() - 16.0;
        for (i, (text, _, error)) in self.toasts.iter().enumerate().rev() {
            let pos = Pos2::new(area.left() + 16.0, y);
            let (fill, edge, fg) = if *error { (Color32::from_rgb(0xc6, 0x3b, 0x3f), Color32::from_rgb(0xe5, 0x48, 0x4d), Color32::WHITE) } else { (LAYER1(), BORDER(), TEXT()) };
            let r = egui::Area::new(egui::Id::new(("toast", i))).order(egui::Order::Foreground).pivot(Align2::LEFT_BOTTOM).fixed_pos(pos).show(ctx, |ui| {
                Frame::NONE.fill(fill).stroke(Stroke::new(1.0, edge)).corner_radius(CornerRadius::same(3)).inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
                    ui.label(egui::RichText::new(text).color(fg).size(13.0));
                });
            });
            y -= r.response.rect.height() + 6.0;
        }
        if !self.toasts.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    fn screenshots(&mut self, ctx: &egui::Context) {
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            if let Some(path) = self.screenshot.clone() {
                save_png(&path, &img);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.screenshot.is_some() {
            self.frames += 1;
            ctx.request_repaint();
            if !self.shot_requested && self.frames >= SCREENSHOT_FRAMES && self.started.elapsed().as_secs_f32() >= 2.0 {
                self.shot_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
        }
    }
}

/// View menu panels, in the order of their Ctrl+1…6 shortcuts.
const PANELS: [&str; 6] = ["Sidebar", "Navigator", "Toolbar", "Statusbar", "Toolbox", "Editors"];

impl App {
    fn panel_flag(&mut self, i: usize) -> &mut bool {
        match i {
            0 => &mut self.show_sidebar,
            1 => &mut self.show_navigator,
            2 => &mut self.show_toolbar,
            3 => &mut self.show_statusbar,
            4 => &mut self.show_toolbox,
            _ => &mut self.show_editors,
        }
    }
}

/// The drag payload of a Working Diagrams row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkingRow(pub Eid);

/// The multiplicity choices of the property editor and the inline editor (uml.json `uml.multiplicity`).
pub(crate) const MULTIPLICITIES: [&str; 5] = ["0..1", "1", "0..*", "1..*", "*"];

/// Format menu toggles, in the original's order, with their shortcuts (keymaps/uml.json, linux.json).
/// The Format menu's on/off items: the view field, the name and the command (whose keystroke the
/// menu shows).
const FORMAT_TOGGLES: [(&str, &str, &str); 24] = [
    ("autoResize", "Auto Resize", "format:auto-resize"),
    ("showShadow", "Show Shadow", "format:show-shadow"),
    ("parentStyle", "Inherit Style", "format:inherit-style"),
    ("wordWrap", "Word Wrap", "format:word-wrap"),
    ("showVisibility", "Show Visibility", "format:show-visibility"),
    ("showNamespace", "Show Namespace", "format:show-namespace"),
    ("showProperty", "Show Property", "format:show-property"),
    ("showType", "Show Type", "format:show-type"),
    ("showMultiplicity", "Show Multiplicity", "format:show-multiplicity"),
    ("showOperationSignature", "Show Operation Signature", "format:show-operation-signature"),
    ("showDiagramName", "Show Diagram Name", "format:show-diagram-name"),
    ("suppressAttributes", "Suppress Attributes", "format:suppress-attributes"),
    ("suppressOperations", "Suppress Operations", "format:suppress-operations"),
    ("suppressReceptions", "Suppress Receptions", "format:suppress-receptions"),
    ("suppressLiterals", "Suppress Literals", "format:suppress-literals"),
    ("suppressColumns", "Suppress Columns", "erd:suppress-columns"),
    // SysML (sysml/menus/sysml.json), between separators
    ("suppressPropertyValues", "Suppress Property Values", "format:suppress-property-values"),
    ("suppressConstraints", "Suppress Constraints", "format:suppress-constraints"),
    ("suppressParts", "Suppress Parts", "format:suppress-parts"),
    ("suppressPorts", "Suppress Ports", "format:suppress-ports"),
    ("suppressReferences", "Suppress References", "format:suppress-references"),
    ("suppressValues", "Suppress Values", "format:suppress-values"),
    ("suppressProperties", "Suppress Properties", "format:suppress-properties"),
    ("suppressFlowProperties", "Suppress Flow Properties", "format:suppress-flow-properties"),
];

/// The small padlock shown on a locked toolbox item.
fn draw_lock(p: &egui::Painter, c: Pos2) {
    let body = Rect::from_center_size(c + Vec2::new(0.0, 2.0), Vec2::new(9.0, 7.0));
    p.rect_filled(body, CornerRadius::same(1), Color32::WHITE);
    let pts: Vec<Pos2> = (0..=8).map(|i| {
        let a = std::f32::consts::PI * i as f32 / 8.0;
        c + Vec2::new(-3.0 * a.cos(), -1.5 - 3.5 * a.sin())
    }).collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::WHITE)));
}

/// True while a text field has the keyboard (the diagram canvas can hold focus too, and then the
/// editing shortcuts such as Ctrl+Z must still work).
pub(crate) fn text_field_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.focused()).map(|id| egui::TextEdit::load_state(ctx, id).is_some()).unwrap_or(false)
}

fn save_png(path: &std::path::Path, img: &egui::ColorImage) {
    let [w, h] = img.size;
    let mut buf = Vec::with_capacity(w * h * 4);
    for c in &img.pixels {
        buf.extend_from_slice(&[c.r(), c.g(), c.b(), 255]);
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = image::save_buffer(path, &buf, w as u32, h as u32, image::ColorType::Rgba8) {
        eprintln!("could not save {}: {e}", path.display());
    }
}

/// A section header with a caret ("Properties", "Documentation"); returns true when clicked.
fn section_header(ui: &mut egui::Ui, title: &str, open: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
    let p = ui.painter();
    let c = Pos2::new(rect.left() + 4.0, rect.center().y);
    let tri = if open {
        vec![c + Vec2::new(-2.0, -2.0), c + Vec2::new(6.0, -2.0), c + Vec2::new(2.0, 3.0)]
    } else {
        vec![c + Vec2::new(0.0, -4.0), c + Vec2::new(5.0, 0.0), c + Vec2::new(0.0, 4.0)]
    };
    p.add(egui::Shape::convex_polygon(tri, TEXT(), Stroke::NONE));
    p.text(Pos2::new(rect.left() + 15.0, rect.center().y), Align2::LEFT_CENTER, title, FontId::proportional(13.0), TEXT());
    resp.clicked()
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.window_ui(ui);
    }
}

impl App {
    /// One frame of the whole window.
    pub(crate) fn window_ui(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        // the theme follows the preference live (app-context.js:1518-1527)
        let light = self.pref("theme.default").as_str() == Some("light");
        if self.theme_applied != Some(light) {
            set_light_theme(light);
            crate::setup_style(&ctx);
            self.theme_applied = Some(light);
        }
        self.handle_close(&ctx);
        // the macOS title bar, and the splash screen until the start-up file is open (window.rs)
        if self.shell_ui(ui) {
            return;
        }
        self.api_poll(&ctx);
        self.poll_validation();
        // remember the window's bounds and maximized state (window.js:26-94); they are written with
        // the other preferences when they change
        if self.screenshot.is_none() {
            let (inner, outer, maximized) = ctx.input(|i| (i.viewport().inner_rect, i.viewport().outer_rect, i.viewport().maximized.unwrap_or(false)));
            if let (Some(inner), Some(outer)) = (inner, outer) {
                let mut ws = self.window_state.clone().unwrap_or_else(|| json!({}));
                ws["isMaximized"] = json!(maximized);
                // a maximized window keeps the bounds it had before
                if !maximized {
                    ws["x"] = json!(outer.min.x.round());
                    ws["y"] = json!(outer.min.y.round());
                    ws["width"] = json!(inner.width().round());
                    ws["height"] = json!(inner.height().round());
                }
                self.window_state = Some(ws);
            }
        }
        self.dropped_files(&ctx);
        // a command picked in the palette runs a frame later, once the palette is gone (command-palette.js:106-112)
        if let Some(id) = self.palette_run.take() {
            self.run_command(&ctx, id);
        }
        self.shortcuts(&ctx);
        let title = self.title();
        if ctx.input(|i| i.viewport().title.as_deref() != Some(title.as_str())) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }

        Panel::top("menu").exact_size(22.0).show_separator_line(false).frame(Frame::NONE.fill(LAYER0()).inner_margin(Margin::symmetric(6, 0))).show(ui, |ui| {
            self.menu_bar(ui);
        });
        if self.show_statusbar {
            Panel::bottom("status").exact_size(STATUSBAR_HEIGHT).show_separator_line(false).frame(Frame::NONE.fill(LAYER1())).show(ui, |ui| self.status_ui(ui));
        }
        use crate::splitter::{Edge, Split, KEEP_ABOVE, MIN_SPLIT};
        let area = ui.available_rect_before_wrap();
        let mut sidebar_rect = None;
        let mut navigator_rect = None;
        if self.show_sidebar {
            let w = self.panel_size("sidebar");
            let r = Panel::left("sidebar").exact_size(w).show_separator_line(false).frame(Frame::NONE.fill(LAYER0())).show(ui, |ui| {
                let mut toolbox = None;
                if self.show_toolbox {
                    // Working Diagrams keeps 40 px, also when the window gets smaller (sidebar-view.js:69-75, 150-156)
                    let max = (ui.available_height() - KEEP_ABOVE).max(MIN_SPLIT);
                    let h = self.panel_size("toolbox").min(max);
                    toolbox = Some((Panel::bottom("toolbox").exact_size(h).show_separator_line(false).frame(Frame::NONE.fill(LAYER1())).show(ui, |ui| self.toolbox_ui(ui)).response.rect, max));
                }
                self.working_ui(ui);
                if let Some((rect, max)) = toolbox {
                    self.splitter(ui, "toolbox", rect, Edge::Top, true, MIN_SPLIT, max, false);
                }
            });
            sidebar_rect = Some(r.response.rect);
        }
        if self.show_navigator {
            let w = self.panel_size("navigator");
            let r = Panel::right("navigator").exact_size(w).show_separator_line(false).frame(Frame::NONE.fill(LAYER0())).show(ui, |ui| {
                let mut editors = None;
                if self.show_editors {
                    // Model Explorer keeps 40 px (navigator-view.js:65-71, 151-157)
                    let max = (ui.available_height() - KEEP_ABOVE).max(MIN_SPLIT);
                    let h = self.panel_size("editors").min(max);
                    editors = Some((Panel::bottom("editors").exact_size(h).show_separator_line(false).frame(Frame::NONE.fill(LAYER1())).show(ui, |ui| self.editors_ui(ui)).response.rect, max));
                }
                self.explorer_ui(ui);
                if let Some((rect, max)) = editors {
                    self.splitter(ui, "editors", rect, Edge::Top, true, MIN_SPLIT, max, false);
                }
            });
            navigator_rect = Some(r.response.rect);
        }
        if self.show_toolbar {
            Panel::right("toolbar").exact_size(TOOLBAR_WIDTH).show_separator_line(false).frame(Frame::NONE.fill(TOOLBAR())).show(ui, |ui| self.toolbar_ui(ui));
        }
        // the panels under the diagram, each with a splitter on top (panel-manager.js:56-82, 149-209)
        let mut bottoms: Vec<(&'static str, Rect)> = vec![];
        let above = ui.available_rect_before_wrap().top();
        type Body = fn(&mut App, &mut egui::Ui);
        let panels: [(&'static str, bool, Body); 5] = [
            ("validation", self.show_validation, App::validation_ui),
            ("markdown", self.show_markdown, App::markdown_ui),
            ("thumbnails", self.show_thumbnails, App::thumbnails_ui),
            ("relationships", self.show_relationships, App::relationships_ui),
            // the Find Results panel sits under the diagram, above the status bar
            ("findResults", self.find_results.is_some(), App::find_results_ui),
        ];
        for (id, on, body) in panels {
            if on {
                let max = (ui.available_rect_before_wrap().bottom() - above - 20.0).max(MIN_SPLIT);
                let h = self.panel_size(id).min(max);
                let r = Panel::bottom(id).exact_size(h).show_separator_line(false).frame(Frame::NONE.fill(LAYER1())).show(ui, |ui| body(self, ui));
                bottoms.push((id, r.response.rect));
            }
        }
        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.canvas_ui(ui);
        });
        // splitters last, so that they win over the canvas under their edge
        let others = |a: &App, me: &str| {
            let side = |id: &str, shown: bool| if shown && id != me { a.panel_size(id) } else { 0.0 };
            side("sidebar", a.show_sidebar) + side("navigator", a.show_navigator) + if a.show_toolbar { TOOLBAR_WIDTH } else { 0.0 }
        };
        let max = (area.width() - others(self, "sidebar")).max(0.0);
        let rect = sidebar_rect.unwrap_or(Rect::from_x_y_ranges(area.left()..=area.left() + GRIP_HIDDEN, area.y_range()));
        match self.splitter(ui, "sidebar", rect, Edge::Right, self.show_sidebar, 0.0, max, true) {
            Some(Split::Hide) => self.show_sidebar = false,
            Some(Split::Resize(_)) => self.show_sidebar = true,
            None => {}
        }
        let max = (area.width() - others(self, "navigator")).max(0.0);
        let rect = navigator_rect.unwrap_or(Rect::from_x_y_ranges(area.right() - GRIP_HIDDEN..=area.right(), area.y_range()));
        match self.splitter(ui, "navigator", rect, Edge::Left, self.show_navigator, 0.0, max, true) {
            Some(Split::Hide) => self.show_navigator = false,
            Some(Split::Resize(_)) => self.show_navigator = true,
            None => {}
        }
        for (id, rect) in bottoms {
            let max = (rect.bottom() - above - 20.0).max(MIN_SPLIT);
            if self.splitter(ui, id, rect, Edge::Top, true, 0.0, max, true) == Some(Split::Hide) {
                match id {
                    "validation" => self.show_validation = false,
                    "markdown" => self.show_markdown = false,
                    "thumbnails" => self.show_thumbnails = false,
                    "relationships" => self.show_relationships = false,
                    _ => self.find_results = None,
                }
            }
        }
        if let Some(v) = self.pending_edit.take() {
            if self.repo.alive(v) {
                self.edit_view(v, None);
            }
        }
        self.take_pending_item();
        self.dialogs(&ctx);
        self.toasts_ui(&ctx);
        if self.screenshot.is_none() {
            self.store_prefs();
            self.auto_backup(std::time::Instant::now());
            ctx.request_repaint_after(crate::prefs::BACKUP_EVERY);
        }
        self.screenshots(&ctx);
        let _ = R::default();
    }
}

/// The app icon as a texture, loaded once (packaging/icons).
fn app_icon(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("app-icon");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(t);
    }
    let img = image::load_from_memory(include_bytes!("../packaging/icons/umlcraft-128.png")).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let t = ctx.load_texture("app-icon", egui::ColorImage::from_rgba_unmultiplied(size, &img.into_raw()), egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    Some(t)
}

/// A clickable colour swatch; a transparent colour shows a checkerboard.
fn swatch(ui: &mut egui::Ui, c: Color32, size: Vec2) -> egui::Response {
    let (rect, r) = ui.allocate_exact_size(size, egui::Sense::click());
    let p = ui.painter();
    if c.a() < 255 {
        let s = size.y / 2.0;
        p.rect_filled(rect, 0.0, Color32::WHITE);
        let mut x = rect.left();
        let mut i = 0;
        while x < rect.right() {
            let w = s.min(rect.right() - x);
            let top = if i % 2 == 0 { rect.top() } else { rect.top() + s };
            p.rect_filled(egui::Rect::from_min_size(Pos2::new(x, top), Vec2::new(w, s)), 0.0, Color32::from_gray(200));
            x += s;
            i += 1;
        }
    }
    p.rect_filled(rect, 0.0, c);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, if r.hovered() { TEXT() } else { BORDER() }), egui::StrokeKind::Inside);
    r
}


impl App {
    /// The elements a type or stereotype field suggests for a text: those of the field's type whose
    /// name contains it, case-insensitive (repository.js:2336-2344).
    pub fn autocomplete_hits(&self, field: &str, text: &str) -> Vec<Eid> {
        let ty = if field == "stereotype" { "UMLStereotype" } else { crate::picker::field_type(field) };
        let q = text.trim().to_lowercase();
        self.repo.all().filter(|e| !self.is_view(*e) && self.repo.kind_of(*e, ty) && self.repo.name(*e).to_lowercase().contains(&q)).collect()
    }

    /// The suggestions under a type or stereotype field: elements of the field's type whose name
    /// contains the text, case-insensitive, with icon and " — namespace" (repository.js:2336-2344;
    /// property-editor-view.js:149-158, 589-640). Returns the one clicked. Whether the pointer is over
    /// the list is kept under `over`, so the field's loss of focus to a click is not taken as typing.
    fn autocomplete_list(&self, ui: &egui::Ui, field_resp: &egui::Response, over: egui::Id, field: &str, text: &str) -> Option<Eid> {
        let hits = self.autocomplete_hits(field, text);
        if hits.is_empty() {
            ui.ctx().data_mut(|d| d.insert_temp(over, false));
            return None;
        }
        let mut picked = None;
        let area = egui::Area::new(over.with("list")).order(egui::Order::Tooltip).fixed_pos(field_resp.rect.left_bottom()).show(ui.ctx(), |ui| {
            Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(field_resp.rect.width());
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for e in hits {
                        let ns = self.repo.parent(e).map(|p| self.repo.name(p)).unwrap_or_default();
                        let row = ui.horizontal(|ui| {
                            let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            icons::draw(ui.painter(), ir, icons::key_for(&self.repo, e));
                            let mut job = egui::text::LayoutJob::default();
                            job.append(&self.label_of(e), 0.0, egui::TextFormat { font_id: FontId::proportional(12.0), color: TEXT(), ..Default::default() });
                            if !ns.is_empty() {
                                job.append(&format!(" — {ns}"), 5.0, egui::TextFormat { font_id: FontId::proportional(11.0), color: DIM(), ..Default::default() });
                            }
                            ui.add(egui::Label::new(job).sense(Sense::click()).wrap_mode(egui::TextWrapMode::Extend))
                        });
                        if row.inner.clicked() {
                            picked = Some(e);
                        }
                    }
                });
            });
        });
        let hovered = area.response.contains_pointer();
        ui.ctx().data_mut(|d| d.insert_temp(over, hovered));
        picked
    }
}

/// The whole number a text starts with, as JavaScript's parseInt reads it: blanks, a sign, digits
/// ("12.7" and "12px" are 12); None without digits.
pub fn parse_int(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let (sign, rest) = match t.strip_prefix('-') {
        Some(r) => (-1, r),
        None => (1, t.strip_prefix('+').unwrap_or(t)),
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<i64>().ok().map(|n| sign * n)
}

/// The number a text starts with, as JavaScript's parseFloat reads it ("2.5e3x" is 2500); None
/// without one.
pub fn parse_real(s: &str) -> Option<f64> {
    let t = s.trim_start();
    // the longest prefix that parses (sign, digits, point, exponent)
    let ends: Vec<usize> = t.char_indices().map(|(i, c)| i + c.len_utf8()).collect();
    ends.into_iter().rev().find_map(|e| {
        let p = &t[..e];
        if p.chars().all(|c| c.is_ascii_digit() || matches!(c, '+' | '-' | '.' | 'e' | 'E')) {
            p.parse::<f64>().ok().filter(|v| v.is_finite())
        } else {
            None
        }
    })
}

/// A soft shadow under the top edge of a scrolled list.
fn scroll_shadow(p: &egui::Painter, r: Rect) {
    for i in 0..6 {
        let a = (60 - i * 10) as u8;
        let y = r.top() + i as f32;
        p.hline(r.x_range(), y, Stroke::new(1.0, Color32::from_black_alpha(a)));
    }
}

/// Give a dialog's primary button the keyboard when nothing else has it, as the original does when a
/// dialog opens (dialog-manager.js:242-313).
pub(crate) fn focus_primary(ui: &egui::Ui, button: &egui::Response) {
    if ui.memory(|m| m.focused().is_none()) {
        button.request_focus();
    }
}
