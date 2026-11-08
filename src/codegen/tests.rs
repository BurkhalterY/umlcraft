//! Code generation for a small model in each language, and reverse engineering of small sources.

use super::*;
use crate::app::App;
use crate::model::Repo;
use serde_json::json;

/// Library model: core { LibraryItem (abstract), Book : LibraryItem, Lendable, Book ⇢ Lendable,
/// Status { AVAILABLE, LOANED } }, core.people { Member → books: Book[*] }.
fn library() -> (Repo, Eid) {
    let mut r = Repo::blank();
    let root = r.root;
    let m = r.create("UMLModel", &[("name", json!("Lib"))], Some((root, "ownedElements")));
    let core = r.create("UMLPackage", &[("name", json!("core"))], Some((m, "ownedElements")));
    let people = r.create("UMLPackage", &[("name", json!("people"))], Some((core, "ownedElements")));
    let item = r.create("UMLClass", &[("name", json!("LibraryItem")), ("documentation", json!("Anything that can be lent.")), ("isAbstract", json!(true))], Some((core, "ownedElements")));
    r.create("UMLAttribute", &[("name", json!("title")), ("visibility", json!("protected")), ("type", json!("String"))], Some((item, "attributes")));
    let count = r.create("UMLAttribute", &[("name", json!("count")), ("type", json!("Integer")), ("defaultValue", json!("0"))], Some((item, "attributes")));
    r.set(count, "isStatic", json!(true));
    let book = r.create("UMLClass", &[("name", json!("Book"))], Some((core, "ownedElements")));
    r.create("UMLAttribute", &[("name", json!("isbn")), ("visibility", json!("private")), ("type", json!("String"))], Some((book, "attributes")));
    let op = r.create("UMLOperation", &[("name", json!("pages"))], Some((book, "operations")));
    r.create("UMLParameter", &[("name", json!("from")), ("type", json!("Integer"))], Some((op, "parameters")));
    r.create("UMLParameter", &[("type", json!("Integer")), ("direction", json!("return"))], Some((op, "parameters")));
    let lend = r.create("UMLInterface", &[("name", json!("Lendable"))], Some((core, "ownedElements")));
    let lop = r.create("UMLOperation", &[("name", json!("lend"))], Some((lend, "operations")));
    r.create("UMLParameter", &[("name", json!("days")), ("type", json!("Integer"))], Some((lop, "parameters")));
    r.create("UMLParameter", &[("type", json!("Boolean")), ("direction", json!("return"))], Some((lop, "parameters")));
    let st = r.create("UMLEnumeration", &[("name", json!("Status"))], Some((core, "ownedElements")));
    for l in ["AVAILABLE", "LOANED"] {
        r.create("UMLEnumerationLiteral", &[("name", json!(l))], Some((st, "literals")));
    }
    let (rb, ri) = (r.ref_to(book), r.ref_to(item));
    r.create("UMLGeneralization", &[("source", rb.clone()), ("target", ri)], Some((book, "ownedElements")));
    let rl = r.ref_to(lend);
    r.create("UMLInterfaceRealization", &[("source", rb.clone()), ("target", rl)], Some((book, "ownedElements")));
    let member = r.create("UMLClass", &[("name", json!("Member"))], Some((people, "ownedElements")));
    let a = r.create("UMLAssociation", &[], Some((member, "ownedElements")));
    let rm = r.ref_to(member);
    r.create("UMLAssociationEnd", &[("reference", rm)], Some((a, "end1")));
    r.create("UMLAssociationEnd", &[("name", json!("books")), ("reference", rb), ("navigable", json!("navigable")), ("multiplicity", json!("0..*"))], Some((a, "end2")));
    (r, core)
}

fn measure() -> crate::canvas::EguiMeasure {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
    out.textures_delta.clear();
    crate::canvas::EguiMeasure { ctx }
}

fn opts() -> GenOpts {
    GenOpts { use_tab: false, indent_spaces: 4, doc: true, dbms: "mysql".into(), quote: true, drop_table: false }
}

fn files(l: Lang) -> Vec<(String, String)> {
    let (r, core) = library();
    generate_files(&r, core, l, &opts()).into_iter().map(|(p, t)| (p.to_string_lossy().replace('\\', "/"), t)).collect()
}

fn file<'a>(f: &'a [(String, String)], name: &str) -> &'a str {
    &f.iter().find(|x| x.0 == name).unwrap_or_else(|| panic!("no {name} in {:?}", f.iter().map(|x| &x.0).collect::<Vec<_>>())).1
}

