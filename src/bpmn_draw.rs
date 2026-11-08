//! Laying out and drawing BPMN views: the sizes and label places of every box (sizeObject and
//! arrangeObject), their shapes and marks (drawObject and drawShadow), the ends and patterns of the
//! lines, and the small icons (extensions/essential/bpmn/elements.js:836-3020).

use crate::bpmn::{self, ACTIVITY_ROUND, PAD};
use crate::flowchart::{cubic, round_box};
use crate::model::{Eid, Repo};
use crate::render::{self, Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;

fn p(x: f64, y: f64) -> P {
    P { x, y }
}

fn set_box(r: &mut Repo, v: Eid, b: R) {
    for (k, val) in [("left", b.x), ("top", b.y), ("width", b.w), ("height", b.h)] {
        r.set(v, k, json!(val));
    }
}

/// A name's smallest width and its height at a width: its widest word (the whole text without
/// word wrap) and its wrapped lines (LabelView.sizeObject, core.js:3462-3487).
fn label_size(ms: &dyn Measure, text: &str, wrap: bool, width: f64) -> (f64, f64) {
    if text.is_empty() {
        return (0.0, 0.0);
    }
    let w = if wrap { views::widest_word(ms, text, 0) } else { ms.width(text, 0) }.ceil();
    let lines = if wrap { views::wrap_text(ms, text, (width - 1.0).max(w).max(1.0), 0).len().max(1) } else { 1 };
    (w, lines as f64 * ms.size())
}

/// A view's smallest size, as its sizeObject computes it (used by pools and lanes for what they
/// hold, and by the layout).
pub fn min_size(r: &Repo, ms: &dyn Measure, v: Eid) -> (f64, f64) {
    let t = r.ty(v);
    let name = r.refd(v, "model").map(|m| r.name(m)).unwrap_or_default();
    let w0 = r.f(v, "width");
    match t {
        "BPMNEventView" => (bpmn::EVENT_MIN, bpmn::EVENT_MIN),
        "BPMNGatewayView" => (bpmn::GATEWAY_MIN, bpmn::GATEWAY_MIN),
        "BPMNMessageView" => bpmn::MESSAGE_MIN,
        "BPMNConversationView" => bpmn::CONVERSATION_MIN,
        "BPMNDataObjectView" | "BPMNDataInputView" | "BPMNDataOutputView" | "BPMNDataStoreView" => bpmn::DATAOBJECT_MIN,
        "BPMNGroupView" => (0.0, 0.0),
        "BPMNTextAnnotationView" => annotation_size(r, ms, v),
        "BPMNPoolView" | "BPMNLaneView" => swimlane_min(r, ms, v),
        _ => {
            let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
            let (lw, lh) = label_size(ms, &name, wrap, w0);
            (lw + 2.0 * PAD, lh + 2.0 * PAD)
        }
    }
}

/// A text annotation's smallest size: its lines, each wrapped to the box, 2 px apart, with 5 px
/// padding, at least 10 × 10 (BPMNTextAnnotationView.sizeObject, elements.js:2584-2616).
fn annotation_size(r: &Repo, ms: &dyn Measure, v: Eid) -> (f64, f64) {
    let text = r.refd(v, "model").map(|m| r.str(m, "text")).unwrap_or_default();
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let width = r.f(v, "width") - 2.0 * PAD;
    let (mut w, mut h) = (0.0f64, 0.0);
    if !text.is_empty() {
        for line in text.split('\n') {
            if wrap {
                let lines = if line.is_empty() { 1 } else { views::wrap_text(ms, line, width.max(1.0), 0).len().max(1) };
                w = w.max(views::widest_word(ms, line, 0).ceil());
                h += lines as f64 * ms.size() + 2.0;
            } else {
                w = w.max(ms.width(line, 0).ceil());
                h += ms.size() + 2.0;
            }
        }
    }
    ((w + 2.0 * PAD).max(bpmn::TEXTANNOTATION_MIN), (h + 2.0 * PAD).max(bpmn::TEXTANNOTATION_MIN))
}

/// The views a pool or lane holds.
fn contained(r: &Repo, v: Eid) -> Vec<Eid> {
    match r.raw(v, "containedViews") {
        Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| r.by_ref(x)).filter(|x| r.alive(*x)).collect(),
        _ => vec![],
    }
}

/// Whether a pool or lane lies down: a lane takes its pool's direction (BPMNLaneView.update).
fn vertical(r: &Repo, v: Eid) -> bool {
    if r.ty(v) == "BPMNLaneView" {
        if let Some(c) = r.refd(v, "containerView").filter(|c| r.ty(*c) == "BPMNPoolView") {
            return r.bool(c, "isVertical");
        }
    }
    r.bool(v, "isVertical")
}

/// A pool's or lane's smallest size: its name (written up the side of a horizontal one) with 5 px
/// around it, beside the widest thing it holds and as high as the highest (BPMNPoolView.sizeObject
/// and BPMNLaneView.sizeObject, elements.js:2799-2826, 2946-2972).
fn swimlane_min(r: &Repo, ms: &dyn Measure, v: Eid) -> (f64, f64) {
    let name = r.refd(v, "model").map(|m| r.name(m)).unwrap_or_default();
    let (tw, th) = if name.is_empty() { (0.0, 0.0) } else { (ms.width(&name, 0).ceil(), ms.size()) };
    let lane = r.ty(v) == "BPMNLaneView";
    let (mut sw, mut sh) = (0.0f64, 0.0f64);
    for c in contained(r, v) {
        let (cw, ch) = min_size(r, ms, c);
        sw = sw.max(cw);
        sh = sh.max(if lane && !vertical(r, v) { ch } else { r.f(c, "height") });
    }
    if vertical(r, v) {
        let (nw, nh) = (tw + 2.0 * PAD, th + 2.0 * PAD);
        if lane { (nw.max(sw), nh + sh) } else { (nw.max(sw), nh.max(sh)) }
    } else {
        // the label is turned: its width is the text's height
        let (nw, nh) = (th + 2.0 * PAD, tw + 2.0 * PAD);
        if lane { (nw + sw, nh.max(sh + 2.0 * PAD)) } else { (nw + sw, nh.max(sh)) }
    }
}

/// Grow a view to its smallest size, or take it with auto resize (NodeView.sizeConstraints).
fn fit(r: &mut Repo, v: Eid, min: (f64, f64)) -> R {
    let mut b = views::node_rect(r, v);
    if r.bool(v, "autoResize") && min.0 > 0.0 && min.1 > 0.0 {
        b.w = min.0;
        b.h = min.1;
    }
    b.w = b.w.max(min.0);
    b.h = b.h.max(min.1);
    r.set(v, "width", json!(b.w));
    r.set(v, "height", json!(b.h));
    b
}

/// Size a BPMN box and place its labels and what it lines up.
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let t = r.ty(v).to_string();
    let m = r.refd(v, "model").filter(|m| r.alive(*m));
    let name = m.map(|m| r.name(m)).unwrap_or_default();
    // a boundary event sits on its activity's border, at the point nearest where it was put
    // (BPMNEventView.arrange, elements.js:1634-1647)
    if t == "BPMNEventView" {
        if let Some(c) = r.refd(v, "containerView").filter(|c| bpmn::is_activity_view(r.ty(*c))) {
            let cb = views::node_rect(r, c);
            let b = views::node_rect(r, v);
            let at = junction2(cb, p(b.x + b.w / 2.0, b.y + b.h / 2.0));
            let h = bpmn::EVENT_MIN / 2.0;
            set_box(r, v, R { x: at.x - h, y: at.y - h, w: 2.0 * h, h: 2.0 * h });
        }
    }
    let min = min_size(r, ms, v);
    let b = fit(r, v, min);
    let Some(l) = r.refd(v, "nameLabel") else { return };
    if bpmn::is_floating(&t) {
        // the name floats below, on one line, shown when there is one (BPMNFloatingNodeView.arrange)
        r.set(l, "text", json!(name));
        r.set(l, "visible", json!(!name.is_empty()));
        let w = ms.width(&name, 0).ceil();
        let h = ms.size();
        let c = p(b.x + b.w / 2.0, b.y + b.h / 2.0);
        let off = views::point_away_line(c, c, r.f(l, "alpha"), r.f(l, "distance"));
        set_box(r, l, R { x: off.x + c.x - w / 2.0, y: off.y + c.y - h / 2.0, w, h });
        return;
    }
    let (tw, th) = if name.is_empty() { (0.0, 0.0) } else { (ms.width(&name, 0).ceil(), ms.size()) };
    r.set(l, "text", json!(name));
    match t.as_str() {
        "BPMNPoolView" | "BPMNLaneView" => {
            let vert = vertical(r, v);
            if t == "BPMNLaneView" {
                r.set(v, "isVertical", json!(vert));
            }
            // the name across the top of a vertical one, up the left side of a horizontal one
            let lb = if vert { R { x: b.x, y: b.y + PAD, w: b.w, h: th } } else { R { x: b.x + PAD, y: b.y, w: th, h: b.h } };
            set_box(r, l, lb);
            if t == "BPMNPoolView" {
                arrange_lanes(r, ms, v, b, lb, vert);
            }
        }
        _ => {
            // the name across the box, centred in its height (BPMNGeneralNodeView.arrangeObject)
            let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
            r.set(l, "wordWrap", json!(wrap));
            let lh = if name.is_empty() { 0.0 } else if wrap { views::wrap_text(ms, &name, (b.w - 1.0).max(1.0), 0).len().max(1) as f64 * ms.size() } else { ms.size() };
            let _ = tw;
            let top = (b.y + b.h / 2.0).round() - (lh / 2.0).round();
            set_box(r, l, R { x: b.x, y: top, w: b.w, h: lh });
        }
    }
}

