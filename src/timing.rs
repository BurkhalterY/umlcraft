//! Timing diagrams: a frame holding lifelines stacked down it, each with its states or conditions
//! stacked inside, the time segments of each state lined up left to right from a common start, time
//! ticks along the frame's bottom, and messages between time segments (UMLTimingDiagram and its views,
//! elements.js:207-210, 13374-13822; timingDiagramFn, lifelineFn, timingStateFn, timeSegmentFn,
//! timeTickFn and messageFn, uml-factory.js:374-434, 1105-1180, 1325-1350, 1559-1700).

use crate::app::App;
use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::json;

const STATE_MIN_H: f64 = 25.0;
const LIFELINE_TOP: f64 = 25.0;
const LIFELINE_BOTTOM: f64 = 25.0;
pub const SEGMENT_H: f64 = STATE_MIN_H;
const SEGMENT_MIN_W: f64 = 8.0;
const TICK_H: f64 = 11.0;
const TICK_MIN: f64 = 9.0;
/// A new lifeline is at least 200 wide (lifelineFn, uml-factory.js:1147-1150).
pub const LIFELINE_MIN_W: f64 = 200.0;
const PAD: f64 = 5.0;

fn rect(r: &Repo, v: Eid) -> R {
    views::node_rect(r, v)
}

fn set_rect(r: &mut Repo, v: Eid, b: R) {
    for (k, x) in [("left", b.x), ("top", b.y), ("width", b.w), ("height", b.h)] {
        r.set(v, k, json!(x));
    }
}

/// The views that `v` contains, of a type.
fn inside(r: &Repo, v: Eid, ty: &str) -> Vec<Eid> {
    let Some(d) = r.parent(v) else { return vec![] };
    r.list(d, "ownedViews").into_iter().filter(|x| r.alive(*x) && r.ty(*x) == ty && r.refd(*x, "containerView") == Some(v)).collect()
}

fn contain(r: &mut Repo, host: Eid, v: Eid) {
    r.set_ref(v, "containerView", Some(host));
    let mut a = match r.raw(host, "containedViews") {
        Some(serde_json::Value::Array(a)) => a.clone(),
        _ => vec![],
    };
    a.push(r.ref_to(v));
    r.set_raw(host, "containedViews", json!(a));
}

