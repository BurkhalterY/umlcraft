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
#[cfg(test)]
mod tests_rev;

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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

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
    /// reverse only: `final` (isLeaf), and tags such as Java's `volatile` and `transient` (a
    /// boolean tag when the value is empty) or `collection` = `List`
    pub is_leaf: bool,
    pub tags: Vec<(String, String)>,
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
    /// reverse only: the operation's own type parameters (template parameters of the operation)
    pub generics: Vec<String>,
    /// reverse only: "constructor" for a constructor (Java, C#, C++, as the 7.x reverses do)
    pub stereotype: String,
    /// reverse only: the exceptions it declares (`throws`), as raised exceptions
    pub raises: Vec<String>,
    /// reverse only: `final` (isLeaf), `synchronized` (concurrency "concurrent") and boolean tags
    /// such as `native`
    pub is_leaf: bool,
    pub concurrent: bool,
    pub tags: Vec<(String, String)>,
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
    /// reverse only: each generic's bound and default (`T extends Item = Book`), for the template
    /// parameter's parameterType and defaultValue
    pub gen_bounds: Vec<(String, String)>,
    pub fields: Vec<Field>,
    pub methods: Vec<Method>,
    pub literals: Vec<String>,
    /// reverse only: each literal's documentation comment
    pub literal_docs: Vec<String>,
    pub nested: Vec<Ty>,
    /// reverse only: a `final`/`sealed` class (isFinalSpecialization and isLeaf)
    pub is_leaf: bool,
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
    /// the file's imports, which resolve the type names its types use
    pub imports: Vec<Import>,
}

/// An import of a reversed file: `import a.b.Book;` (path a, b, Book; known as Book), `import a.b.*;`
/// (wildcard: the names of package a.b), `using N = a.b.Book;` (alias N), `from .book import Book`
/// (module: the path is a module's, not a package's, so the name is looked for anywhere).
#[derive(Clone, Debug, Default)]
pub struct Import {
    pub path: Vec<String>,
    pub alias: String,
    pub wildcard: bool,
    pub module: bool,
}

impl Import {
    /// `a.b.Book` → the import of Book; `a.b.*` → the wildcard import of a.b; `seps` also split
    /// the path (`::`, `\\`, `/`).
    pub fn of(text: &str, seps: &[&str]) -> Import {
        let mut s = text.trim().to_string();
        for sep in seps {
            s = s.replace(sep, ".");
        }
        let mut path: Vec<String> = s.split('.').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect();
        let wildcard = path.last().map(|x| x == "*").unwrap_or(false);
        if wildcard {
            path.pop();
        }
        let alias = if wildcard { String::new() } else { path.last().cloned().unwrap_or_default() };
        Import { path, alias, wildcard, module: false }
    }
}

/// A reversed file: its units and whether it had syntax errors (the rest is still read).
pub struct Parsed {
    pub units: Vec<Unit>,
    pub errors: bool,
}

// ------------------------------------------------------------------ options

#[derive(Clone, Debug, Default)]
pub struct GenOpts {
    pub use_tab: bool,
    pub indent_spaces: i64,
    pub doc: bool,
    /// DDL: "mysql", "sqlite" or "postgresql"
    pub dbms: String,
    pub quote: bool,
    pub drop_table: bool,
    /// C++: the project's header comment at the top of each file (cpp.gen.includeHeader), a
    /// `std::vector<T>` rather than `T*` for many (cpp.gen.useVector), and a `.cpp` per class
    /// (cpp.gen.genCpp)
    pub include_header: bool,
    pub use_vector: bool,
    pub gen_cpp: bool,
    /// C#: auto-properties instead of fields (csharp.gen.propertiesByDefault)
    pub properties: bool,
    /// Python: snake_case module names (python.gen.lowerCase) and a first line such as
    /// `#!/usr/bin/python` (python.gen.installPath)
    pub lower_case: bool,
    pub install_path: String,
    /// the project's header comment text (name, author, company, version, copyright)
    pub header: String,
}

impl GenOpts {
    pub fn indent(&self) -> String {
        if self.use_tab { "\t".into() } else { " ".repeat(self.indent_spaces.clamp(0, 16) as usize) }
    }
}

#[derive(Clone, Debug)]
pub struct RevOpts {
    /// Python: the `self`/`cls` parameter and the magic methods are left out (python.rev.*)
    pub skip_self: bool,
    pub skip_magic: bool,
    pub association: bool,
    pub public_only: bool,
    pub type_hierarchy: bool,
    pub package_overview: bool,
    pub package_structure: bool,
}

/// How an option is edited.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OptKind {
    Check,
    Number,
    Text,
    /// DDL's DBMS: MySQL, SQLite or PostgreSQL
    Dbms,
}

/// One option of a language: its preference key, label, description (the 7.x extensions' texts
/// where they have one), kind and default; `rev` for the Reverse tab.
#[derive(Clone, Debug)]
pub struct Opt {
    pub key: String,
    pub label: &'static str,
    pub desc: &'static str,
    pub kind: OptKind,
    pub default: serde_json::Value,
    pub rev: bool,
}

/// The documentation comment option's key and label: `java.gen.javaDoc` "JavaDoc",
/// `csharp.gen.csharpDoc` "C# Doc", `python.gen.docString` "Docstring" (the 7.x keys), else
/// `<lang>.gen.doc`.
fn doc_opt(l: Lang) -> (&'static str, &'static str) {
    match l {
        Lang::Java => ("javaDoc", "JavaDoc"),
        Lang::CSharp => ("csharpDoc", "C# Doc"),
        Lang::Python => ("docString", "Docstring"),
        Lang::Ddl => ("doc", "Comments"),
        _ => ("doc", "Documentation Comments"),
    }
}

