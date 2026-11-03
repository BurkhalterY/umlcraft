//! The window: menu bar, Working Diagrams, Toolbox, canvas, Model Explorer, Editors and status bar.
//! Colours and sizes measured from the original's window (MECHANICS.md "Window").

use crate::app::{App, Area, AssocInit, Tool};
use crate::canvas::EguiMeasure;
use crate::icons;
use crate::model::Eid;
use crate::render;
use crate::views::{self, R};
use egui::{Align2, Color32, CornerRadius, FontId, Frame, Margin, Panel, Pos2, Rect, Sense, Stroke, Vec2};
use serde_json::{json, Value};

pub const WINDOW_WIDTH: f32 = 1200.0; // window.js:38
pub const WINDOW_HEIGHT: f32 = 900.0; // window.js:39
pub const SIDEBAR_WIDTH: f32 = 200.0;
pub const NAVIGATOR_WIDTH: f32 = 300.0;
pub const TOOLBAR_WIDTH: f32 = 30.0;
pub const STATUSBAR_HEIGHT: f32 = 27.0;
pub const TOOLBOX_HEIGHT: f32 = 400.0;
pub const EDITORS_HEIGHT: f32 = 400.0;
pub const HEADER_HEIGHT: f32 = 34.0;
pub const SCREENSHOT_FRAMES: u32 = 120;

pub const LAYER0: Color32 = Color32::from_rgb(0x3a, 0x3f, 0x41);
pub const LAYER1: Color32 = Color32::from_rgb(0x4a, 0x4c, 0x4e);
pub const TOOLBAR: Color32 = Color32::from_rgb(0x5d, 0x5f, 0x60);
pub const TEXT: Color32 = Color32::from_rgb(0xe3, 0xe3, 0xe3);
pub const DIM: Color32 = Color32::from_rgb(0xb4, 0xb6, 0xb7);
pub const BORDER: Color32 = Color32::from_rgb(0x56, 0x59, 0x5b);
pub const INPUT: Color32 = Color32::from_rgb(0x40, 0x42, 0x44);
pub const INPUT_BORDER: Color32 = Color32::from_rgb(0x63, 0x66, 0x68);
pub const ACCENT: Color32 = Color32::from_rgb(0x1f, 0x7a, 0xe8);
pub const ACCENT_TEXT: Color32 = Color32::from_rgb(0x4f, 0xa6, 0xff);
pub const CURRENT_ROW: Color32 = Color32::from_rgb(0x33, 0x35, 0x3c);
pub const HOVER_ROW: Color32 = Color32::from_rgb(0x55, 0x58, 0x5a);
pub const OK_GREEN: Color32 = Color32::from_rgb(0x8d, 0xc6, 0x3f);

const fn node(label: &'static str, kind: &'static str, icon: &'static str) -> Tool {
    Tool { label, kind, icon, line: false, init: AssocInit::Plain }
}
const fn line(label: &'static str, kind: &'static str, icon: &'static str, init: AssocInit) -> Tool {
    Tool { label, kind, icon, line: true, init }
}

pub struct Group {
    pub id: &'static str,
    pub label: &'static str,
    pub items: &'static [Tool],
}

// toolbox groups and tools: extensions/essential/uml/toolbox/uml.json
const ANNOTATIONS: Group = Group {
    id: "annotations",
    label: "Annotations",
    items: &[node("Text", "Text", "text"), node("Note", "Note", "note"), line("Note Link", "NoteLink", "notelink", AssocInit::Plain)],
};
const CLASS_GROUPS: &[Group] = &[
    Group {
        id: "class",
        label: "Classes (Basic)",
        items: &[
            node("Class", "UMLClass", "class"),
            node("Interface", "UMLInterface", "interface"),
            line("Association", "UMLAssociation", "association", AssocInit::Plain),
            line("Directed Association", "UMLAssociation", "directed", AssocInit::Directed),
            line("Aggregation", "UMLAssociation", "aggregation", AssocInit::Shared),
            line("Composition", "UMLAssociation", "composition", AssocInit::Composite),
            line("Dependency", "UMLDependency", "dependency", AssocInit::Plain),
            line("Generalization", "UMLGeneralization", "generalization", AssocInit::Plain),
            line("Interface Realization", "UMLInterfaceRealization", "realization", AssocInit::Plain),
        ],
    },
    Group {
        id: "class-adv",
        label: "Classes (Advanced)",
        items: &[
            node("Signal", "UMLSignal", "datatype"),
            node("DataType", "UMLDataType", "datatype"),
            node("PrimitiveType", "UMLPrimitiveType", "datatype"),
            node("Enumeration", "UMLEnumeration", "enumeration"),
            line("Realization", "UMLRealization", "realization", AssocInit::Plain),
        ],
    },
    Group {
        id: "package",
        label: "Packages",
        items: &[node("Package", "UMLPackage", "package"), node("Model", "UMLModel", "model"), line("Dependency", "UMLDependency", "dependency", AssocInit::Plain)],
    },
    ANNOTATIONS,
];
const USECASE_GROUPS: &[Group] = &[
    Group {
        id: "usecase",
        label: "Use Cases",
        items: &[
            node("Package", "UMLPackage", "package"),
            node("Use Case Subject", "UMLUseCaseSubject", "subject"),
            node("Use Case", "UMLUseCase", "usecase"),
            node("Actor", "UMLActor", "actor"),
            line("Association", "UMLAssociation", "association", AssocInit::Plain),
            line("Directed Association", "UMLAssociation", "directed", AssocInit::Directed),
            line("Generalization", "UMLGeneralization", "generalization", AssocInit::Plain),
            line("Dependency", "UMLDependency", "dependency", AssocInit::Plain),
            line("Include", "UMLInclude", "include", AssocInit::Plain),
            line("Extend", "UMLExtend", "include", AssocInit::Plain),
        ],
    },
    ANNOTATIONS,
];
const OTHER_GROUPS: &[Group] = &[ANNOTATIONS];

/// Explorer order: kind priority, then the order they were added (core.js:311 getOrdering;
/// priorities from the UML metamodel; kinds without one, such as diagrams, come first).
pub(crate) fn order_rank(t: &str) -> u32 {
    match t {
        "UMLModel" => 1000, "UMLPackage" => 1001, "UMLSubsystem" => 1002, "UMLUseCaseSubject" => 1003,
        "UMLTemplateParameter" => 1101, "UMLParameter" => 1102, "UMLEnumerationLiteral" => 1103, "UMLPort" => 1104,
        "UMLAttribute" => 1105, "UMLOperation" => 1106, "UMLReception" => 1107, "UMLExtensionPoint" => 1108, "UMLSlot" => 1109,
        "UMLClass" => 1201, "UMLInterface" => 1202, "UMLSignal" => 1203, "UMLDataType" => 1204, "UMLPrimitiveType" => 1205,
        "UMLEnumeration" => 1206, "UMLArtifact" => 1207, "UMLComponent" => 1208, "UMLNode" => 1209, "UMLUseCase" => 1210, "UMLActor" => 1211,
        "UMLInstance" => 1301, "UMLObject" => 1302, "UMLCollaboration" => 1401, "UMLStateMachine" => 1402, "UMLInteraction" => 1403, "UMLActivity" => 1404,
        "UMLAssociationEnd" => 1801, "UMLAssociation" => 1802, "UMLDependency" => 1806, "UMLGeneralization" => 1807, "UMLRealization" => 1808,
        "UMLInterfaceRealization" => 1809, "UMLComponentRealization" => 1810, "UMLAbstraction" => 1811, "UMLInclude" => 1816, "UMLExtend" => 1817,
        "UMLTransition" => 1819, "UMLMessage" => 1822, "UMLProfile" => 1901, "UMLStereotype" => 1903,
        _ => 0,
    }
}

fn text_galley_width(ui: &egui::Ui, t: &str, size: f32) -> f32 {
    ui.ctx().fonts_mut(|f| f.layout_no_wrap(t.to_string(), FontId::proportional(size), TEXT).size().x)
}

