//! File > Export > HTML Docs: a static web site of the model, one page per element and an SVG per
//! diagram (extensions/default/html-export).
//!
//! The layout follows the original: `index.html` (a 25%/75% frameset), `contents/navigation.html`,
//! `contents/diagrams.html`, `contents/element_index.html`, `contents/<md5(_id)>.html` for each element
//! and `diagrams/<md5(_id)>.svg` for each diagram. The pages and their style sheet are UMLCraft's own.

use crate::app::App;
use crate::markdown::{parse, spans, Block};
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

fn inline_html(t: &str) -> String {
    let mut out = String::new();
    for (text, bold, italic, code) in spans(t) {
        let mut h = esc(&text);
        if code {
            h = format!("<code>{h}</code>");
        }
        if italic {
            h = format!("<em>{h}</em>");
        }
        if bold {
            h = format!("<strong>{h}</strong>");
        }
        out += &h;
    }
    out
}

/// Markdown to HTML, for the Description sections (the subset the preview draws).
pub fn markdown_html(text: &str) -> String {
    let mut out = String::new();
    let mut list: Option<&str> = None;
    for b in parse(text) {
        let tag = match &b {
            Block::Bullet(..) => Some("ul"),
            Block::Numbered(..) => Some("ol"),
            _ => None,
        };
        if list != tag {
            if let Some(l) = list {
                let _ = write!(out, "</{l}>");
            }
            if let Some(l) = tag {
                let _ = write!(out, "<{l}>");
            }
            list = tag;
        }
        match b {
            Block::Heading(n, t) => {
                let _ = write!(out, "<h{n}>{}</h{n}>", inline_html(&t));
            }
            Block::Paragraph(t) => {
                let _ = write!(out, "<p>{}</p>", inline_html(&t));
            }
            Block::Bullet(_, t) | Block::Numbered(_, _, t) => {
                let _ = write!(out, "<li>{}</li>", inline_html(&t));
            }
            Block::Quote(t) => {
                let _ = write!(out, "<blockquote>{}</blockquote>", inline_html(&t));
            }
            Block::Code(t) => {
                let _ = write!(out, "<pre><code>{}</code></pre>", esc(&t));
            }
            Block::Rule => out += "<hr>",
        }
    }
    if let Some(l) = list {
        let _ = write!(out, "</{l}>");
    }
    out
}

