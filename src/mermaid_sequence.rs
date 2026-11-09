//! Tools > Generate Diagram by Mermaid for sequence diagrams (extensions/default/mermaid:
//! factory/sequence-factory.js, parser/sequence/sequenceDiagram.jison and sequenceDb.js).
//!
//! UMLCraft reads Mermaid's `sequenceDiagram` syntax itself into the same records as Mermaid's
//! parser (participants in order of appearance, then one list of messages and control signals),
//! and builds the diagram from them with the original's rules: a collaboration holding a role per
//! participant, an interaction holding the lifelines, messages and combined fragments, lifelines
//! 150 px apart from x = 25, messages 60 px apart from y = 110.

use crate::app::App;
use crate::model::Eid;
use crate::views::{self, Measure, P, R};
use serde_json::json;
use std::collections::HashMap;

/// The dialog's example (UMLCraft's own).
pub const SEQUENCE_SAMPLE: &str = "sequenceDiagram
    actor Reader
    participant Desk as Front desk
    Reader->>+Desk: Borrow a book
    Desk->>Desk: Check the card
    alt the book is in
        Desk-->>Reader: Here it is
    else it is lent
        Desk--)Reader: Join the waiting list
    end
    create participant Loan
    Desk->>Loan: new
    Note right of Loan: due in 3 weeks
    Desk->>-Reader: Done
";

/// Mermaid's line types (LINETYPE, sequenceDb.js:13-46).
pub mod line {
    pub const SOLID: u8 = 0;
    pub const DOTTED: u8 = 1;
    pub const NOTE: u8 = 2;
    pub const SOLID_CROSS: u8 = 3;
    pub const DOTTED_CROSS: u8 = 4;
    pub const SOLID_OPEN: u8 = 5;
    pub const DOTTED_OPEN: u8 = 6;
    pub const LOOP_START: u8 = 10;
    pub const LOOP_END: u8 = 11;
    pub const ALT_START: u8 = 12;
    pub const ALT_ELSE: u8 = 13;
    pub const ALT_END: u8 = 14;
    pub const OPT_START: u8 = 15;
    pub const OPT_END: u8 = 16;
    pub const ACTIVE_START: u8 = 17;
    pub const ACTIVE_END: u8 = 18;
    pub const PAR_START: u8 = 19;
    pub const PAR_AND: u8 = 20;
    pub const PAR_END: u8 = 21;
    pub const RECT_START: u8 = 22;
    pub const RECT_END: u8 = 23;
    pub const SOLID_POINT: u8 = 24;
    pub const DOTTED_POINT: u8 = 25;
    pub const AUTONUMBER: u8 = 26;
    pub const CRITICAL_START: u8 = 27;
    pub const CRITICAL_OPTION: u8 = 28;
    pub const CRITICAL_END: u8 = 29;
    pub const BREAK_START: u8 = 30;
    pub const BREAK_END: u8 = 31;
    pub const PAR_OVER_START: u8 = 32;
    pub const BIDIRECTIONAL_SOLID: u8 = 33;
    pub const BIDIRECTIONAL_DOTTED: u8 = 34;
}

/// Note placements (PLACEMENT, sequenceDb.js:51-55).
pub const LEFT_OF: u8 = 0;
pub const RIGHT_OF: u8 = 1;
pub const OVER: u8 = 2;

/// A participant: its id, its description (the alias, else the id) and "participant" or "actor".
#[derive(Clone, Debug, PartialEq)]
pub struct SActor {
    pub id: String,
    pub description: String,
    pub kind: String,
}

/// A message or a control signal (a block start or end, an activation, autonumber, a note).
#[derive(Clone, Debug, PartialEq)]
pub struct SMsg {
    pub id: usize,
    pub from: Option<String>,
    pub to: Option<String>,
    pub text: String,
    pub ty: u8,
    pub placement: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SeqDiagram {
    pub actors: Vec<SActor>,
    /// The participants made by `create`, and the id of the message that creates them.
    pub created: HashMap<String, usize>,
    /// The participants named by `destroy`, and the id of the message after it.
    pub destroyed: HashMap<String, usize>,
    pub messages: Vec<SMsg>,
}

/// The arrows, longest first, with their line type (sequenceDiagram.jison:79-89).
const ARROWS: [(&str, u8); 10] = [
    ("<<-->>", line::BIDIRECTIONAL_DOTTED),
    ("<<->>", line::BIDIRECTIONAL_SOLID),
    ("-->>", line::DOTTED),
    ("->>", line::SOLID),
    ("-->", line::DOTTED_OPEN),
    ("->", line::SOLID_OPEN),
    ("--x", line::DOTTED_CROSS),
    ("-x", line::SOLID_CROSS),
    ("--)", line::DOTTED_POINT),
    ("-)", line::SOLID_POINT),
];

/// The open blocks: what `end` closes, and the line type it adds.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Block {
    Box,
    Loop,
    Rect,
    Opt,
    Alt,
    Par,
    Critical,
    Break,
}

