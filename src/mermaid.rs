//! Tools > Generate Diagram by Mermaid: build a class diagram from Mermaid `classDiagram` code
//! (extensions/default/mermaid, factory/class-factory.js).
//!
//! UMLCraft reads Mermaid's class diagram syntax itself: classes (with labels, generics and
//! annotations), members, namespaces, relations with cardinalities and labels, lollipop interfaces,
//! notes and `direction`. Flowcharts, ER diagrams, mind maps and state diagrams are made by
//! mermaid_kinds.rs, sequence diagrams by mermaid_sequence.rs; requirement diagrams are refused.

use crate::app::{App, AssocInit, Tool};
use crate::layout::{Dir, Seps};
use crate::model::Eid;
use crate::views::{self, Measure, P, R};
use serde_json::json;

/// The dialog's starting text.
pub const SAMPLE: &str = "classDiagram
    Vehicle <|-- Car
    Vehicle <|-- Bicycle
    Vehicle : +int wheels
    Vehicle : +String brand
    Vehicle: +start()
    Vehicle: +stop() bool
    class Car{
        +int doors
        +open(int door)
    }
    class Bicycle{
        -bool hasBell
        +ring()
    }
    Garage \"1\" o-- \"*\" Vehicle : parks
";

/// A parsed class.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MClass {
    pub id: String,
    pub label: String,
    pub generics: Vec<String>,
    pub annotations: Vec<String>,
    pub members: Vec<String>,
    pub namespace: Option<String>,
    pub lollipop: bool,
}

/// A parsed relation: from, to, kinds of each end, dashed, cardinalities and label.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MRelation {
    pub a: String,
    pub b: String,
    /// The decoration at a's end and at b's end: "<|" / "|>" (inheritance), "*", "o", "<" / ">", "()" or "".
    pub end_a: String,
    pub end_b: String,
    pub dashed: bool,
    pub card_a: String,
    pub card_b: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MDiagram {
    pub direction: String,
    pub classes: Vec<MClass>,
    pub relations: Vec<MRelation>,
    pub notes: Vec<(Option<String>, String)>,
}

impl MDiagram {
    fn class_mut(&mut self, id: &str, ns: &Option<String>) -> &mut MClass {
        if let Some(i) = self.classes.iter().position(|c| c.id == id) {
            if self.classes[i].namespace.is_none() {
                self.classes[i].namespace = ns.clone();
            }
            return &mut self.classes[i];
        }
        self.classes.push(MClass { id: id.to_string(), label: id.to_string(), namespace: ns.clone(), ..Default::default() });
        self.classes.last_mut().unwrap()
    }
}

/// The Mermaid diagram type: the first keyword of the code (utils.js:1).
pub fn diagram_type(code: &str) -> Result<&'static str, String> {
    let first = code.lines().map(|l| l.trim()).find(|l| !l.is_empty() && !l.starts_with("%%")).unwrap_or("");
    let word = first.split_whitespace().next().unwrap_or("");
    Ok(match word {
        "classDiagram" | "classDiagram-v2" => "class",
        "sequenceDiagram" => "sequence",
        "flowchart" => "flowchart",
        "erDiagram" => "er",
        "mindmap" => "mindmap",
        "requirementDiagram" => "requirement",
        w if w.starts_with("stateDiagram") => "state",
        _ => return Err("Unsupported diagram type".into()),
    })
}

/// A class reference: `Name`, `Name~T~`, `Name["Label"]`, with `:::style` ignored.
fn class_ref(s: &str) -> (String, Vec<String>, Option<String>) {
    let s = s.trim();
    let s = s.split(":::").next().unwrap_or(s).trim();
    let (s, label) = match s.find('[') {
        Some(i) if s.ends_with(']') => (&s[..i], Some(s[i + 1..s.len() - 1].trim().trim_matches(['"', '`']).to_string())),
        _ => (s, None),
    };
    let (name, generics) = match s.split_once('~') {
        Some((n, g)) => (n, g.trim_end_matches('~').split(',').map(|x| x.trim().replace('~', "")).filter(|x| !x.is_empty()).collect()),
        None => (s, vec![]),
    };
    (name.trim().trim_matches('`').to_string(), generics, label)
}

