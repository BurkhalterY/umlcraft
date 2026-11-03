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

#[test]
fn alignment_commands_follow_the_originals_rules() {
    use crate::align::Align;
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(200.0, 100.0, 140.0, 80.0))).unwrap();
    let c3 = a.change(|a| a.create_node("UMLClass", rect(400.0, 20.0, 120.0, 40.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let r = |a: &App, v| crate::views::node_rect(&a.repo, v);
    a.select(vec![c1, c2, c3, e]);
    let undo = a.undo.len();
    a.align(Align::Left);
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    assert!([c1, c2, c3].iter().all(|v| r(&a, *v).x == 40.0));
    a.align(Align::Right);
    assert!([c1, c2, c3].iter().all(|v| r(&a, *v).x + r(&a, *v).w == 180.0), "right edges meet the rightmost");
    a.align(Align::Top);
    assert!([c1, c2, c3].iter().all(|v| r(&a, *v).y == 20.0));
    a.align(Align::Middle);
    // extent 20..100, middle 60
    assert_eq!(r(&a, c1).y, 30.0);
    assert_eq!(r(&a, c3).y, 40.0);
    a.align(Align::Size);
    assert!([c1, c2, c3].iter().all(|v| r(&a, *v).w == 140.0 && r(&a, *v).h == 80.0));
    a.do_undo();
    a.do_undo();
    a.do_undo();
    a.do_undo();
    a.do_undo();
    assert_eq!(r(&a, c2).x, 200.0, "undo restores the layout");

    // space equally: extent 40..520 (480), boxes 100+140+120 = 360, gap 60
    a.align(Align::SpaceH);
    assert_eq!(r(&a, c1).x, 40.0);
    assert_eq!(r(&a, c2).x, 200.0);
    assert_eq!(r(&a, c3).x, 400.0);
    a.change(|a| a.repo.set(c2, "left", json!(150)));
    a.align(Align::SpaceH);
    assert_eq!(r(&a, c2).x, 200.0);
    // overlapping boxes are left alone
    let boxes = [rect(0.0, 0.0, 100.0, 10.0), rect(50.0, 0.0, 100.0, 10.0)];
    assert_eq!(crate::align::arrange(Align::SpaceH, &boxes), boxes.to_vec());

    // z-order: only the order of the selected top-level views changes
    let d = a.diagram.unwrap();
    a.select(vec![c1]);
    a.reorder(true);
    assert_eq!(*a.repo.list(d, "ownedViews").last().unwrap(), c1);
    a.select(vec![c3, e]);
    a.reorder(false);
    assert_eq!(a.repo.list(d, "ownedViews"), vec![c3, e, c2, c1]);
}

#[test]
fn zoom_commands_keep_the_screen_centre() {
    let mut a = App::new(None);
    a.canvas_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)));
    a.change(|a| a.create_node("UMLClass", rect(1000.0, 1000.0, 200.0, 100.0)));
    a.change(|a| a.create_node("UMLClass", rect(1400.0, 1300.0, 200.0, 100.0)));
    a.scroll = crate::views::P { x: 100.0, y: 100.0 };
    let centre = |a: &App| (a.scroll.x + 400.0 / a.zoom, a.scroll.y + 300.0 / a.zoom);
    a.zoom_keep_centre(a.zoom + crate::app::ZOOM_STEP);
    assert!((a.zoom - 1.1).abs() < 1e-9);
    let (x, y) = centre(&a);
    assert!((x - 500.0).abs() < 1e-6 && (y - 400.0).abs() < 1e-6, "centre kept: {x},{y}");
    a.zoom_keep_centre(1.0);
    assert_eq!((a.scroll.x.round(), a.scroll.y.round()), (100.0, 100.0));
    // zooming out near the origin never scrolls into negative space
    a.scroll = crate::views::P::default();
    a.zoom_keep_centre(0.5);
    assert_eq!((a.scroll.x, a.scroll.y), (0.0, 0.0));

    // fit: bounding box 1000..1600 x 1000..1400 fits at 1.0 and is centred
    a.fit();
    assert_eq!(a.zoom, 1.0);
    let (x, y) = centre(&a);
    let b = crate::render::bounds(&a.repo, a.diagram.unwrap()).unwrap();
    assert!((x - (b.x + b.w / 2.0)).abs() < 1e-6 && (y - (b.y + b.h / 2.0)).abs() < 1e-6);
    // a box larger than the window zooms out
    a.change(|a| a.create_node("UMLClass", rect(3000.0, 1000.0, 200.0, 100.0)));
    a.fit();
    assert!(a.zoom < 1.0);
    let b = crate::render::bounds(&a.repo, a.diagram.unwrap()).unwrap();
    assert!(b.w * a.zoom <= 800.0 && b.h * a.zoom <= 600.0);
}

