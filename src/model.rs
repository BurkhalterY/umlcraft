//! The project, held as the same JSON objects the .mdj file contains (in an arena), so anything
//! UMLCraft does not understand is written back untouched. Rules: MECHANICS.md "File format".

use crate::meta;
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};

pub type Eid = usize;

#[derive(Clone, Debug)]
pub enum F {
    V(Value),
    One(Eid),
    Many(Vec<Eid>),
}

#[derive(Clone, Debug)]
pub struct Elem {
    pub ty: String,
    pub id: String,
    pub parent: Option<Eid>,
    /// every key of the object except `_type`, in file order
    pub fields: IndexMap<String, F>,
}

#[derive(Clone, Default)]
pub struct Repo {
    pub elems: Vec<Option<Elem>>,
    pub by_id: HashMap<String, Eid>,
    pub root: Eid,
}

// ---- ids: 2 zero bytes + 8-byte ms timestamp + 2-byte counter + 2-byte random, base64 (core.js:150)
static COUNTER: AtomicU32 = AtomicU32::new(0);

fn rand16() -> u32 {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let mut x = (t.as_nanos() as u64) ^ 0x9E37_79B9_7F4A_7C15 ^ (COUNTER.load(Ordering::Relaxed) as u64).wrapping_mul(0x2545_F491_4F6C_DD1D);
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    (x & 0xffff) as u32
}

pub const ID_BYTES: usize = 14; // core.js:150, sample.mdj ids such as AAAAAAGhohI4jHit75s=

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

pub fn new_id() -> String {
    if COUNTER.load(Ordering::Relaxed) == 0 {
        COUNTER.store(rand16() + 1, Ordering::Relaxed);
    }
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    let c = COUNTER.fetch_add(1, Ordering::Relaxed) & 0xffff;
    let r = rand16();
    let mut bytes = Vec::with_capacity(ID_BYTES);
    bytes.extend_from_slice(&[0u8, 0u8]);
    bytes.extend_from_slice(&ms.to_be_bytes());
    bytes.push((c >> 8) as u8);
    bytes.push((c & 255) as u8);
    bytes.push((r >> 8) as u8);
    bytes.push((r & 255) as u8);
    base64(&bytes)
}

/// A number the way JavaScript writes it: whole numbers without a decimal point.
pub fn num(f: f64) -> Value {
    if f.is_finite() && f.fract() == 0.0 && f.abs() < 1e15 {
        json!(f as i64)
    } else if f.is_finite() {
        json!(f)
    } else {
        json!(0)
    }
}

pub fn rf(id: &str) -> Value {
    json!({ "$ref": id })
}

fn is_elem(v: &Value) -> bool {
    v.as_object().map(|m| m.get("_type").map(|t| t.is_string()).unwrap_or(false)).unwrap_or(false)
}

impl Repo {
    pub fn parse(text: &str) -> Result<Repo, String> {
        let v: Value = serde_json::from_str(text).map_err(|e| format!("Not a valid model file: {e}"))?;
        if !is_elem(&v) {
            return Err("This file is not a model file (no _type on the root).".into());
        }
        let mut r = Repo::default();
        r.root = r.import(v, None);
        Ok(r)
    }

    fn import(&mut self, v: Value, parent: Option<Eid>) -> Eid {
        let Value::Object(map) = v else { unreachable!() };
        let eid = self.elems.len();
        self.elems.push(None);
        let ty = map.get("_type").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let id = map.get("_id").and_then(|t| t.as_str()).unwrap_or("").to_string();
        let mut fields = IndexMap::new();
        for (k, val) in map {
            if k == "_type" {
                continue;
            }
            let f = if is_elem(&val) {
                F::One(self.import(val, Some(eid)))
            } else if matches!(&val, Value::Array(a) if !a.is_empty() && a.iter().all(is_elem)) {
                let Value::Array(a) = val else { unreachable!() };
                F::Many(a.into_iter().map(|x| self.import(x, Some(eid))).collect())
            } else {
                F::V(val)
            };
            fields.insert(k, f);
        }
        if !id.is_empty() {
            self.by_id.insert(id.clone(), eid);
        }
        self.elems[eid] = Some(Elem { ty, id, parent, fields });
        eid
    }

