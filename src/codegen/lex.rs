//! A tolerant tokenizer for the brace languages, and the helpers the reverse engineering parsers
//! share: statement splitting, matching brackets, comma lists and typed declarations. It never
//! fails: unbalanced brackets are reported through `Toks::broken` and the rest is still read.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum K {
    Id,
    Num,
    Str,
    P,
}

#[derive(Clone, Debug)]
pub struct T {
    pub k: K,
    pub s: String,
    /// the documentation comment just before this token (`/** */`, `///`)
    pub doc: String,
    /// a line break comes before this token
    pub nl: bool,
}

impl T {
    pub fn is(&self, s: &str) -> bool {
        self.s == s && self.k != K::Str
    }
    pub fn id(&self) -> bool {
        self.k == K::Id
    }
}

#[derive(Clone, Copy, Default)]
pub struct Cfg {
    /// `#` starts a line comment (PHP), except `#[`
    pub hash_comment: bool,
    /// '…' is a string, not a character literal
    pub sq_strings: bool,
    /// `$name` is one identifier (PHP)
    pub dollar_ids: bool,
    /// `--` starts a line comment (SQL)
    pub dash_comment: bool,
}

pub fn lex(src: &str, cfg: Cfg) -> Vec<T> {
    let c: Vec<char> = src.chars().collect();
    let mut out: Vec<T> = vec![];
    let (mut i, mut doc, mut nl) = (0, String::new(), false);
    let push = |out: &mut Vec<T>, k: K, s: String, doc: &mut String, nl: &mut bool| {
        out.push(T { k, s, doc: std::mem::take(doc), nl: *nl });
        *nl = false;
    };
    while i < c.len() {
        let ch = c[i];
        let next = c.get(i + 1).copied().unwrap_or('\0');
        if ch == '\n' {
            nl = true;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if (ch == '/' && next == '/') || (cfg.hash_comment && ch == '#' && next != '[') || (cfg.dash_comment && ch == '-' && next == '-') {
            let start = i;
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
            let text: String = c[start..i].iter().collect();
            if let Some(d) = text.strip_prefix("///") {
                if !doc.is_empty() {
                    doc.push('\n');
                }
                doc.push_str(d.trim());
            }
        } else if ch == '/' && next == '*' {
            let start = i;
            i += 2;
            while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                if c[i] == '\n' {
                    nl = true;
                }
                i += 1;
            }
            i = (i + 2).min(c.len());
            let text: String = c[start..i].iter().collect();
            if text.starts_with("/**") && text.len() > 4 {
                doc = clean_block_doc(&text);
            }
        } else if ch == '"' || ch == '`' || (ch == '\'' && cfg.sq_strings) {
            let start = i;
            let triple = ch == '"' && next == '"' && c.get(i + 2) == Some(&'"');
            i += if triple { 3 } else { 1 };
            while i < c.len() {
                if c[i] == '\\' {
                    i += 2;
                    continue;
                }
                if triple {
                    if c[i] == '"' && c.get(i + 1) == Some(&'"') && c.get(i + 2) == Some(&'"') {
                        i += 3;
                        break;
                    }
                } else if c[i] == ch {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let i2 = i.min(c.len());
            push(&mut out, K::Str, c[start..i2].iter().collect(), &mut doc, &mut nl);
        } else if ch == '\'' {
            // a character literal ('a', '\n'), otherwise a lifetime or label ('a)
            if next == '\\' || c.get(i + 2) == Some(&'\'') {
                let start = i;
                i += 1;
                while i < c.len() && c[i] != '\'' {
                    i += if c[i] == '\\' { 2 } else { 1 };
                }
                i = (i + 1).min(c.len());
                push(&mut out, K::Str, c[start..i].iter().collect(), &mut doc, &mut nl);
            } else {
                let start = i;
                i += 1;
                while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                    i += 1;
                }
                push(&mut out, K::Id, c[start..i].iter().collect(), &mut doc, &mut nl);
            }
        } else if ch.is_alphabetic() || ch == '_' || (cfg.dollar_ids && ch == '$') || ch == '$' && next.is_alphabetic() {
            let start = i;
            i += 1;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_' || c[i] == '$') {
                i += 1;
            }
            push(&mut out, K::Id, c[start..i].iter().collect(), &mut doc, &mut nl);
        } else if ch.is_ascii_digit() {
            let start = i;
            while i < c.len() && (c[i].is_alphanumeric() || c[i] == '.' || c[i] == '_') {
                i += 1;
            }
            push(&mut out, K::Num, c[start..i].iter().collect(), &mut doc, &mut nl);
        } else {
            let two: String = [ch, next].iter().collect();
            let three: String = c[i..(i + 3).min(c.len())].iter().collect();
            let s = if three == "..." {
                three
            } else if ["::", "->", "=>"].contains(&two.as_str()) {
                two
            } else {
                ch.to_string()
            };
            i += s.chars().count();
            push(&mut out, K::P, s, &mut doc, &mut nl);
        }
    }
    out
}

