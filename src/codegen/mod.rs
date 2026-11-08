//! Tools › <Language>: code generation from the UML model and reverse engineering of source
//! folders into it, one dialog per language with a Generate and a Reverse tab, and DDL from and to
//! ER data models (StarUML 8, https://docs.staruml.io/user-guide/language-support/). The rules
//! follow that page and StarUML's MIT-licensed 7.x generators (staruml-java and the others); the
//! code is UMLCraft's own. Both directions go through a small language-neutral form (`Pkg`, `Ty`).

pub mod lex;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_pro;

pub mod c;
pub mod cpp;
pub mod csharp;
pub mod dart;
pub mod ddl;
pub mod go;
pub mod java;
pub mod javascript;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;
pub mod typescript;

use crate::app::App;
use crate::model::{Eid, Repo};
use crate::views::{self, Measure};
use serde_json::json;
use std::path::{Path, PathBuf};

// ------------------------------------------------------------------ languages

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Java,
    Cpp,
    CSharp,
    C,
    Python,
    Php,
    TypeScript,
    JavaScript,
    Rust,
    Go,
    Ruby,
    Kotlin,
    Dart,
    Swift,
    Ddl,
}

impl Lang {
    /// Tools menu order (language-support page).
    pub const ALL: [Lang; 15] = [
        Lang::Java,
        Lang::Cpp,
        Lang::CSharp,
        Lang::C,
        Lang::Python,
        Lang::Php,
        Lang::TypeScript,
        Lang::JavaScript,
        Lang::Rust,
        Lang::Go,
        Lang::Ruby,
        Lang::Kotlin,
        Lang::Dart,
        Lang::Swift,
        Lang::Ddl,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Lang::Java => "Java",
            Lang::Cpp => "C++",
            Lang::CSharp => "C#",
            Lang::C => "C",
            Lang::Python => "Python",
            Lang::Php => "PHP",
            Lang::TypeScript => "TypeScript",
            Lang::JavaScript => "JavaScript",
            Lang::Rust => "Rust",
            Lang::Go => "Go",
            Lang::Ruby => "Ruby",
            Lang::Kotlin => "Kotlin",
            Lang::Dart => "Dart",
            Lang::Swift => "Swift",
            Lang::Ddl => "DDL",
        }
    }

    /// The preference prefix (`java.gen.indentSpaces`, as the 7.x extensions name them).
    pub fn key(self) -> &'static str {
        match self {
            Lang::Java => "java",
            Lang::Cpp => "cpp",
            Lang::CSharp => "csharp",
            Lang::C => "c",
            Lang::Python => "python",
            Lang::Php => "php",
            Lang::TypeScript => "typescript",
            Lang::JavaScript => "javascript",
            Lang::Rust => "rust",
            Lang::Go => "go",
            Lang::Ruby => "ruby",
            Lang::Kotlin => "kotlin",
            Lang::Dart => "dart",
            Lang::Swift => "swift",
            Lang::Ddl => "ddl",
        }
    }

    /// The name of the model a reverse makes when "(New model)" is chosen: `JavaReverse`, …
    pub fn reverse_model(self) -> String {
        let n = match self {
            Lang::Cpp => "Cpp",
            Lang::CSharp => "CSharp",
            l => l.label(),
        };
        format!("{n}Reverse")
    }

    /// The file extensions a reverse reads (language-support page).
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Lang::Java => &["java"],
            Lang::Cpp => &["h", "hh", "hpp", "hxx", "h++", "cpp", "cc", "cxx", "c++", "ipp", "tpp"],
            Lang::CSharp => &["cs"],
            Lang::C => &["h", "c"],
            Lang::Python => &["py", "pyi"],
            Lang::Php => &["php"],
            Lang::TypeScript => &["ts", "tsx", "mts", "cts"],
            Lang::JavaScript => &["js", "mjs", "cjs", "jsx"],
            Lang::Rust => &["rs"],
            Lang::Go => &["go"],
            Lang::Ruby => &["rb"],
            Lang::Kotlin => &["kt"],
            Lang::Dart => &["dart"],
            Lang::Swift => &["swift"],
            Lang::Ddl => &["sql", "ddl"],
        }
    }

    /// The default indentation: 4 spaces, 2 for Ruby, Dart, TypeScript and JavaScript.
    fn default_indent(self) -> i64 {
        match self {
            Lang::Ruby | Lang::Dart | Lang::TypeScript | Lang::JavaScript => 2,
            _ => 4,
        }
    }
}

// ------------------------------------------------------------------ the language-neutral form

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Kind {
    #[default]
    Class,
    Interface,
    Enum,
}

#[derive(Clone, Debug, Default)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub vis: String,
    pub is_static: bool,
    pub read_only: bool,
    pub default: String,
    pub mult: String,
    pub doc: String,
}

#[derive(Clone, Debug, Default)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub default: String,
}

#[derive(Clone, Debug, Default)]
pub struct Method {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: String,
    pub vis: String,
    pub is_static: bool,
    pub is_abstract: bool,
    pub doc: String,
}

#[derive(Clone, Debug, Default)]
pub struct Ty {
    pub kind: Kind,
    pub name: String,
    pub vis: String,
    pub is_abstract: bool,
    pub doc: String,
    pub stereotype: String,
    /// generalization targets (and, from reverse, any super type whose kind is decided later)
    pub supers: Vec<String>,
    /// interface realization targets
    pub reals: Vec<String>,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub literals: Vec<String>,
    pub nested: Vec<Ty>,
    /// generation only: the package path from the base package, for imports
    pub ns: Vec<String>,
}

impl Ty {
    pub fn new(kind: Kind, name: &str) -> Ty {
        Ty { kind, name: name.to_string(), vis: "public".into(), ..Default::default() }
    }
}

/// A package to generate: its types and sub-packages, `path` from the base package.
#[derive(Clone, Debug, Default)]
pub struct Pkg {
    pub name: String,
    pub path: Vec<String>,
    pub types: Vec<Ty>,
    pub subs: Vec<Pkg>,
}

