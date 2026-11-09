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
        // core rules (validator.js core rules, main-process/application.js:316-320)
        if (t == "Project" || t == "Tag") && r.name(e).trim().is_empty() {
            fail(e, "CORE001", "Name expected.");
        }
        if kind(r, e, "DirectedRelationship") && (r.refd(e, "source").is_none() || r.refd(e, "target").is_none()) {
            fail(e, "CORE002", "[Critical] DirectedRelationship should have source and target.");
        }
        if kind(r, e, "UndirectedRelationship") && [r.child(e, "end1"), r.child(e, "end2")].iter().any(|x| x.and_then(|end| r.refd(end, "reference")).is_none()) {
            fail(e, "CORE003", "[Critical] UndirectedRelationship should have two ends.");
        }
        // UML001 Name expected (rules.js:27)
        if kind(r, e, "UMLModelElement") {
            let exempt = ["UMLRegion", "UMLPseudostate", "UMLFinalState", "UMLControlNode", "UMLEndpoint", "UMLGate", "UMLAction", "UMLObjectNode", "UMLImage", "UMLLifeline", "UMLInstance", "UMLConnector", "UMLAssociationClassLink", "UMLLinkObjectLink"]
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
        // UML016: a metaclass names a type of the metamodel (rules.js:243); UMLCraft knows its own
        // element kinds plus StarUML's behaviour and interaction kinds it reads but does not edit
        if t == "UMLMetaClass" {
            let n = r.name(e);
            let extra = [
                "Element", "Model", "ExtensibleModel", "Relationship", "Behavior", "OpaqueBehavior", "Activity", "StateMachine", "Interaction", "Collaboration", "CollaborationUse",
                "Action", "State", "Pseudostate", "FinalState", "Region", "Transition", "Event", "Lifeline", "Message", "CombinedFragment", "InteractionOperand", "Port", "Connector",
                "Instance", "ActivityNode", "ActivityEdge", "ControlFlow", "ObjectFlow", "ObjectNode", "Pin", "Slot", "Constraint", "Diagram", "View",
            ];
            let ok = crate::meta::known(&n) || crate::meta::known(&format!("UML{n}")) || extra.contains(&n.as_str());
            if !n.is_empty() && !ok {
                fail(e, "UML016", "Metaclass must name a type of the metamodel");
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
    // ERD001 and ERD002 (erd/rules.js:26-57)
    for &e in &all {
        if r.ty(e) != "ERDColumn" {
            continue;
        }
        if r.bool(e, "primaryKey") && r.bool(e, "nullable") {
            fail(e, "ERD001", "Primary key cannot be nullable.");
        }
        if let Some(target) = r.refd(e, "referenceTo").filter(|_| r.bool(e, "foreignKey")) {
            // an end of a relationship of this column's entity, on the other side, must reach the
            // entity that owns the referenced column
            let (Some(entity), Some(other)) = (r.parent(e), r.parent(target)) else { continue };
            let related = all.iter().filter(|x| r.ty(**x) == "ERDRelationship").any(|rel| {
                let ends: Vec<Option<Eid>> = ["end1", "end2"].iter().map(|k| r.child(*rel, k).and_then(|end| r.refd(end, "reference"))).collect();
                (ends[0] == Some(entity) && ends[1] == Some(other)) || (ends[1] == Some(entity) && ends[0] == Some(other))
            });
            if !related {
                fail(e, "ERD002", "No relationship with the foreign key reference.");
            }
        }
    }
    out.extend(behaviour_rules(r, &all));
    out
}

/// UML021-UML057: state machine and activity rules (rules.js:309-775). UMLCraft does not edit these
/// diagrams, but checks them in projects made with StarUML. The messages are UMLCraft's own wording.
fn behaviour_rules(r: &Repo, all: &[Eid]) -> Vec<Problem> {
    let mut out = vec![];
    let mut fail = |elem: Eid, rule: &'static str, message: &'static str| out.push(Problem { elem, rule, message });
    let len = |e: Eid, k: &str| r.list(e, k).len();
    let has_guard = |t: Eid| !r.str(t, "guard").trim().is_empty();
    // transitions and activity edges as (edge, source, target)
    let edges = |ty: &[&str]| -> Vec<(Eid, Eid, Eid)> {
        all.iter().filter(|x| ty.contains(&r.ty(**x))).filter_map(|x| Some((*x, r.refd(*x, "source")?, r.refd(*x, "target")?))).collect()
    };
    let trans = edges(&["UMLTransition"]);
    let flows = edges(&["UMLControlFlow", "UMLObjectFlow"]);
    let outgoing = |es: &[(Eid, Eid, Eid)], v: Eid| -> Vec<Eid> { es.iter().filter(|x| x.1 == v).map(|x| x.0).collect() };
    let incoming = |es: &[(Eid, Eid, Eid)], v: Eid| -> Vec<Eid> { es.iter().filter(|x| x.2 == v).map(|x| x.0).collect() };
    let pkind = |v: Eid| if r.ty(v) == "UMLPseudostate" { r.str(v, "kind") } else { String::new() };
    let is_state = |v: Eid| r.ty(v) == "UMLState";
    for &e in all {
        let t = r.ty(e);
        if t == "UMLPseudostate" {
            let (ins, outs) = (incoming(&trans, e), outgoing(&trans, e));
            match r.str(e, "kind").as_str() {
                "initial" => {
                    if outs.len() > 1 {
                        fail(e, "UML021", "Initial pseudostate can have at most one outgoing transition");
                    }
                    if outs.iter().any(|x| len(*x, "triggers") > 0 || has_guard(*x)) {
                        fail(e, "UML022", "Outgoing transition of an initial pseudostate cannot have a trigger or a guard");
                    }
                }
                "deepHistory" | "shallowHistory" if outs.len() > 1 => fail(e, "UML023", "History pseudostate can have at most one outgoing transition"),
                "join" if ins.len() < 2 || outs.len() != 1 => fail(e, "UML024", "Join pseudostate must have at least two incoming transitions and one outgoing transition"),
                "fork" if outs.len() < 2 || ins.len() != 1 => fail(e, "UML025", "Fork pseudostate must have at least two outgoing transitions and one incoming transition"),
                "junction" if ins.is_empty() || outs.is_empty() => fail(e, "UML026", "Junction pseudostate must have at least one incoming and one outgoing transition"),
                "choice" if ins.is_empty() || outs.is_empty() => fail(e, "UML027", "Choice pseudostate must have at least one incoming and one outgoing transition"),
                _ => {}
            }
        }
        if t == "UMLState" {
            if len(e, "connections") > 0 && r.refd(e, "submachine").is_none() {
                fail(e, "UML028", "Only submachine states can have connection point references");
            }
            if r.refd(e, "submachine").is_some() && len(e, "regions") > 0 {
                fail(e, "UML029", "State cannot have both a submachine and regions");
            }
        }
        if t == "UMLRegion" {
            let count = |k: &str| r.list(e, "vertices").into_iter().filter(|v| pkind(*v) == k).count();
            if count("initial") > 1 {
                fail(e, "UML030", "Region can have at most one initial vertex");
            }
            if count("deepHistory") > 1 {
                fail(e, "UML031", "Region can have at most one deep history vertex");
            }
            if count("shallowHistory") > 1 {
                fail(e, "UML032", "Region can have at most one shallow history vertex");
            }
        }
        if t == "UMLFinalState" {
            for (cond, rule, msg) in [
                (!outgoing(&trans, e).is_empty(), "UML033", "Final state cannot have outgoing transitions"),
                (len(e, "regions") > 0, "UML034", "Final state cannot have regions"),
                (r.refd(e, "submachine").is_some(), "UML035", "Final state cannot reference a submachine"),
                (len(e, "entryActivities") > 0, "UML036", "Final state cannot have an entry behaviour"),
                (len(e, "exitActivities") > 0, "UML037", "Final state cannot have an exit behaviour"),
                (len(e, "doActivities") > 0, "UML038", "Final state cannot have a do activity"),
            ] {
                if cond {
                    fail(e, rule, msg);
                }
            }
        }
        if t == "UMLTransition" {
            let (Some(s), Some(g)) = (r.refd(e, "source"), r.refd(e, "target")) else { continue };
            let plain = len(e, "triggers") == 0 && !has_guard(e);
            if pkind(s) == "fork" {
                if !plain {
                    fail(e, "UML039", "Fork segment cannot have guards or triggers");
                }
                if !is_state(g) {
                    fail(e, "UML041", "Fork segment must target a state");
                }
            }
            if pkind(g) == "join" {
                if !plain {
                    fail(e, "UML040", "Join segment cannot have guards or triggers");
                }
                if !is_state(s) {
                    fail(e, "UML042", "Join segment must come from a state");
                }
            }
            if r.ty(s) == "UMLPseudostate" && pkind(s) != "initial" && len(e, "triggers") > 0 {
                fail(e, "UML043", "Transition from a pseudostate cannot have triggers");
            }
        }
        // UML044: a state machine's context is not an interface
        if t == "UMLStateMachine" && r.parent(e).map(|p| r.ty(p) == "UMLInterface").unwrap_or(false) {
            fail(e, "UML044", "State machine cannot be owned by an interface");
        }
        // activities
        let (ins, outs) = (incoming(&flows, e), outgoing(&flows, e));
        let mixed = || {
            let es: Vec<Eid> = ins.iter().chain(outs.iter()).copied().collect();
            !(es.iter().all(|x| r.ty(*x) == "UMLObjectFlow") || es.iter().all(|x| r.ty(*x) == "UMLControlFlow"))
        };
        match t {
            "UMLDecisionNode" if ins.is_empty() || ins.len() > 2 || outs.is_empty() => fail(e, "UML045", "Decision node must have one or two incoming edges and at least one outgoing edge"),
            "UMLMergeNode" => {
                if outs.len() != 1 {
                    fail(e, "UML046", "Merge node must have exactly one outgoing edge");
                }
                if mixed() {
                    fail(e, "UML047", "Edges of a merge node must be all object flows or all control flows");
                }
            }
            "UMLInitialNode" => {
                if !ins.is_empty() {
                    fail(e, "UML048", "Initial node cannot have incoming edges");
                }
                if outs.iter().any(|x| r.ty(*x) != "UMLControlFlow") {
                    fail(e, "UML049", "Only control flows can leave an initial node");
                }
            }
            "UMLActivityFinalNode" | "UMLFlowFinalNode" if !outs.is_empty() => fail(e, "UML050", "Final node cannot have outgoing edges"),
            "UMLForkNode" => {
                if ins.len() != 1 {
                    fail(e, "UML051", "Fork node must have exactly one incoming edge");
                }
                if mixed() {
                    fail(e, "UML052", "Edges of a fork node must be all object flows or all control flows");
                }
            }
            "UMLJoinNode" => {
                if outs.len() != 1 {
                    fail(e, "UML053", "Join node must have exactly one outgoing edge");
                }
                if mixed() {
                    fail(e, "UML054", "Edges of a join node must be all object flows or all control flows");
                }
            }
            "UMLObjectNode" if !r.bool(e, "isControlType") && ins.iter().chain(outs.iter()).any(|x| r.ty(*x) != "UMLObjectFlow") => {
                fail(e, "UML055", "Edges of an object node must be object flows")
            }
            "UMLControlFlow" => {
                let object = |v: Option<Eid>| v.map(|v| r.ty(v) == "UMLObjectNode" && !r.bool(v, "isControlType")).unwrap_or(false);
                if object(r.refd(e, "source")) || object(r.refd(e, "target")) {
                    fail(e, "UML056", "Control flow cannot connect an object node");
                }
            }
            "UMLObjectFlow" => {
                let action = |v: Option<Eid>| v.map(|v| r.ty(v) == "UMLAction").unwrap_or(false);
                if action(r.refd(e, "source")) && action(r.refd(e, "target")) {
                    fail(e, "UML057", "Object flow cannot connect two actions directly");
                }
            }
            _ => {}
        }
    }
    out
}

impl App {
    /// Validate, when the preference is on (after open and save, validation-panel.js:43-47). With a
    /// window it runs on a thread over a copy of the model, as the original sends the file to its main
    /// process (application.js:316-320), and the results arrive through `poll_validation`; without
    /// one (command line, tests) it runs at once.
    pub fn auto_validate(&mut self) {
        if !self.pref("validation.validateModel").as_bool().unwrap_or(true) {
            return;
        }
        let Some(ctx) = self.validate_in_background.clone() else {
            self.problems = validate(&self.repo);
            self.validated = true;
            return;
        };
        let repo = self.repo.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if tx.send(validate(&repo)).is_ok() {
                ctx.request_repaint();
            }
        });
        // a newer validation replaces one still running
        self.validation_rx = Some(rx);
    }

    /// Take the results of the background validation when they are ready.
    pub fn poll_validation(&mut self) {
        let Some(rx) = &self.validation_rx else { return };
        match rx.try_recv() {
            Ok(problems) => {
                self.problems = problems;
                self.validated = true;
                self.validation_rx = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => self.validation_rx = None,
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
