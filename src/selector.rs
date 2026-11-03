//! The element selector language of `repository.select` and the command-line `-s` option
//! (src/core/repository.js:2095-2253):
//!
//! - `A::B` takes the children of A named B; a term with no `::` before it searches the whole project;
//! - `@Type` keeps elements of that type or a subtype;
//! - `.field` follows a reference or collection field;
//! - `[field=value]` keeps elements whose field has that value;
//! - a bare `Name` keeps elements with that name.
//!
//! Example: `Package1::@UMLClass`, `Class1.operations[isAbstract=false]`.

use crate::model::{Eid, Repo};
use serde_json::Value;

enum Part {
    Name(String),
    Type(String),
    Field(String),
    Filter(String, String),
}

fn parse_term(t: &str) -> Result<Vec<Part>, String> {
    let mut parts = vec![];
    let mut rest = t.trim();
    let ident = |s: &str| -> usize { s.find(['@', '.', '[']).unwrap_or(s.len()) };
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('@') {
            let n = ident(r);
            if n == 0 {
                return Err(format!("A type name is expected after '@' in \"{t}\""));
            }
            parts.push(Part::Type(r[..n].to_string()));
            rest = &r[n..];
        } else if let Some(r) = rest.strip_prefix('.') {
            let n = ident(r);
            if n == 0 {
                return Err(format!("A field name is expected after '.' in \"{t}\""));
            }
            parts.push(Part::Field(r[..n].to_string()));
            rest = &r[n..];
        } else if let Some(r) = rest.strip_prefix('[') {
            let end = r.find(']').ok_or_else(|| format!("']' is expected in \"{t}\""))?;
            let (k, v) = r[..end].split_once('=').ok_or_else(|| format!("'=' is expected in \"[{}]\"", &r[..end]))?;
            parts.push(Part::Filter(k.trim().to_string(), v.trim().trim_matches(['"', '\'']).to_string()));
            rest = &r[end + 1..];
        } else {
            let n = ident(rest);
            parts.push(Part::Name(rest[..n].trim().to_string()));
            rest = &rest[n..];
        }
    }
    Ok(parts)
}

fn value_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Select elements; a malformed selector gives a descriptive error.
pub fn select(r: &Repo, selector: &str) -> Result<Vec<Eid>, String> {
    let sel = selector.trim();
    if sel.is_empty() {
        return Err("The selector is empty".into());
    }
    let terms: Vec<&str> = sel.split("::").collect();
    let mut current: Option<Vec<Eid>> = None;
    for term in terms {
        let parts = parse_term(term)?;
        // the first term looks at every element; each later one at the children of the last result
        let mut set: Vec<Eid> = match &current {
            None => r.all().collect(),
            Some(c) => c.iter().flat_map(|e| r.children(*e).into_iter().map(|(_, x)| x)).collect(),
        };
        for p in parts {
            set = match p {
                Part::Name(n) if n.is_empty() => set,
                Part::Name(n) => set.into_iter().filter(|e| r.name(*e) == n).collect(),
                Part::Type(t) => set.into_iter().filter(|e| r.ty(*e) == t || r.kind_of(*e, &t)).collect(),
                Part::Field(f) => {
                    let mut out = vec![];
                    for e in set {
                        if let Some(x) = r.refd(e, &f) {
                            out.push(x);
                        }
                        out.extend(r.list(e, &f));
                        if let Some(Value::Array(a)) = r.raw(e, &f) {
                            out.extend(a.iter().filter_map(|x| r.by_ref(x)));
                        }
                    }
                    out
                }
                Part::Filter(k, v) => set.into_iter().filter(|e| r.val(*e, &k).map(|x| value_text(&x) == v).unwrap_or(false)).collect(),
            };
        }
        let mut seen = std::collections::HashSet::new();
        set.retain(|e| seen.insert(*e));
        current = Some(set);
    }
    Ok(current.unwrap_or_default())
}
