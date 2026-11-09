//! File > New From Template: the UML templates (extensions/essential/uml/templates). Each starts a new
//! project with these models and empty diagrams, plus the UML Standard Profile.
//!
//! Model > Apply Profile > UML Standard Profile (v2) adds that profile to the open project
//! (uml-commands.js:81-100).

use crate::model::{Eid, Repo};
use serde_json::json;

/// A template: its menu label and its models, each with (diagram kind, diagram name).
pub struct Template {
    pub label: &'static str,
    pub models: &'static [(&'static str, &'static str, &'static str)],
}

/// The UML templates, in menu order. Diagram names are the original's where the docs give them,
/// otherwise the model's name (estimated).
pub const TEMPLATES: [Template; 11] = [
    Template { label: "UML Minimal", models: &[("Model", "UMLClassDiagram", "Main")] },
    Template {
        label: "UML Conventional",
        models: &[
            ("Use Case Model", "UMLUseCaseDiagram", "Use Cases"),
            ("Analysis Model", "UMLClassDiagram", "Analysis"),
            ("Design Model", "UMLClassDiagram", "Classes"),
            ("Implementation Model", "UMLComponentDiagram", "Components"),
            ("Deployment Model", "UMLDeploymentDiagram", "Deployment"),
        ],
    },
    Template {
        label: "4+1 View Model",
        models: &[
            ("Scenarios", "UMLUseCaseDiagram", "Scenarios"),
            ("Logical View", "UMLClassDiagram", "Logical View"),
            ("Development View", "UMLComponentDiagram", "Development View"),
            ("Process View", "UMLComponentDiagram", "Process View"),
            ("Physical View", "UMLDeploymentDiagram", "Physical View"),
        ],
    },
    Template {
        label: "Rational",
        models: &[
            ("Use Case View", "UMLUseCaseDiagram", "Use Case View"),
            ("Logical View", "UMLClassDiagram", "Logical View"),
            ("Component View", "UMLComponentDiagram", "Component View"),
            ("Deployment View", "UMLDeploymentDiagram", "Deployment View"),
        ],
    },
    // the ERD template: a data model "Data Model" with the diagram "Main", without the UML profile
    // (erd/templates/DataModel.mdj, File › New From Template › Data Model in erd/menus/erd.json)
    Template { label: "Data Model", models: &[("Data Model", "ERDDiagram", "Main")] },
    // the C4 template: a C4 model with its "System Context Diagram" (c4/templates/C4Model.mdj)
    Template { label: "C4 Model", models: &[("C4Model", "C4Diagram", "System Context Diagram")] },
    // the wireframe template: a wireframe with its "Wireframe Diagram" showing a phone frame
    // (wireframe/templates/WireframeModel.mdj)
    Template { label: "Wireframe Model", models: &[("Wireframe", "WFWireframeDiagram", "Wireframe Diagram")] },
    // the BPMN template: a process "Business Process" with its "BPMN Diagram"
    // (bpmn/templates/BusinessProcessModel.mdj)
    Template { label: "BPMN Business Process Modeling", models: &[("Business Process", "BPMNDiagram", "BPMN Diagram")] },
    // the cloud templates: a model with its diagram "Main" (aws/templates/AWSArchitecture.mdj)
    Template { label: "AWS Architecture", models: &[("AWS Architecture", "AWSDiagram", "Main")] },
    Template { label: "Azure Architecture", models: &[("Azure Architecture", "AzureDiagram", "Main")] },
    Template { label: "GCP Architecture", models: &[("GCP Architecture", "GCPDiagram", "Main")] },
];

