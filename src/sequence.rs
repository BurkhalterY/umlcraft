//! Sequence diagrams: lifelines with their dashed line, messages between them at the height they
//! were drawn, activations, endpoints and gates (extensions/essential/uml: elements.js:10950-11990,
//! uml-factory.js:259-310, 1105-1510).

use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;

pub const ACTIVATION_W: f64 = 14.0; // elements.js:193-201
pub const ACTIVATION_MIN_H: f64 = 25.0;
pub const LINE_MIN_H: f64 = 50.0;
pub const LIFELINE_TOP: f64 = 40.0;
pub const HEAD_H: f64 = 40.0; // SEQ_OBJECT_MINHEIGHT
pub const HEAD_MIN_W: f64 = 30.0;
pub const MULTI_MARGIN: f64 = 5.0;
pub const SELF_W: f64 = 30.0;
pub const SELF_H: f64 = 20.0;
pub const ENDPOINT_SIZE: f64 = 15.0; // elements.js:239-240

/// The message tools: tool id and message sort (toolbox/uml.json "uml.sequence", "uml.sequence-adv").
pub fn message_sort(tool: &str) -> Option<&'static str> {
    Some(match tool {
        "UMLMessage" | "UMLSelfMessage" | "UMLFoundMessage" | "UMLLostMessage" => "synchCall",
        "UMLAsyncMessage" => "asynchCall",
        "UMLReplyMessage" => "reply",
        "UMLCreateMessage" => "createMessage",
        "UMLDeleteMessage" => "deleteMessage",
        "UMLAsyncSignalMessage" => "asynchSignal",
        _ => return None,
    })
}

/// A lifeline's view: its head (a name compartment) and its line part (UMLSeqLifelineView,
/// elements.js:11098-11105).
pub fn create_lifeline_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "UMLSeqLifelineView",
        &[
            ("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)),
            ("left", json!(rect.x)), ("top", json!(LIFELINE_TOP)), ("width", json!(rect.w)), ("height", json!(rect.h.max(200.0))),
        ],
        Some((diagram, "ownedViews")),
    );
    let nc = views::name_compartment(r, v, model);
    r.set_ref(v, "nameCompartment", Some(nc));
    let lp = r.create("UMLLinePartView", &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true))], Some((v, "subViews")));
    r.set_ref(v, "linePart", Some(lp));
    v
}

/// A message's view from `tail` to `head` (line parts or endpoint views) at the height `y`: its
/// labels and its activation (UMLSeqMessageView, elements.js:11608-11650, 11950-11965).
pub fn create_message_view(r: &mut Repo, diagram: Eid, model: Eid, tail: Eid, head: Eid, y: f64) -> Eid {
    let pts = if tail == head { vec![P { x: 0.0, y }; 4] } else { vec![P { x: 0.0, y }, P { x: 0.0, y }] };
    let e = r.create(
        "UMLSeqMessageView",
        &[
            ("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("head", r.ref_to(head)), ("tail", r.ref_to(tail)),
            ("lineStyle", json!(0)), ("points", json!(views::format_points(&pts))),
        ],
        Some((diagram, "ownedViews")),
    );
    let half = std::f64::consts::FRAC_PI_2;
    for (field, a, dist) in [("nameLabel", half, 10.0), ("stereotypeLabel", half, 25.0), ("propertyLabel", -half, 10.0)] {
        let l = views::edge_label(r, e, Some(model), a, dist, 1, false);
        r.set_ref(e, field, Some(l));
    }
    let act = r.create("UMLActivationView", &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("width", json!(ACTIVATION_W)), ("height", json!(ACTIVATION_MIN_H))], Some((e, "subViews")));
    r.set_ref(e, "activation", Some(act));
    e
}

/// An endpoint or gate: a 15 px dot or square centred on `at` (UMLMessageEndpointView, elements.js:11451-11515).
pub fn create_endpoint_view(r: &mut Repo, diagram: Eid, model: Eid, at: P) -> Eid {
    let ty = if r.ty(model) == "UMLGate" { "UMLGateView" } else { "UMLEndpointView" };
    let h = ENDPOINT_SIZE / 2.0;
    r.create(
        ty,
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(at.x - h)), ("top", json!(at.y - h)), ("width", json!(ENDPOINT_SIZE)), ("height", json!(ENDPOINT_SIZE))],
        Some((diagram, "ownedViews")),
    )
}

/// The text a message shows: its sequence number, guard and iteration, then `target = name(args)`
/// or its operation's or signal's signature (UMLMessage.getString, elements.js:3870-3935).
pub fn message_text(r: &Repo, d: Eid, m: Eid, show_type: bool) -> String {
    let mut s = String::new();
    let flag = |k: &str| r.raw(d, k).and_then(|v| v.as_bool()).unwrap_or(true);
    if flag("showSequenceNumber") {
        let numbering = r.str(d, "sequenceNumbering");
        if numbering == "custom" {
            let n = r.str(m, "sequenceNumber");
            if !n.is_empty() {
                s += &format!("{n} ");
            }
        } else if let Some(i) = r.parent(m).map(|p| r.list(p, "messages")).and_then(|l| l.iter().position(|x| *x == m)) {
            s += &format!("{} ", i + 1);
        }
    }
    let guard = r.str(m, "guard");
    if !guard.is_empty() {
        s += &format!("[{guard}] ");
    }
    let it = r.str(m, "iteration");
    if !it.is_empty() {
        s += &format!("{}{it}] ", if r.bool(m, "isConcurrentIteration") { "*||[" } else { "*[" });
    }
    if !s.is_empty() {
        s += ": ";
    }
    let target = r.str(m, "assignmentTarget");
    if !target.is_empty() {
        s += &format!("{target} = ");
    }
    let args = r.str(m, "arguments");
    let sig = flag("showSignature");
    match r.refd(m, "signature").filter(|x| r.alive(*x)) {
        Some(op) if r.ty(op) == "UMLOperation" => {
            s += &r.name(op);
            if sig {
                if !args.is_empty() {
                    s += &format!("({args})");
                } else {
                    let params: Vec<String> = r.list(op, "parameters").into_iter().filter(|p| r.str(*p, "direction") != "return").map(|p| {
                        let ty = views::type_name(r, p, "type");
                        if ty.is_empty() || !show_type { r.name(p) } else { format!("{}: {ty}", r.name(p)) }
                    }).collect();
                    s += &format!("({})", params.join(", "));
                    let ret = r.list(op, "parameters").into_iter().find(|p| r.str(*p, "direction") == "return").map(|p| views::type_name(r, p, "type")).unwrap_or_default();
                    if show_type && !ret.is_empty() {
                        s += &format!(":{ret}");
                    }
                }
            } else {
                s += "()";
            }
        }
        Some(sg) => {
            s += &r.name(sg);
            if sig && !args.is_empty() {
                s += &format!("({args})");
            }
        }
        None => {
            s += &r.name(m);
            if sig && !args.is_empty() {
                s += &format!("({args})");
            }
        }
    }
    s
}

