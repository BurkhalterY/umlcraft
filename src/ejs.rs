//! EJS templates for `umlcraft ejs` and the `-o` file names of `umlcraft ejs` and `umlcraft image`
//! (src/main-process/cli.js:132-195, src/engine/default-commands.js:1184-1260). StarUML renders them
//! with the ejs package 3.1.10; UMLCraft follows its tags and whitespace rules (node_modules/ejs/lib/ejs.js
//! generateSource, scanLine and _addOutput): `<% code %>`, `<%= escaped %>`, `<%- raw %>`, `<%# comment %>`,
//! `<%%` and `%%>` for literal delimiters, `-%>` removing the line break after it, `<%_` and `_%>`
//! removing the spaces and tabs around them, and `include(path, data)`. The code runs in the
//! JavaScript subset of src/js.rs; the data are `app`, `element` and `filenamify`.

use crate::js::{self, Interp, Thrown, V};
use crate::model::{Eid, Repo};
use std::path::{Path, PathBuf};

/// The template as JavaScript source: text becomes `__append("…")`, `<%= x %>` `__append(__escape(x))`,
/// `<%- x %>` `__append(x)` and `<% x %>` the code itself.
pub fn compile(template: &str) -> Result<String, String> {
    // slurp spaces and tabs before <%_ and after _%>
    let text = slurp(template);
    let parts = split_tags(&text);
    let mut src = String::new();
    #[derive(PartialEq, Clone, Copy)]
    enum Mode {
        Eval,
        Escaped,
        Raw,
        Comment,
        Literal,
    }
    let mut mode: Option<Mode> = None;
    let mut truncate = false;
    let is_open = |p: &str| p.starts_with("<%") && !p.starts_with("<%%");
    let is_close = |p: &str| matches!(p, "%>" | "-%>" | "_%>");
    for (i, line) in parts.iter().enumerate() {
        if TAGS.contains(&line.as_str()) && is_open(line) {
            let closing = parts.get(i + 2).map(String::as_str).unwrap_or("");
            if !is_close(closing) {
                return Err(format!("Could not find matching close tag for \"{line}\"."));
            }
        }
        let add_output = |line: &str, truncate: &mut bool, src: &mut String| {
            let mut line = line;
            if *truncate {
                line = line.strip_prefix("\r\n").or_else(|| line.strip_prefix('\n')).or_else(|| line.strip_prefix('\r')).unwrap_or(line);
                *truncate = false;
            }
            if line.is_empty() {
                return;
            }
            src.push_str(";__append(");
            src.push_str(&js_string(line));
            src.push_str(")\n");
        };
        match line.as_str() {
            "<%" | "<%_" => mode = Some(Mode::Eval),
            "<%=" => mode = Some(Mode::Escaped),
            "<%-" => mode = Some(Mode::Raw),
            "<%#" => mode = Some(Mode::Comment),
            "<%%" => {
                mode = Some(Mode::Literal);
                src.push_str(";__append(\"<%\")\n");
            }
            "%%>" => {
                mode = Some(Mode::Literal);
                src.push_str(";__append(\"%>\")\n");
            }
            "%>" | "-%>" | "_%>" => {
                if mode == Some(Mode::Literal) {
                    add_output(line, &mut truncate, &mut src);
                }
                mode = None;
                truncate = line.starts_with('-') || line.starts_with('_');
            }
            _ => match mode {
                Some(Mode::Eval) => {
                    src.push_str(";");
                    src.push_str(line);
                    src.push('\n');
                }
                Some(Mode::Escaped) => {
                    src.push_str(";__append(__escape(");
                    src.push_str(strip_semi(line));
                    src.push_str("))\n");
                }
                Some(Mode::Raw) => {
                    src.push_str(";__append(");
                    src.push_str(strip_semi(line));
                    src.push_str(")\n");
                }
                Some(Mode::Comment) => {}
                Some(Mode::Literal) | None => add_output(line, &mut truncate, &mut src),
            },
        }
    }
    Ok(src)
}

/// The delimiters, in the order ejs's regular expression tries them at one position.
const TAGS: &[&str] = &["<%%", "%%>", "<%=", "<%-", "<%_", "<%#", "<%", "%>", "-%>", "_%>"];

