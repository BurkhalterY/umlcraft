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
        crate::ui::set_light_theme(p.get("theme.default").and_then(|v| v.as_str()) == Some("light"));
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
        self.show_relationships = p.bool("view.relationships.visibility", false);
        self.show_thumbnails = p.bool("view.diagramThumbnails.visibility", false);
        self.show_minimap = p.bool("view.minimap.visibility", true);
        self.show_markdown = p.bool("view.markdown.visibility", false);
        for (id, _) in crate::splitter::PANELS {
            if let Some(s) = p.get(&format!("_viewState.{id}.size")).and_then(|v| v.as_f64()) {
                self.panel_sizes.insert(id.to_string(), s as f32);
            }
        }
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
            ("view.relationships.visibility", json!(self.show_relationships)),
            ("view.diagramThumbnails.visibility", json!(self.show_thumbnails)),
            ("view.minimap.visibility", json!(self.show_minimap)),
            ("view.markdown.visibility", json!(self.show_markdown)),
        ];
        for (k, v) in vals {
            self.prefs.set(k, v);
        }
        for (id, size) in &self.panel_sizes {
            self.prefs.set(&format!("_viewState.{id}.size"), json!(size.round()));
        }
        if let Some(ws) = self.window_state.clone() {
            self.prefs.set("windowState.main", ws);
        }
        self.prefs.flush();
    }

    /// File > Open Recent: most recent first, without duplicates, at most 10 (open-recent/main.js:39).
    pub fn recent_files(&self) -> Vec<PathBuf> {
        self.prefs.get("recent-files").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(PathBuf::from)).collect()).unwrap_or_default()
    }

    pub fn add_recent(&mut self, p: &std::path::Path) {
        let full = std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        let mut list = self.recent_files();
        list.retain(|x| *x != full);
        list.insert(0, full);
        list.truncate(10);
        self.set_recent(list);
    }

    pub fn set_recent(&mut self, list: Vec<PathBuf>) {
        let v: Vec<Value> = list.iter().map(|p| json!(p.display().to_string())).collect();
        self.prefs.set("recent-files", Value::Array(v));
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
        self.open_order = self.open.clone();
        self.diagram = current;
    }
}

/// How a preference is edited in the Preferences dialog.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Header,
    Check,
    Number,
    Text,
    Color,
    LineStyle,
    StereotypeDisplay,
    Theme,
}

/// The preferences UMLCraft uses, by section, with StarUML's keys, labels and defaults
/// (resources/default/preferences/default.json, extensions/essential/uml/preferences/uml.json).
pub fn items() -> Vec<(&'static str, &'static str, &'static str, Kind, Value)> {
    let mut v = vec![
        ("General", "", "General", Kind::Header, Value::Null),
        ("General", "general.working-file", "Auto load working file", Kind::Check, json!(false)),
        ("General", "general.auto-backup", "Auto backup", Kind::Check, json!(true)),
        ("General", "theme.default", "Default Theme", Kind::Theme, json!("dark")),
        ("General", "", "View", Kind::Header, Value::Null),
        ("General", "view.font", "Font Face", Kind::Text, json!("Arial")),
        ("General", "view.fontSize", "Font Size", Kind::Number, json!(13)),
        ("General", "view.fillColor", "Fill Color", Kind::Color, json!("#ffffff")),
        ("General", "view.lineColor", "Line Color", Kind::Color, json!("#000000")),
        ("General", "view.fontColor", "Font Color", Kind::Color, json!("#000000")),
        ("General", "view.lineStyle", "Line Style", Kind::LineStyle, json!(1)),
        ("General", "view.showShadow", "Show Shadow", Kind::Check, json!(true)),
        ("General", "", "Diagram Editor", Kind::Header, Value::Null),
        ("General", "diagramEditor.showGrid", "Show Grid", Kind::Check, json!(true)),
        ("General", "diagramEditor.snapToGrid", "Snap to Grid", Kind::Check, json!(true)),
        ("General", "diagramEditor.gridSize", "Grid Size", Kind::Number, json!(8)),
        ("General", "diagramEditor.backgroundColor", "Background Color", Kind::Color, json!("#ffffff")),
        ("General", "diagramEditor.gridColor", "Grid Color", Kind::Color, json!("#e0e0e0")),
        ("General", "", "Validation", Kind::Header, Value::Null),
        ("General", "validation.validateModel", "Validate Model", Kind::Check, json!(true)),
    ];
    // per element kind: fill colour (null = the general one), stereotype display and compartments
    let kinds: [(&str, &str, Option<(&str, bool, bool, Option<bool>)>); 15] = [
        ("Package", "package", None),
        ("Model", "model", None),
        ("Class", "class", Some(("label", false, false, None))),
        ("Interface", "interface", Some(("icon", true, true, None))),
        ("Signal", "signal", None),
        ("DataType", "datatype", None),
        ("PrimitiveType", "primitivetype", None),
        ("Enumeration", "enumeration", Some(("label", true, true, Some(false)))),
        ("Use Case", "usecase", Some(("label", true, true, None))),
        ("Actor", "actor", Some(("label", true, true, None))),
        ("Use Case Subject", "subsystem", None),
        ("Artifact", "artifact", Some(("icon", true, true, None))),
        ("Component", "component", Some(("label", true, true, None))),
        ("Node", "node", Some(("label", true, true, None))),
        ("Note", "note", None),
    ];
    for (label, id, more) in kinds {
        let key = |f: &str| -> &'static str { Box::leak(format!("uml.{id}.{f}").into_boxed_str()) };
        v.push(("UML", "", label, Kind::Header, Value::Null));
        v.push(("UML", key("fillColor"), "Fill Color", Kind::Color, Value::Null));
        if let Some((display, sa, so, sl)) = more {
            v.push(("UML", key("stereotypeDisplay"), "Stereotype Display", Kind::StereotypeDisplay, json!(display)));
            v.push(("UML", key("suppressAttributes"), "Suppress Attributes", Kind::Check, json!(sa)));
            v.push(("UML", key("suppressOperations"), "Suppress Operations", Kind::Check, json!(so)));
            if let Some(l) = sl {
                v.push(("UML", key("suppressLiterals"), "Suppress Literals", Kind::Check, json!(l)));
            }
        }
    }
    v
}

