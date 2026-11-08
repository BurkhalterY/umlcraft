//! Model > Find (Ctrl+F): a dialog, then a "Find Results" panel under the diagram (extensions/default/find).

use crate::app::App;
use crate::model::Eid;
use egui::Vec2;

/// The Find dialog's fields.
#[derive(Clone, Debug, Default)]
pub struct FindDialog {
    pub keyword: String,
    pub case_sensitive: bool,
    pub documentation: bool,
    pub fresh: bool,
}

impl App {
    /// Every model element (not views) whose name, or name and documentation, contains the keyword;
    /// case-insensitive unless asked; an empty keyword finds nothing (find/main.js:79).
    pub fn find(&self, keyword: &str, case_sensitive: bool, documentation: bool) -> Vec<Eid> {
        if keyword.trim().is_empty() {
            return vec![];
        }
        let norm = |s: String| if case_sensitive { s } else { s.to_lowercase() };
        let k = norm(keyword.to_string());
        self.repo
            .all()
            .filter(|e| !self.is_view(*e) && self.repo.kind_of(*e, "Model"))
            .filter(|e| {
                let mut text = self.repo.name(*e);
                if documentation {
                    text = format!("{text}\n{}", self.repo.str(*e, "documentation"));
                }
                norm(text).contains(&k)
            })
            .collect()
    }

    /// The full path name, owners joined by "::" (Element.getPathname).
    pub fn path_name(&self, e: Eid) -> String {
        self.repo.path(e).into_iter().skip(1).map(|x| self.label_of(x)).collect::<Vec<_>>().join("::")
    }

    /// Open the Find dialog with the last keyword and options.
    pub fn open_find(&mut self) {
        self.find_dialog = Some(FindDialog { fresh: true, ..self.find_last.clone() });
    }

    /// The Find dialog: keyword, "Case sensitive", "Find in documentation"; Enter is OK (find/main.js:46).
    pub fn find_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut fd) = self.find_dialog.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        crate::dialogs::dialog(ctx, "find", "Find", |ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut fd.keyword).hint_text("Keyword").desired_width(260.0));
            if fd.fresh {
                r.request_focus();
                fd.fresh = false;
            }
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ok = true;
            }
            ui.checkbox(&mut fd.case_sensitive, "Case sensitive");
            ui.checkbox(&mut fd.documentation, "Find in documentation");
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
            let found = self.find(&fd.keyword, fd.case_sensitive, fd.documentation);
            self.find_results = Some((fd.keyword.clone(), found));
        }
        if ok || cancel {
            self.find_last = fd;
            self.find_dialog = None;
        } else {
            self.find_dialog = Some(fd);
        }
    }

    /// The Find Results panel: `Find Results "<keyword>" — N elements.`, one row per element with its icon
    /// and full path; a click selects and reveals it in the explorer; × closes the panel (find/main.js:114-143).
    pub fn find_results_ui(&mut self, ui: &mut egui::Ui) {
        let Some((kw, found)) = self.find_results.clone() else { return };
        let mut close = false;
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(format!("Find Results \"{kw}\" — {} elements.", found.len())).size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("×").clicked() {
                    close = true;
                }
            });
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            let alive: Vec<Eid> = found.iter().copied().filter(|e| self.repo.alive(*e)).collect();
            for e in alive {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::hover());
                    crate::icons::draw(ui.painter(), r, crate::icons::key_for(&self.repo, e));
                    if ui.selectable_label(self.focus == Some(e), self.path_name(e)).clicked() {
                        self.focus = Some(e);
                        self.reveal(e);
                    }
                });
            }
        });
        if close {
            self.find_results = None;
        }
    }
}
