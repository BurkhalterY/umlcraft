//! TypeScript: a `.ts` module per type in its package folder (`export class`, `interface`, `enum`,
//! relative imports of the super types), and the reverse of TypeScript and JavaScript files
//! (namespaces, classes with `extends`/`implements`, interfaces, enums, fields, parameter
//! properties, `this.x` assignments in constructors, methods and accessors). After
//! staruml-typescript's generator.

use super::java::super_lists;
use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "any".to_string() } else { primitive(t, "string", "number", "boolean", "number") };
    let t = match t.as_str() {
        "int" | "long" | "float" | "double" | "short" | "byte" => "number".to_string(),
        "bool" => "boolean".into(),
        _ => t,
    };
    if is_many(mult) { format!("{t}[]") } else { t }
}

fn default_return(t: &str) -> &'static str {
    match t {
        "number" => "0",
        "boolean" => "false",
        "string" => "\"\"",
        "void" => "",
        _ if t.ends_with("[]") => "[]",
        _ => "null",
    }
}

fn vis(v: &str) -> &'static str {
    match v {
        "protected" => "protected ",
        "private" => "private ",
        _ => "",
    }
}

/// The relative module path from one package folder to a type's module (`../model/Book`).
pub fn relative(from: &[String], to: &[String], file: &str) -> String {
    let common = from.iter().zip(to.iter()).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = vec![];
    if from.len() == common {
        parts.push(".".into());
    } else {
        for _ in common..from.len() {
            parts.push("..".into());
        }
    }
    parts.extend(to[common..].iter().cloned());
    parts.push(file.to_string());
    parts.join("/")
}

/// The import lines of the tree's types a type refers to (supers, realizations, field types).
pub fn imports(tree: &Pkg, p: &Pkg, t: &Ty, ext: &str) -> Vec<String> {
    let paths = tree.type_paths();
    let mut out = vec![];
    let names = t.supers.iter().chain(t.reals.iter()).cloned().chain(t.fields.iter().map(|f| f.ty.clone())).chain(t.methods.iter().flat_map(|m| m.params.iter().map(|p| p.ty.trim_end_matches("[]").to_string()).chain([m.ret.trim_end_matches("[]").to_string()])));
    let mut seen = vec![];
    for n in names {
        if n == t.name || seen.contains(&n) {
            continue;
        }
        if let Some((tn, tp)) = paths.iter().find(|x| x.0 == n) {
            seen.push(n.clone());
            out.push(format!("import {{ {} }} from \"{}{ext}\";", ident(tn), relative(&p.path, tp, &ident(tn))));
        }
    }
    out
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    per_type(tree, |t| format!("{}.ts", ident(&t.name)), |p, t| {
        let mut w = W::new(o);
        let imps = imports(tree, p, t, "");
        for i in &imps {
            w.line(i);
        }
        if !imps.is_empty() {
            w.blank();
        }
        write_type(&mut w, t, o, "export ");
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts, export: &str) {
    w.doc(o, &t.doc, DocStyle::Block);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    let name = ident(&t.name);
    match t.kind {
        Kind::Enum => {
            w.line(&format!("{export}enum {name} {{"));
            w.inc();
            let lits: Vec<String> = t.literals.iter().map(|l| ident(l)).collect();
            w.line(&lits.join(",\n"));
            w.dec();
            w.line("}");
            return;
        }
        Kind::Interface => {
            let ext = if t.supers.is_empty() { String::new() } else { format!(" extends {}", t.supers.join(", ")) };
            w.line(&format!("{export}interface {name}{gen}{ext} {{"));
            w.inc();
            for f in &t.fields {
                w.doc(o, &f.doc, DocStyle::Block);
                w.line(&format!("{}{}{}: {};", if f.read_only { "readonly " } else { "" }, ident(&f.name), if is_optional(&f.mult) { "?" } else { "" }, map_type(&f.ty, &f.mult)));
            }
            for m in &t.methods {
                w.doc(o, &m.doc, DocStyle::Block);
                w.line(&format!("{}({}): {};", ident(&m.name), params(m), ret(m)));
            }
            w.dec();
            w.line("}");
            return;
        }
        Kind::Class => {}
    }
    let abs = t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
    let mut head = format!("{export}{}class {name}{gen}", if abs { "abstract " } else { "" });
    if let Some(s) = t.supers.first() {
        head.push_str(&format!(" extends {s}"));
    }
    if !t.reals.is_empty() {
        head.push_str(&format!(" implements {}", t.reals.join(", ")));
    }
    w.line(&format!("{head} {{"));
    w.inc();
    for f in &t.fields {
        w.doc(o, &f.doc, DocStyle::Block);
        let mut s = vis(&f.vis).to_string();
        if f.is_static {
            s.push_str("static ");
        }
        if f.read_only {
            s.push_str("readonly ");
        }
        s.push_str(&format!("{}{}: {}", ident(&f.name), if is_optional(&f.mult) { "?" } else { "" }, map_type(&f.ty, &f.mult)));
        if !f.default.is_empty() {
            s.push_str(&format!(" = {}", f.default));
        }
        w.line(&format!("{s};"));
    }
    for m in &t.methods {
        w.blank();
        w.doc(o, &m.doc, DocStyle::Block);
        let mut s = vis(&m.vis).to_string();
        if m.is_static {
            s.push_str("static ");
        }
        if m.is_abstract {
            s.push_str("abstract ");
        }
        let r = ret(m);
        s.push_str(&format!("{}({}): {r}", ident(&m.name), params(m)));
        if m.is_abstract {
            w.line(&format!("{s};"));
            continue;
        }
        w.line(&format!("{s} {{"));
        w.inc();
        w.line("// TODO implement here");
        let d = default_return(&r);
        if !d.is_empty() && !m.ret.is_empty() {
            w.line(&format!("return {d};"));
        }
        w.dec();
        w.line("}");
    }
    w.dec();
    w.line("}");
    for n in &t.nested {
        w.blank();
        write_type(w, n, o, "export ");
    }
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
        let d = if p.default.is_empty() { String::new() } else { format!(" = {}", p.default) };
        format!("{}: {}{d}", ident(&p.name), map_type(pt, pm))
    }).collect::<Vec<_>>().join(", ")
}

