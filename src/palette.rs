//! View > Command Palette (Ctrl+Shift+P) and View > Quick Find (Ctrl+P): a box at the top of the window
//! with a filtered list under it (ui/command-palette.js, ui/quick-find.js).

use crate::app::App;
use crate::export::Format;
use crate::model::Eid;
use egui::{Align2, Key, Vec2};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Commands,
    Find,
}

/// The open palette: its mode, the typed text and the highlighted row.
#[derive(Clone, Debug)]
pub struct Palette {
    pub mode: Mode,
    pub query: String,
    pub selected: usize,
    pub fresh: bool,
}

/// A command the palette can run: its id, its name and its shortcut.
pub struct Command {
    pub id: &'static str,
    pub name: &'static str,
    pub key: &'static str,
}

const fn c(id: &'static str, name: &'static str, key: &'static str) -> Command {
    Command { id, name, key }
}

/// The commands with a display name, as the palette lists them (command ids from StarUML's menus).
pub const COMMANDS: &[Command] = &[
    c("application:new", "New", "Ctrl+N"),
    c("project:open", "Open...", "Ctrl+O"),
    c("project:save", "Save", "Ctrl+S"),
    c("project:save-as", "Save As...", "Ctrl+Shift+S"),
    c("project:export-diagram-to-png", "Export Diagram As PNG...", ""),
    c("project:export-diagram-to-jpeg", "Export Diagram As JPEG...", ""),
    c("project:export-diagram-to-svg", "Export Diagram As SVG...", ""),
    c("application:preferences", "Preferences...", ""),
    c("edit:undo", "Undo", "Ctrl+Z"),
    c("edit:redo", "Redo", "Ctrl+Y"),
    c("edit:cut", "Cut", "Ctrl+X"),
    c("edit:copy", "Copy", "Ctrl+C"),
    c("edit:copy-diagram-as-image", "Copy Diagram As Image", "Ctrl+Shift+C"),
    c("edit:paste", "Paste", "Ctrl+V"),
    c("edit:delete", "Delete", "Delete"),
    c("edit:delete-from-model", "Delete from Model", "Ctrl+Delete"),
    c("edit:move-up", "Move Up", "Ctrl+Shift+Up"),
    c("edit:move-down", "Move Down", "Ctrl+Shift+Down"),
    c("edit:select-all", "Select All", "Ctrl+A"),
    c("edit:select-in-explorer", "Select In Explorer", "Ctrl+E"),
    c("edit:select-in-diagram", "Select In Diagram", "Ctrl+D"),
    c("edit:open-sub-diagram", "Open Sub-Diagram", "Ctrl+Shift+D"),
    c("format:font", "Font...", "Ctrl+Shift+F"),
    c("format:fill-color", "Fill Color...", "Ctrl+Shift+I"),
    c("format:line-color", "Line Color...", "Ctrl+Shift+L"),
    c("format:linestyle-rectilinear", "Line Style: Rectilinear", "Ctrl+L"),
    c("format:linestyle-oblique", "Line Style: Oblique", "Ctrl+B"),
    c("format:linestyle-roundrect", "Line Style: Rounded Rectilinear", "Ctrl+Alt+L"),
    c("format:linestyle-curve", "Line Style: Curve", "Ctrl+Alt+B"),
    c("format:auto-resize", "Auto Resize", "Ctrl+Shift+R"),
    c("format:show-shadow", "Show Shadow", "Ctrl+Shift+H"),
    c("format:word-wrap", "Word Wrap", "Ctrl+Shift+W"),
    c("format:show-visibility", "Show Visibility", "Ctrl+Shift+V"),
    c("format:show-namespace", "Show Namespace", "Ctrl+Shift+N"),
    c("format:show-property", "Show Property", "Ctrl+Shift+B"),
    c("format:show-type", "Show Type", "Ctrl+Shift+Y"),
    c("format:show-multiplicity", "Show Multiplicity", "Ctrl+Shift+M"),
    c("format:show-operation-signature", "Show Operation Signature", "Ctrl+Shift+J"),
    c("format:suppress-attributes", "Suppress Attributes", "Ctrl+Shift+A"),
    c("format:suppress-operations", "Suppress Operations", "Ctrl+Shift+O"),
    c("format:suppress-literals", "Suppress Literals", "Ctrl+Shift+T"),
    c("alignment:send-to-back", "Send To Back", ""),
    c("alignment:bring-to-front", "Bring To Front", ""),
    c("alignment:align-left", "Align Left", ""),
    c("alignment:align-right", "Align Right", ""),
    c("alignment:align-center", "Align Center", ""),
    c("alignment:align-top", "Align Top", ""),
    c("alignment:align-bottom", "Align Bottom", ""),
    c("alignment:align-middle", "Align Middle", ""),
    c("alignment:space-equally-horizontally", "Space Equally, Horizontally", ""),
    c("alignment:space-equally-vertically", "Space Equally, Vertically", ""),
    c("alignment:set-width-equally", "Set Width Equally", ""),
    c("alignment:set-height-equally", "Set Height Equally", ""),
    c("alignment:set-size-equally", "Set Size Equally", ""),
    c("diagram-generator:overview", "Diagram Generator: Overview", ""),
    c("diagram-generator:overview-expanded", "Diagram Generator: Overview Expanded", ""),
    c("diagram-generator:type-hierarchy", "Diagram Generator: Type Hierarchy", ""),
    c("diagram-generator:package-structure", "Diagram Generator: Package Structure", ""),
    c("relationship-view:toggle", "Relationships", "Ctrl+Alt+R"),
    c("diagram-layout:auto", "Layout: Auto", ""),
    c("diagram-layout:top-bottom", "Layout: Top to Bottom", ""),
    c("diagram-layout:bottom-top", "Layout: Bottom to Top", ""),
    c("diagram-layout:left-right", "Layout: Left to Right", ""),
    c("diagram-layout:right-left", "Layout: Right to Left", ""),
    c("model:tag-editor", "Tag Editor...", "Ctrl+T"),
    c("view:quick-find", "Quick Find...", "Ctrl+P"),
    c("view:close-diagram", "Close Diagram", "F4"),
    c("view:close-other-diagrams", "Close Other Diagrams", "Ctrl+F4"),
    c("view:close-all-diagrams", "Close All Diagrams", "Shift+F4"),
    c("view:next-diagram", "Next Diagram", "Ctrl+Shift+]"),
    c("view:previous-diagram", "Previous Diagram", "Ctrl+Shift+["),
    c("view:zoom-in", "Zoom In", "Ctrl+="),
    c("view:zoom-out", "Zoom Out", "Ctrl+-"),
    c("view:actual-size", "Actual Size", "Ctrl+0"),
    c("view:fit-to-window", "Fit To Window", "Ctrl+9"),
    c("view:show-grid", "Show Grid", "Ctrl+G"),
    c("view:snap-to-grid", "Snap to Grid", "Ctrl+Shift+G"),
    c("view:sidebar", "Sidebar", "Ctrl+1"),
    c("view:navigator", "Navigator", "Ctrl+2"),
    c("view:toolbar", "Toolbar", "Ctrl+3"),
    c("view:statusbar", "Statusbar", "Ctrl+4"),
    c("view:toolbox", "Toolbox", "Ctrl+5"),
    c("view:editors", "Editors", "Ctrl+6"),
];

