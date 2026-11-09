//! File > Export > HTML Docs: a static web site of the model, one page per element and an SVG per
//! diagram (extensions/default/html-export).
//!
//! The layout follows the original: `index.html` (a 25%/75% frameset), `contents/navigation.html`,
//! `contents/diagrams.html`, `contents/element_index.html`, `contents/<md5(_id)>.html` for each element
//! and `diagrams/<md5(_id)>.svg` for each diagram. The pages, their style sheet and the icon files
//! (`assets/icons/<key>.svg`, the Model Explorer's icons) are UMLCraft's own.

use crate::app::App;
use crate::model::Eid;
use crate::views::{self, Measure};
use std::fmt::Write as _;
use std::path::Path;

/// MD5 of a text, as 32 lower-case hex digits (RFC 1321).
pub fn md5_hex(text: &str) -> String {
    let s: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    let k: Vec<u32> = (0..64).map(|i| ((i as f64 + 1.0).sin().abs() * 4294967296.0) as u32).collect();
    let mut msg = text.as_bytes().to_vec();
    let bits = (msg.len() as u64).wrapping_mul(8);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bits.to_le_bytes());
    let (mut a0, mut b0, mut c0, mut d0) = (0x67452301u32, 0xefcdab89u32, 0x98badcfeu32, 0x10325476u32);
    for chunk in msg.chunks(64) {
        let m: Vec<u32> = chunk.chunks(4).map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]])).collect();
        let (mut a, mut b, mut c, mut d) = (a0, b0, c0, d0);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let f = f.wrapping_add(a).wrapping_add(k[i]).wrapping_add(m[g]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(f.rotate_left(s[i]));
        }
        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }
    [a0, b0, c0, d0].iter().flat_map(|x| x.to_le_bytes()).map(|b| format!("{b:02x}")).collect()
}

/// Escape text for HTML.
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Markdown to HTML, for the Description sections and the member tables' descriptions: GitHub
/// Flavored Markdown as the original's renderer (marked) reads it, raw HTML kept, in a "prose" block
/// (html-export/main.js:125-128).
pub fn markdown_html(text: &str) -> String {
    let mut out = String::new();
    pulldown_cmark::html::push_html(&mut out, crate::markdown::events(text).into_iter());
    format!("<div class=\"prose\">{}</div>", out.trim_end())
}

/// An icon of the Model Explorer as a 16 × 16 SVG file: the same drawing, recorded off-screen and
/// written as SVG shapes (the original ships an icon font, assets/icon-font).
pub fn icon_svg(ctx: &egui::Context, key: &str) -> String {
    use egui::epaint::{ColorMode, Shape};
    let out = ctx.run_ui(egui::RawInput::default(), |ui| {
        let p = ui.ctx().layer_painter(egui::LayerId::background());
        crate::icons::draw(&p, egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(16.0)), key);
    });
    let text = crate::ui::TEXT();
    // the theme's text colour becomes a dark grey on the white pages
    let color = |c: egui::Color32| -> String {
        if c.a() == 0 {
            return "none".into();
        }
        let c = if c == text { egui::Color32::from_rgb(0x24, 0x24, 0x24) } else { c };
        let [r, g, b, a] = c.to_srgba_unmultiplied();
        if a == 255 { format!("#{r:02x}{g:02x}{b:02x}") } else { format!("rgba({r},{g},{b},{:.3})", a as f32 / 255.0) }
    };
    let paint = |fill: egui::Color32, stroke: egui::Color32, width: f32| format!("fill=\"{}\" stroke=\"{}\" stroke-width=\"{width}\"", color(fill), if width > 0.0 { color(stroke) } else { "none".into() });
    let pts = |ps: &[egui::Pos2]| ps.iter().map(|p| format!("{:.2},{:.2}", p.x, p.y)).collect::<Vec<_>>().join(" ");
    let solid = |c: &ColorMode| match c {
        ColorMode::Solid(c) => *c,
        _ => egui::Color32::TRANSPARENT,
    };
    fn walk(s: &Shape, f: &mut dyn FnMut(&Shape)) {
        match s {
            Shape::Vec(v) => v.iter().for_each(|x| walk(x, f)),
            s => f(s),
        }
    }
    let mut svg = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" viewBox=\"0 0 16 16\" stroke-linecap=\"round\" stroke-linejoin=\"round\">");
    for cs in &out.shapes {
        walk(&cs.shape, &mut |s| {
            let _ = match s {
                Shape::LineSegment { points, stroke } => write!(svg, "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" {}/>", points[0].x, points[0].y, points[1].x, points[1].y, paint(egui::Color32::TRANSPARENT, stroke.color, stroke.width)),
                Shape::Path(p) => {
                    let tag = if p.closed { "polygon" } else { "polyline" };
                    write!(svg, "<{tag} points=\"{}\" {}/>", pts(&p.points), paint(p.fill, solid(&p.stroke.color), p.stroke.width))
                }
                Shape::Circle(c) => write!(svg, "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"{:.2}\" {}/>", c.center.x, c.center.y, c.radius, paint(c.fill, c.stroke.color, c.stroke.width)),
                Shape::Ellipse(e) => write!(svg, "<ellipse cx=\"{:.2}\" cy=\"{:.2}\" rx=\"{:.2}\" ry=\"{:.2}\" {}/>", e.center.x, e.center.y, e.radius.x, e.radius.y, paint(e.fill, e.stroke.color, e.stroke.width)),
                Shape::Rect(r) => write!(
                    svg,
                    "<rect x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" rx=\"{}\" {}/>",
                    r.rect.min.x,
                    r.rect.min.y,
                    r.rect.width(),
                    r.rect.height(),
                    r.corner_radius.nw,
                    paint(r.fill, r.stroke.color, r.stroke.width)
                ),
                Shape::QuadraticBezier(q) => write!(
                    svg,
                    "<path d=\"M{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}{}\" {}/>",
                    q.points[0].x, q.points[0].y, q.points[1].x, q.points[1].y, q.points[2].x, q.points[2].y,
                    if q.closed { " Z" } else { "" },
                    paint(q.fill, solid(&q.stroke.color), q.stroke.width)
                ),
                Shape::CubicBezier(c) => write!(
                    svg,
                    "<path d=\"M{:.2},{:.2} C{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}{}\" {}/>",
                    c.points[0].x, c.points[0].y, c.points[1].x, c.points[1].y, c.points[2].x, c.points[2].y, c.points[3].x, c.points[3].y,
                    if c.closed { " Z" } else { "" },
                    paint(c.fill, solid(&c.stroke.color), c.stroke.width)
                ),
                _ => Ok(()),
            };
        });
    }
    svg.push_str("</svg>\n");
    svg
}

