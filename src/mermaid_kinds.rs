//! Tools > Generate Diagram by Mermaid for flowcharts, ER diagrams, mind maps and state diagrams
//! (extensions/default/mermaid/factory: flowchart-factory.js, er-factory.js, mindmap-factory.js,
//! state-factory.js). UMLCraft reads the Mermaid syntax itself; what StarUML takes from each kind
//! is kept, the rest of the syntax (styles, classes, clicks, subgraphs, icons) is skipped.

use crate::app::{App, AssocInit, Tool};
use crate::layout::{Dir, Seps};
use crate::model::Eid;
use crate::views::{self, Measure, R};
use serde_json::json;

/// The dialog's examples (UMLCraft's own).
pub const FLOWCHART_SAMPLE: &str = "flowchart TD
    Order([New order]) --> Stock{In stock?}
    Stock -->|Yes| Pack[Pack the parcel]
    Stock -->|No| Supplier[[Ask the supplier]]
    Supplier --> Stock
    Pack --> Db[(Orders)]
    Db --> Ship([Shipped])
";
pub const ER_SAMPLE: &str = "erDiagram
    MEMBER ||--o{ LOAN : borrows
    BOOK ||--o{ LOAN : \"is lent in\"
    MEMBER {
        int id PK
        string name
        string email UK
    }
    LOAN {
        int id PK
        int memberId FK
        int bookId FK
        date due
    }
    BOOK {
        string isbn PK
        string title
    }
";
pub const MINDMAP_SAMPLE: &str = "mindmap
  root((Library))
    Members
      Students
      Teachers
    Items
      Books
      DVDs
    Services
      Loans
      Reservations
";
pub const STATE_SAMPLE: &str = "stateDiagram-v2
    [*] --> Available
    Available --> OnLoan : borrow
    OnLoan --> Available : return
    OnLoan --> Overdue : due date passed
    Overdue --> Available : return
    Available --> [*] : withdraw
";

/// Mermaid's direction, read the way StarUML lays it out: its layout puts the head of a line above
/// its tail, so the directions of flowcharts, ER and state diagrams are reversed (utils.js
/// reverseDirection), while class diagrams and mind maps keep theirs.
pub(crate) fn dir(d: &str, reverse: bool) -> Dir {
    let d = match (d, reverse) {
        ("TB" | "TD", true) => "BT",
        ("BT", true) => "TB",
        ("LR", true) => "RL",
        ("RL", true) => "LR",
        (d, _) => d,
    };
    match d {
        "BT" => Dir::BottomTop,
        "LR" => Dir::LeftRight,
        "RL" => Dir::RightLeft,
        _ => Dir::TopBottom,
    }
}

pub(crate) fn lines(code: &str) -> Vec<String> {
    code.lines().map(|l| l.split("%%").next().unwrap_or("").trim_end().to_string()).collect()
}

pub(crate) fn unquote(s: &str) -> String {
    let s = s.trim();
    s.strip_prefix('"').and_then(|x| x.strip_suffix('"')).unwrap_or(s).to_string()
}

// ------------------------------------------------------------------ flowchart

/// A flowchart node: its id, its text and its Mermaid shape.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FNode {
    pub id: String,
    pub text: String,
    pub shape: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flowchart {
    pub direction: String,
    pub nodes: Vec<FNode>,
    pub edges: Vec<(String, String, String)>,
}

/// The shapes, longest openings first: (open, close, shape).
const SHAPES: [(&str, &str, &str); 15] = [
    ("(((", ")))", "doublecircle"),
    ("([", "])", "stadium"),
    ("[[", "]]", "subroutine"),
    ("[(", ")]", "cylinder"),
    ("((", "))", "circle"),
    ("{{", "}}", "hexagon"),
    ("[/", "/]", "lean_right"),
    ("[\\", "\\]", "lean_left"),
    ("[/", "\\]", "trapezoid"),
    ("[\\", "/]", "inv_trapezoid"),
    ("[", "]", "square"),
    ("(", ")", "round"),
    ("{", "}", "diamond"),
    (">", "]", "odd"),
    ("", "", "square"),
];

/// Read one node reference: `id`, `id[text]`, `id(text)`…; returns the node and the rest.
fn flow_node(s: &str) -> Option<(FNode, &str)> {
    let s = s.trim_start();
    let end = s.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-' || c == '.')).unwrap_or(s.len());
    // an id stops before an arrow's dashes
    let mut idlen = end;
    if let Some(i) = s[..end].find("--").or_else(|| s[..end].find("-.")) {
        idlen = i;
    }
    let id = s[..idlen].trim_end_matches('-').to_string();
    if id.is_empty() {
        return None;
    }
    let rest = &s[id.len()..];
    for (open, close, shape) in SHAPES.iter().filter(|s| !s.0.is_empty()) {
        if let Some(body) = rest.strip_prefix(open) {
            // the closing mark: after a quoted text, or the first one
            let close_at = if body.trim_start().starts_with('"') {
                let q = body.find('"').unwrap();
                body[q + 1..].find('"').map(|e| q + 1 + e + 1).and_then(|after| body[after..].find(close).map(|c| after + c))
            } else {
                body.find(close)
            };
            if let Some(c) = close_at {
                let text = unquote(&body[..c]);
                let rest = &body[c + close.len()..];
                let rest = rest.strip_prefix(":::").map(|r| r.trim_start_matches(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '-')).unwrap_or(rest);
                return Some((FNode { id, text, shape }, rest));
            }
        }
    }
    let rest = rest.strip_prefix(":::").map(|r| r.trim_start_matches(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '-')).unwrap_or(rest);
    Some((FNode { id, text: String::new(), shape: "" }, rest))
}

/// Read a link and its text: `-->`, `---`, `-.->`, `==>`, `--o`, `--x`, `<-->`, with any length,
/// and a text as `-->|text|` or `-- text -->`.
fn flow_link(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start();
    let b = s.as_bytes();
    let starts = |i: usize| i < b.len() && matches!(b[i], b'-' | b'=' | b'.' | b'<');
    let marked = !b.is_empty() && matches!(b[0], b'o' | b'x') && starts(1);
    if !starts(0) && !marked {
        return None;
    }
    // the link's own characters, then an "o" or "x" end mark right before a space
    let token = |s: &str| -> usize {
        let b = s.as_bytes();
        let mut e = 0;
        if e < b.len() && matches!(b[e], b'o' | b'x' | b'<') {
            e += 1;
        }
        while e < b.len() && matches!(b[e], b'-' | b'=' | b'.' | b'>') {
            e += 1;
        }
        if e < b.len() && matches!(b[e], b'o' | b'x') && (e + 1 == b.len() || b[e + 1].is_ascii_whitespace()) {
            e += 1;
        }
        e
    };
    let e = token(s);
    let tok = &s[..e];
    let mut rest = &s[e..];
    let mut text = String::new();
    // an opening "--", "-." or "==" with the text before the rest of the link
    let complete = tok.ends_with('>') || tok.ends_with('o') || tok.ends_with('x') || tok.trim_start_matches('<').len() >= 3;
    if !complete {
        let closers = ["-->", "---", ".->", "-.-", "==>", "==="];
        if let Some(i) = closers.iter().filter_map(|c| rest.find(c)).min() {
            text = unquote(&rest[..i]);
            let after = &rest[i..];
            rest = &after[token(after)..];
        }
    }
    let rest = rest.trim_start();
    if let Some(body) = rest.strip_prefix('|') {
        if let Some(c) = body.find('|') {
            return Some((unquote(&body[..c]), &body[c + 1..]));
        }
    }
    Some((text, rest))
}

pub fn parse_flowchart(code: &str) -> Result<Flowchart, String> {
    let mut fc = Flowchart::default();
    let all = lines(code);
    let mut it = all.iter().map(|l| l.trim()).filter(|l| !l.is_empty());
    let head = it.next().ok_or("Failed to parse flowchart")?;
    fc.direction = head.split_whitespace().nth(1).unwrap_or("TB").to_string();
    let add = |fc: &mut Flowchart, n: &FNode| {
        match fc.nodes.iter_mut().find(|x| x.id == n.id) {
            Some(x) => {
                if !n.text.is_empty() || !n.shape.is_empty() {
                    x.text = n.text.clone();
                    x.shape = n.shape;
                }
            }
            None => fc.nodes.push(n.clone()),
        }
    };
    for raw in it {
        for stmt in raw.split(';') {
            let stmt = stmt.trim();
            let word = stmt.split_whitespace().next().unwrap_or("");
            if stmt.is_empty() || matches!(word, "style" | "classDef" | "class" | "click" | "linkStyle" | "subgraph" | "end" | "direction") {
                continue;
            }
            // a chain: nodes (several joined by "&") with links between them
            let mut rest = stmt;
            let mut prev: Vec<String> = vec![];
            let mut pending: Option<String> = None;
            loop {
                let mut group = vec![];
                loop {
                    let Some((n, r)) = flow_node(rest) else { break };
                    add(&mut fc, &n);
                    group.push(n.id.clone());
                    rest = r.trim_start();
                    match rest.strip_prefix('&') {
                        Some(r) => rest = r,
                        None => break,
                    }
                }
                if group.is_empty() {
                    break;
                }
                if let Some(t) = pending.take() {
                    for a in &prev {
                        for b in &group {
                            fc.edges.push((a.clone(), b.clone(), t.clone()));
                        }
                    }
                }
                prev = group;
                match flow_link(rest) {
                    Some((t, r)) => {
                        pending = Some(t);
                        rest = r;
                    }
                    None => break,
                }
            }
        }
    }
    if fc.nodes.is_empty() {
        return Err("Failed to parse flowchart".into());
    }
    Ok(fc)
}

/// The flowchart type of a Mermaid shape (createNode, flowchart-factory.js:150-185); odd, lean
/// left, trapezoid and double circle shapes become processes.
pub fn fc_type(shape: &str) -> &'static str {
    match shape {
        "diamond" => "FCDecision",
        "stadium" => "FCTerminator",
        "round" => "FCAlternateProcess",
        "subroutine" => "FCPredefinedProcess",
        "cylinder" => "FCDatabase",
        "circle" => "FCConnector",
        "hexagon" => "FCPreparation",
        "lean_right" => "FCData",
        "inv_trapezoid" => "FCManualOperation",
        _ => "FCProcess",
    }
}

// ------------------------------------------------------------------ ER

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EColumn {
    pub ty: String,
    pub name: String,
    pub keys: Vec<String>,
    pub comment: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EEntity {
    pub id: String,
    pub label: String,
    pub columns: Vec<EColumn>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ERel {
    pub a: String,
    pub b: String,
    pub card_a: &'static str,
    pub card_b: &'static str,
    pub identifying: bool,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ErDiagram {
    pub direction: String,
    pub entities: Vec<EEntity>,
    pub rels: Vec<ERel>,
}

/// A crow's-foot token as the ERD cardinality (getCardinality, er-factory.js:170-185).
fn cardinality(tok: &str) -> Option<&'static str> {
    Some(match tok {
        "||" => "1",
        "|o" | "o|" => "0..1",
        "}o" | "o{" => "0..*",
        "}|" | "|{" => "1..*",
        _ => return None,
    })
}

fn er_entity(er: &mut ErDiagram, name: &str) -> usize {
    let (id, label) = match (name.find('['), name.rfind(']')) {
        (Some(a), Some(b)) if b > a => (name[..a].trim().to_string(), unquote(&name[a + 1..b])),
        _ => (name.trim().to_string(), String::new()),
    };
    match er.entities.iter().position(|e| e.id == id) {
        Some(i) => {
            if !label.is_empty() {
                er.entities[i].label = label;
            }
            i
        }
        None => {
            er.entities.push(EEntity { id, label, columns: vec![] });
            er.entities.len() - 1
        }
    }
}

pub fn parse_er(code: &str) -> Result<ErDiagram, String> {
    let mut er = ErDiagram { direction: "TB".into(), ..Default::default() };
    let all = lines(code);
    let mut it = all.iter().map(|l| l.trim()).filter(|l| !l.is_empty()).skip(1).peekable();
    while let Some(l) = it.next() {
        if let Some(d) = l.strip_prefix("direction") {
            er.direction = d.trim().to_string();
            continue;
        }
        // an entity block: NAME { type name PK,FK "comment" … }
        if let Some(name) = l.strip_suffix('{') {
            let i = er_entity(&mut er, name.trim());
            for row in it.by_ref() {
                if row.starts_with('}') {
                    break;
                }
                let (body, comment) = match row.find('"') {
                    Some(q) => (&row[..q], unquote(&row[q..])),
                    None => (row, String::new()),
                };
                let mut words = body.split_whitespace();
                let (Some(ty), Some(name)) = (words.next(), words.next()) else { continue };
                let keys: Vec<String> = words.collect::<Vec<_>>().join(" ").split(',').map(|k| k.trim().to_uppercase()).filter(|k| !k.is_empty()).collect();
                er.entities[i].columns.push(EColumn { ty: ty.to_string(), name: name.to_string(), keys, comment });
            }
            continue;
        }
        // a relationship: A ||--o{ B : label
        let (left, label) = match l.split_once(':') {
            Some((a, b)) => (a, unquote(b)),
            None => (l, String::new()),
        };
        let mut words = left.split_whitespace();
        let (Some(a), Some(spec), Some(b)) = (words.next(), words.next(), words.next()) else {
            er_entity(&mut er, left.trim());
            continue;
        };
        if spec.len() < 6 {
            continue;
        }
        let (ca, mid, cb) = (&spec[..2], &spec[2..spec.len() - 2], &spec[spec.len() - 2..]);
        let (Some(card_a), Some(card_b)) = (cardinality(ca), cardinality(cb)) else { continue };
        er_entity(&mut er, a);
        er_entity(&mut er, b);
        er.rels.push(ERel { a: a.trim().to_string(), b: b.trim().to_string(), card_a, card_b, identifying: mid == "--", label });
    }
    if er.entities.is_empty() {
        return Err("Failed to parse ER diagram".into());
    }
    Ok(er)
}

// ------------------------------------------------------------------ mind map

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MNode {
    pub text: String,
    pub children: Vec<MNode>,
}

/// A mind map node's text: inside its shape's marks, else the whole line.
fn mind_text(s: &str) -> String {
    let s = s.split(":::").next().unwrap_or(s).trim();
    for (open, close) in [("((", "))"), ("))", "(("), ("{{", "}}"), ("(", ")"), (")", "("), ("[", "]")] {
        if let Some(a) = s.find(open) {
            if s.ends_with(close) && a + open.len() <= s.len() - close.len() {
                return unquote(&s[a + open.len()..s.len() - close.len()]);
            }
        }
    }
    unquote(s)
}

pub fn parse_mindmap(code: &str) -> Result<MNode, String> {
    let all = lines(code);
    let rows: Vec<(usize, String)> = all
        .iter()
        .skip_while(|l| l.trim() != "mindmap")
        .skip(1)
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("::"))
        .map(|l| (l.len() - l.trim_start().len(), mind_text(l)))
        .collect();
    let Some((_, root_text)) = rows.first().cloned() else { return Err("Failed to parse mindmap".into()) };
    // a stack of (indent, path of child indices from the root)
    let mut root = MNode { text: root_text, children: vec![] };
    let mut stack: Vec<(usize, Vec<usize>)> = vec![(rows[0].0, vec![])];
    for (ind, text) in rows.into_iter().skip(1) {
        while stack.len() > 1 && stack.last().map(|s| s.0 >= ind).unwrap_or(false) {
            stack.pop();
        }
        let path = stack.last().unwrap().1.clone();
        let mut node = &mut root;
        for &i in &path {
            node = &mut node.children[i];
        }
        node.children.push(MNode { text, children: vec![] });
        let mut p = path;
        p.push(node.children.len() - 1);
        stack.push((ind, p));
    }
    Ok(root)
}

// ------------------------------------------------------------------ state

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SNode {
    pub id: String,
    pub label: String,
    /// "stateStart", "stateEnd", "choice", "fork", "join" or "rect"
    pub shape: &'static str,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateDiagram {
    pub direction: String,
    pub nodes: Vec<SNode>,
    pub edges: Vec<(String, String, String)>,
    /// notes: the state they are on and their text
    pub notes: Vec<(String, String)>,
}

pub fn parse_state(code: &str) -> Result<StateDiagram, String> {
    let mut sd = StateDiagram { direction: "TB".into(), ..Default::default() };
    let all = lines(code);
    let mut it = all.iter().map(|l| l.trim()).filter(|l| !l.is_empty()).skip(1);
    let node = |sd: &mut StateDiagram, id: &str, shape: &'static str| {
        if !sd.nodes.iter().any(|n| n.id == id) {
            sd.nodes.push(SNode { id: id.to_string(), label: String::new(), shape });
        }
    };
    while let Some(l) = it.next() {
        if let Some(d) = l.strip_prefix("direction") {
            sd.direction = d.trim().to_string();
            continue;
        }
        if l == "}" || l.starts_with("classDef") || l.starts_with("class ") || l == "--" {
            continue;
        }
        if let Some(rest) = l.strip_prefix("note ") {
            // note left of X : text, or a block up to "end note"
            let rest = rest.trim_start_matches("left of").trim_start_matches("right of").trim();
            let (target, text) = match rest.split_once(':') {
                Some((t, x)) => (t.trim().to_string(), x.trim().to_string()),
                None => {
                    let mut body = vec![];
                    for b in it.by_ref() {
                        if b == "end note" {
                            break;
                        }
                        body.push(b.to_string());
                    }
                    (rest.to_string(), body.join("\n"))
                }
            };
            sd.notes.push((target, text));
            continue;
        }
        if let Some(rest) = l.strip_prefix("state ") {
            let rest = rest.trim().trim_end_matches('{').trim();
            // state "Description" as Id, or state Id <<choice>>
            if let Some((desc, id)) = rest.split_once(" as ") {
                let id = id.trim();
                node(&mut sd, id, "rect");
                if let Some(n) = sd.nodes.iter_mut().find(|n| n.id == id) {
                    n.label = unquote(desc);
                }
            } else {
                let mut w = rest.split_whitespace();
                let id = w.next().unwrap_or("");
                let shape = match w.next() {
                    Some("<<choice>>") => "choice",
                    Some("<<fork>>") => "fork",
                    Some("<<join>>") => "join",
                    _ => "rect",
                };
                node(&mut sd, id, shape);
                if let Some(n) = sd.nodes.iter_mut().find(|n| n.id == id) {
                    n.shape = shape;
                }
            }
            continue;
        }
        // a transition: A --> B : label, with [*] as the start or end
        if let Some((a, rest)) = l.split_once("-->") {
            let (b, label) = match rest.split_once(':') {
                Some((b, x)) => (b.trim(), x.trim().to_string()),
                None => (rest.trim(), String::new()),
            };
            let a = a.trim();
            let a = if a == "[*]" {
                node(&mut sd, "root_start", "stateStart");
                "root_start"
            } else {
                node(&mut sd, a, "rect");
                a
            };
            let b = if b == "[*]" {
                node(&mut sd, "root_end", "stateEnd");
                "root_end"
            } else {
                node(&mut sd, b, "rect");
                b
            };
            sd.edges.push((a.to_string(), b.to_string(), label));
            continue;
        }
        // Id : description
        if let Some((id, desc)) = l.split_once(':') {
            let id = id.trim();
            node(&mut sd, id, "rect");
            if let Some(n) = sd.nodes.iter_mut().find(|n| n.id == id) {
                n.label = desc.trim().to_string();
            }
            continue;
        }
        node(&mut sd, l.trim_end_matches('{').trim(), "rect");
    }
    if sd.nodes.is_empty() {
        return Err("Failed to parse state diagram".into());
    }
    Ok(sd)
}

// ------------------------------------------------------------------ generation

impl App {
    /// The element the generated diagram goes in: the focused element when `ok` accepts it, else the
    /// project.
    /// The element a generated diagram goes in: the project when no base is given, else the base if
    /// the kind accepts it (the factories' "prepare base model" steps).
    pub(crate) fn mermaid_base(&self, base: Option<Eid>, ty: &str) -> Result<Eid, String> {
        match base {
            None => Ok(self.repo.root),
            Some(b) if crate::mermaid::base_ok(self, ty, b) => Ok(b),
            Some(_) => Err(crate::mermaid::base_error(ty).into()),
        }
    }

    /// Lay the new diagram out with Mermaid's separations (50 each) and keep it as one undo step.
    pub(crate) fn finish_mermaid(&mut self, d: Eid, undo: usize, direction: Dir, ms: &dyn Measure) -> Eid {
        self.open_diagram(d);
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        self.layout_diagram_with(direction, Seps { node: 50.0, edge: 50.0, rank: 50.0 });
        self.undo.truncate(undo + 1);
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        self.focus = Some(d);
        d
    }

    /// A Mermaid flowchart: an FCFlowchart "Flowchart by Mermaid" with its diagram, a node per vertex
    /// and a flow per link, owned by its source (createFlowchart, flowchart-factory.js:16-75).
    pub fn generate_flowchart(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let fc = parse_flowchart(code)?;
        let base = self.mermaid_base(base, "flowchart")?;
        let undo = self.undo.len();
        let d = self.change(|a| {
            let model = a.repo.create("FCFlowchart", &[("name", json!("Flowchart by Mermaid"))], Some((base, "ownedElements")));
            let d = a.repo.create("FCFlowchartDiagram", &[("name", json!("Flowchart Diagram by Mermaid"))], Some((model, "ownedElements")));
            let mut made = vec![];
            for (i, n) in fc.nodes.iter().enumerate() {
                let ty = fc_type(n.shape);
                let name = if n.text.is_empty() { n.id.clone() } else { n.text.clone() };
                let m = a.repo.create(ty, &[("name", json!(name))], Some((model, "ownedElements")));
                let v = crate::flowchart::create_view(&mut a.repo, d, m, R { x: 10.0 + i as f64, y: 10.0, w: 1.0, h: 1.0 });
                a.apply_view_defaults(v);
                made.push((n.id.clone(), m, v));
            }
            for (s, t, text) in &fc.edges {
                let (Some(sv), Some(tv)) = (made.iter().find(|x| &x.0 == s), made.iter().find(|x| &x.0 == t)) else { continue };
                let (rs, rt) = (a.repo.ref_to(sv.1), a.repo.ref_to(tv.1));
                let m = a.repo.create("FCFlow", &[("name", json!(text)), ("source", rs), ("target", rt)], Some((sv.1, "ownedElements")));
                if let Some(e) = views::create_edge_view(&mut a.repo, d, Some(m), sv.2, tv.2) {
                    a.apply_view_defaults(e);
                }
            }
            Some(d)
        });
        let d = d.ok_or("Failed to parse flowchart")?;
        Ok(self.finish_mermaid(d, undo, dir(&fc.direction, true), ms))
    }

    /// A Mermaid ER diagram: an ERDDataModel "Data Model by Mermaid", entities with their columns,
    /// and relationships owned by their first entity (createErDiagram, er-factory.js:15-160).
    /// UMLCraft reads "--" as identifying and ".." as not; the original compares an object with a
    /// string there and so makes every relationship non-identifying (er-factory.js:46).
    pub fn generate_er(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let er = parse_er(code)?;
        let base = self.mermaid_base(base, "er")?;
        let undo = self.undo.len();
        let d = self.change(|a| {
            let model = a.repo.create("ERDDataModel", &[("name", json!("Data Model by Mermaid"))], Some((base, "ownedElements")));
            let d = a.repo.create("ERDDiagram", &[("name", json!("ER Diagram by Mermaid"))], Some((model, "ownedElements")));
            let mut made = vec![];
            for (i, e) in er.entities.iter().enumerate() {
                let name = if e.label.is_empty() { e.id.clone() } else { e.label.clone() };
                let m = a.repo.create("ERDEntity", &[("name", json!(name))], Some((model, "ownedElements")));
                for c in &e.columns {
                    let col = a.repo.create("ERDColumn", &[("name", json!(c.name)), ("type", json!(c.ty))], Some((m, "columns")));
                    for (key, field) in [("PK", "primaryKey"), ("FK", "foreignKey"), ("UK", "unique")] {
                        if c.keys.iter().any(|k| k == key) {
                            a.repo.set(col, field, json!(true));
                        }
                    }
                    if !c.comment.is_empty() {
                        a.repo.set(col, "documentation", json!(c.comment));
                    }
                }
                let v = crate::erd::create_view(&mut a.repo, d, m, R { x: 10.0 + i as f64, y: 10.0, w: 1.0, h: 1.0 });
                a.apply_view_defaults(v);
                made.push((e.id.clone(), m, v));
            }
            for rel in &er.rels {
                let (Some(x), Some(y)) = (made.iter().find(|m| m.0 == rel.a), made.iter().find(|m| m.0 == rel.b)) else { continue };
                let m = a.repo.create("ERDRelationship", &[("name", json!(rel.label))], Some((x.1, "ownedElements")));
                if !rel.identifying {
                    a.repo.set(m, "identifying", json!(false));
                }
                for (k, ent, card) in [("end1", x.1, rel.card_a), ("end2", y.1, rel.card_b)] {
                    let rf = a.repo.ref_to(ent);
                    let end = a.repo.create("ERDRelationshipEnd", &[("reference", rf)], Some((m, k)));
                    a.repo.set(end, "cardinality", json!(card));
                }
                if let Some(e) = views::create_edge_view(&mut a.repo, d, Some(m), x.2, y.2) {
                    a.apply_view_defaults(e);
                }
            }
            Some(d)
        });
        let d = d.ok_or("Failed to parse ER diagram")?;
        Ok(self.finish_mermaid(d, undo, dir(&er.direction, true), ms))
    }

    /// A Mermaid mind map: an MMMindmap "Mindmap by Mermaid", a node per line and an edge from each
    /// child to its parent (createMindmap, mindmap-factory.js:20-110).
    pub fn generate_mindmap(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let root = parse_mindmap(code)?;
        let base = self.mermaid_base(base, "mindmap")?;
        let undo = self.undo.len();
        let d = self.change(|a| {
            let model = a.repo.create("MMMindmap", &[("name", json!("Mindmap by Mermaid"))], Some((base, "ownedElements")));
            let d = a.repo.create("MMMindmapDiagram", &[("name", json!("Mindmap Diagram by Mermaid"))], Some((model, "ownedElements")));
            fn add(a: &mut App, model: Eid, d: Eid, n: &MNode, i: &mut f64) -> (Eid, Eid) {
                let m = a.repo.create("MMNode", &[("name", json!(n.text))], Some((model, "ownedElements")));
                let v = crate::notations::create_view(&mut a.repo, d, m, R { x: 10.0 + *i, y: 10.0, w: 1.0, h: 1.0 });
                *i += 1.0;
                a.apply_view_defaults(v);
                for c in &n.children {
                    let (cm, cv) = add(a, model, d, c, i);
                    let (rs, rt) = (a.repo.ref_to(cm), a.repo.ref_to(m));
                    let e = a.repo.create("MMEdge", &[("source", rs), ("target", rt)], Some((m, "ownedElements")));
                    if let Some(ev) = views::create_edge_view(&mut a.repo, d, Some(e), cv, v) {
                        a.apply_view_defaults(ev);
                    }
                }
                (m, v)
            }
            let mut i = 0.0;
            add(a, model, d, &root, &mut i);
            Some(d)
        });
        let d = d.ok_or("Failed to parse mindmap")?;
        Ok(self.finish_mermaid(d, undo, Dir::TopBottom, ms))
    }

    /// A Mermaid state diagram: a UMLStateMachine "Statemachine by Mermaid" with a region and its
    /// statechart, states and pseudostates, transitions in the region, and notes linked to their
    /// states (createStateDiagram, state-factory.js:15-130).
    pub fn generate_state(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let sd = parse_state(code)?;
        let base = self.mermaid_base(base, "state")?;
        let undo = self.undo.len();
        let d = self.change(|a| {
            let sm = a.repo.create("UMLStateMachine", &[("name", json!("Statemachine by Mermaid"))], Some((base, "ownedElements")));
            let region = a.repo.create("UMLRegion", &[], Some((sm, "regions")));
            let d = a.repo.create("UMLStatechartDiagram", &[("name", json!("Statechart Diagram by Mermaid"))], Some((sm, "ownedElements")));
            let mut made = vec![];
            for (i, n) in sd.nodes.iter().enumerate() {
                let (ty, kind) = match n.shape {
                    "stateStart" => ("UMLPseudostate", Some("initial")),
                    "stateEnd" => ("UMLFinalState", None),
                    "choice" | "fork" | "join" => ("UMLPseudostate", Some(n.shape)),
                    _ => ("UMLState", None),
                };
                let name = if n.label.is_empty() { n.id.clone() } else { n.label.clone() };
                let m = a.repo.create(ty, &[("name", json!(name))], Some((region, "vertices")));
                if let Some(k) = kind {
                    a.repo.set(m, "kind", json!(k));
                }
                let v = views::create_node_view(&mut a.repo, d, m, R { x: 10.0 + i as f64, y: 10.0, w: 1.0, h: 1.0 })?;
                a.apply_view_defaults(v);
                made.push((n.id.clone(), m, v));
            }
            for (s, t, label) in &sd.edges {
                let (Some(x), Some(y)) = (made.iter().find(|m| &m.0 == s), made.iter().find(|m| &m.0 == t)) else { continue };
                let (rs, rt) = (a.repo.ref_to(x.1), a.repo.ref_to(y.1));
                let m = a.repo.create("UMLTransition", &[("name", json!(label)), ("source", rs), ("target", rt)], Some((region, "transitions")));
                if let Some(e) = views::create_edge_view(&mut a.repo, d, Some(m), x.2, y.2) {
                    a.apply_view_defaults(e);
                }
            }
            for (i, (target, text)) in sd.notes.iter().enumerate() {
                let n = views::create_note_view(&mut a.repo, d, R { x: 20.0 + i as f64 * 160.0, y: 10.0, w: 140.0, h: 50.0 }, "UMLNoteView");
                a.apply_view_defaults(n);
                a.repo.set(n, "text", json!(text));
                a.repo.set(n, "wordWrap", json!(false));
                a.repo.set(n, "autoResize", json!(true));
                if let Some(x) = made.iter().find(|m| &m.0 == target) {
                    let tool = Tool { label: "", kind: "NoteLink", icon: "", line: true, init: AssocInit::Plain };
                    a.create_edge(&tool, x.2, n);
                }
            }
            Some(d)
        });
        let d = d.ok_or("Failed to parse state diagram")?;
        Ok(self.finish_mermaid(d, undo, dir(&sd.direction, true), ms))
    }
}