/// A pool lines up its lanes in their order, top to bottom (left to right in a vertical pool), each
/// from the name's side to the far edge (BPMNPoolView.arrangeObject, elements.js:2827-2870).
fn arrange_lanes(r: &mut Repo, ms: &dyn Measure, pool: Eid, b: R, lb: R, vert: bool) {
    let mut lanes: Vec<Eid> = contained(r, pool).into_iter().filter(|x| r.ty(*x) == "BPMNLaneView").collect();
    if vert {
        lanes.sort_by(|a, c| r.f(*a, "left").total_cmp(&r.f(*c, "left")));
    } else {
        lanes.sort_by(|a, c| r.f(*a, "top").total_cmp(&r.f(*c, "top")));
    }
    let mut at = if vert { b.x } else { b.y };
    for lane in lanes {
        r.set(lane, "isVertical", json!(vert));
        let min = min_size(r, ms, lane);
        let lr = views::node_rect(r, lane);
        let nb = if vert {
            let top = lb.y + lb.h + PAD;
            R { x: at, y: top, w: lr.w.max(min.0), h: (b.y + b.h - top).max(min.1) }
        } else {
            let left = lb.x + lb.w + PAD;
            R { x: left, y: at, w: (b.x + b.w - left).max(min.0), h: lr.h.max(min.1) }
        };
        set_box(r, lane, nb);
        at += if vert { nb.w } else { nb.h };
        layout(r, ms, lane);
    }
}

