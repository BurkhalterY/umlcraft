//! Code generation and reverse engineering for Rust, Go, Ruby, Kotlin, Dart and Swift.

use super::tests::*;
use super::*;

#[test]
fn generates_rust_go_and_ruby() {
    let f = files(Lang::Rust);
    has(file(&f, "book.rs"), &["use crate::library_item::LibraryItem;", "pub struct Book {", "pub base: LibraryItem,", "isbn: String,", "impl Book {", "pub fn pages(&self, from: i32) -> i32 {", "todo!()", "impl Lendable for Book {", "fn lend(&self, days: i32) -> bool {"]);
    has(file(&f, "lendable.rs"), &["pub trait Lendable {", "fn lend(&self, days: i32) -> bool;"]);
    has(file(&f, "mod.rs"), &["pub mod people;", "pub mod book;"]);
    has(file(&f, "people/member.rs"), &["pub books: Vec<Book>,"]);
    has(file(&f, "library_item.rs"), &["pub const COUNT: i32 = 0;"]);
    let f = files(Lang::Go);
    has(file(&f, "book.go"), &["package core", "type Book struct {", "LibraryItem", "isbn string", "func (b *Book) Pages(from int) int {", "return 0"]);
    has(file(&f, "status.go"), &["type Status int", "StatusAVAILABLE Status = iota", "StatusLOANED"]);
    has(file(&f, "people/member.go"), &["package people", "Books []Book"]);
    has(file(&f, "lendable.go"), &["type Lendable interface {", "Lend(days int) bool"]);
    let f = files(Lang::Ruby);
    has(file(&f, "book.rb"), &["require_relative 'library_item'", "class Book < LibraryItem", "include Lendable", "@isbn = nil", "def pages(from)"]);
    has(file(&f, "people/member.rb"), &["module People", "attr_accessor :books # Array<Book>"]);
    has(file(&f, "lendable.rb"), &["module Lendable", "raise NotImplementedError"]);
    has(file(&f, "status.rb"), &["module Status", "AVAILABLE = 1"]);
}

#[test]
fn generates_kotlin_dart_and_swift() {
    let f = files(Lang::Kotlin);
    has(file(&f, "Book.kt"), &["class Book : LibraryItem(), Lendable {", "private var isbn: String = \"\"", "fun pages(from: Int): Int {", "TODO(\"Not yet implemented\")"]);
    has(file(&f, "LibraryItem.kt"), &["abstract class LibraryItem {", "companion object {", "var count: Int = 0"]);
    has(file(&f, "people/Member.kt"), &["package people", "var books: MutableList<Book> = mutableListOf()"]);
    has(file(&f, "Status.kt"), &["enum class Status {"]);
    let f = files(Lang::Dart);
    has(file(&f, "book.dart"), &["import 'library_item.dart';", "class Book extends LibraryItem implements Lendable {", "late String _isbn;", "int pages(int from) {", "throw UnimplementedError();"]);
    has(file(&f, "lendable.dart"), &["abstract interface class Lendable {", "bool lend(int days);"]);
    has(file(&f, "people/member.dart"), &["import '../book.dart';", "List<Book> books = [];"]);
    let f = files(Lang::Swift);
    has(file(&f, "Book.swift"), &["import Foundation", "public class Book: LibraryItem, Lendable {", "private var isbn: String = \"\"", "override init() {", "public func pages(from: Int) -> Int {", "fatalError(\"Not implemented\")"]);
    has(file(&f, "Lendable.swift"), &["public protocol Lendable {", "func lend(days: Int) -> Bool"]);
    has(file(&f, "Status.swift"), &["public enum Status {", "case AVAILABLE"]);
    has(file(&f, "people/Member.swift"), &["public var books: [Book] = []"]);
}

fn literals(r: &crate::model::Repo, e: Eid) -> Vec<String> {
    r.list(e, "literals").into_iter().map(|l| r.name(l)).collect()
}