#[test]
fn working_diagrams_cycle_and_close() {
    let mut a = App::new(None);
    let d1 = a.diagram.unwrap();
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    let d3 = a.add_diagram(None, "UMLUseCaseDiagram");
    assert_eq!(a.open, vec![d1, d2, d3]);
    assert_eq!(a.diagram, Some(d3));
    a.cycle_diagram(1);
    assert_eq!(a.diagram, Some(d1), "next wraps to the first");
    a.cycle_diagram(-1);
    assert_eq!(a.diagram, Some(d3), "previous wraps to the last");
    a.cycle_diagram(-1);
    assert_eq!(a.diagram, Some(d2));
    // closing the current diagram activates the one that slid into its place
    a.close_diagram(None);
    assert_eq!((a.open.clone(), a.diagram), (vec![d1, d3], Some(d3)));
    // closing the last one in the list falls back to the first
    a.close_diagram(None);
    assert_eq!((a.open.clone(), a.diagram), (vec![d1], Some(d1)));
    a.open_diagram(d2);
    a.open_diagram(d3);
    a.close_other_diagrams();
    assert_eq!((a.open.clone(), a.diagram), (vec![d3], Some(d3)));
    a.open_diagram(d1);
    a.close_all_diagrams();
    assert!(a.open.is_empty() && a.diagram.is_none());
    a.cycle_diagram(1);
    assert!(a.diagram.is_none());
}

#[test]
fn diagram_navigation_shortcuts() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let d1 = a.diagram.unwrap();
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    let d3 = a.add_diagram(None, "UMLClassDiagram");
    let mut press = |a: &mut App, key: Key, modifiers: Modifiers| {
        let ev = egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    let cs = Modifiers::COMMAND | Modifiers::SHIFT;
    press(&mut a, Key::CloseCurlyBracket, cs);
    assert_eq!(a.diagram, Some(d1));
    press(&mut a, Key::OpenBracket, cs);
    assert_eq!(a.diagram, Some(d3));
    press(&mut a, Key::F4, Modifiers::COMMAND);
    assert_eq!(a.open, vec![d3], "Ctrl+F4 closes the others");
    a.open_diagram(d2);
    press(&mut a, Key::F4, Modifiers::SHIFT);
    assert!(a.open.is_empty(), "Shift+F4 closes all");
}