/// The arrows of a relation, longest first, with the decoration at each end.
const ARROWS: [(&str, &str, &str, bool); 16] = [
    ("<|--", "<|", "", false),
    ("--|>", "", "|>", false),
    ("<|..", "<|", "", true),
    ("..|>", "", "|>", true),
    ("*--", "*", "", false),
    ("--*", "", "*", false),
    ("o--", "o", "", false),
    ("--o", "", "o", false),
    ("<--", "<", "", false),
    ("-->", "", ">", false),
    ("<..", "<", "", true),
    ("..>", "", ">", true),
    ("()--", "()", "", false),
    ("--()", "", "()", false),
    ("--", "", "", false),
    ("..", "", "", true),
];

fn parse_relation(line: &str) -> Option<MRelation> {
    let (body, label) = match line.split_once(" : ").or_else(|| line.rsplit_once(':').filter(|(a, _)| a.contains("--") || a.contains(".."))) {
        Some((b, l)) => (b, l.trim().to_string()),
        None => (line, String::new()),
    };
    for (arrow, ea, eb, dashed) in ARROWS {
        let Some(i) = body.find(arrow) else { continue };
        let (left, right) = (body[..i].trim(), body[i + arrow.len()..].trim());
        let split_card = |s: &str, at_end: bool| -> (String, String) {
            // a quoted cardinality next to the arrow
            let q: Vec<&str> = s.split('"').collect();
            if q.len() >= 3 {
                if at_end {
                    (q[0].trim().to_string(), q[1].to_string())
                } else {
                    (q[2].trim().to_string(), q[1].to_string())
                }
            } else {
                (s.to_string(), String::new())
            }
        };
        let (a, card_a) = split_card(left, true);
        let (b, card_b) = split_card(right, false);
        if a.is_empty() || b.is_empty() || a.contains(' ') || b.contains(' ') {
            return None;
        }
        return Some(MRelation { a: class_ref(&a).0, b: class_ref(&b).0, end_a: ea.into(), end_b: eb.into(), dashed, card_a, card_b, label });
    }
    None
}

