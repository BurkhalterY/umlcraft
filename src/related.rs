//! The inline edit's "add related element" buttons: a new box next to the edited one, joined by a
//! relationship (extensions/essential/uml/quickedits/uml.json, uml-commands.js:684-2104,
//! ui/quickedit-manager.js:894-1036).

use crate::app::{App, AssocInit, Tool};
use crate::model::Eid;
use crate::views::{self, node_rect, R};
use serde_json::json;

/// Where the new box goes relative to the edited one.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Place {
    Below,
    Above,
    Right,
    Left,
}

/// One button: its label, the new element's type, the relationship, which way it points, and where
/// the new box goes. `from_new`: the relationship runs from the new element to the edited one.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Related {
    pub label: &'static str,
    pub kind: &'static str,
    pub rel: &'static str,
    pub init: AssocInit,
    pub from_new: bool,
    pub place: Place,
}

const fn r(label: &'static str, kind: &'static str, rel: &'static str, init: AssocInit, from_new: bool, place: Place) -> Related {
    Related { label, kind, rel, init, from_new, place }
}

const NOTE: Related = r("Add Note", "Note", "NoteLink", AssocInit::Plain, false, Place::Right);
/// Add Constraint: run by `add_constraint_to`, not as a related box (uml-commands.js:727).
const CONSTRAINT: Related = r("Add Constraint", "Constraint", "ConstraintLink", AssocInit::Plain, false, Place::Left);