    pub fn export(&self, eid: Eid) -> Value {
        let e = self.e(eid);
        let mut m = Map::new();
        m.insert("_type".into(), json!(e.ty));
        for (k, f) in &e.fields {
            let v = match f {
                F::V(v) => v.clone(),
                F::One(c) => self.export(*c),
                F::Many(cs) => Value::Array(cs.iter().map(|c| self.export(*c)).collect()),
            };
            m.insert(k.clone(), v);
        }
        Value::Object(m)
    }

    /// JSON with one TAB per level ("tab" indent), as the original writes it (repository.js:1586).
    pub fn serialize(&self) -> String {
        let v = self.export(self.root);
        let mut buf = Vec::new();
        let fmt = serde_json::ser::PrettyFormatter::with_indent(b"\t");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
        serde::Serialize::serialize(&v, &mut ser).ok();
        String::from_utf8(buf).unwrap_or_default()
    }

    pub fn blank() -> Repo {
        let mut r = Repo::default();
        let root = r.alloc("Project", None);
        r.root = root;
        r.set(root, "name", json!("Untitled"));
        let model = r.create("UMLModel", &[("name", json!("Model"))], Some((root, "ownedElements")));
        r.create("UMLClassDiagram", &[("name", json!("Main")), ("defaultDiagram", json!(true))], Some((model, "ownedElements")));
        r
    }