impl Pkg {
    /// Every package of the tree, this one first.
    pub fn all(&self) -> Vec<&Pkg> {
        let mut out = vec![self];
        for s in &self.subs {
            out.extend(s.all());
        }
        out
    }
    /// Where each type of the tree lives: (name, package path).
    pub fn type_paths(&self) -> Vec<(String, Vec<String>)> {
        self.all().iter().flat_map(|p| p.types.iter().map(|t| (t.name.clone(), p.path.clone()))).collect()
    }
}

/// What one reversed file declares: types under a namespace path.
#[derive(Clone, Debug, Default)]
pub struct Unit {
    pub ns: Vec<String>,
    pub types: Vec<Ty>,
}

/// A reversed file: its units and whether it had syntax errors (the rest is still read).
pub struct Parsed {
    pub units: Vec<Unit>,
    pub errors: bool,
}

// ------------------------------------------------------------------ options

#[derive(Clone, Debug)]
pub struct GenOpts {
    pub use_tab: bool,
    pub indent_spaces: i64,
    pub doc: bool,
    /// DDL: "mysql", "sqlite" or "postgresql"
    pub dbms: String,
    pub quote: bool,
    pub drop_table: bool,
}

impl GenOpts {
    pub fn indent(&self) -> String {
        if self.use_tab { "\t".into() } else { " ".repeat(self.indent_spaces.clamp(0, 16) as usize) }
    }
}

#[derive(Clone, Debug)]
pub struct RevOpts {
    pub association: bool,
    pub public_only: bool,
    pub type_hierarchy: bool,
    pub package_overview: bool,
    pub package_structure: bool,
}

/// The options as preferences: `<lang>.gen.*` and `<lang>.rev.*` (the 7.x extensions' keys).
pub fn gen_opts(prefs: &crate::prefs::Prefs, l: Lang) -> GenOpts {
    let k = l.key();
    let int = |n: &str, d: i64| prefs.get(&format!("{k}.gen.{n}")).and_then(|v| v.as_i64()).unwrap_or(d);
    let st = |n: &str, d: &str| prefs.get(&format!("{k}.gen.{n}")).and_then(|v| v.as_str()).unwrap_or(d).to_string();
    GenOpts {
        // Go is indented with tabs (gofmt)
        use_tab: prefs.bool(&format!("{k}.gen.useTab"), l == Lang::Go),
        indent_spaces: int("indentSpaces", l.default_indent()),
        doc: prefs.bool(&format!("{k}.gen.doc"), true),
        dbms: st("dbms", "mysql"),
        quote: prefs.bool(&format!("{k}.gen.quoteIdentifiers"), true),
        drop_table: prefs.bool(&format!("{k}.gen.dropTable"), false),
    }
}

pub fn rev_opts(prefs: &crate::prefs::Prefs, l: Lang) -> RevOpts {
    let k = l.key();
    let b = |n: &str, d: bool| prefs.bool(&format!("{k}.rev.{n}"), d);
    RevOpts {
        association: b("association", true),
        public_only: b("publicOnly", false),
        // DDL: an ER diagram instead of a type hierarchy
        type_hierarchy: b(if l == Lang::Ddl { "erDiagram" } else { "typeHierarchy" }, true),
        package_overview: b("packageOverview", true),
        package_structure: b("packageStructure", true),
    }
}

fn store_opts(prefs: &mut crate::prefs::Prefs, l: Lang, g: &GenOpts, r: &RevOpts) {
    let k = l.key();
    prefs.set(&format!("{k}.gen.useTab"), json!(g.use_tab));
    prefs.set(&format!("{k}.gen.indentSpaces"), json!(g.indent_spaces));
    prefs.set(&format!("{k}.gen.doc"), json!(g.doc));
    if l == Lang::Ddl {
        prefs.set(&format!("{k}.gen.dbms"), json!(g.dbms));
        prefs.set(&format!("{k}.gen.quoteIdentifiers"), json!(g.quote));
        prefs.set(&format!("{k}.gen.dropTable"), json!(g.drop_table));
    } else {
        prefs.set(&format!("{k}.rev.association"), json!(r.association));
        prefs.set(&format!("{k}.rev.publicOnly"), json!(r.public_only));
        prefs.set(&format!("{k}.rev.packageOverview"), json!(r.package_overview));
        prefs.set(&format!("{k}.rev.packageStructure"), json!(r.package_structure));
    }
    prefs.set(&format!("{k}.rev.{}", if l == Lang::Ddl { "erDiagram" } else { "typeHierarchy" }), json!(r.type_hierarchy));
    prefs.flush();
}

// ------------------------------------------------------------------ writing source text

/// Source text with an indentation level.
pub struct W {
    pub out: String,
    unit: String,
    level: usize,
}

