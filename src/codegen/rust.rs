//! Rust: a module file per type (`library_item.rs`) in its package folder, with a `mod.rs` per
//! folder declaring them. A class is a `pub struct` whose super class is a `base` field, its
//! operations an `impl` block (static ones without `&self`, static attributes associated
//! constants), each realized interface an `impl Trait for Type`; an interface is a trait, an
//! enumeration an enum. The reverse reads modules, structs, enums, traits and `impl` blocks.

use super::lex::{self, Cfg, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "()".to_string() } else { primitive(t, "String", "i32", "bool", "f64") };
    let t = match t.as_str() {
        "int" => "i32".to_string(),
        "long" => "i64".into(),
        "double" => "f64".into(),
        "float" => "f32".into(),
        "boolean" => "bool".into(),
        "string" => "String".into(),
        _ => t,
    };
    if is_many(mult) {
        format!("Vec<{t}>")
    } else if is_optional(mult) {
        format!("Option<{t}>")
    } else {
        t
    }
}

fn default_value(t: &str) -> String {
    match t {
        "i32" | "i64" | "u32" | "u64" | "usize" | "isize" | "u8" | "i8" | "u16" | "i16" => "0".into(),
        "f64" | "f32" => "0.0".into(),
        "bool" => "false".into(),
        "&str" | "&'static str" => "\"\"".into(),
        _ => "Default::default()".into(),
    }
}

fn vis(v: &str) -> &'static str {
    match v {
        "public" => "pub ",
        "protected" | "package" => "pub(crate) ",
        _ => "",
    }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

