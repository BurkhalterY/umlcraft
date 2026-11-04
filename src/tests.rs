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
    assert_eq!(crate::views::operation_text(&a.repo, op, crate::views::Show::ALL), "+placeOrder(item: Order, qty: int): boolean");
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
    // Ctrl+Shift+B and Ctrl+Shift+Y hide the property strings and the types; Ctrl+Shift+Y is not Redo
    press(&mut a, Key::B, cs);
    press(&mut a, Key::Y, cs);
    assert!(!a.repo.bool(c1, "showProperty") && !a.repo.bool(c1, "showType"));
    assert!(a.repo.alive(c1));
}

#[test]
fn show_type_and_property() {
    use crate::views::{attribute_text, operation_text, Show};
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let at = a.change(|a| a.add_feature(m, "UMLAttribute")).unwrap();
    crate::parse::apply_attribute(&mut a.repo, at, "-items: Item[*] = none");
    a.repo.set(at, "isOrdered", json!(true));
    a.repo.set(at, "isReadOnly", json!(true));
    assert_eq!(attribute_text(&a.repo, at, Show::ALL), "-items: Item[*] = none {readOnly, ordered}");
    assert_eq!(attribute_text(&a.repo, at, Show { ty: false, prop: false, ..Show::ALL }), "-items[*] = none");
    let op = a.change(|a| a.add_feature(m, "UMLOperation")).unwrap();
    crate::parse::apply_operation(&mut a.repo, op, "+find(key: String): Item");
    assert_eq!(operation_text(&a.repo, op, Show { ty: false, ..Show::ALL }), "+find(key)");
    // association ends show their property strings unless showProperty is off
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c, c2)).unwrap();
    let asc = a.repo.refd(e, "model").unwrap();
    let end2 = a.repo.child(asc, "end2").unwrap();
    a.repo.set(end2, "isUnique", json!(true));
    let ms = measure();
    let d = a.diagram.unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let hp = a.repo.refd(e, "headPropertyLabel").unwrap();
    assert_eq!(a.repo.str(hp, "text"), "{unique}");
    a.repo.set(e, "showProperty", json!(false));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(hp, "text"), "");
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

#[test]
fn copy_and_paste_views_and_models() {
    use crate::app::Area;
    let mut a = App::new(None);
    a.canvas_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0)));
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let c3 = a.change(|a| a.create_node("UMLClass", rect(440.0, 40.0, 100.0, 60.0))).unwrap();
    let e12 = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let e23 = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c2, c3)).unwrap();
    let m1 = a.repo.refd(c1, "model").unwrap();

    // views: the line to an uncopied box stays behind; pasted views share the models
    a.last_area = Area::Canvas;
    a.select(vec![c1, c2, e12, e23]);
    let text = a.copy_text().unwrap();
    let before = a.repo.list(d, "ownedViews").len();
    let classes = a.repo.all_of("UMLClass").len();
    a.paste_text(&text);
    let views = a.repo.list(d, "ownedViews");
    assert_eq!(views.len(), before + 3, "two boxes and the line between them");
    assert_eq!(a.repo.all_of("UMLClass").len(), classes, "no model copies");
    assert_eq!(a.selection.len(), 3, "pasted views are selected");
    let pasted_box = a.selection.iter().copied().find(|v| a.repo.refd(*v, "model") == Some(m1)).unwrap();
    let pasted_edge = a.selection.iter().copied().find(|v| crate::views::is_edge(&a.repo, *v)).unwrap();
    assert!(a.selection.contains(&a.repo.refd(pasted_edge, "tail").unwrap()), "the line joins the pasted boxes");
    // the group (40..340 x 40..100) is centred on the screen centre (400, 300)
    assert_eq!((a.repo.f(pasted_box, "left"), a.repo.f(pasted_box, "top")), (250.0, 270.0));

    // cut views removes them, models stay
    a.select(vec![c3]);
    a.cmd_cut(None);
    assert!(!a.repo.alive(c3) && !a.repo.alive(e23));
    assert_eq!(a.repo.all_of("UMLClass").len(), classes);

    // a model: deep copy with new ids into the selected element, or its owner
    let at = a.change(|a| a.add_feature(m1, "UMLAttribute")).unwrap();
    a.last_area = Area::Explorer;
    a.select(vec![]);
    a.focus = Some(m1);
    let text = a.copy_text().unwrap();
    a.paste_text(&text);
    let copy = a.focus.unwrap();
    assert_ne!(copy, m1);
    assert_eq!(a.repo.name(copy), a.repo.name(m1));
    assert_eq!(a.repo.parent(copy), Some(m1), "a class can own classes, so the copy goes inside the selected class");
    let attrs = a.repo.list(copy, "attributes");
    assert_eq!(attrs.len(), 1);
    assert_ne!(a.repo.id(attrs[0]), a.repo.id(at));
    assert_eq!(a.repo.parent(attrs[0]), Some(copy));
    // an operation cannot hold a class: the copy goes to its owner
    let op = a.change(|a| a.add_feature(m1, "UMLOperation")).unwrap();
    a.focus = Some(op);
    a.paste_text(&text);
    assert_eq!(a.repo.parent(a.focus.unwrap()), Some(m1));
    // an attribute pastes into a class
    a.focus = Some(at);
    let text = a.copy_text().unwrap();
    a.focus = Some(copy);
    a.paste_text(&text);
    assert_eq!(a.repo.list(copy, "attributes").len(), 2);
    // cut a model deletes it
    a.focus = Some(copy);
    a.cmd_cut(None);
    assert!(!a.repo.alive(copy));
    // pasting views whose model is gone does nothing
    a.select(vec![]);
    let n = a.repo.list(d, "ownedViews").len();
    a.paste_text(&a.clipboard.clone().unwrap());
    a.last_area = Area::Canvas;
    a.select(vec![c1]);
    let text = a.copy_text().unwrap();
    a.change(|a| a.delete_models(&[m1]));
    let n2 = a.repo.list(d, "ownedViews").len();
    a.paste_text(&text);
    assert_eq!(a.repo.list(d, "ownedViews").len(), n2);
    // other text is ignored
    a.paste_text("hello");
    let _ = n;
    // the saved file still parses
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn clipboard_shortcuts_arrive_as_events() {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    a.last_area = crate::app::Area::Canvas;
    a.select(vec![c1]);
    let mut send = |a: &mut App, ev: egui::Event| {
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    send(&mut a, egui::Event::Copy);
    let text = a.clipboard.clone().expect("Ctrl+C copies");
    send(&mut a, egui::Event::Paste(text));
    assert_eq!(a.repo.list(d, "ownedViews").len(), 2);
    send(&mut a, egui::Event::Cut);
    assert_eq!(a.repo.list(d, "ownedViews").len(), 1);
}

#[test]
fn copy_diagram_as_image_deselects_first() {
    let ms = measure();
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    a.select(vec![c1]);
    let d = a.diagram.unwrap();
    let info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let png = crate::render::to_png(&a.repo, d, &info, &ms, 1.0);
    assert!(png.is_ok());
    a.copy_diagram_image(png);
    assert!(a.selection.is_empty());
    if std::env::var("DISPLAY").is_ok() {
        assert!(a.message.is_none(), "{:?}", a.message);
        let img = a.sys_clipboard.as_mut().unwrap().get_image().unwrap();
        assert!(img.width > 100 && img.bytes[3] == 255, "an opaque picture of the diagram");
    }
    a.copy_diagram_image(Err("The diagram is empty.".into()));
    assert!(a.message.is_some());
}

#[test]
fn move_up_down_and_open_sub_diagram() {
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let c1 = a.change(|a| a.add_model(model, "UMLClass"));
    let p1 = a.change(|a| a.add_model(model, "UMLPackage"));
    let c2 = a.change(|a| a.add_model(model, "UMLClass"));
    let kids = |a: &App| a.repo.list(model, "ownedElements");
    let before = kids(&a);
    a.move_element(c2, true);
    let after = kids(&a);
    // c2 swaps with c1, skipping the package in between (another explorer group)
    assert_eq!(after.iter().position(|x| *x == c2), before.iter().position(|x| *x == c1));
    assert_eq!(after.iter().position(|x| *x == p1), before.iter().position(|x| *x == p1));
    a.move_element(c2, true);
    assert_eq!(kids(&a), after, "already first of its group");
    a.do_undo();
    assert_eq!(kids(&a), before);

    a.focus = Some(c1);
    a.open_sub_diagram();
    assert_eq!(a.toasts.last().map(|t| t.0.as_str()), Some("No sub-diagram in the selected element"));
    let d = a.change(|a| a.add_diagram(Some(c1), "UMLClassDiagram"));
    a.open_diagram(a.repo.all_of("Diagram")[0]);
    a.focus = Some(c1);
    a.open_sub_diagram();
    assert_eq!(a.diagram, Some(d));
}

#[test]
fn rounded_and_curved_line_styles() {
    use crate::views::{drawn_points, P};
    let pts = [P { x: 0.0, y: 0.0 }, P { x: 100.0, y: 0.0 }, P { x: 100.0, y: 100.0 }];
    assert_eq!(drawn_points(&pts, 1), pts.to_vec());
    let round = drawn_points(&pts, 2);
    assert_eq!((round[0], *round.last().unwrap()), (pts[0], pts[2]));
    assert!(!round.contains(&pts[1]), "the corner is cut");
    assert!(round.iter().any(|p| p.x == 90.0 && p.y == 0.0), "the rounding starts one radius before the corner");
    let curve = drawn_points(&pts, 3);
    assert_eq!((curve[0], *curve.last().unwrap()), (pts[0], pts[2]));
    assert!(curve.len() > 3);

    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(240.0, 240.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let e2 = a.change(|a| a.create_edge(&tool("UMLDependency", AssocInit::Plain), c1, c2)).unwrap();
    a.select(vec![e, e2, c1]);
    a.set_line_style(3);
    assert_eq!(a.line_style_state(), Some(3));
    a.select(vec![e]);
    a.set_line_style(2);
    a.select(vec![e, e2]);
    assert_eq!(a.line_style_state(), None, "mixed");
    // rounded rectilinear is routed like rectilinear
    let ms = measure();
    crate::views::layout_diagram(&mut a.repo, &ms, a.diagram.unwrap());
    assert_eq!(crate::views::points_of(&a.repo, e).len(), 3, "one elbow");
}

#[test]
fn inline_item_keys() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let lay = |a: &mut App| a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let editing = |a: &App| a.editing.as_ref().map(|e| e.kind.clone());
    lay(&mut a);
    // Ctrl+Enter on the name adds an attribute and edits it once laid out
    a.item_key(v, EditKind::Name(m), ItemKey::Add);
    let at1 = a.repo.list(m, "attributes")[0];
    assert_eq!(a.pending_item, Some((v, at1, 3)));
    lay(&mut a);
    a.take_pending_item();
    assert_eq!(editing(&a), Some(EditKind::Feature(at1)));
    // Ctrl+Enter on an item adds a sibling; Ctrl+Shift+Enter an operation
    a.item_key(v, EditKind::Feature(at1), ItemKey::Add);
    let at2 = a.repo.list(m, "attributes")[1];
    a.item_key(v, EditKind::Feature(at2), ItemKey::AddOperation);
    let op = a.repo.list(m, "operations")[0];
    lay(&mut a);
    // Up/Down walk name → attributes → operations
    a.item_key(v, EditKind::Name(m), ItemKey::Nav(false));
    assert_eq!(editing(&a), Some(EditKind::Feature(at1)));
    a.item_key(v, EditKind::Feature(at2), ItemKey::Nav(false));
    assert_eq!(editing(&a), Some(EditKind::Feature(op)));
    a.item_key(v, EditKind::Feature(at1), ItemKey::Nav(true));
    assert_eq!(editing(&a), Some(EditKind::Name(m)));
    // template parameters come before the name
    let tp = a.change(|a| a.add_feature(m, "UMLTemplateParameter")).unwrap();
    lay(&mut a);
    a.item_key(v, EditKind::Name(m), ItemKey::Nav(true));
    assert_eq!(editing(&a), Some(EditKind::Feature(tp)));
    a.item_key(v, EditKind::Feature(tp), ItemKey::Nav(false));
    assert_eq!(editing(&a), Some(EditKind::Name(m)));
    a.delete_models(&[tp]);
    lay(&mut a);
    // Ctrl+Up reorders
    a.item_key(v, EditKind::Feature(at2), ItemKey::Move(true));
    assert_eq!(a.repo.list(m, "attributes"), vec![at2, at1]);
    lay(&mut a);
    // Ctrl+Delete deletes and moves to the item now at that index
    a.pending_item = None;
    a.item_key(v, EditKind::Feature(at2), ItemKey::Delete);
    assert!(!a.repo.alive(at2));
    lay(&mut a);
    a.take_pending_item();
    assert_eq!(editing(&a), Some(EditKind::Feature(at1)));
    // adding to a suppressed compartment shows it again
    a.change(|a| a.repo.set(v, "suppressOperations", json!(true)));
    a.item_key(v, EditKind::Name(m), ItemKey::AddOperation);
    assert!(!a.repo.bool(v, "suppressOperations"));
}

#[test]
fn inline_editor_keys_through_frames() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let mut frame = |a: &mut App, events: Vec<egui::Event>| {
        let input = egui::RawInput { screen_rect: Some(screen), events, ..Default::default() };
        let mut out = ctx.run_ui(input, |ui| {
            egui::CentralPanel::no_frame().show_inside(ui, |ui| a.canvas_ui(ui));
            a.take_pending_item();
        });
        out.textures_delta.clear();
    };
    let key = |k: Key, m: Modifiers| egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: m };
    frame(&mut a, vec![]);
    a.edit_view(v, None);
    frame(&mut a, vec![]);
    frame(&mut a, vec![key(Key::Enter, Modifiers::COMMAND)]);
    assert_eq!(a.repo.list(m, "attributes").len(), 1, "Ctrl+Enter adds an attribute");
    frame(&mut a, vec![]);
    frame(&mut a, vec![]);
    let at = a.repo.list(m, "attributes")[0];
    assert_eq!(a.editing.as_ref().map(|e| e.kind.clone()), Some(crate::app::EditKind::Feature(at)), "and edits it");
    frame(&mut a, vec![egui::Event::Text("-id: int".into())]);
    frame(&mut a, vec![key(Key::Enter, Modifiers::NONE)]);
    assert_eq!(a.repo.name(at), "id");
    assert_eq!(a.repo.str(at, "visibility"), "private");
    assert!(a.editing.is_none());

    // a note: Enter commits, Shift+Enter is a new line
    let n = a.change(|a| a.create_node("Note", rect(300.0, 40.0, 140.0, 60.0))).unwrap();
    a.edit_view(n, None);
    frame(&mut a, vec![]);
    frame(&mut a, vec![egui::Event::Text("one".into())]);
    frame(&mut a, vec![key(Key::Enter, Modifiers::SHIFT)]);
    frame(&mut a, vec![egui::Event::Text("two".into())]);
    frame(&mut a, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(a.editing.is_none());
    assert_eq!(a.repo.str(n, "text"), "one\ntwo");
}

