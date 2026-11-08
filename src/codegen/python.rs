//! Python: a module per class (`library_item.py`) in its package folder with an `__init__.py`,
//! type hints, `ABC`/`@abstractmethod` for interfaces and abstract operations, `Enum` for
//! enumerations, private members as `__name` and protected ones as `_name`; and the reverse of
//! `.py`/`.pyi` files by indentation (classes, bases, class attributes, `self.x` assignments in
//! `__init__`, methods with their hints and decorators). After staruml-python's generator.

use super::*;

pub fn map_type(t: &str, mult: &str) -> String {
    let t = primitive(t, "str", "int", "bool", "float");
    if is_many(mult) {
        format!("list[{}]", if t.is_empty() { "object" } else { &t })
    } else if is_optional(mult) && !t.is_empty() {
        format!("{t} | None")
    } else {
        t
    }
}

fn hinted(n: &str, t: &str) -> String {
    if t.is_empty() { n.to_string() } else { format!("{n}: {t}") }
}

fn member_name(n: &str, vis: &str) -> String {
    let n = ident(n);
    match vis {
        "private" => format!("__{n}"),
        "protected" => format!("_{n}"),
        _ => n,
    }
}

fn default_of(t: &str) -> &'static str {
    match t {
        "int" => "0",
        "float" => "0.0",
        "bool" => "False",
        "str" => "\"\"",
        _ if t.starts_with("list[") => "[]",
        _ => "None",
    }
}

fn many(t: &str) -> (&str, &'static str) {
    match t.strip_suffix("[]") {
        Some(e) => (e, "*"),
        None => (t, ""),
    }
}

/// A class's module name: snake_case (python.gen.lowerCase, PEP 8), else the class name.
fn module_name(n: &str, o: &GenOpts) -> String {
    if o.lower_case { snake(&ident(n)) } else { ident(n) }
}

pub fn generate(tree: &Pkg, o: &GenOpts) -> Vec<OutFile> {
    let paths = tree.type_paths();
    let mut out = per_type(tree, |t| format!("{}.py", module_name(&t.name, o)), |p, t| {
        let mut w = W::new(o);
        // the interpreter line (python.gen.installPath)
        if !o.install_path.trim().is_empty() {
            w.line(o.install_path.trim());
            w.blank();
        }
        let interface = t.kind == Kind::Interface;
        let abs = interface || t.is_abstract || t.methods.iter().any(|m| m.is_abstract);
        if abs {
            w.line("from abc import ABC, abstractmethod");
        }
        if t.kind == Kind::Enum {
            w.line("from enum import Enum");
        }
        for n in t.supers.iter().chain(t.reals.iter()) {
            if let Some((tn, tp)) = paths.iter().find(|x| &x.0 == n) {
                let module: Vec<String> = tp.iter().map(|s| ident(s)).chain([module_name(tn, o)]).collect();
                w.line(&format!("from {} import {}", module.join("."), ident(tn)));
            }
        }
        let _ = p;
        w.blank();
        w.blank();
        write_class(&mut w, t, o, abs);
        w.out
    });
    // a package folder is a Python package
    for p in tree.all() {
        if !p.path.is_empty() {
            let dir: PathBuf = p.path.iter().collect();
            out.push((dir.join("__init__.py"), String::new()));
        }
    }
    out
}