impl App {
    // ------------------------------------------------------------ explorer
    pub fn explorer_children(&self, e: Eid) -> Vec<Eid> {
        let mut list: Vec<(u32, usize, Eid)> = self
            .repo
            .children(e)
            .into_iter()
            .filter(|(k, _)| k != "ownedViews" && k != "subViews")
            .enumerate()
            .map(|(i, (_, c))| (order_rank(self.repo.ty(c)), i, c))
            .collect();
        if self.sort_by_name {
            list.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| self.repo.name(a.2).cmp(&self.repo.name(b.2))));
        } else {
            list.sort_by_key(|x| (x.0, x.1));
        }
        list.into_iter().map(|x| x.2).collect()
    }

    pub fn label_of(&self, e: Eid) -> String {
        let r = &self.repo;
        let t = r.ty(e);
        let name = r.name(e);
        let nm = |x: Option<Eid>| x.map(|x| r.name(x)).unwrap_or_default();
        if r.has(e, "end1") && r.has(e, "end2") {
            let a = r.child(e, "end1").and_then(|x| r.refd(x, "reference"));
            let b = r.child(e, "end2").and_then(|x| r.refd(x, "reference"));
            let s = format!("({}—{})", nm(a), nm(b));
            return if name.is_empty() { s } else { format!("{name} {s}") };
        }
        if r.has(e, "source") && r.has(e, "target") {
            let s = format!("({}→{})", nm(r.refd(e, "source")), nm(r.refd(e, "target")));
            return if name.is_empty() { s } else { format!("{name} {s}") };
        }
        if r.kind_of(e, "RelationshipEnd") || t == "UMLAssociationEnd" {
            return if name.is_empty() { format!("({})", nm(r.refd(e, "reference"))) } else { name };
        }
        if t == "Tag" {
            return format!("{name} = {}", r.str(e, "value"));
        }
        if name.is_empty() { format!("({})", t.trim_start_matches("UML")) } else { name }
    }

    fn tree_row(&mut self, ui: &mut egui::Ui, e: Eid, depth: usize, has_kids: bool, open: bool, search_parent: Option<String>) {
        let w = ui.available_width();
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::click_and_drag());
        let x0 = rect.left() + 4.0 + depth as f32 * 16.0;
        let p = ui.painter();
        if has_kids {
            let c = Pos2::new(x0 + 8.0, rect.center().y);
            let tri = if open {
                vec![c + Vec2::new(-3.5, -1.5), c + Vec2::new(3.5, -1.5), c + Vec2::new(0.0, 3.0)]
            } else {
                vec![c + Vec2::new(-1.5, -3.5), c + Vec2::new(3.0, 0.0), c + Vec2::new(-1.5, 3.5)]
            };
            p.add(egui::Shape::convex_polygon(tri, TEXT, Stroke::NONE));
        }
        let ir = Rect::from_min_size(Pos2::new(x0 + 16.0, rect.center().y - 8.0), Vec2::splat(16.0));
        icons::draw(p, ir, icons::key_for(&self.repo, e));
        let mut label = self.label_of(e);
        if self.show_stereotype {
            let st = views::stereotype_text(&self.repo, e);
            if !st.is_empty() {
                label = format!("{st} {label}");
            }
        }
        if let Some(parent) = search_parent {
            label = format!("{label} — {parent}");
        }
        let tx = ir.right() + 6.0;
        let tw = text_galley_width(ui, &label, 13.0);
        if self.focus == Some(e) {
            let lr = Rect::from_min_size(Pos2::new(tx - 3.0, rect.top() + 1.0), Vec2::new(tw + 6.0, 20.0));
            p.rect_filled(lr, 0.0, Color32::from_rgb(0x2d, 0x32, 0x38));
            p.rect_stroke(lr, 0.0, Stroke::new(1.0, Color32::from_rgb(0x4f, 0x99, 0xff)), egui::StrokeKind::Inside);
        }
        p.text(Pos2::new(tx, rect.center().y), Align2::LEFT_CENTER, &label, FontId::proportional(13.0), TEXT);

        let ty = self.repo.ty(e).to_string();
        if ty != "Project" {
            resp.dnd_set_drag_payload(e);
        }
        // dropping a row on another moves it there (model-explorer-view.js:182-241, app-context.js:1117-1144)
        if let Some(src) = resp.dnd_hover_payload::<Eid>() {
            let ok = self.can_relocate(*src, e);
            if ok {
                ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0x4f, 0x99, 0xff)), egui::StrokeKind::Inside);
            } else if *src != e {
                ui.ctx().set_cursor_icon(egui::CursorIcon::NotAllowed);
            }
        }
        if let Some(src) = resp.dnd_release_payload::<Eid>() {
            let src = *src;
            if self.can_relocate(src, e) {
                let field = self.repo.field_of(src).unwrap_or_default();
                self.change(|a| a.repo.relocate(src, e, &field));
                self.expanded.insert(e);
                self.focus = Some(src);
                self.reveal(src);
            }
        }
        if resp.clicked() || resp.secondary_clicked() {
            self.last_area = Area::Explorer;
            let on_toggle = resp.interact_pointer_pos().map(|q| q.x < x0 + 16.0).unwrap_or(false);
            if on_toggle && has_kids && resp.clicked() {
                if !self.expanded.remove(&e) {
                    self.expanded.insert(e);
                }
            } else {
                self.focus_element(e);
            }
        }
        if resp.double_clicked() {
            if self.is_diagram(e) {
                self.open_diagram(e);
            } else if has_kids && !self.expanded.remove(&e) {
                self.expanded.insert(e);
            }
        }
        resp.context_menu(|ui| self.model_menu(ui, e));
        if self.reveal_pending == Some(e) {
            resp.scroll_to_me(Some(egui::Align::Center));
            self.reveal_pending = None;
        }
    }

    fn tree(&mut self, ui: &mut egui::Ui, e: Eid, depth: usize) {
        let kids = self.explorer_children(e);
        let open = self.expanded.contains(&e);
        self.tree_row(ui, e, depth, !kids.is_empty(), open, None);
        if open {
            for k in kids {
                if self.repo.alive(k) {
                    self.tree(ui, k, depth + 1);
                }
            }
        }
    }

    fn model_menu(&mut self, ui: &mut egui::Ui, e: Eid) {
        let t = self.repo.ty(e).to_string();
        let is_pkg = self.repo.kind_of(e, "UMLPackage") || t == "Project";
        let is_classifier = self.repo.kind_of(e, "UMLClassifier");
        let mut adds: Vec<(&str, &str, bool)> = vec![];
        if is_pkg {
            if t == "Project" {
                adds.push(("Model", "UMLModel", false));
            }
            for (l, k) in [("Package", "UMLPackage"), ("Class", "UMLClass"), ("Interface", "UMLInterface"), ("Enumeration", "UMLEnumeration"), ("DataType", "UMLDataType"), ("Actor", "UMLActor"), ("Use Case", "UMLUseCase")] {
                adds.push((l, k, false));
            }
        }
        for (l, k) in [("Attribute", "UMLAttribute"), ("Operation", "UMLOperation"), ("Literal", "UMLEnumerationLiteral"), ("Parameter", "UMLParameter")] {
            if self.can_add_feature(e, k) {
                adds.push((l, k, true));
            }
        }
        if !adds.is_empty() {
            ui.menu_button("Add", |ui| {
                for (l, k, feature) in adds {
                    if ui.button(l).clicked() {
                        let created = self.change(|a| if feature { a.add_feature(e, k) } else { Some(a.add_model(e, k)) });
                        self.expanded.insert(e);
                        self.focus = created;
                        ui.close();
                    }
                }
            });
        }
        if is_pkg || is_classifier {
            ui.menu_button("Add Diagram", |ui| {
                for (l, k) in [("Class Diagram", "UMLClassDiagram"), ("Use Case Diagram", "UMLUseCaseDiagram")] {
                    if ui.button(l).clicked() {
                        self.change(|a| a.add_diagram(Some(e), k));
                        self.expanded.insert(e);
                        ui.close();
                    }
                }
            });
        }
        if self.is_diagram(e) && ui.button("Open Diagram").clicked() {
            self.open_diagram(e);
            ui.close();
        }
        if !self.is_diagram(e) && t != "Project" && ui.button("Select In Diagram").clicked() {
            self.focus = Some(e);
            self.select_in_diagram();
            ui.close();
        }
        ui.separator();
        self.last_area = Area::Explorer;
        self.clipboard_items(ui);
        self.move_items(ui);
        if ui.add_enabled(t != "Project", egui::Button::new("Delete from Model")).clicked() {
            self.change(|a| a.delete_models(&[e]));
            ui.close();
        }
    }

    fn explorer_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(HEADER_HEIGHT);
            ui.add_space(13.0);
            ui.label(egui::RichText::new("MODEL EXPLORER").color(TEXT).size(13.0));
            ui.add_space(8.0);
            let w = ui.available_width() - 30.0;
            let r = ui.add_sized(Vec2::new(w, 23.0), egui::TextEdit::singleline(&mut self.search).hint_text("🔍").margin(Margin::symmetric(10, 3)));
            let _ = r;
            let (r, resp) = ui.allocate_exact_size(Vec2::new(14.0, 20.0), Sense::click());
            let c = r.center();
            let p = ui.painter();
            p.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-4.0, -1.5), c + Vec2::new(4.0, -1.5), c + Vec2::new(0.0, -6.5)], TEXT, Stroke::NONE));
            p.add(egui::Shape::convex_polygon(vec![c + Vec2::new(-4.0, 1.5), c + Vec2::new(4.0, 1.5), c + Vec2::new(0.0, 6.5)], TEXT, Stroke::NONE));
            // the settings button opens the model-explorer-settings menu (model-explorer-view.js:632-641)
            let resp = resp.on_hover_text("Settings");
            egui::Popup::menu(&resp).show(|ui| self.explorer_settings_menu(ui));
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let q = self.search.trim().to_lowercase();
            if q.is_empty() {
                let root = self.repo.root;
                self.tree(ui, root, 0);
            } else {
                // quick search: names containing the text, at most 99, with " — parent" (model-explorer-view.js:644-686)
                let all: Vec<Eid> = self.repo.all().filter(|e| !self.is_view(*e) && self.repo.name(*e).to_lowercase().contains(&q)).take(99).collect();
                for e in all {
                    let parent = self.repo.parent(e).map(|p| self.repo.name(p)).filter(|n| !n.is_empty());
                    let before = self.focus;
                    self.tree_row(ui, e, 0, false, false, parent);
                    // picking a result jumps to it in the tree
                    if self.focus == Some(e) && before != Some(e) {
                        self.search.clear();
                        self.reveal(e);
                    }
                }
            }
            ui.add_space(8.0);
        });
        // arrow keys walk the tree while it is the last area used
        if self.last_area == Area::Explorer && self.editing.is_none() && !text_field_focused(ui.ctx()) && self.search.trim().is_empty() {
            let keys = ui.input(|i| (i.key_pressed(egui::Key::ArrowUp), i.key_pressed(egui::Key::ArrowDown), i.key_pressed(egui::Key::ArrowLeft), i.key_pressed(egui::Key::ArrowRight)));
            self.tree_keys(keys);
        }
        if ui.ui_contains_pointer() && ui.input(|i| i.key_pressed(egui::Key::Delete)) && self.last_area == Area::Explorer && !text_field_focused(ui.ctx()) {
            if let Some(f) = self.focus.filter(|f| self.repo.ty(*f) != "Project" && !self.is_view(*f)) {
                self.change(|a| a.delete_models(&[f]));
            }
        }
    }

    /// The model-explorer-settings menu: sort order, stereotype text, Expand All and Collapse All.
    fn explorer_settings_menu(&mut self, ui: &mut egui::Ui) {
        if ui.radio(!self.sort_by_name, "Sort By Added").clicked() {
            self.sort_by_name = false;
            ui.close();
        }
        if ui.radio(self.sort_by_name, "Sort By Name").clicked() {
            self.sort_by_name = true;
            ui.close();
        }
        ui.separator();
        if ui.checkbox(&mut self.show_stereotype, "Show Stereotype Text").clicked() {
            ui.close();
        }
        ui.separator();
        if ui.button("Expand All").clicked() {
            self.expand_all();
            ui.close();
        }
        if ui.button("Collapse All").clicked() {
            self.expanded.clear();
            ui.close();
        }
    }

    /// Expand All: open every element that has children in the tree (model-explorer-view.js:455-477).
    pub fn expand_all(&mut self) {
        let all: Vec<Eid> = self.repo.all().filter(|e| !self.is_view(*e)).collect();
        for e in all {
            if !self.explorer_children(e).is_empty() {
                self.expanded.insert(e);
            }
        }
    }

    /// The rows the tree shows, top to bottom.
    pub fn visible_rows(&self) -> Vec<Eid> {
        fn walk(a: &App, e: Eid, out: &mut Vec<Eid>) {
            out.push(e);
            if a.expanded.contains(&e) {
                for k in a.explorer_children(e) {
                    if a.repo.alive(k) {
                        walk(a, k, out);
                    }
                }
            }
        }
        let mut out = vec![];
        walk(self, self.repo.root, &mut out);
        out
    }

    /// Up/Down move to the previous or next row; Right opens a row or goes to its first child; Left
    /// closes it or goes to its parent (the tree widget's own keys).
    pub fn tree_keys(&mut self, (up, down, left, right): (bool, bool, bool, bool)) {
        if !(up || down || left || right) {
            return;
        }
        let rows = self.visible_rows();
        let Some(f) = self.focus.filter(|f| rows.contains(f)) else {
            if let Some(&first) = rows.first() {
                self.focus_element(first);
            }
            return;
        };
        let i = rows.iter().position(|x| *x == f).unwrap_or(0);
        let kids = !self.explorer_children(f).is_empty();
        let target = if up {
            i.checked_sub(1).map(|j| rows[j])
        } else if down {
            rows.get(i + 1).copied()
        } else if right {
            if kids && !self.expanded.contains(&f) {
                self.expanded.insert(f);
                None
            } else if kids {
                self.explorer_children(f).first().copied()
            } else {
                None
            }
        } else if kids && self.expanded.contains(&f) {
            self.expanded.remove(&f);
            None
        } else {
            self.repo.parent(f)
        };
        if let Some(t) = target {
            self.focus_element(t);
            self.reveal(t);
        }
    }

    // ------------------------------------------------------------ editors
    fn fields_for(&self, e: Eid) -> Vec<(&'static str, &'static str)> {
        let r = &self.repo;
        let t = r.ty(e);
        let end = |p: &str| -> Vec<(&'static str, &'static str)> {
            let names: [(&'static str, &'static str); 13] = [
                ("name", "text"), ("reference", "ref"), ("stereotype", "text"), ("visibility", "enum"), ("navigable", "enum"), ("aggregation", "enum"), ("multiplicity", "text"),
                ("defaultValue", "text"), ("isReadOnly", "bool"), ("isOrdered", "bool"), ("isUnique", "bool"), ("isDerived", "bool"), ("isID", "bool"),
            ];
            names
                .iter()
                .map(|(n, k)| {
                    let full: &'static str = match p {
                        "end1" => Box::leak(format!("end1.{n}").into_boxed_str()),
                        "end2" => Box::leak(format!("end2.{n}").into_boxed_str()),
                        _ => n,
                    };
                    (full, *k)
                })
                .collect()
        };
        if t == "Project" {
            return vec![("name", "text"), ("author", "text"), ("company", "text"), ("copyright", "text"), ("version", "text")];
        }
        if self.is_diagram(e) {
            return vec![("name", "text"), ("defaultDiagram", "bool")];
        }
        if t == "UMLAssociation" {
            let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isDerived", "bool"), ("-", "")];
            f.extend(end("end1"));
            f.push(("-", ""));
            f.extend(end("end2"));
            return f;
        }
        if r.has(e, "source") && r.has(e, "target") {
            let mut f = vec![("name", "text"), ("source", "ref"), ("target", "ref"), ("stereotype", "text"), ("visibility", "enum")];
            if t == "UMLGeneralization" {
                f.push(("discriminator", "text"));
            }
            if t == "UMLExtend" {
                f.push(("condition", "text"));
            }
            return f;
        }
        match t {
            "UMLAttribute" => vec![
                ("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isStatic", "bool"), ("isLeaf", "bool"), ("type", "type"), ("multiplicity", "text"),
                ("isReadOnly", "bool"), ("isOrdered", "bool"), ("isUnique", "bool"), ("defaultValue", "text"), ("isDerived", "bool"), ("aggregation", "enum"), ("isID", "bool"),
            ],
            "UMLOperation" => vec![
                ("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isStatic", "bool"), ("isLeaf", "bool"), ("concurrency", "enum"), ("isQuery", "bool"),
                ("isAbstract", "bool"), ("specification", "text"),
            ],
            "UMLParameter" => vec![("name", "text"), ("stereotype", "text"), ("type", "type"), ("multiplicity", "text"), ("defaultValue", "text"), ("direction", "enum")],
            "UMLEnumerationLiteral" => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum")],
            "UMLAssociationEnd" => end(""),
            _ if r.kind_of(e, "UMLClassifier") => {
                let mut f = vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum"), ("isAbstract", "bool"), ("isFinalSpecialization", "bool"), ("isLeaf", "bool")];
                if t == "UMLClass" {
                    f.push(("isActive", "bool"));
                }
                f
            }
            _ if r.kind_of(e, "UMLModelElement") => vec![("name", "text"), ("stereotype", "text"), ("visibility", "enum")],
            _ if r.has(e, "name") || r.kind_of(e, "Model") => vec![("name", "text")],
            _ => vec![],
        }
    }

    fn target(&self, e: Eid, path: &str) -> Option<(Eid, String)> {
        match path.split_once('.') {
            Some((a, b)) => self.repo.child(e, a).map(|c| (c, b.to_string())),
            None => Some((e, path.to_string())),
        }
    }

    fn display_value(&self, obj: Eid, field: &str, kind: &str) -> String {
        if kind == "type" || field == "stereotype" {
            return views::type_name(&self.repo, obj, field);
        }
        match self.repo.val(obj, field) {
            Some(Value::String(s)) => s,
            Some(Value::Null) | None => String::new(),
            Some(v) => v.to_string(),
        }
    }

    fn commit_text(&mut self, obj: Eid, field: &str, kind: &str, text: String) {
        if self.display_value(obj, field, kind) == text {
            return;
        }
        let field = field.to_string();
        let kind = kind.to_string();
        self.change(|a| {
            if kind == "type" || field == "stereotype" {
                let linked = a.repo.raw(obj, &field).and_then(|v| a.repo.by_ref(v)).map(|t| a.repo.name(t));
                if linked.as_deref() == Some(text.as_str()) && !text.is_empty() {
                    return;
                }
                if text.is_empty() && field == "type" {
                    a.repo.set_raw(obj, "type", json!(""));
                } else if text.is_empty() {
                    a.repo.remove(obj, &field);
                } else {
                    a.repo.set(obj, &field, json!(text));
                }
            } else {
                a.repo.set(obj, &field, json!(text));
            }
        });
    }

    fn enum_options(field: &str) -> &'static [&'static str] {
        match field {
            "visibility" => &["public", "protected", "private", "package"],
            "aggregation" => &["none", "shared", "composite"],
            "navigable" => &["unspecified", "navigable", "notNavigable"],
            "direction" => &["in", "inout", "out", "return"],
            "concurrency" => &["sequential", "guarded", "concurrent"],
            _ => &[],
        }
    }

    fn prop_row(&mut self, ui: &mut egui::Ui, e: Eid, path: &'static str, kind: &str) {
        let Some((obj, field)) = self.target(e, path) else { return };
        ui.horizontal(|ui| {
            ui.set_min_height(29.0);
            let (lr, _) = ui.allocate_exact_size(Vec2::new(116.0, 22.0), Sense::hover());
            let p = ui.painter();
            let label_right = lr.right() - 10.0;
            match path.split_once('.') {
                Some((pre, f)) => {
                    let fw = text_galley_width(ui, f, 12.0);
                    p.text(Pos2::new(label_right, lr.center().y), Align2::RIGHT_CENTER, f, FontId::proportional(12.0), TEXT);
                    p.text(Pos2::new(label_right - fw, lr.center().y), Align2::RIGHT_CENTER, format!("{pre}."), FontId::proportional(12.0), DIM);
                }
                None => {
                    p.text(Pos2::new(label_right, lr.center().y), Align2::RIGHT_CENTER, path, FontId::proportional(12.0), TEXT);
                }
            }
            let w = ui.available_width() - 2.0;
            match kind {
                "text" | "type" => {
                    let cur = self.display_value(obj, &field, kind);
                    let buf = self.prop_buf.entry(path.to_string()).or_insert(cur);
                    let r = ui.add_sized(Vec2::new(w, 23.0), egui::TextEdit::singleline(buf).font(FontId::proportional(12.0)).margin(Margin::symmetric(9, 3)));
                    if r.lost_focus() {
                        let text = buf.clone();
                        self.commit_text(obj, &field, kind, text);
                    }
                }
                "bool" => {
                    let mut v = self.repo.bool(obj, &field);
                    if ui.checkbox(&mut v, "").changed() {
                        let f = field.clone();
                        self.change(|a| a.repo.set(obj, &f, json!(v)));
                    }
                }
                "enum" => {
                    let cur = self.display_value(obj, &field, kind);
                    let mut sel = cur.clone();
                    egui::ComboBox::from_id_salt(path).selected_text(&sel).width(w).show_ui(ui, |ui| {
                        for o in Self::enum_options(&field) {
                            ui.selectable_value(&mut sel, o.to_string(), *o);
                        }
                    });
                    if sel != cur {
                        let f = field.clone();
                        self.change(|a| a.repo.set(obj, &f, json!(sel)));
                    }
                }
                "ref" => {
                    if let Some(t) = self.repo.refd(obj, &field) {
                        let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        icons::draw(ui.painter(), ir, icons::key_for(&self.repo, t));
                        ui.label(egui::RichText::new(self.repo.name(t)).size(12.0).color(TEXT));
                    }
                }
                _ => {}
            }
        });
    }

    fn editors_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(HEADER_HEIGHT);
            ui.add_space(13.0);
            ui.label(egui::RichText::new("EDITORS").color(TEXT).size(13.0));
        });
        let focus = self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f));
        if self.prop_for != focus {
            self.prop_buf.clear();
            self.prop_for = focus;
            self.doc_buf = focus.map(|f| self.repo.str(f, "documentation")).unwrap_or_default();
        }
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Frame::NONE.inner_margin(Margin { left: 13, right: 13, top: 0, bottom: 10 }).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                // Styles comes first and only shows for a view selection (style-editor-view.js:666-668)
                let views_selected = self.selection.iter().any(|v| self.repo.alive(*v));
                if views_selected {
                    if section_header(ui, "Styles", self.style_open) {
                        self.style_open = !self.style_open;
                    }
                    if self.style_open {
                        self.style_ui(ui);
                        ui.add_space(6.0);
                    }
                    ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), Stroke::new(1.0, BORDER));
                }
                if section_header(ui, "Properties", focus.is_some() && self.props_open) && focus.is_some() {
                    self.props_open = !self.props_open;
                }
                if let (Some(e), true) = (focus, self.props_open) {
                    for (path, kind) in self.fields_for(e) {
                        if path == "-" {
                            ui.add_space(8.0);
                            ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), Stroke::new(1.0, BORDER));
                            ui.add_space(8.0);
                        } else {
                            self.prop_row(ui, e, path, kind);
                        }
                    }
                    let t = self.repo.ty(e).to_string();
                    let mut buttons: Vec<(&str, &str)> = vec![];
                    if t == "UMLEnumeration" {
                        buttons.push(("+ Literal", "UMLEnumerationLiteral"));
                    }
                    if self.can_add_feature(e, "UMLAttribute") {
                        buttons.push(("+ Attribute", "UMLAttribute"));
                        buttons.push(("+ Operation", "UMLOperation"));
                    }
                    if t == "UMLOperation" {
                        buttons.push(("+ Parameter", "UMLParameter"));
                    }
                    if !buttons.is_empty() {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.add_space(116.0);
                            for (label, ty) in buttons {
                                if ui.add(egui::Button::new(egui::RichText::new(label).size(12.0)).fill(HOVER_ROW).stroke(Stroke::new(1.0, INPUT_BORDER))).clicked() {
                                    if let Some(c) = self.change(|a| a.add_feature(e, ty)) {
                                        self.focus = Some(c);
                                        self.expanded.insert(e);
                                    }
                                }
                            }
                        });
                        ui.add_space(6.0);
                    }
                }
                ui.painter().hline(ui.max_rect().x_range(), ui.cursor().top(), Stroke::new(1.0, BORDER));
                let has_doc = focus.map(|f| self.repo.kind_of(f, "ExtensibleModel") || self.repo.has(f, "documentation") || !crate::meta::known(self.repo.ty(f))).unwrap_or(false);
                if section_header(ui, "Documentation", has_doc && self.doc_open) && has_doc {
                    self.doc_open = !self.doc_open;
                }
                if let (Some(e), true, true) = (focus, has_doc, self.doc_open) {
                    let r = ui.add_sized(Vec2::new(ui.available_width(), 180.0), egui::TextEdit::multiline(&mut self.doc_buf).font(FontId::proportional(13.0)));
                    if r.lost_focus() && self.repo.str(e, "documentation") != self.doc_buf {
                        let text = self.doc_buf.clone();
                        self.change(|a| a.repo.set(e, "documentation", json!(text)));
                    }
                }
            });
        });
    }

    // ------------------------------------------------------------ sidebar
    fn working_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(HEADER_HEIGHT);
            ui.add_space(13.0);
            ui.label(egui::RichText::new("WORKING DIAGRAMS").color(TEXT).size(13.0));
        });
        ui.spacing_mut().item_spacing.y = 0.0;
        for d in self.open.clone() {
            if !self.repo.alive(d) {
                continue;
            }
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
            let current = self.diagram == Some(d);
            let p = ui.painter();
            if current {
                p.rect_filled(rect, 0.0, CURRENT_ROW);
            } else if resp.hovered() {
                p.rect_filled(rect, 0.0, Color32::from_rgb(0x33, 0x36, 0x3a));
            }
            icons::draw(p, Rect::from_min_size(Pos2::new(rect.left() + 28.0, rect.top() + 7.0), Vec2::splat(16.0)), icons::key_for(&self.repo, d));
            let name = self.repo.name(d);
            let nx = rect.left() + 46.0;
            p.text(Pos2::new(nx, rect.center().y), Align2::LEFT_CENTER, &name, FontId::proportional(13.0), if current { ACCENT_TEXT } else { TEXT });
            let nw = text_galley_width(ui, &name, 13.0);
            if let Some(owner) = self.repo.parent(d).map(|o| self.repo.name(o)).filter(|o| !o.is_empty()) {
                p.text(Pos2::new(nx + nw + 4.0, rect.center().y + 1.0), Align2::LEFT_CENTER, format!("— {owner}"), FontId::proportional(11.0), DIM);
            }
            let close = Rect::from_center_size(Pos2::new(rect.right() - 16.0, rect.center().y), Vec2::splat(16.0));
            if resp.hovered() {
                p.text(close.center(), Align2::CENTER_CENTER, "×", FontId::proportional(14.0), DIM);
            }
            // the menu acts on the current diagram; right-clicking a row does not make it current
            resp.context_menu(|ui| self.diagram_close_items(ui));
            if resp.clicked() {
                if resp.interact_pointer_pos().map(|q| close.contains(q)).unwrap_or(false) {
                    self.close_diagram(Some(d));
                } else {
                    self.open_diagram(d);
                    self.focus_element(d);
                }
            }
        }
    }

    fn toolbox_ui(&mut self, ui: &mut egui::Ui) {
        let top = ui.cursor().top();
        ui.painter().text(Pos2::new(ui.max_rect().left() + 13.0, top + 17.0), Align2::LEFT_CENTER, "TOOLBOX", FontId::proportional(13.0), TEXT);
        ui.add_space(26.0);
        // without a diagram the toolbox is empty (toolbox-view.js:307-310)
        if self.diagram.is_none() {
            return;
        }
        // a group opens by itself when a diagram of its type becomes current (toolbox-view.js:298-305)
        let dty = self.diagram.map(|d| self.repo.ty(d).to_string());
        if dty != self.toolbox_for {
            match dty.as_deref() {
                Some("UMLClassDiagram") => self.open_groups.insert("class"),
                Some("UMLUseCaseDiagram") => self.open_groups.insert("usecase"),
                _ => false,
            };
            self.toolbox_for = dty;
        }
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 26.0) / 2.0);
            let (r, resp) = ui.allocate_exact_size(Vec2::new(26.0, 24.0), Sense::click());
            if self.tool.is_none() {
                ui.painter().rect_filled(r, CornerRadius::same(2), TOOLBAR);
            }
            icons::draw(ui.painter(), Rect::from_center_size(r.center(), Vec2::splat(16.0)), "select");
            if resp.clicked() {
                self.tool = None;
            }
        });
        ui.add_space(5.0);
        let groups: &[Group] = match self.diagram.map(|d| self.repo.ty(d).to_string()).as_deref() {
            Some("UMLClassDiagram") => CLASS_GROUPS,
            Some("UMLUseCaseDiagram") => USECASE_GROUPS,
            Some(_) => OTHER_GROUPS,
            None => &[],
        };
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for g in groups {
                let open = self.open_groups.contains(g.id);
                let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
                let p = ui.painter();
                p.text(Pos2::new(rect.left() + 13.0, rect.center().y), Align2::LEFT_CENTER, g.label, FontId::proportional(13.0), DIM);
                p.hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, BORDER));
                let c = Pos2::new(rect.right() - 16.0, rect.center().y);
                let tri = if open {
                    vec![c + Vec2::new(-4.0, 3.0), c + Vec2::new(4.0, 3.0), c + Vec2::new(0.0, -3.0)]
                } else {
                    vec![c + Vec2::new(-4.0, -3.0), c + Vec2::new(4.0, -3.0), c + Vec2::new(0.0, 3.0)]
                };
                p.add(egui::Shape::convex_polygon(tri, TEXT, Stroke::NONE));
                if resp.clicked() {
                    if open {
                        self.open_groups.remove(g.id);
                    } else {
                        self.open_groups.insert(g.id);
                    }
                    // opening or closing a group goes back to Select (toolbox-view.js:446-449)
                    self.tool = None;
                    self.tool_sticky = false;
                }
                if !open {
                    continue;
                }
                for it in g.items {
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.5), Sense::click());
                    let p = ui.painter();
                    let selected = self.tool == Some(*it);
                    if selected {
                        p.rect_filled(rect, 0.0, ACCENT);
                    } else if resp.hovered() {
                        p.rect_filled(rect, 0.0, HOVER_ROW);
                    }
                    icons::draw(p, Rect::from_min_size(Pos2::new(rect.left() + 14.0, rect.top() + 8.0), Vec2::splat(16.0)), it.icon);
                    p.text(Pos2::new(rect.left() + 34.0, rect.center().y), Align2::LEFT_CENTER, it.label, FontId::proportional(13.0), if selected { Color32::WHITE } else { TEXT });
                    if selected && self.tool_sticky {
                        draw_lock(p, Pos2::new(rect.right() - 18.0, rect.center().y));
                    }
                    if resp.clicked() {
                        self.toolbox_click(*it);
                    }
                }
            }
        });
    }

    /// A click on a toolbox item: select it; on the active item, lock it; on a locked item, unlock it and
    /// go back to Select. A double-click therefore locks (toolbox-view.js:451-457).
    pub(crate) fn toolbox_click(&mut self, it: crate::app::Tool) {
        if self.tool != Some(it) {
            self.tool = Some(it);
            self.tool_sticky = false;
        } else if !self.tool_sticky {
            self.tool_sticky = true;
        } else {
            self.tool = None;
            self.tool_sticky = false;
        }
    }

    fn toolbar_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            for (key, tip) in [("zoomIn", "Zoom In"), ("zoomOut", "Zoom Out"), ("fit", "Fit To Window"), ("grid", "Show Grid"), ("image", "Export Diagram as PNG")] {
                let (r, resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                if resp.hovered() {
                    ui.painter().rect_filled(r, CornerRadius::same(2), Color32::from_rgb(0x6d, 0x70, 0x71));
                }
                icons::draw(ui.painter(), r.shrink(3.0), key);
                if resp.on_hover_text(tip).clicked() {
                    match key {
                        "zoomIn" => self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP),
                        "zoomOut" => self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP),
                        "fit" => self.fit(),
                        "grid" => self.show_grid = !self.show_grid,
                        _ => self.export_current(&ui.ctx().clone(), crate::export::Format::Png),
                    }
                }
            }
        });
    }

    fn status_ui(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let p = ui.painter();
        p.hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, BORDER));
        // right side: check mark and zoom
        let zr = Rect::from_min_max(Pos2::new(rect.right() - 57.0, rect.top()), rect.right_bottom());
        p.vline(zr.left(), rect.y_range(), Stroke::new(1.0, BORDER));
        p.text(zr.center(), Align2::CENTER_CENTER, format!("{}%", (self.zoom * 100.0).round()), FontId::proportional(13.0), TEXT);
        let vr = Rect::from_min_max(Pos2::new(zr.left() - 33.0, rect.top()), Pos2::new(zr.left(), rect.bottom()));
        p.vline(vr.left(), rect.y_range(), Stroke::new(1.0, BORDER));
        p.circle_filled(vr.center(), 7.5, OK_GREEN);
        let c = vr.center();
        p.add(egui::Shape::line(vec![c + Vec2::new(-3.5, 0.0), c + Vec2::new(-1.0, 2.5), c + Vec2::new(3.5, -2.5)], Stroke::new(2.0, LAYER0)));
        // left side: where the focused element sits
        let Some(f) = self.focus.filter(|f| self.repo.alive(*f)) else { return };
        let mut chain = self.repo.path(f);
        if chain.len() > 3 {
            chain = chain.split_off(chain.len() - 3);
        }
        let mut x = rect.left();
        for e in chain {
            let label = self.label_of(e);
            let tw = text_galley_width(ui, &label, 13.0);
            let p = ui.painter();
            icons::draw(p, Rect::from_min_size(Pos2::new(x + 8.0, rect.center().y - 8.0), Vec2::splat(16.0)), icons::key_for(&self.repo, e));
            p.text(Pos2::new(x + 28.0, rect.center().y), Align2::LEFT_CENTER, &label, FontId::proportional(13.0), TEXT);
            x += 36.0 + tw;
            p.vline(x, rect.y_range(), Stroke::new(1.0, BORDER));
        }
        ui.painter().text(Pos2::new(x + 8.0, rect.center().y), Align2::LEFT_CENTER, format!("[{}]", self.repo.ty(f)), FontId::proportional(11.0), DIM);
    }

    // ------------------------------------------------------------ menus and shortcuts
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let measure = EguiMeasure { ctx: ui.ctx().clone() };
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New                     Ctrl+N").clicked() {
                    self.cmd_new(&measure);
                    ui.close();
                }
                if ui.button("Open...                 Ctrl+O").clicked() {
                    self.cmd_open(&measure);
                    ui.close();
                }
                ui.separator();
                if ui.button("Save                    Ctrl+S").clicked() {
                    self.save(&measure);
                    ui.close();
                }
                if ui.button("Save As...      Ctrl+Shift+S").clicked() {
                    self.save_as(&measure);
                    ui.close();
                }
                ui.separator();
                if ui.button("Close").clicked() {
                    self.cmd_new(&measure);
                    ui.close();
                }
                ui.separator();
                ui.menu_button("Export Diagram As", |ui| {
                    use crate::export::Format;
                    for f in [Format::Png, Format::Jpeg, Format::Svg] {
                        if ui.button(format!("{}...", f.label())).clicked() {
                            self.export_current(&ui.ctx().clone(), f);
                            ui.close();
                        }
                    }
                    ui.separator();
                    for f in [Format::Png, Format::Jpeg, Format::Svg] {
                        if ui.button(format!("All to {}s...", f.label())).clicked() {
                            self.export_all_cmd(&ui.ctx().clone(), f);
                            ui.close();
                        }
                    }
                });
                ui.separator();
                if ui.button("Preferences...").clicked() {
                    self.prefs_open = Some("General".into());
                    ui.close();
                }
                ui.separator();
                if ui.button("Exit                    Ctrl+Q").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo                Ctrl+Z")).clicked() {
                    self.do_undo();
                    ui.close();
                }
                if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo                Ctrl+Y")).clicked() {
                    self.do_redo();
                    ui.close();
                }
                ui.separator();
                self.clipboard_items(ui);
                if ui.button("Delete              Delete").clicked() {
                    self.delete_selection(false);
                    ui.close();
                }
                if ui.button("Delete From Model   Ctrl+Delete").clicked() {
                    self.delete_selection(true);
                    ui.close();
                }
                ui.separator();
                if ui.button("Select All          Ctrl+A").clicked() {
                    self.select_all();
                    ui.close();
                }
                if ui.button("Select In Explorer  Ctrl+E").clicked() {
                    if let Some(f) = self.focus {
                        self.reveal(f);
                    }
                    ui.close();
                }
                if ui.add_enabled(self.focus.is_some(), egui::Button::new("Select In Diagram   Ctrl+D")).clicked() {
                    self.select_in_diagram();
                    ui.close();
                }
                if ui.add_enabled(self.focus.is_some(), egui::Button::new("Open Sub-Diagram    Ctrl+Shift+D")).clicked() {
                    self.open_sub_diagram();
                    ui.close();
                }
                ui.separator();
                self.move_items(ui);
            });
            ui.menu_button("Format", |ui| {
                if ui.add_enabled(!self.selection.is_empty(), egui::Button::new("Font...                         Ctrl+Shift+F")).clicked() {
                    self.open_font_dialog();
                    ui.close();
                }
                self.color_menus(ui);
                self.line_style_menu(ui);
                self.stereotype_display_menu(ui);
                self.format_toggles(ui);
                ui.separator();
                self.alignment_menu(ui);
            });
            ui.menu_button("Model", |ui| {
                if ui.button("Add Class Diagram").clicked() {
                    let f = self.focus;
                    self.change(|a| a.add_diagram(f, "UMLClassDiagram"));
                    ui.close();
                }
                if ui.button("Add Use Case Diagram").clicked() {
                    let f = self.focus;
                    self.change(|a| a.add_diagram(f, "UMLUseCaseDiagram"));
                    ui.close();
                }
                ui.separator();
                for (label, ty) in [("Add Attribute", "UMLAttribute"), ("Add Operation", "UMLOperation"), ("Add Literal", "UMLEnumerationLiteral")] {
                    let ok = self.focus.map(|f| self.can_add_feature(f, ty)).unwrap_or(false);
                    if ui.add_enabled(ok, egui::Button::new(label)).clicked() {
                        let f = self.focus.unwrap();
                        if let Some(c) = self.change(|a| a.add_feature(f, ty)) {
                            self.focus = Some(c);
                        }
                        ui.close();
                    }
                }
            });
            ui.menu_button("View", |ui| {
                self.diagram_close_items(ui);
                ui.separator();
                let many = self.open.len() > 1;
                if ui.add_enabled(many, egui::Button::new("Next Diagram        Ctrl+Shift+]")).clicked() {
                    self.cycle_diagram(1);
                    ui.close();
                }
                if ui.add_enabled(many, egui::Button::new("Previous Diagram    Ctrl+Shift+[")).clicked() {
                    self.cycle_diagram(-1);
                    ui.close();
                }
                ui.separator();
                if ui.button("Zoom In             Ctrl+=").clicked() {
                    self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP);
                }
                if ui.button("Zoom Out            Ctrl+-").clicked() {
                    self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP);
                }
                if ui.button("Actual Size         Ctrl+0").clicked() {
                    self.zoom_keep_centre(1.0);
                }
                if ui.button("Fit To Window       Ctrl+9").clicked() {
                    self.fit();
                    ui.close();
                }
                ui.separator();
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.checkbox(&mut self.snap, "Snap to Grid");
                ui.separator();
                for (i, label) in PANELS.iter().enumerate() {
                    let mut on = *self.panel_flag(i);
                    if ui.checkbox(&mut on, format!("{label:<14}Ctrl+{}", i + 1)).clicked() {
                        *self.panel_flag(i) = on;
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About UMLCraft").clicked() {
                    self.about_open = true;
                    ui.close();
                }
            });
        });
    }

    /// The Style editor: colour swatches, line style, the show and suppress toggles, z-order and alignment.
    fn style_ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 4.0;
        // font face and size: merged, blank when the views differ (style-editor-view.js:180-227)
        let (face, size) = self.font_state();
        ui.horizontal(|ui| {
            let mut f = face.clone().unwrap_or_default();
            let r = ui.add(egui::TextEdit::singleline(&mut f).hint_text("—").desired_width(110.0).font(FontId::proportional(12.0)));
            if r.lost_focus() && !f.trim().is_empty() && Some(f.trim().to_string()) != face {
                self.set_font(Some(f.trim()), None);
            }
            let label = size.map(|s| format!("{s}")).unwrap_or_else(|| "—".into());
            egui::ComboBox::from_id_salt("style-size").selected_text(label).width(60.0).show_ui(ui, |ui| {
                for s in crate::style::SIZES {
                    if ui.selectable_label(size == Some(s), format!("{s}")).clicked() {
                        self.set_font(None, Some(s));
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            for (field, label) in crate::style::COLORS {
                if self.color_views(field).is_empty() {
                    continue;
                }
                self.color_swatch(ui, field, label);
                ui.label(egui::RichText::new(label.trim_end_matches(" Color")).size(12.0).color(DIM));
                ui.add_space(6.0);
            }
        });
        if !self.flag_views("stereotypeDisplay").is_empty() {
            // the stereotype display dropdown, empty when mixed (style-editor-view.js)
            let cur = self.stereotype_display_state();
            let shown = crate::style::STEREOTYPE_DISPLAYS.iter().find(|d| Some(d.0) == cur.as_deref()).map(|d| d.1).unwrap_or("");
            egui::ComboBox::from_id_salt("style-stereotype").selected_text(egui::RichText::new(shown).size(12.0)).width(150.0).show_ui(ui, |ui| {
                for (value, label) in crate::style::STEREOTYPE_DISPLAYS {
                    if ui.selectable_label(cur.as_deref() == Some(value), label).clicked() {
                        self.set_stereotype_display(value);
                    }
                }
            });
        }
        if self.selection.iter().any(|v| self.repo.alive(*v) && views::is_edge(&self.repo, *v)) {
            ui.horizontal(|ui| {
                let cur = self.line_style_state();
                for (style, label) in [(0, "Rectilinear"), (1, "Oblique"), (2, "Rounded"), (3, "Curve")] {
                    if ui.radio(cur == Some(style), egui::RichText::new(label).size(12.0)).clicked() {
                        self.set_line_style(style);
                    }
                }
            });
        }
        for (field, label) in FORMAT_TOGGLES {
            if self.flag_views(field).is_empty() {
                continue;
            }
            let mut on = self.flag_state(field) == Some(true);
            let short = label.split("  ").next().unwrap_or(label).trim();
            if ui.checkbox(&mut on, egui::RichText::new(short).size(12.0)).clicked() {
                self.toggle_flag(field);
            }
        }
        ui.horizontal_wrapped(|ui| {
            let small = |t: &str| egui::Button::new(egui::RichText::new(t).size(11.0)).fill(HOVER_ROW).stroke(Stroke::new(1.0, INPUT_BORDER));
            if ui.add(small("Back")).on_hover_text("Send To Back").clicked() {
                self.reorder(false);
            }
            if ui.add(small("Front")).on_hover_text("Bring To Front").clicked() {
                self.reorder(true);
            }
            for (how, label, short) in [
                (crate::align::Align::Left, "Align Left", "L"),
                (crate::align::Align::Center, "Align Center", "C"),
                (crate::align::Align::Right, "Align Right", "R"),
                (crate::align::Align::Top, "Align Top", "T"),
                (crate::align::Align::Middle, "Align Middle", "M"),
                (crate::align::Align::Bottom, "Align Bottom", "B"),
                (crate::align::Align::SpaceH, "Space Equally, Horizontally", "↔"),
                (crate::align::Align::SpaceV, "Space Equally, Vertically", "↕"),
                (crate::align::Align::Width, "Set Width Equally", "W"),
                (crate::align::Align::Height, "Set Height Equally", "H"),
                (crate::align::Align::Size, "Set Size Equally", "S"),
            ] {
                if ui.add(small(short)).on_hover_text(label).clicked() {
                    self.align(how);
                }
            }
        });
    }

    /// A colour swatch that opens a colour picker; a mixed selection shows a transparent swatch.
    fn color_swatch(&mut self, ui: &mut egui::Ui, field: &str, label: &str) {
        let cur = self.color_state(field);
        let mut c = cur.map(|c| Color32::from_rgb(c.0, c.1, c.2)).unwrap_or(Color32::TRANSPARENT);
        let r = egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).on_hover_text(label);
        if r.changed() {
            self.set_color(field, crate::render::Rgba(c.r(), c.g(), c.b(), 255));
        }
    }

    /// Format > Fill Color and Line Color: a colour picker on the selected views (default-commands.js:966-991).
    pub(crate) fn color_menus(&mut self, ui: &mut egui::Ui) {
        for (field, label) in crate::style::COLORS.into_iter().take(2) {
            ui.add_enabled_ui(!self.color_views(field).is_empty(), |ui| {
                ui.menu_button(label, |ui| {
                    let cur = self.color_state(field).unwrap_or(crate::render::Rgba(255, 255, 255, 255));
                    let mut c = Color32::from_rgb(cur.0, cur.1, cur.2);
                    if egui::color_picker::color_picker_color32(ui, &mut c, egui::color_picker::Alpha::Opaque) {
                        self.set_color(field, crate::render::Rgba(c.r(), c.g(), c.b(), 255));
                    }
                });
            });
        }
    }

    /// Format > Line Style, with a check on the style the selected lines share (default-commands.js:993-1011, 1337-1342).
    pub(crate) fn line_style_menu(&mut self, ui: &mut egui::Ui) {
        let has = self.selection.iter().any(|v| self.repo.alive(*v) && views::is_edge(&self.repo, *v));
        ui.add_enabled_ui(has, |ui| {
            ui.menu_button("Line Style", |ui| {
                let cur = self.line_style_state();
                for (style, label) in [(0, "Rectilinear              Ctrl+L"), (1, "Oblique                    Ctrl+B"), (2, "Rounded Rectilinear  Ctrl+Alt+L"), (3, "Curve                        Ctrl+Alt+B")] {
                    let mut on = cur == Some(style);
                    if ui.checkbox(&mut on, label).clicked() {
                        self.set_line_style(style);
                        ui.close();
                    }
                }
            });
        });
    }

    /// Move Up and Move Down (Edit menu and explorer menu).
    pub(crate) fn move_items(&mut self, ui: &mut egui::Ui) {
        let e = self.focus.filter(|f| self.repo.alive(*f) && self.repo.ty(*f) != "Project" && !self.is_view(*f));
        if ui.add_enabled(e.is_some(), egui::Button::new("Move Up             Ctrl+Shift+Up")).clicked() {
            self.move_element(e.unwrap(), true);
            ui.close();
        }
        if ui.add_enabled(e.is_some(), egui::Button::new("Move Down           Ctrl+Shift+Down")).clicked() {
            self.move_element(e.unwrap(), false);
            ui.close();
        }
    }

    /// Cut, Copy and Paste (Edit menu, canvas menu and explorer menu).
    pub(crate) fn clipboard_items(&mut self, ui: &mut egui::Ui) {
        let can = self.can_copy();
        if ui.add_enabled(can, egui::Button::new("Cut                 Ctrl+X")).clicked() {
            self.cmd_cut(Some(&ui.ctx().clone()));
            ui.close();
        }
        if ui.add_enabled(can, egui::Button::new("Copy                Ctrl+C")).clicked() {
            self.cmd_copy(Some(&ui.ctx().clone()));
            ui.close();
        }
        if ui.add_enabled(self.diagram.is_some(), egui::Button::new("Copy Diagram As Image  Ctrl+Shift+C")).clicked() {
            self.cmd_copy_diagram_image(&ui.ctx().clone());
            ui.close();
        }
        if ui.button("Paste               Ctrl+V").clicked() {
            self.cmd_paste();
            ui.close();
        }
    }

    /// Close Diagram, Close Other Diagrams and Close All Diagrams (View menu and Working Diagrams menu).
    fn diagram_close_items(&mut self, ui: &mut egui::Ui) {
        let has = self.diagram.is_some();
        if ui.add_enabled(has, egui::Button::new("Close Diagram       F4")).clicked() {
            self.close_diagram(None);
            ui.close();
        }
        if ui.add_enabled(has, egui::Button::new("Close Other Diagrams  Ctrl+F4")).clicked() {
            self.close_other_diagrams();
            ui.close();
        }
        if ui.add_enabled(!self.open.is_empty(), egui::Button::new("Close All Diagrams  Shift+F4")).clicked() {
            self.close_all_diagrams();
            ui.close();
        }
    }

    /// The Format menu's on/off items, checked when every selected view that has the flag is on,
    /// and enabled only when a selected view has it (uml.json order; default-commands.js:1319-1350).
    /// Format > Stereotype Display, checked on the value the selected views share.
    pub(crate) fn stereotype_display_menu(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(!self.flag_views("stereotypeDisplay").is_empty(), |ui| {
            ui.menu_button("Stereotype Display", |ui| {
                let cur = self.stereotype_display_state();
                for (i, (value, label)) in crate::style::STEREOTYPE_DISPLAYS.into_iter().enumerate() {
                    let mut on = cur.as_deref() == Some(value);
                    if ui.checkbox(&mut on, format!("{label:<24}Ctrl+Shift+{i}")).clicked() {
                        self.set_stereotype_display(value);
                        ui.close();
                    }
                }
            });
        });
    }

    pub(crate) fn format_toggles(&mut self, ui: &mut egui::Ui) {
        for (i, (field, label)) in FORMAT_TOGGLES.into_iter().enumerate() {
            if i == 2 || i == 6 {
                ui.separator();
            }
            let has = !self.flag_views(field).is_empty();
            let mut on = self.flag_state(field) == Some(true);
            if ui.add_enabled(has, egui::Checkbox::new(&mut on, label)).clicked() {
                self.toggle_flag(field);
                ui.close();
            }
        }
    }

    /// Format > Alignment, in the main menu and the canvas menu (alignment.json order).
    pub(crate) fn alignment_menu(&mut self, ui: &mut egui::Ui) {
        ui.add_enabled_ui(self.can_align(), |ui| {
            ui.menu_button("Alignment", |ui| {
                if ui.button("Send To Back").clicked() {
                    self.reorder(false);
                    ui.close();
                }
                if ui.button("Bring To Front").clicked() {
                    self.reorder(true);
                    ui.close();
                }
                for (i, (how, label)) in crate::align::Align::ALL.into_iter().enumerate() {
                    if i == 0 || i == 6 || i == 8 {
                        ui.separator();
                    }
                    if ui.button(label).clicked() {
                        self.align(how);
                        ui.close();
                    }
                }
            });
        });
    }

    pub(crate) fn shortcuts(&mut self, ctx: &egui::Context) {
        use egui::{Key, KeyboardShortcut, Modifiers};
        let typing = self.editing.is_some() || text_field_focused(ctx);
        let measure = EguiMeasure { ctx: ctx.clone() };
        let sc = |m: Modifiers, k: Key| KeyboardShortcut::new(m, k);
        let cs = Modifiers::COMMAND | Modifiers::SHIFT;
        let hit = |s: KeyboardShortcut| ctx.input_mut(|i| i.consume_shortcut(&s));
        // Ctrl+Shift toggles first: a Ctrl shortcut also matches when Shift is held (Ctrl+N, Ctrl+A…)
        if !typing {
            if hit(sc(cs, Key::F)) {
                self.open_font_dialog();
            }
            // Ctrl+Shift+0…5 before Ctrl+0 (Actual Size), which also matches with Shift held
            for (i, key) in [Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5].into_iter().enumerate() {
                if hit(sc(cs, key)) {
                    self.set_stereotype_display(crate::style::STEREOTYPE_DISPLAYS[i].0);
                }
            }
            for (key, field) in [(Key::I, "fillColor"), (Key::L, "lineColor")] {
                if hit(sc(cs, key)) && !self.color_views(field).is_empty() {
                    self.color_dialog = Some(field);
                }
            }
            for (key, field) in [(Key::R, "autoResize"), (Key::V, "showVisibility"), (Key::N, "showNamespace"), (Key::M, "showMultiplicity"), (Key::J, "showOperationSignature"), (Key::A, "suppressAttributes"), (Key::O, "suppressOperations"), (Key::T, "suppressLiterals")] {
                if hit(sc(cs, key)) {
                    self.toggle_flag(field);
                }
            }
            // Ctrl+Shift+V reaches egui as a paste (when the clipboard holds text), not as a key
            let shift_paste = ctx.input_mut(|i| {
                let n = i.events.len();
                if i.modifiers.command && i.modifiers.shift {
                    i.events.retain(|e| !matches!(e, egui::Event::Paste(_)));
                }
                n != i.events.len()
            });
            if shift_paste {
                self.toggle_flag("showVisibility");
            }
        }
        if hit(sc(Modifiers::COMMAND, Key::N)) {
            self.cmd_new(&measure);
        }
        if hit(sc(Modifiers::COMMAND, Key::O)) {
            self.cmd_open(&measure);
        }
        if hit(sc(cs, Key::S)) {
            self.save_as(&measure);
        }
        if hit(sc(Modifiers::COMMAND, Key::S)) {
            self.save(&measure);
        }
        if hit(sc(Modifiers::COMMAND, Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if typing {
            return;
        }
        // Esc anywhere goes back to the Select tool and clears the lock (toolbox-view.js:473-479)
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.drag.is_none() {
            self.tool = None;
            self.tool_sticky = false;
        }
        if hit(sc(Modifiers::COMMAND, Key::Y)) || hit(sc(cs, Key::Z)) {
            self.do_redo();
        }
        if hit(sc(Modifiers::COMMAND, Key::Z)) {
            self.do_undo();
        }
        // Ctrl+C, Ctrl+X and Ctrl+V reach egui as Copy, Cut and Paste events, not as keys
        let clip = ctx.input_mut(|i| {
            let mut out = None;
            i.events.retain(|e| match e {
                egui::Event::Copy | egui::Event::Cut | egui::Event::Paste(_) if out.is_none() => {
                    out = Some(e.clone());
                    false
                }
                _ => true,
            });
            out
        });
        let shift = ctx.input(|i| i.modifiers.shift);
        match clip {
            // Ctrl+Shift+C also arrives as a Copy event
            Some(egui::Event::Copy) if shift => self.cmd_copy_diagram_image(ctx),
            Some(egui::Event::Copy) => self.cmd_copy(Some(ctx)),
            Some(egui::Event::Cut) => self.cmd_cut(Some(ctx)),
            Some(egui::Event::Paste(t)) => self.paste_text(&t),
            _ => {}
        }
        if hit(sc(Modifiers::COMMAND, Key::Delete)) {
            self.delete_selection(true);
        }
        if hit(sc(Modifiers::COMMAND, Key::A)) {
            self.select_all();
        }
        if hit(sc(Modifiers::COMMAND, Key::E)) {
            if let Some(f) = self.focus {
                self.reveal(f);
            }
        }
        // Ctrl+Shift+D first: Ctrl+D also matches when Shift is held
        if hit(sc(cs, Key::D)) {
            self.open_sub_diagram();
        }
        if hit(sc(Modifiers::COMMAND, Key::D)) {
            self.select_in_diagram();
        }
        for (key, up) in [(Key::ArrowUp, true), (Key::ArrowDown, false)] {
            if hit(sc(cs, key)) {
                if let Some(f) = self.focus.filter(|f| self.repo.alive(*f) && self.repo.ty(*f) != "Project") {
                    self.move_element(f, up);
                }
            }
        }
        // Ctrl+Alt first: Ctrl+L and Ctrl+B also match when Alt is held
        let ca = Modifiers::COMMAND | Modifiers::ALT;
        if hit(sc(ca, Key::L)) {
            self.set_line_style(2);
        }
        if hit(sc(ca, Key::B)) {
            self.set_line_style(3);
        }
        if hit(sc(Modifiers::COMMAND, Key::L)) {
            self.set_line_style(0);
        }
        if hit(sc(Modifiers::COMMAND, Key::B)) {
            self.set_line_style(1);
        }
        if hit(sc(cs, Key::H)) {
            self.toggle_shadow();
        }
        if hit(sc(Modifiers::COMMAND, Key::Equals)) || hit(sc(Modifiers::COMMAND, Key::Plus)) {
            self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP);
        }
        if hit(sc(Modifiers::COMMAND, Key::Minus)) {
            self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP);
        }
        // View > Sidebar, Navigator, Toolbar, Statusbar, Toolbox, Editors (keymaps/linux.json)
        for (i, key) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6].into_iter().enumerate() {
            if hit(sc(Modifiers::COMMAND, key)) {
                let f = self.panel_flag(i);
                *f = !*f;
            }
        }
        if hit(sc(Modifiers::COMMAND, Key::Num0)) {
            self.zoom_keep_centre(1.0);
        }
        if hit(sc(Modifiers::COMMAND, Key::Num9)) {
            self.fit();
        }
        // Ctrl+Shift+G first: Ctrl+G also matches when Shift is held
        if hit(sc(cs, Key::G)) {
            self.snap = !self.snap;
        }
        if hit(sc(Modifiers::COMMAND, Key::G)) {
            self.show_grid = !self.show_grid;
        }
        // Shift+F4 first: a plain F4 shortcut also matches when Shift is held
        if hit(sc(Modifiers::SHIFT, Key::F4)) {
            self.close_all_diagrams();
        }
        if hit(sc(Modifiers::COMMAND, Key::F4)) {
            self.close_other_diagrams();
        }
        if hit(sc(Modifiers::NONE, Key::F4)) {
            self.close_diagram(None);
        }
        // Ctrl+Shift+] and [: with Shift held most layouts report } and {
        if hit(sc(cs, Key::CloseBracket)) || hit(sc(cs, Key::CloseCurlyBracket)) {
            self.cycle_diagram(1);
        }
        if hit(sc(cs, Key::OpenBracket)) || hit(sc(cs, Key::OpenCurlyBracket)) {
            self.cycle_diagram(-1);
        }
    }

    fn cmd_new(&mut self, ms: &EguiMeasure) {
        if self.confirm_discard(ms) {
            self.save_working_diagrams();
            self.set_project(crate::model::Repo::blank(), None);
        }
    }

    fn cmd_open(&mut self, ms: &EguiMeasure) {
        if !self.confirm_discard(ms) {
            return;
        }
        if let Some(p) = rfd::FileDialog::new().set_title("Open Model").add_filter("Models", &["mdj"]).add_filter("All Files", &["*"]).pick_file() {
            self.open_path(p);
        }
    }

    /// Export the current diagram as PNG, JPEG or SVG. The selection is cleared first; with no diagram
    /// open the toast "No diagram to export." appears (default-commands.js:360-445).
    pub(crate) fn export_current(&mut self, ctx: &egui::Context, f: crate::export::Format) {
        let Some(d) = self.diagram else {
            self.toast("No diagram to export.");
            return;
        };
        self.select(vec![]);
        let ms = EguiMeasure { ctx: ctx.clone() };
        let title = format!("Export Diagram as {}", f.label());
        let Some(p) = rfd::FileDialog::new().set_title(&title).set_file_name(self.export_name(d, f)).add_filter(f.label(), &[f.ext()]).save_file() else {
            return;
        };
        let result = crate::export::render(self, d, &ms, f, ctx.pixels_per_point()).and_then(|bytes| std::fs::write(&p, bytes).map_err(|e| e.to_string()));
        if let Err(e) = result {
            self.message = Some(("Failed to export the diagram.".into(), e));
        }
    }

    /// Export every diagram into a folder the person picks (default-commands.js:452-535).
    pub(crate) fn export_all_cmd(&mut self, ctx: &egui::Context, f: crate::export::Format) {
        if self.repo.all_of("Diagram").is_empty() {
            self.toast("No diagram to export.");
            return;
        }
        let Some(folder) = rfd::FileDialog::new().set_title("Select a folder where all diagrams to be exported").pick_folder() else { return };
        self.select(vec![]);
        let ms = EguiMeasure { ctx: ctx.clone() };
        match self.export_all(&folder, &ms, f, ctx.pixels_per_point()) {
            Ok(n) => self.toast(&format!("{n} diagrams exported.")),
            Err(e) => self.message = Some(("Failed to export the diagrams.".into(), e)),
        }
    }

    pub(crate) fn cmd_copy_diagram_image(&mut self, ctx: &egui::Context) {
        let Some(d) = self.diagram else { return };
        let ms = EguiMeasure { ctx: ctx.clone() };
        let info = views::layout_diagram(&mut self.repo, &ms, d);
        let png = render::to_png(&self.repo, d, &info, &ms, 1.0);
        self.copy_diagram_image(png);
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        if self.about_open {
            egui::Window::new("About UMLCraft").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
                ui.label(concat!("UMLCraft ", env!("CARGO_PKG_VERSION")));
                ui.label("A UML modeler for class and use case diagrams that opens and saves .mdj model files.");
                ui.label("Written in Rust with egui. The source code is in the app folder of this project; its README lists every file.");
                if ui.button("OK").clicked() {
                    self.about_open = false;
                }
            });
        }
        self.diagram_picker_ui(ctx);
        self.color_dialog_ui(ctx);
        self.preferences_ui(ctx);
        self.font_dialog_ui(ctx);
        if let Some((title, detail)) = self.message.clone() {
            egui::Window::new(title).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
                ui.label(detail);
                if ui.button("OK").clicked() {
                    self.message = None;
                }
            });
        }
    }

    /// The Element List Picker as Select In Diagram uses it: "Select Diagram to show", one row per
    /// diagram with its icon, name and (path); "Do not specify" until a row is picked; double-click is OK
    /// (element-list-picker-dialog.js:73-131).
    fn diagram_picker_ui(&mut self, ctx: &egui::Context) {
        let Some((views, mut chosen)) = self.diagram_picker.clone() else { return };
        let mut close = false;
        let mut ok = false;
        egui::Window::new("Select Diagram to show").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_min_width(320.0);
            for (i, v) in views.iter().enumerate() {
                let Some(d) = self.repo.parent(*v).filter(|d| self.repo.alive(*d)) else { continue };
                let mut owners = self.repo.path(d);
                owners.pop();
                let path: Vec<String> = owners.into_iter().map(|p| self.repo.name(p)).filter(|n| !n.is_empty()).collect();
                let text = format!("{}  ({})", self.repo.name(d), path.join("::"));
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                    icons::draw(ui.painter(), r, icons::key_for(&self.repo, d));
                    let resp = ui.selectable_label(chosen == Some(i), text);
                    if resp.clicked() {
                        chosen = Some(i);
                    }
                    if resp.double_clicked() {
                        chosen = Some(i);
                        ok = true;
                    }
                });
            }
            let mut none = chosen.is_none();
            if ui.checkbox(&mut none, "Do not specify").changed() && none {
                chosen = None;
            }
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    ok = true;
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                }
            });
        });
        if ok {
            self.diagram_picker = None;
            if let Some(v) = chosen.and_then(|i| views.get(i)).copied().filter(|v| self.repo.alive(*v)) {
                self.show_view(v);
            }
        } else if close {
            self.diagram_picker = None;
        } else {
            self.diagram_picker = Some((views, chosen));
        }
    }

    pub(crate) fn open_font_dialog(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        let (face, size) = self.font_state();
        let size = size.map(|s| if s.fract() == 0.0 { format!("{}", s as i64) } else { s.to_string() });
        self.font_dialog = Some((face.unwrap_or_default(), size.unwrap_or_default(), self.color_state("fontColor")));
    }

    /// Format > Font (Ctrl+Shift+F): face, size and colour, pre-filled with the merged values (blank where the
    /// selected views differ). OK applies what was filled in; a size that is not a number is ignored
    /// (default-commands.js:940-964).
    fn font_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some((mut face, mut size, mut color)) = self.font_dialog.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        egui::Window::new("Font").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            egui::Grid::new("font-grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label("Face");
                ui.add(egui::TextEdit::singleline(&mut face).hint_text("—").desired_width(160.0));
                ui.end_row();
                ui.label("Size");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut size).hint_text("—").desired_width(60.0));
                    egui::ComboBox::from_id_salt("font-sizes").selected_text("").width(20.0).show_ui(ui, |ui| {
                        for s in crate::style::SIZES {
                            if ui.selectable_label(false, format!("{s}")).clicked() {
                                size = format!("{s}");
                            }
                        }
                    });
                });
                ui.end_row();
                ui.label("Color");
                let mut c = color.map(|c| Color32::from_rgb(c.0, c.1, c.2)).unwrap_or(Color32::TRANSPARENT);
                if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                    color = Some(crate::render::Rgba(c.r(), c.g(), c.b(), 255));
                }
                ui.end_row();
            });
            ui.horizontal(|ui| {
                ok = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
            });
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                cancel = true;
            }
        });
        if ok {
            let f = face.trim();
            let sz = size.trim().parse::<f64>().ok().filter(|s| *s > 0.0);
            self.set_font(if f.is_empty() { None } else { Some(f) }, sz);
            if let Some(c) = color {
                if Some(c) != self.color_state("fontColor") {
                    self.style_edit = None;
                    self.set_color("fontColor", c);
                }
            }
            self.font_dialog = None;
        } else if cancel {
            self.font_dialog = None;
        } else {
            self.font_dialog = Some((face, size, color));
        }
    }

    /// The colour window of Format > Fill Color (Ctrl+Shift+I) and Line Color (Ctrl+Shift+L): the merged
    /// colour of the selection, applied as it changes (default-commands.js:966-991).
    fn color_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(field) = self.color_dialog else { return };
        let label = crate::style::COLORS.iter().find(|c| c.0 == field).map(|c| c.1).unwrap_or("Color");
        let mut close = false;
        egui::Window::new(label).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            let cur = self.color_state(field).unwrap_or(crate::render::Rgba(255, 255, 255, 255));
            let mut c = Color32::from_rgb(cur.0, cur.1, cur.2);
            if egui::color_picker::color_picker_color32(ui, &mut c, egui::color_picker::Alpha::Opaque) {
                self.set_color(field, crate::render::Rgba(c.r(), c.g(), c.b(), 255));
            }
            if ui.button("OK").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape)) {
                close = true;
            }
        });
        if close || self.color_views(field).is_empty() {
            self.color_dialog = None;
        }
    }

    /// Toasts stack upward inside the diagram area and hide after 5 s (toast-manager.js:36-72).
    fn toasts_ui(&mut self, ctx: &egui::Context) {
        self.toasts.retain(|(_, t)| t.elapsed().as_secs_f32() < 5.0);
        let Some(area) = self.canvas_rect else { return };
        let mut y = area.bottom() - 16.0;
        for (i, (text, _)) in self.toasts.iter().enumerate().rev() {
            let pos = Pos2::new(area.left() + 16.0, y);
            let r = egui::Area::new(egui::Id::new(("toast", i))).order(egui::Order::Foreground).pivot(Align2::LEFT_BOTTOM).fixed_pos(pos).show(ctx, |ui| {
                Frame::NONE.fill(LAYER1).stroke(Stroke::new(1.0, BORDER)).corner_radius(CornerRadius::same(3)).inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
                    ui.label(egui::RichText::new(text).color(TEXT).size(13.0));
                });
            });
            y -= r.response.rect.height() + 6.0;
        }
        if !self.toasts.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }

    fn screenshots(&mut self, ctx: &egui::Context) {
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            if let Some(path) = self.screenshot.clone() {
                save_png(&path, &img);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.screenshot.is_some() {
            self.frames += 1;
            ctx.request_repaint();
            if !self.shot_requested && self.frames >= SCREENSHOT_FRAMES && self.started.elapsed().as_secs_f32() >= 2.0 {
                self.shot_requested = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
            }
        }
    }

    fn handle_close(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_working_diagrams();
        }
        if ctx.input(|i| i.viewport().close_requested()) && self.modified && self.screenshot.is_none() && !self.quit_after_save {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            let ms = EguiMeasure { ctx: ctx.clone() };
            if self.confirm_discard(&ms) {
                self.quit_after_save = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

/// View menu panels, in the order of their Ctrl+1…6 shortcuts.
const PANELS: [&str; 6] = ["Sidebar", "Navigator", "Toolbar", "Statusbar", "Toolbox", "Editors"];

impl App {
    fn panel_flag(&mut self, i: usize) -> &mut bool {
        match i {
            0 => &mut self.show_sidebar,
            1 => &mut self.show_navigator,
            2 => &mut self.show_toolbar,
            3 => &mut self.show_statusbar,
            4 => &mut self.show_toolbox,
            _ => &mut self.show_editors,
        }
    }
}

/// Format menu toggles, in the original's order, with their shortcuts (keymaps/uml.json, linux.json).
const FORMAT_TOGGLES: [(&str, &str); 9] = [
    ("autoResize", "Auto Resize                       Ctrl+Shift+R"),
    ("showShadow", "Show Shadow                     Ctrl+Shift+H"),
    ("showVisibility", "Show Visibility                 Ctrl+Shift+V"),
    ("showNamespace", "Show Namespace                  Ctrl+Shift+N"),
    ("showMultiplicity", "Show Multiplicity               Ctrl+Shift+M"),
    ("showOperationSignature", "Show Operation Signature    Ctrl+Shift+J"),
    ("suppressAttributes", "Suppress Attributes             Ctrl+Shift+A"),
    ("suppressOperations", "Suppress Operations            Ctrl+Shift+O"),
    ("suppressLiterals", "Suppress Literals                 Ctrl+Shift+T"),
];

/// The small padlock shown on a locked toolbox item.
fn draw_lock(p: &egui::Painter, c: Pos2) {
    let body = Rect::from_center_size(c + Vec2::new(0.0, 2.0), Vec2::new(9.0, 7.0));
    p.rect_filled(body, CornerRadius::same(1), Color32::WHITE);
    let pts: Vec<Pos2> = (0..=8).map(|i| {
        let a = std::f32::consts::PI * i as f32 / 8.0;
        c + Vec2::new(-3.0 * a.cos(), -1.5 - 3.5 * a.sin())
    }).collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.5, Color32::WHITE)));
}

