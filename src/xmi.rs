//! File > Export > XMI Export (v2.1): the model as OMG XMI 2.1 with the UML 2 vocabulary, model
//! elements only, no diagrams (https://docs.staruml.io/user-guide/managing-project/).
//!
//! The layout of the file is UMLCraft's own (the StarUML exporter was not read): packages hold
//! their elements as `packagedElement`; a classifier holds its attributes, operations, literals and
//! generalizations; associations, dependencies and the other relationships are written in the
//! package that holds them, or in the nearest package above their owner. String types (an attribute
//! typed "int" by text) have no element to point at and are left out.

use crate::app::App;
use crate::model::{Eid, Repo};
use std::fmt::Write;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Relationships written at package level (an InterfaceRealization or Generalization stays inside
/// its classifier, as UML 2 has it).
const PACKAGE_LEVEL: [&str; 9] = ["UMLAssociation", "UMLDependency", "UMLAbstraction", "UMLRealization", "UMLComponentRealization", "UMLDeployment", "UMLCommunicationPath", "UMLLink", "UMLTemplateBinding"];

struct Writer<'a> {
    r: &'a Repo,
    out: String,
}

impl Writer<'_> {
    fn id(&self, e: Eid) -> String {
        esc(&self.r.id(e))
    }

    /// `name="…" visibility="…"` and the boolean flags this element has set.
    fn common(&self, e: Eid) -> String {
        let r = self.r;
        let mut a = format!(" xmi:id=\"{}\"", self.id(e));
        let name = r.name(e);
        if !name.is_empty() {
            let _ = write!(a, " name=\"{}\"", esc(&name));
        }
        if crate::meta::order_of(r.ty(e), "visibility").is_some() && !r.kind_of(e, "Relationship") {
            let _ = write!(a, " visibility=\"{}\"", esc(&r.str(e, "visibility")));
        }
        for f in ["isAbstract", "isStatic", "isLeaf", "isReadOnly", "isDerived", "isQuery", "isOrdered", "isActive"] {
            if r.bool(e, f) {
                let _ = write!(a, " {f}=\"true\"");
            }
        }
        a
    }

    /// `type="id"` when the type is a reference to an element.
    fn type_attr(&self, e: Eid, field: &str) -> String {
        match self.r.refd(e, field) {
            Some(t) => format!(" type=\"{}\"", self.id(t)),
            None => String::new(),
        }
    }

    fn doc(&mut self, e: Eid, ind: &str) {
        let d = self.r.str(e, "documentation");
        if !d.trim().is_empty() {
            let _ = writeln!(self.out, "{ind}<ownedComment xmi:type=\"uml:Comment\" xmi:id=\"{}-comment\"><body>{}</body></ownedComment>", self.id(e), esc(&d));
        }
    }

    fn multiplicity(&mut self, e: Eid, ind: &str) {
        let m = self.r.str(e, "multiplicity");
        let (lo, hi) = match m.trim() {
            "" => return,
            "*" => ("0", "*"),
            x => match x.split_once("..") {
                Some((a, b)) => (a.trim(), b.trim()),
                None => (x, x),
            },
        };
        let id = self.id(e);
        let _ = writeln!(self.out, "{ind}<lowerValue xmi:type=\"uml:LiteralInteger\" xmi:id=\"{id}-lower\" value=\"{}\"/>", esc(lo));
        let _ = writeln!(self.out, "{ind}<upperValue xmi:type=\"uml:LiteralUnlimitedNatural\" xmi:id=\"{id}-upper\" value=\"{}\"/>", esc(hi));
    }

    /// A model element and what it holds; `tag` is the containing property (packagedElement…).
    fn element(&mut self, e: Eid, tag: &str, depth: usize) {
        let r = self.r;
        let ind = "\t".repeat(depth);
        let t = r.ty(e).to_string();
        let uml = t.trim_start_matches("UML");
        let mut attrs = self.common(e);
        match t.as_str() {
            "UMLAttribute" | "UMLSlot" => {
                attrs += &self.type_attr(e, "type");
                if r.str(e, "aggregation") != "none" && !r.str(e, "aggregation").is_empty() {
                    let _ = write!(attrs, " aggregation=\"{}\"", esc(&r.str(e, "aggregation")));
                }
            }
            "UMLParameter" => {
                attrs += &self.type_attr(e, "type");
                let _ = write!(attrs, " direction=\"{}\"", esc(&r.str(e, "direction")));
            }
            "UMLGeneralization" => {
                if let Some(g) = r.refd(e, "target") {
                    let _ = write!(attrs, " general=\"{}\"", self.id(g));
                }
            }
            "UMLInclude" => {
                if let Some(g) = r.refd(e, "target") {
                    let _ = write!(attrs, " addition=\"{}\"", self.id(g));
                }
            }
            "UMLExtend" => {
                if let Some(g) = r.refd(e, "target") {
                    let _ = write!(attrs, " extendedCase=\"{}\"", self.id(g));
                }
            }
            "UMLObject" => {
                if let Some(c) = r.refd(e, "classifier") {
                    let _ = write!(attrs, " classifier=\"{}\"", self.id(c));
                }
            }
            _ if r.has(e, "source") && r.has(e, "target") => {
                // client → supplier (Dependency, Realization, InterfaceRealization, …)
                if let (Some(s), Some(g)) = (r.refd(e, "source"), r.refd(e, "target")) {
                    let _ = write!(attrs, " client=\"{}\" supplier=\"{}\"", self.id(s), self.id(g));
                    if t == "UMLInterfaceRealization" {
                        let _ = write!(attrs, " contract=\"{}\"", self.id(g));
                    }
                }
            }
            _ => {}
        }
        let kind = match t.as_str() {
            "UMLAttribute" | "UMLAssociationEnd" | "UMLLinkEnd" => "Property",
            "UMLObject" => "InstanceSpecification",
            "UMLModel" => "Model",
            _ => uml,
        };
        let open = format!("{ind}<{tag} xmi:type=\"uml:{kind}\"{attrs}");
        // the children, by kind
        let mut inner = Writer { r, out: String::new() };
        let ci = format!("{ind}\t");
        inner.doc(e, &ci);
        if t == "UMLAttribute" || t == "UMLParameter" {
            inner.multiplicity(e, &ci);
            let dv = r.str(e, "defaultValue");
            if !dv.is_empty() {
                let _ = writeln!(inner.out, "{ci}<defaultValue xmi:type=\"uml:OpaqueExpression\" xmi:id=\"{}-default\"><body>{}</body></defaultValue>", self.id(e), esc(&dv));
            }
        }
        if t == "UMLAssociation" || t == "UMLLink" || t == "UMLCommunicationPath" {
            let ends: Vec<Eid> = ["end1", "end2"].iter().filter_map(|k| r.child(e, k)).collect();
            for end in ends {
                let mut a = self.common(end);
                a += &self.type_attr(end, "reference");
                let agg = r.str(end, "aggregation");
                if agg == "shared" || agg == "composite" {
                    let _ = write!(a, " aggregation=\"{agg}\"");
                }
                let _ = write!(a, " association=\"{}\"", self.id(e));
                let _ = writeln!(inner.out, "{ci}<ownedEnd xmi:type=\"uml:Property\"{a}>");
                inner.multiplicity(end, &format!("{ci}\t"));
                let _ = writeln!(inner.out, "{ci}</ownedEnd>");
            }
        }
        for (field, child_tag) in [
            ("literals", "ownedLiteral"),
            ("attributes", "ownedAttribute"),
            ("operations", "ownedOperation"),
            ("receptions", "ownedReception"),
            ("parameters", "ownedParameter"),
            ("extensionPoints", "extensionPoint"),
            ("slots", "slot"),
        ] {
            for c in r.list(e, field) {
                inner.element(c, child_tag, depth + 1);
            }
        }
        let is_pkg = r.kind_of(e, "UMLPackage");
        for c in r.list(e, "ownedElements") {
            let ct = r.ty(c);
            if r.kind_of(c, "Diagram") || ct.ends_with("Diagram") || !ct.starts_with("UML") || ct == "UMLStereotype" && !is_pkg {
                continue;
            }
            let child_tag = match ct {
                "UMLGeneralization" => "generalization",
                "UMLInterfaceRealization" => "interfaceRealization",
                "UMLInclude" => "include",
                "UMLExtend" => "extend",
                "UMLConstraint" => "ownedRule",
                _ if PACKAGE_LEVEL.contains(&ct) && !is_pkg => continue,
                _ if is_pkg => "packagedElement",
                _ => "nestedClassifier",
            };
            inner.element(c, child_tag, depth + 1);
        }
        // relationships owned by the classifiers of this package come up to it
        if is_pkg {
            for c in r.list(e, "ownedElements") {
                if r.kind_of(c, "UMLPackage") {
                    continue;
                }
                let mut rels = vec![];
                collect_rels(r, c, &mut rels);
                for x in rels {
                    inner.element(x, "packagedElement", depth + 1);
                }
            }
        }
        if t == "UMLConstraint" {
            let _ = writeln!(inner.out, "{ci}<specification xmi:type=\"uml:OpaqueExpression\" xmi:id=\"{}-spec\"><body>{}</body></specification>", self.id(e), esc(&r.str(e, "specification")));
        }
        if inner.out.is_empty() {
            let _ = writeln!(self.out, "{open}/>");
        } else {
            let _ = writeln!(self.out, "{open}>");
            self.out += &inner.out;
            let _ = writeln!(self.out, "{ind}</{tag}>");
        }
    }
}

