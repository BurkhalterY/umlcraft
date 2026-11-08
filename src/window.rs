//! The application shell, as StarUML's main process does it (src/main-process/*.js): one window per
//! project, what a window opens at start-up, its splash screen, the macOS title bar, the size it
//! opens at, closing with the unsaved-changes prompt, and relaunching.
//!
//! StarUML opens every window in one Electron process. Each UMLCraft window is a process of its
//! own: New, New From Template and Open (when this window already holds a file or changes) start
//! another `umlcraft` with the internal option `--new-window`, which is the original's
//! `skipLoadBackup` (application.js:246-258).

use crate::app::App;
use crate::canvas::EguiMeasure;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// The window was opened by another window (New, New From Template, Open): it skips the
/// auto-backup and the working file (`skipLoadBackup`, application.js:246-258).
pub const NEW_WINDOW: &str = "--new-window";
/// `--template <name or file.mdj>`: start a new project from a template (the `template` window
/// parameter, application.js:249-253).
pub const TEMPLATE: &str = "--template";

/// A relaunch was asked (application:reload): a new process starts once this one ends.
static RELAUNCH: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
thread_local! {
    /// The windows opened in a test, by their arguments (tests start no process).
    pub static SPAWNED: std::cell::RefCell<Vec<Vec<String>>> = const { std::cell::RefCell::new(vec![]) };
}

/// The state of the window around the project.
#[derive(Default)]
pub struct Shell {
    /// What to open once the window shows (while it is waiting, the splash screen is drawn).
    pub startup: Option<Startup>,
    frames: u32,
    /// The window size has been fitted to the screen.
    fitted: bool,
}

/// What a window opens at start-up (the window's initParams, app-context.js:688-728).
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Startup {
    pub file: Option<PathBuf>,
    pub template: Option<String>,
    /// `skipLoadBackup`: set for the windows another window opens.
    pub skip_backup: bool,
    /// `loadWorking`: set when the app is launched without a file.
    pub load_working: bool,
    /// `--diagram` and `--select`: a diagram to show and an element to select in it.
    pub diagram: Option<String>,
    pub select: Option<String>,
}

impl Startup {
    /// The start-up asked by the command line: a `*.mdj` file that exists, `--template`,
    /// `--new-window`, `--diagram` and `--select`.
    pub fn from_args(args: &[String]) -> Startup {
        let mut s = Startup::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--diagram" => {
                    s.diagram = args.get(i + 1).cloned();
                    i += 1;
                }
                "--select" => {
                    s.select = args.get(i + 1).cloned();
                    i += 1;
                }
                "--screenshot" => i += 1,
                TEMPLATE => {
                    s.template = args.get(i + 1).cloned();
                    i += 1;
                }
                NEW_WINDOW => s.skip_backup = true,
                a if a.ends_with(".mdj") && std::path::Path::new(a).exists() => s.file = Some(PathBuf::from(a)),
                _ => {}
            }
            i += 1;
        }
        // the app launched by the system without a file opens the working file (main.js:20-31)
        s.load_working = s.file.is_none() && s.template.is_none() && !s.skip_backup;
        s
    }
}

/// Start another UMLCraft window with these arguments. The process is waited for on a thread of
/// its own, so it leaves nothing behind when it ends.
pub fn spawn(args: &[String]) -> std::io::Result<()> {
    #[cfg(test)]
    {
        SPAWNED.with(|s| s.borrow_mut().push(args.to_vec()));
        Ok(())
    }
    #[cfg(not(test))]
    {
        use std::process::{Command, Stdio};
        let exe = std::env::current_exe()?;
        let mut child = Command::new(exe).args(args).stdin(Stdio::null()).spawn()?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}

/// Ask for a relaunch once the window has closed (application:reload, default-commands.js:90-93).
pub fn request_relaunch() {
    RELAUNCH.store(true, Ordering::SeqCst);
}

/// After the last window closed: start the app again with the same arguments if asked to.
pub fn relaunch_if_requested(args: &[String]) {
    if RELAUNCH.load(Ordering::SeqCst) {
        let _ = spawn(args);
    }
}

/// The answer to "Do you want to save the changes?" (window.js:203-222).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SaveChoice {
    Save,
    DontSave,
    Cancel,
}

/// The native "Save Changes" box with Save, Don't Save and Cancel.
pub fn ask_save_changes() -> SaveChoice {
    let r = rfd::MessageDialog::new()
        .set_title("Save Changes")
        .set_description("Do you want to save the changes?")
        .set_buttons(rfd::MessageButtons::YesNoCancelCustom("Save".into(), "Don't Save".into(), "Cancel".into()))
        .show();
    // the custom buttons come back by label (some platforms report Yes/No instead)
    match r {
        rfd::MessageDialogResult::Yes => SaveChoice::Save,
        rfd::MessageDialogResult::Custom(b) if b == "Save" => SaveChoice::Save,
        rfd::MessageDialogResult::No => SaveChoice::DontSave,
        rfd::MessageDialogResult::Custom(b) if b == "Don't Save" => SaveChoice::DontSave,
        _ => SaveChoice::Cancel,
    }
}