/// The UML Standard Profile in its order (uml/profiles/UMLStandardProfile.mfj): the metaclasses, each
/// followed by the stereotypes that come after it, with the metaclasses each stereotype extends.
/// UMLCraft leaves out the profile's stereotype icons (the original's artwork); its diagrams are laid
/// out by UMLCraft.
pub const STANDARD_PROFILE_ELEMENTS: &[(&str, &[&str])] = &[
    ("UMLClassifier", &[]), ("Realization", &["UMLClassifier"]), ("Specification", &["UMLClassifier"]), ("entity", &["UMLClass"]),
    ("UMLClass", &[]), ("UMLArtifact", &[]), ("File", &["UMLArtifact"]), ("Document", &["UMLArtifact"]), ("Executable", &["UMLArtifact"]),
    ("Library", &["UMLArtifact"]), ("Script", &["UMLArtifact"]), ("Source", &["UMLArtifact"]),
    ("UMLBehavioralFeature", &[]), ("Destroy", &["UMLBehavioralFeature"]), ("Auxiliary", &["UMLClass"]), ("Focus", &["UMLClass"]), ("Metaclass", &["UMLClass"]),
    ("ImplementationClass", &["UMLClass"]), ("Type", &["UMLClass"]), ("Utility", &["UMLClass"]),
    ("UMLComponent", &[]), ("Entity", &["UMLComponent"]), ("Implementation", &["UMLComponent"]), ("Process", &["UMLComponent"]), ("Service", &["UMLComponent"]),
    ("Subsystem", &["UMLComponent"]),
    ("UMLDependency", &[]), ("Create", &["UMLBehavioralFeature", "UMLDependency"]), ("Call", &["UMLDependency"]), ("Instantiate", &["UMLDependency"]),
    ("Responsibility", &["UMLDependency"]), ("Send", &["UMLDependency"]),
    ("UMLAbstraction", &[]), ("Derive", &["UMLAbstraction"]), ("Refine", &["UMLAbstraction"]), ("Trace", &["UMLAbstraction"]),
    ("UMLPackage", &[]), ("Framework", &["UMLPackage"]), ("ModelLibrary", &["UMLPackage"]),
    ("UMLModel", &[]), ("BuildComponent", &["UMLComponent"]), ("Metamodel", &["UMLModel"]), ("SystemModel", &["UMLModel"]), ("control", &["UMLClass"]), ("boundary", &["UMLClass"]),
];
/// The stereotypes that generalize File.
const FILE_KINDS: [&str; 5] = ["Document", "Executable", "Library", "Script", "Source"];
pub const STANDARD_PROFILE: &str = "UMLStandardProfile";

/// Add the UML Standard Profile under the project, unless a profile of that name is there already:
/// its metaclasses, and its stereotypes with their generalization to File and their extensions of
/// the metaclasses, each owned by its stereotype.
pub fn add_standard_profile(r: &mut Repo) -> Option<Eid> {
    let root = r.root;
    if r.all().any(|e| r.ty(e) == "UMLProfile" && r.name(e) == STANDARD_PROFILE) {
        return None;
    }
    let p = r.create("UMLProfile", &[("name", json!(STANDARD_PROFILE))], Some((root, "ownedElements")));
    let mut file = None;
    let mut metas: Vec<(&str, Eid)> = vec![];
    for (name, extends) in STANDARD_PROFILE_ELEMENTS {
        if name.starts_with("UML") {
            let mc = r.create("UMLMetaClass", &[("name", json!(name))], Some((p, "ownedElements")));
            metas.push((name, mc));
            continue;
        }
        let st = r.create("UMLStereotype", &[("name", json!(name))], Some((p, "ownedElements")));
        crate::app::add_stereotype_icon(r, st);
        if *name == "File" {
            file = Some(st);
        }
        if let (true, Some(f)) = (FILE_KINDS.contains(name), file) {
            let (s, t) = (r.ref_to(st), r.ref_to(f));
            r.create("UMLGeneralization", &[("source", s), ("target", t)], Some((st, "ownedElements")));
        }
        for m in *extends {
            if let Some((_, mc)) = metas.iter().find(|(n, _)| n == m) {
                let (s, t) = (r.ref_to(st), r.ref_to(*mc));
                r.create("UMLExtension", &[("source", s), ("target", t)], Some((st, "ownedElements")));
            }
        }
    }
    add_profile_diagrams(r, p);
    Some(p)
}