/// Parse Mermaid `classDiagram` code.
pub fn parse_class_diagram(code: &str) -> Result<MDiagram, String> {
    if diagram_type(code)? != "class" {
        return Err("Unsupported diagram type".into());
    }
    let mut d = MDiagram { direction: "TB".into(), ..Default::default() };
    let mut ns: Option<String> = None;
    let mut open_class: Option<String> = None;
    for raw in code.lines().skip_while(|l| !l.trim().starts_with("classDiagram")).skip(1) {
        let line = raw.split("%%").next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(c) = open_class.clone() {
            if line == "}" {
                open_class = None;
            } else if let Some(a) = line.strip_prefix("<<").and_then(|x| x.strip_suffix(">>")) {
                d.class_mut(&c, &ns).annotations.push(a.trim().to_string());
            } else {
                d.class_mut(&c, &ns).members.push(line.to_string());
            }
            continue;
        }
        if line == "}" {
            ns = None;
            continue;
        }
        if let Some(dir) = line.strip_prefix("direction ") {
            d.direction = dir.trim().to_string();
        } else if let Some(n) = line.strip_prefix("namespace ") {
            ns = Some(n.trim_end_matches('{').trim().to_string());
        } else if let Some(rest) = line.strip_prefix("note ") {
            let rest = rest.trim();
            let (target, text) = match rest.strip_prefix("for ") {
                Some(r) => {
                    let (t, txt) = r.split_once(char::is_whitespace).unwrap_or((r, ""));
                    (Some(t.to_string()), txt)
                }
                None => (None, rest),
            };
            d.notes.push((target, text.trim().trim_matches('"').replace("\\n", "\n")));
        } else if let Some(rest) = line.strip_prefix("class ") {
            let rest = rest.trim();
            let (head, body) = match rest.find('{') {
                Some(i) => (&rest[..i], Some(rest[i + 1..].trim())),
                None => (rest, None),
            };
            let (id, generics, label) = class_ref(head);
            let c = d.class_mut(&id, &ns);
            if !generics.is_empty() {
                c.generics = generics;
            }
            if let Some(l) = label {
                c.label = l;
            }
            match body {
                Some(b) if b.ends_with('}') => {
                    let inner = b.trim_end_matches('}').trim();
                    if !inner.is_empty() {
                        c.members.push(inner.to_string());
                    }
                }
                Some(_) => open_class = Some(id),
                None => {}
            }
        } else if let Some(a) = line.strip_prefix("<<") {
            // <<interface>> Name
            if let Some((ann, name)) = a.split_once(">>") {
                d.class_mut(name.trim(), &ns).annotations.push(ann.trim().to_string());
            }
        } else if let Some(r) = parse_relation(line) {
            for (id, end) in [(&r.a, &r.end_a), (&r.b, &r.end_b)] {
                let c = d.class_mut(id, &ns);
                if end == "()" {
                    c.lollipop = true;
                }
            }
            d.relations.push(r);
        } else if let Some((cls, member)) = line.split_once(':') {
            // Name : +member
            let (id, _, _) = class_ref(cls);
            if !id.is_empty() && !id.contains(' ') {
                d.class_mut(&id, &ns).members.push(member.trim().to_string());
            }
        } else if !line.contains(' ') {
            d.class_mut(&class_ref(line).0, &ns);
        } else {
            return Err(format!("Cannot read \"{line}\""));
        }
    }
    Ok(d)
}

/// A Mermaid member (`+int age`, `+eat(String food) bool`, with `$` static and `*` abstract) as
/// UMLCraft's inline syntax (`+age: int`, `+eat(food: String): bool`), plus static and abstract.
pub fn member_text(m: &str) -> (String, bool, bool, bool) {
    let mut m = m.trim().replace('~', "<");
    // List~int~ becomes List<int>: close each opened bracket
    let mut fixed = String::new();
    let mut open = false;
    for ch in m.chars() {
        if ch == '<' {
            fixed.push(if open { '>' } else { '<' });
            open = !open;
        } else {
            fixed.push(ch);
        }
    }
    m = fixed;
    let mut is_static = false;
    let mut is_abstract = false;
    if let Some(x) = m.strip_suffix('$') {
        is_static = true;
        m = x.trim().to_string();
    }
    if let Some(x) = m.strip_suffix('*') {
        is_abstract = true;
        m = x.trim().to_string();
    }
    let vis = match m.chars().next() {
        Some(c @ ('+' | '-' | '#' | '~')) => {
            m = m[1..].to_string();
            c.to_string()
        }
        _ => "~".to_string(),
    };
    let swap = |s: &str| -> String {
        // "Type name" becomes "name: Type"; a single word stays the name
        let s = s.trim();
        match s.rsplit_once(char::is_whitespace) {
            Some((t, n)) => format!("{}: {}", n.trim(), t.trim()),
            None => s.to_string(),
        }
    };
    if let (Some(o), Some(c)) = (m.find('('), m.rfind(')')) {
        let name = m[..o].trim();
        let params: Vec<String> = m[o + 1..c].split(',').map(|p| p.trim()).filter(|p| !p.is_empty()).map(swap).collect();
        let ret = m[c + 1..].trim();
        let text = if ret.is_empty() { format!("{vis}{name}({})", params.join(", ")) } else { format!("{vis}{name}({}): {ret}", params.join(", ")) };
        (text, true, is_static, is_abstract)
    } else {
        (format!("{vis}{}", swap(&m)), false, is_static, is_abstract)
    }
}