/// Commands whose id or name contains the text, case-insensitive (command-palette.js:53-55).
pub fn matching(query: &str) -> Vec<&'static Command> {
    let q = query.trim().to_lowercase();
    COMMANDS.iter().filter(|c| q.is_empty() || c.id.contains(&q) || c.name.to_lowercase().contains(&q)).collect()
}

/// A Quick Find row: a model, or one of its views on a diagram.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hit {
    Model(Eid),
    View(Eid),
}

impl App {
    /// Quick Find results: models whose name contains the text (at most 99), each followed by its views
    /// on diagrams (quick-find.js:46-81).
    pub fn find_hits(&self, query: &str) -> Vec<Hit> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return vec![];
        }
        let models: Vec<Eid> = self.repo.all().filter(|e| !self.is_view(*e) && self.repo.name(*e).to_lowercase().contains(&q)).take(99).collect();
        let mut out = vec![];
        for m in models {
            out.push(Hit::Model(m));
            for d in self.repo.all_of("Diagram") {
                for v in self.repo.list(d, "ownedViews") {
                    if self.repo.refd(v, "model") == Some(m) {
                        out.push(Hit::View(v));
                    }
                }
            }
        }
        out
    }

    /// Picking a Quick Find row: a model is selected in the explorer (a diagram also opens); a view is
    /// selected in its diagram (quick-find.js:105-120).
    pub fn pick_hit(&mut self, h: Hit) {
        match h {
            Hit::Model(m) => {
                self.reveal(m);
                if self.is_diagram(m) {
                    self.open_diagram(m);
                }
                self.focus = Some(m);
            }
            Hit::View(v) => {
                if let Some(m) = self.repo.refd(v, "model") {
                    self.reveal(m);
                }
                self.show_view(v);
            }
        }
    }

    pub fn open_palette(&mut self, mode: Mode) {
        self.palette = Some(Palette { mode, query: String::new(), selected: 0, fresh: true });
    }

    /// Run a palette command by id.
    pub fn run_command(&mut self, ctx: &egui::Context, id: &str) {
        use crate::align::Align;
        let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
        let toggles = [
            ("format:auto-resize", "autoResize"), ("format:show-shadow", "showShadow"), ("format:word-wrap", "wordWrap"), ("format:show-visibility", "showVisibility"),
            ("format:show-namespace", "showNamespace"), ("format:show-property", "showProperty"), ("format:show-type", "showType"), ("format:show-multiplicity", "showMultiplicity"), ("format:show-operation-signature", "showOperationSignature"),
            ("format:suppress-attributes", "suppressAttributes"), ("format:suppress-operations", "suppressOperations"), ("format:suppress-literals", "suppressLiterals"),
        ];
        if let Some((_, f)) = toggles.iter().find(|t| t.0 == id) {
            self.toggle_flag(f);
            return;
        }
        let aligns = [
            ("alignment:align-left", Align::Left), ("alignment:align-right", Align::Right), ("alignment:align-center", Align::Center),
            ("alignment:align-top", Align::Top), ("alignment:align-bottom", Align::Bottom), ("alignment:align-middle", Align::Middle),
            ("alignment:space-equally-horizontally", Align::SpaceH), ("alignment:space-equally-vertically", Align::SpaceV),
            ("alignment:set-width-equally", Align::Width), ("alignment:set-height-equally", Align::Height), ("alignment:set-size-equally", Align::Size),
        ];
        if let Some((_, how)) = aligns.iter().find(|t| t.0 == id) {
            self.align(*how);
            return;
        }
        match id {
            "application:new" => {
                if self.confirm_discard(&ms) {
                    self.save_working_diagrams();
                    self.set_project(crate::model::Repo::blank(), None);
                }
            }
            "project:open" => {
                if self.confirm_discard(&ms) {
                    if let Some(p) = rfd::FileDialog::new().set_title("Open Model").add_filter("Models", &["mdj"]).pick_file() {
                        self.open_path(p);
                    }
                }
            }
            "project:save" => {
                self.save(&ms);
            }
            "project:save-as" => {
                self.save_as(&ms);
            }
            "project:export-diagram-to-png" => self.export_current(ctx, Format::Png),
            "project:export-diagram-to-jpeg" => self.export_current(ctx, Format::Jpeg),
            "project:export-diagram-to-svg" => self.export_current(ctx, Format::Svg),
            "application:preferences" => self.prefs_open = Some("General".into()),
            "edit:undo" => self.do_undo(),
            "edit:redo" => self.do_redo(),
            "edit:cut" => self.cmd_cut(Some(ctx)),
            "edit:copy" => self.cmd_copy(Some(ctx)),
            "edit:copy-diagram-as-image" => self.cmd_copy_diagram_image(ctx),
            "edit:paste" => self.cmd_paste(),
            "edit:delete" => self.delete_selection(false),
            "edit:delete-from-model" => self.delete_selection(true),
            "edit:move-up" | "edit:move-down" => {
                if let Some(f) = self.focus.filter(|f| self.repo.alive(*f) && self.repo.ty(*f) != "Project") {
                    self.move_element(f, id == "edit:move-up");
                }
            }
            "edit:select-all" => self.select_all(),
            "edit:select-in-explorer" => {
                if let Some(f) = self.focus {
                    self.reveal(f);
                }
            }
            "edit:select-in-diagram" => self.select_in_diagram(),
            "edit:open-sub-diagram" => self.open_sub_diagram(),
            "format:font" => self.open_font_dialog(),
            "format:fill-color" => self.color_dialog = Some("fillColor"),
            "format:line-color" => self.color_dialog = Some("lineColor"),
            "format:linestyle-rectilinear" => self.set_line_style(0),
            "format:linestyle-oblique" => self.set_line_style(1),
            "format:linestyle-roundrect" => self.set_line_style(2),
            "format:linestyle-curve" => self.set_line_style(3),
            "alignment:send-to-back" => self.reorder(false),
            "relationship-view:toggle" => self.show_relationships = !self.show_relationships,
            "diagram-generator:overview" => drop(self.generate_diagram(crate::generate::Generator::Overview, &ms)),
            "diagram-generator:overview-expanded" => drop(self.generate_diagram(crate::generate::Generator::OverviewExpanded, &ms)),
            "diagram-generator:type-hierarchy" => drop(self.generate_diagram(crate::generate::Generator::TypeHierarchy, &ms)),
            "diagram-generator:package-structure" => drop(self.generate_diagram(crate::generate::Generator::PackageStructure, &ms)),
            "diagram-layout:auto" | "diagram-layout:top-bottom" => self.layout_diagram(crate::layout::Dir::TopBottom),
            "diagram-layout:bottom-top" => self.layout_diagram(crate::layout::Dir::BottomTop),
            "diagram-layout:left-right" => self.layout_diagram(crate::layout::Dir::LeftRight),
            "diagram-layout:right-left" => self.layout_diagram(crate::layout::Dir::RightLeft),
            "alignment:bring-to-front" => self.reorder(true),
            "model:tag-editor" => self.open_tag_editor(),
            "view:quick-find" => self.open_palette(Mode::Find),
            "view:close-diagram" => self.close_diagram(None),
            "view:close-other-diagrams" => self.close_other_diagrams(),
            "view:close-all-diagrams" => self.close_all_diagrams(),
            "view:next-diagram" => self.cycle_diagram(1),
            "view:previous-diagram" => self.cycle_diagram(-1),
            "view:zoom-in" => self.zoom_keep_centre(self.zoom + crate::app::ZOOM_STEP),
            "view:zoom-out" => self.zoom_keep_centre(self.zoom - crate::app::ZOOM_STEP),
            "view:actual-size" => self.zoom_keep_centre(1.0),
            "view:fit-to-window" => self.fit(),
            "view:show-grid" => self.show_grid = !self.show_grid,
            "view:snap-to-grid" => self.snap = !self.snap,
            "view:sidebar" => self.show_sidebar = !self.show_sidebar,
            "view:navigator" => self.show_navigator = !self.show_navigator,
            "view:toolbar" => self.show_toolbar = !self.show_toolbar,
            "view:statusbar" => self.show_statusbar = !self.show_statusbar,
            "view:toolbox" => self.show_toolbox = !self.show_toolbox,
            "view:editors" => self.show_editors = !self.show_editors,
            _ => {}
        }
    }

    /// The palette box: the field at the top centre, the rows under it. Up/Down move, Enter picks, Esc
    /// or a click elsewhere closes it.
    pub fn palette_ui(&mut self, ctx: &egui::Context) {
        let Some(mut pal) = self.palette.clone() else { return };
        let hint = match pal.mode {
            Mode::Commands => "Search commands...",
            Mode::Find => "Find models, views and diagrams...",
        };
        let rows: Vec<(String, String, Option<Hit>, Option<&'static str>)> = match pal.mode {
            Mode::Commands => matching(&pal.query).into_iter().map(|c| (c.name.to_string(), c.key.to_string(), None, Some(c.id))).collect(),
            Mode::Find => self
                .find_hits(&pal.query)
                .into_iter()
                .map(|h| match h {
                    Hit::Model(m) => {
                        let ns = self.repo.parent(m).map(|p| self.repo.name(p)).filter(|n| !n.is_empty()).map(|n| format!(" — {n}")).unwrap_or_default();
                        (format!("{}{ns}", self.label_of(m)), String::new(), Some(h), None)
                    }
                    Hit::View(v) => {
                        let d = self.repo.parent(v).map(|d| self.repo.name(d)).unwrap_or_default();
                        (format!("      in {d}"), String::new(), Some(h), None)
                    }
                })
                .collect(),
        };
        let (up, down, enter, esc) = ctx.input_mut(|i| {
            (i.consume_key(egui::Modifiers::NONE, Key::ArrowUp), i.consume_key(egui::Modifiers::NONE, Key::ArrowDown), i.consume_key(egui::Modifiers::NONE, Key::Enter), i.consume_key(egui::Modifiers::NONE, Key::Escape))
        });
        if up {
            pal.selected = pal.selected.saturating_sub(1);
        }
        if down && pal.selected + 1 < rows.len() {
            pal.selected += 1;
        }
        let mut picked = enter.then_some(pal.selected);
        let mut lost = false;
        egui::Area::new(egui::Id::new("palette")).order(egui::Order::Foreground).anchor(Align2::CENTER_TOP, Vec2::new(0.0, 40.0)).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(460.0);
                let r = ui.add(egui::TextEdit::singleline(&mut pal.query).hint_text(hint).desired_width(f32::INFINITY));
                if pal.fresh {
                    r.request_focus();
                    pal.fresh = false;
                } else if r.lost_focus() && !enter {
                    lost = true;
                }
                if r.changed() {
                    pal.selected = 0;
                }
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    for (i, (name, key, _, _)) in rows.iter().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.selectable_label(i == pal.selected, name).clicked() {
                                picked = Some(i);
                            }
                            if !key.is_empty() {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.weak(key));
                            }
                        });
                    }
                });
            });
        });
        if esc || (lost && picked.is_none()) {
            self.palette = None;
            return;
        }
        if let Some(i) = picked {
            self.palette = None;
            match rows.get(i) {
                Some((_, _, Some(h), _)) => self.pick_hit(*h),
                Some((_, _, None, Some(id))) => self.palette_run = Some(id),
                _ => {}
            }
            return;
        }
        self.palette = Some(pal);
    }
}