/// The package-level relationships owned by a classifier or by classifiers nested in it.
fn collect_rels(r: &Repo, e: Eid, out: &mut Vec<Eid>) {
    for c in r.list(e, "ownedElements") {
        if PACKAGE_LEVEL.contains(&r.ty(c)) {
            out.push(c);
        } else if r.kind_of(c, "UMLClassifier") {
            collect_rels(r, c, out);
        }
    }
}

/// The project as an XMI 2.1 document.
pub fn export(r: &Repo) -> String {
    let mut w = Writer { r, out: String::new() };
    w.out += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    w.out += "<xmi:XMI xmi:version=\"2.1\" xmlns:uml=\"http://schema.omg.org/spec/UML/2.1\" xmlns:xmi=\"http://schema.omg.org/spec/XMI/2.1\">\n";
    let _ = writeln!(w.out, "\t<xmi:Documentation exporter=\"UMLCraft\" exporterVersion=\"{}\"/>", env!("CARGO_PKG_VERSION"));
    let root = r.root;
    let _ = writeln!(w.out, "\t<uml:Model xmi:type=\"uml:Model\" xmi:id=\"{}\" name=\"{}\">", esc(&r.id(root)), esc(&r.name(root)));
    for c in r.list(root, "ownedElements") {
        let ct = r.ty(c);
        if ct.starts_with("UML") && !ct.ends_with("Diagram") {
            w.element(c, "packagedElement", 2);
        }
    }
    w.out += "\t</uml:Model>\n</xmi:XMI>\n";
    w.out
}