#[test]
fn name_and_stereotype_expressions() {
    use crate::parse::{apply_attribute, apply_name, apply_operation, name_text};
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let c = a.add_model(model, "UMLClass");
    let order = a.add_model(model, "UMLClass");
    a.repo.set(order, "name", json!("Order"));
    apply_name(&mut a.repo, c, "<<entity>> #Customer");
    assert_eq!(a.repo.name(c), "Customer");
    assert_eq!(a.repo.str(c, "stereotype"), "entity");
    assert_eq!(a.repo.str(c, "visibility"), "protected");
    assert_eq!(name_text(&a.repo, c), "<<entity>> Customer");
    // an existing stereotype is linked; no stereotype clears it
    let st = a.add_model(model, "UMLStereotype");
    a.repo.set(st, "name", json!("entity"));
    apply_name(&mut a.repo, c, "«entity» Customer");
    assert_eq!(a.repo.refd(c, "stereotype"), Some(st));
    apply_name(&mut a.repo, c, "Customer");
    assert!(!a.repo.has(c, "stereotype"));
    // attributes and operations: stereotype, and a type naming a classifier is linked
    let at = a.add_feature(c, "UMLAttribute").unwrap();
    apply_attribute(&mut a.repo, at, "<<id>> -order: Order");
    assert_eq!(a.repo.str(at, "stereotype"), "id");
    assert_eq!(a.repo.refd(at, "type"), Some(order));
    assert_eq!(crate::views::attribute_text(&a.repo, at, crate::views::Show::ALL), "«id» -order: Order");
    apply_attribute(&mut a.repo, at, "«id» -order: int");
    assert_eq!(a.repo.str(at, "type"), "int");
    let op = a.add_feature(c, "UMLOperation").unwrap();
    apply_operation(&mut a.repo, op, "<<query>> +total(o: Order): int");
    assert_eq!(a.repo.str(op, "stereotype"), "query");
    let p = a.repo.list(op, "parameters")[0];
    assert_eq!(a.repo.refd(p, "type"), Some(order));
}

#[test]
fn operation_parameters_follow_their_position() {
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let c = a.add_model(model, "UMLClass");
    let op = a.add_feature(c, "UMLOperation").unwrap();
    crate::parse::apply_operation(&mut a.repo, op, "f(a: int, b: int): int");
    let ps = a.repo.list(op, "parameters");
    crate::parse::apply_operation(&mut a.repo, op, "f(x: long, b: int): int");
    let ps2 = a.repo.list(op, "parameters");
    assert_eq!(ps, ps2, "renamed in place");
    assert_eq!(a.repo.name(ps2[0]), "x");
    crate::parse::apply_operation(&mut a.repo, op, "f");
    assert!(a.repo.list(op, "parameters").is_empty(), "no parentheses, no parameters");
}

