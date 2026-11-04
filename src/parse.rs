//! Reading what the person types on a diagram line: "<<entity>> +Order" for a name, "-id: int[1] = 0"
//! for an attribute and "+placeOrder(item: Order, qty: int): boolean" for an operation (text formats of
//! elements.js:870, 957; grammars uml-name.pegjs, uml-attribute.pegjs, uml-operation.pegjs).

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

/// A leading `<<stereotype>>` (or `«stereotype»`, as it is drawn) and the rest of the text.
pub fn split_stereotype(s: &str) -> (Option<String>, &str) {
    let s = s.trim_start();
    for (open, close) in [("<<", ">>"), ("«", "»")] {
        if let Some(rest) = s.strip_prefix(open) {
            if let Some(end) = rest.find(close) {
                return (Some(rest[..end].trim().to_string()), rest[end + close.len()..].trim_start());
            }
        }
    }
    (None, s)
}

/// Set or clear the stereotype: an existing UMLStereotype of that name is linked, otherwise the text
/// is kept as a string (uml-commands.js:288-325).
fn set_stereotype(r: &mut Repo, e: Eid, st: Option<String>) {
    match st.filter(|s| !s.is_empty()) {
        None => r.remove(e, "stereotype"),
        Some(name) => match r.all_of("UMLStereotype").into_iter().find(|x| r.name(*x) == name) {
            Some(x) => r.set_ref(e, "stereotype", Some(x)),
            None => r.set(e, "stereotype", json!(name)),
        },
    }
}

/// Take a leading visibility symbol, setting it when there is one.
fn take_visibility<'a>(r: &mut Repo, e: Eid, s: &'a str) -> &'a str {
    match s.chars().next().and_then(visibility_of) {
        Some(v) => {
            r.set(e, "visibility", json!(v));
            s[1..].trim_start()
        }
        None => s,
    }
}

/// The text a name edit starts with: `<<stereotype>> name`.
pub fn name_text(r: &Repo, e: Eid) -> String {
    let st = crate::views::type_name(r, e, "stereotype");
    // an object's edit also holds its classifier: "name : Class" (uml-object.pegjs)
    let name = crate::views::display_name(r, e);
    if st.is_empty() { name } else { format!("<<{st}>> {name}") }
}