impl App {
    /// File › Export › XMI Export (v2.1)...: a save dialog, then the file.
    pub fn export_xmi_dialog(&mut self) {
        let Some(mut p) = rfd::FileDialog::new().set_title("Export XMI").set_file_name("Untitled.xmi").add_filter("XMI Files", &["xmi"]).save_file() else { return };
        if p.extension().is_none() {
            p.set_extension("xmi");
        }
        match std::fs::write(&p, export(&self.repo)) {
            Ok(_) => self.toast(&format!("Exported to {}", p.display())),
            Err(err) => self.message = Some(("Failed to export XMI.".into(), err.to_string())),
        }
    }
}

// ------------------------------------------------------------------ import

/// An XML element: its tag, attributes in order, children and the text directly inside it.
#[derive(Debug, Default)]
pub struct Node {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub text: String,
}

impl Node {
    pub fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str())
    }
    fn child(&self, tag: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.tag == tag)
    }
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out += &rest[..i];
        rest = &rest[i..];
        let Some(j) = rest.find(';') else { break };
        let ent = &rest[1..j];
        let c = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ if ent.starts_with("#x") => u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32),
            _ if ent.starts_with('#') => ent[1..].parse().ok().and_then(char::from_u32),
            _ => None,
        };
        match c {
            Some(c) => {
                out.push(c);
                rest = &rest[j + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out + rest
}

/// Read an XML document into its root element. Comments, processing instructions and doctypes are
/// skipped; CDATA is kept as text.
pub fn parse_xml(src: &str) -> Result<Node, String> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut stack: Vec<Node> = vec![Node::default()];
    while i < b.len() {
        if b[i] != b'<' {
            let j = src[i..].find('<').map(|k| i + k).unwrap_or(b.len());
            let t = unescape(&src[i..j]);
            if let Some(top) = stack.last_mut() {
                top.text += &t;
            }
            i = j;
            continue;
        }
        let rest = &src[i..];
        if rest.starts_with("<!--") {
            i += rest.find("-->").ok_or("Unclosed comment")? + 3;
        } else if rest.starts_with("<![CDATA[") {
            let end = rest.find("]]>").ok_or("Unclosed CDATA")?;
            if let Some(top) = stack.last_mut() {
                top.text += &rest[9..end];
            }
            i += end + 3;
        } else if rest.starts_with("<?") || rest.starts_with("<!") {
            i += rest.find('>').ok_or("Unclosed declaration")? + 1;
        } else if rest.starts_with("</") {
            let end = rest.find('>').ok_or("Unclosed end tag")?;
            let name = rest[2..end].trim();
            let node = stack.pop().ok_or("Unexpected end tag")?;
            if node.tag != name {
                return Err(format!("Expected </{}>, found </{name}>", node.tag));
            }
            stack.last_mut().ok_or("Unexpected end tag")?.children.push(node);
            i += end + 1;
        } else {
            // a start tag: name, then attributes with quoted values
            let mut j = i + 1;
            while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' && b[j] != b'/' {
                j += 1;
            }
            let mut node = Node { tag: src[i + 1..j].to_string(), ..Default::default() };
            loop {
                while j < b.len() && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j >= b.len() {
                    return Err("Unclosed start tag".into());
                }
                if b[j] == b'>' {
                    stack.push(node);
                    j += 1;
                    break;
                }
                if b[j] == b'/' {
                    stack.last_mut().ok_or("Bad nesting")?.children.push(node);
                    j = src[j..].find('>').map(|k| j + k + 1).ok_or("Unclosed start tag")?;
                    break;
                }
                let eq = src[j..].find('=').map(|k| j + k).ok_or("Attribute without a value")?;
                let key = src[j..eq].trim().to_string();
                let mut q = eq + 1;
                while q < b.len() && b[q].is_ascii_whitespace() {
                    q += 1;
                }
                let quote = *b.get(q).ok_or("Attribute without a value")?;
                if quote != b'"' && quote != b'\'' {
                    return Err("Unquoted attribute".into());
                }
                let end = src[q + 1..].find(quote as char).map(|k| q + 1 + k).ok_or("Unclosed attribute")?;
                node.attrs.push((key, unescape(&src[q + 1..end])));
                j = end + 1;
            }
            i = j;
        }
    }
    let mut doc = stack.pop().ok_or("Empty document")?;
    if !stack.is_empty() {
        return Err("Unclosed element".into());
    }
    doc.children.retain(|c| !c.tag.is_empty());
    doc.children.pop().ok_or_else(|| "Empty document".to_string())
}