fn rect(r: &Repo, v: Eid) -> R {
    views::node_rect(r, v)
}

fn set_rect(r: &mut Repo, v: Eid, b: R) {
    for (k, val) in [("left", b.x), ("top", b.y), ("width", b.w), ("height", b.h)] {
        r.set(v, k, json!(val));
    }
}

/// The message views of a diagram, in their order.
fn messages(r: &Repo, d: Eid) -> Vec<Eid> {
    r.list(d, "ownedViews").into_iter().filter(|v| r.alive(*v) && r.ty(*v) == "UMLSeqMessageView").collect()
}

fn first_y(r: &Repo, e: Eid) -> f64 {
    views::points_of(r, e).first().map(|p| p.y).unwrap_or(0.0)
}

fn last_y(r: &Repo, e: Eid) -> f64 {
    views::points_of(r, e).last().map(|p| p.y).unwrap_or(0.0)
}

fn activation_shown(r: &Repo, e: Eid) -> bool {
    r.refd(e, "activation").map(|a| r.raw(a, "visible").and_then(|v| v.as_bool()) != Some(false)).unwrap_or(false)
}

/// The front-most shown activation on a line part at the height y (UMLLinePartView.getActivationAt,
/// elements.js:11025-11048).
fn activation_at(r: &Repo, msgs: &[Eid], line: Eid, y: f64) -> Option<Eid> {
    let mut best: Option<Eid> = None;
    for &m in msgs {
        if r.refd(m, "head") != Some(line) || !activation_shown(r, m) {
            continue;
        }
        let Some(a) = r.refd(m, "activation") else { continue };
        let b = rect(r, a);
        if b.y <= y && b.y + b.h > y && best.map(|x| rect(r, x).y < b.y).unwrap_or(true) {
            best = Some(a);
        }
    }
    best
}

/// Lay out a sequence diagram after its boxes: lifelines at the top (or where they are created),
/// messages kept at their height between the line parts and activations, activations stretched
/// over the messages they receive, and the interaction's messages numbered top to bottom
/// (elements.js:10984-10998, 11049-11070, 11240-11300, 11518-11600, 11700-11950).
pub fn layout(r: &mut Repo, ms: &dyn Measure, d: Eid) {
    let mut msgs = messages(r, d);
    // the interaction's messages follow their height, then their left end (regulateSequenceNumber)
    let key = |r: &Repo, e: Eid| {
        let p = views::points_of(r, e).first().copied().unwrap_or_default();
        (p.y, p.x)
    };
    msgs.sort_by(|a, b| key(r, *a).partial_cmp(&key(r, *b)).unwrap_or(std::cmp::Ordering::Equal));
    if let Some(inter) = r.parent(d).filter(|p| r.ty(*p) == "UMLInteraction") {
        let order: Vec<Eid> = msgs.iter().filter_map(|e| r.refd(*e, "model")).filter(|m| r.parent(*m) == Some(inter)).collect();
        let cur = r.list(inter, "messages");
        if order.len() == cur.len() && order != cur {
            r.set_list(inter, "messages", order);
        }
    }
    let show_act = r.raw(d, "showActivation").and_then(|v| v.as_bool()).unwrap_or(true);
    // a lifeline created by a message stands at that message's height
    for v in r.list(d, "ownedViews") {
        if r.ty(v) != "UMLSeqLifelineView" || !r.alive(v) {
            continue;
        }
        let lp = r.refd(v, "linePart");
        let created = msgs.iter().find(|e| r.refd(**e, "head") == lp && r.refd(**e, "model").map(|m| r.str(m, "messageSort") == "createMessage").unwrap_or(false));
        let top = match created {
            Some(e) => (last_y(r, *e) - HEAD_H / 2.0).round(),
            None => LIFELINE_TOP,
        };
        r.set(v, "top", json!(top));
    }
    // activations: shown for synchronous, asynchronous and delete calls received by a lifeline;
    // their tops, ascending, so that nested ones sit half an activation to the right
    for &e in &msgs {
        let (Some(m), Some(a), Some(head)) = (r.refd(e, "model"), r.refd(e, "activation"), r.refd(e, "head")) else { continue };
        let sort = r.str(m, "messageSort");
        let to_line = r.ty(head) == "UMLLinePartView";
        let shown = to_line && show_act && matches!(sort.as_str(), "synchCall" | "asynchCall" | "deleteMessage");
        if shown {
            r.remove(a, "visible");
        } else {
            r.set_raw(a, "visible", json!(false));
        }
        let y = last_y(r, e);
        let parent = if to_line { activation_at(r, &msgs, head, y - 1.0).filter(|p| *p != a) } else { None };
        let hb = rect(r, head);
        let left = match (parent, shown) {
            (Some(p), true) => rect(r, p).x + ACTIVATION_W / 2.0,
            (Some(p), false) => rect(r, p).x,
            (None, true) => hb.x - ACTIVATION_W / 2.0,
            (None, false) => hb.x,
        };
        let h = r.f(a, "height").max(ACTIVATION_MIN_H);
        set_rect(r, a, R { x: left, y, w: ACTIVATION_W, h });
    }
    // their bottoms, descending, to cover the activations of the messages they receive
    for &e in msgs.iter().rev() {
        let (Some(a), Some(head)) = (r.refd(e, "activation"), r.refd(e, "head")) else { continue };
        if !activation_shown(r, e) {
            continue;
        }
        let b = rect(r, a);
        let mut bottom = b.y + ACTIVATION_MIN_H;
        for &o in &msgs {
            if o == e || r.refd(o, "head") != Some(head) {
                continue;
            }
            let y = last_y(r, o);
            if b.y <= y && b.y + b.h > y && activation_shown(r, o) {
                let ob = rect(r, r.refd(o, "activation").unwrap());
                bottom = bottom.max(ob.y + ob.h);
            }
        }
        if b.y + b.h < bottom + 3.0 {
            r.set(a, "height", json!(bottom + 3.0 - b.y));
        }
    }
    // the points: from the tail's side to the head's, at the message's height
    for &e in &msgs {
        let (Some(tail), Some(head)) = (r.refd(e, "tail"), r.refd(e, "head")) else { continue };
        if !r.alive(tail) || !r.alive(head) {
            continue;
        }
        let y = first_y(r, e);
        let act_box = |r: &Repo, line: Eid, y: f64| if r.ty(line) == "UMLLinePartView" { activation_at(r, &msgs, line, y).map(|a| rect(r, a)) } else { None };
        let from = act_box(r, tail, y);
        let tb = rect(r, tail);
        let hb = rect(r, head);
        let own = r.refd(e, "activation").filter(|_| activation_shown(r, e)).map(|a| rect(r, a));
        let pts = if tail == head {
            let x0 = from.map(|b| b.x + b.w).unwrap_or(tb.x + tb.w);
            let x3 = own.map(|b| b.x + b.w).unwrap_or(tb.x + tb.w);
            vec![P { x: x0, y }, P { x: x0 + SELF_W, y }, P { x: x0 + SELF_W, y: y + SELF_H }, P { x: x3, y: y + SELF_H }]
        } else if hb.x + hb.w / 2.0 >= tb.x + tb.w / 2.0 {
            let x0 = from.map(|b| b.x + b.w).unwrap_or(tb.x + tb.w);
            let x1 = own.map(|b| b.x).unwrap_or(hb.x);
            vec![P { x: x0, y }, P { x: x1, y }]
        } else {
            let x0 = from.map(|b| b.x).unwrap_or(tb.x);
            let x1 = own.map(|b| b.x + b.w).unwrap_or(hb.x + hb.w);
            vec![P { x: x0, y }, P { x: x1, y }]
        };
        // a create message ends at the side of the lifeline's head (arrange, elements.js:11780-11797)
        let mut pts = pts;
        if r.refd(e, "model").map(|m| r.str(m, "messageSort") == "createMessage").unwrap_or(false) && pts.len() == 2 {
            if let Some(lv) = r.parent(head).filter(|p| r.ty(*p) == "UMLSeqLifelineView") {
                let lb = rect(r, lv);
                pts[1].x = if pts[1].x > pts[0].x { lb.x } else { lb.x + lb.w };
            }
        }
        r.set(e, "points", json!(views::format_points(&pts)));
        // endpoints and gates sit on the message
        for end in [tail, head] {
            if r.kind_of(end, "UMLMessageEndpointView") {
                let h = ENDPOINT_SIZE;
                r.set(end, "top", json!(pts[1.min(pts.len() - 1)].y - h / 2.0));
                r.set(end, "width", json!(h));
                r.set(end, "height", json!(h));
            }
        }
        // the labels: «create» and «destroy» by sort, else the stereotype
        views::layout_edge_labels(r, ms, e, &pts);
    }
    // the line parts reach 15 px past the lowest message of their lifeline
    for v in r.list(d, "ownedViews") {
        if r.ty(v) != "UMLSeqLifelineView" || !r.alive(v) {
            continue;
        }
        let (Some(lp), Some(m)) = (r.refd(v, "linePart"), r.refd(v, "model")) else { continue };
        let b = rect(r, v);
        let head_bottom = r.refd(v, "nameCompartment").map(|nc| r.f(nc, "top") + r.f(nc, "height")).unwrap_or(b.y + HEAD_H);
        let top = head_bottom + 1.0;
        let mut lowest: f64 = 0.0;
        for &e in &msgs {
            let Some(mm) = r.refd(e, "model") else { continue };
            if r.refd(mm, "source") == Some(m) || r.refd(mm, "target") == Some(m) {
                for p in views::points_of(r, e) {
                    lowest = lowest.max(p.y);
                }
                if let Some(a) = r.refd(e, "activation").filter(|_| activation_shown(r, e) && r.refd(e, "head") == Some(lp)) {
                    let ab = rect(r, a);
                    lowest = lowest.max(ab.y + ab.h);
                }
            }
        }
        let min_line = if lowest > top + LINE_MIN_H { lowest - top + 15.0 } else { LINE_MIN_H };
        let h = b.h.max(top - b.y + min_line);
        r.set(v, "height", json!(h));
        let cx = (b.x + b.w / 2.0).round();
        set_rect(r, lp, R { x: cx, y: top, w: 1.0, h: b.y + h - top });
    }
}

