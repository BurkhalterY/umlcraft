//! Dialog plumbing shared by every dialog (dialogs/dialog-manager.js): modal windows, the native
//! message boxes, and the Element List Picker (dialogs/element-list-picker-dialog.js).

use crate::app::App;
use crate::icons;
use crate::model::Eid;
use egui::{Sense, Vec2};

/// A modal dialog: centred, not resizable, with a backdrop that keeps the rest of the window out of
/// reach. Tab and Shift+Tab only reach the dialog's own controls (egui gives focus only to the top
/// modal layer), as the original's Kendo windows do (dialog-manager.js:99-121, 242-313).
pub fn dialog<R>(ctx: &egui::Context, id: &str, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Modal::new(egui::Id::new(("dialog", id)))
        .show(ctx, |ui| {
            if !title.is_empty() {
                ui.label(egui::RichText::new(title).strong());
                ui.add_space(4.0);
            }
            add(ui)
        })
        .inner
}

/// The kinds of native message box (dialog-manager.js:649-845).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BoxKind {
    /// "Information", OK.
    Info,
    /// "Alert", a warning with OK.
    Alert,
}

/// The Element List Picker: a title, the elements to choose from, the chosen row ("Do not specify"
/// while none is) and what OK does with the choice.
#[derive(Clone, Debug)]
pub struct ListPicker {
    pub title: String,
    pub items: Vec<Eid>,
    pub chosen: Option<usize>,
    pub action: ListAction,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListAction {
    /// Select In Diagram: the view to show in each listed diagram (default-commands.js:877-907).
    ShowView(Vec<Eid>),
    /// A message's Select Operation: sets its `signature` (uml-commands.js:2525-2546).
    MessageOperation(Eid),
    /// An association end's Select Owner Attribute: sets its `ownerAttribute` (uml-commands.js:1465-1476).
    OwnerAttribute(Eid),
}

impl App {
    /// An alert box ("Alert", OK), shown at the next frame (dialog-manager.js:675-688).
    pub fn alert(&mut self, text: &str) {
        self.boxes.push((BoxKind::Alert, text.to_string()));
    }

    /// An information box ("Information", OK) (dialog-manager.js:690-703).
    pub fn info_box(&mut self, text: &str) {
        self.boxes.push((BoxKind::Info, text.to_string()));
    }

    /// A native confirmation box: "Confirmation", the message, OK and Cancel; true for OK
    /// (dialog-manager.js:660-673). Screenshots answer OK without asking.
    pub fn confirm(&self, text: &str) -> bool {
        self.screenshot.is_some()
            || rfd::MessageDialog::new().set_level(rfd::MessageLevel::Info).set_title("Confirmation").set_description(text).set_buttons(rfd::MessageButtons::OkCancel).show()
                == rfd::MessageDialogResult::Ok
    }

