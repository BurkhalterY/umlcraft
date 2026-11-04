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
        "UMLPackage" | "UMLModel" | "UMLSubsystem" => vec![
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
        "UMLDataType" | "UMLPrimitiveType" | "UMLSignal" | "UMLEnumeration" => vec![
            r("Add Sub-Type", model_ty_static(model_ty), "UMLGeneralization", Plain, true, Below),
            r("Add Super-Type", model_ty_static(model_ty), "UMLGeneralization", Plain, false, Above),
        ],
        _ => vec![],
    };
    v.push(NOTE);
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
        // a note added this way is 100×50 (uml-commands.js:684)
        let size = if a.kind == "Note" { (100.0, 50.0) } else { views::default_size(a.kind) };
        let rect = self.related_rect(v, &a, size);
        let new = self.change(|app| {
            let n = app.create_node(a.kind, rect)?;
            if a.kind == "Note" {
                app.repo.set(n, "text", json!(""));
            }
            let tool = Tool { label: "", kind: a.rel, icon: "", line: true, init: a.init };
            let (tail, head) = if a.from_new { (n, v) } else { (v, n) };
            app.create_edge(&tool, tail, head);
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
    /// Add Note on a line: a 100×50 note to the right of the line's name label, joined to the line by a
    /// note link, as one undo step; the note's text edit opens (uml-commands.js:684).
    pub fn add_note_to_edge(&mut self, e: Eid) -> Option<Eid> {
        let anchor = match self.repo.refd(e, "nameLabel") {
            Some(l) => node_rect(&self.repo, l),
            None => {
                let pts = views::points_of(&self.repo, e);
                let mid = pts.get(pts.len() / 2).copied().unwrap_or_default();
                R { x: mid.x, y: mid.y, w: 0.0, h: 0.0 }
            }
        };
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