fn has(text: &str, lines: &[&str]) {
    for l in lines {
        assert!(text.lines().any(|x| x.trim() == *l), "missing {l:?} in\n{text}");
    }
}

#[test]
fn generates_java() {
    let f = files(Lang::Java);
    has(file(&f, "Book.java"), &["import java.util.*;", "public class Book extends LibraryItem implements Lendable {", "private String isbn;", "public int pages(int from) {", "// TODO implement here", "return 0;", "public boolean lend(int days) {", "public Book() {"]);
    has(file(&f, "LibraryItem.java"), &["public abstract class LibraryItem {", "protected String title;", "public static int count = 0;", "* Anything that can be lent."]);
    has(file(&f, "Lendable.java"), &["public interface Lendable {", "boolean lend(int days);"]);
    has(file(&f, "Status.java"), &["public enum Status {", "AVAILABLE, LOANED;"]);
    has(file(&f, "people/Member.java"), &["package people;", "public List<Book> books;"]);
}

#[test]
fn generates_csharp_cpp_and_c() {
    let f = files(Lang::CSharp);
    has(file(&f, "Book.cs"), &["public class Book : LibraryItem, Lendable {", "private string isbn;", "public int pages(int from) {", "return 0;"]);
    has(file(&f, "people/Member.cs"), &["namespace people {", "public List<Book> books;"]);
    has(file(&f, "Status.cs"), &["public enum Status {", "AVAILABLE,", "LOANED"]);
    let f = files(Lang::Cpp);
    has(file(&f, "Book.h"), &["#ifndef _BOOK_H", "class Book : public LibraryItem, public Lendable {", "public:", "Book();", "int pages(int from);", "private:", "std::string isbn;"]);
    has(file(&f, "Book.cpp"), &["#include \"Book.h\"", "int Book::pages(int from) {", "return 0;"]);
    has(file(&f, "Lendable.h"), &["virtual bool lend(int days) = 0;"]);
    has(file(&f, "people/Member.h"), &["namespace people {", "std::vector<Book> books;", "#include \"../Book.h\""]);
    let f = files(Lang::C);
    has(file(&f, "Book.h"), &["typedef struct Book {", "LibraryItem base;", "char* isbn;", "} Book;", "int Book_pages(Book* self, int from);"]);
    has(file(&f, "Book.c"), &["int Book_pages(Book* self, int from) {"]);
    has(file(&f, "Status.h"), &["STATUS_AVAILABLE,", "STATUS_LOANED"]);
    has(file(&f, "Lendable.h"), &["bool (*lend)(void* self, int days);"]);
}

#[test]
fn generates_python_php_typescript_and_javascript() {
    let f = files(Lang::Python);
    has(file(&f, "book.py"), &["from library_item import LibraryItem", "class Book(LibraryItem, Lendable):", "self.__isbn: str = \"\"", "def pages(self, from: int) -> int:", "pass"]);
    has(file(&f, "library_item.py"), &["from abc import ABC, abstractmethod", "class LibraryItem(ABC):", "count: int = 0", "\"\"\"Anything that can be lent.\"\"\""]);
    has(file(&f, "lendable.py"), &["@abstractmethod", "def lend(self, days: int) -> bool:"]);
    has(file(&f, "status.py"), &["class Status(Enum):", "AVAILABLE = 1"]);
    assert!(f.iter().any(|x| x.0 == "people/__init__.py"));
    let f = files(Lang::Php);
    has(file(&f, "Book.php"), &["<?php", "class Book extends LibraryItem implements Lendable", "private string $isbn;", "public function pages(int $from): int", "return 0;"]);
    has(file(&f, "people/Member.php"), &["namespace people;", "public array $books;"]);
    has(file(&f, "Status.php"), &["enum Status", "case AVAILABLE;"]);
    let f = files(Lang::TypeScript);
    has(file(&f, "Book.ts"), &["import { LibraryItem } from \"./LibraryItem\";", "export class Book extends LibraryItem implements Lendable {", "private isbn: string;", "pages(from: number): number {", "return 0;"]);
    has(file(&f, "people/Member.ts"), &["import { Book } from \"../Book\";", "books: Book[];"]);
    has(file(&f, "Lendable.ts"), &["export interface Lendable {", "lend(days: number): boolean;"]);
    let f = files(Lang::JavaScript);
    has(file(&f, "Book.js"), &["import { LibraryItem } from \"./LibraryItem.js\";", "export class Book extends LibraryItem {", "super();", "this.#isbn = null;", "pages(from) {"]);
    has(file(&f, "Status.js"), &["export const Status = Object.freeze({", "AVAILABLE: \"AVAILABLE\","]);
}

