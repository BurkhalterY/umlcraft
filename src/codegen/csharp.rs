//! C#: a `.cs` file per top-level type in its package folder, inside `namespace A.B { … }`, and the
//! reverse of `.cs` files (block and file-scoped namespaces, classes, structs, records, interfaces,
//! enums, fields, properties and methods). Rules after staruml-csharp's generator.

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "object".to_string() } else { primitive(t, "string", "int", "bool", "double") };
    if is_many(mult) { format!("List<{t}>") } else { t }
}

fn default_return(t: &str) -> &'static str {
    match t {
        "bool" => "false",
        "int" | "long" | "short" | "byte" | "double" | "float" | "decimal" | "uint" | "ulong" => "0",
        "string" => "\"\"",
        _ => "null",
    }
}

fn vis(v: &str) -> &'static str {
    match v {
        "public" => "public ",
        "protected" => "protected ",
        "private" => "private ",
        "package" => "internal ",
        _ => "",
    }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    per_type(tree, |t| format!("{}.cs", ident(&t.name)), |p, t| {
        let mut w = W::new(o);
        w.line("using System;");
        w.line("using System.Collections.Generic;");
        w.blank();
        if !p.path.is_empty() {
            w.line(&format!("namespace {} {{", p.path.join(".")));
            w.inc();
        }
        write_type(&mut w, t, o);
        if !p.path.is_empty() {
            w.dec();
            w.line("}");
        }
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    w.doc(o, &t.doc, DocStyle::Slashes);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    let name = ident(&t.name);
    let bases: Vec<String> = t.supers.iter().chain(t.reals.iter()).cloned().collect();
    let colon = if bases.is_empty() { String::new() } else { format!(" : {}", bases.join(", ")) };
    match t.kind {
        Kind::Class => {
            let abs = t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
            w.line(&format!("{}{}class {name}{gen}{colon} {{", vis(&t.vis), if abs { "abstract " } else { "" }));
        }
        Kind::Interface => w.line(&format!("{}interface {name}{gen}{colon} {{", vis(&t.vis))),
        Kind::Enum => w.line(&format!("{}enum {name} {{", vis(&t.vis))),
    }
    w.inc();
    if t.kind == Kind::Enum {
        let lits: Vec<String> = t.literals.iter().map(|l| ident(l)).collect();
        w.line(&lits.join(",\n"));
    }
    for f in &t.fields {
        w.blank();
        w.doc(o, &f.doc, DocStyle::Slashes);
        let mut s = if t.kind == Kind::Interface { String::new() } else { vis(&f.vis).to_string() };
        if f.read_only && f.is_static && !f.default.is_empty() {
            s.push_str("const ");
        } else {
            if f.is_static {
                s.push_str("static ");
            }
            if f.read_only {
                s.push_str("readonly ");
            }
        }
        s.push_str(&format!("{} {}", map_type(&f.ty, &f.mult), ident(&f.name)));
        if t.kind == Kind::Interface {
            s.push_str(" { get; set; }");
        } else if !f.default.is_empty() {
            s.push_str(&format!(" = {};", f.default));
        } else {
            s.push(';');
        }
        w.line(&s);
    }
    for m in &t.methods {
        w.blank();
        let mut doc = m.doc.trim().to_string();
        if o.doc && !doc.is_empty() {
            doc = format!("<summary>\n{doc}\n</summary>");
        }
        for p in &m.params {
            doc.push_str(&format!("\n<param name=\"{}\"></param>", p.name));
        }
        if !m.ret.is_empty() {
            doc.push_str("\n<returns></returns>");
        }
        w.doc(o, doc.trim(), DocStyle::Slashes);
        let (rt, rm) = many(&m.ret);
        let ret = if m.ret.is_empty() { "void".to_string() } else { map_type(rt, rm) };
        let params: Vec<String> = m.params.iter().map(|p| {
            let (pt, pm) = many(&p.ty);
            format!("{} {}", map_type(pt, pm), ident(&p.name))
        }).collect();
        let interface = t.kind == Kind::Interface;
        let mut s = if interface { String::new() } else { vis(&m.vis).to_string() };
        if m.is_static {
            s.push_str("static ");
        }
        if m.is_abstract && !interface {
            s.push_str("abstract ");
        }
        s.push_str(&format!("{ret} {}({})", ident(&m.name), params.join(", ")));
        if interface || m.is_abstract {
            w.line(&format!("{s};"));
        } else {
            w.line(&format!("{s} {{"));
            w.inc();
            w.line("// TODO implement here");
            if !m.ret.is_empty() {
                w.line(&format!("return {};", default_return(&ret)));
            }
            w.dec();
            w.line("}");
        }
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o);
    }
    w.dec();
    w.line("}");
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 18] = ["public", "protected", "private", "internal", "static", "readonly", "const", "abstract", "virtual", "override", "sealed", "extern", "unsafe", "volatile", "async", "new", "partial", "required"];
const KINDS: [&str; 5] = ["class", "interface", "enum", "struct", "record"];

