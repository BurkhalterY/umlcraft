//! Tests that drive the editing operations the mouse and menus use, and check the saved file.

use crate::app::{App, AssocInit, Tool};
use crate::canvas::EguiMeasure;
use crate::model::Repo;
use crate::views::R;
use serde_json::{json, Value};

fn measure() -> EguiMeasure {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
    out.textures_delta.clear();
    EguiMeasure { ctx }
}

fn tool(kind: &'static str, init: AssocInit) -> Tool {
    Tool { label: "", kind, icon: "", line: true, init }
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> R {
    R { x, y, w, h }
}

#[test]
fn ids_look_like_the_originals() {
    let id = crate::model::new_id();
    assert_eq!(id.len(), 20);
    assert!(id.starts_with("AAAA"));
    assert!(id.ends_with('='));
}

#[test]
fn builds_edits_and_saves_a_model() {
    let ms = measure();
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(320.0, 40.0, 160.0, 100.0))).unwrap();
    let i1 = a.change(|a| a.create_node("UMLInterface", rect(40.0, 240.0, 140.0, 80.0))).unwrap();
    let e1 = a.change(|a| a.create_node("UMLEnumeration", rect(320.0, 240.0, 140.0, 100.0))).unwrap();
    let p1 = a.change(|a| a.create_node("UMLPackage", rect(560.0, 240.0, 140.0, 100.0))).unwrap();
    let n1 = a.change(|a| a.create_node("Note", rect(560.0, 40.0, 140.0, 60.0))).unwrap();
    a.change(|a| a.repo.set(n1, "text", json!("A note")));
    let m1 = a.repo.refd(c1, "model").unwrap();
    let m2 = a.repo.refd(c2, "model").unwrap();
    assert_eq!(a.repo.name(m1), "Class1");
    assert_eq!(a.repo.name(m2), "Class2");

    let at = a.change(|a| a.add_feature(m1, "UMLAttribute")).unwrap();
    a.change(|a| crate::parse::apply_attribute(&mut a.repo, at, "-id: int[1] = 0"));
    assert_eq!(a.repo.name(at), "id");
    assert_eq!(a.repo.str(at, "type"), "int");
    assert_eq!(a.repo.str(at, "visibility"), "private");
    assert_eq!(a.repo.str(at, "multiplicity"), "1");
    assert_eq!(a.repo.str(at, "defaultValue"), "0");
    let op = a.change(|a| a.add_feature(m1, "UMLOperation")).unwrap();
    a.change(|a| crate::parse::apply_operation(&mut a.repo, op, "+placeOrder(item: Order, qty: int): boolean"));
    assert_eq!(crate::views::operation_text(&a.repo, op, true, true), "+placeOrder(item: Order, qty: int): boolean");
    let em = a.repo.refd(e1, "model").unwrap();
    a.change(|a| a.add_feature(em, "UMLEnumerationLiteral"));
    a.change(|a| a.repo.set(m1, "isAbstract", json!(true)));

    let gen = tool("UMLGeneralization", AssocInit::Plain);
    assert!(a.can_connect(&gen, c2, c1));
    a.change(|a| a.create_edge(&gen, c2, c1)).unwrap();
    a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Composite), c1, e1)).unwrap();
    a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Directed), c2, e1)).unwrap();
    a.change(|a| a.create_edge(&tool("UMLDependency", AssocInit::Plain), c1, p1)).unwrap();
    let ir = tool("UMLInterfaceRealization", AssocInit::Plain);
    assert!(a.can_connect(&ir, c1, i1));
    assert!(!a.can_connect(&ir, c1, c2));
    a.change(|a| a.create_edge(&ir, c1, i1)).unwrap();
    a.change(|a| a.create_edge(&tool("NoteLink", AssocInit::Plain), n1, c2)).unwrap();

    let model = a.repo.children(a.repo.root)[0].1;
    a.change(|a| a.add_diagram(Some(model), "UMLUseCaseDiagram"));
    let ac = a.change(|a| a.create_node("UMLActor", rect(40.0, 80.0, 40.0, 80.0))).unwrap();
    let u1 = a.change(|a| a.create_node("UMLUseCase", rect(200.0, 60.0, 120.0, 40.0))).unwrap();
    let u2 = a.change(|a| a.create_node("UMLUseCase", rect(200.0, 200.0, 120.0, 40.0))).unwrap();
    a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), ac, u1)).unwrap();
    let inc = tool("UMLInclude", AssocInit::Plain);
    assert!(a.can_connect(&inc, u1, u2));
    a.change(|a| a.create_edge(&inc, u1, u2)).unwrap();

    // undo and redo restore the same model
    let before = a.repo.serialize();
    a.do_undo();
    assert_ne!(before, a.repo.serialize());
    a.do_redo();
    assert_eq!(before, a.repo.serialize());

    let out = std::env::temp_dir().join("umlcraft-test-new.mdj");
    assert!(a.save_to(out.clone(), &ms));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("{\n\t\"_type\": \"Project\",\n\t\"_id\": "));
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["documentVersion"], json!(1));
    // the saved file opens again and saves byte-for-byte the same
    let again = Repo::parse(&text).unwrap();
    assert_eq!(again.serialize(), text);

    // deleting a class from the model removes its views and the lines attached to it
    let views_before = a.repo.all_of("View").len();
    a.change(|a| a.delete_models(&[m1]));
    assert!(!a.repo.alive(m1));
    assert!(!a.repo.alive(c1));
    assert!(a.repo.all_of("View").len() < views_before);
    assert!(a.repo.all_of("UMLInterfaceRealization").is_empty());
    assert!(a.repo.all_of("UMLGeneralization").is_empty());
}