fn signature(m: &Method) -> String {
    let mut ps = vec![];
    if !m.is_static {
        ps.push("&self".to_string());
    }
    for p in &m.params {
        let (pt, pm) = many(&p.ty);
        ps.push(format!("{}: {}", snake(&ident(&p.name)), map_type(pt, pm)));
    }
    let (rt, rm) = many(&m.ret);
    let ret = if m.ret.is_empty() { String::new() } else { format!(" -> {}", map_type(rt, rm)) };
    format!("fn {}({}){ret}", snake(&ident(&m.name)), ps.join(", "))
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let paths = tree.type_paths();
    let traits: Vec<Ty> = tree.all().iter().flat_map(|p| p.types.iter()).filter(|t| t.kind == Kind::Interface).cloned().collect();
    let mut out = per_type(tree, |t| format!("{}.rs", snake(&ident(&t.name))), |p, t| {
        let mut w = W::new(o);
        let mut uses = vec![];
        for n in t.supers.iter().chain(t.reals.iter()).chain(t.fields.iter().map(|f| &f.ty)) {
            if let Some((tn, tp)) = paths.iter().find(|x| &x.0 == n && x.0 != t.name) {
                let path: Vec<String> = ["crate".to_string()].into_iter().chain(tp.iter().map(|s| snake(&ident(s)))).chain([snake(&ident(tn)), ident(tn)]).collect();
                let u = format!("use {};", path.join("::"));
                if !uses.contains(&u) {
                    uses.push(u);
                }
            }
        }
        let _ = p;
        for u in &uses {
            w.line(u);
        }
        if !uses.is_empty() {
            w.blank();
        }
        write_type(&mut w, t, o, &traits);
        w.out
    });
    // a mod.rs per folder declares its modules
    for p in tree.all() {
        let dir: PathBuf = p.path.iter().map(|s| snake(&ident(s))).collect();
        let mut w = W::new(o);
        for s in &p.subs {
            w.line(&format!("pub mod {};", snake(&ident(s.path.last().map(|x| x.as_str()).unwrap_or("")))));
        }
        for t in &p.types {
            w.line(&format!("pub mod {};", snake(&ident(&t.name))));
        }
        out.push((dir.join("mod.rs"), w.out));
    }
    // the package folders in snake case, as their modules
    for f in out.iter_mut() {
        let parts: Vec<String> = f.0.iter().map(|s| s.to_string_lossy().to_string()).collect();
        let n = parts.len();
        f.0 = parts.iter().enumerate().map(|(i, s)| if i + 1 < n { snake(&ident(s)) } else { s.clone() }).collect();
    }
    out
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts, traits: &[Ty]) {
    w.doc(o, &t.doc, DocStyle::Slashes);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    let name = ident(&t.name);
    match t.kind {
        Kind::Enum => {
            w.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
            w.line(&format!("{}enum {name} {{", vis(&t.vis)));
            w.inc();
            for l in &t.literals {
                w.line(&format!("{},", ident(l)));
            }
            w.dec();
            w.line("}");
        }
        Kind::Interface => {
            let sup = if t.supers.is_empty() { String::new() } else { format!(": {}", t.supers.join(" + ")) };
            w.line(&format!("{}trait {name}{gen}{sup} {{", vis(&t.vis)));
            w.inc();
            for m in &t.methods {
                w.doc(o, &m.doc, DocStyle::Slashes);
                w.line(&format!("{};", signature(m)));
            }
            w.dec();
            w.line("}");
        }
        Kind::Class => {
            w.line(&format!("{}struct {name}{gen} {{", vis(&t.vis)));
            w.inc();
            if let Some(s) = t.supers.first() {
                w.line(&format!("pub base: {s},"));
            }
            for f in t.fields.iter().filter(|f| !f.is_static) {
                w.doc(o, &f.doc, DocStyle::Slashes);
                w.line(&format!("{}{}: {},", vis(&f.vis), snake(&ident(&f.name)), map_type(&f.ty, &f.mult)));
            }
            w.dec();
            w.line("}");
            let statics: Vec<&Field> = t.fields.iter().filter(|f| f.is_static).collect();
            if !t.methods.is_empty() || !statics.is_empty() {
                w.blank();
                w.line(&format!("impl{gen} {name}{gen} {{"));
                w.inc();
                for f in statics {
                    let ty = map_type(&f.ty, &f.mult);
                    let v = if f.default.is_empty() { default_value(&ty) } else { f.default.clone() };
                    w.line(&format!("{}const {}: {ty} = {v};", vis(&f.vis), snake(&ident(&f.name)).to_uppercase()));
                }
                for m in &t.methods {
                    w.blank();
                    w.doc(o, &m.doc, DocStyle::Slashes);
                    w.line(&format!("{}{} {{", vis(&m.vis), signature(m)));
                    w.inc();
                    w.line("todo!()");
                    w.dec();
                    w.line("}");
                }
                w.dec();
                w.line("}");
            }
            for r in &t.reals {
                w.blank();
                w.line(&format!("impl {r} for {name} {{"));
                w.inc();
                if let Some(tr) = traits.iter().find(|x| &x.name == r) {
                    for m in &tr.methods {
                        w.line(&format!("{} {{", signature(m)));
                        w.inc();
                        w.line("todo!()");
                        w.dec();
                        w.line("}");
                    }
                }
                w.dec();
                w.line("}");
            }
        }
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o, traits);
    }
}

// ------------------------------------------------------------------ reverse

fn vis_at(h: &[T], i: &mut usize) -> String {
    if *i < h.len() && h[*i].is("pub") {
        *i += 1;
        if *i < h.len() && h[*i].is("(") {
            *i = lex::close(h, *i) + 1;
            return "package".into();
        }
        return "public".into();
    }
    "private".into()
}

/// Drop `#[…]`, `#![…]` and doc attributes.
fn skip_attrs(mut t: &[T]) -> &[T] {
    loop {
        if t.len() >= 3 && t[0].is("#") && t[1].is("!") && t[2].is("[") {
            t = &t[(lex::close(t, 2) + 1).min(t.len())..];
        } else if t.len() >= 2 && t[0].is("#") && t[1].is("[") {
            t = &t[(lex::close(t, 1) + 1).min(t.len())..];
        } else {
            return t;
        }
    }
}