pub fn parse(src: &str) -> Parsed {
    let t = lex::lex(src, Cfg::default());
    let mut units = vec![];
    space(&t, vec![], &[], &mut units);
    Parsed { units, errors: !lex::balanced(&t) }
}

fn space(t: &[T], ns: Vec<String>, outer: &[Import], units: &mut Vec<Unit>) {
    let mut unit = Unit { ns: ns.clone(), types: vec![], imports: outer.to_vec() };
    let mut file_ns = ns.clone();
    for st in lex::statements(t, false) {
        let mut h = lex::skip_annotations(st.head);
        if h.first().map(|x| x.is("global")).unwrap_or(false) {
            h = &h[1..];
        }
        if h.first().map(|x| x.is("using")).unwrap_or(false) && st.block.is_none() && !h.get(1).map(|x| x.is("static") || x.is("(") || x.is("var")).unwrap_or(false) {
            // `using A.B;` opens a namespace; `using N = A.B.C;` names a type or a namespace
            match lex::find_top(h, "=", true) {
                Some(e) => {
                    let mut i = Import::of(&lex::join(&h[e + 1..]), &[]);
                    i.alias = h.get(1).map(|x| x.s.clone()).unwrap_or_default();
                    unit.imports.push(i);
                }
                None => unit.imports.push(Import { wildcard: true, ..Import::of(&lex::join(&h[1..]), &[]) }),
            }
            continue;
        }
        if h.first().map(|x| x.is("namespace")).unwrap_or(false) {
            let mut sub = ns.clone();
            sub.extend(lex::join(&h[1..]).split('.').map(|s| s.trim().to_string()));
            match st.block {
                Some(b) => space(b, sub, &unit.imports.clone(), units),
                // a file-scoped namespace holds the rest of the file
                None => {
                    file_ns = sub;
                    unit.ns = file_ns.clone();
                }
            }
        } else if let Some(b) = st.block {
            if let Some(k) = kind_at(h) {
                unit.types.push(read_type(h, k, b));
            }
        }
    }
    let _ = file_ns;
    units.push(unit);
}

fn kind_at(h: &[T]) -> Option<usize> {
    let k = h.iter().position(|x| x.k == K::Id && KINDS.contains(&x.s.as_str()))?;
    if h[..k].iter().all(|x| x.id() && MODS.contains(&x.s.as_str())) { Some(k) } else { None }
}