#[test]
fn keeps_files_from_the_original_unchanged() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../checks/original");
    let ms = measure();
    for name in ["sample.mdj", "sample2.mdj", "other.mdj"] {
        let p = dir.join(name);
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        // without changes the file is written back exactly
        assert_eq!(Repo::parse(&text).unwrap().serialize(), text, "{name}");
        // after laying out every diagram the structure and key order stay the same
        let mut a = App::new(None);
        a.open_path(p.clone());
        a.prepare_for_save(&ms);
        let a1: Value = serde_json::from_str(&text).unwrap();
        let b1: Value = serde_json::from_str(&a.repo.serialize()).unwrap();
        same_shape(&a1, &b1, name);
    }
}

fn same_shape(a: &Value, b: &Value, path: &str) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let kx: Vec<&String> = x.keys().collect();
            let ky: Vec<&String> = y.keys().collect();
            assert_eq!(kx, ky, "keys at {path}");
            for k in x.keys() {
                same_shape(&x[k], &y[k], &format!("{path}/{k}"));
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            assert_eq!(x.len(), y.len(), "length at {path}");
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                same_shape(p, q, &format!("{path}[{i}]"));
            }
        }
        _ => {}
    }
}

#[test]
fn ctrl_z_undoes_while_the_canvas_has_focus() {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0)));
    assert_eq!(a.repo.all_of("UMLClass").len(), 1);
    let canvas = egui::Id::new("canvas-under-test");
    let frame = |a: &mut App, input: egui::RawInput, focus: bool| {
        let mut out = ctx.run_ui(input, |ui| {
            let r = ui.interact(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0)), canvas, egui::Sense::click());
            if focus {
                r.request_focus();
            }
            a.shortcuts(ui.ctx());
        });
        out.textures_delta.clear();
    };
    frame(&mut a, egui::RawInput::default(), true);
    frame(&mut a, egui::RawInput::default(), false);
    assert_eq!(ctx.memory(|m| m.focused()), Some(canvas));
    let press = egui::Event::Key { key: egui::Key::Z, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::COMMAND };
    frame(&mut a, egui::RawInput { events: vec![press], ..Default::default() }, false);
    assert_eq!(a.repo.all_of("UMLClass").len(), 0, "Ctrl+Z should undo the new class");
}

