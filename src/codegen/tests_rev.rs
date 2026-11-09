//! Reverse engineering across files: imports and qualified names, generics as template
//! parameters, overloads, nested types, annotations and the per-language idioms.

use super::tests::*;
use super::*;

/// Reverse several files (text, folder) of a language into a new model.
fn reverse_files(l: Lang, files: &[(&str, &[&str])], o: &RevOpts) -> (Repo, Eid) {
    let mut r = Repo::blank();
    let root = r.root;
    let m = r.create("UMLModel", &[("name", json!(l.reverse_model()))], Some((root, "ownedElements")));
    let mut units = vec![];
    for (src, rel) in files {
        let rel: Vec<String> = rel.iter().map(|s| s.to_string()).collect();
        let p = parse_source(l, src, &rel, o);
        assert!(!p.errors, "{l:?} reported errors in {src}");
        units.extend(p.units);
    }
    import_units(&mut r, m, units, o);
    (r, m)
}

fn opts() -> RevOpts {
    RevOpts { skip_self: true, skip_magic: true, association: true, public_only: false, type_hierarchy: false, package_overview: false, package_structure: false }
}

/// The class `name` whose package path (from the model) is `path`.
fn at(r: &Repo, m: Eid, path: &[&str]) -> Eid {
    let mut cur = m;
    for s in path {
        cur = r.list(cur, "ownedElements").into_iter().find(|c| r.name(*c) == *s).unwrap_or_else(|| panic!("no {s} in {path:?}"));
    }
    cur
}

/// The association end2 targets of a class, by end name.
fn assoc(r: &Repo, c: Eid, end: &str) -> Option<Eid> {
    r.list(c, "ownedElements").into_iter().filter(|a| r.ty(*a) == "UMLAssociation").filter_map(|a| r.child(a, "end2")).find(|e| r.name(*e) == end).and_then(|e| r.refd(e, "reference"))
}

fn tparams(r: &Repo, e: Eid) -> Vec<String> {
    r.list(e, "templateParameters").into_iter().map(|p| r.name(p)).collect()
}

