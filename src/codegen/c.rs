//! C: a header and a source file per class in its package folder. A class is a `typedef struct`
//! (its first member is the super class), its operations are functions `Name_op(Name* self, …)`
//! (static ones without `self`); an interface is a struct of function pointers; an enumeration is
//! a `typedef enum` with `NAME_LITERAL` constants. The reverse reads structs, unions, enums and
//! typedefs, and attaches each function to the struct its name starts with (`Name_`) or whose
//! pointer its first parameter takes.

use super::lex::{self, Cfg, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "void*".to_string() } else { primitive(t, "char*", "int", "bool", "double") };
    if is_many(mult) { format!("{t}*") } else { t }
}

fn default_return(t: &str) -> &'static str {
    match t {
        "bool" => "false",
        _ if t.ends_with('*') => "NULL",
        "int" | "long" | "short" | "char" | "unsigned" | "double" | "float" | "size_t" => "0",
        _ => "{0}",
    }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

fn signature(t: &Ty, m: &Method, pointer: bool) -> String {
    let name = ident(&t.name);
    let (rt, rm) = many(&m.ret);
    let ret = if m.ret.is_empty() { "void".to_string() } else { map_type(rt, rm) };
    let mut ps = vec![];
    if !m.is_static {
        ps.push(if pointer { "void* self".to_string() } else { format!("{name}* self") });
    }
    for p in &m.params {
        let (pt, pm) = many(&p.ty);
        ps.push(format!("{} {}", map_type(pt, pm), ident(&p.name)));
    }
    if ps.is_empty() {
        ps.push("void".into());
    }
    if pointer { format!("{ret} (*{})({})", ident(&m.name), ps.join(", ")) } else { format!("{ret} {name}_{}({})", ident(&m.name), ps.join(", ")) }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let mut out = vec![];
    let paths = tree.type_paths();
    for p in tree.all() {
        for t in &p.types {
            let dir: PathBuf = p.path.iter().collect();
            let name = ident(&t.name);
            let guard = format!("_{}_H", name.to_uppercase());
            let mut h = W::new(o);
            h.line(&format!("#ifndef {guard}"));
            h.line(&format!("#define {guard}"));
            h.blank();
            h.line("#include <stdbool.h>");
            for n in t.supers.iter().chain(t.fields.iter().map(|f| &f.ty)) {
                if let Some((tn, tp)) = paths.iter().find(|x| &x.0 == n && x.0 != t.name) {
                    let up = "../".repeat(p.path.len());
                    let rel: Vec<String> = tp.iter().cloned().chain([format!("{}.h", ident(tn))]).collect();
                    h.line(&format!("#include \"{up}{}\"", rel.join("/")));
                }
            }
            h.blank();
            h.doc(o, &t.doc, DocStyle::Block);
            match t.kind {
                Kind::Enum => {
                    h.line("typedef enum {");
                    h.inc();
                    let lits: Vec<String> = t.literals.iter().map(|l| format!("{}_{}", name.to_uppercase(), ident(l).to_uppercase())).collect();
                    h.line(&lits.join(",\n"));
                    h.dec();
                    h.line(&format!("}} {name};"));
                }
                Kind::Interface => {
                    h.line(&format!("typedef struct {name} {{"));
                    h.inc();
                    for m in &t.methods {
                        h.doc(o, &m.doc, DocStyle::Block);
                        h.line(&format!("{};", signature(t, m, true)));
                    }
                    h.dec();
                    h.line(&format!("}} {name};"));
                }
                Kind::Class => {
                    h.line(&format!("typedef struct {name} {{"));
                    h.inc();
                    if let Some(s) = t.supers.first() {
                        h.line(&format!("{s} base;"));
                    }
                    for f in t.fields.iter().filter(|f| !f.is_static) {
                        h.doc(o, &f.doc, DocStyle::Block);
                        h.line(&format!("{}{} {};", if f.read_only { "const " } else { "" }, map_type(&f.ty, &f.mult), ident(&f.name)));
                    }
                    h.dec();
                    h.line(&format!("}} {name};"));
                    for f in t.fields.iter().filter(|f| f.is_static) {
                        h.line(&format!("extern {} {name}_{};", map_type(&f.ty, &f.mult), ident(&f.name)));
                    }
                    for m in &t.methods {
                        h.blank();
                        h.doc(o, &m.doc, DocStyle::Block);
                        h.line(&format!("{};", signature(t, m, false)));
                    }
                }
            }
            h.blank();
            h.line(&format!("#endif //{guard}"));
            out.push((dir.join(format!("{name}.h")), h.out));
            if t.kind != Kind::Class {
                continue;
            }
            let mut c = W::new(o);
            c.line(&format!("#include \"{name}.h\""));
            for f in t.fields.iter().filter(|f| f.is_static) {
                c.blank();
                let def = if f.default.is_empty() { String::new() } else { format!(" = {}", f.default) };
                c.line(&format!("{} {name}_{}{def};", map_type(&f.ty, &f.mult), ident(&f.name)));
            }
            for m in &t.methods {
                c.blank();
                c.line(&format!("{} {{", signature(t, m, false)));
                c.inc();
                c.line("// TODO implement here");
                if !m.ret.is_empty() {
                    let (rt, rm) = many(&m.ret);
                    c.line(&format!("return {};", default_return(&map_type(rt, rm))));
                }
                c.dec();
                c.line("}");
            }
            out.push((dir.join(format!("{name}.c")), c.out));
        }
    }
    out
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 6] = ["static", "extern", "inline", "register", "volatile", "_Noreturn"];