/// A lifeline's head: at least 30 wide and 40 high, its name ("name[selector]: Type") in the middle
/// (UMLSeqLifelineView.sizeAsCanonicalForm and arrangeAsCanonicalForm, elements.js:11230-11270).
pub fn layout_lifeline(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return };
    if actor_icon(r, v) {
        // the iconic form: the actor in the top 40 px, the name under it, then the line
        // (sizeAsIconicForm and arrangeAsIconicForm, elements.js:11247-11251, 11293-11311)
        let min_w = views::name_min_width(r, ms, v, m, None, false).max(ICON_MIN);
        let w = r.f(v, "width").max(min_w);
        r.set(v, "width", json!(w));
        let (x, y) = (r.f(v, "left"), r.f(v, "top"));
        let nh = views::layout_name(r, ms, v, m, x, y + HEAD_H, w, None, false);
        if let Some(nm) = r.refd(v, "nameCompartment").and_then(|nc| r.refd(nc, "nameLabel")) {
            r.set(nm, "underline", json!(false));
        }
        let h = r.f(v, "height").max(HEAD_H + nh + LINE_MIN_H);
        r.set(v, "height", json!(h));
        return;
    }
    let min_w = views::name_min_width(r, ms, v, m, None, false).max(HEAD_MIN_W);
    let w = r.f(v, "width").max(min_w);
    r.set(v, "width", json!(w));
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    views::layout_name(r, ms, v, m, x, y, w, None, false);
    if let Some(nc) = r.refd(v, "nameCompartment") {
        // the head is 40 high whatever the name needs, the name centred in it
        let nh = r.f(nc, "height");
        let pad = ((HEAD_H - nh) / 2.0).max(0.0).floor();
        for l in r.list(nc, "subViews") {
            let t = r.f(l, "top");
            r.set(l, "top", json!(t + pad));
        }
        r.set(nc, "height", json!(HEAD_H.max(nh)));
        if let Some(nm) = r.refd(nc, "nameLabel") {
            r.set(nm, "underline", json!(false));
        }
    }
    let h = r.f(v, "height").max(HEAD_H + LINE_MIN_H);
    r.set(v, "height", json!(h));
}

const ICON_MIN: f64 = 24.0; // ICONICVIEW_ICONMINWIDTH (elements.js:73-74)

/// What a lifeline in icon display draws above its name.
#[derive(Clone, Debug, PartialEq)]
pub enum LifelineIcon {
    /// The actor's stick figure.
    Actor,
    /// An icon script or picture with its natural size.
    Icon(views::StereotypeIcon, f64, f64),
}

/// The node icon: a box with depth, in the node's 45 × 45 (UMLNodeViewMixin.drawIcon, elements.js:8032-8070).
const NODE_ICON: &str = "!icon 45 45\nfp 0 10 34 10 34 44 0 44\nfp 0 10 10 0 44 0 34 10\nfp 44 0 34 10 34 44 44 34\npg 0 10 34 10 34 44 0 44\npg 0 10 10 0 44 0 34 10\npg 44 0 34 10 34 44 44 34";
/// The use case ellipse, 70 × 32 (USECASE_ICON_MINWIDTH/HEIGHT, elements.js:96-99).
const USECASE_ICON: &str = "!icon 70 32\nfe 0 0 70 32\ne 0 0 70 32";
/// The interface ball (drawBallNotation, elements.js:6060-6063).
const BALL_ICON: &str = "!icon 100 100\nfe 0 0 100 100\ne 0 0 100 100";