fn write_class(w: &mut W, t: &Ty, o: &GenOpts, abs: bool) {
    let mut bases: Vec<String> = t.supers.iter().chain(t.reals.iter()).map(|b| ident(b)).collect();
    if t.kind == Kind::Enum {
        bases.push("Enum".into());
    } else if abs && bases.is_empty() {
        bases.push("ABC".into());
    }
    let b = if bases.is_empty() { String::new() } else { format!("({})", bases.join(", ")) };
    w.line(&format!("class {}{b}:", ident(&t.name)));
    w.inc();
    w.doc(o, &t.doc, DocStyle::DocString);
    let mut empty = !o.doc || t.doc.trim().is_empty();
    if t.kind == Kind::Enum {
        for (i, l) in t.literals.iter().enumerate() {
            w.line(&format!("{} = {}", ident(l), i + 1));
            empty = false;
        }
    }
    for f in t.fields.iter().filter(|f| f.is_static) {
        let ty = map_type(&f.ty, &f.mult);
        let v = if f.default.is_empty() { default_of(&ty).to_string() } else { f.default.clone() };
        w.line(&format!("{} = {v}", hinted(&member_name(&f.name, &f.vis), &ty)));
        empty = false;
    }
    let inst: Vec<&Field> = t.fields.iter().filter(|f| !f.is_static).collect();
    if !inst.is_empty() && t.kind != Kind::Interface {
        w.blank();
        w.line("def __init__(self):");
        w.inc();
        for f in inst {
            let ty = map_type(&f.ty, &f.mult);
            let v = if f.default.is_empty() { default_of(&ty).to_string() } else { f.default.clone() };
            w.line(&format!("self.{} = {v}", hinted(&member_name(&f.name, &f.vis), &ty)));
            if o.doc && !f.doc.trim().is_empty() {
                w.line(&format!("\"\"\"{}\"\"\"", f.doc.trim()));
            }
        }
        w.dec();
        empty = false;
    }
    for m in &t.methods {
        w.blank();
        let abstract_m = m.is_abstract || t.kind == Kind::Interface;
        if m.is_static {
            w.line("@staticmethod");
        }
        if abstract_m {
            w.line("@abstractmethod");
        }
        let mut ps: Vec<String> = if m.is_static { vec![] } else { vec!["self".into()] };
        for p in &m.params {
            let (pt, pm) = many(&p.ty);
            let mut s = hinted(&ident(&p.name), &map_type(pt, pm));
            if !p.default.is_empty() {
                s.push_str(&format!(" = {}", p.default));
            }
            ps.push(s);
        }
        let (rt, rm) = many(&m.ret);
        let ret = if m.ret.is_empty() { " -> None".to_string() } else { format!(" -> {}", map_type(rt, rm)) };
        w.line(&format!("def {}({}){ret}:", member_name(&m.name, &m.vis), ps.join(", ")));
        w.inc();
        w.doc(o, &m.doc, DocStyle::DocString);
        w.line("pass");
        w.dec();
        empty = false;
    }
    for n in &t.nested {
        w.blank();
        write_class(w, n, o, n.kind == Kind::Interface || n.is_abstract);
        empty = false;
    }
    if empty {
        w.line("pass");
    }
    w.dec();
}

// ------------------------------------------------------------------ reverse

struct Line {
    indent: usize,
    text: String,
}

/// Logical lines: comments dropped, bracketed continuations and `\` joined, blank lines skipped;
/// a triple-quoted string stays one line.
fn logical_lines(src: &str) -> (Vec<Line>, bool) {
    let mut out = vec![];
    let mut cur = String::new();
    let mut indent = 0;
    let mut depth = 0i32;
    let mut triple: Option<&str> = None;
    for raw in src.lines() {
        if cur.is_empty() && triple.is_none() {
            indent = raw.len() - raw.trim_start().len();
        }
        let mut chars = raw.chars().peekable();
        let mut line = String::new();
        let mut quote: Option<char> = None;
        while let Some(c) = chars.next() {
            if let Some(q) = triple {
                line.push(c);
                if line.ends_with(q) {
                    triple = None;
                }
                continue;
            }
            if let Some(q) = quote {
                line.push(c);
                if c == '\\' {
                    if let Some(n) = chars.next() {
                        line.push(n);
                    }
                } else if c == q {
                    quote = None;
                }
                continue;
            }
            match c {
                '#' => break,
                '"' | '\'' => {
                    line.push(c);
                    let rest: String = chars.clone().take(2).collect();
                    if rest == format!("{c}{c}") {
                        line.push(c);
                        line.push(c);
                        chars.next();
                        chars.next();
                        triple = Some(if c == '"' { "\"\"\"" } else { "'''" });
                        // the opening quotes are not the closing ones
                        let marker = line.len();
                        let _ = marker;
                    } else {
                        quote = Some(c);
                    }
                }
                '(' | '[' | '{' => {
                    depth += 1;
                    line.push(c);
                }
                ')' | ']' | '}' => {
                    depth -= 1;
                    line.push(c);
                }
                _ => line.push(c),
            }
        }
        let trimmed = line.trim();
        let cont = trimmed.ends_with('\\');
        cur.push_str(trimmed.trim_end_matches('\\'));
        if triple.is_some() {
            cur.push('\n');
            continue;
        }
        if depth > 0 || cont {
            cur.push(' ');
            continue;
        }
        if !cur.trim().is_empty() {
            out.push(Line { indent, text: cur.trim().to_string() });
        }
        cur.clear();
    }
    let broken = depth != 0 || triple.is_some();
    if !cur.trim().is_empty() {
        out.push(Line { indent, text: cur.trim().to_string() });
    }
    (out, broken)
}