#[test]
fn java_imports_generics_overloads_and_annotations() {
    let a = "package a.model; public class Book {}";
    let b = "package b.model; public class Book {} class Date {}";
    let app = r#"
        package app;
        import a.model.Book;
        import java.util.Date;
        import java.util.*;
        import static java.lang.Math.max;
        /** An entity. */
        @Entity(name = "loan")
        public class Loan {
            @Id @GeneratedValue(strategy = GenerationType.AUTO)
            private Long id;
            private Book book;
            private b.model.Book other;
            private Date when;
            private java.util.List<Book> history;
            private Inner inner;
            /** Make one. */
            public Loan() {}
            public @Nullable String name() { return null; }
            @Override
            public String toString() { return ""; }
            public void add(int n) {}
            public void add(String s) {}
            public <U extends Comparable<U>> U map(U u) throws java.io.IOException, LoanError { return u; }
            class Inner {}
        }
        class LoanError extends Exception {}
        class Box<T extends Comparable<T>, K> { T item; K key; }
    "#;
    let star = "package c; import b.model.*; public class Shelf { private Book book; }";
    let (r, m) = reverse_files(Lang::Java, &[(a, &[]), (b, &[]), (app, &[]), (star, &[])], &opts());
    let loan = at(&r, m, &["app", "Loan"]);
    let a_book = at(&r, m, &["a", "model", "Book"]);
    let b_book = at(&r, m, &["b", "model", "Book"]);
    assert_eq!(r.str(loan, "documentation"), "An entity.");
    // the explicit import wins; a qualified name is followed; a wildcard import opens a package
    assert_eq!(assoc(&r, loan, "book"), Some(a_book));
    assert_eq!(assoc(&r, loan, "other"), Some(b_book));
    assert_eq!(assoc(&r, loan, "history"), Some(a_book), "java.util.List<Book>");
    assert_eq!(assoc(&r, at(&r, m, &["c", "Shelf"]), "book"), Some(b_book));
    // a name imported from outside the folder stays a name, even when a reversed type has it
    assert_eq!(r.str(attr(&r, loan, "when"), "type"), "Date");
    // annotations are skipped wherever they are
    assert_eq!(r.str(attr(&r, loan, "id"), "type"), "Long");
    assert_eq!(r.str(attr(&r, loan, "id"), "visibility"), "private");
    let ops: Vec<String> = r.list(loan, "operations").into_iter().map(|o| r.name(o)).collect();
    assert_eq!(ops, ["Loan", "name", "toString", "add", "add", "map"], "overloads of the same arity are kept");
    let ctor = op(&r, loan, "Loan");
    assert_eq!(r.str(ctor, "stereotype"), "constructor");
    assert_eq!(r.str(ctor, "documentation"), "Make one.");
    let adds: Vec<Vec<String>> = r.list(loan, "operations").into_iter().filter(|o| r.name(*o) == "add").map(|o| sig(&r, o)).collect();
    assert_eq!(adds, [vec!["n: int"], vec!["s: String"]]);
    // a nested type is found first
    assert_eq!(assoc(&r, loan, "inner"), Some(at(&r, m, &["app", "Loan", "Inner"])));
    // generics: template parameters with their bounds; a type parameter stays a name
    let map = op(&r, loan, "map");
    assert_eq!(tparams(&r, map), ["U"]);
    assert_eq!(sig(&r, map), ["u: U", ": U"]);
    let err = at(&r, m, &["app", "LoanError"]);
    assert_eq!(r.ref_list(map, "raisedExceptions"), vec![err], "IOException is not in the folder");
    let bx = at(&r, m, &["app", "Box"]);
    assert_eq!(tparams(&r, bx), ["T", "K"]);
    let t = r.list(bx, "templateParameters")[0];
    assert_eq!(r.str(t, "parameterType"), "Comparable<T>");
    assert_eq!(r.str(attr(&r, bx, "item"), "type"), "T");
    // final, synchronized and the modifiers kept as tags (the 7.x Java reverse)
    let src = r#"
        public final class Account {
            private final int id = 1;
            private volatile transient int hits;
            private java.util.Set<String> names;
            public final synchronized native void touch();
        }
        @interface Marker { String value() default "x"; }
        enum Level { /** The low one. */ LOW, HIGH }
    "#;
    let (r, m) = reverse_files(Lang::Java, &[(src, &[])], &opts());
    let acc = at(&r, m, &["Account"]);
    assert!(r.bool(acc, "isFinalSpecialization") && r.bool(acc, "isLeaf"));
    let id = attr(&r, acc, "id");
    assert!(r.bool(id, "isLeaf") && r.bool(id, "isReadOnly"));
    let tags = |e: Eid| r.list(e, "tags").into_iter().map(|t| format!("{}={}", r.name(t), if r.str(t, "kind") == "boolean" { r.bool(t, "checked").to_string() } else { r.str(t, "value") })).collect::<Vec<_>>();
    assert_eq!(tags(attr(&r, acc, "hits")), ["volatile=true", "transient=true"]);
    let names = attr(&r, acc, "names");
    assert_eq!(tags(names), ["collection=Set"]);
    assert_eq!(r.str(names, "multiplicity"), "*");
    let touch = op(&r, acc, "touch");
    assert!(r.bool(touch, "isLeaf"));
    assert_eq!(r.str(touch, "concurrency"), "concurrent");
    assert_eq!(tags(touch), ["native=true"]);
    let marker = at(&r, m, &["Marker"]);
    assert_eq!((r.ty(marker), r.str(marker, "stereotype").as_str()), ("UMLClass", "annotationType"));
    assert_eq!(tags(op(&r, marker, "value")), ["default=\"x\""]);
    let low = r.list(at(&r, m, &["Level"]), "literals")[0];
    assert_eq!(r.str(low, "documentation"), "The low one.");
}