/// The buttons for a model type (uml.json quick edits).
pub fn actions(model_ty: &str) -> Vec<Related> {
    use AssocInit::*;
    use Place::*;
    let mut v = match model_ty {
        "UMLClass" => vec![
            r("Add Sub-Class", "UMLClass", "UMLGeneralization", Plain, true, Below),
            r("Add Super-Class", "UMLClass", "UMLGeneralization", Plain, false, Above),
            r("Add Provided Interface", "UMLInterface", "UMLInterfaceRealization", Plain, false, Above),
            r("Add Required Interface", "UMLInterface", "UMLDependency", Plain, false, Above),
            r("Add Associated Class", "UMLClass", "UMLAssociation", Plain, false, Right),
            r("Add Aggregated Class", "UMLClass", "UMLAssociation", Shared, true, Right),
            r("Add Composited Class", "UMLClass", "UMLAssociation", Composite, true, Right),
        ],
        "UMLInterface" => vec![
            r("Add Sub-Interface", "UMLInterface", "UMLGeneralization", Plain, true, Below),
            r("Add Super-Interface", "UMLInterface", "UMLGeneralization", Plain, false, Above),
            r("Add Realizing Class", "UMLClass", "UMLInterfaceRealization", Plain, true, Below),
        ],
        // a subsystem has the interface buttons instead (quickedits/uml.json "uml.subsystem")
        "UMLSubsystem" => vec![
            r("Add Provided Interface", "UMLInterface", "UMLInterfaceRealization", Plain, false, Above),
            r("Add Required Interface", "UMLInterface", "UMLDependency", Plain, false, Above),
        ],
        "UMLPackage" | "UMLModel" => vec![
            r("Add Sub-Package", "UMLPackage", "Containment", Plain, true, Below),
            r("Add Dependant Package", "UMLPackage", "UMLDependency", Plain, true, Below),
            r("Add Depending Package", "UMLPackage", "UMLDependency", Plain, false, Above),
        ],
        "UMLUseCase" => vec![
            r("Add Associated Actor", "UMLActor", "UMLAssociation", Plain, true, Left),
            r("Add Associated Use Case", "UMLUseCase", "UMLAssociation", Plain, false, Right),
            r("Add Included Use Case", "UMLUseCase", "UMLInclude", Plain, false, Below),
            r("Add Extended Use Case", "UMLUseCase", "UMLExtend", Plain, false, Below),
        ],
        "UMLActor" => vec![
            r("Add Sub-Actor", "UMLActor", "UMLGeneralization", Plain, true, Below),
            r("Add Super-Actor", "UMLActor", "UMLGeneralization", Plain, false, Above),
            r("Add Associated Use Case", "UMLUseCase", "UMLAssociation", Plain, false, Right),
        ],
        // deployment and component buttons (uml-commands.js:1192-1251, 1830-1891)
        "UMLNode" => vec![
            r("Add Communicating Node", "UMLNode", "UMLCommunicationPath", Plain, false, Right),
            r("Add Deployed Artifact", "UMLArtifact", "UMLDeployment", Plain, true, Below),
            r("Add Deployed Component", "UMLComponent", "UMLDeployment", Plain, true, Below),
        ],
        "UMLComponent" => vec![
            r("Add Provided Interface", "UMLInterface", "UMLInterfaceRealization", Plain, false, Above),
            r("Add Required Interface", "UMLInterface", "UMLDependency", Plain, false, Above),
        ],
        // profiles (uml-commands.js:4069-4127)
        "UMLMetaClass" => vec![r("Add Stereotype", "UMLStereotype", "UMLExtension", Plain, true, Below)],
        "UMLStereotype" => vec![
            r("Add Sub-Stereotype", "UMLStereotype", "UMLGeneralization", Plain, true, Below),
            r("Add Super-Stereotype", "UMLStereotype", "UMLGeneralization", Plain, false, Above),
        ],
        // a state: a new state joined by an outgoing (right) or incoming (left) transition, an
        // initial state above and a final state below (uml-commands.js:2933-3215; sides of the
        // transitions estimated)
        "UMLState" => vec![
            r("Add Outgoing Transition", "UMLState", "UMLTransition", Plain, false, Right),
            r("Add Incoming Transition", "UMLState", "UMLTransition", Plain, true, Left),
            r("Add Initial State", "Initial", "UMLTransition", Plain, true, Above),
            r("Add Final State", "UMLFinalState", "UMLTransition", Plain, false, Below),
        ],
        // an action: a new action joined by an outgoing control flow (below) or an incoming one
        // (above), a new object node joined by an object flow, an initial node above and a final
        // node below (quickedits/uml.json "uml.action"; uml-commands.js:3600-4067)
        "UMLAction" => vec![
            r("Add Outgoing Control Flow", "UMLAction", "UMLControlFlow", Plain, false, Below),
            r("Add Incoming Control Flow", "UMLAction", "UMLControlFlow", Plain, true, Above),
            r("Add Outgoing Object Flow", "UMLObjectNode", "UMLObjectFlow", Plain, false, Below),
            r("Add Incoming Object Flow", "UMLObjectNode", "UMLObjectFlow", Plain, true, Above),
            r("Add Initial Node", "UMLInitialNode", "UMLControlFlow", Plain, true, Above),
            r("Add Final Node", "UMLActivityFinalNode", "UMLControlFlow", Plain, false, Below),
        ],
        // an object node: object flows first, then control flows, which make it a control type
        // (quickedits/uml.json "uml.object-node")
        "UMLObjectNode" | "UMLCentralBufferNode" | "UMLDataStoreNode" => vec![
            r("Add Outgoing Object Flow", "UMLObjectNode", "UMLObjectFlow", Plain, false, Below),
            r("Add Incoming Object Flow", "UMLObjectNode", "UMLObjectFlow", Plain, true, Above),
            r("Add Outgoing Control Flow", "UMLAction", "UMLControlFlow", Plain, false, Below),
            r("Add Incoming Control Flow", "UMLAction", "UMLControlFlow", Plain, true, Above),
        ],
        // an object: a linked object to the right (uml-commands.js:1675)
        "UMLObject" => vec![r("Add Linked Object", "UMLObject", "UMLLink", Plain, false, Right)],
        "UMLDataType" | "UMLPrimitiveType" | "UMLSignal" | "UMLEnumeration" => vec![
            r("Add Sub-Type", model_ty_static(model_ty), "UMLGeneralization", Plain, true, Below),
            r("Add Super-Type", model_ty_static(model_ty), "UMLGeneralization", Plain, false, Above),
        ],
        // a flowchart node: a process or decision below (outgoing) or above (incoming), joined by a
        // flow (quickedits/flowchart.json "fc.node"; flowchart-commands.js:16-95)
        t if crate::flowchart::is_node_type(t) => {
            return vec![
                r("Add Outgoing Process", "FCProcess", "FCFlow", Plain, false, Below),
                r("Add Incoming Process", "FCProcess", "FCFlow", Plain, true, Above),
                r("Add Outgoing Decision", "FCDecision", "FCFlow", Plain, false, Below),
                r("Add Incoming Decision", "FCDecision", "FCFlow", Plain, true, Above),
                NOTE,
            ];
        }
        // an entity: a new entity to the right joined one to one, one to many or many to many
        // (quickedits/erd.json "erd.entity"; erd-commands.js:60-130); a relationship: Add Note
        "ERDEntity" => {
            return vec![
                r("Add One to One", "ERDEntity", "ERDRelationship", Plain, false, Right),
                r("Add One to Many", "ERDEntity", "ERDRelationshipOneToMany", Plain, false, Right),
                r("Add Many to Many", "ERDEntity", "ERDRelationshipManyToMany", Plain, false, Right),
                NOTE,
            ]
        }
        "ERDRelationship" => return vec![NOTE],
        // every BPMN quick edit but a text annotation's adds a text annotation (quickedits/bpmn.json)
        t if t.starts_with("BPMN") => return crate::bpmn::related(t),
        // C4 elements and relationships: Add Note (quickedits/c4.json)
        t if t.starts_with("C4") => return vec![NOTE],
        // a flow: only Add Note ("fc.edge")
        "FCFlow" | "DFDDataFlow" => return vec![NOTE],
        // data flow nodes (dfd/quickedits/dfd.json; dfd-commands.js:19-118)
        "DFDExternalEntity" => return vec![r("Add Outgoing Process", "DFDProcess", "DFDDataFlow", Plain, false, Below), NOTE],
        "DFDProcess" => {
            return vec![
                r("Add Incoming External Entity", "DFDExternalEntity", "DFDDataFlow", Plain, true, Above),
                r("Add Outgoing Process", "DFDProcess", "DFDDataFlow", Plain, false, Below),
                r("Add Outgoing Data Store", "DFDDataStore", "DFDDataFlow", Plain, false, Below),
                NOTE,
            ]
        }
        "DFDDataStore" => return vec![r("Add Incoming Process", "DFDProcess", "DFDDataFlow", Plain, true, Above), NOTE],
        // a mind map node: a node to the left or right, its edge from the new node (an oblique line,
        // mindmap-commands.js:16-60); no note
        // named wireframe widgets and tab lists: Add Note (wireframe/quickedits/wireframe.json)
        "WFButton" | "WFText" | "WFRadio" | "WFCheckbox" | "WFLink" | "WFTab" | "WFInput" | "WFDropdown" | "WFTabList" => return vec![NOTE],
        t if t.starts_with("WF") => return vec![],
        "MMNode" => return vec![r("Add Node Left", "MMNode", "MMEdge", Plain, true, Left), r("Add Node Right", "MMNode", "MMEdge", Plain, true, Right)],
        _ => vec![],
    };
    v.push(NOTE);
    v.push(CONSTRAINT);
    v
}

