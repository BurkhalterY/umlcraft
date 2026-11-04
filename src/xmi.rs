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
