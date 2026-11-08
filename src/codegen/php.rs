//! PHP: a `.php` file per type in its package folder, under `namespace A\B;` (classes, interfaces,
//! enumerations as PHP 8.1 enums, typed properties, constants for static read-only attributes),
//! and the reverse of `.php` files (namespaces, classes, interfaces, traits, enums with their
//! cases, properties, constants and functions). After staruml-php's generator.

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    if is_many(mult) {
        return "array".into();
    }
    let t = primitive(t, "string", "int", "bool", "float");
    match t.as_str() {
        "integer" | "long" | "short" => "int".into(),
        "double" | "real" => "float".into(),
        "boolean" => "bool".into(),
        "String" => "string".into(),
        _ => t,
    }
}

fn default_return(t: &str) -> &'static str {
    match t {
        "int" => "0",
        "float" => "0.0",
        "bool" => "false",
        "string" => "''",
        "array" => "[]",
        _ => "null",
    }
}

fn vis(v: &str) -> &'static str {
    match v {
        "protected" => "protected",
        "private" => "private",
        _ => "public",
    }
}

fn typed(t: &str, mult: &str) -> String {
    let m = map_type(t, mult);
    if m.is_empty() { String::new() } else { format!("{m} ") }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    per_type(tree, |t| format!("{}.php", ident(&t.name)), |p, t| {
        let mut w = W::new(o);
        w.line("<?php");
        w.blank();
        if !p.path.is_empty() {
            w.line(&format!("namespace {};", p.path.iter().map(|s| ident(s)).collect::<Vec<_>>().join("\\")));
            w.blank();
        }
        write_type(&mut w, t, o);
        for n in &t.nested {
            w.blank();
            write_type(&mut w, n, o);
        }
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    w.doc(o, &t.doc, DocStyle::Block);
    let name = ident(&t.name);
    match t.kind {
        Kind::Enum => {
            w.line(&format!("enum {name}"));
            w.line("{");
            w.inc();
            for l in &t.literals {
                w.line(&format!("case {};", ident(l)));
            }
        }
        Kind::Interface => {
            let ext = if t.supers.is_empty() { String::new() } else { format!(" extends {}", t.supers.join(", ")) };
            w.line(&format!("interface {name}{ext}"));
            w.line("{");
            w.inc();
        }
        Kind::Class => {
            let abs = t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
            let mut head = format!("{}class {name}", if abs { "abstract " } else { "" });
            if let Some(s) = t.supers.first() {
                head.push_str(&format!(" extends {s}"));
            }
            if !t.reals.is_empty() {
                head.push_str(&format!(" implements {}", t.reals.join(", ")));
            }
            w.line(&head);
            w.line("{");
            w.inc();
        }
    }
    for f in &t.fields {
        w.blank();
        w.doc(o, &f.doc, DocStyle::Block);
        if f.is_static && f.read_only {
            let v = if f.default.is_empty() { "null".to_string() } else { f.default.clone() };
            w.line(&format!("{} const {} = {v};", vis(&f.vis), ident(&f.name)));
            continue;
        }
        if t.kind != Kind::Class {
            continue;
        }
        let mut s = format!("{} ", vis(&f.vis));
        if f.is_static {
            s.push_str("static ");
        } else if f.read_only {
            s.push_str("readonly ");
        }
        s.push_str(&format!("{}${}", typed(&f.ty, &f.mult), ident(&f.name)));
        if !f.default.is_empty() {
            s.push_str(&format!(" = {}", f.default));
        }
        w.line(&format!("{s};"));
    }
    for m in &t.methods {
        w.blank();
        let mut doc = m.doc.trim().to_string();
        for p in &m.params {
            let (pt, pm) = many(&p.ty);
            doc.push_str(&format!("\n@param {} ${}", map_type(pt, pm), ident(&p.name)).replace("  ", " "));
        }
        if !m.ret.is_empty() {
            let (rt, rm) = many(&m.ret);
            doc.push_str(&format!("\n@return {}", map_type(rt, rm)));
        }
        w.doc(o, doc.trim(), DocStyle::Block);
        let ps: Vec<String> = m.params.iter().map(|p| {
            let (pt, pm) = many(&p.ty);
            let d = if p.default.is_empty() { String::new() } else { format!(" = {}", p.default) };
            format!("{}${}{d}", typed(pt, pm), ident(&p.name))
        }).collect();
        let (rt, rm) = many(&m.ret);
        let r = if m.ret.is_empty() { "void".to_string() } else { map_type(rt, rm) };
        let interface = t.kind == Kind::Interface;
        let mut s = format!("{} ", vis(&m.vis));
        if m.is_abstract && !interface {
            s.push_str("abstract ");
        }
        if m.is_static {
            s.push_str("static ");
        }
        s.push_str(&format!("function {}({}): {r}", ident(&m.name), ps.join(", ")));
        if interface || m.is_abstract {
            w.line(&format!("{s};"));
            continue;
        }
        w.line(&s);
        w.line("{");
        w.inc();
        w.line("// TODO implement here");
        if !m.ret.is_empty() {
            w.line(&format!("return {};", default_return(&r)));
        }
        w.dec();
        w.line("}");
    }
    w.dec();
    w.line("}");
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 8] = ["public", "protected", "private", "static", "abstract", "final", "readonly", "var"];

pub fn parse(src: &str) -> Parsed {
    let t = lex::lex(src, Cfg { hash_comment: true, sq_strings: true, dollar_ids: true, ..Default::default() });
    let mut units = vec![];
    space(&t, vec![], &mut units);
    Parsed { units, errors: !lex::balanced(&t) }
}

fn ns_path(h: &[T]) -> Vec<String> {
    lex::join(&h[1..]).split('\\').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

fn space(t: &[T], ns: Vec<String>, units: &mut Vec<Unit>) {
    let mut unit = Unit { ns: ns.clone(), types: vec![], ..Default::default() };
    for st in lex::statements(t, false) {
        let mut h = lex::skip_annotations(st.head);
        // `<?php` and `?>` around the code
        while h.len() >= 2 && h[0].is("<") && h[1].is("?") {
            h = &h[if h.get(2).map(|x| x.is("php")).unwrap_or(false) { 3 } else { 2 }..];
        }
        if h.first().map(|x| x.is("use")).unwrap_or(false) && !h.get(1).map(|x| x.is("function") || x.is("const")).unwrap_or(false) {
            // `use A\B\C;`, `use A\B\C as D;` and `use A\B\{C, D as E};`
            let prefix = lex::join(&h[1..]);
            let items: Vec<String> = match st.block {
                Some(b) => lex::commas(b).into_iter().map(|p| format!("{prefix}{}", lex::join(p))).collect(),
                None => prefix.split(',').map(|s| s.to_string()).collect(),
            };
            for it in items {
                let (path, alias) = match it.split_once(" as ") {
                    Some((p, a)) => (p.to_string(), Some(a.trim().to_string())),
                    None => (it.clone(), None),
                };
                let mut i = Import::of(&path, &["\\"]);
                if let Some(a) = alias {
                    i.alias = a;
                }
                unit.imports.push(i);
            }
            continue;
        }
        if h.first().map(|x| x.is("namespace")).unwrap_or(false) {
            match st.block {
                Some(b) => space(b, ns_path(h), units),
                None => {
                    if !unit.types.is_empty() {
                        units.push(std::mem::take(&mut unit));
                    }
                    unit.ns = ns_path(h);
                }
            }
            continue;
        }
        let Some(b) = st.block else { continue };
        if let Some(ty) = read_type(h, b) {
            unit.types.push(ty);
        }
    }
    units.push(unit);
}

fn read_type(h: &[T], body: &[T]) -> Option<Ty> {
    let k = h.iter().position(|x| x.k == K::Id && ["class", "interface", "trait", "enum"].contains(&x.s.as_str()))?;
    if !h[..k].iter().all(|x| x.id() && MODS.contains(&x.s.as_str())) {
        return None;
    }
    let kind = match h[k].s.as_str() {
        "interface" => Kind::Interface,
        "enum" => Kind::Enum,
        _ => Kind::Class,
    };
    let mut t = Ty::new(kind, &h.get(k + 1).map(|x| x.s.clone()).unwrap_or_default());
    t.doc = lex::doc_of(h);
    t.is_abstract = h[..k].iter().any(|x| x.is("abstract"));
    if h[k].is("trait") {
        t.stereotype = "trait".into();
    }
    let rest = &h[(k + 2).min(h.len())..];
    let rest = if kind == Kind::Enum && rest.first().map(|x| x.is(":")).unwrap_or(false) { &rest[2.min(rest.len())..] } else { rest };
    let (s, r) = super::java::super_lists(rest, &["extends"], &["implements"], &[]);
    t.supers = s.iter().map(|x| x.trim_start_matches('\\').to_string()).collect();
    t.reals = r.iter().map(|x| x.trim_start_matches('\\').to_string()).collect();
    for st in lex::statements(body, false) {
        let h = lex::skip_annotations(st.head);
        if h.is_empty() {
            continue;
        }
        let doc = lex::doc_of(st.head);
        if h[0].is("case") {
            if let Some(n) = h.get(1) {
                t.literals.push(n.s.clone());
            }
            continue;
        }
        if h[0].is("use") {
            continue;
        }
        let mut i = 0;
        let mut mods = vec![];
        while i < h.len() && h[i].id() && MODS.contains(&h[i].s.as_str()) {
            mods.push(h[i].s.clone());
            i += 1;
        }
        let vis = vis_of(&mods, "public");
        let is_static = mods.iter().any(|m| m == "static");
        if i < h.len() && h[i].is("const") {
            // `const [type] NAME = value`
            let rest = &h[i + 1..];
            for part in lex::commas(rest) {
                let e = lex::find_top(part, "=", false).unwrap_or(part.len());
                let Some(n) = part[..e].iter().rev().find(|x| x.id()) else { continue };
                let ty = if e >= 2 { lex::join(&part[..e - 1]) } else { String::new() };
                t.fields.push(Field { name: n.s.clone(), ty, vis: vis.clone(), is_static: true, read_only: true, default: if e < part.len() { lex::join(&part[e + 1..]) } else { String::new() }, doc: doc.clone(), ..Default::default() });
            }
            continue;
        }
        if let Some(f) = h[i..].iter().position(|x| x.is("function")).map(|f| f + i) {
            let mut j = f + 1;
            if j < h.len() && h[j].is("&") {
                j += 1;
            }
            let Some(n) = h.get(j) else { continue };
            let Some(p) = h[j..].iter().position(|x| x.is("(")).map(|p| p + j) else { continue };
            let q = lex::close(h, p);
            let mut params = vec![];
            for part in lex::commas(&h[p + 1..q]) {
                let part = lex::skip_annotations(part);
                let Some(v) = part.iter().position(|x| x.s.starts_with('$')) else { continue };
                let promoted = part[..v].iter().any(|x| x.id() && ["public", "protected", "private"].contains(&x.s.as_str()));
                let ty_toks: Vec<T> = part[..v].iter().filter(|x| !(x.id() && MODS.contains(&x.s.as_str())) && !x.is("&") && !x.is("...")).cloned().collect();
                let def = lex::find_top(part, "=", false).map(|e| lex::join(&part[e + 1..])).unwrap_or_default();
                let name = part[v].s.trim_start_matches('$').to_string();
                let ty = lex::join(&ty_toks);
                if promoted && n.is("__construct") {
                    let pm: Vec<String> = part[..v].iter().map(|x| x.s.clone()).collect();
                    t.fields.push(Field { name: name.clone(), ty: ty.clone(), vis: vis_of(&pm, "public"), read_only: pm.iter().any(|m| m == "readonly"), ..Default::default() });
                }
                params.push(Param { name, ty, default: def });
            }
            if n.is("__construct") {
                continue;
            }
            let ret = if q + 1 < h.len() && h[q + 1].is(":") { lex::join(&h[q + 2..]) } else { String::new() };
            t.methods.push(Method { name: n.s.clone(), params, ret, vis, is_static, is_abstract: mods.iter().any(|m| m == "abstract") || (kind == Kind::Interface && st.block.is_none()), doc , ..Default::default() });
            continue;
        }
        // properties: `[type] $a [= v], $b`
        let rest = &h[i..];
        let mut ty = String::new();
        for (k, part) in lex::commas(rest).into_iter().enumerate() {
            let Some(v) = part.iter().position(|x| x.s.starts_with('$')) else { continue };
            if k == 0 {
                ty = lex::join(&part[..v]);
            }
            let def = lex::find_top(part, "=", false).map(|e| lex::join(&part[e + 1..])).unwrap_or_default();
            t.fields.push(Field { name: part[v].s.trim_start_matches('$').to_string(), ty: ty.clone(), vis: vis.clone(), is_static, read_only: mods.iter().any(|m| m == "readonly"), default: def, doc: doc.clone(), ..Default::default() });
        }
    }
    Some(t)
}