/// Reverse a source text and import it into a new model; returns the repo and the model.
fn reverse(l: Lang, src: &str, rel: &[&str], file: &str) -> (Repo, Eid) {
    let mut r = Repo::blank();
    let root = r.root;
    let m = r.create("UMLModel", &[("name", json!(l.reverse_model()))], Some((root, "ownedElements")));
    let rel: Vec<String> = rel.iter().map(|s| s.to_string()).collect();
    let p = parse_source(l, src, &rel, file);
    assert!(!p.errors, "{l:?} reported errors");
    import_units(&mut r, m, p.units, &RevOpts { association: true, public_only: false, type_hierarchy: false, package_overview: false, package_structure: false });
    (r, m)
}

fn find(r: &Repo, ty: &str, name: &str) -> Eid {
    r.all_of(ty).into_iter().find(|e| r.name(*e) == name).unwrap_or_else(|| panic!("no {ty} {name}"))
}

fn attr(r: &Repo, c: Eid, name: &str) -> Eid {
    r.list(c, "attributes").into_iter().find(|a| r.name(*a) == name).unwrap_or_else(|| panic!("no attribute {name} in {}", r.name(c)))
}

fn op(r: &Repo, c: Eid, name: &str) -> Eid {
    r.list(c, "operations").into_iter().find(|a| r.name(*a) == name).unwrap_or_else(|| panic!("no operation {name} in {}", r.name(c)))
}

/// The parameters of an operation as `name: type` (`: type` for the return).
fn sig(r: &Repo, o: Eid) -> Vec<String> {
    r.list(o, "parameters").into_iter().map(|p| format!("{}: {}", r.name(p), crate::views::type_name(r, p, "type"))).collect()
}

fn generalizes(r: &Repo, child: Eid, parent: Eid, kind: &str) -> bool {
    r.all_of(kind).into_iter().any(|g| r.refd(g, "source") == Some(child) && r.refd(g, "target") == Some(parent))
}

/// The shape the reverse tests share: Shape (abstract or interface) with area(); Circle extends or
/// realizes it with radius: double and a list of tags; Color { RED, GREEN }.
fn check_shapes(r: &Repo, pkg_path: &[&str], radius_ty: &str, shape_kind: &str) {
    let circle = find(r, "UMLClass", "Circle");
    let shape = find(r, shape_kind, "Shape");
    let mut p = r.parent(circle).unwrap();
    for n in pkg_path.iter().rev() {
        assert_eq!(&r.name(p), n);
        p = r.parent(p).unwrap();
    }
    let rad = attr(r, circle, "radius");
    assert_eq!(crate::views::type_name(r, rad, "type"), radius_ty);
    let area = op(r, circle, "area");
    assert!(sig(r, area).iter().any(|s| s.starts_with(": ")), "area returns something: {:?}", sig(r, area));
    let kind = if shape_kind == "UMLInterface" { "UMLInterfaceRealization" } else { "UMLGeneralization" };
    assert!(generalizes(r, circle, shape, kind), "Circle → Shape as {kind}");
    let color = find(r, "UMLEnumeration", "Color");
    let lits: Vec<String> = r.list(color, "literals").into_iter().map(|l| r.name(l)).collect();
    assert_eq!(lits, ["RED", "GREEN"]);
}

