//! The Element Picker dialog, "Select Element" (dialogs/element-picker-dialog.js), and the name lookup
//! of the property editor's type and stereotype fields (property-editor-view.js:575-696,
//! core/repository.js:2353-2361).

use crate::app::App;
use crate::icons;
use crate::model::Eid;
use egui::{Align2, Sense, Vec2};
use std::collections::HashSet;

/// An open Element Picker: the reference it edits, the required type, the choice so far.
#[derive(Clone, Debug)]
pub struct ElementPicker {
    pub obj: Eid,
    pub field: String,
    pub ty: &'static str,
    pub chosen: Option<Eid>,
    pub search: String,
    pub open: HashSet<Eid>,
    pub fresh: bool,
    /// File > Export > Fragment: OK saves the chosen element instead of setting a reference.
    pub export_fragment: bool,
    /// The Tag dialog's reference: OK fills the dialog instead of setting a field.
    pub tag_reference: bool,
    /// The Element List Editor's Add: OK appends to its list.
    pub list_add: bool,
    /// The Frame tool: OK creates a frame of the chosen element (or of none) in this diagram, here.
    pub frame: Option<(Eid, crate::views::R)>,
}

/// The element type a reference field accepts.
pub fn field_type(field: &str) -> &'static str {
    match field {
        "stereotype" => "UMLStereotype",
        "signal" => "UMLSignal",
        "reference" => "Model",
        _ => "UMLClassifier",
    }
}

impl App {
    /// The children shown in the picker tree: model elements in the order they were added.
    fn picker_children(&self, e: Eid) -> Vec<Eid> {
        self.repo.children(e).into_iter().filter(|(k, _)| k != "ownedViews" && k != "subViews").map(|(_, c)| c).collect()
    }

    /// Find an element of type `ty` named `name`: in the owner's namespace, then each enclosing one,
    /// then anywhere in the project (repository.js:2353-2361).
    pub fn resolve_name(&self, obj: Eid, name: &str, ty: &str) -> Option<Eid> {
        let fits = |x: Eid| self.repo.kind_of(x, ty) && self.repo.name(x) == name;
        let mut ns = self.repo.parent(obj);
        while let Some(n) = ns {
            if let Some(x) = self.picker_children(n).into_iter().find(|x| fits(*x)) {
                return Some(x);
            }
            ns = self.repo.parent(n);
        }
        self.repo.all().find(|x| !self.is_view(*x) && fits(*x))
    }

    /// Open the picker for a reference field, with the current value selected and its owners expanded;
    /// with no value, "Do not specify" starts checked.
    /// The Element Picker for a field that takes elements of a given type (a message's signature: an
    /// operation or a signal).
    pub fn open_typed_picker(&mut self, obj: Eid, field: &str, ty: &'static str) {
        self.open_element_picker(obj, field);
        if let Some(p) = self.element_picker.as_mut() {
            p.ty = ty;
        }
    }

    pub fn open_element_picker(&mut self, obj: Eid, field: &str) {
        let chosen = self.repo.raw(obj, field).and_then(|v| self.repo.by_ref(v)).filter(|x| self.repo.alive(*x));
        let mut open: HashSet<Eid> = HashSet::from([self.repo.root]);
        if let Some(c) = chosen {
            open.extend(self.repo.path(c));
        }
        self.element_picker = Some(ElementPicker { obj, field: field.to_string(), ty: field_type(field), chosen, search: String::new(), open, fresh: true, export_fragment: false, tag_reference: false, list_add: false, frame: None });
    }

    /// The Frame tool's "Select an element to be represented as Frame": any model element, nothing
    /// chosen at first (uml-commands.js:244-262).
    pub fn open_frame_picker(&mut self, diagram: Eid, rect: crate::views::R) {
        let open: HashSet<Eid> = HashSet::from([self.repo.root]);
        self.element_picker = Some(ElementPicker { obj: diagram, field: String::new(), ty: "Model", chosen: None, search: String::new(), open, fresh: true, export_fragment: false, tag_reference: false, list_add: false, frame: Some((diagram, rect)) });
    }

    /// Create a frame of `model` (or of nothing), behind the other views, and select it.
    pub fn create_frame(&mut self, diagram: Eid, model: Option<Eid>, rect: crate::views::R) -> Option<Eid> {
        if !self.repo.alive(diagram) {
            return None;
        }
        let v = self.change(|a| {
            let v = crate::views::create_frame_view(&mut a.repo, diagram, model, rect);
            a.apply_view_defaults(v);
            // a frame's zIndex is -1: it is drawn behind the views of its diagram (elements.js:12254)
            let mut list = a.repo.list(diagram, "ownedViews");
            list.retain(|x| *x != v);
            list.insert(0, v);
            a.repo.set_list(diagram, "ownedViews", list);
            v
        });
        if self.diagram == Some(diagram) {
            self.select(vec![v]);
        }
        Some(v)
    }

