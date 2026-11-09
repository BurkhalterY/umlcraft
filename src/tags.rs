//! Model > Tag Editor (Ctrl+T) and the Tag dialog: an element's tags, with its stereotype's attributes
//! offered first as slots (tag-editor-dialog.js, tag-value-editor-dialog.js).

use crate::app::App;
use crate::model::Eid;
use egui::Vec2;
use serde_json::json;

pub const KINDS: [&str; 5] = ["string", "reference", "boolean", "number", "enum"];

/// The Tag dialog's fields while it is open.
#[derive(Clone, Debug, Default)]
pub struct TagEdit {
    /// The tag being edited, or None for a new one.
    pub tag: Option<Eid>,
    pub name: String,
    pub kind: String,
    pub value: String,
    pub checked: bool,
    pub number: f64,
    pub options: String,
    pub reference: Option<Eid>,
    pub hidden: bool,
    /// Editing a stereotype slot: the name is fixed.
    pub slot: bool,
}

/// A row of the Tag Editor: a tag, or an empty stereotype slot (an attribute of the stereotype).
#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    Tag(Eid),
    Slot(Eid),
}

impl App {
    /// The element's stereotype, when it is a UMLStereotype element.
    fn stereotype_of(&self, e: Eid) -> Option<Eid> {
        self.repo.refd(e, "stereotype").filter(|s| self.repo.kind_of(*s, "UMLStereotype") || self.repo.ty(*s) == "UMLStereotype")
    }

    /// The stereotype's attributes, inherited ones included, sorted by name.
    fn stereotype_slots(&self, e: Eid) -> Vec<Eid> {
        let Some(st) = self.stereotype_of(e) else { return vec![] };
        let mut out = vec![];
        let mut seen = vec![];
        let mut todo = vec![st];
        while let Some(s) = todo.pop() {
            if seen.contains(&s) {
                continue;
            }
            seen.push(s);
            out.extend(self.repo.list(s, "attributes"));
            // generalizations from s to its parents
            for o in self.repo.all() {
                if self.repo.ty(o) == "UMLGeneralization" && self.repo.refd(o, "source") == Some(s) {
                    if let Some(t) = self.repo.refd(o, "target") {
                        todo.push(t);
                    }
                }
            }
        }
        out.sort_by_key(|a| self.repo.name(*a));
        out
    }

    /// The rows of the Tag Editor: stereotype slots first (with their tag if there is one), then the
    /// other tags, both sorted by name (tag-editor-dialog.js:128-153).
    pub fn tag_rows(&self, e: Eid) -> Vec<Row> {
        let tags = self.repo.list(e, "tags");
        let mut rows = vec![];
        let mut bound = vec![];
        for slot in self.stereotype_slots(e) {
            let name = self.repo.name(slot);
            match tags.iter().copied().find(|t| self.repo.name(*t) == name && !bound.contains(t)) {
                Some(t) => {
                    bound.push(t);
                    rows.push(Row::Tag(t));
                }
                None => rows.push(Row::Slot(slot)),
            }
        }
        let mut rest: Vec<Eid> = tags.into_iter().filter(|t| !bound.contains(t)).collect();
        rest.sort_by_key(|t| self.repo.name(*t));
        rows.extend(rest.into_iter().map(Row::Tag));
        rows
    }

    /// The value of a tag as text, by kind.
    pub fn tag_value_text(&self, t: Eid) -> String {
        match self.repo.str(t, "kind").as_str() {
            "boolean" => self.repo.bool(t, "checked").to_string(),
            "number" => {
                let n = self.repo.f(t, "number");
                if n.fract() == 0.0 { format!("{}", n as i64) } else { n.to_string() }
            }
            "reference" => self.repo.refd(t, "reference").map(|r| self.repo.name(r)).unwrap_or_default(),
            _ => self.repo.str(t, "value"),
        }
    }