/// Every option of a language, in the dialog's and the Preferences dialog's order: the 7.x
/// extensions' (staruml-java, staruml-cpp and staruml-csharp `preferences/preference.json`,
/// staruml-python's), and the same set for the languages that had no 7.x extension.
pub fn options(l: Lang) -> Vec<Opt> {
    let k = l.key();
    let o = |n: &str, label: &'static str, desc: &'static str, kind: OptKind, default: serde_json::Value, rev: bool| Opt { key: format!("{k}.{}.{n}", if rev { "rev" } else { "gen" }), label, desc, kind, default, rev };
    let mut v = vec![];
    let (dk, dl) = doc_opt(l);
    if l == Lang::Ddl {
        v.push(o("dbms", "DBMS", "Database management system the DDL is written for.", OptKind::Dbms, json!("mysql"), false));
        v.push(o("quoteIdentifiers", "Quote Identifiers", "Quote table and column names.", OptKind::Check, json!(true), false));
        v.push(o("dropTable", "Drop Table", "Write DROP TABLE IF EXISTS before each table.", OptKind::Check, json!(false), false));
        v.push(o(dk, dl, "Write documentation as comments.", OptKind::Check, json!(true), false));
        v.push(o("erDiagram", "ER Diagram", "Create an ER diagram for the reversed tables.", OptKind::Check, json!(true), true));
        return v;
    }
    let doc_desc = match l {
        Lang::Java => "Generate JavaDoc comments.",
        Lang::CSharp => "Generate C# Doc comments.",
        Lang::Python => "Generate docstrings.",
        _ => "Generate documentation comments.",
    };
    if l == Lang::Python {
        v.push(o("installPath", "Installation Path", "Installation path of python (the first line of each file; empty for none).", OptKind::Text, json!(""), false));
    }
    if l != Lang::Cpp {
        v.push(o(dk, dl, doc_desc, OptKind::Check, json!(true), false));
    }
    // Go is indented with tabs (gofmt)
    v.push(o("useTab", "Use Tab", "Use Tab for indentation instead of spaces.", OptKind::Check, json!(l == Lang::Go), false));
    v.push(o("indentSpaces", "Indent Spaces", "Number of spaces for indentation.", OptKind::Number, json!(l.default_indent()), false));
    match l {
        Lang::Cpp => {
            v.push(o("includeHeader", "Include default header", "Include default header.", OptKind::Check, json!(true), false));
            v.push(o("useVector", "Use vector instead of *", "Use vector<> instead of pointer.", OptKind::Check, json!(true), false));
            v.push(o("genCpp", "Generate *.cpp file", "Generate cpp file", OptKind::Check, json!(true), false));
            v.push(o(dk, dl, doc_desc, OptKind::Check, json!(true), false));
        }
        Lang::CSharp => v.push(o("propertiesByDefault", "Properties instead of fields", "Generate properties instead of fields if not explicitly specified.", OptKind::Check, json!(false), false)),
        Lang::Python => v.push(o("lowerCase", "Lower Case Filenames", "Make filenames lower case, according to PEP8 recommendation.", OptKind::Check, json!(true), false)),
        _ => {}
    }
    if l == Lang::Python {
        v.push(o("skipSelfParam", "Skip self parameter", "Don't show self parameter in methods.", OptKind::Check, json!(true), true));
        v.push(o("skipMagicMethods", "Skip Magic Methods", "Don't show Python's magic methods. Like __repr__, __hash__, etc.", OptKind::Check, json!(true), true));
    }
    v.push(o("association", "Use Association", "Reverse fields as UML Associations.", OptKind::Check, json!(true), true));
    v.push(o("publicOnly", "Public Only", "Reverse public members only.", OptKind::Check, json!(false), true));
    v.push(o("typeHierarchy", "Type Hierarchy Diagram", "Create a type hierarchy diagram for all classes and interfaces", OptKind::Check, json!(true), true));
    v.push(o("packageOverview", "Package Overview Diagram", "Create overview diagram for each package", OptKind::Check, json!(true), true));
    v.push(o("packageStructure", "Package Structure Diagram", "Create a package structure diagram for all packages", OptKind::Check, json!(true), true));
    v
}

/// An option's value: the preference, else its default (`java.gen.javaDoc` falls back to the
/// `java.gen.doc` that earlier UMLCraft versions saved).
fn opt_value(prefs: &crate::prefs::Prefs, l: Lang, name: &str, rev: bool) -> serde_json::Value {
    let k = l.key();
    let key = format!("{k}.{}.{name}", if rev { "rev" } else { "gen" });
    if let Some(v) = prefs.get(&key) {
        return v.clone();
    }
    if !rev && name == doc_opt(l).0 {
        if let Some(v) = prefs.get(&format!("{k}.gen.doc")) {
            return v.clone();
        }
    }
    options(l).into_iter().find(|o| o.key == key).map(|o| o.default).unwrap_or(serde_json::Value::Null)
}

/// The options as preferences: `<lang>.gen.*` and `<lang>.rev.*` (the 7.x extensions' keys).
pub fn gen_opts(prefs: &crate::prefs::Prefs, l: Lang) -> GenOpts {
    let v = |n: &str| opt_value(prefs, l, n, false);
    let b = |n: &str, d: bool| v(n).as_bool().unwrap_or(d);
    let s = |n: &str, d: &str| v(n).as_str().unwrap_or(d).to_string();
    GenOpts {
        use_tab: b("useTab", false),
        indent_spaces: v("indentSpaces").as_i64().unwrap_or(l.default_indent()),
        doc: b(doc_opt(l).0, true),
        dbms: s("dbms", "mysql"),
        quote: b("quoteIdentifiers", true),
        drop_table: b("dropTable", false),
        include_header: b("includeHeader", true),
        use_vector: b("useVector", true),
        gen_cpp: b("genCpp", true),
        properties: b("propertiesByDefault", false),
        lower_case: b("lowerCase", true),
        install_path: s("installPath", ""),
        header: String::new(),
    }
}