fn label(r: &mut Repo, v: Eid, model: Option<Eid>, align: i64) -> Eid {
    let mut props = vec![("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(align)), ("height", json!(views::LABEL_H))];
    if let Some(m) = model {
        props.insert(0, ("model", r.ref_to(m)));
    }
    let l = r.create("LabelView", &props, Some((v, "subViews")));
    r.set_ref(v, "nameLabel", Some(l));
    l
}

/// The frame a new timing diagram starts with, 695 × 595 at (5, 5) (_addTimingFrame, uml-factory.js:374-380).
pub fn create_frame(r: &mut Repo, d: Eid) -> Eid {
    views::create_frame_view_of(r, "UMLTimingFrameView", d, Some(d), R { x: 5.0, y: 5.0, w: 695.0, h: 595.0 })
}

/// A lifeline's view in a frame, at least 200 wide, its name written upwards on the left.
pub fn create_lifeline_view(r: &mut Repo, d: Eid, model: Eid, frame: Eid, rect: R) -> Eid {
    let v = r.create(
        "UMLTimingLifelineView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w.max(LIFELINE_MIN_W))), ("height", json!(rect.h))],
        Some((d, "ownedViews")),
    );
    label(r, v, Some(model), 2);
    contain(r, frame, v);
    v
}

/// A state or condition's view in a lifeline view (UMLTimingStateView).
pub fn create_state_view(r: &mut Repo, d: Eid, model: Eid, lifeline: Eid, at: P) -> Eid {
    let v = r.create(
        "UMLTimingStateView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(at.x)), ("top", json!(at.y)), ("width", json!(0)), ("height", json!(STATE_MIN_H))],
        Some((d, "ownedViews")),
    );
    label(r, v, Some(model), 1);
    contain(r, lifeline, v);
    v
}

/// A time segment's view on a state: 25 high, at least `min_w` wide when made.
pub fn create_segment_view(r: &mut Repo, d: Eid, model: Eid, state: Eid, b: R, min_w: f64) -> Eid {
    let v = r.create(
        "UMLTimeSegmentView",
        &[("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(b.x)), ("top", json!(b.y)), ("width", json!(b.w.max(min_w))), ("height", json!(SEGMENT_H))],
        Some((d, "ownedViews")),
    );
    contain(r, state, v);
    v
}

/// A time tick on a frame (a view without a model).
pub fn create_tick_view(r: &mut Repo, d: Eid, frame: Eid, b: R, min_w: f64) -> Eid {
    let v = r.create(
        "UMLTimeTickView",
        &[("font", json!(views::FONT)), ("parentStyle", json!(false)), ("left", json!(b.x)), ("top", json!(b.y)), ("width", json!(b.w.max(min_w) - 1.0)), ("height", json!(TICK_H)), ("text", json!(""))],
        Some((d, "ownedViews")),
    );
    contain(r, frame, v);
    v
}

/// A message between two time segments, its ends at their middle heights.
pub fn create_message_view(r: &mut Repo, d: Eid, model: Eid, tail: Eid, head: Eid, from: P, to: P) -> Eid {
    let e = r.create(
        "UMLTimingMessageView",
        &[
            ("model", r.ref_to(model)), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("head", r.ref_to(head)), ("tail", r.ref_to(tail)),
            ("lineStyle", json!(1)), ("points", json!(views::format_points(&[from, to]))),
        ],
        Some((d, "ownedViews")),
    );
    let half = std::f64::consts::FRAC_PI_2;
    for (field, a, dist) in [("nameLabel", half, 15.0), ("stereotypeLabel", half, 30.0), ("propertyLabel", -half, 15.0)] {
        let l = views::edge_label(r, e, Some(model), a, dist, 1, false);
        r.set_ref(e, field, Some(l));
    }
    e
}

/// Lay out every timing frame of the diagram and what it holds (arrangeObject of the frame, lifeline,
/// state and segment views).
pub fn layout(r: &mut Repo, ms: &dyn Measure, d: Eid) {
    let lh = ms.size();
    let frames: Vec<Eid> = r.list(d, "ownedViews").into_iter().filter(|v| r.alive(*v) && r.ty(*v) == "UMLTimingFrameView").collect();
    for f in frames {
        let lifelines = inside(r, f, "UMLTimingLifelineView");
        // the states' names: where the time line starts
        let mut start = r.f(f, "left");
        for l in &lifelines {
            for s in inside(r, *l, "UMLTimingStateView") {
                let name = r.refd(s, "model").map(|m| r.name(m)).unwrap_or_default();
                let w = ms.width(&name, 0).ceil();
                if let Some(nl) = r.refd(s, "nameLabel") {
                    r.set(nl, "text", json!(name));
                }
                start = start.max(r.f(s, "left") + w + PAD);
            }
        }
        let fb = rect(r, f);
        let mut y = r.refd(f, "frameTypeLabel").map(|t| r.f(t, "top") + r.f(t, "height")).unwrap_or(fb.y + PAD + lh) + PAD;
        let mut ls = lifelines.clone();
        ls.sort_by(|a, b| r.f(*a, "top").total_cmp(&r.f(*b, "top")));
        for l in ls {
            r.set(l, "top", json!(y));
            layout_lifeline(r, ms, l, f, start);
            y += r.f(l, "height");
        }
        // time ticks along the bottom, left to right from the start
        let mut ticks = inside(r, f, "UMLTimeTickView");
        ticks.sort_by(|a, b| r.f(*a, "left").total_cmp(&r.f(*b, "left")));
        let mut x = start;
        let fb = rect(r, f);
        for t in ticks {
            let w = r.f(t, "width").max(TICK_MIN);
            set_rect(r, t, R { x, y: fb.y + fb.h - 5.0, w, h: TICK_H });
            x += w;
        }
    }
}

fn layout_lifeline(r: &mut Repo, ms: &dyn Measure, v: Eid, frame: Eid, start: f64) {
    let lh = ms.size();
    let text = r.refd(v, "model").map(|m| views::box_name(r, m)).unwrap_or_default();
    let tw = ms.width(&text, 1).ceil();
    let states = inside(r, v, "UMLTimingStateView");
    let sh: f64 = states.iter().map(|s| r.f(*s, "height").max(STATE_MIN_H)).sum();
    let h = r.f(v, "height").max(PAD + tw + PAD).max(LIFELINE_TOP + sh + LIFELINE_BOTTOM);
    let fb = rect(r, frame);
    let top = r.f(v, "top");
    set_rect(r, v, R { x: fb.x, y: top, w: fb.w, h });
    let label_x = fb.x + PAD * 2.0;
    if let Some(nl) = r.refd(v, "nameLabel") {
        r.set(nl, "text", json!(text));
        r.set(nl, "font", json!(views::font_str(&ms.face(), lh, 1)));
        set_rect(r, nl, R { x: label_x, y: top, w: lh, h });
    }
    // states stacked from 25 px down, their names right-aligned before the start
    let mut ss = states;
    ss.sort_by(|a, b| r.f(*a, "top").total_cmp(&r.f(*b, "top")));
    let mut y = top + LIFELINE_TOP;
    let sx = label_x + lh + PAD * 2.0;
    let mut segments = vec![];
    for s in ss {
        let sh = r.f(s, "height").max(STATE_MIN_H);
        set_rect(r, s, R { x: sx, y, w: fb.x + fb.w - sx, h: sh });
        if let Some(nl) = r.refd(s, "nameLabel") {
            set_rect(r, nl, R { x: sx, y, w: (start - PAD - sx).max(0.0), h: sh });
        }
        for g in inside(r, s, "UMLTimeSegmentView") {
            r.set(g, "top", json!((y + sh / 2.0).round() - (SEGMENT_H / 2.0).floor()));
            r.set(g, "height", json!(SEGMENT_H));
            let w = r.f(g, "width").max(SEGMENT_MIN_W);
            r.set(g, "width", json!(w));
            segments.push(g);
        }
        y += sh;
    }
    // the time segments follow one another from the start
    segments.sort_by(|a, b| r.f(*a, "left").total_cmp(&r.f(*b, "left")));
    let mut x = start;
    for g in segments {
        r.set(g, "left", json!(x));
        x += r.f(g, "width");
    }
}

/// A lifeline's time segments in order, each with the one before it.
fn previous(r: &Repo, g: Eid) -> Option<Eid> {
    let lifeline = r.refd(g, "containerView").and_then(|s| r.refd(s, "containerView"))?;
    let mut all: Vec<Eid> = inside(r, lifeline, "UMLTimingStateView").into_iter().flat_map(|s| inside(r, s, "UMLTimeSegmentView")).collect();
    all.sort_by(|a, b| r.f(*a, "left").total_cmp(&r.f(*b, "left")));
    let i = all.iter().position(|x| *x == g)?;
    i.checked_sub(1).map(|j| all[j])
}

/// Place each message's ends at its time segments' middle heights, snapped to a segment's end when
/// within 10 px (UMLTimingMessageView.arrangeObject).
pub fn layout_messages(r: &mut Repo, ms: &dyn Measure, d: Eid) {
    let msgs: Vec<Eid> = r.list(d, "ownedViews").into_iter().filter(|v| r.alive(*v) && r.ty(*v) == "UMLTimingMessageView").collect();
    for e in msgs {
        let (Some(t), Some(h)) = (r.refd(e, "tail"), r.refd(e, "head")) else { continue };
        let mut pts = views::points_of(r, e);
        if pts.len() < 2 {
            pts = vec![P { x: 0.0, y: 0.0 }, P { x: 0.0, y: 0.0 }];
        }
        let (tb, hb) = (rect(r, t), rect(r, h));
        let snap = |x: f64, b: R| if (x - b.x).abs() <= 10.0 { b.x } else if (x - (b.x + b.w)).abs() <= 10.0 { b.x + b.w } else { x };
        let n = pts.len();
        let tp = P { x: snap(pts[0].x, tb), y: (tb.y + tb.h / 2.0).round() };
        let hp = P { x: snap(pts[n - 1].x, hb), y: (hb.y + hb.h / 2.0).round() };
        let pts = vec![tp, hp];
        r.set(e, "points", json!(views::format_points(&pts)));
        views::layout_edge_labels(r, ms, e, &pts);
    }
}

/// Draw a lifeline: a line along its bottom and its name upwards.
pub fn draw_lifeline(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba, fc: Rgba) {
    let b = rect(r, v);
    c.polyline(&[P { x: b.x, y: b.y + b.h }, P { x: b.x + b.w, y: b.y + b.h }], line, false);
    if let Some(nl) = r.refd(v, "nameLabel") {
        let (_, size) = views::view_font(r, v);
        c.text_up(&r.str(nl, "text"), rect(r, nl), 1, size, fc);
    }
}

/// Draw a state's name, right-aligned before the time line.
pub fn draw_state(c: &mut dyn Canvas, r: &Repo, v: Eid, fc: Rgba) {
    if let Some(nl) = r.refd(v, "nameLabel") {
        let (_, size) = views::view_font(r, v);
        c.text(&r.str(nl, "text"), rect(r, nl), 1, 0, size, fc, false);
    }
}

/// Draw a time segment: a line across its middle, and a step from the previous segment's height.
pub fn draw_segment(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba) {
    let b = rect(r, v);
    let m = (SEGMENT_H / 2.0).floor();
    let y = b.y + m;
    c.polyline(&[P { x: b.x, y }, P { x: b.x + b.w + 1.0, y }], line, false);
    if let Some(p) = previous(r, v) {
        c.polyline(&[P { x: b.x, y }, P { x: b.x, y: r.f(p, "top") + m }], line, false);
    }
}

/// Draw a time tick: a short mark and its text 10 px under it.
pub fn draw_tick(c: &mut dyn Canvas, r: &Repo, v: Eid, line: Rgba, fc: Rgba) {
    let b = rect(r, v);
    c.polyline(&[P { x: b.x, y: b.y }, P { x: b.x, y: b.y + b.h }], line, false);
    let t = r.str(v, "text");
    if !t.is_empty() {
        let (_, size) = views::view_font(r, v);
        let w = c.width(&t, 0) * size / 13.0;
        c.text(&t, R { x: b.x - w / 2.0, y: b.y + b.h + 10.0, w, h: size }, 2, 0, size, fc, false);
    }
}

impl App {
    /// The timing diagram's tools: a lifeline in the frame under the point, a state or condition in
    /// the lifeline under it, a time segment on a state, a time tick on a frame.
    pub fn create_timing_node(&mut self, d: Eid, kind: &str, rect: R) -> Option<Option<Eid>> {
        if self.repo.ty(d) != "UMLTimingDiagram" {
            return None;
        }
        let at = P { x: rect.x, y: rect.y };
        let under = |a: &App, ty: &str| {
            a.repo.list(d, "ownedViews").into_iter().rev().find(|v| a.repo.alive(*v) && a.repo.ty(*v) == ty && { let b = views::node_rect(&a.repo, *v); at.x >= b.x && at.x <= b.x + b.w && at.y >= b.y && at.y <= b.y + b.h })
        };
        let grid = self.grid_size();
        Some(match kind {
            "UMLLifeline" => under(self, "UMLTimingFrameView").and_then(|f| self.timing_lifeline(d, f, rect)),
            "UMLTimingState" => {
                let lifeline = under(self, "UMLTimingLifelineView").or_else(|| under(self, "UMLTimingStateView").and_then(|s| self.repo.refd(s, "containerView")));
                lifeline.and_then(|l| self.timing_state(d, l, at))
            }
            "UMLTimeSegment" => under(self, "UMLTimingStateView").and_then(|s| self.time_segment(d, s, rect)),
            "UMLTimeTick" => under(self, "UMLTimingFrameView").map(|f| create_tick_view(&mut self.repo, d, f, rect, grid * 3.0)),
            _ => return None,
        })
    }

    fn timing_lifeline(&mut self, d: Eid, frame: Eid, rect: R) -> Option<Eid> {
        let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
        let classifier = self.repo.parent(inter).filter(|c| self.repo.kind_of(*c, "UMLClassifier"))?;
        let names: std::collections::HashSet<String> = self.repo.list(classifier, "attributes").into_iter().map(|a| self.repo.name(a)).collect();
        let role_name = (1..).map(|i| format!("Role{i}")).find(|n| !names.contains(n)).unwrap();
        let role = self.repo.create("UMLAttribute", &[("name", json!(role_name))], Some((classifier, "attributes")));
        let name = self.new_name(inter, "UMLLifeline");
        let rr = self.repo.ref_to(role);
        let m = self.repo.create("UMLLifeline", &[("name", json!(name)), ("represent", rr)], Some((inter, "participants")));
        let v = create_lifeline_view(&mut self.repo, d, m, frame, rect);
        self.apply_view_defaults(v);
        Some(v)
    }

    /// A state or condition: a constraint "State-Condition1"… of the lifeline (timingStateFn).
    pub fn timing_state(&mut self, d: Eid, lifeline: Eid, at: P) -> Option<Eid> {
        let lm = self.repo.refd(lifeline, "model")?;
        let names: std::collections::HashSet<String> = self.repo.children(lm).into_iter().map(|(_, c)| self.repo.name(c)).collect();
        let name = (1..).map(|i| format!("State-Condition{i}")).find(|n| !names.contains(n)).unwrap();
        let m = self.repo.create("UMLConstraint", &[("name", json!(name))], Some((lm, "ownedElements")));
        let v = create_state_view(&mut self.repo, d, m, lifeline, at);
        self.apply_view_defaults(v);
        Some(v)
    }

    /// A time segment: a state invariant of the interaction covering the state's lifeline, its
    /// invariant the state (timeSegmentFn).
    pub fn time_segment(&mut self, d: Eid, state: Eid, rect: R) -> Option<Eid> {
        let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
        let sm = self.repo.refd(state, "model")?;
        let lm = self.repo.parent(sm)?;
        let names: std::collections::HashSet<String> = self.repo.list(inter, "fragments").into_iter().map(|f| self.repo.name(f)).collect();
        let name = (1..).map(|i| format!("Time Segment{i}")).find(|n| !names.contains(n)).unwrap();
        let (rc, ri) = (self.repo.ref_to(lm), self.repo.ref_to(sm));
        let m = self.repo.create("UMLStateInvariant", &[("name", json!(name)), ("covered", rc), ("invariant", ri)], Some((inter, "fragments")));
        let grid = self.grid_size();
        let v = create_segment_view(&mut self.repo, d, m, state, rect, grid * 3.0);
        self.apply_view_defaults(v);
        Some(v)
    }

    /// A message between two time segments, from the first's lifeline to the second's.
    pub fn timing_message(&mut self, d: Eid, sort: &str, tail: Eid, head: Eid, from: P, to: P) -> Option<Eid> {
        let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
        let (tm, hm) = (self.repo.refd(tail, "model")?, self.repo.refd(head, "model")?);
        let (s, t) = (self.repo.refd(tm, "covered")?, self.repo.refd(hm, "covered")?);
        let name = if matches!(sort, "reply" | "createMessage" | "deleteMessage") { String::new() } else { self.new_name(inter, "UMLMessage") };
        let (rs, rt) = (self.repo.ref_to(s), self.repo.ref_to(t));
        let m = self.repo.create("UMLMessage", &[("name", json!(name)), ("source", rs), ("target", rt)], Some((inter, "messages")));
        if sort != "synchCall" {
            self.repo.set(m, "messageSort", json!(sort));
        }
        let e = create_message_view(&mut self.repo, d, m, tail, head, from, to);
        self.apply_view_defaults(e);
        Some(e)
    }

    /// Create State/Condition on a lifeline's quick edit, and Create Time Segment on a state's, at
    /// the middle of the view (uml-commands.js:2665-2715).
    pub fn add_timing(&mut self, v: Eid) -> Option<Eid> {
        let d = self.diagram?;
        let b = views::node_rect(&self.repo, v);
        let at = P { x: (b.x + b.w / 2.0).round(), y: (b.y + b.h / 2.0).round() };
        self.change(|a| match a.repo.ty(v) {
            "UMLTimingLifelineView" => a.timing_state(d, v, at),
            "UMLTimingStateView" => a.time_segment(d, v, R { x: at.x, y: at.y, w: 0.0, h: 0.0 }),
            _ => None,
        })
    }
}

/// After a time segment is moved onto another state: the state is its invariant, and the state's
/// lifeline what it covers (UMLTimeSegmentView.changeContainerViewOperation).
pub fn segment_moved(r: &mut Repo, v: Eid, state: Eid) {
    let (Some(m), Some(sm)) = (r.refd(v, "model"), r.refd(state, "model")) else { return };
    r.set_ref(m, "invariant", Some(sm));
    if let Some(lm) = r.refd(state, "containerView").and_then(|l| r.refd(l, "model")) {
        r.set_ref(m, "covered", Some(lm));
    }
}

/// Which timing views hold which (canContainViewKind of the frame and state views).
pub fn can_contain(container_ty: &str, view_ty: &str) -> Option<bool> {
    let timing = |t: &str| matches!(t, "UMLTimingFrameView" | "UMLTimingLifelineView" | "UMLTimingStateView" | "UMLTimeSegmentView" | "UMLTimeTickView");
    if !timing(container_ty) && !timing(view_ty) {
        return None;
    }
    Some(matches!((container_ty, view_ty), ("UMLTimingFrameView", "UMLTimingLifelineView") | ("UMLTimingStateView", "UMLTimeSegmentView")))
}