/// The title bar of a macOS window is hidden and the app draws its own (window.js:114-116,
/// titlebar-view.js:62-75); elsewhere the system's title bar stays.
pub const HIDDEN_TITLE_BAR: bool = cfg!(target_os = "macos");
/// The height of the drawn title bar, and the room left on its left for the traffic lights.
const TITLE_BAR_H: f32 = 28.0;
const TRAFFIC_LIGHTS_W: f32 = 78.0;

/// The window options of the drawn title bar.
pub fn title_bar_options(v: egui::ViewportBuilder) -> egui::ViewportBuilder {
    if HIDDEN_TITLE_BAR {
        v.with_fullsize_content_view(true).with_titlebar_shown(false).with_title_shown(false)
    } else {
        v
    }
}

/// The saved bounds kept on the screen: negative coordinates become 0 (window.js:56-62).
pub fn clamp_saved_position(x: f32, y: f32) -> (f32, f32) {
    (x.max(0.0), y.max(0.0))
}

/// The window fitted to the screen it opens on: no wider or taller than the screen, its right and
/// bottom edges pulled in when they stick out (window.js:84-94). Returns the new (x, y, w, h).
pub fn fit_to_screen(x: f32, y: f32, w: f32, h: f32, sw: f32, sh: f32) -> (f32, f32, f32, f32) {
    let (w, h) = (w.min(sw), h.min(sh));
    let x = if x + w > sw && x < sw { sw - w } else { x };
    let y = if y + h > sh && y < sh { sh - h } else { y };
    (x.max(0.0), y.max(0.0), w, h)
}

impl App {
    /// Open what the window was started for, in the original's order (app-context.js:688-728):
    /// the auto-backup after a crash (unless another window opened this one), the file, the
    /// template, the working file when "Auto load working file" is on and the app was launched
    /// without a file, else a new project.
    pub fn start(&mut self, s: Startup) {
        if !(s.skip_backup || self.screenshot.is_some()) && self.recover_backup() {
        } else if let Some(f) = s.file.clone() {
            self.open_path(f);
        } else if let Some(t) = s.template.clone() {
            self.open_template(&t);
        } else if s.load_working && self.screenshot.is_none() && self.working_file_to_load().is_some() {
            self.auto_load_working_file();
        } else if self.screenshot.is_none() {
            self.file_new();
        }
        self.show_by_name(s.diagram.as_deref(), s.select.as_deref());
    }

    /// The working file, when it is to open at launch.
    fn working_file_to_load(&self) -> Option<PathBuf> {
        self.pref("general.working-file").as_bool().filter(|b| *b)?;
        self.working_file().filter(|f| f.exists())
    }

    /// A new project in this window, from the default template; the working file is forgotten
    /// (doFileNew, default-commands.js:106-116).
    pub fn file_new(&mut self) {
        self.save_working_diagrams();
        self.set_project(crate::model::Repo::blank(), None);
        self.set_working_file(None);
    }

    /// A new project from a template: a built-in template by its menu label, or any `.mdj` file,
    /// opened without its file name so the next Save asks for one (loadAsTemplate,
    /// project-manager.js:182-197).
    pub fn open_template(&mut self, t: &str) {
        if let Some(tpl) = crate::templates::TEMPLATES.iter().find(|x| x.label == t) {
            self.save_working_diagrams();
            self.set_project(crate::templates::project(tpl), None);
            return;
        }
        match std::fs::read_to_string(t).map_err(|e| e.to_string()).and_then(|x| crate::model::Repo::parse(&x)) {
            Ok(repo) => {
                self.save_working_diagrams();
                self.set_project(repo, None);
            }
            Err(e) => self.message = Some((format!("Failed to open {t}"), e)),
        }
    }

    /// Open a window with these arguments; when that fails, say so.
    fn open_window(&mut self, args: Vec<String>) -> bool {
        match spawn(&args) {
            Ok(()) => true,
            Err(e) => {
                self.toast_error(&format!("Failed to open a new window: {e}"));
                false
            }
        }
    }

    /// File > New (application:new): a new window with a new project (application.js:246-248).
    pub fn new_window(&mut self) {
        self.open_window(vec![NEW_WINDOW.into()]);
    }

    /// File > New From Template (application:new-from-template): a new window with the template
    /// (application.js:249-253).
    pub fn new_window_from_template(&mut self, label: &str) {
        self.open_window(vec![NEW_WINDOW.into(), TEMPLATE.into(), label.into()]);
    }

    /// Open a model file (project:open): in a new window when this one already holds a file or
    /// changes, else here (default-commands.js:211-234).
    pub fn open_file(&mut self, p: PathBuf) {
        if self.modified || self.filename.is_some() {
            let full = std::fs::canonicalize(&p).unwrap_or(p);
            self.open_window(vec![NEW_WINDOW.into(), full.display().to_string()]);
        } else {
            self.open_path(p);
        }
    }

    /// The Open dialog, then the file it picked (default-commands.js:221-234).
    pub fn open_dialog(&mut self) {
        if let Some(p) = rfd::FileDialog::new().set_title("Select a model file...").add_filter("Models", &["mdj"]).add_filter("All Files", &["*"]).pick_file() {
            self.open_file(p);
        }
    }