pub fn rev_opts(prefs: &crate::prefs::Prefs, l: Lang) -> RevOpts {
    let b = |n: &str, d: bool| opt_value(prefs, l, n, true).as_bool().unwrap_or(d);
    RevOpts {
        skip_self: b("skipSelfParam", true),
        skip_magic: b("skipMagicMethods", true),
        association: b("association", true),
        public_only: b("publicOnly", false),
        // DDL: an ER diagram instead of a type hierarchy
        type_hierarchy: b(if l == Lang::Ddl { "erDiagram" } else { "typeHierarchy" }, true),
        package_overview: b("packageOverview", true),
        package_structure: b("packageStructure", true),
    }
}

/// The project's header comment (`/** … */` with its name, author, company, version and
/// copyright), empty when none is set.
pub fn project_header(r: &Repo) -> String {
    let p = r.root;
    let mut lines = vec![];
    let name = r.name(p);
    if !name.is_empty() {
        lines.push(format!("Project {name}"));
    }
    for (k, tag) in [("author", "@author"), ("company", "@company"), ("version", "@version"), ("copyright", "@copyright")] {
        let v = r.str(p, k);
        if !v.is_empty() {
            lines.push(format!("{tag} {v}"));
        }
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut s = String::from("/**\n");
    for l in lines {
        s.push_str(&format!(" * {l}\n"));
    }
    s.push_str(" */\n");
    s
}

/// The options' widgets (in the dialog's Options and in the Preferences dialog's language
/// sections): each change is saved at once. `rev` picks the Reverse tab's (None: all, with the
/// section headers of the 7.x Preferences).
pub fn options_ui(ui: &mut egui::Ui, prefs: &mut crate::prefs::Prefs, l: Lang, rev: Option<bool>, descriptions: bool) {
    let mut changed = false;
    let use_tab = gen_opts(prefs, l).use_tab;
    let mut last_rev = None;
    for o in options(l) {
        if rev.is_some_and(|r| r != o.rev) {
            continue;
        }
        if rev.is_none() && last_rev != Some(o.rev) {
            let title = format!("{} {}", l.label(), if o.rev { "Reverse Engineering" } else { if l == Lang::Ddl { "Generation" } else { "Code Generation" } });
            ui.label(egui::RichText::new(title).strong());
            last_rev = Some(o.rev);
        }
        let name = o.key.rsplit('.').next().unwrap_or("").to_string();
        let cur = opt_value(prefs, l, &name, o.rev);
        let mut new = None;
        ui.horizontal(|ui| match o.kind {
            OptKind::Check => {
                let mut b = cur.as_bool().unwrap_or(false);
                if ui.checkbox(&mut b, o.label).changed() {
                    new = Some(json!(b));
                }
            }
            OptKind::Number => {
                let mut n = cur.as_i64().unwrap_or(4);
                ui.add_enabled_ui(!use_tab, |ui| {
                    ui.label(o.label);
                    if ui.add(egui::DragValue::new(&mut n).range(0..=16)).changed() {
                        new = Some(json!(n));
                    }
                });
            }
            OptKind::Text => {
                let mut s = cur.as_str().unwrap_or("").to_string();
                ui.label(o.label);
                if ui.add(egui::TextEdit::singleline(&mut s).desired_width(220.0)).changed() {
                    new = Some(json!(s));
                }
            }
            OptKind::Dbms => {
                let mut s = cur.as_str().unwrap_or("mysql").to_string();
                ui.label(o.label);
                for (v, t) in [("mysql", "MySQL"), ("sqlite", "SQLite"), ("postgresql", "PostgreSQL")] {
                    if ui.radio_value(&mut s, v.to_string(), t).changed() {
                        new = Some(json!(s.clone()));
                    }
                }
            }
        });
        if descriptions {
            ui.label(egui::RichText::new(o.desc).small().weak());
        }
        if let Some(v) = new {
            prefs.set(&o.key, v);
            changed = true;
        }
    }
    if changed {
        prefs.flush();
    }
}

/// The Preferences dialog's sections for the languages, after the notations'.
pub fn pref_sections() -> Vec<&'static str> {
    Lang::ALL.iter().map(|l| l.label()).collect()
}

/// The Preferences dialog's section of a language, when `section` is one (its options and
/// descriptions).
pub fn prefs_section_ui(ui: &mut egui::Ui, prefs: &mut crate::prefs::Prefs, section: &str) {
    if let Some(l) = Lang::ALL.into_iter().find(|l| l.label() == section) {
        options_ui(ui, prefs, l, None, true);
    }
}

