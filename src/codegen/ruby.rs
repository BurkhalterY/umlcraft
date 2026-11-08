//! Ruby: a file per type (`library_item.rb`) in its package folder, the packages as nested
//! modules; a class with `attr_accessor`s set in `initialize`, class variables for static
//! attributes, `self.` methods for static operations and `private`/`protected` sections; an
//! interface as a module whose methods raise `NotImplementedError`, included by its realizers; an
//! enumeration as a module of constants. The reverse reads modules, classes, `include`s, attribute
//! macros, constants, class variables, `@x` assignments in `initialize` and methods.

use super::*;

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

fn type_note(t: &str, mult: &str) -> String {
    let t = primitive(t, "String", "Integer", "Boolean", "Float");
    if t.is_empty() {
        String::new()
    } else if is_many(mult) {
        format!("Array<{t}>")
    } else {
        t
    }
}

fn initial(f: &Field) -> String {
    if !f.default.is_empty() {
        return f.default.clone();
    }
    if is_many(&f.mult) { "[]".into() } else { "nil".into() }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let paths = tree.type_paths();
    per_type(tree, |t| format!("{}.rb", snake(&ident(&t.name))), |p, t| {
        let mut w = W::new(o);
        let mut reqs = vec![];
        for n in t.supers.iter().chain(t.reals.iter()) {
            if let Some((tn, tp)) = paths.iter().find(|x| &x.0 == n) {
                let rel = super::typescript::relative(&p.path, tp, &snake(&ident(tn)));
                let r = format!("require_relative '{}'", rel.trim_start_matches("./"));
                if !reqs.contains(&r) {
                    reqs.push(r);
                }
            }
        }
        for r in &reqs {
            w.line(r);
        }
        if !reqs.is_empty() {
            w.blank();
        }
        for m in &p.path {
            w.line(&format!("module {}", uc_first(&ident(m))));
            w.inc();
        }
        write_type(&mut w, t, o);
        for _ in &p.path {
            w.dec();
            w.line("end");
        }
        w.out
    })
}

fn write_type(w: &mut W, t: &Ty, o: &GenOpts) {
    w.doc(o, &t.doc, DocStyle::Hash);
    let name = uc_first(&ident(&t.name));
    match t.kind {
        Kind::Enum => {
            w.line(&format!("module {name}"));
            w.inc();
            for (i, l) in t.literals.iter().enumerate() {
                w.line(&format!("{} = {}", ident(l).to_uppercase(), i + 1));
            }
            w.dec();
            w.line("end");
            return;
        }
        Kind::Interface => {
            w.line(&format!("module {name}"));
            w.inc();
            for s in &t.supers {
                w.line(&format!("include {s}"));
            }
            for (i, m) in t.methods.iter().enumerate() {
                if i > 0 || !t.supers.is_empty() {
                    w.blank();
                }
                w.doc(o, &m.doc, DocStyle::Hash);
                w.line(&format!("def {}{}", snake(&ident(&m.name)), params(m)));
                w.inc();
                w.line("raise NotImplementedError");
                w.dec();
                w.line("end");
            }
            w.dec();
            w.line("end");
            return;
        }
        Kind::Class => {}
    }
    let sup = t.supers.first().map(|s| format!(" < {s}")).unwrap_or_default();
    w.line(&format!("class {name}{sup}"));
    w.inc();
    for r in &t.reals {
        w.line(&format!("include {r}"));
    }
    let inst: Vec<&Field> = t.fields.iter().filter(|f| !f.is_static).collect();
    for f in t.fields.iter().filter(|f| f.is_static) {
        if f.read_only {
            w.line(&format!("{} = {}", snake(&ident(&f.name)).to_uppercase(), initial(f)));
        } else {
            w.line(&format!("@@{} = {}", snake(&ident(&f.name)), initial(f)));
        }
    }
    for f in inst.iter().filter(|f| f.vis == "public" || f.vis.is_empty()) {
        let note = type_note(&f.ty, &f.mult);
        let m = if f.read_only { "attr_reader" } else { "attr_accessor" };
        w.line(&if note.is_empty() || !o.doc { format!("{m} :{}", snake(&ident(&f.name))) } else { format!("{m} :{} # {note}", snake(&ident(&f.name))) });
    }
    if !inst.is_empty() {
        w.blank();
        w.line("def initialize");
        w.inc();
        if !t.supers.is_empty() {
            w.line("super");
        }
        for f in &inst {
            w.line(&format!("@{} = {}", snake(&ident(&f.name)), initial(f)));
        }
        w.dec();
        w.line("end");
    }
    let mut section = "public";
    for vis in ["public", "protected", "private"] {
        for m in t.methods.iter().filter(|m| (if m.vis.is_empty() || m.vis == "package" { "public" } else { m.vis.as_str() }) == vis) {
            if vis != section {
                w.blank();
                w.line(vis);
                section = vis;
            }
            w.blank();
            let mut doc = m.doc.trim().to_string();
            for p in &m.params {
                let note = type_note(many(&p.ty).0, many(&p.ty).1);
                doc.push_str(&format!("\n@param {} [{}]", snake(&ident(&p.name)), if note.is_empty() { "Object".into() } else { note }));
            }
            if !m.ret.is_empty() {
                doc.push_str(&format!("\n@return [{}]", type_note(many(&m.ret).0, many(&m.ret).1)));
            }
            w.doc(o, doc.trim(), DocStyle::Hash);
            let pre = if m.is_static { "self." } else { "" };
            w.line(&format!("def {pre}{}{}", snake(&ident(&m.name)), params(m)));
            w.inc();
            if m.is_abstract {
                w.line("raise NotImplementedError");
            } else {
                w.line("# TODO implement here");
            }
            w.dec();
            w.line("end");
        }
    }
    for n in &t.nested {
        w.blank();
        write_type(w, n, o);
    }
    w.dec();
    w.line("end");
}