/// The icon of a lifeline in icon display (Icon or Icon with Label): its own stereotype's icon, else
/// the icon of its role's type: an actor, a use case, an interface's ball, an artifact, a component,
/// a node, or the type's stereotype icon. Other types keep the box (UMLSeqLifelineView.drawIcon,
/// elements.js:11112-11206).
pub fn lifeline_icon(r: &Repo, v: Eid) -> Option<LifelineIcon> {
    if !matches!(r.str(v, "stereotypeDisplay").as_str(), "icon" | "icon-label") {
        return None;
    }
    let m = r.refd(v, "model")?;
    if let Some((i, w, h)) = views::stereotype_icon(r, m) {
        return Some(LifelineIcon::Icon(i, w, h));
    }
    let t = r.refd(m, "represent").and_then(|a| r.refd(a, "type"))?;
    let script = |s: &str, w: f64, h: f64| Some(LifelineIcon::Icon(views::StereotypeIcon::Script(s.to_string()), w, h));
    if r.kind_of(t, "UMLActor") {
        Some(LifelineIcon::Actor)
    } else if r.kind_of(t, "UMLUseCase") {
        script(USECASE_ICON, 70.0, 32.0)
    } else if r.kind_of(t, "UMLInterface") {
        script(BALL_ICON, 100.0, 100.0)
    } else if r.kind_of(t, "UMLArtifact") {
        views::kind_icon("UMLArtifactView", "icon").map(|(i, w, h)| LifelineIcon::Icon(i, w, h))
    } else if r.kind_of(t, "UMLComponent") {
        views::kind_icon("UMLComponentView", "icon").map(|(i, w, h)| LifelineIcon::Icon(i, w, h))
    } else if r.kind_of(t, "UMLNode") {
        script(NODE_ICON, 45.0, 45.0)
    } else {
        views::stereotype_icon(r, t).map(|(i, w, h)| LifelineIcon::Icon(i, w, h))
    }
}

/// Whether a lifeline shows as an icon rather than a box.
pub fn actor_icon(r: &Repo, v: Eid) -> bool {
    lifeline_icon(r, v).is_some()
}

/// Whether a classifier dropped on a sequence diagram gets a lifeline in icon display: an actor, a use
/// case, an interface, an artifact, a component, a node, or anything with a stereotype icon
/// (viewForSequenceDiagramFn, uml-factory.js:2788-2799).
pub fn shown_as_icon(r: &Repo, m: Eid) -> bool {
    ["UMLActor", "UMLUseCase", "UMLInterface", "UMLArtifact", "UMLComponent", "UMLNode"].iter().any(|k| r.kind_of(m, k)) || views::stereotype_icon(r, m).is_some()
}

/// Draw a lifeline: its head (a second one behind for a multi-instance lifeline, a bar each side for
/// an active class), its name and its dashed line (drawAsCanonicalForm and UMLLinePartView.drawObject,
/// elements.js:11085-11092, 11310-11360).
pub fn draw_lifeline(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba) {
    let b = rect(r, v);
    let hb = r.refd(v, "nameCompartment").map(|nc| r.f(nc, "top") + r.f(nc, "height")).unwrap_or(b.y + HEAD_H);
    let head = R { x: b.x, y: b.y, w: b.w, h: hb - b.y };
    let m = r.refd(v, "model");
    if let Some(icon) = lifeline_icon(r, v) {
        // the icon, as wide as its ratio allows in the top 40 px (computeIconRect)
        let area = R { x: b.x, y: b.y, w: b.w, h: HEAD_H };
        match icon {
            LifelineIcon::Actor => crate::render::draw_stick_figure(c, crate::render::fit_ratio(area, views::ACTOR_ICON_W, views::ACTOR_ICON_H), fill, line),
            LifelineIcon::Icon(i, w, h) => crate::render::draw_stereotype_icon(c, crate::render::fit_ratio(area, w.max(1.0), h.max(1.0)), &i, line, fill),
        }
        if let Some(lp) = r.refd(v, "linePart") {
            let lb = rect(r, lp);
            let cx = lb.x + lb.w / 2.0;
            crate::render::dashes(c, &[P { x: cx, y: lb.y }, P { x: cx, y: lb.y + lb.h }], line, 3.0, 3.0);
        }
        return;
    }
    let pts = |b: R| vec![P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }];
    if r.bool(v, "showShadow") {
        let s = Rgba(crate::render::SHADOW_COLOR.0, crate::render::SHADOW_COLOR.1, crate::render::SHADOW_COLOR.2, (255.0 * crate::render::SHADOW_ALPHA) as u8);
        let o = crate::render::SHADOW_OFFSET;
        c.polygon(&pts(R { x: head.x + o, y: head.y + o, ..head }), Some(s), None);
    }
    if m.map(|m| r.bool(m, "isMultiInstance")).unwrap_or(false) {
        c.polygon(&pts(R { x: head.x + MULTI_MARGIN, y: head.y + MULTI_MARGIN, ..head }), Some(fill), Some(line));
    }
    c.polygon(&pts(head), Some(fill), Some(line));
    if m.and_then(|m| r.refd(m, "represent")).and_then(|a| r.refd(a, "type")).map(|t| r.bool(t, "isActive")).unwrap_or(false) {
        for x in [head.x + 8.0, head.x + head.w - 8.0] {
            c.polyline(&[P { x, y: head.y }, P { x, y: head.y + head.h }], line, false);
        }
    }
    if let Some(lp) = r.refd(v, "linePart") {
        let lb = rect(r, lp);
        let cx = lb.x + lb.w / 2.0;
        crate::render::dashes(c, &[P { x: cx, y: lb.y }, P { x: cx, y: lb.y + lb.h }], line, 3.0, 3.0);
    }
}

/// Draw an endpoint (a filled dot) or a gate (a small box) (elements.js:11498-11515).
pub fn draw_endpoint(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba) {
    let b = rect(r, v);
    if r.ty(v) == "UMLGateView" {
        c.rect(b, Some(fill), Some(line));
    } else {
        c.ellipse(P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 }, b.w / 2.0, b.h / 2.0, Some(line), Some(line));
    }
}

// ------------------------------------------------------------------ fragments

const PAD: f64 = 5.0; // COMPARTMENT_*_PADDING
const LABEL_GAP: f64 = 5.0; // LABEL_INTERVAL (elements.js)
pub const FRAGMENT_MIN_W: f64 = 80.0; // FRAME_MINWIDTH (elements.js:218-229)
pub const FRAGMENT_MIN_H: f64 = 30.0;
pub const OPERAND_MIN_H: f64 = 30.0;
const GUARD_X: f64 = 20.0;
const GUARD_Y: f64 = 15.0;
pub const INVARIANT_MIN: (f64, f64) = (50.0, 20.0); // elements.js:212-216

fn label(r: &mut Repo, parent: Eid, model: Eid) -> Eid {
    let m = r.ref_to(model);
    r.create("LabelView", &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true))], Some((parent, "subViews")))
}