/// Name expression `<<stereotype>> [+|#|-|~]name`: no stereotype clears it (uml-name.pegjs).
pub fn apply_name(r: &mut Repo, e: Eid, text: &str) {
    let (st, rest) = split_stereotype(text.trim());
    set_stereotype(r, e, st);
    let rest = take_visibility(r, e, rest);
    r.set(e, "name", json!(rest.trim()));
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

/// Keep a link to a classifier that is already linked and still has this name; else link an existing
/// classifier of that name; else store the text (uml-commands.js:331-392).
fn set_type(r: &mut Repo, e: Eid, ty: &str) {
    let linked = r.raw(e, "type").and_then(|v| r.by_ref(v)).map(|t| r.name(t));
    if linked.as_deref() == Some(ty) && !ty.is_empty() {
        return;
    }
    if ty.is_empty() {
        r.remove(e, "type");
    } else if let Some(c) = r.all_of("UMLClassifier").into_iter().find(|c| r.name(*c) == ty) {
        r.set_ref(e, "type", Some(c));
    } else {
        r.set(e, "type", json!(ty));
    }
}

pub fn apply_attribute(r: &mut Repo, a: Eid, text: &str) {
    let (st, s) = split_stereotype(text.trim());
    set_stereotype(r, a, st);
    let mut s = take_visibility(r, a, s);
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
    let (st, s) = split_stereotype(text.trim());
    set_stereotype(r, o, st);
    let s = take_visibility(r, o, s);
    let Some(open) = s.find('(') else {
        // no parentheses: no parameters (uml-commands.js:398)
        let name = s.split(':').next().unwrap_or("").trim();
        r.set(o, "name", json!(name));
        for p in r.list(o, "parameters") {
            r.detach(p);
        }
        return;
    };
    let close = s.rfind(')').filter(|c| *c > open).unwrap_or(s.len());
    r.set(o, "name", json!(s[..open].trim()));
    let inner = &s[open + 1..close];
    let after = if close < s.len() { &s[close + 1..] } else { "" };
    let ret_ty = after.trim().strip_prefix(':').map(|x| x.trim().to_string()).unwrap_or_default();

    let old = r.list(o, "parameters");
    // existing parameters are updated in place by position
    let plain: Vec<Eid> = old.iter().copied().filter(|p| r.str(*p, "direction") != "return").collect();
    let mut by_pos = plain.into_iter();
    let mut keep: Vec<Eid> = vec![];
    for spec in inner.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
        let (dir, rest) = match spec.split_once(char::is_whitespace) {
            Some((d, rest)) if d == "in" || d == "inout" || d == "out" => (d, rest.trim()),
            _ => ("in", spec),
        };
        let (name, ty, mult, default) = split_typed(rest);
        let p = match by_pos.next() {
            Some(p) => p,
            None => r.create("UMLParameter", &[], Some((o, "parameters"))),
        };
        r.set(p, "name", json!(name));
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

/// Template parameter expression `<<stereotype>> name : type = default`: the type is kept as text in
/// parameterType (uml-templateparameter.pegjs; uml-commands.js:622-662).
pub fn apply_template_parameter(r: &mut Repo, t: Eid, text: &str) {
    let (st, rest) = split_stereotype(text.trim());
    set_stereotype(r, t, st);
    let (name, ty, _, default) = split_typed(rest);
    r.set(t, "name", json!(name));
    r.set(t, "parameterType", json!(ty));
    r.set(t, "defaultValue", json!(default));
}

// ---- the quick-edit grammars: does the text parse? (grammars/*.pegjs)

/// The text grammars of the inline edits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Grammar {
    /// `<<st>> vis name` (uml-name.pegjs)
    Name,
    /// `<<st>> vis name : type [mult] = default` (uml-attribute.pegjs)
    Attribute,
    /// `<<st>> vis name(params) : type [mult]` (uml-operation.pegjs)
    Operation,
    /// `<<st>> name : type = default` (uml-templateparameter.pegjs)
    TemplateParameter,
    /// `<<st>> vis name : type`, the name and the type optional (uml-object.pegjs)
    Object,
    /// `<<st>> vis name : type = value` (uml-slot.pegjs)
    Slot,
}

/// A parsing expression grammar run over the text: each step returns the position after what it
/// matched, or None. Repetitions are greedy and never give characters back, as in PEG.js.
struct Peg<'a> {
    s: &'a [char],
    /// the identifier accepts `'` (name, object, slot grammars)
    quote_in_id: bool,
    /// the type accepts `[`, `]` (template parameter, object, slot) and `'` (object, slot)
    brackets_in_type: bool,
    quote_in_type: bool,
}

