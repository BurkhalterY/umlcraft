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
pub const UNDO_LIMIT: usize = 200;
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
    pub open_groups: HashSet<&'static str>,
    pub prop_buf: HashMap<String, String>,
    pub prop_for: Option<Eid>,
    pub doc_buf: String,
    pub props_open: bool,
    pub doc_open: bool,
    pub last_area: Area,
    pub ctx_target: Option<Eid>,
    pub ctx_pos: P,
    pub about_open: bool,
    pub message: Option<(String, String)>,
    pub show_sidebar: bool,
    pub show_navigator: bool,
    pub show_statusbar: bool,
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
    pub toasts: Vec<(String, std::time::Instant)>,
    /// Select In Diagram with several diagrams: the views to choose from and the chosen row.
    pub diagram_picker: Option<(Vec<Eid>, Option<usize>)>,
    /// The last text UMLCraft put on the clipboard, used when the system clipboard cannot be read.
    pub clipboard: Option<String>,
    /// The system clipboard, kept open: on Linux an image copied to it is served while it lives.
    pub sys_clipboard: Option<arboard::Clipboard>,
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
            open_groups: ["class", "usecase"].into_iter().collect(),
            prop_buf: HashMap::new(),
            prop_for: None,
            doc_buf: String::new(),
            props_open: true,
            doc_open: true,
            last_area: Area::Other,
            ctx_target: None,
            ctx_pos: P::default(),
            about_open: false,
            message: None,
            show_sidebar: true,
            show_navigator: true,
            show_statusbar: true,
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
        };
        a.set_project(Repo::blank(), None);
        a
    }

    // ------------------------------------------------------------ project
    pub fn set_project(&mut self, repo: Repo, filename: Option<PathBuf>) {
        self.repo = repo;
        self.filename = filename;
        self.modified = false;
        self.undo.clear();
        self.redo.clear();
        self.selection.clear();
        self.tool = None;
        self.drag = None;
        self.editing = None;
        self.info.clear();
        let diagrams = self.repo.all_of("Diagram");
        let d = diagrams.iter().copied().find(|d| self.repo.bool(*d, "defaultDiagram")).or(diagrams.first().copied());
        self.diagram = d;
        self.open = d.into_iter().collect();
        self.focus = None;
        self.zoom = 1.0;
        self.scroll = P::default();
        self.scrolls.clear();
        self.centre_pending = None;
        self.diagram_picker = None;
        self.expanded.clear();
        let root = self.repo.root;
        self.expanded.insert(root);
        for (_, c) in self.repo.children(root) {
            self.expanded.insert(c);
        }
    }

    pub fn title(&self) -> String {
        let base = self
            .filename
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".into());
        format!("{base}{} — UMLCraft", if self.modified { " •" } else { "" })
    }

    pub fn open_path(&mut self, p: PathBuf) {
        match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| Repo::parse(&t)) {
            Ok(repo) => self.set_project(repo, Some(p)),
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
                self.filename = Some(p);
                self.modified = false;
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
            "UMLClassDiagram" => "ClassDiagram",
            "UMLUseCaseDiagram" => "UseCaseDiagram",
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

    pub fn create_node(&mut self, kind: &str, rect: R) -> Option<Eid> {
        let d = self.diagram?;
        if kind == "Note" || kind == "Text" {
            let ty = if kind == "Note" { "UMLNoteView" } else { "UMLTextView" };
            return Some(views::create_note_view(&mut self.repo, d, rect, ty));
        }
        let parent = self.package_for(Some(d));
        let name = self.new_name(parent, kind);
        let m = self.repo.create(kind, &[("name", json!(name))], Some((parent, "ownedElements")));
        views::create_node_view(&mut self.repo, d, m, rect)
    }

    pub fn create_view_of(&mut self, model: Eid, rect: R) -> Option<Eid> {
        let d = self.diagram?;
        views::create_node_view(&mut self.repo, d, model, rect)
    }

    pub fn can_connect(&self, tool: &Tool, tail: Eid, head: Eid) -> bool {
        let r = &self.repo;
        if tool.kind == "NoteLink" {
            return r.ty(tail) == "UMLNoteView" || r.ty(head) == "UMLNoteView";
        }
        if views::is_edge(r, tail) || views::is_edge(r, head) {
            return false;
        }
        let (Some(tm), Some(hm)) = (r.refd(tail, "model"), r.refd(head, "model")) else { return false };
        let classifier = |m: Eid| r.kind_of(m, "UMLClassifier");
        match tool.kind {
            "UMLAssociation" => classifier(tm) && classifier(hm),
            "UMLGeneralization" => classifier(tm) && classifier(hm) && tm != hm,
            "UMLInterfaceRealization" => classifier(tm) && r.ty(hm) == "UMLInterface",
            "UMLInclude" | "UMLExtend" => r.ty(tm) == "UMLUseCase" && r.ty(hm) == "UMLUseCase",
            _ => true,
        }
    }

    pub fn create_edge(&mut self, tool: &Tool, tail: Eid, head: Eid) -> Option<Eid> {
        let d = self.diagram?;
        if tool.kind == "NoteLink" {
            return views::create_edge_view(&mut self.repo, d, None, tail, head);
        }
        let tm = self.repo.refd(tail, "model")?;
        let hm = self.repo.refd(head, "model")?;
        let m = if tool.kind == "UMLAssociation" {
            let m = self.repo.create("UMLAssociation", &[], Some((tm, "ownedElements")));
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

    pub fn add_feature(&mut self, owner: Eid, ty: &str) -> Option<Eid> {
        let field = match ty {
            "UMLAttribute" => "attributes",
            "UMLOperation" => "operations",
            "UMLEnumerationLiteral" => "literals",
            "UMLParameter" => "parameters",
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
        let mut gone: HashSet<String> = views.iter().filter(|v| self.repo.alive(**v)).map(|v| self.repo.id(*v)).collect();
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
            // each diagram keeps its own scroll position (diagram-manager.js:218-228, 243)
            if let Some(old) = self.diagram {
                self.scrolls.insert(old, self.scroll);
            }
            self.diagram = Some(d);
            self.selection.clear();
            self.scroll = self.scrolls.get(&d).copied().unwrap_or_default();
        }
        self.tool = None;
    }

    /// Close a working diagram. When it was the current one, the diagram that slid into its place
    /// becomes current, or else the first one (diagram-manager.js:106-138).
    pub fn close_diagram(&mut self, d: Option<Eid>) {
        let Some(d) = d.or(self.diagram) else { return };
        if let Some(i) = self.open.iter().position(|x| *x == d) {
            self.open.remove(i);
            // closing any working diagram clears the selection and the quick edit (app-context.js:1310-1317)
            self.selection.clear();
            self.editing = None;
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
        self.toasts.push((text.to_string(), std::time::Instant::now()));
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