/// Restore Default Settings in a language's section: its options go back to their defaults.
pub fn restore_defaults(prefs: &mut crate::prefs::Prefs, section: &str) {
    if let Some(l) = Lang::ALL.into_iter().find(|l| l.label() == section) {
        for o in options(l) {
            prefs.values.remove(&o.key);
        }
        prefs.values.remove(&format!("{}.gen.doc", l.key()));
    }
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
            ..Default::default()
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
    let o = &GenOpts { header: project_header(r), ..o.clone() };
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
pub fn parse_source(l: Lang, src: &str, rel: &[String], o: &RevOpts) -> Parsed {
    match l {
        Lang::Java => java::parse(src),
        Lang::Cpp => cpp::parse(src, rel),
        Lang::CSharp => csharp::parse(src),
        Lang::C => c::parse(src, rel),
        Lang::Python => python::parse(src, rel, o),
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
    // Python's `Book | None` and TypeScript's `Book | null | undefined`
    if t.contains('|') && !t.contains('<') && !t.contains('[') {
        let parts: Vec<&str> = t.split('|').map(|p| p.trim()).collect();
        let kept: Vec<&str> = parts.iter().copied().filter(|p| !["None", "null", "undefined"].contains(p)).collect();
        if kept.len() == 1 && parts.len() > 1 {
            return (kept[0].to_string(), "0..1".into());
        }
    }
    const MANY: &[&str] = &[
        "List", "ArrayList", "LinkedList", "Set", "HashSet", "TreeSet", "Collection", "Iterable", "IList", "ICollection", "IEnumerable", "Array", "MutableList", "MutableSet", "Vec", "vector", "std::vector", "std::list", "list", "set", "Sequence", "Seq", "SortedSet", "NavigableSet", "LinkedHashSet", "Deque", "ArrayDeque", "Queue", "frozenset", "tuple", "deque",
    ];
    const ONE: [&str; 4] = ["Option", "Optional", "optional", "Nullable"];
    // wrappers that hold one of the type: smart pointers, boxes, Python's ClassVar and Final
    const WRAP: [&str; 10] = ["shared_ptr", "unique_ptr", "weak_ptr", "Box", "Rc", "Arc", "ClassVar", "Final", "Readonly", "Lazy"];
    for (open, shut) in [('<', '>'), ('[', ']')] {
        if let Some(i) = t.find(open) {
            if t.ends_with(shut) {
                // `java.util.List`, `typing.List` and `std::vector` are List and vector
                let head = t[..i].trim();
                let head = head.rsplit(['.', ':']).next().unwrap_or(head);
                let inner = t[i + 1..t.len() - 1].trim();
                if WRAP.contains(&head) && !inner.contains(',') {
                    return element_type(inner);
                }
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
    /// the index of the unit (file) it comes from, for its imports
    unit: usize,
}

/// A type name as a path: `a.b.Book`, `a::b::Book`, `\A\B\Book` → its segments; None when it is
/// not a plain (qualified) name (`Map<K, V>`, `int*`, `(A, B)`).
pub fn name_path(t: &str) -> Option<Vec<String>> {
    // a pointer or reference to a type refers to it: `const Book*`, `Book&`, `&mut Book`
    let mut t = t.trim();
    for p in ["const ", "struct ", "class ", "typename ", "&mut ", "&", "mut "] {
        t = t.strip_prefix(p).unwrap_or(t).trim();
    }
    let t = t.strip_suffix(" const").unwrap_or(t).trim_end_matches(['*', '&', ' ']);
    let t = t.trim_start_matches("::").trim_start_matches('\\');
    let segs: Vec<String> = t.split("::").flat_map(|s| s.split(['.', '\\'])).map(|s| s.to_string()).collect();
    let ok = |s: &String| !s.is_empty() && s.chars().next().map(|c| c.is_alphabetic() || c == '_' || c == '$').unwrap_or(false) && s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$');
    if segs.iter().all(ok) { Some(segs) } else { None }
}

/// A super type's name without its type arguments: `Base<T>` → `Base`, `a.Base[T]` → `a.Base`.
fn raw_name(t: &str) -> String {
    let t = t.trim();
    let end = t.find(['<', '[', '(']).unwrap_or(t.len());
    t[..end].trim().to_string()
}

/// The classifier or package reached from `from` by the names in `segs` (packages, then types and
/// their nested types).
fn walk_path(r: &Repo, from: Eid, segs: &[String]) -> Option<Eid> {
    let mut cur = from;
    for s in segs {
        cur = r.list(cur, "ownedElements").into_iter().find(|c| r.name(*c) == *s && (r.kind_of(*c, "UMLPackage") || classifier_kind(r.ty(*c)).is_some()))?;
    }
    Some(cur)
}

/// The names from `target` down to `e` (`a`, `b`, `Book`).
fn path_from(r: &Repo, target: Eid, e: Eid) -> Vec<String> {
    let mut out = vec![];
    let mut cur = Some(e);
    while let Some(c) = cur {
        if c == target {
            break;
        }
        out.push(r.name(c));
        cur = r.parent(c);
    }
    out.reverse();
    out
}

/// Resolves the type names used in reversed code (the 7.x reverses' order, `_findType` in
/// staruml-java's code-analyzer.js): a type parameter stays a name; then a nested type of the
/// type or of the types around it; an explicit import; a type of the same package or of the
/// packages around it; a wildcard import; and last any type of that name, the reversed ones first.
/// A name imported from outside the folder (`import java.util.Date;`) stays a name.
struct Resolver<'a> {
    target: Eid,
    made: &'a [(String, Eid)],
    others: &'a [(String, Eid)],
}

impl Resolver<'_> {
    fn is_classifier(r: &Repo, e: Eid) -> bool {
        classifier_kind(r.ty(e)).is_some() || r.kind_of(e, "UMLClassifier")
    }

    fn resolve(&self, r: &Repo, from: Eid, imports: &[Import], tparams: &[String], text: &str) -> Option<Eid> {
        let segs = name_path(text)?;
        let found = |e: Eid| Some(e).filter(|e| Self::is_classifier(r, *e));
        // the scopes around `from`: its types, then its packages, up to the target
        let mut scopes = vec![];
        let mut cur = Some(from);
        while let Some(c) = cur {
            scopes.push(c);
            if c == self.target {
                break;
            }
            cur = r.parent(c);
        }
        if segs.len() > 1 {
            // a qualified name: from the scopes outwards, then through an import's alias
            for &s in &scopes {
                if let Some(e) = walk_path(r, s, &segs).and_then(found) {
                    return Some(e);
                }
            }
            if let Some(i) = imports.iter().find(|i| !i.wildcard && i.alias == segs[0]) {
                let mut p = i.path.clone();
                p.extend(segs[1..].iter().cloned());
                if let Some(e) = walk_path(r, self.target, &p).and_then(found) {
                    return Some(e);
                }
                // `import * as m from './m'` then `m.Book`: the reversed Book
                if i.module && segs.len() == 2 {
                    return self.made.iter().find(|m| m.0 == segs[1]).map(|m| m.1);
                }
            }
            // a reversed type whose package path ends with the qualifier
            let last = segs.last().unwrap();
            return self.made.iter().find(|m| &m.0 == last && path_from(r, self.target, m.1).ends_with(&segs)).map(|m| m.1);
        }
        let n = &segs[0];
        // a type parameter of the operation or of the types around it
        let type_param = tparams.contains(n) || scopes.iter().any(|s| r.list(*s, "templateParameters").into_iter().any(|p| r.name(p) == *n));
        if type_param {
            return None;
        }
        let child = |s: Eid| r.list(s, "ownedElements").into_iter().find(|c| r.name(*c) == *n && Self::is_classifier(r, *c));
        // nested types of the type and the types around it
        for &s in scopes.iter().take_while(|s| Self::is_classifier(r, **s)) {
            if r.name(s) == *n && s != from {
                return Some(s);
            }
            if let Some(e) = child(s) {
                return Some(e);
            }
        }
        // an explicit import
        if let Some(i) = imports.iter().find(|i| !i.wildcard && i.alias == *n) {
            if let Some(e) = walk_path(r, self.target, &i.path).and_then(found) {
                return Some(e);
            }
            let real = i.path.last().unwrap_or(n);
            if i.module {
                // a module path: the reversed type of that name whose path shares the most with it
                let score = |e: Eid| path_from(r, self.target, e).iter().filter(|s| i.path.contains(s)).count();
                return self.made.iter().filter(|m| &m.0 == real).max_by_key(|m| score(m.1)).map(|m| m.1);
            }
            // a reversed type at that path under a package of the target (reversed into a package
            // that already had the outer ones)
            return self.made.iter().find(|m| &m.0 == real && i.path.ends_with(&path_from(r, self.target, m.1))).map(|m| m.1);
        }
        // the type's package and the packages around it
        for &s in scopes.iter().skip_while(|s| Self::is_classifier(r, **s)) {
            if let Some(e) = child(s) {
                return Some(e);
            }
        }
        // wildcard imports
        for i in imports.iter().filter(|i| i.wildcard) {
            if let Some(e) = walk_path(r, self.target, &i.path).and_then(child) {
                return Some(e);
            }
        }
        self.made.iter().chain(self.others.iter()).find(|m| m.0 == *n).map(|m| m.1)
    }
}

/// Import reversed units under `target`: packages merged by name, types merged by name in their
/// package, members deduplicated, then super types, typed fields (as associations when asked),
/// operations and parameters, with type names resolved through each file's imports and packages.
/// Returns the number of types.
pub fn import_units(r: &mut Repo, target: Eid, units: Vec<Unit>, o: &RevOpts) -> usize {
    let mut pend: Vec<Pending> = vec![];
    fn child_pkg(r: &mut Repo, parent: Eid, name: &str) -> Eid {
        if let Some(p) = r.list(parent, "ownedElements").into_iter().find(|c| r.kind_of(*c, "UMLPackage") && r.name(*c) == name) {
            return p;
        }
        r.create("UMLPackage", &[("name", json!(name))], Some((parent, "ownedElements")))
    }
    fn add_type(r: &mut Repo, owner: Eid, t: Ty, unit: usize, pend: &mut Vec<Pending>) {
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
                if t.is_leaf {
                    r.set(e, "isFinalSpecialization", json!(true));
                    r.set(e, "isLeaf", json!(true));
                }
                e
            }
        };
        // the template parameters (their bounds are typed once every type exists)
        if r.list(eid, "templateParameters").is_empty() {
            for g in &t.generics {
                r.create("UMLTemplateParameter", &[("name", json!(g))], Some((eid, "templateParameters")));
            }
        }
        let mut t = t;
        for n in std::mem::take(&mut t.nested) {
            add_type(r, eid, n, unit, pend);
        }
        pend.push(Pending { eid, ty: t, unit });
    }
    let mut count = 0;
    let mut imports: Vec<Vec<Import>> = vec![];
    for (k, u) in units.into_iter().enumerate() {
        let mut owner = target;
        for n in &u.ns {
            if !n.is_empty() {
                owner = child_pkg(r, owner, n);
            }
        }
        for t in u.types {
            add_type(r, owner, t, k, &mut pend);
        }
        imports.push(u.imports);
    }
    // the types by simple name: the reversed ones first, then the model's others
    let made: Vec<(String, Eid)> = pend.iter().map(|p| (r.name(p.eid), p.eid)).collect();
    let others: Vec<(String, Eid)> = r.all_of("UMLClassifier").into_iter().map(|c| (r.name(c), c)).collect();
    let res = Resolver { target, made: &made, others: &others };
    // a type name: a reference to the classifier it resolves to, else the name as written
    let set_type = |r: &mut Repo, e: Eid, field: &str, ty: &str, from: Eid, imp: &[Import], tp: &[String]| {
        if ty.trim().is_empty() {
            return;
        }
        match res.resolve(r, from, imp, tp, ty) {
            Some(c) => r.set_ref(e, field, Some(c)),
            None => r.set(e, field, json!(ty.trim())),
        }
    };
    let sig_of = |ty: &str| {
        let (elem, mult) = element_type(ty);
        format!("{}{}", base_name(&elem), if mult.is_empty() { "" } else { "*" })
    };
    for p in &pend {
        count += 1;
        let e = p.eid;
        let t = &p.ty;
        let imp: &[Import] = imports.get(p.unit).map(|v| v.as_slice()).unwrap_or(&[]);
        let public = |v: &str| !o.public_only || v == "public" || v.is_empty();
        // the template parameters' bounds and defaults
        let tps = r.list(e, "templateParameters");
        for (k, (bound, default)) in t.gen_bounds.iter().enumerate() {
            let Some(&tp) = tps.get(k) else { break };
            if !bound.is_empty() && r.str(tp, "parameterType").is_empty() && r.refd(tp, "parameterType").is_none() {
                set_type(r, tp, "parameterType", &raw_name(bound), e, imp, &[]);
                if raw_name(bound) != bound.trim() {
                    r.set(tp, "parameterType", json!(bound.trim()));
                }
            }
            if !default.is_empty() && r.str(tp, "defaultValue").is_empty() {
                r.set(tp, "defaultValue", json!(default));
            }
        }
        for (k, l) in t.literals.iter().enumerate() {
            if !r.list(e, "literals").into_iter().any(|x| r.name(x) == *l) {
                let lit = r.create("UMLEnumerationLiteral", &[("name", json!(l))], Some((e, "literals")));
                if let Some(d) = t.literal_docs.get(k).filter(|d| !d.is_empty()) {
                    r.set(lit, "documentation", json!(d));
                }
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
            let target_cls = res.resolve(r, e, imp, &[], &elem).filter(|c| r.kind_of(*c, "UMLClassifier"));
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
                    if f.read_only {
                        r.set(end, "isReadOnly", json!(true));
                    }
                    add_tags(r, end, &f.tags);
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
            set_type(r, a, "type", &elem, e, imp, &[]);
            if !mult.is_empty() {
                r.set(a, "multiplicity", json!(mult));
            }
            if f.read_only {
                r.set(a, "isReadOnly", json!(true));
            }
            if f.is_leaf {
                r.set(a, "isLeaf", json!(true));
            }
            add_tags(r, a, &f.tags);
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
            // the same operation from a header and its source file is kept once; overloads (other
            // parameter types) are kept apart
            let sig: Vec<String> = m.params.iter().map(|p| sig_of(&p.ty)).collect();
            let same = r.list(e, "operations").into_iter().any(|x| {
                let ps: Vec<String> = r.list(x, "parameters").into_iter().filter(|p| r.str(*p, "direction") != "return").map(|p| format!("{}{}", base_name(&views::type_name(r, p, "type")), if r.str(p, "multiplicity").is_empty() { "" } else { "*" })).collect();
                r.name(x) == m.name && (ps == sig || (ps.len() == sig.len() && ps.iter().zip(&sig).all(|(a, b)| a.is_empty() || b.is_empty() || a == b)))
            });
            if same {
                continue;
            }
            let op = r.create("UMLOperation", &[("name", json!(m.name))], Some((e, "operations")));
            if !m.vis.is_empty() && m.vis != "public" {
                r.set(op, "visibility", json!(m.vis));
            }
            if !m.stereotype.is_empty() {
                r.set(op, "stereotype", json!(m.stereotype));
            }
            if m.is_static {
                r.set(op, "isStatic", json!(true));
            }
            if m.is_abstract {
                r.set(op, "isAbstract", json!(true));
            }
            if m.is_leaf {
                r.set(op, "isLeaf", json!(true));
            }
            if m.concurrent {
                r.set(op, "concurrency", json!("concurrent"));
            }
            add_tags(r, op, &m.tags);
            if !m.doc.is_empty() {
                r.set(op, "documentation", json!(m.doc));
            }
            for g in &m.generics {
                r.create("UMLTemplateParameter", &[("name", json!(g))], Some((op, "templateParameters")));
            }
            for p in &m.params {
                let pe = r.create("UMLParameter", &[("name", json!(p.name))], Some((op, "parameters")));
                let (elem, mult) = element_type(&p.ty);
                set_type(r, pe, "type", &elem, e, imp, &m.generics);
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
                set_type(r, pe, "type", &elem, e, imp, &m.generics);
                if !mult.is_empty() {
                    r.set(pe, "multiplicity", json!(mult));
                }
            }
            // the declared exceptions, when they resolve to a classifier
            let raised: Vec<Eid> = m.raises.iter().filter_map(|x| res.resolve(r, e, imp, &m.generics, x)).collect();
            if !raised.is_empty() {
                r.set_ref_list(op, "raisedExceptions", &raised);
            }
        }
        // super types: a realization when an interface is reached from a class, else a generalization
        for (n, explicit_real) in t.supers.iter().map(|s| (s, false)).chain(t.reals.iter().map(|s| (s, true))) {
            let Some(target) = res.resolve(r, r.parent(e).unwrap_or(e), imp, &[], &raw_name(n)).or_else(|| res.resolve(r, e, imp, &[], &raw_name(n))) else { continue };
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

/// Tags on an element: a boolean one (checked) when the value is empty, else a string one.
fn add_tags(r: &mut Repo, e: Eid, tags: &[(String, String)]) {
    for (n, v) in tags {
        if v.is_empty() {
            r.create("Tag", &[("name", json!(n)), ("kind", json!("boolean")), ("checked", json!(true))], Some((e, "tags")));
        } else {
            r.create("Tag", &[("name", json!(n)), ("kind", json!("string")), ("value", json!(v))], Some((e, "tags")));
        }
    }
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

    #[cfg(test)]
    /// Write the base package's code into `out`: existing files are overwritten. Returns the
    /// files written, relative to `out`.
    pub fn generate_code(&mut self, l: Lang, base: Eid, out: &Path) -> Result<Vec<PathBuf>, String> {
        let o = gen_opts(&self.prefs, l);
        let files = generate_files(&self.repo, base, l, &o);
        let never = AtomicBool::new(false);
        Ok(write_files(files, out, &never, &mut |_| {})?.unwrap_or_default())
    }

    #[cfg(test)]
    /// Reverse a source folder into `target` (or a new model `<Lang>Reverse` in the project), as
    /// one undo step, with the diagrams the options ask for; the imported package is selected.
    /// Returns the log lines.
    pub fn reverse_code(&mut self, l: Lang, target: Option<Eid>, dir: &Path, ms: &dyn Measure) -> Result<Vec<String>, String> {
        let never = AtomicBool::new(false);
        let mut log = vec![];
        let o = rev_opts(&self.prefs, l);
        let src = read_sources(l, dir, &o, &never, &mut |s| log.push(s))?.unwrap_or_default();
        log.extend(self.import_sources(l, target, src, ms));
        Ok(log)
    }

    /// The second half of a reverse, on the main thread: what `read_sources` read goes into
    /// `target` (or a new model) as one undo step, then the diagrams. Returns the log lines.
    pub fn import_sources(&mut self, l: Lang, target: Option<Eid>, src: Sources, ms: &dyn Measure) -> Vec<String> {
        let o = rev_opts(&self.prefs, l);
        let mut log = vec![];
        let target = target.filter(|t| self.repo.alive(*t));
        let undo = self.undo.len();
        let root = self.repo.root;
        let made_new = target.is_none();
        let (units, tables) = (src.units, src.tables);
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
        log.push(format!("{} files analyzed, {count} {} imported.", src.files, if l == Lang::Ddl { "tables" } else { "types" }));
        if src.errors > 0 {
            log.push(format!("{} files had syntax errors; the rest of them was analyzed.", src.errors));
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
        log
    }
}

/// Write generated files under `out` (existing ones are overwritten), logging each one; stops
/// between two files when `cancel` is set (`Ok(None)`). Returns the files written, relative to
/// `out`. Runs on the dialog's worker thread.
pub fn write_files(files: Vec<OutFile>, out: &Path, cancel: &AtomicBool, log: &mut dyn FnMut(String)) -> Result<Option<Vec<PathBuf>>, String> {
    let mut done = vec![];
    for (rel, text) in files {
        if cancel.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let p = out.join(&rel);
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        std::fs::write(&p, text).map_err(|e| format!("{}: {e}", p.display()))?;
        log(format!("Generated {}", rel.display()));
        done.push(rel);
    }
    Ok(Some(done))
}

/// What a reverse read from a source folder, before it goes into the model.
#[derive(Default)]
pub struct Sources {
    pub files: usize,
    pub errors: usize,
    pub units: Vec<Unit>,
    pub tables: Vec<ddl::Table>,
}

/// The first half of a reverse, on the dialog's worker thread: every source file of the folder is
/// read and parsed, with a log line each; stops between two files when `cancel` is set (`Ok(None)`).
pub fn read_sources(l: Lang, dir: &Path, o: &RevOpts, cancel: &AtomicBool, log: &mut dyn FnMut(String)) -> Result<Option<Sources>, String> {
    let files = source_files(dir, l);
    if files.is_empty() {
        return Err(format!("No {} source files found in {}.", l.label(), dir.display()));
    }
    let mut s = Sources { files: files.len(), ..Default::default() };
    for f in &files {
        if cancel.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let Ok(bytes) = std::fs::read(f) else {
            s.errors += 1;
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        let rel: Vec<String> = f.parent().and_then(|p| p.strip_prefix(dir).ok()).map(|p| p.iter().map(|s| s.to_string_lossy().to_string()).collect()).unwrap_or_default();
        log(format!("Reading {}", f.strip_prefix(dir).unwrap_or(f).display()));
        if l == Lang::Ddl {
            let (t, err) = ddl::parse(&text);
            s.errors += err as usize;
            s.tables.extend(t);
        } else {
            let p = parse_source(l, &text, &rel, o);
            s.errors += p.errors as usize;
            s.units.extend(p.units);
        }
    }
    Ok(Some(s))
}

/// A Generate or a Reverse running on a worker thread: its messages, and the flag that stops it
/// (the Cancel button, or closing the dialog).
pub struct Job {
    pub cancel: Arc<AtomicBool>,
    rx: mpsc::Receiver<JobMsg>,
    out: PathBuf,
    target: Option<Eid>,
}

enum JobMsg {
    Log(String),
    Generated(Result<Option<Vec<PathBuf>>, String>),
    Read(Result<Option<Sources>, String>),
}

impl Job {
    /// Start a Generate: the files' text is made from a copy of the model, then written, on the
    /// worker thread.
    pub fn generate(repo: Repo, l: Lang, base: Eid, o: GenOpts, out: PathBuf, ctx: Option<egui::Context>) -> Job {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let (c, dir) = (cancel.clone(), out.clone());
        std::thread::spawn(move || {
            let files = generate_files(&repo, base, l, &o);
            let t2 = tx.clone();
            let ctx2 = ctx.clone();
            let res = write_files(files, &dir, &c, &mut |s| {
                let _ = t2.send(JobMsg::Log(s));
                if let Some(x) = &ctx2 {
                    x.request_repaint();
                }
            });
            let _ = tx.send(JobMsg::Generated(res));
            if let Some(x) = &ctx {
                x.request_repaint();
            }
        });
        Job { cancel, rx, out, target: None }
    }

    /// Start a Reverse: the files are read and parsed on the worker thread; the import into the
    /// model happens on the main thread when they are all read.
    pub fn reverse(l: Lang, dir: PathBuf, o: RevOpts, target: Option<Eid>, ctx: Option<egui::Context>) -> Job {
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel();
        let c = cancel.clone();
        let d2 = dir.clone();
        std::thread::spawn(move || {
            let t2 = tx.clone();
            let ctx2 = ctx.clone();
            let res = read_sources(l, &d2, &o, &c, &mut |s| {
                let _ = t2.send(JobMsg::Log(s));
                if let Some(x) = &ctx2 {
                    x.request_repaint();
                }
            });
            let _ = tx.send(JobMsg::Read(res));
            if let Some(x) = &ctx {
                x.request_repaint();
            }
        });
        Job { cancel, rx, out: dir, target }
    }
}

impl App {
    /// Take a running job's news into the dialog's log; when the job is over, finish it (a
    /// Reverse is imported here) and return true.
    pub fn poll_codegen_job(&mut self, d: &mut Dialog, ms: &dyn Measure) -> bool {
        let Some(job) = d.job.as_ref() else { return false };
        let mut end = None;
        loop {
            match job.rx.try_recv() {
                Ok(JobMsg::Log(s)) => d.log.push(s),
                Ok(m) => {
                    end = Some(m);
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    end = Some(JobMsg::Read(Err("the job stopped unexpectedly".into())));
                    break;
                }
            }
        }
        let Some(end) = end else { return false };
        let job = d.job.take().unwrap();
        match end {
            JobMsg::Generated(Ok(Some(files))) => {
                d.log.push(format!("{} files generated in {}.", files.len(), job.out.display()));
                d.last_out = Some(job.out);
            }
            JobMsg::Read(Ok(Some(src))) => {
                let lines = self.import_sources(d.lang, job.target, src, ms);
                d.log.extend(lines);
            }
            JobMsg::Generated(Ok(None)) | JobMsg::Read(Ok(None)) => d.log.push("Cancelled.".into()),
            JobMsg::Generated(Err(e)) | JobMsg::Read(Err(e)) => d.log.push(format!("Error: {e}")),
            JobMsg::Log(_) => {}
        }
        true
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
    /// the Generate or Reverse running on its worker thread
    pub job: Option<Job>,
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
        self.codegen = Some(Dialog { lang: l, tab: Tab::Generate, base, target, out_dir: folder(self, "lastOutputFolder"), src_dir: folder(self, "lastSourceFolder"), log: vec![], last_out: None, job: None });
    }

    pub fn codegen_ui(&mut self, ctx: &egui::Context) {
        let Some(mut d) = self.codegen.take() else { return };
        let l = d.lang;
        let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
        self.poll_codegen_job(&mut d, &ms);
        let running = d.job.is_some();
        if running {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        let mut cancel = false;
        let mut open = true;
        let (mut run, mut close) = (false, false);
        let bases = self.codegen_bases(l);
        let targets: Vec<Eid> = if l == Lang::Ddl { bases.clone() } else { bases.iter().copied().collect() };
        let label = |a: &App, e: Eid| views::display_name(&a.repo, e);
        egui::Window::new(l.label()).id(egui::Id::new("codegen-dialog")).open(&mut open).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(520.0);
            // the form stays as it is while a job runs
            ui.add_enabled_ui(!running, |ui| {
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
                // the language's options for this tab, saved as preferences at once
                let rev = d.tab == Tab::Reverse;
                egui::CollapsingHeader::new("Options").default_open(true).show(ui, |ui| options_ui(ui, &mut self.prefs, l, Some(rev), false));
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
                    // the button becomes Cancel while the job runs
                    if running {
                        cancel = ui.button("Cancel").clicked();
                    } else {
                        run = ui.add_enabled(ok, egui::Button::new(if d.tab == Tab::Generate { "Generate" } else { "Reverse" })).clicked();
                    }
                });
            });
            close |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if cancel {
            if let Some(j) = &d.job {
                j.cancel.store(true, Ordering::Relaxed);
            }
        }
        if run {
            let k = l.key();
            match d.tab {
                Tab::Generate => {
                    let out = PathBuf::from(d.out_dir.trim());
                    self.prefs.set(&format!("{k}.lastOutputFolder"), json!(d.out_dir.trim()));
                    self.prefs.flush();
                    let o = gen_opts(&self.prefs, l);
                    d.job = Some(Job::generate(self.repo.clone(), l, d.base.unwrap(), o, out, Some(ctx.clone())));
                }
                Tab::Reverse => {
                    self.prefs.set(&format!("{k}.lastSourceFolder"), json!(d.src_dir.trim()));
                    self.prefs.flush();
                    let o = rev_opts(&self.prefs, l);
                    d.job = Some(Job::reverse(l, PathBuf::from(d.src_dir.trim()), o, d.target, Some(ctx.clone())));
                }
            }
        }
        if open && !close {
            self.codegen = Some(d);
        } else if let Some(j) = &d.job {
            // closing the dialog stops the job
            j.cancel.store(true, Ordering::Relaxed);
        }
    }
}