    /// Apply a picked element (or none) to the reference, as one undo step.
    pub fn apply_picked(&mut self, obj: Eid, field: &str, chosen: Option<Eid>) {
        let field = field.to_string();
        self.change(|a| match chosen {
            // Assign Participant adds to a choreography's participants (bpmn-commands.js:90-103)
            Some(c) if matches!(field.as_str(), "upperParticipants" | "lowerParticipants") => {
                let mut list = a.repo.ref_list(obj, &field);
                if !list.contains(&c) {
                    list.push(c);
                }
                a.repo.set_ref_list(obj, &field, &list);
            }
            Some(c) => a.repo.set_ref(obj, &field, Some(c)),
            None if field == "type" => a.repo.set_raw(obj, "type", serde_json::json!("")),
            None => a.repo.remove(obj, &field),
        });
        self.prop_buf.clear();
    }

    fn picker_tree(&self, ui: &mut egui::Ui, e: Eid, depth: usize, p: &mut ElementPicker, ok: &mut bool) {
        let kids = self.picker_children(e);
        ui.horizontal(|ui| {
            ui.add_space(depth as f32 * 14.0);
            let open = p.open.contains(&e);
            let (ar, resp) = ui.allocate_exact_size(Vec2::new(12.0, 16.0), Sense::click());
            if !kids.is_empty() {
                ui.painter().text(ar.center(), Align2::CENTER_CENTER, if open { "▾" } else { "▸" }, egui::FontId::proportional(11.0), ui.visuals().text_color());
                if resp.clicked() {
                    if open {
                        p.open.remove(&e);
                    } else {
                        p.open.insert(e);
                    }
                }
            }
            let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
            icons::draw(ui.painter(), ir, icons::key_for(&self.repo, e));
            let label = if e == self.repo.root && self.repo.name(e).is_empty() { "(Project)".to_string() } else { self.label_of(e) };
            let r = ui.selectable_label(p.chosen == Some(e), label);
            if r.clicked() {
                p.chosen = Some(e);
            }
            if r.double_clicked() {
                p.chosen = Some(e);
                *ok = true;
            }
            if p.fresh && p.chosen == Some(e) {
                r.scroll_to_me(Some(egui::Align::Center));
            }
        });
        if p.open.contains(&e) {
            for k in kids {
                self.picker_tree(ui, k, depth + 1, p, ok);
            }
        }
    }

    /// "Select Element": a search box (focused on open), the project tree, "Do not specify", OK and
    /// Cancel. Double-click is OK. A choice of the wrong type is refused with an alert.
    /// OK in the Element Picker: fill the Tag dialog, export the fragment, or set the reference field.
    pub fn picker_ok(&mut self, p: ElementPicker) {
        self.element_picker = None;
        if let Some((d, rect)) = p.frame {
            self.create_frame(d, p.chosen.filter(|c| self.repo.alive(*c)), rect);
        } else if p.list_add {
            if let (Some((obj, field, _)), Some(c)) = (self.list_editor, p.chosen.filter(|c| self.repo.alive(*c))) {
                let mut items = self.repo.ref_list(obj, field);
                items.push(c);
                self.change(|a| a.repo.set_ref_list(obj, field, &items));
                self.list_editor = Some((obj, field, Some(items.len() - 1)));
            }
        } else if p.tag_reference {
            if let Some(ed) = self.tag_edit.as_mut() {
                ed.reference = p.chosen.filter(|c| self.repo.alive(*c));
            }
        } else if p.export_fragment {
            if let Some(c) = p.chosen.filter(|c| self.repo.alive(*c)) {
                self.save_fragment(c);
            }
        } else {
            match p.chosen {
                Some(c) if !self.repo.kind_of(c, p.ty) => {
                    self.message = Some((format!("Selected element ({}) is not an instance of {}", self.repo.name(c), p.ty), String::new()));
                }
                chosen if self.repo.alive(p.obj) => self.apply_picked(p.obj, &p.field, chosen),
                _ => {}
            }
        }
    }