/// Whether a diagram of the Mermaid kind `ty` can be generated into `base` (the factories' base
/// checks: class-factory.js:40-50, sequence-factory.js:66-75, er-factory.js:31-37,
/// flowchart-factory.js:31-37, mindmap-factory.js:32-38, state-factory.js:36-46). The class
/// factory also takes the project, where it makes a "Package by Mermaid".
pub fn base_ok(a: &App, ty: &str, base: Eid) -> bool {
    let r = &a.repo;
    if !r.alive(base) || a.is_diagram(base) {
        return false;
    }
    match ty {
        "class" => base == r.root || r.kind_of(base, "UMLPackage"),
        "er" => r.kind_of(base, "UMLPackage"),
        "sequence" | "state" => r.kind_of(base, "UMLPackage") || r.kind_of(base, "UMLClassifier") || r.ty(base) == "UMLOperation",
        _ => r.kind_of(base, "ExtensibleModel"),
    }
}

/// The error when the base does not fit (the same factories).
pub fn base_error(ty: &str) -> &'static str {
    match ty {
        "class" | "er" => "Base element must be a UMLPackage",
        "sequence" | "state" => "Base element must be a package, classifer or operation",
        _ => "Base element must be a model element",
    }
}

impl App {
    /// The dialog's and the API's generation: into the element focused in the explorer when the kind
    /// accepts it, otherwise as with no base (the original always passes no base,
    /// main.js:135-150; using the focused element is UMLCraft's choice).
    pub fn generate_mermaid(&mut self, code: &str, ms: &dyn Measure) -> Result<Eid, String> {
        let ty = diagram_type(code)?;
        let base = self.focus.filter(|f| *f != self.repo.root && base_ok(self, ty, *f));
        self.generate_mermaid_in(code, base, ms)
    }

