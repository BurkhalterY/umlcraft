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
}

/// The element type a reference field accepts.
pub fn field_type(field: &str) -> &'static str {
    match field {
        "stereotype" => "UMLStereotype",
        "signal" => "UMLSignal",
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
    pub fn open_element_picker(&mut self, obj: Eid, field: &str) {
        let chosen = self.repo.raw(obj, field).and_then(|v| self.repo.by_ref(v)).filter(|x| self.repo.alive(*x));
        let mut open: HashSet<Eid> = HashSet::from([self.repo.root]);
        if let Some(c) = chosen {
            open.extend(self.repo.path(c));
        }
        self.element_picker = Some(ElementPicker { obj, field: field.to_string(), ty: field_type(field), chosen, search: String::new(), open, fresh: true, export_fragment: false, tag_reference: false });
    }

    /// Apply a picked element (or none) to the reference, as one undo step.
    pub fn apply_picked(&mut self, obj: Eid, field: &str, chosen: Option<Eid>) {
        let field = field.to_string();
        self.change(|a| match chosen {
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
        if p.tag_reference {
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
        egui::Window::new("Select Element").collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
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
