//! Java: a `.java` file per top-level type in its package folder, and the reverse of `.java`
//! files (rules of staruml-java's code-generator.js: package declaration from the base package,
//! a default constructor, `// TODO implement here` bodies with a default return).

use super::lex::{self, Cfg, Decl, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "Object".to_string() } else { primitive(t, "String", "int", "boolean", "double") };
    if is_many(mult) { format!("List<{}>", boxed(&t)) } else { t }
}

fn boxed(t: &str) -> String {
    match t {
        "int" => "Integer".into(),
        "boolean" => "Boolean".into(),
        "double" => "Double".into(),
        "long" => "Long".into(),
        "float" => "Float".into(),
        "char" => "Character".into(),
        _ => t.into(),
    }
}

/// The default return of a method body, by type (writeMethod in code-generator.js).
pub fn default_return(t: &str) -> &'static str {
    match t {
        "boolean" => "false",
        "int" | "long" | "short" | "byte" => "0",
        "float" => "0.0f",
        "double" => "0.0d",
        "char" => "'0'",
        "String" => "\"\"",
        _ => "null",
    }
}

fn vis(v: &str) -> &'static str {
    match v {
        "public" => "public ",
        "protected" => "protected ",
        "private" => "private ",
        _ => "",
    }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let interfaces: Vec<Ty> = tree.all().iter().flat_map(|p| p.types.iter()).filter(|t| t.kind == Kind::Interface).cloned().collect();
    per_type(tree, |t| format!("{}.java", ident(&t.name)), |p, t| {
        let mut w = W::new(o);
        if !p.path.is_empty() {
            w.line(&format!("package {};", p.path.join(".")));
            w.blank();
        }
        w.line("import java.util.*;");
        w.blank();
        write_type(&mut w, t, o, &interfaces);
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts, interfaces: &[Ty]) {
    w.doc(o, &t.doc, DocStyle::Block);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    let name = ident(&t.name);
    match t.kind {
        Kind::Class => {
            let abs = t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
            let mut head = format!("{}{}class {name}{gen}", vis(&t.vis), if abs { "abstract " } else { "" });
            if let Some(s) = t.supers.first() {
                head.push_str(&format!(" extends {s}"));
            }
            if !t.reals.is_empty() {
                head.push_str(&format!(" implements {}", t.reals.join(", ")));
            }
            w.line(&format!("{head} {{"));
            w.inc();
            fields(w, t, o);
            w.blank();
            w.doc(o, "Default constructor", DocStyle::Block);
            w.line(&format!("{}{name}() {{", vis(&t.vis)));
            w.line("}");
            for m in &t.methods {
                w.blank();
                method(w, m, o, false, None);
            }
            // the operations of the realized interfaces, to implement (writeClass, code-generator.js)
            for r in &t.reals {
                let Some(i) = interfaces.iter().find(|i| &i.name == r) else { continue };
                for m in i.methods.iter().filter(|m| !t.methods.iter().any(|x| x.name == m.name)) {
                    w.blank();
                    let m = Method { vis: "public".into(), is_abstract: false, ..m.clone() };
                    method(w, &m, o, false, Some(&i.name));
                }
            }
        }
        Kind::Interface => {
            let mut head = format!("{}interface {name}{gen}", vis(&t.vis));
            if !t.supers.is_empty() {
                head.push_str(&format!(" extends {}", t.supers.join(", ")));
            }
            w.line(&format!("{head} {{"));
            w.inc();
            fields(w, t, o);
            for m in &t.methods {
                w.blank();
                method(w, m, o, true, None);
            }
        }
        Kind::Enum => {
            w.line(&format!("{}enum {name} {{", vis(&t.vis)));
            w.inc();
            let lits: Vec<String> = t.literals.iter().map(|l| ident(l)).collect();
            w.line(&format!("{};", lits.join(", ")));
            fields(w, t, o);
            for m in &t.methods {
                w.blank();
                method(w, m, o, false, None);
            }
        }
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o, interfaces);
    }
    w.dec();
    w.line("}");
}