fn ret(m: &Method) -> String {
    let (rt, rm) = many(&m.ret);
    if m.ret.is_empty() { "void".into() } else { map_type(rt, rm) }
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 13] = ["public", "private", "protected", "static", "readonly", "abstract", "declare", "override", "async", "accessor", "export", "default", "const"];

/// Read a TypeScript (`typed`) or JavaScript file.
pub fn parse(src: &str, rel: &[String], typed: bool) -> Parsed {
    let t = lex::lex(src, Cfg { sq_strings: true, ..Default::default() });
    let mut units = vec![];
    space(&t, rel.to_vec(), typed, &mut units);
    Parsed { units, errors: !lex::balanced(&t) }
}

fn space(t: &[T], ns: Vec<String>, typed: bool, units: &mut Vec<Unit>) {
    let mut unit = Unit { ns: ns.clone(), types: vec![], ..Default::default() };
    let sts = lex::statements(t, true);
    let mut skip_from = false;
    for (si, st) in sts.iter().enumerate() {
        if let Some(imps) = imports_of(&sts, si) {
            unit.imports.extend(imps);
            // `import { A } from 'm'`: the `from 'm'` is the next statement
            skip_from = st.block.is_some();
            continue;
        }
        if std::mem::take(&mut skip_from) && st.head.first().map(|x| x.is("from")).unwrap_or(false) {
            continue;
        }
        let mut h = lex::skip_annotations(st.head);
        while !h.is_empty() && h[0].id() && ["export", "default", "declare", "abstract", "const"].contains(&h[0].s.as_str()) && h.len() > 1 {
            h = &h[1..];
        }
        let Some(b) = st.block else { continue };
        if h.is_empty() {
            continue;
        }
        if (h[0].is("namespace") || h[0].is("module")) && h.len() > 1 && h[1].k == K::Id {
            let mut sub = ns.clone();
            sub.extend(lex::join(&h[1..]).split('.').map(|s| s.trim().to_string()));
            space(b, sub, typed, units);
        } else if let Some(ty) = read_type(st.head, b, typed) {
            unit.types.push(ty);
        }
    }
    units.push(unit);
}