#[test]
fn reverses_rust() {
    let rs = r#"
/// A shape.
pub trait Shape {
    fn area(&self) -> f64;
}
#[derive(Debug)]
pub enum Color { RED, GREEN(u8), }
pub struct Circle {
    pub radius: f64,
    tags: Vec<String>,
    pub(crate) label: Option<String>,
}
impl Circle {
    pub const SIDES: u32 = 0;
    pub fn new(radius: f64) -> Self { Circle { radius, tags: vec![], label: None } }
    pub fn scale(&mut self, factor: f64) {}
}
impl Shape for Circle {
    fn area(&self) -> f64 { 3.14 * self.radius * self.radius }
}
mod inner { pub struct Hidden<'a> { pub s: &'a str } }
"#;
    let (r, _) = reverse(Lang::Rust, rs, &["geo"]);
    let circle = find(&r, "UMLClass", "Circle");
    assert!(generalizes(&r, circle, find(&r, "UMLInterface", "Shape"), "UMLInterfaceRealization"));
    assert_eq!(crate::views::type_name(&r, attr(&r, circle, "radius"), "type"), "f64");
    assert_eq!(r.str(attr(&r, circle, "tags"), "multiplicity"), "*");
    assert_eq!(r.str(attr(&r, circle, "label"), "multiplicity"), "0..1");
    assert_eq!(r.str(attr(&r, circle, "label"), "visibility"), "package");
    assert!(r.bool(op(&r, circle, "new"), "isStatic"));
    assert!(!r.bool(op(&r, circle, "scale"), "isStatic"));
    assert!(r.bool(attr(&r, circle, "SIDES"), "isStatic"));
    assert_eq!(literals(&r, find(&r, "UMLEnumeration", "Color")), ["RED", "GREEN"]);
    let hidden = find(&r, "UMLClass", "Hidden");
    assert_eq!(r.name(r.parent(hidden).unwrap()), "inner");
}

#[test]
fn reverses_go() {
    let go = r#"
package geo

// Shape is a shape.
type Shape interface {
	Area() float64
}

type Color int

const (
	ColorRED Color = iota
	ColorGREEN
)

type Base struct{ id int }

type Circle struct {
	Base
	Radius   float64
	tags, ks []string `json:"tags"`
}

func NewCircle(radius float64) *Circle { return &Circle{Radius: radius} }

func (c *Circle) Area() float64 { return 3.14 * c.Radius * c.Radius }

func (c Circle) scale(a, b float64) (float64, error) { return 0, nil }
"#;
    let (r, _) = reverse(Lang::Go, go, &[]);
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.name(r.parent(circle).unwrap()), "geo");
    assert!(generalizes(&r, circle, find(&r, "UMLClass", "Base"), "UMLGeneralization"));
    assert_eq!(r.str(attr(&r, circle, "Radius"), "visibility"), "public");
    assert_eq!(r.str(attr(&r, circle, "ks"), "multiplicity"), "*");
    assert_eq!(sig(&r, op(&r, circle, "scale")), ["a: float64", "b: float64", ": float64"]);
    assert!(r.bool(op(&r, circle, "NewCircle"), "isStatic"));
    assert_eq!(literals(&r, find(&r, "UMLEnumeration", "Color")), ["RED", "GREEN"]);
    op(&r, find(&r, "UMLInterface", "Shape"), "Area");
}

#[test]
fn reverses_ruby() {
    let rb = r#"
module Geo
  # A shape.
  module Shape
    def area
      raise NotImplementedError
    end
  end

  module Color
    RED = 1
    GREEN = 2
  end

  class Circle < Base
    include Shape
    SIDES = 0
    attr_accessor :radius
    attr_reader :name

    def initialize(radius = 1.0)
      @radius = radius
      if radius > 0
        @tags = []
      end
    end

    def area
      3.14 * @radius ** 2
    end

    def self.unit(scale = 1); end

    private

    def secret(a, b)
      [1, 2].each do |x|
        puts x
      end
    end
  end

  class Base; end
end
"#;
    let (r, _) = reverse(Lang::Ruby, rb, &[]);
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.name(r.parent(circle).unwrap()), "Geo");
    assert!(generalizes(&r, circle, find(&r, "UMLInterface", "Shape"), "UMLInterfaceRealization"));
    assert!(generalizes(&r, circle, find(&r, "UMLClass", "Base"), "UMLGeneralization"));
    assert!(r.bool(attr(&r, circle, "name"), "isReadOnly"));
    attr(&r, circle, "tags");
    assert!(r.bool(op(&r, circle, "unit"), "isStatic"));
    assert_eq!(r.str(op(&r, circle, "secret"), "visibility"), "private");
    assert_eq!(literals(&r, find(&r, "UMLEnumeration", "Color")), ["RED", "GREEN"]);
    assert_eq!(r.str(find(&r, "UMLInterface", "Shape"), "documentation"), "A shape.");
}

