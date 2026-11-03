//! Preferences kept between runs, in a JSON file in the user's configuration folder, as StarUML keeps
//! its preferences (core/preference-manager.js). Keys follow the original's names where there is one:
//! `diagramEditor.showGrid`, `explorer.sortBy`, `_workingDiagrams.<path>`, `_viewState.<panel>`…

use crate::app::App;
use crate::model::Eid;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

/// Where the preferences file lives: the system's per-user configuration folder.
pub fn default_path() -> Option<PathBuf> {
    let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = if cfg!(target_os = "windows") {
        env("APPDATA")
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support"))
    } else {
        env("XDG_CONFIG_HOME").or_else(|| env("HOME").map(|h| h.join(".config")))
    };
    base.map(|b| b.join("UMLCraft").join("preferences.json"))
}

#[derive(Default)]
pub struct Prefs {
    pub path: Option<PathBuf>,
    pub values: Map<String, Value>,
    /// What was last written, to write only on change.
    written: String,
}

impl Prefs {
    pub fn load(path: Option<PathBuf>) -> Prefs {
        let values = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        let written = Value::Object(values.clone()).to_string();
        Prefs { path, values, written }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn bool(&self, key: &str, def: bool) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(def)
    }

    pub fn set(&mut self, key: &str, v: Value) {
        self.values.insert(key.to_string(), v);
    }

    /// Write the file when something changed (and there is a file).
    pub fn flush(&mut self) {
        let text = Value::Object(self.values.clone()).to_string();
        if text == self.written {
            return;
        }
        let Some(p) = &self.path else {
            self.written = text;
            return;
        };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let pretty = serde_json::to_string_pretty(&Value::Object(self.values.clone())).unwrap_or_default();
        if std::fs::write(p, pretty).is_ok() {
            self.written = text;
        }
    }
}

impl App {
    /// Take the remembered settings (at start-up).
    pub fn apply_prefs(&mut self) {
        let p = &self.prefs;
        self.show_grid = p.bool("diagramEditor.showGrid", true);
        self.snap = p.bool("diagramEditor.snapToGrid", true);
        self.sort_by_name = p.get("explorer.sortBy").and_then(|v| v.as_str()) == Some("name");
        self.show_stereotype = p.bool("explorer.showStereotype", true);
        self.show_sidebar = p.bool("_viewState.sidebar.visible", true);
        self.show_navigator = p.bool("_viewState.navigator.visible", true);
        self.show_statusbar = p.bool("_viewState.statusbar.visible", true);
        self.show_toolbar = p.bool("_viewState.toolbar.visible", true);
        self.show_toolbox = p.bool("_viewState.toolbox.visible", true);
        self.show_editors = p.bool("_viewState.editors.visible", true);
        self.props_open = p.bool("_viewState.propertyEditor.visible", true);
        self.doc_open = p.bool("_viewState.documentationEditor.visible", true);
        self.style_open = p.bool("_viewState.styleEditor.visible", true);
    }

    /// Record the current settings; the file is written only when they changed.
    pub fn store_prefs(&mut self) {
        let vals = [
            ("diagramEditor.showGrid", json!(self.show_grid)),
            ("diagramEditor.snapToGrid", json!(self.snap)),
            ("explorer.sortBy", json!(if self.sort_by_name { "name" } else { "added" })),
            ("explorer.showStereotype", json!(self.show_stereotype)),
            ("_viewState.sidebar.visible", json!(self.show_sidebar)),
            ("_viewState.navigator.visible", json!(self.show_navigator)),
            ("_viewState.statusbar.visible", json!(self.show_statusbar)),
            ("_viewState.toolbar.visible", json!(self.show_toolbar)),
            ("_viewState.toolbox.visible", json!(self.show_toolbox)),
            ("_viewState.editors.visible", json!(self.show_editors)),
            ("_viewState.propertyEditor.visible", json!(self.props_open)),
            ("_viewState.documentationEditor.visible", json!(self.doc_open)),
            ("_viewState.styleEditor.visible", json!(self.style_open)),
        ];
        for (k, v) in vals {
            self.prefs.set(k, v);
        }
        self.prefs.flush();
    }

    fn working_key(&self) -> Option<String> {
        let f = self.filename.as_ref()?;
        let full = std::fs::canonicalize(f).unwrap_or_else(|_| f.clone());
        Some(format!("_workingDiagrams.{}", full.display()))
    }

    /// Remember the open diagrams of this file and the current one (diagram-manager.js:152-169).
    pub fn save_working_diagrams(&mut self) {
        let Some(key) = self.working_key() else { return };
        let ids: Vec<String> = self.open.iter().filter(|d| self.repo.alive(**d)).map(|d| self.repo.id(*d)).collect();
        let current = self.diagram.filter(|d| self.repo.alive(*d)).map(|d| self.repo.id(d));
        self.prefs.set(&key, json!({ "diagrams": ids, "current": current }));
        self.prefs.flush();
    }

    /// Reopen the diagrams that were open when this file was last saved or closed (diagram-manager.js:170-196).
    pub fn restore_working_diagrams(&mut self) {
        let Some(key) = self.working_key() else { return };
        let Some(saved) = self.prefs.get(&key).cloned() else { return };
        let find = |a: &App, id: &str| -> Option<Eid> { a.repo.by_id.get(id).copied().filter(|e| a.repo.alive(*e) && a.is_diagram(*e)) };
        let ids: Vec<String> = saved.get("diagrams").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
        let mut open: Vec<Eid> = ids.iter().filter_map(|id| find(self, id)).collect();
        for d in self.open.clone() {
            if !open.contains(&d) {
                open.push(d);
            }
        }
        if open.is_empty() {
            return;
        }
        let current = saved.get("current").and_then(|v| v.as_str()).and_then(|id| find(self, id)).or(open.first().copied());
        self.open = open;
        self.diagram = current;
    }
}
