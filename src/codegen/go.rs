//! Go: a file per type (`library_item.go`) in its package folder, `package <folder>`. A class is a
//! struct embedding its super class, its operations methods on a pointer receiver (static ones
//! functions `TypeOp`), its static attributes package variables; an interface is an interface
//! embedding its super interfaces; an enumeration a named int with `iota` constants. Public names
//! are capitalized, the others not. The reverse reads structs, interfaces, methods, `NewX`
//! constructors and typed constant groups (as enumerations).

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "interface{}".to_string() } else { primitive(t, "string", "int", "bool", "float64") };
    let t = match t.as_str() {
        "double" => "float64".to_string(),
        "boolean" => "bool".into(),
        "String" => "string".into(),
        _ => t,
    };
    if is_many(mult) {
        format!("[]{t}")
    } else if is_optional(mult) {
        format!("*{t}")
    } else {
        t
    }
}

fn zero(t: &str) -> &'static str {
    match t {
        "int" | "int64" | "int32" | "uint" | "float64" | "float32" | "byte" | "rune" => "0",
        "bool" => "false",
        "string" => "\"\"",
        _ => "nil",
    }
}

fn exported(n: &str, vis: &str) -> String {
    let n = ident(n);
    if vis == "public" { uc_first(&n) } else { lc_first(&n) }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

fn params(m: &Method) -> String {
    m.params.iter().map(|p| {
        let (pt, pm) = many(&p.ty);
        format!("{} {}", lc_first(&ident(&p.name)), map_type(pt, pm))
    }).collect::<Vec<_>>().join(", ")
}

fn ret(m: &Method) -> String {
    let (rt, rm) = many(&m.ret);
    if m.ret.is_empty() { String::new() } else { format!(" {}", map_type(rt, rm)) }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    per_type(tree, |t| format!("{}.go", snake(&ident(&t.name))), |p, t| {
        let mut w = W::new(o);
        let pkg = p.path.last().cloned().unwrap_or_else(|| p.name.clone());
        w.line(&format!("package {}", snake(&ident(&pkg)).replace('_', "")));
        w.blank();
        write_type(&mut w, t, o);
        w.out
    })
}

fn doc(w: &mut W, o: &GenOpts, name: &str, text: &str) {
    if o.doc && !text.trim().is_empty() {
        let mut first = true;
        for l in text.trim().lines() {
            w.line(&if first { format!("// {name} {l}") } else { format!("// {l}") });
            first = false;
        }
    }
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    let name = exported(&t.name, &t.vis);
    doc(w, o, &name, &t.doc);
    match t.kind {
        Kind::Enum => {
            w.line(&format!("type {name} int"));
            w.blank();
            w.line("const (");
            w.inc();
            for (i, l) in t.literals.iter().enumerate() {
                w.line(&if i == 0 { format!("{name}{} {name} = iota", ident(l)) } else { format!("{name}{}", ident(l)) });
            }
            w.dec();
            w.line(")");
        }
        Kind::Interface => {
            w.line(&format!("type {name} interface {{"));
            w.inc();
            for s in &t.supers {
                w.line(s);
            }
            for m in &t.methods {
                doc(w, o, &exported(&m.name, "public"), &m.doc);
                w.line(&format!("{}({}){}", exported(&m.name, &m.vis), params(m), ret(m)));
            }
            w.dec();
            w.line("}");
        }
        Kind::Class => {
            w.line(&format!("type {name} struct {{"));
            w.inc();
            for s in &t.supers {
                w.line(s);
            }
            for f in t.fields.iter().filter(|f| !f.is_static) {
                doc(w, o, &exported(&f.name, &f.vis), &f.doc);
                w.line(&format!("{} {}", exported(&f.name, &f.vis), map_type(&f.ty, &f.mult)));
            }
            w.dec();
            w.line("}");
            for f in t.fields.iter().filter(|f| f.is_static) {
                w.blank();
                let ty = map_type(&f.ty, &f.mult);
                let v = if f.default.is_empty() { zero(&ty).to_string() } else { f.default.clone() };
                let kw = if f.read_only { "const" } else { "var" };
                w.line(&format!("{kw} {name}{} {ty} = {v}", uc_first(&ident(&f.name))));
            }
            let recv = lc_first(&name).chars().next().unwrap_or('x');
            for m in &t.methods {
                w.blank();
                let r = ret(m);
                if m.is_static {
                    doc(w, o, &format!("{name}{}", uc_first(&ident(&m.name))), &m.doc);
                    w.line(&format!("func {name}{}({}){r} {{", uc_first(&ident(&m.name)), params(m)));
                } else {
                    doc(w, o, &exported(&m.name, &m.vis), &m.doc);
                    w.line(&format!("func ({recv} *{name}) {}({}){r} {{", exported(&m.name, &m.vis), params(m)));
                }
                w.inc();
                w.line("// TODO implement here");
                if !r.is_empty() {
                    w.line(&format!("return {}", zero(r.trim())));
                }
                w.dec();
                w.line("}");
            }
        }
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o);
    }
}