    /// Show the waiting message boxes and the error box, natively, as the original does through its
    /// main process (application.js:392-399). Screenshots draw them as modal windows instead, since a
    /// native box would wait for a click.
    pub(crate) fn message_boxes_ui(&mut self, ctx: &egui::Context) {
        if self.screenshot.is_none() {
            for (kind, text) in std::mem::take(&mut self.boxes) {
                let (title, level) = match kind {
                    BoxKind::Info => ("Information", rfd::MessageLevel::Info),
                    BoxKind::Alert => ("Alert", rfd::MessageLevel::Warning),
                };
                rfd::MessageDialog::new().set_level(level).set_title(title).set_description(&text).set_buttons(rfd::MessageButtons::Ok).show();
            }
            // an error box: the title and the details (showErrorBox)
            if let Some((title, detail)) = self.message.take() {
                rfd::MessageDialog::new().set_level(rfd::MessageLevel::Error).set_title(&title).set_description(if detail.is_empty() { &title } else { &detail }).set_buttons(rfd::MessageButtons::Ok).show();
            }
            return;
        }
        if self.message.is_none() {
            if let Some((kind, text)) = self.boxes.first().cloned() {
                self.message = Some((if kind == BoxKind::Info { "Information" } else { "Alert" }.to_string(), text));
                self.boxes.remove(0);
            }
        }
        if let Some((title, detail)) = self.message.clone() {
            dialog(ctx, "message", &title, |ui| {
                ui.set_max_width(420.0);
                ui.label(detail);
                ui.add_space(6.0);
                let ok = ui.button("OK");
                crate::ui::focus_primary(ui, &ok);
                if ok.clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Escape)) {
                    self.message = None;
                }
            });
        }
    }

    /// The Element List Picker's row text: the element's label and its path name, the names from
    /// the top-level element down to it (element-list-picker-dialog.js:49-56; core.js:1128-1135).
    pub fn list_row_text(&self, e: Eid) -> String {
        let mut owners = self.repo.path(e);
        owners.remove(0);
        let path: Vec<String> = owners.into_iter().map(|p| self.repo.name(p)).collect();
        format!("{} ({})", self.label_of(e), path.join("::"))
    }

    /// The operations a message can call: those its target lifeline's type inherits, through
    /// generalizations and interface realizations, then its own (uml-commands.js:2530-2536;
    /// elements.js:1215-1267, 1357-1378).
    pub fn message_operations(&self, m: Eid) -> Vec<Eid> {
        let Some(t) = self.message_target_type(m) else { return vec![] };
        let r = &self.repo;
        let generals = |c: Eid| -> Vec<Eid> {
            r.all()
                .filter(|g| matches!(r.ty(*g), "UMLGeneralization" | "UMLInterfaceRealization") && r.refd(*g, "source") == Some(c))
                .filter_map(|g| r.refd(g, "target"))
                .collect()
        };
        let mut ancestors: Vec<Eid> = generals(t);
        let mut i = 0;
        while i < ancestors.len() {
            for g in generals(ancestors[i]) {
                if !ancestors.contains(&g) && g != t {
                    ancestors.push(g);
                }
            }
            i += 1;
        }
        let mut out: Vec<Eid> = vec![];
        for e in ancestors.into_iter().chain(std::iter::once(t)) {
            for o in r.list(e, "operations") {
                if !out.contains(&o) {
                    out.push(o);
                }
            }
        }
        out
    }

    /// Select Operation on a message: the list picker over the operations of the target's type, or the
    /// alert "Lifeline should have a type." (uml-commands.js:2525-2546).
    pub fn select_message_operation(&mut self, m: Eid) {
        if self.message_target_type(m).is_none() {
            self.alert("Lifeline should have a type.");
            return;
        }
        let items = self.message_operations(m);
        self.list_picker = Some(ListPicker { title: "Select Operation".into(), items, chosen: None, action: ListAction::MessageOperation(m) });
    }

    /// Select Owner Attribute on an association end: the attributes of the classifier at the other
    /// end (uml-commands.js:1465-1476).
    pub fn select_owner_attribute(&mut self, end: Eid) {
        let items = self.owner_attribute_options(end);
        self.list_picker = Some(ListPicker { title: "Select Owner Attribute".into(), items, chosen: None, action: ListAction::OwnerAttribute(end) });
    }

    /// OK in the list picker: the chosen element, or none for "Do not specify".
    pub fn list_picker_ok(&mut self, p: ListPicker) {
        let chosen = p.chosen.and_then(|i| p.items.get(i)).copied().filter(|e| self.repo.alive(*e));
        match p.action {
            ListAction::ShowView(views) => {
                // "Do not specify" shows nothing
                if let Some(v) = p.chosen.and_then(|i| views.get(i)).copied().filter(|v| self.repo.alive(*v) && chosen.is_some()) {
                    self.show_view(v);
                }
            }
            ListAction::MessageOperation(m) if self.repo.alive(m) => {
                self.change(|a| a.repo.set_ref(m, "signature", chosen));
            }
            ListAction::OwnerAttribute(end) if self.repo.alive(end) => {
                self.change(|a| a.repo.set_ref(end, "ownerAttribute", chosen));
            }
            _ => {}
        }
    }

    /// The Element List Picker: one row per element with its icon, label and (path); "Do not specify"
    /// starts checked and a picked row unchecks it; double-click is OK (element-list-picker-dialog.js:73-131).
    pub(crate) fn list_picker_ui(&mut self, ctx: &egui::Context) {
        let Some(mut p) = self.list_picker.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        let title = p.title.clone();
        dialog(ctx, "list-picker", &title, |ui| {
            ui.set_min_width(320.0);
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for (i, e) in p.items.iter().enumerate() {
                    if !self.repo.alive(*e) {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        icons::draw(ui.painter(), r, icons::key_for(&self.repo, *e));
                        let resp = ui.selectable_label(p.chosen == Some(i), self.list_row_text(*e));
                        if resp.clicked() {
                            p.chosen = Some(i);
                        }
                        if resp.double_clicked() {
                            p.chosen = Some(i);
                            ok = true;
                        }
                    });
                }
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let mut none = p.chosen.is_none();
                if ui.checkbox(&mut none, "Do not specify").changed() && none {
                    p.chosen = None;
                }
                ui.add_space(12.0);
                let b = ui.button("OK");
                crate::ui::focus_primary(ui, &b);
                ok |= b.clicked();
                cancel = ui.button("Cancel").clicked();
            });
            ok |= ui.input(|i| i.key_pressed(egui::Key::Enter));
            cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if ok {
            self.list_picker = None;
            self.list_picker_ok(p);
        } else if cancel {
            self.list_picker = None;
        } else {
            self.list_picker = Some(p);
        }
    }
}