/// The profile's three diagrams (StandardProfileL2, Robustness Stereotypes, StandardProfileL3, in
/// uml/profiles/UMLStandardProfile.mfj and the templates), with the views of the elements each shows.
/// The original's hand placement is not copied: each metaclass starts a row, with the stereotypes
/// that extend it to its right.
const PROFILE_DIAGRAMS: [(&str, &[&str]); 3] = [
    (
        "StandardProfileL2",
        &[
            "UMLClassifier", "Realization", "Specification", "UMLClass", "Auxiliary", "Focus", "Metaclass", "ImplementationClass", "Type", "Utility", "UMLArtifact", "File", "Document", "Executable",
            "Library", "Script", "Source", "UMLBehavioralFeature", "Destroy", "Create", "UMLComponent", "Entity", "Implementation", "Process", "Service", "Subsystem", "UMLDependency", "Call",
            "Instantiate", "Responsibility", "Send", "UMLAbstraction", "Derive", "Refine", "Trace", "UMLPackage", "Framework", "ModelLibrary",
        ],
    ),
    ("Robustness Stereotypes", &["UMLClass", "entity", "control", "boundary"]),
    ("StandardProfileL3", &["UMLModel", "Metamodel", "SystemModel", "UMLComponent", "BuildComponent"]),
];

fn add_profile_diagrams(r: &mut Repo, profile: Eid) {
    use crate::views::{create_edge_view, create_node_view, R};
    let elements: Vec<(String, Eid)> = r.children(profile).into_iter().map(|(_, e)| (r.name(e), e)).collect();
    let find = |n: &str| elements.iter().find(|(x, _)| x == n).map(|(_, e)| *e);
    for (name, shown) in PROFILE_DIAGRAMS {
        let d = r.create("UMLProfileDiagram", &[("name", json!(name))], Some((profile, "ownedElements")));
        let (mut row, mut col) = (-1.0f64, 0.0f64);
        let mut views: Vec<(Eid, Eid)> = vec![];
        for n in shown {
            let Some(m) = find(n) else { continue };
            if n.starts_with("UML") {
                row += 1.0;
                col = 0.0;
            } else {
                col += 1.0;
            }
            let at = R { x: 20.0 + col * 150.0, y: 20.0 + row * 150.0, w: 1.0, h: 1.0 };
            if let Some(v) = create_node_view(r, d, m, at) {
                views.push((m, v));
            }
        }
        // the extensions and generalizations between the elements shown
        for (m, v) in views.clone() {
            for (_, rel) in r.children(m) {
                if !matches!(r.ty(rel), "UMLExtension" | "UMLGeneralization") {
                    continue;
                }
                let Some(t) = r.refd(rel, "target") else { continue };
                if let Some((_, tv)) = views.iter().find(|(x, _)| *x == t) {
                    create_edge_view(r, d, Some(rel), v, *tv);
                }
            }
        }
    }
}

/// `add_standard_profile` on an App's project (for `App::change`).
pub fn add_standard_profile_app(a: &mut crate::app::App) -> Option<Eid> {
    add_standard_profile(&mut a.repo)
}

/// A new project from a template; its first diagram opens by default.
pub fn project(t: &Template) -> Repo {
    let mut r = Repo::blank();
    let root = r.root;
    for (_, m) in r.children(root) {
        r.detach(m);
    }
    let erd = t.models.iter().all(|m| matches!(m.1, "ERDDiagram" | "C4Diagram" | "WFWireframeDiagram" | "BPMNDiagram") || crate::cloud::is_diagram(m.1));
    for (i, (model, kind, diagram)) in t.models.iter().enumerate() {
        let ty = match *kind {
            "ERDDiagram" => "ERDDataModel",
            "C4Diagram" => "C4Model",
            "WFWireframeDiagram" => "WFWireframe",
            "BPMNDiagram" => "BPMNProcess",
            k if crate::cloud::is_diagram(k) => crate::cloud::model_of(k).unwrap_or("UMLModel"),
            _ => "UMLModel",
        };
        let m = r.create(ty, &[("name", json!(model))], Some((root, "ownedElements")));
        let d = r.create(kind, &[("name", json!(diagram))], Some((m, "ownedElements")));
        if i == 0 {
            r.set(d, "defaultDiagram", json!(true));
        }
        // a phone frame, 328 × 520 at (80, 72)
        if *kind == "WFWireframeDiagram" {
            let f = r.create("WFMobileFrame", &[("name", json!("MobileFrame1"))], Some((m, "ownedElements")));
            crate::wireframe::create_view(&mut r, d, f, crate::views::R { x: 80.0, y: 72.0, w: 328.0, h: 520.0 });
        }
    }
    // the 4+1 View Model and Rational templates have no profile (uml/templates/4p1ViewModel.mdj,
    // Rational.mdj)
    if !erd && matches!(t.label, "UML Minimal" | "UML Conventional") {
        add_standard_profile(&mut r);
    }
    r
}