/// The UML type an XMI element declares: `uml:Class` gives "Class".
fn uml_type(n: &Node) -> Option<&str> {
    n.attr("xmi:type").or_else(|| n.attr("type").filter(|t| t.starts_with("uml:"))).and_then(|t| t.strip_prefix("uml:"))
}

/// What a reference is: an id in this file, or a type named by text (a primitive library href).
enum Ref {
    Id(String),
    Name(String),
}

fn reference(n: &Node, key: &str) -> Option<Ref> {
    if let Some(v) = n.attr(key).filter(|v| !v.is_empty()) {
        return Some(Ref::Id(v.split_whitespace().next()?.to_string()));
    }
    let c = n.child(key)?;
    if let Some(id) = c.attr("xmi:idref") {
        return Some(Ref::Id(id.to_string()));
    }
    let href = c.attr("href")?;
    Some(Ref::Name(href.rsplit(['#', '/']).next()?.to_string()))
}

struct Importer<'a> {
    r: &'a mut Repo,
    ids: std::collections::HashMap<String, Eid>,
    /// (element, field, reference) set once every element exists
    refs: Vec<(Eid, &'static str, Ref)>,
    /// association ends given as `memberEnd` ids of properties owned elsewhere
    member_ends: Vec<(Eid, Vec<String>)>,
}