impl W {
    pub fn new(o: &GenOpts) -> W {
        W { out: String::new(), unit: o.indent(), level: 0 }
    }
    pub fn line(&mut self, s: &str) {
        for l in s.split('\n') {
            if !l.is_empty() {
                for _ in 0..self.level {
                    self.out.push_str(&self.unit);
                }
                self.out.push_str(l);
            }
            self.out.push('\n');
        }
    }
    pub fn blank(&mut self) {
        if !self.out.is_empty() && !self.out.ends_with("\n\n") && !self.out.ends_with("{\n") && !self.out.ends_with(":\n") {
            self.out.push('\n');
        }
    }
    pub fn inc(&mut self) {
        self.level += 1;
    }
    pub fn dec(&mut self) {
        self.level = self.level.saturating_sub(1);
    }
    /// A documentation comment: `/** … */`, `///`, `#` or `"""` style.
    pub fn doc(&mut self, o: &GenOpts, text: &str, style: DocStyle) {
        let text = text.trim();
        if !o.doc || text.is_empty() {
            return;
        }
        match style {
            DocStyle::Block => {
                self.line("/**");
                for l in text.lines() {
                    self.line(&format!(" * {l}").trim_end().to_string());
                }
                self.line(" */");
            }
            DocStyle::Slashes => {
                for l in text.lines() {
                    self.line(format!("/// {l}").trim_end());
                }
            }
            DocStyle::Hash => {
                for l in text.lines() {
                    self.line(format!("# {l}").trim_end());
                }
            }
            DocStyle::DocString => {
                if text.contains('\n') {
                    self.line("\"\"\"");
                    self.line(text);
                    self.line("\"\"\"");
                } else {
                    self.line(&format!("\"\"\"{text}\"\"\""));
                }
            }
            DocStyle::Dashes => {
                for l in text.lines() {
                    self.line(format!("-- {l}").trim_end());
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum DocStyle {
    Block,
    Slashes,
    Hash,
    DocString,
    Dashes,
}


/// A multiplicity that holds many (`*`, `0..*`, `1..*`, `2..5`).
pub fn is_many(m: &str) -> bool {
    let m = m.trim();
    if m.is_empty() {
        return false;
    }
    let upper = m.rsplit("..").next().unwrap_or(m).trim();
    upper == "*" || upper.parse::<i64>().map(|n| n > 1).unwrap_or(false)
}

pub fn is_optional(m: &str) -> bool {
    m.trim() == "0..1"
}

/// The UML primitive names (String, Integer, Boolean, Real, UnlimitedNatural) in a language's
/// terms: string, int, bool, double…; other names are kept.
pub fn primitive(t: &str, string: &str, int: &str, boolean: &str, real: &str) -> String {
    match t {
        "String" => string.into(),
        "Integer" | "UnlimitedNatural" => int.into(),
        "Boolean" => boolean.into(),
        "Real" => real.into(),
        _ => t.into(),
    }
}

pub fn lc_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

pub fn uc_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// `LibraryItem` → `library_item` (Python, Ruby, Rust and Dart file names).
pub fn snake(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).map(|n| n.is_lowercase()).unwrap_or(false);
            if i > 0 && (prev_lower || (next_lower && chars[i - 1].is_uppercase())) && !out.ends_with('_') {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else if c == ' ' || c == '-' {
            out.push('_');
        } else {
            out.push(c);
        }
    }
    out
}

/// A name usable as an identifier: spaces become underscores.
pub fn ident(s: &str) -> String {
    let s: String = s.trim().chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '$' { c } else { '_' }).collect();
    if s.is_empty() { "_".into() } else { s }
}

// ------------------------------------------------------------------ model → neutral form

fn classifier_kind(ty: &str) -> Option<Kind> {
    match ty {
        "UMLClass" | "UMLDataType" | "UMLPrimitiveType" | "UMLSignal" | "UMLComponent" => Some(Kind::Class),
        "UMLInterface" => Some(Kind::Interface),
        "UMLEnumeration" => Some(Kind::Enum),
        _ => None,
    }
}

/// The base package's tree as the neutral form. The base itself is the output folder: its
/// sub-packages become sub-folders (CodeGenerator.generate in the 7.x extensions).
pub fn gen_tree(r: &Repo, base: Eid) -> Pkg {
    let rels: Vec<Eid> = r.all().filter(|e| matches!(r.ty(*e), "UMLGeneralization" | "UMLInterfaceRealization" | "UMLAssociation")).collect();
    fn pkg(r: &Repo, e: Eid, path: Vec<String>, rels: &[Eid]) -> Pkg {
        let mut p = Pkg { name: r.name(e), path: path.clone(), ..Default::default() };
        for c in r.list(e, "ownedElements") {
            if classifier_kind(r.ty(c)).is_some() {
                let mut t = ty_of(r, c, rels);
                t.ns = path.clone();
                p.types.push(t);
            } else if r.kind_of(c, "UMLPackage") {
                let mut sp = path.clone();
                sp.push(r.name(c));
                p.subs.push(pkg(r, c, sp, rels));
            }
        }
        p
    }
    pkg(r, base, vec![], &rels)
}

fn ty_of(r: &Repo, c: Eid, rels: &[Eid]) -> Ty {
    let kind = classifier_kind(r.ty(c)).unwrap_or_default();
    let mut t = Ty::new(kind, &r.name(c));
    t.vis = r.str(c, "visibility");
    t.is_abstract = r.bool(c, "isAbstract");
    t.doc = r.str(c, "documentation");
    t.stereotype = views::type_name(r, c, "stereotype");
    t.generics = r.list(c, "templateParameters").into_iter().map(|p| r.name(p)).collect();
    for &rel in rels {
        let src = r.refd(rel, "source");
        let tgt = r.refd(rel, "target");
        match r.ty(rel) {
            "UMLGeneralization" if src == Some(c) => t.supers.extend(tgt.map(|x| r.name(x))),
            "UMLInterfaceRealization" if src == Some(c) => t.reals.extend(tgt.map(|x| r.name(x))),
            "UMLAssociation" => {
                let (Some(e1), Some(e2)) = (r.child(rel, "end1"), r.child(rel, "end2")) else { continue };
                let nav = |e: Eid| r.str(e, "navigable");
                for (mine, other) in [(e1, e2), (e2, e1)] {
                    if r.refd(mine, "reference") != Some(c) {
                        continue;
                    }
                    // the other end becomes a field when it is navigable, or both are unspecified
                    let both = nav(mine) != "navigable" && nav(mine) != "notNavigable" && nav(other) != "navigable" && nav(other) != "notNavigable";
                    if nav(other) != "navigable" && !both {
                        continue;
                    }
                    let Some(oc) = r.refd(other, "reference") else { continue };
                    let name = if r.name(other).is_empty() { lc_first(&r.name(oc)) } else { r.name(other) };
                    t.fields.push(Field { name, ty: r.name(oc), vis: r.str(other, "visibility"), mult: r.str(other, "multiplicity"), doc: r.str(other, "documentation"), ..Default::default() });
                    if e1 == e2 {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    let assoc_fields = std::mem::take(&mut t.fields);
    for a in r.list(c, "attributes") {
        t.fields.push(Field {
            name: r.name(a),
            ty: views::type_name(r, a, "type"),
            vis: r.str(a, "visibility"),
            is_static: r.bool(a, "isStatic"),
            read_only: r.bool(a, "isReadOnly"),
            default: r.str(a, "defaultValue"),
            mult: r.str(a, "multiplicity"),
            doc: r.str(a, "documentation"),
        });
    }
    t.fields.extend(assoc_fields);
    for o in r.list(c, "operations") {
        let mut m = Method {
            name: r.name(o),
            vis: r.str(o, "visibility"),
            is_static: r.bool(o, "isStatic"),
            is_abstract: r.bool(o, "isAbstract"),
            doc: r.str(o, "documentation"),
            ..Default::default()
        };
        for p in r.list(o, "parameters") {
            let ty = views::type_name(r, p, "type");
            let ty = if is_many(&r.str(p, "multiplicity")) { format!("{ty}[]") } else { ty };
            if r.str(p, "direction") == "return" {
                m.ret = ty;
            } else {
                m.params.push(Param { name: r.name(p), ty, default: r.str(p, "defaultValue") });
            }
        }
        t.methods.push(m);
    }
    t.literals = r.list(c, "literals").into_iter().map(|l| r.name(l)).collect();
    for n in r.list(c, "ownedElements") {
        if classifier_kind(r.ty(n)).is_some() {
            t.nested.push(ty_of(r, n, rels));
        }
    }
    t
}

/// One generated file: its path under the output folder and its text.
pub type OutFile = (PathBuf, String);

/// The files of a language for a base package (DDL: an ER data model).
pub fn generate_files(r: &Repo, base: Eid, l: Lang, o: &GenOpts) -> Vec<OutFile> {
    if l == Lang::Ddl {
        return ddl::generate(r, base, o);
    }
    let tree = gen_tree(r, base);
    match l {
        Lang::Java => java::generate(&tree, o),
        Lang::Cpp => cpp::generate(&tree, o),
        Lang::CSharp => csharp::generate(&tree, o),
        Lang::C => c::generate(&tree, o),
        Lang::Python => python::generate(&tree, o),
        Lang::Php => php::generate(&tree, o),
        Lang::TypeScript => typescript::generate(&tree, o),
        Lang::JavaScript => javascript::generate(&tree, o),
        Lang::Rust => rust::generate(&tree, o),
        Lang::Go => go::generate(&tree, o),
        Lang::Ruby => ruby::generate(&tree, o),
        Lang::Kotlin => kotlin::generate(&tree, o),
        Lang::Dart => dart::generate(&tree, o),
        Lang::Swift => swift::generate(&tree, o),
        Lang::Ddl => unreachable!(),
    }
}

/// A file for each top-level type of every package, in the package's folder.
pub fn per_type(tree: &Pkg, file: impl Fn(&Ty) -> String, mut text: impl FnMut(&Pkg, &Ty) -> String) -> Vec<OutFile> {
    let mut out = vec![];
    for p in tree.all() {
        for t in &p.types {
            let mut path: PathBuf = p.path.iter().collect();
            path.push(file(t));
            out.push((path, text(p, t)));
        }
    }
    out
}

/// Reverse one file's text (`rel`: its folder path from the source folder, for the languages whose
/// namespaces are folders: C, C++ without namespaces, Python, JavaScript, TypeScript, Rust, Ruby,
/// Dart and Swift).
pub fn parse_source(l: Lang, src: &str, rel: &[String]) -> Parsed {
    match l {
        Lang::Java => java::parse(src),
        Lang::Cpp => cpp::parse(src, rel),
        Lang::CSharp => csharp::parse(src),
        Lang::C => c::parse(src, rel),
        Lang::Python => python::parse(src, rel),
        Lang::Php => php::parse(src),
        Lang::TypeScript => typescript::parse(src, rel, true),
        Lang::JavaScript => typescript::parse(src, rel, false),
        Lang::Rust => rust::parse(src, rel),
        Lang::Go => go::parse(src),
        Lang::Ruby => ruby::parse(src, rel),
        Lang::Kotlin => kotlin::parse(src),
        Lang::Dart => dart::parse(src, rel),
        Lang::Swift => swift::parse(src, rel),
        Lang::Ddl => Parsed { units: vec![], errors: false },
    }
}

/// The source files of a folder, recursively, skipping hidden folders, `node_modules` and
/// `__pycache__` (and Go's `_test.go` files), sorted by path.
pub fn source_files(dir: &Path, l: Lang) -> Vec<PathBuf> {
    let mut out = vec![];
    fn walk(d: &Path, l: Lang, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        let mut items: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        items.sort();
        for p in items {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if p.is_dir() {
                if !name.starts_with('.') && name != "node_modules" && name != "__pycache__" {
                    walk(&p, l, out);
                }
            } else {
                let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                if l.extensions().contains(&ext.as_str()) && !(l == Lang::Go && name.ends_with("_test.go")) {
                    out.push(p);
                }
            }
        }
    }
    walk(dir, l, &mut out);
    out
}

// ------------------------------------------------------------------ neutral form → model

/// The element type, its multiplicity, for a collection or optional type: `List<Book>` → (Book, *),
/// `Book[]`, `[]Book`, `[Book]`, `Vec<Book>`, `list[Book]` → (Book, *); `Option<Book>`, `Book?`,
/// `Optional[Book]` → (Book, 0..1).
pub fn element_type(t: &str) -> (String, String) {
    let t = t.trim();
    if let Some(e) = t.strip_suffix("[]") {
        return (e.trim().to_string(), "*".into());
    }
    if let Some(e) = t.strip_prefix("[]") {
        return (e.trim().to_string(), "*".into());
    }
    if t.starts_with('[') && t.ends_with(']') && !t.contains(':') {
        return (t[1..t.len() - 1].trim().to_string(), "*".into());
    }
    if let Some(e) = t.strip_suffix('?') {
        return (e.trim().to_string(), "0..1".into());
    }
    const MANY: [&str; 22] = [
        "List", "ArrayList", "LinkedList", "Set", "HashSet", "TreeSet", "Collection", "Iterable", "IList", "ICollection", "IEnumerable", "Array", "MutableList", "MutableSet", "Vec", "vector", "std::vector", "std::list", "list", "set", "Sequence", "Seq",
    ];
    const ONE: [&str; 4] = ["Option", "Optional", "std::optional", "Nullable"];
    for (open, shut) in [('<', '>'), ('[', ']')] {
        if let Some(i) = t.find(open) {
            if t.ends_with(shut) {
                let head = t[..i].trim();
                let inner = t[i + 1..t.len() - 1].trim();
                if inner.contains(',') {
                    continue;
                }
                if MANY.contains(&head) {
                    return (inner.to_string(), "*".into());
                }
                if ONE.contains(&head) {
                    return (inner.to_string(), "0..1".into());
                }
            }
        }
    }
    (t.to_string(), String::new())
}

/// The name a type refers to: `std::shared_ptr<a::Book>` → `Book`, `const Book*` → `Book`.
pub fn base_name(t: &str) -> String {
    let mut t = t.trim();
    for p in ["const ", "struct ", "enum ", "unsigned ", "&mut ", "&", "*", "?", "mut "] {
        t = t.trim_start_matches(p).trim();
    }
    let t = t.split('<').next().unwrap_or(t).split('[').next().unwrap_or(t);
    let t = t.trim_end_matches(['*', '&', '?', ' ']);
    t.rsplit(['.', ':', '\\']).next().unwrap_or(t).trim().to_string()
}

struct Pending {
    eid: Eid,
    ty: Ty,
}

/// Import reversed units under `target`: packages merged by name, types merged by name in their
/// package, members deduplicated, then super types, typed fields (as associations when asked),
/// operations and parameters. Returns the number of types.
pub fn import_units(r: &mut Repo, target: Eid, units: Vec<Unit>, o: &RevOpts) -> usize {
    let mut pend: Vec<Pending> = vec![];
    fn child_pkg(r: &mut Repo, parent: Eid, name: &str) -> Eid {
        if let Some(p) = r.list(parent, "ownedElements").into_iter().find(|c| r.kind_of(*c, "UMLPackage") && r.name(*c) == name) {
            return p;
        }
        r.create("UMLPackage", &[("name", json!(name))], Some((parent, "ownedElements")))
    }
    fn add_type(r: &mut Repo, owner: Eid, t: Ty, pend: &mut Vec<Pending>) {
        let ty = match t.kind {
            Kind::Class => "UMLClass",
            Kind::Interface => "UMLInterface",
            Kind::Enum => "UMLEnumeration",
        };
        let existing = r.list(owner, "ownedElements").into_iter().find(|c| r.name(*c) == t.name && classifier_kind(r.ty(*c)).is_some());
        let eid = match existing {
            Some(e) => e,
            None => {
                let e = r.create(ty, &[("name", json!(t.name))], Some((owner, "ownedElements")));
                if !t.doc.is_empty() {
                    r.set(e, "documentation", json!(t.doc));
                }
                if !t.vis.is_empty() && t.vis != "public" {
                    r.set(e, "visibility", json!(t.vis));
                }
                if !t.stereotype.is_empty() {
                    r.set(e, "stereotype", json!(t.stereotype));
                }
                if t.is_abstract && t.kind == Kind::Class {
                    r.set(e, "isAbstract", json!(true));
                }
                for g in &t.generics {
                    r.create("UMLTemplateParameter", &[("name", json!(g))], Some((e, "templateParameters")));
                }
                e
            }
        };
        let mut t = t;
        for n in std::mem::take(&mut t.nested) {
            add_type(r, eid, n, pend);
        }
        pend.push(Pending { eid, ty: t });
    }
    let mut count = 0;
    for u in units {
        let mut owner = target;
        for n in &u.ns {
            if !n.is_empty() {
                owner = child_pkg(r, owner, n);
            }
        }
        for t in u.types {
            add_type(r, owner, t, &mut pend);
        }
    }
    // the types by simple name: the reversed ones first, then the model's others
    let made: Vec<(String, Eid)> = pend.iter().map(|p| (r.name(p.eid), p.eid)).collect();
    let others: Vec<(String, Eid)> = r.all_of("UMLClassifier").into_iter().map(|c| (r.name(c), c)).collect();
    let find = |n: &str| made.iter().chain(others.iter()).find(|m| m.0 == n && !n.is_empty()).map(|m| m.1);
    let set_type = |r: &mut Repo, e: Eid, ty: &str| {
        if ty.is_empty() {
            return;
        }
        match find(&base_name(ty)).filter(|_| base_name(ty) == ty.trim()) {
            Some(c) => r.set_ref(e, "type", Some(c)),
            None => r.set(e, "type", json!(ty)),
        }
    };
    for p in &pend {
        count += 1;
        let e = p.eid;
        let t = &p.ty;
        let public = |v: &str| !o.public_only || v == "public" || v.is_empty();
        for l in &t.literals {
            if !r.list(e, "literals").into_iter().any(|x| r.name(x) == *l) {
                r.create("UMLEnumerationLiteral", &[("name", json!(l))], Some((e, "literals")));
            }
        }
        for f in &t.fields {
            if !public(&f.vis) || r.list(e, "attributes").into_iter().any(|x| r.name(x) == f.name) {
                continue;
            }
            let (elem, mut mult) = element_type(&f.ty);
            // a collection type's multiplicity wins over an optional mark (`tags?: string[]`)
            if !f.mult.is_empty() && mult.is_empty() {
                mult = f.mult.clone();
            }
            let target_cls = find(&base_name(&elem)).filter(|_| base_name(&elem) == elem.trim()).filter(|c| r.kind_of(*c, "UMLClassifier"));
            if o.association && !f.is_static {
                if let Some(tc) = target_cls {
                    let a = r.create("UMLAssociation", &[], Some((e, "ownedElements")));
                    let rs = r.ref_to(e);
                    r.create("UMLAssociationEnd", &[("reference", rs)], Some((a, "end1")));
                    let rt = r.ref_to(tc);
                    let end = r.create("UMLAssociationEnd", &[("name", json!(f.name)), ("reference", rt), ("navigable", json!("navigable"))], Some((a, "end2")));
                    if !f.vis.is_empty() && f.vis != "public" {
                        r.set(end, "visibility", json!(f.vis));
                    }
                    if !mult.is_empty() {
                        r.set(end, "multiplicity", json!(mult));
                    }
                    continue;
                }
            }
            let a = r.create("UMLAttribute", &[("name", json!(f.name))], Some((e, "attributes")));
            if !f.vis.is_empty() && f.vis != "public" {
                r.set(a, "visibility", json!(f.vis));
            }
            if f.is_static {
                r.set(a, "isStatic", json!(true));
            }
            set_type(r, a, &elem);
            if !mult.is_empty() {
                r.set(a, "multiplicity", json!(mult));
            }
            if f.read_only {
                r.set(a, "isReadOnly", json!(true));
            }
            if !f.default.is_empty() {
                r.set(a, "defaultValue", json!(f.default));
            }
            if !f.doc.is_empty() {
                r.set(a, "documentation", json!(f.doc));
            }
        }
        for m in &t.methods {
            if !public(&m.vis) {
                continue;
            }
            // the same operation from a header and its source file is kept once
            if r.list(e, "operations").into_iter().any(|x| r.name(x) == m.name && r.list(x, "parameters").into_iter().filter(|p| r.str(*p, "direction") != "return").count() == m.params.len()) {
                continue;
            }
            let op = r.create("UMLOperation", &[("name", json!(m.name))], Some((e, "operations")));
            if !m.vis.is_empty() && m.vis != "public" {
                r.set(op, "visibility", json!(m.vis));
            }
            if m.is_static {
                r.set(op, "isStatic", json!(true));
            }
            if m.is_abstract {
                r.set(op, "isAbstract", json!(true));
            }
            if !m.doc.is_empty() {
                r.set(op, "documentation", json!(m.doc));
            }
            for p in &m.params {
                let pe = r.create("UMLParameter", &[("name", json!(p.name))], Some((op, "parameters")));
                let (elem, mult) = element_type(&p.ty);
                set_type(r, pe, &elem);
                if !mult.is_empty() {
                    r.set(pe, "multiplicity", json!(mult));
                }
                if !p.default.is_empty() {
                    r.set(pe, "defaultValue", json!(p.default));
                }
            }
            let ret = m.ret.trim();
            if !ret.is_empty() && !["void", "()", "Unit", "None", "Void"].contains(&ret) {
                let pe = r.create("UMLParameter", &[("direction", json!("return"))], Some((op, "parameters")));
                let (elem, mult) = element_type(ret);
                set_type(r, pe, &elem);
                if !mult.is_empty() {
                    r.set(pe, "multiplicity", json!(mult));
                }
            }
        }
        // super types: a realization when an interface is reached from a class, else a generalization
        for (n, explicit_real) in t.supers.iter().map(|s| (s, false)).chain(t.reals.iter().map(|s| (s, true))) {
            let Some(target) = find(&base_name(n)) else { continue };
            if target == e {
                continue;
            }
            let real = r.ty(target) == "UMLInterface" && r.ty(e) != "UMLInterface";
            let kind = if real || (explicit_real && r.ty(e) != "UMLInterface") { "UMLInterfaceRealization" } else { "UMLGeneralization" };
            let exists = r.list(e, "ownedElements").into_iter().any(|x| r.ty(x) == kind && r.refd(x, "target") == Some(target));
            if !exists {
                let (rs, rt) = (r.ref_to(e), r.ref_to(target));
                r.create(kind, &[("source", rs), ("target", rt)], Some((e, "ownedElements")));
            }
        }
    }
    count
}

/// A visibility word in UML terms ("package" for Java's default, C#'s internal…).
pub fn vis_of(mods: &[String], default: &str) -> String {
    for m in mods {
        match m.as_str() {
            "public" | "open" => return "public".into(),
            "protected" => return "protected".into(),
            "private" | "fileprivate" => return "private".into(),
            "internal" => return "package".into(),
            _ => {}
        }
    }
    default.into()
}

// ------------------------------------------------------------------ running

impl App {
    /// The base packages a Generate offers (ER data models for DDL), in model order.
    pub fn codegen_bases(&self, l: Lang) -> Vec<Eid> {
        let mut out = vec![];
        fn walk(r: &Repo, e: Eid, l: Lang, out: &mut Vec<Eid>) {
            for c in r.list(e, "ownedElements") {
                let ok = if l == Lang::Ddl { r.ty(c) == "ERDDataModel" } else { r.kind_of(c, "UMLPackage") && r.ty(c) != "UMLProfile" };
                if ok {
                    out.push(c);
                }
                walk(r, c, l, out);
            }
        }
        walk(&self.repo, self.repo.root, l, &mut out);
        out
    }

    /// Write the base package's code into `out`: existing files are overwritten. Returns the
    /// files written, relative to `out`.
    pub fn generate_code(&mut self, l: Lang, base: Eid, out: &Path) -> Result<Vec<PathBuf>, String> {
        let o = gen_opts(&self.prefs, l);
        let files = generate_files(&self.repo, base, l, &o);
        let mut done = vec![];
        for (rel, text) in files {
            let p = out.join(&rel);
            if let Some(d) = p.parent() {
                std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
            }
            std::fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))?;
            done.push(rel);
        }
        Ok(done)
    }

    /// Reverse a source folder into `target` (or a new model `<Lang>Reverse` in the project), as
    /// one undo step, with the diagrams the options ask for; the imported package is selected.
    /// Returns the log lines.
    pub fn reverse_code(&mut self, l: Lang, target: Option<Eid>, dir: &Path, ms: &dyn Measure) -> Result<Vec<String>, String> {
        let o = rev_opts(&self.prefs, l);
        let files = source_files(dir, l);
        if files.is_empty() {
            return Err(format!("No {} source files found in {}.", l.label(), dir.display()));
        }
        let mut log = vec![];
        let mut errors = 0;
        let mut units = vec![];
        let mut tables = vec![];
        for f in &files {
            let Ok(bytes) = std::fs::read(f) else {
                errors += 1;
                continue;
            };
            let text = String::from_utf8_lossy(&bytes);
            let rel: Vec<String> = f.parent().and_then(|p| p.strip_prefix(dir).ok()).map(|p| p.iter().map(|s| s.to_string_lossy().to_string()).collect()).unwrap_or_default();
            log.push(format!("Reading {}", f.strip_prefix(dir).unwrap_or(f).display()));
            if l == Lang::Ddl {
                let (t, err) = ddl::parse(&text);
                errors += err as usize;
                tables.extend(t);
            } else {
                let p = parse_source(l, &text, &rel);
                errors += p.errors as usize;
                units.extend(p.units);
            }
        }
        let undo = self.undo.len();
        let root = self.repo.root;
        let made_new = target.is_none();
        let (pkg, count) = self.change(|a| {
            let pkg = match target {
                Some(t) => t,
                None if l == Lang::Ddl => root,
                None => a.repo.create("UMLModel", &[("name", json!(l.reverse_model()))], Some((root, "ownedElements"))),
            };
            if l == Lang::Ddl {
                ddl::import(a, pkg, &tables, made_new, &o)
            } else {
                let n = import_units(&mut a.repo, pkg, units, &o);
                (pkg, n)
            }
        });
        log.push(format!("{} files analyzed, {count} {} imported.", files.len(), if l == Lang::Ddl { "tables" } else { "types" }));
        if errors > 0 {
            log.push(format!("{errors} files had syntax errors; the rest of them was analyzed."));
        }
        // the diagrams the options ask for
        if l == Lang::Ddl {
            if o.type_hierarchy {
                ddl::er_diagram(self, pkg, ms);
            }
        } else {
            use crate::generate::Generator;
            if o.type_hierarchy {
                self.generate_diagram_in(pkg, Generator::TypeHierarchy, ms);
            }
            if o.package_structure && self.repo.list(pkg, "ownedElements").iter().any(|c| self.repo.kind_of(*c, "UMLPackage")) {
                self.generate_diagram_in(pkg, Generator::PackageStructure, ms);
            }
            if o.package_overview {
                let mut pkgs = vec![pkg];
                let mut i = 0;
                while i < pkgs.len() {
                    let kids: Vec<Eid> = self.repo.list(pkgs[i], "ownedElements").into_iter().filter(|c| self.repo.kind_of(*c, "UMLPackage")).collect();
                    pkgs.extend(kids);
                    i += 1;
                }
                for p in pkgs {
                    if self.repo.list(p, "ownedElements").iter().any(|c| classifier_kind(self.repo.ty(*c)).is_some()) {
                        self.generate_diagram_in(p, Generator::Overview, ms);
                    }
                }
            }
        }
        self.undo.truncate(undo + 1);
        self.focus = Some(pkg);
        self.reveal(pkg);
        Ok(log)
    }
}

// ------------------------------------------------------------------ the dialog

#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Generate,
    Reverse,
}

pub struct Dialog {
    pub lang: Lang,
    pub tab: Tab,
    pub base: Option<Eid>,
    /// None: "(New model)"
    pub target: Option<Eid>,
    pub out_dir: String,
    pub src_dir: String,
    pub log: Vec<String>,
    pub last_out: Option<PathBuf>,
}

impl App {
    /// Tools › <Language>: the dialog, on the Generate tab, with the last used folders.
    pub fn open_codegen(&mut self, l: Lang) {
        let k = l.key();
        let folder = |a: &App, n: &str| a.prefs.get(&format!("{k}.{n}")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let bases = self.codegen_bases(l);
        // the selected package when there is one, else the first
        let base = self.focus.filter(|f| bases.contains(f)).or_else(|| bases.first().copied());
        let target = self.focus.filter(|f| self.repo.alive(*f) && self.repo.kind_of(*f, if l == Lang::Ddl { "ERDDataModel" } else { "UMLPackage" }));
        self.codegen = Some(Dialog { lang: l, tab: Tab::Generate, base, target, out_dir: folder(self, "lastOutputFolder"), src_dir: folder(self, "lastSourceFolder"), log: vec![], last_out: None });
    }

    pub fn codegen_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.codegen.take() else { return };
        let l = d.lang;
        let mut g = gen_opts(&self.prefs, l);
        let mut ro = rev_opts(&self.prefs, l);
        let (g0, r0) = (format!("{g:?}"), format!("{ro:?}"));
        let mut open = true;
        let (mut run, mut close) = (false, false);
        let bases = self.codegen_bases(l);
        let targets: Vec<Eid> = if l == Lang::Ddl { bases.clone() } else { bases.iter().copied().collect() };
        let label = |a: &App, e: Eid| views::display_name(&a.repo, e);
        egui::Window::new(l.label()).id(egui::Id::new("codegen-dialog")).open(&mut open).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut d.tab, Tab::Generate, "Generate");
                ui.selectable_value(&mut d.tab, Tab::Reverse, "Reverse");
            });
            ui.separator();
            egui::Grid::new("codegen-grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                match d.tab {
                    Tab::Generate => {
                        ui.label(if l == Lang::Ddl { "Data Model" } else { "Base Package" });
                        let text = d.base.map(|b| label(self, b)).unwrap_or_else(|| "(none)".into());
                        egui::ComboBox::from_id_salt("codegen-base").width(360.0).selected_text(text).show_ui(ui, |ui| {
                            for &b in &bases {
                                ui.selectable_value(&mut d.base, Some(b), label(self, b));
                            }
                        });
                        ui.end_row();
                        ui.label("Output Folder");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut d.out_dir).desired_width(280.0));
                            if ui.button("Browse...").clicked() {
                                if let Some(p) = rfd::FileDialog::new().set_title("Select a folder where generated codes to be located").pick_folder() {
                                    d.out_dir = p.display().to_string();
                                }
                            }
                        });
                        ui.end_row();
                    }
                    Tab::Reverse => {
                        ui.label(if l == Lang::Ddl { "Target Data Model" } else { "Target Package" });
                        let text = d.target.map(|b| label(self, b)).unwrap_or_else(|| "(New model)".into());
                        egui::ComboBox::from_id_salt("codegen-target").width(360.0).selected_text(text).show_ui(ui, |ui| {
                            ui.selectable_value(&mut d.target, None, "(New model)");
                            for &b in &targets {
                                ui.selectable_value(&mut d.target, Some(b), label(self, b));
                            }
                        });
                        ui.end_row();
                        ui.label("Source Folder");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut d.src_dir).desired_width(280.0));
                            if ui.button("Browse...").clicked() {
                                if let Some(p) = rfd::FileDialog::new().set_title("Select a folder containing source files").pick_folder() {
                                    d.src_dir = p.display().to_string();
                                }
                            }
                        });
                        ui.end_row();
                    }
                }
            });
            ui.add_space(4.0);
            egui::CollapsingHeader::new("Options").default_open(true).show(ui, |ui| match d.tab {
                Tab::Generate => {
                    if l == Lang::Ddl {
                        ui.horizontal(|ui| {
                            ui.label("DBMS");
                            for (v, t) in [("mysql", "MySQL"), ("sqlite", "SQLite"), ("postgresql", "PostgreSQL")] {
                                ui.radio_value(&mut g.dbms, v.to_string(), t);
                            }
                        });
                        ui.checkbox(&mut g.quote, "Quote identifiers");
                        ui.checkbox(&mut g.drop_table, "Drop tables first (DROP TABLE IF EXISTS)");
                        ui.checkbox(&mut g.doc, "Comments from documentation");
                    } else {
                        ui.checkbox(&mut g.doc, "Documentation comments");
                        ui.checkbox(&mut g.use_tab, "Use tab for indentation");
                        ui.add_enabled_ui(!g.use_tab, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Indent spaces");
                                ui.add(egui::DragValue::new(&mut g.indent_spaces).range(0..=16));
                            });
                        });
                    }
                }
                Tab::Reverse => {
                    if l == Lang::Ddl {
                        ui.checkbox(&mut ro.type_hierarchy, "Create an ER diagram");
                    } else {
                        ui.checkbox(&mut ro.association, "Fields typed by a class as associations");
                        ui.checkbox(&mut ro.public_only, "Public members only");
                        ui.checkbox(&mut ro.type_hierarchy, "Create a type hierarchy diagram");
                        ui.checkbox(&mut ro.package_structure, "Create a package structure diagram");
                        ui.checkbox(&mut ro.package_overview, "Create package overview diagrams");
                    }
                }
            });
            ui.add_space(4.0);
            egui::ScrollArea::vertical().max_height(160.0).auto_shrink([false, true]).show(ui, |ui| {
                ui.set_min_height(80.0);
                for line in &d.log {
                    ui.monospace(line);
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Copy Log").clicked() {
                    ui.ctx().copy_text(d.log.join("\n"));
                }
                if ui.button("Clear Log").clicked() {
                    d.log.clear();
                }
                if d.tab == Tab::Generate {
                    let can_open = d.last_out.is_some();
                    if ui.add_enabled(can_open, egui::Button::new("Open Folder")).clicked() {
                        if let Some(p) = &d.last_out {
                            ui.ctx().open_url(egui::OpenUrl::new_tab(format!("file://{}", p.display())));
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = ui.button("Close").clicked();
                    let ok = match d.tab {
                        Tab::Generate => d.base.is_some() && !d.out_dir.trim().is_empty(),
                        Tab::Reverse => !d.src_dir.trim().is_empty(),
                    };
                    run = ui.add_enabled(ok, egui::Button::new(if d.tab == Tab::Generate { "Generate" } else { "Reverse" })).clicked();
                });
            });
            close |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if format!("{g:?}") != g0 || format!("{ro:?}") != r0 {
            store_opts(&mut self.prefs, l, &g, &ro);
        }
        if run {
            let k = l.key();
            match d.tab {
                Tab::Generate => {
                    let out = PathBuf::from(d.out_dir.trim());
                    self.prefs.set(&format!("{k}.lastOutputFolder"), json!(d.out_dir.trim()));
                    self.prefs.flush();
                    match self.generate_code(l, d.base.unwrap(), &out) {
                        Ok(files) => {
                            for f in &files {
                                d.log.push(format!("Generated {}", f.display()));
                            }
                            d.log.push(format!("{} files generated in {}.", files.len(), out.display()));
                            d.last_out = Some(out);
                        }
                        Err(e) => d.log.push(format!("Error: {e}")),
                    }
                }
                Tab::Reverse => {
                    self.prefs.set(&format!("{k}.lastSourceFolder"), json!(d.src_dir.trim()));
                    self.prefs.flush();
                    let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
                    match self.reverse_code(l, d.target, Path::new(d.src_dir.trim()), &ms) {
                        Ok(lines) => d.log.extend(lines),
                        Err(e) => d.log.push(format!("Error: {e}")),
                    }
                }
            }
        }
        if open && !close {
            self.codegen = Some(d);
        }
    }
}
