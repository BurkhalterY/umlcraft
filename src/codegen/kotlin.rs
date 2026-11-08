//! Kotlin: a `.kt` file per type in its package folder, `package a.b`; classes `open` when another
//! class extends them, static members in a `companion object`, `TODO()` bodies, interfaces and
//! `enum class`es. The reverse reads packages, classes (primary constructor properties included),
//! objects, interfaces, enum classes, properties, functions and companion objects.

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "Any".to_string() } else { primitive(t, "String", "Int", "Boolean", "Double") };
    let t = match t.as_str() {
        "int" => "Int".to_string(),
        "long" => "Long".into(),
        "double" => "Double".into(),
        "float" => "Float".into(),
        "boolean" | "bool" => "Boolean".into(),
        "string" => "String".into(),
        _ => t,
    };
    if is_many(mult) {
        format!("MutableList<{t}>")
    } else if is_optional(mult) {
        format!("{t}?")
    } else {
        t
    }
}

fn initial(t: &str) -> Option<&'static str> {
    match t {
        "Int" | "Long" | "Short" | "Byte" => Some("0"),
        "Double" => Some("0.0"),
        "Float" => Some("0.0f"),
        "Boolean" => Some("false"),
        "String" => Some("\"\""),
        _ if t.ends_with('?') => Some("null"),
        _ if t.starts_with("MutableList<") => Some("mutableListOf()"),
        _ => None,
    }
}

fn vis(v: &str) -> &'static str {
    match v {
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
    let extended: Vec<String> = tree.all().iter().flat_map(|p| p.types.iter()).flat_map(|t| t.supers.clone()).collect();
    per_type(tree, |t| format!("{}.kt", ident(&t.name)), |p, t| {
        let mut w = W::new(o);
        if !p.path.is_empty() {
            w.line(&format!("package {}", p.path.iter().map(|s| ident(s)).collect::<Vec<_>>().join(".")));
            w.blank();
        }
        write_type(&mut w, t, o, &extended);
        w.out
    })
}

fn field(w: &mut W, f: &Field, o: &GenOpts, interface: bool) {
    w.doc(o, &f.doc, DocStyle::Block);
    let ty = map_type(&f.ty, &f.mult);
    let kw = if f.read_only { "val" } else { "var" };
    if interface {
        w.line(&format!("{kw} {}: {ty}", ident(&f.name)));
        return;
    }
    let init = if f.default.is_empty() { initial(&ty).map(String::from) } else { Some(f.default.clone()) };
    match init {
        Some(v) => w.line(&format!("{}{kw} {}: {ty} = {v}", vis(&f.vis), ident(&f.name))),
        None => w.line(&format!("{}lateinit var {}: {ty}", vis(&f.vis), ident(&f.name))),
    }
}

fn method(w: &mut W, m: &Method, o: &GenOpts, interface: bool) {
    w.doc(o, &m.doc, DocStyle::Block);
    let ps: Vec<String> = m.params.iter().map(|p| {
        let (pt, pm) = many(&p.ty);
        let d = if p.default.is_empty() { String::new() } else { format!(" = {}", p.default) };
        format!("{}: {}{d}", ident(&p.name), map_type(pt, pm))
    }).collect();
    let (rt, rm) = many(&m.ret);
    let ret = if m.ret.is_empty() { String::new() } else { format!(": {}", map_type(rt, rm)) };
    let pre = if interface { String::new() } else { format!("{}{}", vis(&m.vis), if m.is_abstract { "abstract " } else { "" }) };
    let s = format!("{pre}fun {}({}){ret}", ident(&m.name), ps.join(", "));
    if interface || m.is_abstract {
        w.line(&s);
    } else {
        w.line(&format!("{s} {{"));
        w.inc();
        w.line("TODO(\"Not yet implemented\")");
        w.dec();
        w.line("}");
    }
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts, extended: &[String]) {
    w.doc(o, &t.doc, DocStyle::Block);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    let name = ident(&t.name);
    let interface = t.kind == Kind::Interface;
    match t.kind {
        Kind::Enum => {
            w.line(&format!("{}enum class {name} {{", vis(&t.vis)));
            w.inc();
            let lits: Vec<String> = t.literals.iter().map(|l| ident(l)).collect();
            w.line(&lits.join(",\n"));
        }
        Kind::Interface => {
            let sup = if t.supers.is_empty() { String::new() } else { format!(" : {}", t.supers.join(", ")) };
            w.line(&format!("{}interface {name}{gen}{sup} {{", vis(&t.vis)));
            w.inc();
        }
        Kind::Class => {
            let abs = t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
            let open = if abs { "abstract " } else if extended.contains(&t.name) { "open " } else { "" };
            let mut bases: Vec<String> = t.supers.iter().map(|s| format!("{s}()")).collect();
            bases.extend(t.reals.iter().cloned());
            let sup = if bases.is_empty() { String::new() } else { format!(" : {}", bases.join(", ")) };
            w.line(&format!("{}{open}class {name}{gen}{sup} {{", vis(&t.vis)));
            w.inc();
        }
    }
    let mut first = true;
    for f in t.fields.iter().filter(|f| !f.is_static) {
        first = false;
        field(w, f, o, interface);
    }
    for m in t.methods.iter().filter(|m| !m.is_static) {
        if !first {
            w.blank();
        }
        first = false;
        method(w, m, o, interface);
    }
    let sf: Vec<&Field> = t.fields.iter().filter(|f| f.is_static).collect();
    let sm: Vec<&Method> = t.methods.iter().filter(|m| m.is_static).collect();
    if !sf.is_empty() || !sm.is_empty() {
        w.blank();
        w.line("companion object {");
        w.inc();
        for f in sf {
            let f = Field { read_only: f.read_only, ..f.clone() };
            field(w, &f, o, false);
        }
        for m in sm {
            method(w, m, o, false);
        }
        w.dec();
        w.line("}");
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o, extended);
    }
    w.dec();
    w.line("}");
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 26] = [
    "public", "private", "protected", "internal", "open", "abstract", "final", "sealed", "data", "inner", "annotation", "value", "override", "lateinit", "const", "suspend", "inline", "operator", "infix", "tailrec", "external", "expect", "actual", "vararg", "crossinline", "noinline",
];

