//! Communication diagrams: lifelines as boxes joined by connectors between the roles they represent,
//! and messages drawn as short arrows beside a connector, numbered in the interaction's order
//! (UMLCommunicationDiagram, UMLCommLifelineView and UMLCommMessageView, elements.js:12906-13330;
//! lifelineFn, messageFn and connectorFn, uml-factory.js:1105-1340, 1950-1990).

use crate::app::{App, Tool};
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;
use std::f64::consts::{FRAC_PI_2, PI};

/// A new lifeline's smallest box (lifelineFn, uml-factory.js:1136-1143).
pub const LIFELINE_MIN: (f64, f64) = (100.0, 30.0);
/// A second box behind a multi-instance lifeline (MULTI_INSTANCE_MARGIN, elements.js:199).
const MULTI_MARGIN: f64 = 5.0;
/// Half the length of a message's arrow, and the length of its head's sides (elements.js:13148-13190).
const HALF_ARROW: f64 = 20.0;
const HEAD_LEN: f64 = 12.0;

/// Whether a line tool belongs to a communication diagram's own tools.
pub fn is_message_tool(kind: &str) -> bool {
    matches!(kind, "UMLForwardMessage" | "UMLReverseMessage")
}

fn comm_lifeline(r: &Repo, v: Eid) -> bool {
    r.ty(v) == "UMLCommLifelineView"
}

/// The role a lifeline view's lifeline represents, if it is a feature.
fn role(r: &Repo, v: Eid) -> Option<Eid> {
    let m = r.refd(v, "model")?;
    r.refd(m, "represent").filter(|x| r.alive(*x) && r.kind_of(*x, "UMLFeature"))
}

/// Whether a line of this kind may join these views on a communication diagram, or None when the
/// rule is not the communication diagram's: a connector joins two lifelines that represent roles; a
/// message goes on a connector, or between two such lifelines with a new connector (uml-factory.js:
/// 1231-1240, 1325-1330, 1956-1965).
pub fn can_link(r: &Repo, kind: &str, tail: Eid, head: Eid) -> Option<bool> {
    let lifelines = comm_lifeline(r, tail) && comm_lifeline(r, head) && role(r, tail).is_some() && role(r, head).is_some();
    if is_message_tool(kind) {
        return Some(lifelines || (r.ty(head) == "UMLConnectorView" && r.refd(head, "model").is_some()));
    }
    if matches!(kind, "UMLConnector" | "UMLSelfConnector") && (comm_lifeline(r, tail) || comm_lifeline(r, head)) {
        return Some(lifelines);
    }
    None
}

/// A lifeline's view: a box with its name compartment (UMLCommLifelineView).
pub fn create_lifeline_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let v = r.create(
        "UMLCommLifelineView",
        &[
            ("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)),
            ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w.max(LIFELINE_MIN.0))), ("height", json!(rect.h.max(LIFELINE_MIN.1))),
        ],
        Some((diagram, "ownedViews")),
    );
    let nc = views::name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    v
}

/// Size a lifeline's box to its name, `name[selector]: Type` not underlined, at the top.
pub fn layout_lifeline(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return };
    let min_w = views::name_min_width(r, ms, v, m, None, false);
    let w = r.f(v, "width").max(min_w);
    r.set(v, "width", json!(w));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    views::layout_name(r, ms, v, m, x, y, w, None, false);
    if let Some(nc) = r.refd(v, "nameCompartment") {
        if let Some(nm) = r.refd(nc, "nameLabel") {
            r.set(nm, "underline", json!(false));
        }
        let h = r.f(v, "height").max(r.f(nc, "height"));
        r.set(v, "height", json!(h));
    }
}

/// Draw a lifeline: a box, with a second one behind it when it stands for many instances
/// (drawAsCanonicalForm, elements.js:13023-13037).
pub fn draw_lifeline(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba) {
    let b = views::node_rect(r, v);
    if r.refd(v, "model").map(|m| r.bool(m, "isMultiInstance")).unwrap_or(false) {
        c.rect(R { x: b.x + MULTI_MARGIN, y: b.y + MULTI_MARGIN, ..b }, Some(fill), Some(line));
    }
    if r.bool(v, "showShadow") {
        let s = Rgba(crate::render::SHADOW_COLOR.0, crate::render::SHADOW_COLOR.1, crate::render::SHADOW_COLOR.2, (255.0 * crate::render::SHADOW_ALPHA) as u8);
        let o = crate::render::SHADOW_OFFSET;
        c.rect(R { x: b.x + o, y: b.y + o, ..b }, Some(s), None);
    }
    c.rect(b, Some(fill), Some(line));
}

