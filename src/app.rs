//! Project state, undo/redo, open/save and model operations.

use crate::meta;
use crate::model::{Eid, Repo};
use crate::views::{self, Info, Measure, P, R};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub const ZOOM_MIN: f64 = 0.1; // diagram-editor.js:2396
pub const ZOOM_MAX: f64 = 3.0;
pub const ZOOM_STEP: f64 = 0.1;
pub const UNDO_LIMIT: usize = 100; // repository.js:37
pub const DOCUMENT_VERSION: i64 = 1; // core.js:57

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AssocInit {
    Plain,
    Directed,
    Shared,
    Composite,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tool {
    pub label: &'static str,
    pub kind: &'static str,
    pub icon: &'static str,
    pub line: bool,
    pub init: AssocInit,
}

#[derive(Clone)]
pub struct Snap {
    repo: Repo,
    diagram: Option<Eid>,
    open: Vec<Eid>,
    selection: Vec<Eid>,
    focus: Option<Eid>,
}

#[derive(Clone, Debug)]
pub enum Drag {
    Pan { start: egui_pos::Pos, scroll: P },
    Mini,
    Rubber { start: P, now: P, add: bool, base: Vec<Eid> },
    CreateNode { start: P, now: P },
    CreateEdge { start: P, now: P, tail: Eid },
    /// `collapse`: the pressed member of a multi-selection, which alone stays selected if the mouse
    /// is released without moving.
    Move { start: P, orig: Vec<(Eid, P)>, edges: Vec<(Eid, Vec<P>)>, lead: Option<Eid>, changed: bool, collapse: Option<Eid> },
    Resize { handle: usize, start: P, orig: R, view: Eid, changed: bool },
    Point { edge: Eid, index: usize, insert: Option<P>, changed: bool },
    /// Dragging a line's label: its centre goes from `orig` by the mouse delta.
    Label { label: Eid, edge: Eid, start: P, orig: P, changed: bool },
    /// Dragging the first (`tail`) or last point of a line to reconnect it.
    Reconnect { edge: Eid, tail: bool, now: P },
    /// Dragging a segment of a rectilinear line across: points `index` and `index + 1` move together.
    Segment { edge: Eid, index: usize, start: P, orig: Vec<P>, horizontal: bool, changed: bool },
}

pub mod egui_pos {
    #[derive(Clone, Copy, Debug)]
    pub struct Pos {
        pub x: f32,
        pub y: f32,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EditKind {
    Name(Eid),
    Text(Eid),
    Feature(Eid),
}

#[derive(Clone, Debug)]
pub struct Editing {
    pub kind: EditKind,
    pub text: String,
    pub rect: R,
    pub multiline: bool,
    pub center: bool,
    pub fresh: bool,
    /// The box the edit belongs to (a class view for its name and its compartment items).
    pub host: Option<Eid>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Area {
    Canvas,
    Explorer,
    Other,
}

pub struct App {
    pub repo: Repo,
    pub filename: Option<PathBuf>,
    pub modified: bool,
    pub diagram: Option<Eid>,
    pub open: Vec<Eid>,
    pub selection: Vec<Eid>,
    pub focus: Option<Eid>,
    pub zoom: f64,
    pub scroll: P,
    pub show_grid: bool,
    pub snap: bool,
    pub tool: Option<Tool>,
    pub tool_sticky: bool,
    pub undo: Vec<Snap>,
    pub redo: Vec<Snap>,
    pub drag: Option<Drag>,
    pub hover: Option<Eid>,
    pub editing: Option<Editing>,
    pub info: HashMap<Eid, Info>,
    pub expanded: HashSet<Eid>,
    pub search: String,
    pub sort_by_name: bool,
    /// Tree labels show «stereotype» before the name (explorer.showStereotype, default on).
    pub show_stereotype: bool,
    pub open_groups: HashSet<&'static str>,
    /// The diagram type the toolbox groups were last opened for.
    pub toolbox_for: Option<String>,
    pub prop_buf: HashMap<String, String>,
    pub prop_for: Option<Eid>,
    /// The elements the Properties section shows when several are selected.
    pub prop_set: Vec<Eid>,
    pub doc_buf: String,
    pub props_open: bool,
    pub style_open: bool,
    pub doc_open: bool,
    pub last_area: Area,
    pub ctx_target: Option<Eid>,
    pub ctx_pos: P,
    pub about_open: bool,
    pub message: Option<(String, String)>,
    pub show_sidebar: bool,
    pub show_navigator: bool,
    pub show_statusbar: bool,
    pub show_toolbar: bool,
    pub show_toolbox: bool,
    pub show_editors: bool,
    pub canvas_rect: Option<egui::Rect>,
    pub screenshot: Option<PathBuf>,
    pub frames: u32,
    pub started: std::time::Instant,
    pub shot_requested: bool,
    pub quit_after_save: bool,
    pub pending_edit: Option<Eid>,
    /// A compartment item to edit inline once its row is laid out: (box view, item, frames left).
    pub pending_item: Option<(Eid, Eid, u8)>,
    pub reveal_pending: Option<Eid>,
    /// Scroll origin of each diagram, kept while the project is open (the original's `_originX/Y`).
    pub scrolls: HashMap<Eid, P>,
    /// A view to centre on once the canvas size is known.
    pub centre_pending: Option<Eid>,
    /// Short notices shown at the bottom of the diagram area, with the time they appeared.
    /// Toasts: text, when shown, and whether it is an error (red) rather than information.
    pub toasts: Vec<(String, std::time::Instant, bool)>,
    /// Select In Diagram with several diagrams: the views to choose from and the chosen row.
    pub diagram_picker: Option<(Vec<Eid>, Option<usize>)>,
    /// The last text UMLCraft put on the clipboard, used when the system clipboard cannot be read.
    pub clipboard: Option<String>,
    /// The system clipboard, kept open: on Linux an image copied to it is served while it lives.
    pub sys_clipboard: Option<arboard::Clipboard>,
    /// The colour field last changed and when, so a picker drag is one undo step.
    pub style_edit: Option<(String, std::time::Instant)>,
    /// The Color dialog, opened by Ctrl+Shift+I, Ctrl+Shift+L and the Style editor's swatches.
    pub color_dialog: Option<crate::style::ColorDialog>,
    /// The Font window (Ctrl+Shift+F): face and size as typed, and the text colour.
    pub font_dialog: Option<(String, String, Option<crate::render::Rgba>)>,
    pub prefs: crate::prefs::Prefs,
    /// The Preferences window, open on this section.
    pub prefs_open: Option<String>,
    /// The Tag Editor: the element, the selected row, and the Tag dialog being edited.
    pub tag_editor: Option<Eid>,
    pub tag_row: Option<crate::tags::Row>,
    pub tag_edit: Option<crate::tags::TagEdit>,
    /// The Command Palette or Quick Find, while open, and a picked command to run next frame.
    pub palette: Option<crate::palette::Palette>,
    pub palette_run: Option<&'static str>,
    /// Model > Find: the open dialog, the last options, and the results panel (keyword, elements).
    pub find_dialog: Option<crate::find::FindDialog>,
    pub find_last: crate::find::FindDialog,
    pub find_results: Option<(String, Vec<Eid>)>,
    /// Validation: the problems found, whether the model was validated, and the results panel.
    pub problems: Vec<crate::validate::Problem>,
    pub validated: bool,
    pub show_validation: bool,
    /// Panel sizes changed by dragging a splitter (see splitter.rs).
    pub panel_sizes: std::collections::HashMap<String, f32>,
    /// The splitter being dragged: its panel, the size it started at and how far it has moved.
    pub split_drag: Option<(&'static str, f32, f32)>,
    /// View > Relationships: the panel under the diagram (view.relationships.visibility).
    pub show_relationships: bool,
    /// View > Diagram Thumbnails: the panel under the diagram (view.diagramThumbnails.visibility).
    pub show_thumbnails: bool,
    /// View > Minimap: the overview in the canvas corner (view.minimap.visibility, on by default).
    pub show_minimap: bool,
    /// View > Markdown Documentation: the panel (view.markdown.visibility), its mode and its text.
    pub show_markdown: bool,
    /// The open Mermaid dialog's code.
    pub mermaid_code: Option<String>,
    /// The theme the style was last set for (true: light).
    pub theme_applied: Option<bool>,
    /// The element being renamed in the Model Explorer: its text, and whether the edit just opened.
    pub tree_rename: Option<(Eid, String, bool)>,
    /// The user's keymap.json bindings.
    pub user_keymap: Vec<(crate::keymap::Stroke, String)>,
    /// Requests from the local HTTP API, answered each frame.
    pub api_rx: Option<std::sync::mpsc::Receiver<crate::api::ApiRequest>>,
    /// The window's bounds and maximized state, kept as windowState.main.
    pub window_state: Option<serde_json::Value>,
    pub md_preview: bool,
    pub md_buf: String,
    pub md_for: Option<Eid>,
    pub md_editing: bool,
    /// Give the keyboard back to the canvas on the next frame (after an inline edit closes).
    pub refocus_canvas: bool,
    /// The selection as the canvas last saw it, to notice changes.
    pub last_selection: Vec<Eid>,
    /// When the last auto-backup was written.
    pub last_backup: std::time::Instant,
    /// The Print to PDF dialog is open.
    pub print_open: bool,
    /// The open Element Picker ("Select Element").
    pub element_picker: Option<crate::picker::ElementPicker>,
    /// The open input dialog.
    pub input_dialog: Option<crate::input::InputDialog>,
}

impl App {
    pub fn new(screenshot: Option<PathBuf>) -> App {
        let mut a = App {
            repo: Repo::blank(),
            filename: None,
            modified: false,
            diagram: None,
            open: vec![],
            selection: vec![],
            focus: None,
            zoom: 1.0,
            scroll: P::default(),
            show_grid: true,
            snap: true,
            tool: None,
            tool_sticky: false,
            undo: vec![],
            redo: vec![],
            drag: None,
            hover: None,
            editing: None,
            info: HashMap::new(),
            expanded: HashSet::new(),
            search: String::new(),
            sort_by_name: false,
            show_stereotype: true,
            open_groups: HashSet::new(),
            toolbox_for: None,
            prop_buf: HashMap::new(),
            prop_for: None,
            prop_set: vec![],
            doc_buf: String::new(),
            props_open: true,
            style_open: true,
            doc_open: true,
            last_area: Area::Other,
            ctx_target: None,
            ctx_pos: P::default(),
            about_open: false,
            message: None,
            show_sidebar: true,
            show_navigator: true,
            show_statusbar: true,
            show_toolbar: true,
            show_toolbox: true,
            show_editors: true,
            canvas_rect: None,
            screenshot,
            frames: 0,
            started: std::time::Instant::now(),
            shot_requested: false,
            quit_after_save: false,
            pending_edit: None,
            pending_item: None,
            reveal_pending: None,
            scrolls: HashMap::new(),
            centre_pending: None,
            toasts: vec![],
            diagram_picker: None,
            clipboard: None,
            sys_clipboard: None,
            style_edit: None,
            color_dialog: None,
            font_dialog: None,
            prefs: crate::prefs::Prefs::default(),
            prefs_open: None,
            tag_editor: None,
            tag_row: None,
            tag_edit: None,
            palette: None,
            palette_run: None,
            find_dialog: None,
            find_last: Default::default(),
            find_results: None,
            problems: vec![],
            validated: false,
            show_validation: false,
            panel_sizes: Default::default(),
            split_drag: None,
            show_relationships: false,
            show_thumbnails: false,
            show_minimap: true,
            show_markdown: false,
            mermaid_code: None,
            theme_applied: None,
            tree_rename: None,
            user_keymap: vec![],
            api_rx: None,
            window_state: None,
            md_preview: false,
            md_buf: String::new(),
            md_for: None,
            md_editing: false,
            refocus_canvas: false,
            last_selection: vec![],
            last_backup: std::time::Instant::now(),
            print_open: false,
            element_picker: None,
            input_dialog: None,
        };
        a.set_project(Repo::blank(), None);
        a
    }

    // ------------------------------------------------------------ project
    pub fn set_project(&mut self, repo: Repo, filename: Option<PathBuf>) {
        self.repo = repo;
        self.migrate();
        self.filename = filename;
        self.modified = false;
        self.undo.clear();
        self.redo.clear();
        self.selection.clear();
        self.tool = None;
        self.drag = None;
        self.editing = None;
        self.info.clear();
        // every default diagram opens and the first becomes current; with none, the first diagram (app-context.js:987-999)
        let diagrams = self.repo.all_of("Diagram");
        let defaults: Vec<Eid> = diagrams.iter().copied().filter(|d| self.repo.bool(*d, "defaultDiagram")).collect();
        self.open = if defaults.is_empty() { diagrams.first().copied().into_iter().collect() } else { defaults };
        self.diagram = self.open.first().copied();
        // the project is selected in the explorer (app-context.js:680-682)
        self.focus = Some(self.repo.root);
        self.zoom = 1.0;
        self.scroll = P::default();
        self.scrolls.clear();
        // closing a project clears the validation errors (validation-panel.js:136-142)
        self.problems.clear();
        self.validated = false;
        self.centre_pending = None;
        self.diagram_picker = None;
        self.expanded.clear();
        let root = self.repo.root;
        self.expanded.insert(root);
        for (_, c) in self.repo.children(root) {
            self.expanded.insert(c);
        }
    }

    /// Bring a project older than documentVersion 1 up to date: every use case subject goes to the
    /// back of its diagram (document-compatibility-resolver.js:25-41).
    fn migrate(&mut self) {
        let root = self.repo.root;
        if self.repo.ty(root) != "Project" || self.repo.f(root, "documentVersion") >= DOCUMENT_VERSION as f64 {
            return;
        }
        for d in self.repo.all_of("Diagram") {
            let views = self.repo.list(d, "ownedViews");
            let (back, front): (Vec<Eid>, Vec<Eid>) = views.iter().partition(|v| self.repo.ty(**v) == "UMLUseCaseSubjectView");
            if !back.is_empty() {
                self.repo.set_list(d, "ownedViews", back.into_iter().chain(front).collect());
            }
        }
        self.repo.set(root, "documentVersion", json!(DOCUMENT_VERSION));
    }

    /// The window title: "• " when modified, "<file name> — " when there is a file, then the app name
    /// (titlebar-view.js:34-47).
    pub fn title(&self) -> String {
        let base = self.filename.as_ref().and_then(|p| p.file_name()).map(|s| format!("{} — ", s.to_string_lossy())).unwrap_or_default();
        format!("{}{base}UMLCraft", if self.modified { "• " } else { "" })
    }

    pub fn open_path(&mut self, p: PathBuf) {
        match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| Repo::parse(&t)) {
            Ok(repo) => {
                self.save_working_diagrams();
                self.add_recent(&p);
                self.set_working_file(Some(&p));
                self.set_project(repo, Some(p));
                self.restore_working_diagrams();
                self.auto_validate();
            }
            Err(e) => self.message = Some(("Failed to open the file.".into(), e)),
        }
    }

    /// Bring label texts and sizes up to date in every diagram, as the original does when drawing.
    pub fn prepare_for_save(&mut self, ms: &dyn Measure) {
        for d in self.repo.all_of("Diagram") {
            if self.repo.alive(d) {
                views::layout_diagram(&mut self.repo, ms, d);
            }
        }
        let root = self.repo.root;
        if self.repo.ty(root) == "Project" && !self.repo.has(root, "documentVersion") {
            self.repo.set(root, "documentVersion", json!(DOCUMENT_VERSION));
        }
    }

    pub fn save_to(&mut self, p: PathBuf, ms: &dyn Measure) -> bool {
        self.prepare_for_save(ms);
        match std::fs::write(&p, self.repo.serialize()) {
            Ok(_) => {
                self.add_recent(&p);
                self.filename = Some(p);
                self.modified = false;
                self.save_working_diagrams();
                self.auto_validate();
                true
            }
            Err(e) => {
                self.message = Some(("Failed to save the file.".into(), e.to_string()));
                false
            }
        }
    }

    pub fn save(&mut self, ms: &dyn Measure) -> bool {
        match self.filename.clone() {
            Some(p) => self.save_to(p, ms),
            None => self.save_as(ms),
        }
    }

    pub fn save_as(&mut self, ms: &dyn Measure) -> bool {
        let name = self
            .filename
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{}.mdj", self.repo.name(self.repo.root)));
        let Some(mut p) = rfd::FileDialog::new().set_title("Save As").set_file_name(&name).add_filter("Models", &["mdj"]).save_file() else {
            return false;
        };
        if p.extension().map(|e| e != "mdj").unwrap_or(true) {
            p.set_extension("mdj");
        }
        self.save_to(p, ms)
    }

    /// Ask before throwing away changes. Returns false when the person cancels.
    pub fn confirm_discard(&mut self, ms: &dyn Measure) -> bool {
        if !self.modified || self.screenshot.is_some() {
            return true;
        }
        let r = rfd::MessageDialog::new()
            .set_title("UMLCraft")
            .set_description("Do you want to save changes? Your changes will be lost if you don't save them.")
            .set_buttons(rfd::MessageButtons::YesNoCancel)
            .show();
        match r {
            rfd::MessageDialogResult::Yes => self.save(ms),
            rfd::MessageDialogResult::No => true,
            _ => false,
        }
    }

    // ------------------------------------------------------------ undo
    fn snapshot(&self) -> Snap {
        Snap { repo: self.repo.clone(), diagram: self.diagram, open: self.open.clone(), selection: self.selection.clone(), focus: self.focus }
    }
    fn restore(&mut self, s: Snap) {
        self.repo = s.repo;
        self.diagram = s.diagram.filter(|d| self.repo.alive(*d));
        self.open = s.open.into_iter().filter(|d| self.repo.alive(*d)).collect();
        self.selection = s.selection.into_iter().filter(|d| self.repo.alive(*d)).collect();
        self.focus = s.focus.filter(|d| self.repo.alive(*d));
        self.prop_for = None;
        self.prop_set.clear();
    }
    /// Call before every change; then call `done`.
    pub fn checkpoint(&mut self) {
        self.undo.push(self.snapshot());
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
    pub fn done(&mut self) {
        self.modified = true;
        self.prop_for = None;
        self.prop_set.clear();
    }
    pub fn change<T>(&mut self, f: impl FnOnce(&mut App) -> T) -> T {
        self.checkpoint();
        let out = f(self);
        self.done();
        out
    }
    pub fn do_undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            let cur = self.snapshot();
            self.redo.push(cur);
            self.restore(s);
            self.modified = true;
        }
    }
    pub fn do_redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            let cur = self.snapshot();
            self.undo.push(cur);
            self.restore(s);
            self.modified = true;
        }
    }

    // ------------------------------------------------------------ names
    pub fn new_name(&self, parent: Eid, ty: &str) -> String {
        let base = match ty {
            "UMLClass" => "Class",
            "UMLInterface" => "Interface",
            "UMLEnumeration" => "Enumeration",
            "UMLDataType" => "DataType",
            "UMLPrimitiveType" => "PrimitiveType",
            "UMLSignal" => "Signal",
            "UMLPackage" => "Package",
            "UMLModel" => "Model",
            "UMLUseCase" => "UseCase",
            "UMLActor" => "Actor",
            "UMLUseCaseSubject" => "Subject",
            "UMLAttribute" => "Attribute",
            "UMLOperation" => "Operation",
            "UMLEnumerationLiteral" => "Literal",
            "UMLParameter" => "Parameter",
            "UMLTemplateParameter" => "TemplateParameter",
            "UMLClassDiagram" => "ClassDiagram",
            "UMLUseCaseDiagram" => "UseCaseDiagram",
            "UMLPackageDiagram" => "PackageDiagram",
            "UMLProfile" => "Profile",
            "UMLStereotype" => "Stereotype",
            other => other.trim_start_matches("UML"),
        };
        let names: HashSet<String> = self.repo.children(parent).into_iter().map(|(_, c)| self.repo.name(c)).collect();
        let mut i = 1;
        while names.contains(&format!("{base}{i}")) {
            i += 1;
        }
        format!("{base}{i}")
    }

    // ------------------------------------------------------------ model operations
    pub fn package_for(&self, d: Option<Eid>) -> Eid {
        d.and_then(|d| self.repo.parent(d)).unwrap_or(self.repo.root)
    }

    /// A free line from `a` to `b` on the current diagram (common-factory.js:32-44).
    pub fn create_freeline(&mut self, a: P, b: P) -> Option<Eid> {
        let d = self.diagram?;
        let props = [("font", json!(views::FONT)), ("lineStyle", json!(1)), ("points", json!(views::format_points(&[a, b])))];
        let v = self.repo.create("FreelineEdgeView", &props, Some((d, "ownedViews")));
        self.apply_view_defaults(v);
        Some(v)
    }

    pub fn create_node(&mut self, kind: &str, rect: R) -> Option<Eid> {
        let d = self.diagram?;
        if kind == "Note" || kind == "Text" || kind == "TextBox" {
            let ty = match kind {
                "Note" => "UMLNoteView",
                "TextBox" => "UMLTextBoxView",
                _ => "UMLTextView",
            };
            let v = views::create_note_view(&mut self.repo, d, rect, ty);
            self.apply_view_defaults(v);
            return Some(v);
        }
        let parent = self.package_for(Some(d));
        let name = self.new_name(parent, kind);
        // the Robustness tools make a class stereotyped «boundary», «entity» or «control», shown as an
        // icon without compartments (robustness/main.js:17)
        let robust = matches!(kind, "Boundary" | "Entity" | "Control");
        let ty = if robust { "UMLClass" } else { kind };
        let m = self.repo.create(ty, &[("name", json!(name))], Some((parent, "ownedElements")));
        if robust {
            let st_name = kind.to_lowercase();
            let found = self.repo.all().find(|x| self.repo.ty(*x) == "UMLStereotype" && self.repo.name(*x) == st_name);
            match found {
                Some(st) => self.repo.set_ref(m, "stereotype", Some(st)),
                None => self.repo.set(m, "stereotype", json!(st_name)),
            }
        }
        let v = views::create_node_view(&mut self.repo, d, m, rect)?;
        self.apply_view_defaults(v);
        if robust {
            self.repo.set(v, "stereotypeDisplay", json!("icon"));
            self.repo.set(v, "suppressAttributes", json!(true));
            self.repo.set(v, "suppressOperations", json!(true));
        }
        Some(v)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn create_view_of(&mut self, model: Eid, rect: R) -> Option<Eid> {
        let d = self.diagram?;
        views::create_node_view(&mut self.repo, d, model, rect)
    }

    /// Whether a box can hold another box (canContainView, core.js:1961-1986): packages and subjects
    /// hold boxes, and never themselves or a box that contains them.
    pub fn can_contain_view(&self, container: Eid, v: Eid) -> bool {
        let ct = self.repo.ty(container);
        if !(ct == "UMLPackageView" || ct == "UMLModelView" || ct == "UMLSubsystemView" || ct == "UMLUseCaseSubjectView") || !views::is_node(&self.repo, v) {
            return false;
        }
        // not the view itself or anything inside it
        let mut c = Some(container);
        while let Some(x) = c {
            if x == v {
                return false;
            }
            c = self.repo.refd(x, "containerView");
        }
        if ct == "UMLUseCaseSubjectView" {
            return true;
        }
        match (self.repo.refd(container, "model"), self.repo.refd(v, "model")) {
            (Some(cm), Some(vm)) => cm != vm && !self.repo.path(cm).contains(&vm),
            (_, None) => true,
            _ => false,
        }
    }

    /// Put views in a new container (or none), moving their models too: into a package's model, or
    /// to the diagram's owner when released (engine.js:1036-1185, uml/main.js:24-41). Subjects only
    /// hold the views. The moved views come to the front so they draw above the container.
    pub fn set_container(&mut self, vs: &[Eid], to: Option<Eid>) {
        let Some(d) = self.diagram else { return };
        for &v in vs {
            if let Some(old) = self.repo.refd(v, "containerView") {
                let id = self.repo.id(v);
                if let Some(serde_json::Value::Array(a)) = self.repo.raw(old, "containedViews").cloned() {
                    let kept: Vec<serde_json::Value> = a.into_iter().filter(|r| r.get("$ref").and_then(|x| x.as_str()) != Some(id.as_str())).collect();
                    if kept.is_empty() {
                        self.repo.remove(old, "containedViews");
                    } else {
                        self.repo.set_raw(old, "containedViews", json!(kept));
                    }
                }
            }
            self.repo.set_ref(v, "containerView", to);
            if let Some(c) = to {
                let mut a = match self.repo.raw(c, "containedViews") {
                    Some(serde_json::Value::Array(a)) => a.clone(),
                    _ => vec![],
                };
                a.push(self.repo.ref_to(v));
                self.repo.set_raw(c, "containedViews", json!(a));
            }
            let target = match to {
                Some(c) if self.repo.ty(c) == "UMLUseCaseSubjectView" => None,
                Some(c) => self.repo.refd(c, "model"),
                None => Some(self.package_for(Some(d))),
            };
            if let (Some(tm), Some(vm)) = (target, self.repo.refd(v, "model")) {
                if self.repo.parent(vm) != Some(tm) && self.repo.field_of(vm).as_deref() == Some("ownedElements") {
                    self.repo.relocate(vm, tm, "ownedElements");
                }
            }
        }
        let mut list = self.repo.list(d, "ownedViews");
        list.retain(|x| !vs.contains(x));
        list.extend(vs.iter().copied());
        self.repo.set_list(d, "ownedViews", list);
    }

    /// The two ends of a relationship model: (tail, head) = (source, target) or (end1, end2) references.
    pub fn rel_ends(&self, m: Eid) -> Option<(Eid, Eid)> {
        let r = &self.repo;
        if let (Some(s), Some(t)) = (r.refd(m, "source"), r.refd(m, "target")) {
            return Some((s, t));
        }
        let end = |k: &str| r.child(m, k).and_then(|e| r.refd(e, "reference"));
        Some((end("end1")?, end("end2")?))
    }

    /// The first visible top-level view of a model in the current diagram.
    fn shown_view(&self, d: Eid, m: Eid) -> Option<Eid> {
        self.repo
            .list(d, "ownedViews")
            .into_iter()
            .find(|v| self.repo.refd(*v, "model") == Some(m) && !matches!(self.repo.raw(*v, "visible"), Some(serde_json::Value::Bool(false))))
    }

    /// Drop an element from the explorer at `p` (app-context.js:1145-1186, factory.js:871-1205). Returns
    /// the views to select, or an alert.
    /// Whether a model dragged from the explorer can be dropped on the current diagram: an element
    /// with a box, or a relationship whose two ends can be shown.
    pub fn can_drop_model(&self, m: Eid) -> bool {
        let ty = self.repo.ty(m);
        self.diagram.is_some() && self.repo.alive(m) && (views::node_kind(ty).is_some() || (views::edge_view_of(ty).is_some() && self.rel_ends(m).is_some()))
    }

    pub fn drop_model(&mut self, m: Eid, p: P) -> Result<Vec<Eid>, String> {
        let Some(d) = self.diagram else { return Ok(vec![]) };
        let ty = self.repo.ty(m).to_string();
        if views::edge_view_of(&ty).is_some() && self.rel_ends(m).is_some() {
            return self.drop_relationship(d, m, p);
        }
        if views::node_kind(&ty).is_none() {
            return Ok(vec![]);
        }
        // a hidden view of it is shown again instead of a second one
        let hidden = self.repo.list(d, "ownedViews").into_iter().find(|v| self.repo.refd(*v, "model") == Some(m) && matches!(self.repo.raw(*v, "visible"), Some(serde_json::Value::Bool(false))));
        if let Some(v) = hidden {
            self.change(|a| a.repo.set(v, "visible", json!(true)));
            return Ok(vec![v]);
        }
        let v = self.change(|a| {
            let v = a.place_node(d, m, p)?;
            // inside whatever container box is under the drop point (app-context.js:1146-1189); the
            // model stays where it is
            let under = a.repo.list(d, "ownedViews").into_iter().rev().find(|c| {
                *c != v && !matches!(a.repo.raw(*c, "visible"), Some(serde_json::Value::Bool(false))) && {
                    let b = views::node_rect(&a.repo, *c);
                    p.x >= b.x && p.y >= b.y && p.x <= b.x + b.w && p.y <= b.y + b.h
                } && a.can_contain_view(*c, v)
            });
            if let Some(c) = under {
                a.repo.set_ref(v, "containerView", Some(c));
                let mut list = match a.repo.raw(c, "containedViews") {
                    Some(serde_json::Value::Array(l)) => l.clone(),
                    _ => vec![],
                };
                list.push(a.repo.ref_to(v));
                a.repo.set_raw(c, "containedViews", json!(list));
            }
            a.add_relationship_views(d, m, v);
            Some(v)
        });
        Ok(v.into_iter().collect())
    }

    /// A view of a model at `p`, showing the namespace when the model lives elsewhere than the diagram.
    pub(crate) fn place_node(&mut self, d: Eid, m: Eid, p: P) -> Option<Eid> {
        let (w, h) = views::default_size(self.repo.ty(m));
        let v = views::create_node_view(&mut self.repo, d, m, R { x: p.x, y: p.y, w, h })?;
        self.apply_view_defaults(v);
        if self.repo.parent(m) != self.repo.parent(d) && meta::order_of(self.repo.ty(v), "showNamespace").is_some() {
            self.repo.set(v, "showNamespace", json!(true));
        }
        Some(v)
    }

    /// Lines from the new view `v` of `m` for every relationship to a model already shown, to each of
    /// its views, self-relationships included (factory.js:871-1065).
    fn add_relationship_views(&mut self, d: Eid, m: Eid, v: Eid) {
        let rels: Vec<Eid> = self.repo.all().filter(|x| views::edge_view_of(self.repo.ty(*x)).is_some()).collect();
        let views_of = |a: &App, x: Eid| -> Vec<Eid> {
            a.repo
                .list(d, "ownedViews")
                .into_iter()
                .filter(|w| *w != v && a.repo.refd(*w, "model") == Some(x) && !matches!(a.repo.raw(*w, "visible"), Some(serde_json::Value::Bool(false))))
                .collect()
        };
        for rel in rels {
            let Some((t, h)) = self.rel_ends(rel) else { continue };
            let pairs: Vec<(Eid, Eid)> = if t == m && h == m {
                vec![(v, v)]
            } else if t == m {
                views_of(self, h).into_iter().map(|w| (v, w)).collect()
            } else if h == m {
                views_of(self, t).into_iter().map(|w| (w, v)).collect()
            } else {
                vec![]
            };
            for (tv, hv) in pairs {
                views::create_edge_view(&mut self.repo, d, Some(rel), tv, hv);
            }
        }
    }

    fn drop_relationship(&mut self, d: Eid, m: Eid, p: P) -> Result<Vec<Eid>, String> {
        let (t, h) = self.rel_ends(m).unwrap();
        if let Some(e) = self.shown_view(d, m) {
            let mut sel = vec![e];
            sel.extend([self.repo.refd(e, "tail"), self.repo.refd(e, "head")].into_iter().flatten());
            self.select(sel);
            return Err("Relationship View is already existed in this Diagram.".into());
        }
        let e = self.change(|a| {
            let tv = match a.shown_view(d, t) {
                Some(v) => v,
                None => a.place_node(d, t, p)?,
            };
            let hv = match a.shown_view(d, h) {
                Some(v) => v,
                None => a.place_node(d, h, P { x: p.x, y: p.y + 100.0 })?,
            };
            views::create_edge_view(&mut a.repo, d, Some(m), tv, hv)
        });
        Ok(e.into_iter().collect())
    }

    pub fn can_connect(&self, tool: &Tool, tail: Eid, head: Eid) -> bool {
        self.can_link(tool.kind, tail, head)
    }

    /// The kind a line was made with: its model's type, or "NoteLink".
    pub fn edge_kind(&self, e: Eid) -> String {
        match self.repo.refd(e, "model") {
            Some(m) if self.repo.ty(m) == "UMLAssociationClassLink" => "AssociationClass".into(),
            Some(m) => self.repo.ty(m).to_string(),
            None if self.repo.ty(e) == "UMLNoteLinkView" => "NoteLink".into(),
            None if self.repo.ty(e) == "UMLContainmentView" => "Containment".into(),
            None => String::new(),
        }
    }

    /// Whether a line of this kind may join these two views.
    pub fn can_link(&self, kind: &str, tail: Eid, head: Eid) -> bool {
        let r = &self.repo;
        if kind == "NoteLink" {
            return r.ty(tail) == "UMLNoteView" || r.ty(head) == "UMLNoteView";
        }
        if kind == "AssociationClass" {
            // between two classifiers, or from a class to an association line (uml-factory.js:631)
            let (Some(tm), Some(hm)) = (r.refd(tail, "model"), r.refd(head, "model")) else { return false };
            if views::is_edge(r, tail) {
                return false;
            }
            if views::is_edge(r, head) {
                let linked = r.all_of("UMLAssociationClassLink").into_iter().any(|l| r.refd(l, "associationSide") == Some(hm));
                return r.ty(tm) == "UMLClass" && r.ty(hm) == "UMLAssociation" && !linked;
            }
            return r.kind_of(tm, "UMLClassifier") && r.kind_of(hm, "UMLClassifier");
        }
        if views::is_edge(r, tail) || views::is_edge(r, head) {
            return false;
        }
        let (Some(tm), Some(hm)) = (r.refd(tail, "model"), r.refd(head, "model")) else { return false };
        if kind == "Containment" {
            // the tail's element goes into the head's package (shown in the explorer)
            return r.kind_of(hm, "UMLPackage") && tm != hm && (r.parent(tm) == Some(hm) || self.can_relocate(tm, hm));
        }
        let classifier = |m: Eid| r.kind_of(m, "UMLClassifier");
        match kind {
            "UMLAssociation" => classifier(tm) && classifier(hm),
            // deployment diagrams: a path joins two nodes, a deployment puts a classifier on a node and a
            // component realization realizes a component (uml-factory.js:19-146)
            "UMLCommunicationPath" => r.ty(tm) == "UMLNode" && r.ty(hm) == "UMLNode",
            "UMLDeployment" => classifier(tm) && r.ty(hm) == "UMLNode" && tm != hm,
            "UMLComponentRealization" => classifier(tm) && r.ty(hm) == "UMLComponent" && tm != hm,
            // the bound classifier points at its template
            "UMLTemplateBinding" => classifier(tm) && classifier(hm) && tm != hm,
            "UMLGeneralization" => classifier(tm) && classifier(hm) && tm != hm,
            "UMLInterfaceRealization" => classifier(tm) && r.ty(hm) == "UMLInterface",
            "UMLInclude" | "UMLExtend" => r.ty(tm) == "UMLUseCase" && r.ty(hm) == "UMLUseCase",
            _ => true,
        }
    }

    pub fn create_edge(&mut self, tool: &Tool, tail: Eid, head: Eid) -> Option<Eid> {
        let e = self.create_edge_inner(tool, tail, head)?;
        self.apply_view_defaults(e);
        Some(e)
    }

    fn create_edge_inner(&mut self, tool: &Tool, tail: Eid, head: Eid) -> Option<Eid> {
        let d = self.diagram?;
        if tool.kind == "NoteLink" {
            return views::create_edge_view(&mut self.repo, d, None, tail, head);
        }
        if tool.kind == "AssociationClass" {
            return self.create_association_class(d, tail, head);
        }
        if tool.kind == "Containment" {
            // a view-only line; the contained element moves into the container
            let (tm, hm) = (self.repo.refd(tail, "model")?, self.repo.refd(head, "model")?);
            if self.repo.parent(tm) != Some(hm) {
                self.repo.relocate(tm, hm, "ownedElements");
            }
            let props = [("font", json!(views::FONT)), ("head", self.repo.ref_to(head)), ("tail", self.repo.ref_to(tail)), ("lineStyle", json!(views::DEFAULT_LINE_STYLE)), ("points", json!("0:0;0:0"))];
            return Some(self.repo.create("UMLContainmentView", &props, Some((d, "ownedViews"))));
        }
        let tm = self.repo.refd(tail, "model")?;
        let hm = self.repo.refd(head, "model")?;
        let m = if tool.kind == "UMLAssociation" || tool.kind == "UMLCommunicationPath" {
            let m = self.repo.create(tool.kind, &[], Some((tm, "ownedElements")));
            let (r1, r2) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
            let e1 = self.repo.create("UMLAssociationEnd", &[("reference", r1)], Some((m, "end1")));
            let e2 = self.repo.create("UMLAssociationEnd", &[("reference", r2)], Some((m, "end2")));
            match tool.init {
                AssocInit::Directed => {
                    self.repo.set(e1, "navigable", json!("unspecified"));
                    self.repo.set(e2, "navigable", json!("navigable"));
                }
                AssocInit::Shared => self.repo.set(e2, "aggregation", json!("shared")),
                AssocInit::Composite => self.repo.set(e2, "aggregation", json!("composite")),
                AssocInit::Plain => {}
            }
            m
        } else {
            let (s, t) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
            self.repo.create(tool.kind, &[("source", s), ("target", t)], Some((tm, "ownedElements")))
        };
        views::create_edge_view(&mut self.repo, d, Some(m), tail, head)
    }

    /// The Association Class tool (uml-factory.js:631): between two classifiers it creates an
    /// association, a class "AssociationClass" 30 px below the middle of the line, and the dashed link
    /// between them; from a class to an association line it only links them. Returns the class's box
    /// in the first case, the link in the second.
    fn create_association_class(&mut self, d: Eid, tail: Eid, head: Eid) -> Option<Eid> {
        let link = |a: &mut App, class_view: Eid, assoc_view: Eid| -> Option<Eid> {
            let (cm, am) = (a.repo.refd(class_view, "model")?, a.repo.refd(assoc_view, "model")?);
            let (c, s) = (a.repo.ref_to(cm), a.repo.ref_to(am));
            let l = a.repo.create("UMLAssociationClassLink", &[("classSide", c), ("associationSide", s)], Some((cm, "ownedElements")));
            let props = [
                ("model", a.repo.ref_to(l)),
                ("font", json!(views::FONT)),
                ("head", a.repo.ref_to(assoc_view)),
                ("tail", a.repo.ref_to(class_view)),
                ("lineStyle", json!(views::DEFAULT_LINE_STYLE)),
                ("points", json!("0:0;0:0")),
            ];
            Some(a.repo.create("UMLAssociationClassLinkView", &props, Some((d, "ownedViews"))))
        };
        if views::is_edge(&self.repo, head) {
            return link(self, tail, head);
        }
        let assoc = Tool { label: "", kind: "UMLAssociation", icon: "", line: true, init: AssocInit::Plain };
        let e = self.create_edge(&assoc, tail, head)?;
        let (a, b) = (views::node_rect(&self.repo, tail), views::node_rect(&self.repo, head));
        let mid = P { x: (a.x + a.w / 2.0 + b.x + b.w / 2.0) / 2.0, y: (a.y + a.h / 2.0 + b.y + b.h / 2.0) / 2.0 };
        let (w, h) = views::default_size("UMLClass");
        let cv = self.create_node("UMLClass", R { x: (mid.x - w / 2.0).max(0.0).round(), y: (mid.y + 30.0).round(), w, h })?;
        if let Some(cm) = self.repo.refd(cv, "model") {
            self.repo.set(cm, "name", json!("AssociationClass"));
        }
        link(self, cv, e)?;
        Some(cv)
    }

    /// Reconnect one end of a line to another view, updating the model too: `source`/`target` of a
    /// directed relationship, `end1`/`end2.reference` of an association (engine.js:940-1024). Returns
    /// the refusal message when the kinds do not fit (uml/main.js:44-82).
    pub fn reconnect(&mut self, e: Eid, tail: bool, to: Eid) -> Result<(), String> {
        let (Some(t), Some(h)) = (self.repo.refd(e, "tail"), self.repo.refd(e, "head")) else { return Ok(()) };
        if (tail && to == t) || (!tail && to == h) {
            return Ok(());
        }
        let kind = self.edge_kind(e);
        let (nt, nh) = if tail { (to, h) } else { (t, to) };
        if !self.can_link(&kind, nt, nh) {
            let target = self.repo.refd(to, "model").map(|m| self.repo.ty(m).to_string()).unwrap_or_default();
            return Err(if kind.is_empty() || target.is_empty() {
                "Invalid Connection".into()
            } else {
                format!("{} cannot be connected to {}", kind.trim_start_matches("UML"), target.trim_start_matches("UML"))
            });
        }
        self.change(|a| {
            a.repo.set_ref(e, if tail { "tail" } else { "head" }, Some(to));
            // a containment moves the contained element into the container again
            if kind == "Containment" {
                if let (Some(cm), Some(pm)) = (a.repo.refd(nt, "model"), a.repo.refd(nh, "model")) {
                    if a.repo.parent(cm) != Some(pm) {
                        a.repo.relocate(cm, pm, "ownedElements");
                    }
                }
            }
            // the ends are laid out again from the new box
            if let (Some(m), Some(tm)) = (a.repo.refd(e, "model"), a.repo.refd(to, "model")) {
                if a.repo.ty(m) == "UMLAssociationClassLink" {
                    a.repo.set_ref(m, if tail { "classSide" } else { "associationSide" }, Some(tm));
                } else if a.repo.has(m, "end1") {
                    if let Some(end) = a.repo.child(m, if tail { "end1" } else { "end2" }) {
                        a.repo.set_ref(end, "reference", Some(tm));
                    }
                } else {
                    a.repo.set_ref(m, if tail { "source" } else { "target" }, Some(tm));
                }
            }
        });
        Ok(())
    }

    pub fn add_feature(&mut self, owner: Eid, ty: &str) -> Option<Eid> {
        let field = match ty {
            "UMLAttribute" => "attributes",
            "UMLOperation" => "operations",
            "UMLReception" => "receptions",
            "UMLEnumerationLiteral" => "literals",
            "UMLParameter" => "parameters",
            "UMLTemplateParameter" => "templateParameters",
            _ => return None,
        };
        let name = self.new_name(owner, ty);
        Some(self.repo.create(ty, &[("name", json!(name))], Some((owner, field))))
    }

    pub fn can_add_feature(&self, owner: Eid, ty: &str) -> bool {
        let t = self.repo.ty(owner);
        match ty {
            "UMLEnumerationLiteral" => t == "UMLEnumeration",
            "UMLParameter" => t == "UMLOperation",
            // template parameters on classes, interfaces and the other boxes with compartments
            "UMLTemplateParameter" => self.repo.kind_of(owner, "UMLClassifier") && t != "UMLActor" && t != "UMLUseCase" && t != "UMLEnumeration",
            _ => self.repo.kind_of(owner, "UMLClassifier") && t != "UMLActor" && t != "UMLUseCase",
        }
    }

    pub fn add_model(&mut self, parent: Eid, ty: &str) -> Eid {
        let name = self.new_name(parent, ty);
        self.repo.create(ty, &[("name", json!(name))], Some((parent, "ownedElements")))
    }

    pub fn add_diagram(&mut self, parent: Option<Eid>, ty: &str) -> Eid {
        let target = match parent {
            Some(p) if self.repo.kind_of(p, "UMLPackage") || self.repo.ty(p) == "Project" || self.repo.kind_of(p, "UMLClassifier") => p,
            _ => self.package_for(self.diagram),
        };
        let name = self.new_name(target, ty);
        let d = self.repo.create(ty, &[("name", json!(name))], Some((target, "ownedElements")));
        self.open_diagram(d);
        // selected and scrolled to in the explorer (app-context.js:941-944)
        self.reveal(d);
        d
    }

    fn views_of(&self, ids: &HashSet<String>) -> Vec<Eid> {
        let mut out = vec![];
        for d in self.repo.all_of("Diagram") {
            for v in self.repo.list(d, "ownedViews") {
                if let Some(m) = self.repo.raw(v, "model").and_then(|m| m.get("$ref")).and_then(|s| s.as_str()) {
                    if ids.contains(m) {
                        out.push(v);
                    }
                }
            }
        }
        out
    }

    /// Remove views and every line that hangs on them.
    pub fn remove_views(&mut self, views: &[Eid]) {
        // the views inside a removed box go with it (engine.js:281-330)
        let mut all: Vec<Eid> = views.iter().copied().filter(|v| self.repo.alive(*v)).collect();
        let mut i = 0;
        while i < all.len() {
            if let Some(serde_json::Value::Array(cs)) = self.repo.raw(all[i], "containedViews").cloned() {
                for c in cs.iter().filter_map(|x| self.repo.by_ref(x)) {
                    if self.repo.alive(c) && !all.contains(&c) {
                        all.push(c);
                    }
                }
            }
            i += 1;
        }
        let mut gone: HashSet<String> = all.iter().map(|v| self.repo.id(*v)).collect();
        loop {
            let mut grew = false;
            for d in self.repo.all_of("Diagram") {
                for v in self.repo.list(d, "ownedViews") {
                    let id = self.repo.id(v);
                    if gone.contains(&id) {
                        continue;
                    }
                    let hangs = ["head", "tail"].iter().any(|k| {
                        self.repo.raw(v, k).and_then(|x| x.get("$ref")).and_then(|s| s.as_str()).map(|s| gone.contains(s)).unwrap_or(false)
                    });
                    if hangs {
                        gone.insert(id);
                        grew = true;
                    }
                }
            }
            if !grew {
                break;
            }
        }
        for id in &gone {
            if let Some(&v) = self.repo.by_id.get(id) {
                if self.repo.alive(v) {
                    self.repo.detach(v);
                }
            }
        }
        self.repo.clear_refs_to(&gone);
        let repo = &self.repo;
        self.selection.retain(|v| repo.alive(*v));
    }

    pub fn delete_models(&mut self, models: &[Eid]) {
        let mut ids: HashSet<String> = HashSet::new();
        for m in models {
            if !self.repo.alive(*m) {
                continue;
            }
            let mut all = vec![];
            self.repo.descendants(*m, &mut all);
            ids.extend(all.into_iter().map(|x| self.repo.id(x)));
        }
        // relationships that connect to removed elements go too
        loop {
            let mut grew = false;
            let all: Vec<Eid> = self.repo.all().collect();
            for o in all {
                let id = self.repo.id(o);
                if ids.contains(&id) {
                    continue;
                }
                let r = &self.repo;
                let refid = |e: Eid, k: &str| r.raw(e, k).and_then(|x| x.get("$ref")).and_then(|s| s.as_str()).map(|s| s.to_string());
                let mut ends = vec![refid(o, "source"), refid(o, "target")];
                for k in ["end1", "end2"] {
                    if let Some(end) = r.child(o, k) {
                        ends.push(refid(end, "reference"));
                    }
                }
                let is_rel = r.kind_of(o, "Relationship") || r.has(o, "source") || r.has(o, "end1");
                if is_rel && ends.iter().flatten().any(|e| ids.contains(e)) {
                    let mut all = vec![];
                    self.repo.descendants(o, &mut all);
                    ids.extend(all.into_iter().map(|x| self.repo.id(x)));
                    grew = true;
                }
            }
            if !grew {
                break;
            }
        }
        let views = self.views_of(&ids);
        self.remove_views(&views);
        let doomed: Vec<Eid> = ids.iter().filter_map(|id| self.repo.by_id.get(id).copied()).collect();
        for e in doomed {
            if !self.repo.alive(e) {
                continue;
            }
            let parent_doomed = self.repo.parent(e).map(|p| self.repo.alive(p) && ids.contains(&self.repo.id(p))).unwrap_or(false);
            if !parent_doomed {
                self.repo.detach(e);
            }
        }
        self.repo.clear_refs_to(&ids);
        let repo = &self.repo;
        self.open.retain(|d| repo.alive(*d));
        if self.diagram.map(|d| !repo.alive(d)).unwrap_or(false) {
            self.diagram = self.open.first().copied();
        }
        self.focus = self.focus.filter(|f| repo.alive(*f));
        self.selection.retain(|v| repo.alive(*v));
    }

    pub fn open_diagram(&mut self, d: Eid) {
        if !self.open.contains(&d) {
            self.open.push(d);
        }
        if self.diagram != Some(d) {
            // switching diagrams closes the inline edit, keeping what was typed (app-context.js:1297-1324)
            self.close_inline(true);
            // each diagram keeps its own scroll position (diagram-manager.js:218-228, 243)
            if let Some(old) = self.diagram {
                self.scrolls.insert(old, self.scroll);
            }
            self.diagram = Some(d);
            self.selection.clear();
            self.scroll = self.scrolls.get(&d).copied().unwrap_or_default();
            self.remove_broken_views(d);
        }
        self.tool = None;
    }

    /// The views of a diagram that cannot be drawn: a model reference that leads nowhere (or no model
    /// where the kind needs one), a line missing an end, a line whose relationship is missing an end,
    /// and a box or line without its name compartment or name label (diagram-manager.js:521-641).
    pub fn broken_views(&self, d: Eid) -> Vec<Eid> {
        let r = &self.repo;
        let dangling = |v: Eid, k: &str| matches!(r.raw(v, k), Some(x) if r.by_ref(x).map(|t| !r.alive(t)).unwrap_or(true));
        r.list(d, "ownedViews")
            .into_iter()
            .filter(|&v| {
                let ty = r.ty(v);
                let needs_model = ty.starts_with("UML") && !matches!(ty, "UMLNoteView" | "UMLTextView" | "UMLTextBoxView" | "UMLNoteLinkView" | "UMLContainmentView" | "UMLCustomTextView" | "UMLCustomNoteView");
                if dangling(v, "model") || (needs_model && !r.has(v, "model") && crate::meta::known(ty)) {
                    return true;
                }
                // a free line has no ends to lose
                if views::is_edge(r, v) && !views::is_freeline(r, v) {
                    if r.refd(v, "tail").is_none() || r.refd(v, "head").is_none() || dangling(v, "tail") || dangling(v, "head") {
                        return true;
                    }
                    if let Some(m) = r.refd(v, "model") {
                        if views::edge_view_of(r.ty(m)).is_some() && self.rel_ends(m).is_none() {
                            return true;
                        }
                    }
                }
                ["nameCompartment", "nameLabel"].iter().any(|k| crate::meta::order_of(ty, k).is_some() && r.has(v, "model") && (r.refd(v, k).is_none() || dangling(v, k)))
            })
            .collect()
    }

    /// Silently drop the broken views of a diagram when it becomes current, with the lines on them.
    pub fn remove_broken_views(&mut self, d: Eid) {
        let broken = self.broken_views(d);
        if !broken.is_empty() {
            self.remove_views(&broken);
        }
    }

    /// Close a working diagram. When it was the current one, the diagram that slid into its place
    /// becomes current, or else the first one (diagram-manager.js:106-138).
    pub fn close_diagram(&mut self, d: Option<Eid>) {
        let Some(d) = d.or(self.diagram) else { return };
        if let Some(i) = self.open.iter().position(|x| *x == d) {
            self.open.remove(i);
            // closing any working diagram clears the selection and the quick edit (app-context.js:1310-1317)
            self.close_inline(true);
            self.selection.clear();
            if self.diagram == Some(d) {
                self.scrolls.insert(d, self.scroll);
                let next = self.open.get(i).or(self.open.first()).copied();
                self.diagram = None;
                if let Some(n) = next {
                    self.open_diagram(n);
                }
            }
        }
    }

    /// View > Close Other Diagrams: keep only the current diagram open (diagram-manager.js:201-207).
    pub fn close_other_diagrams(&mut self) {
        self.open = self.diagram.into_iter().collect();
        self.editing = None;
    }

    /// View > Close All Diagrams.
    pub fn close_all_diagrams(&mut self) {
        self.open.clear();
        self.diagram = None;
        self.selection.clear();
        self.editing = None;
        self.tool = None;
    }

    /// View > Next Diagram (`step` 1) and Previous Diagram (`step` -1), wrapping around the working
    /// diagrams (diagram-manager.js:271-292).
    pub fn cycle_diagram(&mut self, step: isize) {
        let n = self.open.len() as isize;
        if n == 0 {
            return;
        }
        let i = self.diagram.and_then(|d| self.open.iter().position(|x| *x == d)).map(|i| i as isize).unwrap_or(-step.max(0));
        let next = self.open[(i + step).rem_euclid(n) as usize];
        self.open_diagram(next);
    }

    pub fn select(&mut self, views: Vec<Eid>) {
        if views.len() == 1 {
            self.focus = Some(self.repo.refd(views[0], "model").unwrap_or(views[0]));
        } else if views.is_empty() {
            self.focus = None;
        }
        self.selection = views;
    }

    /// Edit > Move Up / Move Down (Ctrl+Shift+Up/Down): swap the element with the previous or next
    /// sibling of the same explorer group in its owner's field (default-commands.js:843-857, engine.js:504-583).
    pub fn move_element(&mut self, e: Eid, up: bool) {
        let (Some(p), Some(field)) = (self.repo.parent(e), self.repo.field_of(e)) else { return };
        if self.is_view(e) {
            return;
        }
        let mut list = self.repo.list(p, &field);
        let Some(i) = list.iter().position(|x| *x == e) else { return };
        let rank = crate::ui::order_rank(self.repo.ty(e));
        let same = |x: &Eid| crate::ui::order_rank(self.repo.ty(*x)) == rank;
        let j = if up { list[..i].iter().rposition(same) } else { list[i + 1..].iter().position(same).map(|k| i + 1 + k) };
        let Some(j) = j else { return };
        list.swap(i, j);
        self.change(|a| a.repo.set_list(p, &field, list));
        self.focus = Some(e);
    }

    /// Edit > Open Sub-Diagram (Ctrl+Shift+D): open the first diagram the element owns, or toast
    /// "No sub-diagram in the selected element" (default-commands.js:909-934).
    pub fn open_sub_diagram(&mut self) {
        let Some(e) = self.focus.filter(|f| self.repo.alive(*f)) else { return };
        let mut owner = e;
        for key in ["subactivity", "submachine"] {
            if let Some(x) = self.repo.refd(e, key) {
                owner = x;
            }
        }
        let sub = self.repo.list(owner, "ownedElements").into_iter().find(|c| self.is_diagram(*c));
        match sub {
            Some(d) => self.open_diagram(d),
            None => self.toast("No sub-diagram in the selected element"),
        }
    }

    pub fn toast(&mut self, text: &str) {
        self.toasts.push((text.to_string(), std::time::Instant::now(), false));
    }

    /// An error toast (toast-manager.js:36-72).
    pub fn toast_error(&mut self, text: &str) {
        self.toasts.push((text.to_string(), std::time::Instant::now(), true));
    }

    /// Edit > Select In Diagram (Ctrl+D): the views of the focused model that sit directly in a
    /// diagram. One view is shown at once; several diagrams open a picker; none shows a toast
    /// (default-commands.js:877-907).
    pub fn select_in_diagram(&mut self) {
        let Some(m) = self.focus.filter(|f| self.repo.alive(*f)) else { return };
        let mut views: Vec<Eid> = vec![];
        for d in self.repo.all_of("Diagram") {
            if let Some(v) = self.repo.list(d, "ownedViews").into_iter().find(|v| self.repo.refd(*v, "model") == Some(m)) {
                views.push(v);
            }
        }
        match views.len() {
            0 => self.toast("No diagrams to show"),
            1 => self.show_view(views[0]),
            _ => self.diagram_picker = Some((views, None)),
        }
    }

    /// Open the view's diagram, select the view and centre the canvas on it (diagram-manager.js:340-348, 484-490).
    pub fn show_view(&mut self, v: Eid) {
        let Some(d) = self.repo.parent(v).filter(|d| self.is_diagram(*d)) else { return };
        self.open_diagram(d);
        self.select(vec![v]);
        self.centre_pending = Some(v);
    }

    /// Bounding box of a view: its box, or the box around a line's points.
    pub fn view_bounds(&self, v: Eid) -> R {
        if views::is_edge(&self.repo, v) {
            let pts = views::points_of(&self.repo, v);
            let (mut x1, mut y1, mut x2, mut y2) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for p in &pts {
                x1 = x1.min(p.x);
                y1 = y1.min(p.y);
                x2 = x2.max(p.x);
                y2 = y2.max(p.y);
            }
            if pts.is_empty() {
                return R::default();
            }
            R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
        } else {
            views::node_rect(&self.repo, v)
        }
    }

    pub fn focus_element(&mut self, e: Eid) {
        self.focus = Some(e);
        self.selection = match self.diagram {
            Some(d) => self.repo.list(d, "ownedViews").into_iter().filter(|v| self.repo.refd(*v, "model") == Some(e)).take(1).collect(),
            None => vec![],
        };
    }

    /// The line style shared by the selected lines, if they agree.
    pub fn line_style_state(&self) -> Option<i64> {
        let mut it = self.selection.iter().filter(|v| self.repo.alive(**v) && views::is_edge(&self.repo, **v)).map(|v| self.repo.val(*v, "lineStyle").and_then(|x| x.as_i64()).unwrap_or(0));
        let first = it.next()?;
        it.all(|s| s == first).then_some(first)
    }

    pub fn set_line_style(&mut self, style: i64) {
        let edges: Vec<Eid> = self.selection.iter().copied().filter(|v| views::is_edge(&self.repo, *v)).collect();
        if edges.is_empty() {
            return;
        }
        self.change(|a| {
            for e in edges {
                a.repo.set(e, "lineStyle", json!(style));
                let pts = views::points_of(&a.repo, e);
                if pts.len() > 2 {
                    a.repo.set(e, "points", json!(views::format_points(&[pts[0], pts[pts.len() - 1]])));
                }
            }
        });
    }

    /// The selected views that have this display flag (for example showVisibility on classifier views).
    pub fn flag_views(&self, field: &str) -> Vec<Eid> {
        self.selection.iter().copied().filter(|v| self.repo.alive(*v) && meta::order_of(self.repo.ty(*v), field).is_some()).collect()
    }

    /// The merged value of a flag over the selection: Some(value) when every view agrees.
    pub fn flag_state(&self, field: &str) -> Option<bool> {
        let vs = self.flag_views(field);
        let first = self.repo.bool(*vs.first()?, field);
        vs.iter().all(|v| self.repo.bool(*v, field) == first).then_some(first)
    }

    /// Format toggles (Show Shadow, Show Visibility, Suppress Attributes…): invert the shared value on
    /// every selected view that has the flag, or set it when the selection is mixed. One undo step
    /// (default-commands.js:1013-1029; uml-commands.js:271-279, 4596-4651).
    pub fn toggle_flag(&mut self, field: &str) {
        let vs = self.flag_views(field);
        if vs.is_empty() {
            return;
        }
        let value = match self.flag_state(field) {
            Some(v) => !v,
            None => true,
        };
        self.change(|a| {
            for v in vs {
                a.repo.set(v, field, json!(value));
            }
        });
    }

    pub fn toggle_shadow(&mut self) {
        self.toggle_flag("showShadow");
    }

    pub fn set_zoom(&mut self, z: f64) {
        self.zoom = (z.clamp(ZOOM_MIN, ZOOM_MAX) * 10.0).round() / 10.0;
    }

    /// Open the diagram and select the element with these names (start-up options).
    pub fn show_by_name(&mut self, diagram: Option<&str>, element: Option<&str>) {
        if let Some(name) = diagram {
            if let Some(d) = self.repo.all_of("Diagram").into_iter().find(|d| self.repo.name(*d) == name) {
                self.open_diagram(d);
            }
        }
        if let (Some(name), Some(d)) = (element, self.diagram) {
            let view = self.repo.list(d, "ownedViews").into_iter().find(|v| self.repo.refd(*v, "model").map(|m| self.repo.name(m) == name).unwrap_or(false));
            if let Some(v) = view {
                self.select(vec![v]);
                self.centre_pending = Some(v);
            }
        }
    }

    pub fn is_diagram(&self, e: Eid) -> bool {
        self.repo.kind_of(e, "Diagram") || self.repo.ty(e).ends_with("Diagram")
    }

    pub fn is_view(&self, e: Eid) -> bool {
        meta::is_kind_of(self.repo.ty(e), "View") || self.repo.ty(e).ends_with("View")
    }
}