pub fn parse(src: &str) -> Parsed {
    let t = lex::lex(src, Cfg::default());
    let mut unit = Unit::default();
    for st in lex::statements(&t, true) {
        let h = lex::skip_annotations(st.head);
        if h.first().map(|x| x.is("package")).unwrap_or(false) {
            unit.ns = lex::join(&h[1..]).split('.').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        } else if h.first().map(|x| x.is("import")).unwrap_or(false) {
            // `import a.b.C`, `import a.b.*`, `import a.b.C as D`
            let as_at = h.iter().position(|x| x.is("as"));
            let mut i = Import::of(&lex::join(&h[1..as_at.unwrap_or(h.len())]), &[]);
            if let Some(a) = as_at.and_then(|a| h.get(a + 1)) {
                i.alias = a.s.clone();
            }
            unit.imports.push(i);
        } else if let Some(ty) = read_type(h, st.block) {
            unit.types.push(ty);
        }
    }
    Parsed { units: vec![unit], errors: !lex::balanced(&t) }
}

fn read_type(h: &[T], body: Option<&[T]>) -> Option<Ty> {
    let k = h.iter().position(|x| x.k == K::Id && ["class", "interface", "object"].contains(&x.s.as_str()))?;
    if !h[..k].iter().all(|x| x.id() && (MODS.contains(&x.s.as_str()) || x.is("enum") || x.is("fun") || x.is("companion"))) {
        return None;
    }
    if h[..k].iter().any(|x| x.is("companion")) {
        return None;
    }
    let mods: Vec<String> = h[..k].iter().map(|x| x.s.clone()).collect();
    let kind = if mods.iter().any(|m| m == "enum") {
        Kind::Enum
    } else if h[k].is("interface") {
        Kind::Interface
    } else {
        Kind::Class
    };
    let name = h.get(k + 1).filter(|x| x.id()).map(|x| x.s.clone())?;
    let mut t = Ty::new(kind, &name);
    t.doc = lex::doc_of(h);
    t.vis = vis_of(&mods, "public");
    t.is_abstract = mods.iter().any(|m| m == "abstract" || m == "sealed");
    if h[k].is("object") {
        t.stereotype = "object".into();
    } else if mods.iter().any(|m| m == "data") {
        t.stereotype = "data".into();
    }
    let (g, mut i) = lex::generic_params(h, k + 2);
    t.generics = g.iter().map(|p| p.0.clone()).collect();
    t.gen_bounds = g.into_iter().map(|p| (p.1, p.2)).collect();
    // the primary constructor: `[private] [constructor] (val a: Int, …)`
    while i < h.len() && h[i].id() && (MODS.contains(&h[i].s.as_str()) || h[i].is("constructor")) {
        i += 1;
    }
    while i < h.len() && h[i].is("@") {
        i += 2;
    }
    if i < h.len() && h[i].is("(") {
        let q = lex::close(h, i);
        for p in lex::commas(&h[i + 1..q]) {
            let p = lex::skip_annotations(p);
            let pm: Vec<String> = p.iter().take_while(|x| x.id() && (MODS.contains(&x.s.as_str()) || x.is("val") || x.is("var"))).map(|x| x.s.clone()).collect();
            if !pm.iter().any(|m| m == "val" || m == "var") {
                continue;
            }
            if let Some((n, ty, d)) = lex::colon_param(p, &[]) {
                t.fields.push(Field { name: n, ty, vis: vis_of(&pm, "public"), read_only: pm.iter().any(|m| m == "val"), default: d, ..Default::default() });
            }
        }
        i = q + 1;
    }
    if i < h.len() && h[i].is(":") {
        let end = h[i..].iter().position(|x| x.is("where")).map(|p| p + i).unwrap_or(h.len());
        for p in lex::commas(&h[i + 1..end]) {
            // `Base(args)` is the super class, the others interfaces; `by delegate` dropped
            let p = match p.iter().position(|x| x.is("by")) {
                Some(b) => &p[..b],
                None => p,
            };
            let p = match p.iter().position(|x| x.is("(")) {
                Some(c) => &p[..c],
                None => p,
            };
            t.supers.push(lex::join(p));
        }
    }
    let Some(body) = body else { return Some(t) };
    let mut body = body;
    if kind == Kind::Enum {
        let end = lex::find_top(body, ";", false).unwrap_or(body.len());
        for p in lex::commas(&body[..end]) {
            if let Some(x) = lex::skip_annotations(p).first().filter(|x| x.id()) {
                t.literals.push(x.s.clone());
            }
        }
        body = &body[(end + 1).min(body.len())..];
    }
    members(&mut t, body, false, kind == Kind::Interface);
    Some(t)
}