    pub fn element_picker_ui(&mut self, ctx: &egui::Context) {
        let Some(mut p) = self.element_picker.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        let title = if p.frame.is_some() {
            "Select an element to be represented as Frame"
        } else if p.list_add {
            "Select Element to Add"
        } else {
            "Select Element"
        };
        egui::Window::new(title).id(egui::Id::new("element-picker")).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(360.0);
            let r = ui.add(egui::TextEdit::singleline(&mut p.search).hint_text("Quick search").desired_width(f32::INFINITY));
            // Enter is OK, except in the search box (`use-enter-key`)
            if !r.has_focus() && !r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ok = true;
            }
            if p.fresh {
                r.request_focus();
            }
            let q = p.search.trim().to_lowercase();
            if !q.is_empty() {
                // matches by name, with their namespace; picking one selects it in the tree
                let hits: Vec<Eid> = self.repo.all().filter(|x| !self.is_view(*x) && self.repo.kind_of(*x, "Model") && self.repo.name(*x).to_lowercase().contains(&q)).take(99).collect();
                egui::ScrollArea::vertical().id_salt("picker-search").max_height(120.0).show(ui, |ui| {
                    for x in hits {
                        ui.horizontal(|ui| {
                            let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            icons::draw(ui.painter(), ir, icons::key_for(&self.repo, x));
                            let mut owners = self.repo.path(x);
                            owners.pop();
                            let ns: Vec<String> = owners.into_iter().map(|o| self.repo.name(o)).filter(|n| !n.is_empty()).collect();
                            if ui.selectable_label(false, format!("{}  ({})", self.repo.name(x), ns.join("::"))).clicked() {
                                p.chosen = Some(x);
                                p.open.extend(self.repo.path(x));
                                p.search.clear();
                                p.fresh = true;
                            }
                        });
                    }
                });
                ui.separator();
            }
            let fresh = p.fresh;
            egui::ScrollArea::vertical().id_salt("picker-tree").max_height(320.0).auto_shrink([false, true]).show(ui, |ui| {
                let root = self.repo.root;
                self.picker_tree(ui, root, 0, &mut p, &mut ok);
            });
            if fresh {
                p.fresh = false;
            }
            let mut none = p.chosen.is_none();
            if ui.checkbox(&mut none, "Do not specify").changed() && none {
                p.chosen = None;
            }
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    ok = true;
                }
                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel = true;
                }
            });
        });
        if ok {
            self.picker_ok(p);
        } else if cancel {
            self.element_picker = None;
        } else {
            self.element_picker = Some(p);
        }
    }
}

impl App {
    /// The Element List Editor (element-list-editor-dialog.js): titled `<element>.<field>`, the
    /// referenced elements with icon, name and (path); Add opens the Element Picker "Select Element
    /// to Add" with no type filter; Remove, Up and Down apply at once and keep the moved item selected.
    pub fn list_editor_ui(&mut self, ctx: &egui::Context) {
        let Some((obj, field, mut sel)) = self.list_editor else { return };
        if !self.repo.alive(obj) {
            self.list_editor = None;
            return;
        }
        // the picker is open over the editor: wait for it
        if self.element_picker.as_ref().map(|p| p.list_add).unwrap_or(false) {
            return;
        }
        let items = self.repo.ref_list(obj, field);
        let title = format!("{}.{}", self.repo.name(obj), field);
        let (mut add, mut remove, mut up, mut down, mut close) = (false, false, false, false, false);
        egui::Window::new(title).id(egui::Id::new("list-editor")).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_min_width(360.0);
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                if items.is_empty() {
                    ui.weak("No elements");
                }
                for (i, x) in items.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
                        crate::icons::draw(ui.painter(), ir, crate::icons::key_for(&self.repo, *x));
                        let owner = self.repo.parent(*x).map(|p| self.path_name(p)).unwrap_or_default();
                        let label = format!("{} ({owner})", self.repo.name(*x));
                        if ui.selectable_label(sel == Some(i), label).clicked() {
                            sel = Some(i);
                        }
                    });
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                add = ui.button("Add").clicked();
                remove = ui.add_enabled(sel.is_some(), egui::Button::new("Remove")).clicked();
                up = ui.add_enabled(sel.map(|i| i > 0).unwrap_or(false), egui::Button::new("Up")).clicked();
                down = ui.add_enabled(sel.map(|i| i + 1 < items.len()).unwrap_or(false), egui::Button::new("Down")).clicked();
                close = ui.button("Close").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape));
            });
        });
        self.list_editor = Some((obj, field, sel));
        if close {
            self.list_editor = None;
        } else if add {
            self.open_element_picker(self.repo.root, "");
            if let Some(p) = self.element_picker.as_mut() {
                p.ty = "Model";
                p.list_add = true;
                p.chosen = None;
            }
        } else if let Some(i) = sel {
            let mut list = items.clone();
            let to = if remove {
                list.remove(i);
                if list.is_empty() { None } else { Some(i.min(list.len() - 1)) }
            } else if up && i > 0 {
                list.swap(i, i - 1);
                Some(i - 1)
            } else if down && i + 1 < list.len() {
                list.swap(i, i + 1);
                Some(i + 1)
            } else {
                return;
            };
            self.change(|a| a.repo.set_ref_list(obj, field, &list));
            self.list_editor = Some((obj, field, to));
        }
    }
}
