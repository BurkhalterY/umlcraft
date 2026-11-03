//! The input dialog: a message and one line of text, pre-filled and selected; Enter is OK and Esc
//! cancels (dialogs/dialog-manager.js:366-401). Working Diagrams › Rename uses it.

use crate::app::App;
use crate::model::Eid;
use egui::{Align2, Vec2};
use serde_json::json;

/// What OK does with the text.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InputAction {
    RenameDiagram(Eid),
}

#[derive(Clone, Debug)]
pub struct InputDialog {
    pub message: String,
    pub text: String,
    pub action: InputAction,
    pub fresh: bool,
}

impl App {
    /// View › Rename Diagram (Working Diagrams menu): "Enter diagram name", for the current diagram
    /// (default-commands.js:1117-1127).
    pub fn rename_diagram(&mut self) {
        let Some(d) = self.diagram else { return };
        self.input_dialog = Some(InputDialog { message: "Enter diagram name".into(), text: self.repo.name(d), action: InputAction::RenameDiagram(d), fresh: true });
    }

    /// Run the dialog's action with the entered text.
    pub fn apply_input(&mut self, action: InputAction, text: String) {
        match action {
            InputAction::RenameDiagram(d) => {
                if self.repo.alive(d) && self.repo.name(d) != text {
                    self.change(|a| a.repo.set(d, "name", json!(text)));
                }
            }
        }
    }

    pub fn input_dialog_ui(&mut self, ctx: &egui::Context) {
        let Some(mut dlg) = self.input_dialog.clone() else { return };
        let (mut ok, mut cancel) = (false, false);
        egui::Window::new("UMLCraft").title_bar(false).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(320.0);
            ui.label(&dlg.message);
            let out = egui::TextEdit::singleline(&mut dlg.text).desired_width(f32::INFINITY).show(ui);
            if dlg.fresh {
                // pre-filled and selected
                out.response.request_focus();
                let mut state = out.state.clone();
                let all = egui::text::CCursorRange::two(egui::text::CCursor::new(0), egui::text::CCursor::new(dlg.text.chars().count()));
                state.cursor.set_char_range(Some(all));
                state.store(ui.ctx(), out.response.id);
                dlg.fresh = false;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ok = true;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                cancel = true;
            }
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    ok = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
        });
        if ok {
            self.input_dialog = None;
            self.apply_input(dlg.action, dlg.text);
        } else if cancel {
            self.input_dialog = None;
        } else {
            self.input_dialog = Some(dlg);
        }
    }
}
