//! Interaction overview diagrams: an activity whose actions call interactions, shown as interaction
//! uses ("ref" boxes) or as frames drawing a sequence diagram in small, joined by the activity's
//! control nodes and flows (UMLInteractionOverviewDiagram and UMLInteractionInlineView,
//! elements.js:13823-13880; interactionOverviewDiagramFn, interactionUseInOverviewFn,
//! interactionInOverviewFn and viewForInteractionOverviewDiagramFn, uml-factory.js:441-530,
//! 1833-1950, 3306-3420).

use crate::app::App;
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Info, Measure, P, R};
use serde_json::json;
use std::collections::{HashMap, HashSet};

/// The sequence diagram an inline interaction's action calls.
pub fn inline_target(r: &Repo, v: Eid) -> Option<Eid> {
    let m = r.refd(v, "model")?;
    let t = r.refd(m, "target").filter(|t| r.alive(*t))?;
    (r.ty(t) == "UMLSequenceDiagram").then_some(t)
}

/// Lay out the sequence diagrams the inline interactions show, adding their views' layout to `info`.
pub fn layout_inlines(r: &mut Repo, ms: &dyn Measure, d: Eid, info: &mut HashMap<Eid, Info>) {
    let inlines: Vec<Eid> = r.list(d, "ownedViews").into_iter().filter(|v| r.ty(*v) == "UMLInteractionInlineView").collect();
    let mut done = HashSet::new();
    for v in inlines {
        if let Some(t) = inline_target(r, v) {
            if done.insert(t) {
                info.extend(views::layout_diagram(r, ms, t));
            }
        }
    }
}

/// Draws onto another canvas, scaled by `s` and moved by (`dx`, `dy`).
struct Shrunk<'a> {
    c: &'a mut dyn Canvas,
    s: f64,
    dx: f64,
    dy: f64,
}

impl Shrunk<'_> {
    fn p(&self, p: P) -> P {
        P { x: self.dx + p.x * self.s, y: self.dy + p.y * self.s }
    }
    fn r(&self, b: R) -> R {
        let o = self.p(P { x: b.x, y: b.y });
        R { x: o.x, y: o.y, w: b.w * self.s, h: b.h * self.s }
    }
}

impl Measure for Shrunk<'_> {
    fn width(&self, text: &str, style: u8) -> f64 {
        self.c.width(text, style)
    }
    fn size(&self) -> f64 {
        self.c.size()
    }
    fn face(&self) -> String {
        self.c.face()
    }
    fn width_face(&self, text: &str, style: u8, face: &str) -> f64 {
        self.c.width_face(text, style, face)
    }
}

impl Canvas for Shrunk<'_> {
    fn polyline(&mut self, pts: &[P], color: Rgba, dashed: bool) {
        let pts: Vec<P> = pts.iter().map(|p| self.p(*p)).collect();
        self.c.polyline(&pts, color, dashed);
    }
    fn polygon(&mut self, pts: &[P], fill: Option<Rgba>, stroke: Option<Rgba>) {
        let pts: Vec<P> = pts.iter().map(|p| self.p(*p)).collect();
        self.c.polygon(&pts, fill, stroke);
    }
    fn ellipse(&mut self, c: P, rx: f64, ry: f64, fill: Option<Rgba>, stroke: Option<Rgba>) {
        let c = self.p(c);
        self.c.ellipse(c, rx * self.s, ry * self.s, fill, stroke);
    }
    fn text(&mut self, t: &str, b: R, align: i64, style: u8, size: f64, color: Rgba, underline: bool) {
        let b = self.r(b);
        self.c.text(t, b, align, style, size * self.s, color, underline);
    }
    fn text_up(&mut self, t: &str, b: R, style: u8, size: f64, color: Rgba) {
        let b = self.r(b);
        self.c.text_up(t, b, style, size * self.s, color);
    }
    fn image(&mut self, b: R, data_url: &str) {
        let b = self.r(b);
        self.c.image(b, data_url);
    }
    fn set_face(&mut self, face: &str) {
        self.c.set_face(face);
    }
}