/// A combined fragment's view: a frame with its operator and name in the tab, and a compartment of
/// operand views (UMLCombinedFragmentView, elements.js:12495-12520).
pub fn create_fragment_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let v = r.create(
        "UMLCombinedFragmentView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h))],
        Some((diagram, "ownedViews")),
    );
    let nl = label(r, v, model);
    r.set_ref(v, "nameLabel", Some(nl));
    let tl = label(r, v, model);
    r.set_ref(v, "frameTypeLabel", Some(tl));
    let oc = r.create("UMLInteractionOperandCompartmentView", &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(true))], Some((v, "subViews")));
    r.set_ref(v, "operandCompartment", Some(oc));
    v
}

/// An operand's view in a fragment's operand compartment, `h` high, with its guard label.
pub fn create_operand_view(r: &mut Repo, compartment: Eid, operand: Eid, h: f64) -> Eid {
    let ov = r.create("UMLInteractionOperandView", &[("model", r.ref_to(operand)), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("height", json!(h))], Some((compartment, "subViews")));
    let gl = label(r, ov, operand);
    r.set_ref(ov, "guardLabel", Some(gl));
    ov
}

/// A continuation's or an interaction use's view (UMLContinuationView, UMLInteractionUseView,
/// elements.js:12098-12110, 12603-12620).
pub fn create_box_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let ty = if r.ty(model) == "UMLContinuation" { "UMLContinuationView" } else { "UMLInteractionUseView" };
    let v = r.create(
        ty,
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h))],
        Some((diagram, "ownedViews")),
    );
    let nl = label(r, v, model);
    r.set_ref(v, "nameLabel", Some(nl));
    if ty == "UMLInteractionUseView" {
        let tl = label(r, v, model);
        r.set_ref(v, "frameTypeLabel", Some(tl));
    }
    v
}

/// A state invariant on a lifeline: a rounded box with `{invariant}` on its line, kept in the
/// lifeline view's sub-views (stateInvariantFn, uml-factory.js:1506-1550).
pub fn create_invariant_view(r: &mut Repo, lifeline: Eid, model: Eid, y: f64) -> Eid {
    let v = r.create(
        "UMLStateInvariantView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(0)), ("top", json!(y)), ("width", json!(INVARIANT_MIN.0)), ("height", json!(INVARIANT_MIN.1))],
        Some((lifeline, "subViews")),
    );
    let l = label(r, v, model);
    r.set(l, "horizontalAlignment", json!(2));
    r.set(l, "verticalAlignment", json!(5));
    r.set_ref(v, "invariantLabel", Some(l));
    v
}

fn set_label(r: &mut Repo, l: Eid, text: &str, b: R, bold: bool, ms: &dyn Measure) {
    r.set(l, "text", json!(text));
    r.set(l, "font", json!(views::font_str(&ms.face(), ms.size(), if bold { 1 } else { 0 })));
    if text.is_empty() {
        r.set_raw(l, "visible", json!(false));
    } else {
        r.remove(l, "visible");
    }
    set_rect(r, l, b);
}

/// The text of an interaction use: `recipient = Interaction(arguments): value`
/// (UMLInteractionUseView.update, elements.js:12625-12650).
pub fn interaction_use_text(r: &Repo, m: Eid) -> String {
    let mut s = String::new();
    if let Some(x) = r.refd(m, "returnValueRecipient") {
        s += &format!("{} = ", r.name(x));
    }
    if let Some(x) = r.refd(m, "refersTo") {
        s += &r.name(x);
    }
    let args = r.str(m, "arguments");
    if !args.is_empty() {
        s += &format!("({args})");
    }
    let rv = r.str(m, "returnValue");
    if !rv.is_empty() {
        s += &format!(": {rv}");
    }
    s
}

/// Lay out a combined fragment, a continuation or an interaction use (sizeObject and arrangeObject
/// of their views, elements.js:12098-12720).
pub fn layout_fragment(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return };
    // on an interaction overview diagram the view shows an action calling the interaction use
    // (UMLInteractionUseView.update, elements.js:12625-12630)
    let m = if r.ty(m) == "UMLAction" { r.refd(m, "target").filter(|t| r.alive(*t)).unwrap_or(m) } else { m };
    let lh = ms.size();
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    match r.ty(v) {
        "UMLContinuationView" => {
            let name = r.name(m);
            let tw = ms.width(&name, 0).ceil();
            let w = r.f(v, "width").max(tw).max(INVARIANT_MIN.0);
            let h = r.f(v, "height").max(lh).max(INVARIANT_MIN.1);
            set_rect(r, v, R { x, y, w, h });
            if let Some(l) = r.refd(v, "nameLabel") {
                set_label(r, l, &name, R { x: x + (w - tw) / 2.0, y: y + (h - lh) / 2.0, w: tw, h: lh }, false, ms);
            }
        }
        "UMLInteractionUseView" => {
            let text = interaction_use_text(r, m);
            let (rw, nw) = (ms.width("ref", 1).ceil(), ms.width(&text, 0).ceil());
            let hh = lh + 2.0 * PAD;
            let w = r.f(v, "width").max(rw + 2.0 * PAD + lh + nw).max(FRAGMENT_MIN_W);
            let h = r.f(v, "height").max(hh).max(FRAGMENT_MIN_H);
            set_rect(r, v, R { x, y, w, h });
            if let Some(l) = r.refd(v, "frameTypeLabel") {
                set_label(r, l, "ref", R { x: x + PAD, y: y + PAD, w: rw, h: lh }, true, ms);
            }
            if let Some(l) = r.refd(v, "nameLabel") {
                let nx = x + PAD + rw + hh / 2.0 + 2.0 * PAD;
                set_label(r, l, &text, R { x: nx, y: y + PAD, w: (x + w - PAD - nx).max(0.0), h: lh }, false, ms);
            }
        }
        _ => {
            // a combined fragment: the operator (bold) and name in the tab, then the operands
            let op = r.raw(m, "interactionOperator").and_then(|v| v.as_str().map(String::from)).unwrap_or_else(|| "seq".into());
            let name = r.name(m);
            let (tw, nw) = (ms.width(&op, 1).ceil(), ms.width(&name, 0).ceil());
            let hh = lh + 2.0 * PAD;
            let tab_w = tw + nw + if !op.is_empty() && !name.is_empty() { LABEL_GAP } else { 0.0 } + 2.0 * PAD + hh / 2.0;
            let operands: Vec<Eid> = r.list(m, "operands").into_iter().filter(|o| r.alive(*o)).collect();
            let comp = r.refd(v, "operandCompartment");
            // one operand view per operand, keeping their heights
            let mut views_: Vec<Eid> = vec![];
            if let Some(oc) = comp {
                let old = r.list(oc, "subViews");
                for &o in &operands {
                    let ov = match old.iter().copied().find(|x| r.refd(*x, "model") == Some(o)) {
                        Some(ov) => ov,
                        None => create_operand_view(r, oc, o, OPERAND_MIN_H),
                    };
                    views_.push(ov);
                }
                for x in old {
                    if !views_.contains(&x) {
                        r.detach(x);
                    }
                }
                r.set_list(oc, "subViews", views_.clone());
            }
            let guard_text = |r: &Repo, o: Eid| {
                let g = r.str(o, "guard");
                if g.is_empty() { g } else { format!("[{g}]") }
            };
            let min_ops: f64 = views_.iter().map(|ov| r.f(*ov, "height").max(OPERAND_MIN_H)).sum::<f64>();
            let last_min = views_.last().map(|_| OPERAND_MIN_H).unwrap_or(0.0);
            let fixed: f64 = views_.iter().take(views_.len().saturating_sub(1)).map(|ov| r.f(*ov, "height").max(OPERAND_MIN_H)).sum::<f64>();
            let min_h = if views_.is_empty() { (hh + 30.0).max(FRAGMENT_MIN_H) } else { hh + fixed + last_min };
            let _ = min_ops;
            let guard_w = views_.iter().filter_map(|ov| r.refd(*ov, "model")).map(|o| ms.width(&guard_text(r, o), 0).ceil() + 2.0 * GUARD_X).fold(FRAGMENT_MIN_W, f64::max);
            let w = r.f(v, "width").max((tab_w + 10.0).max(FRAGMENT_MIN_W)).max(guard_w);
            let h = r.f(v, "height").max(min_h);
            set_rect(r, v, R { x, y, w, h });
            if let Some(l) = r.refd(v, "frameTypeLabel") {
                set_label(r, l, &op, R { x: x + PAD, y: y + PAD, w: tw, h: lh }, true, ms);
            }
            if let Some(l) = r.refd(v, "nameLabel") {
                let nx = if op.is_empty() { x + PAD } else { x + PAD + tw + 2.0 * LABEL_GAP };
                set_label(r, l, &name, R { x: nx, y: y + PAD, w: nw, h: lh }, false, ms);
            }
            if let Some(oc) = comp {
                set_rect(r, oc, R { x, y: y + hh, w, h: h - hh });
                if views_.is_empty() {
                    r.set_raw(oc, "visible", json!(false));
                } else {
                    r.remove(oc, "visible");
                }
            }
            let mut top = y + hh;
            let n = views_.len();
            for (i, ov) in views_.into_iter().enumerate() {
                let oh = if i + 1 == n { (y + h - top).max(OPERAND_MIN_H) } else { r.f(ov, "height").max(OPERAND_MIN_H) };
                set_rect(r, ov, R { x, y: top, w, h: oh });
                if let (Some(gl), Some(o)) = (r.refd(ov, "guardLabel"), r.refd(ov, "model")) {
                    let g = guard_text(r, o);
                    let gw = ms.width(&g, 0).ceil();
                    set_label(r, gl, &g, R { x: x + GUARD_X, y: top + GUARD_Y, w: gw, h: lh }, false, ms);
                }
                top += oh;
            }
        }
    }
}

