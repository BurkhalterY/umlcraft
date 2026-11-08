//! Element kinds UMLCraft knows: parent kind and the fields in the order the .mdj format writes
//! them, with their defaults (a field equal to its default is left out of the file).
//! See MECHANICS.md "File format".

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

pub struct Kind {
    pub sup: Option<&'static str>,
    pub attrs: Vec<(&'static str, Option<Value>)>,
}

fn table() -> &'static HashMap<&'static str, Kind> {
    static T: OnceLock<HashMap<&'static str, Kind>> = OnceLock::new();
    T.get_or_init(build)
}

fn build() -> HashMap<&'static str, Kind> {
    let mut t: HashMap<&'static str, Kind> = HashMap::new();
    let mut def = |name: &'static str, sup: Option<&'static str>, attrs: Vec<(&'static str, Option<Value>)>| {
        t.insert(name, Kind { sup, attrs });
    };
    let n = || None;
    let d = |v: Value| Some(v);

    def("Element", None, vec![("_id", n()), ("_parent", n())]);
    def("Model", Some("Element"), vec![("name", n()), ("ownedElements", n())]);
    def("ExtensibleModel", Some("Model"), vec![("documentation", d(json!(""))), ("tags", n())]);
    def("Tag", Some("Model"), vec![("kind", n()), ("value", n()), ("reference", n()), ("checked", d(json!(false))), ("number", d(json!(0))), ("options", d(json!(""))), ("hidden", d(json!(false)))]);
    def("Relationship", Some("ExtensibleModel"), vec![]);
    // a link to an element or a URL; showName shows its own name instead (core.js:1308-1334)
    def("Hyperlink", Some("Model"), vec![("reference", n()), ("url", d(json!(""))), ("showName", d(json!(false)))]);
    def("DirectedRelationship", Some("Relationship"), vec![("source", n()), ("target", n())]);
    def("RelationshipEnd", Some("ExtensibleModel"), vec![("reference", n())]);
    def("UndirectedRelationship", Some("Relationship"), vec![("end1", n()), ("end2", n())]);
    def("Diagram", Some("ExtensibleModel"), vec![("visible", d(json!(true))), ("defaultDiagram", d(json!(false))), ("ownedViews", n())]);
    def("Project", Some("ExtensibleModel"), vec![("author", n()), ("company", n()), ("copyright", n()), ("version", n()), ("documentVersion", d(json!(0)))]);

    def("View", Some("Element"), vec![
        ("model", n()), ("subViews", n()), ("containerView", n()), ("containedViews", n()), ("visible", d(json!(true))), ("enabled", d(json!(true))),
        ("lineColor", d(json!("#000000"))), ("fillColor", d(json!("#ffffff"))), ("fontColor", d(json!("#000000"))), ("font", n()), ("parentStyle", n()),
        ("showShadow", d(json!(true))), ("containerChangeable", d(json!(false))), ("containerExtending", d(json!(false))),
    ]);
    def("NodeView", Some("View"), vec![("left", d(json!(0))), ("top", d(json!(0))), ("width", d(json!(0))), ("height", d(json!(0))), ("autoResize", d(json!(false)))]);
    def("EdgeView", Some("View"), vec![("head", n()), ("tail", n()), ("lineStyle", d(json!(0))), ("points", n())]);
    // a free line: points only; line mode 0 solid, 1 dotted, 2 dashed and end styles 0 flat, 1 stick
    // arrow, 2 solid arrow (style-editor-view.js:296-319; the values are estimated)
    def("FreelineEdgeView", Some("EdgeView"), vec![("lineMode", d(json!(0))), ("tailEndStyle", d(json!(0))), ("headEndStyle", d(json!(0)))]);
    let label_attrs = || vec![("underline", d(json!(false))), ("text", d(json!(""))), ("horizontalAlignment", d(json!(2))), ("verticalAlignment", d(json!(5))), ("wordWrap", d(json!(false)))];
    def("LabelView", Some("NodeView"), label_attrs());
    def("ParasiticView", Some("NodeView"), vec![("alpha", n()), ("distance", n())]);
    def("NodeParasiticView", Some("ParasiticView"), vec![("hostNode", n())]);
    def("EdgeParasiticView", Some("ParasiticView"), vec![("hostEdge", n()), ("edgePosition", d(json!(0)))]);
    def("NodeLabelView", Some("NodeParasiticView"), label_attrs());
    def("EdgeLabelView", Some("EdgeParasiticView"), label_attrs());