/// The text of a `/** … */` comment without its stars.
pub fn clean_block_doc(text: &str) -> String {
    let inner = text.trim_start_matches("/**").trim_end_matches("*/");
    let lines: Vec<String> = inner.lines().map(|l| l.trim().trim_start_matches('*').trim().to_string()).collect();
    lines.join("\n").trim().to_string()
}

/// The index of the bracket closing the one at `i` (or the end when it is missing).
pub fn close(t: &[T], i: usize) -> usize {
    let (open, shut) = match t[i].s.as_str() {
        "(" => ("(", ")"),
        "[" => ("[", "]"),
        "<" => ("<", ">"),
        _ => ("{", "}"),
    };
    let mut depth = 0;
    for (j, x) in t.iter().enumerate().skip(i) {
        if x.k != K::P {
            continue;
        }
        if x.s == open {
            depth += 1;
        } else if x.s == shut {
            depth -= 1;
            if depth == 0 {
                return j;
            }
        } else if open == "<" && (x.s == ";" || x.s == "{" || x.s == ")") {
            return j.saturating_sub(1);
        }
    }
    t.len().saturating_sub(1).max(i)
}

/// Whether the brackets are balanced: a file that is not counts as one with errors.
pub fn balanced(t: &[T]) -> bool {
    let mut depth: i64 = 0;
    for x in t {
        if x.k == K::P {
            match x.s.as_str() {
                "{" | "(" | "[" => depth += 1,
                "}" | ")" | "]" => depth -= 1,
                _ => {}
            }
            if depth < 0 {
                return false;
            }
        }
    }
    depth == 0
}

/// One statement of a body: its tokens, and the `{ … }` block that ends it, if any.
pub struct Stmt<'a> {
    pub head: &'a [T],
    pub block: Option<&'a [T]>,
}

/// Split a body into statements: each ends at `;`, `,` (when `comma`), after a `{ … }` block, or
/// (when `nl`) at a line break outside brackets that does not continue the statement.
pub fn statements(t: &[T], nl: bool) -> Vec<Stmt<'_>> {
    let mut out = vec![];
    let (mut start, mut i, mut depth) = (0, 0, 0i32);
    let cont = |p: &T| p.k == K::P && [",", "=", ":", ".", "->", "=>", "(", "<", "|", "&", "+", "-", "*", "@"].contains(&p.s.as_str());
    while i < t.len() {
        let x = &t[i];
        if nl && depth == 0 && x.nl && i > start && !cont(&t[i - 1]) && !(x.k == K::P && [".", ":", "{", ",", ")", "->", "=", "?", "where", "|"].contains(&x.s.as_str())) && !x.is("where") {
            out.push(Stmt { head: &t[start..i], block: None });
            start = i;
        }
        if x.k == K::P {
            match x.s.as_str() {
                "(" | "[" => depth += 1,
                ")" | "]" => depth -= 1,
                ";" if depth <= 0 => {
                    if i > start {
                        out.push(Stmt { head: &t[start..i], block: None });
                    }
                    start = i + 1;
                    depth = 0;
                }
                "{" if depth <= 0 => {
                    let j = close(t, i);
                    // an initializer (`= { … }`, `= new X { … }`) does not end the statement
                    let head = &t[start..i];
                    let init = find_top(head, "=", false).is_some() || find_top(head, "=>", false).is_some();
                    if init && !nl {
                        i = j + 1;
                        continue;
                    }
                    out.push(Stmt { head: &t[start..i], block: Some(&t[(i + 1).min(j)..j]) });
                    i = j + 1;
                    // `};` and `} Name;` (C typedefs) stay with the block they follow
                    start = i;
                    depth = 0;
                    continue;
                }
                _ => {}
            }
        }
        i += 1;
    }
    if start < t.len() {
        out.push(Stmt { head: &t[start..], block: None });
    }
    out
}