    /// `mermaid:generate-diagram(code, base?)`: generate the diagram of the code's kind, in one undo
    /// step, laid out and opened, in `base` or by default in the project (handleGenerateDiagram,
    /// main.js:155-185). Returns the diagram.
    pub fn generate_mermaid_in(&mut self, code: &str, base: Option<Eid>, ms: &dyn Measure) -> Result<Eid, String> {
        let ty = diagram_type(code)?;
        match ty {
            "flowchart" => return self.generate_flowchart(code, base, ms),
            "er" => return self.generate_er(code, base, ms),
            "mindmap" => return self.generate_mindmap(code, base, ms),
            "state" => return self.generate_state(code, base, ms),
            "sequence" => return self.generate_sequence(code, base, ms),
            "class" => {}
            _ => return Err(format!("UMLCraft cannot generate {ty} diagrams from Mermaid yet.")),
        }
        let md = parse_class_diagram(code)?;
        // the container: the base package, or a new "Package by Mermaid" in the project
        let base = match base {
            Some(b) if b != self.repo.root && base_ok(self, "class", b) => Some(b),
            Some(b) if b != self.repo.root => return Err(base_error("class").into()),
            _ => None,
        };
        let undo = self.undo.len();
        let d = self.change(|a| {
            let root = a.repo.root;
            let pkg = base.unwrap_or_else(|| a.repo.create("UMLPackage", &[("name", json!("Package by Mermaid"))], Some((root, "ownedElements"))));
            let d = a.add_diagram(Some(pkg), "UMLClassDiagram");
            a.repo.set(d, "name", json!("Class Diagram by Mermaid"));
            // namespaces become packages (no view)
            let mut spaces: Vec<(String, Eid)> = vec![];
            for c in &md.classes {
                if let Some(n) = &c.namespace {
                    if !spaces.iter().any(|s| &s.0 == n) {
                        let p = a.repo.create("UMLPackage", &[("name", json!(n))], Some((pkg, "ownedElements")));
                        spaces.push((n.clone(), p));
                    }
                }
            }
            // classes first, so that member types can refer to them
            let mut made: Vec<(String, Eid)> = vec![];
            for c in &md.classes {
                let owner = c.namespace.as_ref().and_then(|n| spaces.iter().find(|s| &s.0 == n)).map(|s| s.1).unwrap_or(pkg);
                let lower: Vec<String> = c.annotations.iter().map(|x| x.to_lowercase()).collect();
                let kind = if c.lollipop || lower.iter().any(|x| x == "interface") {
                    "UMLInterface"
                } else if lower.iter().any(|x| x == "enumeration") {
                    "UMLEnumeration"
                } else {
                    "UMLClass"
                };
                let e = a.repo.create(kind, &[("name", json!(c.label))], Some((owner, "ownedElements")));
                if lower.iter().any(|x| x == "abstract") {
                    a.repo.set(e, "isAbstract", json!(true));
                }
                if let Some(st) = c.annotations.iter().find(|x| !["interface", "enumeration", "abstract"].contains(&x.to_lowercase().as_str())) {
                    a.repo.set(e, "stereotype", json!(st));
                }
                for g in &c.generics {
                    a.repo.create("UMLTemplateParameter", &[("name", json!(g))], Some((e, "templateParameters")));
                }
                made.push((c.id.clone(), e));
            }
            for c in &md.classes {
                let e = made.iter().find(|m| m.0 == c.id).unwrap().1;
                for m in &c.members {
                    let (text, is_op, is_static, is_abstract) = member_text(m);
                    let f = if a.repo.ty(e) == "UMLEnumeration" && !is_op {
                        a.add_feature(e, "UMLEnumerationLiteral")
                    } else {
                        a.add_feature(e, if is_op { "UMLOperation" } else { "UMLAttribute" })
                    };
                    let Some(f) = f else { continue };
                    if is_op {
                        crate::parse::apply_operation(&mut a.repo, f, &text);
                    } else {
                        crate::parse::apply_attribute(&mut a.repo, f, &text);
                    }
                    if is_static {
                        a.repo.set(f, "isStatic", json!(true));
                    }
                    if is_abstract && is_op {
                        a.repo.set(f, "isAbstract", json!(true));
                    }
                }
            }
            // relationships, owned by their source
            let of = |id: &str| made.iter().find(|m| m.0 == id).map(|m| m.1);
            for r in &md.relations {
                let (Some(x), Some(y)) = (of(&r.a), of(&r.b)) else { continue };
                let directed = |a: &mut App, kind: &str, s: Eid, t: Eid| {
                    let (rs, rt) = (a.repo.ref_to(s), a.repo.ref_to(t));
                    let e = a.repo.create(kind, &[("source", rs), ("target", rt)], Some((s, "ownedElements")));
                    if !r.label.is_empty() {
                        a.repo.set(e, "name", json!(r.label));
                    }
                };
                match (r.end_a.as_str(), r.end_b.as_str(), r.dashed) {
                    ("<|", _, false) => directed(a, "UMLGeneralization", y, x),
                    (_, "|>", false) => directed(a, "UMLGeneralization", x, y),
                    ("<|", _, true) => directed(a, "UMLInterfaceRealization", y, x),
                    (_, "|>", true) => directed(a, "UMLInterfaceRealization", x, y),
                    ("()", _, _) => directed(a, "UMLInterfaceRealization", y, x),
                    (_, "()", _) => directed(a, "UMLInterfaceRealization", x, y),
                    ("<", _, true) => directed(a, "UMLDependency", y, x),
                    (_, _, true) => directed(a, "UMLDependency", x, y),
                    (ea, eb, false) => {
                        let m = a.repo.create("UMLAssociation", &[], Some((x, "ownedElements")));
                        if !r.label.is_empty() {
                            a.repo.set(m, "name", json!(r.label));
                        }
                        for (k, cls, deco, card) in [("end1", x, ea, &r.card_a), ("end2", y, eb, &r.card_b)] {
                            let rf = a.repo.ref_to(cls);
                            let end = a.repo.create("UMLAssociationEnd", &[("reference", rf)], Some((m, k)));
                            match deco {
                                "*" => a.repo.set(end, "aggregation", json!("composite")),
                                "o" => a.repo.set(end, "aggregation", json!("shared")),
                                "<" | ">" => a.repo.set(end, "navigable", json!("navigable")),
                                _ => {}
                            }
                            if !card.is_empty() {
                                a.repo.set(end, "multiplicity", json!(card));
                            }
                        }
                    }
                }
            }
            // views, lines and notes
            let mut placed = vec![];
            for (i, (_, e)) in made.iter().enumerate() {
                let p = P { x: 20.0 + (i % 6) as f64 * 220.0, y: 20.0 + (i / 6) as f64 * 200.0 };
                if let Some(v) = a.place_node(d, *e, p) {
                    if a.repo.ty(*e) == "UMLInterface" && md.classes.iter().any(|c| c.lollipop && made.iter().any(|m| m.0 == c.id && m.1 == *e)) {
                        a.repo.set(v, "stereotypeDisplay", json!("icon"));
                    }
                    placed.push((*e, v));
                }
            }
            a.connect_all(d, &placed, |_| true);
            for (i, (target, text)) in md.notes.iter().enumerate() {
                let n = a.create_node("Note", R { x: 20.0 + i as f64 * 160.0, y: 600.0, w: 140.0, h: 50.0 })?;
                a.repo.set(n, "text", json!(text));
                a.repo.set(n, "autoResize", json!(true));
                a.repo.set(n, "wordWrap", json!(false));
                if let Some(v) = target.as_deref().and_then(|t| of(t)).and_then(|e| placed.iter().find(|p| p.0 == e)).map(|p| p.1) {
                    let tool = Tool { label: "", kind: "NoteLink", icon: "", line: true, init: AssocInit::Plain };
                    a.create_edge(&tool, n, v);
                }
            }
            Some(d)
        });
        let d = d.ok_or("The diagram could not be created.")?;
        // laid out with the real sizes, as part of the same undo step
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        let dir = match md.direction.as_str() {
            "BT" => Dir::BottomTop,
            "LR" => Dir::LeftRight,
            "RL" => Dir::RightLeft,
            _ => Dir::TopBottom,
        };
        self.layout_diagram_with(dir, Seps { node: 50.0, edge: 50.0, rank: 50.0 });
        self.undo.truncate(undo + 1);
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        self.focus = Some(d);
        Ok(d)
    }

