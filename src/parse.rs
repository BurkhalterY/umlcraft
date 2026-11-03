//! Reading what the person types on a diagram line: "-id: int[1] = 0" for an attribute and
//! "+placeOrder(item: Order, qty: int): boolean" for an operation (text formats of elements.js:870, 957).

use crate::model::{Eid, Repo};
use serde_json::{json, Value};

fn visibility_of(c: char) -> Option<&'static str> {
    match c {
        '+' => Some("public"),
        '-' => Some("private"),
        '#' => Some("protected"),
        '~' => Some("package"),
        _ => None,
    }
}

/// Split "name: Type[mult] = default" into its parts.
fn split_typed(s: &str) -> (String, String, String, String) {
    let (rest, default) = match s.split_once('=') {
        Some((a, b)) => (a, b.trim().to_string()),
        None => (s, String::new()),
    };
    let (rest, mult) = match (rest.find('['), rest.rfind(']')) {
        (Some(a), Some(b)) if b > a => (format!("{}{}", &rest[..a], &rest[b + 1..]), rest[a + 1..b].trim().to_string()),
        _ => (rest.to_string(), String::new()),
    };
    let (name, ty) = match rest.split_once(':') {
        Some((n, t)) => (n.trim().to_string(), t.trim().to_string()),
        None => (rest.trim().to_string(), String::new()),
    };
    (name, ty, mult, default)
}

/// Keep a link to a classifier that is already linked and still has this name, else store the text.
fn set_type(r: &mut Repo, e: Eid, ty: &str) {
    let linked = r.raw(e, "type").and_then(|v| r.by_ref(v)).map(|t| r.name(t));
    if linked.as_deref() == Some(ty) && !ty.is_empty() {
        return;
    }
    if ty.is_empty() {
        r.remove(e, "type");
    } else {
        r.set(e, "type", json!(ty));
    }
}

pub fn apply_attribute(r: &mut Repo, a: Eid, text: &str) {
    let mut s = text.trim();
    if let Some(v) = s.chars().next().and_then(visibility_of) {
        r.set(a, "visibility", json!(v));
        s = s[1..].trim_start();
    }
    if r.ty(a) == "UMLEnumerationLiteral" {
        r.set(a, "name", json!(s.trim()));
        return;
    }
    let derived = s.starts_with('/');
    if derived {
        s = s[1..].trim_start();
    }
    if r.ty(a) == "UMLAttribute" {
        r.set(a, "isDerived", json!(derived));
    }
    let (name, ty, mult, default) = split_typed(s);
    r.set(a, "name", json!(name));
    set_type(r, a, &ty);
    r.set(a, "multiplicity", json!(mult));
    r.set(a, "defaultValue", json!(default));
}

pub fn apply_operation(r: &mut Repo, o: Eid, text: &str) {
    let mut s = text.trim();
    if let Some(v) = s.chars().next().and_then(visibility_of) {
        r.set(o, "visibility", json!(v));
        s = s[1..].trim_start();
    }
    let Some(open) = s.find('(') else {
        let name = s.split(':').next().unwrap_or("").trim();
        r.set(o, "name", json!(name));
        return;
    };
    let close = s.rfind(')').filter(|c| *c > open).unwrap_or(s.len());
    r.set(o, "name", json!(s[..open].trim()));
    let inner = &s[open + 1..close];
    let after = if close < s.len() { &s[close + 1..] } else { "" };
    let ret_ty = after.trim().strip_prefix(':').map(|x| x.trim().to_string()).unwrap_or_default();

    let old = r.list(o, "parameters");
    let mut keep: Vec<Eid> = vec![];
    for spec in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
        let (dir, rest) = match spec.split_once(char::is_whitespace) {
            Some((d, rest)) if d == "in" || d == "inout" || d == "out" => (d, rest.trim()),
            _ => ("in", spec),
        };
        let (name, ty, mult, default) = split_typed(rest);
        let found = old.iter().copied().find(|p| r.name(*p) == name && r.str(*p, "direction") != "return" && !keep.contains(p));
        let p = match found {
            Some(p) => p,
            None => r.create("UMLParameter", &[("name", json!(name))], Some((o, "parameters"))),
        };
        r.set(p, "direction", json!(dir));
        set_type(r, p, &ty);
        r.set(p, "multiplicity", json!(mult));
        r.set(p, "defaultValue", json!(default));
        keep.push(p);
    }
    let ret = old.iter().copied().find(|p| r.str(*p, "direction") == "return");
    if !ret_ty.is_empty() {
        let p = match ret {
            Some(p) => p,
            None => r.create("UMLParameter", &[("direction", json!("return"))], Some((o, "parameters"))),
        };
        set_type(r, p, &ret_ty);
        keep.push(p);
    }
    for x in r.list(o, "parameters") {
        if !keep.contains(&x) {
            r.detach(x);
        }
    }
    let _: Option<Value> = None;
    if !keep.is_empty() {
        r.set_list(o, "parameters", keep);
    }
}