/// Writes the example model used for the README pictures:
/// `EXAMPLE_OUT=examples/library.mdj cargo test --release write_example -- --ignored`
#[test]
#[ignore]
fn write_example() {
    let Ok(out) = std::env::var("EXAMPLE_OUT") else { return };
    let ms = measure();
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    a.repo.set(a.repo.root, "name", json!("Library"));
    let main = a.diagram.unwrap();
    a.repo.set(main, "name", json!("Domain"));
    let class = |a: &mut App, kind: &str, name: &str, r: R| {
        let v = a.create_node(kind, r).unwrap();
        let m = a.repo.refd(v, "model").unwrap();
        a.repo.set(m, "name", json!(name));
        (v, m)
    };
    let attr = |a: &mut App, m, text: &str| {
        let x = a.add_feature(m, "UMLAttribute").unwrap();
        crate::parse::apply_attribute(&mut a.repo, x, text);
    };
    let op = |a: &mut App, m, text: &str| {
        let x = a.add_feature(m, "UMLOperation").unwrap();
        crate::parse::apply_operation(&mut a.repo, x, text);
    };
    let (item_v, item) = class(&mut a, "UMLClass", "LibraryItem", rect(24.0, 24.0, 200.0, 120.0));
    a.repo.set(item, "isAbstract", json!(true));
    attr(&mut a, item, "#title: String");
    attr(&mut a, item, "#year: int");
    op(&mut a, item, "+isAvailable(): boolean");
    let (book_v, book) = class(&mut a, "UMLClass", "Book", rect(8.0, 232.0, 150.0, 90.0));
    attr(&mut a, book, "-isbn: String");
    attr(&mut a, book, "-pages: int");
    let (dvd_v, dvd) = class(&mut a, "UMLClass", "DVD", rect(176.0, 232.0, 130.0, 90.0));
    attr(&mut a, dvd, "-minutes: int");
    let (loan_v, loan) = class(&mut a, "UMLClass", "Loan", rect(400.0, 24.0, 220.0, 120.0));
    attr(&mut a, loan, "-due: Date");
    attr(&mut a, loan, "-status: LoanStatus = ACTIVE");
    op(&mut a, loan, "+extend(days: int)");
    op(&mut a, loan, "+isOverdue(today: Date): boolean");
    let (member_v, member) = class(&mut a, "UMLClass", "Member", rect(400.0, 232.0, 220.0, 100.0));
    attr(&mut a, member, "-name: String");
    attr(&mut a, member, "-cardNumber: String");
    op(&mut a, member, "+borrow(item: LibraryItem): Loan");
    let (lend_v, _) = class(&mut a, "UMLInterface", "Lendable", rect(181.0, 400.0, 120.0, 80.0));
    let (status_v, status) = class(&mut a, "UMLEnumeration", "LoanStatus", rect(400.0, 400.0, 150.0, 110.0));
    for l in ["ACTIVE", "RETURNED", "OVERDUE"] {
        let x = a.add_feature(status, "UMLEnumerationLiteral").unwrap();
        a.repo.set(x, "name", json!(l));
    }
    let gen = tool("UMLGeneralization", AssocInit::Plain);
    a.create_edge(&gen, book_v, item_v);
    a.create_edge(&gen, dvd_v, item_v);
    let e = a.create_edge(&tool("UMLAssociation", AssocInit::Directed), loan_v, item_v).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    let end2 = a.repo.child(m, "end2").unwrap();
    a.repo.set(end2, "multiplicity", json!("1"));
    let e = a.create_edge(&tool("UMLAssociation", AssocInit::Composite), loan_v, member_v).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    a.repo.set(m, "name", json!("borrows"));
    let end1 = a.repo.child(m, "end1").unwrap();
    a.repo.set(end1, "multiplicity", json!("0..*"));
    a.create_edge(&tool("UMLInterfaceRealization", AssocInit::Plain), dvd_v, lend_v);
    let _ = status_v;
    let note = a.create_node("Note", rect(240.0, 160.0, 140.0, 50.0)).unwrap();
    a.repo.set(note, "text", json!("Overdue after the due date."));
    a.create_edge(&tool("NoteLink", AssocInit::Plain), note, loan_v);

    let uc = a.add_diagram(Some(model), "UMLUseCaseDiagram");
    a.repo.set(uc, "name", json!("Use Cases"));
    let subject = a.create_node("UMLUseCaseSubject", rect(200.0, 24.0, 300.0, 380.0)).unwrap();
    let sm = a.repo.refd(subject, "model").unwrap();
    a.repo.set(sm, "name", json!("Library System"));
    let (reader, _) = class(&mut a, "UMLActor", "Member", rect(56.0, 150.0, 50.0, 80.0));
    let (lib, _) = class(&mut a, "UMLActor", "Librarian", rect(580.0, 280.0, 60.0, 80.0));
    let (borrow, _) = class(&mut a, "UMLUseCase", "Borrow an item", rect(270.0, 70.0, 160.0, 50.0));
    let (ret, _) = class(&mut a, "UMLUseCase", "Return an item", rect(270.0, 300.0, 160.0, 50.0));
    let (card, _) = class(&mut a, "UMLUseCase", "Check the card", rect(270.0, 180.0, 160.0, 50.0));
    let assoc = tool("UMLAssociation", AssocInit::Plain);
    a.create_edge(&assoc, reader, borrow);
    a.create_edge(&assoc, reader, ret);
    a.create_edge(&assoc, lib, ret);
    a.create_edge(&tool("UMLInclude", AssocInit::Plain), borrow, card);
    a.open_diagram(main);
    assert!(a.save_to(std::path::PathBuf::from(out), &ms));
}

#[test]
fn png_export_follows_the_originals_size_rules() {
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let d = a.diagram.unwrap();
    let info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b = crate::render::bounds(&a.repo, d).unwrap();
    let png = crate::render::to_png(&a.repo, d, &info, &ms, 1.0).unwrap();
    let img = image::load_from_memory(&png).unwrap().to_rgba8();
    // bounds grown by 10 on every side, plus 30 (diagram-export.js:113-117)
    assert_eq!(img.width(), (b.w + 2.0 * 10.0 + 30.0).ceil() as u32);
    assert_eq!(img.height(), (b.h + 2.0 * 10.0 + 30.0).ceil() as u32);
    // transparent background, no grid
    assert_eq!(img.get_pixel(0, 0).0[3], 0);
    assert_eq!(img.get_pixel(img.width() - 1, img.height() - 1).0[3], 0);
    let dark = img.pixels().filter(|p| p.0[3] > 200 && p.0[0] < 60 && p.0[1] < 60 && p.0[2] < 60).count();
    assert!(dark > 1000, "the drawing is in the picture ({dark} dark pixels)");
    if let Ok(out) = std::env::var("PNG_OUT") {
        std::fs::write(out, &png).unwrap();
    }
    // SVG export: bounds grown by 10, no background (diagram-export.js:178-182)
    let svg = crate::render::to_svg(&a.repo, d, &info, &ms).unwrap();
    assert!(svg.contains(&format!("width=\"{:.0}\"", b.w + 20.0)));
    assert!(!svg.contains("fill=\"#ffffff\"/>\n<"), "no background rectangle");
}