    /// The Mermaid dialog: the code, a Clear button, OK and Cancel; errors show as a toast.
    pub fn mermaid_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut code) = self.mermaid_code.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        egui::Window::new("Generate Diagram by Mermaid").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            ui.horizontal(|ui| {
                // the Examples menu (main.js:185-205; the examples are UMLCraft's own)
                ui.menu_button("Examples", |ui| {
                    for (label, text) in [
                        ("Class Diagram", SAMPLE),
                        ("Sequence Diagram", crate::mermaid_sequence::SEQUENCE_SAMPLE),
                        ("State Diagram", crate::mermaid_kinds::STATE_SAMPLE),
                        ("Flowchart", crate::mermaid_kinds::FLOWCHART_SAMPLE),
                        ("Entity Relationship Diagram", crate::mermaid_kinds::ER_SAMPLE),
                        ("Mindmap", crate::mermaid_kinds::MINDMAP_SAMPLE),
                    ] {
                        if ui.button(label).clicked() {
                            code = text.to_string();
                            ui.close();
                        }
                    }
                });
                if ui.button("Clear").clicked() {
                    code.clear();
                }
            });
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap: f32| {
                    let mut job = highlight(text.as_str());
                    job.wrap.max_width = wrap;
                    ui.fonts_mut(|f| f.layout_job(job))
                };
                ui.add(egui::TextEdit::multiline(&mut code).code_editor().layouter(&mut layouter).desired_width(520.0).desired_rows(18));
            });
            ui.horizontal(|ui| {
                ok = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
            });
            // the code is a textarea: Ctrl+Enter is OK, Esc cancels (dialog-manager.js:148-162)
            ok |= ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter));
            cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if ok {
            let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
            match self.generate_mermaid(&code, &ms) {
                Ok(_) => self.mermaid_code = None,
                Err(e) => {
                    self.toast_error(&e);
                    self.mermaid_code = Some(code);
                }
            }
        } else if cancel {
            self.mermaid_code = None;
        } else {
            self.mermaid_code = Some(code);
        }
    }
}