fn fields(w: &mut W, t: &Ty, o: &GenOpts) {
    for f in &t.fields {
        w.blank();
        w.doc(o, &f.doc, DocStyle::Block);
        let mut s = vis(&f.vis).to_string();
        if f.is_static {
            s.push_str("static ");
        }
        if f.read_only {
            s.push_str("final ");
        }
        s.push_str(&format!("{} {}", map_type(&f.ty, &f.mult), ident(&f.name)));
        if !f.default.is_empty() {
            s.push_str(&format!(" = {}", f.default));
        }
        s.push(';');
        w.line(&s);
    }
}

fn method(w: &mut W, m: &Method, o: &GenOpts, in_interface: bool, from: Option<&str>) {
    let mut doc = m.doc.trim().to_string();
    for p in &m.params {
        doc.push_str(&format!("\n@param {}", p.name));
    }
    if !m.ret.is_empty() {
        doc.push_str("\n@return");
    }
    w.doc(o, doc.trim(), DocStyle::Block);
    let ret = if m.ret.is_empty() { "void".to_string() } else { map_type(m.ret.trim_end_matches("[]"), if m.ret.ends_with("[]") { "*" } else { "" }) };
    let params: Vec<String> = m.params.iter().map(|p| format!("{} {}", map_type(p.ty.trim_end_matches("[]"), if p.ty.ends_with("[]") { "*" } else { "" }), ident(&p.name))).collect();
    let mut s = if in_interface { String::new() } else { vis(&m.vis).to_string() };
    if m.is_static {
        s.push_str("static ");
    }
    if m.is_abstract && !in_interface {
        s.push_str("abstract ");
    }
    s.push_str(&format!("{ret} {}({})", ident(&m.name), params.join(", ")));
    if in_interface || m.is_abstract {
        w.line(&format!("{s};"));
        return;
    }
    w.line(&format!("{s} {{"));
    w.inc();
    match from {
        Some(i) => w.line(&format!("// TODO implement {i}.{}() here", m.name)),
        None => w.line("// TODO implement here"),
    }
    if !m.ret.is_empty() {
        w.line(&format!("return {};", default_return(&ret)));
    }
    w.dec();
    w.line("}");
}

// ------------------------------------------------------------------ reverse

/// The `extends A, B` and `implements C` lists of a type head (`stop` words end them).
pub fn super_lists(h: &[T], ext: &[&str], imp: &[&str], stop: &[&str]) -> (Vec<String>, Vec<String>) {
    let (mut supers, mut reals) = (vec![], vec![]);
    let (mut cur, mut into): (Vec<T>, u8) = (vec![], 0);
    let flush = |cur: &mut Vec<T>, into: u8, supers: &mut Vec<String>, reals: &mut Vec<String>| {
        for p in lex::commas(cur) {
            let n = lex::join(p);
            match into {
                1 => supers.push(n),
                2 => reals.push(n),
                _ => {}
            }
        }
        cur.clear();
    };
    for x in h {
        if x.id() && ext.contains(&x.s.as_str()) {
            flush(&mut cur, into, &mut supers, &mut reals);
            into = 1;
        } else if x.id() && imp.contains(&x.s.as_str()) {
            flush(&mut cur, into, &mut supers, &mut reals);
            into = 2;
        } else if x.id() && stop.contains(&x.s.as_str()) {
            flush(&mut cur, into, &mut supers, &mut reals);
            into = 0;
        } else {
            cur.push(x.clone());
        }
    }
    flush(&mut cur, into, &mut supers, &mut reals);
    (supers, reals)
}