    // ------------------------------------------------------------ access
    pub fn e(&self, eid: Eid) -> &Elem {
        self.elems[eid].as_ref().expect("element was removed")
    }
    pub fn em(&mut self, eid: Eid) -> &mut Elem {
        self.elems[eid].as_mut().expect("element was removed")
    }
    pub fn alive(&self, eid: Eid) -> bool {
        self.elems.get(eid).map(|x| x.is_some()).unwrap_or(false)
    }
    pub fn ty(&self, eid: Eid) -> &str {
        &self.e(eid).ty
    }
    pub fn kind_of(&self, eid: Eid, base: &str) -> bool {
        meta::is_kind_of(self.ty(eid), base)
    }
    pub fn raw(&self, eid: Eid, key: &str) -> Option<&Value> {
        match self.e(eid).fields.get(key) {
            Some(F::V(v)) => Some(v),
            _ => None,
        }
    }
    /// A field's value, or its metamodel default.
    pub fn val(&self, eid: Eid, key: &str) -> Option<Value> {
        self.raw(eid, key).cloned().or_else(|| meta::default_of(self.ty(eid), key))
    }
    pub fn str(&self, eid: Eid, key: &str) -> String {
        match self.val(eid, key) {
            Some(Value::String(s)) => s,
            _ => String::new(),
        }
    }
    pub fn name(&self, eid: Eid) -> String {
        self.str(eid, "name")
    }
    pub fn bool(&self, eid: Eid, key: &str) -> bool {
        matches!(self.val(eid, key), Some(Value::Bool(true)))
    }
    pub fn f(&self, eid: Eid, key: &str) -> f64 {
        self.val(eid, key).and_then(|v| v.as_f64()).unwrap_or(0.0)
    }
    pub fn has(&self, eid: Eid, key: &str) -> bool {
        self.e(eid).fields.contains_key(key)
    }
    pub fn by_ref(&self, v: &Value) -> Option<Eid> {
        let id = v.as_object()?.get("$ref")?.as_str()?;
        self.by_id.get(id).copied().filter(|e| self.alive(*e))
    }
    pub fn refd(&self, eid: Eid, key: &str) -> Option<Eid> {
        self.raw(eid, key).and_then(|v| self.by_ref(v))
    }
    pub fn child(&self, eid: Eid, key: &str) -> Option<Eid> {
        match self.e(eid).fields.get(key) {
            Some(F::One(c)) => Some(*c),
            _ => None,
        }
    }
    pub fn list(&self, eid: Eid, key: &str) -> Vec<Eid> {
        match self.e(eid).fields.get(key) {
            Some(F::Many(v)) => v.clone(),
            Some(F::One(c)) => vec![*c],
            _ => vec![],
        }
    }
    pub fn parent(&self, eid: Eid) -> Option<Eid> {
        self.e(eid).parent
    }
    pub fn id(&self, eid: Eid) -> String {
        self.e(eid).id.clone()
    }
    pub fn ref_to(&self, eid: Eid) -> Value {
        rf(&self.e(eid).id)
    }
    pub fn children(&self, eid: Eid) -> Vec<(String, Eid)> {
        let mut out = vec![];
        for (k, f) in &self.e(eid).fields {
            match f {
                F::One(c) => out.push((k.clone(), *c)),
                F::Many(cs) => out.extend(cs.iter().map(|c| (k.clone(), *c))),
                F::V(_) => {}
            }
        }
        out
    }
    pub fn descendants(&self, eid: Eid, out: &mut Vec<Eid>) {
        out.push(eid);
        for (_, c) in self.children(eid) {
            self.descendants(c, out);
        }
    }
    pub fn all(&self) -> impl Iterator<Item = Eid> + '_ {
        self.elems.iter().enumerate().filter(|(_, e)| e.is_some()).map(|(i, _)| i)
    }
    pub fn all_of(&self, base: &str) -> Vec<Eid> {
        self.all().filter(|e| self.kind_of(*e, base) || self.ty(*e) == base).collect()
    }

    // ------------------------------------------------------------ writing, in the file's key order
    fn insert_ordered(&mut self, eid: Eid, key: &str, f: F) {
        let ty = self.ty(eid).to_string();
        let el = self.em(eid);
        if let Some(slot) = el.fields.get_mut(key) {
            *slot = f;
            return;
        }
        let Some(pos) = meta::order_of(&ty, key) else {
            el.fields.insert(key.to_string(), f);
            return;
        };
        let at = el
            .fields
            .keys()
            .position(|k| meta::order_of(&ty, k).map(|p| p > pos).unwrap_or(false))
            .unwrap_or(el.fields.len());
        el.fields.shift_insert(at, key.to_string(), f);
    }

    /// Set a plain field. Values equal to the default, empty strings with a default and null are left out.
    pub fn set(&mut self, eid: Eid, key: &str, v: Value) {
        let d = meta::default_of(self.ty(eid), key);
        let omit = v.is_null()
            || (v.as_str() == Some("") && d.is_some())
            || d.as_ref().map(|d| json_eq(d, &v)).unwrap_or(false);
        if omit {
            self.em(eid).fields.shift_remove(key);
        } else {
            self.insert_ordered(eid, key, F::V(v));
        }
    }
    /// Set a field exactly (also null or default), keeping order.
    pub fn set_raw(&mut self, eid: Eid, key: &str, v: Value) {
        self.insert_ordered(eid, key, F::V(v));
    }
    pub fn remove(&mut self, eid: Eid, key: &str) {
        self.em(eid).fields.shift_remove(key);
    }
    pub fn set_ref(&mut self, eid: Eid, key: &str, target: Option<Eid>) {
        match target {
            Some(t) => {
                let r = self.ref_to(t);
                self.insert_ordered(eid, key, F::V(r));
            }
            None => self.remove(eid, key),
        }
    }

    fn alloc(&mut self, ty: &str, parent: Option<Eid>) -> Eid {
        let id = new_id();
        let eid = self.elems.len();
        let mut fields = IndexMap::new();
        fields.insert("_id".to_string(), F::V(json!(id)));
        if let Some(p) = parent {
            fields.insert("_parent".to_string(), F::V(rf(&self.e(p).id)));
        }
        self.elems.push(Some(Elem { ty: ty.to_string(), id: id.clone(), parent, fields }));
        self.by_id.insert(id, eid);
        eid
    }

    /// Build a new element with keys in file order and attach it to parent[field].
    pub fn create(&mut self, ty: &str, props: &[(&str, Value)], parent: Option<(Eid, &str)>) -> Eid {
        let eid = self.alloc(ty, parent.map(|p| p.0));
        for (k, v) in props {
            self.set(eid, k, v.clone());
        }
        if let Some((p, field)) = parent {
            self.attach(eid, p, field, None);
        }
        eid
    }

    pub fn attach(&mut self, eid: Eid, parent: Eid, field: &str, index: Option<usize>) {
        let pref = self.ref_to(parent);
        self.em(eid).parent = Some(parent);
        self.em(eid).fields.insert("_parent".into(), F::V(pref));
        if field == "end1" || field == "end2" {
            self.insert_ordered(parent, field, F::One(eid));
            return;
        }
        match self.em(parent).fields.get_mut(field) {
            Some(F::Many(v)) => match index {
                Some(i) if i <= v.len() => v.insert(i, eid),
                _ => v.push(eid),
            },
            _ => self.insert_ordered(parent, field, F::Many(vec![eid])),
        }
    }

    /// Remove an element (and everything inside it) from its owner.
    pub fn detach(&mut self, eid: Eid) {
        if let Some(p) = self.parent(eid) {
            if self.alive(p) {
                let mut empty = None;
                for (k, f) in self.em(p).fields.iter_mut() {
                    match f {
                        F::Many(v) => {
                            if let Some(i) = v.iter().position(|x| *x == eid) {
                                v.remove(i);
                                if v.is_empty() {
                                    empty = Some(k.clone());
                                }
                                break;
                            }
                        }
                        F::One(c) if *c == eid => {
                            empty = Some(k.clone());
                            break;
                        }
                        _ => {}
                    }
                }
                if let Some(k) = empty {
                    self.em(p).fields.shift_remove(&k);
                }
            }
        }
        let mut all = vec![];
        self.descendants(eid, &mut all);
        for x in all {
            let id = self.e(x).id.clone();
            self.by_id.remove(&id);
            self.elems[x] = None;
        }
    }

    /// Reorder a Many field (e.g. subViews) to the given order.
    pub fn set_list(&mut self, eid: Eid, key: &str, items: Vec<Eid>) {
        if items.is_empty() {
            self.remove(eid, key);
        } else {
            self.insert_ordered(eid, key, F::Many(items));
        }
    }

    /// Remove every reference to the given ids (single refs dropped, ref lists filtered).
    pub fn clear_refs_to(&mut self, ids: &HashSet<String>) {
        let all: Vec<Eid> = self.all().collect();
        for eid in all {
            let el = self.em(eid);
            let mut drop = vec![];
            for (k, f) in el.fields.iter_mut() {
                if k == "_parent" {
                    continue;
                }
                if let F::V(v) = f {
                    if let Some(id) = v.get("$ref").and_then(|x| x.as_str()) {
                        if ids.contains(id) {
                            drop.push(k.clone());
                        }
                    } else if let Value::Array(a) = v {
                        if a.iter().any(|x| x.get("$ref").is_some()) {
                            a.retain(|x| x.get("$ref").and_then(|r| r.as_str()).map(|r| !ids.contains(r)).unwrap_or(true));
                            if a.is_empty() {
                                drop.push(k.clone());
                            }
                        }
                    }
                }
            }
            for k in drop {
                el.fields.shift_remove(&k);
            }
        }
    }

    pub fn path(&self, eid: Eid) -> Vec<Eid> {
        let mut out = vec![eid];
        let mut cur = self.parent(eid);
        while let Some(p) = cur {
            out.insert(0, p);
            cur = self.parent(p);
        }
        out
    }
}

pub fn json_eq(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}
