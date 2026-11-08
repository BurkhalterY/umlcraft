//! DDL for MySQL, SQLite and PostgreSQL: `CREATE TABLE` statements from an ER data model, and ER
//! data models from `.sql`/`.ddl` files (language-support page; the statement layout follows the
//! 7.x ERD extension: columns, then the primary key, then the foreign keys).

use super::lex::{self, Cfg, K, T};
use super::*;
use crate::layout::Dir;
use crate::views::R;

fn quote(o: &GenOpts, n: &str) -> String {
    if !o.quote {
        return n.to_string();
    }
    match o.dbms.as_str() {
        "mysql" => format!("`{n}`"),
        _ => format!("\"{n}\""),
    }
}

/// The file `<data model>.sql` with a table per entity.
pub fn generate(r: &Repo, model: Eid, o: &GenOpts) -> Vec<OutFile> {
    let entities: Vec<Eid> = r.list(model, "ownedElements").into_iter().filter(|e| r.ty(*e) == "ERDEntity").collect();
    let mut w = W::new(o);
    let mut fks = vec![];
    for &e in &entities {
        let table = r.name(e);
        w.doc(o, &r.str(e, "documentation"), DocStyle::Dashes);
        if o.drop_table {
            w.line(&format!("DROP TABLE IF EXISTS {};", quote(o, &table)));
        }
        w.line(&format!("CREATE TABLE {} (", quote(o, &table)));
        w.inc();
        let cols = r.list(e, "columns");
        let mut lines = vec![];
        for &c in &cols {
            let mut s = format!("{} {}", quote(o, &r.name(c)), sql_type(r, c));
            if !r.bool(c, "nullable") || r.bool(c, "primaryKey") {
                s.push_str(" NOT NULL");
            }
            if r.bool(c, "unique") && !r.bool(c, "primaryKey") {
                s.push_str(" UNIQUE");
            }
            if o.doc && o.dbms == "mysql" && !r.str(c, "documentation").is_empty() {
                s.push_str(&format!(" COMMENT '{}'", r.str(c, "documentation").replace('\'', "''")));
            }
            lines.push(s);
        }
        let pk: Vec<String> = cols.iter().filter(|c| r.bool(**c, "primaryKey")).map(|c| quote(o, &r.name(*c))).collect();
        if !pk.is_empty() {
            lines.push(format!("PRIMARY KEY ({})", pk.join(", ")));
        }
        for &c in &cols {
            let Some(rc) = r.refd(c, "referenceTo") else { continue };
            let Some(rt) = r.parent(rc) else { continue };
            let fk = format!("FOREIGN KEY ({}) REFERENCES {} ({})", quote(o, &r.name(c)), quote(o, &r.name(rt)), quote(o, &r.name(rc)));
            // SQLite cannot add a constraint later: its foreign keys stay in the table
            if o.dbms == "sqlite" {
                lines.push(fk);
            } else {
                fks.push(format!("ALTER TABLE {} ADD {fk};", quote(o, &table)));
            }
        }
        let n = lines.len();
        for (i, l) in lines.into_iter().enumerate() {
            w.line(&if i + 1 < n { format!("{l},") } else { l });
        }
        w.dec();
        w.line(");");
        w.blank();
    }
    for f in fks {
        w.line(&f);
    }
    vec![(PathBuf::from(format!("{}.sql", ident(&r.name(model)))), w.out)]
}

fn sql_type(r: &Repo, c: Eid) -> String {
    let t = r.str(c, "type");
    let t = if t.is_empty() { "VARCHAR".to_string() } else { t };
    let len = match r.raw(c, "length") {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) if n.as_f64() != Some(0.0) => n.to_string(),
        _ => String::new(),
    };
    if len.is_empty() || t.contains('(') { t } else { format!("{t}({len})") }
}

// ------------------------------------------------------------------ reverse

#[derive(Default, Debug, Clone)]
pub struct Column {
    pub name: String,
    pub ty: String,
    pub length: String,
    pub pk: bool,
    pub nullable: bool,
    pub unique: bool,
    pub refs: Option<(String, String)>,
    pub doc: String,
}