/// The preference id of a view's kind, for the per-kind UML preferences.
fn kind_id(view_ty: &str, model_ty: Option<&str>) -> Option<&'static str> {
    if view_ty == "UMLNoteView" {
        return Some("note");
    }
    Some(match model_ty? {
        "UMLPackage" => "package",
        "UMLModel" => "model",
        "UMLClass" => "class",
        "UMLInterface" => "interface",
        "UMLSignal" => "signal",
        "UMLDataType" => "datatype",
        "UMLPrimitiveType" => "primitivetype",
        "UMLEnumeration" => "enumeration",
        "UMLUseCase" => "usecase",
        "UMLActor" => "actor",
        "UMLUseCaseSubject" => "subsystem",
        "UMLArtifact" => "artifact",
        "UMLComponent" => "component",
        "UMLNode" => "node",
        _ => return None,
    })
}

impl App {
    /// A preference, or its default.
    pub fn pref(&self, key: &str) -> Value {
        if let Some(v) = self.prefs.get(key) {
            return v.clone();
        }
        items().into_iter().find(|i| i.1 == key).map(|i| i.4).unwrap_or(Value::Null)
    }

    pub fn grid_size(&self) -> f64 {
        self.pref("diagramEditor.gridSize").as_f64().filter(|g| *g >= 1.0).unwrap_or(8.0)
    }

    /// Give a new view the preferred style: font, colours, line style and shadow, then the per-kind fill
    /// colour, stereotype display and suppressed compartments (factory.js applies the same preferences).
    pub fn apply_view_defaults(&mut self, v: Eid) {
        if !self.repo.alive(v) {
            return;
        }
        let face = self.pref("view.font").as_str().unwrap_or("Arial").to_string();
        let size = self.pref("view.fontSize").as_f64().filter(|s| *s > 0.0).unwrap_or(13.0);
        let ty = self.repo.ty(v).to_string();
        let model_ty = self.repo.refd(v, "model").map(|m| self.repo.ty(m).to_string());
        let mut sets: Vec<(&str, Value)> = vec![];
        if (face.as_str(), size) != ("Arial", 13.0) {
            sets.push(("font", json!(crate::views::font_str(&face, size, 0))));
        }
        let kind = kind_id(&ty, model_ty.as_deref());
        let fill = kind.map(|k| self.pref(&format!("uml.{k}.fillColor"))).filter(|f| f.is_string()).unwrap_or_else(|| self.pref("view.fillColor"));
        for (field, v) in [("fillColor", fill), ("lineColor", self.pref("view.lineColor")), ("fontColor", self.pref("view.fontColor"))] {
            if v.is_string() {
                sets.push((field, v));
            }
        }
        sets.push(("showShadow", json!(self.pref("view.showShadow").as_bool().unwrap_or(true))));
        if crate::views::is_edge(&self.repo, v) {
            sets.push(("lineStyle", json!(self.pref("view.lineStyle").as_i64().unwrap_or(1))));
        }
        if let Some(k) = kind {
            for f in ["stereotypeDisplay", "suppressAttributes", "suppressOperations", "suppressLiterals"] {
                let p = self.pref(&format!("uml.{k}.{f}"));
                if !p.is_null() {
                    sets.push((f, p));
                }
            }
        }
        for (field, value) in sets {
            if crate::meta::order_of(&ty, field).is_some() {
                self.repo.set(v, field, value);
            }
        }
    }