    // UML models
    def("UMLModelElement", Some("ExtensibleModel"), vec![("stereotype", n()), ("visibility", d(json!("public"))), ("templateParameters", n())]);
    def("UMLFeature", Some("UMLModelElement"), vec![("isStatic", d(json!(false))), ("isLeaf", d(json!(false))), ("featureDirection", d(json!("provided")))]);
    def("UMLStructuralFeature", Some("UMLFeature"), vec![("type", n()), ("multiplicity", d(json!(""))), ("isReadOnly", d(json!(false))), ("isOrdered", d(json!(false))), ("isUnique", d(json!(false))), ("defaultValue", d(json!("")))]);
    // a port: an attribute shown on its classifier's border (uml/metamodel.json)
    def("UMLPort", Some("UMLAttribute"), vec![("isBehavior", d(json!(false))), ("isService", d(json!(false))), ("isConjugated", d(json!(false)))]);
    // an action's pins (elements.js:3080-3090)
    def("UMLPin", Some("UMLStructuralFeature"), vec![]);
    def("UMLInputPin", Some("UMLPin"), vec![]);
    def("UMLOutputPin", Some("UMLPin"), vec![]);
    // an expansion region's input or output (elements.js:3095)
    def("UMLExpansionNode", Some("UMLPin"), vec![]);
    def("UMLParameter", Some("UMLStructuralFeature"), vec![("direction", d(json!("in")))]);
    def("UMLBehavioralFeature", Some("UMLFeature"), vec![("parameters", n()), ("raisedExceptions", n()), ("concurrency", d(json!("sequential")))]);
    def("UMLAttribute", Some("UMLStructuralFeature"), vec![("isDerived", d(json!(false))), ("aggregation", d(json!("none"))), ("isID", d(json!(false)))]);
    def("UMLOperation", Some("UMLBehavioralFeature"), vec![("isQuery", d(json!(false))), ("isAbstract", d(json!(false))), ("specification", d(json!(""))), ("preconditions", n()), ("bodyConditions", n()), ("postconditions", n())]);
    def("UMLReception", Some("UMLBehavioralFeature"), vec![("signal", n())]);
    def("UMLClassifier", Some("UMLModelElement"), vec![("attributes", n()), ("operations", n()), ("receptions", n()), ("behaviors", n()), ("isAbstract", d(json!(false))), ("isFinalSpecialization", d(json!(false))), ("isLeaf", d(json!(false)))]);
    def("UMLDirectedRelationship", Some("DirectedRelationship"), vec![("stereotype", n()), ("visibility", d(json!("public")))]);
    def("UMLRelationshipEnd", Some("RelationshipEnd"), vec![
        ("stereotype", d(json!(""))), ("visibility", d(json!("public"))), ("navigable", d(json!("unspecified"))), ("aggregation", d(json!("none"))), ("multiplicity", d(json!(""))),
        ("defaultValue", d(json!(""))), ("isReadOnly", d(json!(false))), ("isOrdered", d(json!(false))), ("isUnique", d(json!(false))), ("isDerived", d(json!(false))), ("isID", d(json!(false))),
    ]);
    def("UMLUndirectedRelationship", Some("UndirectedRelationship"), vec![("stereotype", n()), ("visibility", d(json!("public")))]);
    def("UMLConstraint", Some("UMLModelElement"), vec![("specification", d(json!(""))), ("constrainedElements", n())]);
    def("UMLPackage", Some("UMLModelElement"), vec![("importedElements", n())]);
    def("UMLModel", Some("UMLPackage"), vec![("viewpoint", d(json!("")))]);
    def("UMLProfile", Some("UMLPackage"), vec![]);
    def("UMLClass", Some("UMLClassifier"), vec![("isActive", d(json!(false)))]);
    def("UMLStereotype", Some("UMLClass"), vec![("icon", n())]);
    // a stereotype's icon: an icon script in `content` (or an image), drawn width × height at least
    // (uml/metamodel.json; a new one is 40 × 40, elements.js:4071-4090)
    def("UMLImage", Some("UMLModelElement"), vec![("width", d(json!(0))), ("height", d(json!(0))), ("image", n()), ("content", d(json!(""))), ("smallIcon", d(json!("")))]);
    // profiles: a metaclass names a UML type; an extension joins a stereotype to it (names estimated)
    def("UMLMetaClass", Some("UMLModelElement"), vec![]);
    def("UMLExtension", Some("UMLDirectedRelationship"), vec![]);
    // a collaboration, and a use of one typed by it (uml/metamodel.json)
    def("UMLCollaboration", Some("UMLClassifier"), vec![]);
    def("UMLCollaborationUse", Some("UMLModelElement"), vec![("type", n())]);
    // the diamond an n-ary association's ends meet at (elements.js:2096)
    def("UMLNaryAssociationNode", Some("UMLClassifier"), vec![]);
    def("UMLInterface", Some("UMLClassifier"), vec![]);
    def("UMLSignal", Some("UMLClassifier"), vec![]);
    def("UMLDataType", Some("UMLClassifier"), vec![]);
    def("UMLPrimitiveType", Some("UMLDataType"), vec![]);
    def("UMLEnumerationLiteral", Some("UMLModelElement"), vec![]);
    def("UMLTemplateParameter", Some("UMLModelElement"), vec![("parameterType", d(json!(""))), ("defaultValue", d(json!("")))]);
    def("UMLEnumeration", Some("UMLDataType"), vec![("literals", n())]);
    def("UMLDependency", Some("UMLDirectedRelationship"), vec![("mapping", d(json!("")))]);
    // a role binding: from a collaboration use to the part that plays a role (uml/metamodel.json)
    def("UMLRoleBinding", Some("UMLDependency"), vec![("roleName", d(json!("")))]);
    def("UMLAbstraction", Some("UMLDependency"), vec![]);
    def("UMLRealization", Some("UMLAbstraction"), vec![]);
    def("UMLGeneralization", Some("UMLDirectedRelationship"), vec![("discriminator", d(json!("")))]);
    def("UMLInterfaceRealization", Some("UMLRealization"), vec![]);
    def("UMLAssociationEnd", Some("UMLRelationshipEnd"), vec![("qualifiers", n()), ("ownerAttribute", n())]);
    def("UMLAssociation", Some("UMLUndirectedRelationship"), vec![("isDerived", d(json!(false)))]);
    def("UMLComponent", Some("UMLClassifier"), vec![("isIndirectlyInstantiated", d(json!(true)))]);
    def("UMLSubsystem", Some("UMLComponent"), vec![]);
    def("UMLArtifact", Some("UMLClassifier"), vec![("fileName", d(json!("")))]);
    def("UMLNode", Some("UMLClassifier"), vec![]);
    def("UMLComponentRealization", Some("UMLRealization"), vec![]);
    def("UMLDeployment", Some("UMLDependency"), vec![]);
    // a class bound to a template, with its parameter substitutions (uml/metamodel.json)
    def("UMLTemplateBinding", Some("UMLDirectedRelationship"), vec![("parameterSubstitutions", n())]);
    def("UMLCommunicationPath", Some("UMLAssociation"), vec![]);
    // behaviours a classifier or package owns (uml/metamodel.json)
    def("UMLBehavior", Some("UMLModelElement"), vec![("isReentrant", d(json!(true))), ("parameters", n()), ("preconditions", n()), ("postconditions", n())]);
    def("UMLOpaqueBehavior", Some("UMLBehavior"), vec![]);
    def("UMLInteractionFragment", Some("UMLBehavior"), vec![]);
    def("UMLInteraction", Some("UMLInteractionFragment"), vec![("messages", n()), ("participants", n()), ("fragments", n())]);
    def("UMLStateMachine", Some("UMLBehavior"), vec![("regions", n())]);
    def("UMLCombinedFragment", Some("UMLInteractionFragment"), vec![("interactionOperator", n()), ("operands", n())]);
    // sequence diagrams (uml/metamodel.json, elements.js:3603-3990)
    def("UMLMessageEndpoint", Some("UMLModelElement"), vec![]);
    def("UMLLifeline", Some("UMLMessageEndpoint"), vec![("selector", d(json!(""))), ("represent", n()), ("isMultiInstance", d(json!(false)))]);
    def("UMLGate", Some("UMLMessageEndpoint"), vec![]);
    def("UMLEndpoint", Some("UMLMessageEndpoint"), vec![]);
    def("UMLMessage", Some("UMLDirectedRelationship"), vec![
        ("messageSort", d(json!("synchCall"))), ("signature", n()), ("connector", n()), ("sequenceNumber", d(json!(""))), ("arguments", d(json!(""))), ("assignmentTarget", d(json!(""))),
        ("guard", d(json!(""))), ("iteration", d(json!(""))), ("isConcurrentIteration", d(json!(false))),
    ]);
    def("UMLStateInvariant", Some("UMLInteractionFragment"), vec![("covered", n()), ("invariant", d(json!("")))]);
    def("UMLContinuation", Some("UMLInteractionFragment"), vec![("setting", d(json!(false)))]);
    def("UMLInteractionUse", Some("UMLInteractionFragment"), vec![("refersTo", n()), ("arguments", d(json!(""))), ("returnValue", d(json!(""))), ("returnValueRecipient", n())]);
    def("UMLInteractionOperand", Some("UMLInteractionFragment"), vec![("guard", d(json!("")))]);
    def("UMLRegion", Some("UMLModelElement"), vec![("vertices", n()), ("transitions", n())]);
    // states, transitions, events and actions, for the explorer (uml/metamodel.json)
    def("UMLVertex", Some("UMLModelElement"), vec![]);
    def("UMLState", Some("UMLVertex"), vec![("regions", n()), ("entryActivities", n()), ("doActivities", n()), ("exitActivities", n()), ("submachine", n()), ("connections", n())]);
    def("UMLPseudostate", Some("UMLVertex"), vec![("kind", d(json!("initial")))]);
    // a reference to entry or exit points of a submachine state (elements.js:2900-2910)
    def("UMLConnectionPointReference", Some("UMLVertex"), vec![("entry", n()), ("exit", n())]);
    def("UMLFinalState", Some("UMLState"), vec![]);
    def("UMLTransition", Some("UMLDirectedRelationship"), vec![("kind", d(json!("external"))), ("guard", d(json!(""))), ("triggers", n()), ("effects", n())]);
    def("UMLEvent", Some("UMLModelElement"), vec![("kind", d(json!("anyReceive"))), ("value", d(json!(""))), ("expression", d(json!(""))), ("targetOperation", n()), ("targetSignal", n())]);
    def("UMLActivityNode", Some("UMLModelElement"), vec![]);
    def("UMLAction", Some("UMLActivityNode"), vec![
        ("kind", d(json!("opaque"))), ("inputs", n()), ("outputs", n()), ("triggers", n()), ("target", n()), ("subactivity", n()), ("isLocallyReentrant", d(json!(false))),
        ("isSynchronous", d(json!(true))), ("language", d(json!(""))), ("body", n()), ("localPreconditions", n()), ("localPostconditions", n()),
    ]);
    // activity nodes and edges (uml/metamodel.json, elements.js:3100-3580)
    def("UMLObjectNode", Some("UMLActivityNode"), vec![("type", n()), ("isControlType", d(json!(false))), ("ordering", d(json!("FIFO"))), ("inStates", n())]);
    def("UMLCentralBufferNode", Some("UMLObjectNode"), vec![]);
    def("UMLDataStoreNode", Some("UMLCentralBufferNode"), vec![]);
    def("UMLActivityParameterNode", Some("UMLObjectNode"), vec![("parameter", n())]);
    def("UMLControlNode", Some("UMLActivityNode"), vec![]);
    def("UMLInitialNode", Some("UMLControlNode"), vec![]);
    def("UMLFinalNode", Some("UMLControlNode"), vec![]);
    def("UMLActivityFinalNode", Some("UMLFinalNode"), vec![]);
    def("UMLFlowFinalNode", Some("UMLFinalNode"), vec![]);
    def("UMLForkNode", Some("UMLControlNode"), vec![]);
    def("UMLJoinNode", Some("UMLControlNode"), vec![]);
    def("UMLMergeNode", Some("UMLControlNode"), vec![]);
    def("UMLDecisionNode", Some("UMLControlNode"), vec![]);
    def("UMLActivityGroup", Some("UMLModelElement"), vec![("subgroups", n()), ("nodes", n()), ("edges", n())]);
    def("UMLActivityPartition", Some("UMLActivityGroup"), vec![]);
    def("UMLInterruptibleActivityRegion", Some("UMLActivityGroup"), vec![]);
    // a structured activity is an action holding nodes and edges; an expansion region one with a mode
    // (elements.js:3469-3527)
    def("UMLStructuredActivityNode", Some("UMLAction"), vec![("mustIsolate", d(json!(false))), ("nodes", n()), ("edges", n())]);
    def("UMLExpansionRegion", Some("UMLStructuredActivityNode"), vec![("mode", d(json!("iterative")))]);
    def("UMLExceptionHandler", Some("UMLDirectedRelationship"), vec![("exceptionTypes", n()), ("handlerBody", n())]);
    def("UMLActivityEdgeConnector", Some("UMLActivityNode"), vec![]);
    def("UMLActivityEdge", Some("UMLDirectedRelationship"), vec![("guard", d(json!(""))), ("weight", d(json!("")))]);
    def("UMLControlFlow", Some("UMLActivityEdge"), vec![]);
    def("UMLObjectFlow", Some("UMLActivityEdge"), vec![]);
    def("UMLActivityInterrupt", Some("UMLActivityEdge"), vec![]);
    def("UMLActivity", Some("UMLBehavior"), vec![("isReadOnly", d(json!(false))), ("isSingleExecution", d(json!(false))), ("nodes", n()), ("edges", n()), ("groups", n())]);
    def("UMLTemplateParameterSubstitution", Some("UMLModelElement"), vec![("formal", n()), ("actual", n())]);
    def("UMLExtensionPoint", Some("UMLModelElement"), vec![("location", d(json!("")))]);
    def("UMLUseCase", Some("UMLClassifier"), vec![("extensionPoints", n())]);
    def("UMLActor", Some("UMLClassifier"), vec![]);
    def("UMLInclude", Some("UMLDirectedRelationship"), vec![]);
    // an association class: the class and the association it details (estimated field names)
    def("UMLAssociationClassLink", Some("UMLModelElement"), vec![("classSide", n()), ("associationSide", n())]);
    // information flows: an item conveyed along a flow between elements (field names estimated)
    def("UMLInformationItem", Some("UMLClassifier"), vec![("represented", n())]);
    def("UMLInformationFlow", Some("UMLDirectedRelationship"), vec![("conveyed", n())]);
    // objects, their slots and the links between them (object diagrams; field names estimated)
    def("UMLInstance", Some("UMLModelElement"), vec![("classifier", n()), ("slots", n())]);
    def("UMLObject", Some("UMLInstance"), vec![]);
    for name in ["UMLArtifactInstance", "UMLComponentInstance", "UMLNodeInstance"] {
        def(name, Some("UMLInstance"), vec![]);
    }
    def("UMLSlot", Some("UMLModelElement"), vec![("definingFeature", n()), ("type", n()), ("value", d(json!("")))]);
    def("UMLLink", Some("UMLUndirectedRelationship"), vec![("association", n())]);
    def("UMLLinkEnd", Some("UMLRelationshipEnd"), vec![]);
    // a connector joins two features, ports or parts (uml/metamodel.json)
    def("UMLConnector", Some("UMLUndirectedRelationship"), vec![("type", n()), ("kind", d(json!("assembly")))]);
    def("UMLConnectorEnd", Some("UMLRelationshipEnd"), vec![]);
    def("UMLLinkObjectLink", Some("UMLModelElement"), vec![("objectSide", n()), ("linkSide", n())]);
    def("UMLExtend", Some("UMLDirectedRelationship"), vec![("condition", d(json!(""))), ("extensionLocations", n())]);
    def("UMLUseCaseSubject", Some("UMLSubsystem"), vec![("represent", n())]);
    def("UMLDiagram", Some("Diagram"), vec![]);
    def("UMLClassDiagram", Some("UMLDiagram"), vec![]);
    def("UMLCompositeStructureDiagram", Some("UMLDiagram"), vec![]);
    def("UMLUseCaseDiagram", Some("UMLDiagram"), vec![]);
    def("UMLStatechartDiagram", Some("UMLDiagram"), vec![]);
    def("UMLActivityDiagram", Some("UMLDiagram"), vec![]);
    def("UMLPackageDiagram", Some("UMLDiagram"), vec![]);
    def("UMLComponentDiagram", Some("UMLDiagram"), vec![]);
    def("UMLDeploymentDiagram", Some("UMLDiagram"), vec![]);
    def("UMLObjectDiagram", Some("UMLDiagram"), vec![]);
    def("UMLInformationFlowDiagram", Some("UMLDiagram"), vec![]);
    def("UMLProfileDiagram", Some("UMLDiagram"), vec![]);
    def("UMLSequenceDiagram", Some("UMLDiagram"), vec![("showSequenceNumber", d(json!(true))), ("sequenceNumbering", d(json!("auto"))), ("showSignature", d(json!(true))), ("showActivation", d(json!(true)))]);
    def("UMLCommunicationDiagram", Some("UMLDiagram"), vec![("showSequenceNumber", d(json!(true))), ("sequenceNumbering", d(json!("auto"))), ("showSignature", d(json!(true)))]);