#[test]
fn reverses_java() {
    let src = r#"
package com.acme.geo;

import java.util.*;

/** A shape. */
public abstract class Shape {
    public abstract double area();
}

@Entity
public class Circle extends Shape implements Comparable<Circle> {
    private double radius = 1.0;
    protected List<Shape> parts;
    public static final int SIDES = 0;
    public Circle(double r) { this.radius = r; }
    @Override
    public double area() { return Math.PI * radius * radius; }
    public void scale(double factor, Map<String, Integer> opts) { }
    class Inner { int x; }
}

enum Color { RED, GREEN; Color() {} }
"#;
    let (r, _) = reverse(Lang::Java, src, &[], "Shapes.java");
    check_shapes(&r, &["com", "acme", "geo"], "double", "UMLClass");
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.str(attr(&r, circle, "radius"), "defaultValue"), "1.0");
    assert_eq!(r.str(attr(&r, circle, "radius"), "visibility"), "private");
    assert!(r.bool(attr(&r, circle, "SIDES"), "isStatic"));
    assert_eq!(sig(&r, op(&r, circle, "scale")), ["factor: double", "opts: Map<String, Integer>"]);
    assert!(r.bool(find(&r, "UMLClass", "Shape"), "isAbstract"));
    assert_eq!(r.str(find(&r, "UMLClass", "Shape"), "documentation"), "A shape.");
    // List<Shape> parts is an association to Shape with multiplicity *
    let a = r.all_of("UMLAssociation").into_iter().find(|a| r.child(*a, "end2").map(|e| r.name(e) == "parts").unwrap_or(false)).expect("parts association");
    assert_eq!(r.str(r.child(a, "end2").unwrap(), "multiplicity"), "*");
    find(&r, "UMLClass", "Inner");
}

#[test]
fn reverses_csharp_cpp_and_c() {
    let cs = r#"
namespace Acme.Geo {
    /// <summary>A shape.</summary>
    public interface Shape { double Area(); }
    public enum Color { RED, GREEN }
    public class Circle : Shape {
        private double radius;
        public string Name { get; set; }
        public double area() { return 0; }
        public Circle() { }
    }
}
"#;
    let (r, _) = reverse(Lang::CSharp, cs, &[], "Shapes.cs");
    check_shapes(&r, &["Acme", "Geo"], "double", "UMLInterface");
    assert_eq!(crate::views::type_name(&r, attr(&r, find(&r, "UMLClass", "Circle"), "Name"), "type"), "string");
    let cpp = r#"
#include <vector>
namespace geo {
/** A shape. */
class Shape {
public:
    virtual ~Shape() {}
    virtual double area() const = 0;
};
enum class Color { RED, GREEN };
class Circle : public Shape {
public:
    Circle(double r);
    double area() const override;
    static int count;
private:
    double radius;
    std::vector<int> tags;
};
}
double geo::Circle::area() const { return 0; }
"#;
    let (r, _) = reverse(Lang::Cpp, cpp, &[], "shapes.h");
    check_shapes(&r, &["geo"], "double", "UMLInterface");
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.str(attr(&r, circle, "radius"), "visibility"), "private");
    assert_eq!(r.str(attr(&r, circle, "tags"), "multiplicity"), "*");
    let c = r#"
#ifndef SHAPES_H
#define SHAPES_H
typedef struct Shape {
    double (*area)(void* self);
} Shape;
typedef enum { RED, GREEN } Color;
typedef struct Circle {
    Shape base;
    double radius;
} Circle;
double Circle_area(Circle* self);
Circle* Circle_new(double radius);
#endif
"#;
    let (r, _) = reverse(Lang::C, c, &["geo"], "shapes.h");
    check_shapes(&r, &["geo"], "double", "UMLInterface");
    assert!(r.bool(op(&r, find(&r, "UMLClass", "Circle"), "new"), "isStatic"));
}

#[test]
fn reverses_python_php_typescript_and_javascript() {
    let py = r#"
from abc import ABC, abstractmethod
from enum import Enum

class Shape(ABC):
    """A shape."""
    @abstractmethod
    def area(self) -> float:
        ...

class Color(Enum):
    RED = 1
    GREEN = 2

class Circle(Shape):
    sides = 0

    def __init__(self, radius: float = 1.0):
        self.radius: float = radius
        self._tags: list[str] = []
        if radius < 0:
            self.__bad = True

    def area(self) -> float:
        return 3.14 * self.radius ** 2

    @staticmethod
    def unit(scale: float = 1.0) -> "Circle":
        return Circle(scale)
"#;
    let (r, _) = reverse(Lang::Python, py, &["geo"], "shapes.py");
    check_shapes(&r, &["geo"], "float", "UMLInterface");
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.str(attr(&r, circle, "tags"), "visibility"), "protected");
    assert_eq!(r.str(attr(&r, circle, "tags"), "multiplicity"), "*");
    assert_eq!(r.str(attr(&r, circle, "bad"), "visibility"), "private");
    assert!(r.bool(attr(&r, circle, "sides"), "isStatic"));
    assert!(r.bool(op(&r, circle, "unit"), "isStatic"));
    assert_eq!(r.str(find(&r, "UMLInterface", "Shape"), "documentation"), "A shape.");
    let php = r#"<?php