/// `word` at the start of `s`, in any case, followed by a space or the end: the rest, trimmed.
fn keyword<'a>(s: &'a str, word: &str) -> Option<&'a str> {
    let head = s.get(..word.len())?;
    if !head.eq_ignore_ascii_case(word) {
        return None;
    }
    let rest = &s[word.len()..];
    (rest.is_empty() || rest.starts_with(char::is_whitespace)).then(|| rest.trim())
}

/// A message text without its `wrap:` / `nowrap:` prefix (parseMessage, sequenceDb.js:290-300).
fn message_text(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix(':').unwrap_or(s);
    for p in ["wrap:", "nowrap:"] {
        if let Some(head) = s.get(..p.len()) {
            if head.eq_ignore_ascii_case(p) {
                return s[p.len()..].trim().to_string();
            }
        }
    }
    s.trim().to_string()
}

impl SeqDiagram {
    /// addActor (sequenceDb.js:101-150): a known participant keeps its record unless a description
    /// is given; a new one gets its id as description.
    fn add_actor(&mut self, id: &str, description: Option<String>, kind: Option<&str>) {
        let old = self.actors.iter().position(|a| a.id == id);
        if old.is_some() && description.is_none() {
            return;
        }
        let a = SActor { id: id.to_string(), description: description.unwrap_or_else(|| id.to_string()), kind: kind.unwrap_or("participant").to_string() };
        match old {
            Some(i) => self.actors[i] = a,
            None => self.actors.push(a),
        }
    }

    fn push(&mut self, from: Option<&str>, to: Option<&str>, text: &str, ty: u8, placement: Option<u8>) {
        let id = self.messages.len();
        self.messages.push(SMsg { id, from: from.map(String::from), to: to.map(String::from), text: text.to_string(), ty, placement });
    }

    /// How many activations of `part` are open (activationCount, sequenceDb.js:150-170).
    fn activations(&self, part: &str) -> i32 {
        self.messages.iter().filter(|m| m.from.as_deref() == Some(part)).map(|m| match m.ty {
            line::ACTIVE_START => 1,
            line::ACTIVE_END => -1,
            _ => 0,
        }).sum()
    }

    fn deactivate(&mut self, part: &str) -> Result<(), String> {
        if self.activations(part) < 1 {
            return Err(format!("Trying to inactivate an inactive participant ({part})"));
        }
        self.push(Some(part), None, "", line::ACTIVE_END, None);
        Ok(())
    }
}