// ------------------------------------------------------------------ reverse

fn go_vis(n: &str) -> String {
    if n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) { "public".into() } else { "package".into() }
}

/// Go parameters: `a, b int, c string` (a name without a type takes the next type).
fn go_params(t: &[T]) -> Vec<Param> {
    let parts = lex::commas(t);
    let mut out: Vec<Param> = vec![];
    let mut pending: Vec<String> = vec![];
    let named = parts.iter().any(|p| p.len() >= 2 && p[0].id() && !p[0].is("func") && !p[1].is("."));
    for p in parts {
        if !named {
            out.push(Param { name: String::new(), ty: lex::join(p), ..Default::default() });
            continue;
        }
        if p.len() == 1 {
            pending.push(p[0].s.clone());
            continue;
        }
        let ty = lex::join(&p[1..]);
        for n in pending.drain(..) {
            out.push(Param { name: n, ty: ty.clone(), ..Default::default() });
        }
        out.push(Param { name: p[0].s.clone(), ty, ..Default::default() });
    }
    out
}

/// The first result type: `int`, `(int, error)` → `int`; `error` alone is kept.
fn go_result(t: &[T]) -> String {
    if t.is_empty() {
        return String::new();
    }
    if t[0].is("(") {
        let q = lex::close(t, 0);
        let ps = go_params(&t[1..q]);
        return ps.into_iter().map(|p| p.ty).find(|x| x != "error").unwrap_or_default();
    }
    lex::join(t)
}

pub fn parse(src: &str) -> Parsed {
    let t = lex::lex(src, Cfg::default());
    let mut pkg = String::new();
    let mut types: Vec<Ty> = vec![];
    let mut aliases: Vec<String> = vec![];
    let mut methods: Vec<(String, Method)> = vec![];
    let mut consts: Vec<(String, Vec<String>)> = vec![];
    let stmts = lex::statements(&t, true);
    for st in &stmts {
        let h = st.head;
        if h.is_empty() {
            continue;
        }
        let doc = lex::doc_of(h);
        if h[0].is("package") {
            pkg = h.get(1).map(|x| x.s.clone()).unwrap_or_default();
        } else if h[0].is("type") {
            if h.len() == 1 || (h.len() >= 2 && h[1].is("(")) {
                // a group: `type ( A struct { … } \n B int )`
                if let Some(b) = st.block {
                    for s in lex::statements(b, true) {
                        type_spec(s.head, s.block, &doc, &mut types, &mut aliases);
                    }
                } else if h.len() >= 2 {
                    let q = lex::close(h, 1);
                    for s in lex::statements(&h[2..q], true) {
                        type_spec(s.head, s.block, &doc, &mut types, &mut aliases);
                    }
                }
            } else {
                type_spec(&h[1..], st.block, &doc, &mut types, &mut aliases);
            }
        } else if h[0].is("func") {
            let mut i = 1;
            let mut recv = None;
            if i < h.len() && h[i].is("(") {
                let q = lex::close(h, i);
                recv = h[i + 1..q].iter().rev().find(|x| x.id()).map(|x| x.s.clone());
                i = q + 1;
            }
            let Some(name) = h.get(i).filter(|x| x.id()).map(|x| x.s.clone()) else { continue };
            let mut j = i + 1;
            if j < h.len() && h[j].is("[") {
                j = lex::close(h, j) + 1;
            }
            if j >= h.len() || !h[j].is("(") {
                continue;
            }
            let q = lex::close(h, j);
            let m = Method { name: name.clone(), params: go_params(&h[j + 1..q]), ret: go_result(&h[q + 1..]), vis: go_vis(&name), doc, ..Default::default() };
            match recv {
                Some(r) => methods.push((r, m)),
                // `NewBook(…) *Book` is a static operation of Book
                None if name.starts_with("New") => {
                    let target = name[3..].to_string();
                    methods.push((target, Method { is_static: true, ..m }));
                }
                None => {}
            }
        } else if h[0].is("const") {
            let body: Vec<&[T]> = if let Some(b) = st.block.filter(|_| h.len() == 1) {
                lex::statements(b, true).into_iter().map(|s| s.head).collect()
            } else if h.len() >= 2 && h[1].is("(") {
                let q = lex::close(h, 1);
                lex::statements(&h[2..q], true).into_iter().map(|s| s.head).collect()
            } else {
                vec![&h[1..]]
            };
            let mut group: Option<(String, Vec<String>)> = None;
            for spec in body {
                let Some(n) = spec.first().filter(|x| x.id()) else { continue };
                let typed = spec.get(1).filter(|x| x.id() && !x.is("=")).map(|x| x.s.clone());
                match (&mut group, typed) {
                    (None, Some(ty)) => group = Some((ty, vec![n.s.clone()])),
                    (Some(g), Some(ty)) if ty == g.0 => g.1.push(n.s.clone()),
                    (Some(g), None) => g.1.push(n.s.clone()),
                    _ => {}
                }
            }
            if let Some(g) = group {
                consts.push(g);
            }
        }
    }
    // a named basic type with typed constants is an enumeration
    for (ty, names) in consts {
        if !aliases.contains(&ty) {
            continue;
        }
        let mut e = Ty::new(Kind::Enum, &ty);
        e.vis = go_vis(&ty);
        e.literals = names.into_iter().map(|n| n.strip_prefix(&ty).filter(|r| !r.is_empty()).map(|r| r.trim_start_matches('_').to_string()).unwrap_or(n)).collect();
        types.push(e);
    }
    for (r, m) in methods {
        if let Some(t) = types.iter_mut().find(|t| t.name == r && t.kind == Kind::Class) {
            t.methods.push(m);
        }
    }
    let ns = if pkg.is_empty() || pkg == "main" { vec![] } else { vec![pkg] };
    Parsed { units: vec![Unit { ns, types }], errors: !lex::balanced(&t) }
}