    /// Start editing a row (or a new tag when `row` is None). An empty slot takes the attribute's name, and
    /// its kind and default come from the attribute's type (tag-value-editor-dialog.js:186-227).
    pub fn edit_tag(&mut self, row: Option<Row>) {
        let mut ed = TagEdit { kind: "string".into(), ..Default::default() };
        match row {
            Some(Row::Tag(t)) => {
                let r = &self.repo;
                ed.tag = Some(t);
                ed.name = r.name(t);
                ed.kind = { let k = r.str(t, "kind"); if k.is_empty() { "string".into() } else { k } };
                ed.value = r.str(t, "value");
                ed.checked = r.bool(t, "checked");
                ed.number = r.f(t, "number");
                ed.options = r.str(t, "options");
                ed.reference = r.refd(t, "reference");
                ed.hidden = r.bool(t, "hidden");
                ed.slot = self.tag_editor.map(|e| self.stereotype_slots(e).iter().any(|s| self.repo.name(*s) == ed.name)).unwrap_or(false);
            }
            Some(Row::Slot(a)) => {
                let r = &self.repo;
                ed.name = r.name(a);
                ed.slot = true;
                ed.value = r.str(a, "defaultValue");
                let ty = r.refd(a, "type");
                let tname = crate::views::type_name(r, a, "type").to_lowercase();
                if let Some(t) = ty.filter(|t| r.ty(*t) == "UMLEnumeration") {
                    ed.kind = "enum".into();
                    ed.options = r.list(t, "literals").iter().map(|l| r.name(*l)).collect::<Vec<_>>().join("\n");
                } else if ty.is_some() {
                    ed.kind = "reference".into();
                } else if tname == "number" || tname == "integer" || tname == "int" {
                    ed.kind = "number".into();
                    ed.number = ed.value.parse().unwrap_or(0.0);
                } else if tname == "boolean" {
                    ed.kind = "boolean".into();
                    ed.checked = ed.value == "true";
                }
            }
            None => {}
        }
        self.tag_edit = Some(ed);
    }

    /// OK in the Tag dialog: create or update the tag, storing only the fields of its kind
    /// (tag-editor-dialog.js:56-95).
    pub fn apply_tag_edit(&mut self, e: Eid, ed: &TagEdit) {
        self.change(|a| {
            let t = match ed.tag.filter(|t| a.repo.alive(*t)) {
                Some(t) => t,
                None => a.repo.create("Tag", &[], Some((e, "tags"))),
            };
            for k in ["value", "reference", "checked", "number", "options"] {
                a.repo.remove(t, k);
            }
            a.repo.set(t, "name", json!(ed.name));
            a.repo.set_raw(t, "kind", json!(ed.kind));
            match ed.kind.as_str() {
                "boolean" => a.repo.set(t, "checked", json!(ed.checked)),
                "number" => a.repo.set(t, "number", crate::model::num(ed.number)),
                "reference" => a.repo.set_ref(t, "reference", ed.reference),
                "enum" => {
                    a.repo.set(t, "value", json!(ed.value));
                    a.repo.set(t, "options", json!(ed.options));
                }
                _ => a.repo.set(t, "value", json!(ed.value)),
            }
            a.repo.set(t, "hidden", json!(ed.hidden));
        });
    }

    pub fn open_tag_editor(&mut self) {
        if let Some(f) = self.focus.filter(|f| self.repo.alive(*f) && self.repo.kind_of(*f, "ExtensibleModel")) {
            self.tag_editor = Some(f);
            self.tag_row = None;
        }
    }