/// Parse Mermaid `sequenceDiagram` code into Mermaid's records.
pub fn parse_sequence(code: &str) -> Result<SeqDiagram, String> {
    if crate::mermaid::diagram_type(code)? != "sequence" {
        return Err("Unsupported diagram type".into());
    }
    let mut d = SeqDiagram::default();
    let mut stack: Vec<Block> = vec![];
    let (mut last_created, mut last_destroyed): (Option<String>, Option<String>) = (None, None);
    let mut in_descr = false;
    let mut started = false;
    for raw in code.lines() {
        // a multi-line accDescr { … } is skipped
        if in_descr {
            in_descr = !raw.contains('}');
            continue;
        }
        let mut text = raw;
        if !started {
            let t = text.trim();
            if t.is_empty() || t.starts_with("%%") {
                continue;
            }
            started = true;
            text = &t["sequenceDiagram".len().min(t.len())..];
        }
        // comments: `%%` and `#` run to the end of the line; `;` separates statements
        let text = text.split("%%").next().unwrap_or("");
        let text = text.split('#').next().unwrap_or("");
        for st in text.split(';') {
            let s = st.trim();
            if s.is_empty() {
                continue;
            }
            let lower = s.to_ascii_lowercase();
            if lower.starts_with("accdescr") && s.contains('{') {
                in_descr = !s.contains('}');
                continue;
            }
            if lower.starts_with("acctitle") || lower.starts_with("accdescr") || lower.starts_with("title:") || keyword(s, "title").is_some() {
                continue;
            }
            // participants: `participant X [as Text]`, `actor X [as Text]`, with `create` before
            let (create, s1) = match keyword(s, "create") {
                Some(rest) => (true, rest),
                None => (false, s),
            };
            let decl = keyword(s1, "participant").map(|r| (r, "participant")).or_else(|| keyword(s1, "actor").map(|r| (r, "actor")));
            if let Some((rest, kind)) = decl {
                let (id, desc) = match rest.find(" as ").or_else(|| rest.find("\tas ")) {
                    Some(i) => (rest[..i].trim(), Some(message_text(&rest[i + 4..]))),
                    None => (rest.trim(), None),
                };
                if id.is_empty() {
                    return Err("Failed to parse sequence diagram".into());
                }
                if create {
                    if d.actors.iter().any(|a| a.id == id) {
                        return Err("It is not possible to have actors with the same id, even if one is destroyed before the next is created. Use 'AS' aliases to simulate the behavior".into());
                    }
                    last_created = Some(id.to_string());
                }
                d.add_actor(id, desc, Some(kind));
                if create {
                    d.created.insert(id.to_string(), d.messages.len());
                }
                continue;
            }
            if create {
                return Err("Failed to parse sequence diagram".into());
            }
            if let Some(id) = keyword(s, "destroy") {
                last_destroyed = Some(id.to_string());
                d.destroyed.insert(id.to_string(), d.messages.len());
                continue;
            }
            if keyword(s, "box").is_some() {
                stack.push(Block::Box);
                continue;
            }
            if lower == "end" {
                let end = match stack.pop() {
                    Some(Block::Box) => None,
                    Some(Block::Loop) => Some(line::LOOP_END),
                    Some(Block::Rect) => Some(line::RECT_END),
                    Some(Block::Opt) => Some(line::OPT_END),
                    Some(Block::Alt) => Some(line::ALT_END),
                    Some(Block::Par) => Some(line::PAR_END),
                    Some(Block::Critical) => Some(line::CRITICAL_END),
                    Some(Block::Break) => Some(line::BREAK_END),
                    None => return Err("Failed to parse sequence diagram".into()),
                };
                if let Some(t) = end {
                    d.push(None, None, "", t, None);
                }
                continue;
            }
            // the blocks and their sections; par_over before par
            let blocks: [(&str, Option<Block>, Option<Block>, u8); 11] = [
                ("loop", Some(Block::Loop), None, line::LOOP_START),
                ("rect", Some(Block::Rect), None, line::RECT_START),
                ("opt", Some(Block::Opt), None, line::OPT_START),
                ("alt", Some(Block::Alt), None, line::ALT_START),
                ("else", None, Some(Block::Alt), line::ALT_ELSE),
                ("par_over", Some(Block::Par), None, line::PAR_OVER_START),
                ("par", Some(Block::Par), None, line::PAR_START),
                ("and", None, Some(Block::Par), line::PAR_AND),
                ("critical", Some(Block::Critical), None, line::CRITICAL_START),
                ("option", None, Some(Block::Critical), line::CRITICAL_OPTION),
                ("break", Some(Block::Break), None, line::BREAK_START),
            ];
            if let Some((rest, open, within, ty)) = blocks.iter().find_map(|(w, o, i, t)| keyword(s, w).map(|r| (r, *o, *i, *t))) {
                if let Some(b) = within {
                    if stack.last() != Some(&b) {
                        return Err("Failed to parse sequence diagram".into());
                    }
                }
                if let Some(b) = open {
                    stack.push(b);
                }
                d.push(None, None, &message_text(rest), ty, None);
                continue;
            }
            if let Some(rest) = keyword(s, "autonumber") {
                let ok = rest.is_empty() || rest.eq_ignore_ascii_case("off") || rest.split_whitespace().all(|n| n.parse::<u32>().is_ok());
                if !ok {
                    return Err("Failed to parse sequence diagram".into());
                }
                d.push(None, None, "", line::AUTONUMBER, None);
                continue;
            }
            if let Some(id) = keyword(s, "activate") {
                d.add_actor(id, None, None);
                d.push(Some(id), None, "", line::ACTIVE_START, None);
                continue;
            }
            if let Some(id) = keyword(s, "deactivate") {
                d.add_actor(id, None, None);
                d.deactivate(id)?;
                continue;
            }
            // `Note left of A: text`, `Note right of A: text`, `Note over A[,B]: text`
            if let Some(rest) = keyword(s, "note") {
                let (who, txt) = rest.split_once(':').ok_or("Failed to parse sequence diagram")?;
                let (placement, who) = if let Some(w) = keyword(who, "left of") {
                    (LEFT_OF, w)
                } else if let Some(w) = keyword(who, "right of") {
                    (RIGHT_OF, w)
                } else if let Some(w) = keyword(who, "over") {
                    (OVER, w)
                } else {
                    return Err("Failed to parse sequence diagram".into());
                };
                let ids: Vec<&str> = if placement == OVER { who.split(',').map(str::trim).take(2).collect() } else { vec![who.trim()] };
                if ids.iter().any(|i| i.is_empty()) {
                    return Err("Failed to parse sequence diagram".into());
                }
                for i in &ids {
                    d.add_actor(i, None, None);
                }
                let (from, to) = (ids[0], *ids.last().unwrap());
                d.push(Some(from), Some(to), &message_text(txt), line::NOTE, Some(placement));
                continue;
            }
            // links, properties and details only name their participant
            if let Some(rest) = ["links", "link", "properties", "details"].iter().find_map(|w| keyword(s, w)) {
                let id = rest.split(':').next().unwrap_or("").trim();
                d.add_actor(id, None, None);
                continue;
            }
            // a message: `A->>B: text`, `A->>+B: text`, `A->>-B: text`
            let (from, arrow, ty) = s
                .char_indices()
                .filter(|(i, c)| *i > 0 && (*c == '-' || *c == '<'))
                .find_map(|(i, _)| ARROWS.iter().find(|(a, _)| s[i..].starts_with(a)).map(|(a, t)| (&s[..i], *a, *t)))
                .ok_or("Failed to parse sequence diagram")?;
            let after = &s[from.len() + arrow.len()..];
            let (act, after) = match after.trim_start().chars().next() {
                Some('+') => (1, &after.trim_start()[1..]),
                Some('-') => (-1, &after.trim_start()[1..]),
                _ => (0, after),
            };
            let (to, txt) = after.split_once(':').ok_or("Failed to parse sequence diagram")?;
            let (from, to) = (from.trim(), to.trim());
            if from.is_empty() || to.is_empty() {
                return Err("Failed to parse sequence diagram".into());
            }
            d.add_actor(from, None, None);
            d.add_actor(to, None, None);
            // a created participant needs its creating message next, a destroyed one its destroying message
            if let Some(c) = last_created.take() {
                if to != c {
                    return Err(format!("The created participant {c} does not have an associated creating message after its declaration. Please check the sequence diagram."));
                }
            } else if let Some(x) = last_destroyed.take() {
                if to != x && from != x {
                    return Err(format!("The destroyed participant {x} does not have an associated destroying message after its declaration. Please check the sequence diagram."));
                }
            }
            d.push(Some(from), Some(to), &message_text(txt), ty, None);
            match act {
                1 => d.push(Some(to), None, "", line::ACTIVE_START, None),
                -1 => d.deactivate(from)?,
                _ => {}
            }
        }
    }
    if !stack.is_empty() {
        return Err("Failed to parse sequence diagram".into());
    }
    Ok(d)
}