/// A state invariant: at least 50 × 20, centred on its lifeline's line, `{invariant}` in the middle
/// (UMLStateInvariantView.sizeObject and arrangeObject, elements.js:12035-12060).
pub fn layout_invariant(r: &mut Repo, ms: &dyn Measure, lifeline: Eid, v: Eid) {
    let Some(m) = r.refd(v, "model").filter(|m| r.alive(*m)) else { return };
    let text = format!("{{{}}}", r.str(m, "invariant"));
    let (tw, lh) = (ms.width(&text, 0).ceil(), ms.size());
    let w = r.f(v, "width").max(tw).max(INVARIANT_MIN.0);
    let h = r.f(v, "height").max(lh).max(INVARIANT_MIN.1);
    let cx = r.refd(lifeline, "linePart").map(|lp| r.f(lp, "left")).unwrap_or(r.f(lifeline, "left") + r.f(lifeline, "width") / 2.0);
    let y = r.f(v, "top");
    set_rect(r, v, R { x: cx - w / 2.0, y, w, h });
    if let Some(l) = r.refd(v, "invariantLabel") {
        set_label(r, l, &text, R { x: cx - tw / 2.0, y: y + (h - lh) / 2.0, w: tw, h: lh }, false, ms);
    }
}

fn draw_text(c: &mut dyn Canvas, r: &Repo, l: Eid, fc: Rgba) {
    if r.raw(l, "visible").and_then(|v| v.as_bool()) == Some(false) {
        return;
    }
    let t = r.str(l, "text");
    let font = r.str(l, "font");
    let size = font.split(';').nth(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(13.0);
    let style = font.split(';').nth(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
    c.text(&t, rect(r, l), 0, style, size, fc, false);
}

/// The tab of a frame: a box over the top-left corner, its bottom-right corner cut by half its height
/// (UMLCustomFrameView.drawObject, elements.js:12218-12245).
fn frame_tab(c: &mut dyn Canvas, b: R, right_of_labels: f64, label_bottom: f64, fill: Option<Rgba>, line: Rgba) {
    let y = label_bottom + PAD;
    let h = y - b.y;
    let x = right_of_labels + PAD + h / 2.0;
    let pts = [P { x: b.x, y: b.y }, P { x: b.x, y }, P { x: x - h / 2.0, y }, P { x, y: b.y + h / 2.0 }, P { x, y: b.y }];
    c.polygon(&pts, fill, Some(line));
}

/// Draw a combined fragment, a continuation or an interaction use.
pub fn draw_fragment(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = rect(r, v);
    let corners = |b: R| vec![P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }];
    match r.ty(v) {
        "UMLContinuationView" => {
            c.polygon(&crate::flowchart::round_box(b, 10.0), Some(fill), Some(line));
            if let Some(l) = r.refd(v, "nameLabel") {
                draw_text(c, r, l, fc);
            }
        }
        "UMLInteractionUseView" => {
            c.polygon(&corners(b), Some(fill), Some(line));
            if let Some(tl) = r.refd(v, "frameTypeLabel") {
                let lb = rect(r, tl);
                frame_tab(c, b, lb.x + lb.w, lb.y + lb.h, None, line);
                draw_text(c, r, tl, fc);
            }
            if let Some(l) = r.refd(v, "nameLabel") {
                draw_text(c, r, l, fc);
            }
        }
        _ => {
            // the frame is not filled: what it covers shows through (UMLCustomFrameView.drawObject)
            let mut pts = corners(b);
            pts.push(pts[0]);
            c.polyline(&pts, line, false);
            let labels: Vec<Eid> = ["frameTypeLabel", "nameLabel"].iter().filter_map(|f| r.refd(v, f)).collect();
            let shown: Vec<R> = labels.iter().filter(|l| r.raw(**l, "visible").and_then(|x| x.as_bool()) != Some(false)).map(|l| rect(r, *l)).collect();
            let right = shown.iter().map(|l| l.x + l.w).fold(b.x + PAD, f64::max);
            let bottom = shown.iter().map(|l| l.y + l.h).fold(b.y + PAD, f64::max);
            frame_tab(c, b, right, bottom, Some(fill), line);
            for l in labels {
                draw_text(c, r, l, fc);
            }
            if let Some(oc) = r.refd(v, "operandCompartment") {
                for (i, ov) in r.list(oc, "subViews").into_iter().enumerate() {
                    let ob = rect(r, ov);
                    if i > 0 {
                        crate::render::dashes(c, &[P { x: ob.x, y: ob.y }, P { x: ob.x + ob.w - 1.0, y: ob.y }], line, 10.0, 3.0);
                    }
                    if let Some(gl) = r.refd(ov, "guardLabel") {
                        draw_text(c, r, gl, fc);
                    }
                }
            }
        }
    }
}

/// Draw the state invariants of a lifeline (a rounded box with `{invariant}`).
pub fn draw_invariants(c: &mut dyn Canvas, r: &Repo, lifeline: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    for v in r.list(lifeline, "subViews") {
        if r.ty(v) != "UMLStateInvariantView" {
            continue;
        }
        c.polygon(&crate::flowchart::round_box(rect(r, v), 10.0), Some(fill), Some(line));
        if let Some(l) = r.refd(v, "invariantLabel") {
            let lb = rect(r, l);
            let font = r.str(l, "font");
            let size = font.split(';').nth(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(13.0);
            c.text(&r.str(l, "text"), lb, 2, 0, size, fc, false);
        }
    }
}

// ------------------------------------------------------------------ quick edits

use crate::app::{App, AssocInit, Tool};

/// The lifeline quick edit's message buttons (quickedits/uml.json "uml.lifeline-seq").
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SeqAdd {
    MessageLifeline,
    CreateMessageLifeline,
    SelfMessage,
    FoundMessage,
    LostMessage,
    FromGate,
    ToGate,
}

impl App {
    /// Where a new message goes: 35 px (45 for a create message) under the lowest message, or 20 px
    /// down the lifeline's line when there is none (newSeqMessagePosition, uml-commands.js:2240-2270).
    fn next_message_y(&self, d: Eid, from: Eid, interval: f64) -> f64 {
        let lowest = messages(&self.repo, d).into_iter().map(|e| first_y(&self.repo, e)).fold(f64::NEG_INFINITY, f64::max);
        if lowest.is_finite() {
            lowest + interval
        } else {
            self.repo.refd(from, "linePart").map(|lp| self.repo.f(lp, "top") + 20.0).unwrap_or(LIFELINE_TOP + HEAD_H + 20.0)
        }
    }

    fn message_between(&mut self, kind: &'static str, tail: Eid, head: Eid, y: f64) -> Option<Eid> {
        self.line_from = Some((P { x: 0.0, y }, P { x: 0.0, y }));
        let tool = Tool { label: "", kind, icon: "", line: true, init: AssocInit::Plain };
        self.create_edge(&tool, tail, head)
    }

    /// A lifeline quick-edit button, as one undo step: a message to a new lifeline 20 px right of
    /// the rightmost one, a create message to one, a self message, or a found or lost message or a
    /// message from or to a gate 70 px off the lifeline (uml-commands.js:2225-2475).
    pub fn add_seq(&mut self, v: Eid, what: SeqAdd) -> Option<Eid> {
        let d = self.diagram?;
        let interval = if what == SeqAdd::CreateMessageLifeline { 45.0 } else { 35.0 };
        let y = self.next_message_y(d, v, interval);
        let b = rect(&self.repo, v);
        self.change(|a| match what {
            SeqAdd::MessageLifeline | SeqAdd::CreateMessageLifeline => {
                let right = a.repo.list(d, "ownedViews").into_iter().filter(|x| a.repo.ty(*x) == "UMLSeqLifelineView").map(|x| { let r = rect(&a.repo, x); r.x + r.w }).fold(b.x + b.w, f64::max);
                let n = a.create_node("UMLLifeline", R { x: right + 20.0, y: b.y, w: b.w.max(100.0), h: b.h })?;
                let kind = if what == SeqAdd::CreateMessageLifeline { "UMLCreateMessage" } else { "UMLMessage" };
                a.message_between(kind, v, n, y)
            }
            SeqAdd::SelfMessage => a.message_between("UMLMessage", v, v, y),
            SeqAdd::FoundMessage | SeqAdd::LostMessage | SeqAdd::FromGate | SeqAdd::ToGate => {
                let left = matches!(what, SeqAdd::FoundMessage | SeqAdd::FromGate);
                let x = if left { b.x - 70.0 } else { b.x + b.w + 70.0 };
                let kind = if matches!(what, SeqAdd::FromGate | SeqAdd::ToGate) { "UMLGate" } else { "UMLEndpoint" };
                let ep = a.create_node(kind, R { x: x.max(0.0), y, w: ENDPOINT_SIZE, h: ENDPOINT_SIZE })?;
                if left { a.message_between("UMLMessage", ep, v, y) } else { a.message_between("UMLMessage", v, ep, y) }
            }
        })
    }

    /// Add Reply Message: a reply from the message's head back to its tail, 20 px under it
    /// (handleAddReplyMessage, uml-commands.js:2612-2645).
    pub fn add_reply(&mut self, e: Eid) -> Option<Eid> {
        // on a communication diagram, a reply on the same connector, the other way
        if self.repo.ty(e) == "UMLCommMessageView" {
            let (d, host, m) = (self.diagram?, self.repo.refd(e, "hostEdge")?, self.repo.refd(e, "model")?);
            let forward = self.repo.refd(m, "source") != self.repo.refd(host, "tail").and_then(|t| self.repo.refd(t, "model"));
            return self.change(|a| {
                let v = a.comm_message(d, host, forward)?;
                let rm = a.repo.refd(v, "model")?;
                a.repo.set(rm, "messageSort", json!("reply"));
                Some(v)
            });
        }
        let (tail, head) = (self.repo.refd(e, "tail")?, self.repo.refd(e, "head")?);
        if self.repo.ty(tail) != "UMLLinePartView" || self.repo.ty(head) != "UMLLinePartView" {
            return None;
        }
        let y = first_y(&self.repo, e) + 20.0;
        self.change(|a| {
            let r = a.message_between("UMLMessage", head, tail, y)?;
            let m = a.repo.refd(r, "model")?;
            a.repo.set(m, "messageSort", json!("reply"));
            Some(r)
        })
    }

    /// The type of the lifeline a message goes to (its role's type), for Select / Create Operation.
    pub fn message_target_type(&self, m: Eid) -> Option<Eid> {
        let target = self.repo.refd(m, "target")?;
        let role = self.repo.refd(target, "represent")?;
        self.repo.refd(role, "type").filter(|t| self.repo.kind_of(*t, "UMLClassifier"))
    }
}

// ------------------------------------------------------------------ time and duration constraints

/// A duration constraint's smallest size and its arrowheads (elements.js:203-205).
pub const DURATION_MIN_W: f64 = 14.0;
pub const DURATION_MIN_H: f64 = 16.0;
const DURATION_ARROW: f64 = 4.0;

/// What a time or duration constraint shows: `{min..max}`, with "min" and "max" for empty bounds
/// (UMLTimeConstraintView.update, UMLDurationConstraintView.update).
pub fn interval_text(r: &Repo, m: Eid) -> String {
    let (min, max) = (r.str(m, "min"), r.str(m, "max"));
    format!("{{{}..{}}}", if min.is_empty() { "min" } else { &min }, if max.is_empty() { "max" } else { &max })
}

/// A duration constraint's view, with its label 20 px up and left of its centre
/// (UMLDurationConstraintView constructor).
pub fn create_duration_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let m = r.ref_to(model);
    let v = r.create(
        "UMLDurationConstraintView",
        &[("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(DURATION_MIN_W)), ("height", json!(rect.h.max(DURATION_MIN_H)))],
        Some((diagram, "ownedViews")),
    );
    // on a timing diagram it lies across
    if r.ty(diagram) == "UMLTimingDiagram" {
        r.set(v, "width", json!(rect.w.max(DURATION_MIN_H)));
        r.set(v, "height", json!(DURATION_MIN_W));
    }
    let l = r.create(
        "NodeLabelView",
        &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("alpha", json!(3.0 * std::f64::consts::FRAC_PI_4)), ("distance", json!(20)), ("height", json!(views::LABEL_H))],
        Some((v, "subViews")),
    );
    r.set_ref(v, "nameLabel", Some(l));
    v
}

/// A time constraint's label view at `at`, joined to the message view `msg` by a plain link
/// (timeConstraintFn, uml-factory.js:1718-1775).
pub fn create_time_views(r: &mut Repo, diagram: Eid, model: Eid, msg: Eid, at: P) -> Eid {
    let v = r.create(
        "UMLTimeConstraintView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("left", json!(at.x)), ("top", json!(at.y)), ("width", json!(0)), ("height", json!(views::LABEL_H))],
        Some((diagram, "ownedViews")),
    );
    r.create(
        "UMLTimeConstraintLinkView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("tail", r.ref_to(msg)), ("head", r.ref_to(v)), ("lineStyle", json!(0)), ("points", json!("0:0;0:0"))],
        Some((diagram, "ownedViews")),
    );
    v
}

/// Size a time constraint's label to its text.
pub fn layout_time(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let Some(m) = r.refd(v, "model") else { return };
    let t = interval_text(r, m);
    let (face, size) = views::view_font(r, v);
    let w = views::Scaled { ms, face, size }.width(&t, 0).ceil();
    r.set(v, "text", json!(t));
    r.set(v, "width", json!(w));
    r.set(v, "height", json!(size));
}

/// A duration constraint is 14 wide, stretching down only; its label sits up and left of it
/// (sizeObject, arrangeObject; NodeParasiticView.arrange, core.js:3600-3616).
pub fn layout_duration(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    // on a timing diagram it lies across: 14 high, stretching right
    if timing(r, v) {
        r.set(v, "height", json!(DURATION_MIN_W));
        let w = r.f(v, "width").max(DURATION_MIN_H);
        r.set(v, "width", json!(w));
    } else {
        r.set(v, "width", json!(DURATION_MIN_W));
        let h = r.f(v, "height").max(DURATION_MIN_H);
        r.set(v, "height", json!(h));
    }
    let (Some(m), Some(l)) = (r.refd(v, "model"), r.refd(v, "nameLabel")) else { return };
    let t = interval_text(r, m);
    let (face, size) = views::view_font(r, v);
    let w = views::Scaled { ms, face, size }.width(&t, 0).ceil();
    let b = rect(r, v);
    let c = P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
    let d = views::point_away_line(c, c, r.f(l, "alpha"), r.f(l, "distance"));
    r.set(l, "text", json!(t));
    r.set(l, "width", json!(w));
    r.set(l, "height", json!(size));
    r.set(l, "left", json!(d.x + c.x - w / 2.0));
    r.set(l, "top", json!(d.y + c.y - size / 2.0));
}

/// Draw a duration constraint: a line across its top and its bottom, and a double arrow between
/// them down its middle (UMLDurationConstraintView.drawObject); and its label.
pub fn draw_duration(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba, fc: Rgba) {
    if timing(r, v) {
        draw_duration_across(c, r, v, line, fc);
        return;
    }
    let b = rect(r, v);
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let seg = |c: &mut dyn Canvas, a: (f64, f64), z: (f64, f64)| c.polyline(&[P { x: a.0, y: a.1 }, P { x: z.0, y: z.1 }], line, false);
    seg(c, (x1, y1), (x2, y1));
    seg(c, (x1, y2), (x2, y2));
    let cx = x1 + (b.w / 2.0).round();
    seg(c, (cx, y1), (cx, y2));
    let a = DURATION_ARROW;
    c.polyline(&[P { x: cx - a, y: y1 + a * 2.0 }, P { x: cx, y: y1 }, P { x: cx + a, y: y1 + a * 2.0 }], line, false);
    c.polyline(&[P { x: cx - a, y: y2 - a * 2.0 }, P { x: cx, y: y2 }, P { x: cx + a, y: y2 - a * 2.0 }], line, false);
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_text(c, r, l, fc);
    }
}

fn timing(r: &Repo, v: Eid) -> bool {
    r.parent(v).map(|d| r.ty(d) == "UMLTimingDiagram").unwrap_or(false)
}

/// On a timing diagram: a bar at each end and a double arrow across the middle.
fn draw_duration_across(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba, fc: Rgba) {
    let b = rect(r, v);
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let seg = |c: &mut dyn Canvas, a: (f64, f64), z: (f64, f64)| c.polyline(&[P { x: a.0, y: a.1 }, P { x: z.0, y: z.1 }], line, false);
    seg(c, (x1, y1), (x1, y2));
    seg(c, (x2, y1), (x2, y2));
    let cy = y1 + (b.h / 2.0).round();
    seg(c, (x1, cy), (x2, cy));
    let a = DURATION_ARROW;
    c.polyline(&[P { x: x1 + a * 2.0, y: cy - a }, P { x: x1, y: cy }, P { x: x1 + a * 2.0, y: cy + a }], line, false);
    c.polyline(&[P { x: x2 - a * 2.0, y: cy - a }, P { x: x2, y: cy }, P { x: x2 - a * 2.0, y: cy + a }], line, false);
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_text(c, r, l, fc);
    }
}

/// Draw a time constraint's label.
pub fn draw_time(c: &mut dyn Canvas, r: &Repo, v: Eid, fc: Rgba) {
    let t = r.str(v, "text");
    let (_, size) = views::view_font(r, v);
    c.text(&t, rect(r, v), 2, 0, size, fc, false);
}