#[test]
fn select_in_diagram_and_scroll_per_diagram() {
    let mut a = App::new(None);
    a.canvas_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)));
    let d1 = a.diagram.unwrap();
    let v1 = a.change(|a| a.create_node("UMLClass", rect(2000.0, 1500.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v1, "model").unwrap();
    a.scroll = crate::views::P { x: 40.0, y: 80.0 };
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    assert_eq!((a.scroll.x, a.scroll.y), (0.0, 0.0), "a new diagram starts at the origin");
    a.scroll = crate::views::P { x: 8.0, y: 16.0 };
    a.open_diagram(d1);
    assert_eq!((a.scroll.x, a.scroll.y), (40.0, 80.0), "the scroll position is remembered per diagram");
    a.open_diagram(d2);
    assert_eq!((a.scroll.x, a.scroll.y), (8.0, 16.0));

    // shown in one diagram: that diagram opens and the view is selected and centred
    a.focus = Some(m);
    a.select_in_diagram();
    assert_eq!(a.diagram, Some(d1));
    assert_eq!(a.selection, vec![v1]);
    assert_eq!(a.centre_pending, Some(v1));
    a.centre_on(crate::views::P { x: 2050.0, y: 1530.0 });
    assert_eq!((a.scroll.x, a.scroll.y), (1650.0, 1230.0));

    // shown in two diagrams: a picker
    a.open_diagram(d2);
    let v2 = a.change(|a| a.create_view_of(m, rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    a.focus = Some(m);
    a.select_in_diagram();
    assert_eq!(a.diagram_picker.as_ref().map(|p| p.0.clone()), Some(vec![v1, v2]));

    // shown nowhere: a toast
    a.diagram_picker = None;
    let other = a.change(|a| a.add_model(a.repo.root, "UMLClass"));
    a.focus = Some(other);
    a.select_in_diagram();
    assert_eq!(a.toasts.last().map(|t| t.0.as_str()), Some("No diagrams to show"));
}

#[test]
fn selection_clicks_follow_the_original() {
    use egui::{Modifiers, Pos2};
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let c3 = a.change(|a| a.create_node("UMLClass", rect(440.0, 40.0, 100.0, 60.0))).unwrap();
    let click = |a: &mut App, x: f32, y: f32, m: Modifiers| {
        a.on_press(canvas, Pos2::new(x, y), m);
        a.on_release(canvas.min, Pos2::new(x, y));
    };
    click(&mut a, 50.0, 50.0, Modifiers::NONE);
    assert_eq!(a.selection, vec![c1]);
    click(&mut a, 250.0, 50.0, Modifiers::SHIFT);
    assert_eq!(a.selection, vec![c1, c2], "Shift+click adds");
    // Shift+drag never moves anything
    a.on_press(canvas, Pos2::new(450.0, 50.0), Modifiers::SHIFT);
    a.on_move(canvas, Pos2::new(480.0, 90.0));
    a.on_release(canvas.min, Pos2::new(480.0, 90.0));
    assert_eq!(a.selection, vec![c1, c2, c3]);
    assert_eq!(a.repo.f(c3, "left"), 440.0, "a Shift press does not drag");
    // a plain click on a member, without moving, collapses the group to it
    click(&mut a, 250.0, 50.0, Modifiers::NONE);
    assert_eq!(a.selection, vec![c2]);
    // Ctrl+press pans and never selects or moves
    a.on_press(canvas, Pos2::new(450.0, 50.0), Modifiers::COMMAND);
    a.on_move(canvas, Pos2::new(400.0, 20.0));
    a.on_release(canvas.min, Pos2::new(400.0, 20.0));
    assert_eq!(a.selection, vec![c2]);
    assert_eq!(a.repo.f(c3, "left"), 440.0);
    assert_eq!((a.scroll.x, a.scroll.y), (50.0, 30.0));
    // Tab cycles through the top-level views and wraps
    a.select_next_view();
    assert_eq!(a.selection, vec![c3]);
    a.select_next_view();
    assert_eq!(a.selection, vec![c1]);
    a.select(vec![]);
    a.select_next_view();
    assert_eq!(a.selection, vec![c1]);
}

#[test]
fn format_toggles_invert_the_shared_value() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let note = a.change(|a| a.create_node("Note", rect(440.0, 40.0, 100.0, 60.0))).unwrap();
    a.select(vec![c1, c2, note]);
    assert_eq!(a.flag_views("suppressAttributes"), vec![c1, c2], "only views with the flag");
    assert_eq!(a.flag_state("suppressAttributes"), Some(false));
    let undo = a.undo.len();
    a.toggle_flag("suppressAttributes");
    assert_eq!(a.undo.len(), undo + 1);
    assert!(a.repo.bool(c1, "suppressAttributes") && a.repo.bool(c2, "suppressAttributes"));
    a.toggle_flag("suppressAttributes");
    assert_eq!(a.flag_state("suppressAttributes"), Some(false));
    // mixed: set to true
    a.change(|a| a.repo.set(c1, "showVisibility", json!(false)));
    assert_eq!(a.flag_state("showVisibility"), None);
    a.toggle_flag("showVisibility");
    assert!(a.repo.bool(c1, "showVisibility") && a.repo.bool(c2, "showVisibility"));

    // Ctrl+Shift+N toggles the namespace, and does not start a new project
    let mut press = |a: &mut App, key: Key, modifiers: Modifiers| {
        let ev = egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    let cs = Modifiers::COMMAND | Modifiers::SHIFT;
    press(&mut a, Key::N, cs);
    assert!(a.repo.alive(c1) && a.repo.bool(c1, "showNamespace"));
    press(&mut a, Key::A, cs);
    assert!(a.repo.bool(c1, "suppressAttributes"));
    assert_eq!(a.selection, vec![c1, c2, note], "Ctrl+Shift+A is not Select All");
}

#[test]
fn ctrl_shift_g_toggles_snap_not_the_grid() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let mut press = |a: &mut App, modifiers: Modifiers| {
        let ev = egui::Event::Key { key: Key::G, physical_key: None, pressed: true, repeat: false, modifiers };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    press(&mut a, Modifiers::COMMAND | Modifiers::SHIFT);
    assert!(!a.snap && a.show_grid);
    press(&mut a, Modifiers::COMMAND);
    assert!(!a.snap && !a.show_grid);
}