fn read_type(head: &[T], body: &[T], typed: bool) -> Option<Ty> {
    let h = lex::skip_annotations(head);
    let k = h.iter().position(|x| x.k == K::Id && ["class", "interface", "enum"].contains(&x.s.as_str()))?;
    if !h[..k].iter().all(|x| x.id() && ["export", "default", "declare", "abstract", "const"].contains(&x.s.as_str())) {
        return None;
    }
    let kind = match h[k].s.as_str() {
        "interface" => Kind::Interface,
        "enum" => Kind::Enum,
        _ => Kind::Class,
    };
    let name = h.get(k + 1).filter(|x| x.id() && !x.is("extends")).map(|x| x.s.clone()).unwrap_or_else(|| "default".into());
    let mut t = Ty::new(kind, &name);
    t.doc = lex::doc_of(head);
    t.is_abstract = h[..k].iter().any(|x| x.is("abstract"));
    let (g, i) = lex::generic_params(h, k + 2);
    t.generics = g.iter().map(|p| p.0.clone()).collect();
    t.gen_bounds = g.into_iter().map(|p| (p.1, p.2)).collect();
    let (s, r) = super_lists(&h[i.min(h.len())..], &["extends"], &["implements"], &[]);
    t.supers = s;
    t.reals = r;
    if kind == Kind::Enum {
        for p in lex::commas(body) {
            if let Some(x) = p.first() {
                t.literals.push(x.s.trim_matches(['"', '\'']).to_string());
            }
        }
        return Some(t);
    }
    for st in lex::statements(body, true) {
        member(&mut t, st.head, st.block, typed);
    }
    Some(t)
}

fn member(t: &mut Ty, head: &[T], block: Option<&[T]>, typed: bool) {
    let mut h = lex::skip_annotations(head);
    let doc = lex::doc_of(head);
    let mut mods = vec![];
    while h.len() > 1 && h[0].id() && MODS.contains(&h[0].s.as_str()) && !(h[1].is("(") || h[1].is(":") || h[1].is("=") || h[1].is("?") || h[1].is(";")) {
        mods.push(h[0].s.clone());
        h = &h[1..];
    }
    let mut accessor = "";
    if h.len() > 1 && (h[0].is("get") || h[0].is("set")) && h[1].id() {
        accessor = if h[0].is("get") { "get" } else { "set" };
        h = &h[1..];
    }
    if !h.is_empty() && h[0].is("*") {
        h = &h[1..];
    }
    let mut private_hash = false;
    if h.len() > 1 && h[0].is("#") {
        private_hash = true;
        h = &h[1..];
    }
    if h.is_empty() || !(h[0].id() || h[0].k == K::Str) {
        return;
    }
    let name = h[0].s.trim_matches(['"', '\'']).to_string();
    let vis = if private_hash { "private".to_string() } else { vis_of(&mods, "public") };
    let is_static = mods.iter().any(|m| m == "static");
    // a method: name [<T>] (params) [: ret]
    let mut i = 1;
    if i < h.len() && h[i].is("?") {
        i += 1;
    }
    let mut mg = vec![];
    if i < h.len() && h[i].is("<") {
        let (g, j) = lex::generics(h, i);
        mg = g;
        i = j;
    }
    if i < h.len() && h[i].is("(") {
        let q = lex::close(h, i);
        let ps: Vec<Param> = lex::commas(&h[i + 1..q]).into_iter().filter_map(|p| lex::colon_param(p, &MODS)).filter(|p| p.0 != "this").map(|(n, ty, d)| Param { name: n, ty: if typed { ty.trim_end_matches('?').to_string() } else { String::new() }, default: d }).collect();
        let ret = if typed && q + 1 < h.len() && h[q + 1].is(":") { lex::join(&h[q + 2..]) } else { String::new() };
        if name == "constructor" {
            // parameter properties (`constructor(private a: number)`) are fields
            for p in lex::commas(&h[i + 1..q]) {
                let p = lex::skip_annotations(p);
                if p.first().map(|x| x.id() && ["public", "private", "protected", "readonly"].contains(&x.s.as_str())).unwrap_or(false) {
                    let pm: Vec<String> = p.iter().take_while(|x| x.id() && MODS.contains(&x.s.as_str())).map(|x| x.s.clone()).collect();
                    if let Some((n, ty, d)) = lex::colon_param(p, &MODS) {
                        t.fields.push(Field { name: n, ty: if typed { ty } else { String::new() }, vis: vis_of(&pm, "public"), read_only: pm.iter().any(|m| m == "readonly"), default: d, ..Default::default() });
                    }
                }
            }
            // `this.x = …` in the body
            if let Some(b) = block {
                for (k, x) in b.iter().enumerate() {
                    if x.is("this") && b.get(k + 1).map(|y| y.is(".")).unwrap_or(false) && b.get(k + 3).map(|y| y.is("=")).unwrap_or(false) && !b.get(k + 4).map(|y| y.is("=")).unwrap_or(false) {
                        if let Some(n) = b.get(k + 2).filter(|y| y.id()) {
                            if !t.fields.iter().any(|f| f.name == n.s) {
                                let end = b[k + 4..].iter().position(|y| y.is(";") || y.nl).map(|e| e + k + 4).unwrap_or(b.len());
                                let def = lex::join(&b[k + 4..end]);
                                t.fields.push(Field { name: n.s.clone(), vis: "public".into(), default: if def.contains("=>") || def.len() > 40 { String::new() } else { def }, ..Default::default() });
                            }
                        }
                    }
                }
            }
            return;
        }
        if accessor == "get" {
            if !t.fields.iter().any(|f| f.name == name) {
                t.fields.push(Field { name, ty: ret, vis, is_static, read_only: true, doc, ..Default::default() });
            }
            return;
        }
        if accessor == "set" {
            if let Some(f) = t.fields.iter_mut().find(|f| f.name == name) {
                f.read_only = false;
            }
            return;
        }
        let is_abstract = mods.iter().any(|m| m == "abstract");
        t.methods.push(Method { name, params: ps, ret, vis, is_static, is_abstract, doc, generics: mg, ..Default::default() });
        return;
    }
    // a field: name[?][!][: T][= v]
    let Some((n, ty, d)) = lex::colon_param(h, &[]) else { return };
    let d = if d.contains("=>") || d.starts_with("function") || d.len() > 60 { String::new() } else { d };
    t.fields.push(Field {
        name: n,
        ty: if typed { ty.trim_end_matches('?').to_string() } else { String::new() },
        mult: if typed && ty.ends_with('?') { "0..1".into() } else { String::new() },
        vis,
        is_static,
        read_only: mods.iter().any(|m| m == "readonly"),
        default: d,
        doc,
        ..Default::default()
    });
}