    /// File > Preferences: sections on the left, their settings on the right; every change applies at
    /// once and there is only Close (preference-dialog.js:41-279).
    pub fn preferences_ui(&mut self, ctx: &egui::Context) {
        let Some(section) = self.prefs_open.clone() else { return };
        let mut open = true;
        let mut pick = None;
        // Esc closes, as the window's X does
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !egui::Popup::is_any_open(ctx) {
            open = false;
        }
        egui::Window::new("Preferences").open(&mut open).collapsible(false).resizable(true).default_size([620.0, 460.0]).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(120.0);
                    for s in ["General", "UML"] {
                        if ui.selectable_label(section == s, s).clicked() {
                            pick = Some(s.to_string());
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| ui.vertical(|ui| {
                    egui::Grid::new("prefs-grid").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
                        for (sec, key, label, kind, def) in items() {
                            if sec != section {
                                continue;
                            }
                            if kind == Kind::Header {
                                ui.label(egui::RichText::new(label).strong());
                                ui.end_row();
                                continue;
                            }
                            ui.label(label);
                            let cur = self.prefs.get(key).cloned().unwrap_or(def.clone());
                            if let Some(new) = pref_widget(ui, key, kind, &cur) {
                                self.prefs.set(key, new);
                            }
                            ui.end_row();
                        }
                    });
                    ui.add_space(8.0);
                    if ui.button("Restore Default Settings").clicked() {
                        for (sec, key, ..) in items() {
                            if sec == section && !key.is_empty() {
                                self.prefs.values.remove(key);
                            }
                        }
                    }
                }));
            });
        });
        // Show Grid and Snap to Grid are the same settings as the View menu's
        self.show_grid = self.pref("diagramEditor.showGrid").as_bool().unwrap_or(true);
        self.snap = self.pref("diagramEditor.snapToGrid").as_bool().unwrap_or(true);
        self.prefs_open = if open { Some(pick.unwrap_or(section)) } else { None };
    }
}

fn pref_widget(ui: &mut egui::Ui, key: &str, kind: Kind, cur: &Value) -> Option<Value> {
    match kind {
        Kind::Check => {
            let mut b = cur.as_bool().unwrap_or(false);
            ui.checkbox(&mut b, "").changed().then(|| json!(b))
        }
        Kind::Number => {
            let mut n = cur.as_f64().unwrap_or(0.0);
            ui.add(egui::DragValue::new(&mut n).range(1.0..=200.0).speed(0.2)).changed().then(|| json!(n.round() as i64))
        }
        Kind::Text => {
            let mut t = cur.as_str().unwrap_or("").to_string();
            ui.add(egui::TextEdit::singleline(&mut t).desired_width(160.0)).changed().then(|| json!(t))
        }
        Kind::Color => {
            let c = cur.as_str().map(|s| crate::render::parse_color(s, crate::render::Rgba(255, 255, 255, 255)));
            let mut col = c.map(|c| egui::Color32::from_rgb(c.0, c.1, c.2)).unwrap_or(egui::Color32::TRANSPARENT);
            let mut out = None;
            ui.horizontal(|ui| {
                if egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::Opaque).changed() {
                    out = Some(json!(crate::style::hex(crate::render::Rgba(col.r(), col.g(), col.b(), 255))));
                }
                // a per-kind colour can go back to "use the general colour"
                if key.starts_with("uml.") && cur.is_string() && ui.small_button("Default").clicked() {
                    out = Some(Value::Null);
                }
            });
            out
        }
        Kind::LineStyle => {
            let mut s = cur.as_i64().unwrap_or(1);
            let names = ["Rectilinear", "Oblique", "Rounded Rectilinear", "Curve"];
            let before = s;
            egui::ComboBox::from_id_salt(key).selected_text(names[s.clamp(0, 3) as usize]).show_ui(ui, |ui| {
                for (i, n) in names.iter().enumerate() {
                    ui.selectable_value(&mut s, i as i64, *n);
                }
            });
            (s != before).then(|| json!(s))
        }
        Kind::StereotypeDisplay => {
            let mut s = cur.as_str().unwrap_or("label").to_string();
            let before = s.clone();
            let shown = crate::style::STEREOTYPE_DISPLAYS.iter().find(|d| d.0 == s).map(|d| d.1).unwrap_or("");
            egui::ComboBox::from_id_salt(key).selected_text(shown).show_ui(ui, |ui| {
                for (v, l) in crate::style::STEREOTYPE_DISPLAYS {
                    ui.selectable_value(&mut s, v.to_string(), l);
                }
            });
            (s != before).then(|| json!(s))
        }
        Kind::Theme => {
            let mut s = cur.as_str().unwrap_or("dark").to_string();
            let before = s.clone();
            egui::ComboBox::from_id_salt(key).selected_text(if s == "light" { "Light" } else { "Dark" }).show_ui(ui, |ui| {
                ui.selectable_value(&mut s, "dark".to_string(), "Dark");
                ui.selectable_value(&mut s, "light".to_string(), "Light");
            });
            (s != before).then(|| json!(s))
        }
        Kind::Header => None,
    }
}