/// Mermaid source with colours, for the dialog: `%%` comments, quoted strings, `<<stereotypes>>`,
/// relationship arrows and the keywords (the original uses a CodeMirror mode; the colours are UMLCraft's).
pub fn highlight(text: &str) -> egui::text::LayoutJob {
    use egui::{text::TextFormat, Color32, FontId};
    let mono = FontId::monospace(12.0);
    let fmt = |c: Color32| TextFormat { font_id: mono.clone(), color: c, ..Default::default() };
    let (plain, keyword, arrow, comment, string) =
        (crate::ui::TEXT(), Color32::from_rgb(0x4f, 0x99, 0xff), Color32::from_rgb(0xd8, 0xa0, 0x3c), Color32::from_gray(0x80), Color32::from_rgb(0x8f, 0xbf, 0x6f));
    const KEYWORDS: [&str; 12] = ["classDiagram", "classDiagram-v2", "class", "direction", "note", "for", "namespace", "TB", "BT", "LR", "RL", "TD"];
    let mut job = egui::text::LayoutJob::default();
    let mut rest = text;
    while !rest.is_empty() {
        // a comment runs to the end of the line
        if rest.starts_with("%%") {
            let end = rest.find('\n').unwrap_or(rest.len());
            job.append(&rest[..end], 0.0, fmt(comment));
            rest = &rest[end..];
            continue;
        }
        for (open, close, color) in [("\"", "\"", string), ("<<", ">>", keyword)] {
            if rest.starts_with(open) {
                if let Some(k) = rest[open.len()..].find(close) {
                    let end = open.len() + k + close.len();
                    job.append(&rest[..end], 0.0, fmt(color));
                    rest = &rest[end..];
                }
            }
        }
        if rest.is_empty() {
            break;
        }
        let c = rest.chars().next().unwrap_or(' ');
        // an arrow: a run of < > | * o . - characters, two or more, with a - or ..
        if "<>|*.-".contains(c) {
            let end = rest.find(|x: char| !"<>|*o.-".contains(x)).unwrap_or(rest.len());
            let run = &rest[..end];
            // a lone "-" is a visibility, not an arrow
            let is_arrow = run.chars().count() >= 2 && (run.contains('-') || run.contains(".."));
            job.append(run, 0.0, fmt(if is_arrow { arrow } else { plain }));
            rest = &rest[end..];
            continue;
        }
        if rest.starts_with("classDiagram-v2") {
            job.append("classDiagram-v2", 0.0, fmt(keyword));
            rest = &rest["classDiagram-v2".len()..];
            continue;
        }
        if c.is_alphanumeric() || c == '_' {
            let end = rest.find(|x: char| !(x.is_alphanumeric() || x == '_')).unwrap_or(rest.len());
            let word = &rest[..end];
            job.append(word, 0.0, fmt(if KEYWORDS.contains(&word) { keyword } else { plain }));
            rest = &rest[end..];
            continue;
        }
        let end = c.len_utf8();
        job.append(&rest[..end], 0.0, fmt(plain));
        rest = &rest[end..];
    }
    job
}