/// Where a boundary event goes on its activity's box: on the line from the box's centre through the
/// event's centre, the crossing with the border nearest the event when it is inside, or the one
/// between them when it is outside (BPMNEventView._junction2, elements.js:1569-1632).
pub fn junction2(r: R, q: P) -> P {
    let (x1, y1, x2, y2) = (r.x, r.y, r.x + r.w, r.y + r.h);
    let c = p((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    if c.x == q.x || c.y == q.y {
        // straight across: the side the point is towards (Coord.orthoJunction)
        return if c.x == q.x {
            if q.y < c.y { p(c.x, y1) } else { p(c.x, y2) }
        } else if q.x < c.x {
            p(x1, c.y)
        } else {
            p(x2, c.y)
        };
    }
    let lean = (q.y - c.y) / (q.x - c.x);
    let cp = [p(x1, (lean * (x1 - c.x) + c.y).round()), p(x2, (lean * (x2 - c.x) + c.y).round()), p(((y1 - c.y) / lean + c.x).round(), y1), p(((y2 - c.y) / lean + c.x).round(), y2)];
    let inside = q.x >= x1 && q.x <= x2 && q.y >= y1 && q.y <= y2;
    if inside {
        let d = |a: P| ((a.x - q.x).powi(2) + (a.y - q.y).powi(2)).sqrt();
        return *cp.iter().min_by(|a, b| d(**a).total_cmp(&d(**b))).unwrap();
    }
    let (lx, hx, ly, hy) = (c.x.min(q.x), c.x.max(q.x), c.y.min(q.y), c.y.max(q.y));
    cp.iter().copied().find(|a| a.x >= x1 && a.x <= x2 && a.y >= y1 && a.y <= y2 && a.x >= lx && a.x <= hx && a.y >= ly && a.y <= hy).unwrap_or(c)
}

// ------------------------------------------------------------------ drawing helpers

/// An ellipse as points, closed.
fn ellipse_pts(c: P, rx: f64, ry: f64) -> Vec<P> {
    (0..=48).map(|i| i as f64 / 48.0 * std::f64::consts::TAU).map(|a| p(c.x + rx * a.cos(), c.y + ry * a.sin())).collect()
}

fn closed(pts: &[P]) -> Vec<P> {
    let mut v = pts.to_vec();
    if let (Some(a), Some(b)) = (v.first().copied(), v.last().copied()) {
        if a != b {
            v.push(a);
        }
    }
    v
}

/// Fill a polygon that may be concave, cut into triangles (ear clipping): the shapes of the event
/// marks are not convex.
pub fn fill_any(c: &mut dyn Canvas, pts: &[P], fill: Rgba) {
    let mut v: Vec<P> = pts.to_vec();
    if v.len() > 1 && v[0] == v[v.len() - 1] {
        v.pop();
    }
    if v.len() < 3 {
        return;
    }
    let area: f64 = (0..v.len()).map(|i| { let (a, b) = (v[i], v[(i + 1) % v.len()]); a.x * b.y - b.x * a.y }).sum();
    if area < 0.0 {
        v.reverse();
    }
    let cross = |a: P, b: P, q: P| (b.x - a.x) * (q.y - a.y) - (b.y - a.y) * (q.x - a.x);
    let mut guard = 0;
    while v.len() > 3 && guard < 1000 {
        guard += 1;
        let n = v.len();
        let mut cut = None;
        for i in 0..n {
            let (a, b, d) = (v[(i + n - 1) % n], v[i], v[(i + 1) % n]);
            if cross(a, b, d) <= 0.0 {
                continue;
            }
            let inside = (0..n).filter(|j| *j != i && *j != (i + n - 1) % n && *j != (i + 1) % n).any(|j| {
                let q = v[j];
                cross(a, b, q) > 0.0 && cross(b, d, q) > 0.0 && cross(d, a, q) > 0.0
            });
            if !inside {
                cut = Some(i);
                break;
            }
        }
        let Some(i) = cut else { break };
        let n = v.len();
        c.polygon(&[v[(i + n - 1) % n], v[i], v[(i + 1) % n]], Some(fill), None);
        v.remove(i);
    }
    c.polygon(&v, Some(fill), None);
}

/// A closed shape: filled (concave shapes too), then outlined.
fn shape(c: &mut dyn Canvas, pts: &[P], fill: Option<Rgba>, line: Option<Rgba>) {
    if let Some(f) = fill {
        fill_any(c, pts, f);
    }
    if let Some(l) = line {
        c.polyline(&closed(pts), l, false);
    }
}

fn seg(c: &mut dyn Canvas, a: P, b: P, line: Rgba) {
    c.polyline(&[a, b], line, false);
}

fn shadow_color() -> Rgba {
    Rgba(render::SHADOW_COLOR.0, render::SHADOW_COLOR.1, render::SHADOW_COLOR.2, (255.0 * render::SHADOW_ALPHA) as u8)
}

fn shifted(pts: &[P]) -> Vec<P> {
    pts.iter().map(|q| p(q.x + render::SHADOW_OFFSET, q.y + render::SHADOW_OFFSET)).collect()
}

/// The marks under an activity or choreography: loop, sequential or parallel multi-instance,
/// compensation, sub-process and ad hoc, 14 px each, centred at the bottom (BPMNActivityView.drawObject,
/// elements.js:999-1074).
fn draw_markers(c: &mut dyn Canvas, markers: &[&str], xc: f64, bottom: f64, line: Rgba, fc: Rgba, size: f64) {
    let mut mx = xc - (markers.len() as f64 * 14.0 / 2.0).round();
    for mk in markers {
        match *mk {
            "loop" => {
                // an arc from -1.35π to 0.3π, 4 around (mx + 6, bottom - 9), with its arrow head
                let (cx, cy) = (mx + 6.0, bottom - 9.0);
                let (a0, a1) = (-std::f64::consts::PI * 1.35, std::f64::consts::PI * 0.3);
                let arc: Vec<P> = (0..=16).map(|i| a0 + (a1 - a0) * i as f64 / 16.0).map(|a| p(cx + 4.0 * a.cos(), cy + 4.0 * a.sin())).collect();
                c.polyline(&arc, line, false);
                seg(c, p(mx + 5.0, bottom - 5.0), p(mx + 1.0, bottom - 5.0), line);
                seg(c, p(mx + 5.0, bottom - 5.0), p(mx + 5.0, bottom - 9.0), line);
            }
            "seq" => {
                for dy in [13.0, 9.0, 5.0] {
                    seg(c, p(mx + 2.0, bottom - dy), p(mx + 12.0, bottom - dy), line);
                }
            }
            "par" => {
                for dx in [6.0, 2.0, 10.0] {
                    seg(c, p(mx + dx, bottom - 14.0), p(mx + dx, bottom - 4.0), line);
                }
            }
            "compensate" => {
                shape(c, &[p(mx + 1.0, bottom - 9.0), p(mx + 7.0, bottom - 15.0), p(mx + 7.0, bottom - 3.0)], None, Some(line));
                shape(c, &[p(mx + 7.0, bottom - 9.0), p(mx + 13.0, bottom - 15.0), p(mx + 13.0, bottom - 3.0)], None, Some(line));
            }
            "sub" => {
                c.rect(R { x: mx + 1.0, y: bottom - 15.0, w: 12.0, h: 12.0 }, None, Some(line));
                seg(c, p(mx + 3.0, bottom - 9.0), p(mx + 11.0, bottom - 9.0), line);
                seg(c, p(mx + 7.0, bottom - 13.0), p(mx + 7.0, bottom - 5.0), line);
            }
            "adhoc" => {
                c.text("~", R { x: mx + 1.0, y: bottom - 15.0, w: 14.0, h: size }, 0, 0, size, fc, false);
            }
            _ => {}
        }
        if *mk != "adhoc" {
            mx += 14.0;
        }
    }
}

/// An activity's marks from its flags (BPMNActivityView.update and its kinds' update).
fn activity_markers(r: &Repo, v: Eid, m: Eid) -> Vec<&'static str> {
    let mut mk = vec![];
    if r.bool(m, "isLoop") {
        mk.push("loop");
    }
    if r.bool(m, "isMultiInstance") {
        mk.push(if r.bool(m, "isSequential") { "seq" } else { "par" });
    }
    if r.bool(m, "isForCompensation") {
        mk.push("compensate");
    }
    let called_sub = r.refd(m, "calledRef").map(|c| r.kind_of(c, "BPMNSubProcess")).unwrap_or(false);
    match r.ty(v) {
        "BPMNSubProcessView" | "BPMNTransactionView" => mk.push("sub"),
        "BPMNAdHocSubProcessView" => mk.extend(["sub", "adhoc"]),
        "BPMNCallActivityView" if called_sub => mk.push("sub"),
        _ => {}
    }
    mk
}

/// A task's type mark in its top left corner, 14 px square at (6, 6) (BPMNTaskView.drawObject,
/// elements.js:1219-1323); a call activity shows the mark of the task it calls.
fn draw_task_mark(c: &mut dyn Canvas, ty: &str, instantiate: bool, b: R, line: Rgba, fill: Rgba) {
    let (x1, y1) = (b.x + 6.0, b.y + 6.0);
    let (x2, y2) = (x1 + 14.0, y1 + 14.0);
    let (xc, yc) = (((x1 + x2) / 2.0).round(), ((y1 + y2) / 2.0).round());
    match ty {
        "BPMNSendTask" => {
            shape(c, &[p(x1, y1), p(xc, y1 + 5.0), p(x2, y1)], Some(line), None);
            shape(c, &[p(x1, y1 + 2.0), p(xc, y1 + 7.0), p(x2, y1 + 2.0), p(x2, y2 - 4.0), p(x1, y2 - 4.0)], Some(line), None);
        }
        "BPMNReceiveTask" => {
            c.rect(R { x: x1, y: y1 + 1.0, w: x2 - x1, h: y2 - 3.0 - y1 - 1.0 }, None, Some(line));
            seg(c, p(x1, y1 + 1.0), p(xc, y1 + 7.0), line);
            seg(c, p(xc, y1 + 7.0), p(x2, y1 + 1.0), line);
            if instantiate {
                c.ellipse(p((x1 + x2) / 2.0, (y1 - 3.0 + y2 + 2.0) / 2.0), (x2 - x1 + 8.0) / 2.0, (y2 - y1 + 5.0) / 2.0, None, Some(line));
            }
        }
        "BPMNServiceTask" => {
            // a gear: a thick ring with six teeth
            c.thick_polyline(&ellipse_pts(p(xc, yc), 4.0, 4.0), line, 3.0);
            for (dx, dy) in [(0.0, -6.0), (0.0, 6.0), (-5.0, 3.0), (-5.0, -3.0), (5.0, 3.0), (5.0, -3.0)] {
                c.ellipse(p(xc + dx, yc + dy), 2.0, 2.0, Some(line), Some(line));
            }
            let _ = fill;
        }
        "BPMNUserTask" => {
            c.ellipse(p(xc, y1 + 3.0), 3.0, 3.0, None, Some(line));
            c.rect(R { x: x1 + 2.0, y: y1 + 7.0, w: x2 - x1 - 4.0, h: y2 - y1 - 7.0 }, None, Some(line));
        }
        "BPMNManualTask" => {
            shape(
                c,
                &[p(x1, yc - 4.0), p(x1 + 3.0, yc - 4.0), p(x1 + 5.0, y1), p(x2 - 4.0, y1), p(x2 - 4.0, y1 + 3.0), p(x2, y1 + 3.0), p(x2, y2 - 2.0), p(x1 + 5.0, y2 - 2.0), p(x1 + 3.0, y2 - 4.0), p(x1, y2 - 4.0)],
                None,
                Some(line),
            );
            for dy in [3.0, 6.0, 9.0] {
                seg(c, p(xc, y1 + dy), p(x2, y1 + dy), line);
            }
        }
        "BPMNBusinessRuleTask" => {
            c.rect(R { x: x1, y: y1, w: x2 - x1, h: y2 - 3.0 - y1 }, None, Some(line));
            seg(c, p(x1, y1 + 3.0), p(x2, y1 + 3.0), line);
            seg(c, p(x1, y1 + 6.0), p(x2, y1 + 6.0), line);
            seg(c, p(x1 + 3.0, y1), p(x1 + 3.0, y2 - 3.0), line);
        }
        "BPMNScriptTask" => {
            shape(c, &[p(x1 + 3.0, y1), p(x2, y1), p(x2 - 3.0, y1 + 4.0), p(x2, y1 + 8.0), p(x2 - 3.0, y1 + 12.0), p(x1, y1 + 12.0), p(x1 + 3.0, y1 + 8.0), p(x1, y1 + 4.0)], None, Some(line));
            seg(c, p(x1 + 3.0, y1 + 3.0), p(x2 - 4.0, y1 + 3.0), line);
            seg(c, p(x1 + 3.5, y1 + 6.0), p(x2 - 3.5, y1 + 6.0), line);
            seg(c, p(x1 + 4.0, y1 + 9.0), p(x2 - 3.0, y1 + 9.0), line);
        }
        _ => {}
    }
}

fn draw_wrapped_label(c: &mut dyn Canvas, r: &Repo, v: Eid, fc: Rgba) {
    let Some(l) = r.refd(v, "nameLabel") else { return };
    if r.raw(l, "visible").and_then(|x| x.as_bool()) == Some(false) {
        return;
    }
    let t = r.str(l, "text");
    if t.is_empty() {
        return;
    }
    let (face, size) = views::view_font(r, v);
    let style = r.str(v, "font").split(';').nth(2).and_then(|x| x.parse::<u8>().ok()).unwrap_or(0);
    let b = views::node_rect(r, l);
    let lines = if r.bool(l, "wordWrap") { views::wrap_text(&views::Scaled { ms: &*c, face, size }, &t, b.w - 1.0, style) } else { vec![t] };
    for (i, s) in lines.iter().enumerate() {
        c.text(s, R { x: b.x, y: b.y + i as f64 * size, w: b.w, h: size }, 2, style, size, fc, false);
    }
}

/// The events a model's definitions make (BPMNEventView.drawObject, elements.js:1649-1700).
fn has_def(r: &Repo, m: Eid, ty: &str) -> bool {
    r.list(m, "eventDefinitions").into_iter().any(|d| r.alive(d) && r.ty(d) == ty)
}

/// Draw a BPMN box (each view's drawShadow and drawObject).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let Some(m) = r.refd(v, "model") else { return };
    let t = r.ty(v).to_string();
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let (xc, yc) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    let shadow = r.bool(v, "showShadow");
    let sc = shadow_color();
    let size = views::view_font(r, v).1;
    match t.as_str() {
        _ if bpmn::is_activity_view(&t) => {
            let rb = round_box(b, ACTIVITY_ROUND);
            if shadow {
                c.polygon(&shifted(&rb), Some(sc), None);
            }
            c.polygon(&rb, Some(fill), Some(line));
            if t == "BPMNCallActivityView" {
                c.thick_polyline(&closed(&rb), line, 3.0);
            }
            if t == "BPMNTransactionView" {
                // a second border 3 px in, rounded by 8 (BPMNTransactionView.drawObject)
                let inner = round_box(R { x: x1 + 3.0, y: y1 + 3.0, w: b.w - 6.0, h: b.h - 6.0 }, 8.0);
                c.polyline(&closed(&inner), line, false);
            }
            draw_markers(c, &activity_markers(r, v, m), xc.round(), y2, line, fc, size);
            let shown_ty = if t == "BPMNCallActivityView" { r.refd(m, "calledRef").map(|x| r.ty(x).to_string()).unwrap_or_default() } else { r.ty(m).to_string() };
            let inst = if t == "BPMNCallActivityView" { r.refd(m, "calledRef").map(|x| r.bool(x, "instantiate")).unwrap_or(false) } else { r.bool(m, "instantiate") };
            draw_task_mark(c, &shown_ty, inst, b, line, fill);
            draw_wrapped_label(c, r, v, fc);
        }
        "BPMNChoreographyTaskView" | "BPMNSubChoreographyView" => draw_choreography(c, r, v, m, b, fill, line, fc, size),
        "BPMNEventView" => {
            if shadow {
                c.ellipse(p(xc + render::SHADOW_OFFSET, yc + render::SHADOW_OFFSET), b.w / 2.0, b.h / 2.0, Some(sc), None);
            }
            draw_event(c, r, m, b, fill, line);
        }
        "BPMNGatewayView" => {
            let dia = [p(x1, yc), p(xc, y1), p(x2, yc), p(xc, y2)];
            if shadow {
                c.polygon(&shifted(&dia), Some(sc), None);
            }
            c.polygon(&dia, Some(fill), Some(line));
            draw_gateway(c, r, m, b, line);
        }
        "BPMNMessageView" => {
            if shadow {
                c.rect(R { x: x1 + render::SHADOW_OFFSET, y: y1 + render::SHADOW_OFFSET, ..b }, Some(sc), None);
            }
            let (xm, ym) = (xc.round(), yc.round());
            c.rect(b, Some(fill), Some(line));
            seg(c, p(x1, y1), p(xm, ym), line);
            seg(c, p(xm, ym), p(x2, y1), line);
        }
        "BPMNConversationView" => {
            let g = b.w * 0.25;
            let ym = yc.round();
            let hex = [p(x1 + g, y1), p(x2 - g, y1), p(x2, ym), p(x2 - g, y2), p(x1 + g, y2), p(x1, ym)];
            c.polygon(&hex, Some(fill), Some(line));
            if r.ty(m) == "BPMNCallConversation" {
                c.thick_polyline(&closed(&hex), line, 3.0);
            }
            let called = r.refd(m, "calledRef").map(|x| r.kind_of(x, "BPMNCollaboration")).unwrap_or(false);
            if r.ty(m) == "BPMNSubConversation" || (r.ty(m) == "BPMNCallConversation" && called) {
                let xm = xc.round();
                c.rect(R { x: xm - 6.0, y: y2 - 15.0, w: 12.0, h: 12.0 }, None, Some(line));
                seg(c, p(xm - 4.0, y2 - 9.0), p(xm + 4.0, y2 - 9.0), line);
                seg(c, p(xm, y2 - 13.0), p(xm, y2 - 5.0), line);
            }
        }
        "BPMNDataObjectView" | "BPMNDataInputView" | "BPMNDataOutputView" => {
            let fx = x2 - b.w * 0.3;
            let fy = y1 + b.w * 0.3;
            let page = [p(x1, y1), p(fx, y1), p(x2, fy), p(x2, y2), p(x1, y2)];
            if shadow {
                c.polygon(&shifted(&page), Some(sc), None);
            }
            c.polygon(&page, Some(fill), Some(line));
            c.polyline(&[p(fx, y1), p(fx, fy), p(x2, fy), p(fx, y1)], line, false);
            if r.bool(m, "isCollection") {
                let xm = xc.round();
                for dx in [-3.0, 0.0, 3.0] {
                    seg(c, p(xm + dx, y2 - 3.0), p(xm + dx, y2 - 12.0), line);
                }
            }
            if t != "BPMNDataObjectView" {
                // an arrow in the top left corner: hollow for an input, filled for an output
                let w = b.w;
                let (ax1, ay1) = (x1 + 3.0, y1 + 3.0);
                let (ax2, ay2) = (x1 + (w * 0.6).round(), y1 + (w * 0.6).round());
                let ayc = ((ay1 + ay2) / 2.0).round();
                let k = (w * 0.12).round();
                let arrow = [p(ax1, ayc - k), p(ax2 - k, ayc - k), p(ax2 - k, ay1), p(ax2, ayc), p(ax2 - k, ay2), p(ax2 - k, ayc + k), p(ax1, ayc + k)];
                if t == "BPMNDataOutputView" {
                    shape(c, &arrow, Some(line), None);
                } else {
                    shape(c, &arrow, None, Some(line));
                }
            }
        }
        "BPMNDataStoreView" => {
            let (pts, arc) = cylinder(b);
            if shadow {
                c.polygon(&shifted(&pts), Some(sc), None);
            }
            c.polygon(&pts, Some(fill), Some(line));
            c.polyline(&arc, line, false);
            draw_wrapped_label(c, r, v, fc);
        }
        "BPMNTextAnnotationView" => draw_annotation(c, r, v, m, b, line, fc),
        "BPMNGroupView" => {
            c.rect(b, Some(fill), None);
            let pts = [p(x1, y1), p(x2, y1), p(x2, y2), p(x1, y2), p(x1, y1)];
            render::dashes(c, &pts, line, 3.0, 3.0);
        }
        "BPMNPoolView" | "BPMNLaneView" => draw_swimlane(c, r, v, m, b, line, fc),
        _ => {}
    }
    // a floating node's name, below it
    if bpmn::is_floating(&t) {
        if let Some(l) = r.refd(v, "nameLabel") {
            render::draw_label(c, r, l, fc);
        }
    }
}