namespace Acme\Geo;

interface Shape { public function area(): float; }
enum Color: string { case RED = 'r'; case GREEN = 'g'; }
/** A circle. */
final class Circle implements Shape {
    const SIDES = 0;
    private float $radius = 1.0;
    protected array $tags = [];
    public function __construct(private string $name) {}
    public function area(): float { return 0.0; }
    public static function unit(float $scale = 1.0): Circle { return new Circle(); }
}
"#;
    let (r, _) = reverse(Lang::Php, php, &[], "Shapes.php");
    check_shapes(&r, &["Acme", "Geo"], "float", "UMLInterface");
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.str(attr(&r, circle, "name"), "visibility"), "private");
    assert_eq!(sig(&r, op(&r, circle, "unit")), ["scale: float", ": Circle"]);
    let ts = r#"
export interface Shape {
  area(): number;
}
export enum Color { RED, GREEN }
/** A circle. */
export class Circle implements Shape {
  private radius: number = 1;
  static count = 0
  tags?: string[]
  constructor(public name: string) {}
  area(): number { return 0 }
  get label(): string { return this.name }
}
"#;
    let (r, _) = reverse(Lang::TypeScript, ts, &["geo"], "shapes.ts");
    check_shapes(&r, &["geo"], "number", "UMLInterface");
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.str(attr(&r, circle, "tags"), "multiplicity"), "*");
    assert!(r.bool(attr(&r, circle, "label"), "isReadOnly"));
    attr(&r, circle, "name");
    let js = r#"
export class Shape {
  area() { throw new Error("abstract"); }
}
export class Circle extends Shape {
  static count = 0;
  #secret = 1;
  constructor(radius) {
    super();
    this.radius = radius;
  }
  area() { return Math.PI * this.radius ** 2; }
}
"#;
    let (r, _) = reverse(Lang::JavaScript, js, &[], "shapes.js");
    let circle = find(&r, "UMLClass", "Circle");
    assert!(generalizes(&r, circle, find(&r, "UMLClass", "Shape"), "UMLGeneralization"));
    attr(&r, circle, "radius");
    assert_eq!(r.str(attr(&r, circle, "secret"), "visibility"), "private");
    op(&r, circle, "area");
}