    // UML views
    def("UMLCompartmentView", Some("NodeView"), vec![]);
    def("UMLNameCompartmentView", Some("UMLCompartmentView"), vec![("stereotypeLabel", n()), ("nameLabel", n()), ("namespaceLabel", n()), ("propertyLabel", n())]);
    def("UMLListCompartmentView", Some("UMLCompartmentView"), vec![]);
    for name in ["UMLAttributeView", "UMLOperationView", "UMLReceptionView", "UMLEnumerationLiteralView", "UMLExtensionPointView", "UMLSlotView"] {
        def(name, Some("LabelView"), vec![]);
    }
    for name in [
        "UMLAttributeCompartmentView", "UMLOperationCompartmentView", "UMLReceptionCompartmentView", "UMLTemplateParameterCompartmentView",
        "UMLEnumerationLiteralCompartmentView", "UMLExtensionPointCompartmentView", "UMLQualifierCompartmentView", "UMLSlotCompartmentView",
    ] {
        def(name, Some("UMLListCompartmentView"), vec![]);
    }
    def("UMLGeneralNodeView", Some("NodeView"), vec![
        ("stereotypeDisplay", d(json!("label"))), ("showVisibility", d(json!(true))), ("showNamespace", d(json!(false))), ("showProperty", d(json!(true))),
        ("showType", d(json!(true))), ("nameCompartment", n()), ("wordWrap", d(json!(false))),
    ]);
    def("UMLClassifierView", Some("UMLGeneralNodeView"), vec![
        ("suppressAttributes", d(json!(false))), ("suppressOperations", d(json!(false))), ("suppressReceptions", d(json!(true))), ("showMultiplicity", d(json!(true))),
        ("showOperationSignature", d(json!(true))), ("attributeCompartment", n()), ("operationCompartment", n()), ("receptionCompartment", n()),
        ("templateParameterCompartment", n()),
    ]);
    for name in ["UMLStereotypeView", "UMLInformationItemView", "UMLClassView", "UMLInterfaceView", "UMLDataTypeView", "UMLPrimitiveTypeView", "UMLSignalView", "UMLActorView", "UMLComponentView", "UMLArtifactView", "UMLNodeView"] {
        def(name, Some("UMLClassifierView"), vec![]);
    }
    def("UMLEnumerationView", Some("UMLClassifierView"), vec![("suppressLiterals", d(json!(false))), ("enumerationLiteralCompartment", n())]);
    def("UMLPackageView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLSubsystemView", Some("UMLPackageView"), vec![]);
    def("UMLUseCaseView", Some("UMLClassifierView"), vec![("extensionPointCompartment", n())]);
    def("UMLUseCaseSubjectView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLObjectView", Some("UMLGeneralNodeView"), vec![("slotCompartment", n())]);
    def("UMLCollaborationView", Some("UMLGeneralNodeView"), vec![("templateParameterCompartment", n())]);
    // statecharts: a state with its name, pseudostates and final states as small marks (estimated fields)
    def("UMLStateView", Some("UMLGeneralNodeView"), vec![]);
    // a pseudostate is a floating node with hidden labels, a final state a plain node (elements.js:8835, 9111)
    def("UMLPseudostateView", Some("UMLFloatingNodeView"), vec![]);
    def("UMLFinalStateView", Some("NodeView"), vec![]);
    def("UMLConnectionPointReferenceView", Some("UMLFloatingNodeView"), vec![]);
    // activities: actions and object nodes with their name, control nodes as marks without one
    // (elements.js:9903-10300)
    def("UMLActionView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLObjectNodeView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLCentralBufferNodeView", Some("UMLObjectNodeView"), vec![]);
    def("UMLDataStoreNodeView", Some("UMLObjectNodeView"), vec![]);
    def("UMLControlNodeView", Some("NodeView"), vec![]);
    def("UMLActivityParameterNodeView", Some("UMLObjectNodeView"), vec![]);
    def("UMLInterruptibleActivityRegionView", Some("NodeView"), vec![]);
    def("UMLActivityEdgeConnectorView", Some("NodeView"), vec![]);
    def("UMLStructuredActivityNodeView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLExpansionRegionView", Some("UMLStructuredActivityNodeView"), vec![]);
    def("UMLSwimlaneView", Some("NodeView"), vec![("nameLabel", n()), ("isVertical", d(json!(true)))]);
    def("UMLCollaborationUseView", Some("UMLGeneralNodeView"), vec![]);
    // sequence diagram views (uml/metamodel.json, elements.js:11017-12800)
    def("UMLLinePartView", Some("NodeView"), vec![]);
    def("UMLSeqLifelineView", Some("UMLGeneralNodeView"), vec![("linePart", n())]);
    def("UMLCommLifelineView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLCommMessageView", Some("EdgeNodeView"), vec![("nameLabel", n()), ("stereotypeLabel", n()), ("propertyLabel", n()), ("showProperty", d(json!(true))), ("showType", d(json!(true)))]);
    def("UMLMessageEndpointView", Some("NodeView"), vec![]);
    def("UMLEndpointView", Some("UMLMessageEndpointView"), vec![]);
    def("UMLGateView", Some("UMLMessageEndpointView"), vec![]);
    def("UMLActivationView", Some("NodeView"), vec![]);
    def("UMLSeqMessageView", Some("EdgeView"), vec![("nameLabel", n()), ("stereotypeLabel", n()), ("propertyLabel", n()), ("activation", n()), ("showProperty", d(json!(true))), ("showType", d(json!(true)))]);
    def("UMLStateInvariantView", Some("NodeView"), vec![("invariantLabel", n())]);
    def("UMLContinuationView", Some("NodeView"), vec![("nameLabel", n())]);
    def("UMLInteractionUseView", Some("NodeView"), vec![("nameLabel", n()), ("frameTypeLabel", n()), ("wordWrap", d(json!(false)))]);
    def("UMLInteractionOperandView", Some("NodeView"), vec![("guardLabel", n())]);
    def("UMLListCompartmentView", Some("NodeView"), vec![]);
    def("UMLInteractionOperandCompartmentView", Some("UMLListCompartmentView"), vec![]);
    def("UMLCombinedFragmentView", Some("UMLCustomFrameView"), vec![("operandCompartment", n())]);
    def("UMLPartView", Some("UMLGeneralNodeView"), vec![("showMultiplicity", d(json!(true)))]);
    def("UMLMetaClassView", Some("UMLGeneralNodeView"), vec![]);
    // artifact, component and node instances: drawn like their classifiers (base type estimated)
    for name in ["UMLArtifactInstanceView", "UMLComponentInstanceView", "UMLNodeInstanceView"] {
        def(name, Some("UMLObjectView"), vec![]);
    }
    // a frame shows a model element (often a diagram) by name, with a tab naming the diagram kind
    def("UMLCustomFrameView", Some("NodeView"), vec![("nameLabel", n()), ("frameTypeLabel", n())]);
    def("UMLFrameView", Some("UMLCustomFrameView"), vec![("showType", d(json!(false))), ("showNamespace", d(json!(false))), ("showDiagramName", d(json!(false)))]);
    // plain shapes: a rectangle, a rounded rectangle, an ellipse (elements.js:14332-14378)
    def("ShapeView", Some("NodeView"), vec![]);
    // a picture, kept as a data URL with its natural size (uml/metamodel.json ImageView)
    def("ImageView", Some("NodeView"), vec![("imageWidth", d(json!(0))), ("imageHeight", d(json!(0))), ("imageData", d(json!("")))]);
    for name in ["RectangleView", "RoundRectView", "EllipseView"] {
        def(name, Some("ShapeView"), vec![]);
    }
    // a node with floating labels around it, placed by angle and distance from its centre (elements.js:5162-5188)
    def("UMLFloatingNodeView", Some("NodeView"), vec![("nameLabel", n()), ("stereotypeLabel", n()), ("propertyLabel", n()), ("showProperty", d(json!(true)))]);
    def("UMLNaryAssociationNodeView", Some("UMLFloatingNodeView"), vec![("showVisibility", d(json!(false)))]);
    // a pin: an 18 px square on its action's border (elements.js:9662-9800)
    def("UMLPinView", Some("UMLFloatingNodeView"), vec![]);
    def("UMLInputPinView", Some("UMLPinView"), vec![]);
    def("UMLOutputPinView", Some("UMLPinView"), vec![]);
    def("UMLExpansionNodeView", Some("UMLPinView"), vec![]);
    def("UMLPortView", Some("UMLFloatingNodeView"), vec![("showVisibility", d(json!(false))), ("showType", d(json!(true))), ("showMultiplicity", d(json!(true)))]);
    def("HyperlinkView", Some("NodeView"), vec![("nameLabel", n()), ("typeLabel", n())]);
    def("UMLCustomTextView", Some("NodeView"), vec![("text", d(json!(""))), ("wordWrap", d(json!(true))), ("horzAlign", d(json!(0))), ("vertAlign", d(json!(3)))]);
    def("UMLCustomNoteView", Some("UMLCustomTextView"), vec![]);
    def("UMLNoteView", Some("UMLCustomNoteView"), vec![]);
    def("UMLTextView", Some("UMLCustomTextView"), vec![]);
    def("UMLTextBoxView", Some("UMLCustomTextView"), vec![]);
    // a constraint shows its specification in braces; its link is a dashed line (base types estimated)
    def("UMLConstraintView", Some("UMLCustomTextView"), vec![]);
    def("UMLConstraintLinkView", Some("EdgeView"), vec![]);
    def("UMLNoteLinkView", Some("EdgeView"), vec![]);
    def("UMLContainmentView", Some("EdgeView"), vec![]);
    def("UMLAssociationClassLinkView", Some("EdgeView"), vec![]);
    def("UMLLinkObjectLinkView", Some("EdgeView"), vec![]);
    def("UMLGeneralEdgeView", Some("EdgeView"), vec![
        ("stereotypeDisplay", d(json!("label"))), ("showVisibility", d(json!(false))), ("showProperty", d(json!(true))),
        ("nameLabel", n()), ("stereotypeLabel", n()), ("propertyLabel", n()),
    ]);
    def("UMLUndirectedRelationshipView", Some("UMLGeneralEdgeView"), vec![
        ("showMultiplicity", d(json!(true))), ("showType", d(json!(true))), ("showEndOrder", d(json!(0))), ("tailRoleNameLabel", n()), ("tailPropertyLabel", n()),
        ("tailMultiplicityLabel", n()), ("headRoleNameLabel", n()), ("headPropertyLabel", n()), ("headMultiplicityLabel", n()),
    ]);
    def("UMLAssociationView", Some("UMLUndirectedRelationshipView"), vec![("tailQualifiersCompartment", n()), ("headQualifiersCompartment", n())]);
    for name in ["UMLGeneralizationView", "UMLDependencyView", "UMLRealizationView", "UMLInterfaceRealizationView", "UMLIncludeView", "UMLExtendView", "UMLComponentRealizationView", "UMLDeploymentView", "UMLTemplateBindingView", "UMLInformationFlowView", "UMLExtensionView", "UMLTransitionView", "UMLControlFlowView", "UMLObjectFlowView"] {
        def(name, Some("UMLGeneralEdgeView"), vec![]);
    }
    // an exception handler and an activity interrupt carry a zig-zag mark in their middle (elements.js:10537-10657)
    def("EdgeNodeView", Some("EdgeParasiticView"), vec![]);
    def("UMLZigZagAdornmentView", Some("EdgeNodeView"), vec![]);
    for name in ["UMLExceptionHandlerView", "UMLActivityInterruptView"] {
        def(name, Some("UMLGeneralEdgeView"), vec![("adornment", n())]);
    }
    def("UMLCommunicationPathView", Some("UMLAssociationView"), vec![]);
    def("UMLLinkView", Some("UMLUndirectedRelationshipView"), vec![]);
    def("UMLConnectorView", Some("UMLUndirectedRelationshipView"), vec![]);
    def("UMLRoleBindingView", Some("UMLGeneralEdgeView"), vec![("roleNameLabel", n())]);
    // flowcharts (flowchart/metamodel.json): every node is an FCModelElement owned by its flowchart,
    // shown by an FC…View with a word-wrapped name label
    def("FCModelElement", Some("ExtensibleModel"), vec![]);
    def("FCFlowchart", Some("FCModelElement"), vec![]);
    def("FCFlowchartDiagram", Some("Diagram"), vec![]);
    def("FCFlow", Some("DirectedRelationship"), vec![]);
    def("FCGeneralNodeView", Some("NodeView"), vec![("nameLabel", n()), ("wordWrap", d(json!(true)))]);
    def("FCGeneralEdgeView", Some("EdgeView"), vec![("nameLabel", n())]);
    def("FCFlowView", Some("FCGeneralEdgeView"), vec![]);
    // C4 models (c4/metamodel.json)
    def("C4Element", Some("ExtensibleModel"), vec![("technology", d(json!(""))), ("description", d(json!("")))]);
    def("C4Model", Some("C4Element"), vec![]);
    def("C4Diagram", Some("Diagram"), vec![]);
    for ty in ["C4Person", "C4SoftwareSystem", "C4Component"] {
        def(ty, Some("C4Element"), vec![]);
    }
    def("C4Container", Some("C4Element"), vec![("kind", n())]);
    def("C4Relationship", Some("DirectedRelationship"), vec![("technology", d(json!(""))), ("description", d(json!("")))]);
    def("C4GeneralNodeView", Some("NodeView"), vec![("nameLabel", n()), ("technologyLabel", n()), ("descriptionLabel", n()), ("wordWrap", d(json!(true)))]);
    def("C4GeneralEdgeView", Some("EdgeView"), vec![("descriptionLabel", n()), ("technologyLabel", n())]);
    for name in ["C4ElementView", "C4PersonView", "C4SoftwareSystemView", "C4ContainerView", "C4ComponentView"] {
        def(name, Some("C4GeneralNodeView"), vec![]);
    }
    def("C4RelationshipView", Some("C4GeneralEdgeView"), vec![]);
    // entity-relationship diagrams (erd/metamodel.json)
    def("ERDElement", Some("ExtensibleModel"), vec![]);
    def("ERDDataModel", Some("ERDElement"), vec![]);
    def("ERDDiagram", Some("Diagram"), vec![]);
    def("ERDColumn", Some("ERDElement"), vec![
        ("type", d(json!(""))), ("length", d(json!(""))), ("primaryKey", d(json!(false))), ("foreignKey", d(json!(false))), ("referenceTo", n()), ("nullable", d(json!(false))), ("unique", d(json!(false))),
    ]);
    def("ERDEntity", Some("ERDElement"), vec![("columns", n())]);
    def("ERDRelationshipEnd", Some("RelationshipEnd"), vec![("cardinality", d(json!("1")))]);
    def("ERDRelationship", Some("UndirectedRelationship"), vec![("identifying", d(json!(true)))]);
    def("ERDColumnView", Some("LabelView"), vec![]);
    def("ERDColumnCompartmentView", Some("NodeView"), vec![]);
    def("ERDEntityView", Some("NodeView"), vec![("suppressColumns", d(json!(false))), ("nameLabel", n()), ("columnCompartment", n())]);
    def("ERDRelationshipView", Some("EdgeView"), vec![("nameLabel", n()), ("tailNameLabel", n()), ("headNameLabel", n())]);
    // data flow diagrams (dfd/metamodel.json): elements with an id, owned by a data flow model
    def("DFDElement", Some("ExtensibleModel"), vec![("id", d(json!("")))]);
    def("DFDDataFlowModel", Some("DFDElement"), vec![]);
    for ty in ["DFDExternalEntity", "DFDProcess", "DFDDataStore"] {
        def(ty, Some("DFDElement"), vec![]);
    }
    def("DFDDataFlow", Some("DirectedRelationship"), vec![]);
    def("DFDDiagram", Some("Diagram"), vec![]);
    def("DFDGeneralNodeView", Some("NodeView"), vec![("nameLabel", n()), ("wordWrap", d(json!(true)))]);
    def("DFDGeneralEdgeView", Some("EdgeView"), vec![("nameLabel", n())]);
    def("DFDExternalEntityView", Some("DFDGeneralNodeView"), vec![]);
    def("DFDProcessView", Some("DFDGeneralNodeView"), vec![("idLabel", n())]);
    def("DFDDataStoreView", Some("DFDGeneralNodeView"), vec![("idLabel", n())]);
    def("DFDDataFlowView", Some("DFDGeneralEdgeView"), vec![]);
    // mind maps (mindmap/metamodel.json)
    def("MMElement", Some("ExtensibleModel"), vec![]);
    def("MMMindmap", Some("MMElement"), vec![]);
    def("MMMindmapDiagram", Some("Diagram"), vec![]);
    def("MMNode", Some("MMElement"), vec![]);
    def("MMEdge", Some("DirectedRelationship"), vec![]);
    def("MMNodeView", Some("NodeView"), vec![("nameLabel", n()), ("wordWrap", d(json!(true)))]);
    def("MMEdgeView", Some("EdgeView"), vec![]);
    // wireframes (wireframe/metamodel.json)
    def("WFElement", Some("ExtensibleModel"), vec![("id", d(json!("")))]);
    def("WFWireframe", Some("WFElement"), vec![]);
    def("WFWireframeDiagram", Some("Diagram"), vec![]);
    def("WFFrame", Some("WFElement"), vec![]);
    def("WFMobileFrame", Some("WFFrame"), vec![("orientation", d(json!("portrait")))]);
    def("WFWebFrame", Some("WFFrame"), vec![]);
    def("WFDesktopFrame", Some("WFFrame"), vec![]);
    for ty in ["WFButton", "WFText", "WFLink", "WFInput", "WFDropdown", "WFPanel", "WFImage", "WFAvatar"] {
        def(ty, Some("WFElement"), vec![]);
    }
    for ty in ["WFRadio", "WFCheckbox", "WFSwitch"] {
        def(ty, Some("WFElement"), vec![("checked", d(json!(false)))]);
    }
    for ty in ["WFTab", "WFTabList"] {
        def(ty, Some("WFElement"), vec![("position", d(json!("top")))]);
    }
    def("WFSeparator", Some("WFElement"), vec![("isVertical", d(json!(false)))]);
    def("WFSlider", Some("WFElement"), vec![("value", d(json!(0)))]);
    def("WFGeneralNodeView", Some("NodeView"), vec![("nameLabel", n()), ("wordWrap", d(json!(true)))]);
    def("WFFrameView", Some("NodeView"), vec![]);
    for ty in ["WFMobileFrameView", "WFWebFrameView", "WFDesktopFrameView"] {
        def(ty, Some("WFFrameView"), vec![]);
    }
    for ty in ["WFButtonView", "WFTextView", "WFRadioView", "WFCheckboxView", "WFTabView", "WFInputView", "WFDropdownView"] {
        def(ty, Some("WFGeneralNodeView"), vec![]);
    }
    def("WFLinkView", Some("WFTextView"), vec![]);
    for ty in ["WFSwitchView", "WFTabListView", "WFPanelView", "WFImageView", "WFSeparatorView", "WFAvatarView", "WFSliderView"] {
        def(ty, Some("NodeView"), vec![]);
    }
    for (_, ty, _) in crate::flowchart::NODES {
        if ty != "FCFlow" {
            def(ty, Some("FCModelElement"), vec![]);
        }
    }
    for name in [
        "FCProcessView", "FCTerminatorView", "FCDecisionView", "FCDelayView", "FCPredefinedProcessView", "FCAlternateProcessView", "FCDataView", "FCDocumentView",
        "FCMultiDocumentView", "FCPreparationView", "FCDisplayView", "FCManualInputView", "FCManualOperationView", "FCCardView", "FCPunchedTapeView", "FCConnectorView",
        "FCOffPageConnectorView", "FCOrView", "FCSummingJunctionView", "FCCollateView", "FCSortView", "FCMergeView", "FCExtractView", "FCStoredDataView", "FCDatabaseView",
        "FCDirectAccessStorageView", "FCInternalStorageView",
    ] {
        def(name, Some("FCGeneralNodeView"), vec![]);
    }
    t
}

