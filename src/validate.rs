//! Model validation: StarUML's UML rules for the kinds UMLCraft models (src/core/validator.js,
//! extensions/essential/uml/rules.js), and the Validation Results panel (ui/validation-panel.js).

use crate::app::App;
use crate::model::{Eid, Repo};
use std::collections::HashSet;

/// A failed rule: the element, the rule id and its message.
#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub elem: Eid,
    pub rule: &'static str,
    pub message: &'static str,
}

fn kind(r: &Repo, e: Eid, base: &str) -> bool {
    r.ty(e) == base || r.kind_of(e, base)
}

/// The parents a classifier generalizes, in order.
fn parents(r: &Repo, c: Eid) -> Vec<Eid> {
    r.all().filter(|g| r.ty(*g) == "UMLGeneralization" && r.refd(*g, "source") == Some(c)).filter_map(|g| r.refd(g, "target")).collect()
}

/// Every ancestor of a classifier, without repeats (and stopping on cycles).
fn ancestors(r: &Repo, c: Eid) -> Vec<Eid> {
    let mut out = vec![];
    let mut todo = parents(r, c);
    while let Some(p) = todo.pop() {
        if out.contains(&p) {
            continue;
        }
        out.push(p);
        todo.extend(parents(r, p));
    }
    out
}

/// The association ends opposite to a classifier: (end, its association).
fn opposite_ends(r: &Repo, c: Eid) -> Vec<Eid> {
    let mut out = vec![];
    for a in r.all().filter(|a| r.ty(*a) == "UMLAssociation") {
        let (e1, e2) = (r.child(a, "end1"), r.child(a, "end2"));
        if let (Some(e1), Some(e2)) = (e1, e2) {
            if r.refd(e1, "reference") == Some(c) {
                out.push(e2);
            }
            if r.refd(e2, "reference") == Some(c) {
                out.push(e1);
            }
        }
    }
    out
}

fn signature(r: &Repo, o: Eid) -> String {
    // stereotype and visibility hidden, type and signature shown (rules.js:83)
    let t = crate::views::operation_text(r, o, crate::views::Show { vis: false, ..crate::views::Show::ALL });
    let st = crate::views::stereotype_text(r, o);
    t.strip_prefix(&st).unwrap_or(&t).trim().to_string()
}