impl Importer<'_> {
    fn common(&mut self, n: &Node, e: Eid) {
        if let Some(id) = n.attr("xmi:id") {
            self.ids.insert(id.to_string(), e);
        }
        if let Some(v) = n.attr("visibility") {
            self.r.set(e, "visibility", serde_json::json!(v));
        }
        for f in ["isAbstract", "isStatic", "isLeaf", "isReadOnly", "isDerived", "isQuery", "isOrdered", "isActive"] {
            if n.attr(f) == Some("true") && crate::meta::order_of(self.r.ty(e), f).is_some() {
                self.r.set(e, f, serde_json::json!(true));
            }
        }
        if let Some(c) = n.children.iter().find(|c| c.tag == "ownedComment") {
            let body = c.child("body").map(|b| b.text.clone()).or_else(|| c.attr("body").map(str::to_string)).unwrap_or_default();
            self.r.set(e, "documentation", serde_json::json!(body.trim()));
        }
        let mult = |k: &str| n.child(k).and_then(|v| v.attr("value")).map(str::to_string);
        if crate::meta::order_of(self.r.ty(e), "multiplicity").is_some() {
            let m = match (mult("lowerValue"), mult("upperValue")) {
                (Some(l), Some(u)) if l == u => l,
                (Some(l), Some(u)) if l == "0" && u == "*" => "0..*".to_string(),
                (Some(l), Some(u)) => format!("{l}..{u}"),
                (None, Some(u)) => u,
                (Some(l), None) => l,
                (None, None) => String::new(),
            };
            if !m.is_empty() {
                self.r.set(e, "multiplicity", serde_json::json!(m));
            }
        }
    }

    fn create(&mut self, ty: &str, n: &Node, parent: Eid, field: &str) -> Eid {
        let name = n.attr("name").unwrap_or("");
        let e = self.r.create(ty, &[("name", serde_json::json!(name))], Some((parent, field)));
        if name.is_empty() {
            self.r.remove(e, "name");
        }
        self.common(n, e);
        e
    }

    /// A directed relationship (`source` = owner or client, `target` = the other end).
    fn directed(&mut self, ty: &str, n: &Node, owner: Eid, source: Option<Ref>, target: Option<Ref>) {
        let e = self.r.create(ty, &[], Some((owner, "ownedElements")));
        self.common(n, e);
        if let Some(s) = n.attr("name") {
            self.r.set(e, "name", serde_json::json!(s));
        }
        match source {
            Some(s) => self.refs.push((e, "source", s)),
            None => self.r.set_ref(e, "source", Some(owner)),
        }
        if let Some(t) = target {
            self.refs.push((e, "target", t));
        }
    }

    /// An element under a package-like owner, or a member under a classifier.
    fn node(&mut self, n: &Node, parent: Eid) {
        // members often leave out xmi:type; their tag says what they are
        let by_tag = match n.tag.as_str() {
            "ownedAttribute" => Some("Property"),
            "ownedOperation" => Some("Operation"),
            "ownedLiteral" => Some("EnumerationLiteral"),
            "generalization" => Some("Generalization"),
            "interfaceRealization" => Some("InterfaceRealization"),
            "include" => Some("Include"),
            "extend" => Some("Extend"),
            _ => None,
        };
        let Some(t) = uml_type(n).or(by_tag) else { return };
        let r_ty = |s: &str| format!("UML{s}");
        match (n.tag.as_str(), t) {
            ("ownedAttribute", _) | (_, "Property") => {
                // a property with an association is one of its ends, made with the association
                if n.attr("association").is_some() || !self.r.kind_of(parent, "UMLClassifier") {
                    return;
                }
                let e = self.create("UMLAttribute", n, parent, "attributes");
                if let Some(a) = n.attr("aggregation").filter(|a| *a != "none") {
                    self.r.set(e, "aggregation", serde_json::json!(a));
                }
                if let Some(dv) = n.child("defaultValue").and_then(|d| d.child("body").map(|b| b.text.clone()).or_else(|| d.attr("value").map(str::to_string))) {
                    self.r.set(e, "defaultValue", serde_json::json!(dv));
                }
                if let Some(x) = reference(n, "type") {
                    self.refs.push((e, "type", x));
                }
            }
            ("ownedOperation", _) | (_, "Operation") => {
                let e = self.create("UMLOperation", n, parent, "operations");
                for p in n.children.iter().filter(|c| c.tag == "ownedParameter") {
                    let pe = self.create("UMLParameter", p, e, "parameters");
                    if let Some(d) = p.attr("direction") {
                        self.r.set(pe, "direction", serde_json::json!(d));
                    }
                    if let Some(x) = reference(p, "type") {
                        self.refs.push((pe, "type", x));
                    }
                }
            }
            ("ownedLiteral", _) | (_, "EnumerationLiteral") => {
                self.create("UMLEnumerationLiteral", n, parent, "literals");
            }
            ("generalization", _) | (_, "Generalization") => self.directed("UMLGeneralization", n, parent, None, reference(n, "general")),
            ("interfaceRealization", _) | (_, "InterfaceRealization") => {
                let target = reference(n, "contract").or_else(|| reference(n, "supplier"));
                self.directed("UMLInterfaceRealization", n, parent, None, target)
            }
            ("include", _) | (_, "Include") => self.directed("UMLInclude", n, parent, None, reference(n, "addition")),
            ("extend", _) | (_, "Extend") => self.directed("UMLExtend", n, parent, None, reference(n, "extendedCase")),
            (_, "Dependency" | "Usage" | "Abstraction" | "Realization") => {
                let ty = if t == "Realization" { "UMLRealization" } else { "UMLDependency" };
                self.directed(ty, n, parent, reference(n, "client"), reference(n, "supplier"))
            }
            (_, "Association") => {
                let e = self.create("UMLAssociation", n, parent, "ownedElements");
                let ends: Vec<&Node> = n.children.iter().filter(|c| c.tag == "ownedEnd").collect();
                if ends.len() == 2 {
                    for (k, end) in ["end1", "end2"].into_iter().zip(ends) {
                        let x = self.r.create("UMLAssociationEnd", &[], Some((e, k)));
                        self.common(end, x);
                        if let Some(nm) = end.attr("name") {
                            self.r.set(x, "name", serde_json::json!(nm));
                        }
                        if let Some(a) = end.attr("aggregation").filter(|a| *a != "none") {
                            self.r.set(x, "aggregation", serde_json::json!(a));
                        }
                        if let Some(rf) = reference(end, "type") {
                            self.refs.push((x, "reference", rf));
                        }
                    }
                } else if let Some(m) = n.attr("memberEnd") {
                    self.member_ends.push((e, m.split_whitespace().map(str::to_string).collect()));
                }
            }
            (_, "Package" | "Model" | "Profile") => {
                let ty = r_ty(t);
                let e = self.create(&ty, n, parent, "ownedElements");
                for c in &n.children {
                    self.node(c, e);
                }
            }
            (_, "Class" | "Interface" | "Enumeration" | "DataType" | "PrimitiveType" | "Signal" | "Actor" | "UseCase" | "Component" | "Artifact" | "Node" | "Stereotype") => {
                let ty = r_ty(t);
                let e = self.create(&ty, n, parent, "ownedElements");
                for c in &n.children {
                    self.node(c, e);
                }
            }
            (_, "InstanceSpecification") => {
                let e = self.create("UMLObject", n, parent, "ownedElements");
                if let Some(c) = reference(n, "classifier") {
                    self.refs.push((e, "classifier", c));
                }
            }
            (_, "Constraint") => {
                let e = self.create("UMLConstraint", n, parent, "ownedElements");
                let spec = n.child("specification").and_then(|s| s.child("body").map(|b| b.text.clone())).unwrap_or_default();
                self.r.set(e, "specification", serde_json::json!(spec));
            }
            // anything else nested in a package keeps its contents' packages and classifiers
            _ => {}
        }
    }
}