fn slurp(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut rest = t;
    while let Some(i) = rest.find("<%_") {
        out.push_str(rest[..i].trim_end_matches([' ', '\t']));
        out.push_str("<%_");
        rest = &rest[i + 3..];
    }
    out.push_str(rest);
    let mut out2 = String::with_capacity(out.len());
    let mut rest = out.as_str();
    while let Some(i) = rest.find("_%>") {
        out2.push_str(&rest[..i + 3]);
        rest = rest[i + 3..].trim_start_matches([' ', '\t']);
    }
    out2.push_str(rest);
    out2
}

/// The text and the delimiters, as ejs's parseTemplateText splits them: at each place the leftmost
/// delimiter, the first in TAGS order when several start there.
fn split_tags(t: &str) -> Vec<String> {
    let mut out = vec![];
    let mut rest = t;
    loop {
        let mut best: Option<(usize, &str)> = None;
        for tag in TAGS {
            if let Some(i) = rest.find(tag) {
                if best.map(|(b, _)| i < b).unwrap_or(true) {
                    best = Some((i, tag));
                }
            }
        }
        match best {
            Some((i, tag)) => {
                if i > 0 {
                    out.push(rest[..i].to_string());
                }
                out.push(tag.to_string());
                rest = &rest[i + tag.len()..];
            }
            None => {
                if !rest.is_empty() {
                    out.push(rest.to_string());
                }
                return out;
            }
        }
    }
}

/// `x;  ` → `x  ` (ejs stripSemi).
fn strip_semi(s: &str) -> &str {
    let t = s.trim_end();
    t.strip_suffix(';').unwrap_or(s)
}

fn js_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Render a template with `{app, element, filenamify}`; `dir` is the folder includes are read from.
pub fn render(template: &str, repo: &Repo, filename: &str, element: Option<Eid>, dir: Option<&Path>) -> Result<String, String> {
    let src = compile(template)?;
    let prog = js::parse(&src)?;
    let mut it = Interp::new(repo, filename);
    it.include = Some(include);
    if let Some(d) = dir {
        it.dirs.push(d.to_path_buf());
    }
    let env = it.new_scope();
    for (n, v) in [("app", V::Host("app")), ("element", element.map(V::Elem).unwrap_or(V::Undef))] {
        it.set_global(n, v);
    }
    match it.run(&prog, &env) {
        Ok(()) => Ok(it.out.pop().unwrap_or_default()),
        Err(t) => Err(js::thrown_message(&mut it, &t)),
    }
}

/// `include(path, data)`: the template at the path (".ejs" added when it has no extension), relative
/// to the including template, rendered with the same data and `data` (ejs.js includeFile).
fn include(it: &mut Interp, path: &str, data: V) -> Result<V, Thrown> {
    let base = it.dirs.last().cloned().unwrap_or_else(|| PathBuf::from("."));
    let mut p = base.join(path);
    if p.extension().is_none() {
        p.set_extension("ejs");
    }
    let fail = |it: &mut Interp, m: String| -> Result<V, Thrown> {
        let e = js::obj([("name".to_string(), js::s("Error")), ("message".to_string(), js::s(&m))].into_iter().collect());
        let _ = it;
        Err(Thrown(e))
    };
    if it.dirs.len() > 50 {
        return fail(it, "Too many nested includes".into());
    }
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) => return fail(it, format!("Could not read {}: {e}", p.display())),
    };
    let prog = match compile(&text).and_then(|src| js::parse(&src)) {
        Ok(p) => p,
        Err(m) => return fail(it, m),
    };
    let env = it.new_scope();
    for k in it.keys(&data) {
        let v = it.get(&data, &k)?;
        js::declare_in(&env, &k, v);
    }
    it.dirs.push(p.parent().map(Path::to_path_buf).unwrap_or(base));
    it.out.push(String::new());
    let r = it.run(&prog, &env);
    let out = it.out.pop().unwrap_or_default();
    it.dirs.pop();
    r?;
    Ok(js::s(&out))
}
