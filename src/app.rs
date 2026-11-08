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
/// The space between a box and a constraint added to its left (estimated).
pub const CONSTRAINT_GAP: f64 = 30.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AssocInit {
    Plain,
    Directed,
    Shared,
    Composite,
    /// composite at the tail (end1): SysML's Add Composited Block (sysml-commands.js:39-62)
    CompositeTail,
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

/// A "Select One" dialog: a message and radio buttons; OK runs the purpose with the chosen index.
#[derive(Clone, Debug)]
pub struct SelectOne {
    pub message: String,
    pub options: Vec<&'static str>,
    pub selected: usize,
    pub purpose: SelectPurpose,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SelectPurpose {
    /// Model › Add › Constraint on an operation: the field the constraint goes in.
    ConstraintKind(Eid),
    /// Model › Add › Entry, Do or Exit Activity on a state, or Effect on a transition: the kind of
    /// behaviour, created in that field (uml-commands.js:109-190).
    BehaviorKind(Eid, &'static str),
    /// A SysML connector's Create Item Flow: the direction (sysml-commands.js:175-215).
    ItemFlow(Eid),
}

/// The behaviour kinds of an entry, do or exit activity or an effect (uml-commands.js:111-116).
pub const BEHAVIOR_KINDS: [(&str, &str); 4] = [("OpaqueBehavior", "UMLOpaqueBehavior"), ("Activity", "UMLActivity"), ("StateMachine", "UMLStateMachine"), ("Interaction", "UMLInteraction")];

/// The constraint kinds of an operation and the field each goes in (uml-commands.js:185).
pub const OPERATION_CONSTRAINTS: [(&str, &str); 4] = [("Typical", "ownedElements"), ("Precondition", "preconditions"), ("BodyCondition", "bodyConditions"), ("Postcondition", "postconditions")];

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
    /// The text the edit showed when it opened (taken on its first frame): only a changed text is
    /// checked against the grammar.
    pub orig: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Area {
    Canvas,
    Explorer,
    Other,
}

/// The lowest a message may be drawn: just under the lifelines' heads.
const LIFELINE_MESSAGE_MIN_Y: f64 = crate::sequence::LIFELINE_TOP + crate::sequence::HEAD_H + 10.0;

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
    /// When a rejected inline edit started shaking (it cannot close for 400 ms, quickedit-manager.js:407-427).
    pub edit_shake: Option<std::time::Instant>,
    /// The multiplicity typed in the open association end edit's combo, not yet applied.
    pub edit_combo: Option<(Eid, String)>,
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
    /// Where the line being drawn was pressed and released: a message keeps that height.
    pub line_from: Option<(P, P)>,
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
    /// The Element List Picker (Select In Diagram with several diagrams, Select Operation…).
    pub list_picker: Option<crate::dialogs::ListPicker>,
    /// Information and alert boxes waiting to be shown (dialog-manager.js:675-703).
    pub boxes: Vec<(crate::dialogs::BoxKind, String)>,
    /// The last text UMLCraft put on the clipboard, used when the system clipboard cannot be read.
    pub clipboard: Option<String>,
    /// The system clipboard, kept open: on Linux an image copied to it is served while it lives.
    pub sys_clipboard: Option<arboard::Clipboard>,
    /// The colour field last changed and when, so a picker drag is one undo step.
    pub style_edit: Option<(String, std::time::Instant)>,
    /// The working diagrams in the order they were opened, which Next/Previous Diagram follow; the
    /// displayed (dragged) order replaces it when the project is saved (diagram-manager.js:152-160, 271-293).
    pub open_order: Vec<Eid>,
    /// A "Select One" dialog waiting for an answer (dialog-manager.js:433-476).
    pub select_one: Option<SelectOne>,
    /// The "Delete Views Only" question, for views that can neither be deleted alone nor hidden.
    pub delete_prompt: bool,
    /// The Element List Editor: the element, the reference-list field and the selected row.
    pub list_editor: Option<(Eid, &'static str, Option<usize>)>,
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
    /// The open Tools › <Language> dialog (code generation and reverse engineering).
    pub codegen: Option<crate::codegen::Dialog>,
    /// The theme the style was last set for (true: light).
    pub theme_applied: Option<bool>,
    /// The element being renamed in the Model Explorer: its text, and whether the edit just opened.
    pub tree_rename: Option<(Eid, String, bool)>,
    /// The user's keymap.json bindings.
    pub user_keymap: Vec<(crate::keymap::Stroke, String)>,
    /// A press in the diagram deselects the Model Explorer: its row is no longer highlighted, though
    /// the model stays the focus the editors show, until the tree is used again (app-context.js:1325-1337).
    pub tree_cleared: bool,
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
    /// The project draws with colour variables (it held some when opened, or was converted): new views
    /// get variables too (colorvars.rs).
    pub color_vars: bool,
    /// The Convert / Keep Colors question, asked when a file of StarUML 7 or earlier is opened.
    pub color_prompt: bool,
    /// The Export Image dialog (Ctrl+Shift+E) is open, with the outcome of its last action.
    pub export_image_open: bool,
    pub export_image_result: Option<String>,
    /// The open Element Picker ("Select Element").
    pub element_picker: Option<crate::picker::ElementPicker>,
    /// The Icon Picker of a cloud element (icon-picker-dialog.js).
    pub icon_picker: Option<crate::cloud::IconPicker>,
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
            edit_shake: None,
            edit_combo: None,
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
            line_from: None,
            pending_item: None,
            reveal_pending: None,
            scrolls: HashMap::new(),
            centre_pending: None,
            toasts: vec![],
            list_picker: None,
            boxes: vec![],
            clipboard: None,
            sys_clipboard: None,
            style_edit: None,
            color_dialog: None,
            list_editor: None,
            select_one: None,
            delete_prompt: false,
            open_order: vec![],
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
            codegen: None,
            theme_applied: None,
            tree_rename: None,
            user_keymap: vec![],
            tree_cleared: false,
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
            color_vars: false,
            color_prompt: false,
            export_image_open: false,
            export_image_result: None,
            element_picker: None,
            icon_picker: None,
            input_dialog: None,
        };
        a.set_project(Repo::blank(), None);
        a
    }

    // ------------------------------------------------------------ project
    pub fn set_project(&mut self, repo: Repo, filename: Option<PathBuf>) {
        self.repo = repo;
        self.migrate();
        self.color_vars = crate::colorvars::uses_vars(&self.repo);
        self.color_prompt = false;
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
        self.open_order = self.open.clone();
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
        self.list_picker = None;
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
                // a file without colour variables comes from StarUML 7 or earlier: ask whether to
                // convert its colours (https://docs.staruml.io/user-guide/managing-project/)
                self.color_prompt = !self.color_vars && crate::colorvars::has_fixed_colors(&self.repo);
            }
            Err(e) => self.message = Some(("Failed to open the file.".into(), e)),
        }
    }

    /// File › Import › StarUML 1 File (.uml): the file becomes the new, unsaved project
    /// (extensions/default/staruml-v1/main.js:34 loadFile, project-manager.js:204 loadFromJson).
    pub fn import_staruml_v1(&mut self, p: PathBuf) {
        let text = std::fs::read(&p).map(|b| String::from_utf8_lossy(&b).into_owned()).map_err(|e| e.to_string());
        match text.and_then(|t| crate::staruml_v1::import(&t)) {
            Ok(repo) => self.set_project(repo, None),
            Err(e) => self.message = Some(("Failed to import the file.".into(), e)),
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
        // saving puts the working diagrams in their displayed order (diagram-manager.js:152-160)
        self.open_order = self.open.clone();
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
        let Some(mut p) = rfd::FileDialog::new().set_title("Save As").set_file_name(&name).add_filter("Models", &["mdj"]).add_filter("All Files", &["*"]).save_file() else {
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
            .set_title("Save Changes")
            .set_description("Do you want to save the changes?")
            .set_buttons(rfd::MessageButtons::YesNoCancelCustom("Save".into(), "Don't Save".into(), "Cancel".into()))
            .show();
        // the custom buttons come back by label (some platforms report Yes/No instead)
        match r {
            rfd::MessageDialogResult::Yes => self.save(ms),
            rfd::MessageDialogResult::Custom(b) if b == "Save" => self.save(ms),
            rfd::MessageDialogResult::No => true,
            rfd::MessageDialogResult::Custom(b) if b == "Don't Save" => true,
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
            "UMLInteractionOperand" => "Operand",
            // a flowchart's elements drop "FC" (FCModelElement.getDisplayClassName); its diagram is a
            // "FlowchartDiagram" (flowchart/elements.js:42-75)
            "FCFlowchartDiagram" => "FlowchartDiagram",
            other if other.starts_with("FC") => &other[2..],
            // a data flow element drops "DFD" (DFDElement.getDisplayClassName, dfd/elements.js:42-45)
            other if other.starts_with("DFD") && other != "DFDDiagram" => &other[3..],
            // and an ERD element "ERD" (ERDElement.getDisplayClassName, erd/elements.js:35-38)
            other if other.starts_with("ERD") && other != "ERDDiagram" => &other[3..],
            // and a wireframe element "WF" (WFElement.getDisplayClassName, wireframe/elements.js:42-45)
            other if other.starts_with("WF") && other != "WFWireframeDiagram" => &other[2..],
            // and a SysML element "SysML" (SysMLElementMixin.getDisplayClassName, sysml/elements.js:40-50)
            other if other.starts_with("SysML") => crate::sysml::display_name(other),
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

    /// A view of a constraint at `p` (50×20, grown to its text), joined by a constraint link to each
    /// view of the element that owns it on the diagram (factory.js:899-918).
    pub fn place_constraint(&mut self, d: Eid, c: Eid, p: P) -> Eid {
        let props = [
            ("model", self.repo.ref_to(c)),
            ("font", json!(views::FONT)),
            ("parentStyle", json!(false)),
            ("left", crate::model::num(p.x)),
            ("top", crate::model::num(p.y)),
            ("width", crate::model::num(50.0)),
            ("height", crate::model::num(20.0)),
        ];
        let cv = self.repo.create("UMLConstraintView", &props, Some((d, "ownedViews")));
        self.apply_view_defaults(cv);
        if let Some(owner) = self.repo.parent(c) {
            let targets: Vec<Eid> = self.repo.list(d, "ownedViews").into_iter().filter(|v| *v != cv && self.repo.refd(*v, "model") == Some(owner) && views::is_node(&self.repo, *v)).collect();
            for t in targets {
                self.link_constraint(d, cv, t);
            }
        }
        cv
    }

    fn link_constraint(&mut self, d: Eid, cv: Eid, target: Eid) -> Eid {
        let props = [("font", json!(views::FONT)), ("head", self.repo.ref_to(target)), ("tail", self.repo.ref_to(cv)), ("lineStyle", json!(views::DEFAULT_LINE_STYLE)), ("points", json!("0:0;0:0"))];
        self.repo.create("UMLConstraintLinkView", &props, Some((d, "ownedViews")))
    }

    /// Add Constraint on a box: a constraint owned by its element, shown 50×20 to the left of the
    /// box with a constraint link, as one undo step; its specification edit opens (uml-commands.js:727).
    pub fn add_constraint_to(&mut self, v: Eid) -> Option<Eid> {
        let d = self.diagram?;
        let m = self.repo.refd(v, "model")?;
        // a line's constraint goes left of its name label, like its note goes right (uml-commands.js:727-782)
        let edge = views::is_edge(&self.repo, v);
        let b = if edge { self.edge_anchor(v) } else { views::node_rect(&self.repo, v) };
        let cv = self.change(|a| {
            let name = a.new_name(m, "UMLConstraint");
            let c = a.repo.create("UMLConstraint", &[("name", json!(name))], Some((m, "ownedElements")));
            let p = P { x: (b.x - 50.0 - CONSTRAINT_GAP).max(0.0).round(), y: (if edge { b.y - 20.0 } else { b.y }).max(0.0).round() };
            let cv = a.place_constraint(d, c, p);
            if edge {
                a.link_constraint(d, cv, v);
            }
            Some(cv)
        })?;
        self.select(vec![cv]);
        self.pending_edit = Some(cv);
        Some(cv)
    }

    /// Whether a robustness tool can create its class: its stereotype is in the project, or the person
    /// agreed to apply the UML Standard Profile, which brings it (robustness/main.js:41). Other tools
    /// are always ready.
    pub fn robustness_profile_ready(&mut self, kind: &str) -> bool {
        if !matches!(kind, "Boundary" | "Entity" | "Control") {
            return true;
        }
        let st = kind.to_lowercase();
        if self.repo.all().any(|x| self.repo.ty(x) == "UMLStereotype" && self.repo.name(x) == st) {
            return true;
        }
        // a confirmation box (robustness/main.js:41-67); screenshots do not ask
        let ok = self.screenshot.is_none() && self.confirm(&format!("{kind} requires UML Standard Profile. Do you want to apply?"));
        if ok {
            self.apply_standard_profile();
        }
        ok
    }

    /// Answer a "Select One" dialog with the chosen option.
    pub fn select_one_ok(&mut self, d: SelectOne) {
        match d.purpose {
            SelectPurpose::ConstraintKind(op) if self.repo.alive(op) => {
                let field = OPERATION_CONSTRAINTS.get(d.selected).map(|x| x.1).unwrap_or("ownedElements");
                let c = self.change(|a| {
                    let name = a.new_name(op, "UMLConstraint");
                    a.repo.create("UMLConstraint", &[("name", json!(name))], Some((op, field)))
                });
                self.expanded.insert(op);
                self.focus = Some(c);
                self.reveal(c);
            }
            SelectPurpose::BehaviorKind(owner, field) if self.repo.alive(owner) => {
                let ty = BEHAVIOR_KINDS.get(d.selected).map(|x| x.1).unwrap_or("UMLOpaqueBehavior");
                let b = self.change(|a| {
                    let name = a.new_name(owner, ty);
                    let b = a.repo.create(ty, &[("name", json!(name))], Some((owner, field)));
                    if ty == "UMLStateMachine" {
                        a.repo.create("UMLRegion", &[], Some((b, "regions")));
                    }
                    b
                });
                self.expanded.insert(owner);
                self.reveal(b);
            }
            SelectPurpose::ItemFlow(c) if self.repo.alive(c) => self.add_item_flow(c, d.selected == 1),
            _ => {}
        }
    }

    /// Model › Apply Profile › UML Standard Profile (v2), as one undo step (uml-commands.js:81-100).
    pub fn apply_standard_profile(&mut self) {
        let name = crate::templates::STANDARD_PROFILE;
        if self.repo.all().any(|e| self.repo.ty(e) == "UMLProfile" && self.repo.name(e) == name) {
            self.toast_error(&format!("{name} already exists."));
            return;
        }
        match self.change(crate::templates::add_standard_profile_app) {
            Some(p) => {
                self.reveal(p);
                self.toast(&format!("{name} is successfully applied. Look at under Project."));
            }
            None => self.toast_error(&format!("{name} already exists.")),
        }
    }

    /// A free line from `a` to `b` on the current diagram (common-factory.js:32-44).
    pub fn create_freeline(&mut self, a: P, b: P) -> Option<Eid> {
        let d = self.diagram?;
        let props = [("font", json!(views::FONT)), ("lineStyle", json!(1)), ("points", json!(views::format_points(&[a, b])))];
        let v = self.repo.create("FreelineEdgeView", &props, Some((d, "ownedViews")));
        self.apply_view_defaults(v);
        Some(v)
    }

    /// The region a statechart diagram's states go in: the first region of the state machine that
    /// owns the diagram, made if it has none.
    pub fn statechart_region(&mut self, d: Eid) -> Option<Eid> {
        let sm = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLStateMachine")?;
        match self.repo.list(sm, "regions").into_iter().find(|x| self.repo.alive(*x)) {
            Some(rg) => Some(rg),
            None => Some(self.repo.create("UMLRegion", &[], Some((sm, "regions")))),
        }
    }

    /// The activity an activity diagram's nodes go in: the activity that owns the diagram.
    pub fn diagram_activity(&self, d: Eid) -> Option<Eid> {
        self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLActivity")
    }

    pub fn create_node(&mut self, kind: &str, rect: R) -> Option<Eid> {
        let d = self.diagram?;
        if matches!(kind, "Note" | "Text" | "TextBox" | "Rect" | "RoundRect" | "Ellipse" | "Image") {
            let ty = match kind {
                "Note" => "UMLNoteView",
                "TextBox" => "UMLTextBoxView",
                "Rect" => "RectangleView",
                "RoundRect" => "RoundRectView",
                "Ellipse" => "EllipseView",
                "Image" => "ImageView",
                _ => "UMLTextView",
            };
            let v = views::create_note_view(&mut self.repo, d, rect, ty);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // an activity diagram's nodes go in the nodes of its activity (uml-factory.js:3774-3826); the
        // Send Signal, Accept Signal and Accept Time Event tools make actions of those kinds
        // (uml.json "uml.activity-adv")
        // a swimlane: a partition in the activity's groups (uml.json "UMLSwimlaneVert", "UMLSwimlaneHorz")
        if kind == "SwimlaneVert" || kind == "SwimlaneHorz" {
            let act = self.diagram_activity(d)?;
            let name = self.new_name(act, "UMLActivityPartition");
            let m = self.repo.create("UMLActivityPartition", &[("name", json!(name))], Some((act, "groups")));
            let v = views::create_swimlane_view(&mut self.repo, d, m, rect, kind == "SwimlaneVert");
            self.apply_view_defaults(v);
            return Some(v);
        }
        // a lifeline goes in the participants of the diagram's interaction and represents a new role
        // "Role1"… of the classifier that owns the interaction; it is at least 200 high (lifelineFn,
        // uml-factory.js:1105-1180). Endpoints and gates are participants too.
        // a timing diagram's lifelines, states, time segments and time ticks (uml-factory.js:1105-1180, 1559-1700)
        if let Some(v) = self.create_timing_node(d, kind, rect) {
            if let Some(v) = v {
                self.apply_view_defaults(v);
            }
            return v;
        }
        if matches!(kind, "UMLLifeline" | "UMLEndpoint" | "UMLGate") {
            let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
            let name = self.new_name(inter, kind);
            if kind != "UMLLifeline" {
                let m = self.repo.create(kind, &[("name", json!(name))], Some((inter, "participants")));
                let v = crate::sequence::create_endpoint_view(&mut self.repo, d, m, P { x: rect.x, y: rect.y });
                self.apply_view_defaults(v);
                return Some(v);
            }
            let classifier = self.repo.parent(inter).filter(|c| self.repo.kind_of(*c, "UMLClassifier"))?;
            let names: HashSet<String> = self.repo.list(classifier, "attributes").into_iter().map(|a| self.repo.name(a)).collect();
            let role_name = (1..).map(|i| format!("Role{i}")).find(|n| !names.contains(n)).unwrap();
            let role = self.repo.create("UMLAttribute", &[("name", json!(role_name))], Some((classifier, "attributes")));
            let rr = self.repo.ref_to(role);
            let m = self.repo.create(kind, &[("name", json!(name)), ("represent", rr)], Some((inter, "participants")));
            // on a communication diagram, a box at least 100 × 30 (uml-factory.js:1136-1143)
            let v = if self.repo.ty(d) == "UMLCommunicationDiagram" {
                crate::communication::create_lifeline_view(&mut self.repo, d, m, rect)
            } else {
                crate::sequence::create_lifeline_view(&mut self.repo, d, m, rect)
            };
            self.apply_view_defaults(v);
            return Some(v);
        }
        // an interaction overview's interaction uses and inline interactions (uml-factory.js:1833-1950)
        if matches!(kind, "UMLInteractionUseInOverview" | "UMLInteractionInOverview") {
            return self.create_overview_node(d, kind, rect);
        }
        // a time constraint belongs to the message it is dropped on, its label 20 px under the point and
        // joined to the message; a duration constraint belongs to the interaction (timeConstraintFn,
        // durationConstraintFn, uml-factory.js:1718-1840)
        if kind == "UMLTimeConstraint" {
            // on a message, or on a timing diagram's message or time segment
            let at = P { x: rect.x, y: rect.y };
            let msg = self.repo.list(d, "ownedViews").into_iter().rev().find(|v| match self.repo.ty(*v) {
                "UMLSeqMessageView" | "UMLTimingMessageView" => crate::views::points_of(&self.repo, *v).windows(2).any(|w| crate::canvas::dist_to_seg(at, w[0], w[1]) <= 5.0),
                "UMLTimeSegmentView" => { let b = views::node_rect(&self.repo, *v); at.x >= b.x && at.x <= b.x + b.w && at.y >= b.y && at.y <= b.y + b.h }
                _ => false,
            })?;
            let mm = self.repo.refd(msg, "model")?;
            let names: HashSet<String> = self.repo.children(mm).into_iter().map(|(_, c)| self.repo.name(c)).collect();
            let name = (1..).map(|i| format!("Time Constraint{i}")).find(|n| !names.contains(n)).unwrap();
            let m = self.repo.create(kind, &[("name", json!(name))], Some((mm, "ownedElements")));
            let v = crate::sequence::create_time_views(&mut self.repo, d, m, msg, P { x: rect.x, y: rect.y + 20.0 });
            self.apply_view_defaults(v);
            return Some(v);
        }
        if kind == "UMLDurationConstraint" {
            let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
            let names: HashSet<String> = self.repo.children(inter).into_iter().map(|(_, c)| self.repo.name(c)).collect();
            let name = (1..).map(|i| format!("Duration Constraint{i}")).find(|n| !names.contains(n)).unwrap();
            let m = self.repo.create(kind, &[("name", json!(name))], Some((inter, "ownedElements")));
            let v = crate::sequence::create_duration_view(&mut self.repo, d, m, rect);
            // on a timing diagram, in the frame under its far corner
            if self.repo.ty(d) == "UMLTimingDiagram" {
                let (x2, y2) = (rect.x + rect.w, rect.y + rect.h);
                if let Some(f) = self.repo.list(d, "ownedViews").into_iter().rev().find(|f| self.repo.ty(*f) == "UMLTimingFrameView" && { let b = views::node_rect(&self.repo, *f); x2 >= b.x && x2 <= b.x + b.w && y2 >= b.y && y2 <= b.y + b.h }) {
                    self.set_container(&[v], Some(f));
                }
            }
            self.apply_view_defaults(v);
            return Some(v);
        }
        // combined fragments (with one operand), continuations and interaction uses are fragments of
        // the interaction; a state invariant covers the lifeline it is dropped on and sits on its line
        // (combinedFragmentFn, stateInvariantFn, uml-factory.js:1506-1550, 1687-1716, 3736-3744)
        if matches!(kind, "UMLCombinedFragment" | "UMLContinuation" | "UMLInteractionUse" | "UMLStateInvariant") {
            let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
            let lifeline = (kind == "UMLStateInvariant").then(|| {
                self.repo.list(d, "ownedViews").into_iter().rev().find(|v| self.repo.ty(*v) == "UMLSeqLifelineView" && { let b = views::node_rect(&self.repo, *v); rect.x >= b.x && rect.x <= b.x + b.w && rect.y >= b.y && rect.y <= b.y + b.h })
            });
            if lifeline == Some(None) {
                return None;
            }
            let name = self.new_name(inter, kind);
            let m = self.repo.create(kind, &[("name", json!(name))], Some((inter, "fragments")));
            if kind == "UMLCombinedFragment" {
                self.repo.set(m, "interactionOperator", json!("seq"));
                self.repo.create("UMLInteractionOperand", &[("name", json!("Operand1"))], Some((m, "operands")));
            }
            let v = match lifeline.flatten() {
                Some(lv) => {
                    let lm = self.repo.refd(lv, "model")?;
                    self.repo.set_ref(m, "covered", Some(lm));
                    crate::sequence::create_invariant_view(&mut self.repo, lv, m, rect.y)
                }
                None if kind == "UMLCombinedFragment" => crate::sequence::create_fragment_view(&mut self.repo, d, m, rect),
                None => crate::sequence::create_box_view(&mut self.repo, d, m, rect),
            };
            self.apply_view_defaults(v);
            return Some(v);
        }
        // a BPMN element goes in the diagram's owner, or in the lane or pool it is drawn in; a boundary
        // event on the box under the point (bpmn-factory.js:98-145, toolbox/bpmn.json)
        if let Some(v) = self.create_bpmn_node(d, kind, rect) {
            return v;
        }
        // a cloud element goes in the element that owns the diagram, or in the group or zone it is
        // drawn in, named after its class ("AWSGroup1"; a callout "1"), with its first icon
        // (aws-factory.js:70-95; defaultModelAndViewFn, factory.js:520-560)
        if crate::cloud::is_node_type(kind) {
            let owner = self.repo.parent(d)?;
            let vt = format!("{kind}View");
            let holder = self.repo.list(d, "ownedViews").into_iter().rev().find(|c| crate::cloud::can_contain(self.repo.ty(*c), &vt) && { let b = views::node_rect(&self.repo, *c); rect.x >= b.x && rect.x <= b.x + b.w && rect.y >= b.y && rect.y <= b.y + b.h });
            let parent = holder.and_then(|c| self.repo.refd(c, "model")).unwrap_or(owner);
            let name = if kind.ends_with("Callout") { "1".to_string() } else { self.new_name(parent, kind) };
            let m = self.repo.create(kind, &[("name", json!(name))], Some((parent, "ownedElements")));
            crate::cloud::init_model(&mut self.repo, m);
            let v = crate::cloud::create_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            crate::cloud::init_view(&mut self.repo, v);
            if let Some(c) = holder {
                self.set_container(&[v], Some(c));
            }
            return Some(v);
        }
        if let Some(v) = self.create_sysml_node(d, kind, rect) {
            return v;
        }
        // a C4 element goes in the element that owns the diagram, named after its class ("C4Person1"…);
        // the container tools set its kind (c4-factory.js:70-90, toolbox/c4.json)
        if let Some((ty, ckind)) = crate::c4::tool(kind) {
            let owner = self.repo.parent(d)?;
            let names: HashSet<String> = self.repo.children(owner).into_iter().map(|(_, c)| self.repo.name(c)).collect();
            let name = (1..).map(|i| format!("{ty}{i}")).find(|n| !names.contains(n)).unwrap();
            let m = self.repo.create(ty, &[("name", json!(name))], Some((owner, "ownedElements")));
            if ty == "C4Container" {
                self.repo.set(m, "kind", json!(ckind.unwrap_or("etc")));
            }
            let v = crate::c4::create_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // an entity goes in the data model that owns the diagram (erd-factory.js:70-75)
        if kind == "ERDEntity" {
            let owner = self.repo.parent(d)?;
            let name = self.new_name(owner, kind);
            let m = self.repo.create(kind, &[("name", json!(name))], Some((owner, "ownedElements")));
            let v = crate::erd::create_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // a data flow or mind map node goes in the element that owns the diagram (dfd-factory.js:70-80,
        // mindmap-factory.js:68-80)
        if crate::notations::is_node_type(kind) {
            let owner = self.repo.parent(d)?;
            let base = crate::notations::name_base(kind);
            let names: HashSet<String> = self.repo.children(owner).into_iter().map(|(_, c)| self.repo.name(c)).collect();
            let name = (1..).map(|i| format!("{base}{i}")).find(|n| !names.contains(n)).unwrap();
            let m = self.repo.create(kind, &[("name", json!(name))], Some((owner, "ownedElements")));
            let v = crate::notations::create_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // a wireframe widget goes in the wireframe that owns the diagram, or in the frame, panel or tab
        // list it is drawn in, named after its class ("Button1"; wireframe-factory.js:75-95,
        // elements.js:42-45, diagram-editor.js containerView)
        if crate::wireframe::is_node_type(kind) {
            let owner = self.repo.parent(d)?;
            let vt = format!("{kind}View");
            let holder = self.repo.list(d, "ownedViews").into_iter().rev().find(|c| crate::wireframe::can_contain(self.repo.ty(*c), &vt) && { let b = views::node_rect(&self.repo, *c); rect.x >= b.x && rect.x <= b.x + b.w && rect.y >= b.y && rect.y <= b.y + b.h });
            let parent = holder.and_then(|c| self.repo.refd(c, "model")).unwrap_or(owner);
            let name = self.new_name(parent, kind);
            let m = self.repo.create(kind, &[("name", json!(name))], Some((parent, "ownedElements")));
            let v = crate::wireframe::create_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            crate::wireframe::link_color(&mut self.repo, v);
            if let Some(c) = holder {
                self.set_container(&[v], Some(c));
            }
            return Some(v);
        }
        // a flowchart node goes in the flowchart that owns the diagram (flowchart-factory.js:73-110)
        if crate::flowchart::is_node_type(kind) {
            let owner = self.repo.parent(d)?;
            let name = match crate::flowchart::name_base(kind) {
                Some("") => String::new(),
                Some(base) => {
                    let names: HashSet<String> = self.repo.children(owner).into_iter().map(|(_, c)| self.repo.name(c)).collect();
                    (1..).map(|i| format!("{base}{i}")).find(|n| !names.contains(n)).unwrap()
                }
                None => self.new_name(owner, kind),
            };
            let m = self.repo.create(kind, &[("name", json!(name))], Some((owner, "ownedElements")));
            let v = crate::flowchart::create_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // an interruptible region is a group of the activity (uml-factory.js:3836-3840)
        if kind == "UMLInterruptibleActivityRegion" {
            let act = self.diagram_activity(d)?;
            let name = self.new_name(act, kind);
            let m = self.repo.create(kind, &[("name", json!(name))], Some((act, "groups")));
            let v = views::create_node_view(&mut self.repo, d, m, rect)?;
            self.apply_view_defaults(v);
            return Some(v);
        }
        let action = views::action_kind(kind);
        if action.is_some()
            || matches!(
                kind,
                "UMLObjectNode" | "UMLCentralBufferNode" | "UMLDataStoreNode" | "UMLActivityParameterNode" | "UMLInitialNode" | "UMLActivityFinalNode" | "UMLFlowFinalNode" | "UMLForkNode" | "UMLJoinNode" | "UMLMergeNode" | "UMLDecisionNode" | "UMLStructuredActivityNode" | "UMLExpansionRegion" | "UMLActivityEdgeConnector"
            )
        {
            let act = self.diagram_activity(d)?;
            let ty = if action.is_some() { "UMLAction" } else { kind };
            // an edge connector is named "A" (uml-factory.js:3872-3880)
            let name = if kind == "UMLActivityEdgeConnector" { "A".to_string() } else { self.new_name(act, ty) };
            let m = self.repo.create(ty, &[("name", json!(name))], Some((act, "nodes")));
            if let Some(k) = action.filter(|k| *k != "opaque") {
                self.repo.set(m, "kind", json!(k));
            }
            let v = views::create_node_view(&mut self.repo, d, m, rect)?;
            self.apply_view_defaults(v);
            // drawn inside a swimlane, region or structured activity: it goes in that container
            let holder = self.repo.list(d, "ownedViews").into_iter().rev().find(|c| *c != v && self.can_contain_view(*c, v) && { let b = views::node_rect(&self.repo, *c); rect.x >= b.x && rect.x <= b.x + b.w && rect.y >= b.y && rect.y <= b.y + b.h });
            if let Some(c) = holder {
                self.set_container(&[v], Some(c));
            }
            return Some(v);
        }
        // a statechart's states go in the first region of its state machine (uml-factory.js:2005)
        let pseudo = views::pseudostate_kind(kind);
        if pseudo.is_some() || matches!(kind, "UMLState" | "UMLFinalState" | "CompositeState" | "SubmachineState" | "OrthogonalState") {
            let region = self.statechart_region(d)?;
            let ty = if pseudo.is_some() { "UMLPseudostate" } else if kind == "UMLFinalState" { kind } else { "UMLState" };
            // pseudostates and final states are made unnamed at their smallest size, whatever box is
            // dragged (pseudostateFn, finalStateFn, uml-factory.js:2116-2180)
            let mark = ty != "UMLState";
            let name = if mark { String::new() } else { self.new_name(region, ty) };
            let rect = if mark {
                let (w, h) = views::default_size(kind);
                R { w, h, ..rect }
            } else {
                rect
            };
            let m = self.repo.create(ty, &[("name", json!(name))], Some((region, "vertices")));
            if let Some(k) = pseudo {
                self.repo.set(m, "kind", json!(k));
            }
            // a composite state has one region, an orthogonal state two
            let regions = match kind {
                "CompositeState" => 1,
                "OrthogonalState" => 2,
                _ => 0,
            };
            for _ in 0..regions {
                self.repo.create("UMLRegion", &[], Some((m, "regions")));
            }
            // a submachine state comes with a new state machine of its own, with one region, next to
            // the diagram's state machine (uml-factory.js:2046; where it goes is estimated)
            if kind == "SubmachineState" {
                let sm = self.repo.parent(d)?;
                let owner = self.repo.parent(sm).unwrap_or(self.repo.root);
                let name = self.new_name(owner, "UMLStateMachine");
                let sub = self.repo.create("UMLStateMachine", &[("name", json!(name))], Some((owner, "ownedElements")));
                self.repo.create("UMLRegion", &[], Some((sub, "regions")));
                self.repo.set_ref(m, "submachine", Some(sub));
            }
            let v = views::create_node_view(&mut self.repo, d, m, rect)?;
            self.apply_view_defaults(v);
            // drawn inside a state: it goes in that state (its first region)
            let host = self.repo.list(d, "ownedViews").into_iter().rev().find(|c| *c != v && self.repo.ty(*c) == "UMLStateView" && { let b = views::node_rect(&self.repo, *c); rect.x >= b.x && rect.x <= b.x + b.w && rect.y >= b.y && rect.y <= b.y + b.h });
            if let Some(c) = host {
                self.set_container(&[v], Some(c));
            }
            return Some(v);
        }
        let parent = self.package_for(Some(d));
        let name = self.new_name(parent, kind);
        // an n-ary association node starts without a name (uml.json "model-init")
        if kind == "UMLNaryAssociationNode" {
            let m = self.repo.create(kind, &[("name", json!(""))], Some((parent, "ownedElements")));
            let v = views::create_nary_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // a hyperlink is a model element of the diagram's owner, linking to nothing yet
        if kind == "Hyperlink" {
            let m = self.repo.create("Hyperlink", &[("name", json!(name))], Some((parent, "ownedElements")));
            let v = views::create_hyperlink_view(&mut self.repo, d, m, rect);
            self.apply_view_defaults(v);
            return Some(v);
        }
        // the Robustness tools make a class stereotyped «boundary», «entity» or «control», shown as an
        // icon without compartments (robustness/main.js:17)
        let robust = matches!(kind, "Boundary" | "Entity" | "Control");
        let ty = if robust { "UMLClass" } else { kind };
        let m = self.repo.create(ty, &[("name", json!(name))], Some((parent, "ownedElements")));
        add_stereotype_icon(&mut self.repo, m);
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
        // a timing frame holds lifelines, a state time segments (elements.js:13440-13460)
        if let Some(ok) = crate::timing::can_contain(ct, self.repo.ty(v)) {
            return ok && container != v;
        }
        // a BPMN pool holds lanes and flow elements, a lane flow elements (bpmn/elements.js:2898-2910, 3007-3016)
        if crate::bpmn::is_node_view(ct) || crate::bpmn::is_node_view(self.repo.ty(v)) {
            let mut c = Some(container);
            while let Some(x) = c {
                if x == v {
                    return false;
                }
                c = self.repo.refd(x, "containerView");
            }
            return crate::bpmn::can_contain(ct, self.repo.ty(v));
        }
        // a cloud group or zone holds the other boxes of its notation (canContainViewKind)
        if crate::cloud::is_node_view(ct) || crate::cloud::is_node_view(self.repo.ty(v)) {
            let mut c = Some(container);
            while let Some(x) = c {
                if x == v {
                    return false;
                }
                c = self.repo.refd(x, "containerView");
            }
            return crate::cloud::can_contain(ct, self.repo.ty(v));
        }
        // a wireframe frame or panel holds widgets, a tab list holds tabs (wireframe/elements.js)
        if crate::wireframe::is_node_view(ct) || crate::wireframe::is_node_view(self.repo.ty(v)) {
            let mut c = Some(container);
            while let Some(x) = c {
                if x == v {
                    return false;
                }
                c = self.repo.refd(x, "containerView");
            }
            return crate::wireframe::can_contain(ct, self.repo.ty(v));
        }
        // a state holds states and pseudostates, which then belong to its first region
        let vertex = |x: Eid| self.repo.refd(x, "model").map(|m| self.repo.kind_of(m, "UMLVertex")).unwrap_or(false);
        if ct == "UMLStateView" && !vertex(v) {
            return false;
        }
        // a swimlane holds actions, control nodes and object nodes (UMLSwimlaneView.canContainViewKind,
        // elements.js:10574-10581)
        // and so does an interruptible region; a structured activity or expansion region also holds
        // structured activities (elements.js:10783-10790, 10845-10853)
        if matches!(ct, "UMLSwimlaneView" | "UMLInterruptibleActivityRegionView" | "UMLStructuredActivityNodeView" | "UMLExpansionRegionView") {
            let vt = self.repo.ty(v);
            let structured = matches!(vt, "UMLStructuredActivityNodeView" | "UMLExpansionRegionView");
            if structured && !matches!(ct, "UMLStructuredActivityNodeView" | "UMLExpansionRegionView") {
                return false;
            }
            // not itself or a box that holds it
            let mut c = Some(container);
            while let Some(x) = c {
                if x == v {
                    return false;
                }
                c = self.repo.refd(x, "containerView");
            }
            return structured || matches!(vt, "UMLActionView" | "UMLControlNodeView" | "UMLObjectNodeView" | "UMLCentralBufferNodeView" | "UMLDataStoreNodeView" | "UMLActivityParameterNodeView" | "UMLActivityEdgeConnectorView" | "UMLFinalStateView");
        }
        if !(ct == "UMLPackageView" || ct == "UMLModelView" || ct == "UMLSubsystemView" || ct == "UMLUseCaseSubjectView" || ct == "UMLStateView") || !views::is_node(&self.repo, v) {
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
            // a time segment moved to another state takes that state and its lifeline; other timing
            // views keep their models where they are
            if self.repo.ty(v).starts_with("UMLTim") || self.repo.ty(v) == "UMLDurationConstraintView" {
                if let (Some(c), "UMLTimeSegmentView") = (to, self.repo.ty(v)) {
                    crate::timing::segment_moved(&mut self.repo, v, c);
                }
                continue;
            }
            // a state or pseudostate goes in the first region of the state it is put in, or of the
            // diagram's state machine when released (uml/main.js:24-41)
            if let Some(vm) = self.repo.refd(v, "model").filter(|m| self.repo.kind_of(*m, "UMLVertex")) {
                let region = match to.and_then(|c| self.repo.refd(c, "model").map(|s| (c, s))).filter(|(_, s)| self.repo.kind_of(*s, "UMLState")) {
                    Some((c, s)) => {
                        // regions split the state's height evenly, top to bottom (estimated)
                        let rs: Vec<Eid> = self.repo.list(s, "regions").into_iter().filter(|x| self.repo.alive(*x)).collect();
                        let (cb, vb) = (views::node_rect(&self.repo, c), views::node_rect(&self.repo, v));
                        let i = if rs.len() > 1 { (((vb.y - cb.y) / cb.h.max(1.0) * rs.len() as f64).floor().max(0.0) as usize).min(rs.len() - 1) } else { 0 };
                        Some(match rs.get(i) {
                            Some(&rg) => rg,
                            None => self.repo.create("UMLRegion", &[], Some((s, "regions"))),
                        })
                    }
                    None => self.statechart_region(d),
                };
                if let Some(rg) = region.filter(|rg| self.repo.parent(vm) != Some(*rg)) {
                    self.repo.relocate(vm, rg, "vertices");
                }
                continue;
            }
            // an activity node goes in the nodes of the partition it is put in, or of the diagram's
            // activity when released (uml/main.js:24-41; engine.js:1158-1166)
            if let Some(vm) = self.repo.refd(v, "model").filter(|m| self.repo.kind_of(*m, "UMLActivityNode")) {
                let target = match to.and_then(|c| self.repo.refd(c, "model")) {
                    Some(pm) if self.repo.kind_of(pm, "UMLActivityGroup") || self.repo.kind_of(pm, "UMLStructuredActivityNode") => Some(pm),
                    _ => self.diagram_activity(d),
                };
                if let Some(t) = target.filter(|t| self.repo.parent(vm) != Some(*t) && self.repo.field_of(vm).as_deref() == Some("nodes")) {
                    self.repo.relocate(vm, t, "nodes");
                }
                continue;
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
        // BPMN pools and lanes stay behind what they hold
        if self.repo.ty(d) == "BPMNDiagram" {
            crate::bpmn::restack(&mut self.repo, d);
        }
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
    /// Whether a diagram of this type takes a view of the element (each diagram's canAcceptModel,
    /// elements.js:5900-13990). Kinds UMLCraft only reads accept anything.
    pub fn diagram_accepts(&self, d: Eid, m: Eid) -> bool {
        const CLASS: &[&str] = &[
            "Hyperlink", "Diagram", "UMLConstraint", "UMLPackage", "UMLClassifier", "UMLInstance", "UMLPort", "UMLTemplateParameter", "UMLAttribute", "UMLOperation", "UMLReception",
            "UMLEnumerationLiteral", "UMLSlot", "UMLGeneralization", "UMLDependency", "UMLInterfaceRealization", "UMLComponentRealization", "UMLAssociation", "UMLLink", "UMLConnector",
        ];
        let kinds: Vec<&str> = match self.repo.ty(d) {
            "UMLClassDiagram" => CLASS.to_vec(),
            "UMLInformationFlowDiagram" => CLASS.iter().copied().chain(["UMLInformationFlow"]).collect(),
            // component and deployment diagrams: the class diagram's list without slots
            "UMLComponentDiagram" | "UMLDeploymentDiagram" => CLASS.iter().copied().filter(|k| *k != "UMLSlot").collect(),
            "UMLPackageDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLPackage", "UMLSubsystem", "UMLClassifier", "UMLInstance", "UMLDependency"],
            "UMLCompositeStructureDiagram" => vec![
                "Hyperlink", "Diagram", "UMLConstraint", "UMLPackage", "UMLClassifier", "UMLPort", "UMLTemplateParameter", "UMLAttribute", "UMLOperation", "UMLReception",
                "UMLEnumerationLiteral", "UMLCollaborationUse", "UMLGeneralization", "UMLDependency", "UMLInterfaceRealization", "UMLComponentRealization", "UMLAssociation", "UMLLink", "UMLConnector",
            ],
            "UMLObjectDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLClassifier", "UMLInstance", "UMLSlot", "UMLDependency", "UMLLink"],
            "UMLStatechartDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLVertex", "UMLTransition", "UMLStateMachine"],
            // UMLActivityDiagram.canAcceptModel takes pins too (elements.js:9638-9649)
            "UMLActivityDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLActivityNode", "UMLActivityEdge", "UMLActivity", "UMLPin"],
            // FCFlowchartDiagram.canAcceptModel (flowchart/elements.js:68-75)
            "FCFlowchartDiagram" => vec!["Hyperlink", "Diagram", "FCModelElement", "FCFlow"],
            "DFDDiagram" => vec!["Hyperlink", "Diagram", "DFDExternalEntity", "DFDProcess", "DFDDataStore", "DFDDataFlow"],
            "MMMindmapDiagram" => vec!["Hyperlink", "Diagram", "MMNode", "MMEdge"],
            // WFWireframeDiagram.canAcceptModel (wireframe/elements.js:55-61)
            "WFWireframeDiagram" => vec!["Hyperlink", "Diagram", "WFElement"],
            // ERDDiagram.canAcceptModel (erd/elements.js:40-47)
            "ERDDiagram" => vec!["Hyperlink", "Diagram", "ERDEntity", "ERDColumn", "ERDRelationship"],
            // C4Diagram.canAcceptModel (c4/elements.js:75-80)
            "C4Diagram" => vec!["Hyperlink", "Diagram", "C4Element", "C4Relationship"],
            // BPMNDiagram.canAcceptModel (bpmn/elements.js:65-80)
            "BPMNDiagram" => vec!["Hyperlink", "Diagram", "BPMNParticipant", "BPMNLane", "BPMNActivity", "BPMNEvent", "BPMNGateway", "BPMNSequenceFlow", "BPMNMessage", "BPMNMessageFlow"],
            // AWSDiagram.canAcceptModel and the others' (aws/elements.js:105-115)
            t if crate::cloud::is_diagram(t) => crate::cloud::accepted(t).to_vec(),
            // UMLInteractionOverviewDiagram.canAcceptModel (elements.js:13823-13834)
            "UMLInteractionOverviewDiagram" => vec!["Hyperlink", "Diagram", "UMLInteractionUse", "UMLInteraction"],
            // UMLSequenceDiagram.canAcceptModel (elements.js:10958-10985), and
            // UMLCommunicationDiagram's and UMLTimingDiagram's (elements.js:12914-12946, 13374-13389)
            "UMLSequenceDiagram" | "UMLCommunicationDiagram" | "UMLTimingDiagram" => {
                // a role of the collaboration that owns the interaction
                if self.repo.ty(m) == "UMLAttribute" {
                    return self.repo.parent(m) == self.repo.parent(d).and_then(|i| self.repo.parent(i));
                }
                vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLClassifier"]
            }
            "UMLUseCaseDiagram" => vec![
                "Hyperlink", "Diagram", "UMLConstraint", "UMLPackage", "UMLClassifier", "UMLUseCaseSubject", "UMLInstance", "UMLExtensionPoint", "UMLTemplateParameter", "UMLAttribute",
                "UMLOperation", "UMLReception", "UMLGeneralization", "UMLDependency", "UMLInterfaceRealization", "UMLComponentRealization", "UMLAssociation", "UMLLink", "UMLConnector",
            ],
            "UMLProfileDiagram" => vec!["Hyperlink", "Diagram", "UMLConstraint", "UMLMetaClass", "UMLStereotype", "UMLEnumeration", "UMLGeneralization", "UMLDependency", "UMLInterfaceRealization", "UMLComponentRealization", "UMLAssociation"],
            t if crate::sysml::accepts(t).is_some() => crate::sysml::accepts(t).unwrap_or_default(),
            _ => return true,
        };
        kinds.iter().any(|k| self.repo.kind_of(m, k))
    }

    pub fn can_drop_model(&self, m: Eid) -> bool {
        if !self.diagram.map(|d| self.diagram_accepts(d, m)).unwrap_or(false) {
            return false;
        }
        let ty = self.repo.ty(m);
        // the open diagram dropped onto itself makes a frame of it (factory.js:1174-1185)
        // ports, parts, pins and connection point references go onto their parent's box (drop_feature)
        let feature = matches!(ty, "UMLPort" | "UMLAttribute" | "UMLInputPin" | "UMLOutputPin" | "UMLConnectionPointReference");
        self.diagram.is_some() && self.repo.alive(m) && (self.is_diagram(m) || feature || views::node_kind(ty).is_some() || crate::bpmn::is_node_type(ty) || crate::sysml::droppable(ty) || ty == "UMLStateMachine" || ty == "UMLActivity" || ty == "UMLConstraint" || (views::edge_view_of(ty).is_some() && self.rel_ends(m).is_some()))
    }

    pub fn drop_model(&mut self, m: Eid, p: P) -> Result<Vec<Eid>, String> {
        let Some(d) = self.diagram else { return Ok(vec![]) };
        // the drop is checked again (app-context.js:1155-1157)
        if !self.diagram_accepts(d, m) {
            return Ok(vec![]);
        }
        let ty = self.repo.ty(m).to_string();
        // a hidden view of it inside the box under the drop point, such as an attribute hidden by
        // Delete, is shown again (app-context.js:1158-1183)
        if let Some(c) = self.hit_view(p, true) {
            let mut stack = self.repo.list(c, "subViews");
            while let Some(s) = stack.pop() {
                if self.repo.refd(s, "model") == Some(m) && self.repo.raw(s, "visible") == Some(&json!(false)) {
                    self.change(|a| a.repo.set(s, "visible", json!(true)));
                    return Ok(vec![s]);
                }
                stack.extend(self.repo.list(s, "subViews"));
            }
        }
        if m == d {
            // a 700×600 frame of the diagram, its top left at the drop point (factory.js:1174-1185)
            return Ok(self.create_frame(d, Some(d), views::R { x: p.x, y: p.y, w: 700.0, h: 600.0 }).into_iter().collect());
        }
        if self.is_diagram(m) {
            // another diagram: a hyperlink to it, at its minimum size (factory.js:1186-1198)
            let v = self.change(|a| {
                let v = a.create_node("Hyperlink", views::R { x: p.x, y: p.y, w: 0.0, h: 0.0 })?;
                let link = a.repo.refd(v, "model")?;
                a.repo.set_ref(link, "reference", Some(m));
                Some(v)
            });
            if let Some(link) = v.and_then(|v| self.repo.refd(v, "model")) {
                self.reveal(link);
            }
            return Ok(v.into_iter().collect());
        }
        if let Some(res) = self.drop_feature(d, m, p) {
            return res;
        }
        if views::edge_view_of(&ty).is_some() && self.rel_ends(m).is_some() {
            return self.drop_relationship(d, m, p);
        }
        if let Some(res) = self.drop_sysml(d, m, p) {
            return res;
        }
        // a classifier dropped on a sequence diagram becomes a lifeline representing a new role of that
        // type; a role of the collaboration a lifeline representing it (viewForSequenceDiagramFn,
        // uml-factory.js:2763-2850)
        // an interaction use, an interaction or a sequence diagram on an interaction overview diagram
        // becomes an action calling it
        if self.repo.ty(d) == "UMLInteractionOverviewDiagram" && (matches!(self.repo.ty(m), "UMLInteractionUse" | "UMLInteraction" | "UMLSequenceDiagram")) {
            let v = self.change(|a| a.drop_on_overview(d, m, p));
            return Ok(v.into_iter().collect());
        }
        let comm = self.repo.ty(d) == "UMLCommunicationDiagram";
        let timing = self.repo.ty(d) == "UMLTimingDiagram";
        if (self.repo.ty(d) == "UMLSequenceDiagram" || comm || timing) && (self.repo.kind_of(m, "UMLClassifier") || self.repo.ty(m) == "UMLAttribute") {
            let v = self.change(|a| {
                // a communication lifeline at its smallest (viewForCommunicationDiagramFn, uml-factory.js:2981-3070)
                let h = if comm { crate::communication::LIFELINE_MIN.1 } else if timing { 1.0 } else { 200.0 };
                let v = a.create_node("UMLLifeline", views::R { x: p.x, y: p.y, w: 100.0, h })?;
                let lm = a.repo.refd(v, "model")?;
                let role = a.repo.refd(lm, "represent")?;
                if a.repo.ty(m) == "UMLAttribute" {
                    a.repo.set_ref(lm, "represent", Some(m));
                    a.repo.detach(role);
                } else {
                    a.repo.set_ref(role, "type", Some(m));
                }
                Some(v)
            });
            return Ok(v.into_iter().collect());
        }
        // a constraint gets its view and links to its owner's boxes (factory.js:899-918)
        if ty == "UMLConstraint" {
            let v = self.change(|a| Some(a.place_constraint(d, m, p)));
            return Ok(v.into_iter().collect());
        }
        // an activity dropped on an activity diagram becomes an action without a name whose
        // subactivity is that activity (uml-factory.js:3450-3480)
        if ty == "UMLActivity" {
            let v = self.change(|a| {
                let (w, h) = views::default_size("UMLAction");
                let v = a.create_node("UMLAction", views::R { x: p.x, y: p.y, w, h })?;
                let s = a.repo.refd(v, "model")?;
                a.repo.set(s, "name", json!(""));
                a.repo.set_ref(s, "subactivity", Some(m));
                Some(v)
            });
            return Ok(v.into_iter().collect());
        }
        // a state machine dropped on a statechart becomes a submachine state that refers to it
        // (uml-factory.js:3407)
        if ty == "UMLStateMachine" {
            let v = self.change(|a| {
                let (w, h) = views::default_size("UMLState");
                let v = a.create_node("UMLState", views::R { x: p.x, y: p.y, w, h })?;
                let s = a.repo.refd(v, "model")?;
                a.repo.set_ref(s, "submachine", Some(m));
                Some(v)
            });
            if let Some(s) = v.and_then(|v| self.repo.refd(v, "model")) {
                self.reveal(s);
            }
            return Ok(v.into_iter().collect());
        }
        if views::node_kind(&ty).is_none() && !crate::flowchart::is_node_type(&ty) && !crate::notations::is_node_type(&ty) && ty != "ERDEntity" && !crate::c4::is_node_type(&ty) && !crate::wireframe::is_node_type(&ty) && !crate::bpmn::is_node_type(&ty) && !crate::cloud::is_node_type(&ty) && crate::sysml::view_of(&ty).is_none() {
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

    /// A port, part, pin or connection point reference dragged from the explorer goes onto its
    /// parent's box under the point; elsewhere it is refused with the original's alert: a port on
    /// its classifier or on a part of that type, a part (any attribute) on its classifier, a pin on
    /// its action, a connection point reference on its state (viewForGeneralDiagramFn,
    /// viewForStatechartDiagramFn, viewForActivityDiagramFn, uml-factory.js:2466-2510, 3412-3478).
    fn drop_feature(&mut self, d: Eid, m: Eid, p: P) -> Option<Result<Vec<Eid>, String>> {
        let ty = self.repo.ty(m).to_string();
        let dt = self.repo.ty(d).to_string();
        let general = matches!(dt.as_str(), "UMLClassDiagram" | "UMLPackageDiagram" | "UMLObjectDiagram" | "UMLComponentDiagram" | "UMLDeploymentDiagram" | "UMLProfileDiagram" | "UMLUseCaseDiagram");
        let parent = self.repo.parent(m);
        let host = self.hit_view(p, true);
        let hm = host.and_then(|h| self.repo.refd(h, "model"));
        let hty = host.map(|h| self.repo.ty(h).to_string()).unwrap_or_default();
        let on_parent = hm.is_some() && hm == parent;
        let classifier_box = on_parent && hm.map(|x| self.repo.kind_of(x, "UMLClassifier")).unwrap_or(false) && views::node_kind(self.repo.ty(hm.unwrap())).map(|k| k.view == hty).unwrap_or(false);
        let (ok, refused) = match ty.as_str() {
            "UMLPort" if general => (classifier_box || (hty == "UMLPartView" && hm.and_then(|x| self.repo.refd(x, "type")) == parent), "Port should be dropped on it's parent Classifier or Part"),
            "UMLAttribute" if general => (classifier_box, "Part (Attribute) should be dropped on it's parent Classifier"),
            "UMLInputPin" | "UMLOutputPin" if dt == "UMLActivityDiagram" => (on_parent && hty == "UMLActionView", "Pin should be dropped on it's parent Action"),
            "UMLConnectionPointReference" if dt == "UMLStatechartDiagram" => (on_parent && hty == "UMLStateView", "ConnectionPointReference should be dropped on it's parent State"),
            _ => return None,
        };
        let Some(h) = host.filter(|_| ok) else { return Some(Err(refused.into())) };
        let v = self.change(|a| {
            let v = match ty.as_str() {
                "UMLPort" => views::create_port_view(&mut a.repo, d, m, h, p),
                "UMLAttribute" => views::create_part_view(&mut a.repo, d, m, h, R { x: p.x, y: p.y, w: 0.0, h: 0.0 }),
                "UMLConnectionPointReference" => views::create_connection_point_view(&mut a.repo, d, m, h, p),
                _ => views::create_pin_view(&mut a.repo, d, m, h, p),
            };
            a.apply_view_defaults(v);
            v
        });
        Some(Ok(vec![v]))
    }

    fn drop_relationship(&mut self, d: Eid, m: Eid, p: P) -> Result<Vec<Eid>, String> {
        let (t, h) = self.rel_ends(m).unwrap();
        if let Some(e) = self.shown_view(d, m) {
            let mut sel = vec![e];
            sel.extend([self.repo.refd(e, "tail"), self.repo.refd(e, "head")].into_iter().flatten());
            self.select(sel);
            // a connector's own words (uml-factory.js:2525, 2657)
            let what = if self.repo.ty(m) == "UMLConnector" { "Connector" } else { "Relationship" };
            return Err(format!("{what} View is already existed in this Diagram."));
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
            Some(m) if self.repo.ty(m) == "UMLLinkObjectLink" => "LinkObject".into(),
            Some(m) => self.repo.ty(m).to_string(),
            None if self.repo.ty(e) == "UMLNoteLinkView" => "NoteLink".into(),
            None if self.repo.ty(e) == "UMLContainmentView" => "Containment".into(),
            None if self.repo.ty(e) == "UMLConstraintLinkView" => "ConstraintLink".into(),
            None => String::new(),
        }
    }

    /// Whether a line of this kind may join these two views.
    pub fn can_link(&self, kind: &str, tail: Eid, head: Eid) -> bool {
        let r = &self.repo;
        if kind == "NoteLink" {
            return r.ty(tail) == "UMLNoteView" || r.ty(head) == "UMLNoteView";
        }
        if kind == "ConstraintLink" {
            return r.ty(tail) == "UMLConstraintView" && r.ty(head) != "UMLConstraintView";
        }
        // a link object: between two objects, or from an object to a link that has none yet
        if kind == "LinkObject" {
            let (Some(tm), Some(hm)) = (r.refd(tail, "model"), r.refd(head, "model")) else { return false };
            if views::is_edge(r, tail) || r.ty(tm) != "UMLObject" {
                return false;
            }
            if views::is_edge(r, head) {
                let linked = r.all_of("UMLLinkObjectLink").into_iter().any(|l| r.refd(l, "linkSide") == Some(hm));
                return r.ty(hm) == "UMLLink" && !linked;
            }
            return r.kind_of(hm, "UMLInstance");
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
            // with an n-ary association node, the other end is the class it links (uml-factory.js:743-761)
            let nary = |m: Eid| r.ty(m) == "UMLNaryAssociationNode";
            if nary(tm) || nary(hm) {
                return (nary(tm) && r.ty(hm) == "UMLClass") || (nary(hm) && r.ty(tm) == "UMLClass");
            }
            return r.kind_of(tm, "UMLClassifier") && r.kind_of(hm, "UMLClassifier");
        }
        // a communication diagram's connectors and messages (uml-factory.js:1231-1330, 1956-1965)
        if let Some(ok) = crate::communication::can_link(r, kind, tail, head) {
            return ok;
        }
        // BPMN lines (bpmn-factory.js:20-25)
        if let Some(ok) = crate::bpmn::can_link(r, kind, tail, head) {
            return ok;
        }
        if views::is_edge(r, tail) || views::is_edge(r, head) {
            return false;
        }
        let (Some(tm), Some(hm)) = (r.refd(tail, "model"), r.refd(head, "model")) else { return false };
        if let Some(ok) = crate::sysml::can_link(self, kind, tm, hm) {
            return ok;
        }
        if kind == "Containment" {
            // the tail's element goes into the head's package (shown in the explorer)
            return r.kind_of(hm, "UMLPackage") && tm != hm && (r.parent(tm) == Some(hm) || self.can_relocate(tm, hm));
        }
        let classifier = |m: Eid| r.kind_of(m, "UMLClassifier");
        match kind {
            "UMLAssociation" => classifier(tm) && classifier(hm),
            // a connector joins two features: ports, parts or attributes (uml-factory.js:32-38)
            "UMLConnector" => r.kind_of(tm, "UMLFeature") && r.kind_of(hm, "UMLFeature"),
            // a role binding: from a collaboration use to a part or attribute (uml-factory.js:48-54)
            "UMLRoleBinding" => r.ty(tm) == "UMLCollaborationUse" && r.kind_of(hm, "UMLAttribute"),
            // a link joins two objects (uml-factory.js)
            "UMLLink" => r.kind_of(tm, "UMLInstance") && r.kind_of(hm, "UMLInstance"),
            // deployment diagrams: a path joins two nodes, a deployment puts a classifier on a node and a
            // component realization realizes a component (uml-factory.js:19-146)
            "UMLCommunicationPath" => r.ty(tm) == "UMLNode" && r.ty(hm) == "UMLNode",
            "UMLDeployment" => classifier(tm) && r.ty(hm) == "UMLNode" && tm != hm,
            "UMLComponentRealization" => classifier(tm) && r.ty(hm) == "UMLComponent" && tm != hm,
            "UMLInformationFlow" => tm != hm,
            // a stereotype extends a metaclass
            "UMLExtension" => r.ty(tm) == "UMLStereotype" && r.ty(hm) == "UMLMetaClass",
            // the bound classifier points at its template
            "UMLTemplateBinding" => classifier(tm) && classifier(hm) && tm != hm,
            "UMLGeneralization" => classifier(tm) && classifier(hm) && tm != hm,
            "UMLInterfaceRealization" => classifier(tm) && r.ty(hm) == "UMLInterface",
            "UMLInclude" | "UMLExtend" => r.ty(tm) == "UMLUseCase" && r.ty(hm) == "UMLUseCase",
            // a transition joins two vertices: states and pseudostates
            "UMLTransition" => r.kind_of(tm, "UMLVertex") && r.kind_of(hm, "UMLVertex"),
            // a control or object flow joins two activity nodes (uml-factory.js:119-127)
            // a flow joins two flowchart nodes (flowchart-factory.js:20-27)
            "FCFlow" => r.kind_of(tm, "FCModelElement") && r.kind_of(hm, "FCModelElement"),
            // a data flow joins two DFD elements, a mind map edge two nodes (dfd-factory.js:20-26,
            // mindmap-factory.js:20-27)
            "DFDDataFlow" => r.kind_of(tm, "DFDElement") && r.kind_of(hm, "DFDElement"),
            "MMEdge" => r.ty(tm) == "MMNode" && r.ty(hm) == "MMNode",
            // a relationship joins two entities (entityLinkPrecondition, erd-factory.js:20-26)
            // a message joins two lifelines, endpoints or gates (messageFn, uml-factory.js:1185-1215)
            // on a timing diagram, between two time segments (UMLTimingMessageView.canConnectTo)
            _ if crate::sequence::message_sort(kind).is_some() && r.ty(tail) == "UMLTimeSegmentView" => r.ty(head) == "UMLTimeSegmentView",
            _ if crate::sequence::message_sort(kind).is_some() => r.kind_of(tm, "UMLMessageEndpoint") && r.kind_of(hm, "UMLMessageEndpoint"),
            // a C4 relationship joins two C4 elements (linkPrecondition, c4-factory.js:20-25)
            "C4Relationship" => r.kind_of(tm, "C4Element") && r.kind_of(hm, "C4Element"),
            // a cloud line joins two elements of its notation (linkPrecondition, aws-factory.js:20-27)
            k if crate::cloud::edge_view_of(k).is_some() => crate::cloud::element_kind(k).map(|ek| r.kind_of(tm, ek) && r.kind_of(hm, ek)).unwrap_or(false),
            "ERDRelationship" | "ERDRelationshipOneToMany" | "ERDRelationshipManyToMany" => r.ty(tm) == "ERDEntity" && r.ty(hm) == "ERDEntity",
            "UMLControlFlow" | "UMLActivityInterrupt" => r.kind_of(tm, "UMLActivityNode") && r.kind_of(hm, "UMLActivityNode"),
            // an exception handler joins two actions (actionLinkPrecondition, uml-factory.js:129-135)
            "UMLExceptionHandler" => r.kind_of(tm, "UMLAction") && r.kind_of(hm, "UMLAction"),
            // an object flow may also end on a pin (UMLObjectFlowView.canConnectTo, elements.js:10528)
            "UMLObjectFlow" => {
                let ok = |m: Eid| r.kind_of(m, "UMLActivityNode") || r.kind_of(m, "UMLPin");
                ok(tm) && ok(hm)
            }
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
        // an object flow between two actions runs from a new output pin of the first to a new input
        // pin of the second, each where the line between the actions leaves their boxes
        // (uml-factory.js:2313-2370)
        let action = |a: &Self, v: Eid| a.repo.refd(v, "model").map(|m| a.repo.ty(m) == "UMLAction").unwrap_or(false);
        if tool.kind == "UMLObjectFlow" && action(self, tail) && action(self, head) && !views::is_edge(&self.repo, tail) && !views::is_edge(&self.repo, head) {
            let (tb, hb) = (views::node_rect(&self.repo, tail), views::node_rect(&self.repo, head));
            let (tc, hc) = (P { x: tb.x + tb.w / 2.0, y: tb.y + tb.h / 2.0 }, P { x: hb.x + hb.w / 2.0, y: hb.y + hb.h / 2.0 });
            let out = self.create_feature_on(tail, "UMLOutputPin", views::port_junction(tb, hc), R::default()).ok()?;
            let inp = self.create_feature_on(head, "UMLInputPin", views::port_junction(hb, tc), R::default()).ok()?;
            return self.create_edge_inner(tool, out, inp);
        }
        // an exception handler ends on a new input pin of the handler action, where the line meets it;
        // the action is its handler body (exceptionHandlerFn, uml-factory.js:2367-2425)
        if tool.kind == "UMLExceptionHandler" {
            let (tm, hm) = (self.repo.refd(tail, "model")?, self.repo.refd(head, "model")?);
            let (tb, hb) = (views::node_rect(&self.repo, tail), views::node_rect(&self.repo, head));
            let tc = P { x: tb.x + tb.w / 2.0, y: tb.y + tb.h / 2.0 };
            let pin = self.create_feature_on(head, "UMLInputPin", views::port_junction(hb, tc), R::default()).ok()?;
            let pm = self.repo.refd(pin, "model")?;
            let owner = self.diagram_activity(d).unwrap_or(tm);
            let field = if self.repo.ty(owner) == "UMLActivity" { "edges" } else { "ownedElements" };
            let (s, t, hb) = (self.repo.ref_to(tm), self.repo.ref_to(pm), self.repo.ref_to(hm));
            let m = self.repo.create("UMLExceptionHandler", &[("source", s), ("target", t), ("handlerBody", hb)], Some((owner, field)));
            return views::create_edge_view(&mut self.repo, d, Some(m), tail, pin);
        }
        if tool.kind == "NoteLink" {
            return views::create_edge_view(&mut self.repo, d, None, tail, head);
        }
        if let Some(v) = self.comm_edge(d, tool, tail, head) {
            return v;
        }
        // a message, from line part to line part (or endpoint), at the height it was drawn; found and
        // lost messages start or end at a new endpoint (messageFn, foundMessageFn, lostMessageFn,
        // uml-factory.js:1185-1510)
        // a timing diagram's message joins two time segments at the points it was drawn between
        if let (Some(sort), true) = (crate::sequence::message_sort(tool.kind), self.repo.ty(d) == "UMLTimingDiagram") {
            let (from, to) = self.line_from.take().unwrap_or_default();
            return self.timing_message(d, sort, tail, head, from, to);
        }
        if let Some(sort) = crate::sequence::message_sort(tool.kind) {
            let inter = self.repo.parent(d).filter(|p| self.repo.ty(*p) == "UMLInteraction")?;
            let (from, to) = self.line_from.take().unwrap_or_default();
            let line = |a: &App, v: Eid| if a.repo.ty(v) == "UMLSeqLifelineView" { a.repo.refd(v, "linePart").unwrap_or(v) } else { v };
            let endpoint = |a: &mut App, at: P| -> Option<Eid> {
                let name = a.new_name(inter, "UMLEndpoint");
                let m = a.repo.create("UMLEndpoint", &[("name", json!(name))], Some((inter, "participants")));
                let v = crate::sequence::create_endpoint_view(&mut a.repo, d, m, at);
                a.apply_view_defaults(v);
                Some(v)
            };
            let (t, h) = match tool.kind {
                "UMLFoundMessage" => (endpoint(self, from)?, line(self, head)),
                "UMLLostMessage" => (line(self, tail), endpoint(self, to)?),
                _ => (line(self, tail), line(self, head)),
            };
            let (tm, hm) = (self.repo.refd(t, "model")?, self.repo.refd(h, "model")?);
            let name = if matches!(sort, "reply" | "createMessage" | "deleteMessage") { String::new() } else { self.new_name(inter, "UMLMessage") };
            let (rs, rt) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
            let m = self.repo.create("UMLMessage", &[("name", json!(name)), ("source", rs), ("target", rt)], Some((inter, "messages")));
            if sort != "synchCall" {
                self.repo.set(m, "messageSort", json!(sort));
            }
            let y = if from.y > 0.0 { from.y } else { to.y };
            return Some(crate::sequence::create_message_view(&mut self.repo, d, m, t, h, y.max(LIFELINE_MESSAGE_MIN_Y)));
        }
        // an ERD relationship, owned by its first entity, with an end at each; the one-to-many and
        // many-to-many tools make its ends "0..*" (toolbox/erd.json "model-init")
        if tool.kind.starts_with("ERDRelationship") {
            let (tm, hm) = (self.repo.refd(tail, "model")?, self.repo.refd(head, "model")?);
            let m = self.repo.create("ERDRelationship", &[], Some((tm, "ownedElements")));
            let (r1, r2) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
            let e1 = self.repo.create("ERDRelationshipEnd", &[("reference", r1)], Some((m, "end1")));
            let e2 = self.repo.create("ERDRelationshipEnd", &[("reference", r2)], Some((m, "end2")));
            if tool.kind == "ERDRelationshipManyToMany" {
                self.repo.set(e1, "cardinality", json!("0..*"));
            }
            if tool.kind != "ERDRelationship" {
                self.repo.set(e2, "cardinality", json!("0..*"));
            }
            return views::create_edge_view(&mut self.repo, d, Some(m), tail, head);
        }
        if tool.kind == "AssociationClass" {
            return self.create_association_class(d, tail, head);
        }
        if tool.kind == "SysMLConnector" {
            return self.create_sysml_connector(d, tail, head);
        }
        if tool.kind == "LinkObject" {
            return self.create_class_link(d, tail, head, true);
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
        let m = if tool.kind == "UMLAssociation" || tool.kind == "UMLCommunicationPath" || tool.kind == "UMLLink" || tool.kind == "UMLConnector" {
            let m = self.repo.create(tool.kind, &[], Some((tm, "ownedElements")));
            let (r1, r2) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
            // a link joins two objects through link ends
            let end = match tool.kind {
                "UMLLink" => "UMLLinkEnd",
                "UMLConnector" => "UMLConnectorEnd",
                _ => "UMLAssociationEnd",
            };
            let e1 = self.repo.create(end, &[("reference", r1)], Some((m, "end1")));
            let e2 = self.repo.create(end, &[("reference", r2)], Some((m, "end2")));
            match tool.init {
                AssocInit::Directed => {
                    self.repo.set(e1, "navigable", json!("unspecified"));
                    self.repo.set(e2, "navigable", json!("navigable"));
                }
                AssocInit::Shared => self.repo.set(e2, "aggregation", json!("shared")),
                AssocInit::Composite => self.repo.set(e2, "aggregation", json!("composite")),
                AssocInit::CompositeTail => self.repo.set(e1, "aggregation", json!("composite")),
                AssocInit::Plain => {}
            }
            m
        } else {
            let (s, t) = (self.repo.ref_to(tm), self.repo.ref_to(hm));
            // a transition belongs to the region of its source state
            let owner = match self.repo.parent(tm) {
                Some(rg) if tool.kind == "UMLTransition" && self.repo.ty(rg) == "UMLRegion" => (rg, "transitions"),
                // a flow belongs to the edges of the diagram's activity (uml-factory.js:2253-2300)
                _ if matches!(tool.kind, "UMLControlFlow" | "UMLObjectFlow" | "UMLActivityInterrupt") && self.diagram_activity(d).is_some() => (self.diagram_activity(d)?, "edges"),
                _ => (tm, "ownedElements"),
            };
            self.repo.create(tool.kind, &[("source", s), ("target", t)], Some(owner))
        };
        views::create_edge_view(&mut self.repo, d, Some(m), tail, head)
    }

    /// The Association Class tool (uml-factory.js:631): between two classifiers it creates an
    /// association, a class "AssociationClass" 30 px below the middle of the line, and the dashed link
    /// between them; from a class to an association line it only links them. Returns the class's box
    /// in the first case, the link in the second.
    fn create_association_class(&mut self, d: Eid, tail: Eid, head: Eid) -> Option<Eid> {
        self.create_class_link(d, tail, head, false)
    }

    /// The Association Class tool, or with `objects` the Link Object tool (uml-factory.js:631, 873):
    /// the same pattern with an Object "LinkObject", a Link and a UMLLinkObjectLink (its field names
    /// `objectSide` and `linkSide` are estimated).
    fn create_class_link(&mut self, d: Eid, tail: Eid, head: Eid, objects: bool) -> Option<Eid> {
        let (rel, node, name, link_ty, view_ty, side1, side2) = if objects {
            ("UMLLink", "UMLObject", "LinkObject", "UMLLinkObjectLink", "UMLLinkObjectLinkView", "objectSide", "linkSide")
        } else {
            ("UMLAssociation", "UMLClass", "AssociationClass", "UMLAssociationClassLink", "UMLAssociationClassLinkView", "classSide", "associationSide")
        };
        let link = |a: &mut App, class_view: Eid, assoc_view: Eid| -> Option<Eid> {
            let (cm, am) = (a.repo.refd(class_view, "model")?, a.repo.refd(assoc_view, "model")?);
            let (c, s) = (a.repo.ref_to(cm), a.repo.ref_to(am));
            let l = a.repo.create(link_ty, &[(side1, c), (side2, s)], Some((cm, "ownedElements")));
            let props = [
                ("model", a.repo.ref_to(l)),
                ("font", json!(views::FONT)),
                ("head", a.repo.ref_to(assoc_view)),
                ("tail", a.repo.ref_to(class_view)),
                ("lineStyle", json!(views::DEFAULT_LINE_STYLE)),
                ("points", json!("0:0;0:0")),
            ];
            Some(a.repo.create(view_ty, &props, Some((d, "ownedViews"))))
        };
        if views::is_edge(&self.repo, head) {
            return link(self, tail, head);
        }
        // a class and an n-ary association node: only the link, from the class (uml-factory.js:743-761)
        let nary = |a: &App, v: Eid| a.repo.refd(v, "model").map(|m| a.repo.ty(m) == "UMLNaryAssociationNode").unwrap_or(false);
        if !objects && nary(self, head) {
            return link(self, tail, head);
        }
        if !objects && nary(self, tail) {
            return link(self, head, tail);
        }
        let assoc = Tool { label: "", kind: rel, icon: "", line: true, init: AssocInit::Plain };
        let e = self.create_edge(&assoc, tail, head)?;
        let (a, b) = (views::node_rect(&self.repo, tail), views::node_rect(&self.repo, head));
        let mid = P { x: (a.x + a.w / 2.0 + b.x + b.w / 2.0) / 2.0, y: (a.y + a.h / 2.0 + b.y + b.h / 2.0) / 2.0 };
        let (w, h) = views::default_size(node);
        let cv = self.create_node(node, R { x: (mid.x - w / 2.0).max(0.0).round(), y: (mid.y + 30.0).round(), w, h })?;
        if let Some(cm) = self.repo.refd(cv, "model") {
            self.repo.set(cm, "name", json!(name));
        }
        link(self, cv, e)?;
        Some(cv)
    }

    /// Reconnect one end of a line to another view, updating the model too: `source`/`target` of a
    /// directed relationship, `end1`/`end2.reference` of an association (engine.js:940-1024). Returns
    /// the refusal message when the kinds do not fit (uml/main.js:44-82).
    pub fn reconnect(&mut self, e: Eid, tail: bool, to: Eid) -> Result<(), String> {
        let (Some(t), Some(h)) = (self.repo.refd(e, "tail"), self.repo.refd(e, "head")) else { return Ok(()) };
        // a sequence message goes onto the lifeline's line part (uml/main.js:45-47)
        let to = if self.repo.ty(e) == "UMLSeqMessageView" && self.repo.ty(to) == "UMLSeqLifelineView" { self.repo.refd(to, "linePart").unwrap_or(to) } else { to };
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
            // a communication connector refers to the new lifeline's role, and the messages on it
            // now come from or go to the new lifeline (uml/main.js:49-68)
            if a.repo.ty(to) == "UMLCommLifelineView" && a.repo.ty(e) == "UMLConnectorView" {
                crate::communication::reconnected(&mut a.repo, e, tail);
                return;
            }
            // the ends are laid out again from the new box
            if let (Some(m), Some(tm)) = (a.repo.refd(e, "model"), a.repo.refd(to, "model")) {
                if a.repo.ty(m) == "UMLAssociationClassLink" {
                    a.repo.set_ref(m, if tail { "classSide" } else { "associationSide" }, Some(tm));
                } else if a.repo.ty(m) == "UMLLinkObjectLink" {
                    a.repo.set_ref(m, if tail { "objectSide" } else { "linkSide" }, Some(tm));
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
        if let Some(x) = self.add_sysml_feature(owner, ty) {
            return x;
        }
        let field = match ty {
            "UMLAttribute" | "UMLPort" => "attributes",
            "UMLOperation" => "operations",
            "UMLReception" => "receptions",
            "UMLExtensionPoint" => "extensionPoints",
            "UMLSlot" => "slots",
            "UMLEnumerationLiteral" => "literals",
            "UMLParameter" => "parameters",
            "UMLTemplateParameter" => "templateParameters",
            "UMLTemplateParameterSubstitution" => "parameterSubstitutions",
            // states into a region, regions into a state, triggers, actions into an activity
            // (StarUML's menu files the first two in ownedElements; the metamodel's fields are used here)
            "UMLState" => "vertices",
            "UMLRegion" => "regions",
            "UMLEvent" => "triggers",
            "UMLAction" => "nodes",
            "UMLInteractionOperand" => "operands",
            "ERDColumn" => "columns",
            _ => return None,
        };
        let name = self.new_name(owner, ty);
        Some(self.repo.create(ty, &[("name", json!(name))], Some((owner, field))))
    }

    /// Add Internal Transition on a state: a transition of kind internal from the state to itself, in
    /// its first region; a state without regions is refused (uml-commands.js:3098).
    pub fn add_internal_transition(&mut self, s: Eid) -> Result<Eid, String> {
        let Some(rg) = self.repo.list(s, "regions").into_iter().find(|x| self.repo.alive(*x)) else {
            return Err("Simple State cannot have Internal Transitions.".into());
        };
        let r = self.repo.ref_to(s);
        Ok(self.change(|a| a.repo.create("UMLTransition", &[("source", r.clone()), ("target", r), ("kind", json!("internal"))], Some((rg, "transitions")))))
    }

    /// Add Qualifier on an association end: a new attribute in its qualifiers, drawn in a box at
    /// that end of the line.
    pub fn add_qualifier(&mut self, end: Eid) -> Eid {
        let name = self.new_name(end, "UMLAttribute");
        self.repo.create("UMLAttribute", &[("name", json!(name))], Some((end, "qualifiers")))
    }

    pub fn can_add_feature(&self, owner: Eid, ty: &str) -> bool {
        let t = self.repo.ty(owner);
        match ty {
            "UMLEnumerationLiteral" => t == "UMLEnumeration",
            "UMLParameter" => t == "UMLOperation",
            "UMLTemplateParameterSubstitution" => t == "UMLTemplateBinding",
            "UMLState" => t == "UMLRegion",
            "UMLRegion" => t == "UMLState",
            "UMLEvent" => t == "UMLAction" || t == "UMLTransition",
            "UMLAction" => t == "UMLActivity",
            "UMLInteractionOperand" => t == "UMLCombinedFragment",
            "ERDColumn" => t == "ERDEntity",
            "UMLExtensionPoint" => t == "UMLUseCase",
            "UMLSlot" => self.repo.kind_of(owner, "UMLInstance"),
            // template parameters on classes, interfaces and the other boxes with compartments
            "UMLTemplateParameter" => self.repo.kind_of(owner, "UMLClassifier") && t != "UMLActor" && t != "UMLUseCase" && t != "UMLEnumeration",
            // an actor takes attributes and operations (quickedits/uml.json "uml.actor"), not receptions
            "UMLAttribute" | "UMLOperation" if t == "UMLActor" => true,
            _ => self.repo.kind_of(owner, "UMLClassifier") && t != "UMLActor" && t != "UMLUseCase",
        }
    }

    /// The Port and Part tools: a port on the border of the classifier box under the point, or a part
    /// inside it, its model a new attribute of that classifier (a composite one for a part). On a part,
    /// they go to the part's type. Elsewhere they are refused (uml-factory.js:996-1100).
    pub fn create_feature_box(&mut self, kind: &str, at: P, rect: R) -> Result<Eid, String> {
        let d = self.diagram.ok_or_else(String::new)?;
        let refused = match kind {
            // portFn and parameterFn (sysml-factory.js:116-228)
            "SysMLPort" => "Port should be placed in a block or a part".to_string(),
            "SysMLConstraintParameter" => "Constraint Parameter should be placed in a block or a part".to_string(),
            _ => format!("{} cannot be placed here.", if kind == "UMLPort" { "UMLPort" } else { "UMLAttribute" }),
        };
        let host = self
            .repo
            .list(d, "ownedViews")
            .into_iter()
            .rev()
            .filter(|v| views::is_node(&self.repo, *v) && !matches!(self.repo.ty(*v), "UMLPortView" | "UMLInputPinView" | "UMLOutputPinView" | "UMLExpansionNodeView" | "UMLConnectionPointReferenceView") && !crate::sysml::floating(self.repo.ty(*v)) && !matches!(self.repo.raw(*v, "visible"), Some(serde_json::Value::Bool(false))))
            .find(|v| {
                let b = views::node_rect(&self.repo, *v);
                at.x >= b.x && at.y >= b.y && at.x <= b.x + b.w && at.y <= b.y + b.h
            })
            .ok_or(refused.clone())?;
        self.create_feature_on(host, kind, at, rect)
    }

    /// A port or part on a given box (see `create_feature_box`), or an input or output pin on an
    /// action's box, in the action's inputs or outputs (uml-factory.js:3827-3834).
    pub fn create_feature_on(&mut self, host: Eid, kind: &str, at: P, rect: R) -> Result<Eid, String> {
        let d = self.diagram.ok_or_else(String::new)?;
        if crate::sysml::parasitic(kind) {
            return self.create_sysml_feature(d, host, kind, at);
        }
        // a connection point reference goes in the connections of the state under the point
        // (connectionPointReferenceFn, uml-factory.js:2183-2214)
        if kind == "UMLConnectionPointReference" {
            let hm = self.repo.refd(host, "model").filter(|m| self.repo.kind_of(*m, "UMLState")).ok_or_else(|| format!("{kind} cannot be placed here."))?;
            let m = self.repo.create(kind, &[("name", json!(""))], Some((hm, "connections")));
            let v = views::create_connection_point_view(&mut self.repo, d, m, host, at);
            self.apply_view_defaults(v);
            return Ok(v);
        }
        if matches!(kind, "UMLInputPin" | "UMLOutputPin" | "UMLInputExpansionNode" | "UMLOutputExpansionNode") {
            let refused = format!("{kind} cannot be placed here.");
            let hm = self.repo.refd(host, "model").filter(|m| self.repo.kind_of(*m, "UMLAction")).ok_or(refused)?;
            // an expansion node is an unnamed UMLExpansionNode (uml-factory.js:3849-3866)
            let (ty, name) = match kind {
                "UMLInputExpansionNode" | "UMLOutputExpansionNode" => ("UMLExpansionNode", String::new()),
                _ => (kind, self.new_name(hm, kind)),
            };
            let field = if matches!(kind, "UMLInputPin" | "UMLInputExpansionNode") { "inputs" } else { "outputs" };
            let m = self.repo.create(ty, &[("name", json!(name))], Some((hm, field)));
            let v = views::create_pin_view(&mut self.repo, d, m, host, at);
            self.apply_view_defaults(v);
            return Ok(v);
        }
        let refused = format!("{} cannot be placed here.", if kind == "UMLPort" { "UMLPort" } else { "UMLAttribute" });
        let hm = self.repo.refd(host, "model").ok_or(refused.clone())?;
        let owner = if self.repo.ty(hm) == "UMLAttribute" {
            match self.repo.refd(hm, "type").filter(|t| self.repo.kind_of(*t, "UMLClassifier")) {
                Some(t) => t,
                None => return Err(if kind == "UMLPort" { "Type should be assigned to have Ports".into() } else { "Type should be assigned to have internal Parts".into() }),
            }
        } else if self.repo.kind_of(hm, "UMLClassifier") {
            hm
        } else {
            return Err(refused);
        };
        let prefix = if kind == "UMLPort" { "Port" } else { "Part" };
        let names: HashSet<String> = self.repo.list(owner, "attributes").into_iter().map(|a| self.repo.name(a)).collect();
        let name = (1..).map(|i| format!("{prefix}{i}")).find(|n| !names.contains(n)).unwrap();
        let v = if kind == "UMLPort" {
            let m = self.repo.create("UMLPort", &[("name", json!(name))], Some((owner, "attributes")));
            views::create_port_view(&mut self.repo, d, m, host, at)
        } else {
            let m = self.repo.create("UMLAttribute", &[("name", json!(name)), ("aggregation", json!("composite"))], Some((owner, "attributes")));
            views::create_part_view(&mut self.repo, d, m, host, rect)
        };
        self.apply_view_defaults(v);
        Ok(v)
    }

    /// Put the picture from a file into an Image view and give the view the picture's natural size,
    /// as one undo step (app-context.js:1360-1382).
    pub fn set_image_file(&mut self, v: Eid, path: &std::path::Path) -> Result<(), String> {
        let (data, w, h) = crate::picture::load_file(path)?;
        self.change(|a| {
            for (k, val) in [("imageData", json!(data)), ("imageWidth", json!(w)), ("imageHeight", json!(h)), ("width", json!(w)), ("height", json!(h))] {
                a.repo.set(v, k, val);
            }
        });
        Ok(())
    }

    /// The Image view's file picker (.png, .jpg, .jpeg, .svg); a picture that cannot be read is an
    /// error toast.
    pub fn pick_image(&mut self, v: Eid) {
        let Some(path) = rfd::FileDialog::new().add_filter("Image files", &["png", "jpeg", "jpg", "svg"]).pick_file() else { return };
        if let Err(e) = self.set_image_file(v, &path) {
            self.toast_error(&e);
        }
    }

    /// A picture file dropped on the canvas: an Image view at the picture's natural size, centred on
    /// the drop point (app-context.js:1404-1427).
    pub fn drop_image_file(&mut self, path: &std::path::Path, at: P) -> Result<Eid, String> {
        let (data, w, h) = crate::picture::load_file(path)?;
        let rect = R { x: (at.x - (w as f64 / 2.0).round()).max(0.0), y: (at.y - (h as f64 / 2.0).round()).max(0.0), w: w as f64, h: h as f64 };
        let v = self.change(|a| {
            let v = a.create_node("Image", rect)?;
            for (k, val) in [("imageData", json!(data)), ("imageWidth", json!(w)), ("imageHeight", json!(h))] {
                a.repo.set(v, k, val);
            }
            Some(v)
        });
        v.ok_or_else(String::new)
    }

    pub fn add_model(&mut self, parent: Eid, ty: &str) -> Eid {
        // a BPMN element: an event definition in its event's definitions, a gateway with its direction
        if ty.starts_with("BPMN") {
            return self.add_bpmn_model(parent, ty);
        }
        let name = self.new_name(parent, ty);
        let m = self.repo.create(ty, &[("name", json!(name))], Some((parent, "ownedElements")));
        add_stereotype_icon(&mut self.repo, m);
        // a C4 container is of no particular kind (c4/elements.js:95-100)
        if ty == "C4Container" {
            self.repo.set(m, "kind", json!("etc"));
        }
        // a cloud element starts with its icon (the constructors in aws, azure, gcp elements.js)
        crate::cloud::init_model(&mut self.repo, m);
        // a state machine starts with one region (uml-factory.js:606-626)
        if ty == "UMLStateMachine" {
            self.repo.create("UMLRegion", &[], Some((m, "regions")));
        }
        m
    }

    pub fn add_diagram(&mut self, parent: Option<Eid>, ty: &str) -> Eid {
        // a BPMN diagram goes in a BPMN root element or activity, else in a new process (bpmn-factory.js:28-57)
        if ty == "BPMNDiagram" {
            let d = self.add_bpmn_diagram(parent);
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        if crate::sysml::is_diagram(ty) {
            return self.add_sysml_diagram(parent, ty);
        }
        let target = match parent {
            Some(p) if self.repo.kind_of(p, "UMLPackage") || self.repo.ty(p) == "Project" || self.repo.kind_of(p, "UMLClassifier") => p,
            // an activity diagram added on an activity goes in it (uml-factory.js:574)
            Some(p) if ty == "UMLActivityDiagram" && self.repo.ty(p) == "UMLActivity" => p,
            _ => self.package_for(self.diagram),
        };
        // a flowchart diagram goes in a flowchart: the target if it is one, else a new "Flowchart1"…
        // (flowchart-factory.js:31-62)
        // and so do a data flow diagram (in any DFD element, else a new "DataFlowModel1") and a mind
        // map diagram (in a mind map, else a new "Mindmap1") (dfd-factory.js:30-55, mindmap-factory.js:30-60)
        // a sequence diagram goes in an interaction: the target if it is one, else a new interaction in
        // a classifier, else a new collaboration with an interaction; it starts with a frame showing
        // it (sequenceDiagramFn and _addFrame, uml-factory.js:251-310)
        // an interaction overview diagram goes in an activity: the target if it is one, else a new
        // "Activity1" in the interaction, the classifier's new interaction or a new collaboration's
        // interaction; it starts with a frame (interactionOverviewDiagramFn, uml-factory.js:441-530)
        if ty == "UMLInteractionOverviewDiagram" {
            let target = match parent {
                Some(p) if matches!(self.repo.ty(p), "UMLActivity" | "UMLInteraction") || self.repo.kind_of(p, "UMLClassifier") || self.repo.kind_of(p, "UMLPackage") || self.repo.ty(p) == "Project" => p,
                _ => self.package_for(self.diagram),
            };
            let act = if self.repo.ty(target) == "UMLActivity" {
                target
            } else {
                let inter = if self.repo.ty(target) == "UMLInteraction" {
                    target
                } else {
                    let holder = if self.repo.kind_of(target, "UMLClassifier") {
                        target
                    } else {
                        let name = self.new_name(target, "UMLCollaboration");
                        self.repo.create("UMLCollaboration", &[("name", json!(name))], Some((target, "ownedElements")))
                    };
                    let name = self.new_name(holder, "UMLInteraction");
                    self.repo.create("UMLInteraction", &[("name", json!(name))], Some((holder, "ownedElements")))
                };
                let name = self.new_name(inter, "UMLActivity");
                self.repo.create("UMLActivity", &[("name", json!(name))], Some((inter, "ownedElements")))
            };
            let name = self.new_name(act, ty);
            let d = self.repo.create(ty, &[("name", json!(name))], Some((act, "ownedElements")));
            let f = views::create_frame_view(&mut self.repo, d, Some(d), R { x: 8.0, y: 8.0, w: 692.0, h: 592.0 });
            self.apply_view_defaults(f);
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        // a timing diagram too, with a timing frame (timingDiagramFn, uml-factory.js:374-434)
        if ty == "UMLTimingDiagram" {
            let target = match parent {
                Some(p) if self.repo.ty(p) == "UMLInteraction" || self.repo.kind_of(p, "UMLClassifier") || self.repo.kind_of(p, "UMLPackage") || self.repo.ty(p) == "Project" => p,
                _ => self.package_for(self.diagram),
            };
            let target = if self.repo.ty(target) == "UMLInteraction" {
                target
            } else {
                let holder = if self.repo.kind_of(target, "UMLCollaboration") {
                    target
                } else {
                    let name = self.new_name(target, "UMLCollaboration");
                    self.repo.create("UMLCollaboration", &[("name", json!(name))], Some((target, "ownedElements")))
                };
                let name = self.new_name(holder, "UMLInteraction");
                self.repo.create("UMLInteraction", &[("name", json!(name))], Some((holder, "ownedElements")))
            };
            let name = self.new_name(target, ty);
            let d = self.repo.create(ty, &[("name", json!(name))], Some((target, "ownedElements")));
            let f = crate::timing::create_frame(&mut self.repo, d);
            self.apply_view_defaults(f);
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        // and a communication diagram the same way, but in a collaboration's new interaction
        // (communicationDiagramFn, uml-factory.js:312-370)
        if ty == "UMLSequenceDiagram" || ty == "UMLCommunicationDiagram" {
            let target = match parent {
                Some(p) if self.repo.ty(p) == "UMLInteraction" || self.repo.kind_of(p, "UMLClassifier") || self.repo.kind_of(p, "UMLPackage") || self.repo.ty(p) == "Project" => p,
                _ => self.package_for(self.diagram),
            };
            let target = if self.repo.ty(target) == "UMLInteraction" {
                target
            } else {
                let holder = if self.repo.kind_of(target, if ty == "UMLCommunicationDiagram" { "UMLCollaboration" } else { "UMLClassifier" }) {
                    target
                } else {
                    let name = self.new_name(target, "UMLCollaboration");
                    self.repo.create("UMLCollaboration", &[("name", json!(name))], Some((target, "ownedElements")))
                };
                let name = self.new_name(holder, "UMLInteraction");
                self.repo.create("UMLInteraction", &[("name", json!(name))], Some((holder, "ownedElements")))
            };
            let name = self.new_name(target, ty);
            let d = self.repo.create(ty, &[("name", json!(name))], Some((target, "ownedElements")));
            let f = views::create_frame_view(&mut self.repo, d, Some(d), R { x: 8.0, y: 8.0, w: 692.0, h: 592.0 });
            self.apply_view_defaults(f);
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        // a cloud diagram goes in the element it is added to; right under the project, in a new
        // "AWSModel1" (diagramFn, aws-factory.js:32-75, azure-factory.js, gcp-factory.js)
        if let Some(model_ty) = crate::cloud::model_of(ty) {
            let target = parent.filter(|p| self.repo.alive(*p) && !self.repo.kind_of(*p, "Diagram")).unwrap_or(self.repo.root);
            let name = self.new_name(target, ty);
            let holder = if self.repo.ty(target) == "Project" {
                let mname = self.new_name(target, model_ty);
                self.repo.create(model_ty, &[("name", json!(mname))], Some((target, "ownedElements")))
            } else {
                target
            };
            let d = self.repo.create(ty, &[("name", json!(name))], Some((holder, "ownedElements")));
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        // a C4 diagram goes in the C4 element it is added to; elsewhere in a new "C4Model1" in the
        // project (c4-factory.js:28-55)
        if ty == "C4Diagram" {
            let target = match parent {
                Some(p) if self.repo.kind_of(p, "C4Element") => p,
                _ => self.repo.root,
            };
            let target = if self.repo.ty(target) == "Project" {
                let name = self.new_name(target, "C4Model");
                self.repo.create("C4Model", &[("name", json!(name))], Some((target, "ownedElements")))
            } else {
                target
            };
            let name = self.new_name(target, ty);
            let d = self.repo.create(ty, &[("name", json!(name))], Some((target, "ownedElements")));
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        let holder = match ty {
            "FCFlowchartDiagram" => Some(("FCFlowchart", "Flowchart", "FCFlowchart")),
            "DFDDiagram" => Some(("DFDElement", "DataFlowModel", "DFDDataFlowModel")),
            "MMMindmapDiagram" => Some(("MMMindmap", "Mindmap", "MMMindmap")),
            // a wireframe diagram goes in a wireframe, else in a new "Wireframe1" (wireframe-factory.js:20-50)
            "WFWireframeDiagram" => Some(("WFWireframe", "Wireframe", "WFWireframe")),
            // an ER diagram goes in a data model, else in a new "Data Model1" (erd-factory.js:30-55)
            "ERDDiagram" => Some(("ERDDataModel", "Data Model", "ERDDataModel")),
            _ => None,
        };
        if let Some((kind, base, new_ty)) = holder {
            let target = match parent {
                Some(p) if self.repo.kind_of(p, kind) => p,
                Some(p) if self.repo.kind_of(p, "ExtensibleModel") && !self.repo.kind_of(p, "Diagram") => p,
                _ => self.package_for(self.diagram),
            };
            let target = if self.repo.kind_of(target, kind) {
                target
            } else {
                let names: HashSet<String> = self.repo.children(target).into_iter().map(|(_, c)| self.repo.name(c)).collect();
                let name = (1..).map(|i| format!("{base}{i}")).find(|n| !names.contains(n)).unwrap();
                self.repo.create(new_ty, &[("name", json!(name))], Some((target, "ownedElements")))
            };
            let name = self.new_name(target, ty);
            let d = self.repo.create(ty, &[("name", json!(name))], Some((target, "ownedElements")));
            self.open_diagram(d);
            self.reveal(d);
            return d;
        }
        // a profile diagram goes in a profile: the target if it is one, else a new profile "Profile" (uml-factory.js:184)
        let target = if ty == "UMLProfileDiagram" && self.repo.ty(target) != "UMLProfile" {
            self.repo.create("UMLProfile", &[("name", json!("Profile"))], Some((target, "ownedElements")))
        } else {
            target
        };
        // a structural diagram made right under the Project goes in a new model "Model" (uml-factory.js:150-180)
        let structural = matches!(ty, "UMLClassDiagram" | "UMLPackageDiagram" | "UMLObjectDiagram" | "UMLComponentDiagram" | "UMLDeploymentDiagram" | "UMLUseCaseDiagram" | "UMLInformationFlowDiagram");
        let target = if structural && self.repo.ty(target) == "Project" {
            self.repo.create("UMLModel", &[("name", json!("Model"))], Some((target, "ownedElements")))
        } else {
            target
        };
        // a composite structure diagram outside a classifier goes in a new collaboration, named
        // "Composite Structures1"… (uml-factory.js:213-244)
        // a statechart diagram goes in a state machine: the target if it is one, else a new one with
        // one region (uml-factory.js:529-571, 606)
        let target = if ty == "UMLStatechartDiagram" && self.repo.ty(target) != "UMLStateMachine" {
            let name = self.new_name(target, "UMLStateMachine");
            let sm = self.repo.create("UMLStateMachine", &[("name", json!(name))], Some((target, "ownedElements")));
            self.repo.create("UMLRegion", &[], Some((sm, "regions")));
            sm
        } else {
            target
        };
        // an activity diagram goes in an activity: the target if it is one, else a new one
        // (uml-factory.js:572-597)
        let target = if ty == "UMLActivityDiagram" && self.repo.ty(target) != "UMLActivity" {
            let name = self.new_name(target, "UMLActivity");
            self.repo.create("UMLActivity", &[("name", json!(name))], Some((target, "ownedElements")))
        } else {
            target
        };
        let (target, name) = if ty == "UMLCompositeStructureDiagram" && !self.repo.kind_of(target, "UMLClassifier") {
            let names: HashSet<String> = self.repo.children(target).into_iter().map(|(_, c)| self.repo.name(c)).collect();
            let name = (1..).map(|i| format!("Composite Structures{i}")).find(|n| !names.contains(n)).unwrap();
            let cname = self.new_name(target, "UMLCollaboration");
            (self.repo.create("UMLCollaboration", &[("name", json!(cname))], Some((target, "ownedElements"))), name)
        } else {
            (target, self.new_name(target, ty))
        };
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
        if !self.open_order.contains(&d) {
            self.open_order.push(d);
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
                let needs_model = ty.starts_with("UML") && !matches!(ty, "UMLNoteView" | "UMLTextView" | "UMLTextBoxView" | "UMLNoteLinkView" | "UMLContainmentView" | "UMLCustomTextView" | "UMLCustomNoteView" | "UMLConstraintLinkView");
                if dangling(v, "model") || (needs_model && !r.has(v, "model") && crate::meta::known(ty)) {
                    return true;
                }
                // a communication message without the connector it rides on (diagram-manager.js:523-530)
                if ty == "UMLCommMessageView" && (r.refd(v, "hostEdge").is_none() || dangling(v, "hostEdge")) {
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
        // the open order, then any working diagram it does not know yet
        let order: Vec<Eid> = self.open_order.iter().filter(|d| self.open.contains(d)).chain(self.open.iter().filter(|d| !self.open_order.contains(d))).copied().collect();
        let n = order.len() as isize;
        if n == 0 {
            return;
        }
        let i = self.diagram.and_then(|d| order.iter().position(|x| *x == d)).map(|i| i as isize).unwrap_or(-step.max(0));
        let next = order[(i + step).rem_euclid(n) as usize];
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
            _ => {
                let items: Vec<Eid> = views.iter().filter_map(|v| self.repo.parent(*v)).collect();
                self.list_picker = Some(crate::dialogs::ListPicker { title: "Select Diagram to show".into(), items, chosen: None, action: crate::dialogs::ListAction::ShowView(views) });
            }
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
        self.tree_cleared = false;
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

/// A new stereotype gets its empty 40 × 40 icon, as the original's constructor makes one
/// (elements.js:4071-4101); its content takes an icon script.
pub fn add_stereotype_icon(r: &mut crate::model::Repo, m: Eid) {
    if r.ty(m) == "UMLStereotype" && r.child(m, "icon").is_none() {
        r.create("UMLImage", &[("width", json!(40)), ("height", json!(40))], Some((m, "icon")));
    }
}