fn model_ty_static(t: &str) -> &'static str {
    match t {
        "UMLDataType" => "UMLDataType",
        "UMLPrimitiveType" => "UMLPrimitiveType",
        "UMLSignal" => "UMLSignal",
        _ => "UMLEnumeration",
    }
}

impl App {
    /// Where the new box goes: next to the edited one (70 px below or to the side, 100 px above), or
    /// 10 px beyond the last box already placed that way for the same kind of relationship; never
    /// into negative space (quickedit-manager.js:894-1036).
    fn related_rect(&self, v: Eid, a: &Related, size: (f64, f64)) -> R {
        let b = node_rect(&self.repo, v);
        let (w, h) = size;
        let mut x = match a.place {
            Place::Right => b.x + b.w + 70.0,
            // a box made at its smallest size starts 100 px left, as the original places every new
            // box (getLeftPosition, quickedit-manager.js:970-977)
            Place::Left if w <= 1.0 => b.x - 100.0,
            Place::Left => b.x - 70.0 - w,
            _ => b.x + (b.w - w) / 2.0,
        };
        let mut y = match a.place {
            Place::Below => b.y + b.h + 70.0,
            Place::Above => b.y - 100.0 - h,
            _ => b.y,
        };
        // boxes already joined to this one the same way, on the same side
        if let Some(d) = self.diagram {
            let same: Vec<R> = self
                .repo
                .list(d, "ownedViews")
                .into_iter()
                .filter(|e| views::is_edge(&self.repo, *e) && self.edge_kind(*e) == a.rel)
                .filter_map(|e| {
                    let (t, hd) = (self.repo.refd(e, "tail")?, self.repo.refd(e, "head")?);
                    let other = if t == v { hd } else if hd == v { t } else { return None };
                    let o = node_rect(&self.repo, other);
                    let side = match a.place {
                        Place::Below => o.y >= b.y + b.h,
                        Place::Above => o.y + o.h <= b.y,
                        Place::Right => o.x >= b.x + b.w,
                        Place::Left => o.x + o.w <= b.x,
                    };
                    side.then_some(o)
                })
                .collect();
            match a.place {
                Place::Below | Place::Above => {
                    if let Some(o) = same.iter().max_by(|p, q| (p.x + p.w).total_cmp(&(q.x + q.w))) {
                        x = o.x + o.w + 10.0;
                        y = o.y;
                    }
                }
                Place::Right | Place::Left => {
                    if let Some(o) = same.iter().max_by(|p, q| (p.y + p.h).total_cmp(&(q.y + q.h))) {
                        x = o.x;
                        y = o.y + o.h + 10.0;
                    }
                }
            }
        }
        R { x: self.snap_round(x.max(0.0)), y: self.snap_round(y.max(0.0)), w, h }
    }