fn type_spec(h: &[T], block: Option<&[T]>, doc: &str, types: &mut Vec<Ty>, aliases: &mut Vec<String>) {
    let Some(name) = h.first().filter(|x| x.id()).map(|x| x.s.clone()) else { return };
    let mut i = 1;
    let mut generics = vec![];
    if i < h.len() && h[i].is("[") {
        let q = lex::close(h, i);
        generics = lex::commas(&h[i + 1..q]).iter().filter_map(|p| p.first()).map(|x| x.s.clone()).collect();
        i = q + 1;
    }
    let kind = h.get(i).map(|x| x.s.as_str()).unwrap_or("");
    let doc = if lex::doc_of(h).is_empty() { doc.to_string() } else { lex::doc_of(h) };
    match (kind, block) {
        ("struct", Some(b)) => {
            let mut t = Ty::new(Kind::Class, &name);
            t.vis = go_vis(&name);
            t.doc = doc;
            t.generics = generics;
            for line in lex::statements(b, true) {
                let mut f: Vec<T> = line.head.iter().filter(|x| x.k != K::Str).cloned().collect();
                if f.is_empty() {
                    continue;
                }
                // an embedded type is the super class
                if f.len() == 1 || (f.len() == 2 && f[0].is("*")) || (f.len() == 3 && f[1].is(".")) {
                    t.supers.push(base_name(&lex::join(&f)));
                    continue;
                }
                let c = f.iter().position(|x| x.is(",")).map(|_| ()).is_some();
                let names: Vec<String>;
                let ty: String;
                if c {
                    let last = f.iter().rposition(|x| x.is(",")).unwrap();
                    names = f[..=last + 1].iter().filter(|x| x.id()).map(|x| x.s.clone()).collect();
                    ty = lex::join(&f[last + 2..]);
                } else {
                    names = vec![f[0].s.clone()];
                    ty = lex::join(&f.split_off(1));
                }
                for n in names {
                    t.fields.push(Field { vis: go_vis(&n), name: n, ty: ty.clone(), ..Default::default() });
                }
            }
            types.push(t);
        }
        ("interface", Some(b)) => {
            let mut t = Ty::new(Kind::Interface, &name);
            t.vis = go_vis(&name);
            t.doc = doc;
            for line in lex::statements(b, true) {
                let l = line.head;
                match l.iter().position(|x| x.is("(")) {
                    Some(p) if p == 1 => {
                        let q = lex::close(l, p);
                        let n = l[0].s.clone();
                        t.methods.push(Method { vis: go_vis(&n), name: n, params: go_params(&l[p + 1..q]), ret: go_result(&l[q + 1..]), doc: lex::doc_of(l), ..Default::default() });
                    }
                    _ if !l.is_empty() && !l.iter().any(|x| x.is("|") || x.is("~")) => t.supers.push(base_name(&lex::join(l))),
                    _ => {}
                }
            }
            types.push(t);
        }
        _ if !kind.is_empty() && h.len() == i + 1 => aliases.push(name),
        _ => {}
    }
}