/// The fields the original hides from the Properties table: those of its metamodels without
/// `"visible": true`, by the kind that declares them (resources/default/metamodel.json and the
/// extensions' metamodel.json files).
const HIDDEN: [(&str, &[&str]); 29] = [
    ("Element", &["_id", "_parent"]),
    ("Model", &["ownedElements"]),
    ("ExtensibleModel", &["documentation", "tags"]),
    ("Diagram", &["visible", "ownedViews"]),
    ("UndirectedRelationship", &["end1", "end2"]),
    ("UMLModelElement", &["templateParameters"]),
    ("UMLClassifier", &["attributes", "operations", "receptions", "behaviors"]),
    ("UMLEnumeration", &["literals"]),
    ("UMLOperation", &["preconditions", "bodyConditions", "postconditions"]),
    ("UMLBehavior", &["parameters", "preconditions", "postconditions"]),
    ("UMLInstance", &["slots"]),
    ("UMLTemplateBinding", &["parameterSubstitutions"]),
    ("UMLInteraction", &["messages", "participants", "fragments"]),
    ("UMLCombinedFragment", &["operands"]),
    ("UMLStateMachine", &["regions"]),
    ("UMLRegion", &["vertices", "transitions"]),
    ("UMLState", &["regions", "entryActivities", "doActivities", "exitActivities", "connections"]),
    ("UMLTransition", &["triggers", "effects"]),
    ("UMLActivity", &["nodes", "edges", "groups"]),
    ("UMLActivityGroup", &["subgroups", "nodes", "edges"]),
    ("UMLStructuredActivityNode", &["nodes", "edges"]),
    ("UMLAction", &["inputs", "outputs", "triggers", "localPreconditions", "localPostconditions"]),
    ("UMLImage", &["smallIcon"]),
    ("ERDEntity", &["columns"]),
    ("BPMNEvent", &["eventDefinitions"]),
    ("SysMLBlock", &["constraints", "parts", "ports", "references", "values", "properties", "flowProperties"]),
    ("SysMLConstraintBlock", &["parameters"]),
    ("AWSElement", &["icon"]),
    ("AzureElement", &["icon"]),
];

/// The visible fields that hold a list, shown empty when the file leaves them out.
const LISTS: [&str; 19] = [
    "stakeholders", "parameters", "raisedExceptions", "constrainedElements", "importedElements", "qualifiers", "extensionPoints", "extensionLocations", "entry", "exit", "inStates", "exceptionTypes",
    "represented", "conveyed", "realizingConnectors", "realizingActivityEdges", "realizingMessages", "upperParticipants", "lowerParticipants",
];

/// The visible fields that hold one reference (with the stereotype and an instance's classifier).
const REFS: [&str; 35] = [
    "association", "associationSide", "calledRef", "classSide", "connector", "covered", "default", "definingFeature", "formal", "handlerBody", "initiatingParticipant", "itemProperty", "linkSide",
    "messageRef", "objectSide", "ownerAttribute", "parameter", "quantityKind", "reference", "referenceTo", "refersTo", "represent", "returnValueRecipient", "signal", "signature", "source",
    "subactivity", "submachine", "target", "targetOperation", "targetSignal", "unit", "stereotype", "classifier", "icon",
];

/// Whether a field of a kind is in its Properties table.
fn visible_field(ty: &str, field: &str) -> bool {
    let hidden = HIDDEN.iter().any(|(k, fs)| fs.contains(&field) && crate::meta::is_kind_of(ty, k)) || (field == "icon" && crate::meta::is_kind_of(ty, "GCPElement"));
    !hidden
}