/// True while a text field has the keyboard (the diagram canvas can hold focus too, and then the
/// editing shortcuts such as Ctrl+Z must still work).
fn text_field_focused(ctx: &egui::Context) -> bool {
    ctx.memory(|m| m.focused()).map(|id| egui::TextEdit::load_state(ctx, id).is_some()).unwrap_or(false)
}

fn save_png(path: &std::path::Path, img: &egui::ColorImage) {
    let [w, h] = img.size;
    let mut buf = Vec::with_capacity(w * h * 4);
    for c in &img.pixels {
        buf.extend_from_slice(&[c.r(), c.g(), c.b(), 255]);
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = image::save_buffer(path, &buf, w as u32, h as u32, image::ColorType::Rgba8) {
        eprintln!("could not save {}: {e}", path.display());
    }
}

/// A section header with a caret ("Properties", "Documentation"); returns true when clicked.
fn section_header(ui: &mut egui::Ui, title: &str, open: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 30.0), Sense::click());
    let p = ui.painter();
    let c = Pos2::new(rect.left() + 4.0, rect.center().y);
    let tri = if open {
        vec![c + Vec2::new(-2.0, -2.0), c + Vec2::new(6.0, -2.0), c + Vec2::new(2.0, 3.0)]
    } else {
        vec![c + Vec2::new(0.0, -4.0), c + Vec2::new(5.0, 0.0), c + Vec2::new(0.0, 4.0)]
    };
    p.add(egui::Shape::convex_polygon(tri, TEXT, Stroke::NONE));
    p.text(Pos2::new(rect.left() + 15.0, rect.center().y), Align2::LEFT_CENTER, title, FontId::proportional(13.0), TEXT);
    resp.clicked()
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_close(&ctx);
        self.shortcuts(&ctx);
        let title = self.title();
        if ctx.input(|i| i.viewport().title.as_deref() != Some(title.as_str())) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        }

        Panel::top("menu").exact_size(22.0).show_separator_line(false).frame(Frame::NONE.fill(LAYER0).inner_margin(Margin::symmetric(6, 0))).show(ui, |ui| {
            self.menu_bar(ui);
        });
        if self.show_statusbar {
            Panel::bottom("status").exact_size(STATUSBAR_HEIGHT).show_separator_line(false).frame(Frame::NONE.fill(LAYER1)).show(ui, |ui| self.status_ui(ui));
        }
        if self.show_sidebar {
            Panel::left("sidebar").exact_size(SIDEBAR_WIDTH).show_separator_line(false).frame(Frame::NONE.fill(LAYER0)).show(ui, |ui| {
                if self.show_toolbox {
                    Panel::bottom("toolbox").exact_size(TOOLBOX_HEIGHT).show_separator_line(false).frame(Frame::NONE.fill(LAYER1)).show(ui, |ui| self.toolbox_ui(ui));
                }
                self.working_ui(ui);
            });
        }
        if self.show_navigator {
            Panel::right("navigator").exact_size(NAVIGATOR_WIDTH).show_separator_line(false).frame(Frame::NONE.fill(LAYER0)).show(ui, |ui| {
                if self.show_editors {
                    Panel::bottom("editors").exact_size(EDITORS_HEIGHT).show_separator_line(false).frame(Frame::NONE.fill(LAYER1)).show(ui, |ui| self.editors_ui(ui));
                }
                self.explorer_ui(ui);
            });
        }
        if self.show_toolbar {
            Panel::right("toolbar").exact_size(TOOLBAR_WIDTH).show_separator_line(false).frame(Frame::NONE.fill(TOOLBAR)).show(ui, |ui| self.toolbar_ui(ui));
        }
        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.canvas_ui(ui);
        });
        if let Some(v) = self.pending_edit.take() {
            if self.repo.alive(v) {
                self.edit_view(v, None);
            }
        }
        self.take_pending_item();
        self.dialogs(&ctx);
        self.toasts_ui(&ctx);
        if self.screenshot.is_none() {
            self.store_prefs();
        }
        self.screenshots(&ctx);
        let _ = R::default();
    }
}
