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
fn order_rank(t: &str) -> u32 {
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

    fn tree_row(&mut self, ui: &mut egui::Ui, e: Eid, depth: usize, has_kids: bool, open: bool) {
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
        let label = self.label_of(e);
        let tx = ir.right() + 6.0;
        let tw = text_galley_width(ui, &label, 13.0);
        if self.focus == Some(e) {
            let lr = Rect::from_min_size(Pos2::new(tx - 3.0, rect.top() + 1.0), Vec2::new(tw + 6.0, 20.0));
            p.rect_filled(lr, 0.0, Color32::from_rgb(0x2d, 0x32, 0x38));
            p.rect_stroke(lr, 0.0, Stroke::new(1.0, Color32::from_rgb(0x4f, 0x99, 0xff)), egui::StrokeKind::Inside);
        }
        p.text(Pos2::new(tx, rect.center().y), Align2::LEFT_CENTER, &label, FontId::proportional(13.0), TEXT);

        let ty = self.repo.ty(e).to_string();
        if views::node_kind(&ty).is_some() {
            resp.dnd_set_drag_payload(e);
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
        self.tree_row(ui, e, depth, !kids.is_empty(), open);
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
        ui.separator();
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
            if resp.on_hover_text("Sort by name").clicked() {
                self.sort_by_name = !self.sort_by_name;
            }
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let q = self.search.trim().to_lowercase();
            if q.is_empty() {
                let root = self.repo.root;
                self.tree(ui, root, 0);
            } else {
                let all: Vec<Eid> = self.repo.all().filter(|e| !self.is_view(*e)).collect();
                for e in all {
                    if self.label_of(e).to_lowercase().contains(&q) {
                        self.tree_row(ui, e, 0, false, false);
                    }
                }
            }
            ui.add_space(8.0);
        });
        if ui.ui_contains_pointer() && ui.input(|i| i.key_pressed(egui::Key::Delete)) && self.last_area == Area::Explorer && !text_field_focused(ui.ctx()) {
            if let Some(f) = self.focus.filter(|f| self.repo.ty(*f) != "Project" && !self.is_view(*f)) {
                self.change(|a| a.delete_models(&[f]));
            }
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
                    if resp.double_clicked() {
                        self.tool = Some(*it);
                        self.tool_sticky = true;
                    } else if resp.clicked() {
                        self.tool = if selected { None } else { Some(*it) };
                        self.tool_sticky = false;
                    }
                }
            }
        });
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
                        "zoomIn" => self.set_zoom(self.zoom + crate::app::ZOOM_STEP),
                        "zoomOut" => self.set_zoom(self.zoom - crate::app::ZOOM_STEP),
                        "fit" => self.fit(),
                        "grid" => self.show_grid = !self.show_grid,
                        _ => self.export_png(),
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
                if ui.button("Export Diagram As PNG...").clicked() {
                    self.export_png();
                    ui.close();
                }
                if ui.button("Export Diagram As SVG...").clicked() {
                    self.export_svg(&measure);
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
            });
            ui.menu_button("Format", |ui| {
                ui.menu_button("Line Style", |ui| {
                    if ui.button("Rectilinear   Ctrl+L").clicked() {
                        self.set_line_style(0);
                        ui.close();
                    }
                    if ui.button("Oblique       Ctrl+B").clicked() {
                        self.set_line_style(1);
                        ui.close();
                    }
                });
                if ui.button("Show Shadow   Ctrl+Shift+H").clicked() {
                    self.toggle_shadow();
                    ui.close();
                }
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
                if ui.button("Close Diagram       F4").clicked() {
                    self.close_diagram(None);
                    ui.close();
                }
                ui.separator();
                if ui.button("Zoom In             Ctrl+=").clicked() {
                    self.set_zoom(self.zoom + crate::app::ZOOM_STEP);
                }
                if ui.button("Zoom Out            Ctrl+-").clicked() {
                    self.set_zoom(self.zoom - crate::app::ZOOM_STEP);
                }
                if ui.button("Actual Size         Ctrl+0").clicked() {
                    self.set_zoom(1.0);
                }
                if ui.button("Fit To Window       Ctrl+9").clicked() {
                    self.fit();
                    ui.close();
                }
                ui.separator();
                ui.checkbox(&mut self.show_grid, "Show Grid");
                ui.checkbox(&mut self.snap, "Snap to Grid");
                ui.separator();
                ui.checkbox(&mut self.show_sidebar, "Sidebar");
                ui.checkbox(&mut self.show_navigator, "Navigator");
                ui.checkbox(&mut self.show_statusbar, "Statusbar");
            });
            ui.menu_button("Help", |ui| {
                if ui.button("About UMLCraft").clicked() {
                    self.about_open = true;
                    ui.close();
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
        if hit(sc(Modifiers::COMMAND, Key::Y)) || hit(sc(cs, Key::Z)) {
            self.do_redo();
        }
        if hit(sc(Modifiers::COMMAND, Key::Z)) {
            self.do_undo();
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
            self.set_zoom(self.zoom + crate::app::ZOOM_STEP);
        }
        if hit(sc(Modifiers::COMMAND, Key::Minus)) {
            self.set_zoom(self.zoom - crate::app::ZOOM_STEP);
        }
        if hit(sc(Modifiers::COMMAND, Key::Num0)) {
            self.set_zoom(1.0);
        }
        if hit(sc(Modifiers::COMMAND, Key::Num9)) {
            self.fit();
        }
        if hit(sc(Modifiers::COMMAND, Key::G)) {
            self.show_grid = !self.show_grid;
        }
        if hit(sc(cs, Key::G)) {
            self.snap = !self.snap;
        }
        if hit(sc(Modifiers::NONE, Key::F4)) {
            self.close_diagram(None);
        }
    }

    fn cmd_new(&mut self, ms: &EguiMeasure) {
        if self.confirm_discard(ms) {
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

    fn export_png(&mut self) {
        if self.diagram.is_none() {
            return;
        }
        let name = self.diagram.map(|d| self.repo.name(d)).unwrap_or_else(|| "diagram".into());
        if let Some(p) = rfd::FileDialog::new().set_title("Export Diagram as PNG").set_file_name(format!("{name}.png")).add_filter("PNG", &["png"]).save_file() {
            self.pending_png = Some(p);
        }
    }

    fn export_svg(&mut self, ms: &EguiMeasure) {
        let Some(d) = self.diagram else { return };
        let info = views::layout_diagram(&mut self.repo, ms, d);
        let Some(svg) = render::to_svg(&self.repo, d, &info, ms) else { return };
        let name = self.repo.name(d);
        if let Some(p) = rfd::FileDialog::new().set_title("Export Diagram as SVG").set_file_name(format!("{name}.svg")).add_filter("SVG", &["svg"]).save_file() {
            if let Err(e) = std::fs::write(&p, svg) {
                self.message = Some(("Failed to export the diagram.".into(), e.to_string()));
            }
        }
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
        if let Some((title, detail)) = self.message.clone() {
            egui::Window::new(title).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
                ui.label(detail);
                if ui.button("OK").clicked() {
                    self.message = None;
                }
            });
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
            if let Some(path) = self.pending_png.take() {
                // export: the canvas area of the window, cropped to the diagram
                if let (Some(rect), Some(d)) = (self.canvas_rect, self.diagram) {
                    let ppp = ctx.pixels_per_point();
                    let mut crop = rect;
                    if let Some(b) = render::bounds(&self.repo, d) {
                        let to = |x: f64, y: f64| Pos2::new(rect.min.x + ((x - self.scroll.x) * self.zoom) as f32, rect.min.y + ((y - self.scroll.y) * self.zoom) as f32);
                        let m = 10.0;
                        let br = Rect::from_two_pos(to(b.x - m, b.y - m), to(b.x + b.w + m + render::SHADOW_OFFSET, b.y + b.h + m + render::SHADOW_OFFSET));
                        crop = br.intersect(rect);
                    }
                    let region = img.region(&crop, Some(ppp));
                    save_png(&path, &region);
                }
            } else if let Some(path) = self.screenshot.clone() {
                save_png(&path, &img);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.pending_png.is_some() && !self.shot_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
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
                Panel::bottom("toolbox").exact_size(TOOLBOX_HEIGHT).show_separator_line(false).frame(Frame::NONE.fill(LAYER1)).show(ui, |ui| self.toolbox_ui(ui));
                self.working_ui(ui);
            });
        }
        if self.show_navigator {
            Panel::right("navigator").exact_size(NAVIGATOR_WIDTH).show_separator_line(false).frame(Frame::NONE.fill(LAYER0)).show(ui, |ui| {
                Panel::bottom("editors").exact_size(EDITORS_HEIGHT).show_separator_line(false).frame(Frame::NONE.fill(LAYER1)).show(ui, |ui| self.editors_ui(ui));
                self.explorer_ui(ui);
            });
        }
        Panel::right("toolbar").exact_size(TOOLBAR_WIDTH).show_separator_line(false).frame(Frame::NONE.fill(TOOLBAR)).show(ui, |ui| self.toolbar_ui(ui));
        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.canvas_ui(ui);
        });
        if let Some(v) = self.pending_edit.take() {
            if self.repo.alive(v) {
                self.edit_view(v, None);
            }
        }
        self.dialogs(&ctx);
        self.screenshots(&ctx);
        let _ = R::default();
    }
}
