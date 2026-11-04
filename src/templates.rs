//! File > New From Template: the UML templates (extensions/essential/uml/templates). Each starts a new
//! project with these models and empty diagrams. The original's templates also import the UML
//! Standard Profile; UMLCraft's do not (it has no profile import yet).

use crate::model::Repo;
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
    r
}