fn params(m: &Method) -> String {
    if m.params.is_empty() {
        return String::new();
    }
    let ps: Vec<String> = m.params.iter().map(|p| if p.default.is_empty() { snake(&ident(&p.name)) } else { format!("{} = {}", snake(&ident(&p.name)), p.default) }).collect();
    format!("({})", ps.join(", "))
}

// ------------------------------------------------------------------ reverse

enum Frame {
    Module { name: String, consts: Vec<String>, methods: Vec<Method>, has_types: bool, includes: Vec<String>, doc: String },
    Class { ty: Ty, section: String, singleton: bool },
    Def { init: bool },
    Other,
}

/// A line without its comment (outside strings).
fn strip_comment(l: &str) -> (String, Option<String>) {
    let mut quote: Option<char> = None;
    let mut prev = ' ';
    for (i, c) in l.char_indices() {
        match quote {
            Some(q) if c == q && prev != '\\' => quote = None,
            Some(_) => {}
            None if c == '"' || c == '\'' => quote = Some(c),
            None if c == '#' && !l[i..].starts_with("#{") => return (l[..i].to_string(), Some(l[i + 1..].trim().to_string())),
            _ => {}
        }
        prev = c;
    }
    (l.to_string(), None)
}

fn word<'a>(l: &'a str, w: &str) -> Option<&'a str> {
    l.strip_prefix(w).filter(|r| r.is_empty() || r.starts_with([' ', '(', ';', '\t']))
}

fn opens_block(l: &str) -> bool {
    for k in ["if", "unless", "while", "until", "case", "begin", "for"] {
        if word(l, k).is_some() {
            return true;
        }
    }
    // `x = if …`, `foo do |x|`, `loop do`
    let t = l.trim_end();
    t.ends_with(" do") || t == "do" || (t.contains(" do |") && t.ends_with('|')) || [" = if ", " = case ", " = begin", " = unless "].iter().any(|k| t.contains(k))
}

