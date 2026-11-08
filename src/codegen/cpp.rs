//! C++: a header (`Name.h`, with an include guard) and a source file (`Name.cpp`) per class in its
//! package folder, the packages as nested namespaces; and the reverse of headers and sources
//! (namespaces, classes, structs, enums, access sections, members, templates). Rules after
//! staruml-cpp's generator (`_NAME_H` guards, `public:`/`protected:`/`private:` sections,
//! pure virtual operations for abstract ones).

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "void*".to_string() } else { primitive(t, "std::string", "int", "bool", "double") };
    if is_many(mult) { format!("std::vector<{t}>") } else { t }
}

fn default_return(t: &str) -> &'static str {
    match t {
        "bool" => "false",
        "int" | "long" | "short" | "char" | "unsigned" | "double" | "float" | "size_t" => "0",
        "std::string" => "\"\"",
        _ if t.ends_with('*') => "nullptr",
        _ => "{}",
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
        format!("{} {}", map_type(pt, pm), ident(&p.name))
    }).collect::<Vec<_>>().join(", ")
}

fn ret(m: &Method) -> String {
    let (rt, rm) = many(&m.ret);
    if m.ret.is_empty() { "void".into() } else { map_type(rt, rm) }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let mut out = vec![];
    for p in tree.all() {
        for t in &p.types {
            let dir: PathBuf = p.path.iter().collect();
            let name = ident(&t.name);
            let guard = format!("_{}_H", name.to_uppercase());
            let mut h = W::new(o);
            h.line(&format!("#ifndef {guard}"));
            h.line(&format!("#define {guard}"));
            h.blank();
            h.line("#include <string>");
            h.line("#include <vector>");
            // the headers of the super types and field types of this tree
            let paths = tree.type_paths();
            let mut incs = vec![];
            for n in t.supers.iter().chain(t.reals.iter()).chain(t.fields.iter().map(|f| &f.ty)) {
                if let Some((tn, tp)) = paths.iter().find(|x| &x.0 == n && x.0 != t.name) {
                    let rel: Vec<String> = tp.iter().cloned().chain([format!("{}.h", ident(tn))]).collect();
                    let up = "../".repeat(p.path.len());
                    let inc = format!("#include \"{up}{}\"", rel.join("/"));
                    if !incs.contains(&inc) {
                        incs.push(inc);
                    }
                }
            }
            for i in incs {
                h.line(&i);
            }
            h.blank();
            open_ns(&mut h, &p.path);
            write_decl(&mut h, t, o);
            close_ns(&mut h, &p.path);
            h.blank();
            h.line(&format!("#endif //{guard}"));
            out.push((dir.join(format!("{name}.h")), h.out));
            if t.kind != Kind::Class || t.methods.iter().all(|m| m.is_abstract) {
                continue;
            }
            let mut c = W::new(o);
            c.line(&format!("#include \"{name}.h\""));
            c.blank();
            open_ns(&mut c, &p.path);
            write_bodies(&mut c, t, &name);
            close_ns(&mut c, &p.path);
            out.push((dir.join(format!("{name}.cpp")), c.out));
        }
    }
    out
}

fn open_ns(w: &mut W, path: &[String]) {
    for n in path {
        w.line(&format!("namespace {} {{", ident(n)));
    }
    if !path.is_empty() {
        w.blank();
    }
}

fn close_ns(w: &mut W, path: &[String]) {
    for n in path.iter().rev() {
        w.blank();
        w.line(&format!("}} // namespace {}", ident(n)));
    }
}