/// Auto-backup every 10 minutes and crash recovery (app-context.js:91, 697-708, 809-811, 1533-1553).
pub const BACKUP_EVERY: std::time::Duration = std::time::Duration::from_secs(600);

impl App {
    fn backup_path(&self) -> Option<PathBuf> {
        self.prefs.path.as_ref().and_then(|p| p.parent()).map(|d| d.join("__backup.mdj"))
    }

    /// Write the backup when it is due and the preference is on, and remember where it is.
    pub fn auto_backup(&mut self, now: std::time::Instant) {
        if now.duration_since(self.last_backup) < BACKUP_EVERY {
            return;
        }
        self.last_backup = now;
        if !self.pref("general.auto-backup").as_bool().unwrap_or(true) {
            return;
        }
        let Some(p) = self.backup_path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if std::fs::write(&p, self.repo.serialize()).is_ok() {
            self.prefs.set("__backup_filename", json!(p.display().to_string()));
            self.prefs.flush();
        }
    }

    /// A clean exit: the backup is not offered at the next launch.
    pub fn clear_backup(&mut self) {
        if let Some(p) = self.prefs.get("__backup_filename").and_then(|v| v.as_str()).map(PathBuf::from) {
            let _ = std::fs::remove_file(p);
        }
        self.prefs.values.remove("__backup_filename");
        self.prefs.flush();
    }

    /// At launch: after an abnormal exit the backup is loaded as a modified, untitled project and then
    /// deleted, with the toast "App was unexpectedly terminated. Auto backup file has loaded.". Returns
    /// true when it was loaded.
    pub fn recover_backup(&mut self) -> bool {
        let Some(p) = self.prefs.get("__backup_filename").and_then(|v| v.as_str()).map(PathBuf::from) else { return false };
        let loaded = std::fs::read_to_string(&p).ok().and_then(|t| crate::model::Repo::parse(&t).ok());
        let _ = std::fs::remove_file(&p);
        self.prefs.values.remove("__backup_filename");
        self.prefs.flush();
        let Some(repo) = loaded else { return false };
        self.set_project(repo, None);
        self.modified = true;
        self.toast("App was unexpectedly terminated. Auto backup file has loaded.");
        true
    }

    /// The working file: the last project opened from a file, kept as `__working_filename`; New
    /// forgets it (app-context.js:979-982, default-commands.js:114).
    pub fn working_file(&self) -> Option<PathBuf> {
        self.prefs.get("__working_filename").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(PathBuf::from)
    }

    pub fn set_working_file(&mut self, p: Option<&std::path::Path>) {
        let v = p.map(|p| json!(std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()).display().to_string())).unwrap_or(Value::Null);
        self.prefs.set("__working_filename", v);
        self.prefs.flush();
    }

    /// "Auto load working file": with no file given, the working file opens at launch
    /// (app-context.js:688-728).
    pub fn auto_load_working_file(&mut self) {
        if !self.pref("general.working-file").as_bool().unwrap_or(false) {
            return;
        }
        if let Some(f) = self.working_file().filter(|f| f.exists()) {
            self.open_path(f);
        }
    }
}