/// A data store's cylinder: the outline and the front of its top (BPMNDataStoreView.drawObject,
/// elements.js:2500-2528).
fn cylinder(b: R) -> (Vec<P>, Vec<P>) {
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let xm = (x1 + x2) / 2.0;
    let g = (b.h / 8.0).floor();
    let k = 0.5522848;
    let (ox, oy) = (b.w / 2.0 * k, g * k);
    let mut pts = vec![p(x1, y1 + g)];
    cubic(&mut pts, p(x1, y1 + g), p(x1, y1 + g - oy), p(xm - ox, y1), p(xm, y1));
    cubic(&mut pts, p(xm, y1), p(xm + ox, y1), p(x2, y1 + g - oy), p(x2, y1 + g));
    pts.push(p(x2, y2 - g));
    cubic(&mut pts, p(x2, y2 - g), p(x2, y2 - g + oy), p(xm + ox, y2), p(xm, y2));
    cubic(&mut pts, p(xm, y2), p(xm - ox, y2), p(x1, y2 - g + oy), p(x1, y2 - g));
    let mut arc = vec![p(x1, y1 + g)];
    cubic(&mut arc, p(x1, y1 + g), p(x1, y1 + g + oy), p(xm - ox, y1 + 2.0 * g), p(xm, y1 + 2.0 * g));
    cubic(&mut arc, p(xm, y1 + 2.0 * g), p(xm + ox, y1 + 2.0 * g), p(x2, y1 + g + oy), p(x2, y1 + g));
    (pts, arc)
}

/// A choreography: a rounded box with a band per participant, the upper ones from the top and the
/// lower ones from the bottom, the initiating one grey, and the loop marks above the lower bands
/// (BPMNChoreographyActivityView.drawObject, elements.js:1394-1509).
#[allow(clippy::too_many_arguments)]
fn draw_choreography(c: &mut dyn Canvas, r: &Repo, v: Eid, m: Eid, b: R, fill: Rgba, line: Rgba, fc: Rgba, size: f64) {
    let (x1, x2) = (b.x, b.x + b.w);
    let rb = round_box(b, ACTIVITY_ROUND);
    if r.bool(v, "showShadow") {
        c.polygon(&shifted(&rb), Some(shadow_color()), None);
    }
    c.polygon(&rb, Some(fill), None);
    let h = size + 2.0 * PAD;
    let upper = r.ref_list(m, "upperParticipants");
    let lower = r.ref_list(m, "lowerParticipants");
    let grey = render::parse_color(bpmn::INITIATING_FILL, fill);
    if let Some(ip) = r.refd(m, "initiatingParticipant") {
        // the band of the initiating participant, its outer corners rounded
        // a band, its two top or two bottom corners rounded (the round box's corners run clockwise
        // from the top right, 9 points each)
        let band = |top: f64, round_top: bool, round_bottom: bool| {
            let full = round_box(R { x: x1, y: top, w: b.w, h }, ACTIVITY_ROUND);
            if round_top {
                let mut v: Vec<P> = full[27..36].to_vec();
                v.extend_from_slice(&full[0..9]);
                v.extend([p(x2, top + h), p(x1, top + h)]);
                v
            } else if round_bottom {
                let mut v = vec![p(x1, top), p(x2, top)];
                v.extend_from_slice(&full[9..27]);
                v
            } else {
                vec![p(x1, top), p(x2, top), p(x2, top + h), p(x1, top + h)]
            }
        };
        if let Some(ui) = upper.iter().position(|x| *x == ip) {
            let pts = band(b.y + ui as f64 * h, ui == 0, false);
            fill_any(c, &pts, grey);
        } else if let Some(li) = lower.iter().position(|x| *x == ip) {
            let k = lower.len() - li - 1;
            let top = b.y + b.h - (k as f64 + 1.0) * h;
            let pts = band(top, false, li == lower.len() - 1);
            fill_any(c, &pts, grey);
        }
    }
    let mut tp = b.y;
    for q in &upper {
        tp += PAD;
        c.text(&r.name(*q), R { x: x1, y: tp, w: b.w, h: size }, 2, 0, size, fc, false);
        tp += size + PAD;
        seg(c, p(x1, tp), p(x2, tp), line);
    }
    let mut bp = b.y + b.h;
    for q in lower.iter().rev() {
        bp -= PAD + size;
        c.text(&r.name(*q), R { x: x1, y: bp, w: b.w, h: size }, 2, 0, size, fc, false);
        bp -= PAD;
        seg(c, p(x1, bp), p(x2, bp), line);
    }
    c.polyline(&closed(&rb), line, false);
    let mut mk = vec![];
    match r.str(m, "loopType").as_str() {
        "standard" => mk.push("loop"),
        "multiInstanceSequential" => mk.push("seq"),
        "multiInstanceParallel" => mk.push("par"),
        _ => {}
    }
    if r.ty(v) == "BPMNSubChoreographyView" {
        mk.push("sub");
    }
    let bottom = b.y + b.h - h * lower.len() as f64;
    draw_markers(c, &mk, (b.x + b.w / 2.0).round(), bottom, line, fc, size);
    draw_wrapped_label(c, r, v, fc);
}