#[derive(PartialEq)]
enum Ctx {
    Class(usize),
    Def(usize, bool),
    Other,
}

/// The name, visibility and whether a Python name is a dunder.
fn py_name(n: &str) -> (String, String) {
    if n.starts_with("__") && n.ends_with("__") {
        (n.to_string(), "public".into())
    } else if let Some(s) = n.strip_prefix("__") {
        (s.to_string(), "private".into())
    } else if let Some(s) = n.strip_prefix('_') {
        (s.to_string(), "protected".into())
    } else {
        (n.to_string(), "public".into())
    }
}

/// `name: T = v`, `name = v` or `name: T` → (name, type, value, annotated).
fn assignment(s: &str) -> Option<(String, String, String, bool)> {
    let (left, value) = match find_eq(s) {
        Some(i) => (s[..i].trim(), s[i + 1..].trim().to_string()),
        None => (s.trim(), String::new()),
    };
    let (name, ty, annotated) = match left.find(':') {
        Some(c) => (left[..c].trim(), left[c + 1..].trim().to_string(), true),
        None if !value.is_empty() => (left, String::new(), false),
        None => return None,
    };
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') || name.is_empty() {
        return None;
    }
    Some((name.to_string(), ty, value, annotated))
}

/// The first `=` that is an assignment (not `==`, `<=`, `>=`, `!=`), outside brackets.
fn find_eq(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0;
    for i in 0..b.len() {
        match b[i] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'=' if depth == 0 => {
                let prev = if i > 0 { b[i - 1] } else { b' ' };
                let next = b.get(i + 1).copied().unwrap_or(b' ');
                if next != b'=' && !b"=<>!+-*/%&|^:".contains(&prev) {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split at top-level commas.
pub fn split_commas(s: &str) -> Vec<String> {
    let mut out = vec![];
    let mut depth = 0;
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    for c in s.chars() {
        if let Some(q) = quote {
            cur.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                quote = Some(c);
                cur.push(c);
            }
            '(' | '[' | '{' | '<' => {
                depth += 1;
                cur.push(c);
            }
            ')' | ']' | '}' | '>' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out.retain(|x| !x.is_empty());
    out
}

fn docstring(s: &str) -> Option<String> {
    for q in ["\"\"\"", "'''"] {
        if s.starts_with(q) && s.ends_with(q) && s.len() >= 6 {
            return Some(s[3..s.len() - 3].trim().to_string());
        }
    }
    for q in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(q) && s.ends_with(q) {
            return Some(s[1..s.len() - 1].to_string());
        }
    }
    None
}

/// `class Box[T, U: Item = Book]` and `def f[T](…)` (PEP 695): the name before the brackets and
/// the type parameters as (name, bound, default).
fn pep695(head: &str) -> (&str, Vec<(String, String, String)>) {
    let head = head.trim();
    let paren = head.find('(').unwrap_or(head.len());
    let Some(open) = head[..paren].find('[') else { return (head[..paren].trim(), vec![]) };
    let close = head[..paren].rfind(']').unwrap_or(paren);
    let ps = split_commas(&head[open + 1..close.max(open + 1)])
        .into_iter()
        .map(|p| {
            let (left, default) = match find_eq(&p) {
                Some(i) => (p[..i].trim().to_string(), p[i + 1..].trim().to_string()),
                None => (p.trim().to_string(), String::new()),
            };
            let (n, bound) = match left.split_once(':') {
                Some((n, b)) => (n.trim().to_string(), b.trim().to_string()),
                None => (left.clone(), String::new()),
            };
            (n.trim_start_matches('*').to_string(), bound, default)
        })
        .collect();
    (head[..open].trim(), ps)
}

/// `import a.b`, `import a.b as c`, `from a.b import C, D as E` and `from .m import C`: the
/// imported names, with module paths (a relative import starts from the file's folder).
fn imports_of(text: &str, rel: &[String]) -> Vec<Import> {
    let module_path = |m: &str| -> Vec<String> {
        let dots = m.chars().take_while(|c| *c == '.').count();
        let mut p: Vec<String> = if dots > 0 { rel[..rel.len().saturating_sub(dots - 1)].to_vec() } else { vec![] };
        p.extend(m.trim_start_matches('.').split('.').filter(|s| !s.is_empty()).map(|s| s.to_string()));
        p
    };
    let alias_of = |s: &str| -> (String, String) {
        match s.split_once(" as ") {
            Some((n, a)) => (n.trim().to_string(), a.trim().to_string()),
            None => (s.trim().to_string(), s.trim().rsplit('.').next().unwrap_or("").to_string()),
        }
    };
    let mut out = vec![];
    if let Some(rest) = text.strip_prefix("from ") {
        let Some((m, names)) = rest.split_once(" import ") else { return out };
        let base = module_path(m.trim());
        for n in split_commas(names.trim().trim_start_matches('(').trim_end_matches(')')) {
            if n == "*" {
                out.push(Import { path: base.clone(), wildcard: true, module: true, ..Default::default() });
                continue;
            }
            let (real, alias) = alias_of(&n);
            let mut path = base.clone();
            path.push(real);
            out.push(Import { path, alias, wildcard: false, module: true });
        }
    } else if let Some(rest) = text.strip_prefix("import ") {
        for n in split_commas(rest) {
            let (real, alias) = alias_of(&n);
            let path = module_path(&real);
            // `import a.b` is known as `a`, then `a.b.C`; `import a.b as c` as `c`
            let alias = if n.contains(" as ") { alias } else { path.first().cloned().unwrap_or_default() };
            let path = if n.contains(" as ") { path } else { path.into_iter().take(1).collect() };
            out.push(Import { path, alias, wildcard: false, module: true });
        }
    }
    out
}

pub fn parse(src: &str, rel: &[String], o: &RevOpts) -> Parsed {
    let (lines, broken) = logical_lines(src);
    let mut imports = vec![];
    let mut types: Vec<(Ty, Option<usize>)> = vec![];
    let mut stack: Vec<(usize, Ctx)> = vec![];
    let mut decorators: Vec<String> = vec![];
    let mut just_opened = false;
    for l in &lines {
        while stack.last().map(|s| s.0 >= l.indent).unwrap_or(false) {
            stack.pop();
        }
        let text = l.text.as_str();
        let top_class = match stack.last() {
            Some((_, Ctx::Class(i))) => Some(*i),
            _ => None,
        };
        if let Some(d) = docstring(text) {
            if just_opened {
                match stack.last() {
                    Some((_, Ctx::Class(i))) => types[*i].0.doc = d,
                    Some((_, Ctx::Def(i, _))) => {
                        if let Some(m) = types[*i].0.methods.last_mut() {
                            m.doc = d;
                        }
                    }
                    _ => {}
                }
            }
            just_opened = false;
            continue;
        }
        just_opened = false;
        if text.starts_with("import ") || (text.starts_with("from ") && text.contains(" import ")) {
            imports.extend(imports_of(text, rel));
            continue;
        }
        if let Some(d) = text.strip_prefix('@') {
            decorators.push(d.split('(').next().unwrap_or("").trim().to_string());
            continue;
        }
        if let Some(rest) = text.strip_prefix("class ") {
            let head = rest.trim_end_matches(':').trim();
            let (name, tps) = pep695(head);
            let bases = match head.find('(') {
                Some(p) => head[p + 1..].trim_end_matches(')').to_string(),
                None => String::new(),
            };
            let mut t = Ty::new(Kind::Class, name);
            t.generics = tps.iter().map(|p| p.0.clone()).collect();
            t.gen_bounds = tps.into_iter().map(|p| (p.1, p.2)).collect();
            let mut abc = false;
            for b in split_commas(&bases) {
                if b.contains('=') {
                    abc |= b.contains("ABCMeta");
                    continue;
                }
                let simple = base_name(&b);
                match simple.as_str() {
                    "Enum" | "IntEnum" | "StrEnum" | "Flag" | "IntFlag" => t.kind = Kind::Enum,
                    "Protocol" => {
                        t.kind = Kind::Interface;
                        if let Some((_, inner)) = b.split_once('[') {
                            if t.generics.is_empty() {
                                t.generics = split_commas(inner.trim_end_matches(']'));
                            }
                        }
                    }
                    "ABC" => abc = true,
                    "object" => {}
                    "Generic" => {
                        let inner = b.split_once('[').map(|x| x.1.trim_end_matches(']')).unwrap_or("");
                        if t.generics.is_empty() {
                            t.generics = split_commas(inner);
                        }
                    }
                    _ => t.supers.push(b.split('[').next().unwrap_or(&b).trim().to_string()),
                }
            }
            t.is_abstract = abc;
            let (n, vis) = py_name(name);
            t.name = n;
            t.vis = vis;
            types.push((t, top_class));
            stack.push((l.indent, Ctx::Class(types.len() - 1)));
            decorators.clear();
            just_opened = true;
            continue;
        }
        let def = text.strip_prefix("async def ").or_else(|| text.strip_prefix("def "));
        if let Some(rest) = def {
            let decs = std::mem::take(&mut decorators);
            let Some(ci) = top_class else {
                stack.push((l.indent, Ctx::Other));
                continue;
            };
            let p = rest.find('(').unwrap_or(rest.len());
            let (raw_name, mtps) = pep695(&rest[..p]);
            let mgen: Vec<String> = mtps.into_iter().map(|x| x.0).collect();
            let close = rest.rfind(')').unwrap_or(rest.len());
            let params_text = if p < close { &rest[p + 1..close] } else { "" };
            // the return hint runs to the `:` that opens the body (`-> T: ...` on one line)
            let ret = rest[close.min(rest.len())..].split_once("->").map(|(_, r)| {
                let mut depth = 0;
                let end = r.char_indices().find(|(_, c)| {
                    match c {
                        '[' | '(' => depth += 1,
                        ']' | ')' => depth -= 1,
                        _ => {}
                    }
                    *c == ':' && depth == 0
                });
                r[..end.map(|e| e.0).unwrap_or(r.len())].trim().to_string()
            });
            let ret = ret.unwrap_or_default();
            let is_static = decs.iter().any(|d| d == "staticmethod" || d == "classmethod");
            let is_init = raw_name == "__init__";
            stack.push((l.indent, Ctx::Def(ci, is_init)));
            if decs.iter().any(|d| d == "property" || d.ends_with(".setter")) {
                // a property reads as a field
                let (n, vis) = py_name(raw_name);
                if !types[ci].0.fields.iter().any(|f| f.name == n) {
                    types[ci].0.fields.push(Field { name: n, ty: ret, vis, read_only: true, ..Default::default() });
                }
                continue;
            }
            // magic methods (`__repr__`, `__init__`…) are skipped unless asked (python.rev.skipMagicMethods)
            if raw_name.starts_with("__") && raw_name.ends_with("__") && o.skip_magic {
                continue;
            }
            let mut params = vec![];
            for (k, ps) in split_commas(params_text).into_iter().enumerate() {
                // the self parameter is skipped unless asked (python.rev.skipSelfParam)
                if k == 0 && !decs.iter().any(|d| d == "staticmethod") && (ps == "self" || ps == "cls") && o.skip_self {
                    continue;
                }
                if ps == "*" || ps == "/" {
                    continue;
                }
                let (left, def) = match find_eq(&ps) {
                    Some(i) => (ps[..i].trim().to_string(), ps[i + 1..].trim().to_string()),
                    None => (ps.clone(), String::new()),
                };
                let (n, ty) = match left.split_once(':') {
                    Some((n, t)) => (n.trim().to_string(), t.trim().to_string()),
                    None => (left.trim().to_string(), String::new()),
                };
                params.push(Param { name: n.trim_start_matches('*').to_string(), ty, default: def });
            }
            let (n, vis) = py_name(raw_name);
            types[ci].0.methods.push(Method { name: n, params, ret, vis, is_static, is_abstract: decs.iter().any(|d| d.ends_with("abstractmethod")), generics: mgen, ..Default::default() });
            just_opened = true;
            continue;
        }
        if text.ends_with(':') {
            stack.push((l.indent, Ctx::Other));
            decorators.clear();
            continue;
        }
        if let Some(ci) = top_class {
            if let Some((name, ty, value, annotated)) = assignment(text) {
                if types[ci].0.kind == Kind::Enum {
                    types[ci].0.literals.push(name);
                } else {
                    let (n, vis) = py_name(&name);
                    // an annotated class attribute is an instance field (dataclass style); a plain one,
                    // or one hinted `ClassVar[…]`, is static; `Final[…]` is read-only
                    let head = ty.split('[').next().unwrap_or("").rsplit('.').next().unwrap_or("").trim().to_string();
                    let is_static = !annotated || head == "ClassVar";
                    types[ci].0.fields.push(Field { name: n, ty, vis, is_static, read_only: head == "Final", default: value, ..Default::default() });
                }
            }
            continue;
        }
        // `self.x = …` anywhere in `__init__`
        let init = stack.iter().rev().find_map(|s| match s.1 {
            Ctx::Def(ci, true) => Some(ci),
            Ctx::Class(_) | Ctx::Def(_, false) => Some(usize::MAX),
            _ => None,
        });
        if let Some(ci) = init.filter(|c| *c != usize::MAX) {
            if let Some(rest) = text.strip_prefix("self.") {
                if let Some((name, ty, value, _)) = assignment(rest) {
                    let (n, vis) = py_name(&name);
                    if !name.contains('.') && !types[ci].0.fields.iter().any(|f| f.name == n) {
                        types[ci].0.fields.push(Field { name: n, ty, vis, default: value, ..Default::default() });
                    }
                }
            }
        }
    }
    // an ABC without fields whose operations are all abstract is an interface
    for (t, _) in types.iter_mut() {
        if t.kind == Kind::Class && t.is_abstract && t.fields.is_empty() && !t.methods.is_empty() && t.methods.iter().all(|m| m.is_abstract) {
            t.kind = Kind::Interface;
            t.is_abstract = false;
        }
    }
    // nest the inner classes, innermost first
    let mut slots: Vec<Option<Ty>> = types.iter().map(|t| Some(t.0.clone())).collect();
    let parents: Vec<Option<usize>> = types.iter().map(|t| t.1).collect();
    for i in (0..slots.len()).rev() {
        if let Some(p) = parents[i] {
            let t = slots[i].take().unwrap();
            if let Some(pt) = slots[p].as_mut() {
                pt.nested.insert(0, t);
            }
        }
    }
    let top: Vec<Ty> = slots.into_iter().flatten().collect();
    Parsed { units: vec![Unit { ns: rel.to_vec(), types: top, imports }], errors: broken }
}