/// The java.util collections the 7.x reverse reads as multiplicity `*` (code-analyzer.js).
const COLLECTIONS: [&str; 13] = ["Collection", "Set", "SortedSet", "NavigableSet", "HashSet", "TreeSet", "LinkedHashSet", "List", "ArrayList", "LinkedList", "Deque", "ArrayDeque", "Queue"];

pub const MODS: [&str; 13] = ["public", "protected", "private", "static", "final", "abstract", "synchronized", "native", "transient", "volatile", "strictfp", "default", "sealed"];

pub fn parse(src: &str) -> Parsed {
    let t = lex::lex(src, Cfg::default());
    let mut unit = Unit::default();
    for st in lex::statements(&t, false) {
        let h = st.head;
        if h.first().map(|x| x.is("package")).unwrap_or(false) {
            unit.ns = lex::join(&h[1..]).split('.').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        } else if h.first().map(|x| x.is("import")).unwrap_or(false) {
            // `import a.b.C;`, `import a.b.*;`; static imports name members, not types
            if !h.get(1).map(|x| x.is("static")).unwrap_or(false) {
                unit.imports.push(Import::of(&lex::join(&h[1..]), &[]));
            }
        } else if let (Some(b), Some(ty)) = (st.block, type_head(h)) {
            let mut ty = read_type(ty, b, false);
            // the documentation comment may come before the annotations
            if ty.doc.is_empty() {
                ty.doc = lex::doc_of(h);
            }
            unit.types.push(ty);
        }
    }
    Parsed { errors: !lex::balanced(&t), units: vec![unit] }
}

/// A type declaration's head: (kind word position, tokens).
fn type_head(h: &[T]) -> Option<&[T]> {
    let h = lex::skip_annotations(h);
    let k = h.iter().position(|x| x.k == K::Id && ["class", "interface", "enum", "record"].contains(&x.s.as_str()))?;
    // `new X() { … }` and lambdas are not declarations
    if h[..k].iter().any(|x| !x.id() && !x.is("@")) {
        return None;
    }
    Some(h)
}

fn read_type(h: &[T], body: &[T], in_interface: bool) -> Ty {
    let k = h.iter().position(|x| x.k == K::Id && ["class", "interface", "enum", "record"].contains(&x.s.as_str())).unwrap();
    let mods: Vec<String> = h[..k].iter().filter(|x| x.id()).map(|x| x.s.clone()).collect();
    let kind = match h[k].s.as_str() {
        "interface" => Kind::Interface,
        "enum" => Kind::Enum,
        _ => Kind::Class,
    };
    let is_annotation = k > 0 && h[k - 1].is("@");
    let name = h.get(k + 1).map(|x| x.s.clone()).unwrap_or_default();
    // an annotation type is a class «annotationType» (the 7.x reverse); its elements read as an interface's
    let mut t = Ty::new(if is_annotation { Kind::Class } else { kind }, &name);
    t.doc = lex::doc_of(h);
    t.vis = vis_of(&mods, if in_interface { "public" } else { "package" });
    t.is_abstract = mods.iter().any(|m| m == "abstract");
    t.is_leaf = mods.iter().any(|m| m == "final");
    if is_annotation {
        t.stereotype = "annotationType".into();
    }
    let (g, mut i) = lex::generic_params(h, k + 2);
    t.generics = g.iter().map(|p| p.0.clone()).collect();
    t.gen_bounds = g.into_iter().map(|p| (p.1, p.2)).collect();
    // a record's components are its fields
    if i < h.len() && h[i].is("(") {
        let j = lex::close(h, i);
        for p in lex::commas(&h[i + 1..j]) {
            if let Some((n, ty, _)) = lex::typed_param(p, &MODS) {
                t.fields.push(Field { name: n, ty, vis: "private".into(), read_only: true, ..Default::default() });
            }
        }
        i = j + 1;
    }
    let (supers, reals) = super_lists(&h[i.min(h.len())..], &["extends"], &["implements"], &["permits"]);
    t.supers = supers;
    t.reals = reals;
    members(&mut t, body, kind == Kind::Interface);
    t
}