/// An event: a thin circle to start, a double one in between (dashed when a boundary event does
/// not cancel its activity), a thick one to end, and the mark of its definition, filled when it
/// throws; two definitions or more make a pentagon, or a cross when parallel
/// (BPMNEventView.drawObject, elements.js:1649-2045).
fn draw_event(c: &mut dyn Canvas, r: &Repo, m: Eid, b: R, fill: Rgba, line: Rgba) {
    let t = r.ty(m);
    let (x0, y0, xr, yb) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let ctr = p((x0 + xr) / 2.0, (y0 + yb) / 2.0);
    let (rx, ry) = (b.w / 2.0, b.h / 2.0);
    let parent = r.parent(m);
    let in_sub = parent.map(|x| r.kind_of(x, "BPMNSubProcess") || r.parent(x).map(|y| r.kind_of(y, "BPMNSubProcess")).unwrap_or(false)).unwrap_or(false);
    let dashed_circle = |c: &mut dyn Canvas, rx: f64, ry: f64| render::dashes(c, &ellipse_pts(ctr, rx, ry), line, 3.0, 3.0);
    let gap = 3.0;
    match t {
        "BPMNStartEvent" => {
            c.ellipse(ctr, rx, ry, Some(fill), None);
            if in_sub && !r.bool(m, "isInterrupting") {
                dashed_circle(c, rx, ry);
            } else {
                c.ellipse(ctr, rx, ry, None, Some(line));
            }
        }
        "BPMNIntermediateCatchEvent" | "BPMNIntermediateThrowEvent" => {
            c.ellipse(ctr, rx, ry, Some(fill), Some(line));
            c.ellipse(ctr, rx - gap, ry - gap, None, Some(line));
        }
        "BPMNBoundaryEvent" if r.bool(m, "cancelActivity") => {
            c.ellipse(ctr, rx, ry, Some(fill), Some(line));
            c.ellipse(ctr, rx - gap, ry - gap, None, Some(line));
        }
        "BPMNBoundaryEvent" => {
            c.ellipse(ctr, rx, ry, Some(fill), None);
            dashed_circle(c, rx, ry);
            dashed_circle(c, rx - gap, ry - gap);
        }
        "BPMNEndEvent" => {
            c.ellipse(ctr, rx, ry, Some(fill), None);
            c.thick_polyline(&ellipse_pts(ctr, rx, ry), line, 3.0);
        }
        _ => {
            c.ellipse(ctr, rx, ry, Some(fill), Some(line));
        }
    }
    let defs: Vec<Eid> = r.list(m, "eventDefinitions").into_iter().filter(|d| r.alive(*d)).collect();
    if defs.is_empty() {
        return;
    }
    let throw = matches!(t, "BPMNEndEvent" | "BPMNIntermediateThrowEvent");
    let multiple = defs.len() > 1;
    let g = (b.w / 4.3).round();
    let (x1, y1, x2, y2) = (x0 + g, y0 + g, xr - g, yb - g);
    let (xc, yc) = (ctr.x, ctr.y);
    let w = (x2 - x1).round();
    // a mark: filled with the line colour when the event throws, outlined when it catches
    let mark = |c: &mut dyn Canvas, pts: &[P]| if throw { shape(c, pts, Some(line), Some(line)) } else { shape(c, pts, None, Some(line)) };
    if multiple {
        if r.bool(m, "parallelMultiple") {
            let k = w * 0.15;
            shape(
                c,
                &[p(xc - k, y1), p(xc + k, y1), p(xc + k, yc - k), p(x2, yc - k), p(x2, yc + k), p(xc + k, yc + k), p(xc + k, y2), p(xc - k, y2), p(xc - k, yc + k), p(x1, yc + k), p(x1, yc - k), p(xc - k, yc - k)],
                None,
                Some(line),
            );
        } else {
            let (yt1, yt2, xt1) = ((w * 0.35).round(), (w * 0.1).round(), (w * 0.25).round());
            mark(c, &[p(xc, y1), p(x2, y1 + yt1), p(x2 - xt1, y2 - yt2), p(x1 + xt1, y2 - yt2), p(x1, y1 + yt1)]);
        }
        return;
    }
    let top = y0 + (g * 1.2).round();
    let bot = yb - (g * 1.2).round();
    if has_def(r, m, "BPMNMessageEventDefinition") {
        if throw {
            let k = (w * 0.1).round();
            shape(c, &[p(x1, top), p(x2, top), p(xc, yc - k)], Some(line), None);
            shape(c, &[p(x1, top + k), p(xc, yc), p(x2, top + k), p(x2, bot), p(x1, bot)], Some(line), None);
        } else {
            c.rect(R { x: x1, y: top, w: x2 - x1, h: bot - top }, None, Some(line));
            seg(c, p(x1, top), p(xc, yc), line);
            seg(c, p(xc, yc), p(x2, top), line);
        }
    }
    if has_def(r, m, "BPMNTimerEventDefinition") {
        c.ellipse(ctr, (x2 - x1) / 2.0, (y2 - y1) / 2.0, None, Some(line));
        seg(c, p(xc, yc), p(xc, y0 + (g * 1.3).round()), line);
        seg(c, p(xc, yc), p(xr - (g * 1.3).round(), yc), line);
    }
    if has_def(r, m, "BPMNErrorEventDefinition") {
        let (a, h5, t1) = ((w * 0.33).round(), (w * 0.5).round(), (w * 0.1).round());
        mark(c, &[p(x1, y2 - t1), p(x1 + a, y1), p(x2 - a, y1 + h5), p(x2, y1 + t1), p(x2 - a, y2), p(x1 + a, y1 + h5)]);
    }
    if has_def(r, m, "BPMNEscalationEventDefinition") {
        let (a, h6) = ((w * 0.2).round(), (w * 0.6).round());
        mark(c, &[p(xc, y1), p(x1 + a, y2), p(xc, y1 + h6), p(x2 - a, y2)]);
    }
    if has_def(r, m, "BPMNCancelEventDefinition") {
        let k = w * 0.15;
        mark(
            c,
            &[p(x1, y1 + k), p(x1 + k, y1), p(xc, yc - k * 0.9), p(x2 - k, y1), p(x2, y1 + k), p(xc + k * 0.9, yc), p(x2, y2 - k), p(x2 - k, y2), p(xc, yc + k * 0.9), p(x1 + k, y2), p(x1, y2 - k), p(xc - k * 0.9, yc)],
        );
    }
    if has_def(r, m, "BPMNCompensateEventDefinition") {
        let s = w * 0.1;
        mark(c, &[p(x1 - s, yc), p(xc - s, y1), p(xc - s, y2)]);
        mark(c, &[p(xc - s, yc), p(x2 - s, y1), p(x2 - s, y2)]);
    }
    if has_def(r, m, "BPMNConditionalEventDefinition") {
        let s = w * 0.1;
        let k = (w * 0.3).round();
        c.rect(R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }, None, Some(line));
        seg(c, p(x1 + s, y1 + k), p(x2 - s, y1 + k), line);
        seg(c, p(x1 + s, yc), p(x2 - s, yc), line);
        seg(c, p(x1 + s, y2 - k), p(x2 - s, y2 - k), line);
    }
    if has_def(r, m, "BPMNLinkEventDefinition") {
        let k = (w * 0.3).round();
        mark(c, &[p(x1, yc - k), p(x2 - k, yc - k), p(x2 - k, y1), p(x2, yc), p(x2 - k, y2), p(x2 - k, yc + k), p(x1, yc + k)]);
    }
    if has_def(r, m, "BPMNSignalEventDefinition") {
        let k = (w * 0.1).round();
        mark(c, &[p(xc, y1), p(x2, y2 - k), p(x1, y2 - k)]);
    }
    if has_def(r, m, "BPMNTerminateEventDefinition") {
        c.ellipse(ctr, (x2 - x1) / 2.0, (y2 - y1) / 2.0, Some(line), Some(line));
    }
}