#[test]
fn reverses_kotlin() {
    let kt = r#"
package com.acme.geo

/** A shape. */
interface Shape {
    fun area(): Double
}

enum class Color { RED, GREEN; fun lower() = name.lowercase() }

data class Circle(val radius: Double, private var tags: List<String> = listOf()) : Base(), Shape {
    var label: String? = null
    override fun area(): Double = 3.14 * radius * radius
    fun scale(factor: Double, vararg extra: Int) { }
    companion object {
        const val SIDES = 0
        fun unit(): Circle = Circle(1.0)
    }
}

open class Base
"#;
    let (r, _) = reverse(Lang::Kotlin, kt, &[]);
    check_shapes(&r, &["com", "acme", "geo"], "Double", "UMLInterface");
    let circle = find(&r, "UMLClass", "Circle");
    assert!(generalizes(&r, circle, find(&r, "UMLClass", "Base"), "UMLGeneralization"));
    assert!(r.bool(attr(&r, circle, "radius"), "isReadOnly"));
    assert_eq!(r.str(attr(&r, circle, "tags"), "visibility"), "private");
    assert_eq!(r.str(attr(&r, circle, "label"), "multiplicity"), "0..1");
    assert!(r.bool(op(&r, circle, "unit"), "isStatic"));
    assert!(r.bool(attr(&r, circle, "SIDES"), "isStatic"));
}

#[test]
fn reverses_dart() {
    let dart = r#"
/// A shape.
abstract class Shape {
  double area();
}
enum Color { RED, GREEN }
class Circle extends Shape {
  double radius = 1.0;
  final List<String> _tags = [];
  static int count = 0;
  Circle(this.radius);
  Circle.unit() : radius = 1;
  @override
  double area() => 3.14 * radius * radius;
  String get label => 'c';
  void scale(double factor, {required int times, bool round = false}) {}
}
"#;
    let (r, _) = reverse(Lang::Dart, dart, &["geo"]);
    let shape = find(&r, "UMLInterface", "Shape");
    let circle = find(&r, "UMLClass", "Circle");
    assert!(generalizes(&r, circle, shape, "UMLInterfaceRealization"));
    assert_eq!(r.str(attr(&r, circle, "tags"), "visibility"), "private");
    assert!(r.bool(attr(&r, circle, "label"), "isReadOnly"));
    assert_eq!(sig(&r, op(&r, circle, "scale")), ["factor: double", "times: int", "round: bool"]);
    assert!(r.list(circle, "operations").iter().all(|o| r.name(*o) != "unit" && r.name(*o) != "Circle"));
    assert_eq!(literals(&r, find(&r, "UMLEnumeration", "Color")), ["RED", "GREEN"]);
}

#[test]
fn reverses_swift() {
    let swift = r#"
import Foundation

/// A shape.
protocol Shape {
    func area() -> Double
}

enum Color: String {
    case RED, GREEN
}

public class Circle {
    var radius: Double = 1.0
    private var tags: [String] = []
    static let sides = 0
    var label: String { return "c" }
    init(radius: Double) { self.radius = radius }
    public func scale(by factor: Double, _ times: Int) {}
    class func unit() -> Circle { return Circle(radius: 1) }
}

extension Circle: Shape {
    func area() -> Double { return 3.14 * radius * radius }
}
"#;
    let (r, _) = reverse(Lang::Swift, swift, &["geo"]);
    check_shapes(&r, &["geo"], "Double", "UMLInterface");
    let circle = find(&r, "UMLClass", "Circle");
    assert_eq!(r.str(attr(&r, circle, "tags"), "multiplicity"), "*");
    assert!(r.bool(attr(&r, circle, "label"), "isReadOnly"));
    assert!(r.bool(attr(&r, circle, "sides"), "isStatic"));
    assert_eq!(sig(&r, op(&r, circle, "scale")), ["factor: Double", "times: Int"]);
    assert!(r.bool(op(&r, circle, "unit"), "isStatic"));
}