/// The imports of the statement at `si`, when it is one: `import { A, B as C } from './m'`,
/// `import D from './d'`, `import * as ns from './m'` and `import type …` (the braces read as a
/// block, so the module comes in the next statement). Their paths are module paths.
fn imports_of(sts: &[lex::Stmt<'_>], si: usize) -> Option<Vec<Import>> {
    let h = sts[si].head;
    if !h.first().map(|x| x.is("import")).unwrap_or(false) {
        return None;
    }
    let mut names: Vec<(String, String)> = vec![];
    let mut rest = &h[1..];
    if rest.first().map(|x| x.is("type")).unwrap_or(false) {
        rest = &rest[1..];
    }
    let module_tok = |t: &[T]| t.iter().position(|x| x.is("from")).and_then(|f| t.get(f + 1)).filter(|x| x.k == K::Str).map(|x| x.s.trim_matches(['"', '\'', '`']).to_string());
    let mut module = module_tok(rest);
    // a default or namespace import before `from` or `,`
    let mut i = 0;
    while i < rest.len() && !rest[i].is("from") {
        if rest[i].is("*") && rest.get(i + 1).map(|x| x.is("as")).unwrap_or(false) {
            if let Some(n) = rest.get(i + 2) {
                names.push((String::new(), n.s.clone()));
            }
            i += 3;
            continue;
        }
        if rest[i].id() {
            names.push((rest[i].s.clone(), rest[i].s.clone()));
        }
        i += 1;
    }
    if let Some(b) = sts[si].block {
        for p in lex::commas(b) {
            let p: Vec<&T> = p.iter().filter(|x| !x.is("type")).collect();
            match p.iter().position(|x| x.is("as")) {
                Some(a) if a > 0 && a + 1 < p.len() => names.push((p[a - 1].s.clone(), p[a + 1].s.clone())),
                _ => {
                    if let Some(n) = p.first().filter(|x| x.id()) {
                        names.push((n.s.clone(), n.s.clone()));
                    }
                }
            }
        }
        if module.is_none() {
            module = sts.get(si + 1).and_then(|n| module_tok(n.head));
        }
    }
    let path: Vec<String> = module.unwrap_or_default().split('/').filter(|s| !s.is_empty() && *s != "." && *s != "..").map(|s| s.to_string()).collect();
    Some(
        names
            .into_iter()
            .map(|(real, alias)| {
                let mut p = path.clone();
                if !real.is_empty() {
                    p.push(real);
                }
                Import { path: p, alias, wildcard: false, module: true }
            })
            .collect(),
    )
}