fn write_decl(w: &mut W, t: &Ty, o: &GenOpts) {
    w.doc(o, &t.doc, DocStyle::Block);
    let name = ident(&t.name);
    if !t.generics.is_empty() {
        w.line(&format!("template<{}>", t.generics.iter().map(|g| format!("typename {g}")).collect::<Vec<_>>().join(", ")));
    }
    if t.kind == Kind::Enum {
        w.line(&format!("enum class {name} {{"));
        w.inc();
        let lits: Vec<String> = t.literals.iter().map(|l| ident(l)).collect();
        w.line(&lits.join(",\n"));
        w.dec();
        w.line("};");
        return;
    }
    let bases: Vec<String> = t.supers.iter().chain(t.reals.iter()).map(|b| format!("public {b}")).collect();
    let colon = if bases.is_empty() { String::new() } else { format!(" : {}", bases.join(", ")) };
    w.line(&format!("class {name}{colon} {{"));
    let interface = t.kind == Kind::Interface;
    for section in ["public", "protected", "private"] {
        let fields: Vec<&Field> = t.fields.iter().filter(|f| sec(&f.vis) == section).collect();
        let methods: Vec<&Method> = t.methods.iter().filter(|m| sec(&m.vis) == section).collect();
        let ctor = section == "public" && t.kind == Kind::Class;
        if fields.is_empty() && methods.is_empty() && !ctor && !(interface && section == "public") {
            continue;
        }
        w.line(&format!("{section}:"));
        w.inc();
        if ctor {
            w.line(&format!("{name}();"));
        }
        if interface && section == "public" {
            w.line(&format!("virtual ~{name}() = default;"));
        }
        for f in fields {
            w.doc(o, &f.doc, DocStyle::Block);
            let mut s = String::new();
            if f.is_static {
                s.push_str("static ");
            }
            if f.read_only {
                s.push_str("const ");
            }
            s.push_str(&format!("{} {}", map_type(&f.ty, &f.mult), ident(&f.name)));
            if !f.default.is_empty() && !(f.is_static && !f.read_only) {
                s.push_str(&format!(" = {}", f.default));
            }
            w.line(&format!("{s};"));
        }
        for m in methods {
            w.doc(o, &m.doc, DocStyle::Block);
            let abs = m.is_abstract || interface;
            let pre = if m.is_static { "static " } else if abs { "virtual " } else { "" };
            w.line(&format!("{pre}{} {}({}){};", ret(m), ident(&m.name), params(m), if abs { " = 0" } else { "" }));
        }
        w.dec();
    }
    for n in &t.nested {
        w.blank();
        w.inc();
        write_decl(w, n, o);
        w.dec();
    }
    w.line("};");
}

fn sec(v: &str) -> &'static str {
    match v {
        "protected" => "protected",
        "private" => "private",
        _ => "public",
    }
}

fn write_bodies(w: &mut W, t: &Ty, name: &str) {
    w.line(&format!("{name}::{name}() {{"));
    w.line("}");
    for f in t.fields.iter().filter(|f| f.is_static && !f.read_only) {
        w.blank();
        let def = if f.default.is_empty() { String::new() } else { format!(" = {}", f.default) };
        w.line(&format!("{} {name}::{}{def};", map_type(&f.ty, &f.mult), ident(&f.name)));
    }
    for m in t.methods.iter().filter(|m| !m.is_abstract) {
        w.blank();
        let r = ret(m);
        w.line(&format!("{r} {name}::{}({}) {{", ident(&m.name), params(m)));
        w.inc();
        w.line("// TODO implement here");
        if !m.ret.is_empty() {
            w.line(&format!("return {};", default_return(&r)));
        }
        w.dec();
        w.line("}");
    }
}

// ------------------------------------------------------------------ reverse

pub const MODS: [&str; 14] = ["static", "virtual", "inline", "explicit", "extern", "mutable", "constexpr", "friend", "public", "protected", "private", "typedef", "register", "consteval"];

/// Read a header or source file: types under their namespaces, or under the folders when there
/// is none (`rel`).
pub fn parse(src: &str, rel: &[String]) -> Parsed {
    let src = strip_preprocessor(src);
    let t = lex::lex(&src, Cfg::default());
    let mut units = vec![];
    space(&t, vec![], rel, &mut units);
    Parsed { units, errors: !lex::balanced(&t) }
}

/// Preprocessor lines (with their continuations) are dropped.
pub fn strip_preprocessor(src: &str) -> String {
    let mut out = String::new();
    let mut cont = false;
    for l in src.lines() {
        let pre = cont || l.trim_start().starts_with('#');
        cont = pre && l.trim_end().ends_with('\\');
        if !pre {
            out.push_str(l);
        }
        out.push('\n');
    }
    out
}