    fn snap_round(&self, n: f64) -> f64 {
        n.round()
    }

    /// Run a button: create the new element and its box, join them, select the new box and edit its name.
    /// One undo step.
    pub fn add_related(&mut self, v: Eid, a: Related) -> Option<Eid> {
        if a.kind == "Constraint" {
            return self.add_constraint_to(v);
        }
        // a note added this way is 100×50 (uml-commands.js:684)
        let size = if a.kind == "Note" { (100.0, 50.0) } else { views::default_size(a.kind) };
        let rect = self.related_rect(v, &a, size);
        let new = self.change(|app| {
            // a control flow on an object node makes it a control type (uml-commands.js:3607-3612)
            if a.rel == "UMLControlFlow" {
                if let Some(m) = app.repo.refd(v, "model").filter(|m| app.repo.kind_of(*m, "UMLObjectNode")) {
                    app.repo.set(m, "isControlType", json!(true));
                }
            }
            let n = app.create_node(a.kind, rect)?;
            if a.kind == "Note" {
                app.repo.set(n, "text", json!(""));
            }
            // a BPMN text annotation says so (bpmn-commands.js:196)
            if a.kind == "BPMNTextAnnotation" {
                if let Some(m) = app.repo.refd(n, "model") {
                    app.repo.set(m, "text", json!(crate::bpmn::ANNOTATION_TEXT));
                }
            }
            let tool = Tool { label: "", kind: a.rel, icon: "", line: true, init: a.init };
            let (tail, head) = if a.from_new { (n, v) } else { (v, n) };
            let e = app.create_edge(&tool, tail, head);
            // a mind map's quick edge is oblique (mindmap-commands.js:30-32)
            if let Some(e) = e.filter(|_| a.rel == "MMEdge") {
                app.repo.set(e, "lineStyle", json!(1));
            }
            Some(n)
        })?;
        self.select(vec![new]);
        if let Some(m) = self.repo.refd(new, "model") {
            self.reveal(m);
        }
        self.pending_edit = Some(new);
        Some(new)
    }
}

impl App {
    /// A port's Add Provided or Required Interface: the interface goes on the side of the box the port
    /// overhangs, joined by an interface realization or a dependency; a port with no classifier type
    /// first gets a new class "<port>Type" next to its box's classifier (uml-commands.js:1192-1290).
    pub fn add_port_interface(&mut self, v: Eid, provided: bool) -> Option<Eid> {
        let m = self.repo.refd(v, "model")?;
        let container = self.repo.refd(v, "containerView");
        let place = match container.map(|c| (node_rect(&self.repo, v), node_rect(&self.repo, c))) {
            Some((b, c)) if b.x + b.w > c.x + c.w => Place::Right,
            Some((b, c)) if b.x < c.x => Place::Left,
            Some((b, c)) if b.y + b.h > c.y + c.h => Place::Below,
            _ => Place::Above,
        };
        if !self.repo.refd(m, "type").map(|t| self.repo.kind_of(t, "UMLClassifier")).unwrap_or(false) {
            let owner = container.and_then(|c| self.repo.refd(c, "model")).and_then(|cm| self.repo.parent(cm));
            if let Some(owner) = owner {
                let name = format!("{}Type", self.repo.name(m));
                self.change(|a| {
                    let t = a.repo.create("UMLClass", &[("name", json!(name))], Some((owner, "ownedElements")));
                    a.repo.set_ref(m, "type", Some(t));
                });
            }
        }
        let rel = if provided { "UMLInterfaceRealization" } else { "UMLDependency" };
        let label = if provided { "Add Provided Interface" } else { "Add Required Interface" };
        self.add_related(v, Related { label, kind: "UMLInterface", rel, init: AssocInit::Plain, from_new: false, place })
    }