/// Run every rule over the repository (validator.js:56).
pub fn validate(r: &Repo) -> Vec<Problem> {
    let mut out = vec![];
    let mut fail = |elem: Eid, rule: &'static str, message: &'static str| out.push(Problem { elem, rule, message });
    let all: Vec<Eid> = r.all().collect();
    for &e in &all {
        let t = r.ty(e);
        // UML001 Name expected (rules.js:27)
        if kind(r, e, "UMLModelElement") {
            let exempt = ["UMLRegion", "UMLPseudostate", "UMLFinalState", "UMLControlNode", "UMLEndpoint", "UMLGate", "UMLAction", "UMLObjectNode", "UMLImage", "UMLLifeline", "UMLInstance", "UMLConnector", "UMLAssociationClassLink"]
                .iter()
                .any(|x| kind(r, e, x))
                || (t == "UMLParameter" && r.str(e, "direction") == "return")
                || (kind(r, e, "UMLStructuralFeature") && r.has(e, "type"));
            if !exempt && r.name(e).trim().is_empty() {
                fail(e, "UML001", "Name expected");
            }
        }
        // UML002 Name is already defined (rules.js:45)
        if kind(r, e, "UMLModelElement") && t != "UMLOperation" && t != "UMLActivityEdgeConnector" && !r.name(e).is_empty() {
            if let (Some(p), Some(f)) = (r.parent(e), r.field_of(e)) {
                let name = r.name(e);
                if r.list(p, &f).into_iter().any(|o| o != e && r.name(o) == name && r.ty(o) != "UMLOperation") {
                    fail(e, "UML002", "Name is already defined");
                }
            }
        }
        if kind(r, e, "UMLClassifier") {
            let anc = ancestors(r, e);
            // UML003 Conflict with inherited attributes (rules.js:67)
            let inherited: HashSet<String> = anc.iter().flat_map(|a| r.list(*a, "attributes")).map(|x| r.name(x)).collect();
            for at in r.list(e, "attributes") {
                if !r.name(at).is_empty() && inherited.contains(&r.name(at)) {
                    fail(at, "UML003", "Conflict with inherited attributes");
                }
            }
            // UML004 Signature conflict (rules.js:83)
            let ops = r.list(e, "operations");
            for (i, o) in ops.iter().enumerate() {
                if ops[..i].iter().any(|p| signature(r, *p) == signature(r, *o)) {
                    fail(*o, "UML004", "Signature conflict");
                }
            }
            // UML006 Final and leaf element must not have child (rules.js:112)
            if (r.bool(e, "isFinalSpecialization") || r.bool(e, "isLeaf")) && all.iter().any(|g| r.ty(*g) == "UMLGeneralization" && r.refd(*g, "target") == Some(e)) {
                fail(e, "UML006", "Final and leaf element must not have child");
            }
            // UML007 Duplicated generalizations (rules.js:127)
            let ps = parents(r, e);
            if ps.iter().enumerate().any(|(i, p)| ps[..i].contains(p)) {
                fail(e, "UML007", "Duplicated generalizations");
            }
            // UML008 Circular generalizations (rules.js:139)
            if anc.contains(&e) {
                fail(e, "UML008", "Circular generalizations");
            }
            // UML009 Duplicated realizations (rules.js:151)
            let reals: Vec<Eid> = all.iter().filter(|x| r.ty(**x) == "UMLInterfaceRealization" && r.refd(**x, "source") == Some(e)).filter_map(|x| r.refd(*x, "target")).collect();
            if reals.iter().enumerate().any(|(i, p)| reals[..i].contains(p)) {
                fail(e, "UML009", "Duplicated realizations");
            }
            // UML010 Duplicated role names of associated classifiers (rules.js:163)
            let names: Vec<String> = opposite_ends(r, e).into_iter().filter(|x| r.str(*x, "navigable") != "notNavigable" && !r.name(*x).is_empty()).map(|x| r.name(x)).collect();
            if names.iter().enumerate().any(|(i, n)| names[..i].contains(n)) {
                fail(e, "UML010", "Duplicated role names of associated classifiers");
            }
        }
        // UML011 Interface members must be public (rules.js:182)
        if t == "UMLInterface" {
            for m in r.list(e, "attributes").into_iter().chain(r.list(e, "operations")) {
                if r.str(m, "visibility") != "public" {
                    fail(m, "UML011", "Interface members must be public");
                }
            }
        }
        // UML012 Aggregation must be one in an association (rules.js:196)
        if t == "UMLAssociation" {
            let agg = |k: &str| r.child(e, k).map(|x| r.str(x, "aggregation")).map(|a| a == "shared" || a == "composite").unwrap_or(false);
            if agg("end1") && agg("end2") {
                fail(e, "UML012", "Aggregation must be one in an association");
            }
        }
        // UML013-015: an instance's classifier has the matching kind (rules.js:210-232)
        for (inst, ty, rule, msg) in [
            ("UMLArtifactInstance", "UMLArtifact", "UML013", "Type of artifact instance must be an artifact"),
            ("UMLComponentInstance", "UMLComponent", "UML014", "Type of component instance must be a component"),
            ("UMLNodeInstance", "UMLNode", "UML015", "Type of node instance must be a node"),
        ] {
            if t == inst {
                if let Some(c) = r.refd(e, "classifier") {
                    if !kind(r, c, ty) {
                        fail(e, rule, msg);
                    }
                }
            }
        }
        if t == "UMLStereotype" {
            // UML017: a stereotype only generalizes stereotypes (rules.js:254)
            if parents(r, e).iter().any(|p| r.ty(*p) != "UMLStereotype") {
                fail(e, "UML017", "Stereotype can only generalize stereotypes");
            }
            // UML018: a stereotype is owned by a profile (rules.js:266)
            if r.parent(e).map(|p| r.ty(p) != "UMLProfile").unwrap_or(true) {
                fail(e, "UML018", "Stereotype must be owned by a profile");
            }
        }
        // UML019: an actor associates only with use cases, components and classes (rules.js:281)
        if t == "UMLActor" {
            let ok = |x: Eid| ["UMLUseCase", "UMLComponent", "UMLClass"].iter().any(|k| kind(r, x, k)) && r.ty(x) != "UMLStereotype";
            if opposite_ends(r, e).into_iter().filter_map(|x| r.refd(x, "reference")).any(|x| !ok(x)) {
                fail(e, "UML019", "Actor can only be associated with use cases, components and classes");
            }
        }
        // UML020: a use case does not include itself, even transitively (rules.js:297)
        if t == "UMLUseCase" {
            let includes = |u: Eid| -> Vec<Eid> { all.iter().filter(|x| r.ty(**x) == "UMLInclude" && r.refd(**x, "source") == Some(u)).filter_map(|x| r.refd(*x, "target")).collect() };
            let mut seen = vec![];
            let mut todo = includes(e);
            while let Some(u) = todo.pop() {
                if seen.contains(&u) {
                    continue;
                }
                seen.push(u);
                todo.extend(includes(u));
            }
            if seen.contains(&e) {
                fail(e, "UML020", "Use case cannot include itself");
            }
        }
    }
    out
}

impl App {
    /// Validate now, when the preference is on (after open and save, validation-panel.js:43-47).
    pub fn auto_validate(&mut self) {
        if self.pref("validation.validateModel").as_bool().unwrap_or(true) {
            self.problems = validate(&self.repo);
            self.validated = true;
        }
    }

    /// The Validation Results panel: "Validation Results — N errors." or "— No errors.", one row per
    /// problem with its message, (rule), the element's icon, name and [Type]; a click reveals the element.
    pub fn validation_ui(&mut self, ui: &mut egui::Ui) {
        let probs: Vec<Problem> = self.problems.iter().filter(|p| self.repo.alive(p.elem)).cloned().collect();
        let mut close = false;
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let title = if probs.is_empty() { "Validation Results — No errors.".to_string() } else { format!("Validation Results — {} errors.", probs.len()) };
            ui.label(egui::RichText::new(title).size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("×").clicked() {
                    close = true;
                }
            });
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for p in probs {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::from_rgb(0xe5, 0x48, 0x4d), "⊗");
                    let text = format!("{} ({})", p.message, p.rule);
                    let r = ui.selectable_label(self.focus == Some(p.elem), text);
                    let (ir, _) = ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                    crate::icons::draw(ui.painter(), ir, crate::icons::key_for(&self.repo, p.elem));
                    ui.label(format!("{} [{}]", self.label_of(p.elem), self.repo.ty(p.elem)));
                    if r.clicked() {
                        self.focus = Some(p.elem);
                        self.reveal(p.elem);
                    }
                });
            }
        });
        if close {
            self.show_validation = false;
        }
    }
}
