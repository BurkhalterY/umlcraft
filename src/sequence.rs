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

/// Draw a lifeline: its head (a second one behind for a multi-instance lifeline, a bar each side for
/// an active class), its name and its dashed line (drawAsCanonicalForm and UMLLinePartView.drawObject,
/// elements.js:11085-11092, 11310-11360).
pub fn draw_lifeline(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba) {
    let b = rect(r, v);
    let hb = r.refd(v, "nameCompartment").map(|nc| r.f(nc, "top") + r.f(nc, "height")).unwrap_or(b.y + HEAD_H);
    let head = R { x: b.x, y: b.y, w: b.w, h: hb - b.y };
    let m = r.refd(v, "model");
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