/// A gateway's mark in its diamond: none for an exclusive one, a thick circle for an inclusive
/// one, a thick plus for a parallel one, and a star for a complex one; an event-based one shows a
/// pentagon in two circles (one when it instantiates; a cross when it instantiates in parallel)
/// (BPMNGatewayView.drawObject, elements.js:2233-2321).
fn draw_gateway(c: &mut dyn Canvas, r: &Repo, m: Eid, b: R, line: Rgba) {
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    let g = b.w / 4.0;
    let (x1, y1, x2, y2) = (x - g, y - g, x + g, y + g);
    match r.ty(m) {
        "BPMNInclusiveGateway" => c.thick_polyline(&ellipse_pts(p(x, y), g, g), line, 3.0),
        "BPMNParallelGateway" => {
            c.thick_polyline(&[p(x, y1), p(x, y2)], line, 3.0);
            c.thick_polyline(&[p(x1, y), p(x2, y)], line, 3.0);
        }
        "BPMNComplexGateway" => {
            c.thick_polyline(&[p(x, y1), p(x, y2)], line, 3.0);
            c.thick_polyline(&[p(x1, y), p(x2, y)], line, 3.0);
            let k = g * 0.3;
            c.thick_polyline(&[p(x1 + k, y1 + k), p(x2 - k, y2 - k)], line, 3.0);
            c.thick_polyline(&[p(x2 - k, y1 + k), p(x1 + k, y2 - k)], line, 3.0);
        }
        "BPMNEventBasedGateway" => {
            c.ellipse(p(x, y), g * 1.25, g * 1.25, None, Some(line));
            let inst = r.bool(m, "instantiate");
            if !inst {
                c.ellipse(p(x, y), g, g, None, Some(line));
            }
            if inst && r.str(m, "eventGatewayType") == "parallel" {
                let k = g * 2.0 * 0.15;
                shape(
                    c,
                    &[p(x - k, y1), p(x + k, y1), p(x + k, y - k), p(x2, y - k), p(x2, y + k), p(x + k, y + k), p(x + k, y2), p(x - k, y2), p(x - k, y + k), p(x1, y + k), p(x1, y - k), p(x - k, y - k)],
                    None,
                    Some(line),
                );
            } else {
                let (yt1, yt2, xt1) = ((g * 2.0 * 0.35).round(), (g * 2.0 * 0.1).round(), (g * 2.0 * 0.25).round());
                shape(c, &[p(x, y1), p(x2, y1 + yt1), p(x2 - xt1, y2 - yt2), p(x1 + xt1, y2 - yt2), p(x1, y1 + yt1)], None, Some(line));
            }
        }
        _ => {}
    }
}

/// A text annotation: its lines from the top left, 5 px in, and a bracket on its left
/// (BPMNTextAnnotationView.drawObject, elements.js:2618-2662).
fn draw_annotation(c: &mut dyn Canvas, r: &Repo, v: Eid, m: Eid, b: R, line: Rgba, fc: Rgba) {
    let text = r.str(m, "text");
    let (face, size) = views::view_font(r, v);
    if !text.is_empty() {
        let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
        let align = r.val(v, "horzAlign").and_then(|x| x.as_i64()).unwrap_or(0);
        let ms = views::Scaled { ms: &*c, face: face.clone(), size };
        let rows: Vec<Vec<String>> = text.split('\n').map(|l| if wrap && !l.is_empty() { views::wrap_text(&ms, l, (b.w - 1.0 - 2.0 * PAD).max(1.0), 0) } else { vec![l.to_string()] }).collect();
        let total: f64 = rows.iter().map(|ls| ls.len() as f64 * size + 2.0).sum();
        let mut y = match r.val(v, "vertAlign").and_then(|x| x.as_i64()).unwrap_or(3) {
            5 => b.y + ((b.h - total) / 2.0).round(),
            4 => b.y + b.h - PAD - total,
            _ => b.y + PAD,
        };
        for ls in rows {
            for l in &ls {
                c.text(l, R { x: b.x + PAD, y, w: b.w - 2.0 * PAD, h: size }, align, 0, size, fc, false);
                y += size;
            }
            y += 2.0;
        }
    }
    let (x1, y1, y2) = (b.x, b.y, b.y + b.h);
    c.polyline(&[p(x1 + 10.0, y1), p(x1, y1), p(x1, y2), p(x1 + 10.0, y2)], line, false);
}

/// A pool: its border, the line between its name and its lanes, and three bars at the bottom when
/// it has several participants; a lane: the line along its bottom (its right side when vertical)
/// (BPMNPoolView.drawObject and BPMNLaneView.drawObject, elements.js:2872-2896, 2993-3001).
fn draw_swimlane(c: &mut dyn Canvas, r: &Repo, v: Eid, m: Eid, b: R, line: Rgba, fc: Rgba) {
    let (x1, y1, x2, y2) = (b.x, b.y, b.x + b.w, b.y + b.h);
    let vert = vertical(r, v);
    let l = r.refd(v, "nameLabel");
    let lb = l.map(|l| views::node_rect(r, l)).unwrap_or(R { x: x1 + PAD, y: y1 + PAD, w: 0.0, h: 0.0 });
    if r.ty(v) == "BPMNPoolView" {
        c.rect(b, None, Some(line));
        if vert {
            let y = lb.y + lb.h + PAD;
            seg(c, p(x1, y), p(x2, y), line);
        } else {
            let x = lb.x + lb.w + PAD;
            seg(c, p(x, y1), p(x, y2), line);
        }
        if r.f(m, "maximum") > 1.0 || r.val(m, "maximum").and_then(|x| x.as_i64()).unwrap_or(1) > 1 {
            let mx = ((x1 + x2) / 2.0).round();
            for dx in [0.0, -4.0, 4.0] {
                seg(c, p(mx + dx, y2 - 14.0), p(mx + dx, y2 - 4.0), line);
            }
        }
    } else if vert {
        seg(c, p(x2, y1), p(x2, y2), line);
    } else {
        seg(c, p(x1, y2), p(x2, y2), line);
    }
    let name = r.name(m);
    if !name.is_empty() {
        let (_, size) = views::view_font(r, v);
        let style = r.str(v, "font").split(';').nth(2).and_then(|x| x.parse::<u8>().ok()).unwrap_or(0);
        if vert {
            c.text(&name, R { x: lb.x, y: lb.y, w: lb.w, h: size }, 2, style, size, fc, false);
        } else {
            c.text_up(&name, lb, style, size, fc);
        }
    }
}

// ------------------------------------------------------------------ lines

/// Draw a BPMN line: its pattern, width and ends (the constructors and update() of the BPMN edge
/// views, elements.js:935-968, 2180-2200, 2675-2775; the end shapes of EdgeView.drawLineEnd,
/// core.js:2730-2930).
pub fn draw_edge(c: &mut dyn Canvas, r: &Repo, e: Eid, pts: &[P]) {
    let line = render::parse_color(&r.str(e, "lineColor"), render::BLACK);
    let style = r.val(e, "lineStyle").and_then(|v| v.as_i64()).unwrap_or(0);
    let drawn = views::drawn_points(pts, style);
    let m = r.refd(e, "model");
    let ty = r.ty(e);
    // (tail, head) ends: "flat", "stick", "solid", "blank", "diamond", "circle", "slash"
    let (mut tail, mut head) = ("flat", "flat");
    match ty {
        "BPMNSequenceFlowView" => {
            head = "solid";
            if let Some(m) = m {
                let src = r.refd(m, "source");
                if src.map(|s| r.kind_of(s, "BPMNGateway") && r.refd(s, "default") == Some(m)).unwrap_or(false) {
                    tail = "slash";
                }
                if src.map(|s| r.kind_of(s, "BPMNActivity")).unwrap_or(false) && !r.str(m, "condition").is_empty() {
                    tail = "diamond";
                }
            }
            c.polyline(&drawn, line, false);
        }
        "BPMNMessageFlowView" => {
            tail = "circle";
            head = "blank";
            render::dashes(c, &drawn, line, 6.0, 4.0);
        }
        "BPMNConversationLinkView" => c.thick_polyline(&drawn, line, 3.0),
        "BPMNAssociationView" => {
            match m.map(|m| r.str(m, "associationDirection")).as_deref() {
                Some("one") => head = "stick",
                Some("both") => {
                    head = "stick";
                    tail = "stick";
                }
                _ => {}
            }
            c.polyline(&drawn, line, true);
        }
        "BPMNDataAssociationView" => {
            head = "stick";
            c.polyline(&drawn, line, true);
        }
        _ => c.polyline(&drawn, line, true),
    }
    let n = pts.len();
    end(c, pts[n - 1], pts[n - 2], head, line);
    end(c, pts[0], pts[1], tail, line);
    let fc = render::parse_color(&r.str(e, "fontColor"), render::BLACK);
    if let Some(l) = r.refd(e, "nameLabel") {
        render::draw_label(c, r, l, fc);
    }
}

