//! File > Export > Fragment and File > Import > Fragment: one element and everything it owns, as a
//! `.mfj` file (default-commands.js:242-338, project-manager.js:225-308).

use crate::app::App;
use crate::model::Eid;
use std::path::Path;

impl App {
    /// The `.mfj` text of an element: its subtree as tab-indented JSON, like a project file.
    pub fn fragment_text(&self, e: Eid) -> String {
        let v = self.repo.export(e);
        let mut buf = Vec::new();
        let fmt = serde_json::ser::PrettyFormatter::with_indent(b"\t");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
        serde::Serialize::serialize(&v, &mut ser).ok();
        String::from_utf8(buf).unwrap_or_default()
    }

    /// Add a fragment to the project's owned elements, with new ids; one undo step.
    pub fn import_fragment(&mut self, path: &Path) -> Result<Eid, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if v.get("_type").and_then(|t| t.as_str()).is_none() {
            return Err("This is not a model fragment.".into());
        }
        let root = self.repo.root;
        let made = self.change(|a| a.repo.insert_copies(vec![v], root, "ownedElements"));
        let e = *made.first().ok_or("This is not a model fragment.")?;
        self.reveal(e);
        Ok(e)
    }

    /// File > Import > Fragment...: pick a `.mfj` file.
    pub fn import_fragment_dialog(&mut self) {
        let Some(p) = rfd::FileDialog::new().set_title("Import Model Fragment").add_filter("Model Fragments", &["mfj"]).add_filter("All Files", &["*"]).pick_file() else { return };
        if let Err(e) = self.import_fragment(&p) {
            self.message = Some(("Failed to import the fragment.".into(), e));
        }
    }

    /// File > Export > Fragment...: the selected element, or one picked in the Element Picker when
    /// nothing is selected; then a Save dialog with `Fragment.mfj`.
    pub fn export_fragment_dialog(&mut self) {
        match self.focus.filter(|f| self.repo.alive(*f) && !self.is_view(*f) && *f != self.repo.root) {
            Some(e) => self.save_fragment(e),
            None => {
                self.open_element_picker(self.repo.root, "");
                if let Some(p) = self.element_picker.as_mut() {
                    p.ty = "Model";
                    p.export_fragment = true;
                    p.chosen = None;
                }
            }
        }
    }

    pub fn save_fragment(&mut self, e: Eid) {
        let Some(mut p) = rfd::FileDialog::new().set_title("Export Model Fragment").set_file_name("Fragment.mfj").add_filter("Model Fragments", &["mfj"]).save_file() else { return };
        if p.extension().is_none() {
            p.set_extension("mfj");
        }
        if let Err(err) = std::fs::write(&p, self.fragment_text(e)) {
            self.message = Some(("Failed to export the fragment.".into(), err.to_string()));
        }
    }
}