const STYLE: &str = "body{font-family:system-ui,-apple-system,'Segoe UI',Arial,sans-serif;font-size:14px;color:#222;margin:16px 20px;line-height:1.45}
a{color:#1f6fd1;text-decoration:none}a:hover{text-decoration:underline}
h1{font-size:22px;margin:0 0 4px}h2{font-size:16px;margin:22px 0 6px;border-bottom:1px solid #ddd;padding-bottom:3px}
.type{color:#666}.path{color:#888;font-size:12px;margin-bottom:10px}.st{color:#7a4f00}
table{border-collapse:collapse;width:100%;margin:4px 0}th,td{border:1px solid #ddd;padding:4px 8px;text-align:left;vertical-align:top}th{background:#f4f4f4}
ul.links{margin:4px 0;padding-left:20px}.grid{display:flex;flex-wrap:wrap;gap:14px}
.tile{width:220px;border:1px solid #ddd;border-radius:4px;padding:6px;text-align:center}.tile img{max-width:100%;max-height:150px}
.nav{font-size:13px}.nav details{margin-left:12px}.nav summary{cursor:pointer}.nav .leaf{margin-left:26px;display:block}
.diagram{border:1px solid #eee}pre{background:#f6f6f6;padding:8px;overflow:auto}blockquote{color:#666;border-left:3px solid #ddd;margin:4px 0;padding-left:10px}";

impl App {
    fn page(&self, e: Eid) -> String {
        format!("{}.html", md5_hex(&self.repo.id(e)))
    }

    fn link(&self, e: Eid) -> String {
        format!("<a href=\"{}\">{}</a>", self.page(e), esc(&self.display_name(e)))
    }

    fn display_name(&self, e: Eid) -> String {
        let n = self.label_of(e);
        if n.trim().is_empty() { "(unnamed)".into() } else { n }
    }

    /// The elements of the model tree under `e`, in their order (views left out).
    fn doc_children(&self, e: Eid) -> Vec<Eid> {
        self.repo.children(e).into_iter().filter(|(k, _)| k != "ownedViews" && k != "subViews").map(|(_, c)| c).collect()
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

    fn nav_tree(&self, e: Eid, depth: usize, out: &mut String) {
        let mut kids = self.doc_children(e);
        kids.sort_by_key(|k| self.display_name(*k).to_lowercase());
        let label = format!("<a href=\"{}\" target=\"diagram\">{}</a>", self.page(e), esc(&self.display_name(e)));
        if kids.is_empty() || depth >= 8 {
            let _ = write!(out, "<span class=\"leaf\">{label}</span>");
            return;
        }
        let _ = write!(out, "<details{}><summary>{label}</summary>", if depth == 0 { " open" } else { "" });
        for k in kids {
            self.nav_tree(k, depth + 1, out);
        }
        out.push_str("</details>");
    }

    fn links_section(&self, out: &mut String, title: &str, items: &[Eid]) {
        if items.is_empty() {
            return;
        }
        let _ = write!(out, "<h2>{}</h2><ul class=\"links\">", esc(title));
        for i in items {
            let _ = write!(out, "<li>{}</li>", self.link(*i));
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

    fn member_table(&self, out: &mut String, title: &str, items: &[Eid]) {
        if items.is_empty() {
            return;
        }
        let _ = write!(out, "<h2>{}</h2><table><tr><th>Visibility</th><th>Name</th><th>Type</th><th>Description</th></tr>", esc(title));
        for i in items {
            let ty = match self.repo.ty(*i) {
                "UMLOperation" => self
                    .repo
                    .list(*i, "parameters")
                    .into_iter()
                    .find(|p| self.repo.str(*p, "direction") == "return")
                    .map(|p| views::type_name(&self.repo, p, "type"))
                    .unwrap_or_default(),
                "UMLTemplateParameter" => self.repo.str(*i, "parameterType"),
                // a reception's signal; an extension point's location
                "UMLReception" => views::type_name(&self.repo, *i, "signal"),
                "UMLExtensionPoint" => self.repo.str(*i, "location"),
                _ => views::type_name(&self.repo, *i, "type"),
            };
            let shown = if self.repo.ty(*i) == "UMLOperation" { views::operation_text(&self.repo, *i, views::Show { vis: false, ..views::Show::ALL }) } else { self.repo.name(*i) };
            let _ = write!(
                out,
                "<tr><td>{}</td><td><a href=\"{}\">{}</a></td><td>{}</td><td>{}</td></tr>",
                esc(&self.repo.str(*i, "visibility")),
                self.page(*i),
                esc(&shown),
                esc(&ty),
                markdown_html(&self.repo.str(*i, "documentation"))
            );
        }
        out.push_str("</table>");
    }

    /// The page of one element: header, diagram with its image map, description and sections.
    fn element_page(&self, e: Eid, svg_bounds: &std::collections::HashMap<Eid, views::R>) -> String {
        let r = &self.repo;
        let mut b = String::new();
        let st = views::stereotype_text(r, e);
        let path = self.path_name(e);
        let _ = write!(b, "<h1>{}</h1><div class=\"type\">{}{}</div>", esc(&self.display_name(e)), esc(r.ty(e)), if st.is_empty() { String::new() } else { format!(" <span class=\"st\">{}</span>", esc(&st)) });
        let _ = write!(b, "<div class=\"path\">{}</div>", esc(&path));
        if self.is_diagram(e) {
            let id = md5_hex(&r.id(e));
            let _ = write!(b, "<img class=\"diagram\" src=\"../diagrams/{id}.svg\" usemap=\"#map-{id}\" alt=\"{}\"><map name=\"map-{id}\">", esc(&r.name(e)));
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
        let doc = r.str(e, "documentation");
        if !doc.trim().is_empty() {
            let _ = write!(b, "<h2>Description</h2>{}", markdown_html(&doc));
        }
        let spec = r.str(e, "specification");
        if !spec.trim().is_empty() {
            let _ = write!(b, "<h2>Specification</h2><pre>{}</pre>", esc(&spec));
        }
        // the ends of a relationship
        if r.has(e, "end1") {
            for (title, k) in [("End1", "end1"), ("End2", "end2")] {
                if let Some(x) = r.child(e, k).and_then(|end| r.refd(end, "reference")) {
                    self.links_section(&mut b, title, &[x]);
                }
            }
        } else if let (Some(s), Some(t)) = (r.refd(e, "source"), r.refd(e, "target")) {
            self.links_section(&mut b, "Source", &[s]);
            self.links_section(&mut b, "Target", &[t]);
        }
        if r.kind_of(e, "UMLClassifier") {
            self.links_section(&mut b, "Generalized Elements", &self.related(e, "UMLGeneralization", true));
            self.links_section(&mut b, "Specialized Elements", &self.related(e, "UMLGeneralization", false));
            self.links_section(&mut b, "Interfaces", &self.related(e, "UMLInterfaceRealization", true));
            if r.ty(e) == "UMLInterface" {
                self.links_section(&mut b, "Implementing Classifiers", &self.related(e, "UMLInterfaceRealization", false));
            }
            self.links_section(&mut b, "Components", &self.related(e, "UMLComponentRealization", true));
            if r.ty(e) == "UMLComponent" {
                self.links_section(&mut b, "Realizing Classifiers", &self.related(e, "UMLComponentRealization", false));
            }
            self.links_section(&mut b, "Deployment Targets", &self.related(e, "UMLDeployment", true));
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
        self.member_table(&mut b, "Template Parameters", &r.list(e, "templateParameters"));
        self.member_table(&mut b, "Literals", &r.list(e, "literals"));
        self.member_table(&mut b, "Attributes", &r.list(e, "attributes"));
        self.member_table(&mut b, "Operations", &r.list(e, "operations"));
        self.member_table(&mut b, "Receptions", &r.list(e, "receptions"));
        self.member_table(&mut b, "Extension Points", &r.list(e, "extensionPoints"));
        self.member_table(&mut b, "Parameters", &r.list(e, "parameters"));
        let owned: Vec<Eid> = self
            .doc_children(e)
            .into_iter()
            .filter(|c| !matches!(r.ty(*c), "UMLAttribute" | "UMLOperation" | "UMLReception" | "UMLExtensionPoint" | "UMLEnumerationLiteral" | "UMLParameter" | "UMLTemplateParameter"))
            .collect();
        self.links_section(&mut b, "Owned Elements", &owned);
        Self::html_page(&self.display_name(e), &b)
    }

    /// Write the site into `out` (created if needed).
    pub fn export_html(&mut self, out: &Path, ms: &dyn Measure) -> Result<(), String> {
        let io = |e: std::io::Error| e.to_string();
        for sub in ["contents", "diagrams", "assets"] {
            std::fs::create_dir_all(out.join(sub)).map_err(io)?;
        }
        std::fs::write(out.join("assets/style.css"), STYLE).map_err(io)?;
        // the diagrams, as SVG named by the MD5 of their id
        let diagrams = self.print_diagrams(true);
        let mut bounds = std::collections::HashMap::new();
        for &d in &diagrams {
            let svg = crate::export::render(self, d, ms, crate::export::Format::Svg, 1.0)?;
            std::fs::write(out.join(format!("diagrams/{}.svg", md5_hex(&self.repo.id(d)))), svg).map_err(io)?;
            if let Some(b) = crate::render::bounds(&self.repo, d) {
                bounds.insert(d, b);
            }
        }
        let root = self.repo.root;
        let title = self.display_name(root);
        let index = format!(
            "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{}</title></head>\n<frameset cols=\"25%,75%\"><frame src=\"contents/navigation.html\" name=\"navigation\"><frame src=\"contents/{}\" name=\"diagram\"></frameset></html>\n",
            esc(&title),
            self.page(root)
        );
        std::fs::write(out.join("index.html"), index).map_err(io)?;
        // navigation: Home, All Diagrams, Index, then the model tree
        let mut nav = format!(
            "<div class=\"nav\"><p><a href=\"{}\" target=\"diagram\">Home</a> · <a href=\"diagrams.html\" target=\"diagram\">All Diagrams</a> · <a href=\"element_index.html\" target=\"diagram\">Index</a></p>",
            self.page(root)
        );
        self.nav_tree(root, 0, &mut nav);
        nav.push_str("</div>");
        std::fs::write(out.join("contents/navigation.html"), Self::html_page(&title, &nav)).map_err(io)?;
        // all diagrams
        let mut grid = String::from("<h1>All Diagrams</h1><div class=\"grid\">");
        for &d in &diagrams {
            let _ = write!(grid, "<a class=\"tile\" href=\"{}\"><img src=\"../diagrams/{}.svg\" alt=\"\"><div>{}</div></a>", self.page(d), md5_hex(&self.repo.id(d)), esc(&self.repo.name(d)));
        }
        grid.push_str("</div>");
        std::fs::write(out.join("contents/diagrams.html"), Self::html_page("All Diagrams", &grid)).map_err(io)?;
        // the element index, by type, names sorted
        let elements = self.doc_elements();
        let mut by_type: std::collections::BTreeMap<String, Vec<Eid>> = Default::default();
        for &e in &elements {
            by_type.entry(self.repo.ty(e).to_string()).or_default().push(e);
        }
        let mut idx = String::from("<h1>Index</h1>");
        for (ty, mut list) in by_type {
            list.sort_by_key(|e| self.display_name(*e).to_lowercase());
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
            Err(e) => self.message = Some(("Failed to export HTML docs.".into(), e)),
        }
    }
}