    /// The Tag Editor window: rows, then Add, Edit, Delete and Close.
    pub fn tag_editor_ui(&mut self, ctx: &egui::Context) {
        let Some(e) = self.tag_editor.filter(|e| self.repo.alive(*e)) else {
            self.tag_editor = None;
            return;
        };
        // a key the Tag dialog takes (Esc) must not also close the editor in the same frame
        let dialog_open = self.tag_edit.is_some() || self.element_picker.is_some();
        if self.tag_edit.is_some() {
            self.tag_dialog_ui(ctx, e);
        }
        let rows = self.tag_rows(e);
        let mut close = false;
        let title = format!("Tag Editor - {}", self.repo.name(e));
        crate::dialogs::dialog(ctx, "tag-editor", &title, |ui| {
            ui.set_min_width(380.0);
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                if rows.is_empty() {
                    ui.weak("No tags");
                }
                for row in &rows {
                    let (name, value, slot) = match row {
                        Row::Tag(t) => {
                            let st = self.stereotype_slots(e).iter().any(|s| self.repo.name(*s) == self.repo.name(*t));
                            (self.repo.name(*t), self.tag_value_text(*t), st)
                        }
                        Row::Slot(a) => (self.repo.name(*a), String::new(), true),
                    };
                    let st = if slot { self.stereotype_of(e).map(|s| format!("«{}» ", self.repo.name(s))).unwrap_or_default() } else { String::new() };
                    // the tag icon, the name, the value styled by kind, and the referenced element's
                    // icon for a reference (tag-editor-dialog.js:27-41, 97-112)
                    let (kind, referenced) = match row {
                        Row::Tag(t) => (self.repo.str(*t, "kind"), self.repo.refd(*t, "reference").filter(|_| self.repo.str(*t, "kind") == "reference")),
                        Row::Slot(_) => (String::new(), None),
                    };
                    let mut job = egui::text::LayoutJob::default();
                    let dim = matches!(row, Row::Slot(_));
                    let base = if dim { crate::ui::DIM() } else { crate::ui::TEXT() };
                    let fmt = |c: egui::Color32| egui::TextFormat { font_id: egui::FontId::proportional(13.0), color: c, ..Default::default() };
                    job.append(&format!("{st}{name} = "), 0.0, fmt(base));
                    let value_color = match kind.as_str() {
                        "boolean" | "number" => egui::Color32::from_rgb(0x4f, 0x99, 0xff),
                        "reference" => egui::Color32::from_rgb(0xd8, 0xa0, 0x3c),
                        "hidden" => crate::ui::DIM(),
                        _ => base,
                    };
                    job.append(&value, 0.0, fmt(value_color));
                    let r = ui
                        .horizontal(|ui| {
                            let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
                            crate::icons::draw(ui.painter(), ir, "tag");
                            let r = ui.selectable_label(self.tag_row.as_ref() == Some(row), job);
                            if let Some(x) = referenced {
                                let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
                                crate::icons::draw(ui.painter(), ir, crate::icons::key_for(&self.repo, x));
                            }
                            r
                        })
                        .inner;
                    if r.clicked() {
                        self.tag_row = Some(row.clone());
                    }
                    if r.double_clicked() {
                        self.edit_tag(Some(row.clone()));
                    }
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Add").clicked() {
                    self.edit_tag(None);
                }
                if ui.add_enabled(self.tag_row.is_some(), egui::Button::new("Edit")).clicked() {
                    self.edit_tag(self.tag_row.clone());
                }
                let can_delete = matches!(self.tag_row, Some(Row::Tag(_)));
                if ui.add_enabled(can_delete, egui::Button::new("Delete")).clicked() {
                    if let Some(Row::Tag(t)) = self.tag_row.clone() {
                        // a confirmation box (tag-editor-dialog.js:221)
                        if self.confirm("Delete this tag?") {
                            self.change(|a| a.repo.detach(t));
                            self.tag_row = None;
                        }
                    }
                }
                // Esc closes the editor, unless the Tag dialog is open over it
                if ui.button("Close").clicked() || (!dialog_open && ui.input(|i| i.key_pressed(egui::Key::Escape))) {
                    close = true;
                }
            });
        });
        if close {
            self.tag_editor = None;
            self.tag_edit = None;
        }
    }

    fn tag_dialog_ui(&mut self, ctx: &egui::Context, e: Eid) {
        let Some(mut ed) = self.tag_edit.clone() else { return };
        // the Element Picker is open over the dialog: wait for it
        if self.element_picker.as_ref().map(|p| p.tag_reference).unwrap_or(false) {
            return;
        }
        let (mut ok, mut cancel, mut pick) = (false, false, false);
        crate::dialogs::dialog(ctx, "tag", "Tag", |ui| {
            egui::Grid::new("tag-grid").num_columns(2).spacing([10.0, 6.0]).show(ui, |ui| {
                ui.label("Name");
                ui.add_enabled(!ed.slot, egui::TextEdit::singleline(&mut ed.name).desired_width(200.0));
                ui.end_row();
                ui.label("Kind");
                egui::ComboBox::from_id_salt("tag-kind").selected_text(ed.kind.clone()).show_ui(ui, |ui| {
                    for k in KINDS {
                        ui.selectable_value(&mut ed.kind, k.to_string(), k);
                    }
                });
                ui.end_row();
                ui.label("Value");
                match ed.kind.as_str() {
                    "boolean" => {
                        egui::ComboBox::from_id_salt("tag-bool").selected_text(ed.checked.to_string()).show_ui(ui, |ui| {
                            ui.selectable_value(&mut ed.checked, false, "false");
                            ui.selectable_value(&mut ed.checked, true, "true");
                        });
                    }
                    "number" => {
                        ui.add(egui::DragValue::new(&mut ed.number));
                    }
                    "reference" => {
                        // the referenced element, and a button that opens the Element Picker
                        // ("Select Element", any Model, the current one selected; tag-value-editor-dialog.js:165-174)
                        ui.horizontal(|ui| {
                            match ed.reference.filter(|r| self.repo.alive(*r)) {
                                Some(x) => {
                                    let (ir, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
                                    crate::icons::draw(ui.painter(), ir, crate::icons::key_for(&self.repo, x));
                                    ui.label(self.repo.name(x));
                                }
                                None => {
                                    ui.weak("—");
                                }
                            }
                            if ui.button("…").on_hover_text("Select Element").clicked() {
                                pick = true;
                            }
                        });
                    }
                    "enum" => {
                        let opts: Vec<String> = ed.options.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
                        egui::ComboBox::from_id_salt("tag-enum").selected_text(ed.value.clone()).show_ui(ui, |ui| {
                            for o in opts {
                                ui.selectable_value(&mut ed.value, o.clone(), o);
                            }
                        });
                    }
                    _ => {
                        ui.add(egui::TextEdit::multiline(&mut ed.value).desired_rows(3).desired_width(200.0));
                    }
                }
                ui.end_row();
                if ed.kind == "enum" {
                    ui.label("Options");
                    ui.add(egui::TextEdit::multiline(&mut ed.options).desired_rows(3).desired_width(200.0).hint_text("one per line"));
                    ui.end_row();
                }
                ui.label("");
                ui.checkbox(&mut ed.hidden, "Hidden");
                ui.end_row();
            });
            ui.horizontal(|ui| {
                ok = ui.button("OK").clicked();
                cancel = ui.button("Cancel").clicked();
                // Enter is OK (Ctrl+Enter in the options box), Esc cancels (dialog-manager.js:148-162)
                ok |= ui.input(|i| i.key_pressed(egui::Key::Enter) && (i.modifiers.command || ed.kind != "enum"));
                cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
            });
        });
        if ok && !ed.name.trim().is_empty() {
            ed.name = ed.name.trim().to_string();
            self.apply_tag_edit(e, &ed);
            self.tag_edit = None;
        } else if cancel {
            self.tag_edit = None;
        } else {
            let current = ed.reference;
            self.tag_edit = Some(ed);
            if pick {
                self.open_element_picker(self.repo.root, "");
                if let Some(p) = self.element_picker.as_mut() {
                    p.ty = "Model";
                    p.tag_reference = true;
                    p.chosen = current;
                }
                if let Some(c) = current.filter(|c| self.repo.alive(*c)) {
                    let path = self.repo.path(c);
                    if let Some(p) = self.element_picker.as_mut() {
                        p.open.extend(path);
                    }
                }
            }
        }
    }
}