/// All fields of a kind in file order (super-kind fields first).
pub fn attrs(ty: &str) -> Vec<(&'static str, Option<Value>)> {
    let t = table();
    let mut chain = vec![];
    let mut cur = Some(ty);
    while let Some(c) = cur {
        match t.get(c) {
            Some(k) => {
                chain.push(k);
                cur = k.sup;
            }
            None => break,
        }
    }
    let mut out = vec![];
    for k in chain.iter().rev() {
        for (n, d) in &k.attrs {
            out.push((*n, d.clone()));
        }
    }
    out
}

pub fn order_of(ty: &str, field: &str) -> Option<usize> {
    attrs(ty).iter().position(|(n, _)| *n == field)
}

pub fn default_of(ty: &str, field: &str) -> Option<Value> {
    attrs(ty).into_iter().find(|(n, _)| *n == field).and_then(|(_, d)| d)
}

pub fn is_kind_of(ty: &str, base: &str) -> bool {
    let t = table();
    let mut cur = Some(ty);
    while let Some(c) = cur {
        if c == base {
            return true;
        }
        cur = t.get(c).and_then(|k| k.sup);
    }
    false
}

pub fn known(ty: &str) -> bool {
    table().contains_key(ty)
}

/// The kind's name with a static lifetime (for keys made from a type name), or "" if unknown.
pub fn static_name(ty: &str) -> &'static str {
    table().get_key_value(ty).map(|(k, _)| *k).unwrap_or("")
}

/// Whether an element of another notation than UML takes its name as typed, without the UML name
/// grammar (flowcharts, data flow diagrams, mind maps: their quick edits only set the name).
pub fn plain_named(ty: &str) -> bool {
    ["FC", "DFD", "MM", "ERD", "C4", "WF"].iter().any(|p| ty.starts_with(p))
}