/// Read an XMI document into a new model "XMIImported" under the project; returns that model.
pub fn import(r: &mut Repo, text: &str) -> Result<Eid, String> {
    let doc = parse_xml(text)?;
    // the uml:Model, either the root or inside xmi:XMI
    let model = if doc.tag.ends_with("Model") || uml_type(&doc) == Some("Model") { &doc } else { doc.children.iter().find(|c| c.tag.ends_with("Model") || uml_type(c) == Some("Model") || c.tag.ends_with("Package")).ok_or("No UML model in this file.")? };
    let root = r.root;
    let target = r.create("UMLModel", &[("name", serde_json::json!("XMIImported"))], Some((root, "ownedElements")));
    let mut im = Importer { r, ids: Default::default(), refs: vec![], member_ends: vec![] };
    for c in &model.children {
        im.node(c, target);
    }
    // references, now that every element exists
    let refs = std::mem::take(&mut im.refs);
    for (e, field, x) in refs {
        match x {
            Ref::Id(id) => {
                if let Some(t) = im.ids.get(&id) {
                    im.r.set_ref(e, field, Some(*t));
                }
            }
            Ref::Name(name) if field == "type" => im.r.set(e, "type", serde_json::json!(name)),
            Ref::Name(_) => {}
        }
    }
    // association ends given as properties of the classes: they become the ends
    let mut props = std::collections::HashMap::new();
    collect_props(&model.children, &mut props);
    for (a, ends) in std::mem::take(&mut im.member_ends) {
        for (k, id) in ["end1", "end2"].into_iter().zip(ends) {
            let x = im.r.create("UMLAssociationEnd", &[], Some((a, k)));
            if let Some(p) = props.get(&id) {
                im.common(p, x);
                if let Some(nm) = p.attr("name") {
                    im.r.set(x, "name", serde_json::json!(nm));
                }
                if let Some(Ref::Id(t)) = reference(p, "type") {
                    if let Some(t) = im.ids.get(&t) {
                        im.r.set_ref(x, "reference", Some(*t));
                    }
                }
            }
        }
    }
    Ok(target)
}

