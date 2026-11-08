//! JavaScript: an ES module per type in its package folder (`export class` with JSDoc types,
//! fields set in the constructor, `#name` for private members, enumerations as frozen objects,
//! interfaces as classes whose methods throw). The reverse shares the TypeScript reader
//! (`typescript::parse` without types). After staruml-javascript's ES6 generator.

use super::typescript::{imports, map_type};
use super::*;

fn member_name(n: &str, vis: &str) -> String {
    if vis == "private" { format!("#{}", ident(n)) } else { ident(n) }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    per_type(tree, |t| format!("{}.js", ident(&t.name)), |p, t| {
        let mut w = W::new(o);
        let mut only_supers = t.clone();
        only_supers.fields.clear();
        only_supers.methods.clear();
        only_supers.reals.clear();
        let imps = imports(tree, p, &only_supers, ".js");
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

fn jsdoc(o: &GenOpts, doc: &str, tags: &[String]) -> String {
    let mut s = if o.doc { doc.trim().to_string() } else { String::new() };
    for t in tags {
        if !s.is_empty() {
            s.push('\n');
        }
        s.push_str(t);
    }
    s
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    let name = ident(&t.name);
    if t.kind == Kind::Enum {
        w.doc(o, &jsdoc(o, &t.doc, &["@enum {string}".into()]), DocStyle::Block);
        w.line(&format!("export const {name} = Object.freeze({{"));
        w.inc();
        let lits: Vec<String> = t.literals.iter().map(|l| format!("{}: \"{}\"", ident(l), ident(l))).collect();
        w.line(&lits.join(",\n"));
        w.dec();
        w.line("});");
        return;
    }
    let mut tags = vec![];
    if t.kind == Kind::Interface {
        tags.push("@interface".to_string());
    }
    if t.is_abstract {
        tags.push("@abstract".into());
    }
    for r in &t.reals {
        tags.push(format!("@implements {{{r}}}"));
    }
    w.doc(&GenOpts { doc: true, ..o.clone() }, &jsdoc(o, &t.doc, &tags), DocStyle::Block);
    let ext = t.supers.first().map(|s| format!(" extends {s}")).unwrap_or_default();
    w.line(&format!("export class {name}{ext} {{"));
    w.inc();
    for f in t.fields.iter().filter(|f| f.is_static) {
        w.doc(&GenOpts { doc: true, ..o.clone() }, &jsdoc(o, &f.doc, &[format!("@type {{{}}}", map_type(&f.ty, &f.mult))]), DocStyle::Block);
        let v = if f.default.is_empty() { "null".to_string() } else { f.default.clone() };
        w.line(&format!("static {} = {v};", member_name(&f.name, &f.vis)));
    }
    for f in t.fields.iter().filter(|f| !f.is_static && f.vis == "private") {
        w.line(&format!("{};", member_name(&f.name, &f.vis)));
    }
    let inst: Vec<&Field> = t.fields.iter().filter(|f| !f.is_static).collect();
    if t.kind == Kind::Class {
        w.blank();
        w.line("constructor() {");
        w.inc();
        if !t.supers.is_empty() {
            w.line("super();");
        }
        for f in inst {
            w.doc(&GenOpts { doc: true, ..o.clone() }, &jsdoc(o, &f.doc, &[format!("@type {{{}}}", map_type(&f.ty, &f.mult))]), DocStyle::Block);
            let v = if f.default.is_empty() { "null".to_string() } else { f.default.clone() };
            w.line(&format!("this.{} = {v};", member_name(&f.name, &f.vis)));
        }
        w.dec();
        w.line("}");
    }
    for m in &t.methods {
        w.blank();
        let mut tags: Vec<String> = m.params.iter().map(|p| format!("@param {{{}}} {}", map_type(p.ty.trim_end_matches("[]"), if p.ty.ends_with("[]") { "*" } else { "" }), ident(&p.name))).collect();
        if !m.ret.is_empty() {
            tags.push(format!("@returns {{{}}}", map_type(m.ret.trim_end_matches("[]"), if m.ret.ends_with("[]") { "*" } else { "" })));
        }
        if m.is_abstract {
            tags.push("@abstract".into());
        }
        let d = jsdoc(o, &m.doc, &tags);
        w.doc(&GenOpts { doc: true, ..o.clone() }, &d, DocStyle::Block);
        let ps: Vec<String> = m.params.iter().map(|p| if p.default.is_empty() { ident(&p.name) } else { format!("{} = {}", ident(&p.name), p.default) }).collect();
        w.line(&format!("{}{}({}) {{", if m.is_static { "static " } else { "" }, member_name(&m.name, &m.vis), ps.join(", ")));
        w.inc();
        if m.is_abstract || t.kind == Kind::Interface {
            w.line(&format!("throw new Error(\"{}.{} is not implemented\");", name, ident(&m.name)));
        } else {
            w.line("// TODO implement here");
        }
        w.dec();
        w.line("}");
    }
    w.dec();
    w.line("}");
    for n in &t.nested {
        w.blank();
        write_type(w, n, o);
    }
}
