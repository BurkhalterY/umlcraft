//! Dart: a library per type (`library_item.dart`) in its package folder with relative imports;
//! private members as `_name`, `late` fields without a value, `UnimplementedError` bodies,
//! interfaces as `abstract interface class`, enumerations as `enum`. The reverse reads classes,
//! mixins, enums, `extends`/`with`/`implements`, fields, getters, methods and their positional and
//! named parameters.

use super::lex::{self, Cfg, K, T};
use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = if t.is_empty() { "dynamic".to_string() } else { primitive(t, "String", "int", "bool", "double") };
    let t = match t.as_str() {
        "boolean" => "bool".to_string(),
        "long" | "Integer" => "int".into(),
        "float" => "double".into(),
        "string" => "String".into(),
        _ => t,
    };
    if is_many(mult) {
        format!("List<{t}>")
    } else if is_optional(mult) && t != "dynamic" {
        format!("{t}?")
    } else {
        t
    }
}

fn member_name(n: &str, vis: &str) -> String {
    if vis == "private" { format!("_{}", lc_first(&ident(n))) } else { lc_first(&ident(n)) }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let paths = tree.type_paths();
    per_type(tree, |t| format!("{}.dart", snake(&ident(&t.name))), |p, t| {
        let mut w = W::new(o);
        let mut imps = vec![];
        let refs = t.supers.iter().chain(t.reals.iter()).cloned().chain(t.fields.iter().map(|f| f.ty.clone())).chain(t.methods.iter().flat_map(|m| m.params.iter().map(|p| p.ty.trim_end_matches("[]").to_string()).chain([m.ret.trim_end_matches("[]").to_string()])));
        for n in refs {
            if let Some((tn, tp)) = paths.iter().find(|x| x.0 == n && x.0 != t.name) {
                let rel = super::typescript::relative(&p.path, tp, &snake(&ident(tn)));
                let i = format!("import '{}.dart';", rel.trim_start_matches("./"));
                if !imps.contains(&i) {
                    imps.push(i);
                }
            }
        }
        for i in &imps {
            w.line(i);
        }
        if !imps.is_empty() {
            w.blank();
        }
        write_type(&mut w, t, o);
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    w.doc(o, &t.doc, DocStyle::Slashes);
    let name = ident(&t.name);
    let gen = if t.generics.is_empty() { String::new() } else { format!("<{}>", t.generics.join(", ")) };
    if t.kind == Kind::Enum {
        w.line(&format!("enum {name} {{"));
        w.inc();
        let lits: Vec<String> = t.literals.iter().map(|l| ident(l)).collect();
        w.line(&lits.join(",\n"));
        w.dec();
        w.line("}");
        return;
    }
    let interface = t.kind == Kind::Interface;
    let mut head = if interface {
        format!("abstract interface class {name}{gen}")
    } else {
        let abs = t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
        format!("{}class {name}{gen}", if abs { "abstract " } else { "" })
    };
    if interface {
        if !t.supers.is_empty() {
            head.push_str(&format!(" implements {}", t.supers.join(", ")));
        }
    } else {
        if let Some(s) = t.supers.first() {
            head.push_str(&format!(" extends {s}"));
        }
        if !t.reals.is_empty() {
            head.push_str(&format!(" implements {}", t.reals.join(", ")));
        }
    }
    w.line(&format!("{head} {{"));
    w.inc();
    for f in &t.fields {
        w.doc(o, &f.doc, DocStyle::Slashes);
        let ty = map_type(&f.ty, &f.mult);
        let mut s = String::new();
        if f.is_static {
            s.push_str("static ");
        }
        if f.read_only {
            s.push_str(if f.is_static && !f.default.is_empty() { "const " } else { "final " });
        }
        let n = member_name(&f.name, &f.vis);
        if interface {
            w.line(&format!("{ty} get {n};"));
            continue;
        }
        if !f.default.is_empty() {
            s.push_str(&format!("{ty} {n} = {};", f.default));
        } else if ty.ends_with('?') || ty == "dynamic" {
            s.push_str(&format!("{ty} {n};"));
        } else if is_many(&f.mult) {
            s.push_str(&format!("{ty} {n} = [];"));
        } else {
            s = format!("late {s}{ty} {n};");
        }
        w.line(&s);
    }
    for m in &t.methods {
        w.blank();
        w.doc(o, &m.doc, DocStyle::Slashes);
        let ps: Vec<String> = m.params.iter().map(|p| {
            let (pt, pm) = many(&p.ty);
            format!("{} {}", map_type(pt, pm), lc_first(&ident(&p.name)))
        }).collect();
        let (rt, rm) = many(&m.ret);
        let ret = if m.ret.is_empty() { "void".to_string() } else { map_type(rt, rm) };
        let s = format!("{}{ret} {}({})", if m.is_static { "static " } else { "" }, member_name(&m.name, &m.vis), ps.join(", "));
        if interface || m.is_abstract {
            w.line(&format!("{s};"));
        } else {
            w.line(&format!("{s} {{"));
            w.inc();
            w.line("// TODO implement here");
            w.line("throw UnimplementedError();");
            w.dec();
            w.line("}");
        }
    }
    w.dec();
    w.line("}");
    for n in &t.nested {
        w.blank();
        write_type(w, n, o);
    }
}

// ------------------------------------------------------------------ reverse

const MODS: [&str; 10] = ["static", "final", "const", "late", "external", "abstract", "covariant", "factory", "required", "var"];
const CLASS_MODS: [&str; 7] = ["abstract", "base", "interface", "final", "sealed", "mixin", "augment"];

fn dart_name(n: &str) -> (String, String) {
    match n.strip_prefix('_') {
        Some(s) => (s.to_string(), "private".into()),
        None => (n.to_string(), "public".into()),
    }
}

pub fn parse(src: &str, rel: &[String]) -> Parsed {
    let t = lex::lex(src, Cfg { sq_strings: true, ..Default::default() });
    let mut types = vec![];
    for st in lex::statements(&t, false) {
        let h = lex::skip_annotations(st.head);
        let Some(b) = st.block else { continue };
        let Some(k) = h.iter().position(|x| x.k == K::Id && ["class", "mixin", "enum"].contains(&x.s.as_str())) else { continue };
        if !h[..k].iter().all(|x| x.id() && CLASS_MODS.contains(&x.s.as_str())) {
            continue;
        }
        // `mixin class`: the kind word is the last one
        let k = if h[k].is("mixin") && h.get(k + 1).map(|x| x.is("class")).unwrap_or(false) { k + 1 } else { k };
        let mods: Vec<String> = h[..k].iter().map(|x| x.s.clone()).collect();
        let Some(raw) = h.get(k + 1).filter(|x| x.id()) else { continue };
        let (name, vis) = dart_name(&raw.s);
        let kind = if h[k].is("enum") {
            Kind::Enum
        } else if mods.iter().any(|m| m == "interface") {
            Kind::Interface
        } else {
            Kind::Class
        };
        let mut ty = Ty::new(kind, &name);
        ty.vis = vis;
        ty.doc = lex::doc_of(st.head);
        ty.is_abstract = mods.iter().any(|m| m == "abstract");
        if h[k].is("mixin") {
            ty.stereotype = "mixin".into();
        }
        let (g, i) = lex::generic_params(h, k + 2);
        ty.generics = g.iter().map(|p| p.0.clone()).collect();
        ty.gen_bounds = g.into_iter().map(|p| (p.1, p.2)).collect();
        let (s, r) = super::java::super_lists(&h[i.min(h.len())..], &["extends", "with", "on"], &["implements"], &[]);
        ty.supers = s;
        ty.reals = r;
        if kind == Kind::Interface {
            ty.supers.append(&mut ty.reals);
        }
        let mut body = b;
        if kind == Kind::Enum {
            let end = lex::find_top(body, ";", false).unwrap_or(body.len());
            for p in lex::commas(&body[..end]) {
                if let Some(x) = lex::skip_annotations(p).first().filter(|x| x.id()) {
                    ty.literals.push(x.s.clone());
                }
            }
            body = &body[(end + 1).min(body.len())..];
        }
        for m in lex::statements(body, false) {
            member(&mut ty, &name, m.head, m.block.is_some());
        }
        // an abstract class of abstract methods only is an interface
        if ty.kind == Kind::Class && ty.is_abstract && ty.fields.is_empty() && !ty.methods.is_empty() && ty.methods.iter().all(|m| m.is_abstract) {
            ty.kind = Kind::Interface;
            ty.is_abstract = false;
            ty.supers.append(&mut ty.reals);
        }
        types.push(ty);
    }
    Parsed { units: vec![Unit { ns: rel.to_vec(), types, ..Default::default() }], errors: !lex::balanced(&t) }
}

fn member(t: &mut Ty, class: &str, head: &[T], has_body: bool) {
    let h = lex::skip_annotations(head);
    if h.is_empty() || h[0].is("factory") {
        return;
    }
    let doc = lex::doc_of(head);
    let arrow = h.iter().position(|x| x.is("=>"));
    let decl = &h[..arrow.unwrap_or(h.len())];
    // a getter `Type get name` reads as a read-only field; setters are skipped
    if let Some(g) = decl.iter().position(|x| x.is("get")) {
        if let Some(n) = decl.get(g + 1).filter(|x| x.id()) {
            let (name, vis) = dart_name(&n.s);
            let mut i = 0;
            let is_static = decl.first().map(|x| x.is("static")).unwrap_or(false);
            if is_static {
                i = 1;
            }
            if !t.fields.iter().any(|f| f.name == name) {
                t.fields.push(Field { name, vis, ty: lex::join(&decl[i..g]), is_static, read_only: true, doc, ..Default::default() });
            }
            return;
        }
    }
    if decl.iter().any(|x| x.is("set")) || decl.iter().any(|x| x.is("operator")) {
        return;
    }
    for d in lex::typed_decls(decl, &MODS) {
        if d.name.is_empty() {
            continue;
        }
        let (name, vis) = dart_name(&d.name);
        let is_static = d.mods.iter().any(|m| m == "static");
        match d.params {
            Some(ps) => {
                // constructors (`Name(…)`, `Name.named(…)`) are skipped
                if d.ty.is_empty() || d.ty.ends_with('.') || d.name == class {
                    continue;
                }
                let abs = !has_body && arrow.is_none() && !d.mods.iter().any(|m| m == "external");
                t.methods.push(Method { name, params: ps.into_iter().map(|(n, ty, def)| Param { name: n, ty, default: def }).collect(), ret: d.ty, vis, is_static, is_abstract: abs, doc: doc.clone() , ..Default::default() });
            }
            None => {
                let read_only = d.mods.iter().any(|m| m == "final" || m == "const");
                t.fields.push(Field { name, ty: d.ty, vis, is_static, read_only, default: d.default, doc: doc.clone(), ..Default::default() });
            }
        }
    }
}