fn members(t: &mut Ty, body: &[T], is_static: bool, interface: bool) {
    for st in lex::statements(body, true) {
        let h = lex::skip_annotations(st.head);
        if h.is_empty() {
            continue;
        }
        let doc = lex::doc_of(st.head);
        if h.iter().take_while(|x| x.id()).any(|x| x.is("companion")) {
            if let Some(b) = st.block {
                members(t, b, true, interface);
            }
            continue;
        }
        if let Some(n) = read_type(h, st.block) {
            t.nested.push(n);
            continue;
        }
        let mut i = 0;
        let mut mods = vec![];
        while i < h.len() && h[i].id() && MODS.contains(&h[i].s.as_str()) {
            mods.push(h[i].s.clone());
            i += 1;
        }
        let vis = vis_of(&mods, "public");
        let Some(kw) = h.get(i) else { continue };
        if kw.is("val") || kw.is("var") {
            let rest = &h[i + 1..];
            // `by lazy { … }` delegates are dropped from the value
            let rest = match rest.iter().position(|x| x.is("by")) {
                Some(b) => &rest[..b],
                None => rest,
            };
            if let Some((n, ty, d)) = lex::colon_param(rest, &[]) {
                if ["get", "set"].contains(&n.as_str()) {
                    continue;
                }
                t.fields.push(Field { name: n, ty, vis, is_static, read_only: kw.is("val"), default: d, doc, ..Default::default() });
            }
        } else if kw.is("fun") {
            let mut j = i + 1;
            if j < h.len() && h[j].is("<") {
                j = lex::close(h, j) + 1;
            }
            // an extension's receiver `Type.name`
            let Some(p) = h[j..].iter().position(|x| x.is("(")).map(|p| p + j) else { continue };
            let Some(name) = h[..p].iter().rev().find(|x| x.id()).map(|x| x.s.clone()) else { continue };
            let q = lex::close(h, p);
            let params = lex::commas(&h[p + 1..q]).into_iter().filter_map(|x| lex::colon_param(x, &MODS)).map(|(n, ty, d)| Param { name: n, ty, default: d }).collect();
            let ret = if q + 1 < h.len() && h[q + 1].is(":") {
                let end = h[q + 2..].iter().position(|x| x.is("=") || x.is("where")).map(|e| e + q + 2).unwrap_or(h.len());
                lex::join(&h[q + 2..end])
            } else {
                String::new()
            };
            let is_abstract = mods.iter().any(|m| m == "abstract") || (interface && st.block.is_none() && !h.iter().any(|x| x.is("=")));
            t.methods.push(Method { name, params, ret, vis, is_static, is_abstract, doc , ..Default::default() });
        }
    }
}