/// Split at top-level commas (outside (), [], {} and <>).
pub fn commas(t: &[T]) -> Vec<&[T]> {
    let mut out = vec![];
    let (mut start, mut depth, mut angle) = (0, 0i32, 0i32);
    for (i, x) in t.iter().enumerate() {
        if x.k != K::P {
            continue;
        }
        match x.s.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth -= 1,
            "<" => angle += 1,
            ">" if angle > 0 => angle -= 1,
            "," if depth == 0 && angle == 0 => {
                out.push(&t[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < t.len() {
        out.push(&t[start..]);
    }
    out.retain(|p| !p.is_empty());
    out
}

/// Position of the first top-level token equal to `s` (outside brackets and, when `angles`, <>).
pub fn find_top(t: &[T], s: &str, angles: bool) -> Option<usize> {
    let (mut depth, mut angle) = (0i32, 0i32);
    for (i, x) in t.iter().enumerate() {
        if x.k == K::P && x.s == s && depth == 0 && angle == 0 {
            return Some(i);
        }
        if x.k != K::P {
            continue;
        }
        match x.s.as_str() {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth -= 1,
            "<" if angles => angle += 1,
            ">" if angles && angle > 0 => angle -= 1,
            _ => {}
        }
    }
    None
}

/// Tokens as source text: a space only between two words, and after a comma.
pub fn join(t: &[T]) -> String {
    let mut s = String::new();
    for (i, x) in t.iter().enumerate() {
        if i > 0 {
            let p = &t[i - 1];
            let word = |y: &T| y.k != K::P || y.s == "...";
            if (word(p) && word(x)) || p.is(",") || p.is("=") || x.is("=") || p.is("|") || x.is("|") || x.is("=>") || p.is("=>") {
                s.push(' ');
            }
        }
        s.push_str(&x.s);
    }
    s
}

/// Drop leading annotations (`@Name(…)`, `[Attr]`, `#[attr]`) from a declaration.
pub fn skip_annotations(mut t: &[T]) -> &[T] {
    loop {
        if t.len() >= 2 && t[0].is("@") && t[1].id() && !t[1].is("interface") {
            let mut i = 2;
            while i + 1 < t.len() && t[i].is(".") && t[i + 1].id() {
                i += 2;
            }
            if i < t.len() && t[i].is("(") {
                i = close(t, i) + 1;
            }
            t = &t[i.min(t.len())..];
        } else if !t.is_empty() && t[0].is("[") {
            t = &t[(close(t, 0) + 1).min(t.len())..];
        } else if t.len() >= 2 && t[0].is("#") && t[1].is("[") {
            t = &t[(close(t, 1) + 1).min(t.len())..];
        } else {
            return t;
        }
    }
}

/// The first documentation comment of a declaration's tokens.
pub fn doc_of(t: &[T]) -> String {
    t.iter().find(|x| !x.doc.is_empty()).map(|x| x.doc.clone()).unwrap_or_default()
}

/// Generic parameter names of `<T, U extends X>` starting at `t[i]` (`<`); returns them and the
/// index after `>`.
pub fn generics(t: &[T], i: usize) -> (Vec<String>, usize) {
    if i >= t.len() || !t[i].is("<") {
        return (vec![], i);
    }
    let j = close(t, i);
    let names = commas(&t[i + 1..j]).iter().filter_map(|p| p.iter().find(|x| x.id() && !x.is("in") && !x.is("out") && !x.is("const") && !x.is("typename") && !x.is("class")).map(|x| x.s.clone())).collect();
    (names, j + 1)
}

/// A declaration in the `Type name` languages (Java, C#, C++, C, Dart): its modifiers, type, name,
/// parameters (for a method), default value and the text after the parameter list.
#[derive(Default, Debug, Clone)]
pub struct Decl {
    pub mods: Vec<String>,
    pub ty: String,
    pub name: String,
    pub params: Option<Vec<(String, String, String)>>,
    pub default: String,
    pub tail: String,
}

/// Read `mods Type name(params) tail` or `mods Type a = 1, b` (`mods` lists the modifier words).
pub fn typed_decls(t: &[T], mods: &[&str]) -> Vec<Decl> {
    let t = skip_annotations(t);
    let mut i = 0;
    let mut d = Decl::default();
    while i < t.len() && t[i].id() && mods.contains(&t[i].s.as_str()) {
        d.mods.push(t[i].s.clone());
        i += 1;
        // `pub(crate)`-like groups after a modifier are skipped
    }
    let rest = &t[i..];
    if let Some(p) = find_top(rest, "(", true) {
        // the name is the identifier before the parameters (`~X` for a destructor)
        if p == 0 || !rest[p - 1].id() {
            // `operator==(...)` and the like: keep what is there
            if p == 0 {
                return vec![];
            }
        }
        let mut name = rest[p - 1].s.clone();
        let mut ty_end = p - 1;
        if p >= 2 && rest[p - 2].is("~") {
            name = format!("~{name}");
            ty_end = p - 2;
        }
        if p >= 2 && rest[p - 2].is("operator") {
            ty_end = p - 2;
            name = format!("operator{name}");
        }
        d.name = name;
        d.ty = join(&rest[..ty_end]);
        let q = close(rest, p);
        let inner = &rest[p + 1..q.max(p + 1)];
        d.params = Some(commas(inner).into_iter().filter_map(|pt| typed_param(pt, mods)).collect());
        d.tail = join(&rest[(q + 1).min(rest.len())..]);
        return vec![d];
    }
    // fields: `Type a = 1, b, c = 2`
    let mut out = vec![];
    for (k, part) in commas(rest).into_iter().enumerate() {
        let (left, default) = match find_top(part, "=", true) {
            Some(e) => (&part[..e], join(&part[e + 1..])),
            None => (part, String::new()),
        };
        let Some(ni) = left.iter().rposition(|x| x.id()) else { continue };
        let mut e = Decl { mods: d.mods.clone(), name: left[ni].s.clone(), default, ..Default::default() };
        // array brackets after the name (`int a[10]`) belong to the type
        let after = join(&left[ni + 1..]);
        if k == 0 {
            e.ty = format!("{}{}", join(&left[..ni]), if after.starts_with('[') { "[]" } else { "" });
        } else {
            e.ty = out.first().map(|f: &Decl| f.ty.clone()).unwrap_or_default();
        }
        out.push(e);
    }
    out
}

/// A parameter `Type name = default` (`final`, `ref`, `this.` and the like dropped). Returns
/// (name, type, default).
pub fn typed_param(t: &[T], mods: &[&str]) -> Option<(String, String, String)> {
    let mut t = skip_annotations(t);
    while !t.is_empty() && t[0].id() && (mods.contains(&t[0].s.as_str()) || ["ref", "out", "in", "params", "required", "this"].contains(&t[0].s.as_str())) {
        t = &t[1..];
    }
    while !t.is_empty() && (t[0].is("{") || t[0].is("[") || t[0].is(".")) {
        t = &t[1..];
    }
    while !t.is_empty() && (t[t.len() - 1].is("}") || t[t.len() - 1].is("]")) && !join(t).ends_with("[]") {
        t = &t[..t.len() - 1];
    }
    if t.is_empty() {
        return None;
    }
    let (left, default) = match find_top(t, "=", true) {
        Some(e) => (&t[..e], join(&t[e + 1..])),
        None => (t, String::new()),
    };
    if left.len() == 1 {
        return if left[0].is("void") { None } else { Some((String::new(), left[0].s.clone(), default)) };
    }
    let ni = left.iter().rposition(|x| x.id())?;
    let after = join(&left[ni + 1..]);
    let ty = format!("{}{}", join(&left[..ni]), if after.starts_with('[') { "[]" } else { "" });
    Some((left[ni].s.clone(), ty, default))
}

/// A parameter `name?: Type = default` of the colon languages (TypeScript, Kotlin, Swift, Rust).
pub fn colon_param(t: &[T], skip: &[&str]) -> Option<(String, String, String)> {
    let mut t = skip_annotations(t);
    while !t.is_empty() && t[0].id() && skip.contains(&t[0].s.as_str()) {
        t = &t[1..];
    }
    if t.is_empty() {
        return None;
    }
    let (left, default) = match find_top(t, "=", true) {
        Some(e) => (&t[..e], join(&t[e + 1..])),
        None => (t, String::new()),
    };
    match find_top(left, ":", true) {
        Some(c) => {
            let name = left[..c].iter().rev().find(|x| x.id()).map(|x| x.s.clone()).unwrap_or_default();
            let mut ty = join(&left[c + 1..]);
            if left[..c].iter().any(|x| x.is("?")) && !ty.ends_with('?') {
                ty.push('?');
            }
            Some((name, ty, default))
        }
        None => {
            let name = left.iter().rev().find(|x| x.id())?.s.clone();
            Some((name, String::new(), default))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_statements_and_declarations() {
        let t = lex("/** The id. */ private int id = 0; public List<String> names(int a, Map<K, V> m) { return null; }", Cfg::default());
        let st = statements(&t, false);
        assert_eq!(st.len(), 2);
        let f = typed_decls(st[0].head, &["private", "public"]);
        assert_eq!((f[0].name.as_str(), f[0].ty.as_str(), f[0].default.as_str()), ("id", "int", "0"));
        assert_eq!(doc_of(st[0].head), "The id.");
        let m = typed_decls(st[1].head, &["private", "public"]);
        assert_eq!(m[0].name, "names");
        assert_eq!(m[0].ty, "List<String>");
        let p = m[0].params.as_ref().unwrap();
        assert_eq!(p[1], ("m".to_string(), "Map<K, V>".to_string(), String::new()));
    }
}