/// One end of a line, pointing from `p1` to the tip `p0`.
fn end(c: &mut dyn Canvas, p0: P, p1: P, kind: &str, line: Rgba) {
    let th = (p1.y - p0.y).atan2(p1.x - p0.x);
    let at = |len: f64, a: f64| p(p0.x + len * a.cos(), p0.y + len * a.sin());
    let half = std::f64::consts::PI / 8.0;
    let (a1, a2) = (at(render::ARROW_LENGTH, th - half), at(render::ARROW_LENGTH, th + half));
    match kind {
        "stick" => c.polyline(&[a1, p0, a2], line, false),
        "solid" => c.polygon(&[a1, p0, a2], Some(line), Some(line)),
        "blank" => c.polygon(&[a1, p0, a2], Some(render::WHITE), Some(line)),
        "diamond" => c.polygon(&[a1, p0, a2, at(2.0 * render::ARROW_LENGTH, th)], Some(render::WHITE), Some(line)),
        // a small white circle 3 across, 3 px in
        "circle" => c.ellipse(at(3.0, th), 3.0, 3.0, Some(render::WHITE), Some(line)),
        // a slash across the line, 7 and 14 px in
        "slash" => {
            let (q8, q11) = (at(7.0, th), at(14.0, th));
            let side = |q: P, a: f64| p(q.x + 7.0 * a.cos(), q.y + 7.0 * a.sin());
            c.polyline(&[side(q8, th - std::f64::consts::FRAC_PI_2), side(q11, th + std::f64::consts::FRAC_PI_2)], line, false);
        }
        _ => {}
    }
}

// ------------------------------------------------------------------ icons