pub fn parse(src: &str, rel: &[String]) -> Parsed {
    let t = lex::lex(src, Cfg::default());
    let mut units = vec![];
    module(&t, rel.to_vec(), &mut units);
    Parsed { units, errors: !lex::balanced(&t) }
}

fn module(t: &[T], ns: Vec<String>, units: &mut Vec<Unit>) {
    let mut imports: Vec<Import> = vec![];
    let mut types: Vec<Ty> = vec![];
    let mut impls: Vec<Ty> = vec![];
    for st in lex::statements(t, false) {
        let h = skip_attrs(st.head);
        let doc = lex::doc_of(st.head);
        let mut i = 0;
        let v = vis_at(h, &mut i);
        while i < h.len() && h[i].id() && ["unsafe", "const", "async", "default", "auto"].contains(&h[i].s.as_str()) {
            i += 1;
        }
        let Some(kw) = h.get(i) else { continue };
        let name = h.get(i + 1).map(|x| x.s.clone()).unwrap_or_default();
        match kw.s.as_str() {
            "use" => {
                // `use crate::a::B;`, `use a::{B, C as D};`, `use a::*;` (module paths: a file is a module)
                let prefix: Vec<T> = h[i + 1..].iter().filter(|x| !x.is("crate") && !x.is("self") && !x.is("super")).cloned().collect();
                let prefix = lex::join(&prefix);
                let items: Vec<String> = match st.block {
                    Some(b) => lex::commas(b).into_iter().map(|p| format!("{prefix}{}", lex::join(p))).collect(),
                    None => vec![prefix],
                };
                for it in items {
                    let (path, alias) = match it.split_once(" as ") {
                        Some((p, a)) => (p.to_string(), Some(a.trim().to_string())),
                        None => (it.clone(), None),
                    };
                    let mut imp = Import::of(&path, &["::"]);
                    imp.module = true;
                    if let Some(a) = alias {
                        imp.alias = a;
                    }
                    imports.push(imp);
                }
            }
            "mod" => {
                if let Some(b) = st.block {
                    let mut sub = ns.clone();
                    sub.push(name);
                    module(b, sub, units);
                }
            }
            "struct" | "union" => {
                let mut ty = Ty::new(Kind::Class, &name);
                ty.vis = v;
                ty.doc = doc;
                let g = lex::generic_params(h, i + 2).0;
                ty.generics = g.iter().map(|p| p.0.clone()).collect();
                ty.gen_bounds = g.into_iter().map(|p| (p.1, p.2)).collect();
                if let Some(b) = st.block {
                    for f in lex::commas(b) {
                        let f = skip_attrs(f);
                        let mut j = 0;
                        let fv = vis_at(f, &mut j);
                        let Some((n, fty, _)) = lex::colon_param(&f[j..], &[]) else { continue };
                        if n == "base" || n == "parent" || n == "super_" {
                            ty.supers.push(base_name(&fty));
                        } else {
                            ty.fields.push(Field { name: n, ty: fty, vis: fv, doc: lex::doc_of(f), ..Default::default() });
                        }
                    }
                }
                types.push(ty);
            }
            "enum" => {
                let mut ty = Ty::new(Kind::Enum, &name);
                ty.vis = v;
                ty.doc = doc;
                if let Some(b) = st.block {
                    for p in lex::commas(b) {
                        if let Some(x) = skip_attrs(p).first().filter(|x| x.id()) {
                            ty.literals.push(x.s.clone());
                        }
                    }
                }
                types.push(ty);
            }
            "trait" => {
                let mut ty = Ty::new(Kind::Interface, &name);
                ty.vis = v;
                ty.doc = doc;
                let (g, j) = lex::generic_params(h, i + 2);
                ty.generics = g.iter().map(|p| p.0.clone()).collect();
                ty.gen_bounds = g.into_iter().map(|p| (p.1, p.2)).collect();
                if j < h.len() && h[j].is(":") {
                    let end = h[j..].iter().position(|x| x.is("where")).map(|p| p + j).unwrap_or(h.len());
                    for part in h[j + 1..end].split(|x| x.is("+")) {
                        let s = lex::join(part);
                        if !s.is_empty() && !s.starts_with('\'') && s != "Sized" && s != "Send" && s != "Sync" {
                            ty.supers.push(s);
                        }
                    }
                }
                if let Some(b) = st.block {
                    for m in lex::statements(b, false) {
                        if let Some(mut f) = function(m.head) {
                            f.vis = "public".into();
                            f.is_abstract = m.block.is_none();
                            ty.methods.push(f);
                        }
                    }
                }
                types.push(ty);
            }
            "impl" => {
                // impl<T> [Trait for] Type<T> { … }
                let (_, j) = lex::generics(h, i + 1);
                let rest = &h[j.min(h.len())..];
                let end = rest.iter().position(|x| x.is("where")).unwrap_or(rest.len());
                let rest = &rest[..end];
                let (tr, target) = match rest.iter().position(|x| x.is("for")) {
                    Some(f) => (Some(lex::join(&rest[..f])), lex::join(&rest[f + 1..])),
                    None => (None, lex::join(rest)),
                };
                let mut ty = Ty::new(Kind::Class, &base_name(&target));
                match tr {
                    Some(tr) => ty.reals.push(base_name(&tr)),
                    None => {
                        if let Some(b) = st.block {
                            for m in lex::statements(b, false) {
                                let mh = skip_attrs(m.head);
                                let mut k = 0;
                                let mv = vis_at(mh, &mut k);
                                // associated constants are static read-only attributes
                                if mh.get(k).map(|x| x.is("const")).unwrap_or(false) && !mh.iter().any(|x| x.is("fn")) {
                                    if let Some((n, cty, d)) = lex::colon_param(&mh[k + 1..], &[]) {
                                        ty.fields.push(Field { name: n, ty: cty, vis: mv, is_static: true, read_only: true, default: d, ..Default::default() });
                                    }
                                    continue;
                                }
                                if let Some(mut f) = function(mh) {
                                    f.vis = mv;
                                    if f.ret == "Self" {
                                        f.ret = ty.name.clone();
                                    }
                                    f.doc = lex::doc_of(m.head);
                                    ty.methods.push(f);
                                }
                            }
                        }
                    }
                }
                impls.push(ty);
            }
            _ => {}
        }
    }
    // impl blocks join their type in this module, or stand for it (merged on import)
    for im in impls {
        match types.iter_mut().find(|x| x.name == im.name) {
            Some(t) => {
                t.methods.extend(im.methods);
                t.fields.extend(im.fields);
                t.reals.extend(im.reals);
            }
            None => types.push(im),
        }
    }
    units.push(Unit { ns, types, imports });
}

/// `[pub] [const] [async] [unsafe] fn name<T>(params) -> Ret`: static when there is no `self`.
fn function(h: &[T]) -> Option<Method> {
    let h = skip_attrs(h);
    let f = h.iter().position(|x| x.is("fn"))?;
    let name = h.get(f + 1)?.s.clone();
    let p = h[f..].iter().position(|x| x.is("("))? + f;
    let q = lex::close(h, p);
    let mut is_static = true;
    let mut params = vec![];
    for part in lex::commas(&h[p + 1..q]) {
        let has_self = part.iter().any(|x| x.is("self")) && part.iter().take_while(|x| !x.is(":")).any(|x| x.is("self"));
        if has_self {
            is_static = false;
            continue;
        }
        if let Some((n, ty, _)) = lex::colon_param(part, &["mut"]) {
            params.push(Param { name: n, ty, ..Default::default() });
        }
    }
    let mut ret = String::new();
    if q + 1 < h.len() && h[q + 1].is("->") {
        let end = h[q + 2..].iter().position(|x| x.is("where")).map(|e| e + q + 2).unwrap_or(h.len());
        ret = lex::join(&h[q + 2..end]);
    }
    Some(Method { name, params, ret, is_static, vis: "private".into(), ..Default::default() })
}