#[test]
fn cpp_namespaces_using_and_templates() {
    let geo = r#"
        namespace geo {
            class Point { public: double x; };
            template <typename T, int N = 3>
            class Grid {
            public:
                T cells[N];
                template <typename U> U at(U u) const;
                void move(int dx);
                void move(double d);
                Grid();
            };
            template <> class Grid<bool> { public: int bits; };
        }
    "#;
    let app = r#"
        #include "geo.h"
        namespace app {
            using namespace geo;
            class Map {
            public:
                Point* origin;
                geo::Point corner;
                std::shared_ptr<Point> shared;
                std::vector<Point> path;
            };
        }
    "#;
    let (r, m) = reverse_files(Lang::Cpp, &[(geo, &[]), (app, &[])], &opts());
    let point = at(&r, m, &["geo", "Point"]);
    let map = at(&r, m, &["app", "Map"]);
    assert_eq!(assoc(&r, map, "origin"), Some(point), "using namespace geo; a pointer is a reference");
    assert_eq!(assoc(&r, map, "corner"), Some(point), "a qualified name");
    assert_eq!(assoc(&r, map, "shared"), Some(point), "a smart pointer");
    assert_eq!(assoc(&r, map, "path"), Some(point));
    let grid = at(&r, m, &["geo", "Grid"]);
    assert_eq!(tparams(&r, grid), ["T", "N"]);
    let n = r.list(grid, "templateParameters")[1];
    assert_eq!(r.str(n, "defaultValue"), "3");
    assert_eq!(r.str(attr(&r, grid, "cells"), "type"), "T", "a template parameter stays a name");
    assert_eq!(tparams(&r, op(&r, grid, "at")), ["U"]);
    assert_eq!(r.list(grid, "operations").into_iter().filter(|o| r.name(*o) == "move").count(), 2);
    assert_eq!(r.str(op(&r, grid, "Grid"), "stereotype"), "constructor");
    // the specialization merges into the template
    assert_eq!(r.list(at(&r, m, &["geo"]), "ownedElements").into_iter().filter(|c| r.name(*c) == "Grid").count(), 1);
    attr(&r, grid, "bits");
}

#[test]
fn csharp_usings_and_constraints() {
    let lib = "namespace Lib.Models { public class Book {} public class Entity {} }";
    let app = r#"
        using System;
        using Lib.Models;
        using M = Lib.Models;
        namespace App {
            public class Repo<T> where T : Entity, new() {
                private Book book;
                private M.Book other;
                public Repo() {}
                public T2 Get<T2>(int id) { return default; }
                public void Put(int id) {}
                public void Put(string key) {}
            }
        }
    "#;
    let (r, m) = reverse_files(Lang::CSharp, &[(lib, &[]), (app, &[])], &opts());
    let book = at(&r, m, &["Lib", "Models", "Book"]);
    let repo = at(&r, m, &["App", "Repo"]);
    assert_eq!(assoc(&r, repo, "book"), Some(book));
    assert_eq!(assoc(&r, repo, "other"), Some(book), "through the alias");
    let t = r.list(repo, "templateParameters")[0];
    assert_eq!(r.str(t, "parameterType"), "Entity, new()");
    assert_eq!(tparams(&r, op(&r, repo, "Get")), ["T2"]);
    assert_eq!(r.str(op(&r, repo, "Repo"), "stereotype"), "constructor");
    assert_eq!(r.list(repo, "operations").into_iter().filter(|o| r.name(*o) == "Put").count(), 2);
}

#[test]
fn python_imports_hints_and_decorators() {
    let shapes = r#"
class Shape[T: Item = int]:
    def area[U](self, unit: U) -> T: ...

class Item:
    pass
"#;
    let app = r#"
from dataclasses import dataclass
from typing import ClassVar, Final, Optional
from geo.shapes import Shape as S
from .local import Thing

@dataclass(frozen=True)
class Drawing:
    shape: S
    maybe: Optional[S]
    other: "S | None" = None
    count: ClassVar[int] = 0
    LIMIT: Final[int] = 3

    @property
    def size(self) -> int:
        return 1

    def __repr__(self) -> str:
        return ""

    @staticmethod
    def make(n: int) -> "Drawing":
        return Drawing()
"#;
    let local = "class Thing:\n    pass\n";
    let (r, m) = reverse_files(Lang::Python, &[(shapes, &["geo"]), (app, &["app"]), (local, &["app"])], &opts());
    let shape = at(&r, m, &["geo", "Shape"]);
    let d = at(&r, m, &["app", "Drawing"]);
    assert_eq!(assoc(&r, d, "shape"), Some(shape), "through the import's alias");
    assert_eq!(assoc(&r, d, "maybe"), Some(shape));
    assert!(r.bool(attr(&r, d, "count"), "isStatic"), "ClassVar");
    assert!(r.bool(attr(&r, d, "LIMIT"), "isReadOnly"), "Final");
    assert!(r.bool(attr(&r, d, "size"), "isReadOnly"), "a property");
    assert!(r.list(d, "operations").iter().all(|o| r.name(*o) != "__repr__"), "magic methods are skipped");
    assert!(r.bool(op(&r, d, "make"), "isStatic"));
    // PEP 695 generics
    assert_eq!(tparams(&r, shape), ["T"]);
    let t = r.list(shape, "templateParameters")[0];
    assert_eq!(r.refd(t, "parameterType"), Some(at(&r, m, &["geo", "Item"])));
    assert_eq!(r.str(t, "defaultValue"), "int");
    assert_eq!(tparams(&r, op(&r, shape, "area")), ["U"]);
    assert_eq!(sig(&r, op(&r, shape, "area")), ["unit: U", ": T"]);
    // the options keep self and the magic methods
    let o = RevOpts { skip_self: false, skip_magic: false, ..opts() };
    let (r, m) = reverse_files(Lang::Python, &[(app, &["app"])], &o);
    let d = at(&r, m, &["app", "Drawing"]);
    assert_eq!(sig(&r, op(&r, d, "__repr__")), ["self: ", ": str"]);
}

