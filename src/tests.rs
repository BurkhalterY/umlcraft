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

    // fit: bounding box 1000..1600 x 1000..1400 fits at 1.0 and is centred (on the laid-out
    // views, as each frame lays the diagram out first)
    let ms = measure();
    crate::views::layout_diagram(&mut a.repo, &ms, a.diagram.unwrap());
    a.fit();
    assert_eq!(a.zoom, 1.0);
    let (x, y) = centre(&a);
    let b = crate::render::bounds(&a.repo, a.diagram.unwrap()).unwrap();
    assert!((x - (b.x + b.w / 2.0)).abs() < 1e-6 && (y - (b.y + b.h / 2.0)).abs() < 1e-6);
    // a box larger than the window zooms out
    a.change(|a| a.create_node("UMLClass", rect(3000.0, 1000.0, 200.0, 100.0)));
    crate::views::layout_diagram(&mut a.repo, &ms, a.diagram.unwrap());
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
    // after dragging d3 to the front, Next still follows the open order until a save
    a.open = vec![d3, d1, d2];
    a.cycle_diagram(1);
    assert_eq!(a.diagram, Some(d3), "d2 → d3 in open order");
    let dir = std::env::temp_dir().join(format!("umlcraft-order-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    assert!(a.save_to(dir.join("o.mdj"), &measure()));
    a.cycle_diagram(1);
    assert_eq!(a.diagram, Some(d1), "after saving, the displayed order: d3 → d1");
    let _ = std::fs::remove_dir_all(&dir);
    a.open = vec![d1, d2, d3];
    a.open_order = a.open.clone();
    a.open_diagram(d2);
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
    assert_eq!(a.selection, vec![l], "the label itself is selected");
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

/// Checks a PDF's structure (every cross-reference offset lands on its object, every stream has
/// its length) and returns its objects' bodies, streams inflated, by number.
fn pdf_objects(pdf: &[u8]) -> std::collections::BTreeMap<usize, Vec<u8>> {
    use std::io::Read;
    let text = String::from_utf8_lossy(pdf);
    assert!(text.starts_with("%PDF-1.4"));
    assert!(text.trim_end().ends_with("%%EOF"));
    let start: usize = text.rsplit("startxref\n").next().unwrap().lines().next().unwrap().trim().parse().unwrap();
    assert!(pdf[start..].starts_with(b"xref\n0 "), "startxref points at the table");
    let table = String::from_utf8(pdf[start..].to_vec()).unwrap();
    let count: usize = table.lines().nth(1).unwrap().split(' ').nth(1).unwrap().parse().unwrap();
    assert!(table.contains(&format!("/Size {count}")));
    let mut out = std::collections::BTreeMap::new();
    for (i, line) in table.lines().skip(3).take(count - 1).enumerate() {
        let n = i + 1;
        assert!(line.ends_with(" 00000 n "), "{line}");
        let off: usize = line[..10].parse().unwrap();
        let head = format!("{n} 0 obj\n");
        assert!(pdf[off..].starts_with(head.as_bytes()), "object {n} is at its offset");
        let body_start = off + head.len();
        let mut body = pdf[body_start..].to_vec();
        if let Some(s) = body.windows(7).position(|w| w == b"stream\n").filter(|s| !body[..*s].windows(6).any(|w| w == b"endobj")) {
            let dict = String::from_utf8_lossy(&body[..s]).to_string();
            let len: usize = dict.split("/Length ").nth(1).unwrap().split(|c: char| !c.is_ascii_digit()).next().unwrap().parse().unwrap();
            let data = body[s + 7..s + 7 + len].to_vec();
            assert!(body[s + 7 + len..].starts_with(b"\nendstream\nendobj\n"), "stream {n} has its length");
            body = if dict.contains("/FlateDecode") {
                let mut inflated = vec![];
                flate2::read::ZlibDecoder::new(&data[..]).read_to_end(&mut inflated).unwrap();
                [dict.as_bytes(), b"stream\n", &inflated].concat()
            } else {
                [dict.as_bytes(), b"stream\n", &data].concat()
            };
        } else {
            let end = body.windows(8).position(|w| w == b"\nendobj\n").unwrap();
            body.truncate(end);
        }
        out.insert(n, body);
    }
    out
}

/// The text a reader gets from each page of a PDF: each font's glyphs through its ToUnicode map,
/// a line per text object.
fn pdf_text(pdf: &[u8]) -> Vec<String> {
    let objs = pdf_objects(pdf);
    let s = |n: usize| String::from_utf8_lossy(&objs[&n]).to_string();
    let refnum = |t: &str, key: &str| -> Option<usize> { t.split(key).nth(1)?.trim().split(' ').next()?.parse().ok() };
    let mut pages = vec![];
    for body in objs.values() {
        let t = String::from_utf8_lossy(body).to_string();
        if !t.starts_with("<< /Type /Page ") {
            continue;
        }
        let res = s(refnum(&t, "/Resources ").unwrap());
        let fonts = res.split("/Font << ").nth(1).unwrap().split(">>").next().unwrap().to_string();
        let mut maps: std::collections::HashMap<String, std::collections::HashMap<String, String>> = Default::default();
        let words: Vec<&str> = fonts.split_whitespace().collect();
        for w in words.chunks(4) {
            let font = s(w[1].parse().unwrap());
            let cmap = s(refnum(&font, "/ToUnicode ").unwrap());
            let mut m = std::collections::HashMap::new();
            for l in cmap.lines().filter(|l| l.starts_with('<') && !l.starts_with("<<") && !l.starts_with("<0000> <FFFF>")) {
                let parts: Vec<&str> = l.split(['<', '>', ' ']).filter(|x| !x.is_empty()).collect();
                let units: Vec<u16> = parts[1].as_bytes().chunks(4).map(|c| u16::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap()).collect();
                m.insert(parts[0].to_string(), String::from_utf16(&units).unwrap());
            }
            maps.insert(w[0].trim_start_matches('/').to_string(), m);
        }
        let content = s(refnum(&t, "/Contents ").unwrap());
        let mut text = String::new();
        let mut font = String::new();
        let tokens: Vec<&str> = content.split_whitespace().collect();
        for (i, tok) in tokens.iter().enumerate() {
            if *tok == "Tf" {
                font = tokens[i - 2].trim_start_matches('/').to_string();
            } else if *tok == "Tj" {
                let hex = tokens[i - 1].trim_matches(['<', '>']);
                for g in hex.as_bytes().chunks(4) {
                    text += &maps[&font][std::str::from_utf8(g).unwrap()];
                }
            } else if *tok == "ET" {
                text.push('\n');
            }
        }
        pages.push(text);
    }
    pages
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
    assert_eq!(text.matches("/Type /Page ").count(), all.len());
    let pages = pdf_text(&pdf);
    assert_eq!(pages.len(), all.len());
    assert!(pages[0].lines().any(|l| l == a.path_name(all[0])), "the diagram name is printed");
    assert!(pages[0].lines().any(|l| l == "LibraryItem"), "the diagram's text is text: {}", pages[0]);
    let one = a.print_pdf(&PrintOptions { all: false, show_name: false, size: "letter".into(), landscape: false }, &ms).unwrap();
    let text = String::from_utf8_lossy(&one);
    assert_eq!(text.matches("/Type /Page ").count(), 1);
    assert!(text.contains("/MediaBox [0 0 612.00 792.00]"));
    let d = a.diagram.unwrap();
    assert!(!pdf_text(&one)[0].lines().any(|l| l == a.path_name(d)), "no name");
    assert!(a.pdf_of(&[], &o, &ms).is_err());
}

#[test]
fn pdf_is_vector_with_embedded_fonts() {
    use crate::pdf::PrintOptions;
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let pdf = a.print_pdf(&PrintOptions::default(), &ms).unwrap();
    let objs = pdf_objects(&pdf);
    let all: String = objs.values().map(|b| String::from_utf8_lossy(b).to_string()).collect();
    assert!(!all.contains("/DCTDecode") && !all.contains("/Subtype /Image"), "no picture of the diagram");
    assert!(all.contains("/Subtype /Type0") && all.contains("/Encoding /Identity-H") && all.contains("/CIDFontType2"));
    assert!(all.contains("/FontFile2 "), "the fonts are embedded");
    assert!(!all.contains("/Helvetica"), "no unembedded standard font");
    // the font files are subsets that still parse
    let file = objs.values().find(|b| b.starts_with(b"<< /Length1 ")).unwrap();
    let data = &file[file.windows(7).position(|w| w == b"stream\n").unwrap() + 7..];
    let face = ttf_parser::Face::parse(data, 0).expect("a font file that parses");
    assert!(face.tables().glyf.is_some() && face.number_of_glyphs() > 100);
    assert!(data.len() < 200_000, "a subset ({} bytes)", data.len());
    // lines, boxes, dashes and curves are drawn as paths
    assert!(all.contains(" re "));
    assert!(all.contains(" l S\n"));
    assert!(all.contains("[3] 0 d"), "the note's dashed link");
    assert!(all.contains(" c h"), "the interface's circle");
    assert!(all.contains("/ExtGState << /GS0 "), "the shadows' opacity");
}

#[test]
fn pdf_text_beyond_latin1() {
    use crate::pdf::PrintOptions;
    let ms = measure();
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 220.0, 100.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.change(|a| a.repo.set(m, "name", json!("Café Привет Ωμέγα")));
    let pdf = a.print_pdf(&PrintOptions { all: false, ..PrintOptions::default() }, &ms).unwrap();
    let page = &pdf_text(&pdf)[0];
    assert!(page.lines().any(|l| l == "Café Привет Ωμέγα"), "{page}");
}

#[test]
fn pdf_pictures() {
    use crate::pdf::PrintOptions;
    let ms = measure();
    let dir = std::env::temp_dir().join(format!("umlcraft-pdf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("half.png");
    image::RgbaImage::from_pixel(8, 4, image::Rgba([255, 0, 0, 128])).save(&png).unwrap();
    let jpg = dir.join("blue.jpg");
    image::RgbImage::from_pixel(8, 4, image::Rgb([0, 0, 255])).save(&jpg).unwrap();
    let mut a = App::new(None);
    a.drop_image_file(&png, crate::views::P { x: 100.0, y: 100.0 }).unwrap();
    a.drop_image_file(&jpg, crate::views::P { x: 200.0, y: 100.0 }).unwrap();
    let pdf = a.print_pdf(&PrintOptions { all: false, ..PrintOptions::default() }, &ms).unwrap();
    let all: String = pdf_objects(&pdf).values().map(|b| String::from_utf8_lossy(b).to_string()).collect();
    assert_eq!(all.matches("/Subtype /Image").count(), 3, "the PNG, its alpha mask and the JPEG");
    assert!(all.contains("/SMask "), "the PNG keeps its transparency");
    assert_eq!(all.matches("/DCTDecode").count(), 1, "the JPEG as it is");
    assert_eq!(all.matches(" Do Q").count(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn export_box_includes_sub_views() {
    let ms = measure();
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let mut beyond = 0;
    for d in a.repo.all_of("Diagram") {
        crate::views::layout_diagram(&mut a.repo, &ms, d);
        let b = crate::render::bounds(&a.repo, d).unwrap();
        for v in a.repo.list(d, "ownedViews") {
            if crate::views::is_edge(&a.repo, v) {
                continue;
            }
            let top = crate::views::node_rect(&a.repo, v);
            let mut stack = a.repo.list(v, "subViews");
            while let Some(s) = stack.pop() {
                if a.repo.raw(s, "visible") == Some(&json!(false)) {
                    continue;
                }
                stack.extend(a.repo.list(s, "subViews"));
                let r = crate::views::node_rect(&a.repo, s);
                if r.w <= 0.0 || (a.repo.ty(s).ends_with("LabelView") && a.repo.str(s, "text").is_empty()) {
                    continue;
                }
                assert!(r.x >= b.x && r.y >= b.y && r.x + r.w <= b.x + b.w + 1e-6 && r.y + r.h <= b.y + b.h + 1e-6, "{} inside the box", a.repo.ty(s));
                if r.y + r.h > top.y + top.h || r.x < top.x || r.x + r.w > top.x + top.w {
                    beyond += 1;
                }
            }
        }
    }
    assert!(beyond > 0, "some sub-view sticks out of its view (the interface's name under its circle)");
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
    let book = a.repo.all().find(|e| a.repo.name(*e) == "Book" && a.repo.ty(*e) == "UMLClass").unwrap();
    let rc = a.add_feature(book, "UMLReception").unwrap();
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
    let bp = std::fs::read_to_string(dir.join(format!("contents/{}.html", md5_hex(&a.repo.id(book))))).unwrap();
    assert!(bp.contains("Generalized Elements") && bp.contains("LibraryItem") && bp.contains("<h2>Attributes</h2>"));
    assert!(bp.contains("<h2>Receptions</h2>") && bp.contains(&a.repo.name(rc)));
    let index = std::fs::read_to_string(dir.join("contents/element_index.html")).unwrap();
    assert!(index.contains("<h2>UMLClass ("));
    // the generic sections: properties, tags, relationships and the diagrams that show it
    assert!(bp.contains("<h2>Properties</h2>") && bp.contains("<td>isAbstract</td><td><span class=\"label\">false</span></td>"), "a default value as a label");
    assert!(bp.contains("<h2>Relationships</h2>") && bp.contains("<h2>Diagrams</h2>"));
    let _ = std::fs::remove_dir_all(&dir);
    let t = a.change(|a| a.repo.create("Tag", &[("name", json!("reviewed")), ("kind", json!("boolean")), ("checked", json!(true))], Some((book, "tags"))));
    let lib = a.repo.all().find(|e| a.repo.name(*e) == "LibraryItem").unwrap();
    let dep = a.change(|a| { let (s, t) = (a.repo.ref_to(book), a.repo.ref_to(lib)); a.repo.create("UMLDependency", &[("source", s), ("target", t)], Some((book, "ownedElements"))) });
    a.export_html(&dir, &ms).unwrap();
    let bp = std::fs::read_to_string(dir.join(format!("contents/{}.html", md5_hex(&a.repo.id(book))))).unwrap();
    assert!(bp.contains("<h2>Tags</h2>") && bp.contains("<td>reviewed</td><td><span class=\"label\">boolean</span></td><td><span class=\"label\">true</span></td>"));
    assert!(bp.contains("<h2>Dependencies</h2>"));
    let lp = std::fs::read_to_string(dir.join(format!("contents/{}.html", md5_hex(&a.repo.id(lib))))).unwrap();
    assert!(lp.contains("<h2>Dependants</h2>"));
    let index = std::fs::read_to_string(dir.join("contents/element_index.html")).unwrap();
    assert!(index.contains("<h2>Tag (1)</h2>") && index.contains("reviewed"), "tags are in the index under Tag");
    let _ = (t, dep);
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
    assert_eq!(models, vec!["Use Case Model", "Analysis Model", "Design Model", "Implementation Model", "Deployment Model", "UMLStandardProfile"]);
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
    // the reception's inline edit has Select Signal, which opens the Element Picker on UMLSignal
    use crate::app::EditKind;
    use crate::canvas::{item_buttons, ItemKey};
    assert!(item_buttons(&a.repo, None, &EditKind::Feature(rc)).iter().any(|b| b.3 == ItemKey::SelectSignal));
    a.item_key(v, EditKind::Feature(rc), ItemKey::SelectSignal);
    let p = a.element_picker.as_ref().unwrap();
    assert_eq!((p.obj, p.field.as_str(), p.ty, p.chosen), (rc, "signal", "UMLSignal", Some(sm)));
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

#[test]
fn object_diagrams() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let ms = measure();
    let mut a = App::new(None);
    let cls = a.change(|a| a.create_node("UMLClass", rect(40.0, 300.0, 100.0, 60.0))).unwrap();
    let cm = a.repo.refd(cls, "model").unwrap();
    a.repo.set(cm, "name", json!("Book"));
    let d = a.add_diagram(None, "UMLObjectDiagram");
    let o1 = a.change(|a| a.create_node("UMLObject", rect(40.0, 40.0, 100.0, 40.0))).unwrap();
    let o2 = a.change(|a| a.create_node("UMLObject", rect(300.0, 40.0, 100.0, 40.0))).unwrap();
    assert_eq!(a.repo.ty(o1), "UMLObjectView");
    let m1 = a.repo.refd(o1, "model").unwrap();
    // "name : Class" sets the classifier, and the name is drawn underlined
    a.edit_view(o1, None);
    a.editing.as_mut().unwrap().text = "b1 : Book".into();
    a.close_inline(true);
    assert_eq!((a.repo.name(m1).as_str(), a.repo.refd(m1, "classifier")), ("b1", Some(cm)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(a.repo.refd(o1, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "b1 : Book");
    assert!(a.repo.bool(nl, "underline"));
    // Ctrl+Enter adds a slot, edited as "name = value"
    a.item_key(o1, EditKind::Name(m1), ItemKey::Add);
    let slot = a.repo.list(m1, "slots")[0];
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    a.edit_item(o1, slot);
    a.editing.as_mut().unwrap().text = "title : String = \"Dune\"".into();
    a.close_inline(true);
    assert_eq!((a.repo.name(slot).as_str(), a.repo.str(slot, "value").as_str()), ("title", "\"Dune\""));
    assert_eq!(crate::views::slot_text(&a.repo, slot), "title: String = \"Dune\"");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info.get(&o1).unwrap().compartments[0].rows.len(), 1);
    // links join objects only
    let t = tool("UMLLink", AssocInit::Directed);
    assert!(a.can_connect(&t, o1, o2));
    let e = a.change(|a| a.create_edge(&t, o1, o2)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLLinkView");
    let lm = a.repo.refd(e, "model").unwrap();
    let end2 = a.repo.child(lm, "end2").unwrap();
    assert_eq!((a.repo.ty(end2), a.repo.str(end2, "navigable").as_str()), ("UMLLinkEnd", "navigable"));
    // its ends show role names and multiplicities, like an association's
    a.repo.set(end2, "name", json!("copy"));
    a.repo.set(end2, "multiplicity", json!("1"));
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    let role = a.repo.refd(e, "headRoleNameLabel").unwrap();
    assert_eq!(a.repo.str(role, "text"), "+copy");
    assert!(a.repo.refd(e, "headQualifiersCompartment").is_none());
    assert!(!a.fields_for(lm).iter().any(|f| f.0.ends_with("ownerAttribute")));
    assert!(a.broken_views(d).is_empty());
    // Add Linked Object: a new object to the right, with a link
    let b = crate::related::actions("UMLObject").into_iter().find(|b| b.label == "Add Linked Object").unwrap();
    let o3 = a.add_related(o2, b).unwrap();
    assert_eq!(a.repo.ty(o3), "UMLObjectView");
    assert!(crate::views::node_rect(&a.repo, o3).x >= 300.0 + 100.0 + 70.0);
    assert_eq!(a.repo.all_of("UMLLink").len(), 2);
    // it all reads back
    let text = a.repo.serialize();
    let b = Repo::parse(&text).unwrap();
    assert_eq!(b.all_of("UMLSlot").len(), 1);
    assert_eq!(b.all_of("UMLObjectDiagram").len(), 1);
}

#[test]
fn state_machine_and_activity_rules() {
    use crate::model::Eid;
    use crate::validate::validate;
    let mut r = Repo::blank();
    let root = r.root;
    let model = r.children(root)[0].1;
    let sm = r.create("UMLStateMachine", &[("name", json!("SM"))], Some((model, "ownedElements")));
    let region = r.create("UMLRegion", &[], Some((sm, "regions")));
    let v = |r: &mut Repo, ty: &str, kind: &str| {
        let props: Vec<(&str, Value)> = if kind.is_empty() { vec![("name", json!("S"))] } else { vec![("kind", json!(kind))] };
        r.create(ty, &props, Some((region, "vertices")))
    };
    let init = v(&mut r, "UMLPseudostate", "initial");
    let init2 = v(&mut r, "UMLPseudostate", "initial");
    let s1 = v(&mut r, "UMLState", "");
    let s2 = v(&mut r, "UMLState", "");
    let fin = v(&mut r, "UMLFinalState", "");
    let fork = v(&mut r, "UMLPseudostate", "fork");
    let tr = |r: &mut Repo, a: Eid, b: Eid, guard: &str| {
        let (sa, sb) = (r.ref_to(a), r.ref_to(b));
        r.create("UMLTransition", &[("source", sa), ("target", sb), ("guard", json!(guard))], Some((region, "transitions")))
    };
    tr(&mut r, init, s1, "x > 0");
    tr(&mut r, init, s2, "");
    tr(&mut r, fin, s1, "");
    tr(&mut r, s1, fork, "");
    tr(&mut r, fork, fin, "");
    let _ = init2;
    // an activity with a merge of mixed flows and a final node with an exit
    let act = r.create("UMLActivity", &[("name", json!("A"))], Some((model, "ownedElements")));
    let node = |r: &mut Repo, ty: &str| r.create(ty, &[], Some((act, "nodes")));
    let (merge, a1, a2, fnode) = (node(&mut r, "UMLMergeNode"), node(&mut r, "UMLAction"), node(&mut r, "UMLAction"), node(&mut r, "UMLActivityFinalNode"));
    let flow = |r: &mut Repo, ty: &str, a: Eid, b: Eid| {
        let (sa, sb) = (r.ref_to(a), r.ref_to(b));
        r.create(ty, &[("source", sa), ("target", sb)], Some((act, "edges")))
    };
    flow(&mut r, "UMLControlFlow", a1, merge);
    flow(&mut r, "UMLObjectFlow", merge, a2);
    flow(&mut r, "UMLObjectFlow", a1, a2);
    flow(&mut r, "UMLControlFlow", fnode, a1);
    let rules: Vec<&str> = validate(&r).into_iter().map(|p| p.rule).collect();
    for want in ["UML021", "UML022", "UML025", "UML030", "UML033", "UML041", "UML047", "UML050", "UML057"] {
        assert!(rules.contains(&want), "{want} in {rules:?}");
    }
    assert!(!rules.contains(&"UML046"), "the merge has one outgoing edge");
}

#[test]
fn uml_standard_profile() {
    let mut a = App::new(None);
    let undo = a.undo.len();
    a.apply_standard_profile();
    assert_eq!(a.undo.len(), undo + 1);
    let p = a.repo.all().find(|e| a.repo.ty(*e) == "UMLProfile").unwrap();
    assert_eq!(a.repo.name(p), "UMLStandardProfile");
    assert_eq!(a.repo.parent(p), Some(a.repo.root));
    let owned: Vec<_> = a.repo.list(p, "ownedElements");
    let sts: Vec<_> = owned.iter().copied().filter(|e| a.repo.ty(*e) == "UMLStereotype").collect();
    assert_eq!(sts.len(), 36);
    assert_eq!(owned.iter().filter(|e| a.repo.ty(**e) == "UMLMetaClass").count(), 9, "with the metaclasses");
    assert_eq!(a.repo.ty(owned[0]), "UMLMetaClass", "in the profile's order");
    // each stereotype extends its metaclasses; «Create» two of them
    let create = sts.iter().copied().find(|s| a.repo.name(*s) == "Create").unwrap();
    let exts: Vec<String> = a.repo.list(create, "ownedElements").into_iter().filter(|x| a.repo.ty(*x) == "UMLExtension").filter_map(|x| a.repo.refd(x, "target")).map(|m| a.repo.name(m)).collect();
    assert_eq!(exts, ["UMLBehavioralFeature", "UMLDependency"]);
    assert!(a.repo.child(create, "icon").is_some(), "each stereotype has its icon object");
    // the file kinds generalize File, and the stereotypes pass UML017 and UML018
    let lib = sts.iter().copied().find(|s| a.repo.name(*s) == "Library").unwrap();
    let g = a.repo.list(lib, "ownedElements")[0];
    assert_eq!(a.repo.refd(g, "target").map(|f| a.repo.name(f)), Some("File".to_string()));
    let rules: Vec<&str> = crate::validate::validate(&a.repo).into_iter().map(|p| p.rule).collect();
    assert!(!rules.contains(&"UML017") && !rules.contains(&"UML018"), "{rules:?}");
    // a second time: refused, nothing changes
    a.apply_standard_profile();
    assert_eq!(a.undo.len(), undo + 1);
    assert_eq!(a.repo.all().filter(|e| a.repo.ty(*e) == "UMLProfile").count(), 1);
    assert!(a.toasts.last().map(|t| t.2).unwrap_or(false), "an error toast");
    // the robustness tools find their stereotypes there, without asking
    assert!(a.robustness_profile_ready("Boundary") && a.robustness_profile_ready("UMLClass"));
    let v = a.change(|a| a.create_node("Control", rect(40.0, 40.0, 60.0, 60.0))).unwrap();
    let st = a.repo.refd(a.repo.refd(v, "model").unwrap(), "stereotype").unwrap();
    assert_eq!(a.repo.parent(st), Some(p));
    // templates include it
    let t = crate::templates::project(&crate::templates::TEMPLATES[0]);
    assert!(t.all().any(|e| t.ty(e) == "UMLProfile"));
}

#[test]
fn xmi_export() {
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let x = crate::xmi::export(&a.repo);
    assert!(x.starts_with("<?xml") && x.contains("xmi:version=\"2.1\""));
    // classes, their members and the relationships, but no diagrams
    let book = a.repo.all().find(|e| a.repo.name(*e) == "Book" && a.repo.ty(*e) == "UMLClass").unwrap();
    assert!(x.contains(&format!("xmi:type=\"uml:Class\" xmi:id=\"{}\" name=\"Book\"", a.repo.id(book))));
    assert!(x.contains("<generalization xmi:type=\"uml:Generalization\""));
    assert!(x.contains("xmi:type=\"uml:Association\"") && x.contains("<ownedEnd xmi:type=\"uml:Property\""));
    assert!(x.contains("<ownedAttribute xmi:type=\"uml:Property\"") && x.contains("<ownedOperation xmi:type=\"uml:Operation\""));
    assert!(x.contains("xmi:type=\"uml:UseCase\"") && x.contains("xmi:type=\"uml:Actor\""));
    assert!(!x.contains("Diagram\""), "no diagrams");
    // every id is defined once, and every reference points at one
    let ids: Vec<&str> = x.split("xmi:id=\"").skip(1).map(|s| s.split('"').next().unwrap()).collect();
    let set: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(ids.len(), set.len(), "unique ids");
    for attr in [" type=\"", " general=\"", " client=\"", " supplier=\"", " association=\"", " addition=\""] {
        for v in x.split(attr).skip(1).map(|s| s.split('"').next().unwrap()) {
            assert!(set.contains(v), "{attr}{v} points nowhere");
        }
    }
    // tags open and close in step
    let opens = x.matches("<packagedElement").count();
    assert_eq!(opens, x.matches("</packagedElement>").count() + x.split("<packagedElement").skip(1).filter(|s| s.split('>').next().unwrap().ends_with('/')).count());
}

#[test]
fn xmi_import() {
    use crate::xmi::{export, import, parse_xml};
    // the reader
    let n = parse_xml("<?xml version=\"1.0\"?><!-- c --><a x='1' y=\"&lt;b&gt;\"><b/><c>t &amp; u<![CDATA[<raw>]]></c></a>").unwrap();
    assert_eq!((n.tag.as_str(), n.attr("x"), n.attr("y")), ("a", Some("1"), Some("<b>")));
    assert_eq!(n.children.len(), 2);
    assert_eq!(n.children[1].text, "t & u<raw>");
    assert!(parse_xml("<a><b></a>").is_err());
    // a round trip through the exporter
    let mut a = App::new(None);
    a.open_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/library.mdj"));
    let x = export(&a.repo);
    let mut b = App::new(None);
    let m = import(&mut b.repo, &x).unwrap();
    assert_eq!(b.repo.name(m), "XMIImported");
    let count = |r: &crate::model::Repo, t: &str| r.all().filter(|e| r.ty(*e) == t).count();
    for t in ["UMLClass", "UMLInterface", "UMLEnumeration", "UMLAttribute", "UMLOperation", "UMLParameter", "UMLGeneralization", "UMLInterfaceRealization", "UMLAssociation", "UMLActor", "UMLUseCase", "UMLInclude", "UMLEnumerationLiteral"] {
        assert_eq!(count(&b.repo, t), count(&a.repo, t), "{t}");
    }
    // references are resolved to the new elements
    let book = b.repo.all().find(|e| b.repo.name(*e) == "Book" && b.repo.ty(*e) == "UMLClass").unwrap();
    let g = b.repo.list(book, "ownedElements").into_iter().find(|e| b.repo.ty(*e) == "UMLGeneralization").unwrap();
    assert_eq!(b.repo.refd(g, "source"), Some(book));
    assert_eq!(b.repo.refd(g, "target").map(|t| b.repo.name(t)), Some("LibraryItem".to_string()));
    let assoc = b.repo.all_of("UMLAssociation")[0];
    assert!(b.repo.child(assoc, "end1").and_then(|e| b.repo.refd(e, "reference")).is_some());
    assert_eq!(b.repo.all_of("Diagram").len(), 1, "only the blank project's diagram: none is imported");
    // memberEnd associations from other tools
    let other = r#"<xmi:XMI xmi:version="2.1" xmlns:xmi="http://schema.omg.org/spec/XMI/2.1" xmlns:uml="http://schema.omg.org/spec/UML/2.1">
      <uml:Model xmi:id="m" name="M">
        <packagedElement xmi:type="uml:Class" xmi:id="A" name="A">
          <ownedAttribute xmi:id="pa" name="b" type="B" association="as"><upperValue xmi:type="uml:LiteralUnlimitedNatural" value="*"/></ownedAttribute>
          <ownedAttribute xmi:id="x" name="size"><type href="http://www.omg.org/spec/UML/20090901/UMLPrimitiveTypes.xmi#Integer"/></ownedAttribute>
        </packagedElement>
        <packagedElement xmi:type="uml:Class" xmi:id="B" name="B"/>
        <packagedElement xmi:type="uml:Association" xmi:id="as" memberEnd="pa pb"><ownedEnd xmi:id="pb" type="A" association="as"/></packagedElement>
      </uml:Model></xmi:XMI>"#;
    let mut c = App::new(None);
    import(&mut c.repo, other).unwrap();
    let ca = c.repo.all().find(|e| c.repo.name(*e) == "A").unwrap();
    let attrs = c.repo.list(ca, "attributes");
    assert_eq!(attrs.len(), 1, "the association end is not an attribute");
    assert_eq!(c.repo.str(attrs[0], "type"), "Integer");
    let ass = c.repo.all_of("UMLAssociation")[0];
    let e1 = c.repo.child(ass, "end1").unwrap();
    assert_eq!((c.repo.name(e1).as_str(), c.repo.str(e1, "multiplicity").as_str()), ("b", "*"));
}

#[test]
fn artifact_component_node_instances() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.add_diagram(None, "UMLDeploymentDiagram");
    let n = a.change(|a| a.create_node("UMLNode", rect(40.0, 300.0, 160.0, 80.0))).unwrap();
    let nm = a.repo.refd(n, "model").unwrap();
    for (kind, view) in [("UMLArtifactInstance", "UMLArtifactInstanceView"), ("UMLComponentInstance", "UMLComponentInstanceView"), ("UMLNodeInstance", "UMLNodeInstanceView")] {
        let v = a.change(|a| a.create_node(kind, rect(40.0, 40.0, 100.0, 60.0))).unwrap();
        assert_eq!(a.repo.ty(v), view);
        assert!(a.repo.refd(v, "slotCompartment").is_some());
    }
    let ni = a.repo.all_of("UMLNodeInstance")[0];
    a.repo.set_ref(ni, "classifier", Some(nm));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let v = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLNodeInstanceView").unwrap();
    let nl = a.repo.refd(a.repo.refd(v, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert!(a.repo.str(nl, "text").ends_with(&format!(" : {}", a.repo.name(nm))) && a.repo.bool(nl, "underline"));
    assert!(!crate::validate::validate(&a.repo).iter().any(|p| p.rule == "UML015"));
    // a component instance typed by a node breaks UML014
    let ci = a.repo.all_of("UMLComponentInstance")[0];
    a.repo.set_ref(ci, "classifier", Some(nm));
    assert!(crate::validate::validate(&a.repo).iter().any(|p| p.rule == "UML014"));
    // Ctrl+Enter adds a slot on any instance
    let use_ = (crate::app::EditKind::Name(ni), crate::canvas::ItemKey::Add);
    a.item_key(v, use_.0, use_.1);
    assert_eq!(a.repo.list(ni, "slots").len(), 1);
}

#[test]
fn tag_reference_uses_the_element_picker() {
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let other = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let om = a.repo.refd(other, "model").unwrap();
    a.focus = Some(m);
    a.open_tag_editor();
    a.edit_tag(None);
    a.tag_edit.as_mut().unwrap().kind = "reference".into();
    a.open_element_picker(a.repo.root, "");
    let p = a.element_picker.as_mut().unwrap();
    p.tag_reference = true;
    p.ty = "Model";
    p.chosen = Some(om);
    // OK fills the dialog, not a field
    let p = a.element_picker.take().unwrap();
    a.picker_ok(p);
    assert_eq!(a.tag_edit.as_ref().unwrap().reference, Some(om));
    assert!(a.element_picker.is_none());
}

#[test]
fn element_list_editor() {
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let cv = a.add_constraint_to(v).unwrap();
    let c = a.repo.refd(cv, "model").unwrap();
    assert!(a.fields_for(c).contains(&("constrainedElements", "reflist")));
    let other = a.change(|a| a.create_node("UMLClass", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    let om = a.repo.refd(other, "model").unwrap();
    // Add through the picker, twice
    a.list_editor = Some((c, "constrainedElements", None));
    for x in [m, om] {
        a.open_element_picker(a.repo.root, "");
        let p = a.element_picker.as_mut().unwrap();
        p.list_add = true;
        p.chosen = Some(x);
        let p = a.element_picker.take().unwrap();
        a.picker_ok(p);
    }
    assert_eq!(a.repo.ref_list(c, "constrainedElements"), vec![m, om]);
    assert_eq!(a.list_editor, Some((c, "constrainedElements", Some(1))));
    // it is saved as a list of references and reads back
    let text = a.repo.serialize();
    let b = Repo::parse(&text).unwrap();
    let bc = b.all_of("UMLConstraint")[0];
    assert_eq!(b.ref_list(bc, "constrainedElements").len(), 2);
    // deleting a referenced element drops it from the list
    a.delete_models(&[om]);
    assert_eq!(a.repo.ref_list(c, "constrainedElements"), vec![m]);
}

#[test]
fn operation_constraint_kinds() {
    use crate::app::{SelectOne, SelectPurpose};
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let op = a.change(|a| a.add_feature(m, "UMLOperation")).unwrap();
    for (i, field) in [(1, "preconditions"), (3, "postconditions"), (0, "ownedElements")] {
        a.select_one = Some(SelectOne { message: String::new(), options: vec![], selected: i, purpose: SelectPurpose::ConstraintKind(op) });
        // Enter is OK
        let ctx = egui::Context::default();
        crate::setup_fonts(&ctx);
        let ev = egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE };
        let mut out = ctx.run_ui(egui::RawInput { events: vec![ev], ..Default::default() }, |ui| a.dialogs(ui.ctx()));
        out.textures_delta.clear();
        assert!(a.select_one.is_none());
        let list = a.repo.list(op, field);
        assert_eq!(list.len(), 1, "{field}");
        assert_eq!(a.repo.ty(list[0]), "UMLConstraint");
    }
}

#[test]
fn structural_diagrams_under_the_project_get_a_model() {
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLClassDiagram"));
    let m = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(m), a.repo.name(m).as_str(), a.repo.parent(m)), ("UMLModel", "Model", Some(root)));
    // inside a package it goes in directly
    let d2 = a.change(|a| a.add_diagram(Some(m), "UMLUseCaseDiagram"));
    assert_eq!(a.repo.parent(d2), Some(m));
}

#[test]
fn information_flow_diagrams() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.add_diagram(None, "UMLInformationFlowDiagram");
    let actor = a.change(|a| a.create_node("UMLActor", rect(40.0, 40.0, 40.0, 70.0))).unwrap();
    let item = a.change(|a| a.create_node("UMLInformationItem", rect(240.0, 40.0, 120.0, 60.0))).unwrap();
    assert_eq!(a.repo.ty(item), "UMLInformationItemView");
    let t = tool("UMLInformationFlow", AssocInit::Plain);
    assert!(a.can_connect(&t, actor, item));
    let e = a.change(|a| a.create_edge(&t, actor, item)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLInformationFlowView");
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(e, "stereotypeLabel").unwrap(), "text"), "«flow»");
    let st = a.repo.refd(a.repo.refd(item, "nameCompartment").unwrap(), "stereotypeLabel").unwrap();
    assert_eq!(a.repo.str(st, "text"), "«information»");
    let fm = a.repo.refd(e, "model").unwrap();
    assert!(a.fields_for(fm).contains(&("conveyed", "reflist")));
}

#[test]
fn profile_diagrams() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLProfileDiagram"));
    let p = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(p), a.repo.name(p).as_str()), ("UMLProfile", "Profile"));
    // inside a profile it goes in directly
    let d2 = a.change(|a| a.add_diagram(Some(p), "UMLProfileDiagram"));
    assert_eq!(a.repo.parent(d2), Some(p));
    a.open_diagram(d);
    let st = a.change(|a| a.create_node("UMLStereotype", rect(40.0, 40.0, 120.0, 60.0))).unwrap();
    let mc = a.change(|a| a.create_node("UMLMetaClass", rect(40.0, 240.0, 120.0, 40.0))).unwrap();
    let mm = a.repo.refd(mc, "model").unwrap();
    a.repo.set(mm, "name", json!("Class"));
    assert_eq!(a.repo.parent(a.repo.refd(st, "model").unwrap()), Some(p), "owned by the profile (UML018)");
    let t = tool("UMLExtension", AssocInit::Plain);
    assert!(a.can_connect(&t, st, mc) && !a.can_connect(&t, mc, st));
    let e = a.change(|a| a.create_edge(&t, st, mc)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLExtensionView");
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    let lbl = |v| a.repo.str(a.repo.refd(a.repo.refd(v, "nameCompartment").unwrap(), "stereotypeLabel").unwrap(), "text");
    assert_eq!(lbl(st), "«stereotype»");
    assert_eq!(lbl(mc), "«metaclass»");
    assert!(!crate::validate::validate(&a.repo).iter().any(|x| x.rule == "UML018"));
    // UML016: "Class" is a metamodel type, "Wombat" is not
    assert!(!crate::validate::validate(&a.repo).iter().any(|x| x.rule == "UML016"));
    a.repo.set(mm, "name", json!("Wombat"));
    assert!(crate::validate::validate(&a.repo).iter().any(|x| x.rule == "UML016"));
    a.repo.set(mm, "name", json!("Class"));
    // a metaclass's Add Stereotype: a stereotype below, extending it
    let b = crate::related::actions("UMLMetaClass").into_iter().find(|b| b.label == "Add Stereotype").unwrap();
    let s2 = a.add_related(mc, b).unwrap();
    assert_eq!(a.repo.ty(s2), "UMLStereotypeView");
    let ext = a.repo.all_of("UMLExtension");
    assert_eq!(ext.len(), 2);
    assert!(ext.iter().any(|x| a.repo.refd(*x, "source") == a.repo.refd(s2, "model") && a.repo.refd(*x, "target") == Some(mm)));
}

#[test]
fn link_object_tool() {
    let mut a = App::new(None);
    let d = a.add_diagram(None, "UMLObjectDiagram");
    let o1 = a.change(|a| a.create_node("UMLObject", rect(40.0, 40.0, 100.0, 40.0))).unwrap();
    let o2 = a.change(|a| a.create_node("UMLObject", rect(340.0, 40.0, 100.0, 40.0))).unwrap();
    let t = tool("LinkObject", AssocInit::Plain);
    assert!(a.can_connect(&t, o1, o2));
    let ov = a.change(|a| a.create_edge(&t, o1, o2)).unwrap();
    let om = a.repo.refd(ov, "model").unwrap();
    assert_eq!((a.repo.ty(om), a.repo.name(om).as_str()), ("UMLObject", "LinkObject"));
    let l = a.repo.all_of("UMLLinkObjectLink");
    assert_eq!(l.len(), 1);
    assert_eq!(a.repo.ty(a.repo.refd(l[0], "linkSide").unwrap()), "UMLLink");
    let lv = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLLinkObjectLinkView").unwrap();
    assert_eq!(a.edge_kind(lv), "LinkObject");
    let link_view = a.repo.refd(lv, "head").unwrap();
    let o3 = a.change(|a| a.create_node("UMLObject", rect(40.0, 300.0, 100.0, 40.0))).unwrap();
    assert!(!a.can_connect(&t, o3, link_view), "a link has one link object at most");
    assert!(a.broken_views(d).is_empty());
    assert!(!crate::validate::validate(&a.repo).iter().any(|p| p.rule == "UML001" && a.repo.ty(p.elem) == "UMLLinkObjectLink"));
}

#[test]
fn shortcut_labels_per_platform() {
    use crate::palette::shortcut_label;
    assert_eq!(shortcut_label("Ctrl+Shift+P", false), "Ctrl+Shift+P");
    assert_eq!(shortcut_label("Ctrl+Shift+P", true), "⌘⇧P");
    assert_eq!(shortcut_label("Alt+Up", true), "⌥↑");
    assert_eq!(shortcut_label("Ctrl+Up", false), "Ctrl+↑");
    assert_eq!(shortcut_label("", true), "");
}

#[test]
fn menu_labels_keep_their_text_off_macos() {
    use crate::ui::menu_label_for;
    assert_eq!(menu_label_for("Save As...      Ctrl+Shift+S", false), "Save As...      Ctrl+Shift+S");
    assert_eq!(menu_label_for("Save As...      Ctrl+Shift+S", true), "Save As...  ⌘⇧S");
    assert_eq!(menu_label_for("Close Diagram  F4", true), "Close Diagram  F4");
    assert_eq!(menu_label_for("Plain", true), "Plain");
}

#[test]
fn create_type_makes_a_class_and_sets_it() {
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let at = a.change(|a| a.add_feature(m, "UMLAttribute")).unwrap();
    let undo = a.undo.len();
    a.apply_input(crate::input::InputAction::CreateType(at), "Money".into());
    assert_eq!(a.undo.len(), undo + 1);
    let t = a.repo.refd(at, "type").unwrap();
    assert_eq!((a.repo.ty(t), a.repo.name(t).as_str()), ("UMLClass", "Money"));
    assert_eq!(a.repo.parent(t), a.repo.parent(m), "next to the class that uses it");
}

#[test]
fn markdown_highlight_keeps_the_text() {
    use crate::markdown::highlight;
    let src = "# Title\n\nSome *em* and **strong** with `code` — é ü\n- item\n1. first\n> quote\n```\nfn x() {}\n```\nunclosed *star and `tick\n";
    let job = highlight(src);
    assert_eq!(job.text, src);
    let color_at = |needle: &str| {
        let at = job.text.find(needle).unwrap();
        job.sections.iter().find(|s| s.byte_range.start.0 <= at && at < s.byte_range.end.0).unwrap().format.color
    };
    assert_ne!(color_at("Title"), color_at("Some"));
    assert_ne!(color_at("`code`"), color_at("Some"));
    assert_eq!(color_at("unclosed"), color_at("Some"), "an unclosed mark stays plain");
    // never panics, whatever the text
    for s in ["*", "**", "`", "é*é", "1. ", "```", "- ", "\n\n", "*é*", "_x_"] {
        assert_eq!(highlight(s).text, s);
    }
}

#[test]
fn mermaid_highlight_keeps_the_text() {
    use crate::mermaid::{highlight, SAMPLE};
    for src in [SAMPLE, "classDiagram-v2\nA-->B\n%% note\nclass X~T~ {\n <<interface>>\n +\"s\"\n}\n", "<<", "\"", "%%", "é<|--ü", "o--", ""] {
        assert_eq!(highlight(src).text, src);
    }
    let job = highlight("A <|-- B");
    let at = job.text.find("<|--").unwrap();
    let sec = job.sections.iter().find(|s| s.byte_range.start.0 <= at && at < s.byte_range.end.0).unwrap();
    assert_ne!(sec.format.color, job.sections[0].format.color, "the arrow is coloured");
}

#[test]
fn export_bounds_include_line_labels() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(340.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    a.repo.set(m, "name", json!("far away"));
    let nl = a.repo.refd(e, "nameLabel").unwrap();
    a.repo.set(nl, "distance", json!(400));
    crate::views::layout_diagram(&mut a.repo, &ms, d);
    let lb = crate::views::node_rect(&a.repo, nl);
    let b = crate::render::bounds(&a.repo, d).unwrap();
    assert!(b.y <= lb.y && b.y + b.h >= lb.y + lb.h && b.x <= lb.x && b.x + b.w >= lb.x + lb.w, "{b:?} covers {lb:?}");
}

#[test]
fn tag_dialog_keys() {
    let mut a = App::new(None);
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    a.focus = Some(m);
    a.open_tag_editor();
    a.edit_tag(None);
    a.tag_edit.as_mut().unwrap().name = "owner".into();
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let key = |k| egui::Event::Key { key: k, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE };
    let mut frame = |a: &mut App, k| {
        let mut out = ctx.run_ui(egui::RawInput { events: vec![key(k)], ..Default::default() }, |ui| a.dialogs(ui.ctx()));
        out.textures_delta.clear();
    };
    // Enter is OK in the Tag dialog; the editor stays open
    frame(&mut a, egui::Key::Enter);
    assert!(a.tag_edit.is_none() && a.tag_editor.is_some());
    assert_eq!(a.repo.list(m, "tags").len(), 1);
    // Esc in the dialog cancels it only
    a.edit_tag(None);
    frame(&mut a, egui::Key::Escape);
    assert!(a.tag_edit.is_none() && a.tag_editor.is_some(), "the editor stays");
    // Esc in the editor closes it
    frame(&mut a, egui::Key::Escape);
    assert!(a.tag_editor.is_none());
}

#[test]
fn new_kinds_take_their_fill_colour_preference() {
    let mut a = App::new(None);
    a.prefs.set("uml.object.fillColor", json!("#ffeeaa"));
    a.prefs.set("uml.stereotype.fillColor", json!("#aaeeff"));
    let o = a.change(|a| a.create_node("UMLObject", rect(40.0, 40.0, 100.0, 40.0))).unwrap();
    let s = a.change(|a| a.create_node("UMLStereotype", rect(240.0, 40.0, 100.0, 60.0))).unwrap();
    assert_eq!(a.repo.str(o, "fillColor"), "#ffeeaa");
    assert_eq!(a.repo.str(s, "fillColor"), "#aaeeff");
}

#[test]
fn inherit_style_toggles_and_leaves_the_rubber_band() {
    use egui::{Modifiers, Pos2};
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(200.0, 40.0, 100.0, 60.0))).unwrap();
    a.select(vec![c1]);
    a.toggle_flag("parentStyle");
    assert!(a.repo.bool(c1, "parentStyle"));
    a.select(vec![c1, c2]);
    a.toggle_flag("parentStyle");
    assert!(a.repo.bool(c1, "parentStyle") && a.repo.bool(c2, "parentStyle"), "a mixed selection is set");
    a.toggle_flag("parentStyle");
    assert!(!a.repo.bool(c1, "parentStyle") && !a.repo.bool(c2, "parentStyle"), "a shared value is inverted");
    a.change(|a| a.repo.set(c1, "parentStyle", json!(true)));
    a.select(vec![]);
    a.on_press(canvas, Pos2::new(10.0, 10.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(400.0, 200.0));
    a.on_release(canvas.min, Pos2::new(400.0, 200.0));
    assert_eq!(a.selection, vec![c2], "a view inheriting its style is not taken by the band");
    a.select_all();
    assert_eq!(a.selection, vec![c1, c2], "but Select All takes the top-level views");
}

#[test]
fn frame_tool_picks_an_element_and_shows_its_diagram() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(300.0, 300.0, 100.0, 60.0))).unwrap();
    let other = a.change(|a| a.add_diagram(None, "UMLUseCaseDiagram"));
    a.open_diagram(d);
    // the Frame tool asks for an element before creating anything
    a.tool = Some(crate::app::Tool { label: "Frame", kind: "Frame", icon: "frame", line: false, init: AssocInit::Plain });
    a.on_press(canvas, Pos2::new(20.0, 20.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(250.0, 200.0));
    a.on_release(canvas.min, Pos2::new(250.0, 200.0));
    let mut p = a.element_picker.clone().expect("the picker opens");
    assert!(p.frame.is_some() && p.chosen.is_none());
    assert_eq!(a.repo.list(d, "ownedViews").len(), 1, "nothing is created yet");
    p.chosen = Some(other);
    a.picker_ok(p);
    let f = a.repo.list(d, "ownedViews")[0];
    assert_eq!(a.repo.ty(f), "UMLFrameView", "the frame goes behind the other views");
    assert_eq!(a.selection, vec![f]);
    assert_eq!(a.repo.refd(f, "model"), Some(other));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let (nl, tl) = (a.repo.refd(f, "nameLabel").unwrap(), a.repo.refd(f, "frameTypeLabel").unwrap());
    assert_eq!(a.repo.str(tl, "text"), "uc");
    let owner = a.repo.parent(other).unwrap();
    assert_eq!(a.repo.str(nl, "text"), a.repo.name(owner), "a diagram's frame names the diagram's owner");
    assert!(a.repo.str(tl, "font").ends_with(";1"), "the kind is bold");
    // Show Type, Namespace and Diagram Name add to the name
    for f_ in ["showType", "showDiagramName"] {
        a.toggle_flag(f_);
    }
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(nl, "text"), format!("[Model] {} [{}]", a.repo.name(owner), a.repo.name(d)));
    // a frame of a class: its kind, and no inline edit
    let p2 = { a.open_frame_picker(d, rect(500.0, 20.0, 10.0, 10.0)); let mut p = a.element_picker.clone().unwrap(); p.chosen = Some(a.repo.refd(c, "model").unwrap()); p };
    a.picker_ok(p2);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let f2 = a.repo.list(d, "ownedViews")[0];
    assert_eq!(a.repo.str(a.repo.refd(f2, "frameTypeLabel").unwrap(), "text"), "class");
    assert!(a.repo.f(f2, "width") >= crate::views::FRAME_MIN_W && a.repo.f(f2, "height") >= 53.0, "a small frame grows to its minimum");
    a.edit_view(f2, None);
    assert!(a.editing.is_none());
    // the file keeps the frame and its labels, and reads back
    let back = Repo::parse(&a.repo.serialize()).unwrap();
    assert_eq!(back.all_of("UMLFrameView").len(), 2);
}

#[test]
fn dropping_the_open_diagram_makes_a_frame_of_it() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    assert!(a.can_drop_model(d));
    let vs = a.drop_model(d, crate::views::P { x: 20.0, y: 30.0 }).unwrap();
    assert_eq!(vs.len(), 1);
    let f = vs[0];
    assert_eq!(a.repo.ty(f), "UMLFrameView");
    assert_eq!(a.repo.refd(f, "model"), Some(d));
    assert_eq!((a.repo.f(f, "left"), a.repo.f(f, "top"), a.repo.f(f, "width"), a.repo.f(f, "height")), (20.0, 30.0, 700.0, 600.0));
    // another diagram makes a hyperlink to it
    let other = a.change(|a| a.add_diagram(None, "UMLClassDiagram"));
    a.open_diagram(d);
    assert!(a.can_drop_model(other));
    let vs = a.drop_model(other, crate::views::P { x: 40.0, y: 700.0 }).unwrap();
    let hv = vs[0];
    assert_eq!(a.repo.ty(hv), "HyperlinkView");
    let link = a.repo.refd(hv, "model").unwrap();
    assert_eq!(a.repo.ty(link), "Hyperlink");
    assert_eq!(a.repo.refd(link, "reference"), Some(other));
    assert_eq!(a.label_of(link), format!("(link to {})", a.repo.name(other)));
    let ms = measure();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(hv, "nameLabel").unwrap(), "text"), a.repo.name(other));
    assert_eq!(a.repo.str(a.repo.refd(hv, "typeLabel").unwrap(), "text"), "link");
    assert!(a.repo.f(hv, "width") >= crate::views::HYPERLINK_MIN_W && a.repo.f(hv, "height") >= crate::views::HYPERLINK_MIN_H);
}

