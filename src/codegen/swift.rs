//! Swift: a `.swift` file per type in its package folder; classes with `init()`, `static` members,
//! `fatalError` bodies for operations that return something, protocols for interfaces, enums with
//! `case`s. The reverse reads classes, structs, actors, protocols, enums and extensions (merged
//! into their type), properties, functions with their argument labels, and protocol conformances.

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "Any".to_string() } else { primitive(t, "String", "Int", "Bool", "Double") };
    let t = match t.as_str() {
        "int" => "Int".to_string(),
        "double" => "Double".into(),
        "float" => "Float".into(),
        "boolean" | "bool" => "Bool".into(),
        "string" => "String".into(),
        _ => t,
    };
    if is_many(mult) {
        format!("[{t}]")
    } else if is_optional(mult) {
        format!("{t}?")
    } else {
        t
    }
}

fn initial(t: &str) -> Option<&'static str> {
    match t {
        "Int" | "Int64" | "UInt" => Some("0"),
        "Double" | "Float" => Some("0.0"),
        "Bool" => Some("false"),
        "String" => Some("\"\""),
        _ if t.ends_with('?') => Some("nil"),
        _ if t.starts_with('[') => Some("[]"),
        _ => None,
    }
}

fn vis(v: &str) -> &'static str {
    match v {
        "public" => "public ",
        "private" => "private ",
        "protected" => "fileprivate ",
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
    per_type(tree, |t| format!("{}.swift", ident(&t.name)), |_, t| {
        let mut w = W::new(o);
        w.line("import Foundation");
        w.blank();
        write_type(&mut w, t, o);
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    w.doc(o, &t.doc, DocStyle::Slashes);
    let name = ident(&t.name);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    let bases: Vec<String> = t.supers.iter().chain(t.reals.iter()).cloned().collect();
    let colon = if bases.is_empty() { String::new() } else { format!(": {}", bases.join(", ")) };
    let protocol = t.kind == Kind::Interface;
    match t.kind {
        Kind::Enum => {
            w.line(&format!("{}enum {name} {{", vis(&t.vis)));
            w.inc();
            for l in &t.literals {
                w.line(&format!("case {}", ident(l)));
            }
            w.dec();
            w.line("}");
            return;
        }
        Kind::Interface => w.line(&format!("{}protocol {name}{colon} {{", vis(&t.vis))),
        Kind::Class => {
            let kw = if t.stereotype == "struct" { "struct" } else { "class" };
            w.line(&format!("{}{kw} {name}{gen}{colon} {{", vis(&t.vis)));
        }
    }
    w.inc();
    for f in &t.fields {
        w.doc(o, &f.doc, DocStyle::Slashes);
        let ty = map_type(&f.ty, &f.mult);
        let st = if f.is_static { "static " } else { "" };
        if protocol {
            w.line(&format!("{st}var {}: {ty} {{ get{} }}", ident(&f.name), if f.read_only { "" } else { " set" }));
            continue;
        }
        let kw = if f.read_only { "let" } else { "var" };
        let init = if f.default.is_empty() { initial(&ty).map(String::from) } else { Some(f.default.clone()) };
        match init {
            Some(v) => w.line(&format!("{}{st}{kw} {}: {ty} = {v}", vis(&f.vis), ident(&f.name))),
            None => w.line(&format!("{}{st}var {}: {ty}!", vis(&f.vis), ident(&f.name))),
        }
    }
    if t.kind == Kind::Class {
        w.blank();
        w.line(&format!("{}init() {{", if t.supers.is_empty() { "" } else { "override " }));
        if !t.supers.is_empty() {
            w.inc();
            w.line("super.init()");
            w.dec();
        }
        w.line("}");
    }
    for m in &t.methods {
        w.blank();
        w.doc(o, &m.doc, DocStyle::Slashes);
        let ps: Vec<String> = m.params.iter().map(|p| {
            let (pt, pm) = many(&p.ty);
            let d = if p.default.is_empty() { String::new() } else { format!(" = {}", p.default) };
            format!("{}: {}{d}", ident(&p.name), map_type(pt, pm))
        }).collect();
        let (rt, rm) = many(&m.ret);
        let ret = if m.ret.is_empty() { String::new() } else { format!(" -> {}", map_type(rt, rm)) };
        let st = if m.is_static { "static " } else { "" };
        let s = format!("{}{st}func {}({}){ret}", if protocol { "" } else { vis(&m.vis) }, ident(&m.name), ps.join(", "));
        if protocol {
            w.line(&s);
            continue;
        }
        w.line(&format!("{s} {{"));
        w.inc();
        w.line("// TODO implement here");
        if !m.ret.is_empty() {
            w.line("fatalError(\"Not implemented\")");
        }
        w.dec();
        w.line("}");
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o);
    }
    w.dec();
    w.line("}");
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 20] = [
    "public", "open", "internal", "fileprivate", "private", "final", "static", "class", "override", "mutating", "nonmutating", "lazy", "weak", "unowned", "required", "convenience", "dynamic", "indirect", "nonisolated", "optional",
];
const KINDS: [&str; 6] = ["class", "struct", "protocol", "enum", "extension", "actor"];

pub fn parse(src: &str, rel: &[String]) -> Parsed {
    let t = lex::lex(src, Cfg::default());
    let mut types: Vec<Ty> = vec![];
    for st in lex::statements(&t, true) {
        if let Some(ty) = read_type(st.head, st.block) {
            match types.iter_mut().find(|x| x.name == ty.name) {
                // an extension adds to its type
                Some(x) => {
                    x.supers.extend(ty.supers);
                    x.fields.extend(ty.fields);
                    x.methods.extend(ty.methods);
                    x.nested.extend(ty.nested);
                }
                None => types.push(ty),
            }
        }
    }
    Parsed { units: vec![Unit { ns: rel.to_vec(), types }], errors: !lex::balanced(&t) }
}

/// The kind word of a type declaration (`class` as a modifier of `class func` is not one).
fn kind_at(h: &[T]) -> Option<usize> {
    let mut i = 0;
    while i < h.len() && h[i].id() && MODS.contains(&h[i].s.as_str()) {
        if h[i].is("class") && h.get(i + 1).map(|x| x.id() && !MODS.contains(&x.s.as_str()) && !["func", "var", "let"].contains(&x.s.as_str())).unwrap_or(false) {
            return Some(i);
        }
        i += 1;
    }
    (i < h.len() && h[i].k == K::Id && KINDS.contains(&h[i].s.as_str())).then_some(i)
}

fn read_type(head: &[T], body: Option<&[T]>) -> Option<Ty> {
    let h = lex::skip_annotations(head);
    let k = kind_at(h)?;
    let body = body?;
    let mods: Vec<String> = h[..k].iter().map(|x| x.s.clone()).collect();
    let kind = match h[k].s.as_str() {
        "protocol" => Kind::Interface,
        "enum" => Kind::Enum,
        _ => Kind::Class,
    };
    let name = lex::join(&h[k + 1..]).split([':', '<', ' ']).next().unwrap_or("").to_string();
    let name = base_name(&name);
    let mut t = Ty::new(kind, &name);
    t.doc = lex::doc_of(head);
    t.vis = vis_of(&mods, "package");
    if h[k].is("struct") || h[k].is("actor") {
        t.stereotype = h[k].s.clone();
    }
    let (g, mut i) = lex::generics(h, k + 2);
    t.generics = g;
    while i < h.len() && h[i].is(".") {
        i += 2;
    }
    if i < h.len() && h[i].is(":") {
        let end = h[i..].iter().position(|x| x.is("where")).map(|p| p + i).unwrap_or(h.len());
        for p in lex::commas(&h[i + 1..end]) {
            let s = lex::join(p);
            // a raw value type of an enum is not a super type
            if kind == Kind::Enum && ["String", "Int", "Character", "Double", "UInt", "Float"].contains(&s.as_str()) {
                continue;
            }
            t.supers.push(s);
        }
    }
    let extension = h[k].is("extension");
    let default_vis = if kind == Kind::Interface { "public" } else { "package" };
    for st in lex::statements(body, true) {
        let mh = lex::skip_annotations(st.head);
        if mh.is_empty() {
            continue;
        }
        if let Some(n) = read_type(mh, st.block) {
            t.nested.push(n);
            continue;
        }
        let doc = lex::doc_of(st.head);
        let mut i = 0;
        let mut mm = vec![];
        while i < mh.len() && mh[i].id() && MODS.contains(&mh[i].s.as_str()) {
            mm.push(mh[i].s.clone());
            i += 1;
        }
        // `private(set)`
        while i < mh.len() && mh[i].is("(") {
            i = lex::close(mh, i) + 1;
        }
        let Some(kw) = mh.get(i) else { continue };
        let vis = vis_of(&mm, default_vis);
        let is_static = mm.iter().any(|m| m == "static" || m == "class");
        if kw.is("case") {
            for p in lex::commas(&mh[i + 1..]) {
                if let Some(x) = p.first().filter(|x| x.id()) {
                    t.literals.push(x.s.clone());
                }
            }
        } else if kw.is("var") || kw.is("let") {
            for p in lex::commas(&mh[i + 1..]) {
                if let Some((n, ty, d)) = lex::colon_param(p, &[]) {
                    let ty = ty.trim_end_matches('!').to_string();
                    // a computed property without a setter is read-only
                    let read_only = kw.is("let") || st.block.map(|b| b.iter().any(|x| x.is("get")) && !b.iter().any(|x| x.is("set"))).unwrap_or(false) || (st.block.is_some() && !st.block.unwrap().iter().any(|x| x.is("get") || x.is("set") || x.is("didSet") || x.is("willSet")));
                    let read_only = read_only && !(kind == Kind::Interface && st.block.map(|b| b.iter().any(|x| x.is("set"))).unwrap_or(false));
                    t.fields.push(Field { name: n, ty, vis: vis.clone(), is_static, read_only, default: d, doc: doc.clone(), ..Default::default() });
                }
            }
        } else if kw.is("func") {
            let Some(n) = mh.get(i + 1) else { continue };
            let mut j = i + 2;
            if j < mh.len() && mh[j].is("<") {
                j = lex::close(mh, j) + 1;
            }
            if j >= mh.len() || !mh[j].is("(") {
                continue;
            }
            let q = lex::close(mh, j);
            let params = lex::commas(&mh[j + 1..q]).into_iter().filter_map(|p| lex::colon_param(p, &[])).map(|(n, ty, d)| Param { name: n, ty: ty.trim_start_matches("inout ").to_string(), default: d }).collect();
            let ret = mh[q + 1..].iter().position(|x| x.is("->")).map(|a| {
                let s = q + 2 + a;
                let e = mh[s..].iter().position(|x| x.is("where")).map(|e| e + s).unwrap_or(mh.len());
                lex::join(&mh[s..e])
            }).unwrap_or_default();
            t.methods.push(Method { name: n.s.clone(), params, ret, vis, is_static, is_abstract: kind == Kind::Interface && st.block.is_none(), doc });
        }
    }
    let _ = extension;
    Some(t)
}