    /// Add Note on a line: a 100×50 note to the right of the line's name label, joined to the line by a
    /// note link, as one undo step; the note's text edit opens (uml-commands.js:684).
    /// Add Choice or Add Fork on a state: the pseudostate 50 px below it, and two new states 50 px
    /// below that, 70 px to its left and 35 px to its right, wired by three transitions, as one
    /// undo step (uml-commands.js:3244-3313, 3386-3455; the pseudostate's place is estimated).
    pub fn add_scaffold(&mut self, v: Eid, kind: &str) -> Option<Eid> {
        let b = node_rect(&self.repo, v);
        let (pw, ph) = views::default_size(kind);
        let (sw, sh) = views::default_size("UMLState");
        let tool = Tool { label: "", kind: "UMLTransition", icon: "", line: true, init: AssocInit::Plain };
        // Add Join: the join 40 px above the state, into it, and two new states 80 px above the join,
        // 40 px left of it and 50 px past its right end, each into the join (handleAddJoin,
        // uml-commands.js:3314-3378)
        if kind == "Join" {
            let p = R { x: b.x, y: (b.y - 40.0 - ph).max(0.0), w: pw, h: ph };
            return self.change(|app| {
                let pv = app.create_node(kind, p)?;
                app.create_edge(&tool, pv, v)?;
                for x in [p.x - 40.0, p.x + p.w + 50.0] {
                    let s = app.create_node("UMLState", R { x: x.max(0.0), y: (p.y - 80.0).max(0.0), w: sw, h: sh })?;
                    app.create_edge(&tool, s, pv)?;
                }
                Some(pv)
            });
        }
        let p = R { x: (b.x + b.w / 2.0 - pw / 2.0).round(), y: b.y + b.h + 50.0, w: pw, h: ph };
        self.change(|app| {
            let pv = app.create_node(kind, p)?;
            app.create_edge(&tool, v, pv)?;
            for x in [p.x - 70.0, p.x + 35.0] {
                let s = app.create_node("UMLState", R { x: x.max(0.0), y: p.y + p.h + 50.0, w: sw, h: sh })?;
                app.create_edge(&tool, pv, s)?;
            }
            Some(pv)
        })
    }

    /// Add Decision, Merge, Fork or Join on an action: the node 40 px below it (decision, fork) or
    /// above it (merge, join), level with its left side, and two new actions 40 px beyond the node,
    /// one 60 px (50 for fork and join) left of it and one at its right edge, wired by control flows
    /// running down the page; one undo step (uml-commands.js:3743-4004; the shift beside nodes
    /// already joined is not followed).
    pub fn add_activity_scaffold(&mut self, v: Eid, kind: &str) -> Option<Eid> {
        let b = node_rect(&self.repo, v);
        let (nw, nh) = views::default_size(kind);
        let below = matches!(kind, "UMLDecisionNode" | "UMLForkNode");
        let ny = if below { b.y + b.h + 40.0 } else { (b.y - 40.0).max(0.0) };
        let n = R { x: b.x, y: ny, w: nw, h: nh };
        let (aw, ah) = views::default_size("UMLAction");
        let left = if matches!(kind, "UMLForkNode" | "UMLJoinNode") { 50.0 } else { 60.0 };
        let ay = if below { n.y + n.h + 40.0 } else { (n.y - 80.0).max(0.0) };
        let tool = Tool { label: "", kind: "UMLControlFlow", icon: "", line: true, init: AssocInit::Plain };
        self.change(|app| {
            let nv = app.create_node(kind, n)?;
            if below { app.create_edge(&tool, v, nv)? } else { app.create_edge(&tool, nv, v)? };
            for x in [n.x - left, n.x + n.w] {
                let s = app.create_node("UMLAction", R { x: x.max(0.0), y: ay, w: aw, h: ah })?;
                if below { app.create_edge(&tool, nv, s)? } else { app.create_edge(&tool, s, nv)? };
            }
            Some(nv)
        })
    }

    /// What a note or constraint added to a line is placed by: its name label, or its middle point.
    pub fn edge_anchor(&self, e: Eid) -> R {
        match self.repo.refd(e, "nameLabel") {
            Some(l) => node_rect(&self.repo, l),
            None => {
                let pts = views::points_of(&self.repo, e);
                let mid = pts.get(pts.len() / 2).copied().unwrap_or_default();
                R { x: mid.x, y: mid.y, w: 0.0, h: 0.0 }
            }
        }
    }

    pub fn add_note_to_edge(&mut self, e: Eid) -> Option<Eid> {
        let anchor = self.edge_anchor(e);
        let rect = R { x: (anchor.x + anchor.w + 20.0).round(), y: (anchor.y - 20.0).max(0.0).round(), w: 100.0, h: 50.0 };
        let note = self.change(|app| {
            let n = app.create_node("Note", rect)?;
            app.repo.set(n, "text", json!(""));
            let tool = Tool { label: "", kind: "NoteLink", icon: "", line: true, init: AssocInit::Plain };
            app.create_edge(&tool, n, e);
            Some(n)
        })?;
        self.select(vec![note]);
        self.pending_edit = Some(note);
        Some(note)
    }
}