/// The small icon of a BPMN element or tool, drawn in the explorer and the toolbox (simple shapes
/// after the notation; not the original's artwork). False when the key is not a BPMN one.
pub fn icon(painter: &egui::Painter, rect: egui::Rect, key: &str, model: egui::Color32, diagram: egui::Color32, rel: egui::Color32) -> bool {
    use egui::{Pos2, Shape, Stroke, Vec2};
    if !key.starts_with("BPMN") {
        return false;
    }
    let o = rect.min;
    let s = rect.width() / 16.0;
    let at = |x: f32, y: f32| Pos2::new(o.x + x * s, o.y + y * s);
    let st = |w: f32, c: egui::Color32| Stroke::new(w, c);
    let line = |pts: &[(f32, f32)], c: egui::Color32| {
        painter.add(Shape::line(pts.iter().map(|(x, y)| at(*x, *y)).collect(), st(1.2, c)));
    };
    let thick = |pts: &[(f32, f32)], c: egui::Color32| {
        painter.add(Shape::line(pts.iter().map(|(x, y)| at(*x, *y)).collect(), st(2.2, c)));
    };
    let dashed = |pts: &[(f32, f32)], c: egui::Color32| {
        let ps: Vec<Pos2> = pts.iter().map(|(x, y)| at(*x, *y)).collect();
        for sh in Shape::dashed_line(&ps, st(1.2, c), 2.0 * s, 1.5 * s) {
            painter.add(sh);
        }
    };
    let circle = |x: f32, y: f32, rad: f32, w: f32, c: egui::Color32| {
        painter.circle_stroke(at(x, y), rad * s, st(w, c));
    };
    let rounded = |x: f32, y: f32, w: f32, h: f32, c: egui::Color32| {
        painter.rect_stroke(egui::Rect::from_min_size(at(x, y), Vec2::new(w * s, h * s)), 3.0 * s, st(1.2, c), egui::StrokeKind::Middle);
    };
    let rect_at = |x: f32, y: f32, w: f32, h: f32, c: egui::Color32| {
        painter.rect_stroke(egui::Rect::from_min_size(at(x, y), Vec2::new(w * s, h * s)), 0.0, st(1.2, c), egui::StrokeKind::Middle);
    };
    let diamond = |c: egui::Color32| line(&[(8.0, 1.5), (14.5, 8.0), (8.0, 14.5), (1.5, 8.0), (8.0, 1.5)], c);
    let y = model;
    match key {
        "BPMNDiagram" => {
            rect_at(1.5, 1.5, 13.0, 13.0, diagram);
            rounded(3.5, 5.5, 5.0, 5.0, diagram);
            line(&[(8.5, 8.0), (11.0, 8.0)], diagram);
            line(&[(13.0, 8.0), (11.0, 6.0), (11.0, 10.0), (13.0, 8.0)], diagram);
        }
        "BPMNProcess" | "BPMNCollaboration" | "BPMNChoreography" | "BPMNGlobalConversation" => {
            line(&[(1.5, 3.5), (6.5, 3.5), (7.5, 5.0), (14.5, 5.0), (14.5, 13.5), (1.5, 13.5), (1.5, 3.5)], y);
            match key {
                "BPMNProcess" => line(&[(4.5, 9.0), (11.5, 9.0)], y),
                "BPMNCollaboration" => {
                    line(&[(4.5, 7.5), (11.5, 7.5)], y);
                    line(&[(4.5, 10.5), (11.5, 10.5)], y);
                }
                "BPMNChoreography" => rounded(5.0, 7.0, 6.0, 5.0, y),
                _ => line(&[(6.0, 7.0), (10.0, 7.0), (11.5, 9.0), (10.0, 11.0), (6.0, 11.0), (4.5, 9.0), (6.0, 7.0)], y),
            }
        }
        "BPMNParticipant" | "BPMNParticipantVert" => {
            rect_at(1.5, 3.5, 13.0, 9.0, y);
            if key == "BPMNParticipant" {
                line(&[(4.5, 3.5), (4.5, 12.5)], y);
            } else {
                rect_at(3.5, 1.5, 9.0, 13.0, y);
            }
        }
        "BPMNLane" | "BPMNLaneVert" => {
            if key == "BPMNLane" {
                rect_at(1.5, 3.5, 13.0, 9.0, y);
                line(&[(4.5, 3.5), (4.5, 12.5)], y);
                line(&[(4.5, 8.0), (14.5, 8.0)], y);
            } else {
                rect_at(3.5, 1.5, 9.0, 13.0, y);
                line(&[(3.5, 4.5), (12.5, 4.5)], y);
                line(&[(8.0, 4.5), (8.0, 14.5)], y);
            }
        }
        "BPMNTask" | "BPMNSendTask" | "BPMNReceiveTask" | "BPMNServiceTask" | "BPMNUserTask" | "BPMNManualTask" | "BPMNBusinessRuleTask" | "BPMNScriptTask" | "BPMNCallActivity" | "BPMNSubProcess"
        | "BPMNAdHocSubProcess" | "BPMNTransaction" => {
            rounded(1.5, 3.0, 13.0, 10.0, y);
            match key {
                "BPMNSendTask" | "BPMNReceiveTask" => {
                    rect_at(3.0, 4.5, 5.0, 3.5, y);
                    line(&[(3.0, 4.5), (5.5, 6.5), (8.0, 4.5)], y);
                }
                "BPMNServiceTask" => circle(5.0, 6.5, 1.8, 1.2, y),
                "BPMNUserTask" => {
                    circle(5.0, 5.5, 1.2, 1.2, y);
                    line(&[(3.0, 9.0), (3.5, 7.5), (6.5, 7.5), (7.0, 9.0)], y);
                }
                "BPMNManualTask" => line(&[(3.0, 8.0), (3.0, 6.0), (5.0, 4.5), (8.0, 4.5)], y),
                "BPMNBusinessRuleTask" => {
                    rect_at(3.0, 4.5, 5.0, 4.0, y);
                    line(&[(3.0, 6.0), (8.0, 6.0)], y);
                }
                "BPMNScriptTask" => {
                    line(&[(4.0, 4.5), (8.0, 4.5), (7.0, 6.5), (8.0, 8.5), (4.0, 8.5), (5.0, 6.5), (4.0, 4.5)], y);
                }
                "BPMNCallActivity" => {
                    painter.rect_stroke(egui::Rect::from_min_size(at(1.5, 3.0), Vec2::new(13.0 * s, 10.0 * s)), 3.0 * s, st(2.2, y), egui::StrokeKind::Middle);
                }
                "BPMNSubProcess" | "BPMNAdHocSubProcess" | "BPMNTransaction" => {
                    rect_at(6.0, 8.0, 4.0, 4.0, y);
                    line(&[(8.0, 8.5), (8.0, 11.5)], y);
                    line(&[(6.5, 10.0), (9.5, 10.0)], y);
                    if key == "BPMNTransaction" {
                        rounded(3.0, 4.5, 10.0, 7.0, y);
                    }
                    if key == "BPMNAdHocSubProcess" {
                        line(&[(11.0, 10.5), (11.7, 9.8), (12.3, 10.5), (13.0, 9.8)], y);
                    }
                }
                _ => {}
            }
        }
        "BPMNChoreographyTask" | "BPMNSubChoreography" => {
            rounded(1.5, 2.0, 13.0, 12.0, y);
            line(&[(1.5, 5.0), (14.5, 5.0)], y);
            line(&[(1.5, 11.0), (14.5, 11.0)], y);
            if key == "BPMNSubChoreography" {
                line(&[(8.0, 6.5), (8.0, 9.5)], y);
                line(&[(6.5, 8.0), (9.5, 8.0)], y);
            }
        }
        "BPMNStartEvent" | "BPMNEvent" | "BPMNCatchEvent" => circle(8.0, 8.0, 6.0, 1.2, y),
        "BPMNIntermediateEvent" | "BPMNIntermediateThrowEvent" | "BPMNIntermediateCatchEvent" | "BPMNThrowEvent" | "BPMNImplicitThrowEvent" => {
            circle(8.0, 8.0, 6.0, 1.2, y);
            circle(8.0, 8.0, 4.3, 1.2, y);
        }
        "BPMNBoundaryEvent" => {
            let ring = |rad: f32| -> Vec<(f32, f32)> { (0..=24).map(|i| i as f32 / 24.0 * std::f32::consts::TAU).map(|a| (8.0 + rad * a.cos(), 8.0 + rad * a.sin())).collect() };
            dashed(&ring(6.0), y);
            dashed(&ring(4.3), y);
        }
        "BPMNEndEvent" => circle(8.0, 8.0, 5.8, 2.4, y),
        "BPMNGateway" | "BPMNExclusiveGateway" => diamond(y),
        "BPMNInclusiveGateway" => {
            diamond(y);
            circle(8.0, 8.0, 2.8, 1.6, y);
        }
        "BPMNParallelGateway" | "BPMNComplexGateway" => {
            diamond(y);
            thick(&[(8.0, 4.5), (8.0, 11.5)], y);
            thick(&[(4.5, 8.0), (11.5, 8.0)], y);
            if key == "BPMNComplexGateway" {
                line(&[(5.8, 5.8), (10.2, 10.2)], y);
                line(&[(10.2, 5.8), (5.8, 10.2)], y);
            }
        }
        "BPMNEventBasedGateway" => {
            diamond(y);
            circle(8.0, 8.0, 3.5, 1.0, y);
            line(&[(8.0, 6.0), (10.0, 7.5), (9.2, 10.0), (6.8, 10.0), (6.0, 7.5), (8.0, 6.0)], y);
        }
        "BPMNDataObject" | "BPMNDataInput" | "BPMNDataOutput" => {
            line(&[(3.0, 1.5), (10.0, 1.5), (13.0, 4.5), (13.0, 14.5), (3.0, 14.5), (3.0, 1.5)], y);
            line(&[(10.0, 1.5), (10.0, 4.5), (13.0, 4.5)], y);
            if key != "BPMNDataObject" {
                let pts = [(4.5, 5.5), (7.0, 5.5), (7.0, 4.0), (9.5, 6.5), (7.0, 9.0), (7.0, 7.5), (4.5, 7.5), (4.5, 5.5)];
                if key == "BPMNDataOutput" {
                    painter.add(Shape::convex_polygon(pts.iter().map(|(a, b)| at(*a, *b)).collect(), y, Stroke::NONE));
                }
                line(&pts, y);
            }
        }
        "BPMNDataStore" => {
            painter.add(Shape::ellipse_stroke(at(8.0, 4.0), Vec2::new(6.0 * s, 2.0 * s), st(1.2, y)));
            line(&[(2.0, 4.0), (2.0, 12.0)], y);
            line(&[(14.0, 4.0), (14.0, 12.0)], y);
            line(&[(2.0, 12.0), (4.0, 13.5), (8.0, 14.0), (12.0, 13.5), (14.0, 12.0)], y);
        }
        "BPMNMessage" => {
            rect_at(1.5, 4.0, 13.0, 8.0, y);
            line(&[(1.5, 4.0), (8.0, 8.5), (14.5, 4.0)], y);
        }
        "BPMNConversation" | "BPMNSubConversation" | "BPMNCallConversation" => {
            let hex = [(4.5, 2.5), (11.5, 2.5), (15.0, 8.0), (11.5, 13.5), (4.5, 13.5), (1.0, 8.0), (4.5, 2.5)];
            if key == "BPMNCallConversation" {
                thick(&hex, y);
            } else {
                line(&hex, y);
            }
            if key == "BPMNSubConversation" {
                line(&[(8.0, 6.0), (8.0, 10.0)], y);
                line(&[(6.0, 8.0), (10.0, 8.0)], y);
            }
        }
        "BPMNTextAnnotation" => {
            line(&[(6.0, 2.0), (2.5, 2.0), (2.5, 14.0), (6.0, 14.0)], y);
            line(&[(5.0, 6.0), (13.0, 6.0)], y);
            line(&[(5.0, 10.0), (11.0, 10.0)], y);
        }
        "BPMNGroup" => {
            painter.rect_stroke(egui::Rect::from_min_size(at(1.5, 2.5), Vec2::new(13.0 * s, 11.0 * s)), 2.0 * s, st(1.0, y), egui::StrokeKind::Middle);
            dashed(&[(1.5, 2.5), (14.5, 2.5), (14.5, 13.5), (1.5, 13.5), (1.5, 2.5)], y);
        }
        "BPMNSequenceFlow" => {
            line(&[(2.0, 14.0), (13.0, 3.0)], rel);
            painter.add(Shape::convex_polygon(vec![at(14.0, 2.0), at(9.5, 3.8), at(12.2, 6.5)], rel, Stroke::NONE));
        }
        "BPMNMessageFlow" => {
            dashed(&[(4.0, 12.0), (13.0, 3.0)], rel);
            circle(3.0, 13.0, 1.5, 1.0, rel);
            line(&[(14.0, 2.0), (9.5, 3.8), (12.2, 6.5), (14.0, 2.0)], rel);
        }
        "BPMNAssociation" | "BPMNMessageLink" => dashed(&[(2.0, 14.0), (14.0, 2.0)], rel),
        "BPMNDataAssociation" => {
            dashed(&[(2.0, 14.0), (14.0, 2.0)], rel);
            line(&[(9.5, 3.0), (14.0, 2.0), (13.0, 6.5)], rel);
        }
        "BPMNConversationLink" => thick(&[(2.0, 14.0), (14.0, 2.0)], rel),
        k if k.ends_with("EventDefinition") => {
            circle(8.0, 8.0, 6.0, 1.2, y);
            match k {
                "BPMNMessageEventDefinition" => {
                    rect_at(4.5, 5.5, 7.0, 5.0, y);
                    line(&[(4.5, 5.5), (8.0, 8.0), (11.5, 5.5)], y);
                }
                "BPMNTimerEventDefinition" => {
                    circle(8.0, 8.0, 3.5, 1.0, y);
                    line(&[(8.0, 8.0), (8.0, 5.5), (8.0, 8.0), (10.0, 8.0)], y);
                }
                "BPMNErrorEventDefinition" => line(&[(4.5, 11.0), (6.5, 4.5), (9.0, 8.5), (11.5, 5.0), (9.5, 11.5), (7.0, 7.5), (4.5, 11.0)], y),
                "BPMNEscalationEventDefinition" => line(&[(8.0, 4.0), (11.0, 12.0), (8.0, 9.0), (5.0, 12.0), (8.0, 4.0)], y),
                "BPMNCancelEventDefinition" => {
                    thick(&[(5.0, 5.0), (11.0, 11.0)], y);
                    thick(&[(11.0, 5.0), (5.0, 11.0)], y);
                }
                "BPMNCompensateEventDefinition" => {
                    line(&[(4.0, 8.0), (8.0, 5.0), (8.0, 11.0), (4.0, 8.0)], y);
                    line(&[(8.0, 8.0), (12.0, 5.0), (12.0, 11.0), (8.0, 8.0)], y);
                }
                "BPMNConditionalEventDefinition" => {
                    rect_at(5.0, 4.5, 6.0, 7.0, y);
                    line(&[(6.0, 6.5), (10.0, 6.5)], y);
                    line(&[(6.0, 9.5), (10.0, 9.5)], y);
                }
                "BPMNLinkEventDefinition" => line(&[(4.5, 6.5), (8.5, 6.5), (8.5, 4.5), (12.0, 8.0), (8.5, 11.5), (8.5, 9.5), (4.5, 9.5), (4.5, 6.5)], y),
                "BPMNSignalEventDefinition" => line(&[(8.0, 4.0), (12.0, 11.0), (4.0, 11.0), (8.0, 4.0)], y),
                "BPMNTerminateEventDefinition" => {
                    painter.circle_filled(at(8.0, 8.0), 3.5 * s, y);
                }
                _ => {}
            }
        }
        _ => {
            rounded(2.0, 3.0, 12.0, 10.0, y);
        }
    }
    true
}