const STYLE: &str = "body{font-family:system-ui,-apple-system,'Segoe UI',Arial,sans-serif;font-size:14px;color:#222;margin:16px 20px;line-height:1.45}
a{color:#1f6fd1;text-decoration:none}a:hover{text-decoration:underline}
h1{font-size:22px;margin:0 0 4px}h2{font-size:16px;margin:22px 0 6px;border-bottom:1px solid #ddd;padding-bottom:3px}
.type{margin:6px 0}.type>span{display:inline-block;background:#eef5fb;border:1px solid #cfe3f3;border-radius:3px;padding:2px 8px;margin-right:8px}
.path{margin:8px 0 10px}.sep{color:#999;margin:0 3px}
table{border-collapse:collapse;width:100%;margin:4px 0}th,td{border:1px solid #ddd;padding:4px 8px;text-align:left;vertical-align:top}th{background:#f4f4f4}
tr:nth-child(even) td{background:#fafafa}
ul.links{margin:4px 0;padding-left:0;list-style:none}ul.links li{padding:1px 0}.grid{display:flex;flex-wrap:wrap;gap:14px}
.tile{width:220px;border:1px solid #ddd;border-radius:4px;padding:6px;text-align:center}.tile img{max-width:100%;max-height:150px}
.icon{width:16px;height:16px;vertical-align:-3px;margin-right:4px}
.nav{font-size:13px;white-space:nowrap}.nav .links{margin-bottom:1em}.nav ol{list-style:none;margin:0;padding-left:16px}.nav>ol{padding-left:0}
.nav details>summary{cursor:pointer}.nav li.leaf{padding-left:14px}
.label{background:#5bc0de;color:#fff;border-radius:3px;padding:0 5px;font-size:12px}.label a{color:#fff}
.diagram{border:1px solid #eee}
.prose pre{background:#f6f6f6;padding:8px;overflow:auto}.prose code{background:#f3f3f3;padding:0 3px;border-radius:3px}.prose pre code{background:none;padding:0}
.prose blockquote{color:#666;border-left:3px solid #ddd;margin:4px 0;padding-left:10px}.prose table{width:auto}
.prose li:has(>input[type=checkbox]){list-style:none}.prose input[type=checkbox]{margin:0 6px 0 -18px}";

impl App {
    fn page(&self, e: Eid) -> String {
        format!("{}.html", md5_hex(&self.repo.id(e)))
    }

    /// The icon of an element, an image of the exported icon set.
    fn icon_html(&self, e: Eid) -> String {
        format!("<img class=\"icon\" src=\"../assets/icons/{}.svg\" alt=\"\">", crate::icons::key_for(&self.repo, e))
    }

    /// A link to an element's page: its icon and its explorer text.
    fn link(&self, e: Eid) -> String {
        format!("<a href=\"{}\">{}{}</a>", self.page(e), self.icon_html(e), esc(&self.label_of(e)))
    }

    /// A value the way the original's pages show it (toValue, html-export/main.js:85-123): null and
    /// void, booleans and numbers as labels, text escaped, an element as a link with its icon and
    /// explorer text, a stereotype as «name».
    fn value_html(&self, v: Option<&serde_json::Value>) -> String {
        use serde_json::Value;
        let label = |t: &str| format!("<span class=\"label\">{}</span>", esc(t));
        match v {
            None => label("void"),
            Some(Value::Null) => label("null"),
            Some(Value::Bool(b)) => label(&b.to_string()),
            Some(Value::Number(n)) => label(&n.to_string()),
            Some(Value::String(s)) => esc(s),
            Some(r) => match self.repo.by_ref(r).filter(|x| self.repo.alive(*x)) {
                Some(x) => self.elem_html(x),
                None => label("null"),
            },
        }
    }

    fn elem_html(&self, x: Eid) -> String {
        if self.repo.ty(x) == "UMLStereotype" {
            format!("<a href=\"{}\">{}«{}»</a>", self.page(x), self.icon_html(x), esc(&self.label_of(x)))
        } else {
            self.link(x)
        }
    }

    /// A type the way toType shows it: "none" when blank, an element as a link with its icon and
    /// name, text escaped (html-export/main.js:62-82).
    fn type_html(&self, v: Option<&serde_json::Value>) -> String {
        use serde_json::Value;
        match v {
            None | Some(Value::Null) => "<span class=\"label\">none</span>".into(),
            Some(Value::String(s)) if s.trim().is_empty() => "<span class=\"label\">none</span>".into(),
            Some(Value::String(s)) => esc(s),
            Some(r) => match self.repo.by_ref(r).filter(|x| self.repo.alive(*x)) {
                Some(x) => format!("<a href=\"{}\">{}{}</a>", self.page(x), self.icon_html(x), esc(&self.repo.name(x))),
                None => "<span class=\"label\">none</span>".into(),
            },
        }
    }

    /// A field's value: a list joined with commas, an owned element as a link, else the value or
    /// the kind's default (properties.ejs).
    fn field_html(&self, e: Eid, f: &str) -> String {
        let r = &self.repo;
        if let Some(serde_json::Value::Array(items)) = r.raw(e, f) {
            return items.iter().map(|x| self.value_html(Some(x))).collect::<Vec<_>>().join("<span>, </span>");
        }
        let owned = r.list(e, f);
        if !owned.is_empty() {
            return owned.iter().map(|x| self.elem_html(*x)).collect::<Vec<_>>().join("<span>, </span>");
        }
        match r.raw(e, f) {
            Some(v) => self.value_html(Some(v)),
            None => match crate::meta::default_of(r.ty(e), f) {
                Some(d) => self.value_html(Some(&d)),
                None if LISTS.contains(&f) => String::new(),
                // a reference left out is null; text left out is empty (the kinds' constructors)
                None if REFS.contains(&f) || (f == "type" && r.kind_of(e, "UMLObjectNode")) => self.value_html(Some(&serde_json::Value::Null)),
                None => String::new(),
            },
        }
    }

    /// A Properties table: one row per visible meta-attribute of the element's kind, in the
    /// metamodel's order (properties.ejs; Element.getMetaAttributes).
    fn properties_table(&self, out: &mut String, e: Eid, title: &str) {
        let ty = self.repo.ty(e);
        let fields: Vec<&str> = crate::meta::attrs(ty).into_iter().map(|f| f.0).filter(|f| visible_field(ty, f)).collect();
        let _ = write!(out, "<h2>{}</h2><table><tr><th width=\"50%\">Name</th><th width=\"50%\">Value</th></tr>", esc(title));
        for f in fields {
            let _ = write!(out, "<tr><td>{}</td><td>{}</td></tr>", esc(f), self.field_html(e, f));
        }
        out.push_str("</table>");
    }

    /// The Tags table: name, kind and the value its kind keeps (content.ejs:607-637).
    fn tags_table(&self, out: &mut String, e: Eid) {
        let tags = self.repo.list(e, "tags");
        if tags.is_empty() {
            return;
        }
        out.push_str("<h2>Tags</h2><table><tr><th>Name</th><th>Kind</th><th>Value</th></tr>");
        for t in tags {
            let kind = self.repo.str(t, "kind");
            let field = match kind.as_str() {
                "boolean" => "checked",
                "reference" => "reference",
                "number" => "number",
                _ => "value",
            };
            let raw = self.repo.raw(t, field).cloned().or_else(|| crate::meta::default_of("Tag", field)).unwrap_or(serde_json::Value::Null);
            let _ = write!(out, "<tr><td>{}</td><td><span class=\"label\">{}</span></td><td>{}</td></tr>", esc(&self.repo.name(t)), esc(&kind), self.value_html(Some(&raw)));
        }
        out.push_str("</table>");
    }

    /// An element's name, "(unnamed)" when blank (toName).
    fn display_name(&self, e: Eid) -> String {
        let n = self.repo.name(e);
        if n.trim().is_empty() { "(unnamed)".into() } else { n }
    }

    /// The elements of the model tree under `e`, in their order (views left out).
    fn doc_children(&self, e: Eid) -> Vec<Eid> {
        self.repo.children(e).into_iter().filter(|(k, _)| k != "ownedViews" && k != "subViews").map(|(_, c)| c).collect()
    }

    /// The children in the explorer's order: kind priority, then the order they were added
    /// (getChildNodes(true), core.js:330-365).
    fn nav_children(&self, e: Eid) -> Vec<Eid> {
        let mut kids: Vec<(u32, usize, Eid)> = self.doc_children(e).into_iter().enumerate().map(|(i, c)| (crate::ui::order_rank(self.repo.ty(c)), i, c)).collect();
        kids.sort_by_key(|k| (k.0, k.1));
        kids.into_iter().map(|k| k.2).collect()
    }

    fn doc_elements(&self) -> Vec<Eid> {
        let mut out = vec![];
        let mut todo = vec![self.repo.root];
        while let Some(e) = todo.pop() {
            out.push(e);
            let mut kids = self.doc_children(e);
            kids.reverse();
            todo.extend(kids);
        }
        out
    }

    fn html_page(title: &str, body: &str) -> String {
        format!("<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{}</title><link rel=\"stylesheet\" href=\"../assets/style.css\"></head>\n<body>{body}</body></html>\n", esc(title))
    }

    /// One node of the navigation tree: its icon and explorer text (the project's in bold), then its
    /// children, eight levels at most (navigation.ejs, node.ejs).
    fn nav_tree(&self, e: Eid, depth: usize, out: &mut String) {
        let kids = if depth >= 7 { vec![] } else { self.nav_children(e) };
        let text = esc(&self.label_of(e));
        let text = if e == self.repo.root { format!("<strong>{text}</strong>") } else { text };
        let label = format!("<a href=\"{}\" target=\"content\">{}{text}</a>", self.page(e), self.icon_html(e));
        if kids.is_empty() {
            let _ = write!(out, "<li class=\"leaf\">{label}</li>");
            return;
        }
        let _ = write!(out, "<li><details{}><summary>{label}</summary><ol>", if depth == 0 { " open" } else { "" });
        for k in kids {
            self.nav_tree(k, depth + 1, out);
        }
        out.push_str("</ol></details></li>");
    }

    /// A titled list of links, left out when empty (list.ejs).
    fn links_section(&self, out: &mut String, title: &str, items: &[Eid]) {
        if items.is_empty() {
            return;
        }
        let _ = write!(out, "<h2>{}</h2><ul class=\"links\">", esc(title));
        for i in items {
            let _ = write!(out, "<li>{}</li>", self.elem_html(*i));
        }
        out.push_str("</ul>");
    }

    /// The relationships of a kind that start (`from`) or end at `e`, mapped to the other end.
    fn related(&self, e: Eid, kind: &str, from: bool) -> Vec<Eid> {
        self.repo
            .all()
            .filter(|x| self.repo.ty(*x) == kind)
            .filter_map(|x| self.rel_ends(x))
            .filter_map(|(s, t)| if from && s == e { Some(t) } else if !from && t == e { Some(s) } else { None })
            .collect()
    }

    /// The relationships of a kind attached to `e` (app.repository.getRelationshipsOf with a filter),
    /// optionally only those it is the target (`Some(false)`) or the source (`Some(true)`) of.
    fn relationships_of(&self, e: Eid, kind: &str, source: Option<bool>) -> Vec<Eid> {
        self.repo
            .all()
            .filter(|x| self.repo.alive(*x) && self.repo.kind_of(*x, kind))
            .filter(|x| match (self.rel_ends(*x), source) {
                (Some((s, _)), Some(true)) => s == e,
                (Some((_, t)), Some(false)) => t == e,
                (Some((s, t)), None) => s == e || t == e,
                _ => false,
            })
            .collect()
    }

    /// A member table: a header row, then one row of cells per item (content.ejs:200-360).
    fn table(&self, out: &mut String, title: &str, heads: &[&str], rows: Vec<Vec<String>>) {
        if rows.is_empty() {
            return;
        }
        let _ = write!(out, "<h2>{}</h2><table><tr>", esc(title));
        for h in heads {
            let _ = write!(out, "<th>{h}</th>");
        }
        out.push_str("</tr>");
        for row in rows {
            out.push_str("<tr>");
            for c in row {
                let _ = write!(out, "<td>{c}</td>");
            }
            out.push_str("</tr>");
        }
        out.push_str("</table>");
    }

    /// The member tables: template parameters, literals, attributes, operations, receptions,
    /// extension points and parameters, each with the original's columns (content.ejs:200-360).
    fn member_tables(&self, b: &mut String, e: Eid) {
        let r = &self.repo;
        let name = |x: Eid| format!("<a href=\"{}\">{}</a>", self.page(x), esc(&self.display_name(x)));
        let doc = |x: Eid| markdown_html(&r.str(x, "documentation"));
        let vis = |x: Eid| esc(&r.str(x, "visibility"));
        let mult = |x: Eid| {
            let m = r.str(x, "multiplicity");
            if m.is_empty() { String::new() } else { format!(" [{}]", esc(&m)) }
        };
        let sig = views::Show { vis: false, ..views::Show::ALL };
        let link_text = |x: Eid, t: String| format!("<a href=\"{}\">{}</a>", self.page(x), esc(&t));
        if r.kind_of(e, "UMLModelElement") {
            let rows = r.list(e, "templateParameters").into_iter().map(|x| vec![vis(x), name(x), self.value_html(r.val(x, "parameterType").as_ref()), doc(x)]).collect();
            self.table(b, "Template Parameters", &["Visibility", "Name", "Type", "Description"], rows);
        }
        if r.kind_of(e, "UMLEnumeration") {
            let rows = r.list(e, "literals").into_iter().map(|x| vec![vis(x), name(x), doc(x)]).collect();
            self.table(b, "Literals", &["Visibility", "Name", "Description"], rows);
        }
        if r.kind_of(e, "UMLClassifier") {
            let rows = r.list(e, "attributes").into_iter().map(|x| vec![vis(x), name(x), format!("{}{}", self.type_html(r.raw(x, "type")), mult(x)), doc(x)]).collect();
            self.table(b, "Attributes", &["Visibility", "Name", "Type", "Description"], rows);
            let rows = r.list(e, "operations").into_iter().map(|x| vec![vis(x), link_text(x, views::operation_text(r, x, sig)), doc(x)]).collect();
            self.table(b, "Operations", &["Visibility", "Name", "Description"], rows);
            let rows = r.list(e, "receptions").into_iter().map(|x| vec![vis(x), link_text(x, views::reception_text(r, x, sig)), self.type_html(r.raw(x, "signal")), doc(x)]).collect();
            self.table(b, "Receptions", &["Visibility", "Name", "Signal", "Description"], rows);
        }
        if r.kind_of(e, "UMLUseCase") {
            let rows = r.list(e, "extensionPoints").into_iter().map(|x| vec![name(x), self.value_html(r.val(x, "location").as_ref()), doc(x)]).collect();
            self.table(b, "Extension Points", &["Name", "Location", "Description"], rows);
        }
        if r.kind_of(e, "UMLBehavioralFeature") {
            let rows = r.list(e, "parameters").into_iter().map(|x| vec![esc(&r.str(x, "direction")), name(x), format!("{}{}", self.value_html(Some(&r.val(x, "type").unwrap_or(serde_json::Value::Null))), mult(x)), doc(x)]).collect();
            self.table(b, "Parameters", &["Direction", "Name", "Type", "Description"], rows);
        }
    }

    /// The behaviour, ERD, DFD and flowchart sections (content.ejs:392-597).
    fn behaviour_sections(&self, b: &mut String, e: Eid) {
        let r = &self.repo;
        let lists = |b: &mut String, pairs: &[(&str, &str)]| {
            for (title, field) in pairs {
                self.links_section(b, title, &r.list(e, field));
            }
        };
        if r.kind_of(e, "UMLBehavior") {
            lists(b, &[("Preconditions", "preconditions"), ("Postconditions", "postconditions")]);
        }
        if r.kind_of(e, "UMLAction") {
            lists(b, &[("Inputs", "inputs"), ("Outputs", "outputs"), ("Triggers", "triggers"), ("Local Preconditions", "localPreconditions"), ("Local Postconditions", "localPostconditions")]);
        }
        if r.kind_of(e, "UMLInteraction") {
            lists(b, &[("Fragments", "fragments"), ("Participants", "participants"), ("Messages", "messages")]);
        }
        if r.kind_of(e, "UMLCombinedFragment") {
            lists(b, &[("Interaction Operands", "operands")]);
        }
        if r.kind_of(e, "UMLActivity") {
            lists(b, &[("Groups", "groups"), ("Nodes", "nodes"), ("Edges", "edges")]);
        }
        if r.kind_of(e, "UMLStateMachine") {
            lists(b, &[("Regions", "regions")]);
        }
        if r.kind_of(e, "UMLRegion") {
            lists(b, &[("Vertices", "vertices"), ("Transitions", "transitions")]);
        }
        if r.kind_of(e, "UMLState") {
            lists(b, &[("Entry Activities", "entryActivities"), ("Do Activities", "doActivities"), ("Exit Activities", "exitActivities"), ("Connections", "connections"), ("Regions", "regions")]);
        }
        if r.kind_of(e, "UMLVertex") {
            self.links_section(b, "Incoming Transitions", &self.relationships_of(e, "UMLTransition", Some(false)));
            self.links_section(b, "Outgoing Transitions", &self.relationships_of(e, "UMLTransition", Some(true)));
        }
        if r.kind_of(e, "UMLTransition") {
            lists(b, &[("Triggers", "triggers"), ("Effects", "effects")]);
        }
        // a data model's entities
        if r.kind_of(e, "ERDDataModel") {
            let entities: Vec<Eid> = r.list(e, "ownedElements").into_iter().filter(|x| r.kind_of(*x, "ERDEntity")).collect();
            self.links_section(b, "Entities", &entities);
        }
        // an entity's columns and related entities
        if r.kind_of(e, "ERDEntity") {
            let label = |v: serde_json::Value| self.value_html(Some(&v));
            let mut rows = vec![];
            for c in r.list(e, "columns") {
                // the type and its length, as getTypeString (erd/elements.js:113-122)
                let len = r.val(c, "length").unwrap_or(serde_json::Value::Null);
                let len = match len {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) if n.as_f64() != Some(0.0) => n.to_string(),
                    _ => String::new(),
                };
                let ty = r.str(c, "type");
                let ty = if len.is_empty() { ty } else { format!("{ty}({len})") };
                let refto = r.refd(c, "referenceTo").filter(|x| r.alive(*x));
                let refto_html = match refto {
                    Some(x) => match r.parent(x) {
                        Some(p) => format!("{} :: {}", self.elem_html(p), self.elem_html(x)),
                        None => self.elem_html(x),
                    },
                    None => label(serde_json::Value::Null),
                };
                let flag = |k: &str| label(r.val(c, k).unwrap_or(serde_json::Value::Bool(false)));
                rows.push(vec![format!("<a href=\"{}\">{}</a>", self.page(c), esc(&self.display_name(c))), esc(&ty), flag("primaryKey"), flag("foreignKey"), refto_html, flag("nullable"), flag("unique")]);
            }
            let _ = write!(b, "<h2>Columns</h2><table><tr><th width=\"20%\">Name</th><th width=\"20%\">Type</th><th width=\"10%\">Primary Key</th><th width=\"10%\">Foreign Key</th><th width=\"20%\">Reference To</th><th width=\"10%\">Nullable</th><th width=\"10%\">Unique</th></tr>");
            for row in rows {
                let _ = write!(b, "<tr>{}</tr>", row.iter().map(|c| format!("<td>{c}</td>")).collect::<String>());
            }
            b.push_str("</table>");
            b.push_str("<h2>Related Entities</h2><table><tr><th width=\"25%\">Entity</th><th width=\"25%\">Cardinality</th><th width=\"25%\">Reference Name</th><th width=\"25%\">Relationship</th></tr>");
            for rel in self.relationships_of(e, "ERDRelationship", None) {
                let (Some(e1), Some(e2)) = (r.child(rel, "end1"), r.child(rel, "end2")) else { continue };
                let end = if r.refd(e1, "reference") == Some(e) { e2 } else { e1 };
                let other = r.raw(end, "reference").cloned().unwrap_or(serde_json::Value::Null);
                let _ = write!(
                    b,
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    self.value_html(Some(&other)),
                    self.value_html(r.val(end, "cardinality").as_ref()),
                    self.value_html(Some(&serde_json::Value::String(r.name(end)))),
                    self.elem_html(rel)
                );
            }
            b.push_str("</table>");
        }
        if r.kind_of(e, "DFDElement") {
            self.links_section(b, "Incoming Data Flows", &self.relationships_of(e, "DFDDataFlow", Some(false)));
            self.links_section(b, "Outgoing Data Flows", &self.relationships_of(e, "DFDDataFlow", Some(true)));
        }
        if r.kind_of(e, "FCModelElement") {
            self.links_section(b, "Incoming Flows", &self.relationships_of(e, "FCFlow", Some(false)));
            self.links_section(b, "Outgoing Flows", &self.relationships_of(e, "FCFlow", Some(true)));
        }
    }

    /// The page of one element: header, diagram with its image map, description and sections.
    fn element_page(&self, e: Eid, svg_bounds: &std::collections::HashMap<Eid, views::R>) -> String {
        let r = &self.repo;
        let mut b = String::new();
        // name; the kind with its icon and the stereotype; the path from the project, each a link
        let _ = write!(b, "<h1>{}</h1><div class=\"type\"><span>{}{}</span>", esc(&self.display_name(e)), self.icon_html(e), esc(r.ty(e)));
        match r.raw(e, "stereotype") {
            Some(serde_json::Value::String(s)) if !s.is_empty() => {
                let _ = write!(b, "<span>«{}»</span>", esc(s));
            }
            Some(v) if r.by_ref(v).is_some() => {
                let _ = write!(b, "<span>{}</span>", self.value_html(Some(v)));
            }
            _ => {}
        }
        b.push_str("</div><div class=\"path\">");
        for (i, x) in r.path(e).into_iter().enumerate() {
            if i > 0 {
                b.push_str("<span class=\"sep\">::</span>");
            }
            let _ = write!(b, "<span class=\"label\">{}</span>", self.elem_html(x));
        }
        b.push_str("</div>");
        if self.is_diagram(e) {
            let id = md5_hex(&r.id(e));
            let _ = write!(b, "<h2>Diagram</h2><img class=\"diagram\" src=\"../diagrams/{id}.svg\" usemap=\"#map-{id}\" alt=\"{}\"><map name=\"map-{id}\">", esc(&r.name(e)));
            if let Some(bb) = svg_bounds.get(&e) {
                // the SVG starts 10 px before the bounds; the topmost view comes first
                let (ox, oy) = (bb.x - 10.0, bb.y - 10.0);
                for v in r.list(e, "ownedViews").into_iter().rev() {
                    let Some(m) = r.refd(v, "model") else { continue };
                    let vb = if views::is_edge(r, v) {
                        let pts = views::points_of(r, v);
                        let (x1, y1) = (pts.iter().map(|p| p.x).fold(f64::INFINITY, f64::min), pts.iter().map(|p| p.y).fold(f64::INFINITY, f64::min));
                        let (x2, y2) = (pts.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max), pts.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max));
                        views::R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
                    } else {
                        views::node_rect(r, v)
                    };
                    let _ = write!(
                        b,
                        "<area shape=\"rect\" coords=\"{},{},{},{}\" href=\"{}\" alt=\"{}\">",
                        (vb.x - ox).round(),
                        (vb.y - oy).round(),
                        (vb.x + vb.w - ox).round(),
                        (vb.y + vb.h - oy).round(),
                        self.page(m),
                        esc(&r.name(m))
                    );
                }
            }
            b.push_str("</map>");
        }
        // the description of every element that can have one, "none" when blank
        if r.kind_of(e, "ExtensibleModel") {
            let doc = r.str(e, "documentation");
            let body = if doc.trim().is_empty() { "<span class=\"label\">none</span>".to_string() } else { markdown_html(&doc) };
            let _ = write!(b, "<h2>Description</h2><div>{body}</div>");
        }
        let spec = r.str(e, "specification");
        if (r.kind_of(e, "UMLConstraint") || r.kind_of(e, "UMLOperation")) && !spec.is_empty() {
            let _ = write!(b, "<h2>Specification</h2><pre>{}</pre>", esc(&spec));
        }
        // the ends of a relationship: source and target, or each end's properties
        if let (Some(e1), Some(e2)) = (r.child(e, "end1"), r.child(e, "end2")) {
            self.properties_table(&mut b, e1, "End1");
            self.properties_table(&mut b, e2, "End2");
        } else if let (Some(s), Some(t)) = (r.refd(e, "source"), r.refd(e, "target")) {
            self.links_section(&mut b, "Source", &[s]);
            self.links_section(&mut b, "Target", &[t]);
        }
        if r.kind_of(e, "UMLClassifier") {
            self.links_section(&mut b, "Generalized Elements", &self.related(e, "UMLGeneralization", true));
            self.links_section(&mut b, "Specialized Elements", &self.related(e, "UMLGeneralization", false));
            self.links_section(&mut b, "Interfaces", &self.related(e, "UMLInterfaceRealization", true));
            self.links_section(&mut b, "Components", &self.related(e, "UMLComponentRealization", true));
            self.links_section(&mut b, "Deployment Targets", &self.related(e, "UMLDeployment", true));
            if r.ty(e) == "UMLInterface" {
                self.links_section(&mut b, "Implementing Classifiers", &self.related(e, "UMLInterfaceRealization", false));
            }
            if r.ty(e) == "UMLComponent" {
                self.links_section(&mut b, "Realizing Classifiers", &self.related(e, "UMLComponentRealization", false));
            }
            if r.ty(e) == "UMLNode" {
                self.links_section(&mut b, "Deployed Elements", &self.related(e, "UMLDeployment", false));
            }
            let associated: Vec<Eid> = r
                .all()
                .filter(|x| r.ty(*x) == "UMLAssociation")
                .filter_map(|x| self.rel_ends(x))
                .filter_map(|(a, c)| if a == e { Some(c) } else if c == e { Some(a) } else { None })
                .collect();
            if r.ty(e) == "UMLActor" {
                self.links_section(&mut b, "Use Cases", &associated.iter().copied().filter(|x| r.ty(*x) == "UMLUseCase").collect::<Vec<_>>());
            }
            if r.ty(e) == "UMLUseCase" {
                self.links_section(&mut b, "Actors", &associated.iter().copied().filter(|x| r.ty(*x) == "UMLActor").collect::<Vec<_>>());
                self.links_section(&mut b, "Included Use Cases", &self.related(e, "UMLInclude", true));
                self.links_section(&mut b, "Extending Use Cases", &self.related(e, "UMLExtend", false));
            }
        }
        self.member_tables(&mut b, e);
        // the diagrams it owns, as pictures (content.ejs:362-390)
        let owned_diagrams: Vec<Eid> = r.list(e, "ownedElements").into_iter().filter(|x| self.is_diagram(*x)).collect();
        if !owned_diagrams.is_empty() {
            b.push_str("<h2>Diagrams</h2><div class=\"grid\">");
            for d in owned_diagrams {
                let _ = write!(b, "<a class=\"tile\" href=\"{}\"><img src=\"../diagrams/{}.svg\" alt=\"\"><div>{}</div></a>", self.page(d), md5_hex(&r.id(d)), esc(&self.display_name(d)));
            }
            b.push_str("</div>");
        }
        self.behaviour_sections(&mut b, e);
        // the generic sections (content.ejs:599-670)
        self.properties_table(&mut b, e, "Properties");
        self.tags_table(&mut b, e);
        if r.kind_of(e, "UMLModelElement") {
            let constraints: Vec<Eid> = r.list(e, "ownedElements").into_iter().filter(|c| r.ty(*c) == "UMLConstraint").collect();
            self.links_section(&mut b, "Constraints", &constraints);
            // every kind of dependency (realizations and deployments too), by its other end (elements.js:518-541)
            let deps: Vec<(Eid, Eid)> = r.all().filter(|x| r.kind_of(*x, "UMLDependency")).filter_map(|x| self.rel_ends(x)).collect();
            self.links_section(&mut b, "Dependencies", &deps.iter().filter(|(s, _)| *s == e).map(|(_, t)| *t).collect::<Vec<_>>());
            self.links_section(&mut b, "Dependants", &deps.iter().filter(|(_, t)| *t == e).map(|(s, _)| *s).collect::<Vec<_>>());
        }
        let rels: Vec<Eid> = r.all().filter(|x| self.rel_ends(*x).map(|(s, t)| s == e || t == e).unwrap_or(false)).collect();
        self.links_section(&mut b, "Relationships", &rels);
        self.links_section(&mut b, "Owned Elements", &r.list(e, "ownedElements"));
        // the diagrams that show it
        if !self.is_diagram(e) {
            let mut shown: Vec<Eid> = vec![];
            for d in r.all_of("Diagram") {
                if !shown.contains(&d) && r.list(d, "ownedViews").into_iter().any(|v| r.refd(v, "model") == Some(e)) {
                    shown.push(d);
                }
            }
            self.links_section(&mut b, "Diagrams", &shown);
        }
        Self::html_page(&r.name(e), &b)
    }

    /// Write the site into `out` (created if needed).
    pub fn export_html(&mut self, out: &Path, ms: &dyn Measure) -> Result<(), String> {
        let io = |e: std::io::Error| e.to_string();
        for sub in ["contents", "diagrams", "assets/icons"] {
            std::fs::create_dir_all(out.join(sub)).map_err(io)?;
        }
        std::fs::write(out.join("assets/style.css"), STYLE).map_err(io)?;
        // the diagrams, as SVG named by the MD5 of their id
        let diagrams = self.print_diagrams(true);
        let mut bounds = std::collections::HashMap::new();
        for &d in &diagrams {
            // an empty diagram gives an empty picture, as the original writes one for every diagram
            let svg = crate::export::render(self, d, ms, crate::export::Format::Svg, 1.0).unwrap_or_else(|_| b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"20\" height=\"20\"></svg>\n".to_vec());
            std::fs::write(out.join(format!("diagrams/{}.svg", md5_hex(&self.repo.id(d)))), svg).map_err(io)?;
            if let Some(b) = crate::render::bounds(&self.repo, d) {
                bounds.insert(d, b);
            }
        }
        let elements = self.doc_elements();
        // the icons of every element shown, in place of the original's icon font
        let mut keys: Vec<&str> = elements.iter().map(|e| crate::icons::key_for(&self.repo, *e)).collect();
        keys.sort();
        keys.dedup();
        let ctx = egui::Context::default();
        for k in keys {
            std::fs::write(out.join(format!("assets/icons/{k}.svg")), icon_svg(&ctx, k)).map_err(io)?;
        }
        let root = self.repo.root;
        let index = format!(
            "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title></title></head>\n<frameset cols=\"25%,75%\"><frame src=\"contents/navigation.html\" name=\"navigation\"><frame src=\"contents/{}\" name=\"content\"></frameset></html>\n",
            self.page(root)
        );
        std::fs::write(out.join("index.html"), index).map_err(io)?;
        // navigation: Home, All Diagrams, Index, then the model tree
        let mut nav = format!(
            "<div class=\"nav\"><div class=\"links\"><a href=\"{}\" target=\"content\">Home</a><span class=\"sep\">|</span><a href=\"diagrams.html\" target=\"content\">All Diagrams</a><span class=\"sep\">|</span><a href=\"element_index.html\" target=\"content\">Index</a></div><ol>",
            self.page(root)
        );
        self.nav_tree(root, 0, &mut nav);
        nav.push_str("</ol></div>");
        std::fs::write(out.join("contents/navigation.html"), Self::html_page("Navigation", &nav)).map_err(io)?;
        // all diagrams
        let mut grid = String::from("<h1>All Diagrams</h1><div class=\"grid\">");
        for &d in &diagrams {
            let _ = write!(grid, "<a class=\"tile\" href=\"{}\"><img src=\"../diagrams/{}.svg\" alt=\"\"><div>{}</div></a>", self.page(d), md5_hex(&self.repo.id(d)), esc(&self.display_name(d)));
        }
        grid.push_str("</div>");
        std::fs::write(out.join("contents/diagrams.html"), Self::html_page("All Diagrams", &grid)).map_err(io)?;
        // the element index, by type, names sorted
        let mut by_type: std::collections::BTreeMap<String, Vec<Eid>> = Default::default();
        for &e in &elements {
            by_type.entry(self.repo.ty(e).to_string()).or_default().push(e);
        }
        let mut idx = String::from("<h1>Index</h1>");
        for (ty, mut list) in by_type {
            list.sort_by_key(|e| self.label_of(*e).to_lowercase());
            let _ = write!(idx, "<h2>{} ({})</h2><ul class=\"links\">", esc(&ty), list.len());
            for e in list {
                let _ = write!(idx, "<li>{}</li>", self.link(e));
            }
            idx.push_str("</ul>");
        }
        std::fs::write(out.join("contents/element_index.html"), Self::html_page("Index", &idx)).map_err(io)?;
        for &e in &elements {
            std::fs::write(out.join(format!("contents/{}", self.page(e))), self.element_page(e, &bounds)).map_err(io)?;
        }
        Ok(())
    }

    /// File > Export > HTML Docs...: pick a folder and write `<folder>/html-docs`.
    pub fn export_html_dialog(&mut self, ms: &dyn Measure) {
        let Some(dir) = rfd::FileDialog::new().set_title("Select a folder where HTML docs to be exported").pick_folder() else { return };
        match self.export_html(&dir.join("html-docs"), ms) {
            Ok(()) => self.toast("HTML docs exported successfully."),
            // an error toast, as in the original; the reason goes to the log (html-export/main.js:242-245)
            Err(e) => {
                eprintln!("Failed to export HTML docs: {e}");
                self.toast_error("Failed to export HTML docs.");
            }
        }
    }
}