#[test]
fn typescript_imports_generics_and_enums() {
    let model = "export class Book {}\nexport interface Entity { id: number; }\n";
    let app = r#"
import { Book as B, Entity } from '../model/book';
import * as models from '../model/book';
import type { Thing } from './thing';

export enum Color { Red = 'r', Green }

export interface Repo<T extends Entity = B> {
  find<K>(id: K): T | undefined;
  all(): T[];
}

export class Shelf {
  first: B;
  second?: models.Book;
  color: Color = Color.Red;
}
"#;
    let (r, m) = reverse_files(Lang::TypeScript, &[(model, &["model"]), (app, &["app"])], &opts());
    let book = at(&r, m, &["model", "Book"]);
    let shelf = at(&r, m, &["app", "Shelf"]);
    assert_eq!(assoc(&r, shelf, "first"), Some(book));
    assert_eq!(assoc(&r, shelf, "second"), Some(book), "through the namespace import");
    let color = at(&r, m, &["app", "Color"]);
    assert_eq!(r.list(color, "literals").into_iter().map(|l| r.name(l)).collect::<Vec<_>>(), ["Red", "Green"]);
    let repo = at(&r, m, &["app", "Repo"]);
    assert_eq!(r.ty(repo), "UMLInterface");
    let t = r.list(repo, "templateParameters")[0];
    assert_eq!(r.refd(t, "parameterType"), Some(at(&r, m, &["model", "Entity"])));
    assert_eq!(r.str(t, "defaultValue"), "B");
    let find = op(&r, repo, "find");
    assert_eq!(tparams(&r, find), ["K"]);
    assert_eq!(sig(&r, find), ["id: K", ": T"]);
    let ret = r.list(find, "parameters")[1];
    assert_eq!(r.str(ret, "multiplicity"), "0..1");
}

#[test]
fn kotlin_php_and_rust_imports() {
    let lib = "package lib.model\nclass Book\n";
    let kt = "package app\nimport lib.model.Book as Volume\nclass Shelf {\n    var first: Volume? = null\n}\nclass Box<out T : Volume>\n";
    let (r, m) = reverse_files(Lang::Kotlin, &[(lib, &[]), (kt, &[])], &opts());
    assert_eq!(assoc(&r, at(&r, m, &["app", "Shelf"]), "first"), Some(at(&r, m, &["lib", "model", "Book"])));
    let bx = at(&r, m, &["app", "Box"]);
    assert_eq!(tparams(&r, bx), ["T"]);
    assert_eq!(r.refd(r.list(bx, "templateParameters")[0], "parameterType"), Some(at(&r, m, &["lib", "model", "Book"])));
    let lib = "<?php\nnamespace Lib\\Model;\nclass Book {}\n";
    let php = "<?php\nnamespace App;\nuse Lib\\Model\\Book as Volume;\nclass Shelf {\n    private Volume $first;\n}\n";
    let (r, m) = reverse_files(Lang::Php, &[(lib, &[]), (php, &[])], &opts());
    assert_eq!(assoc(&r, at(&r, m, &["App", "Shelf"]), "first"), Some(at(&r, m, &["Lib", "Model", "Book"])));
    let lib = "pub struct Book {}\n";
    let rs = "use crate::model::book::Book as Volume;\npub struct Shelf {\n    pub first: Volume,\n}\n";
    let (r, m) = reverse_files(Lang::Rust, &[(lib, &["model"]), (rs, &["app"])], &opts());
    assert_eq!(assoc(&r, at(&r, m, &["app", "Shelf"]), "first"), Some(at(&r, m, &["model", "Book"])));
}