fn members(t: &mut Ty, body: &[T], interface: bool) {
    let mut body = body;
    if t.kind == Kind::Enum {
        // the literals, up to the first `;`
        let end = lex::find_top(body, ";", false).unwrap_or(body.len());
        for p in lex::commas(&body[..end]) {
            let p = lex::skip_annotations(p);
            if let Some(x) = p.first().filter(|x| x.id()) {
                t.literals.push(x.s.clone());
                t.literal_docs.push(lex::doc_of(p));
            }
        }
        body = &body[(end + 1).min(body.len())..];
    }
    for st in lex::statements(body, false) {
        if st.head.is_empty() {
            continue;
        }
        if let Some(h) = type_head(st.head) {
            if let Some(b) = st.block {
                let mut n = read_type(h, b, interface);
                if n.doc.is_empty() {
                    n.doc = lex::doc_of(st.head);
                }
                t.nested.push(n);
                continue;
            }
        }
        // annotations anywhere in the declaration are skipped, and a method's own generics read
        let head = lex::strip_annotations(st.head);
        let (head, mg) = lex::take_method_generics(&head, &MODS);
        for d in lex::typed_decls(&head, &MODS) {
            member(t, d, lex::doc_of(&head), st.block.is_some(), interface, &mg);
        }
    }
}

fn member(t: &mut Ty, d: Decl, doc: String, has_body: bool, interface: bool, generics: &[(String, String, String)]) {
    if d.name.is_empty() {
        return;
    }
    let vis = vis_of(&d.mods, if interface { "public" } else { "package" });
    let is_static = d.mods.iter().any(|m| m == "static");
    match d.params {
        Some(ps) => {
            let ctor = d.ty.is_empty();
            if d.ty.is_empty() && d.name != t.name {
                return;
            }
            // `throws A, B` after the parameters
            let raises = d.tail.split_once("throws").map(|(_, r)| r.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default();
            let mut tags: Vec<(String, String)> = d.mods.iter().filter(|m| *m == "native" || *m == "strictfp").map(|m| (m.clone(), String::new())).collect();
            // an annotation type's element default (`String value() default "";`)
            if let Some((_, v)) = d.tail.split_once("default") {
                tags.push(("default".into(), v.trim().to_string()));
            }
            t.methods.push(Method {
                is_leaf: d.mods.iter().any(|m| m == "final"),
                concurrent: d.mods.iter().any(|m| m == "synchronized"),
                tags,
                stereotype: if ctor { "constructor".into() } else { String::new() },
                generics: generics.iter().map(|g| g.0.clone()).collect(),
                raises,
                name: d.name,
                params: ps.into_iter().map(|(n, ty, def)| Param { name: n, ty, default: def }).collect(),
                ret: d.ty,
                vis,
                is_static,
                is_abstract: d.mods.iter().any(|m| m == "abstract") || (interface && !has_body && !is_static && !d.mods.iter().any(|m| m == "default")),
                doc,
            });
        }
        None => {
            let fin = d.mods.iter().any(|m| m == "final") || interface;
            let mut tags: Vec<(String, String)> = d.mods.iter().filter(|m| *m == "volatile" || *m == "transient").map(|m| (m.clone(), String::new())).collect();
            // the declared collection of a `List<Book>` field (the 7.x reverse's `collection` tag)
            if let Some(head) = d.ty.split('<').next().filter(|_| d.ty.contains('<')) {
                let head = head.rsplit('.').next().unwrap_or(head).trim();
                if COLLECTIONS.contains(&head) {
                    tags.push(("collection".into(), head.to_string()));
                }
            }
            t.fields.push(Field { name: d.name, ty: d.ty, vis, is_static: is_static || interface, read_only: fin, is_leaf: fin, default: d.default, doc, tags, ..Default::default() })
        }
    }
}