const LIFELINE_INTERVAL: f64 = 150.0; // sequence-factory.js:7-8
const MESSAGE_INTERVAL: f64 = 60.0;

fn center(r: &crate::model::Repo, v: Eid) -> P {
    let b = views::node_rect(r, v);
    P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 }
}

impl App {
    /// A Mermaid sequence diagram: "Collaboration by Mermaid" in the base (the project by default; a
    /// package, a classifier or an operation otherwise), holding "Interaction by Mermaid" and its
    /// "Sequence Diagram by Mermaid" (createSequenceDiagram, sequence-factory.js:48-150).
    pub fn generate_sequence(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let sd = parse_sequence(code)?;
        let base = match base {
            None => self.repo.root,
            Some(b) if self.repo.kind_of(b, "UMLPackage") || self.repo.kind_of(b, "UMLClassifier") || self.repo.ty(b) == "UMLOperation" => b,
            Some(_) => return Err("Base element must be a package, classifer or operation".into()),
        };
        let undo = self.undo.len();
        let d = self.change(|a| {
            let r = &mut a.repo;
            let collab = r.create("UMLCollaboration", &[("name", json!("Collaboration by Mermaid"))], Some((base, "ownedElements")));
            let inter = r.create("UMLInteraction", &[("name", json!("Interaction by Mermaid"))], Some((collab, "ownedElements")));
            // activations are shown when any is given, numbers when autonumber appears
            let has_act = sd.messages.iter().any(|m| m.ty == line::ACTIVE_START || m.ty == line::ACTIVE_END);
            let has_num = sd.messages.iter().any(|m| m.ty == line::AUTONUMBER);
            let d = r.create(
                "UMLSequenceDiagram",
                &[("name", json!("Sequence Diagram by Mermaid")), ("showSequenceNumber", json!(has_num)), ("showActivation", json!(has_act))],
                Some((inter, "ownedElements")),
            );
            // the lifelines: a role per participant, and an actor typed onto it for `actor`
            let mut lifelines: HashMap<String, (Eid, Eid)> = HashMap::new();
            let mut x = 25.0;
            for p in &sd.actors {
                let r = &mut a.repo;
                let actor = (p.kind == "actor").then(|| r.create("UMLActor", &[("name", json!(p.description))], Some((collab, "ownedElements"))));
                let role = r.create("UMLAttribute", &[("name", json!(format!("{}Role", p.description)))], Some((collab, "attributes")));
                if let Some(t) = actor {
                    r.set_ref(role, "type", Some(t));
                }
                let rr = r.ref_to(role);
                let m = r.create("UMLLifeline", &[("name", json!(p.description)), ("represent", rr)], Some((inter, "participants")));
                let v = crate::sequence::create_lifeline_view(r, d, m, R { x, y: 40.0, w: 70.0, h: 2000.0 });
                a.apply_view_defaults(v);
                if actor.is_some() {
                    a.repo.set(v, "stereotypeDisplay", json!("icon"));
                }
                lifelines.insert(p.id.clone(), (m, v));
                x += LIFELINE_INTERVAL;
            }
            // the messages and signals, top to bottom
            let mut y = 110.0;
            // the open fragments; par_over makes none (the original then closes the enclosing
            // fragment at its `end`, which UMLCraft does not copy)
            let mut stack: Vec<Option<Eid>> = vec![];
            let mut made: Vec<(Eid, Eid, f64)> = vec![];
            for msg in &sd.messages {
                let operator = match msg.ty {
                    line::LOOP_START => Some("loop"),
                    line::ALT_START => Some("alt"),
                    line::OPT_START => Some("opt"),
                    line::PAR_START => Some("par"),
                    line::CRITICAL_START => Some("critical"),
                    line::BREAK_START => Some("break"),
                    _ => None,
                };
                if let Some(op) = operator {
                    // a combined fragment 55 px above the next row, with its first operand
                    y += 30.0;
                    let r = &mut a.repo;
                    let cf = r.create("UMLCombinedFragment", &[("interactionOperator", json!(op))], Some((inter, "fragments")));
                    let v = crate::sequence::create_fragment_view(r, d, cf, R { x: 10.0, y: y - 55.0, w: 0.0, h: 0.0 });
                    a.apply_view_defaults(v);
                    add_operand(&mut a.repo, v, &msg.text);
                    stack.push(Some(v));
                } else if msg.ty == line::PAR_OVER_START {
                    stack.push(None);
                } else if matches!(msg.ty, line::ALT_ELSE | line::PAR_AND | line::CRITICAL_OPTION) {
                    if let Some(&Some(v)) = stack.last() {
                        add_operand(&mut a.repo, v, &msg.text);
                    }
                } else if matches!(msg.ty, line::LOOP_END | line::ALT_END | line::OPT_END | line::PAR_END | line::CRITICAL_END | line::BREAK_END) {
                    // its bottom 20 px under the last row
                    if let Some(Some(v)) = stack.pop() {
                        let top = a.repo.f(v, "top");
                        a.repo.set(v, "height", json!(y + 20.0 - top));
                    }
                } else if msg.ty == line::NOTE {
                    // a note over, left of or right of the lifelines, 40 high, on the current row
                    let (Some(s), Some(t)) = (msg.from.as_ref().and_then(|f| lifelines.get(f)), msg.to.as_ref().and_then(|t| lifelines.get(t))) else { continue };
                    let (sp, tp) = (center(&a.repo, s.1), center(&a.repo, t.1));
                    let (mut x0, mut x1) = (sp.x.min(tp.x) - 15.0, sp.x.max(tp.x) + 15.0);
                    if msg.placement == Some(LEFT_OF) {
                        x0 = sp.x.min(tp.x) - 100.0 - 15.0;
                        x1 = x0 + 100.0;
                    } else if msg.placement == Some(RIGHT_OF) {
                        x0 = sp.x.max(tp.x) + 15.0;
                        x1 = x0 + 100.0;
                    }
                    let n = views::create_note_view(&mut a.repo, d, R { x: x0, y, w: x1 - x0, h: 40.0 }, "UMLNoteView");
                    a.apply_view_defaults(n);
                    a.repo.set(n, "text", json!(msg.text));
                    a.repo.set(n, "wordWrap", json!(true));
                    a.repo.set(n, "autoResize", json!(false));
                    y += MESSAGE_INTERVAL;
                    continue;
                }
                // a message between two lifelines (activations, autonumber and blocks have none)
                let (Some(s), Some(t)) = (msg.from.as_ref().and_then(|f| lifelines.get(f)).copied(), msg.to.as_ref().and_then(|t| lifelines.get(t)).copied()) else { continue };
                let to = msg.to.as_deref().unwrap_or("");
                let sort = if sd.created.get(to) == Some(&msg.id) {
                    "createMessage"
                } else if sd.destroyed.get(to) == Some(&msg.id) {
                    "deleteMessage"
                } else {
                    match msg.ty {
                        line::DOTTED => "asynchSignal",
                        line::SOLID_POINT => "asynchCall",
                        line::DOTTED_POINT => "reply",
                        _ => "synchCall",
                    }
                };
                let r = &mut a.repo;
                let name = if msg.text.is_empty() { msg.id.to_string() } else { msg.text.clone() };
                let (rs, rt) = (r.ref_to(s.0), r.ref_to(t.0));
                let mut props = vec![("name", json!(name))];
                if sort != "synchCall" {
                    props.push(("messageSort", json!(sort)));
                }
                props.push(("source", rs));
                props.push(("target", rt));
                let m = r.create("UMLMessage", &props, Some((inter, "messages")));
                let (Some(tail), Some(head)) = (r.refd(s.1, "linePart"), r.refd(t.1, "linePart")) else { continue };
                let e = crate::sequence::create_message_view(r, d, m, tail, head, y);
                if let Some(act) = r.refd(e, "activation") {
                    r.set(act, "height", json!(MESSAGE_INTERVAL));
                }
                a.apply_view_defaults(e);
                made.push((s.1, t.1, y));
                y += MESSAGE_INTERVAL;
                // the innermost fragment moves to the message's lifelines and widens to cover them
                if let Some(&Some(v)) = stack.last() {
                    let (sp, tp) = (center(&a.repo, s.1), center(&a.repo, t.1));
                    let (x1, x2) = (sp.x.min(tp.x), sp.x.max(tp.x));
                    a.repo.set(v, "left", json!(x1 - 15.0));
                    let w = (x2 - x1).abs() + 30.0;
                    if a.repo.f(v, "width") < w {
                        a.repo.set(v, "width", json!(w));
                    }
                }
            }
            // each lifeline as high as its messages' span plus 60, at least 120 (adjustLifelineHeights)
            for (_, v) in lifelines.values() {
                let ys: Vec<f64> = made.iter().filter(|(s, t, _)| s == v || t == v).map(|m| m.2).collect();
                let span = if ys.is_empty() { 0.0 } else { ys.iter().cloned().fold(f64::MIN, f64::max) - ys.iter().cloned().fold(f64::MAX, f64::min) };
                a.repo.set(*v, "height", json!((span + 60.0).max(120.0)));
            }
            Some(d)
        });
        let d = d.ok_or("Failed to parse sequence diagram")?;
        self.undo.truncate(undo + 1);
        self.open_diagram(d);
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        self.focus = Some(d);
        Ok(d)
    }
}

/// An operand with its guard, in a fragment view, its view 60 high (addOperand, sequence-factory.js:375-392).
fn add_operand(r: &mut crate::model::Repo, v: Eid, guard: &str) {
    let Some(cf) = r.refd(v, "model") else { return };
    let o = r.create("UMLInteractionOperand", &[("guard", json!(guard))], Some((cf, "operands")));
    if let Some(oc) = r.refd(v, "operandCompartment") {
        crate::sequence::create_operand_view(r, oc, o, MESSAGE_INTERVAL);
    }
}