#[test]
fn reconnect_lines_and_tidy_bend_points() {
    use crate::views::{clean_points, points_of, P};
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let c3 = a.change(|a| a.create_node("UMLClass", rect(400.0, 300.0, 100.0, 60.0))).unwrap();
    let i1 = a.change(|a| a.create_node("UMLInterface", rect(40.0, 300.0, 100.0, 60.0))).unwrap();
    let g = a.change(|a| a.create_edge(&tool("UMLGeneralization", AssocInit::Plain), c1, c2)).unwrap();
    let s = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let gm = a.repo.refd(g, "model").unwrap();
    let m3 = a.repo.refd(c3, "model").unwrap();

    // drag the head of the generalization onto c3: view and model follow
    a.select(vec![g]);
    let head = *points_of(&a.repo, g).last().unwrap();
    a.on_press(canvas, Pos2::new(head.x as f32, head.y as f32), Modifiers::NONE);
    assert!(matches!(a.drag, Some(crate::app::Drag::Reconnect { tail: false, .. })));
    a.on_move(canvas, Pos2::new(450.0, 330.0));
    a.on_release(canvas.min, Pos2::new(450.0, 330.0));
    assert_eq!(a.repo.refd(g, "head"), Some(c3));
    assert_eq!(a.repo.refd(gm, "target"), Some(m3));
    // an association end follows its view
    a.reconnect(s, true, c3).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!(a.repo.refd(a.repo.child(sm, "end1").unwrap(), "reference"), Some(m3));
    // an interface realization must end on an interface
    let r = a.change(|a| a.create_edge(&tool("UMLInterfaceRealization", AssocInit::Plain), c1, i1)).unwrap();
    assert_eq!(a.reconnect(r, false, c2).unwrap_err(), "InterfaceRealization cannot be connected to Class");
    assert_eq!(a.repo.refd(r, "head"), Some(i1));

    // tidy-up
    let pts = [P { x: 0.0, y: 0.0 }, P { x: 50.0, y: 0.0 }, P { x: 100.0, y: 0.0 }, P { x: 103.0, y: 2.0 }, P { x: 100.0, y: 100.0 }];
    let c = clean_points(&pts, false);
    assert_eq!((c.len(), c[0], c[2]), (3, pts[0], pts[4]), "the collinear point and one of the two close points go");
    let rect_pts = [P { x: 0.0, y: 0.0 }, P { x: 50.0, y: 0.0 }, P { x: 100.0, y: 0.0 }, P { x: 100.0, y: 100.0 }];
    assert_eq!(clean_points(&rect_pts, true), vec![rect_pts[0], rect_pts[2], rect_pts[3]]);

    // a rectilinear line: dragging its middle segment across moves both of its points
    a.change(|a| a.repo.set(s, "lineStyle", json!(0)));
    a.change(|a| a.repo.set(s, "points", json!("90:100;90:200;450:200;450:300")));
    a.select(vec![s]);
    a.on_press(canvas, Pos2::new(250.0, 200.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(250.0, 240.0));
    a.on_release(canvas.min, Pos2::new(250.0, 240.0));
    let p = points_of(&a.repo, s);
    assert_eq!((p[1].y, p[2].y), (240.0, 240.0));
    assert_eq!((p[1].x, p[2].x), (90.0, 450.0));
}

#[test]
fn dropping_models_brings_their_relationships() {
    use crate::views::{is_edge, P};
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let model = a.repo.children(a.repo.root)[0].1;
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLGeneralization", AssocInit::Plain), c1, c2)).unwrap();
    let (m1, m2) = (a.repo.refd(c1, "model").unwrap(), a.repo.refd(c2, "model").unwrap());
    let rel = a.repo.refd(e, "model").unwrap();
    let edges = |a: &App| a.repo.list(d, "ownedViews").into_iter().filter(|v| is_edge(&a.repo, *v)).count();

    // a fresh diagram: drop c1, then c2 brings the generalization with it
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    a.drop_model(m1, P { x: 40.0, y: 40.0 }).unwrap();
    let vs = a.drop_model(m2, P { x: 300.0, y: 40.0 }).unwrap();
    assert_eq!(vs.len(), 1);
    let lines: Vec<_> = a.repo.list(d2, "ownedViews").into_iter().filter(|v| is_edge(&a.repo, *v)).collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(a.repo.refd(lines[0], "model"), Some(rel));

    // dropping a relationship already shown selects it and alerts
    assert_eq!(a.drop_model(rel, P::default()).unwrap_err(), "Relationship View is already existed in this Diagram.");
    assert_eq!(a.selection.len(), 3);
    // dropping a relationship whose ends are missing creates them
    let d3 = a.add_diagram(None, "UMLClassDiagram");
    let vs = a.drop_model(rel, P { x: 40.0, y: 40.0 }).unwrap();
    assert_eq!(vs.len(), 1);
    assert_eq!(a.repo.list(d3, "ownedViews").len(), 3);
    // a hidden view comes back instead of a second one
    a.open_diagram(d);
    a.change(|a| a.repo.set(c1, "visible", json!(false)));
    let n = a.repo.list(d, "ownedViews").len();
    assert_eq!(a.drop_model(m1, P::default()).unwrap(), vec![c1]);
    assert_eq!(a.repo.list(d, "ownedViews").len(), n);
    assert_eq!(edges(&a), 1);
    // a model from another package shows its namespace
    let pkg = a.change(|a| a.add_model(model, "UMLPackage"));
    let other = a.change(|a| a.add_model(pkg, "UMLClass"));
    let v = a.drop_model(other, P::default()).unwrap()[0];
    assert!(a.repo.bool(v, "showNamespace"));
    assert!(!a.repo.bool(c2, "showNamespace"));
}

#[test]
fn dropping_boxes_into_a_package_moves_view_and_model() {
    use egui::{Modifiers, Pos2};
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    let owner = a.repo.parent(d).unwrap();
    let pk = a.change(|a| a.create_node("UMLPackage", rect(300.0, 40.0, 300.0, 300.0))).unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let (pm, cm) = (a.repo.refd(pk, "model").unwrap(), a.repo.refd(c, "model").unwrap());
    let drag = |a: &mut App, from: (f32, f32), to: (f32, f32)| {
        a.on_press(canvas, Pos2::new(from.0, from.1), Modifiers::NONE);
        a.on_move(canvas, Pos2::new(to.0, to.1));
        a.on_release(canvas.min, Pos2::new(to.0, to.1));
    };
    // into the package: the class model is now owned by the package, the view contained and in front
    drag(&mut a, (60.0, 60.0), (400.0, 160.0));
    assert_eq!(a.repo.refd(c, "containerView"), Some(pk));
    assert_eq!(a.repo.parent(cm), Some(pm));
    let list = a.repo.list(d, "ownedViews");
    assert!(list.iter().position(|x| *x == c) > list.iter().position(|x| *x == pk));
    // moving the package moves the contained class too
    let (cx, px) = (a.repo.f(c, "left"), a.repo.f(pk, "left"));
    a.select(vec![pk]);
    drag(&mut a, (590.0, 330.0), (630.0, 330.0));
    assert_eq!(a.repo.f(c, "left") - cx, a.repo.f(pk, "left") - px);
    assert!(a.repo.f(pk, "left") > px);
    // a package cannot go into what it contains
    a.select(vec![]);
    let inner = a.change(|a| a.create_node("UMLPackage", rect(700.0, 400.0, 100.0, 100.0))).unwrap();
    a.change(|a| a.set_container(&[inner], Some(pk)));
    assert!(!a.can_contain_view(inner, pk));
    // out onto empty canvas: back to the diagram's owner
    let at = (a.repo.f(c, "left") as f32 + 10.0, a.repo.f(c, "top") as f32 + 10.0);
    a.select(vec![]);
    drag(&mut a, at, (100.0, 500.0));
    assert_eq!(a.repo.refd(c, "containerView"), None);
    assert_eq!(a.repo.parent(cm), Some(owner));
    assert!(!a.repo.raw(pk, "containedViews").map(|v| v.to_string().contains(&a.repo.id(c))).unwrap_or(false));
    a.do_undo();
    assert_eq!(a.repo.parent(cm), Some(pm), "one undo step");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn explorer_drag_relocates_elements() {
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let p1 = a.add_model(model, "UMLPackage");
    let p2 = a.add_model(p1, "UMLPackage");
    let c = a.add_model(model, "UMLClass");
    let at = a.add_feature(c, "UMLAttribute").unwrap();
    assert!(a.can_relocate(c, p2));
    assert!(!a.can_relocate(p1, p2), "not into something inside it");
    assert!(!a.can_relocate(c, model), "already there");
    assert!(!a.can_relocate(at, p1), "a package takes no attributes");
    let id = a.repo.id(c);
    a.repo.relocate(c, p2, "ownedElements");
    assert_eq!(a.repo.parent(c), Some(p2));
    assert_eq!(a.repo.id(c), id, "same element, same id");
    assert_eq!(a.repo.list(c, "attributes"), vec![at], "with its contents");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn rubber_band_selects_by_overlap() {
    use egui::{Modifiers, Pos2};
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let hidden = a.change(|a| a.create_node("UMLClass", rect(40.0, 200.0, 100.0, 60.0))).unwrap();
    a.change(|a| a.repo.set(hidden, "visible", json!(false)));
    // a band touching only the corner of c1 selects it
    a.on_press(canvas, Pos2::new(130.0, 90.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(200.0, 250.0));
    a.on_release(canvas.min, Pos2::new(200.0, 250.0));
    assert_eq!(a.selection, vec![c1], "overlap is enough, hidden views are skipped");
    // Shift+band adds; a selected view counts 5 px larger
    a.on_press(canvas, Pos2::new(250.0, 95.0), Modifiers::SHIFT);
    a.on_move(canvas, Pos2::new(310.0, 140.0));
    a.on_release(canvas.min, Pos2::new(310.0, 140.0));
    assert_eq!(a.selection, vec![c1, c2]);
    // a band 3 px below c1 still reaches it while it is selected
    a.select(vec![c1]);
    a.on_press(canvas, Pos2::new(60.0, 103.0), Modifiers::SHIFT);
    a.on_move(canvas, Pos2::new(70.0, 110.0));
    a.on_release(canvas.min, Pos2::new(70.0, 110.0));
    assert_eq!(a.selection, vec![c1]);
    a.select(vec![]);
    a.on_press(canvas, Pos2::new(60.0, 103.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(70.0, 110.0));
    a.on_release(canvas.min, Pos2::new(70.0, 110.0));
    assert!(a.selection.is_empty(), "but not when it is not selected");
    // the band stops at the edge
    a.on_press(canvas, Pos2::new(500.0, 500.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(-50.0, -50.0));
    assert!(matches!(a.drag, Some(crate::app::Drag::Rubber { now, .. }) if now.x == 0.0 && now.y == 0.0));
    a.on_release(canvas.min, Pos2::new(-50.0, -50.0));
    a.select_all();
    assert_eq!(a.selection, vec![c1, c2]);
}

#[test]
fn toolbox_lock_esc_and_text_on_double_click() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let class = tool("UMLClass", AssocInit::Plain);
    a.toolbox_click(class);
    assert_eq!((a.tool, a.tool_sticky), (Some(class), false));
    a.toolbox_click(class);
    assert!(a.tool_sticky, "clicking the active item locks it");
    a.toolbox_click(class);
    assert_eq!((a.tool, a.tool_sticky), (None, false), "and once more unlocks it");
    a.toolbox_click(class);
    a.toolbox_click(class);
    let ev = egui::Event::Key { key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE };
    let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
    out.textures_delta.clear();
    assert_eq!((a.tool, a.tool_sticky), (None, false), "Esc goes back to Select");

    a.create_text_at(crate::views::P { x: 203.0, y: 101.0 });
    let t = a.selection[0];
    assert_eq!(a.repo.ty(t), "UMLTextView");
    assert_eq!(a.repo.str(t, "text"), "Text");
    assert_eq!((a.repo.f(t, "left"), a.repo.f(t, "width")), (200.0, 100.0));
    assert!(matches!(a.editing.as_ref().map(|e| e.kind.clone()), Some(crate::app::EditKind::Text(x)) if x == t));
}

#[test]
fn dragging_a_line_label_keeps_it_with_the_line() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    a.change(|a| a.repo.set(m, "name", json!("owns")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l = a.repo.refd(e, "nameLabel").unwrap();
    let b = crate::views::node_rect(&a.repo, l);
    let at = Pos2::new((b.x + b.w / 2.0) as f32, (b.y + 6.0) as f32);
    a.on_press(canvas, at, Modifiers::NONE);
    assert!(matches!(a.drag, Some(crate::app::Drag::Label { .. })));
    assert_eq!(a.selection, vec![e]);
    a.on_move(canvas, at + egui::vec2(30.0, -20.0));
    a.on_release(canvas.min, at + egui::vec2(30.0, -20.0));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b2 = crate::views::node_rect(&a.repo, l);
    assert!((b2.x - b.x - 30.0).abs() <= 1.0 && (b2.y - b.y + 20.0).abs() <= 1.0, "moved by the drag: {b:?} -> {b2:?}");
    // the label follows when the line moves
    a.change(|a| {
        a.repo.set(c2, "top", json!(140));
        a.repo.set(c1, "top", json!(140));
    });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b3 = crate::views::node_rect(&a.repo, l);
    assert!((b3.y - b2.y - 100.0).abs() <= 1.0);
    a.do_undo();
    a.do_undo();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(crate::views::node_rect(&a.repo, l).x, b.x, "one undo step");
}

#[test]
fn association_end_inline_edit_by_proximity() {
    use crate::app::EditKind;
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let m = a.repo.refd(e, "model").unwrap();
    let (e1, e2) = (a.repo.child(m, "end1").unwrap(), a.repo.child(m, "end2").unwrap());
    let pts = crate::views::points_of(&a.repo, e);
    let kind = |a: &App| a.editing.as_ref().map(|x| x.kind.clone());
    a.edit_view(e, Some(crate::views::P { x: pts[0].x + 10.0, y: pts[0].y }));
    assert_eq!(kind(&a), Some(EditKind::Name(e1)));
    a.editing.as_mut().unwrap().text = "-owner".into();
    a.close_inline(true);
    assert_eq!((a.repo.name(e1), a.repo.str(e1, "visibility")), ("owner".to_string(), "private".to_string()));
    a.edit_view(e, Some(crate::views::P { x: pts[1].x - 10.0, y: pts[1].y }));
    assert_eq!(kind(&a), Some(EditKind::Name(e2)));
    a.editing = None;
    a.edit_view(e, Some(crate::views::P { x: (pts[0].x + pts[1].x) / 2.0, y: pts[0].y }));
    assert_eq!(kind(&a), Some(EditKind::Name(m)), "the middle edits the association");
}

#[test]
fn realization_lines() {
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let t = tool("UMLRealization", AssocInit::Plain);
    assert!(a.can_connect(&t, c1, c2));
    let e = a.change(|a| a.create_edge(&t, c1, c2)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLRealizationView");
    let m = a.repo.refd(e, "model").unwrap();
    assert_eq!(a.repo.refd(m, "source"), a.repo.refd(c1, "model"));
}

#[test]
fn colours_apply_to_the_selected_views() {
    use crate::render::Rgba;
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    a.select(vec![c1, c2]);
    assert_eq!(a.color_state("fillColor"), Some(Rgba(255, 255, 255, 255)));
    let undo = a.undo.len();
    a.set_color("fillColor", Rgba(255, 0, 0, 255));
    a.set_color("fillColor", Rgba(250, 10, 0, 255));
    assert_eq!(a.undo.len(), undo + 1, "a picker drag is one undo step");
    assert_eq!(a.repo.str(c1, "fillColor"), "#fa0a00");
    a.select(vec![c1]);
    a.style_edit = None;
    a.set_color("fillColor", Rgba(0, 0, 255, 255));
    a.select(vec![c1, c2]);
    assert_eq!(a.color_state("fillColor"), None, "mixed");
    a.do_undo();
    assert_eq!(a.repo.str(c1, "fillColor"), "#fa0a00");
    // Ctrl+Shift+L opens the line colour window, and is not taken as Ctrl+L
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let ev = egui::Event::Key { key: egui::Key::L, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT };
    let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
    out.textures_delta.clear();
    assert_eq!(a.color_dialog.as_ref().map(|d| d.field), Some("lineColor"));
}

#[test]
fn font_size_reaches_the_layout() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 60.0, 40.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let at = a.change(|a| a.add_feature(m, "UMLAttribute")).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let (w13, h13) = (a.repo.f(c, "width"), a.repo.f(c, "height"));
    a.select(vec![c]);
    assert_eq!(a.font_state(), (Some("Arial".into()), Some(13.0)));
    a.set_font(None, Some(26.0));
    assert_eq!(a.repo.str(c, "font"), "Arial;26;0");
    a.change(|a| {
        a.repo.set(c, "width", json!(10));
        a.repo.set(c, "height", json!(10));
    });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.f(c, "width") > w13 && a.repo.f(c, "height") > h13, "the box grows with the text");
    let row = a.repo.list(a.repo.refd(c, "attributeCompartment").unwrap(), "subViews")[0];
    assert_eq!(a.repo.f(row, "height"), 26.0);
    assert!(a.repo.str(row, "font").starts_with("Arial;26;"));
    let nm = a.repo.refd(a.repo.refd(c, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nm, "font"), "Arial;26;1", "the name keeps its bold style");
    let _ = at;
}

#[test]
fn auto_resize_fits_the_content() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 300.0, 200.0))).unwrap();
    a.select(vec![c]);
    a.toggle_flag("autoResize");
    assert!(a.repo.bool(c, "autoResize"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.f(c, "width") < 300.0 && a.repo.f(c, "height") < 200.0, "it shrinks to its content");
    // the corner handle does nothing: the press starts a move instead
    let (x2, y2) = (a.repo.f(c, "left") + a.repo.f(c, "width"), a.repo.f(c, "top") + a.repo.f(c, "height"));
    a.on_press(canvas, Pos2::new(x2 as f32, y2 as f32), Modifiers::NONE);
    assert!(!matches!(a.drag, Some(crate::app::Drag::Resize { .. })));
}

#[test]
fn stereotype_display_commands() {
    use egui::{Key, Modifiers};
    let ms = measure();
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.change(|a| a.repo.set(m, "stereotype", json!("entity")));
    a.select(vec![c]);
    assert_eq!(a.stereotype_display_state().as_deref(), Some("label"));
    let ev = egui::Event::Key { key: Key::Num0, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::COMMAND | Modifiers::SHIFT };
    let zoom = a.zoom;
    let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
    out.textures_delta.clear();
    assert_eq!(a.repo.str(c, "stereotypeDisplay"), "none", "Ctrl+Shift+0");
    assert_eq!(a.zoom, zoom, "not Actual Size");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let st = a.repo.refd(a.repo.refd(c, "nameCompartment").unwrap(), "stereotypeLabel").unwrap();
    assert!(matches!(a.repo.raw(st, "visible"), Some(serde_json::Value::Bool(false))), "the stereotype is hidden");
    a.set_stereotype_display("label");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.raw(st, "visible").is_none());
}

#[test]
fn explorer_keys_and_settings() {
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let p = a.add_model(model, "UMLPackage");
    let c = a.add_model(p, "UMLClass");
    a.repo.set(c, "stereotype", json!("entity"));
    a.expanded.clear();
    a.expand_all();
    let rows = a.visible_rows();
    assert!(rows.contains(&c), "Expand All opens every level");
    a.expanded.clear();
    assert_eq!(a.visible_rows(), vec![a.repo.root], "Collapse All");
    a.expanded.insert(a.repo.root);
    a.focus = Some(model);
    a.tree_keys((false, false, false, true));
    assert!(a.expanded.contains(&model), "Right opens");
    a.tree_keys((false, false, false, true));
    let first = a.explorer_children(model)[0];
    assert_eq!(a.focus, Some(first), "Right again goes to the first child");
    a.tree_keys((false, false, true, false));
    assert_eq!(a.focus, Some(model), "Left goes to the parent");
    a.tree_keys((false, false, true, false));
    assert!(!a.expanded.contains(&model), "Left again closes");
    a.tree_keys((true, false, false, false));
    assert_eq!(a.focus, Some(a.repo.root), "Up");
    a.tree_keys((false, true, false, false));
    assert_eq!(a.focus, Some(model), "Down");
}

#[test]
fn preferences_and_working_diagrams_survive_a_restart() {
    let ms = measure();
    let dir = std::env::temp_dir().join(format!("umlcraft-prefs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let prefs_path = dir.join("preferences.json");
    let file = dir.join("model.mdj");
    std::fs::create_dir_all(&dir).unwrap();

    let mut a = App::new(None);
    a.prefs = crate::prefs::Prefs::load(Some(prefs_path.clone()));
    a.apply_prefs();
    assert!(a.show_grid && a.snap && a.show_stereotype);
    a.show_grid = false;
    a.sort_by_name = true;
    a.show_navigator = false;
    a.store_prefs();
    assert!(prefs_path.exists());
    let d1 = a.diagram.unwrap();
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    let d3 = a.add_diagram(None, "UMLUseCaseDiagram");
    a.close_diagram(Some(d1));
    a.open_diagram(d2);
    assert!(a.save_to(file.clone(), &ms));
    let (id2, id3) = (a.repo.id(d2), a.repo.id(d3));

    let mut b = App::new(None);
    b.prefs = crate::prefs::Prefs::load(Some(prefs_path.clone()));
    b.apply_prefs();
    assert!(!b.show_grid && b.sort_by_name && !b.show_navigator);
    b.open_path(file.clone());
    let ids: Vec<String> = b.open.iter().map(|d| b.repo.id(*d)).collect();
    assert!(ids.starts_with(&[id2.clone(), id3]), "the working diagrams come back in order: {ids:?}");
    assert_eq!(b.diagram.map(|d| b.repo.id(d)), Some(id2), "and the current one");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn every_default_diagram_opens() {
    let mut a = App::new(None);
    let d1 = a.diagram.unwrap();
    let d2 = a.add_diagram(None, "UMLUseCaseDiagram");
    a.repo.set(d2, "defaultDiagram", json!(true));
    let text = a.repo.serialize();
    let mut b = App::new(None);
    b.set_project(Repo::parse(&text).unwrap(), None);
    assert_eq!(b.open.len(), 2);
    assert_eq!(b.diagram.map(|d| b.repo.id(d)), Some(a.repo.id(d1)), "the first default is current");
}

#[test]
fn panel_toggles() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let mut press = |a: &mut App, key: Key, m: Modifiers| {
        let ev = egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: m };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    press(&mut a, Key::Num3, Modifiers::COMMAND);
    assert!(!a.show_toolbar);
    press(&mut a, Key::Num6, Modifiers::COMMAND);
    assert!(!a.show_editors);
    press(&mut a, Key::Num5, Modifiers::COMMAND);
    assert!(!a.show_toolbox);
    press(&mut a, Key::Num1, Modifiers::COMMAND | Modifiers::SHIFT);
    assert!(a.show_sidebar, "Ctrl+Shift+1 is a stereotype display, not the sidebar");
    press(&mut a, Key::Num1, Modifiers::COMMAND);
    assert!(!a.show_sidebar);
}

#[test]
fn preferences_set_new_view_defaults_and_the_grid() {
    let mut a = App::new(None);
    assert_eq!(a.grid_size(), 8.0);
    a.prefs.set("diagramEditor.gridSize", json!(10));
    assert_eq!(a.grid_size(), 10.0);
    a.prefs.set("view.lineColor", json!("#ff0000"));
    a.prefs.set("view.fontSize", json!(16));
    a.prefs.set("uml.class.fillColor", json!("#ffffcc"));
    a.prefs.set("uml.class.suppressOperations", json!(true));
    a.prefs.set("view.lineStyle", json!(0));
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLInterface", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    assert_eq!(a.repo.str(c1, "fillColor"), "#ffffcc");
    assert_eq!(a.repo.str(c2, "fillColor"), "#ffffff", "the general fill when the kind has none");
    assert_eq!(a.repo.str(c1, "lineColor"), "#ff0000");
    assert_eq!(a.repo.str(c1, "font"), "Arial;16;0");
    assert!(a.repo.bool(c1, "suppressOperations"));
    assert_eq!(a.repo.str(c2, "stereotypeDisplay"), "icon");
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    assert_eq!(a.repo.f(e, "lineStyle"), 0.0);
    // with the defaults nothing extra is written
    let mut b = App::new(None);
    let v = b.change(|b| b.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    for k in ["fillColor", "lineColor", "fontColor", "showShadow"] {
        assert!(!b.repo.has(v, k), "{k} left at its default");
    }
}

#[test]
fn export_jpeg_and_all_diagrams() {
    use crate::export::{filenamify, Format};
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let d = a.diagram.unwrap();
    assert_eq!(filenamify("a/b:c?"), "a!b!c!");
    assert_eq!(a.export_name(d, Format::Jpeg), format!("{}.jpg", a.repo.name(d)));
    let jpg = crate::export::render(&mut a, d, &ms, Format::Jpeg, 1.0).unwrap();
    let img = image::load_from_memory(&jpg).unwrap().to_rgb8();
    assert_eq!(img.get_pixel(0, 0).0, [255, 255, 255], "JPEG gets a white background");
    let dir = std::env::temp_dir().join(format!("umlcraft-export-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let n = a.export_all(&dir, &ms, Format::Svg, 1.0).unwrap();
    assert_eq!(n, a.repo.all_of("Diagram").len());
    let names: Vec<String> = std::fs::read_dir(dir.join("svg")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
    assert!(names.iter().any(|f| f.ends_with("_0.svg") && f.contains("!!")), "path joined with :: then made safe: {names:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recent_files() {
    let dir = std::env::temp_dir().join(format!("umlcraft-recent-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let ms = measure();
    let mut a = App::new(None);
    a.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    let f1 = dir.join("one.mdj");
    let f2 = dir.join("two.mdj");
    assert!(a.save_to(f1.clone(), &ms));
    assert!(a.save_to(f2.clone(), &ms));
    a.open_path(f1.clone());
    let r = a.recent_files();
    assert_eq!(r.len(), 2, "no duplicates");
    assert!(r[0].ends_with("one.mdj"), "most recent first");
    for i in 0..12 {
        a.add_recent(&dir.join(format!("f{i}.mdj")));
    }
    assert_eq!(a.recent_files().len(), 10);
    // the working file is the last one opened, not saved, and reopens at launch when asked to
    assert!(a.working_file().unwrap().ends_with("one.mdj"));
    a.prefs.set("general.working-file", json!(true));
    a.prefs.flush();
    let mut b = App::new(None);
    b.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    b.auto_load_working_file();
    assert!(b.filename.as_ref().unwrap().ends_with("one.mdj"));
    // New forgets it
    b.cmd_new(&ms);
    assert_eq!(b.working_file(), None);
    let mut c = App::new(None);
    c.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    c.auto_load_working_file();
    assert_eq!(c.filename, None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn package_diagrams_and_more_kinds() {
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let d = a.change(|a| a.add_diagram(Some(model), "UMLPackageDiagram"));
    assert_eq!(a.repo.name(d), "PackageDiagram1");
    let p1 = a.change(|a| a.create_node("UMLPackage", rect(40.0, 40.0, 140.0, 100.0))).unwrap();
    let p2 = a.change(|a| a.create_node("UMLPackage", rect(300.0, 40.0, 140.0, 100.0))).unwrap();
    let dep = a.change(|a| a.create_edge(&tool("UMLDependency", AssocInit::Plain), p1, p2));
    assert!(dep.is_some());
    let prof = a.add_model(model, "UMLProfile");
    assert_eq!(a.repo.name(prof), "Profile1");
    let st = a.add_model(prof, "UMLStereotype");
    assert_eq!(a.repo.name(st), "Stereotype1");
    // a stereotype of the profile is linked by name in inline expressions
    let c = a.add_model(model, "UMLClass");
    crate::parse::apply_name(&mut a.repo, c, "<<Stereotype1>> C");
    assert_eq!(a.repo.refd(c, "stereotype"), Some(st));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn tag_editor_rows_and_values() {
    use crate::tags::Row;
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let prof = a.add_model(model, "UMLProfile");
    let st = a.add_model(prof, "UMLStereotype");
    a.repo.set(st, "name", json!("entity"));
    let slot_b = a.add_feature(st, "UMLAttribute").unwrap();
    crate::parse::apply_attribute(&mut a.repo, slot_b, "table: string = items");
    let slot_a = a.add_feature(st, "UMLAttribute").unwrap();
    crate::parse::apply_attribute(&mut a.repo, slot_a, "audited: boolean = true");
    let c = a.add_model(model, "UMLClass");
    crate::parse::apply_name(&mut a.repo, c, "<<entity>> Item");
    a.focus = Some(c);
    a.open_tag_editor();
    assert_eq!(a.tag_editor, Some(c));
    // slots come first, sorted by name
    assert_eq!(a.tag_rows(c), vec![Row::Slot(slot_a), Row::Slot(slot_b)]);
    // editing a slot pre-fills from the attribute type
    a.edit_tag(Some(Row::Slot(slot_a)));
    let ed = a.tag_edit.clone().unwrap();
    assert_eq!((ed.name.as_str(), ed.kind.as_str(), ed.checked, ed.slot), ("audited", "boolean", true, true));
    a.apply_tag_edit(c, &ed);
    let t = a.repo.list(c, "tags")[0];
    assert_eq!(a.repo.str(t, "kind"), "boolean");
    assert!(a.repo.bool(t, "checked"));
    assert!(!a.repo.has(t, "value"), "only the fields of the kind are stored");
    assert_eq!(a.tag_rows(c)[0], Row::Tag(t));
    // a plain tag follows the slots
    a.edit_tag(None);
    let mut ed = a.tag_edit.clone().unwrap();
    ed.name = "author".into();
    ed.value = "Ann".into();
    a.apply_tag_edit(c, &ed);
    let rows = a.tag_rows(c);
    assert_eq!(rows.len(), 3);
    let Row::Tag(last) = rows[2].clone() else { panic!() };
    assert_eq!(a.tag_value_text(last), "Ann");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn command_palette_and_quick_find() {
    use crate::palette::{matching, Hit, Mode};
    use egui::{Key, Modifiers};
    assert!(matching("zoom").iter().all(|c| c.id.contains("zoom") || c.name.to_lowercase().contains("zoom")));
    assert!(matching("view:fit").iter().any(|c| c.name == "Fit To Window"), "the id matches too");
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let hits = a.find_hits("loan");
    let Hit::Model(m) = hits[0] else { panic!() };
    assert!(a.repo.name(m).to_lowercase().contains("loan"));
    assert!(hits.iter().any(|h| matches!(h, Hit::View(_))), "views follow their model");
    let view = hits.iter().find_map(|h| if let Hit::View(v) = h { Some(*v) } else { None }).unwrap();
    a.pick_hit(Hit::View(view));
    assert_eq!(a.selection, vec![view]);

    // through frames: Ctrl+Shift+P, type, Enter
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut frame = |a: &mut App, events: Vec<egui::Event>| {
        let mut out = ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
            if let Some(id) = a.palette_run.take() {
                a.run_command(ui.ctx(), id);
            }
            a.shortcuts(ui.ctx());
            a.palette_ui(ui.ctx());
        });
        out.textures_delta.clear();
    };
    let key = |k: Key, m: Modifiers| egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: m };
    frame(&mut a, vec![key(Key::P, Modifiers::COMMAND | Modifiers::SHIFT)]);
    assert_eq!(a.palette.as_ref().map(|p| p.mode), Some(Mode::Commands));
    frame(&mut a, vec![]);
    frame(&mut a, vec![egui::Event::Text("statusbar".into())]);
    frame(&mut a, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(a.palette.is_none());
    frame(&mut a, vec![]);
    assert!(!a.show_statusbar, "the picked command ran");
}

#[test]
fn model_find() {
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let found = a.find("loan", false, false);
    assert!(found.len() >= 2 && found.iter().all(|e| a.repo.name(*e).to_lowercase().contains("loan")));
    assert!(a.find("loan", true, false).is_empty(), "case sensitive");
    assert!(a.find("   ", false, false).is_empty(), "an empty keyword finds nothing");
    let m = found[0];
    a.change(|a| a.repo.set(m, "documentation", json!("a zebra")));
    assert!(a.find("zebra", false, false).is_empty());
    assert_eq!(a.find("zebra", false, true), vec![m], "in documentation");
    assert!(a.path_name(m).contains("::"));
}

#[test]
fn validation_rules() {
    use crate::validate::validate;
    let mut a = App::new(None);
    let model = a.repo.children(a.repo.root)[0].1;
    let rules = |a: &App| -> Vec<&'static str> { validate(&a.repo).into_iter().map(|p| p.rule).collect() };
    let ex = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj");
    let mut lib = App::new(None);
    lib.open_path(ex);
    assert!(lib.validated);
    // the example has a class and an actor both named Member in one package, which StarUML also reports
    assert!(lib.problems.iter().all(|p| p.rule == "UML002" && lib.repo.name(p.elem) == "Member"), "{:?}", lib.problems);
    assert_eq!(lib.problems.len(), 2);

    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let (m1, m2) = (a.repo.refd(c1, "model").unwrap(), a.repo.refd(c2, "model").unwrap());
    assert!(rules(&a).is_empty());
    a.repo.set(m2, "name", a.repo.raw(m1, "name").cloned().unwrap());
    assert!(rules(&a).contains(&"UML002"));
    a.repo.set(m2, "name", serde_json::json!(""));
    assert!(rules(&a).contains(&"UML001"));
    a.repo.set(m2, "name", serde_json::json!("Child"));
    let g = tool("UMLGeneralization", AssocInit::Plain);
    a.create_edge(&g, c2, c1);
    let at1 = a.add_feature(m1, "UMLAttribute").unwrap();
    let at2 = a.add_feature(m2, "UMLAttribute").unwrap();
    a.repo.set(at2, "name", a.repo.raw(at1, "name").cloned().unwrap());
    assert!(rules(&a).contains(&"UML003"));
    a.repo.set(m1, "isLeaf", serde_json::json!(true));
    assert!(rules(&a).contains(&"UML006"));
    a.create_edge(&g, c2, c1);
    assert!(rules(&a).contains(&"UML007"));
    a.create_edge(&g, c1, c2);
    assert!(rules(&a).contains(&"UML008"));
    let o1 = a.add_feature(m1, "UMLOperation").unwrap();
    let o2 = a.add_feature(m1, "UMLOperation").unwrap();
    crate::parse::apply_operation(&mut a.repo, o1, "+f(x: int)");
    crate::parse::apply_operation(&mut a.repo, o2, "-f(y: int)");
    assert!(!rules(&a).contains(&"UML004"), "different parameter names");
    crate::parse::apply_operation(&mut a.repo, o2, "-f(x: int)");
    assert!(rules(&a).contains(&"UML004"));
    let i = a.change(|a| a.create_node("UMLInterface", rect(40.0, 240.0, 100.0, 60.0))).unwrap();
    let im = a.repo.refd(i, "model").unwrap();
    let iop = a.add_feature(im, "UMLOperation").unwrap();
    a.repo.set(iop, "visibility", serde_json::json!("private"));
    assert!(rules(&a).contains(&"UML011"));
    let e = a.create_edge(&tool("UMLAssociation", AssocInit::Composite), c1, c2).unwrap();
    let am = a.repo.refd(e, "model").unwrap();
    a.repo.set(a.repo.child(am, "end1").unwrap(), "aggregation", serde_json::json!("shared"));
    assert!(rules(&a).contains(&"UML012"));
    let st = a.add_model(model, "UMLStereotype");
    assert!(rules(&a).contains(&"UML018"));
    let _ = st;
}

#[test]
fn add_related_elements() {
    use crate::related::actions;
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(200.0, 200.0, 100.0, 60.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    let acts = actions("UMLClass");
    let sub = acts.iter().find(|x| x.label == "Add Sub-Class").copied().unwrap();
    let s1 = a.add_related(c, sub).unwrap();
    let r1 = crate::views::node_rect(&a.repo, s1);
    assert_eq!(r1.y, 200.0 + 60.0 + 70.0, "70 px below");
    let gens: Vec<_> = a.repo.all_of("UMLGeneralization");
    assert_eq!(gens.len(), 1);
    assert_eq!(a.repo.refd(gens[0], "target"), Some(cm), "the new class generalizes the edited one");
    assert_eq!(a.pending_edit, Some(s1), "its name edit opens");
    // a second sub-class goes 10 px right of the first
    let s2 = a.add_related(c, sub).unwrap();
    let r2 = crate::views::node_rect(&a.repo, s2);
    assert_eq!((r2.x, r2.y), (r1.x + r1.w + 10.0, r1.y));
    let sup = acts.iter().find(|x| x.label == "Add Super-Class").copied().unwrap();
    let p = a.add_related(c, sup).unwrap();
    assert_eq!(crate::views::node_rect(&a.repo, p).y, (200.0f64 - 100.0 - crate::views::default_size("UMLClass").1).max(0.0));
    let agg = acts.iter().find(|x| x.label == "Add Aggregated Class").copied().unwrap();
    let part = a.add_related(c, agg).unwrap();
    assert!(crate::views::node_rect(&a.repo, part).x >= 300.0 + 70.0, "to the right");
    let assoc = a.repo.all_of("UMLAssociation")[0];
    let end2 = a.repo.child(assoc, "end2").unwrap();
    assert_eq!(a.repo.refd(end2, "reference"), Some(cm));
    assert_eq!(a.repo.str(end2, "aggregation"), "shared", "the edited class is the whole");
    let note = acts.iter().find(|x| x.label == "Add Note").copied().unwrap();
    let n = a.add_related(c, note).unwrap();
    assert_eq!(a.repo.ty(n), "UMLNoteView");
    let undo = a.undo.len();
    a.add_related(c, sub);
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    assert!(!actions("UMLUseCase").is_empty() && !actions("UMLActor").is_empty() && !actions("UMLPackage").is_empty());
}

#[test]
fn selector_language() {
    use crate::selector::select;
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let r = &a.repo;
    let diagrams = select(r, "@Diagram").unwrap();
    assert_eq!(diagrams.len(), r.all_of("Diagram").len());
    let classes = select(r, "Model::@UMLClass").unwrap();
    assert!(classes.len() >= 5 && classes.iter().all(|c| r.ty(*c) == "UMLClass"));
    let loan_ops = select(r, "Loan.operations").unwrap();
    assert_eq!(loan_ops.len(), 2);
    let concrete = select(r, "@UMLClass[isAbstract=true]").unwrap();
    assert_eq!(concrete.iter().map(|c| r.name(*c)).collect::<Vec<_>>(), vec!["LibraryItem".to_string()]);
    assert_eq!(select(r, "Model::Domain").unwrap().len(), 1);
    assert!(select(r, "Loan[name").is_err());
    assert!(select(r, "@").is_err());
}

#[test]
fn cli_help_and_version() {
    let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(crate::cli::run(&args(&["--version"]), egui::Context::default), Some(0));
    assert_eq!(crate::cli::run(&args(&["model.mdj"]), egui::Context::default), None, "a file opens the window");
    assert_eq!(crate::cli::run(&args(&["image", "/nope.mdj"]), egui::Context::default), Some(1));
}

#[test]
fn inline_edit_closes_on_diagram_switch() {
    let mut a = App::new(None);
    let d1 = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.edit_view(c, None);
    a.editing.as_mut().unwrap().text = "Renamed".into();
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    assert!(a.editing.is_none());
    assert_eq!(a.repo.name(m), "Renamed", "what was typed is kept");
    assert!(a.refocus_canvas);
    let _ = (d1, d2);
}

#[test]
fn auto_backup_and_recovery() {
    let dir = std::env::temp_dir().join(format!("umlcraft-backup-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut a = App::new(None);
    a.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0)));
    let t0 = a.last_backup;
    a.auto_backup(t0 + std::time::Duration::from_secs(5));
    assert!(!dir.join("__backup.mdj").exists(), "not before 10 minutes");
    a.auto_backup(t0 + crate::prefs::BACKUP_EVERY);
    assert!(dir.join("__backup.mdj").exists());
    // the app "crashes": a new one recovers the backup as an untitled, modified project
    let mut b = App::new(None);
    b.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    assert!(b.recover_backup());
    assert!(b.modified && b.filename.is_none());
    assert_eq!(b.repo.all_of("UMLClass").len(), 1);
    assert!(!dir.join("__backup.mdj").exists(), "the backup is deleted once loaded");
    assert!(!b.recover_backup(), "and offered once");
    // a clean exit clears it
    a.auto_backup(t0 + crate::prefs::BACKUP_EVERY * 2);
    a.clear_backup();
    let mut c = App::new(None);
    c.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    assert!(!c.recover_backup());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn template_parameters() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 160.0, 80.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let t = a.change(|a| a.add_feature(m, "UMLTemplateParameter")).unwrap();
    assert_eq!(a.repo.name(t), "TemplateParameter1");
    crate::parse::apply_template_parameter(&mut a.repo, t, "T: Number = int");
    assert_eq!((a.repo.name(t), a.repo.str(t, "parameterType"), a.repo.str(t, "defaultValue")), ("T".to_string(), "Number".to_string(), "int".to_string()));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let tc = a.repo.refd(c, "templateParameterCompartment").unwrap();
    let tb = crate::views::node_rect(&a.repo, tc);
    let cb = crate::views::node_rect(&a.repo, c);
    assert!(tb.y < cb.y && tb.x + tb.w > cb.x + cb.w, "over the top-right corner: {tb:?} {cb:?}");
    let row = a.repo.list(tc, "subViews")[0];
    assert_eq!(a.repo.str(row, "text"), "T: Number = int");
    // the export bounds include the box, and a double-click on it edits the parameter
    let b = crate::render::bounds(&a.repo, d).unwrap();
    assert!(b.y <= tb.y);
    let rb = crate::views::node_rect(&a.repo, row);
    a.edit_view(c, Some(crate::views::P { x: rb.x + 2.0, y: rb.y + 2.0 }));
    assert_eq!(a.editing.as_ref().map(|e| e.kind.clone()), Some(crate::app::EditKind::Feature(t)));
    a.editing.as_mut().unwrap().text = "K".into();
    a.close_inline(true);
    assert_eq!(a.repo.name(t), "K");
    assert_eq!(a.repo.str(t, "parameterType"), "");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn print_to_pdf() {
    use crate::pdf::PrintOptions;
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let all = a.print_diagrams(true);
    assert_eq!(all.len(), a.repo.all_of("Diagram").len());
    let o = PrintOptions::default();
    assert_eq!(o.page(), (841.89, 595.28), "A4 landscape by default");
    let pdf = a.print_pdf(&o, &ms).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.starts_with("%PDF-1.4"));
    assert!(text.trim_end().ends_with("%%EOF"));
    assert_eq!(text.matches("/Type /Page ").count(), all.len());
    assert!(text.contains(&format!("({})", a.path_name(all[0]))), "the diagram name is printed");
    let one = a.print_pdf(&PrintOptions { all: false, show_name: false, size: "letter".into(), landscape: false }, &ms).unwrap();
    let text = String::from_utf8_lossy(&one);
    assert_eq!(text.matches("/Type /Page ").count(), 1);
    assert!(text.contains("/MediaBox [0 0 612.00 792.00]"));
    assert!(!text.contains(" Tj "));
    assert!(a.pdf_of(&[], &o, &ms).is_err());
}

#[test]
fn word_wrap() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 80.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.repo.set(m, "name", json!("A Rather Long Class Name"));
    let at = a.change(|a| a.add_feature(m, "UMLAttribute")).unwrap();
    crate::parse::apply_attribute(&mut a.repo, at, "+firstAttribute: String = some default text");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let wide = crate::views::node_rect(&a.repo, c);
    let row = a.repo.list(a.repo.refd(c, "attributeCompartment").unwrap(), "subViews")[0];
    let line = a.repo.f(row, "height");
    // Ctrl+Shift+W: the box may be as narrow as the widest word, and the rows grow to several lines
    a.select(vec![c]);
    a.toggle_flag("wordWrap");
    assert!(a.repo.bool(c, "wordWrap"));
    a.change(|a| a.repo.set(c, "width", json!(80)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let narrow = crate::views::node_rect(&a.repo, c);
    assert!(narrow.w < wide.w, "{narrow:?} {wide:?}");
    assert!(a.repo.bool(row, "wordWrap"));
    assert!(a.repo.f(row, "height") >= 2.0 * line, "the attribute takes several lines");
    assert!(narrow.h > wide.h);
    let nl = a.repo.refd(a.repo.refd(c, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert!(a.repo.f(nl, "height") >= 2.0 * line);
    // the picture still draws
    crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap();
}

#[test]
fn layout_layers() {
    use crate::layout::{layered, Dir, SEPS};
    let (node_sep, rank_sep) = (SEPS.node, SEPS.rank);
    // a super-class above two sub-classes, side by side and centred under it
    let sizes = [(100.0, 50.0), (80.0, 40.0), (80.0, 40.0)];
    let p = layered(&sizes, &[(0, 1), (0, 2)], Dir::TopBottom, SEPS);
    assert_eq!(p.boxes[1].y, 50.0 + rank_sep);
    assert_eq!(p.boxes[1].y, p.boxes[2].y);
    assert_eq!((p.boxes[2].x - p.boxes[1].x).abs(), 80.0 + node_sep);
    let mid = (p.boxes[1].x + p.boxes[2].x + 80.0) / 2.0;
    assert!((p.boxes[0].x + 50.0 - mid).abs() <= 1.0, "{p:?}");
    // left to right: ranks become columns
    let q = layered(&sizes, &[(0, 1), (0, 2)], Dir::LeftRight, SEPS);
    assert_eq!(q.boxes[1].x, 100.0 + rank_sep);
    // bottom to top: the super-class is below
    let b = layered(&sizes, &[(0, 1), (0, 2)], Dir::BottomTop, SEPS);
    assert!(b.boxes[0].y > b.boxes[1].y);
    // a line over two ranks bends once; cycles do not hang
    let c = layered(&[(10.0, 10.0); 3], &[(0, 1), (1, 2), (0, 2), (2, 0)], Dir::TopBottom, SEPS);
    assert_eq!(c.bends[2].len(), 1);
    assert!(c.boxes.iter().all(|p| p.x >= 0.0 && p.y >= 0.0));
}

#[test]
fn layout_diagram_command() {
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let d = a.diagram.unwrap();
    let undo = a.undo.len();
    a.layout_diagram(crate::layout::Dir::TopBottom);
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // no two top-level boxes overlap
    let boxes: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|v| crate::views::is_node(&a.repo, *v) && !a.repo.has(*v, "containerView")).map(|v| crate::views::node_rect(&a.repo, v)).collect();
    for i in 0..boxes.len() {
        for j in i + 1..boxes.len() {
            let (p, q) = (boxes[i], boxes[j]);
            assert!(p.x + p.w <= q.x || q.x + q.w <= p.x || p.y + p.h <= q.y || q.y + q.h <= p.y, "{p:?} {q:?}");
        }
    }
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn diagram_generator() {
    use crate::generate::Generator;
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    // without a package selected, an alert
    a.focus = None;
    assert!(a.generate_diagram(Generator::Overview, &ms).is_none());
    assert_eq!(a.message.take().map(|m| m.0), Some("To generate Overview diagram, select a Package.".to_string()));
    let pkg = a.repo.all().find(|e| a.repo.ty(*e) == "UMLModel").unwrap();
    let classes = a.repo.list(pkg, "ownedElements").into_iter().filter(|e| a.repo.ty(*e) == "UMLClass").count();
    a.focus = Some(pkg);
    let undo = a.undo.len();
    let d = a.generate_diagram(Generator::Overview, &ms).unwrap();
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    assert_eq!(a.repo.name(d), "Overview");
    assert_eq!(a.diagram, Some(d), "opened");
    let vs = a.repo.list(d, "ownedViews");
    let class_views: Vec<crate::model::Eid> = vs.iter().copied().filter(|v| a.repo.ty(*v) == "UMLClassView").collect();
    assert_eq!(class_views.len(), classes);
    assert!(class_views.iter().all(|v| a.repo.bool(*v, "suppressAttributes")));
    let gens = a.repo.all().filter(|e| a.repo.ty(*e) == "UMLGeneralization").count();
    assert_eq!(vs.iter().filter(|v| a.repo.ty(**v) == "UMLGeneralizationView").count(), gens, "each relationship once");
    // type hierarchy: only generalizations and realizations
    a.focus = Some(pkg);
    let th = a.generate_diagram(Generator::TypeHierarchy, &ms).unwrap();
    let kinds: std::collections::HashSet<String> = a.repo.list(th, "ownedViews").into_iter().filter(|v| crate::views::is_edge(&a.repo, *v)).map(|v| a.repo.ty(v).to_string()).collect();
    assert!(kinds.iter().all(|k| k == "UMLGeneralizationView" || k == "UMLInterfaceRealizationView"), "{kinds:?}");
    // package structure: packages joined by containment lines
    a.focus = Some(pkg);
    let sub = a.change(|a| a.repo.create("UMLPackage", &[("name", json!("Sub"))], Some((pkg, "ownedElements"))));
    let _ = sub;
    a.focus = Some(pkg);
    let ps = a.generate_diagram(Generator::PackageStructure, &ms).unwrap();
    assert_eq!(a.repo.ty(ps), "UMLPackageDiagram");
    assert!(a.repo.list(ps, "ownedViews").iter().any(|v| a.repo.ty(*v) == "UMLContainmentView"));
    crate::export::render(&mut a, ps, &ms, crate::export::Format::Svg, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn element_picker_and_name_lookup() {
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let find = |a: &App, n: &str, t: &str| a.repo.all().find(|e| a.repo.name(*e) == n && a.repo.ty(*e) == t).unwrap();
    let book = find(&a, "Book", "UMLClass");
    let member = find(&a, "Member", "UMLClass");
    let at = a.change(|a| a.add_feature(book, "UMLAttribute")).unwrap();
    // a typed name that names a classifier becomes a reference; other text stays text
    a.commit_text(at, "type", "type", "Member".into());
    assert_eq!(a.repo.refd(at, "type"), Some(member));
    a.commit_text(at, "type", "type", "Nothing".into());
    assert_eq!(a.repo.str(at, "type"), "Nothing");
    // the picker starts on the current value; "Do not specify" clears it
    a.apply_picked(at, "type", Some(member));
    a.open_element_picker(at, "type");
    let p = a.element_picker.clone().unwrap();
    assert_eq!((p.chosen, p.ty), (Some(member), "UMLClassifier"));
    assert!(p.open.contains(&a.repo.parent(member).unwrap()), "owners expanded");
    a.apply_picked(at, "type", None);
    assert_eq!(a.repo.refd(at, "type"), None);
    assert_eq!(crate::picker::field_type("stereotype"), "UMLStereotype");
}

#[test]
fn rename_diagram_dialog() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let old = a.repo.name(d);
    a.rename_diagram();
    let dlg = a.input_dialog.clone().unwrap();
    assert_eq!((dlg.message.as_str(), dlg.text.clone()), ("Enter diagram name", old));
    let undo = a.undo.len();
    a.input_dialog = None;
    a.apply_input(dlg.action, "Overview".into());
    assert_eq!(a.repo.name(d), "Overview");
    assert_eq!(a.undo.len(), undo + 1);
    // the same name changes nothing
    a.apply_input(dlg.action, "Overview".into());
    assert_eq!(a.undo.len(), undo + 1);
}

#[test]
fn small_drags_keep_off_grid_values() {
    let mut a = App::new(None);
    a.snap = true;
    let g = a.grid_size();
    // an off-grid value stays while the drag stays in its cell, then snaps
    let o = 1.5 * g + 1.0;
    assert_eq!(a.snap_from(o, o + 0.3 * g), o);
    assert_eq!(a.snap_from(o, g + 0.5), o);
    assert_eq!(a.snap_from(o, 2.0 * g + 1.0), 2.0 * g);
    assert_eq!(a.snap_from(o, 0.9 * g), g);
    a.snap = false;
    assert_eq!(a.snap_from(13.0, 17.4), 17.0);
}

#[test]
fn broken_views_removed_on_open() {
    // the examples have none
    for f in std::fs::read_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples")).unwrap() {
        let p = f.unwrap().path();
        if p.extension().map(|e| e == "mdj").unwrap_or(false) {
            let mut a = App::new(None);
            a.open_path(p.clone());
            for d in a.repo.all_of("Diagram") {
                assert!(a.broken_views(d).is_empty(), "{p:?} {}", a.repo.name(d));
            }
        }
    }
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLGeneralization", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    let note = a.change(|a| a.create_node("Note", rect(40.0, 200.0, 100.0, 60.0))).unwrap();
    assert!(a.broken_views(d).is_empty());
    // the generalization loses its target: the line goes, the boxes and the note stay
    let g = a.repo.refd(e, "model").unwrap();
    a.repo.remove(g, "target");
    assert_eq!(a.broken_views(d), vec![e]);
    // a class view whose model is gone
    let m2 = a.repo.refd(c2, "model").unwrap();
    a.repo.detach(m2);
    let other = a.add_diagram(None, "UMLClassDiagram");
    assert_eq!(a.diagram, Some(other));
    a.open_diagram(d);
    assert!(!a.repo.alive(e) && !a.repo.alive(c2));
    assert!(a.repo.alive(c1) && a.repo.alive(note));
}

#[test]
fn relationships_panel_rows() {
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let find = |a: &App, n: &str, t: &str| a.repo.all().find(|e| a.repo.name(*e) == n && a.repo.ty(*e) == t).unwrap();
    let item = find(&a, "LibraryItem", "UMLClass");
    let book = find(&a, "Book", "UMLClass");
    let rows = a.relationship_rows(item);
    // Book generalizes LibraryItem: from LibraryItem, Book is the source
    assert!(rows.iter().any(|r| r.other == book && r.role == "(source)"), "{rows:?}");
    let rows = a.relationship_rows(book);
    assert!(rows.iter().any(|r| r.other == item && r.role == "(target)"));
    // an association lists the opposite end's role name
    let member = find(&a, "Member", "UMLClass");
    let rows = a.relationship_rows(member);
    assert!(rows.iter().any(|r| a.repo.ty(r.rel) == "UMLAssociation"), "{rows:?}");
}

#[test]
fn diagram_thumbnails() {
    use crate::thumbs::thumb_fit;
    let (s, at) = thumb_fit(Some(crate::views::R { x: 50.0, y: 40.0, w: 220.0, h: 60.0 }));
    assert_eq!((s, at.x, at.y), (0.5, 40.0, 30.0), "the bounds plus 20 fit 120 x 80");
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let d = a.repo.all_of("Diagram").into_iter().find(|d| a.repo.name(*d) == "Use Cases").unwrap();
    let owner = a.repo.name(a.repo.parent(d).unwrap());
    assert_eq!(a.thumb_label(d), format!("Use Cases (from {owner})"));
    a.scroll = crate::views::P { x: 300.0, y: 200.0 };
    a.open_from_thumb(d);
    assert_eq!((a.diagram, a.focus, a.scroll.x, a.scroll.y), (Some(d), Some(d), 0.0, 0.0));
}

#[test]
fn minimap_toggle_is_remembered() {
    let dir = std::env::temp_dir().join(format!("umlcraft-minimap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut a = App::new(None);
    a.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    a.apply_prefs();
    assert!(a.show_minimap, "on by default");
    a.show_minimap = false;
    a.store_prefs();
    let mut b = App::new(None);
    b.prefs = crate::prefs::Prefs::load(Some(dir.join("preferences.json")));
    b.apply_prefs();
    assert!(!b.show_minimap);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn markdown_blocks_and_spans() {
    use crate::markdown::{parse, spans, Block};
    let b = parse("# Title\n\nSome *text*\nand more.\n\n- one\n  - two\n1. first\n> quoted\n```\nlet x = 1;\n```\n---");
    assert_eq!(
        b,
        vec![
            Block::Heading(1, "Title".into()),
            Block::Paragraph("Some *text* and more.".into()),
            Block::Bullet(0, "one".into()),
            Block::Bullet(1, "two".into()),
            Block::Numbered(0, "1".into(), "first".into()),
            Block::Quote("quoted".into()),
            Block::Code("let x = 1;".into()),
            Block::Rule,
        ]
    );
    assert_eq!(
        spans("a **b** *c* `d` snake_case"),
        vec![
            ("a ".into(), false, false, false),
            ("b".into(), true, false, false),
            (" ".into(), false, false, false),
            ("c".into(), false, true, false),
            (" ".into(), false, false, false),
            ("d".into(), false, false, true),
            (" snake_case".into(), false, false, false),
        ]
    );
}

#[test]
fn html_docs_export() {
    use crate::html::{markdown_html, md5_hex};
    assert_eq!(md5_hex(""), "d41d8cd98f00b204e9800998ecf8427e");
    assert_eq!(md5_hex("The quick brown fox jumps over the lazy dog"), "9e107d9d372bb6826bd81d3542a419d6");
    assert_eq!(md5_hex(&"a".repeat(100)), "36a92cc94a9e0fa21f625f8bfb007adf");
    assert_eq!(markdown_html("# T\n\n- a **b**\n- c"), "<h1>T</h1><ul><li>a <strong>b</strong></li><li>c</li></ul>");
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let dir = std::env::temp_dir().join(format!("umlcraft-html-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    a.export_html(&dir, &ms).unwrap();
    for f in ["index.html", "contents/navigation.html", "contents/diagrams.html", "contents/element_index.html", "assets/style.css"] {
        assert!(dir.join(f).exists(), "{f}");
    }
    let d = a.repo.all_of("Diagram")[0];
    let id = md5_hex(&a.repo.id(d));
    assert!(dir.join(format!("diagrams/{id}.svg")).exists());
    let page = std::fs::read_to_string(dir.join(format!("contents/{id}.html"))).unwrap();
    assert!(page.contains(&format!("usemap=\"#map-{id}\"")) && page.contains("<area shape=\"rect\""));
    let book = a.repo.all().find(|e| a.repo.name(*e) == "Book" && a.repo.ty(*e) == "UMLClass").unwrap();
    let bp = std::fs::read_to_string(dir.join(format!("contents/{}.html", md5_hex(&a.repo.id(book))))).unwrap();
    assert!(bp.contains("Generalized Elements") && bp.contains("LibraryItem") && bp.contains("<h2>Attributes</h2>"));
    let index = std::fs::read_to_string(dir.join("contents/element_index.html")).unwrap();
    assert!(index.contains("<h2>UMLClass ("));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mermaid_class_diagrams() {
    use crate::mermaid::{diagram_type, member_text, parse_class_diagram, SAMPLE};
    assert_eq!(diagram_type("graph TD\nA-->B"), Err("Unsupported diagram type".to_string()));
    assert_eq!(diagram_type("stateDiagram-v2\n"), Ok("state"));
    assert_eq!(member_text("+int age"), ("+age: int".to_string(), false, false, false));
    assert_eq!(member_text("+eat(String food, int n) bool"), ("+eat(food: String, n: int): bool".to_string(), true, false, false));
    assert_eq!(member_text("-List~int~ ids$"), ("-ids: List<int>".to_string(), false, true, false));
    assert_eq!(member_text("draw()*"), ("~draw()".to_string(), true, false, true));
    let md = parse_class_diagram("classDiagram\ndirection LR\nnamespace Shapes {\n class Square~T~\n}\nclass Animal[\"An animal\"]{\n<<interface>>\n+move()\n}\nA \"1\" *-- \"many\" B : has\nC ..|> Animal\nnote for A \"hello\"").unwrap();
    assert_eq!(md.direction, "LR");
    let sq = md.classes.iter().find(|c| c.id == "Square").unwrap();
    assert_eq!((sq.namespace.as_deref(), sq.generics.clone()), (Some("Shapes"), vec!["T".to_string()]));
    let an = md.classes.iter().find(|c| c.id == "Animal").unwrap();
    assert_eq!((an.label.as_str(), an.annotations.clone(), an.members.clone()), ("An animal", vec!["interface".to_string()], vec!["+move()".to_string()]));
    let r = &md.relations[0];
    assert_eq!((r.a.as_str(), r.b.as_str(), r.end_a.as_str(), r.card_a.as_str(), r.card_b.as_str(), r.label.as_str()), ("A", "B", "*", "1", "many", "has"));
    assert_eq!(md.notes, vec![(Some("A".to_string()), "hello".to_string())]);

    let ms = measure();
    let mut a = App::new(None);
    a.focus = None;
    let undo = a.undo.len();
    let d = a.generate_mermaid(SAMPLE, &ms).unwrap();
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    assert_eq!(a.repo.name(d), "Class Diagram by Mermaid");
    assert_eq!(a.repo.name(a.repo.parent(d).unwrap()), "Package by Mermaid");
    let find = |a: &App, n: &str| a.repo.all().find(|e| a.repo.name(*e) == n && a.repo.ty(*e) == "UMLClass").unwrap();
    let (car, vehicle, garage) = (find(&a, "Car"), find(&a, "Vehicle"), find(&a, "Garage"));
    let g = a.repo.all().find(|e| a.repo.ty(*e) == "UMLGeneralization" && a.repo.refd(*e, "source") == Some(car)).unwrap();
    assert_eq!(a.repo.refd(g, "target"), Some(vehicle));
    let attrs: Vec<String> = a.repo.list(vehicle, "attributes").into_iter().map(|x| crate::views::attribute_text(&a.repo, x, crate::views::Show::ALL)).collect();
    assert_eq!(attrs, vec!["+wheels: int", "+brand: String"]);
    let asc = a.repo.all().find(|e| a.repo.ty(*e) == "UMLAssociation").unwrap();
    let e1 = a.repo.child(asc, "end1").unwrap();
    assert_eq!((a.repo.refd(e1, "reference"), a.repo.str(e1, "aggregation"), a.repo.str(e1, "multiplicity")), (Some(garage), "shared".to_string(), "1".to_string()));
    let views = a.repo.list(d, "ownedViews");
    assert_eq!(views.iter().filter(|v| a.repo.ty(**v) == "UMLClassView").count(), 4);
    assert_eq!(views.iter().filter(|v| crate::views::is_edge(&a.repo, **v)).count(), 3);
    assert!(a.generate_mermaid("sequenceDiagram\nA->>B: hi", &ms).is_err());
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn properties_of_several_elements() {
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let u = a.change(|a| a.create_node("UMLUseCase", rect(40.0, 300.0, 100.0, 60.0))).unwrap();
    let (m1, m2, mu) = (a.repo.refd(c1, "model").unwrap(), a.repo.refd(c2, "model").unwrap(), a.repo.refd(u, "model").unwrap());
    a.select(vec![c1, c2]);
    assert_eq!(a.prop_elements(), vec![m1, m2]);
    let both = a.common_fields(&[m1, m2]);
    assert!(both.contains(&("isActive", "bool")));
    let mixed = a.common_fields(&[m1, mu]);
    assert!(mixed.contains(&("name", "text")) && !mixed.contains(&("isActive", "bool")), "{mixed:?}");
    // a text committed to both, as one undo step
    let undo = a.undo.len();
    let t = vec![(m1, "stereotype".to_string()), (m2, "stereotype".to_string())];
    a.commit_all(&t, "text", "entity".into());
    assert_eq!(a.undo.len(), undo + 1);
    assert_eq!((crate::views::stereotype_text(&a.repo, m1), crate::views::stereotype_text(&a.repo, m2)), ("«entity»".to_string(), "«entity»".to_string()));
    // one view selected: the focused element alone
    a.select(vec![c1]);
    a.focus = Some(m1);
    assert_eq!(a.prop_elements(), vec![m1]);
}

#[test]
fn explorer_commands_in_the_palette() {
    let ctx = egui::Context::default();
    let mut a = App::new(None);
    assert!(crate::palette::matching("explorer:").len() >= 5);
    a.run_command(&ctx, "explorer:sort-by-name");
    assert!(a.sort_by_name);
    a.run_command(&ctx, "explorer:sort-by-added");
    assert!(!a.sort_by_name);
    a.run_command(&ctx, "explorer:expand-all");
    assert!(a.expanded.contains(&a.repo.root));
    a.run_command(&ctx, "explorer:collapse-all");
    assert!(a.expanded.is_empty());
}

#[test]
fn window_title_format() {
    let mut a = App::new(None);
    assert_eq!(a.title(), "UMLCraft");
    a.modified = true;
    assert_eq!(a.title(), "• UMLCraft");
    a.filename = Some(std::path::PathBuf::from("/tmp/x/model.mdj"));
    assert_eq!(a.title(), "• model.mdj — UMLCraft");
    a.modified = false;
    assert_eq!(a.title(), "model.mdj — UMLCraft");
}

#[test]
fn drop_into_a_container() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.change(|a| a.create_node("UMLPackage", rect(40.0, 40.0, 300.0, 200.0))).unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(500.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let owner = a.repo.parent(m);
    let vs = a.drop_model(m, crate::views::P { x: 100.0, y: 100.0 }).unwrap();
    let v = vs[0];
    assert_eq!(a.repo.refd(v, "containerView"), Some(pkg));
    assert_eq!(a.repo.parent(m), owner, "the model stays");
    // outside any container: none
    let vs = a.drop_model(m, crate::views::P { x: 700.0, y: 400.0 }).unwrap();
    assert_eq!(a.repo.refd(vs[0], "containerView"), None);
    let _ = d;
}

#[test]
fn light_theme_preference() {
    let mut a = App::new(None);
    assert_eq!(a.pref("theme.default").as_str(), Some("dark"), "dark by default");
    let dark_text = crate::ui::TEXT();
    a.prefs.set("theme.default", json!("light"));
    a.apply_prefs();
    assert!(crate::ui::light_theme());
    assert_ne!(crate::ui::TEXT(), dark_text);
    crate::ui::set_light_theme(false);
    assert_eq!(crate::ui::TEXT(), dark_text);
}

#[test]
fn new_from_template() {
    use crate::templates::{project, TEMPLATES};
    let mut a = App::new(None);
    a.set_project(project(&TEMPLATES[1]), None);
    let models: Vec<String> = a.repo.list(a.repo.root, "ownedElements").into_iter().map(|m| a.repo.name(m)).collect();
    assert_eq!(models, vec!["Use Case Model", "Analysis Model", "Design Model", "Implementation Model", "Deployment Model"]);
    let d = a.diagram.unwrap();
    assert_eq!((a.repo.ty(d), a.repo.name(d).as_str()), ("UMLUseCaseDiagram", "Use Cases"));
    assert_eq!(a.repo.all_of("Diagram").len(), 5);
    // the file reads back, component and deployment diagrams included
    let text = a.repo.serialize();
    assert!(text.contains("UMLComponentDiagram") && text.contains("UMLDeploymentDiagram"));
    Repo::parse(&text).unwrap();
    a.set_project(project(&TEMPLATES[0]), None);
    assert_eq!(a.repo.name(a.diagram.unwrap()), "Main");
}

#[test]
fn model_fragments() {
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let loan = a.repo.all().find(|e| a.repo.name(*e) == "Loan" && a.repo.ty(*e) == "UMLClass").unwrap();
    let text = a.fragment_text(loan);
    assert!(text.starts_with("{\n\t\"_type\": \"UMLClass\""), "{}", &text[..40]);
    let dir = std::env::temp_dir().join(format!("umlcraft-mfj-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("Fragment.mfj");
    std::fs::write(&f, &text).unwrap();
    let undo = a.undo.len();
    let e = a.import_fragment(&f).unwrap();
    assert_eq!(a.undo.len(), undo + 1);
    assert_eq!(a.repo.parent(e), Some(a.repo.root), "added to the project");
    assert_eq!(a.repo.name(e), "Loan");
    assert_ne!(a.repo.id(e), a.repo.id(loan), "new ids");
    assert_eq!(a.repo.list(e, "attributes").len(), a.repo.list(loan, "attributes").len());
    assert_eq!(a.focus, Some(e));
    std::fs::write(&f, "[1, 2]").unwrap();
    assert!(a.import_fragment(&f).is_err());
    Repo::parse(&a.repo.serialize()).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn explorer_rename_in_place() {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let book = a.repo.all().find(|e| a.repo.name(*e) == "Book" && a.repo.ty(*e) == "UMLClass").unwrap();
    a.reveal(book);
    a.last_area = crate::app::Area::Explorer;
    let mut frame = |a: &mut App, events: Vec<egui::Event>| {
        let mut out = ctx.run_ui(egui::RawInput { events, ..Default::default() }, |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| a.explorer_ui(ui));
        });
        out.textures_delta.clear();
    };
    let key = |k: egui::Key| egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE };
    frame(&mut a, vec![]);
    frame(&mut a, vec![key(egui::Key::Enter)]);
    assert_eq!(a.tree_rename.as_ref().map(|r| (r.0, r.1.clone())), Some((book, "Book".to_string())));
    frame(&mut a, vec![]);
    frame(&mut a, vec![egui::Event::Text("Novel".into())]);
    frame(&mut a, vec![key(egui::Key::Enter)]);
    assert!(a.tree_rename.is_none());
    assert_eq!(a.repo.name(book), "Novel", "the selected name was replaced");
    // Esc cancels
    a.start_tree_rename();
    frame(&mut a, vec![]);
    frame(&mut a, vec![egui::Event::Text("X".into())]);
    frame(&mut a, vec![key(egui::Key::Escape)]);
    assert!(a.tree_rename.is_none());
    assert_eq!(a.repo.name(book), "Novel");
}

#[test]
fn note_text_alignment() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let n = a.change(|a| a.create_node("Note", rect(40.0, 40.0, 200.0, 100.0))).unwrap();
    a.repo.set(n, "text", json!("hi"));
    let undo = a.undo.len();
    a.set_text_align(&[n], "horzAlign", 1);
    a.set_text_align(&[n], "vertAlign", 4);
    assert_eq!(a.undo.len(), undo + 2);
    let svg = String::from_utf8(crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap()).unwrap();
    let t = svg.lines().find(|l| l.contains(">hi<")).unwrap();
    assert!(t.contains("text-anchor=\"end\""), "right aligned: {t}");
}

#[test]
fn user_keymap() {
    use crate::keymap::{load, parse_stroke};
    use egui::{Key, Modifiers};
    assert_eq!(parse_stroke("cmdctrl-alt-g"), Some((Modifiers::COMMAND | Modifiers::ALT, Key::G)));
    assert_eq!(parse_stroke("Shift-F4"), Some((Modifiers::SHIFT, Key::F4)));
    assert_eq!(parse_stroke("cmdctrl--"), Some((Modifiers::COMMAND, Key::Minus)));
    assert_eq!(parse_stroke("hyper-x"), None);
    let dir = std::env::temp_dir().join(format!("umlcraft-keymap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("keymap.json"), r#"{"cmdctrl-alt-g": "view:statusbar", "bad key!": "x"}"#).unwrap();
    assert_eq!(load(&dir.join("keymap.json")).unwrap().len(), 1, "bad entries are skipped");
    let mut a = App::new(None);
    a.load_user_keymap(&dir);
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let ev = |m: Modifiers| egui::Event::Key { key: Key::G, physical_key: None, pressed: true, repeat: false, modifiers: m };
    let mut frame = |a: &mut App, m: Modifiers| {
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev(m)], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    let grid = a.show_grid;
    frame(&mut a, Modifiers::COMMAND | Modifiers::ALT);
    assert!(!a.show_statusbar, "the user binding ran");
    assert_eq!(a.show_grid, grid, "and the built-in Ctrl+G did not");
    // a bad file is ignored
    std::fs::write(dir.join("keymap.json"), "not json").unwrap();
    let mut b = App::new(None);
    b.load_user_keymap(&dir);
    assert!(b.user_keymap.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn dialogs_swallow_shortcuts() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let mut press = |a: &mut App| {
        let ev = egui::Event::Key { key: Key::G, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::COMMAND };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
        out.textures_delta.clear();
    };
    let grid = a.show_grid;
    a.rename_diagram();
    assert!(a.modal_open());
    press(&mut a);
    assert_eq!(a.show_grid, grid, "Ctrl+G does not reach the canvas behind the dialog");
    a.input_dialog = None;
    press(&mut a);
    assert_ne!(a.show_grid, grid);
}

#[test]
fn nudge_moves_contained_views() {
    let mut a = App::new(None);
    let pkg = a.change(|a| a.create_node("UMLPackage", rect(40.0, 40.0, 300.0, 200.0))).unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(500.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let v = a.drop_model(m, crate::views::P { x: 100.0, y: 100.0 }).unwrap()[0];
    assert_eq!(a.repo.refd(v, "containerView"), Some(pkg));
    let before = (a.repo.f(v, "left"), a.repo.f(pkg, "left"));
    a.select(vec![pkg, v]);
    let undo = a.undo.len();
    a.nudge_selection(3.0, 0.0);
    assert_eq!(a.undo.len(), undo + 1);
    let after = (a.repo.f(v, "left"), a.repo.f(pkg, "left"));
    assert_eq!(after.0 - before.0, after.1 - before.1, "the contained class moves with its package, once");
}

#[test]
fn core_validation_rules() {
    use crate::validate::validate;
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLGeneralization", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let ge = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    let g = a.repo.refd(ge, "model").unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let ae = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    let asc = a.repo.refd(ae, "model").unwrap();
    let rules = |a: &App| validate(&a.repo).into_iter().map(|p| p.rule).filter(|r| r.starts_with("CORE")).collect::<Vec<_>>();
    assert!(rules(&a).is_empty());
    a.repo.remove(g, "target");
    let end2 = a.repo.child(asc, "end2").unwrap();
    a.repo.remove(end2, "reference");
    let root = a.repo.root;
    a.repo.set(root, "name", json!(""));
    let mut got = rules(&a);
    got.sort();
    assert_eq!(got, vec!["CORE001", "CORE002", "CORE003"]);
}

#[test]
fn robustness_tools() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("Control", rect(40.0, 40.0, 50.0, 56.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    assert_eq!((a.repo.ty(m), a.repo.name(m).as_str()), ("UMLClass", "Control1"));
    assert_eq!(crate::views::stereotype_text(&a.repo, m), "«control»");
    assert!(a.repo.bool(v, "suppressAttributes") && a.repo.str(v, "stereotypeDisplay") == "icon");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info.get(&v).map(|i| i.shape), Some("control"));
    // shown as a box again with the label display
    a.repo.set(v, "stereotypeDisplay", json!("label"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info.get(&v).map(|i| i.shape), Some(""));
}

#[test]
fn left_resize_keeps_the_right_edge() {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 160.0, 100.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    a.repo.set(m, "name", json!("AQuiteLongClassName"));
    let mut frame = |a: &mut App| {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| a.canvas_ui(ui));
        });
        out.textures_delta.clear();
    };
    frame(&mut a);
    let o = crate::views::node_rect(&a.repo, v);
    // the left edge dragged far right, past the content's minimum width
    a.drag = Some(crate::app::Drag::Resize { handle: 7, start: crate::views::P { x: o.x, y: o.y }, orig: o, view: v, changed: true });
    a.repo.set(v, "left", json!(o.x + o.w - 20.0));
    a.repo.set(v, "width", json!(20));
    frame(&mut a);
    let b = crate::views::node_rect(&a.repo, v);
    assert!(b.w > 20.0, "grown back to the content");
    assert_eq!(b.x + b.w, o.x + o.w, "the right edge stays: {b:?} {o:?}");
}

#[test]
fn reorder_working_diagrams() {
    let mut a = App::new(None);
    let d1 = a.diagram.unwrap();
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    let d3 = a.add_diagram(None, "UMLUseCaseDiagram");
    assert_eq!(a.open, vec![d1, d2, d3]);
    a.move_working(d3, d1);
    assert_eq!(a.open, vec![d3, d1, d2]);
    a.move_working(d3, d2);
    assert_eq!(a.open, vec![d1, d2, d3]);
    assert_eq!(a.diagram, Some(d3), "the current diagram stays current");
}

#[test]
fn http_api() {
    use std::io::{Read, Write};
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let (st, all) = a.api_answer("/get_all_diagrams_info", &json!({}), &ms);
    let all: serde_json::Value = serde_json::from_str(&all).unwrap();
    assert_eq!(st, 200);
    assert_eq!(all.as_array().unwrap().len(), a.repo.all_of("Diagram").len());
    assert_eq!(all[0]["type"], "UMLClassDiagram");
    let (_, cur) = a.api_answer("/get_current_diagram_info", &json!({}), &ms);
    let cur: serde_json::Value = serde_json::from_str(&cur).unwrap();
    assert_eq!(cur["id"], a.repo.id(a.diagram.unwrap()));
    let (st, img) = a.api_answer("/get_diagram_image_by_id", &json!({"diagramId": cur["id"]}), &ms);
    assert_eq!(st, 200);
    assert!(img.contains("\"data\":\"iVBOR"), "a base64 PNG");
    let (st, err) = a.api_answer("/get_diagram_image_by_id", &json!({"diagramId": "nope"}), &ms);
    assert_eq!((st, err.contains("Diagram not found or invalid ID")), (500, true));
    let (st, _) = a.api_answer("/generate_diagram", &json!({}), &ms);
    assert_eq!(st, 400);
    let (st, ok) = a.api_answer("/generate_diagram", &json!({"code": "classDiagram\nA <|-- B"}), &ms);
    assert_eq!((st, ok.as_str()), (200, "{\"success\":true}"));

    // over a socket: the window answers while it polls
    let ctx = egui::Context::default();
    let port = 58000 + (std::process::id() % 1000) as u16;
    a.api_rx = crate::api::start(port, ctx.clone());
    assert!(a.api_rx.is_some());
    let client = std::thread::spawn(move || {
        let mut s = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
        let body = "{}";
        write!(s, "POST /get_current_diagram_info HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        out
    });
    let start = std::time::Instant::now();
    while !client.is_finished() && start.elapsed().as_secs() < 10 {
        a.api_poll(&ctx);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let out = client.join().unwrap();
    assert!(out.starts_with("HTTP/1.1 200 OK"), "{out}");
    assert!(out.contains("\"type\":\"UMLClassDiagram\""));
}

#[test]
fn add_template_parameter_from_the_inline_edit() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 160.0, 80.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    a.item_key(c, crate::app::EditKind::Name(m), crate::canvas::ItemKey::AddTemplateParameter);
    let t = a.repo.list(m, "templateParameters");
    assert_eq!(t.len(), 1);
    assert_eq!(a.focus, Some(t[0]));
}

#[test]
fn selection_change_focuses_the_canvas() {
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 160.0, 80.0))).unwrap();
    let mut frame = |a: &mut App| {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            egui::CentralPanel::default().show_inside(ui, |ui| a.canvas_ui(ui));
        });
        out.textures_delta.clear();
    };
    frame(&mut a);
    assert!(ctx.memory(|m| m.focused()).is_none());
    // a selection made elsewhere (Quick Find, the Find results…) gives the canvas the keyboard
    a.last_area = crate::app::Area::Canvas;
    a.select(vec![c]);
    frame(&mut a);
    frame(&mut a);
    assert!(ctx.memory(|m| m.focused()).is_some());
}

#[test]
fn component_and_deployment_diagrams() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.add_diagram(None, "UMLDeploymentDiagram");
    let comp = a.change(|a| a.create_node("UMLComponent", rect(40.0, 40.0, 160.0, 60.0))).unwrap();
    let art = a.change(|a| a.create_node("UMLArtifact", rect(300.0, 40.0, 80.0, 70.0))).unwrap();
    let n1 = a.change(|a| a.create_node("UMLNode", rect(40.0, 250.0, 160.0, 80.0))).unwrap();
    let n2 = a.change(|a| a.create_node("UMLNode", rect(300.0, 250.0, 160.0, 80.0))).unwrap();
    assert_eq!(a.repo.ty(comp), "UMLComponentView");
    assert_eq!(a.repo.name(a.repo.refd(n1, "model").unwrap()), "Node1");
    // the component and node defaults: compartments suppressed, the artifact as an icon
    assert!(a.repo.bool(comp, "suppressAttributes") && a.repo.bool(n1, "suppressOperations"));
    assert_eq!(a.repo.str(art, "stereotypeDisplay"), "icon");
    let t = |k: &'static str| crate::app::Tool { label: "", kind: k, icon: "", line: true, init: crate::app::AssocInit::Plain };
    assert!(a.can_connect(&t("UMLDeployment"), art, n1));
    assert!(!a.can_connect(&t("UMLDeployment"), n1, art));
    assert!(a.can_connect(&t("UMLCommunicationPath"), n1, n2));
    assert!(!a.can_connect(&t("UMLCommunicationPath"), comp, n2));
    assert!(a.can_connect(&t("UMLComponentRealization"), art, comp));
    let dep = a.change(|a| a.create_edge(&t("UMLDeployment"), art, n1)).unwrap();
    let path = a.change(|a| a.create_edge(&t("UMLCommunicationPath"), n1, n2)).unwrap();
    let pm = a.repo.refd(path, "model").unwrap();
    assert_eq!(a.repo.ty(pm), "UMLCommunicationPath");
    assert!(a.repo.child(pm, "end1").is_some() && a.repo.child(pm, "end2").is_some());
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info.get(&art).map(|i| i.shape), Some("artifact"));
    let st = a.repo.refd(dep, "stereotypeLabel").unwrap();
    assert_eq!(a.repo.str(st, "text"), "«deploy»");
    let svg = String::from_utf8(crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap()).unwrap();
    assert!(svg.contains("Node1") && svg.contains("Artifact1"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn deployment_quick_edit_buttons() {
    let mut a = App::new(None);
    a.add_diagram(None, "UMLDeploymentDiagram");
    let n = a.change(|a| a.create_node("UMLNode", rect(100.0, 100.0, 160.0, 80.0))).unwrap();
    let acts = crate::related::actions("UMLNode");
    let deployed = acts.iter().find(|x| x.label == "Add Deployed Artifact").copied().unwrap();
    let v = a.add_related(n, deployed).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    assert_eq!(a.repo.ty(m), "UMLArtifact");
    let dep = a.repo.all().find(|e| a.repo.ty(*e) == "UMLDeployment").unwrap();
    assert_eq!((a.repo.refd(dep, "source"), a.repo.refd(dep, "target")), (Some(m), a.repo.refd(n, "model")));
    assert!(crate::views::node_rect(&a.repo, v).y > 180.0, "below the node");
    let path = acts.iter().find(|x| x.label == "Add Communicating Node").copied().unwrap();
    a.add_related(n, path).unwrap();
    assert!(a.repo.all().any(|e| a.repo.ty(e) == "UMLCommunicationPath"));
}

#[test]
fn subsystems() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let s = a.change(|a| a.create_node("UMLSubsystem", rect(40.0, 40.0, 300.0, 200.0))).unwrap();
    assert_eq!(a.repo.ty(s), "UMLSubsystemView");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info.get(&s).map(|i| i.shape), Some("package"));
    let c = a.change(|a| a.create_node("UMLClass", rect(500.0, 40.0, 100.0, 60.0))).unwrap();
    assert!(a.can_contain_view(s, c), "it holds boxes like a package");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn deleting_a_box_deletes_its_contained_views() {
    let mut a = App::new(None);
    let pkg = a.change(|a| a.create_node("UMLPackage", rect(40.0, 40.0, 300.0, 200.0))).unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(500.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let inner = a.drop_model(m, crate::views::P { x: 100.0, y: 100.0 }).unwrap()[0];
    assert_eq!(a.repo.refd(inner, "containerView"), Some(pkg));
    a.select(vec![pkg]);
    a.delete_selection(false);
    assert!(!a.repo.alive(pkg) && !a.repo.alive(inner), "the contained view goes too");
    assert!(a.repo.alive(c) && a.repo.alive(m), "other views and the models stay");
}

#[test]
fn text_box() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("TextBox", rect(40.0, 40.0, 120.0, 50.0))).unwrap();
    assert_eq!(a.repo.ty(v), "UMLTextBoxView");
    assert!(a.repo.kind_of(v, "UMLCustomTextView"), "aligned like notes and texts");
    a.edit_view(v, None);
    a.editing.as_mut().unwrap().text = "boxed".into();
    a.close_inline(true);
    assert_eq!(a.repo.str(v, "text"), "boxed");
    let svg = String::from_utf8(crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap()).unwrap();
    assert!(svg.contains(">boxed<") && svg.contains("stroke=\"#000000\""), "{svg}");
    assert!(a.broken_views(d).is_empty());
}

#[test]
fn add_note_to_a_line() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let undo = a.undo.len();
    let n = a.add_note_to_edge(e).unwrap();
    assert_eq!(a.undo.len(), undo + 1);
    let b = crate::views::node_rect(&a.repo, n);
    assert_eq!((b.w, b.h), (100.0, 50.0));
    let link = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLNoteLinkView").unwrap();
    assert_eq!((a.repo.refd(link, "tail"), a.repo.refd(link, "head")), (Some(n), Some(e)));
    // a class's Add Note is 100×50 too
    let note = crate::related::actions("UMLClass").into_iter().find(|x| x.kind == "Note").unwrap();
    let n2 = a.add_related(c1, note).unwrap();
    assert_eq!(crate::views::node_rect(&a.repo, n2).w, 100.0);
}

#[test]
fn ownership_dot() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let m1 = a.repo.refd(c1, "model").unwrap();
    let at = a.change(|a| a.add_feature(m1, "UMLAttribute")).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    let asc = a.repo.refd(e, "model").unwrap();
    let end2 = a.repo.child(asc, "end2").unwrap();
    let before = String::from_utf8(crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap()).unwrap();
    a.repo.set_ref(end2, "ownerAttribute", Some(at));
    let after = String::from_utf8(crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap()).unwrap();
    assert_eq!(after.matches("<ellipse").count(), before.matches("<ellipse").count() + 1, "one dot");
}

#[test]
fn derived_association_name() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    a.repo.set(m, "name", json!("owns"));
    a.repo.set(m, "isDerived", json!(true));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(e, "nameLabel").unwrap();
    assert!(a.repo.str(nl, "text").ends_with("/owns"), "{}", a.repo.str(nl, "text"));
}

#[test]
fn colour_dialog_applies_on_ok_only() {
    use crate::render::{parse_color, Rgba};
    use crate::style::{hex, palette, parse_hex};
    let def = Rgba(1, 1, 1, 255);
    assert_eq!(parse_color("#ff000080", def), Rgba(255, 0, 0, 128));
    assert_eq!(parse_color("#f00", def), Rgba(255, 0, 0, 255));
    assert_eq!(parse_color("rgba(255, 255, 255, 0)", def), Rgba(255, 255, 255, 0));
    assert_eq!(parse_color("rgb(1,2,3)", def), Rgba(1, 2, 3, 255));
    assert_eq!(parse_color("#zzzzzz", def), def);
    assert_eq!(hex(Rgba(255, 255, 255, 0)), "#ffffff00");
    assert_eq!(parse_hex("00ff00"), Some(Rgba(0, 255, 0, 255)));
    assert_eq!(parse_hex("#00ff00"), parse_hex("00FF00"));
    assert_eq!(parse_hex("#00ff0"), None);
    assert_eq!(palette().len(), 80);
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    a.select(vec![c]);
    a.open_color_dialog("fillColor");
    let d = a.color_dialog.as_mut().unwrap();
    assert_eq!((d.color, d.text.as_str()), (Rgba(255, 255, 255, 255), "#ffffff"));
    d.color = Rgba(255, 255, 255, 0);
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let key = |k| egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE };
    // Esc cancels: nothing changes
    let mut out = ctx.run_ui(egui::RawInput { events: vec![key(egui::Key::Escape)], ..Default::default() }, |ui| a.dialogs(ui.ctx()));
    out.textures_delta.clear();
    assert!(a.color_dialog.is_none());
    assert_eq!(a.repo.str(c, "fillColor"), "#ffffff");
    a.open_color_dialog("fillColor");
    a.color_dialog.as_mut().unwrap().color = Rgba(255, 255, 255, 0);
    let undo = a.undo.len();
    let mut out = ctx.run_ui(egui::RawInput { events: vec![key(egui::Key::Enter)], ..Default::default() }, |ui| a.dialogs(ui.ctx()));
    out.textures_delta.clear();
    assert!(a.color_dialog.is_none());
    assert_eq!(a.repo.str(c, "fillColor"), "#ffffff00", "Transparent is written with its alpha");
    assert_eq!(a.undo.len(), undo + 1);
}

#[test]
fn undo_keeps_100_operations() {
    let mut a = App::new(None);
    for i in 0..120 {
        a.change(|a| a.create_node("UMLClass", rect(10.0 + i as f64, 10.0, 50.0, 40.0)));
    }
    assert_eq!(a.undo.len(), 100);
}

#[test]
fn old_projects_send_subjects_to_the_back() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    a.change(|a| a.create_node("UMLUseCase", rect(40.0, 40.0, 100.0, 40.0)));
    let s = a.change(|a| a.create_node("UMLUseCaseSubject", rect(10.0, 10.0, 300.0, 300.0))).unwrap();
    // put the subject in front, as an old file could have it, and drop the version
    let mut vs = a.repo.list(d, "ownedViews");
    vs.retain(|v| *v != s);
    vs.push(s);
    a.repo.set_list(d, "ownedViews", vs);
    let mut v: Value = serde_json::from_str(&a.repo.serialize()).unwrap();
    v.as_object_mut().unwrap().remove("documentVersion");
    let sid = a.repo.id(s).to_string();
    let mut b = App::new(None);
    b.set_project(Repo::parse(&v.to_string()).unwrap(), None);
    let d = b.diagram.unwrap();
    assert_eq!(b.repo.id(b.repo.list(d, "ownedViews")[0]), sid);
    assert_eq!(b.repo.f(b.repo.root, "documentVersion"), 1.0);
    assert!(!b.modified);
    // a current project keeps its order
    let mut v2: Value = serde_json::from_str(&a.repo.serialize()).unwrap();
    v2["documentVersion"] = json!(1);
    let mut c = App::new(None);
    c.set_project(Repo::parse(&v2.to_string()).unwrap(), None);
    let d = c.diagram.unwrap();
    assert_eq!(c.repo.id(*c.repo.list(d, "ownedViews").last().unwrap()), sid);
}

#[test]
fn splitters_resize_and_collapse_panels() {
    use crate::splitter::{drag_result, Split};
    assert_eq!(drag_result(200.0, -195.0, 0.0, 900.0, true), Split::Hide);
    assert_eq!(drag_result(0.0, 150.0, 0.0, 900.0, true), Split::Resize(150.0));
    assert_eq!(drag_result(400.0, -390.0, 60.0, 700.0, false), Split::Resize(60.0), "the Toolbox stops at 60 px");
    assert_eq!(drag_result(400.0, 900.0, 60.0, 700.0, false), Split::Resize(700.0));

    let mut a = App::new(None);
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
    let mut frame = |a: &mut App, events: Vec<egui::Event>| {
        let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), events, ..Default::default() }, |ui| a.window_ui(ui));
        out.textures_delta.clear();
    };
    let button = |x: f32, y: f32, pressed: bool| egui::Event::PointerButton { pos: egui::pos2(x, y), button: egui::PointerButton::Primary, pressed, modifiers: Default::default() };
    let moved = |x: f32, y: f32| egui::Event::PointerMoved(egui::pos2(x, y));
    let mut drag = |a: &mut App, from: (f32, f32), to: (f32, f32)| {
        frame(a, vec![moved(from.0, from.1)]);
        frame(a, vec![button(from.0, from.1, true)]);
        for i in 1..=4 {
            let t = i as f32 / 4.0;
            frame(a, vec![moved(from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)]);
        }
        frame(a, vec![button(to.0, to.1, false)]);
        frame(a, vec![]);
    };
    frame(&mut a, vec![]);
    // the sidebar's right edge
    drag(&mut a, (200.0, 300.0), (260.0, 300.0));
    assert_eq!(a.panel_size("sidebar"), 260.0);
    assert!(a.show_sidebar);
    // below 10 px it hides, and its handle at the window edge brings it back
    drag(&mut a, (260.0, 300.0), (5.0, 300.0));
    assert!(!a.show_sidebar);
    drag(&mut a, (2.0, 300.0), (150.0, 300.0));
    assert!(a.show_sidebar);
    assert!((a.panel_size("sidebar") - 148.0).abs() < 4.0, "{}", a.panel_size("sidebar"));
    // the sizes are stored with the preferences
    a.store_prefs();
    assert_eq!(a.prefs.get("_viewState.sidebar.size").and_then(|v| v.as_f64()), Some(a.panel_size("sidebar").round() as f64));
}

#[test]
fn explorer_drop_acceptance() {
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let class = a.repo.all_of("UMLClass")[0];
    assert!(a.can_drop_model(class));
    let attr = a.repo.all_of("UMLAttribute")[0];
    assert!(!a.can_drop_model(attr), "an attribute has no box of its own");
    let assoc = a.repo.all_of("UMLAssociation")[0];
    assert!(a.can_drop_model(assoc));
}

#[test]
fn containment_moves_the_element_into_the_package() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let owner = a.repo.parent(d).unwrap();
    let pv = a.change(|a| a.create_node("UMLPackage", rect(40.0, 40.0, 200.0, 120.0))).unwrap();
    let cv = a.change(|a| a.create_node("UMLClass", rect(300.0, 240.0, 100.0, 60.0))).unwrap();
    let (pm, cm) = (a.repo.refd(pv, "model").unwrap(), a.repo.refd(cv, "model").unwrap());
    assert_eq!(a.repo.parent(cm), Some(owner));
    let t = tool("Containment", AssocInit::Plain);
    assert!(a.can_connect(&t, cv, pv));
    assert!(!a.can_connect(&t, pv, cv), "a class cannot contain a package");
    let e = a.change(|a| a.create_edge(&t, cv, pv)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLContainmentView");
    assert_eq!(a.repo.refd(e, "model"), None);
    assert_eq!(a.repo.parent(cm), Some(pm));
    assert_eq!(a.edge_kind(e), "Containment");
    // a package cannot be put inside itself through its own child
    let sub = a.change(|a| a.create_node("UMLPackage", rect(300.0, 400.0, 100.0, 60.0))).unwrap();
    a.change(|a| a.create_edge(&t, sub, pv)).unwrap();
    assert!(!a.can_connect(&t, pv, sub));
    // Add Sub-Package
    let buttons = crate::related::actions("UMLPackage");
    let b = buttons.into_iter().find(|b| b.label == "Add Sub-Package").unwrap();
    let n = a.add_related(pv, b).unwrap();
    let nm = a.repo.refd(n, "model").unwrap();
    assert_eq!(a.repo.parent(nm), Some(pm));
    assert!(a.repo.list(d, "ownedViews").iter().filter(|v| a.repo.ty(**v) == "UMLContainmentView").count() == 3);
}

#[test]
fn association_class_tool() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(340.0, 40.0, 100.0, 60.0))).unwrap();
    let t = tool("AssociationClass", AssocInit::Plain);
    assert!(a.can_connect(&t, c1, c2));
    let undo = a.undo.len();
    let cv = a.change(|a| a.create_edge(&t, c1, c2)).unwrap();
    assert_eq!(a.undo.len(), undo + 1);
    let cm = a.repo.refd(cv, "model").unwrap();
    assert_eq!((a.repo.ty(cm), a.repo.name(cm).as_str()), ("UMLClass", "AssociationClass"));
    let r = crate::views::node_rect(&a.repo, cv);
    assert_eq!(r.y, 100.0, "30 px below the middle of the line");
    let links = a.repo.all_of("UMLAssociationClassLink");
    assert_eq!(links.len(), 1);
    let assoc = a.repo.refd(links[0], "associationSide").unwrap();
    assert_eq!(a.repo.ty(assoc), "UMLAssociation");
    assert_eq!(a.repo.refd(links[0], "classSide"), Some(cm));
    let lv = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLAssociationClassLinkView").unwrap();
    assert_eq!(a.edge_kind(lv), "AssociationClass");
    let av = a.repo.refd(lv, "head").unwrap();
    // an association that already has its class cannot take a second one
    let c3 = a.change(|a| a.create_node("UMLClass", rect(40.0, 300.0, 100.0, 60.0))).unwrap();
    assert!(!a.can_connect(&t, c3, av));
    // from a class to a plain association: only the link
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c3)).unwrap();
    assert!(a.can_connect(&t, c2, e));
    let l2 = a.change(|a| a.create_edge(&t, c2, e)).unwrap();
    assert_eq!(a.repo.ty(l2), "UMLAssociationClassLinkView");
    assert_eq!(a.repo.all_of("UMLAssociationClassLink").len(), 2);
    // the link ends at the association's middle
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    let pts = crate::views::points_of(&a.repo, lv);
    let apts = crate::views::points_of(&a.repo, av);
    let mid = crate::views::edge_middle(&apts);
    let end = *pts.last().unwrap();
    assert!((end.x - mid.x).abs() < 2.0 && (end.y - mid.y).abs() < 2.0, "{end:?} {mid:?}");
    // it survives a save
    let text = a.repo.serialize();
    assert!(text.contains("\"_type\": \"UMLAssociationClassLinkView\""));
}

#[test]
fn template_binding_tool() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let bound = a.change(|a| a.create_node("UMLClass", rect(40.0, 240.0, 100.0, 60.0))).unwrap();
    let template = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let t = tool("UMLTemplateBinding", AssocInit::Plain);
    assert!(a.can_connect(&t, bound, template));
    assert!(!a.can_connect(&t, bound, bound));
    let e = a.change(|a| a.create_edge(&t, bound, template)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLTemplateBindingView");
    let m = a.repo.refd(e, "model").unwrap();
    assert_eq!(a.repo.refd(m, "source"), a.repo.refd(bound, "model"));
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    let st = a.repo.refd(e, "stereotypeLabel").unwrap();
    assert_eq!(a.repo.str(st, "text"), "«bind»");
}

#[test]
fn receptions_show_when_not_suppressed() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    assert!(a.can_add_feature(m, "UMLReception"));
    let rcp = a.change(|a| a.add_feature(m, "UMLReception")).unwrap();
    assert_eq!(a.repo.list(m, "receptions"), vec![rcp]);
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    let rc = a.repo.refd(v, "receptionCompartment").unwrap();
    assert!(!a.repo.bool(rc, "visible"), "suppressed by default");
    a.select(vec![v]);
    a.toggle_flag("suppressReceptions");
    let h0 = a.repo.f(v, "height");
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.bool(rc, "visible"));
    let rows = a.repo.list(rc, "subViews");
    assert_eq!(rows.len(), 1);
    assert_eq!(a.repo.str(rows[0], "text"), format!("«signal» +{}", a.repo.name(rcp)));
    assert!(a.repo.f(v, "height") >= h0);
    // Ctrl+Shift+E toggles it back
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let ev = egui::Event::Key { key: egui::Key::E, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::COMMAND | egui::Modifiers::SHIFT };
    let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.shortcuts(ui.ctx()));
    out.textures_delta.clear();
    assert!(a.repo.bool(v, "suppressReceptions"));
}

#[test]
fn reception_signal_is_a_reference() {
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let s = a.change(|a| a.create_node("UMLSignal", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let (m, sm) = (a.repo.refd(v, "model").unwrap(), a.repo.refd(s, "model").unwrap());
    let rc = a.change(|a| a.add_feature(m, "UMLReception")).unwrap();
    assert!(a.fields_for(rc).contains(&("signal", "type")));
    let name = a.repo.name(sm);
    a.commit_text(rc, "signal", "type", name);
    assert_eq!(a.repo.refd(rc, "signal"), Some(sm));
}

#[test]
fn free_lines() {
    use crate::views::P;
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let e = a.change(|a| a.create_freeline(P { x: 40.0, y: 40.0 }, P { x: 200.0, y: 120.0 })).unwrap();
    assert_eq!(a.repo.ty(e), "FreelineEdgeView");
    assert!(crate::views::is_edge(&a.repo, e));
    assert!(a.broken_views(d).is_empty(), "a free line has no ends and is not broken");
    // layout keeps its points
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(crate::views::points_of(&a.repo, e), vec![P { x: 40.0, y: 40.0 }, P { x: 200.0, y: 120.0 }]);
    // arrow keys move it
    a.select(vec![e]);
    a.nudge_selection(1.0, 0.0);
    assert_eq!(crate::views::points_of(&a.repo, e)[0], P { x: 41.0, y: 40.0 });
    // pattern and ends are stored and drawn
    a.repo.set(e, "lineMode", json!(2));
    a.repo.set(e, "headEndStyle", json!(2));
    let png = crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap();
    let svg = String::from_utf8(png).unwrap();
    assert!(svg.matches("<polyline").count() > 5, "dashes are drawn as pieces");
    assert!(svg.contains("<polygon"), "a solid arrow head");
    // saved and read back
    let text = a.repo.serialize();
    let b = Repo::parse(&text).unwrap();
    let fe = b.all_of("FreelineEdgeView");
    assert_eq!(fe.len(), 1);
    assert_eq!(b.raw(fe[0], "lineMode"), Some(&json!(2)));
}

#[test]
fn constraints() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("UMLClass", rect(200.0, 100.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    // Add Constraint: owned by the element, to the left of its box, with a link
    let b = crate::related::actions("UMLClass").into_iter().find(|b| b.label == "Add Constraint").unwrap();
    let undo = a.undo.len();
    let cv = a.add_related(v, b).unwrap();
    assert_eq!(a.undo.len(), undo + 1);
    assert_eq!(a.repo.ty(cv), "UMLConstraintView");
    let c = a.repo.refd(cv, "model").unwrap();
    assert_eq!((a.repo.ty(c), a.repo.parent(c)), ("UMLConstraint", Some(m)));
    let r = crate::views::node_rect(&a.repo, cv);
    assert!(r.x + r.w <= 200.0 && r.y == 100.0, "{r:?}");
    let links: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|x| a.repo.ty(*x) == "UMLConstraintLinkView").collect();
    assert_eq!(links.len(), 1);
    assert_eq!(a.edge_kind(links[0]), "ConstraintLink");
    assert!(a.broken_views(d).is_empty());
    assert_eq!(a.pending_edit, Some(cv));
    // the edit sets the specification, drawn in braces
    a.pending_edit = None;
    a.edit_view(cv, None);
    a.editing.as_mut().unwrap().text = "x > 0".into();
    a.close_inline(true);
    assert_eq!(a.repo.str(c, "specification"), "x > 0");
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(cv, "text"), "{x > 0}");
    // dropping it from the explorer onto another diagram links it to the owner's box there
    let d2 = a.add_diagram(None, "UMLClassDiagram");
    a.drop_model(m, crate::views::P { x: 300.0, y: 100.0 }).unwrap();
    assert!(a.can_drop_model(c));
    let vs = a.drop_model(c, crate::views::P { x: 40.0, y: 40.0 }).unwrap();
    assert_eq!(a.repo.ty(vs[0]), "UMLConstraintView");
    assert_eq!(a.repo.list(d2, "ownedViews").iter().filter(|x| a.repo.ty(**x) == "UMLConstraintLinkView").count(), 1);
}

#[test]
fn use_case_extension_points() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let ms = measure();
    let mut a = App::new(None);
    let d = a.add_diagram(None, "UMLUseCaseDiagram");
    let v = a.change(|a| a.create_node("UMLUseCase", rect(40.0, 40.0, 120.0, 40.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let h0 = a.repo.f(v, "height");
    // Ctrl+Enter on the name adds an extension point and edits it
    a.item_key(v, EditKind::Name(m), ItemKey::Add);
    let xp = a.repo.list(m, "extensionPoints")[0];
    assert_eq!(a.repo.ty(xp), "UMLExtensionPoint");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.f(v, "height") > h0, "the ellipse grows");
    assert_eq!(a.info.get(&v).unwrap().compartments.len(), 1);
    // "name: location" sets both
    a.edit_item(v, xp);
    a.editing.as_mut().unwrap().text = "Help: after login".into();
    a.close_inline(true);
    assert_eq!((a.repo.name(xp).as_str(), a.repo.str(xp, "location").as_str()), ("Help", "after login"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let row = a.info.get(&v).unwrap().compartments[0].rows[0].iv;
    assert_eq!(a.repo.str(row, "text"), "Help: after login");
}