impl Peg<'_> {
    fn common(c: char) -> bool {
        // U+00A0 to U+FFF0, as UTF-16 units: characters beyond U+FFFF are surrogate pairs inside it
        c == ' ' || c.is_ascii_alphanumeric() || "`!@$%^&*_|{};.?-\"".contains(c) || ('\u{a0}'..='\u{fff0}').contains(&c) || c > '\u{ffff}'
    }
    fn id_char(&self, c: char) -> bool {
        Self::common(c) || c == '[' || c == ']' || (self.quote_in_id && c == '\'')
    }
    fn type_char(&self, c: char) -> bool {
        Self::common(c) || c == '<' || c == '>' || (self.brackets_in_type && (c == '[' || c == ']')) || (self.quote_in_type && c == '\'')
    }
    fn many(&self, i: usize, f: impl Fn(char) -> bool) -> usize {
        let mut j = i;
        while j < self.s.len() && f(self.s[j]) {
            j += 1;
        }
        j
    }
    fn plus(&self, i: usize, f: impl Fn(char) -> bool) -> Option<usize> {
        let j = self.many(i, f);
        (j > i).then_some(j)
    }
    fn lit(&self, i: usize, t: &str) -> Option<usize> {
        let t: Vec<char> = t.chars().collect();
        (self.s.len() >= i + t.len() && self.s[i..i + t.len()] == t[..]).then_some(i + t.len())
    }
    fn ws(&self, i: usize) -> usize {
        self.many(i, |c| matches!(c, ' ' | '\t' | '\n' | '\r'))
    }
    fn ident(&self, i: usize) -> Option<usize> {
        self.plus(i, |c| self.id_char(c))
    }
    fn stereotype(&self, i: usize) -> Option<usize> {
        let i = self.lit(i, "<<")?;
        let i = self.ident(self.ws(i))?;
        self.lit(self.ws(i), ">>")
    }
    fn visibility(&self, i: usize) -> Option<usize> {
        ["+", "#", "-", "~"].iter().find_map(|v| self.lit(i, v))
    }
    fn typed(&self, i: usize) -> Option<usize> {
        let i = self.lit(i, ":")?;
        self.plus(self.ws(i), |c| self.type_char(c))
    }
    fn multiplicity(&self, i: usize) -> Option<usize> {
        let i = self.ws(self.lit(i, "[")?);
        let i = self.many(i, |c| c.is_ascii_digit() || c == '.' || c == '*');
        self.lit(self.ws(i), "]")
    }
    fn default(&self, i: usize) -> Option<usize> {
        let i = self.lit(i, "=")?;
        self.ident(self.ws(i))
    }
    fn opt(&self, i: usize, f: impl Fn(&Self, usize) -> Option<usize>) -> usize {
        f(self, i).unwrap_or(i)
    }
    /// `stereotype? _ visibility? _`
    fn head(&self, vis: bool) -> usize {
        let i = self.ws(self.opt(0, Self::stereotype));
        if vis { self.ws(self.opt(i, Self::visibility)) } else { i }
    }
    fn param(&self, i: usize) -> Option<usize> {
        let dir = ["in", "out", "inout"].iter().find_map(|d| self.lit(i, d).and_then(|j| { let k = self.ws(j); (k > j).then_some(k) }));
        let i = self.ws(dir.unwrap_or(i));
        let i = self.ws(self.ident(i)?);
        let i = self.ws(self.opt(i, Self::typed));
        let i = self.ws(self.opt(i, Self::multiplicity));
        Some(self.opt(i, Self::default))
    }
    fn param_list(&self, i: usize) -> Option<usize> {
        let p = self.param(i)?;
        let more = self.lit(self.ws(p), ",").and_then(|j| self.param_list(self.ws(j)));
        Some(more.unwrap_or(p))
    }
    fn parameters(&self, i: usize) -> Option<usize> {
        let i = self.ws(self.lit(i, "(")?);
        let i = self.ws(self.opt(i, Self::param_list));
        self.lit(i, ")")
    }
}

/// Whether the text parses with the grammar, all of it (a failure rejects the edit,
/// uml-commands.js:320-325; quickedit-manager.js:407-427).
pub fn parses(g: Grammar, text: &str) -> bool {
    let s: Vec<char> = text.chars().collect();
    let quote = matches!(g, Grammar::Name | Grammar::Object | Grammar::Slot);
    let p = Peg { s: &s, quote_in_id: quote, brackets_in_type: matches!(g, Grammar::TemplateParameter | Grammar::Object | Grammar::Slot), quote_in_type: matches!(g, Grammar::Object | Grammar::Slot) };
    let end = (|| -> Option<usize> {
        match g {
            Grammar::Name => Some(p.ws(p.ident(p.head(true))?)),
            Grammar::Attribute => {
                let i = p.ws(p.ident(p.head(true))?);
                let i = p.ws(p.opt(i, Peg::typed));
                let i = p.ws(p.opt(i, Peg::multiplicity));
                Some(p.opt(i, Peg::default))
            }
            Grammar::Operation => {
                let i = p.ws(p.ident(p.head(true))?);
                let i = p.ws(p.opt(i, Peg::parameters));
                let i = p.ws(p.opt(i, Peg::typed));
                Some(p.opt(i, Peg::multiplicity))
            }
            Grammar::TemplateParameter => {
                let i = p.ws(p.ident(p.head(false))?);
                let i = p.ws(p.opt(i, Peg::typed));
                Some(p.opt(i, Peg::default))
            }
            Grammar::Object => {
                let i = p.ws(p.opt(p.head(true), Peg::ident));
                Some(p.opt(i, Peg::typed))
            }
            Grammar::Slot => {
                let i = p.ws(p.ident(p.head(true))?);
                let i = p.ws(p.opt(i, Peg::typed));
                Some(p.opt(i, Peg::default))
            }
        }
    })();
    end == Some(s.len())
}