    /// A close of the window was asked: with changes, the "Save Changes" box. Save saves and keeps
    /// the window open, Don't Save closes it, Cancel keeps it (window.js:203-222). A close that
    /// goes through saves the working diagrams and removes the auto-backup marker, so no backup
    /// is offered at the next launch (app-context.js:809-811).
    pub(crate) fn handle_close(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.viewport().close_requested()) {
            return;
        }
        if self.screenshot.is_some() {
            return;
        }
        self.save_working_diagrams();
        if self.modified && !self.quit_after_save {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            match ask_save_changes() {
                SaveChoice::Save => {
                    self.save(&EguiMeasure { ctx: ctx.clone() });
                }
                SaveChoice::DontSave => {
                    self.quit_after_save = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                SaveChoice::Cancel => {}
            }
        } else {
            self.clear_backup();
        }
    }

    /// The frame around the project, drawn before the panels: the window fitted to the screen once,
    /// the macOS title bar, and the splash screen until the start-up file is open (app-context.js:737).
    /// Returns true while the splash screen is all the window shows.
    pub(crate) fn shell_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let ctx = ui.ctx().clone();
        self.shell.frames = self.shell.frames.saturating_add(1);
        if !self.shell.fitted && self.screenshot.is_none() {
            let (outer, inner, monitor, maximized) = ctx.input(|i| (i.viewport().outer_rect, i.viewport().inner_rect, i.viewport().monitor_size, i.viewport().maximized.unwrap_or(false)));
            if let (Some(o), Some(inr), Some(m)) = (outer, inner, monitor) {
                self.shell.fitted = true;
                if !maximized {
                    let (x, y, w, h) = fit_to_screen(o.min.x, o.min.y, inr.width(), inr.height(), m.x, m.y);
                    if (w, h) != (inr.width(), inr.height()) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
                    }
                    if (x, y) != (o.min.x, o.min.y) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(x, y)));
                    }
                }
            } else if self.shell.frames > 10 {
                self.shell.fitted = true;
            }
        }
        if HIDDEN_TITLE_BAR {
            self.title_bar_ui(ui);
        }
        let Some(s) = self.shell.startup.take() else { return false };
        if self.shell.frames <= 1 {
            self.shell.startup = Some(s);
            splash_ui(ui);
            ctx.request_repaint();
            return true;
        }
        self.start(s);
        false
    }

    /// The drawn title bar of macOS: the window title in the middle, room for the traffic lights,
    /// dragging moves the window and a double-click zooms it (titlebar-view.js:34-75).
    fn title_bar_ui(&mut self, ui: &mut egui::Ui) {
        use crate::ui::{LAYER0, TEXT};
        let title = self.title();
        egui::Panel::top("titlebar").exact_size(TITLE_BAR_H).show_separator_line(false).frame(egui::Frame::NONE.fill(LAYER0())).show(ui, |ui| {
            let rect = ui.max_rect();
            let r = ui.interact(rect, egui::Id::new("titlebar-drag"), egui::Sense::click_and_drag());
            if r.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            if r.double_clicked() {
                let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
            }
            let text_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x + TRAFFIC_LIGHTS_W, rect.min.y), rect.max);
            ui.painter().with_clip_rect(text_rect).text(rect.center(), egui::Align2::CENTER_CENTER, title, egui::FontId::proportional(13.0), TEXT());
        });
    }
}

/// The splash screen: the app icon and name on the panel colour, until the project is open.
fn splash_ui(ui: &mut egui::Ui) {
    use crate::ui::{DIM, LAYER0, TEXT};
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, LAYER0());
    let ctx = ui.ctx().clone();
    let id = egui::Id::new("splash-icon");
    let tex = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)).unwrap_or_else(|| {
        let t = image::load_from_memory(include_bytes!("../packaging/icons/umlcraft-256.png")).ok().map(|img| {
            let img = img.to_rgba8();
            let size = [img.width() as usize, img.height() as usize];
            ctx.load_texture("splash-icon", egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR)
        });
        ctx.data_mut(|d| d.insert_temp(id, t.clone()));
        t
    });
    let c = rect.center();
    if let Some(t) = tex {
        let icon = egui::Rect::from_center_size(egui::pos2(c.x, c.y - 40.0), egui::vec2(96.0, 96.0));
        ui.painter().image(t.id(), icon, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    }
    ui.painter().text(egui::pos2(c.x, c.y + 30.0), egui::Align2::CENTER_CENTER, "UMLCraft", egui::FontId::proportional(22.0), TEXT());
    ui.painter().text(egui::pos2(c.x, c.y + 56.0), egui::Align2::CENTER_CENTER, env!("CARGO_PKG_VERSION"), egui::FontId::proportional(12.0), DIM());
}

/// A file name picked in the Save dialog: `.mdj` added when it has no extension (default-commands.js:163-165).
pub fn with_model_extension(mut p: PathBuf) -> PathBuf {
    if p.extension().is_none() {
        p.set_extension("mdj");
    }
    p
}