/// `def name(a, b = 1)` / `def self.name a` → (name, static, params, one-line).
fn def_head(rest: &str) -> (String, bool, Vec<Param>, bool) {
    let rest = rest.trim();
    let one_line = rest.contains("; end") || rest.ends_with(";end") || rest.split_once(')').map(|(_, r)| r.trim_start().starts_with('=')).unwrap_or(false) || rest.contains(" = ") && !rest.contains('(');
    let (is_static, rest) = match rest.strip_prefix("self.") {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let end = rest.find(['(', ' ', ';']).unwrap_or(rest.len());
    let name = rest[..end].to_string();
    let mut ps = vec![];
    let after = rest[end..].trim_start();
    let inner = if let Some(a) = after.strip_prefix('(') { a.split(')').next().unwrap_or("").to_string() } else { after.split(';').next().unwrap_or("").to_string() };
    for p in super::python::split_commas(&inner) {
        let (n, d) = match p.split_once('=') {
            Some((n, d)) => (n.trim(), d.trim().to_string()),
            None => (p.trim(), String::new()),
        };
        let n = n.trim_start_matches(['*', '&']).trim_end_matches(':');
        if !n.is_empty() {
            ps.push(Param { name: n.to_string(), default: d, ..Default::default() });
        }
    }
    (name, is_static, ps, one_line)
}

pub fn parse(src: &str, rel: &[String]) -> Parsed {
    let mut stack: Vec<Frame> = vec![];
    let mut units: Vec<Unit> = vec![];
    let mut doc: Vec<String> = vec![];
    let mut heredoc: Option<String> = None;
    let module_path = |stack: &[Frame]| -> Vec<String> {
        let mut p = rel.to_vec();
        p.extend(stack.iter().filter_map(|f| match f {
            Frame::Module { name, .. } => Some(name.clone()),
            _ => None,
        }));
        p
    };
    fn push_type(units: &mut Vec<Unit>, ns: Vec<String>, t: Ty) {
        match units.iter_mut().find(|u| u.ns == ns) {
            Some(u) => u.types.push(t),
            None => units.push(Unit { ns, types: vec![t] }),
        }
    }
    for raw in src.lines() {
        if let Some(h) = &heredoc {
            if raw.trim() == h {
                heredoc = None;
            }
            continue;
        }
        let (code, comment) = strip_comment(raw);
        let l = code.trim();
        if l.is_empty() {
            match comment {
                Some(c) if !c.starts_with('!') && !c.starts_with("frozen_string_literal") => doc.push(c),
                _ => doc.clear(),
            }
            continue;
        }
        if let Some(p) = l.find("<<~").or_else(|| l.find("<<-")) {
            let tag: String = l[p + 3..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !tag.is_empty() {
                heredoc = Some(tag);
            }
        }
        let this_doc = std::mem::take(&mut doc).join("\n");
        if l == "end" || l.starts_with("end ") || l.starts_with("end.") || l == "end)" {
            match stack.pop() {
                Some(Frame::Class { ty, singleton: false, .. }) => {
                    // a class inside a class is nested, else it goes to its module path
                    match stack.iter_mut().rev().find(|f| matches!(f, Frame::Class { singleton: false, .. })) {
                        Some(Frame::Class { ty: outer, .. }) => outer.nested.push(ty),
                        _ => {
                            let ns = module_path(&stack);
                            push_type(&mut units, ns, ty);
                        }
                    }
                }
                Some(Frame::Module { name, consts, methods, has_types, includes, doc }) => {
                    // a module with no types is an interface (methods) or an enumeration (constants)
                    if !has_types && (!methods.is_empty() || !consts.is_empty()) {
                        let mut t = if !methods.is_empty() { Ty::new(Kind::Interface, &name) } else { Ty::new(Kind::Enum, &name) };
                        if t.kind == Kind::Interface {
                            t.methods = methods;
                            t.supers = includes;
                        } else {
                            t.literals = consts;
                        }
                        t.doc = doc;
                        let ns = module_path(&stack);
                        push_type(&mut units, ns, t);
                    }
                }
                _ => {}
            }
            continue;
        }
        if let Some(rest) = word(l, "module") {
            for f in stack.iter_mut() {
                if let Frame::Module { has_types, .. } = f {
                    *has_types = true;
                }
            }
            let name = rest.trim().rsplit("::").next().unwrap_or("").to_string();
            stack.push(Frame::Module { name, consts: vec![], methods: vec![], has_types: false, includes: vec![], doc: this_doc });
            continue;
        }
        if let Some(rest) = word(l, "class") {
            let rest = rest.trim();
            if rest.starts_with("<<") {
                stack.push(Frame::Class { ty: Ty::default(), section: "public".into(), singleton: true });
                continue;
            }
            for f in stack.iter_mut() {
                if let Frame::Module { has_types, .. } = f {
                    *has_types = true;
                }
            }
            let (name, sup) = match rest.split_once('<') {
                Some((n, s)) => (n.trim(), Some(s.trim().split([' ', ';']).next().unwrap_or("").to_string())),
                None => (rest.split([' ', ';']).next().unwrap_or(""), None),
            };
            let mut t = Ty::new(Kind::Class, name.rsplit("::").next().unwrap_or(name));
            t.doc = this_doc;
            if let Some(s) = sup.filter(|s| !s.is_empty() && s != "Object" && !s.starts_with("Struct")) {
                t.supers.push(s.rsplit("::").next().unwrap_or(&s).to_string());
            }
            if l.ends_with("; end") || l.ends_with(";end") {
                let ns = module_path(&stack);
                push_type(&mut units, ns, t);
                continue;
            }
            stack.push(Frame::Class { ty: t, section: "public".into(), singleton: false });
            continue;
        }
        if let Some(rest) = word(l, "def") {
            let (name, is_static, params, one_line) = def_head(rest);
            let in_singleton = matches!(stack.last(), Some(Frame::Class { singleton: true, .. }));
            let class_idx = stack.iter().rposition(|f| matches!(f, Frame::Class { singleton: false, .. }));
            let direct = matches!(stack.last(), Some(Frame::Class { .. }) | Some(Frame::Module { .. }));
            if direct {
                let is_static = is_static || in_singleton;
                match (stack.last_mut(), class_idx) {
                    (Some(Frame::Module { methods, .. }), _) => methods.push(Method { name: name.clone(), params, vis: "public".into(), is_static, is_abstract: true, doc: this_doc, ..Default::default() }),
                    (_, Some(ci)) => {
                        if let Frame::Class { ty, section, .. } = &mut stack[ci] {
                            if name != "initialize" {
                                ty.methods.push(Method { name: name.clone(), params, vis: section.clone(), is_static, doc: this_doc, ..Default::default() });
                            }
                        }
                    }
                    _ => {}
                }
            }
            if !one_line {
                stack.push(Frame::Def { init: name == "initialize" && direct });
            }
            continue;
        }
        if opens_block(l) {
            stack.push(Frame::Other);
            continue;
        }
        match stack.last_mut() {
            Some(Frame::Class { ty, section, singleton: false }) => {
                if ["private", "protected", "public"].contains(&l) {
                    *section = l.to_string();
                } else if let Some(r) = l.strip_prefix("attr_accessor").or_else(|| l.strip_prefix("attr_reader")).or_else(|| l.strip_prefix("attr_writer")) {
                    let read_only = l.starts_with("attr_reader");
                    for n in r.split(',') {
                        let n = n.trim().trim_start_matches(':').trim();
                        if !n.is_empty() && !ty.fields.iter().any(|f| f.name == n) {
                            ty.fields.push(Field { name: n.to_string(), vis: section.clone(), read_only, doc: comment.clone().unwrap_or_default(), ..Default::default() });
                        }
                    }
                } else if let Some(r) = word(l, "include") {
                    for n in r.split(',') {
                        ty.reals.push(n.trim().rsplit("::").next().unwrap_or("").to_string());
                    }
                } else if let Some(r) = l.strip_prefix("@@") {
                    if let Some((n, v)) = r.split_once('=') {
                        ty.fields.push(Field { name: n.trim().to_string(), vis: "private".into(), is_static: true, default: v.trim().to_string(), ..Default::default() });
                    }
                } else if let Some((n, v)) = l.split_once('=') {
                    let n = n.trim();
                    if n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && n.chars().all(|c| c.is_alphanumeric() || c == '_') && !v.starts_with('=') {
                        ty.fields.push(Field { name: n.to_string(), vis: "public".into(), is_static: true, read_only: true, default: v.trim().to_string(), ..Default::default() });
                    }
                }
            }
            Some(Frame::Module { consts, includes, .. }) => {
                if let Some(r) = word(l, "include") {
                    includes.extend(r.split(',').map(|n| n.trim().rsplit("::").next().unwrap_or("").to_string()));
                } else if let Some((n, v)) = l.split_once('=') {
                    let n = n.trim();
                    if n.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && n.chars().all(|c| c.is_alphanumeric() || c == '_') && !v.starts_with('=') {
                        consts.push(n.to_string());
                    }
                }
            }
            _ => {}
        }
        // `@x = …` in `initialize`, at any depth
        if let Some(Frame::Def { init: true }) = stack.iter().rev().find(|f| matches!(f, Frame::Def { .. })) {
            if let Some(r) = l.strip_prefix('@').filter(|r| !r.starts_with('@')) {
                if let Some((n, v)) = r.split_once('=') {
                    let n = n.trim();
                    if !v.starts_with('=') && n.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        if let Some(Frame::Class { ty, .. }) = stack.iter_mut().rev().find(|f| matches!(f, Frame::Class { singleton: false, .. })) {
                            if !ty.fields.iter().any(|f| f.name == n) {
                                ty.fields.push(Field { name: n.to_string(), vis: "private".into(), default: v.trim().to_string(), ..Default::default() });
                            }
                        }
                    }
                }
            }
        }
    }
    let errors = !stack.is_empty() || heredoc.is_some();
    // unclosed frames still give their classes
    while let Some(f) = stack.pop() {
        if let Frame::Class { ty, singleton: false, .. } = f {
            let ns = module_path(&stack);
            push_type(&mut units, ns, ty);
        }
    }
    if units.is_empty() {
        units.push(Unit { ns: rel.to_vec(), types: vec![] });
    }
    Parsed { units, errors }
}