/// A message's view beside the connector view `host`: its name, stereotype and property labels sit
/// 10, 25 and 10 px above it (UMLCommMessageView constructor).
pub fn create_message_view(r: &mut Repo, diagram: Eid, model: Eid, host: Eid) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "UMLCommMessageView",
        &[
            ("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("hostEdge", r.ref_to(host)), ("edgePosition", json!(1)),
            ("alpha", json!(FRAC_PI_2)), ("distance", json!(10)), ("left", json!(0)), ("top", json!(0)), ("width", json!(0)), ("height", json!(0)),
        ],
        Some((diagram, "ownedViews")),
    );
    for (field, dist) in [("nameLabel", 10.0), ("stereotypeLabel", 25.0), ("propertyLabel", 10.0)] {
        let l = r.create(
            "NodeLabelView",
            &[("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("alpha", json!(FRAC_PI_2)), ("distance", json!(dist)), ("height", json!(views::LABEL_H))],
            Some((v, "subViews")),
        );
        r.set_ref(v, field, Some(l));
    }
    v
}

/// The angle of the line from (x1, y1) to (x2, y2), screen y down (getTheta, elements.js:246-260).
fn theta(a: P, b: P) -> f64 {
    let (x, y) = (a.x - b.x, a.y - b.y);
    let mut th = (y.abs() / (x.abs() + 1e-10)).atan();
    if x > 0.0 {
        th = if y > 0.0 { PI - th } else { th + PI };
    } else if y < 0.0 {
        th = 2.0 * PI - th;
    }
    th
}

/// The middle segment of the host connector, and the message's arrow from its tail to its head:
/// 40 long along the connector, pointing from the message's source to its target
/// (_calcPosition, elements.js:13108-13180).
fn arrow(r: &Repo, v: Eid) -> Option<((P, P), P, P)> {
    let host = r.refd(v, "hostEdge").filter(|h| r.alive(*h))?;
    let pts = views::points_of(r, host);
    if pts.len() < 2 {
        return None;
    }
    let (p1, p2) = views::label_anchor(r, v, &pts);
    let th = theta(p1, p2);
    let (tx, ty) = (HALF_ARROW * th.cos(), HALF_ARROW * th.sin());
    let mid = P { x: r.f(v, "left") + tx.abs(), y: r.f(v, "top") + ty.abs() };
    let m = r.refd(v, "model")?;
    let forward = r.refd(host, "tail").and_then(|t| r.refd(t, "model")) == r.refd(m, "source");
    let (a, b) = (P { x: (mid.x + tx).floor(), y: (mid.y - ty).floor() }, P { x: (mid.x - tx).floor(), y: (mid.y + ty).floor() });
    Some(((p1, p2), if forward { a } else { b }, if forward { b } else { a }))
}

/// Place a message beside the middle of its connector and its labels above it
/// (EdgeParasiticView.arrangeObject, core.js:3633-3673; UMLCommMessageView.arrangeObject).
pub fn layout_message(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    for _ in 0..2 {
        let Some(((p1, p2), head, tail)) = arrow(r, v) else { return };
        let (w, h) = ((head.x - tail.x).abs(), (head.y - tail.y).abs());
        r.set(v, "width", json!(w));
        r.set(v, "height", json!(h));
        let d = views::point_away_line(p1, p2, r.f(v, "alpha"), r.f(v, "distance"));
        r.set(v, "left", json!(d.x + p1.x - (w / 2.0).floor()));
        r.set(v, "top", json!(d.y + p1.y - (h / 2.0).floor()));
    }
    let texts = views::edge_label_texts(r, v);
    let (face, size) = views::view_font(r, v);
    let scaled = views::Scaled { ms, face, size };
    let b = views::node_rect(r, v);
    let c = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
    // a property line pushes the name and stereotype 15 px further out
    let property = texts.iter().any(|(f, t)| *f == "propertyLabel" && !t.is_empty());
    if property {
        if let (Some(nl), Some(sl)) = (r.refd(v, "nameLabel"), r.refd(v, "stereotypeLabel")) {
            let nd = r.f(nl, "distance");
            if nd < 25.0 {
                if r.f(sl, "distance") <= nd + 15.0 {
                    r.set(sl, "distance", json!(r.f(sl, "distance") + 15.0));
                }
                r.set(nl, "distance", json!(nd + 15.0));
            }
        }
    }
    for (field, t) in texts {
        let Some(l) = r.refd(v, field) else { continue };
        r.set(l, "text", json!(t));
        r.set(l, "font", json!(r.str(v, "font")));
        if t.is_empty() {
            r.set(l, "visible", json!(false));
        } else {
            r.remove(l, "visible");
        }
        let w = if t.is_empty() { 0.0 } else { scaled.width(&t, 0) };
        let d = views::point_away_line(c, c, r.f(l, "alpha"), r.f(l, "distance"));
        r.set(l, "width", json!(w));
        r.set(l, "height", json!(size));
        r.set(l, "left", json!(d.x + c.x - w / 2.0));
        r.set(l, "top", json!(d.y + c.y - size / 2.0));
    }
}

/// After a message is dragged: keep it where it was put, as an angle and a distance from the
/// middle of its connector.
pub fn moved(r: &mut Repo, v: Eid) {
    let Some(host) = r.refd(v, "hostEdge") else { return };
    let pts = views::points_of(r, host);
    if pts.len() < 2 {
        return;
    }
    let (p1, p2) = views::label_anchor(r, v, &pts);
    let b = views::node_rect(r, v);
    let (alpha, distance) = views::label_polar(p1, p2, P { x: b.x + (b.w / 2.0).floor(), y: b.y + (b.h / 2.0).floor() });
    r.set(v, "alpha", json!(alpha));
    r.set(v, "distance", json!(distance));
}

/// After a connector's end is moved to another lifeline: its end refers to that lifeline's role, and
/// each message on it that came from or went to the old lifeline does so with the new one
/// (uml/main.js:49-68).
pub fn reconnected(r: &mut Repo, e: Eid, tail: bool) {
    let (Some(t), Some(h)) = (r.refd(e, "tail"), r.refd(e, "head")) else { return };
    let new = if tail { t } else { h };
    let other = if tail { h } else { t };
    if let (Some(m), Some(rl)) = (r.refd(e, "model"), role(r, new)) {
        if let Some(end) = r.child(m, if tail { "end1" } else { "end2" }) {
            r.set_ref(end, "reference", Some(rl));
        }
    }
    let (Some(nm), Some(om)) = (r.refd(new, "model"), r.refd(other, "model")) else { return };
    let Some(d) = r.parent(e) else { return };
    for v in r.list(d, "ownedViews") {
        if r.ty(v) != "UMLCommMessageView" || r.refd(v, "hostEdge") != Some(e) {
            continue;
        }
        let Some(msg) = r.refd(v, "model") else { continue };
        if r.refd(msg, "source") == Some(om) {
            r.set_ref(msg, "target", Some(nm));
        } else if r.refd(msg, "target") == Some(om) {
            r.set_ref(msg, "source", Some(nm));
        }
    }
}

/// Draw a message: its arrow, dotted for a reply, with an open head for asynchronous calls and
/// signals, create messages and replies, a filled one otherwise; and its labels
/// (UMLCommMessageView.drawObject).
pub fn draw_message(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba, fc: Rgba) {
    let Some((_, head, tail)) = arrow(r, v) else { return };
    let sort = r.refd(v, "model").map(|m| r.str(m, "messageSort")).unwrap_or_default();
    if sort == "reply" {
        crate::render::dashes(c, &[head, tail], line, 3.0, 3.0);
    } else {
        c.polyline(&[head, tail], line, false);
    }
    let (a, b) = (tail.y - head.y, tail.x - head.x + 0.00001);
    let mut th = (a / b).atan();
    if b < 0.0 {
        th += PI;
    }
    let tip = |t: f64| P { x: (HEAD_LEN * t.cos()).floor() + head.x, y: (HEAD_LEN * t.sin()).floor() + head.y };
    let (a1, a2) = (tip(th - PI / 8.0), tip(th + PI / 8.0));
    if matches!(sort.as_str(), "asynchCall" | "asynchSignal" | "createMessage" | "reply") {
        c.polyline(&[a1, head, a2], line, false);
    } else {
        c.polygon(&[a1, head, a2], Some(line), Some(line));
    }
    let (face, size) = views::view_font(r, v);
    let _ = face;
    for field in ["nameLabel", "stereotypeLabel", "propertyLabel"] {
        let Some(l) = r.refd(v, field) else { continue };
        if r.raw(l, "visible").and_then(|x| x.as_bool()) == Some(false) {
            continue;
        }
        let t = r.str(l, "text");
        if !t.is_empty() {
            c.text(&t, views::node_rect(r, l), 2, 0, size, fc, false);
        }
    }
}

impl App {
    /// A connector between two lifelines' roles, owned by the tail's role, and its view
    /// (connectorFn, uml-factory.js:1950-1990).
    pub fn comm_connector(&mut self, d: Eid, tail: Eid, head: Eid) -> Option<Eid> {
        let (rt, rh) = (role(&self.repo, tail)?, role(&self.repo, head)?);
        let m = self.repo.create("UMLConnector", &[], Some((rt, "ownedElements")));
        let (e1, e2) = (self.repo.ref_to(rt), self.repo.ref_to(rh));
        self.repo.create("UMLConnectorEnd", &[("reference", e1)], Some((m, "end1")));
        self.repo.create("UMLConnectorEnd", &[("reference", e2)], Some((m, "end2")));
        let v = views::create_edge_view(&mut self.repo, d, Some(m), tail, head)?;
        self.apply_view_defaults(v);
        Some(v)
    }

    /// A message on a connector view, from its tail's lifeline to its head's (forward) or back
    /// (reverse), last in the interaction's messages (messageFn, uml-factory.js:1300-1340).
    pub fn comm_message(&mut self, d: Eid, connector: Eid, forward: bool) -> Option<Eid> {
        let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
        let cm = self.repo.refd(connector, "model")?;
        let t = self.repo.refd(connector, "tail").and_then(|v| self.repo.refd(v, "model"))?;
        let h = self.repo.refd(connector, "head").and_then(|v| self.repo.refd(v, "model"))?;
        let (s, g) = if forward { (t, h) } else { (h, t) };
        let name = self.new_name(inter, "UMLMessage");
        let (rs, rg, rc) = (self.repo.ref_to(s), self.repo.ref_to(g), self.repo.ref_to(cm));
        let m = self.repo.create("UMLMessage", &[("name", json!(name)), ("source", rs), ("target", rg), ("connector", rc)], Some((inter, "messages")));
        let v = create_message_view(&mut self.repo, d, m, connector);
        self.apply_view_defaults(v);
        Some(v)
    }

    /// The communication diagram's lines: a connector (to itself for Self Connector), or a message
    /// on the connector it is dropped on, or between two lifelines with a new connector. None when
    /// the tool is not one of them.
    pub fn comm_edge(&mut self, d: Eid, tool: &Tool, tail: Eid, head: Eid) -> Option<Option<Eid>> {
        if self.repo.ty(d) != "UMLCommunicationDiagram" {
            return None;
        }
        match tool.kind {
            "UMLConnector" | "UMLSelfConnector" if comm_lifeline(&self.repo, head) => {
                let tail = if tool.kind == "UMLSelfConnector" { head } else { tail };
                Some(self.comm_connector(d, tail, head))
            }
            k if is_message_tool(k) => {
                let forward = k == "UMLForwardMessage";
                if self.repo.ty(head) == "UMLConnectorView" {
                    return Some(self.comm_message(d, head, forward));
                }
                let c = self.comm_connector(d, tail, head);
                Some(c.and_then(|c| self.comm_message(d, c, forward)))
            }
            _ => None,
        }
    }

    /// A communication lifeline's quick-edit buttons, as one undo step: a lifeline 70 px right
    /// joined by a connector, a connector to itself, or a lifeline 70 px below joined by a
    /// connector carrying a forward or reverse message (uml-commands.js:2735-2880).
    pub fn add_comm(&mut self, v: Eid, what: CommAdd) -> Option<Eid> {
        let d = self.diagram?;
        let b = views::node_rect(&self.repo, v);
        self.change(|a| match what {
            CommAdd::SelfConnector => a.comm_connector(d, v, v),
            CommAdd::ConnectedLifeline | CommAdd::ForwardMessage | CommAdd::ReverseMessage => {
                let right = what == CommAdd::ConnectedLifeline;
                // past the boxes already joined on that side
                let taken: Vec<R> = a.repo.list(d, "ownedViews").into_iter().filter(|e| a.repo.ty(*e) == "UMLConnectorView" && a.repo.refd(*e, "tail") == Some(v)).filter_map(|e| a.repo.refd(e, "head")).filter(|h| *h != v).map(|h| views::node_rect(&a.repo, h)).collect();
                let rect = if right {
                    let x = taken.iter().filter(|o| o.x >= b.x + b.w).map(|o| o.x + o.w + 70.0).fold(b.x + b.w + 70.0, f64::max);
                    R { x, y: b.y, w: LIFELINE_MIN.0, h: LIFELINE_MIN.1 }
                } else {
                    let y = taken.iter().filter(|o| o.y >= b.y + b.h).map(|o| o.y + o.h + 70.0).fold(b.y + b.h + 70.0, f64::max);
                    R { x: b.x, y, w: LIFELINE_MIN.0, h: LIFELINE_MIN.1 }
                };
                let n = a.create_node("UMLLifeline", rect)?;
                let c = a.comm_connector(d, v, n)?;
                match what {
                    CommAdd::ConnectedLifeline => Some(n),
                    _ => a.comm_message(d, c, what == CommAdd::ForwardMessage),
                }
            }
        })
    }

    /// Add Forward / Reverse Message on a connector's quick edit (uml-commands.js:2884-2930).
    pub fn add_comm_message(&mut self, connector: Eid, forward: bool) -> Option<Eid> {
        let d = self.diagram?;
        self.change(|a| a.comm_message(d, connector, forward))
    }
}

/// The communication lifeline quick edit's buttons (quickedits/uml.json "uml.lifeline-comm").
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CommAdd {
    ConnectedLifeline,
    SelfConnector,
    ForwardMessage,
    ReverseMessage,
}