pub fn parse(src: &str, rel: &[String]) -> Parsed {
    let src = super::cpp::strip_preprocessor(src);
    let t = lex::lex(&src, Cfg::default());
    let mut types: Vec<Ty> = vec![];
    let mut funcs: Vec<(String, Method, Option<String>)> = vec![];
    let stmts = lex::statements(&t, false);
    let mut i = 0;
    while i < stmts.len() {
        let st = &stmts[i];
        let h = st.head;
        if let Some(b) = st.block {
            if h.iter().any(|x| x.is("struct") || x.is("union") || x.is("enum")) && !h.iter().any(|x| x.is("(")) {
                if let Some(mut ty) = super::cpp::type_of(h, b) {
                    // `typedef struct X { … } Name;`: the name follows the block
                    if let Some(next) = stmts.get(i + 1) {
                        if h.first().map(|x| x.is("typedef")).unwrap_or(false) && next.block.is_none() {
                            if let Some(n) = next.head.iter().rev().find(|x| x.id()) {
                                ty.name = n.s.clone();
                            }
                            i += 1;
                        }
                    }
                    ty.stereotype.clear();
                    struct_members(&mut ty, b);
                    if !ty.name.is_empty() {
                        types.push(ty);
                    }
                }
                i += 1;
                continue;
            }
        }
        // a function declaration or definition
        if h.iter().any(|x| x.is("(")) && !h.first().map(|x| x.is("typedef")).unwrap_or(false) {
            for d in lex::typed_decls(h, &MODS) {
                let Some(ps) = d.params else { continue };
                let mut params: Vec<Param> = ps.into_iter().filter(|p| !(p.0.is_empty() && p.1 == "void")).map(|(n, ty, def)| Param { name: n, ty, default: def }).collect();
                let self_ty = params.first().filter(|p| p.ty.trim_end().ends_with('*')).map(|p| base_name(&p.ty));
                let m = Method { name: d.name.clone(), ret: d.ty, vis: if d.mods.iter().any(|m| m == "static") { "private".into() } else { "public".into() }, doc: lex::doc_of(h), ..Default::default() };
                let mut m = m;
                m.params = std::mem::take(&mut params);
                funcs.push((d.name, m, self_ty));
            }
        }
        i += 1;
    }
    // each function goes to its struct: by `Name_` prefix, else by its `Name*` first parameter
    for (name, mut m, self_ty) in funcs {
        let by_prefix = types.iter().position(|t| name.starts_with(&format!("{}_", t.name)));
        let by_self = self_ty.as_ref().and_then(|s| types.iter().position(|t| &t.name == s));
        let Some(k) = by_prefix.or(by_self) else { continue };
        if by_prefix.is_some() {
            m.name = name[types[k].name.len() + 1..].to_string();
        }
        let is_self = m.params.first().map(|p| base_name(&p.ty) == types[k].name && p.ty.contains('*')).unwrap_or(false);
        if is_self {
            m.params.remove(0);
        } else {
            m.is_static = true;
        }
        if types[k].kind == Kind::Class {
            types[k].methods.push(m);
        }
    }
    Parsed { units: vec![Unit { ns: rel.to_vec(), types, ..Default::default() }], errors: !lex::balanced(&t) }
}

/// A struct's members: fields, a first member of struct type named `base`/`super`/`parent` as the
/// super class, and function pointers (`ret (*op)(…)`) as operations; a struct of function
/// pointers only is an interface.
fn struct_members(t: &mut Ty, body: &[T]) {
    if t.kind == Kind::Enum {
        return;
    }
    t.fields.clear();
    t.methods.clear();
    let mut first = true;
    for st in lex::statements(body, false) {
        let h = st.head;
        // `ret (*name)(params)`
        if let Some(p) = h.iter().position(|x| x.is("(")) {
            if h.get(p + 1).map(|x| x.is("*")).unwrap_or(false) {
                if let (Some(n), Some(q)) = (h.get(p + 2), h.iter().skip(p + 3).position(|x| x.is("(")).map(|q| q + p + 3)) {
                    let close = lex::close(h, q);
                    let mut params: Vec<Param> = lex::commas(&h[q + 1..close]).into_iter().filter_map(|x| lex::typed_param(x, &MODS)).filter(|p| !(p.0.is_empty() && p.1 == "void")).map(|(n, ty, def)| Param { name: n, ty, default: def }).collect();
                    if params.first().map(|p| p.name == "self" || p.name == "this").unwrap_or(false) {
                        params.remove(0);
                    }
                    t.methods.push(Method { name: n.s.clone(), ret: lex::join(&h[..p]), vis: "public".into(), is_abstract: true, params, doc: lex::doc_of(h), ..Default::default() });
                    first = false;
                    continue;
                }
            }
        }
        for d in lex::typed_decls(h, &MODS) {
            if first && ["base", "super", "parent"].contains(&d.name.as_str()) && !d.ty.contains('*') {
                t.supers.push(base_name(&d.ty));
            } else {
                let read_only = d.ty.starts_with("const ");
                t.fields.push(Field { name: d.name, ty: d.ty.trim_start_matches("const ").to_string(), vis: "public".into(), read_only, doc: lex::doc_of(h), ..Default::default() });
            }
            first = false;
        }
    }
    if !t.methods.is_empty() && t.fields.is_empty() && t.supers.is_empty() {
        t.kind = Kind::Interface;
    } else {
        for m in t.methods.iter_mut() {
            m.is_abstract = false;
        }
    }
}
