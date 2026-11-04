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
    def("UMLInterface", Some("UMLClassifier"), vec![]);
    def("UMLSignal", Some("UMLClassifier"), vec![]);
    def("UMLDataType", Some("UMLClassifier"), vec![]);
    def("UMLPrimitiveType", Some("UMLDataType"), vec![]);
    def("UMLEnumerationLiteral", Some("UMLModelElement"), vec![]);
    def("UMLTemplateParameter", Some("UMLModelElement"), vec![("parameterType", d(json!(""))), ("defaultValue", d(json!("")))]);
    def("UMLEnumeration", Some("UMLDataType"), vec![("literals", n())]);
    def("UMLDependency", Some("UMLDirectedRelationship"), vec![("mapping", d(json!("")))]);
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
    def("UMLCommunicationPath", Some("UMLAssociation"), vec![]);
    def("UMLExtensionPoint", Some("UMLModelElement"), vec![("location", d(json!("")))]);
    def("UMLUseCase", Some("UMLClassifier"), vec![("extensionPoints", n())]);
    def("UMLActor", Some("UMLClassifier"), vec![]);
    def("UMLInclude", Some("UMLDirectedRelationship"), vec![]);
    // an association class: the class and the association it details (estimated field names)
    def("UMLAssociationClassLink", Some("UMLModelElement"), vec![("classSide", n()), ("associationSide", n())]);
    def("UMLExtend", Some("UMLDirectedRelationship"), vec![("condition", d(json!(""))), ("extensionLocations", n())]);
    def("UMLUseCaseSubject", Some("UMLSubsystem"), vec![("represent", n())]);
    def("UMLDiagram", Some("Diagram"), vec![]);
    def("UMLClassDiagram", Some("UMLDiagram"), vec![]);
    def("UMLUseCaseDiagram", Some("UMLDiagram"), vec![]);
    def("UMLPackageDiagram", Some("UMLDiagram"), vec![]);
    def("UMLComponentDiagram", Some("UMLDiagram"), vec![]);
    def("UMLDeploymentDiagram", Some("UMLDiagram"), vec![]);

    // UML views
    def("UMLCompartmentView", Some("NodeView"), vec![]);
    def("UMLNameCompartmentView", Some("UMLCompartmentView"), vec![("stereotypeLabel", n()), ("nameLabel", n()), ("namespaceLabel", n()), ("propertyLabel", n())]);
    def("UMLListCompartmentView", Some("UMLCompartmentView"), vec![]);
    for name in ["UMLAttributeView", "UMLOperationView", "UMLReceptionView", "UMLEnumerationLiteralView", "UMLExtensionPointView"] {
        def(name, Some("LabelView"), vec![]);
    }
    for name in [
        "UMLAttributeCompartmentView", "UMLOperationCompartmentView", "UMLReceptionCompartmentView", "UMLTemplateParameterCompartmentView",
        "UMLEnumerationLiteralCompartmentView", "UMLExtensionPointCompartmentView", "UMLQualifierCompartmentView",
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
    for name in ["UMLClassView", "UMLInterfaceView", "UMLDataTypeView", "UMLPrimitiveTypeView", "UMLSignalView", "UMLActorView", "UMLComponentView", "UMLArtifactView", "UMLNodeView"] {
        def(name, Some("UMLClassifierView"), vec![]);
    }
    def("UMLEnumerationView", Some("UMLClassifierView"), vec![("suppressLiterals", d(json!(false))), ("enumerationLiteralCompartment", n())]);
    def("UMLPackageView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLSubsystemView", Some("UMLPackageView"), vec![]);
    def("UMLUseCaseView", Some("UMLClassifierView"), vec![("extensionPointCompartment", n())]);
    def("UMLUseCaseSubjectView", Some("UMLGeneralNodeView"), vec![]);
    def("UMLCustomTextView", Some("NodeView"), vec![("text", d(json!(""))), ("wordWrap", d(json!(true))), ("horzAlign", d(json!(0))), ("vertAlign", d(json!(3)))]);
    def("UMLCustomNoteView", Some("UMLCustomTextView"), vec![]);
    def("UMLNoteView", Some("UMLCustomNoteView"), vec![]);
    def("UMLTextView", Some("UMLCustomTextView"), vec![]);
    def("UMLTextBoxView", Some("UMLCustomTextView"), vec![]);
    def("UMLNoteLinkView", Some("EdgeView"), vec![]);
    def("UMLContainmentView", Some("EdgeView"), vec![]);
    def("UMLAssociationClassLinkView", Some("EdgeView"), vec![]);
    def("UMLGeneralEdgeView", Some("EdgeView"), vec![
        ("stereotypeDisplay", d(json!("label"))), ("showVisibility", d(json!(false))), ("showProperty", d(json!(true))),
        ("nameLabel", n()), ("stereotypeLabel", n()), ("propertyLabel", n()),
    ]);
    def("UMLUndirectedRelationshipView", Some("UMLGeneralEdgeView"), vec![
        ("showMultiplicity", d(json!(true))), ("showType", d(json!(true))), ("showEndOrder", d(json!(0))), ("tailRoleNameLabel", n()), ("tailPropertyLabel", n()),
        ("tailMultiplicityLabel", n()), ("headRoleNameLabel", n()), ("headPropertyLabel", n()), ("headMultiplicityLabel", n()),
    ]);
    def("UMLAssociationView", Some("UMLUndirectedRelationshipView"), vec![("tailQualifiersCompartment", n()), ("headQualifiersCompartment", n())]);
    for name in ["UMLGeneralizationView", "UMLDependencyView", "UMLRealizationView", "UMLInterfaceRealizationView", "UMLIncludeView", "UMLExtendView", "UMLComponentRealizationView", "UMLDeploymentView"] {
        def(name, Some("UMLGeneralEdgeView"), vec![]);
    }
    def("UMLCommunicationPathView", Some("UMLAssociationView"), vec![]);
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