/// Draw the sequence diagram an inline interaction calls inside its frame, 10 px in and 30 px down,
/// shrunk to fit but never enlarged (UMLInteractionInlineView.drawObject, elements.js:13847-13875).
pub fn draw_inline(c: &mut dyn Canvas, r: &Repo, v: Eid, info: &HashMap<Eid, Info>) {
    let Some(t) = inline_target(r, v) else { return };
    let Some(bx) = crate::render::bounds(r, t) else { return };
    let b = views::node_rect(r, v);
    let s = ((b.w - 20.0) / bx.w.max(1.0)).min((b.h - 40.0) / bx.h.max(1.0)).min(1.0);
    if s <= 0.0 {
        return;
    }
    let mut sc = Shrunk { c, s, dx: b.x + 10.0 - bx.x * s, dy: b.y + 30.0 - bx.y * s };
    crate::render::draw_diagram(&mut sc, r, t, info);
}

impl App {
    /// The interaction overview tools: Interaction Use calls a new interaction through an interaction
    /// use; Interaction (Inline) calls a new interaction's new sequence diagram. Either way the
    /// interaction goes next to the one that owns the diagram's activity, and the diagram shows a
    /// new call-behavior action of the activity (uml-factory.js:1833-1950).
    pub fn create_overview_node(&mut self, d: Eid, kind: &str, rect: R) -> Option<Eid> {
        let act = self.diagram_activity(d)?;
        let inter = self.repo.parent(act)?;
        let owner = self.repo.parent(inter)?;
        let name = self.new_name(owner, "UMLInteraction");
        let called = self.repo.create("UMLInteraction", &[("name", json!(name))], Some((owner, "ownedElements")));
        let target = if kind == "UMLInteractionUseInOverview" {
            let ru = self.repo.ref_to(called);
            self.repo.create("UMLInteractionUse", &[("name", json!("InteractionUse1")), ("refersTo", ru)], Some((inter, "fragments")))
        } else {
            self.repo.create("UMLSequenceDiagram", &[("name", json!("SequenceDiagram1"))], Some((called, "ownedElements")))
        };
        self.overview_action(d, act, target, rect)
    }

    /// A call-behavior action of the activity calling `target`, and its view: an inline frame for a
    /// sequence diagram, an interaction use's box otherwise.
    fn overview_action(&mut self, d: Eid, act: Eid, target: Eid, rect: R) -> Option<Eid> {
        // the original names an inline interaction's action among the activity's fragments, which
        // it has none of, so always "Actions1"; UMLCraft counts the actions
        let base = if self.repo.ty(target) == "UMLSequenceDiagram" { "Actions" } else { "Action" };
        let names: HashSet<String> = self.repo.list(act, "nodes").into_iter().map(|n| self.repo.name(n)).collect();
        let name = (1..).map(|i| format!("{base}{i}")).find(|n| !names.contains(n)).unwrap();
        let rt = self.repo.ref_to(target);
        let m = self.repo.create("UMLAction", &[("name", json!(name)), ("kind", json!("callBehavior")), ("target", rt)], Some((act, "nodes")));
        let v = if self.repo.ty(target) == "UMLSequenceDiagram" {
            views::create_frame_view_of(&mut self.repo, "UMLInteractionInlineView", d, Some(m), rect)
        } else {
            crate::sequence::create_box_view(&mut self.repo, d, m, rect)
        };
        self.apply_view_defaults(v);
        Some(v)
    }

    /// An interaction use, an interaction or a sequence diagram dropped on an interaction overview
    /// diagram: an action calling it (an interaction through a new interaction use when the activity
    /// belongs to an interaction) (viewForInteractionOverviewDiagramFn, uml-factory.js:3306-3420).
    pub fn drop_on_overview(&mut self, d: Eid, m: Eid, p: P) -> Option<Eid> {
        let act = self.diagram_activity(d)?;
        let target = if self.repo.ty(m) == "UMLInteraction" {
            match self.repo.parent(act).filter(|i| self.repo.ty(*i) == "UMLInteraction") {
                Some(inter) => {
                    let ru = self.repo.ref_to(m);
                    self.repo.create("UMLInteractionUse", &[("refersTo", ru)], Some((inter, "fragments")))
                }
                None => m,
            }
        } else {
            m
        };
        let (w, h) = if self.repo.ty(target) == "UMLSequenceDiagram" { (1.0, 1.0) } else { (crate::sequence::FRAGMENT_MIN_W, crate::sequence::FRAGMENT_MIN_H) };
        self.overview_action(d, act, target, R { x: p.x, y: p.y, w, h })
    }
}