fn space(t: &[T], ns: Vec<String>, rel: &[String], units: &mut Vec<Unit>) {
    let path = if ns.is_empty() { rel.to_vec() } else { ns.clone() };
    let mut unit = Unit { ns: path, types: vec![] };
    for st in lex::statements(t, false) {
        let mut h = st.head;
        if h.first().map(|x| x.is("inline")).unwrap_or(false) {
            h = &h[1..];
        }
        if h.first().map(|x| x.is("namespace")).unwrap_or(false) {
            if let Some(b) = st.block {
                let mut sub = ns.clone();
                sub.extend(lex::join(&h[1..]).split("::").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
                space(b, sub, rel, units);
            }
        } else if h.first().map(|x| x.is("extern")).unwrap_or(false) && st.block.is_some() {
            space(st.block.unwrap(), ns.clone(), rel, units);
        } else if let Some(b) = st.block {
            if let Some(ty) = type_of(h, b) {
                unit.types.push(ty);
            }
        }
    }
    units.push(unit);
}

/// `template<…> class|struct|union|enum [class] Name [: bases] { … }` (and C's `typedef struct`).
pub fn type_of(h: &[T], body: &[T]) -> Option<Ty> {
    let mut h = h;
    let mut generics = vec![];
    if h.first().map(|x| x.is("template")).unwrap_or(false) && h.get(1).map(|x| x.is("<")).unwrap_or(false) {
        let (g, i) = lex::generics(h, 1);
        generics = g;
        h = &h[i.min(h.len())..];
    }
    let typedef = h.first().map(|x| x.is("typedef")).unwrap_or(false);
    if typedef {
        h = &h[1..];
    }
    let k = h.iter().position(|x| x.k == K::Id && ["class", "struct", "union", "enum"].contains(&x.s.as_str()))?;
    if k > 0 && !h[..k].iter().all(|x| x.id() && ["export", "static", "extern"].contains(&x.s.as_str())) {
        return None;
    }
    let mut i = k + 1;
    if h[k].is("enum") && h.get(i).map(|x| x.is("class") || x.is("struct")).unwrap_or(false) {
        i += 1;
    }
    // attributes and export macros before the name are skipped: the name is the last identifier
    // before `:`, `final` or the end
    let end = h[i..].iter().position(|x| x.is(":")).map(|p| p + i).unwrap_or(h.len());
    let name = h[i..end].iter().rev().find(|x| x.id() && !x.is("final") && !x.is("alignas")).map(|x| x.s.clone()).unwrap_or_default();
    let kind = if h[k].is("enum") { Kind::Enum } else { Kind::Class };
    let mut t = Ty::new(kind, &name);
    t.generics = generics;
    t.doc = lex::doc_of(h);
    if h[k].is("struct") || h[k].is("union") {
        t.stereotype = h[k].s.clone();
    }
    if end < h.len() && kind == Kind::Class {
        for p in lex::commas(&h[end + 1..]) {
            let p: Vec<T> = p.iter().filter(|x| !(x.id() && ["public", "protected", "private", "virtual"].contains(&x.s.as_str()))).cloned().collect();
            t.supers.push(lex::join(&p));
        }
    }
    if kind == Kind::Enum {
        for p in lex::commas(body) {
            if let Some(x) = p.first().filter(|x| x.id()) {
                t.literals.push(x.s.clone());
            }
        }
        return Some(t);
    }
    let mut access = if h[k].is("class") { "private" } else { "public" }.to_string();
    let mut abstract_count = 0;
    for st in lex::statements(body, false) {
        let mut head = st.head;
        // `public:` sections
        while head.len() >= 2 && head[0].id() && ["public", "protected", "private"].contains(&head[0].s.as_str()) && head[1].is(":") {
            access = head[0].s.clone();
            head = &head[2..];
        }
        while head.len() >= 2 && head[0].is("Q_OBJECT") {
            head = &head[1..];
        }
        if head.is_empty() || head[0].is("friend") || head[0].is("using") || head[0].is("static_assert") {
            continue;
        }
        if let Some(b) = st.block {
            if let Some(n) = type_of(head, b) {
                t.nested.push(n);
                continue;
            }
        }
        if head[0].is("typedef") {
            continue;
        }
        let doc = lex::doc_of(head);
        for d in lex::typed_decls(head, &MODS) {
            if d.name.is_empty() {
                continue;
            }
            let is_static = d.mods.iter().any(|m| m == "static");
            match d.params {
                Some(ps) => {
                    let pure = d.tail.replace(' ', "").contains("=0");
                    if pure {
                        abstract_count += 1;
                    }
                    if d.ty.is_empty() && d.name.trim_start_matches('~') != t.name {
                        continue;
                    }
                    t.methods.push(Method {
                        name: d.name,
                        params: ps.into_iter().filter(|p| !(p.0.is_empty() && p.1 == "void")).map(|(n, ty, def)| Param { name: n, ty, default: def }).collect(),
                        ret: d.ty.trim_start_matches("virtual ").to_string(),
                        vis: access.clone(),
                        is_static,
                        is_abstract: pure,
                        doc: doc.clone(),
                    });
                }
                None => {
                    let read_only = d.ty.starts_with("const ") || d.mods.iter().any(|m| m == "constexpr");
                    let ty = d.ty.trim_start_matches("const ").to_string();
                    t.fields.push(Field { name: d.name, ty, vis: access.clone(), is_static, read_only, default: d.default, doc: doc.clone(), ..Default::default() });
                }
            }
        }
    }
    // a class whose operations are all pure virtual, without fields, is an interface
    if abstract_count > 0 && t.fields.is_empty() && t.methods.iter().filter(|m| !m.name.starts_with('~') && m.name != t.name).all(|m| m.is_abstract) {
        t.kind = Kind::Interface;
        t.methods.retain(|m| !m.name.starts_with('~'));
    } else if abstract_count > 0 {
        t.is_abstract = true;
    }
    Some(t)
}