#[test]
fn hyperlink_tool_shows_its_url_or_its_name() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("Hyperlink", rect(20.0, 20.0, 10.0, 10.0))).unwrap();
    let link = a.repo.refd(v, "model").unwrap();
    assert_eq!(a.repo.name(link), "Hyperlink1");
    assert_eq!(a.label_of(link), "(none)");
    a.change(|a| a.repo.set(link, "url", json!("https://example.com")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(v, "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "https://example.com");
    assert_eq!(a.label_of(link), "(link to https://example.com)");
    a.change(|a| a.repo.set(link, "showName", json!(true)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(nl, "text"), "Hyperlink1");
    assert_eq!(a.label_of(link), "(link: Hyperlink1)");
    assert_eq!(a.fields_for(link).iter().map(|f| f.0).collect::<Vec<_>>(), ["name", "reference", "url", "showName"]);
    a.edit_view(v, None);
    assert!(a.editing.is_none(), "no inline edit");
    let back = Repo::parse(&a.repo.serialize()).unwrap();
    assert_eq!(back.all_of("Hyperlink").len(), 1);
}

#[test]
fn shape_tools_and_their_minimum_size() {
    use egui::{Modifiers, Pos2};
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    for (kind, ty) in [("Rect", "RectangleView"), ("RoundRect", "RoundRectView"), ("Ellipse", "EllipseView")] {
        a.tool = Some(crate::app::Tool { label: "", kind, icon: "", line: false, init: AssocInit::Plain });
        // a 20×30 drag is enlarged to the 50×50 minimum
        a.on_press(canvas, Pos2::new(100.0, 100.0), Modifiers::NONE);
        a.on_move(canvas, Pos2::new(120.0, 130.0));
        a.on_release(canvas.min, Pos2::new(120.0, 130.0));
        let v = *a.repo.list(d, "ownedViews").last().unwrap();
        assert_eq!(a.repo.ty(v), ty);
        assert!(a.repo.refd(v, "model").is_none(), "a shape is a view only");
        assert_eq!((a.repo.f(v, "width"), a.repo.f(v, "height")), (50.0, 50.0));
    }
    // a larger drag keeps its size
    a.tool = Some(crate::app::Tool { label: "", kind: "Note", icon: "", line: false, init: AssocInit::Plain });
    a.on_press(canvas, Pos2::new(300.0, 300.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(500.0, 320.0));
    a.on_release(canvas.min, Pos2::new(500.0, 320.0));
    let n = *a.repo.list(d, "ownedViews").last().unwrap();
    assert_eq!((a.repo.f(n, "width"), a.repo.f(n, "height")), (200.0, 40.0), "a note is at least 40 tall");
    let mut svg = String::new();
    let ms = measure();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    svg.push_str(&crate::render::to_svg(&a.repo, d, &a.info, &ms).unwrap());
    assert!(svg.contains("<ellipse") || svg.contains("ellipse"), "the ellipse is drawn");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn quick_edit_grammars_accept_and_reject_like_the_original() {
    use crate::parse::{parses, Grammar::*};
    for (g, ok) in [
        (Name, "<<entity>> +Order"), (Name, "Order Item"), (Name, "it's"), (Name, "a[1]"),
        (Attribute, "-id: int[1] = 0"), (Attribute, "<< st >> #name : List<String> [0..*] = x"), (Attribute, "x"),
        (Operation, "+placeOrder(item: Order, qty: int): boolean"), (Operation, "f()"), (Operation, "f(in a, out b: int[*] = 3, inout c)"), (Operation, "g(): void[0..1]"),
        (TemplateParameter, "T: Class[] = Object"), (Object, ""), (Object, ": Order"), (Object, "o1: Order"), (Slot, "x: int = 1"), (Name, "emoji 🦀"),
        (Lifeline, "<<st>> -l1[i]: Order"), (Lifeline, "buyer : List<Item[]>"), (Lifeline, "it's"), (Lifeline, "l [ k ]"),
    ] {
        assert!(parses(g, ok), "{g:?} accepts {ok:?}");
    }
    for (g, bad) in [
        (Name, ""), (Name, "a: B"), (Name, "f(x)"), (Name, "<<st"), (Name, "a = b"),
        (Attribute, "a: b: c"), (Attribute, "a = b = c"), (Attribute, "it's"), (Attribute, "f()"), (Attribute, "a: b[x]"),
        (Operation, "f(a, )"), (Operation, "f(a"), (Operation, "f(): (x)"), (TemplateParameter, "+T"), (Slot, "x = 1 = 2"),
        (Lifeline, ""), (Lifeline, ": Order"), (Lifeline, "a[1"), (Lifeline, "a[]"), (Lifeline, "l = x"), (Lifeline, "f(x)"),
    ] {
        assert!(!parses(g, bad), "{g:?} rejects {bad:?}");
    }
}

#[test]
fn a_rejected_inline_edit_stays_open_and_changes_nothing() {
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let name = a.repo.name(m);
    a.edit_view(c, None);
    let ed = a.editing.as_mut().unwrap();
    ed.orig = Some(ed.text.clone());
    ed.text = "Bad: Name".into();
    assert!(!a.try_close_inline(true), "refused");
    assert!(a.editing.as_ref().unwrap().fresh, "the text is selected again");
    assert!(a.edit_shake.is_some());
    assert!(!a.try_close_inline(false), "it cannot close while it shakes");
    assert_eq!(a.repo.name(m), name, "nothing changed");
    // once the shake is over a valid text goes through
    a.edit_shake = Some(std::time::Instant::now() - std::time::Duration::from_millis(500));
    a.editing.as_mut().unwrap().text = "Good Name".into();
    assert!(a.try_close_inline(true));
    assert!(a.editing.is_none());
    assert_eq!(a.repo.name(m), "Good Name");
    // an unchanged text is never checked, and a forced close drops a rejected one
    a.edit_view(c, None);
    a.editing.as_mut().unwrap().orig = Some("Good Name".into());
    a.editing.as_mut().unwrap().text = "".into();
    a.close_inline(true);
    assert!(a.editing.is_none());
    assert_eq!(a.repo.name(m), "Good Name", "an empty name is rejected, so it is dropped");
}

#[test]
fn inline_edit_buttons_carry_their_keys() {
    use crate::app::EditKind;
    use crate::canvas::{item_buttons, ItemKey};
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let e = a.change(|a| a.create_node("UMLEnumeration", rect(300.0, 40.0, 160.0, 100.0))).unwrap();
    let (cm, em) = (a.repo.refd(c, "model").unwrap(), a.repo.refd(e, "model").unwrap());
    let hints = |k: &EditKind, a: &App| item_buttons(&a.repo, None, k).into_iter().map(|b| if b.2.is_empty() { b.1.to_string() } else { format!("{} ({})", b.1, b.2) }).collect::<Vec<_>>();
    assert_eq!(hints(&EditKind::Name(cm), &a), ["Add Attribute (Ctrl+Enter)", "Add Operation (Ctrl+Shift+Enter)", "Add Reception", "Add Port", "Add Part"]);
    assert_eq!(hints(&EditKind::Name(em), &a), ["Add Literal (Ctrl+Enter)", "Add Operation (Ctrl+Shift+Enter)"]);
    let keys: Vec<ItemKey> = item_buttons(&a.repo, None, &EditKind::Feature(cm)).into_iter().map(|b| b.3).collect();
    assert_eq!(keys, [ItemKey::Add, ItemKey::Delete, ItemKey::Move(true), ItemKey::Move(false)]);
    // a button does what its key does
    a.item_key(c, EditKind::Name(cm), ItemKey::Add);
    assert_eq!(a.repo.list(cm, "attributes").len(), 1);
    // Add Reception shows the suppressed receptions again
    assert!(a.repo.bool(c, "suppressReceptions"));
    a.item_key(c, EditKind::Name(cm), ItemKey::AddReception);
    assert_eq!(a.repo.list(cm, "receptions").len(), 1);
    assert!(!a.repo.bool(c, "suppressReceptions"));
}

#[test]
fn nary_association_node_joins_classifiers() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let n = a.change(|a| a.create_node("UMLNaryAssociationNode", rect(200.0, 200.0, 10.0, 10.0))).unwrap();
    assert_eq!(a.repo.ty(n), "UMLNaryAssociationNodeView");
    let nm = a.repo.refd(n, "model").unwrap();
    assert_eq!((a.repo.ty(nm), a.repo.name(nm)), ("UMLNaryAssociationNode", String::new()), "it starts unnamed");
    let cs: Vec<_> = (0..3).map(|i| a.change(|a| a.create_node("UMLClass", rect(40.0 + 200.0 * i as f64, 20.0, 100.0, 60.0))).unwrap()).collect();
    let assoc = tool("UMLAssociation", AssocInit::Plain);
    for c in &cs {
        assert!(a.can_connect(&assoc, *c, n), "an association joins a class to the node");
        a.change(|a| a.create_edge(&assoc, *c, n)).unwrap();
    }
    // the Association Class tool links a class to the node, without a new class
    let ac = tool("AssociationClass", AssocInit::Plain);
    assert!(a.can_connect(&ac, cs[0], n) && a.can_connect(&ac, n, cs[1]));
    let link = a.change(|a| a.create_edge(&ac, n, cs[1])).unwrap();
    assert_eq!(a.repo.ty(link), "UMLAssociationClassLinkView");
    let lm = a.repo.refd(link, "model").unwrap();
    assert_eq!((a.repo.refd(lm, "classSide"), a.repo.refd(lm, "associationSide")), (a.repo.refd(cs[1], "model"), Some(nm)));
    assert_eq!(a.repo.refd(link, "tail"), Some(cs[1]), "drawn from the class");
    a.change(|a| a.repo.set(nm, "name", json!("Booking")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.f(n, "width") >= 30.0 && a.repo.f(n, "height") >= 22.0);
    let nl = a.repo.refd(n, "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "Booking");
    let centre = (a.repo.f(n, "left") + a.repo.f(n, "width") / 2.0, a.repo.f(n, "top") + a.repo.f(n, "height") / 2.0);
    let label_centre = (a.repo.f(nl, "left") + a.repo.f(nl, "width") / 2.0, a.repo.f(nl, "top") + a.repo.f(nl, "height") / 2.0);
    assert_eq!((label_centre.0 - centre.0, label_centre.1 - centre.1), (-15.0, -15.0), "the name floats up and to the left");
    a.edit_view(n, None);
    assert!(matches!(a.editing.as_ref().map(|e| &e.kind), Some(crate::app::EditKind::Name(m)) if *m == nm));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn behaviours_and_substitutions_from_model_add() {
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    let pkg = a.repo.parent(cm).unwrap();
    let kinds = |a: &App, e| a.add_kinds(e).into_iter().map(|k| k.1).collect::<Vec<_>>();
    let on_class = kinds(&a, cm);
    for k in ["UMLInteraction", "UMLStateMachine", "UMLActivity", "UMLOpaqueBehavior"] {
        assert!(on_class.contains(&k), "a classifier takes {k}");
    }
    let on_pkg = kinds(&a, pkg);
    assert!(on_pkg.contains(&"UMLStateMachine") && on_pkg.contains(&"UMLActivity"));
    assert!(!on_pkg.contains(&"UMLInteraction") && !on_pkg.contains(&"UMLOpaqueBehavior"));
    let sm = a.change(|a| a.add_model(cm, "UMLStateMachine"));
    assert_eq!(a.repo.name(sm), "StateMachine1");
    assert_eq!(a.repo.list(sm, "regions").len(), 1, "with its region");
    // a template binding takes parameter substitutions
    let t = a.change(|a| a.create_node("UMLClass", rect(300.0, 40.0, 160.0, 100.0))).unwrap();
    let bind = tool("UMLTemplateBinding", AssocInit::Plain);
    let b = a.change(|a| a.create_edge(&bind, c, t)).unwrap();
    let bm = a.repo.refd(b, "model").unwrap();
    assert!(kinds(&a, bm).contains(&"UMLTemplateParameterSubstitution"));
    let s = a.change(|a| a.add_feature(bm, "UMLTemplateParameterSubstitution")).unwrap();
    assert_eq!(a.repo.list(bm, "parameterSubstitutions"), vec![s]);
    let text = a.repo.serialize();
    assert!(text.contains("\"parameterSubstitutions\"") && !text.contains("\"isReentrant\""), "isReentrant keeps its default");
    Repo::parse(&text).unwrap();
}

#[test]
fn the_wheel_moves_the_origin_in_diagram_units() {
    let mut a = App::new(None);
    a.set_zoom(2.0);
    a.scroll = crate::views::P { x: 100.0, y: 100.0 };
    a.wheel_scroll(egui::vec2(0.0, -40.0), false);
    assert_eq!((a.scroll.x, a.scroll.y), (100.0, 140.0), "not divided by the zoom");
    a.wheel_scroll(egui::vec2(0.0, -30.0), true);
    assert_eq!((a.scroll.x, a.scroll.y), (130.0, 140.0), "Shift scrolls sideways");
    a.wheel_scroll(egui::vec2(500.0, 500.0), false);
    assert_eq!((a.scroll.x, a.scroll.y), (0.0, 0.0), "never past the origin");
}

#[test]
fn collaborations_and_their_uses() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLCollaboration", rect(40.0, 40.0, 10.0, 10.0))).unwrap();
    let u = a.change(|a| a.create_node("UMLCollaborationUse", rect(300.0, 40.0, 10.0, 10.0))).unwrap();
    assert_eq!((a.repo.ty(c), a.repo.ty(u)), ("UMLCollaborationView", "UMLCollaborationUseView"));
    let (cm, um) = (a.repo.refd(c, "model").unwrap(), a.repo.refd(u, "model").unwrap());
    assert_eq!((a.repo.name(cm), a.repo.name(um)), ("Collaboration1".to_string(), "CollaborationUse1".to_string()));
    assert!(a.repo.refd(c, "templateParameterCompartment").is_some());
    a.change(|a| a.repo.set_ref(um, "type", Some(cm)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.f(c, "width") >= crate::views::UC_MIN_W && a.repo.f(c, "height") >= crate::views::UC_MIN_H, "grown to the minimum");
    let nl = a.repo.refd(a.repo.refd(u, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "CollaborationUse1: Collaboration1");
    assert_eq!(crate::parse::name_text(&a.repo, um), "CollaborationUse1", "the name edit holds the name only");
    assert_eq!(a.fields_for(um).iter().map(|f| f.0).collect::<Vec<_>>(), ["name", "stereotype", "visibility", "type"]);
    let svg = crate::render::to_svg(&a.repo, d, &a.info, &ms).unwrap();
    assert!(svg.contains("Collaboration1"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn profile_stereotype_icons() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.package_for(Some(d));
    let st = a.change(|a| a.add_model(pkg, "UMLStereotype"));
    let icon = a.repo.child(st, "icon").expect("a new stereotype has its icon");
    assert_eq!((a.repo.f(icon, "width"), a.repo.f(icon, "height")), (40.0, 40.0));
    a.change(|a| a.repo.set(icon, "content", json!("!icon 20 10\nr 0 0 20 10\n%f #ff0000\nfe 5 2 15 8\n%p 2 2\nl 0 5 20 5")));
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 10.0, 10.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.change(|a| a.repo.set_ref(m, "stereotype", Some(st)));
    a.change(|a| a.repo.set(c, "stereotypeDisplay", json!("icon")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info[&c].shape, "stereotype-icon");
    let nc = a.repo.refd(c, "nameCompartment").unwrap();
    assert!(a.repo.f(c, "width") >= 40.0 && a.repo.f(nc, "top") >= a.repo.f(c, "top") + 40.0, "the icon sits above the name, at least 40 tall");
    let svg = crate::render::to_svg(&a.repo, d, &a.info, &ms).unwrap();
    assert!(svg.contains("#ff0000"), "the script's fill colour is used");
    // decoration: the box with the icon at its top right
    a.change(|a| a.repo.set(c, "stereotypeDisplay", json!("decoration")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info[&c].shape, "");
    assert!(crate::render::to_svg(&a.repo, d, &a.info, &ms).unwrap().contains("#ff0000"));
    // the icon's size is edited as integers
    assert_eq!(a.fields_for(icon), vec![("width", "int"), ("height", "int"), ("image", "image"), ("content", "multiline")]);
    let text = a.repo.serialize();
    assert!(text.contains("\"UMLImage\""));
    let back = Repo::parse(&text).unwrap();
    assert!(back.all_of("UMLImage").len() == 1);
}

#[test]
fn icon_with_label_shows_the_stereotype() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let b = a.change(|a| a.create_node("Boundary", rect(20.0, 20.0, 10.0, 10.0))).unwrap();
    let st = a.repo.refd(a.repo.refd(b, "nameCompartment").unwrap(), "stereotypeLabel").unwrap();
    for (display, shown) in [("icon", false), ("icon-label", true)] {
        a.change(|a| a.repo.set(b, "stereotypeDisplay", json!(display)));
        a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
        assert_eq!(a.repo.raw(st, "visible") != Some(&json!(false)), shown, "{display}");
    }
}

#[test]
fn a_diagram_accepts_the_kinds_its_type_allows() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.package_for(Some(d));
    let actor = a.change(|a| a.add_model(pkg, "UMLActor"));
    let p2 = a.change(|a| a.add_model(pkg, "UMLPackage"));
    let st = a.change(|a| a.add_model(pkg, "UMLStereotype"));
    assert!(a.can_drop_model(actor) && a.can_drop_model(p2), "a class diagram takes classifiers and packages");
    let od = a.change(|a| a.add_diagram(Some(pkg), "UMLObjectDiagram"));
    assert_eq!(a.diagram, Some(od));
    assert!(a.can_drop_model(actor), "an object diagram takes classifiers");
    assert!(!a.can_drop_model(p2), "but not packages");
    assert_eq!(a.drop_model(p2, crate::views::P { x: 10.0, y: 10.0 }), Ok(vec![]), "and the drop does nothing");
    let prof = a.change(|a| a.add_diagram(Some(pkg), "UMLProfileDiagram"));
    assert_eq!(a.diagram, Some(prof));
    assert!(a.can_drop_model(st) && !a.can_drop_model(actor), "a profile diagram takes stereotypes, not actors");
}

#[test]
fn views_paste_into_any_diagram_that_takes_their_kinds() {
    use crate::app::Area;
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let n = a.change(|a| a.create_node("Note", rect(300.0, 300.0, 100.0, 50.0))).unwrap();
    a.last_area = Area::Canvas;
    a.select(vec![c]);
    let class_text = a.copy_text().unwrap();
    a.select(vec![n]);
    let note_text = a.copy_text().unwrap();
    let pkg = a.package_for(Some(d));
    let uc = a.change(|a| a.add_diagram(Some(pkg), "UMLUseCaseDiagram"));
    assert_eq!(a.diagram, Some(uc));
    a.paste_text(&class_text);
    assert!(a.message.is_none(), "a class view pastes into a use case diagram");
    assert_eq!(a.repo.list(uc, "ownedViews").len(), 1);
    let inf = a.change(|a| a.add_diagram(Some(pkg), "UMLInformationFlowDiagram"));
    assert_eq!(a.diagram, Some(inf));
    a.paste_text(&class_text);
    assert!(a.message.take().is_some(), "an information flow diagram takes only the general views");
    a.paste_text(&note_text);
    assert!(a.message.is_none() && a.repo.list(inf, "ownedViews").len() == 1, "such as a note");
}

#[test]
fn a_subsystem_offers_its_interfaces() {
    let mut a = App::new(None);
    let s = a.change(|a| a.create_node("UMLSubsystem", rect(40.0, 200.0, 200.0, 120.0))).unwrap();
    let labels: Vec<&str> = crate::related::actions("UMLSubsystem").iter().map(|r| r.label).collect();
    assert_eq!(labels, ["Add Provided Interface", "Add Required Interface", "Add Note", "Add Constraint"]);
    let provided = crate::related::actions("UMLSubsystem")[0];
    a.add_related(s, provided);
    let sm = a.repo.refd(s, "model").unwrap();
    let real = a.repo.all().find(|e| a.repo.ty(*e) == "UMLInterfaceRealization").expect("the realization is made");
    assert_eq!(a.repo.refd(real, "source"), Some(sm));
}

#[test]
fn association_end_order_arrow() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    a.change(|a| a.repo.set(m, "name", json!("owns")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let polys = |a: &App| crate::render::to_svg(&a.repo, d, &a.info, &ms).unwrap().matches("<polygon").count();
    let plain = polys(&a);
    for order in ["forward", "backward"] {
        a.change(|a| a.repo.set(e, "showEndOrder", json!(order)));
        assert_eq!(polys(&a), plain + 1, "{order} draws one triangle");
    }
    a.change(|a| a.repo.set(e, "showEndOrder", json!("hide")));
    assert_eq!(polys(&a), plain);
}

#[test]
fn ports_parts_and_connectors() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    // a port clicked near the right border goes onto it
    let p = a.change(|a| a.create_feature_box("UMLPort", crate::views::P { x: 390.0, y: 200.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    let pm = a.repo.refd(p, "model").unwrap();
    assert_eq!((a.repo.ty(pm), a.repo.name(pm)), ("UMLPort", "Port1".to_string()));
    assert_eq!(a.repo.parent(pm), Some(cm));
    assert_eq!(a.repo.field_of(pm).as_deref(), Some("attributes"));
    assert_eq!(a.repo.refd(p, "containerView"), Some(c));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let (x, y, w) = (a.repo.f(p, "left"), a.repo.f(p, "top"), a.repo.f(p, "width"));
    assert_eq!(w, 14.0);
    let right = a.repo.f(c, "left") + a.repo.f(c, "width");
    assert_eq!(x + 7.0, right, "its centre is on the right border");
    assert!(y > 100.0 && y < 300.0);
    // a part drawn inside: a composite attribute in a box
    let part = a.change(|a| a.create_feature_box("UMLPart", crate::views::P { x: 151.0, y: 151.0 }, rect(150.0, 150.0, 100.0, 40.0))).unwrap();
    let am = a.repo.refd(part, "model").unwrap();
    assert_eq!((a.repo.name(am), a.repo.str(am, "aggregation")), ("Part1".to_string(), "composite".to_string()));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(a.repo.refd(part, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "Part1");
    // outside any box: refused
    assert_eq!(a.create_feature_box("UMLPort", crate::views::P { x: 900.0, y: 900.0 }, rect(0.0, 0.0, 0.0, 0.0)), Err("UMLPort cannot be placed here.".to_string()));
    // a port on a part without a type is refused; with a type it goes to that type
    assert_eq!(a.create_feature_box("UMLPort", crate::views::P { x: 160.0, y: 160.0 }, rect(0.0, 0.0, 0.0, 0.0)), Err("Type should be assigned to have Ports".to_string()));
    // a connector joins the port and the part
    let conn = tool("UMLConnector", AssocInit::Plain);
    assert!(a.can_connect(&conn, p, part) && !a.can_connect(&conn, c, part));
    let e = a.change(|a| a.create_edge(&conn, p, part)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLConnectorView");
    let em = a.repo.refd(e, "model").unwrap();
    assert_eq!(a.repo.ty(a.repo.child(em, "end1").unwrap()), "UMLConnectorEnd");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // moving the class carries its port and part
    let before = a.repo.f(part, "left");
    let canvas = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0, 700.0));
    a.select(vec![]);
    a.on_press(canvas, egui::Pos2::new(250.0, 110.0), egui::Modifiers::NONE);
    a.on_move(canvas, egui::Pos2::new(270.0, 110.0));
    a.on_release(canvas.min, egui::Pos2::new(270.0, 110.0));
    assert_eq!(a.repo.f(part, "left"), before + 20.0);
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn role_binding_from_a_collaboration_use_to_a_part() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let part = a.change(|a| a.create_feature_box("UMLPart", crate::views::P { x: 151.0, y: 151.0 }, rect(150.0, 150.0, 100.0, 40.0))).unwrap();
    let u = a.change(|a| a.create_node("UMLCollaborationUse", rect(500.0, 100.0, 120.0, 40.0))).unwrap();
    let rb = tool("UMLRoleBinding", AssocInit::Plain);
    assert!(a.can_connect(&rb, u, part));
    assert!(!a.can_connect(&rb, part, u) && !a.can_connect(&rb, u, c), "from the use, to an attribute");
    let e = a.change(|a| a.create_edge(&rb, u, part)).unwrap();
    assert_eq!(a.repo.ty(e), "UMLRoleBindingView");
    let m = a.repo.refd(e, "model").unwrap();
    assert!(a.fields_for(m).iter().any(|f| f.0 == "roleName"));
    a.change(|a| a.repo.set(m, "roleName", json!("buyer")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l = a.repo.refd(e, "roleNameLabel").unwrap();
    assert_eq!(a.repo.str(l, "text"), "buyer");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn image_annotation_from_a_file() {
    use crate::picture::{base64_decode, base64_encode};
    for s in ["", "a", "ab", "abc", "hello world!"] {
        assert_eq!(base64_decode(&base64_encode(s.as_bytes())).unwrap(), s.as_bytes());
    }
    assert_eq!(base64_encode(b"Man"), "TWFu");
    let ms = measure();
    let dir = std::env::temp_dir().join(format!("umlcraft-img-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("red.png");
    image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255])).save(&png).unwrap();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("Image", rect(10.0, 10.0, 100.0, 100.0))).unwrap();
    assert_eq!(a.repo.ty(v), "ImageView");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(!crate::render::to_svg(&a.repo, d, &a.info, &ms).unwrap().contains("<image"), "no picture yet: a crossed box");
    a.set_image_file(v, &png).unwrap();
    assert!(a.repo.str(v, "imageData").starts_with("data:image/png;base64,"));
    assert_eq!((a.repo.f(v, "width"), a.repo.f(v, "height"), a.repo.f(v, "imageWidth")), (40.0, 20.0, 40.0), "the natural size");
    // dropped on the canvas: centred on the point
    let w = a.drop_image_file(&png, crate::views::P { x: 300.0, y: 300.0 }).unwrap();
    assert_eq!((a.repo.f(w, "left"), a.repo.f(w, "top")), (280.0, 290.0));
    assert!(a.drop_image_file(&dir.join("notes.txt"), crate::views::P { x: 0.0, y: 0.0 }).is_err(), "only pictures");
    // the picture reaches the exported PNG
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let out = crate::render::to_png(&a.repo, d, &a.info, &ms, 1.0).unwrap();
    let img = image::load_from_memory(&out).unwrap().to_rgba8();
    assert!(img.pixels().any(|p| p.0 == [255, 0, 0, 255]), "red pixels in the export");
    let back = Repo::parse(&a.repo.serialize()).unwrap();
    assert_eq!(back.all_of("ImageView").len(), 2);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn port_and_part_quick_edit_actions() {
    use crate::app::EditKind;
    use crate::canvas::{item_buttons, ItemKey};
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    a.item_key(c, EditKind::Name(cm), ItemKey::AddPort);
    a.item_key(c, EditKind::Name(cm), ItemKey::AddPart);
    a.item_key(c, EditKind::Name(cm), ItemKey::AddPart);
    let views_of = |a: &App, ty: &str| a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.ty(*v) == ty).collect::<Vec<_>>();
    let (ports, parts) = (views_of(&a, "UMLPortView"), views_of(&a, "UMLPartView"));
    assert_eq!((ports.len(), parts.len()), (1, 2));
    assert_eq!(a.repo.list(cm, "attributes").len(), 3);
    assert_eq!((a.repo.f(parts[0], "left"), a.repo.f(parts[0], "top")), (110.0, 130.0), "the first part at +10, +30");
    assert!(a.repo.f(parts[1], "left") > a.repo.f(parts[0], "left"), "the next one to its right");
    // a port's edit: Select Type, Create Type, Add Connected Part
    let pm = a.repo.refd(ports[0], "model").unwrap();
    let labels: Vec<&str> = item_buttons(&a.repo, None, &EditKind::Name(pm)).into_iter().map(|b| b.1).collect();
    assert_eq!(labels, ["Select Type", "Create Type", "Add Provided Interface", "Add Required Interface", "Add Connected Part"]);
    a.item_key(ports[0], EditKind::Name(pm), ItemKey::AddConnectedPart);
    assert_eq!(views_of(&a, "UMLPartView").len(), 3);
    let conn = a.repo.all().find(|e| a.repo.ty(*e) == "UMLConnector").expect("joined by a connector");
    assert_eq!(a.repo.child(conn, "end1").and_then(|e| a.repo.refd(e, "reference")), Some(pm));
    let newest = *views_of(&a, "UMLPartView").last().unwrap();
    assert!(a.repo.f(newest, "left") < a.repo.f(ports[0], "left"), "inside, away from the right border");
    a.item_key(ports[0], EditKind::Name(pm), ItemKey::SelectType);
    assert!(a.element_picker.is_some());
    a.element_picker = None;
    a.item_key(ports[0], EditKind::Name(pm), ItemKey::CreateType);
    let dialog = a.input_dialog.take().unwrap();
    assert_eq!(dialog.message, "Enter a type name to create");
    a.apply_input(dialog.action, "Plug".into());
    let ty = a.repo.refd(pm, "type").unwrap();
    assert_eq!((a.repo.name(ty), a.repo.parent(ty)), ("Plug".to_string(), Some(cm)), "the new type goes in the classifier");
}

#[test]
fn stereotype_icon_from_a_picture() {
    let ms = measure();
    let dir = std::env::temp_dir().join(format!("umlcraft-icon-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("wide.png");
    image::RgbaImage::from_pixel(1000, 400, image::Rgba([0, 0, 255, 255])).save(&png).unwrap();
    let (data, w, h, ow, oh) = crate::picture::load_icon_picture(&png).unwrap();
    assert_eq!((w, h, ow, oh), (500, 200, 1000, 400), "scaled to 500 on the longer side");
    assert!(data.starts_with("data:image/png;base64,"));
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.package_for(Some(d));
    let st = a.change(|a| a.add_model(pkg, "UMLStereotype"));
    let icon = a.repo.child(st, "icon").unwrap();
    a.change(|a| a.repo.set(icon, "image", json!(json!({ "width": w, "height": h, "data": data }).to_string())));
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 10.0, 10.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    a.change(|a| { a.repo.set_ref(m, "stereotype", Some(st)); a.repo.set(c, "stereotypeDisplay", json!("icon")); });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info[&c].shape, "stereotype-icon");
    let out = crate::render::to_png(&a.repo, d, &a.info, &ms, 1.0).unwrap();
    let img = image::load_from_memory(&out).unwrap().to_rgba8();
    assert!(img.pixels().any(|p| p.0 == [0, 0, 255, 255]), "the picture is drawn");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_port_provides_and_requires_interfaces() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let p = a.change(|a| a.create_feature_box("UMLPort", crate::views::P { x: 400.0, y: 200.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    let ms = measure();
    let d = a.diagram.unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let pm = a.repo.refd(p, "model").unwrap();
    a.item_key(p, EditKind::Name(pm), ItemKey::PortInterface(true));
    let ty = a.repo.refd(pm, "type").expect("the port got a type");
    assert_eq!(a.repo.name(ty), "Port1Type");
    let real = a.repo.all().find(|e| a.repo.ty(*e) == "UMLInterfaceRealization").unwrap();
    assert_eq!(a.repo.refd(real, "source"), Some(pm));
    let iface = a.repo.refd(real, "target").unwrap();
    let iv = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.refd(*v, "model") == Some(iface)).unwrap();
    assert!(a.repo.f(iv, "left") > a.repo.f(c, "left") + a.repo.f(c, "width"), "on the side the port sits on");
    a.item_key(p, EditKind::Name(pm), ItemKey::PortInterface(false));
    assert!(a.repo.all().any(|e| a.repo.ty(e) == "UMLDependency" && a.repo.refd(e, "source") == Some(pm)));
    assert_eq!(a.repo.refd(pm, "type"), Some(ty), "the type is made once");
}

#[test]
fn composite_structure_diagrams_are_editable() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.package_for(Some(d));
    let cs = a.change(|a| a.add_diagram(Some(pkg), "UMLCompositeStructureDiagram"));
    assert_eq!(a.repo.name(cs), "Composite Structures1");
    let collab = a.repo.parent(cs).unwrap();
    assert_eq!((a.repo.ty(collab), a.repo.parent(collab)), ("UMLCollaboration", Some(pkg)), "wrapped in a new collaboration");
    // inside a classifier it goes in directly
    let c = a.change(|a| a.add_model(pkg, "UMLClass"));
    let cs2 = a.change(|a| a.add_diagram(Some(c), "UMLCompositeStructureDiagram"));
    assert_eq!((a.repo.parent(cs2), a.repo.name(cs2)), (Some(c), "CompositeStructureDiagram1".to_string()));
    // its toolbox holds the composite structure tools, and it takes collaboration uses
    let u = a.change(|a| a.add_model(pkg, "UMLCollaborationUse"));
    assert!(a.can_drop_model(u));
    let v = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 200.0, 120.0))).unwrap();
    assert!(a.create_feature_box("UMLPart", crate::views::P { x: 60.0, y: 60.0 }, rect(50.0, 60.0, 80.0, 30.0)).is_ok());
    let _ = v;
}

#[test]
fn states_activities_triggers_and_actions_from_model_add() {
    use crate::app::{SelectOne, SelectPurpose};
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.package_for(Some(d));
    let sm = a.change(|a| a.add_model(pkg, "UMLStateMachine"));
    let region = a.repo.list(sm, "regions")[0];
    let kinds = |a: &App, e| a.add_kinds(e).into_iter().map(|k| k.1).collect::<Vec<_>>();
    assert!(kinds(&a, region).contains(&"UMLState"));
    let st = a.change(|a| a.add_feature(region, "UMLState")).unwrap();
    assert_eq!((a.repo.name(st), a.repo.field_of(st).as_deref()), ("State1".to_string(), Some("vertices")));
    let on_state = kinds(&a, st);
    for k in ["UMLRegion", "entryActivities", "doActivities", "exitActivities"] {
        assert!(on_state.contains(&k), "a state takes {k}");
    }
    // Entry Activity: the kind is asked, then the behaviour goes in entryActivities
    a.select_one_ok(SelectOne { message: String::new(), options: vec![], selected: 1, purpose: SelectPurpose::BehaviorKind(st, "entryActivities") });
    let entry = a.repo.list(st, "entryActivities");
    assert_eq!(entry.len(), 1);
    assert_eq!(a.repo.ty(entry[0]), "UMLActivity");
    // an activity takes actions, an action takes triggers
    let act = a.change(|a| a.add_model(pkg, "UMLActivity"));
    let action = a.change(|a| a.add_feature(act, "UMLAction")).unwrap();
    assert_eq!(a.repo.field_of(action).as_deref(), Some("nodes"));
    let ev = a.change(|a| a.add_feature(action, "UMLEvent")).unwrap();
    assert_eq!(a.repo.field_of(ev).as_deref(), Some("triggers"));
    assert!(a.fields_for(ev).iter().any(|f| f.0 == "kind"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn a_combined_fragment_takes_operands() {
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let pkg = a.package_for(Some(d));
    let cf = a.change(|a| a.repo.create("UMLCombinedFragment", &[("name", json!("Fragment1"))], Some((pkg, "ownedElements"))));
    assert!(a.add_kinds(cf).iter().any(|k| k.1 == "UMLInteractionOperand"));
    let op = a.change(|a| a.add_feature(cf, "UMLInteractionOperand")).unwrap();
    assert_eq!(a.repo.field_of(op).as_deref(), Some("operands"));
    assert!(a.fields_for(op).iter().any(|f| f.0 == "guard"));
}

#[test]
fn a_failing_inline_command_shows_an_error_toast() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let mut a = App::new(None);
    a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let ty = a.change(|a| a.create_node("UMLClass", rect(600.0, 100.0, 100.0, 60.0))).unwrap();
    let part = a.change(|a| a.create_feature_box("UMLPart", crate::views::P { x: 151.0, y: 151.0 }, rect(150.0, 150.0, 120.0, 60.0))).unwrap();
    let pm = a.repo.refd(part, "model").unwrap();
    let tm = a.repo.refd(ty, "model").unwrap();
    a.change(|a| a.repo.set_ref(pm, "type", Some(tm)));
    let port = a.change(|a| a.create_feature_box("UMLPort", crate::views::P { x: 270.0, y: 170.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    assert_eq!(a.repo.refd(port, "containerView"), Some(part));
    // the part loses its type: a connected part can no longer go in it
    a.change(|a| a.repo.set_ref(pm, "type", None));
    let n = a.repo.all().count();
    let m = a.repo.refd(port, "model").unwrap();
    a.item_key(port, EditKind::Name(m), ItemKey::AddConnectedPart);
    let t = a.toasts.last().unwrap();
    assert_eq!((t.0.as_str(), t.2), ("Type should be assigned to have internal Parts", true));
    assert_eq!(a.repo.all().count(), n, "nothing created");
}

#[test]
fn add_constraint_to_a_line() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 140.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 140.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLTemplateBinding", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let undo = a.undo.len();
    let cv = a.add_constraint_to(e).unwrap();
    assert_eq!(a.undo.len(), undo + 1);
    let c = a.repo.refd(cv, "model").unwrap();
    assert_eq!(a.repo.parent(c), a.repo.refd(e, "model"), "owned by the binding");
    let link = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLConstraintLinkView").unwrap();
    assert_eq!((a.repo.refd(link, "tail"), a.repo.refd(link, "head")), (Some(cv), Some(e)));
    let anchor = a.edge_anchor(e);
    assert!(crate::views::node_rect(&a.repo, cv).x < anchor.x, "left of the name label");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn a_rectilinear_bend_point_drags_its_neighbours_along() {
    use crate::views::{move_orthogonal, P};
    // a Z path: right, down, right
    let mut pts = vec![P { x: 0.0, y: 0.0 }, P { x: 50.0, y: 0.0 }, P { x: 50.0, y: 80.0 }, P { x: 120.0, y: 80.0 }];
    move_orthogonal(&mut pts, 1, P { x: 70.0, y: 20.0 });
    assert_eq!(pts[1], P { x: 70.0, y: 20.0 });
    assert_eq!(pts[0], P { x: 0.0, y: 20.0 }, "the horizontal neighbour follows in y");
    assert_eq!(pts[2], P { x: 70.0, y: 80.0 }, "the vertical neighbour follows in x");
    assert_eq!(pts[3], P { x: 120.0, y: 80.0 });
    for w in pts.windows(2) {
        assert!(w[0].x == w[1].x || w[0].y == w[1].y, "still orthogonal");
    }
}

#[test]
fn an_association_end_qualifier_is_drawn_at_its_end() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 100.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 100.0, 100.0, 60.0))).unwrap();
    let tool = crate::app::Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: crate::app::AssocInit::Plain };
    let e = a.change(|a| a.create_edge(&tool, c1, c2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let q = a.repo.refd(e, "tailQualifiersCompartment").unwrap();
    assert_eq!(a.repo.val(q, "visible"), Some(json!(false)), "hidden without qualifiers");
    let m = a.repo.refd(e, "model").unwrap();
    let end1 = a.repo.child(m, "end1").unwrap();
    let undo = a.undo.len();
    let qa = a.change(|a| a.add_qualifier(end1));
    assert_eq!(a.undo.len(), undo + 1);
    assert_eq!(a.repo.list(end1, "qualifiers"), vec![qa]);
    a.change(|a| a.repo.set(qa, "name", json!("id")));
    a.change(|a| a.repo.set(qa, "type", json!("int")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_ne!(a.repo.val(q, "visible"), Some(json!(false)), "shown");
    let b = crate::views::node_rect(&a.repo, q);
    assert_eq!(b.x, 140.0 , "against the right side of the tail box");
    assert!(b.y >= 100.0 && b.y + b.h <= 160.0);
    let pts = crate::views::points_of(&a.repo, e);
    assert_eq!(pts[0].x, b.x + b.w, "the line starts on the qualifier box");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn a_statechart_diagram_with_states_and_a_transition() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let sm = a.repo.parent(d).unwrap();
    assert_eq!(a.repo.ty(sm), "UMLStateMachine");
    let region = a.repo.list(sm, "regions")[0];
    assert_eq!(a.diagram, Some(d));
    let init = a.change(|a| a.create_node("Initial", rect(40.0, 40.0, 20.0, 20.0))).unwrap();
    let s1 = a.change(|a| a.create_node("UMLState", rect(120.0, 30.0, 100.0, 40.0))).unwrap();
    let fin = a.change(|a| a.create_node("UMLFinalState", rect(320.0, 40.0, 26.0, 26.0))).unwrap();
    let ch = a.change(|a| a.create_node("Choice", rect(200.0, 160.0, 60.0, 60.0))).unwrap();
    let (im, s1m) = (a.repo.refd(init, "model").unwrap(), a.repo.refd(s1, "model").unwrap());
    assert_eq!((a.repo.ty(im), a.repo.str(im, "kind").as_str()), ("UMLPseudostate", "initial"));
    assert_eq!(a.repo.list(region, "vertices").len(), 4, "every vertex in the first region");
    let t = tool("UMLTransition", AssocInit::Plain);
    assert!(a.can_connect(&t, init, s1));
    let e1 = a.change(|a| a.create_edge(&t, init, s1)).unwrap();
    let e2 = a.change(|a| a.create_edge(&t, s1, fin)).unwrap();
    assert_eq!(a.repo.ty(e1), "UMLTransitionView");
    let tm = a.repo.refd(e2, "model").unwrap();
    assert_eq!(a.repo.list(region, "transitions").len(), 2);
    // the label: triggers, guard and effect
    let ev = a.change(|a| a.add_feature(tm, "UMLEvent")).unwrap();
    a.change(|a| a.repo.set(ev, "name", json!("done")));
    a.change(|a| a.repo.set(tm, "guard", json!("ok")));
    let ef = a.change(|a| a.repo.create("UMLOpaqueBehavior", &[("name", json!("log"))], Some((tm, "effects"))));
    let _ = ef;
    assert_eq!(crate::views::transition_text(&a.repo, tm), "done [ok] / log");
    let act = a.change(|a| a.repo.create("UMLOpaqueBehavior", &[("name", json!("greet"))], Some((s1m, "entryActivities"))));
    let _ = act;
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l = a.repo.refd(e2, "nameLabel").unwrap();
    assert_eq!(a.repo.str(l, "text"), "done [ok] / log");
    assert_eq!((a.repo.f(ch, "width"), a.repo.f(ch, "height")), (23.0, 19.0), "a point tool: the choice's smallest size");
    assert!(a.repo.f(s1, "height") > crate::views::NAME_MIN_H, "room for the entry activity");
    assert_eq!(a.info[&s1].shape, "state");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn a_transition_expression_is_edited_inline() {
    use crate::parse::{apply_transition, name_text};
    let mut a = App::new(None);
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let s1 = a.change(|a| a.create_node("UMLState", rect(40.0, 40.0, 100.0, 40.0))).unwrap();
    let s2 = a.change(|a| a.create_node("UMLState", rect(300.0, 40.0, 100.0, 40.0))).unwrap();
    let t = tool("UMLTransition", AssocInit::Plain);
    let e = a.change(|a| a.create_edge(&t, s1, s2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    apply_transition(&mut a.repo, m, "start, resume [ready] / go");
    let names = |a: &App, f: &str| a.repo.list(m, f).into_iter().map(|x| a.repo.name(x)).collect::<Vec<_>>();
    assert_eq!(names(&a, "triggers"), ["start", "resume"]);
    assert_eq!(a.repo.str(m, "guard"), "ready");
    assert_eq!(names(&a, "effects"), ["go"]);
    assert_eq!(a.repo.ty(a.repo.list(m, "effects")[0]), "UMLOpaqueBehavior");
    assert_eq!(name_text(&a.repo, m), "start, resume [ready] / go");
    // by position: the first trigger renamed, the second deleted; the guard kept when not given
    let first = a.repo.list(m, "triggers")[0];
    apply_transition(&mut a.repo, m, "begin / stop");
    assert_eq!(a.repo.list(m, "triggers"), vec![first]);
    assert_eq!(names(&a, "triggers"), ["begin"]);
    assert_eq!(a.repo.str(m, "guard"), "ready");
    assert_eq!(names(&a, "effects"), ["stop"]);
    // a self transition: both ends on one state
    let st = a.change(|a| a.create_edge(&t, s1, s1)).unwrap();
    assert_eq!(a.repo.refd(st, "tail"), a.repo.refd(st, "head"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn state_buttons_add_states_with_transitions() {
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    a.prefs.set("uml.state.fillColor", json!("#eeffee"));
    let s = a.change(|a| a.create_node("UMLState", rect(200.0, 200.0, 100.0, 40.0))).unwrap();
    assert_eq!(a.repo.str(s, "fillColor"), "#eeffee", "uml.state.fillColor");
    let sm = a.repo.refd(s, "model").unwrap();
    let act = |l: &str| crate::related::actions("UMLState").into_iter().find(|x| x.label == l).unwrap();
    let ends = |a: &App, n: crate::model::Eid| {
        let e = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLTransitionView" && (a.repo.refd(*v, "tail") == Some(n) || a.repo.refd(*v, "head") == Some(n))).unwrap();
        (a.repo.refd(e, "tail").unwrap(), a.repo.refd(e, "head").unwrap())
    };
    let out = a.add_related(s, act("Add Outgoing Transition")).unwrap();
    assert!(crate::views::node_rect(&a.repo, out).x > 300.0);
    assert_eq!(ends(&a, out), (s, out));
    let init = a.add_related(s, act("Add Initial State")).unwrap();
    let im = a.repo.refd(init, "model").unwrap();
    assert_eq!((a.repo.ty(im), a.repo.str(im, "kind").as_str()), ("UMLPseudostate", "initial"));
    assert!(crate::views::node_rect(&a.repo, init).y < 200.0, "above");
    assert_eq!(ends(&a, init), (init, s));
    let fin = a.add_related(s, act("Add Final State")).unwrap();
    assert!(crate::views::node_rect(&a.repo, fin).y > 240.0, "below");
    assert_eq!(ends(&a, fin), (s, fin));
    assert_eq!(a.repo.parent(im), a.repo.parent(sm), "in the same region");
}

#[test]
fn a_state_adds_a_choice_with_two_branches() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let s = a.change(|a| a.create_node("UMLState", rect(200.0, 100.0, 100.0, 40.0))).unwrap();
    let m = a.repo.refd(s, "model").unwrap();
    let undo = a.undo.len();
    a.item_key(s, EditKind::Name(m), ItemKey::Scaffold("Choice"));
    assert_eq!(a.undo.len(), undo + 1, "one undo step");
    let views = a.repo.list(d, "ownedViews");
    let count = |t: &str| views.iter().filter(|v| a.repo.ty(**v) == t).count();
    assert_eq!((count("UMLPseudostateView"), count("UMLStateView"), count("UMLTransitionView")), (1, 3, 3));
    let p = views.iter().copied().find(|v| a.repo.ty(*v) == "UMLPseudostateView").unwrap();
    assert_eq!(a.repo.str(a.repo.refd(p, "model").unwrap(), "kind"), "choice");
    assert_eq!(a.repo.f(p, "top"), 190.0, "50 px below the state");
    assert_eq!(a.selection, vec![p]);
}

#[test]
fn a_state_machine_dropped_on_a_statechart_is_a_submachine_state() {
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let other = a.change(|a| a.add_model(root, "UMLStateMachine"));
    a.open_diagram(d);
    assert!(a.can_drop_model(other));
    let vs = a.drop_model(other, crate::views::P { x: 100.0, y: 100.0 }).unwrap();
    let s = a.repo.refd(vs[0], "model").unwrap();
    assert_eq!(a.repo.ty(s), "UMLState");
    assert_eq!(a.repo.refd(s, "submachine"), Some(other));
    let sm = a.repo.parent(d).unwrap();
    assert_eq!(a.repo.parent(s), Some(a.repo.list(sm, "regions")[0]));
}

#[test]
fn states_drawn_in_a_composite_state_belong_to_its_region() {
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let top = a.repo.list(a.repo.parent(d).unwrap(), "regions")[0];
    let c = a.change(|a| a.create_node("CompositeState", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    let inner_region = a.repo.list(cm, "regions")[0];
    let s = a.change(|a| a.create_node("UMLState", rect(150.0, 160.0, 100.0, 40.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!(a.repo.refd(s, "containerView"), Some(c));
    assert_eq!(a.repo.parent(sm), Some(inner_region));
    // released on the canvas, it goes back to the state machine's region
    a.change(|a| a.set_container(&[s], None));
    assert_eq!(a.repo.parent(sm), Some(top));
    assert!(a.can_contain_view(c, s));
    // a note is not held by a state
    let n = a.change(|a| a.create_node("Note", rect(120.0, 120.0, 60.0, 40.0))).unwrap();
    assert!(!a.can_contain_view(c, n));
    // a simple state takes a state too, which gives it a region
    let s2 = a.change(|a| a.create_node("UMLState", rect(500.0, 100.0, 200.0, 100.0))).unwrap();
    a.change(|a| a.set_container(&[s], Some(s2)));
    let m2 = a.repo.refd(s2, "model").unwrap();
    assert_eq!(a.repo.parent(sm), a.repo.list(m2, "regions").first().copied());
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn the_submachine_state_tool_makes_its_state_machine() {
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let v = a.change(|a| a.create_node("SubmachineState", rect(100.0, 100.0, 100.0, 40.0))).unwrap();
    let s = a.repo.refd(v, "model").unwrap();
    let sub = a.repo.refd(s, "submachine").unwrap();
    assert_eq!(a.repo.ty(sub), "UMLStateMachine");
    assert_eq!(a.repo.list(sub, "regions").len(), 1);
    assert_ne!(Some(sub), a.repo.parent(d));
}

#[test]
fn an_orthogonal_state_files_states_by_region() {
    let mut a = App::new(None);
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let o = a.change(|a| a.create_node("OrthogonalState", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let om = a.repo.refd(o, "model").unwrap();
    let rs = a.repo.list(om, "regions");
    assert_eq!(rs.len(), 2);
    let up = a.change(|a| a.create_node("UMLState", rect(150.0, 130.0, 80.0, 30.0))).unwrap();
    let down = a.change(|a| a.create_node("UMLState", rect(150.0, 230.0, 80.0, 30.0))).unwrap();
    assert_eq!(a.repo.parent(a.repo.refd(up, "model").unwrap()), Some(rs[0]));
    assert_eq!(a.repo.parent(a.repo.refd(down, "model").unwrap()), Some(rs[1]));
}

#[test]
fn a_state_inline_edit_adds_regions_and_activities() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let mut a = App::new(None);
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let s = a.change(|a| a.create_node("UMLState", rect(100.0, 100.0, 100.0, 40.0))).unwrap();
    let m = a.repo.refd(s, "model").unwrap();
    a.item_key(s, EditKind::Name(m), ItemKey::AddRegion);
    assert_eq!(a.repo.list(m, "regions").len(), 1);
    a.item_key(s, EditKind::Name(m), ItemKey::Activity("doActivities"));
    let d = a.select_one.as_ref().unwrap();
    assert_eq!(d.message, "Select one to create as an do Activity");
    assert!(matches!(d.purpose, crate::app::SelectPurpose::BehaviorKind(x, "doActivities") if x == m));
}

#[test]
fn internal_transitions_need_a_composite_state() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let mut a = App::new(None);
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let s = a.change(|a| a.create_node("UMLState", rect(100.0, 100.0, 100.0, 40.0))).unwrap();
    let m = a.repo.refd(s, "model").unwrap();
    a.item_key(s, EditKind::Name(m), ItemKey::InternalTransition);
    assert_eq!(a.toasts.last().map(|t| t.0.as_str()), Some("Simple State cannot have Internal Transitions."));
    let c = a.change(|a| a.create_node("CompositeState", rect(300.0, 100.0, 200.0, 150.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    let t = a.add_internal_transition(cm).unwrap();
    let rg = a.repo.list(cm, "regions")[0];
    assert_eq!(a.repo.parent(t), Some(rg));
    assert_eq!((a.repo.refd(t, "source"), a.repo.refd(t, "target"), a.repo.str(t, "kind")), (Some(cm), Some(cm), "internal".to_string()));
}

#[test]
fn an_activity_diagram_with_actions_and_flows() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let act = a.repo.parent(d).unwrap();
    assert_eq!(a.repo.ty(act), "UMLActivity");
    assert_eq!(a.repo.name(act), "Activity1");
    let init = a.change(|a| a.create_node("UMLInitialNode", rect(40.0, 40.0, 0.0, 0.0))).unwrap();
    let s1 = a.change(|a| a.create_node("UMLAction", rect(120.0, 30.0, 10.0, 10.0))).unwrap();
    let dec = a.change(|a| a.create_node("UMLDecisionNode", rect(200.0, 160.0, 5.0, 5.0))).unwrap();
    let fork = a.change(|a| a.create_node("UMLForkNode", rect(200.0, 260.0, 120.0, 40.0))).unwrap();
    let send = a.change(|a| a.create_node("SendSignal", rect(300.0, 30.0, 80.0, 40.0))).unwrap();
    let obj = a.change(|a| a.create_node("UMLDataStoreNode", rect(300.0, 130.0, 80.0, 40.0))).unwrap();
    let fin = a.change(|a| a.create_node("UMLActivityFinalNode", rect(320.0, 300.0, 26.0, 26.0))).unwrap();
    assert_eq!(a.repo.list(act, "nodes").len(), 7, "every node in the activity");
    let sm = a.repo.refd(send, "model").unwrap();
    assert_eq!((a.repo.ty(sm), a.repo.str(sm, "kind").as_str()), ("UMLAction", "sendSignal"));
    assert!(a.repo.refd(init, "nameCompartment").is_none(), "a control node has no name");
    let t = tool("UMLControlFlow", AssocInit::Plain);
    assert!(a.can_connect(&t, init, s1));
    let e1 = a.change(|a| a.create_edge(&t, init, s1)).unwrap();
    let e2 = a.change(|a| a.create_edge(&t, s1, fin)).unwrap();
    let of = a.change(|a| a.create_edge(&tool("UMLObjectFlow", AssocInit::Plain), s1, obj)).unwrap();
    assert_eq!(a.repo.ty(e1), "UMLControlFlowView");
    assert_eq!(a.repo.ty(of), "UMLObjectFlowView");
    assert_eq!(a.repo.list(act, "edges").len(), 3, "flows in the activity's edges");
    let fm = a.repo.refd(e2, "model").unwrap();
    a.change(|a| a.repo.set(fm, "guard", json!("ok")));
    a.change(|a| a.repo.set(fm, "weight", json!("2")));
    assert_eq!(crate::views::activity_edge_text(&a.repo, fm), " [ok] {weight=2}");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(init, "width"), a.repo.f(init, "height")), (20.0, 20.0));
    assert_eq!((a.repo.f(dec, "width"), a.repo.f(dec, "height")), (23.0, 19.0));
    assert_eq!((a.repo.f(fork, "width"), a.repo.f(fork, "height")), (120.0, 6.0), "a horizontal bar keeps its thickness");
    assert!(a.repo.f(s1, "width") >= 60.0 && a.repo.f(s1, "height") >= 40.0);
    assert_eq!(a.info[&s1].shape, "action");
    assert_eq!(a.info[&obj].shape, "object-node");
    let nl = a.repo.refd(a.repo.refd(obj, "nameCompartment").unwrap(), "stereotypeLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "«datastore»");
    // an activity dropped on the diagram: an unnamed action calling it
    let other = a.change(|a| a.add_model(root, "UMLActivity"));
    let dropped = a.drop_model(other, crate::views::P { x: 40.0, y: 300.0 }).unwrap();
    let am = a.repo.refd(dropped[0], "model").unwrap();
    assert_eq!(a.repo.parent(am), Some(act));
    assert_eq!(a.repo.refd(am, "subactivity"), Some(other));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nm = a.repo.refd(a.repo.refd(dropped[0], "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nm, "text"), format!(":{}", a.repo.name(other)));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn action_buttons_add_flows_and_scaffolds() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let act = a.repo.parent(d).unwrap();
    let s = a.change(|a| a.create_node("UMLAction", rect(300.0, 300.0, 60.0, 40.0))).unwrap();
    let m = a.repo.refd(s, "model").unwrap();
    let btn = |t: &str, l: &str| crate::related::actions(t).into_iter().find(|x| x.label == l).unwrap();
    let ends = |a: &App, n: crate::model::Eid| {
        let e = a.repo.list(d, "ownedViews").into_iter().find(|v| crate::views::is_edge(&a.repo, *v) && (a.repo.refd(*v, "tail") == Some(n) || a.repo.refd(*v, "head") == Some(n))).unwrap();
        (a.repo.ty(e).to_string(), a.repo.refd(e, "tail").unwrap(), a.repo.refd(e, "head").unwrap())
    };
    let out = a.add_related(s, btn("UMLAction", "Add Outgoing Control Flow")).unwrap();
    assert!(crate::views::node_rect(&a.repo, out).y > 340.0, "below");
    assert_eq!(ends(&a, out), ("UMLControlFlowView".to_string(), s, out));
    let init = a.add_related(s, btn("UMLAction", "Add Initial Node")).unwrap();
    assert_eq!(a.repo.ty(a.repo.refd(init, "model").unwrap()), "UMLInitialNode");
    assert_eq!(ends(&a, init), ("UMLControlFlowView".to_string(), init, s));
    let obj = a.add_related(s, btn("UMLAction", "Add Outgoing Object Flow")).unwrap();
    let om = a.repo.refd(obj, "model").unwrap();
    assert_eq!(a.repo.ty(om), "UMLObjectNode");
    // a control flow from an object node makes it a control type
    a.add_related(obj, btn("UMLObjectNode", "Add Outgoing Control Flow")).unwrap();
    assert!(a.repo.bool(om, "isControlType"));
    // a decision with two branch actions, as one undo step
    let before = a.repo.list(act, "nodes").len();
    let undo = a.undo.len();
    a.item_key(s, EditKind::Name(m), ItemKey::Scaffold("UMLDecisionNode"));
    assert_eq!(a.undo.len(), undo + 1);
    assert_eq!(a.repo.list(act, "nodes").len(), before + 3);
    let dv = a.selection[0];
    assert_eq!(a.repo.ty(a.repo.refd(dv, "model").unwrap()), "UMLDecisionNode");
    assert_eq!((a.repo.f(dv, "left"), a.repo.f(dv, "top")), (300.0, 380.0), "40 px below, level with the left side");
    let join = { a.item_key(s, EditKind::Name(m), ItemKey::Scaffold("UMLJoinNode")); a.selection[0] };
    assert_eq!(a.repo.f(join, "top"), 260.0, "40 px above");
    let into: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.refd(*v, "head") == Some(join)).collect();
    assert_eq!(into.len(), 2, "two actions flow into the join");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn pins_sit_on_their_action_and_take_object_flows() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let s1 = a.change(|a| a.create_node("UMLAction", rect(100.0, 100.0, 80.0, 40.0))).unwrap();
    let s2 = a.change(|a| a.create_node("UMLAction", rect(400.0, 100.0, 80.0, 40.0))).unwrap();
    let (m1, m2) = (a.repo.refd(s1, "model").unwrap(), a.repo.refd(s2, "model").unwrap());
    // the pin tool needs an action under the point
    assert!(a.create_feature_box("UMLInputPin", crate::views::P { x: 10.0, y: 10.0 }, rect(0.0, 0.0, 0.0, 0.0)).is_err());
    let p = a.change(|a| a.create_feature_box("UMLInputPin", crate::views::P { x: 100.0, y: 120.0 }, rect(0.0, 0.0, 0.0, 0.0)).ok()).unwrap();
    assert_eq!(a.repo.ty(p), "UMLInputPinView");
    assert_eq!(a.repo.list(m1, "inputs").len(), 1);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b = crate::views::node_rect(&a.repo, p);
    assert_eq!((b.w, b.h), (18.0, 18.0));
    assert_eq!(b.x + b.w, 109.0, "centred on the left border where it was dropped");
    assert_eq!(crate::views::pin_side(&a.repo, p), "left");
    // an object flow between two actions joins a new output pin to a new input pin
    let of = a.change(|a| a.create_edge(&tool("UMLObjectFlow", AssocInit::Plain), s1, s2)).unwrap();
    let (t, h) = (a.repo.refd(of, "tail").unwrap(), a.repo.refd(of, "head").unwrap());
    assert_eq!((a.repo.ty(t), a.repo.ty(h)), ("UMLOutputPinView", "UMLInputPinView"));
    assert_eq!(a.repo.refd(t, "containerView"), Some(s1));
    assert_eq!(a.repo.list(m1, "outputs").len(), 1);
    assert_eq!(a.repo.list(m2, "inputs").len(), 1);
    let fm = a.repo.refd(of, "model").unwrap();
    assert_eq!(a.repo.ty(a.repo.refd(fm, "source").unwrap()), "UMLOutputPin");
    // the Add Output Pin button puts it on the right border
    let op = a.add_pin(s2, false).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(crate::views::pin_side(&a.repo, op), "right");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn swimlanes_hold_the_activity_nodes_drawn_in_them() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let act = a.repo.parent(d).unwrap();
    let lane = a.change(|a| a.create_node("SwimlaneVert", rect(0.0, 0.0, 10.0, 10.0))).unwrap();
    let hl = a.change(|a| a.create_node("SwimlaneHorz", rect(0.0, 400.0, 10.0, 10.0))).unwrap();
    let pm = a.repo.refd(lane, "model").unwrap();
    assert_eq!((a.repo.ty(pm), a.repo.name(pm).as_str()), ("UMLActivityPartition", "ActivityPartition1"));
    assert_eq!(a.repo.list(act, "groups").len(), 2);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(lane, "width"), a.repo.f(lane, "height")), (crate::views::LANE_THIN.max(a.repo.f(lane, "width")), 300.0));
    assert_eq!(a.repo.f(hl, "width"), 300.0);
    assert!(!a.repo.bool(hl, "isVertical"));
    // an action drawn in the lane belongs to its partition; moved out, back to the activity
    let s = a.change(|a| a.create_node("UMLAction", rect(5.0, 50.0, 60.0, 40.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!(a.repo.parent(sm), Some(pm));
    assert_eq!(a.repo.refd(s, "containerView"), Some(lane));
    assert!(a.can_contain_view(lane, s));
    a.change(|a| a.set_container(&[s], None));
    assert_eq!(a.repo.parent(sm), Some(act));
    assert_eq!(a.repo.field_of(sm).as_deref(), Some("nodes"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("rotate(-90"), "a horizontal lane's name reads upwards");
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn advanced_activity_tools_make_their_elements() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let act = a.repo.parent(d).unwrap();
    // a structured activity holds the actions drawn in it, in its own nodes
    let sv = a.change(|a| a.create_node("UMLStructuredActivityNode", rect(0.0, 0.0, 300.0, 200.0))).unwrap();
    let sm = a.repo.refd(sv, "model").unwrap();
    assert_eq!(a.repo.ty(sv), "UMLStructuredActivityNodeView");
    let inner = a.change(|a| a.create_node("UMLAction", rect(20.0, 60.0, 60.0, 40.0))).unwrap();
    let im = a.repo.refd(inner, "model").unwrap();
    assert_eq!(a.repo.parent(im), Some(sm));
    assert_eq!(a.repo.field_of(im).as_deref(), Some("nodes"));
    // an expansion region shows its mode; expansion nodes sit on its border
    let ev = a.change(|a| a.create_node("UMLExpansionRegion", rect(400.0, 0.0, 200.0, 150.0))).unwrap();
    let em = a.repo.refd(ev, "model").unwrap();
    assert_eq!(a.repo.str(em, "mode"), "iterative");
    let en = a.change(|a| a.create_feature_box("UMLInputExpansionNode", crate::views::P { x: 500.0, y: 0.0 }, rect(0.0, 0.0, 0.0, 0.0)).ok()).unwrap();
    assert_eq!(a.repo.ty(en), "UMLExpansionNodeView");
    let enm = a.repo.refd(en, "model").unwrap();
    assert_eq!((a.repo.ty(enm), a.repo.name(enm).as_str()), ("UMLExpansionNode", ""));
    assert_eq!(a.repo.list(em, "inputs"), vec![enm]);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b = crate::views::node_rect(&a.repo, en);
    assert_eq!(crate::views::pin_side(&a.repo, en), "top");
    assert_eq!((b.w, b.h), (72.0, 18.0), "four cells along the top border");
    let st = a.repo.refd(ev, "nameCompartment").and_then(|nc| a.repo.refd(nc, "stereotypeLabel")).unwrap();
    assert_eq!(a.repo.str(st, "text"), "«iterative»");
    // an interruptible region is a group; an edge connector is named "A"
    let iv = a.change(|a| a.create_node("UMLInterruptibleActivityRegion", rect(0.0, 300.0, 10.0, 10.0))).unwrap();
    let ig = a.repo.refd(iv, "model").unwrap();
    assert!(a.repo.list(act, "groups").contains(&ig));
    let cv = a.change(|a| a.create_node("UMLActivityEdgeConnector", rect(700.0, 300.0, 26.0, 26.0))).unwrap();
    assert_eq!(a.repo.name(a.repo.refd(cv, "model").unwrap()), "A");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(iv, "width"), a.repo.f(iv, "height")), (30.0, 30.0));
    // an exception handler ends on a new input pin of the handler and carries a zig-zag
    let h1 = a.change(|a| a.create_node("UMLAction", rect(100.0, 500.0, 60.0, 40.0))).unwrap();
    let h2 = a.change(|a| a.create_node("UMLAction", rect(400.0, 500.0, 60.0, 40.0))).unwrap();
    let t = tool("UMLExceptionHandler", AssocInit::Plain);
    assert!(a.can_connect(&t, h1, h2));
    let eh = a.change(|a| a.create_edge(&t, h1, h2)).unwrap();
    let ehm = a.repo.refd(eh, "model").unwrap();
    assert_eq!(a.repo.ty(eh), "UMLExceptionHandlerView");
    assert_eq!(a.repo.refd(ehm, "handlerBody"), a.repo.refd(h2, "model"));
    assert_eq!(a.repo.ty(a.repo.refd(ehm, "target").unwrap()), "UMLInputPin");
    assert_eq!(a.repo.ty(a.repo.refd(eh, "head").unwrap()), "UMLInputPinView");
    assert!(a.repo.list(act, "edges").contains(&ehm));
    let ai = a.change(|a| a.create_edge(&tool("UMLActivityInterrupt", AssocInit::Plain), h1, h2)).unwrap();
    assert_eq!(a.repo.ty(a.repo.refd(ai, "model").unwrap()), "UMLActivityInterrupt");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let z = a.repo.refd(ai, "adornment").unwrap();
    assert_eq!((a.repo.ty(z), a.repo.f(z, "width"), a.repo.f(z, "height")), ("UMLZigZagAdornmentView", 25.0, 20.0));
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("A</text>"));
    let text = a.repo.serialize();
    Repo::parse(&text).unwrap();
}

#[test]
fn activity_boxes_show_called_operations_and_states() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let class = a.repo.create("UMLClass", &[("name", serde_json::json!("Order"))], Some((root, "ownedElements")));
    let op = a.repo.create("UMLOperation", &[("name", serde_json::json!("place"))], Some((class, "operations")));
    let v = a.change(|a| a.create_node("UMLAction", rect(0.0, 0.0, 60.0, 40.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    a.repo.set(m, "kind", serde_json::json!("callOperation"));
    let opr = a.repo.ref_to(op);
    a.repo.set(m, "target", opr);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nc = a.repo.refd(v, "nameCompartment").unwrap();
    let ns = a.repo.refd(nc, "namespaceLabel").unwrap();
    assert_eq!(a.repo.str(ns, "text"), "(Order::place)");
    assert_ne!(a.repo.raw(ns, "visible"), Some(&serde_json::json!(false)));
    // unnamed, it shows the operation's name
    a.repo.set(m, "name", serde_json::json!(""));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nm = a.repo.refd(nc, "nameLabel").unwrap();
    assert_eq!(a.repo.str(nm, "text"), "place");
    // an object node in states
    let o = a.change(|a| a.create_node("UMLObjectNode", rect(200.0, 0.0, 60.0, 40.0))).unwrap();
    let om = a.repo.refd(o, "model").unwrap();
    let st = a.repo.create("UMLState", &[("name", serde_json::json!("Paid"))], Some((root, "ownedElements")));
    let sr = a.repo.ref_to(st);
    a.repo.set(om, "inStates", serde_json::json!([sr]));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let ons = a.repo.refd(a.repo.refd(o, "nameCompartment").unwrap(), "namespaceLabel").unwrap();
    assert_eq!(a.repo.str(ons, "text"), "[Paid]");
}

#[test]
fn connection_point_references_sit_on_their_state() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let s = a.change(|a| a.create_node("SubmachineState", rect(100.0, 100.0, 120.0, 60.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    let sub = a.repo.refd(sm, "submachine").unwrap();
    // the tool needs a state under the point
    assert!(a.create_feature_box("UMLConnectionPointReference", crate::views::P { x: 10.0, y: 10.0 }, rect(0.0, 0.0, 0.0, 0.0)).is_err());
    let cp = a.change(|a| a.create_feature_box("UMLConnectionPointReference", crate::views::P { x: 100.0, y: 130.0 }, rect(0.0, 0.0, 0.0, 0.0)).ok()).unwrap();
    let cm = a.repo.refd(cp, "model").unwrap();
    assert_eq!(a.repo.ty(cp), "UMLConnectionPointReferenceView");
    assert_eq!(a.repo.list(sm, "connections"), vec![cm]);
    // it shows the names of the exit points it refers to, with an X
    let ex = a.repo.create("UMLPseudostate", &[("name", json!("abort")), ("kind", json!("exitPoint"))], Some((a.repo.list(sub, "regions")[0], "vertices")));
    let er = a.repo.ref_to(ex);
    a.repo.set(cm, "exit", json!([er]));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b = crate::views::node_rect(&a.repo, cp);
    assert_eq!((b.w, b.h), (14.0, 14.0));
    assert_eq!(b.x + 7.0, 100.0, "centred on the left border");
    let nl = a.repo.refd(cp, "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "abort");
    // the submachine state shows "name: Machine" and keeps room for its mark
    let nm = a.repo.refd(a.repo.refd(s, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nm, "text"), format!("{}: {}", a.repo.name(sm), a.repo.name(sub)));
    // pseudostates are unnamed and take their kind's smallest size
    let j = a.change(|a| a.create_node("Junction", rect(400.0, 100.0, 60.0, 60.0))).unwrap();
    let h = a.change(|a| a.create_node("DeepHistory", rect(400.0, 200.0, 10.0, 10.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(j, "width"), a.repo.f(h, "width")), (15.0, 26.0));
    assert_eq!(a.repo.name(a.repo.refd(j, "model").unwrap()), "");
    assert!(a.repo.refd(j, "nameCompartment").is_none());
    // Add Join: a join above the state with two states above it, all wired by transitions
    let target = a.change(|a| a.create_node("UMLState", rect(400.0, 400.0, 100.0, 40.0))).unwrap();
    let jv = a.add_scaffold(target, "Join").unwrap();
    let jm = a.repo.refd(jv, "model").unwrap();
    assert_eq!(a.repo.str(jm, "kind"), "join");
    assert!(a.repo.f(jv, "top") < 400.0);
    assert_eq!((a.repo.f(jv, "width"), a.repo.f(jv, "height")), (70.0, 6.0));
    let region = a.repo.parent(jm).unwrap();
    let into_join = a.repo.list(region, "transitions").into_iter().filter(|t| a.repo.refd(*t, "target") == Some(jm)).count();
    assert_eq!(into_join, 2);
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn flowcharts_have_every_shape_and_flows() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "FCFlowchartDiagram"));
    let fc = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(fc), a.repo.name(fc).as_str(), a.repo.name(d).as_str()), ("FCFlowchart", "Flowchart1", "FlowchartDiagram1"));
    // a second diagram on the flowchart goes in it
    let d2 = a.change(|a| a.add_diagram(Some(fc), "FCFlowchartDiagram"));
    assert_eq!((a.repo.parent(d2), a.repo.name(d2).as_str()), (Some(fc), "FlowchartDiagram2"));
    a.open_diagram(d);
    let mut views = vec![];
    for (i, (_, ty, _)) in crate::flowchart::NODES.iter().enumerate().filter(|(_, n)| n.1 != "FCFlow") {
        let (x, y) = ((i % 6) as f64 * 160.0, (i / 6) as f64 * 120.0);
        let v = a.change(|a| a.create_node(ty, rect(x, y, 10.0, 10.0))).unwrap();
        assert_eq!(a.repo.ty(v), format!("{ty}View"));
        views.push(v);
    }
    use crate::views::Measure;
    let name = |a: &App, v: crate::model::Eid| a.repo.name(a.repo.refd(v, "model").unwrap());
    assert_eq!(name(&a, views[0]), "Process1");
    assert_eq!(name(&a, views[11]), "Input1", "a manual input is named Input…");
    assert_eq!(name(&a, views[15]), "C1");
    assert_eq!(name(&a, views[17]), "", "an or has no name");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // a process: its name plus 10 px all round; a decision twice its name; an or stays round
    let l = a.repo.refd(views[0], "nameLabel").unwrap();
    assert_eq!(a.repo.str(l, "text"), "Process1");
    assert_eq!(a.repo.f(views[0], "width"), (ms.width("Process1", 0)).ceil() + 20.0);
    assert_eq!(a.repo.f(views[2], "width"), 50f64.max((ms.width("Decision1", 0)).ceil() * 2.0));
    assert_eq!((a.repo.f(views[17], "width"), a.repo.f(views[17], "height")), (30.0, 30.0));
    // a flow between two nodes, owned by its source, with a filled arrow
    let t = tool("FCFlow", AssocInit::Plain);
    assert!(a.can_connect(&t, views[0], views[2]));
    let f = a.change(|a| a.create_edge(&t, views[0], views[2])).unwrap();
    let fm = a.repo.refd(f, "model").unwrap();
    assert_eq!((a.repo.ty(f), a.repo.parent(fm)), ("FCFlowView", a.repo.refd(views[0], "model")));
    assert!(a.repo.refd(f, "stereotypeLabel").is_none());
    // Add Outgoing Decision below a process
    let b = crate::related::actions("FCProcess").into_iter().find(|b| b.label == "Add Outgoing Decision").unwrap();
    let nv = a.add_related(views[0], b).unwrap();
    assert_eq!(a.repo.ty(nv), "FCDecisionView");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("Process1"));
    // the flowchart preferences: a node's own fill colour, the flow's line style
    assert_eq!(a.repo.val(f, "lineStyle").and_then(|x| x.as_i64()), Some(2), "rounded rectilinear by default");
    a.prefs.set("flowchart.decision.fillColor", json!("#ffeeaa"));
    a.prefs.set("uml.action.fillColor", json!("#abcdef"));
    let dv = a.change(|a| a.create_node("FCDecision", rect(0.0, 800.0, 10.0, 10.0))).unwrap();
    assert_eq!(a.repo.str(dv, "fillColor"), "#ffeeaa");
    // a node dragged from the explorer onto another flowchart diagram gets a second view there,
    // with its flows to nodes already shown
    a.open_diagram(d2);
    let pm = a.repo.refd(views[0], "model").unwrap();
    let dm = a.repo.refd(views[2], "model").unwrap();
    a.drop_model(dm, crate::views::P { x: 300.0, y: 300.0 }).unwrap();
    let got = a.drop_model(pm, crate::views::P { x: 100.0, y: 100.0 }).unwrap();
    assert_eq!(a.repo.ty(got[0]), "FCProcessView");
    assert!(a.repo.list(d2, "ownedViews").iter().any(|v| a.repo.ty(*v) == "FCFlowView"));
    let text = a.repo.serialize();
    Repo::parse(&text).unwrap();
}

#[test]
fn data_flow_diagrams_and_mind_maps() {
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "DFDDiagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(model), a.repo.name(model).as_str(), a.repo.name(d).as_str()), ("DFDDataFlowModel", "DataFlowModel1", "DFDDiagram1"));
    let e = a.change(|a| a.create_node("DFDExternalEntity", rect(0.0, 0.0, 1.0, 1.0))).unwrap();
    let p = a.change(|a| a.create_node("DFDProcess", rect(200.0, 0.0, 1.0, 1.0))).unwrap();
    let s = a.change(|a| a.create_node("DFDDataStore", rect(400.0, 0.0, 1.0, 1.0))).unwrap();
    let pm = a.repo.refd(p, "model").unwrap();
    assert_eq!(a.repo.name(pm), "Process1");
    a.repo.set(pm, "id", json!("1.0"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // a process: the id compartment (5 + id + 5), then the name with 10 px around it
    let lh = ms.size();
    assert_eq!(a.repo.f(p, "height"), 5.0 + lh + 5.0 + 10.0 + lh + 10.0);
    assert_eq!(a.repo.str(a.repo.refd(p, "idLabel").unwrap(), "text"), "1.0");
    assert_eq!(a.repo.f(e, "width"), ms.width("ExternalEntity1", 0).ceil() + 20.0);
    assert!(a.repo.refd(e, "idLabel").is_none(), "an external entity does not save its id label");
    let t = tool("DFDDataFlow", AssocInit::Plain);
    assert!(a.can_connect(&t, e, p) && a.can_connect(&t, p, s));
    let f = a.change(|a| a.create_edge(&t, e, p)).unwrap();
    assert_eq!(a.repo.ty(f), "DFDDataFlowView");
    assert_eq!(a.repo.val(f, "lineStyle").and_then(|x| x.as_i64()), Some(2));
    let b = crate::related::actions("DFDProcess").into_iter().find(|b| b.label == "Add Outgoing Data Store").unwrap();
    let nv = a.add_related(p, b).unwrap();
    assert_eq!(a.repo.ty(nv), "DFDDataStoreView");
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("1.0"));
    // a mind map: nodes named MMNode1…, edges without arrows, Add Node Right
    let md = a.change(|a| a.add_diagram(Some(root), "MMMindmapDiagram"));
    let mm = a.repo.parent(md).unwrap();
    assert_eq!((a.repo.ty(mm), a.repo.name(mm).as_str(), a.repo.name(md).as_str()), ("MMMindmap", "Mindmap1", "MMMindmapDiagram1"));
    let n = a.change(|a| a.create_node("MMNode", rect(100.0, 100.0, 1.0, 1.0))).unwrap();
    assert_eq!(a.repo.name(a.repo.refd(n, "model").unwrap()), "MMNode1");
    let b = crate::related::actions("MMNode").into_iter().find(|b| b.label == "Add Node Right").unwrap();
    let n2 = a.add_related(n, b).unwrap();
    assert!(a.repo.f(n2, "left") > a.repo.f(n, "left"));
    let edge = a.repo.list(md, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "MMEdgeView").unwrap();
    assert_eq!(a.repo.refd(edge, "tail"), Some(n2), "the edge runs from the new node");
    assert_eq!(a.repo.val(edge, "lineStyle").and_then(|x| x.as_i64()), Some(1));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, md);
    assert_eq!(a.repo.f(n, "width"), ms.width("MMNode1", 0).ceil() + 16.0);
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn er_diagrams_have_entities_columns_and_crows_feet() {
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "ERDDiagram"));
    let dm = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(dm), a.repo.name(dm).as_str(), a.repo.name(d).as_str()), ("ERDDataModel", "Data Model1", "ERDDiagram1"));
    let e1 = a.change(|a| a.create_node("ERDEntity", rect(0.0, 0.0, 1.0, 1.0))).unwrap();
    let e2 = a.change(|a| a.create_node("ERDEntity", rect(300.0, 0.0, 1.0, 1.0))).unwrap();
    let m1 = a.repo.refd(e1, "model").unwrap();
    assert_eq!(a.repo.name(m1), "Entity1");
    // Add Column (Ctrl+Enter) and its expression "name: type(length)"
    let c = a.change(|a| a.add_feature(m1, "ERDColumn")).unwrap();
    assert_eq!(a.repo.name(c), "Column1");
    crate::erd::apply_column(&mut a.repo, c, "id : VARCHAR(20)");
    assert_eq!((a.repo.name(c).as_str(), a.repo.str(c, "type").as_str(), crate::erd::column_text(&a.repo, c).as_str()), ("id", "VARCHAR", "id: VARCHAR(20)"));
    a.repo.set(c, "primaryKey", json!(true));
    let c2 = a.change(|a| a.add_feature(m1, "ERDColumn")).unwrap();
    crate::erd::apply_column(&mut a.repo, c2, "note");
    a.repo.set(c2, "nullable", json!(true));
    assert_eq!((crate::erd::key_text(&a.repo, c).as_str(), crate::erd::key_text(&a.repo, c2).as_str()), ("PK", "N"));
    assert_eq!(a.repo.raw(c2, "length"), Some(&json!(0)), "no length sets it to 0, as the original does");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let cc = a.repo.refd(e1, "columnCompartment").unwrap();
    assert_eq!(a.repo.list(cc, "subViews").len(), 2);
    assert_eq!(a.info[&e1].compartments[0].rows.len(), 2, "the columns are rows the inline edit walks");
    let lh = ms.size();
    assert_eq!(a.repo.f(e1, "height"), 5.0 + lh + 5.0 + 5.0 + 2.0 * lh + 2.0 + 5.0);
    // a one-to-many relationship: the second end is 0..*
    let t = tool("ERDRelationshipOneToMany", AssocInit::Plain);
    assert!(a.can_connect(&t, e1, e2));
    let rv = a.change(|a| a.create_edge(&t, e1, e2)).unwrap();
    let rm = a.repo.refd(rv, "model").unwrap();
    assert_eq!(a.repo.ty(rv), "ERDRelationshipView");
    assert_eq!(a.repo.str(a.repo.child(rm, "end2").unwrap(), "cardinality"), "0..*");
    assert_eq!(a.repo.str(a.repo.child(rm, "end1").unwrap(), "cardinality"), "1");
    assert!(a.repo.refd(rv, "tailNameLabel").is_some());
    // Add Many to Many on an entity's quick edit
    let b = crate::related::actions("ERDEntity").into_iter().find(|b| b.label == "Add Many to Many").unwrap();
    let nv = a.add_related(e2, b).unwrap();
    assert_eq!(a.repo.ty(nv), "ERDEntityView");
    // suppress columns hides the rows
    a.repo.set(e1, "suppressColumns", json!(true));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.info[&e1].compartments[0].rows.is_empty());
    assert_eq!(a.repo.f(e1, "height") >= 5.0 + lh + 5.0, true);
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("Entity1"));
    // ERD001: a nullable primary key; ERD002: a foreign key to an entity without a relationship
    a.repo.set(c, "nullable", json!(true));
    let m3 = a.repo.refd(nv, "model").unwrap();
    let fk = a.change(|a| a.add_feature(m3, "ERDColumn")).unwrap();
    a.repo.set(fk, "foreignKey", json!(true));
    let cr = a.repo.ref_to(c);
    a.repo.set(fk, "referenceTo", cr);
    let rules: Vec<&str> = crate::validate::validate(&a.repo).into_iter().map(|p| p.rule).collect();
    assert!(rules.contains(&"ERD001") && rules.contains(&"ERD002"));
    // File › New From Template › Data Model
    let t = crate::templates::TEMPLATES.iter().find(|t| t.label == "Data Model").unwrap();
    let p = crate::templates::project(t);
    let dms: Vec<_> = p.children(p.root).into_iter().map(|(_, c)| c).collect();
    assert_eq!(dms.len(), 1, "no UML profile");
    assert_eq!((p.ty(dms[0]), p.name(dms[0]).as_str()), ("ERDDataModel", "Data Model"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn c4_diagrams_have_people_systems_containers_and_boundaries() {
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "C4Diagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(model), a.repo.name(model).as_str(), a.repo.name(d).as_str()), ("C4Model", "C4Model1", "C4Diagram1"));
    let p = a.change(|a| a.create_node("C4Person", rect(0.0, 0.0, 1.0, 1.0))).unwrap();
    let s = a.change(|a| a.create_node("C4SoftwareSystem", rect(300.0, 0.0, 1.0, 1.0))).unwrap();
    let db = a.change(|a| a.create_node("C4ContainerDatabase", rect(600.0, 0.0, 1.0, 1.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!(a.repo.name(sm), "C4SoftwareSystem1");
    let dbm = a.repo.refd(db, "model").unwrap();
    assert_eq!((a.repo.ty(dbm), a.repo.str(dbm, "kind").as_str()), ("C4Container", "database"));
    a.repo.set(sm, "technology", json!("Rust"));
    a.repo.set(sm, "description", json!("Keeps the books"));
    assert_eq!(a.repo.str(p, "fillColor"), "#38bcff", "the C4 preference's default colour");
    a.repo.set(s, "width", json!(300));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(s, "technologyLabel").unwrap(), "text"), "[Software System: Rust]");
    let lh = ms.size();
    // padding 8, name, 2, technology (0.9 size), 2, description, 8, plus the description margin 8
    let h = 8.0 + lh + 2.0 + lh * 0.9 + 2.0 + lh + 8.0 + 8.0;
    assert!((a.repo.f(s, "height") - h).abs() < 0.01, "{} vs {h}", a.repo.f(s, "height"));
    // a dotted relationship with its description and technology
    let t = tool("C4Relationship", AssocInit::Plain);
    assert!(a.can_connect(&t, p, s));
    let rv = a.change(|a| a.create_edge(&t, p, s)).unwrap();
    let rm = a.repo.refd(rv, "model").unwrap();
    a.repo.set(rm, "description", json!("Uses"));
    a.repo.set(rm, "technology", json!("HTTPS"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(rv, "technologyLabel").unwrap(), "text"), "[HTTPS]");
    // a diagram inside the software system shows it as a dashed boundary without its description
    let inner = a.change(|a| a.add_diagram(Some(sm), "C4Diagram"));
    assert_eq!(a.repo.parent(inner), Some(sm));
    a.drop_model(sm, crate::views::P { x: 10.0, y: 10.0 }).unwrap();
    let bv = a.repo.list(inner, "ownedViews")[0];
    assert!(crate::c4::is_boundary(&a.repo, bv));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, inner);
    assert_eq!(a.repo.raw(a.repo.refd(bv, "descriptionLabel").unwrap(), "visible"), Some(&json!(false)));
    let t = crate::templates::TEMPLATES.iter().find(|t| t.label == "C4 Model").unwrap();
    let pr = crate::templates::project(t);
    assert_eq!(pr.children(pr.root).len(), 1);
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn activity_boxes_take_their_fill_colour_preferences() {
    let mut a = App::new(None);
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    a.prefs.set("uml.action.fillColor", json!("#abcdef"));
    a.prefs.set("uml.objectnode.fillColor", json!("#fedcba"));
    let act = a.change(|a| a.create_node("UMLAction", rect(0.0, 0.0, 60.0, 40.0))).unwrap();
    let ds = a.change(|a| a.create_node("UMLDataStoreNode", rect(200.0, 0.0, 60.0, 40.0))).unwrap();
    assert_eq!((a.repo.str(act, "fillColor").as_str(), a.repo.str(ds, "fillColor").as_str()), ("#abcdef", "#fedcba"));
}

#[test]
fn mermaid_makes_flowcharts_er_diagrams_mind_maps_and_statecharts() {
    use crate::mermaid_kinds::*;
    // flowchart parsing: shapes, chains, "&", link texts in both forms
    let fc = parse_flowchart("flowchart LR\n  A([Start]) --> B{Is it?}\n  B -->|Yes| C[OK] --> D[(Db)]\n  B -- No --> E[/In/] & F[\\\\Out/]\n  style A fill:#f00").unwrap();
    assert_eq!(fc.direction, "LR");
    let shape = |id: &str| fc.nodes.iter().find(|n| n.id == id).map(|n| n.shape).unwrap();
    assert_eq!((shape("A"), shape("B"), shape("C"), shape("D"), shape("E"), shape("F")), ("stadium", "diamond", "square", "cylinder", "lean_right", "inv_trapezoid"));
    assert!(fc.edges.contains(&("B".into(), "C".into(), "Yes".into())));
    assert!(fc.edges.contains(&("C".into(), "D".into(), String::new())));
    assert!(fc.edges.contains(&("B".into(), "E".into(), "No".into())) && fc.edges.contains(&("B".into(), "F".into(), "No".into())));
    assert_eq!((fc_type("cylinder"), fc_type("inv_trapezoid"), fc_type("odd")), ("FCDatabase", "FCManualOperation", "FCProcess"));
    // ER parsing
    let er = parse_er(ER_SAMPLE).unwrap();
    assert_eq!(er.entities.len(), 3);
    let member = er.entities.iter().find(|e| e.id == "MEMBER").unwrap();
    assert_eq!(member.columns[2].keys, vec!["UK".to_string()]);
    assert_eq!((er.rels[0].card_a, er.rels[0].card_b, er.rels[0].identifying, er.rels[1].label.as_str()), ("1", "0..*", true, "is lent in"));
    // mind map parsing: the tree by indentation, texts out of their shapes
    let mm = parse_mindmap(MINDMAP_SAMPLE).unwrap();
    assert_eq!((mm.text.as_str(), mm.children.len(), mm.children[1].children[1].text.as_str()), ("Library", 3, "DVDs"));
    // state parsing
    let sd = parse_state("stateDiagram-v2\n  direction LR\n  [*] --> A\n  state C <<choice>>\n  A --> C : go\n  C --> [*]\n  note right of A : first").unwrap();
    assert_eq!(sd.direction, "LR");
    assert_eq!(sd.nodes.iter().map(|n| n.shape).collect::<Vec<_>>(), vec!["stateStart", "rect", "choice", "stateEnd"]);
    assert_eq!(sd.notes, vec![("A".to_string(), "first".to_string())]);
    // generation, each as one undo step
    let ms = measure();
    let mut a = App::new(None);
    for (code, dty, holder) in [(FLOWCHART_SAMPLE, "FCFlowchartDiagram", "Flowchart by Mermaid"), (ER_SAMPLE, "ERDDiagram", "Data Model by Mermaid"), (MINDMAP_SAMPLE, "MMMindmapDiagram", "Mindmap by Mermaid"), (STATE_SAMPLE, "UMLStatechartDiagram", "Statemachine by Mermaid")] {
        let before = a.undo.len();
        let d = a.generate_mermaid(code, &ms).unwrap();
        assert_eq!(a.repo.ty(d), dty);
        assert_eq!(a.repo.name(a.repo.parent(d).unwrap()), holder);
        assert_eq!(a.undo.len(), before + 1);
        assert!(a.repo.list(d, "ownedViews").len() > 3);
    }
    let er = a.repo.all().find(|e| a.repo.ty(*e) == "ERDEntity" && a.repo.name(*e) == "LOAN").unwrap();
    assert_eq!(a.repo.list(er, "columns").len(), 4);
    assert!(a.generate_mermaid("sequenceDiagram\n A->>B: hi", &ms).is_err());
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn sequence_diagrams_have_lifelines_messages_and_activations() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLSequenceDiagram"));
    let inter = a.repo.parent(d).unwrap();
    let collab = a.repo.parent(inter).unwrap();
    assert_eq!((a.repo.ty(inter), a.repo.ty(collab)), ("UMLInteraction", "UMLCollaboration"));
    assert_eq!(a.repo.ty(a.repo.list(d, "ownedViews")[0]), "UMLFrameView", "a new sequence diagram has a frame");
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 100.0, 60.0))).unwrap();
    let l2 = a.change(|a| a.create_node("UMLLifeline", rect(250.0, 300.0, 100.0, 60.0))).unwrap();
    let m1 = a.repo.refd(l1, "model").unwrap();
    assert_eq!((a.repo.name(m1).as_str(), a.repo.name(a.repo.refd(m1, "represent").unwrap()).as_str()), ("Lifeline1", "Role1"));
    assert_eq!(a.repo.parent(a.repo.refd(m1, "represent").unwrap()), Some(collab));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(l1, "top"), a.repo.f(l2, "top")), (40.0, 40.0), "lifelines stand at the top");
    // a call from the first to the second at y = 150, and a reply under it
    let call = tool("UMLMessage", AssocInit::Plain);
    assert!(a.can_connect(&call, l1, l2));
    a.line_from = Some((crate::views::P { x: 100.0, y: 150.0 }, crate::views::P { x: 300.0, y: 150.0 }));
    let e1 = a.change(|a| a.create_edge(&call, l1, l2)).unwrap();
    a.line_from = Some((crate::views::P { x: 300.0, y: 200.0 }, crate::views::P { x: 100.0, y: 200.0 }));
    let e2 = a.change(|a| a.create_edge(&tool("UMLReplyMessage", AssocInit::Plain), l2, l1)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let p1 = crate::views::points_of(&a.repo, e1);
    let lp2 = a.repo.refd(l2, "linePart").unwrap();
    let act = a.repo.refd(e1, "activation").unwrap();
    assert_eq!(a.repo.refd(e1, "head"), Some(lp2));
    assert_eq!((p1[0].y, p1[1].y), (150.0, 150.0));
    assert_eq!(p1[1].x, crate::views::node_rect(&a.repo, act).x, "the call ends at the activation it starts");
    assert_eq!(crate::views::node_rect(&a.repo, act).y, 150.0);
    let mm1 = a.repo.refd(e1, "model").unwrap();
    assert_eq!(a.repo.str(a.repo.refd(e1, "nameLabel").unwrap(), "text"), "1 : Message1");
    assert_eq!(a.repo.str(a.repo.refd(e2, "model").unwrap(), "messageSort"), "reply");
    // moving the reply above the call renumbers them
    a.repo.set(e2, "points", json!("300:120;100:120"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.list(inter, "messages")[1], mm1);
    // a self message is four points to the right of its lifeline
    a.line_from = Some((crate::views::P { x: 100.0, y: 250.0 }, crate::views::P { x: 100.0, y: 250.0 }));
    let e3 = a.change(|a| a.create_edge(&tool("UMLSelfMessage", AssocInit::Plain), l1, l1)).unwrap();
    // a lost message ends at a new endpoint
    a.line_from = Some((crate::views::P { x: 300.0, y: 300.0 }, crate::views::P { x: 500.0, y: 300.0 }));
    let e4 = a.change(|a| a.create_edge(&tool("UMLLostMessage", AssocInit::Plain), l2, l2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(crate::views::points_of(&a.repo, e3).len(), 4);
    let ep = a.repo.refd(e4, "head").unwrap();
    assert_eq!(a.repo.ty(ep), "UMLEndpointView");
    assert_eq!(a.repo.list(inter, "participants").len(), 3);
    // the lifelines reach past their lowest message
    let lb = crate::views::node_rect(&a.repo, l2);
    assert!(lb.y + lb.h >= 300.0 + 15.0);
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("Lifeline1"));
    // the lifeline and message expressions
    let ty = crate::parse::apply_lifeline(&mut a.repo, m1, "<<boundary>> ui[1] : Screen");
    assert_eq!((a.repo.name(m1).as_str(), a.repo.str(m1, "selector").as_str(), ty.as_deref()), ("ui", "1", Some("Screen")));
    crate::parse::apply_message(&mut a.repo, mm1, "total = sum(a, b)");
    assert_eq!((a.repo.name(mm1).as_str(), a.repo.str(mm1, "assignmentTarget").as_str(), a.repo.str(mm1, "arguments").as_str()), ("sum", "total", "a, b"));
    assert_eq!(crate::parse::message_text(&a.repo, mm1), "total = sum(a, b)");
    assert!(crate::sequence::message_text(&a.repo, d, mm1, true).ends_with(": total = sum(a, b)"));
    // a class dropped on the diagram becomes a lifeline of a new role of that type
    let class = a.change(|a| a.repo.create("UMLClass", &[("name", json!("Order"))], Some((root, "ownedElements"))));
    let got = a.drop_model(class, crate::views::P { x: 600.0, y: 100.0 }).unwrap();
    let lm = a.repo.refd(got[0], "model").unwrap();
    assert_eq!(a.repo.refd(a.repo.refd(lm, "represent").unwrap(), "type"), Some(class));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(a.repo.refd(got[0], "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert!(a.repo.str(nl, "text").ends_with(": Order"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn sequence_fragments_invariants_and_interaction_uses() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLSequenceDiagram"));
    let inter = a.repo.parent(d).unwrap();
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 40.0, 100.0, 300.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // a combined fragment with one operand, then a second operand
    let cf = a.change(|a| a.create_node("UMLCombinedFragment", rect(20.0, 120.0, 300.0, 150.0))).unwrap();
    let cm = a.repo.refd(cf, "model").unwrap();
    assert_eq!((a.repo.str(cm, "interactionOperator").as_str(), a.repo.list(cm, "operands").len()), ("seq", 1));
    assert_eq!(a.repo.name(a.repo.list(cm, "operands")[0]), "Operand1");
    let o2 = a.change(|a| a.add_feature(cm, "UMLInteractionOperand")).unwrap();
    assert_eq!(a.repo.name(o2), "Operand2");
    a.repo.set(o2, "guard", json!("else"));
    a.repo.set(cm, "interactionOperator", json!("alt"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let oc = a.repo.refd(cf, "operandCompartment").unwrap();
    let ovs = a.repo.list(oc, "subViews");
    assert_eq!(ovs.len(), 2);
    let (b1, b2) = (crate::views::node_rect(&a.repo, ovs[0]), crate::views::node_rect(&a.repo, ovs[1]));
    assert_eq!(b1.y + b1.h, b2.y, "operands are stacked");
    assert_eq!(b2.y + b2.h, 120.0 + 150.0, "the last one reaches the bottom");
    assert_eq!(a.repo.str(a.repo.refd(ovs[1], "guardLabel").unwrap(), "text"), "[else]");
    assert_eq!(a.repo.str(a.repo.refd(cf, "frameTypeLabel").unwrap(), "text"), "alt");
    // a state invariant on the lifeline, centred on its line
    let sv = a.change(|a| a.create_node("UMLStateInvariant", rect(100.0, 300.0, 50.0, 20.0))).unwrap();
    let sm = a.repo.refd(sv, "model").unwrap();
    assert_eq!(a.repo.refd(sm, "covered"), a.repo.refd(l1, "model"));
    assert_eq!(a.repo.parent(sv), Some(l1), "kept in the lifeline view's sub-views");
    a.repo.set(sm, "invariant", json!("ready"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let lp = a.repo.refd(l1, "linePart").unwrap();
    let sb = crate::views::node_rect(&a.repo, sv);
    assert_eq!(sb.x + sb.w / 2.0, a.repo.f(lp, "left"));
    // a continuation and an interaction use referring to another interaction
    let cv = a.change(|a| a.create_node("UMLContinuation", rect(400.0, 120.0, 10.0, 10.0))).unwrap();
    assert_eq!(a.repo.name(a.repo.refd(cv, "model").unwrap()), "Continuation1");
    let iu = a.change(|a| a.create_node("UMLInteractionUse", rect(400.0, 200.0, 160.0, 30.0))).unwrap();
    let im = a.repo.refd(iu, "model").unwrap();
    let other = a.repo.create("UMLInteraction", &[("name", json!("Login"))], Some((a.repo.parent(inter).unwrap(), "ownedElements")));
    a.repo.set_ref(im, "refersTo", Some(other));
    a.repo.set(im, "arguments", json!("user"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(iu, "nameLabel").unwrap(), "text"), "Login(user)");
    assert_eq!(a.repo.list(inter, "fragments").len(), 4);
    let svg = crate::render::to_svg_sized(&a.repo, d, &a.info, &ms, 0.0).unwrap();
    assert!(svg.contains("{ready}") && svg.contains("[else]") && svg.contains(">ref<"));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn lifeline_and_message_quick_edits() {
    use crate::app::EditKind;
    use crate::canvas::ItemKey;
    use crate::sequence::SeqAdd;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLSequenceDiagram"));
    let inter = a.repo.parent(d).unwrap();
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 40.0, 100.0, 300.0))).unwrap();
    let m1 = a.repo.refd(l1, "model").unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // Add Message with Lifeline: a new lifeline 20 px right, a message 20 px down its line
    a.item_key(l1, EditKind::Name(m1), ItemKey::Seq(SeqAdd::MessageLifeline));
    let lifelines: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.ty(*v) == "UMLSeqLifelineView").collect();
    assert_eq!(lifelines.len(), 2);
    assert_eq!(a.repo.f(lifelines[1], "left"), 50.0 + 100.0 + 20.0);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let msgs: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.ty(*v) == "UMLSeqMessageView").collect();
    assert_eq!(msgs.len(), 1);
    let y1 = crate::views::points_of(&a.repo, msgs[0])[0].y;
    // Self, Found, Lost and To Gate stack 35 px apart
    for w in [SeqAdd::SelfMessage, SeqAdd::FoundMessage, SeqAdd::LostMessage, SeqAdd::ToGate] {
        a.item_key(l1, EditKind::Name(m1), ItemKey::Seq(w));
        a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    }
    let msgs: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.ty(*v) == "UMLSeqMessageView").collect();
    assert_eq!(msgs.len(), 5);
    assert_eq!(crate::views::points_of(&a.repo, msgs[4])[0].y, y1 + 4.0 * 35.0);
    assert!(a.repo.list(inter, "participants").iter().any(|p| a.repo.ty(*p) == "UMLGate"));
    // Add Reply Message: back from the head to the tail, 20 px under
    let mm = a.repo.refd(msgs[0], "model").unwrap();
    a.item_key(msgs[0], EditKind::Name(mm), ItemKey::ReplyMessage);
    let reply = a.repo.list(inter, "messages").into_iter().find(|m| a.repo.str(*m, "messageSort") == "reply").unwrap();
    assert_eq!(a.repo.refd(reply, "target"), Some(m1));
    // Create Type for the lifeline, then Create Operation for the message to it
    let role = a.repo.refd(m1, "represent").unwrap();
    a.apply_input(crate::input::InputAction::CreateLifelineType(role), "Shop".into());
    let shop = a.repo.refd(role, "type").unwrap();
    assert_eq!(a.repo.parent(shop), a.repo.parent(a.repo.parent(role).unwrap()), "in the collaboration's owner");
    a.apply_input(crate::input::InputAction::CreateMessageOperation(reply), "done".into());
    let op = a.repo.refd(reply, "signature").unwrap();
    assert_eq!((a.repo.ty(op), a.repo.parent(op)), ("UMLOperation", Some(shop)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn wireframes_hold_frames_widgets_and_tab_lists() {
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "WFWireframeDiagram"));
    let wf = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(wf), a.repo.name(wf).as_str(), a.repo.name(d).as_str()), ("WFWireframe", "Wireframe1", "WFWireframeDiagram1"));
    // a frame takes its smallest size on a click; a button drawn in it goes in it
    let f = a.change(|a| a.create_node("WFMobileFrame", rect(0.0, 0.0, 1.0, 1.0))).unwrap();
    a.repo.set(f, "width", json!(300));
    a.repo.set(f, "height", json!(500));
    let fm = a.repo.refd(f, "model").unwrap();
    assert_eq!(a.repo.name(fm), "MobileFrame1");
    let b = a.change(|a| a.create_node("WFButton", rect(20.0, 20.0, 1.0, 1.0))).unwrap();
    let bm = a.repo.refd(b, "model").unwrap();
    assert_eq!((a.repo.name(bm).as_str(), a.repo.parent(bm)), ("Button1", Some(fm)));
    assert_eq!(a.repo.refd(b, "containerView"), Some(f));
    // a frame never goes in a frame
    let f2 = a.change(|a| a.create_node("WFFrame", rect(40.0, 40.0, 1.0, 1.0))).unwrap();
    assert_eq!(a.repo.refd(f2, "containerView"), None);
    let f2m = a.repo.refd(f2, "model").unwrap();
    a.change(|a| a.delete_models(&[f2m]));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let lh = ms.size();
    let name_w = crate::views::widest_word(&ms, "Button1", 0).ceil();
    assert_eq!((a.repo.f(b, "width"), a.repo.f(b, "height")), (name_w + 16.0, lh + 12.0), "the name with 8 / 6 padding");
    assert_eq!((a.repo.f(f, "width"), a.repo.f(f, "height")), (300.0, 500.0));
    // a switch is always 26 × 16; a slider 12 high; an avatar round
    let sw = a.change(|a| a.create_node("WFSwitch", rect(20.0, 100.0, 80.0, 80.0))).unwrap();
    let av = a.change(|a| a.create_node("WFAvatar", rect(20.0, 200.0, 60.0, 40.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(sw, "width"), a.repo.f(sw, "height")), (26.0, 16.0));
    assert_eq!((a.repo.f(av, "width"), a.repo.f(av, "height")), (40.0, 40.0));
    // a tab list lines up its tabs; Add Tab puts a new one after the others, taking the list's position
    let tl = a.change(|a| a.create_node("WFTabList", rect(20.0, 300.0, 1.0, 1.0))).unwrap();
    let tlm = a.repo.refd(tl, "model").unwrap();
    assert_eq!(a.repo.parent(tlm), Some(fm));
    let t1 = a.change(|a| a.add_tab(tl)).unwrap();
    let t2 = a.change(|a| a.add_tab(tl)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let t2m = a.repo.refd(t2, "model").unwrap();
    assert_eq!((a.repo.name(t2m).as_str(), a.repo.parent(t2m)), ("Tab2", Some(tlm)));
    assert_eq!(a.repo.f(t1, "left"), 20.0);
    assert_eq!(a.repo.f(t2, "left"), 20.0 + a.repo.f(t1, "width"));
    assert_eq!(a.repo.f(t2, "top"), 300.0);
    assert_eq!(a.repo.f(tl, "width"), a.repo.f(t1, "width") + a.repo.f(t2, "width"));
    a.repo.set(tlm, "position", json!("left"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(t2m, "position"), "left");
    assert_eq!(a.repo.f(t2, "top"), 300.0 + a.repo.f(t1, "height"));
    // the quick edits: Add Note on a widget, Add Tab on a tab list
    assert_eq!(crate::related::actions("WFButton").len(), 1);
    assert!(crate::canvas::item_buttons(&a.repo, None, &crate::app::EditKind::Name(tlm)).iter().any(|b| b.1 == "Add Tab"));
    // the template: a wireframe with a phone frame, without the UML profile
    let t = crate::templates::TEMPLATES.iter().find(|t| t.label == "Wireframe Model").unwrap();
    let pr = crate::templates::project(t);
    assert_eq!(pr.children(pr.root).len(), 1);
    let w = pr.children(pr.root)[0].1;
    assert_eq!(pr.ty(w), "WFWireframe");
    let names: Vec<String> = pr.children(w).into_iter().map(|(_, c)| pr.name(c)).collect();
    assert_eq!(names, ["Wireframe Diagram", "MobileFrame1"]);
    // every view draws
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn communication_diagrams_have_lifelines_connectors_and_messages() {
    use crate::communication::CommAdd;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLCommunicationDiagram"));
    let inter = a.repo.parent(d).unwrap();
    let collab = a.repo.parent(inter).unwrap();
    assert_eq!((a.repo.ty(inter), a.repo.ty(collab), a.repo.name(d).as_str()), ("UMLInteraction", "UMLCollaboration", "CommunicationDiagram1"));
    assert_eq!(a.repo.ty(a.repo.list(d, "ownedViews")[0]), "UMLFrameView");
    // lifelines are boxes at least 100 × 30
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 1.0, 1.0))).unwrap();
    let l2 = a.change(|a| a.create_node("UMLLifeline", rect(350.0, 100.0, 1.0, 1.0))).unwrap();
    assert_eq!(a.repo.ty(l1), "UMLCommLifelineView");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(l1, "width"), a.repo.f(l1, "height")), (100.0, 30.0));
    let (m1, m2) = (a.repo.refd(l1, "model").unwrap(), a.repo.refd(l2, "model").unwrap());
    let (r1, r2) = (a.repo.refd(m1, "represent").unwrap(), a.repo.refd(m2, "represent").unwrap());
    // a forward message drawn between two lifelines makes a connector between their roles first
    let fwd = tool("UMLForwardMessage", AssocInit::Plain);
    assert!(a.can_connect(&fwd, l1, l2));
    let mv = a.change(|a| a.create_edge(&fwd, l1, l2)).unwrap();
    assert_eq!(a.repo.ty(mv), "UMLCommMessageView");
    let cv = a.repo.refd(mv, "hostEdge").unwrap();
    let cm = a.repo.refd(cv, "model").unwrap();
    assert_eq!((a.repo.ty(cm), a.repo.parent(cm)), ("UMLConnector", Some(r1)));
    let e2 = a.repo.child(cm, "end2").unwrap();
    assert_eq!(a.repo.refd(e2, "reference"), Some(r2));
    let msg = a.repo.refd(mv, "model").unwrap();
    assert_eq!((a.repo.refd(msg, "source"), a.repo.refd(msg, "target"), a.repo.refd(msg, "connector")), (Some(m1), Some(m2), Some(cm)));
    // a reverse message dropped on the connector goes the other way
    let rev = tool("UMLReverseMessage", AssocInit::Plain);
    assert!(a.can_connect(&rev, a.diagram.unwrap(), cv));
    let rv = a.change(|a| a.create_edge(&rev, d, cv)).unwrap();
    let rmsg = a.repo.refd(rv, "model").unwrap();
    assert_eq!((a.repo.refd(rmsg, "source"), a.repo.refd(rmsg, "target")), (Some(m2), Some(m1)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // the arrow is 40 long along the horizontal connector, 10 px above its middle, numbered
    assert_eq!((a.repo.f(mv, "width"), a.repo.f(mv, "height")), (40.0, 0.0));
    let pts = crate::views::points_of(&a.repo, cv);
    let midx = ((pts[0].x + pts[1].x) / 2.0).floor();
    assert!((a.repo.f(mv, "left") + 20.0 - midx).abs() <= 1.0, "{} vs {midx}", a.repo.f(mv, "left"));
    assert_eq!(a.repo.f(mv, "top"), pts[0].y - 10.0);
    let nl = a.repo.refd(mv, "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "1 : Message1");
    assert_eq!(a.repo.str(a.repo.refd(rv, "nameLabel").unwrap(), "text"), "2 : Message2");
    // the lifeline's quick edit: a connected lifeline to the right, a self connector, a forward message to a lifeline below
    let labels: Vec<&str> = crate::canvas::item_buttons(&a.repo, Some(l1), &crate::app::EditKind::Name(m1)).into_iter().map(|b| b.1).collect();
    assert!(labels.contains(&"Add Connected Lifeline") && labels.contains(&"Add Reverse Message"));
    a.diagram = Some(d);
    let n = a.add_comm(l1, CommAdd::ConnectedLifeline).unwrap();
    assert_eq!(a.repo.f(n, "left"), 350.0 + 100.0 + 70.0, "past the lifeline already joined on the right");
    let s = a.add_comm(l1, CommAdd::SelfConnector).unwrap();
    assert_eq!((a.repo.refd(s, "tail"), a.repo.refd(s, "head")), (Some(l1), Some(l1)));
    let fm = a.add_comm(l1, CommAdd::ForwardMessage).unwrap();
    let below = a.repo.refd(a.repo.refd(fm, "hostEdge").unwrap(), "head").unwrap();
    assert_eq!(a.repo.f(below, "top"), 100.0 + 30.0 + 70.0);
    // a reply goes on the same connector, back
    let rp = a.add_reply(mv).unwrap();
    let rpm = a.repo.refd(rp, "model").unwrap();
    assert_eq!((a.repo.str(rpm, "messageSort").as_str(), a.repo.refd(rpm, "source")), ("reply", Some(m2)));
    // reconnecting the connector's head to another lifeline moves the messages' ends along
    let l3 = a.change(|a| a.create_node("UMLLifeline", rect(350.0, 400.0, 1.0, 1.0))).unwrap();
    let m3 = a.repo.refd(l3, "model").unwrap();
    a.reconnect(cv, false, l3).unwrap();
    assert_eq!(a.repo.refd(msg, "target"), Some(m3));
    assert_eq!(a.repo.refd(rmsg, "source"), Some(m3));
    assert_eq!(a.repo.refd(e2, "reference"), a.repo.refd(m3, "represent"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn sequence_diagrams_take_time_and_duration_constraints() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLSequenceDiagram"));
    let inter = a.repo.parent(d).unwrap();
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 100.0, 60.0))).unwrap();
    let l2 = a.change(|a| a.create_node("UMLLifeline", rect(300.0, 100.0, 100.0, 60.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    a.line_from = Some((crate::views::P { x: 100.0, y: 150.0 }, crate::views::P { x: 350.0, y: 150.0 }));
    let e = a.change(|a| a.create_edge(&tool("UMLMessage", AssocInit::Plain), l1, l2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // a time constraint clicked on the message belongs to it, its label 20 px under the point, linked to it
    assert!(a.change(|a| a.create_node("UMLTimeConstraint", rect(200.0, 300.0, 1.0, 1.0))).is_none(), "only on a message");
    let tv = a.change(|a| a.create_node("UMLTimeConstraint", rect(200.0, 150.0, 1.0, 1.0))).unwrap();
    let tm = a.repo.refd(tv, "model").unwrap();
    let em = a.repo.refd(e, "model").unwrap();
    assert_eq!((a.repo.ty(tm), a.repo.name(tm).as_str(), a.repo.parent(tm)), ("UMLTimeConstraint", "Time Constraint1", Some(em)));
    assert_eq!(a.repo.f(tv, "top"), 170.0);
    let link = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "UMLTimeConstraintLinkView").unwrap();
    assert_eq!((a.repo.refd(link, "tail"), a.repo.refd(link, "head")), (Some(e), Some(tv)));
    a.repo.set(tm, "min", json!("0"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(tv, "text"), "{0..max}");
    let pts = crate::views::points_of(&a.repo, link);
    assert_eq!(pts[0].y, 150.0, "the link starts on the message");
    // a duration constraint belongs to the interaction: 14 wide, stretching down
    let dv = a.change(|a| a.create_node("UMLDurationConstraint", rect(420.0, 140.0, 60.0, 80.0))).unwrap();
    let dm = a.repo.refd(dv, "model").unwrap();
    assert_eq!((a.repo.name(dm).as_str(), a.repo.parent(dm)), ("Duration Constraint1", Some(inter)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(dv, "width"), a.repo.f(dv, "height")), (14.0, 80.0));
    let nl = a.repo.refd(dv, "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "{min..max}");
    let (lcx, lcy) = (a.repo.f(nl, "left") + a.repo.f(nl, "width") / 2.0, a.repo.f(nl, "top") + a.repo.f(nl, "height") / 2.0);
    assert_eq!((lcx, lcy), (427.0 - 15.0, 180.0 - 15.0), "centred 20 px up and left of the middle");
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn interaction_overview_diagrams_call_interactions() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLInteractionOverviewDiagram"));
    let act = a.repo.parent(d).unwrap();
    let inter = a.repo.parent(act).unwrap();
    let collab = a.repo.parent(inter).unwrap();
    assert_eq!((a.repo.ty(act), a.repo.ty(inter), a.repo.ty(collab)), ("UMLActivity", "UMLInteraction", "UMLCollaboration"));
    assert_eq!(a.repo.name(d), "InteractionOverviewDiagram1");
    // an interaction use calls a new interaction next to the diagram's
    let u = a.change(|a| a.create_node("UMLInteractionUseInOverview", rect(50.0, 80.0, 160.0, 60.0))).unwrap();
    let um = a.repo.refd(u, "model").unwrap();
    assert_eq!((a.repo.ty(um), a.repo.str(um, "kind").as_str(), a.repo.parent(um)), ("UMLAction", "callBehavior", Some(act)));
    let iu = a.repo.refd(um, "target").unwrap();
    assert_eq!((a.repo.ty(iu), a.repo.parent(iu)), ("UMLInteractionUse", Some(inter)));
    let called = a.repo.refd(iu, "refersTo").unwrap();
    assert_eq!((a.repo.name(called).as_str(), a.repo.parent(called)), ("Interaction2", Some(collab)));
    // an inline interaction draws a new sequence diagram of a new interaction
    let f = a.change(|a| a.create_node("UMLInteractionInOverview", rect(300.0, 80.0, 300.0, 250.0))).unwrap();
    assert_eq!(a.repo.ty(f), "UMLInteractionInlineView");
    let fm = a.repo.refd(f, "model").unwrap();
    let sd = a.repo.refd(fm, "target").unwrap();
    assert_eq!((a.repo.ty(sd), a.repo.name(fm).as_str()), ("UMLSequenceDiagram", "Actions1"));
    // control nodes and flows join them
    let ini = a.change(|a| a.create_node("UMLInitialNode", rect(100.0, 20.0, 1.0, 1.0))).unwrap();
    let cf = tool("UMLControlFlow", AssocInit::Plain);
    assert!(a.can_connect(&cf, ini, u));
    let e = a.change(|a| a.create_edge(&cf, ini, u)).unwrap();
    assert_eq!(a.repo.parent(a.repo.refd(e, "model").unwrap()), Some(act));
    // the called sequence diagram gets a lifeline, shown small inside the frame
    a.diagram = Some(sd);
    a.change(|a| a.create_node("UMLLifeline", rect(20.0, 40.0, 100.0, 200.0))).unwrap();
    a.diagram = Some(d);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let lv = a.repo.list(sd, "ownedViews")[0];
    assert!(a.info.contains_key(&lv), "the inline diagram is laid out with it");
    assert_eq!(a.repo.str(a.repo.refd(f, "frameTypeLabel").unwrap(), "text"), "sd");
    let u_text = a.repo.str(a.repo.refd(u, "nameLabel").unwrap(), "text");
    assert_eq!(u_text, "Interaction2");
    // dropping an interaction makes an interaction use calling it
    let v = a.drop_model(called, crate::views::P { x: 50.0, y: 300.0 }).unwrap();
    assert_eq!(a.repo.ty(v[0]), "UMLInteractionUseView");
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn timing_diagrams_have_lifelines_states_segments_and_messages() {
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLTimingDiagram"));
    let inter = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(inter), a.repo.name(d).as_str()), ("UMLInteraction", "TimingDiagram1"));
    let f = a.repo.list(d, "ownedViews")[0];
    assert_eq!((a.repo.ty(f), a.repo.f(f, "left"), a.repo.f(f, "width")), ("UMLTimingFrameView", 5.0, 695.0));
    // a lifeline only goes in a frame; it spans the frame
    assert!(a.change(|a| a.create_node("UMLLifeline", rect(800.0, 800.0, 1.0, 1.0))).is_none());
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 60.0, 1.0, 1.0))).unwrap();
    let l2 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 400.0, 1.0, 1.0))).unwrap();
    assert_eq!((a.repo.ty(l1), a.repo.refd(l1, "containerView")), ("UMLTimingLifelineView", Some(f)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // states in a lifeline, by the tool or Create State/Condition
    let s1 = a.change(|a| a.create_node("UMLTimingState", rect(100.0, a.repo.f(l1, "top") + 10.0, 1.0, 1.0))).unwrap();
    a.diagram = Some(d);
    let s2 = a.add_timing(l1).unwrap();
    let s1m = a.repo.refd(s1, "model").unwrap();
    let l1m = a.repo.refd(l1, "model").unwrap();
    assert_eq!((a.repo.ty(s1m), a.repo.name(s1m).as_str(), a.repo.parent(s1m)), ("UMLConstraint", "State-Condition1", Some(l1m)));
    assert_eq!(a.repo.name(a.repo.refd(s2, "model").unwrap()), "State-Condition2");
    a.repo.set(s1m, "name", json!("Waiting a long while"));
    let s3 = a.add_timing(l2).unwrap();
    // time segments on the states: state invariants of the interaction
    let g1 = a.change(|a| a.time_segment(d, s1, rect(400.0, 0.0, 40.0, 1.0))).unwrap();
    let g2 = a.add_timing(s2).unwrap();
    let g3 = a.add_timing(s3).unwrap();
    let g1m = a.repo.refd(g1, "model").unwrap();
    assert_eq!((a.repo.ty(g1m), a.repo.parent(g1m), a.repo.refd(g1m, "covered"), a.repo.refd(g1m, "invariant")), ("UMLStateInvariant", Some(inter), Some(l1m), Some(s1m)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // lifelines stack down the frame, states 25 px into them, segments from the end of the longest state name
    assert_eq!((a.repo.f(l1, "left"), a.repo.f(l1, "width")), (5.0, 695.0));
    assert_eq!(a.repo.f(l2, "top"), a.repo.f(l1, "top") + a.repo.f(l1, "height"));
    assert_eq!(a.repo.f(s1, "top"), a.repo.f(l1, "top") + 25.0);
    let start = a.repo.f(s1, "left") + ms.width("Waiting a long while", 0).ceil() + 5.0;
    let starts: Vec<f64> = [g1, g2, g3].iter().map(|g| a.repo.f(*g, "left")).collect();
    assert_eq!(starts[0].min(starts[1]), start, "the first segment starts after the names");
    assert_eq!(starts[2], start);
    assert_eq!(a.repo.f(g1, "height"), 25.0);
    // a message between two segments, its ends at their middles
    let msg = tool("UMLMessage", AssocInit::Plain);
    assert!(a.can_connect(&msg, g1, g3));
    let gx = a.repo.f(g1, "left") + 5.0;
    a.line_from = Some((crate::views::P { x: gx, y: 0.0 }, crate::views::P { x: gx + 3.0, y: 0.0 }));
    let e = a.change(|a| a.create_edge(&msg, g1, g3)).unwrap();
    let em = a.repo.refd(e, "model").unwrap();
    assert_eq!((a.repo.refd(em, "source"), a.repo.refd(em, "target")), (Some(l1m), a.repo.refd(l2, "model")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let pts = crate::views::points_of(&a.repo, e);
    assert_eq!(pts[0].y, (a.repo.f(g1, "top") + 12.5).round());
    assert_eq!(pts[0].x, a.repo.f(g1, "left"), "snapped to the segment's start");
    // time ticks along the bottom of the frame, from the start
    let t1 = a.change(|a| a.create_node("UMLTimeTick", rect(300.0, 590.0, 1.0, 1.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(t1, "left"), a.repo.f(t1, "top")), (start, 5.0 + 595.0 - 5.0));
    // a tick edits its own text on one line
    a.edit_view(t1, None);
    assert!(matches!(a.editing.as_ref().map(|x| (x.kind.clone(), x.multiline)), Some((crate::app::EditKind::Text(x), false)) if x == t1));
    a.editing.as_mut().unwrap().text = "t0".into();
    a.close_inline(true);
    assert_eq!(a.repo.str(t1, "text"), "t0");
    // a time constraint on a segment, and a duration constraint lying across in the frame
    let tc = a.change(|a| a.create_node("UMLTimeConstraint", rect(a.repo.f(g1, "left") + 2.0, a.repo.f(g1, "top") + 5.0, 1.0, 1.0))).unwrap();
    assert_eq!(a.repo.parent(a.repo.refd(tc, "model").unwrap()), Some(g1m));
    let dc = a.change(|a| a.create_node("UMLDurationConstraint", rect(300.0, 300.0, 80.0, 30.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(dc, "width"), a.repo.f(dc, "height"), a.repo.refd(dc, "containerView")), (80.0, 14.0, Some(f)));
    // the quick edits
    let labels: Vec<&str> = crate::canvas::item_buttons(&a.repo, Some(l1), &crate::app::EditKind::Name(l1m)).into_iter().map(|b| b.1).collect();
    assert!(labels.contains(&"Create State/Condition"));
    let labels: Vec<&str> = crate::canvas::item_buttons(&a.repo, Some(s1), &crate::app::EditKind::Name(s1m)).into_iter().map(|b| b.1).collect();
    assert_eq!(labels, ["Create Time Segment"]);
    // moving a segment onto another state changes what it covers
    a.change(|a| a.set_container(&[g1], Some(s3)));
    assert_eq!(a.repo.refd(g1m, "invariant"), a.repo.refd(s3, "model"));
    assert_eq!(a.repo.refd(g1m, "covered"), a.repo.refd(l2, "model"));
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn views_move_and_resize_only_the_ways_they_may() {
    use crate::views::{handle_shown, sizable, Sizable};
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLSequenceDiagram"));
    let l = a.change(|a| a.create_node("UMLLifeline", rect(100.0, 100.0, 100.0, 200.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // a lifeline only moves across (MM_HORZ)
    a.select(vec![l]);
    let (x, y) = (a.repo.f(l, "left") + 50.0, a.repo.f(l, "top") + 20.0);
    a.on_press(canvas, Pos2::new(x as f32, y as f32), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(x as f32 + 40.0, y as f32 + 40.0));
    a.on_release(canvas.min, Pos2::new(x as f32 + 40.0, y as f32 + 40.0));
    assert!(a.repo.f(l, "left") > 130.0, "moved across");
    assert_eq!(a.repo.f(l, "top"), 40.0, "not down");
    // and so with the arrow keys
    a.select(vec![l]);
    a.nudge_selection(0.0, 10.0);
    assert_eq!(a.repo.f(l, "top"), 40.0);
    // a duration constraint stretches only down on a sequence diagram: no side or corner handles
    let dc = a.change(|a| a.create_node("UMLDurationConstraint", rect(400.0, 100.0, 14.0, 60.0))).unwrap();
    assert_eq!(sizable(&a.repo, dc), Sizable::Vert);
    assert!(handle_shown(Sizable::Vert, 5) && !handle_shown(Sizable::Vert, 4) && !handle_shown(Sizable::Vert, 3));
    a.select(vec![dc]);
    let b = crate::views::node_rect(&a.repo, dc);
    let corner = Pos2::new((b.x + b.w + 4.0) as f32, (b.y + b.h + 4.0) as f32);
    a.on_press(canvas, corner, Modifiers::NONE);
    assert!(!matches!(a.drag, Some(crate::app::Drag::Resize { .. })), "no corner handle");
    a.on_release(canvas.min, corner);
    // a switch has no handles at all
    let wd = a.change(|a| a.add_diagram(Some(root), "WFWireframeDiagram"));
    a.diagram = Some(wd);
    let sw = a.change(|a| a.create_node("WFSwitch", rect(50.0, 50.0, 1.0, 1.0))).unwrap();
    assert_eq!(sizable(&a.repo, sw), Sizable::None);
}

#[test]
fn delete_hides_compartment_items_and_asks_for_lifelines() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let m1 = a.repo.refd(c1, "model").unwrap();
    let at = a.change(|a| a.add_feature(m1, "UMLAttribute")).unwrap();
    let at2 = a.change(|a| a.add_feature(m1, "UMLAttribute")).unwrap();
    let d = a.diagram.unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let h0 = a.repo.f(c1, "height");
    let rows = a.info[&c1].compartments.iter().flat_map(|c| c.rows.iter()).map(|r| r.iv).collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    let iv = rows[0];
    assert_eq!(a.repo.refd(iv, "model"), Some(at));
    let b = crate::views::node_rect(&a.repo, iv);
    // a click on the row selects the attribute's view alone, and a drag does not move the class
    a.on_press(canvas, Pos2::new((b.x + 5.0) as f32, (b.y + b.h / 2.0) as f32), Modifiers::NONE);
    a.on_move(canvas, Pos2::new((b.x + 60.0) as f32, (b.y + 60.0) as f32));
    a.on_release(canvas.min, Pos2::new((b.x + 60.0) as f32, (b.y + 60.0) as f32));
    assert_eq!(a.selection, vec![iv]);
    assert_eq!(a.focus, Some(at));
    assert_eq!(a.repo.f(c1, "left"), 40.0);
    // Delete hides it: the model stays, the view is kept but takes no room
    a.delete_selection(false);
    assert!(a.repo.alive(at) && a.repo.alive(iv));
    assert_eq!(a.repo.raw(iv, "visible"), Some(&json!(false)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let rows = a.info[&c1].compartments.iter().flat_map(|c| c.rows.iter()).map(|r| r.iv).collect::<Vec<_>>();
    assert_eq!(rows.len(), 1);
    assert_eq!(a.repo.refd(rows[0], "model"), Some(at2));
    assert!(a.repo.f(c1, "height") <= h0);
    // a lifeline cannot go alone: the person is asked
    let sd = a.change(|a| a.add_diagram(Some(a.repo.root), "UMLSequenceDiagram"));
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 100.0, 60.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, sd);
    let lm = a.repo.refd(l1, "model").unwrap();
    a.select(vec![l1]);
    a.delete_selection(false);
    assert!(a.delete_prompt && a.repo.alive(l1));
    a.delete_prompt = false;
    a.delete_views_only();
    assert!(!a.repo.alive(l1));
    assert!(a.repo.alive(lm));
}

#[test]
fn a_hidden_attribute_dropped_on_its_class_shows_again() {
    let ms = measure();
    let mut a = App::new(None);
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let m1 = a.repo.refd(c1, "model").unwrap();
    let at = a.change(|a| a.add_feature(m1, "UMLAttribute")).unwrap();
    let d = a.diagram.unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let iv = a.info[&c1].compartments.iter().flat_map(|c| c.rows.iter()).map(|r| r.iv).next().unwrap();
    a.select(vec![iv]);
    a.delete_selection(false);
    assert_eq!(a.repo.raw(iv, "visible"), Some(&json!(false)));
    let shown = a.drop_model(at, crate::views::P { x: 60.0, y: 60.0 }).unwrap();
    assert_eq!(shown, vec![iv]);
    assert_ne!(a.repo.raw(iv, "visible"), Some(&json!(false)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info[&c1].compartments.iter().flat_map(|c| c.rows.iter()).count(), 1);
    assert_eq!(a.repo.list(d, "ownedViews").len(), 1, "no second view");
}

#[test]
fn line_labels_are_selected_by_area_and_select_all_and_resize_across() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    a.change(|a| a.repo.set(m, "name", json!("borrows many things")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l = a.repo.refd(e, "nameLabel").unwrap();
    a.select_all();
    assert!(a.selection.contains(&c1) && a.selection.contains(&e) && a.selection.contains(&l), "{:?}", a.selection);
    // an area over the label alone selects it
    let b = crate::views::node_rect(&a.repo, l);
    a.select(vec![]);
    a.on_press(canvas, Pos2::new((b.x - 2.0) as f32, (b.y - 12.0) as f32), Modifiers::NONE);
    a.on_move(canvas, Pos2::new((b.x + 20.0) as f32, (b.y - 4.0) as f32));
    a.on_move(canvas, Pos2::new((b.x + 20.0) as f32, (b.y + 4.0) as f32));
    a.on_release(canvas.min, Pos2::new((b.x + 20.0) as f32, (b.y + 4.0) as f32));
    assert_eq!(a.selection, vec![l]);
    // with word wrap a label narrowed by hand wraps, and keeps at least its widest word
    a.change(|a| {
        a.repo.set(l, "wordWrap", json!(true));
        a.repo.set(l, "width", json!(10));
    });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nb = crate::views::node_rect(&a.repo, l);
    assert!(nb.w < b.w && nb.h > b.h, "{b:?} -> {nb:?}");
    assert!(nb.w >= crate::views::widest_word(&ms, "borrows", 0) - 0.5);
    // a widened label keeps its width
    a.change(|a| {
        a.repo.set(l, "wordWrap", json!(false));
        a.repo.set(l, "width", json!(300));
    });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.f(l, "width"), 300.0);
}

#[test]
fn a_port_label_drags_around_its_port() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let d = a.diagram.unwrap();
    a.change(|a| a.create_node("UMLClass", rect(100.0, 100.0, 300.0, 200.0))).unwrap();
    let p = a.change(|a| a.create_feature_box("UMLPort", crate::views::P { x: 390.0, y: 200.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l = a.repo.refd(p, "nameLabel").unwrap();
    let b = crate::views::node_rect(&a.repo, l);
    let at = Pos2::new((b.x + b.w / 2.0) as f32, (b.y + b.h / 2.0) as f32);
    a.on_press(canvas, at, Modifiers::NONE);
    assert_eq!(a.selection, vec![l]);
    a.on_move(canvas, at + egui::vec2(40.0, 30.0));
    a.on_release(canvas.min, at + egui::vec2(40.0, 30.0));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b2 = crate::views::node_rect(&a.repo, l);
    assert!((b2.x - b.x - 40.0).abs() <= 2.0 && (b2.y - b.y - 30.0).abs() <= 2.0, "{b:?} -> {b2:?}");
    // and follows the port
    a.change(|a| a.repo.set(p, "top", json!(a.repo.f(p, "top") + 20.0)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!((crate::views::node_rect(&a.repo, l).y - b2.y - 20.0).abs() <= 1.0);
}

#[test]
fn a_line_label_resizes_across_with_its_handles() {
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
    a.select(vec![l]);
    let b = crate::views::node_rect(&a.repo, l);
    // the right handle sits 4 px outside the right edge, at mid height
    let at = Pos2::new((b.x + b.w + 4.0) as f32, (b.y + b.h / 2.0) as f32);
    a.on_press(canvas, at, Modifiers::NONE);
    assert!(matches!(a.drag, Some(crate::app::Drag::Resize { .. })));
    a.on_move(canvas, at + egui::vec2(40.0, 0.0));
    a.on_release(canvas.min, at + egui::vec2(40.0, 0.0));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let w = a.repo.f(l, "width");
    assert!(w >= b.w + 30.0, "{} -> {w}", b.w);
}

#[test]
fn copy_deselects_what_cannot_be_copied() {
    let ms = measure();
    let mut a = App::new(None);
    let sd = a.change(|a| a.add_diagram(Some(a.repo.root), "UMLSequenceDiagram"));
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 100.0, 60.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, sd);
    a.last_area = crate::app::Area::Canvas;
    a.select(vec![l1]);
    assert!(a.copy_text().is_none());
    assert!(a.selection.is_empty(), "the lifeline is deselected");
    assert!(a.message.as_ref().map(|m| m.1.contains("Sequence or Communication Diagram")).unwrap_or(false));
    // on a class diagram, a label and a box: the box is copied, the label deselected
    a.message = None;
    let d = a.change(|a| a.add_diagram(Some(a.repo.root), "UMLClassDiagram"));
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(a.repo.refd(c1, "nameCompartment").unwrap(), "nameLabel").unwrap();
    a.select(vec![c1, nl]);
    assert!(a.copy_text().is_some());
    assert_eq!(a.selection, vec![c1]);
    assert!(a.message.is_none());
}

#[test]
fn a_box_grows_over_children_that_extend_it() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let p = a.change(|a| a.create_node("UMLPackage", rect(40.0, 40.0, 200.0, 150.0))).unwrap();
    let c = a.change(|a| a.create_node("UMLClass", rect(60.0, 80.0, 100.0, 60.0))).unwrap();
    a.change(|a| {
        a.repo.set_ref(c, "containerView", Some(p));
        let r = a.repo.ref_to(c);
        a.repo.set_raw(p, "containedViews", json!([r]));
        a.repo.set(c, "left", json!(300));
    });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.f(p, "width"), 200.0, "containerExtending is off by default");
    a.change(|a| a.repo.set(c, "containerExtending", json!(true)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let cw = a.repo.f(c, "width");
    assert_eq!(a.repo.f(p, "left") + a.repo.f(p, "width"), 300.0 + cw + 5.0);
}

// ------------------------------------------------------------ StarUML 1 import (staruml_v1.rs)

/// A StarUML 1 file around these project contents, in the XPD format the importer reads.
fn v1_file(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<XPD:PROJECT xmlns:XPD="http://www.staruml.com" version="1">
<XPD:HEADER><XPD:PROFILES><XPD:PROFILE>UMLStandard</XPD:PROFILE></XPD:PROFILES></XPD:HEADER>
<XPD:BODY>
<XPD:OBJ name="DocumentElement" type="UMLProject" guid="P">
<XPD:ATTR name="Title" type="string">Imported</XPD:ATTR>
<XPD:ATTR name="Author" type="string">Ann</XPD:ATTR>
{body}
</XPD:OBJ>
</XPD:BODY>
</XPD:PROJECT>"#
    )
}

/// Import a StarUML 1 file into a new App, save it and give the saved JSON with a finder by id.
fn import_v1(body: &str) -> (App, Value) {
    let ms = measure();
    let mut a = App::new(None);
    let repo = crate::staruml_v1::import(&v1_file(body)).expect("import");
    a.set_project(repo, None);
    a.prepare_for_save(&ms);
    let v: Value = serde_json::from_str(&a.repo.serialize()).unwrap();
    (a, v)
}

fn find<'a>(v: &'a Value, id: &str) -> &'a Value {
    fn walk<'a>(v: &'a Value, id: &str) -> Option<&'a Value> {
        match v {
            Value::Object(m) => {
                if m.get("_id").and_then(|x| x.as_str()) == Some(id) {
                    return Some(v);
                }
                m.values().find_map(|x| walk(x, id))
            }
            Value::Array(a) => a.iter().find_map(|x| walk(x, id)),
            _ => None,
        }
    }
    walk(v, id).unwrap_or_else(|| panic!("no element {id}"))
}

fn all_of_type<'a>(v: &'a Value, ty: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(m) => {
            if m.get("_type").and_then(|x| x.as_str()) == Some(ty) {
                out.push(v);
            }
            m.values().for_each(|x| all_of_type(x, ty, out));
        }
        Value::Array(a) => a.iter().for_each(|x| all_of_type(x, ty, out)),
        _ => {}
    }
}

const V1_CLASS_MODEL: &str = r##"
<XPD:ATTR name="#OwnedElements" type="integer">1</XPD:ATTR>
<XPD:OBJ name="OwnedElements[0]" type="UMLModel" guid="M">
<XPD:ATTR name="Name" type="string">Design</XPD:ATTR>
<XPD:REF name="Namespace">P</XPD:REF>
<XPD:ATTR name="#OwnedDiagrams" type="integer">1</XPD:ATTR>
<XPD:OBJ name="OwnedDiagrams[0]" type="UMLClassDiagram" guid="D">
<XPD:ATTR name="Name" type="string">Classes</XPD:ATTR>
<XPD:ATTR name="DefaultDiagram" type="boolean">True</XPD:ATTR>
<XPD:OBJ name="DiagramView" type="UMLClassDiagramView" guid="DV">
<XPD:ATTR name="#OwnedViews" type="integer">3</XPD:ATTR>
<XPD:OBJ name="OwnedViews[0]" type="UMLClassView" guid="V1">
<XPD:ATTR name="FontFace" type="string">Tahoma</XPD:ATTR>
<XPD:ATTR name="FontSize" type="integer">9</XPD:ATTR>
<XPD:ATTR name="FontStyle" type="integer">3</XPD:ATTR>
<XPD:ATTR name="FillColor" type="string">$0080FFFF</XPD:ATTR>
<XPD:ATTR name="LineColor" type="string">clRed</XPD:ATTR>
<XPD:ATTR name="Left" type="integer">40</XPD:ATTR>
<XPD:ATTR name="Top" type="integer">32</XPD:ATTR>
<XPD:ATTR name="Width" type="integer">120</XPD:ATTR>
<XPD:ATTR name="Height" type="integer">70</XPD:ATTR>
<XPD:ATTR name="StereotypeDisplay" type="UMLStereotypeDisplayKind">sdkIcon</XPD:ATTR>
<XPD:REF name="Model">C1</XPD:REF>
<XPD:OBJ name="NameCompartment" type="UMLNameCompartmentView" guid="V1N">
<XPD:REF name="Model">C1</XPD:REF>
<XPD:OBJ name="NameLabel" type="LabelView" guid="V1NL">
<XPD:ATTR name="FontStyle" type="integer">1</XPD:ATTR>
<XPD:ATTR name="Text" type="string">Order</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="StereotypeLabel" type="LabelView" guid="V1SL">
<XPD:ATTR name="Visible" type="boolean">False</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="AttributeCompartment" type="UMLAttributeCompartmentView" guid="V1A">
<XPD:REF name="Model">C1</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OperationCompartment" type="UMLOperationCompartmentView" guid="V1O">
<XPD:ATTR name="Visible" type="boolean">False</XPD:ATTR>
<XPD:REF name="Model">C1</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedViews[1]" type="UMLClassView" guid="V2">
<XPD:ATTR name="Left" type="integer">300</XPD:ATTR>
<XPD:ATTR name="Top" type="integer">32</XPD:ATTR>
<XPD:ATTR name="Width" type="integer">120</XPD:ATTR>
<XPD:ATTR name="Height" type="integer">70</XPD:ATTR>
<XPD:REF name="Model">C2</XPD:REF>
<XPD:OBJ name="NameCompartment" type="UMLNameCompartmentView" guid="V2N">
<XPD:OBJ name="NameLabel" type="LabelView" guid="V2NL">
<XPD:ATTR name="Text" type="string">Line</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedViews[2]" type="UMLAssociationView" guid="VA">
<XPD:ATTR name="LineStyle" type="LineStyleKind">lsRectilinear</XPD:ATTR>
<XPD:ATTR name="Points" type="Points">160,67;300,67</XPD:ATTR>
<XPD:REF name="Model">A</XPD:REF>
<XPD:REF name="Head">V2</XPD:REF>
<XPD:REF name="Tail">V1</XPD:REF>
<XPD:OBJ name="HeadMultiplicityLabel" type="EdgeLabelView" guid="VAHM">
<XPD:ATTR name="Alpha" type="real">0.5</XPD:ATTR>
<XPD:ATTR name="Distance" type="real">15</XPD:ATTR>
<XPD:ATTR name="EdgePosition" type="EdgePositionKind">epHead</XPD:ATTR>
<XPD:REF name="HostEdge">VA</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
<XPD:ATTR name="#OwnedElements" type="integer">7</XPD:ATTR>
<XPD:OBJ name="OwnedElements[0]" type="UMLClass" guid="C1">
<XPD:ATTR name="Name" type="string">Order</XPD:ATTR>
<XPD:ATTR name="StereotypeName" type="string">entity</XPD:ATTR>
<XPD:ATTR name="IsAbstract" type="boolean">True</XPD:ATTR>
<XPD:REF name="Namespace">M</XPD:REF>
<XPD:ATTR name="#Attributes" type="integer">1</XPD:ATTR>
<XPD:OBJ name="Attributes[0]" type="UMLAttribute" guid="C1a">
<XPD:ATTR name="Name" type="string">id</XPD:ATTR>
<XPD:ATTR name="Visibility" type="UMLVisibilityKind">vkPrivate</XPD:ATTR>
<XPD:ATTR name="OwnerScope" type="UMLScopeKind">skClassifier</XPD:ATTR>
<XPD:ATTR name="Changeability" type="UMLChangeableKind">ckFrozen</XPD:ATTR>
<XPD:ATTR name="TypeExpression" type="string">int</XPD:ATTR>
<XPD:ATTR name="InitialValue" type="string">0</XPD:ATTR>
<XPD:REF name="Owner">C1</XPD:REF>
</XPD:OBJ>
<XPD:ATTR name="#Operations" type="integer">1</XPD:ATTR>
<XPD:OBJ name="Operations[0]" type="UMLOperation" guid="C1o">
<XPD:ATTR name="Name" type="string">total</XPD:ATTR>
<XPD:ATTR name="Concurrency" type="UMLCallConcurrencyKind">cckGuarded</XPD:ATTR>
<XPD:ATTR name="#Parameters" type="integer">2</XPD:ATTR>
<XPD:OBJ name="Parameters[0]" type="UMLParameter" guid="C1p1">
<XPD:ATTR name="Name" type="string">line</XPD:ATTR>
<XPD:REF name="Type_">C2</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="Parameters[1]" type="UMLParameter" guid="C1p2">
<XPD:ATTR name="DirectionKind" type="UMLParameterDirectionKind">pdkReturn</XPD:ATTR>
<XPD:ATTR name="TypeExpression" type="string">double</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
<XPD:ATTR name="#TaggedValues" type="integer">1</XPD:ATTR>
<XPD:OBJ name="TaggedValues[0]" type="TaggedValue" guid="C1t">
<XPD:ATTR name="ProfileName" type="string">Java</XPD:ATTR>
<XPD:ATTR name="TagDefinitionSetName" type="string">Class</XPD:ATTR>
<XPD:ATTR name="Name" type="string">final</XPD:ATTR>
<XPD:ATTR name="DataValue" type="string">true</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[1]" type="UMLClass" guid="C2">
<XPD:ATTR name="Name" type="string">Line</XPD:ATTR>
<XPD:REF name="Namespace">M</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[2]" type="UMLAssociation" guid="A">
<XPD:REF name="Namespace">M</XPD:REF>
<XPD:ATTR name="#Connections" type="integer">2</XPD:ATTR>
<XPD:OBJ name="Connections[0]" type="UMLAssociationEnd" guid="A1">
<XPD:ATTR name="Aggregation" type="UMLAggregationKind">akComposite</XPD:ATTR>
<XPD:ATTR name="IsNavigable" type="boolean">False</XPD:ATTR>
<XPD:REF name="Participant">C1</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="Connections[1]" type="UMLAssociationEnd" guid="A2">
<XPD:ATTR name="Name" type="string">lines</XPD:ATTR>
<XPD:ATTR name="Multiplicity" type="string">1..*</XPD:ATTR>
<XPD:ATTR name="Ordering" type="UMLOrderingKind">okOrdered</XPD:ATTR>
<XPD:REF name="Participant">C2</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[3]" type="UMLComponent" guid="K">
<XPD:ATTR name="Name" type="string">Billing</XPD:ATTR>
<XPD:ATTR name="#Residents" type="integer">1</XPD:ATTR>
<XPD:REF name="Residents[0]">C1</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[4]" type="UMLRealization" guid="R">
<XPD:REF name="Client">C2</XPD:REF>
<XPD:REF name="Supplier">K</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[5]" type="UMLException" guid="X">
<XPD:ATTR name="Name" type="string">Overdraft</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[6]" type="UMLPrimitive" guid="T">
<XPD:ATTR name="Name" type="string">money</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
"##;

#[test]
fn imports_a_staruml1_class_model() {
    let (a, v) = import_v1(V1_CLASS_MODEL);
    // the file becomes a new, unsaved project with its default diagram open (main.js:51)
    assert_eq!(a.filename, None);
    assert!(!a.modified);
    assert_eq!(v["_type"], "Project");
    assert_eq!(v["name"], "Imported");
    assert_eq!(v["author"], "Ann");
    assert!(v.get("_parent").is_none());
    let d = a.diagram.expect("default diagram is open");
    assert_eq!(a.repo.name(d), "Classes");

    let c1 = find(&v, "C1");
    assert_eq!(c1["_type"], "UMLClass");
    assert_eq!(c1["_parent"]["$ref"], "M");
    // string stereotypes stay strings: UMLCraft has no UML Standard Profile to link them to
    assert_eq!(c1["stereotype"], "entity");
    assert_eq!(c1["isAbstract"], true);
    let at = find(&v, "C1a");
    assert_eq!(at["_parent"]["$ref"], "C1");
    assert_eq!(at["visibility"], "private");
    assert_eq!(at["isStatic"], true);
    assert_eq!(at["isReadOnly"], true);
    assert_eq!(at["type"], "int");
    assert_eq!(at["defaultValue"], "0");
    let op = find(&v, "C1o");
    assert_eq!(op["concurrency"], "guarded");
    assert_eq!(find(&v, "C1p1")["type"]["$ref"], "C2");
    assert_eq!(find(&v, "C1p2")["direction"], "return");
    // tagged values become tags named profile.set.name
    let tag = &c1["tags"][0];
    assert_eq!(tag["_type"], "Tag");
    assert_eq!(tag["name"], "Java.Class.final");
    assert_eq!(tag["value"], "true");
    // V1-only fields are not written
    assert!(c1.get("isRoot").is_none() && c1.get("isSpecification").is_none());

    let assoc = find(&v, "A");
    assert_eq!(assoc["end1"]["_type"], "UMLAssociationEnd");
    assert_eq!(assoc["end1"]["aggregation"], "composite");
    assert!(assoc["end1"].get("navigable").is_none(), "unspecified is the default");
    assert_eq!(assoc["end1"]["reference"]["$ref"], "C1");
    assert_eq!(assoc["end2"]["navigable"], "navigable");
    assert_eq!(assoc["end2"]["multiplicity"], "1..*");
    assert_eq!(assoc["end2"]["isOrdered"], true);

    // semantic conversions
    let k = find(&v, "K");
    let cr = &k["ownedElements"][0];
    assert_eq!(cr["_type"], "UMLComponentRealization");
    assert_eq!(cr["source"]["$ref"], "C1");
    assert_eq!(cr["target"]["$ref"], "K");
    assert_eq!(find(&v, "R")["_type"], "UMLComponentRealization");
    assert_eq!(find(&v, "X")["_type"], "UMLSignal");
    assert_eq!(find(&v, "X")["stereotype"], "exception");
    assert_eq!(find(&v, "T")["_type"], "UMLPrimitiveType");

    // views: geometry, font (points to pixels), colours ($00BBGGRR and named), enums
    let v1 = find(&v, "V1");
    assert_eq!(v1["_type"], "UMLClassView");
    assert_eq!(v1["_parent"]["$ref"], "D");
    assert_eq!(v1["model"]["$ref"], "C1");
    assert_eq!(v1["font"], "Tahoma;12;3");
    assert_eq!(v1["fillColor"], "#FFFF80");
    assert_eq!(v1["lineColor"], "#FF0000");
    assert_eq!(v1["left"], 40);
    assert_eq!(v1["stereotypeDisplay"], "icon");
    assert_eq!(v1["suppressOperations"], true);
    assert!(v1.get("suppressAttributes").is_none(), "false is the default");
    assert_eq!(v1["nameCompartment"]["$ref"], "V1N");
    assert_eq!(find(&v, "V1NL")["text"], "Order");
    assert_eq!(find(&v, "V1N")["_parent"]["$ref"], "V1");
    // a missing template parameter compartment is made, hidden
    let tp = v1["templateParameterCompartment"]["$ref"].as_str().unwrap();
    assert_eq!(find(&v, tp)["_type"], "UMLTemplateParameterCompartmentView");
    // the second class view has no compartments in the file: they are made, hidden
    let v2 = find(&v, "V2");
    assert_eq!(v2["suppressAttributes"], true);
    assert_eq!(v2["suppressOperations"], true);
    assert_eq!(v2["font"], "Arial;11;0");
    let va = find(&v, "VA");
    assert!(va.get("lineStyle").is_none(), "rectilinear (0) is the default");
    assert_eq!(find(&v, "V1")["_type"], "UMLClassView");
    // "x,y;x,y" becomes "x:y;x:y" (the ends are then fitted to the boxes, as when drawing)
    let raw = crate::staruml_v1::convert(&v1_file(V1_CLASS_MODEL)).unwrap();
    assert_eq!(find(&raw, "VA")["points"], "160:67;300:67");
    assert_eq!(va["head"]["$ref"], "V2");
    assert_eq!(va["headMultiplicityLabel"]["$ref"], "VAHM");
    let hm = find(&v, "VAHM");
    assert!(hm.get("edgePosition").is_none(), "head (0) is the default");
    assert_eq!(hm["distance"], 15);
    assert_eq!(hm["alpha"], 0.5);
    // the stereotype, name and property labels of an edge are made when missing
    let nl = va["nameLabel"]["$ref"].as_str().unwrap();
    assert_eq!(find(&v, nl)["_type"], "EdgeLabelView");
    // every reference of the diagram resolves
    let mut views = vec![];
    for ty in ["UMLClassView", "UMLAssociationView", "UMLNameCompartmentView", "LabelView", "EdgeLabelView"] {
        all_of_type(&v, ty, &mut views);
    }
    for view in views {
        for key in ["model", "head", "tail", "hostEdge", "nameCompartment", "nameLabel"] {
            if let Some(id) = view.get(key).and_then(|r| r["$ref"].as_str()) {
                assert!(a.repo.by_ref(&json!({ "$ref": id })).is_some(), "{key} of {} resolves", view["_id"]);
            }
        }
    }
}

const V1_BEHAVIOR_MODEL: &str = r##"
<XPD:ATTR name="#OwnedElements" type="integer">3</XPD:ATTR>
<XPD:OBJ name="OwnedElements[0]" type="UMLModel" guid="M">
<XPD:ATTR name="Name" type="string">Behavior</XPD:ATTR>
<XPD:ATTR name="#OwnedCollaborationInstanceSets" type="integer">1</XPD:ATTR>
<XPD:OBJ name="OwnedCollaborationInstanceSets[0]" type="UMLCollaborationInstanceSet" guid="CIS">
<XPD:ATTR name="Name" type="string">Ordering</XPD:ATTR>
<XPD:ATTR name="#InteractionInstanceSets" type="integer">1</XPD:ATTR>
<XPD:OBJ name="InteractionInstanceSets[0]" type="UMLInteractionInstanceSet" guid="IIS">
<XPD:ATTR name="Name" type="string">PlaceOrder</XPD:ATTR>
<XPD:ATTR name="#OwnedDiagrams" type="integer">1</XPD:ATTR>
<XPD:OBJ name="OwnedDiagrams[0]" type="UMLSequenceDiagram" guid="SD">
<XPD:ATTR name="Name" type="string">Sequence</XPD:ATTR>
<XPD:ATTR name="MessageSignature" type="UMLMessageSignatureKind">mskNameAndType</XPD:ATTR>
<XPD:ATTR name="ShowActivation" type="boolean">False</XPD:ATTR>
<XPD:OBJ name="DiagramView" type="UMLSequenceDiagramView" guid="SDV">
<XPD:ATTR name="#OwnedViews" type="integer">3</XPD:ATTR>
<XPD:OBJ name="OwnedViews[0]" type="UMLSeqObjectView" guid="LV1">
<XPD:ATTR name="Left" type="integer">20</XPD:ATTR>
<XPD:REF name="Model">O1</XPD:REF>
<XPD:OBJ name="LifeLine" type="UMLLifeLineView" guid="LV1L">
<XPD:REF name="Model">O1</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedViews[1]" type="UMLSeqObjectView" guid="LV2">
<XPD:ATTR name="Left" type="integer">200</XPD:ATTR>
<XPD:REF name="Model">O2</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OwnedViews[2]" type="UMLSeqStimulusView" guid="MV">
<XPD:ATTR name="Points" type="Points">60,100;240,100</XPD:ATTR>
<XPD:REF name="Model">S</XPD:REF>
<XPD:REF name="Head">LV2</XPD:REF>
<XPD:REF name="Tail">LV1L</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
<XPD:ATTR name="#ParticipatingStimuli" type="integer">1</XPD:ATTR>
<XPD:OBJ name="ParticipatingStimuli[0]" type="UMLStimulus" guid="S">
<XPD:ATTR name="Name" type="string">submit</XPD:ATTR>
<XPD:ATTR name="Iteration" type="string">*</XPD:ATTR>
<XPD:REF name="Sender">O1</XPD:REF>
<XPD:REF name="Receiver">O2</XPD:REF>
<XPD:OBJ name="Action" type="UMLCallAction" guid="SA">
<XPD:ATTR name="IsAsynchronous" type="boolean">True</XPD:ATTR>
<XPD:REF name="Operation">OP</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
<XPD:ATTR name="#ParticipatingInstances" type="integer">2</XPD:ATTR>
<XPD:OBJ name="ParticipatingInstances[0]" type="UMLObject" guid="O1">
<XPD:ATTR name="Name" type="string">client</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="ParticipatingInstances[1]" type="UMLObject" guid="O2">
<XPD:ATTR name="Name" type="string">shop</XPD:ATTR>
<XPD:ATTR name="IsMultiInstance" type="boolean">True</XPD:ATTR>
<XPD:REF name="Classifier">SHOP</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[1]" type="UMLStateMachine" guid="SM">
<XPD:ATTR name="Name" type="string">Lifecycle</XPD:ATTR>
<XPD:OBJ name="Top" type="UMLCompositeState" guid="TOP">
<XPD:ATTR name="#Subvertices" type="integer">4</XPD:ATTR>
<XPD:OBJ name="Subvertices[0]" type="UMLPseudostate" guid="BAR">
<XPD:ATTR name="PseudostateKind" type="UMLPseudostateKind">pkSynchronization</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="Subvertices[1]" type="UMLSimpleState" guid="S1">
<XPD:ATTR name="Name" type="string">Open</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="Subvertices[2]" type="UMLCompositeState" guid="S2">
<XPD:ATTR name="Name" type="string">Busy</XPD:ATTR>
<XPD:ATTR name="#Subvertices" type="integer">1</XPD:ATTR>
<XPD:OBJ name="Subvertices[0]" type="UMLPseudostate" guid="CH">
<XPD:ATTR name="PseudostateKind" type="UMLPseudostateKind">pkDecision</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="Subvertices[3]" type="UMLFinalState" guid="F">
</XPD:OBJ>
</XPD:OBJ>
<XPD:ATTR name="#Transitions" type="integer">3</XPD:ATTR>
<XPD:OBJ name="Transitions[0]" type="UMLTransition" guid="T1">
<XPD:ATTR name="GuardCondition" type="string">ok</XPD:ATTR>
<XPD:REF name="Source">S1</XPD:REF>
<XPD:REF name="Target">BAR</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="Transitions[1]" type="UMLTransition" guid="T2">
<XPD:REF name="Source">S2</XPD:REF>
<XPD:REF name="Target">BAR</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="Transitions[2]" type="UMLTransition" guid="T3">
<XPD:REF name="Source">BAR</XPD:REF>
<XPD:REF name="Target">F</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
<XPD:OBJ name="OwnedElements[2]" type="UMLActivityGraph" guid="AG">
<XPD:ATTR name="Name" type="string">Checkout</XPD:ATTR>
<XPD:ATTR name="#Partitions" type="integer">1</XPD:ATTR>
<XPD:OBJ name="Partitions[0]" type="UMLPartition" guid="PA">
<XPD:ATTR name="Name" type="string">Shop</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="Top" type="UMLCompositeState" guid="ATOP">
<XPD:ATTR name="#Subvertices" type="integer">4</XPD:ATTR>
<XPD:OBJ name="Subvertices[0]" type="UMLPseudostate" guid="AI">
<XPD:ATTR name="PseudostateKind" type="UMLPseudostateKind">pkInitial</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="Subvertices[1]" type="UMLActionState" guid="AS">
<XPD:ATTR name="Name" type="string">Pay</XPD:ATTR>
<XPD:ATTR name="IsDynamic" type="boolean">True</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="Subvertices[2]" type="UMLPseudostate" guid="AD">
<XPD:ATTR name="PseudostateKind" type="UMLPseudostateKind">pkDecision</XPD:ATTR>
</XPD:OBJ>
<XPD:OBJ name="Subvertices[3]" type="UMLObjectFlowState" guid="AO">
<XPD:ATTR name="Name" type="string">Receipt</XPD:ATTR>
</XPD:OBJ>
</XPD:OBJ>
<XPD:ATTR name="#Transitions" type="integer">2</XPD:ATTR>
<XPD:OBJ name="Transitions[0]" type="UMLTransition" guid="AT1">
<XPD:REF name="Source">AI</XPD:REF>
<XPD:REF name="Target">AS</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="Transitions[1]" type="UMLTransition" guid="AT2">
<XPD:REF name="Source">AS</XPD:REF>
<XPD:REF name="Target">AO</XPD:REF>
</XPD:OBJ>
<XPD:ATTR name="#OwnedDiagrams" type="integer">1</XPD:ATTR>
<XPD:OBJ name="OwnedDiagrams[0]" type="UMLActivityDiagram" guid="AGD">
<XPD:ATTR name="Name" type="string">Activity</XPD:ATTR>
<XPD:OBJ name="DiagramView" type="UMLActivityDiagramView" guid="AGDV">
<XPD:ATTR name="#OwnedViews" type="integer">3</XPD:ATTR>
<XPD:OBJ name="OwnedViews[0]" type="UMLPseudostateView" guid="AIV">
<XPD:REF name="Model">AI</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OwnedViews[1]" type="UMLTransitionView" guid="AT1V">
<XPD:REF name="Model">AT1</XPD:REF>
</XPD:OBJ>
<XPD:OBJ name="OwnedViews[2]" type="UMLTransitionView" guid="AT2V">
<XPD:REF name="Model">AT2</XPD:REF>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
</XPD:OBJ>
"##;

#[test]
fn imports_staruml1_interactions_state_machines_and_activities() {
    let (_a, v) = import_v1(V1_BEHAVIOR_MODEL);
    // a collaboration instance set is a collaboration; its objects become attributes
    let cis = find(&v, "CIS");
    assert_eq!(cis["_type"], "UMLCollaboration");
    let o2 = find(&v, "O2");
    assert_eq!(o2["_type"], "UMLAttribute");
    assert_eq!(o2["_parent"]["$ref"], "CIS");
    assert_eq!(o2["type"]["$ref"], "SHOP");
    assert_eq!(o2["multiplicity"], "0..*");
    let inter = find(&v, "IIS");
    assert_eq!(inter["_type"], "UMLInteraction");
    // a sequence diagram keeps its flags
    let sd = find(&v, "SD");
    assert_eq!(sd["_type"], "UMLSequenceDiagram");
    assert_eq!(sd["_parent"]["$ref"], "IIS");
    assert_ne!(sd["showSignature"], false, "true is the default, left out on save");
    assert_eq!(sd["showActivation"], false);
    // one lifeline per lifeline view, representing the role
    let lifelines = inter["participants"].as_array().unwrap();
    assert_eq!(lifelines.len(), 2);
    assert_eq!(lifelines[0]["_type"], "UMLLifeline");
    assert_eq!(lifelines[0]["name"], "client");
    assert_eq!(lifelines[0]["represent"]["$ref"], "O1");
    let lv1 = find(&v, "LV1");
    assert_eq!(lv1["_type"], "UMLSeqLifelineView");
    assert_eq!(lv1["model"]["$ref"], lifelines[0]["_id"]);
    assert_eq!(find(&v, "LV1L")["_type"], "UMLLinePartView");
    // a stimulus is a message between the lifelines, its sort from the V1 action
    let m = find(&v, "S");
    assert_eq!(m["_type"], "UMLMessage");
    assert_eq!(m["source"]["$ref"], lifelines[0]["_id"]);
    assert_eq!(m["target"]["$ref"], lifelines[1]["_id"]);
    assert_eq!(m["messageSort"], "asynchCall");
    assert_eq!(m["signature"]["$ref"], "OP");
    assert_eq!(m["tags"][0]["name"], "_iteration");
    assert_eq!(find(&v, "MV")["_type"], "UMLSeqMessageView");

    // the top state of a state machine becomes its region
    let sm = find(&v, "SM");
    let region = &sm["regions"][0];
    assert_eq!(region["_type"], "UMLRegion");
    let verts: Vec<&str> = region["vertices"].as_array().unwrap().iter().map(|x| x["_id"].as_str().unwrap()).collect();
    assert_eq!(verts, ["BAR", "S1", "S2", "F"]);
    assert_eq!(region["transitions"].as_array().unwrap().len(), 3);
    assert_eq!(find(&v, "T1")["guard"], "ok");
    // two in, one out: a join
    assert_eq!(find(&v, "BAR")["kind"], "join");
    assert_eq!(find(&v, "S1")["_type"], "UMLState");
    let s2 = find(&v, "S2");
    assert_eq!(s2["_type"], "UMLState");
    assert_eq!(s2["regions"][0]["vertices"][0]["_id"], "CH");
    // a decision outside an activity is a choice
    assert_eq!(find(&v, "CH")["kind"], "choice");
    assert_eq!(find(&v, "F")["_type"], "UMLFinalState");

    // an activity graph is an activity with nodes, control flows and partitions
    let act = find(&v, "AG");
    assert_eq!(act["_type"], "UMLActivity");
    assert_eq!(act["groups"][0]["_type"], "UMLActivityPartition");
    let nodes: Vec<(&str, &str)> = act["nodes"].as_array().unwrap().iter().map(|x| (x["_id"].as_str().unwrap(), x["_type"].as_str().unwrap())).collect();
    assert_eq!(nodes, [("AI", "UMLInitialNode"), ("AS", "UMLAction"), ("AD", "UMLDecisionNode"), ("AO", "UMLObjectNode")]);
    assert_eq!(find(&v, "AS")["tags"][0]["name"], "_isDynamic");
    assert_eq!(find(&v, "AT1")["_type"], "UMLControlFlow");
    assert_eq!(find(&v, "AT2")["_type"], "UMLObjectFlow");
    assert!(act.get("regions").is_none());
    assert_eq!(find(&v, "AIV")["_type"], "UMLControlNodeView");
    assert_eq!(find(&v, "AT1V")["_type"], "UMLControlFlowView");
    assert_eq!(find(&v, "AT2V")["_type"], "UMLObjectFlowView");
}

#[test]
fn staruml1_import_reports_a_bad_file() {
    assert!(crate::staruml_v1::import("not xml").is_err());
    assert!(crate::staruml_v1::import("<XPD:PROJECT xmlns:XPD=\"x\"><XPD:HEADER/></XPD:PROJECT>").is_err());
    let mut a = App::new(None);
    let p = std::env::temp_dir().join(format!("umlcraft-v1-{}.uml", std::process::id()));
    std::fs::write(&p, "<broken").unwrap();
    a.import_staruml_v1(p.clone());
    std::fs::remove_file(&p).ok();
    assert!(a.message.is_some());
    // the project is not replaced
    assert_eq!(a.repo.name(a.repo.root), "Untitled");
}

#[test]
fn bpmn_diagrams_have_tasks_events_gateways_and_flows() {
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    // a BPMN diagram outside a process gets a new one (bpmn-factory.js:28-57)
    let d = a.change(|a| a.add_diagram(Some(root), "BPMNDiagram"));
    let pr = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(pr), a.repo.name(pr).as_str(), a.repo.name(d).as_str()), ("BPMNProcess", "BPMNProcess1", "BPMNDiagram1"));
    // inside a process it goes in it
    let d2 = a.change(|a| a.add_diagram(Some(pr), "BPMNDiagram"));
    assert_eq!((a.repo.parent(d2), a.repo.name(d2).as_str()), (Some(pr), "BPMNDiagram2"));
    a.diagram = Some(d);
    // a click makes a task at its smallest size: its name plus 5 px around
    let t = a.change(|a| a.create_node("BPMNSendTask", rect(100.0, 100.0, 1.0, 1.0))).unwrap();
    let tm = a.repo.refd(t, "model").unwrap();
    assert_eq!((a.repo.ty(t), a.repo.ty(tm), a.repo.name(tm).as_str(), a.repo.parent(tm)), ("BPMNTaskView", "BPMNSendTask", "BPMNSendTask1", Some(pr)));
    let ev = a.change(|a| a.create_node("BPMNStartEvent", rect(10.0, 100.0, 1.0, 1.0))).unwrap();
    let gw = a.change(|a| a.create_node("BPMNExclusiveGateway", rect(300.0, 100.0, 1.0, 1.0))).unwrap();
    let end = a.change(|a| a.create_node("BPMNEndEvent", rect(400.0, 100.0, 1.0, 1.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let w = (ms.width("BPMNSendTask1", 0).ceil() + 10.0, ms.size() + 10.0);
    assert_eq!((a.repo.f(t, "width"), a.repo.f(t, "height")), w);
    assert_eq!((a.repo.f(ev, "width"), a.repo.f(ev, "height"), a.repo.f(gw, "width")), (32.0, 32.0, 32.0));
    // a floating name 36 px below the event's centre
    let nl = a.repo.refd(ev, "nameLabel").unwrap();
    assert_eq!(a.repo.ty(nl), "NodeLabelView");
    let label_mid = a.repo.f(nl, "top") + a.repo.f(nl, "height") / 2.0;
    assert!((label_mid - (100.0 + 16.0 + 36.0)).abs() < 1.0, "{label_mid}");
    assert_eq!(a.repo.str(nl, "text"), "BPMNStartEvent1");
    // flows: a sequence flow belongs to its source, rounded rectilinear by default
    let sf = tool("BPMNSequenceFlow", AssocInit::Plain);
    assert!(a.can_connect(&sf, ev, t));
    let f1 = a.change(|a| a.create_edge(&sf, ev, t)).unwrap();
    let f1m = a.repo.refd(f1, "model").unwrap();
    assert_eq!((a.repo.ty(f1), a.repo.parent(f1m), a.repo.f(f1, "lineStyle")), ("BPMNSequenceFlowView", a.repo.refd(ev, "model"), 2.0));
    a.change(|a| a.create_edge(&sf, t, gw)).unwrap();
    let f3 = a.change(|a| a.create_edge(&sf, gw, end)).unwrap();
    // the gateway's default flow, and a message flow between two of them
    let gm = a.repo.refd(gw, "model").unwrap();
    let f3m = a.repo.refd(f3, "model");
    a.repo.set_ref(gm, "default", f3m);
    let mf = a.change(|a| a.create_edge(&tool("BPMNMessageFlow", AssocInit::Plain), t, end)).unwrap();
    assert_eq!(a.repo.ty(mf), "BPMNMessageFlowView");
    // a note is not a sequence flow's end
    let note = a.change(|a| a.create_node("Note", rect(500.0, 300.0, 60.0, 40.0))).unwrap();
    assert!(!a.can_connect(&sf, note, gw));
    // event definitions go in the event's definitions
    let em = a.repo.refd(end, "model").unwrap();
    let labels: Vec<&str> = crate::canvas::item_buttons(&a.repo, Some(end), &crate::app::EditKind::Name(em)).into_iter().map(|b| b.1).collect();
    assert_eq!(labels.len(), 10);
    assert_eq!(labels[0], "Add Compensate Event Definition");
    a.bpmn_quick(end, crate::bpmn::Quick::AddEventDefinition("BPMNErrorEventDefinition"));
    let defs = a.repo.list(em, "eventDefinitions");
    assert_eq!((defs.len(), a.repo.name(defs[0]).as_str()), (1, "BPMNErrorEventDefinition1"));
    let kinds: Vec<&str> = crate::bpmn::add_kinds(&a.repo, pr).into_iter().map(|k| k.1).collect();
    assert!(kinds.contains(&"BPMNStartEvent") && kinds.contains(&"BPMNGateway") && !kinds.contains(&"BPMNProcess"));
    let g2 = a.change(|a| a.add_model(pr, "BPMNGateway"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let saved = a.repo.serialize();
    assert!(saved.contains("\"gatewayDirection\": \"unspecified\""), "the direction has no default");
    assert!(saved.contains("\"_type\": \"BPMNErrorEventDefinition\""));
    assert_eq!(a.repo.name(g2), "BPMNGateway1");
    // every kind of view draws, in a picture
    for ty in [
        "BPMNTask", "BPMNReceiveTask", "BPMNServiceTask", "BPMNUserTask", "BPMNManualTask", "BPMNBusinessRuleTask", "BPMNScriptTask", "BPMNCallActivity", "BPMNSubProcess",
        "BPMNAdHocSubProcess", "BPMNTransaction", "BPMNEventBasedGateway", "BPMNInclusiveGateway", "BPMNComplexGateway", "BPMNParallelGateway", "BPMNIntermediateThrowEvent",
        "BPMNIntermediateCatchEvent", "BPMNMessage", "BPMNConversation", "BPMNSubConversation", "BPMNGroup", "BPMNTextAnnotation", "BPMNSubChoreography",
    ] {
        a.change(|a| a.create_node(ty, rect(50.0, 500.0, 1.0, 1.0))).unwrap();
    }
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    crate::export::render(&mut a, d, &ms, crate::export::Format::Svg, 1.0).unwrap();
    let back = Repo::parse(&saved).unwrap();
    assert_eq!(back.serialize(), saved);
}

#[test]
fn bpmn_pools_hold_lanes_and_lanes_hold_flow_elements() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "BPMNDiagram"));
    let pr = a.repo.parent(d).unwrap();
    let task0 = a.change(|a| a.create_node("BPMNTask", rect(700.0, 400.0, 1.0, 1.0))).unwrap();
    let pool = a.change(|a| a.create_node("BPMNParticipant", rect(0.0, 0.0, 600.0, 300.0))).unwrap();
    let pm = a.repo.refd(pool, "model").unwrap();
    assert_eq!((a.repo.ty(pool), a.repo.ty(pm), a.repo.name(pm).as_str()), ("BPMNPoolView", "BPMNParticipant", "BPMNParticipant1"));
    // the pool goes behind what was there
    assert_eq!(a.repo.list(d, "ownedViews")[0], pool);
    // Add Lane, twice: the lanes stack from the top, right of the name
    let l1 = a.change(|a| a.add_lane(pool)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l2 = a.change(|a| a.add_lane(pool)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let l1m = a.repo.refd(l1, "model").unwrap();
    assert_eq!((a.repo.ty(l1m), a.repo.parent(l1m), a.repo.name(l1m).as_str()), ("BPMNLane", Some(pm), "BPMNLane1"));
    let left = 5.0 + 13.0 + 5.0;
    assert_eq!((a.repo.f(l1, "left"), a.repo.f(l1, "top"), a.repo.f(l1, "width")), (left, 0.0, 600.0 - left));
    assert_eq!(a.repo.f(l2, "top"), a.repo.f(l1, "height"));
    assert_eq!(a.repo.refd(l2, "containerView"), Some(pool));
    // a task drawn in a lane belongs to it (container-views)
    let t = a.change(|a| a.create_node("BPMNTask", rect(100.0, 10.0, 1.0, 1.0))).unwrap();
    let tm = a.repo.refd(t, "model").unwrap();
    assert_eq!((a.repo.parent(tm), a.repo.refd(t, "containerView")), (Some(l1m), Some(l1)));
    // a task moved into the pool's lane goes into it, and out again to the process
    assert!(a.can_contain_view(l2, task0));
    a.change(|a| a.set_container(&[task0], Some(l2)));
    assert_eq!(a.repo.parent(a.repo.refd(task0, "model").unwrap()), a.repo.refd(l2, "model"));
    a.change(|a| a.set_container(&[task0], None));
    assert_eq!(a.repo.parent(a.repo.refd(task0, "model").unwrap()), Some(pr));
    // pools and lanes stay behind the boxes
    let order = a.repo.list(d, "ownedViews");
    assert_eq!(&order[..3], &[pool, l1, l2]);
    // a boundary event on the task sits on its border and belongs to it
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let tb = crate::views::node_rect(&a.repo, t);
    let be = a.change(|a| a.create_node("BPMNBoundaryEvent", rect(tb.x + tb.w - 4.0, tb.y + tb.h / 2.0, 1.0, 1.0))).unwrap();
    assert_eq!(a.repo.parent(a.repo.refd(be, "model").unwrap()), Some(tm));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let tb = crate::views::node_rect(&a.repo, t);
    let bb = crate::views::node_rect(&a.repo, be);
    assert_eq!((bb.w, bb.h), (32.0, 32.0));
    assert!(((bb.x + 16.0) - (tb.x + tb.w)).abs() <= 1.0, "on the right border: {bb:?} {tb:?}");
    assert!(!a.repo.bool(be, "containerChangeable"));
    // Add Boundary Event puts one on the border too
    let be2 = a.change(|a| a.add_boundary_event(t)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let b2 = crate::views::node_rect(&a.repo, be2);
    let (cx, cy) = (b2.x + 16.0, b2.y + 16.0);
    let on = |v: f64, e: f64| (v - e).abs() <= 1.0;
    assert!(on(cx, tb.x) || on(cx, tb.x + tb.w) || on(cy, tb.y) || on(cy, tb.y + tb.h), "{b2:?} {tb:?}");
    // a vertical pool and its lanes
    let vp = a.change(|a| a.create_node("BPMNParticipantVert", rect(800.0, 0.0, 300.0, 500.0))).unwrap();
    assert!(a.repo.bool(vp, "isVertical"));
    let vl = a.change(|a| a.create_node("BPMNLaneVert", rect(810.0, 50.0, 100.0, 300.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.refd(vl, "containerView"), a.repo.f(vl, "left")), (Some(vp), 800.0));
    assert!(a.repo.bool(vl, "isVertical"));
    // the quick edit's Add Lane and Add Text Annotation
    let labels: Vec<&str> = crate::canvas::item_buttons(&a.repo, Some(pool), &crate::app::EditKind::Name(pm)).into_iter().map(|b| b.1).collect();
    assert_eq!(labels, ["Add Lane"]);
    assert_eq!(crate::related::actions("BPMNParticipant")[0].label, "Add Text Annotation");
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn bpmn_choreographies_messages_conversations_and_annotations() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "BPMNDiagram"));
    let pr = a.repo.parent(d).unwrap();
    let c = a.change(|a| a.create_node("BPMNChoreographyTask", rect(200.0, 200.0, 120.0, 100.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    // Create Participant (Upper) and (Lower), then make the upper one initiating
    a.bpmn_quick(c, crate::bpmn::Quick::CreateParticipant(true));
    a.bpmn_quick(c, crate::bpmn::Quick::CreateParticipant(false));
    let up = a.repo.ref_list(cm, "upperParticipants");
    assert_eq!((up.len(), a.repo.name(up[0]).as_str(), a.repo.parent(up[0])), (1, "Participant", Some(pr)));
    a.apply_picked(cm, "initiatingParticipant", Some(up[0]));
    // Assign Participant adds to the list
    a.apply_picked(cm, "lowerParticipants", Some(up[0]));
    assert_eq!(a.repo.ref_list(cm, "lowerParticipants").len(), 2);
    // the initiating message above, the return message below and grey, each with a rectilinear link
    a.bpmn_quick(c, crate::bpmn::Quick::Message(true));
    a.bpmn_quick(c, crate::bpmn::Quick::Message(false));
    let links: Vec<_> = a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.ty(*v) == "BPMNMessageLinkView").collect();
    assert_eq!(links.len(), 2);
    for l in &links {
        assert_eq!((a.repo.f(*l, "lineStyle"), a.repo.refd(*l, "head")), (0.0, Some(c)));
    }
    let msgs: Vec<_> = links.iter().map(|l| a.repo.refd(*l, "tail").unwrap()).collect();
    assert!(a.repo.f(msgs[0], "top") + a.repo.f(msgs[0], "height") <= 200.0);
    assert!(a.repo.f(msgs[1], "top") >= 300.0);
    assert_eq!(a.repo.str(msgs[1], "fillColor"), crate::bpmn::INITIATING_FILL);
    assert_eq!(a.repo.ty(a.repo.refd(msgs[0], "model").unwrap()), "BPMNMessage");
    // conversations: a hexagon, joined by a conversation link
    let cv = a.change(|a| a.create_node("BPMNCallConversation", rect(500.0, 50.0, 1.0, 1.0))).unwrap();
    let pool = a.change(|a| a.create_node("BPMNParticipant", rect(450.0, 300.0, 400.0, 150.0))).unwrap();
    let link = a.change(|a| a.create_edge(&tool("BPMNConversationLink", AssocInit::Plain), cv, pool)).unwrap();
    assert_eq!(a.repo.ty(link), "BPMNConversationLinkView");
    // data: an object, a store, an input and an output, joined by a data association
    let dob = a.change(|a| a.create_node("BPMNDataObject", rect(50.0, 400.0, 1.0, 1.0))).unwrap();
    let ds = a.change(|a| a.create_node("BPMNDataStore", rect(150.0, 400.0, 1.0, 1.0))).unwrap();
    a.change(|a| a.create_node("BPMNDataInput", rect(250.0, 400.0, 1.0, 1.0))).unwrap();
    a.change(|a| a.create_node("BPMNDataOutput", rect(350.0, 400.0, 1.0, 1.0))).unwrap();
    a.change(|a| a.create_edge(&tool("BPMNDataAssociation", AssocInit::Plain), dob, ds)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(dob, "width"), a.repo.f(dob, "height")), (32.0, 40.0));
    assert_eq!((a.repo.f(cv, "width"), a.repo.f(cv, "height")), (38.0, 32.0));
    // Add Text Annotation: "Text annotation" below, with an association from it
    let ann = a.add_related(c, crate::bpmn::ANNOTATION).unwrap();
    let am = a.repo.refd(ann, "model").unwrap();
    assert_eq!((a.repo.ty(am), a.repo.str(am, "text").as_str()), ("BPMNTextAnnotation", "Text annotation"));
    let assoc = a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "BPMNAssociationView").unwrap();
    assert_eq!((a.repo.refd(assoc, "tail"), a.repo.refd(assoc, "head")), (Some(ann), Some(c)));
    // and on a flow
    let t1 = a.change(|a| a.create_node("BPMNTask", rect(50.0, 600.0, 1.0, 1.0))).unwrap();
    let t2 = a.change(|a| a.create_node("BPMNTask", rect(300.0, 600.0, 1.0, 1.0))).unwrap();
    let f = a.change(|a| a.create_edge(&tool("BPMNSequenceFlow", AssocInit::Plain), t1, t2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let ann2 = a.add_annotation_to_edge(f).unwrap();
    assert!(a.repo.list(d, "ownedViews").into_iter().any(|v| a.repo.ty(v) == "BPMNAssociationView" && a.repo.refd(v, "tail") == Some(ann2) && a.repo.refd(v, "head") == Some(f)));
    // the text annotation's edit is its text
    let e = a.bpmn_edit(ann).unwrap().unwrap();
    assert!(e.multiline && e.text == "Text annotation");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    crate::export::render(&mut a, d, &ms, crate::export::Format::Png, 1.0).unwrap();
    // the template: a process "Business Process" with its "BPMN Diagram", no UML profile
    let tp = crate::templates::TEMPLATES.iter().find(|t| t.label == "BPMN Business Process Modeling").unwrap();
    let p = crate::templates::project(tp);
    let kids = p.children(p.root);
    assert_eq!(kids.len(), 1);
    let bp = kids[0].1;
    assert_eq!((p.ty(bp), p.name(bp).as_str()), ("BPMNProcess", "Business Process"));
    let bd = p.list(bp, "ownedElements")[0];
    assert_eq!((p.ty(bd), p.name(bd).as_str(), p.bool(bd, "defaultDiagram")), ("BPMNDiagram", "BPMN Diagram", true));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn a_message_reconnected_onto_a_lifeline_goes_onto_its_line() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.change(|a| a.add_diagram(Some(a.repo.root), "UMLSequenceDiagram"));
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 100.0, 300.0))).unwrap();
    let l2 = a.change(|a| a.create_node("UMLLifeline", rect(250.0, 100.0, 100.0, 300.0))).unwrap();
    let l3 = a.change(|a| a.create_node("UMLLifeline", rect(450.0, 100.0, 100.0, 300.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let e = a.change(|a| a.create_edge(&tool("UMLMessage", AssocInit::Plain), l1, l2)).unwrap();
    a.reconnect(e, false, l3).unwrap();
    let lp3 = a.repo.refd(l3, "linePart").unwrap();
    assert_eq!(a.repo.refd(e, "head"), Some(lp3));
    let m = a.repo.refd(e, "model").unwrap();
    assert_eq!(a.repo.refd(m, "target"), a.repo.refd(lp3, "model"));
}

#[test]
fn a_click_makes_boxes_at_their_smallest_size() {
    let ms = measure();
    for (diagram, kinds) in [
        ("UMLClassDiagram", &["UMLClass", "UMLInterface", "UMLEnumeration", "UMLPackage", "UMLDataType", "UMLSignal", "UMLPrimitiveType"][..]),
        ("UMLUseCaseDiagram", &["UMLUseCase", "UMLActor", "UMLUseCaseSubject"][..]),
        ("UMLComponentDiagram", &["UMLComponent", "UMLArtifact"][..]),
        ("UMLDeploymentDiagram", &["UMLNode"][..]),
        ("UMLStatechartDiagram", &["UMLState"][..]),
    ] {
        let mut a = App::new(None);
        let d = a.change(|a| a.add_diagram(Some(a.repo.root), diagram));
        for k in kinds {
            let (w, h) = crate::views::default_size(k);
            let v = a.change(|a| a.create_node(k, rect(40.0, 40.0, w, h))).unwrap_or_else(|| panic!("{k}"));
            a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
            let (w, h) = (a.repo.f(v, "width"), a.repo.f(v, "height"));
            assert!(w >= 20.0 && h >= 20.0, "{k}: {w}x{h}");
        }
    }
}

#[test]
fn aws_diagrams_have_groups_icons_callouts_and_arrows() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "AWSDiagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(model), a.repo.name(model).as_str(), a.repo.name(d).as_str()), ("AWSModel", "AWSModel1", "AWSDiagram1"));
    // a group, with a service drawn in it: the service belongs to the group
    let g = a.change(|a| a.create_node("AWSGroup", rect(0.0, 0.0, 300.0, 200.0))).unwrap();
    let gm = a.repo.refd(g, "model").unwrap();
    assert_eq!((a.repo.name(gm).as_str(), a.repo.str(gm, "icon").as_str()), ("AWSGroup1", "AWS-Cloud.svg"));
    let s = a.change(|a| a.create_node("AWSService", rect(50.0, 60.0, 1.0, 1.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!(a.repo.parent(sm), Some(gm), "an element drawn in a group goes in it");
    assert_eq!(a.repo.refd(s, "containerView"), Some(g));
    assert_eq!(a.repo.str(sm, "icon"), "Amazon-EC2.svg");
    let c = a.change(|a| a.create_node("AWSCallout", rect(400.0, 0.0, 1.0, 1.0))).unwrap();
    let cm = a.repo.refd(c, "model").unwrap();
    assert_eq!(a.repo.name(cm), "1");
    assert_eq!((a.repo.str(c, "fillColor").as_str(), a.repo.str(c, "fontColor").as_str()), ("#000000", "#ffffff"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    // an icon is at least 48 square, its name floating under it
    assert_eq!((a.repo.f(s, "width"), a.repo.f(s, "height")), (48.0, 48.0));
    let sl = a.repo.refd(s, "nameLabel").unwrap();
    assert_eq!(a.repo.ty(sl), "NodeLabelView");
    assert!(a.repo.f(sl, "top") > a.repo.f(s, "top") + 48.0, "the name is under the icon");
    assert_eq!((a.repo.f(c, "width"), a.repo.f(c, "height")), (24.0, 24.0));
    // a group's name is right of its 32 px icon; its icon sets its colour
    let gl = a.repo.refd(g, "nameLabel").unwrap();
    assert_eq!((a.repo.f(gl, "left"), a.repo.f(gl, "height")), (42.0, 32.0));
    a.repo.set(gm, "icon", json!("Region.svg"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(g, "lineColor"), crate::aws::ORBIT);
    // an arrow joins two AWS elements, in the rounded rectilinear style of the preference
    let t = tool("AWSArrow", AssocInit::Plain);
    assert!(a.can_connect(&t, s, c));
    let e = a.change(|a| a.create_edge(&t, s, c)).unwrap();
    assert_eq!(a.repo.ty(e), "AWSArrowView");
    assert_eq!(a.repo.val(e, "lineStyle"), Some(json!(2)));
    let em = a.repo.refd(e, "model").unwrap();
    assert_eq!((a.repo.refd(em, "source"), a.repo.refd(em, "target")), (Some(sm), Some(cm)));
    // Set Icon renames the element from the file name
    a.set_icon(sm, "name", "Amazon-Simple-Storage-Service_S3.svg");
    assert_eq!((a.repo.name(sm).as_str(), a.repo.str(sm, "icon").as_str()), ("Amazon Simple Storage Service S3", "Amazon-Simple-Storage-Service_S3.svg"));
    // a generic group's Dashed is in the Properties panel
    let gg = a.change(|a| a.create_node("AWSGenericGroup", rect(0.0, 300.0, 200.0, 100.0))).unwrap();
    let ggm = a.repo.refd(gg, "model").unwrap();
    assert!(crate::cloud::fields(a.repo.ty(ggm)).contains(&("dashed", "bool")));
    // the saved file keeps the original's names and opens again
    let text = a.repo.serialize();
    assert!(text.contains("AWSServiceView"));
    let back = Repo::parse(&text).unwrap();
    let svc = back.all_of("AWSService");
    assert_eq!(back.str(svc[0], "icon"), "Amazon-Simple-Storage-Service_S3.svg");
    // Model › Add › AWS Model and the template
    let am = a.change(|a| a.add_model(root, "AWSModel"));
    assert_eq!(a.repo.name(am), "AWSModel2");
    let tp = crate::templates::TEMPLATES.iter().find(|t| t.label == "AWS Architecture").unwrap();
    let pr = crate::templates::project(tp);
    let ms0 = pr.children(pr.root);
    assert_eq!(ms0.len(), 1);
    assert_eq!((pr.ty(ms0[0].1), pr.name(ms0[0].1).as_str()), ("AWSModel", "AWS Architecture"));
    let main = pr.children(ms0[0].1)[0].1;
    assert_eq!((pr.ty(main), pr.name(main).as_str(), pr.bool(main, "defaultDiagram")), ("AWSDiagram", "Main", true));
}

#[test]
fn cloud_icons_come_from_the_icon_folder() {
    use crate::cloud::*;
    assert_eq!(picker_label("Amazon-EC2_Instance.svg"), "Amazon EC2 Instance");
    // the picker strips only ".svg", as the original does; the name drops any extension
    assert_eq!(picker_label("Users.png"), "Users.png");
    assert_eq!(name_from_icon("Users.png"), "Users");
    assert_eq!(initials("Amazon-EC2.svg"), "EC2");
    assert_eq!(initials("App-Services.svg"), "AS");
    let dir = std::env::temp_dir().join(format!("umlcraft-cloud-icons-{}", std::process::id()));
    let base = dir.join("aws/service-icons");
    std::fs::create_dir_all(&base).unwrap();
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#ED7100"/></svg>"##;
    std::fs::write(base.join("Amazon-EC2.svg"), svg).unwrap();
    std::fs::write(base.join("AWS-Lambda.svg"), svg).unwrap();
    let files = list_icons(&dir, "aws/service-icons");
    assert_eq!(files, vec!["AWS-Lambda.svg".to_string(), "Amazon-EC2.svg".to_string()]);
    assert_eq!(filter_icons(&files, "ec2"), vec!["Amazon-EC2.svg".to_string()]);
    assert_eq!(filter_icons(&files, "aws lambda"), vec!["AWS-Lambda.svg".to_string()]);
    let url = icon_url(&dir, "aws/service-icons", "Amazon-EC2.svg").unwrap();
    assert!(url.starts_with("data:image/svg+xml;base64,"));
    assert!(icon_url(&dir, "aws/service-icons", "Missing.svg").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn azure_diagrams_have_groups_services_callouts_and_connectors() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "AzureDiagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(model), a.repo.name(model).as_str(), a.repo.name(d).as_str()), ("AzureModel", "AzureModel1", "AzureDiagram1"));
    let g = a.change(|a| a.create_node("AzureGroup", rect(0.0, 0.0, 300.0, 200.0))).unwrap();
    let gm = a.repo.refd(g, "model").unwrap();
    assert_eq!(a.repo.name(gm), "AzureGroup1");
    let s = a.change(|a| a.create_node("AzureService", rect(40.0, 60.0, 1.0, 1.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!((a.repo.parent(sm), a.repo.str(sm, "icon").as_str()), (Some(gm), "App-Services.svg"));
    let s2 = a.change(|a| a.create_node("AzureService", rect(400.0, 60.0, 1.0, 1.0))).unwrap();
    let c = a.change(|a| a.create_node("AzureCallout", rect(400.0, 200.0, 1.0, 1.0))).unwrap();
    assert_eq!(a.repo.name(a.repo.refd(c, "model").unwrap()), "1");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!((a.repo.f(s, "width"), a.repo.f(s, "height")), (48.0, 48.0));
    // the group's quick edit has Dashed and Rounded; the choice is saved on the group
    let menus = crate::cloud::dropdowns(&a.repo, gm);
    assert_eq!(menus.iter().map(|m| m.1).collect::<Vec<_>>(), vec!["Dashed", "Rounded"]);
    a.repo.set(gm, "rounded", json!(true));
    assert!(a.repo.serialize().contains("\"rounded\": true"));
    // a connector: a filled arrow, dotted when dashed
    let t = tool("AzureConnector", AssocInit::Plain);
    assert!(a.can_connect(&t, s, s2));
    let e = a.change(|a| a.create_edge(&t, s, s2)).unwrap();
    assert_eq!((a.repo.ty(e), a.repo.val(e, "lineStyle")), ("AzureConnectorView", Some(json!(2))));
    let em = a.repo.refd(e, "model").unwrap();
    assert!(!crate::cloud::edge_dotted(&a.repo, e));
    assert_eq!(crate::cloud::dropdowns(&a.repo, em)[0].0, "dashed");
    a.repo.set(em, "dashed", json!(true));
    assert!(crate::cloud::edge_dotted(&a.repo, e));
    assert_eq!(crate::cloud::icon_hint("AzureService"), "Azure Icon");
    let back = Repo::parse(&a.repo.serialize()).unwrap();
    assert_eq!(back.all_of("AzureConnector").len(), 1);
    let tp = crate::templates::TEMPLATES.iter().find(|t| t.label == "Azure Architecture").unwrap();
    let pr = crate::templates::project(tp);
    let m0 = pr.children(pr.root)[0].1;
    assert_eq!((pr.ty(m0), pr.name(m0).as_str()), ("AzureModel", "Azure Architecture"));
    assert_eq!(pr.name(pr.children(m0)[0].1), "Main");
}

#[test]
fn gcp_diagrams_have_zones_cards_products_and_paths() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "GCPDiagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(model), a.repo.name(model).as_str(), a.repo.name(d).as_str()), ("GCPModel", "GCPModel1", "GCPDiagram1"));
    // a zone filled by its kind, holding a product drawn in it
    let z = a.change(|a| a.create_node("GCPZone", rect(0.0, 0.0, 500.0, 300.0))).unwrap();
    let zm = a.repo.refd(z, "model").unwrap();
    let p = a.change(|a| a.create_node("GCPProduct", rect(40.0, 40.0, 1.0, 1.0))).unwrap();
    let pm = a.repo.refd(p, "model").unwrap();
    assert_eq!(a.repo.parent(pm), Some(zm));
    assert_eq!((a.repo.str(pm, "icon").as_str(), a.repo.str(pm, "product").as_str()), ("Compute-Engine.svg", "Compute Engine"));
    assert_eq!(a.repo.str(p, "lineColor"), crate::gcp::CARD_LINE);
    let u = a.change(|a| a.create_node("GCPUser", rect(600.0, 0.0, 1.0, 1.0))).unwrap();
    let um = a.repo.refd(u, "model").unwrap();
    assert_eq!(a.repo.str(um, "icon"), "Users.png");
    let s = a.change(|a| a.create_node("GCPService", rect(600.0, 200.0, 1.0, 1.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(z, "fillColor"), "#F6F6F6", "a project zone");
    a.repo.set(zm, "zoneType", json!("kubernetes-cluster"));
    let menus = crate::cloud::dropdowns(&a.repo, zm);
    assert_eq!((menus[0].0, menus[0].1, menus[0].2.len()), ("zoneType", "Zone Type", 20));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(z, "fillColor"), "#FCE4EC");
    // cards: a user's icon is 48 high above its name, a service's 32 wide left of it
    assert!(a.repo.f(u, "height") >= 48.0 + 16.0 + 13.0 - 0.01);
    let ul = a.repo.refd(u, "nameLabel").unwrap();
    assert_eq!(a.repo.f(ul, "top"), 48.0 + 8.0);
    let sl = a.repo.refd(s, "nameLabel").unwrap();
    assert_eq!(a.repo.f(sl, "left"), 600.0 + 48.0);
    // a product: its icon, name and product line; expanded, its details under them
    let pl = a.repo.refd(p, "productLabel").unwrap();
    assert_eq!(a.repo.str(pl, "text"), "Compute Engine");
    assert_eq!(a.repo.str(pl, "fontColor"), crate::gcp::PRODUCT_FONT);
    let h0 = a.repo.f(p, "height");
    a.repo.set(pm, "expanded", json!(true));
    a.repo.set(pm, "expandedInfo", json!("n1-standard-4"));
    a.repo.set(pm, "machineType", json!("standard"));
    a.repo.set(pm, "cores", json!(4));
    a.repo.set(pm, "ram", json!(15.0));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert!(a.repo.f(p, "height") > h0 + 24.0, "room for the details and their icons");
    assert!(a.repo.f(p, "width") >= 48.0 + 8.0 + 32.0 * 3.0 + 8.0 - 0.01);
    let xl = a.repo.refd(p, "expandedLabel").unwrap();
    assert_eq!(a.repo.raw(xl, "visible"), None);
    // a path is coloured by its kind and dotted when optional
    let t = tool("GCPPath", AssocInit::Plain);
    assert!(a.can_connect(&t, u, p));
    let e = a.change(|a| a.create_edge(&t, u, p)).unwrap();
    let em = a.repo.refd(e, "model").unwrap();
    // the path's preference is Rectilinear (0), which the original's `||` passes over for the general one
    assert_eq!(a.repo.val(e, "lineStyle"), Some(json!(1)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(e, "lineColor"), crate::gcp::GOOGLE_BLUE);
    a.repo.set(em, "pathType", json!("optional-secondary"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(e, "lineColor"), crate::gcp::GOOGLE_GRAY);
    assert!(crate::cloud::edge_dotted(&a.repo, e));
    // Set Product Icon sets the product line, Set Icon the name
    assert_eq!(crate::cloud::icon_hint("GCPProduct"), "GCP Product Icon");
    let (_, field) = crate::cloud::icon_base("GCPProduct").unwrap();
    a.set_icon(pm, field, "Cloud-Run.svg");
    assert_eq!((a.repo.str(pm, "product").as_str(), a.repo.name(pm).as_str()), ("Cloud Run", "GCPProduct1"));
    let text = a.repo.serialize();
    assert!(text.contains("\"zoneType\": \"kubernetes-cluster\""));
    let back = Repo::parse(&text).unwrap();
    assert_eq!(back.all_of("GCPProductView").len(), 1);
    let tp = crate::templates::TEMPLATES.iter().find(|t| t.label == "GCP Architecture").unwrap();
    let pr = crate::templates::project(tp);
    let m0 = pr.children(pr.root)[0].1;
    assert_eq!((pr.ty(m0), pr.name(m0).as_str()), ("GCPModel", "GCP Architecture"));
}

#[test]
fn cloud_quick_edits_open_the_icon_picker() {
    use crate::canvas::{item_buttons, ItemKey};
    let ctx = egui::Context::default();
    crate::setup_fonts(&ctx);
    let mut a = App::new(None);
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "GCPDiagram"));
    let p = a.change(|a| a.create_node("GCPProduct", rect(0.0, 0.0, 1.0, 1.0))).unwrap();
    let pm = a.repo.refd(p, "model").unwrap();
    let z = a.change(|a| a.create_node("GCPZone", rect(300.0, 0.0, 200.0, 100.0))).unwrap();
    // a product's name edit has the GCP Product Icon button; a zone's has none
    a.edit_view(p, None);
    let ed = a.editing.clone().unwrap();
    assert!(matches!(ed.kind, crate::app::EditKind::Name(m) if m == pm));
    let keys = item_buttons(&a.repo, ed.host, &ed.kind);
    assert_eq!(keys.iter().map(|k| (k.1, k.3)).collect::<Vec<_>>(), vec![("GCP Product Icon", ItemKey::CloudIcon)]);
    a.editing = None;
    a.edit_view(z, None);
    let ed = a.editing.clone().unwrap();
    assert!(item_buttons(&a.repo, ed.host, &ed.kind).is_empty());
    a.editing = None;
    // the picker opens on the product's folder and renames its product line; without the icon
    // folder it asks for the file name, and it draws in a frame either way
    a.open_icon_picker(pm);
    let ip = a.icon_picker.clone().unwrap();
    assert_eq!((ip.base, ip.field, ip.selected.as_deref()), ("gcp/products", "product", Some("Compute-Engine.svg")));
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| a.icon_picker_ui(ui.ctx()));
    assert!(a.icon_picker.is_some(), "it stays open until OK or Cancel");
    let dir = std::env::temp_dir().join(format!("umlcraft-cloud-picker-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("gcp/products")).unwrap();
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><circle cx="16" cy="16" r="12" fill="#4284F3"/></svg>"##;
    std::fs::write(dir.join("gcp/products/Cloud-Run.svg"), svg).unwrap();
    crate::cloud::set_icon_dir(Some(dir.clone()));
    let _ = ctx.run_ui(egui::RawInput::default(), |ui| a.icon_picker_ui(ui.ctx()));
    crate::cloud::set_icon_dir(None);
    assert!(a.icon_picker.is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_point_tool_ignores_the_drag() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let d = a.change(|a| a.add_diagram(Some(a.repo.root), "UMLActivityDiagram"));
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
    a.tool = Some(Tool { label: "", kind: "UMLDecisionNode", icon: "", line: false, init: AssocInit::Plain });
    a.on_press(canvas, Pos2::new(100.0, 100.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(300.0, 250.0));
    a.on_release(canvas.min, Pos2::new(300.0, 250.0));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let v = a.selection[0];
    let (w, h) = crate::views::default_size("UMLDecisionNode");
    assert_eq!((a.repo.f(v, "width"), a.repo.f(v, "height")), (w, h));
    assert!((a.repo.f(v, "left") - 100.0).abs() <= 5.0);
}

#[test]
fn a_message_tool_looks_through_other_views_for_a_lifeline() {
    use egui::{Modifiers, Pos2};
    let ms = measure();
    let mut a = App::new(None);
    let d = a.change(|a| a.add_diagram(Some(a.repo.root), "UMLSequenceDiagram"));
    let canvas = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(1000.0, 700.0));
    let l1 = a.change(|a| a.create_node("UMLLifeline", rect(50.0, 100.0, 100.0, 300.0))).unwrap();
    let l2 = a.change(|a| a.create_node("UMLLifeline", rect(300.0, 100.0, 100.0, 300.0))).unwrap();
    // a note over the first lifeline's line
    a.change(|a| a.create_node("Note", rect(60.0, 200.0, 80.0, 40.0))).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    a.tool = Some(Tool { label: "Message", kind: "UMLMessage", icon: "", line: true, init: AssocInit::Plain });
    a.on_press(canvas, Pos2::new(100.0, 220.0), Modifiers::NONE);
    a.on_move(canvas, Pos2::new(350.0, 220.0));
    a.on_release(canvas.min, Pos2::new(350.0, 220.0));
    let e = *a.selection.first().expect("a message");
    let tail = a.repo.refd(e, "tail").unwrap();
    assert!(tail == l1 || a.repo.refd(l1, "linePart") == Some(tail), "the message starts on the lifeline under the note");
    let head = a.repo.refd(e, "head").unwrap();
    assert!(head == l2 || a.repo.refd(l2, "linePart") == Some(head));
}

#[test]
fn a_qualifier_row_is_edited_inline() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 100.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 100.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    let end1 = a.repo.child(m, "end1").unwrap();
    let qa = a.change(|a| a.add_qualifier(end1));
    a.change(|a| a.repo.set(qa, "name", json!("id")));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let q = a.repo.refd(e, "tailQualifiersCompartment").unwrap();
    let b = crate::views::node_rect(&a.repo, q);
    let p = crate::views::P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 };
    let hit = a.hit_qualifier(p);
    assert_eq!(hit, Some(e));
    a.edit_view(e, Some(p));
    assert!(matches!(a.editing.as_ref().map(|x| x.kind.clone()), Some(crate::app::EditKind::Feature(x)) if x == qa));
    a.editing.as_mut().unwrap().text = "key: String".into();
    a.close_inline(true);
    assert_eq!((a.repo.name(qa).as_str(), a.repo.str(qa, "type").as_str()), ("key", "String"));
}

#[test]
fn a_role_label_edits_its_association_end() {
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 100.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 100.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    let m = a.repo.refd(e, "model").unwrap();
    let (end1, end2) = (a.repo.child(m, "end1").unwrap(), a.repo.child(m, "end2").unwrap());
    a.change(|a| {
        a.repo.set(end1, "name", json!("owner"));
        a.repo.set(end2, "name", json!("items"));
    });
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    for (field, end) in [("tailRoleNameLabel", end1), ("headRoleNameLabel", end2)] {
        let l = a.repo.refd(e, field).unwrap();
        a.select(vec![l]);
        a.edit_view(l, None);
        assert!(matches!(a.editing.as_ref().map(|x| x.kind.clone()), Some(crate::app::EditKind::Name(x)) if x == end), "{field}");
        a.close_inline(false);
    }
}

#[test]
fn sysml_requirement_diagrams_have_requirements_and_traces() {
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "SysMLRequirementDiagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!((a.repo.ty(model), a.repo.name(model).as_str(), a.repo.name(d).as_str()), ("UMLModel", "Model1", "RequirementDiagram1"));
    // the diagram starts with a frame showing it, its type and its name
    let f = a.repo.list(d, "ownedViews")[0];
    assert_eq!(a.repo.ty(f), "UMLFrameView");
    assert!(a.repo.bool(f, "showType") && a.repo.bool(f, "showDiagramName"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(f, "frameTypeLabel").unwrap(), "text"), "req");
    assert_eq!(a.repo.str(a.repo.refd(f, "nameLabel").unwrap(), "text"), "[Model] Model1 [RequirementDiagram1]");
    let r1 = a.change(|a| a.create_node("SysMLRequirement", rect(50.0, 50.0, 1.0, 1.0))).unwrap();
    let r2 = a.change(|a| a.create_node("SysMLRequirement", rect(300.0, 50.0, 1.0, 1.0))).unwrap();
    let m1 = a.repo.refd(r1, "model").unwrap();
    assert_eq!((a.repo.ty(r1), a.repo.name(m1).as_str(), a.repo.parent(m1)), ("SysMLRequirementView", "Requirement1", Some(model)));
    assert!(a.repo.bool(r1, "suppressAttributes") && a.repo.bool(r1, "suppressOperations"));
    a.repo.set(m1, "id", json!("R1"));
    a.repo.set(m1, "text", json!("Brakes stop the car"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nc = a.repo.refd(r1, "nameCompartment").unwrap();
    assert_eq!(a.repo.str(a.repo.refd(nc, "stereotypeLabel").unwrap(), "text"), "«requirement»");
    let pv = a.repo.refd(r1, "propertyValueCompartment").unwrap();
    let rows: Vec<String> = a.repo.list(pv, "subViews").iter().map(|x| a.repo.str(*x, "text")).collect();
    assert_eq!(rows, vec!["id = \"R1\"", "text = \"Brakes stop the car\""]);
    assert_eq!(a.repo.str(a.repo.list(pv, "subViews")[0], "property"), "id");
    // a trace between two requirements shows its keyword
    let t = tool("SysMLDeriveReqt", AssocInit::Plain);
    assert!(a.can_connect(&t, r2, r1));
    let e = a.change(|a| a.create_edge(&t, r2, r1)).unwrap();
    assert_eq!(a.repo.ty(e), "SysMLDeriveReqtView");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(e, "stereotypeLabel").unwrap(), "text"), "«deriveReqt»");
    // Add Sub-Requirement: a requirement inside the edited one, joined by a containment
    let sub = crate::related::actions("SysMLRequirement").into_iter().find(|x| x.label == "Add Sub-Requirement").unwrap();
    let s = a.add_related(r1, sub).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    assert_eq!(a.repo.parent(sm), Some(m1));
    assert!(a.repo.list(d, "ownedViews").iter().any(|v| a.repo.ty(*v) == "UMLContainmentView"));
    // Model › Add › Requirement
    assert!(a.add_kinds(model).iter().any(|k| k.1 == "SysMLRequirement"));
    let text = a.repo.serialize();
    assert!(text.contains("\"_type\": \"SysMLRequirement\"") && text.contains("\"_type\": \"SysMLPropertyValueCompartmentView\""));
    Repo::parse(&text).unwrap();
}

#[test]
fn sysml_block_definition_diagrams_have_blocks_with_compartments() {
    use crate::canvas::ItemKey;
    use crate::model::Eid;
    use crate::sysml::Quick;
    use crate::views::Measure;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    a.prefs.set("sysml.block.suppressValues", json!(false));
    let d = a.change(|a| a.add_diagram(Some(root), "SysMLBlockDefinitionDiagram"));
    let model = a.repo.parent(d).unwrap();
    assert_eq!(a.repo.name(d), "BlockDefinitionDiagram1");
    let b = a.change(|a| a.create_node("SysMLBlock", rect(50.0, 50.0, 1.0, 1.0))).unwrap();
    let bm = a.repo.refd(b, "model").unwrap();
    assert_eq!((a.repo.ty(b), a.repo.name(bm).as_str()), ("SysMLBlockView", "Block1"));
    // the Block tool hides attributes and operations; the preference shows the values
    assert!(a.repo.bool(b, "suppressAttributes") && a.repo.bool(b, "suppressOperations") && a.repo.bool(b, "suppressParts"));
    assert!(!a.repo.bool(b, "suppressValues"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nc = a.repo.refd(b, "nameCompartment").unwrap();
    assert_eq!(a.repo.str(a.repo.refd(nc, "stereotypeLabel").unwrap(), "text"), "«block»");
    // Add Part shows the parts compartment, with its title over a composite "Part1"
    a.sysml_item_key(b, bm, None, ItemKey::SysML(Quick::AddItem("parts")));
    assert!(!a.repo.bool(b, "suppressParts"));
    let part = a.repo.list(bm, "parts")[0];
    assert_eq!((a.repo.ty(part), a.repo.name(part).as_str(), a.repo.str(part, "aggregation").as_str()), ("SysMLProperty", "Part1", "composite"));
    a.sysml_item_key(b, bm, None, ItemKey::SysML(Quick::AddItem("flowProperties")));
    // Ctrl+Enter on the name adds a property
    assert!(a.sysml_item_key(b, bm, None, ItemKey::Add));
    assert_eq!(a.repo.name(a.repo.list(bm, "properties")[0]), "Property1");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let pc = a.repo.refd(b, "partCompartment").unwrap();
    let row = a.repo.list(pc, "subViews")[0];
    assert_eq!((a.repo.ty(row), a.repo.str(row, "text").as_str()), ("SysMLPartCompartmentItemView", "+Part1"));
    let title = ms.size() * 0.8 + crate::views::PAD;
    assert!((a.repo.f(row, "top") - (a.repo.f(pc, "top") + title + crate::views::PAD)).abs() < 0.01, "the row sits under the title");
    let fc = a.repo.refd(b, "flowPropertyCompartment").unwrap();
    assert_eq!(a.repo.str(a.repo.list(fc, "subViews")[0], "text"), "inout +FlowProperty1");
    assert!(a.info[&b].compartments.iter().any(|c| c.rows.iter().any(|r| r.iv == row)), "the rows can be edited");
    // a port on the block's border, in its ports
    let at = crate::views::P { x: 50.0 + a.repo.f(b, "width"), y: 70.0 };
    let p1 = a.change(|a| a.create_feature_box("SysMLPort", at, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    assert_eq!(a.repo.ty(p1), "SysMLPortView");
    assert_eq!(a.repo.list(bm, "ports").len(), 1);
    assert_eq!(a.repo.refd(p1, "containerView"), Some(b));
    assert_eq!(a.create_feature_box("SysMLPort", crate::views::P { x: 900.0, y: 900.0 }, rect(0.0, 0.0, 0.0, 0.0)).err(), Some("Port should be placed in a block or a part".to_string()));
    // a connector between two ports belongs to the diagram's owner
    a.sysml_item_key(b, bm, None, ItemKey::SysML(Quick::AddPort));
    let p2 = a.repo.list(d, "ownedViews").into_iter().filter(|v| a.repo.ty(*v) == "SysMLPortView").last().unwrap();
    assert_ne!(p1, p2);
    let t = tool("SysMLConnector", AssocInit::Plain);
    assert!(a.can_connect(&t, p1, p2));
    assert!(!a.can_connect(&t, p1, b));
    let e = a.change(|a| a.create_edge(&t, p1, p2)).unwrap();
    let em = a.repo.refd(e, "model").unwrap();
    assert_eq!((a.repo.ty(e), a.repo.ty(em), a.repo.parent(em)), ("SysMLConnectorView", "SysMLConnector", Some(model)));
    assert!(a.repo.refd(e, "itemFlowToHeadLabel").is_some());
    // Add Composited Block: an association whose end at the edited block is composite
    a.sysml_item_key(b, bm, None, ItemKey::SysML(Quick::CompositedBlock));
    let assoc = a.repo.all_of("UMLAssociation")[0];
    let end1 = a.repo.child(assoc, "end1").unwrap();
    assert_eq!((a.repo.refd(end1, "reference"), a.repo.str(end1, "aggregation").as_str()), (Some(bm), "composite"));
    // value types, interface blocks and constraint blocks keep their keyword
    let vt = a.change(|a| a.create_node("SysMLValueType", rect(400.0, 50.0, 1.0, 1.0))).unwrap();
    let cb = a.change(|a| a.create_node("SysMLConstraintBlock", rect(400.0, 250.0, 1.0, 1.0))).unwrap();
    let cbm = a.repo.refd(cb, "model").unwrap();
    a.repo.set(cbm, "stereotype", json!("mine"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let kw = |a: &App, v: Eid| a.repo.str(a.repo.refd(a.repo.refd(v, "nameCompartment").unwrap(), "stereotypeLabel").unwrap(), "text");
    assert_eq!((kw(&a, vt).as_str(), kw(&a, cb).as_str()), ("«valueType»", "«constraint»"));
    // Model › Add: SysML items in a block, no plain attribute; parameters in a constraint block
    let adds = a.add_kinds(bm);
    assert!(adds.iter().any(|k| k.1 == "sysml:parts") && !adds.iter().any(|k| k.1 == "UMLAttribute"));
    let prm = a.change(|a| a.add_feature(cbm, "sysml:parameters")).unwrap();
    assert_eq!((a.repo.name(prm).as_str(), a.repo.field_of(prm).as_deref()), ("Parameter1", Some("parameters")));
    // a port dragged from the explorer goes onto its block's box only
    let port = a.repo.list(bm, "ports")[0];
    assert!(a.drop_model(port, crate::views::P { x: 700.0, y: 700.0 }).is_err());
    let dropped = a.drop_model(port, crate::views::P { x: 55.0, y: 55.0 }).unwrap();
    assert_eq!(a.repo.refd(dropped[0], "containerView"), Some(b));
    let text = a.repo.serialize();
    Repo::parse(&text).unwrap();
}

#[test]
fn sysml_internal_block_and_parametric_diagrams_have_parts_ports_and_parameters() {
    use crate::canvas::ItemKey;
    use crate::sysml::Quick;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let bdd = a.change(|a| a.add_diagram(Some(root), "SysMLBlockDefinitionDiagram"));
    let model = a.repo.parent(bdd).unwrap();
    let bv = a.change(|a| a.create_node("SysMLBlock", rect(50.0, 50.0, 1.0, 1.0))).unwrap();
    let b2v = a.change(|a| a.create_node("SysMLBlock", rect(300.0, 50.0, 1.0, 1.0))).unwrap();
    let (bm, b2) = (a.repo.refd(bv, "model").unwrap(), a.repo.refd(b2v, "model").unwrap());
    a.sysml_item_key(b2v, b2, None, ItemKey::SysML(Quick::AddItem("flowProperties")));
    let fp = a.repo.list(b2, "flowProperties")[0];
    a.repo.set(fp, "direction", json!("in"));
    // an internal block diagram is offered on a block only, and goes in it
    assert_eq!(crate::sysml::diagram_offered(&a.repo, Some(model), "SysMLInternalBlockDiagram"), Some(false));
    assert_eq!(crate::sysml::diagram_offered(&a.repo, Some(bm), "SysMLInternalBlockDiagram"), Some(true));
    let d = a.change(|a| a.add_diagram(Some(bm), "SysMLInternalBlockDiagram"));
    assert_eq!(a.repo.parent(d), Some(bm));
    let frame = a.repo.list(d, "ownedViews")[0];
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(frame, "nameLabel").unwrap(), "text"), "[Block] Block1 [InternalBlockDiagram1]");
    assert_eq!(a.repo.str(a.repo.refd(frame, "frameTypeLabel").unwrap(), "text"), "ibd");
    // Part and Reference: properties of the diagram's block, with a part view
    let pv = a.change(|a| a.create_node("SysMLPart", rect(100.0, 100.0, 1.0, 1.0))).unwrap();
    let rv = a.change(|a| a.create_node("SysMLReference", rect(100.0, 300.0, 1.0, 1.0))).unwrap();
    let pm = a.repo.refd(pv, "model").unwrap();
    assert_eq!((a.repo.ty(pv), a.repo.name(pm).as_str(), a.repo.field_of(pm).as_deref(), a.repo.str(pm, "aggregation").as_str()), ("SysMLPartView", "Part1", Some("parts"), "composite"));
    assert_eq!(a.repo.field_of(a.repo.refd(rv, "model").unwrap()).as_deref(), Some("references"));
    // a block dropped on the diagram becomes a part typed by it
    let dropped = a.drop_model(b2, crate::views::P { x: 400.0, y: 100.0 }).unwrap()[0];
    let p2 = a.repo.refd(dropped, "model").unwrap();
    assert_eq!((a.repo.name(p2).as_str(), a.repo.refd(p2, "type")), ("Part2", Some(b2)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let nl = a.repo.refd(a.repo.refd(dropped, "nameCompartment").unwrap(), "nameLabel").unwrap();
    assert_eq!(a.repo.str(nl, "text"), "Part2: Block2");
    assert_eq!(a.repo.f(nl, "left"), a.repo.f(dropped, "left") + 10.0 + crate::views::PAD, "the name sits 10 px in");
    // ports: on a typed part they go to its type, on the frame to the diagram's block
    let x2 = a.repo.f(dropped, "left") + a.repo.f(dropped, "width");
    let port = a.change(|a| a.create_feature_box("SysMLPort", crate::views::P { x: x2 - 1.0, y: 110.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    assert_eq!(a.repo.parent(a.repo.refd(port, "model").unwrap()), Some(b2));
    assert_eq!(a.create_feature_box("SysMLPort", crate::views::P { x: 101.0, y: 101.0 }, rect(0.0, 0.0, 0.0, 0.0)).err(), Some("Port should be placed in a typed part".to_string()));
    let fport = a.change(|a| a.create_feature_box("SysMLPort", crate::views::P { x: 9.0, y: 500.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    assert_eq!((a.repo.parent(a.repo.refd(fport, "model").unwrap()), a.repo.refd(fport, "containerView")), (Some(bm), Some(frame)));
    // an item flow on a connector between typed ports
    let port_m = a.repo.refd(port, "model").unwrap();
    let fport_m = a.repo.refd(fport, "model").unwrap();
    let t = tool("SysMLConnector", AssocInit::Plain);
    let e = a.change(|a| a.create_edge(&t, fport, port)).unwrap();
    let c = a.repo.refd(e, "model").unwrap();
    a.ask_item_flow(c);
    assert!(a.select_one.is_none(), "untyped ends refuse");
    a.repo.set_ref(port_m, "type", Some(b2));
    a.repo.set_ref(fport_m, "type", Some(bm));
    a.ask_item_flow(c);
    assert_eq!(a.select_one.as_ref().map(|s| s.options.clone()), Some(vec!["From Block2 To Block1", "From Block1 To Block2"]));
    a.select_one = None;
    a.add_item_flow(c, true);
    let f = a.repo.all_of("SysMLItemFlow")[0];
    assert_eq!((a.repo.refd(f, "source"), a.repo.refd(f, "target"), a.repo.ref_list(f, "realizingConnectors")), (Some(bm), Some(b2), vec![c]));
    a.repo.set_ref_list(f, "conveyed", &[fp]);
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.str(a.repo.refd(e, "itemFlowToHeadLabel").unwrap(), "text"), "FlowProperty1");
    // Create Block on a part: a new block in the part's block, as its type
    a.create_sysml_type(pm, "SysMLBlock", "Engine");
    let eng = a.repo.refd(pm, "type").unwrap();
    assert_eq!((a.repo.ty(eng), a.repo.name(eng).as_str(), a.repo.parent(eng)), ("SysMLBlock", "Engine", Some(bm)));
    // a parametric diagram in a constraint block, with a constraint property and its parameters
    a.diagram = Some(bdd);
    let cbv = a.change(|a| a.create_node("SysMLConstraintBlock", rect(50.0, 400.0, 1.0, 1.0))).unwrap();
    let cb2v = a.change(|a| a.create_node("SysMLConstraintBlock", rect(300.0, 400.0, 1.0, 1.0))).unwrap();
    let (cb, cb2) = (a.repo.refd(cbv, "model").unwrap(), a.repo.refd(cb2v, "model").unwrap());
    let par = a.change(|a| a.add_diagram(Some(cb), "SysMLParametricDiagram"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, par);
    assert_eq!(a.repo.str(a.repo.refd(a.repo.list(par, "ownedViews")[0], "frameTypeLabel").unwrap(), "text"), "par");
    let cpv = a.change(|a| a.create_node("SysMLConstraintProperty", rect(100.0, 100.0, 160.0, 100.0))).unwrap();
    let cp = a.repo.refd(cpv, "model").unwrap();
    assert_eq!((a.repo.ty(cpv), a.repo.name(cp).as_str(), a.repo.field_of(cp).as_deref()), ("SysMLConstraintPropertyView", "Constraint1", Some("constraints")));
    a.sysml_item_key(cpv, cp, None, ItemKey::SysML(Quick::ConstraintParameter));
    assert!(a.repo.all_of("SysMLProperty").iter().all(|p| a.repo.field_of(*p).as_deref() != Some("parameters")), "an untyped constraint property refuses");
    a.repo.set_ref(cp, "type", Some(cb2));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, par);
    a.sysml_item_key(cpv, cp, None, ItemKey::SysML(Quick::ConstraintParameter));
    let prm = a.repo.list(cb2, "parameters")[0];
    let prv = a.repo.list(par, "ownedViews").into_iter().find(|v| a.repo.ty(*v) == "SysMLConstraintParameterView").unwrap();
    assert_eq!((a.repo.name(prm).as_str(), a.repo.refd(prv, "containerView")), ("Parameter1", Some(cpv)));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, par);
    assert_eq!(a.repo.f(prv, "left"), 100.0 + a.repo.f(cpv, "width") - 14.0 + 1.0, "against the inside of the right side");
    // the parameter tool on the frame: a parameter of the diagram's constraint block
    let fpv = a.change(|a| a.create_feature_box("SysMLConstraintParameter", crate::views::P { x: 9.0, y: 500.0 }, rect(0.0, 0.0, 0.0, 0.0))).unwrap();
    assert_eq!(a.repo.parent(a.repo.refd(fpv, "model").unwrap()), Some(cb));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn sysml_viewpoints_and_compartment_toggles() {
    use crate::model::Eid;
    let ms = measure();
    let mut a = App::new(None);
    let root = a.repo.root;
    let d = a.change(|a| a.add_diagram(Some(root), "UMLPackageDiagram"));
    let sv = a.change(|a| a.create_node("SysMLStakeholder", rect(50.0, 50.0, 1.0, 1.0))).unwrap();
    let vpv = a.change(|a| a.create_node("SysMLViewpoint", rect(300.0, 50.0, 1.0, 1.0))).unwrap();
    let vv = a.change(|a| a.create_node("SysMLView", rect(300.0, 300.0, 1.0, 1.0))).unwrap();
    let (s, vp, v) = (a.repo.refd(sv, "model").unwrap(), a.repo.refd(vpv, "model").unwrap(), a.repo.refd(vv, "model").unwrap());
    assert_eq!((a.repo.name(s).as_str(), a.repo.name(vp).as_str(), a.repo.name(v).as_str()), ("Stakeholder1", "Viewpoint1", "View1"));
    a.repo.set(s, "concern", json!("safety"));
    a.repo.set_ref_list(vp, "stakeholders", &[s]);
    a.repo.set(vp, "purpose", json!("review"));
    // a view conforms to a viewpoint, and exposes the stakeholder
    let conform = tool("SysMLConform", AssocInit::Plain);
    let e = a.change(|a| a.create_edge(&conform, vv, vpv)).unwrap();
    let x = a.change(|a| a.create_edge(&tool("SysMLExpose", AssocInit::Plain), vv, sv)).unwrap();
    assert_eq!((a.repo.ty(e), a.repo.ty(x)), ("SysMLConformView", "SysMLExposeView"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let rows = |a: &App, view: Eid| -> Vec<String> { a.repo.list(a.repo.refd(view, "propertyValueCompartment").unwrap(), "subViews").iter().map(|r| a.repo.str(*r, "text")).collect() };
    assert_eq!(rows(&a, sv), vec!["concern = \"safety\""]);
    assert_eq!(rows(&a, vpv), vec!["/stakeholder = Stakeholder1", "/concern = \"safety\"", "purpose = \"review\""]);
    assert_eq!(rows(&a, vv), vec!["/viewpoint = Viewpoint1", "/stakeholder = Stakeholder1"]);
    let label = |a: &App, edge: Eid| a.repo.str(a.repo.refd(edge, "stereotypeLabel").unwrap(), "text");
    assert_eq!((label(&a, e).as_str(), label(&a, x).as_str()), ("«conform»", "«expose»"));
    let st = a.repo.refd(a.repo.refd(sv, "nameCompartment").unwrap(), "stereotypeLabel").unwrap();
    assert_eq!(a.repo.str(st, "text"), "«stakeholder»");
    // Format › Suppress Property Values hides the values; on a block the SysML toggles apply
    a.select(vec![sv]);
    assert_eq!(a.flag_state("suppressPropertyValues"), Some(false));
    a.toggle_flag("suppressPropertyValues");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.repo.raw(a.repo.refd(sv, "propertyValueCompartment").unwrap(), "visible"), Some(&json!(false)));
    let bdd = a.change(|a| a.add_diagram(Some(root), "SysMLBlockDefinitionDiagram"));
    let b = a.change(|a| a.create_node("SysMLBlock", rect(50.0, 50.0, 1.0, 1.0))).unwrap();
    a.select(vec![b]);
    for f in ["suppressConstraints", "suppressParts", "suppressPorts", "suppressReferences", "suppressValues", "suppressProperties", "suppressFlowProperties"] {
        assert_eq!(a.flag_state(f), Some(true), "{f}");
    }
    assert!(a.flag_views("suppressPropertyValues").is_empty(), "a block has no property values");
    a.toggle_flag("suppressParts");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, bdd);
    assert!(a.repo.raw(a.repo.refd(b, "partCompartment").unwrap(), "visible").is_none());
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn mermaid_requirement_diagrams_become_sysml_requirements() {
    let ms = measure();
    let mut a = App::new(None);
    let md = crate::sysml::parse_mermaid(crate::sysml::MERMAID_SAMPLE).unwrap();
    assert_eq!(md.blocks.len(), 3);
    assert_eq!((md.blocks[0].risk.as_str(), md.blocks[0].verify.as_str()), ("High", "Test"));
    assert_eq!(md.rels[1], ("verifies".to_string(), "brake_test".to_string(), "brakes".to_string()));
    let back = crate::sysml::parse_mermaid("requirementDiagram\n  requirement a {\n  id: 1\n  }\n  requirement b {\n  }\n  b <- refines - a\n").unwrap();
    assert_eq!(back.rels[0], ("refines".to_string(), "a".to_string(), "b".to_string()));
    assert!(crate::sysml::parse_mermaid("requirementDiagram\n  requirement a {\n  colour: red\n  }\n").is_err());
    let d = a.generate_mermaid(crate::sysml::MERMAID_SAMPLE, &ms).unwrap();
    assert_eq!((a.repo.ty(d), a.repo.name(d).as_str()), ("SysMLRequirementDiagram", "Requirement Diagram by Mermaid"));
    let pkg = a.repo.parent(d).unwrap();
    assert_eq!(a.repo.name(pkg), "Package by Mermaid");
    let reqs = a.repo.all_of("SysMLRequirement");
    let brakes = reqs.iter().copied().find(|r| a.repo.name(*r) == "brakes").unwrap();
    let abs = reqs.iter().copied().find(|r| a.repo.name(*r) == "abs").unwrap();
    assert_eq!((a.repo.str(brakes, "id").as_str(), a.repo.str(brakes, "documentation").as_str()), ("1", "Risk: High\nVerifyMethod: Test\n"));
    assert_eq!(crate::views::type_name(&a.repo, abs, "stereotype"), "functionalRequirement");
    assert_eq!(a.repo.parent(abs), Some(brakes), "contains moves the requirement into its container");
    let el = a.repo.all_of("UMLClass").into_iter().find(|c| a.repo.name(*c) == "brake_test").unwrap();
    let attrs: Vec<(String, String)> = a.repo.list(el, "attributes").iter().map(|x| (a.repo.name(*x), a.repo.str(*x, "defaultValue"))).collect();
    assert_eq!(attrs, vec![("Type".to_string(), "simulation".to_string()), ("DocRef".to_string(), "tests/brakes.md".to_string())]);
    let v = a.repo.all_of("SysMLVerify")[0];
    assert_eq!((a.repo.refd(v, "source"), a.repo.refd(v, "target")), (Some(el), Some(brakes)));
    let views: Vec<String> = a.repo.list(d, "ownedViews").iter().map(|x| a.repo.ty(*x).to_string()).collect();
    assert!(views.contains(&"UMLContainmentView".to_string()) && views.contains(&"SysMLVerifyView".to_string()) && !views.contains(&"UMLFrameView".to_string()));
    Repo::parse(&a.repo.serialize()).unwrap();
}

#[test]
fn model_add_offers_ports_and_collaborations() {
    let mut a = App::new(None);
    let root = a.repo.root;
    let m = a.change(|a| a.add_model(root, "UMLModel"));
    assert!(a.add_kinds(m).iter().any(|k| k.1 == "UMLCollaboration"));
    let c = a.change(|a| a.add_model(m, "UMLClass"));
    assert!(a.add_kinds(c).iter().any(|k| k.1 == "UMLPort" && k.2));
    let p = a.change(|a| a.add_feature(c, "UMLPort")).unwrap();
    assert_eq!((a.repo.ty(p), a.repo.field_of(p).as_deref()), ("UMLPort", Some("attributes")));
    assert_eq!(a.repo.name(p), "Port1");
}

#[test]
fn quick_edit_components_sit_in_rows_left_and_right() {
    use crate::app::{EditKind, Editing};
    use crate::quickedit::{Act, Comp, Side};
    let mut a = App::new(None);
    let c = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 160.0, 100.0))).unwrap();
    let m = a.repo.refd(c, "model").unwrap();
    let ed = |kind: EditKind, host: Option<crate::model::Eid>| Editing { kind, text: String::new(), rect: R::default(), multiline: false, center: false, fresh: false, host, orig: None };
    let hint = |p: &crate::quickedit::Placed| match &p.comp {
        Comp::Button { hint, .. } | Comp::Menu { hint, .. } => hint.clone(),
        Comp::Combo(_) => "Multiplicity".to_string(),
    };
    let at = |a: &App, e: &Editing, h: &str| a.quick_components(e).into_iter().find(|p| hint(p).starts_with(h)).map(|p| (p.side, p.row));
    // quickedits/uml.json "uml.class"
    let e = ed(EditKind::Name(m), Some(c));
    assert_eq!(at(&a, &e, "Visibility"), Some((Side::Left, 1)));
    assert_eq!(at(&a, &e, "Add Note"), Some((Side::Left, 2)));
    assert_eq!(at(&a, &e, "Add Constraint"), Some((Side::Left, 2)));
    assert_eq!(at(&a, &e, "Add Attribute"), Some((Side::Right, 1)));
    assert_eq!(at(&a, &e, "Add Template Parameter"), Some((Side::Right, 1)));
    assert_eq!(at(&a, &e, "Add Sub-Class"), Some((Side::Right, 2)));
    assert_eq!(at(&a, &e, "Add Composited Class"), Some((Side::Right, 3)));
    assert_eq!(at(&a, &e, "Add Port"), Some((Side::Right, 4)));
    let right1: Vec<String> = a.quick_components(&e).iter().filter(|p| p.side == Side::Right && p.row == 1).map(hint).collect();
    assert_eq!(right1, ["Add Attribute (Ctrl+Enter)", "Add Operation (Ctrl+Shift+Enter)", "Add Template Parameter", "Add Reception"]);
    // an association: its Order of Ends and visibility, and the ends' dropdowns and combo
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let l = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c, c2)).unwrap();
    let am = a.repo.refd(l, "model").unwrap();
    let e = ed(EditKind::Name(am), None);
    assert_eq!(at(&a, &e, "Order of Ends"), Some((Side::Left, 1)));
    assert_eq!(at(&a, &e, "Add Note"), Some((Side::Left, 2)));
    let end = a.repo.child(am, "end1").unwrap();
    let e = ed(EditKind::Name(end), None);
    let left1: Vec<String> = a.quick_components(&e).iter().filter(|p| p.side == Side::Left && p.row == 1).map(hint).collect();
    assert_eq!(left1, ["Aggregation Kind", "Navigability", "Visibility"], "nearest the field first");
    assert_eq!(at(&a, &e, "Add Qualifier"), Some((Side::Left, 2)));
    assert_eq!(at(&a, &e, "Multiplicity"), Some((Side::Right, 1)));
    // an end showing its owner attribute keeps only the owner attribute and navigability
    // (_tailEndSole, elements.js:5650)
    let attr = a.change(|a| a.add_feature(a.repo.refd(c2, "model").unwrap(), "UMLAttribute")).unwrap();
    a.change(|a| a.repo.set_ref(end, "ownerAttribute", Some(attr)));
    let hints: Vec<String> = a.quick_components(&e).iter().map(hint).collect();
    assert_eq!(hints, ["Owner Attribute", "Navigability"]);
    assert!(crate::quickedit::end_owned(&a.repo, end));
    // a transition's Add Trigger Event and Add Effect Behavior; an action's Add Trigger Event
    let root = a.repo.root;
    a.change(|a| a.add_diagram(Some(root), "UMLStatechartDiagram"));
    let s1 = a.change(|a| a.create_node("UMLState", rect(40.0, 40.0, 100.0, 40.0))).unwrap();
    let s2 = a.change(|a| a.create_node("UMLState", rect(300.0, 40.0, 100.0, 40.0))).unwrap();
    let t = a.change(|a| a.create_edge(&tool("UMLTransition", AssocInit::Plain), s1, s2)).unwrap();
    let tm = a.repo.refd(t, "model").unwrap();
    let e = ed(EditKind::Name(tm), None);
    assert_eq!(at(&a, &e, "Add Note"), Some((Side::Left, 1)));
    assert_eq!(at(&a, &e, "Visibility"), None, "uml.transition has none");
    let trig = a.quick_components(&e).into_iter().find_map(|p| match p.comp {
        Comp::Button { act: Act::Key(h, k), hint, .. } if hint == "Add Trigger Event" => Some((h, k)),
        _ => None,
    });
    let (h, k) = trig.unwrap();
    assert_eq!(h, t);
    a.item_key(h, EditKind::Name(tm), k);
    let evs = a.repo.list(tm, "triggers");
    assert_eq!((evs.len(), a.repo.ty(evs[0])), (1, "UMLEvent"));
    a.item_key(t, EditKind::Name(tm), crate::canvas::ItemKey::AddEffect);
    assert!(a.select_one.is_some(), "asks which kind of behaviour");
    a.select_one = None;
    a.change(|a| a.add_diagram(Some(root), "UMLActivityDiagram"));
    let s = a.change(|a| a.create_node("UMLAction", rect(300.0, 300.0, 60.0, 40.0))).unwrap();
    let sm = a.repo.refd(s, "model").unwrap();
    let e = ed(EditKind::Name(sm), Some(s));
    assert_eq!(at(&a, &e, "Add Trigger Event"), Some((Side::Right, 1)));
    assert_eq!(at(&a, &e, "Add Output Pin"), Some((Side::Left, 1)));
    assert_eq!(at(&a, &e, "Add Decision"), Some((Side::Right, 3)));
    assert_eq!(at(&a, &e, "Add Initial Node"), Some((Side::Right, 4)));
    a.item_key(s, EditKind::Name(sm), crate::canvas::ItemKey::AddTrigger);
    assert_eq!(a.repo.list(sm, "triggers").len(), 1);
    let saved = a.repo.serialize();
    assert!(saved.contains("\"triggers\""));
    Repo::parse(&saved).unwrap();
}

#[test]
fn a_typed_multiplicity_applies_when_the_end_edit_closes() {
    use crate::app::EditKind;
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let c1 = a.change(|a| a.create_node("UMLClass", rect(40.0, 40.0, 100.0, 60.0))).unwrap();
    let c2 = a.change(|a| a.create_node("UMLClass", rect(400.0, 40.0, 100.0, 60.0))).unwrap();
    let e = a.change(|a| a.create_edge(&tool("UMLAssociation", AssocInit::Plain), c1, c2)).unwrap();
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let m = a.repo.refd(e, "model").unwrap();
    let end = a.repo.child(m, "end1").unwrap();
    let pts = crate::views::points_of(&a.repo, e);
    a.edit_view(e, Some(crate::views::P { x: pts[0].x + 10.0, y: pts[0].y }));
    assert_eq!(a.editing.as_ref().map(|x| x.kind.clone()), Some(EditKind::Name(end)));
    // Esc drops it
    a.edit_combo = Some((end, "2..5".into()));
    a.close_inline(false);
    assert_eq!(a.repo.str(end, "multiplicity"), "");
    a.edit_view(e, Some(crate::views::P { x: pts[0].x + 10.0, y: pts[0].y }));
    a.editing.as_mut().map(|x| x.orig = Some(x.text.clone()));
    a.edit_combo = Some((end, "2..5".into()));
    a.close_inline(true);
    assert_eq!(a.repo.str(end, "multiplicity"), "2..5");
    assert!(a.edit_combo.is_none());
    // a connector end has the same dropdowns and combo (quickedits/uml.json "uml.connector-end1")
    let conn_end = a.change(|a| a.repo.create("UMLConnectorEnd", &[], None));
    let ed = crate::app::Editing { kind: EditKind::Name(conn_end), text: String::new(), rect: R::default(), multiline: false, center: false, fresh: false, host: None, orig: None };
    assert_eq!(a.quick_components(&ed).len(), 4, "aggregation, navigability, visibility, multiplicity");
}

#[test]
fn actors_take_attributes_and_operations() {
    use crate::app::EditKind;
    use crate::canvas::{item_buttons, ItemKey};
    let ms = measure();
    let mut a = App::new(None);
    let d = a.diagram.unwrap();
    let v = a.change(|a| a.create_node("UMLActor", rect(40.0, 40.0, 40.0, 80.0))).unwrap();
    let m = a.repo.refd(v, "model").unwrap();
    let hints: Vec<&str> = item_buttons(&a.repo, Some(v), &EditKind::Name(m)).into_iter().map(|b| b.1).collect();
    assert_eq!(hints, ["Add Attribute", "Add Operation"], "quickedits/uml.json \"uml.actor\"");
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    let h0 = a.repo.f(v, "height");
    assert!(a.repo.bool(v, "suppressAttributes"), "suppressed by default");
    a.item_key(v, EditKind::Name(m), ItemKey::Add);
    a.item_key(v, EditKind::Name(m), ItemKey::AddOperation);
    assert_eq!((a.repo.list(m, "attributes").len(), a.repo.list(m, "operations").len()), (1, 1));
    assert!(!a.repo.bool(v, "suppressAttributes") && !a.repo.bool(v, "suppressOperations"));
    a.info = crate::views::layout_diagram(&mut a.repo, &ms, d);
    assert_eq!(a.info[&v].shape, "actor");
    assert_eq!(a.info[&v].compartments.len(), 2, "attributes and operations under the name");
    assert!(a.repo.f(v, "height") > h0, "the actor grows for them");
    let nc = a.repo.refd(v, "nameCompartment").unwrap();
    assert!(a.info[&v].compartments[0].y >= a.repo.f(nc, "top") + crate::views::NAME_MIN_H - 0.5);
    a.take_pending_item();
    Repo::parse(&a.repo.serialize()).unwrap();
}