#[test]
fn ddl_round_trip() {
    let mut r = Repo::blank();
    let root = r.root;
    let dm = r.create("ERDDataModel", &[("name", json!("Shop"))], Some((root, "ownedElements")));
    let cust = r.create("ERDEntity", &[("name", json!("customer"))], Some((dm, "ownedElements")));
    let cid = r.create("ERDColumn", &[("name", json!("id")), ("type", json!("INTEGER")), ("primaryKey", json!(true))], Some((cust, "columns")));
    r.create("ERDColumn", &[("name", json!("email")), ("type", json!("VARCHAR")), ("length", json!("80")), ("unique", json!(true))], Some((cust, "columns")));
    let ord = r.create("ERDEntity", &[("name", json!("orders"))], Some((dm, "ownedElements")));
    r.create("ERDColumn", &[("name", json!("id")), ("type", json!("INTEGER")), ("primaryKey", json!(true))], Some((ord, "columns")));
    let fk = r.create("ERDColumn", &[("name", json!("customer_id")), ("type", json!("INTEGER")), ("foreignKey", json!(true)), ("nullable", json!(true))], Some((ord, "columns")));
    r.set_ref(fk, "referenceTo", Some(cid));
    let mut o = opts();
    let out = generate_files(&r, dm, Lang::Ddl, &o);
    assert_eq!(out[0].0, PathBuf::from("Shop.sql"));
    has(&out[0].1, &["CREATE TABLE `customer` (", "`id` INTEGER NOT NULL,", "`email` VARCHAR(80) NOT NULL UNIQUE,", "PRIMARY KEY (`id`)", "`customer_id` INTEGER,", "ALTER TABLE `orders` ADD FOREIGN KEY (`customer_id`) REFERENCES `customer` (`id`);"]);
    o.dbms = "sqlite".into();
    o.drop_table = true;
    let out = generate_files(&r, dm, Lang::Ddl, &o);
    has(&out[0].1, &["DROP TABLE IF EXISTS \"orders\";", "FOREIGN KEY (\"customer_id\") REFERENCES \"customer\" (\"id\")"]);
    // and back
    let (tables, err) = ddl::parse(&out[0].1);
    assert!(!err);
    assert_eq!(tables.len(), 2);
    let sql = r#"
-- shop
CREATE TABLE IF NOT EXISTS public.product (
  id SERIAL PRIMARY KEY,
  name VARCHAR(40) NOT NULL,
  price NUMERIC(8, 2),
  vendor_id INT REFERENCES vendor(id)
);
CREATE TABLE vendor (`id` INT NOT NULL, `name` TEXT, PRIMARY KEY (`id`), UNIQUE KEY uq (`name`));
"#;
    let (tables, err) = ddl::parse(sql);
    assert!(!err);
    let mut a = App::new(None);
    let ms = measure();
    let root = a.repo.root;
    let (model, n) = a.change(|a| ddl::import(a, root, &tables, true, &rev_opts(&a.prefs, Lang::Ddl)));
    assert_eq!(n, 2);
    assert_eq!(a.repo.name(model), "DDLReverse");
    let product = find(&a.repo, "ERDEntity", "product");
    let cols: Vec<Eid> = a.repo.list(product, "columns");
    assert!(a.repo.bool(cols[0], "primaryKey"));
    assert_eq!(a.repo.str(cols[2], "length"), "8,2");
    assert!(a.repo.bool(cols[3], "foreignKey"));
    let vendor = find(&a.repo, "ERDEntity", "vendor");
    assert_eq!(a.repo.refd(cols[3], "referenceTo"), Some(a.repo.list(vendor, "columns")[0]));
    assert!(a.repo.bool(a.repo.list(vendor, "columns")[1], "unique"));
    assert_eq!(a.repo.all_of("ERDRelationship").len(), 1);
    let d = ddl::er_diagram(&mut a, model, &ms).unwrap();
    assert_eq!(a.repo.ty(d), "ERDDiagram");
    assert_eq!(a.repo.list(d, "ownedViews").iter().filter(|v| a.repo.ty(**v) == "ERDEntityView").count(), 2);
}

/// The whole flow through the App: files on disk, generate, then reverse them back into a new
/// model with the diagrams.
#[test]
fn generate_then_reverse_through_the_app() {
    let dir = std::env::temp_dir().join(format!("umlcraft-codegen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut a = App::new(None);
    let (r, core) = library();
    a.repo = r;
    let written = a.generate_code(Lang::Java, core, &dir).unwrap();
    assert_eq!(written.len(), 5);
    assert!(dir.join("people/Member.java").exists());
    // a hidden folder is not read
    std::fs::create_dir_all(dir.join(".git")).unwrap();
    std::fs::write(dir.join(".git/Hidden.java"), "class Hidden {}").unwrap();
    let ms = measure();
    let undo = a.undo.len();
    let log = a.reverse_code(Lang::Java, None, &dir, &ms).unwrap();
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    assert!(log.iter().any(|l| l.contains("5 files analyzed")), "{log:?}");
    let m = a.focus.unwrap();
    assert_eq!(a.repo.name(m), "JavaReverse");
    assert!(a.repo.all_of("UMLClass").iter().all(|c| a.repo.name(*c) != "Hidden"));
    let books = a.repo.all_of("UMLClass").into_iter().filter(|c| a.repo.name(*c) == "Book").count();
    assert_eq!(books, 2, "the original and the reversed Book");
    let diagrams: Vec<String> = a.repo.all_of("Diagram").into_iter().filter(|d| a.repo.path(*d).contains(&m)).map(|d| a.repo.name(d)).collect();
    assert!(diagrams.contains(&"Type Hierarchy".to_string()), "{diagrams:?}");
    assert!(diagrams.contains(&"Package Structure".to_string()), "{diagrams:?}");
    assert!(diagrams.contains(&"Overview".to_string()), "{diagrams:?}");
    // the people package came back with Member and its association to the reversed Book
    let member = a.repo.all_of("UMLClass").into_iter().find(|c| a.repo.name(*c) == "Member" && a.repo.path(*c).contains(&m)).unwrap();
    assert_eq!(a.repo.name(a.repo.parent(member).unwrap()), "people");
    a.do_undo();
    assert!(!a.repo.all_of("UMLModel").iter().any(|x| a.repo.name(*x) == "JavaReverse"));
    let _ = std::fs::remove_dir_all(&dir);
}