#[derive(Default, Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

fn unquote(x: &T) -> String {
    let s = x.s.as_str();
    if x.k == K::Str && s.len() >= 2 {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

/// The last part of `schema.table` (quoted or not), and the index after it.
fn qualified(t: &[T], mut i: usize) -> (String, usize) {
    let mut name = String::new();
    while i < t.len() {
        if t[i].is("[") {
            let j = lex::close(t, i);
            name = lex::join(&t[i + 1..j]);
            i = j + 1;
        } else if t[i].id() || t[i].k == K::Str {
            name = unquote(&t[i]);
            i += 1;
        } else {
            break;
        }
        if i < t.len() && t[i].is(".") {
            i += 1;
        } else {
            break;
        }
    }
    (name, i)
}

fn names_in(t: &[T]) -> Vec<String> {
    lex::commas(t).iter().filter_map(|p| p.first()).map(unquote).collect()
}

fn word(x: &T, w: &str) -> bool {
    x.id() && x.s.eq_ignore_ascii_case(w)
}

/// The tables of a DDL script; returns them and whether it had errors (unbalanced brackets).
pub fn parse(src: &str) -> (Vec<Table>, bool) {
    let t = lex::lex(src, Cfg { sq_strings: true, dash_comment: true, hash_comment: true, ..Default::default() });
    let mut tables: Vec<Table> = vec![];
    let mut later: Vec<(String, Vec<String>, String, Vec<String>)> = vec![];
    for st in lex::statements(&t, false) {
        let h = st.head;
        if h.len() < 3 {
            continue;
        }
        if word(&h[0], "create") {
            let Some(k) = h.iter().position(|x| word(x, "table")) else { continue };
            let mut i = k + 1;
            if i + 2 < h.len() && word(&h[i], "if") {
                i += 3;
            }
            let (name, i) = qualified(h, i);
            let Some(open) = h[i..].iter().position(|x| x.is("(")).map(|p| p + i) else { continue };
            let close = lex::close(h, open);
            let mut table = Table { name: name.clone(), columns: vec![] };
            let mut pks = vec![];
            let mut uniques = vec![];
            for def in lex::commas(&h[open + 1..close]) {
                let mut def = def;
                if word(&def[0], "constraint") {
                    def = &def[2.min(def.len())..];
                }
                if def.is_empty() {
                    continue;
                }
                let paren = def.iter().position(|x| x.is("("));
                let cols_at = |p: usize| names_in(&def[p + 1..lex::close(def, p)]);
                if word(&def[0], "primary") {
                    if let Some(p) = paren {
                        pks.extend(cols_at(p));
                    }
                } else if word(&def[0], "unique") {
                    if let Some(p) = paren {
                        uniques.extend(cols_at(p));
                    }
                } else if word(&def[0], "foreign") {
                    if let Some(p) = paren {
                        let cols = cols_at(p);
                        let rest = &def[lex::close(def, p) + 1..];
                        if let Some(ri) = rest.iter().position(|x| word(x, "references")) {
                            let (rt, j) = qualified(rest, ri + 1);
                            let rcols = if j < rest.len() && rest[j].is("(") { names_in(&rest[j + 1..lex::close(rest, j)]) } else { vec![] };
                            later.push((name.clone(), cols, rt, rcols));
                        }
                    }
                } else if ["key", "index", "check", "fulltext", "spatial"].iter().any(|w| word(&def[0], w)) {
                } else {
                    let c = column(def, &name, &mut later);
                    table.columns.push(c);
                }
            }
            for c in table.columns.iter_mut() {
                if pks.contains(&c.name) {
                    c.pk = true;
                }
                if uniques.contains(&c.name) {
                    c.unique = true;
                }
            }
            match tables.iter_mut().find(|x| x.name == table.name) {
                Some(x) => x.columns.extend(table.columns),
                None => tables.push(table),
            }
        } else if word(&h[0], "alter") && h.iter().any(|x| word(x, "foreign")) {
            let Some(k) = h.iter().position(|x| word(x, "table")) else { continue };
            let mut i = k + 1;
            if i < h.len() && word(&h[i], "only") {
                i += 1;
            }
            let (name, _) = qualified(h, i);
            let f = h.iter().position(|x| word(x, "foreign")).unwrap();
            let Some(p) = h[f..].iter().position(|x| x.is("(")).map(|p| p + f) else { continue };
            let q = lex::close(h, p);
            let cols = names_in(&h[p + 1..q]);
            if let Some(ri) = h[q..].iter().position(|x| word(x, "references")).map(|x| x + q) {
                let (rt, j) = qualified(h, ri + 1);
                let rcols = if j < h.len() && h[j].is("(") { names_in(&h[j + 1..lex::close(h, j)]) } else { vec![] };
                later.push((name, cols, rt, rcols));
            }
        }
    }
    for (table, cols, rt, rcols) in later {
        let Some(tb) = tables.iter_mut().find(|x| x.name == table) else { continue };
        for (i, c) in cols.iter().enumerate() {
            if let Some(col) = tb.columns.iter_mut().find(|x| &x.name == c) {
                col.refs = Some((rt.clone(), rcols.get(i).cloned().unwrap_or_default()));
            }
        }
    }
    (tables, !lex::balanced(&t))
}

fn column(def: &[T], table: &str, later: &mut Vec<(String, Vec<String>, String, Vec<String>)>) -> Column {
    let mut c = Column { name: unquote(&def[0]), nullable: true, ..Default::default() };
    let mut i = 1;
    let mut ty = vec![];
    while i < def.len() && (def[i].id() || def[i].k == K::Num) && !["not", "null", "primary", "unique", "references", "default", "auto_increment", "autoincrement", "comment", "constraint", "check", "generated", "collate", "character"].iter().any(|w| word(&def[i], w)) {
        ty.push(def[i].s.clone());
        i += 1;
    }
    c.ty = ty.join(" ").to_uppercase();
    if i < def.len() && def[i].is("(") {
        let j = lex::close(def, i);
        c.length = lex::join(&def[i + 1..j]).replace(' ', "");
        i = j + 1;
    }
    while i < def.len() {
        let x = &def[i];
        if word(x, "not") && def.get(i + 1).map(|y| word(y, "null")).unwrap_or(false) {
            c.nullable = false;
            i += 1;
        } else if word(x, "primary") {
            c.pk = true;
            c.nullable = false;
        } else if word(x, "unique") {
            c.unique = true;
        } else if word(x, "comment") {
            if let Some(s) = def.get(i + 1) {
                c.doc = unquote(s);
            }
        } else if word(x, "references") {
            let (rt, j) = qualified(def, i + 1);
            let rcols = if j < def.len() && def[j].is("(") { names_in(&def[j + 1..lex::close(def, j)]) } else { vec![] };
            later.push((table.to_string(), vec![c.name.clone()], rt, rcols));
        }
        i += 1;
    }
    c
}

/// Import the tables into a data model (the target, or a new `DDLReverse` one in the project when
/// `new_model`): entities and columns merged by name, a foreign key marked on its column with its
/// referenced column, and an ER relationship from the referenced table (1) to the referencing one
/// (0..*, identifying when the key is also primary). Returns the data model and the table count.
pub fn import(a: &mut App, target: Eid, tables: &[Table], new_model: bool, _o: &RevOpts) -> (Eid, usize) {
    let r = &mut a.repo;
    let model = if new_model || r.ty(target) != "ERDDataModel" { r.create("ERDDataModel", &[("name", json!(Lang::Ddl.reverse_model()))], Some((target, "ownedElements"))) } else { target };
    let mut ents: Vec<(String, Eid)> = r.list(model, "ownedElements").into_iter().filter(|e| r.ty(*e) == "ERDEntity").map(|e| (r.name(e), e)).collect();
    for t in tables {
        let e = match ents.iter().find(|x| x.0 == t.name) {
            Some(x) => x.1,
            None => {
                let e = r.create("ERDEntity", &[("name", json!(t.name))], Some((model, "ownedElements")));
                ents.push((t.name.clone(), e));
                e
            }
        };
        for c in &t.columns {
            if r.list(e, "columns").into_iter().any(|x| r.name(x) == c.name) {
                continue;
            }
            let col = r.create("ERDColumn", &[("name", json!(c.name)), ("type", json!(c.ty))], Some((e, "columns")));
            if !c.length.is_empty() {
                r.set(col, "length", json!(c.length));
            }
            for (k, v) in [("primaryKey", c.pk), ("nullable", c.nullable && !c.pk), ("unique", c.unique)] {
                if v {
                    r.set(col, k, json!(true));
                }
            }
            if !c.doc.is_empty() {
                r.set(col, "documentation", json!(c.doc));
            }
        }
    }
    for t in tables {
        let Some(e) = ents.iter().find(|x| x.0 == t.name).map(|x| x.1) else { continue };
        for c in &t.columns {
            let Some((rt, rc)) = &c.refs else { continue };
            let Some(te) = ents.iter().find(|x| &x.0 == rt).map(|x| x.1) else { continue };
            let Some(col) = r.list(e, "columns").into_iter().find(|x| r.name(*x) == c.name) else { continue };
            r.set(col, "foreignKey", json!(true));
            let target_col = r.list(te, "columns").into_iter().find(|x| &r.name(*x) == rc).or_else(|| r.list(te, "columns").into_iter().find(|x| r.bool(*x, "primaryKey")));
            if let Some(tc) = target_col {
                r.set_ref(col, "referenceTo", Some(tc));
            }
            let rel = r.create("ERDRelationship", &[], Some((e, "ownedElements")));
            if !c.pk {
                r.set(rel, "identifying", json!(false));
            }
            for (k, ent, card) in [("end1", te, "1"), ("end2", e, "0..*")] {
                let rf = r.ref_to(ent);
                let end = r.create("ERDRelationshipEnd", &[("reference", rf)], Some((rel, k)));
                r.set(end, "cardinality", json!(card));
            }
        }
    }
    (model, tables.len())
}

/// An ER diagram of the data model's entities and relationships, laid out.
pub fn er_diagram(a: &mut App, model: Eid, ms: &dyn Measure) -> Option<Eid> {
    if a.repo.ty(model) != "ERDDataModel" {
        return None;
    }
    let undo = a.undo.len();
    let d = a.change(|a| {
        let d = a.repo.create("ERDDiagram", &[("name", json!("ER Diagram"))], Some((model, "ownedElements")));
        let ents: Vec<Eid> = a.repo.list(model, "ownedElements").into_iter().filter(|e| a.repo.ty(*e) == "ERDEntity").collect();
        let mut placed = vec![];
        for (i, &m) in ents.iter().enumerate() {
            let v = crate::erd::create_view(&mut a.repo, d, m, R { x: 10.0 + i as f64, y: 10.0, w: 1.0, h: 1.0 });
            a.apply_view_defaults(v);
            placed.push((m, v));
        }
        for &(m, _) in &placed {
            let rels: Vec<Eid> = a.repo.list(m, "ownedElements").into_iter().filter(|x| a.repo.ty(*x) == "ERDRelationship").collect();
            for rel in rels {
                let ends: Vec<Option<Eid>> = ["end1", "end2"].iter().map(|k| a.repo.child(rel, k).and_then(|e| a.repo.refd(e, "reference"))).collect();
                let v = |e: Option<Eid>| e.and_then(|e| placed.iter().find(|p| p.0 == e)).map(|p| p.1);
                if let (Some(x), Some(y)) = (v(ends[0]), v(ends[1])) {
                    if let Some(ev) = views::create_edge_view(&mut a.repo, d, Some(rel), x, y) {
                        a.apply_view_defaults(ev);
                    }
                }
            }
        }
        d
    });
    Some(a.finish_mermaid(d, undo, Dir::TopBottom, ms))
}