fn collect_props<'a>(nodes: &'a [Node], out: &mut std::collections::HashMap<String, &'a Node>) {
    for n in nodes {
        if n.attr("association").is_some() {
            if let Some(id) = n.attr("xmi:id") {
                out.insert(id.to_string(), n);
            }
        }
        collect_props(&n.children, out);
    }
}

impl App {
    /// File › Import › XMI Import (v2.1)...: the elements go into a new model "XMIImported", as one
    /// undo step; diagrams are not imported.
    pub fn import_xmi_dialog(&mut self) {
        let Some(p) = rfd::FileDialog::new().set_title("Import XMI").add_filter("XMI Files", &["xmi", "xml", "uml"]).add_filter("All Files", &["*"]).pick_file() else { return };
        let text = match std::fs::read_to_string(&p) {
            Ok(t) => t,
            Err(e) => {
                self.message = Some(("Failed to import XMI.".into(), e.to_string()));
                return;
            }
        };
        if let Err(e) = crate::xmi::parse_xml(&text) {
            self.message = Some(("Failed to import XMI.".into(), e));
            return;
        }
        match self.change(|a| import(&mut a.repo, &text)) {
            Ok(m) => self.reveal(m),
            Err(e) => {
                self.do_undo();
                self.redo.clear();
                self.message = Some(("Failed to import XMI.".into(), e));
            }
        }
    }
}