fn read_type(h: &[T], k: usize, body: &[T]) -> Ty {
    let mods: Vec<String> = h[..k].iter().map(|x| x.s.clone()).collect();
    // `record struct` and `record class`
    let mut k2 = k;
    if h[k].is("record") && h.get(k + 1).map(|x| x.is("struct") || x.is("class")).unwrap_or(false) {
        k2 += 1;
    }
    let kind = match h[k].s.as_str() {
        "interface" => Kind::Interface,
        "enum" => Kind::Enum,
        _ => Kind::Class,
    };
    let mut t = Ty::new(kind, &h.get(k2 + 1).map(|x| x.s.clone()).unwrap_or_default());
    t.doc = clean_xml_doc(&lex::doc_of(h));
    t.vis = vis_of(&mods, "package");
    t.is_abstract = mods.iter().any(|m| m == "abstract");
    t.is_leaf = mods.iter().any(|m| m == "sealed");
    if h[k].is("struct") {
        t.stereotype = "struct".into();
    }
    let (g, mut i) = lex::generic_params(h, k2 + 2);
    t.generics = g.iter().map(|p| p.0.clone()).collect();
    t.gen_bounds = where_bounds(h, &t.generics);
    if i < h.len() && h[i].is("(") {
        // a positional record's parameters are its properties
        let j = lex::close(h, i);
        for p in lex::commas(&h[i + 1..j]) {
            if let Some((n, ty, _)) = lex::typed_param(p, &MODS) {
                t.fields.push(Field { name: n, ty, vis: "public".into(), ..Default::default() });
            }
        }
        i = j + 1;
    }
    if i < h.len() && h[i].is(":") {
        let end = h[i..].iter().position(|x| x.is("where")).map(|p| p + i).unwrap_or(h.len());
        // the bases: an interface is named I + capital by convention; the kind is checked again on import
        for p in lex::commas(&h[i + 1..end]) {
            let n = lex::join(p);
            let simple = base_name(&n);
            let looks_interface = simple.len() > 1 && simple.starts_with('I') && simple.chars().nth(1).map(|c| c.is_uppercase()).unwrap_or(false);
            if kind == Kind::Enum {
                continue;
            }
            if looks_interface && kind != Kind::Interface { t.reals.push(n) } else { t.supers.push(n) }
        }
    }
    if kind == Kind::Enum {
        for p in lex::commas(body) {
            let p = lex::skip_annotations(p);
            if let Some(x) = p.first().filter(|x| x.id()) {
                t.literals.push(x.s.clone());
            }
        }
        return t;
    }
    let interface = kind == Kind::Interface;
    let default_vis = if interface { "public" } else { "private" };
    for st in lex::statements(body, false) {
        let head = lex::skip_annotations(st.head);
        if head.is_empty() {
            continue;
        }
        if let (Some(k), Some(b)) = (kind_at(head), st.block) {
            t.nested.push(read_type(head, k, b));
            continue;
        }
        let doc = clean_xml_doc(&lex::doc_of(st.head));
        // a property: `int X { get; set; }` or `int X => …;`
        let arrow = lex::find_top(head, "=>", true);
        let has_paren = lex::find_top(&head[..arrow.unwrap_or(head.len())], "(", true).is_some();
        if (st.block.is_some() && !has_paren) || (arrow.is_some() && !has_paren) {
            let decl = &head[..arrow.unwrap_or(head.len())];
            for d in lex::typed_decls(decl, &MODS) {
                let read_only = st.block.map(|b| !b.iter().any(|x| x.is("set") || x.is("init"))).unwrap_or(true);
                t.fields.push(Field { vis: vis_of(&d.mods, default_vis), is_static: d.mods.iter().any(|m| m == "static"), read_only, name: d.name, ty: d.ty, default: d.default, doc: doc.clone(), ..Default::default() });
            }
            continue;
        }
        for d in lex::typed_decls(head, &MODS) {
            let vis = vis_of(&d.mods, default_vis);
            let is_static = d.mods.iter().any(|m| m == "static" || m == "const");
            match d.params {
                Some(ps) => {
                    if d.ty.is_empty() && d.name != t.name && d.name.trim_start_matches('~') != t.name {
                        continue;
                    }
                    let ctor = d.ty.is_empty() && d.name == t.name;
                    t.methods.push(Method {
                        stereotype: if ctor { "constructor".into() } else { String::new() },
                        generics: d.generics.clone(),
                        raises: vec![],
                        is_leaf: d.mods.iter().any(|m| m == "sealed"),
                        concurrent: false,
                        tags: vec![],
                        name: d.name,
                        params: ps.into_iter().map(|(n, ty, def)| Param { name: n, ty, default: def }).collect(),
                        ret: d.ty,
                        vis,
                        is_static,
                        is_abstract: d.mods.iter().any(|m| m == "abstract") || (interface && st.block.is_none()),
                        doc: doc.clone(),
                    });
                }
                None => t.fields.push(Field { name: d.name, ty: d.ty, vis, is_static, read_only: d.mods.iter().any(|m| m == "readonly" || m == "const"), default: d.default, doc: doc.clone(), ..Default::default() }),
            }
        }
    }
    t
}

/// The constraints of `where T : A, new() where U : B` for each generic, as (bound, default).
fn where_bounds(h: &[T], names: &[String]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = names.iter().map(|_| (String::new(), String::new())).collect();
    let starts: Vec<usize> = h.iter().enumerate().filter(|(_, x)| x.is("where")).map(|(i, _)| i).collect();
    for (k, &s) in starts.iter().enumerate() {
        let end = starts.get(k + 1).copied().unwrap_or(h.len());
        let part = &h[s + 1..end];
        if let (Some(n), Some(c)) = (part.first(), lex::find_top(part, ":", true)) {
            if let Some(i) = names.iter().position(|x| *x == n.s) {
                out[i].0 = lex::join(&part[c + 1..]);
            }
        }
    }
    out
}

/// `<summary>text</summary>` → text.
fn clean_xml_doc(d: &str) -> String {
    let mut s = d.to_string();
    for tag in ["<summary>", "</summary>"] {
        s = s.replace(tag, "");
    }
    s.lines().map(|l| l.trim()).filter(|l| !l.starts_with("<param") && !l.starts_with("<returns")).collect::<Vec<_>>().join("\n").trim().to_string()
}
