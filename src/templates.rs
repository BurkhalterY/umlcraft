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
pub const TEMPLATES: [Template; 4] = [
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
];

/// The UML Standard Profile in its order (uml/profiles/UMLStandardProfile.mfj): the metaclasses, each
/// followed by the stereotypes that come after it, with the metaclasses each stereotype extends.
/// UMLCraft leaves out the profile's stereotype icons (the original's artwork) and its diagrams.
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
/// The stereotypes of the profile, in its order.
pub fn standard_stereotypes() -> Vec<&'static str> {
    STANDARD_PROFILE_ELEMENTS.iter().filter(|(n, _)| !n.starts_with("UML")).map(|(n, _)| *n).collect()
}
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
    Some(p)
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
    for (i, (model, kind, diagram)) in t.models.iter().enumerate() {
        let m = r.create("UMLModel", &[("name", json!(model))], Some((root, "ownedElements")));
        let d = r.create(kind, &[("name", json!(diagram))], Some((m, "ownedElements")));
        if i == 0 {
            r.set(d, "defaultDiagram", json!(true));
        }
    }
    add_standard_profile(&mut r);
    r
}
