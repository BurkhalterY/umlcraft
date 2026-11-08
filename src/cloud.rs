//! Cloud architecture diagrams (AWS, Azure and GCP): what the three notations share. Their groups
//! and zones hold other elements as packages do, their services and resources show an icon, their
//! callouts a number, and their lines a name (extensions/essential/aws, azure and gcp:
//! metamodel.json, elements.js, *-factory.js, *-commands.js).
//!
//! The original draws each icon from an SVG or PNG file under its resources/assets folder. Those
//! icon sets are the vendors' trademarked artwork, so UMLCraft does not bundle them: the
//! `cloud.iconFolder` preference names a folder laid out the same way (aws/service-icons,
//! aws/group-icons, aws/resource-icons, aws/general-icons, azure, gcp/products, gcp/services,
//! gcp/users-and-devices, gcp/expanded-icons), and the icons are read from it. Without it, an icon is
//! a rounded square with the initials of its name.

use crate::model::{Eid, Repo};
use crate::render::{Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// The preference that names the icon folder (UMLCraft's own; docs/features/8-beyond-staruml.md).
pub const ICON_FOLDER_PREF: &str = "cloud.iconFolder";

// ------------------------------------------------------------------ the notations

/// Whether a model type is a box on a cloud diagram.
pub fn is_node_type(ty: &str) -> bool {
    crate::aws::is_node_type(ty)
}

/// Whether a view type is the view of such a box.
pub fn is_node_view(ty: &str) -> bool {
    ty.strip_suffix("View").map(is_node_type).unwrap_or(false)
}

/// The lines of the cloud notations and their views.
pub fn edge_view_of(model_ty: &str) -> Option<&'static str> {
    match model_ty {
        "AWSArrow" => Some("AWSArrowView"),
        _ => None,
    }
}

pub fn is_edge_view(ty: &str) -> bool {
    matches!(ty, "AWSArrowView")
}

/// The common kind of a notation's elements, by the prefix of a type ("AWSService" → "AWSElement").
pub fn element_kind(ty: &str) -> Option<&'static str> {
    if ty.starts_with("AWS") {
        Some("AWSElement")
    } else {
        None
    }
}

/// The diagram kinds, their models and their Model › Add Diagram and Model › Add labels
/// (aws/menus/aws.json): (diagram, model, diagram label, model label).
pub const DIAGRAMS: [(&str, &str, &str, &str); 1] = [("AWSDiagram", "AWSModel", "AWS Architecture Diagram", "AWS Model")];

pub fn is_diagram(ty: &str) -> bool {
    DIAGRAMS.iter().any(|d| d.0 == ty)
}

/// The model a diagram kind is wrapped in when it is made right under the project.
pub fn model_of(diagram_ty: &str) -> Option<&'static str> {
    DIAGRAMS.iter().find(|d| d.0 == diagram_ty).map(|d| d.1)
}

/// The folder (under the icon folder) a model type takes its icon from, and the field Set Icon
/// renames from the icon's file name (`__assetBaseDir` of each class; aws-commands.js:31-60,
/// gcp-commands.js:31-80). Types without one have no icon picker.
pub fn icon_base(model_ty: &str) -> Option<(&'static str, &'static str)> {
    crate::aws::icon_base(model_ty)
}

/// The icon a new element starts with (the constructors in each elements.js).
pub fn default_icon(model_ty: &str) -> Option<&'static str> {
    crate::aws::default_icon(model_ty)
}

/// What a diagram takes a view of (canAcceptModel of each diagram, aws/elements.js:105-115).
pub fn accepted(diagram_ty: &str) -> &'static [&'static str] {
    match diagram_ty {
        "AWSDiagram" => &["Hyperlink", "Diagram", "AWSElement", "AWSArrow"],
        _ => &[],
    }
}

/// A new element's own values: its first icon (the constructors).
pub fn init_model(r: &mut Repo, m: Eid) {
    if let Some(i) = default_icon(r.ty(m)) {
        r.set(m, "icon", json!(i));
    }
}

/// Whether a box can hold another: a group or zone holds the other boxes of its notation
/// (canContainViewKind of each group view).
pub fn can_contain(container_ty: &str, view_ty: &str) -> bool {
    crate::aws::can_contain(container_ty, view_ty)
}

/// A new element's view: a general box with a word-wrapped name label, or an icon with its name
/// floating under it (AWSGeneralNodeView, AWSIconNodeView constructors).
pub fn create_view(r: &mut Repo, diagram: Eid, model: Eid, rect: R) -> Eid {
    let ty = format!("{}View", r.ty(model));
    let m = r.ref_to(model);
    let v = r.create(
        &ty,
        &[
            ("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(false)), ("containerChangeable", json!(true)),
            ("left", json!(rect.x)), ("top", json!(rect.y)), ("width", json!(rect.w)), ("height", json!(rect.h)),
        ],
        Some((diagram, "ownedViews")),
    );
    if is_icon_view(&ty) {
        // 42 below the icon's centre (alpha -π/2)
        let l = r.create(
            "NodeLabelView",
            &[("model", m), ("font", json!(views::FONT)), ("alpha", json!(-std::f64::consts::FRAC_PI_2)), ("distance", json!(42)), ("height", json!(views::LABEL_H))],
            Some((v, "subViews")),
        );
        r.set_ref(v, "nameLabel", Some(l));
    } else {
        r.set(v, "wordWrap", json!(true));
        let l = r.create(
            "LabelView",
            &[("model", m), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(label_align(&ty))), ("verticalAlignment", json!(5)), ("wordWrap", json!(true))],
            Some((v, "subViews")),
        );
        r.set_ref(v, "nameLabel", Some(l));
    }
    v
}

/// The colours a view's constructor sets, over the preferred ones: a callout is black with white
/// text (AWSCalloutView constructor).
pub fn init_view(r: &mut Repo, v: Eid) {
    crate::aws::init_view(r, v);
}

/// Whether a view is an icon with a floating name (AWSIconNodeView).
pub fn is_icon_view(view_ty: &str) -> bool {
    crate::aws::is_icon_view(view_ty)
}

/// A general box's label alignment (Canvas.AL_LEFT 0, AL_CENTER 2).
fn label_align(view_ty: &str) -> i64 {
    if view_ty == "AWSGroupView" {
        0
    } else {
        2
    }
}

/// Size a box and place its labels (sizeObject, arrangeObject and update of each view).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    if !r.refd(v, "model").map(|m| r.alive(m)).unwrap_or(false) {
        return;
    }
    crate::aws::layout(r, ms, v);
}

/// Draw a box.
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    if r.refd(v, "model").is_none() {
        return;
    }
    crate::aws::draw(c, r, v, fill, line, fc);
}

/// What a line's update sets on its view, saved with it.
pub fn update_edge(_r: &mut Repo, _e: Eid) {}

/// The quick edit's dropdowns of an element: (field, hint, options as (value, label)).
pub fn dropdowns(_r: &Repo, _m: Eid) -> Vec<(&'static str, &'static str, Vec<(Value, &'static str)>)> {
    vec![]
}

/// The attributes the Properties panel shows (the `visible` ones of each metamodel.json; the icon
/// is hidden there).
pub fn fields(ty: &str) -> Vec<(&'static str, &'static str)> {
    let mut f = vec![("name", "text")];
    if edge_view_of(ty).is_some() {
        f.extend([("source", "ref"), ("target", "ref")]);
    }
    match ty {
        "AWSGenericGroup" => f.push(("dashed", "bool")),
        _ => {}
    }
    f
}

/// The icon button's hint in the quick edit ("AWS Icon"; quickedits/aws.json).
pub fn icon_hint(model_ty: &str) -> &'static str {
    if model_ty.starts_with("AWS") {
        "AWS Icon"
    } else {
        "Icon"
    }
}

// ------------------------------------------------------------------ shared layout and drawing

/// Measure a word-wrapped name: (the narrowest the label may be, its lines at `width`).
pub(crate) fn measure_label(ms: &dyn Measure, text: &str, width: f64, wrap: bool, style: u8) -> (f64, usize) {
    let min = if wrap { views::widest_word(ms, text, style) } else { ms.width(text, style) }.ceil();
    let lines = if wrap && !text.is_empty() { views::wrap_text(ms, text, width.max(1.0), style).len() } else { 1 };
    (min, lines.max(1))
}

/// Put a label at a box, with its text and the view's font.
pub(crate) fn set_label(r: &mut Repo, l: Eid, text: &str, font: &str, b: R, wrap: bool) {
    r.set(l, "text", json!(text));
    r.set(l, "font", json!(font));
    r.set(l, "wordWrap", json!(wrap));
    for (k, val) in [("left", b.x), ("top", b.y), ("width", b.w), ("height", b.h)] {
        r.set(l, k, json!(val));
    }
}

/// Grow a box to its smallest size (to exactly that size with autoResize).
pub(crate) fn fit(r: &mut Repo, v: Eid, min_w: f64, min_h: f64) -> (f64, f64) {
    let (w0, h0) = (r.f(v, "width"), r.f(v, "height"));
    let (w, h) = if r.bool(v, "autoResize") { (min_w, min_h) } else { (w0.max(min_w), h0.max(min_h)) };
    r.set(v, "width", json!(w));
    r.set(v, "height", json!(h));
    (w, h)
}

/// The view's font and a measure scaled to it.
pub(crate) fn font_of(r: &Repo, v: Eid) -> (String, f64) {
    views::view_font(r, v)
}

/// Lay out a general box: its name `pad` inside, word-wrapped (AWSGeneralNodeView.sizeObject and
/// arrangeObject); returns the box and the label's lines and line height.
pub(crate) fn layout_general(r: &mut Repo, ms: &dyn Measure, v: Eid, pad: f64) -> (R, usize, f64) {
    let m = r.refd(v, "model").unwrap();
    let (face, size) = font_of(r, v);
    let sm = views::Scaled { ms, face: face.clone(), size };
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let text = r.name(m);
    let w0 = r.f(v, "width");
    let (min_lw, _) = measure_label(&sm, &text, 0.0, wrap, 0);
    let lw = (w0.max(min_lw + 2.0 * pad) - 2.0 * pad).max(1.0);
    let (_, lines) = measure_label(&sm, &text, lw, wrap, 0);
    let (w, h) = fit(r, v, min_lw + 2.0 * pad, lines as f64 * size + 2.0 * pad);
    let b = R { x: r.f(v, "left"), y: r.f(v, "top"), w, h };
    if let Some(l) = r.refd(v, "nameLabel") {
        set_label(r, l, &text, &views::font_str(&face, size, 0), R { x: b.x + pad, y: b.y + pad, w: w - 2.0 * pad, h: lines as f64 * size }, wrap);
    }
    (b, lines, size)
}

/// Lay out an icon box: at least `min` on each side, its name floating 42 under its centre and
/// shown only when there is one (AWSIconNodeView.sizeObject, update and arrange).
pub(crate) fn layout_icon(r: &mut Repo, ms: &dyn Measure, v: Eid, min: f64) {
    let m = r.refd(v, "model").unwrap();
    fit(r, v, min, min);
    if let Some(l) = r.refd(v, "nameLabel") {
        let (face, size) = font_of(r, v);
        let sm = views::Scaled { ms, face: face.clone(), size };
        r.set(l, "font", json!(views::font_str(&face, size, 0)));
        let name = r.name(m);
        views::place_floating_label(r, &sm, v, l, &name, true);
    }
}

/// Draw a label's lines in its box, centred top to bottom (verticalAlignment AL_MIDDLE).
pub(crate) fn draw_label(c: &mut dyn Canvas, r: &Repo, l: Eid, fc: Rgba) {
    if r.raw(l, "visible").and_then(|x| x.as_bool()) == Some(false) {
        return;
    }
    let t = r.str(l, "text");
    if t.is_empty() {
        return;
    }
    let font = r.str(l, "font");
    let mut it = font.split(';');
    let face = it.next().filter(|f| !f.is_empty()).unwrap_or("Arial").to_string();
    let size = it.next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(13.0);
    let style = it.next().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
    let b = views::node_rect(r, l);
    let align = r.val(l, "horizontalAlignment").and_then(|x| x.as_i64()).unwrap_or(2);
    let lines = if r.bool(l, "wordWrap") { views::wrap_text(&views::Scaled { ms: &*c, face, size }, &t, b.w, style) } else { vec![t] };
    let top = b.y + ((b.h - lines.len() as f64 * size) / 2.0).max(0.0);
    for (i, s) in lines.iter().enumerate() {
        c.text(s, R { x: b.x, y: top + i as f64 * size, w: b.w, h: size }, align, style, size, fc, false);
    }
}

/// A rectangle's outline, solid or in dashes of `pattern` (canvas.rect with a dash pattern).
pub(crate) fn outline(c: &mut dyn Canvas, pts: &[P], line: Rgba, pattern: Option<(f64, f64)>) {
    let mut ring = pts.to_vec();
    ring.push(pts[0]);
    match pattern {
        Some((on, off)) => crate::render::dashes(c, &ring, line, on, off),
        None => c.polyline(&ring, line, false),
    }
}

pub(crate) fn rect_pts(b: R) -> Vec<P> {
    vec![P { x: b.x, y: b.y }, P { x: b.x + b.w, y: b.y }, P { x: b.x + b.w, y: b.y + b.h }, P { x: b.x, y: b.y + b.h }]
}

/// A callout: a filled circle (in the view's fill colour) with its number centred
/// (AWSCalloutView.drawObject).
pub(crate) fn draw_callout(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    c.ellipse(P { x: b.x + b.w / 2.0, y: b.y + b.h / 2.0 }, b.w / 2.0, b.h / 2.0, Some(fill), None);
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_label(c, r, l, fc);
    }
}

/// Lay out a callout: at least 24 × 24, its number across it, centred top to bottom
/// (AWSCalloutView.sizeObject and arrange).
pub(crate) fn layout_callout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let m = r.refd(v, "model").unwrap();
    let (face, size) = font_of(r, v);
    let sm = views::Scaled { ms, face: face.clone(), size };
    let (w, h) = fit(r, v, CALLOUT_MIN, CALLOUT_MIN);
    let (x, y) = (r.f(v, "left"), r.f(v, "top"));
    let text = r.name(m);
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let (_, lines) = measure_label(&sm, &text, w, wrap, 0);
    let lh = lines as f64 * size;
    if let Some(l) = r.refd(v, "nameLabel") {
        set_label(r, l, &text, &views::font_str(&face, size, 0), R { x, y: y + ((h - lh) / 2.0).round(), w, h: lh }, wrap);
    }
}

pub const CALLOUT_MIN: f64 = 24.0;

// ------------------------------------------------------------------ icons

fn icon_dir_lock() -> &'static RwLock<Option<PathBuf>> {
    static D: OnceLock<RwLock<Option<PathBuf>>> = OnceLock::new();
    D.get_or_init(|| RwLock::new(None))
}

type Cache = HashMap<PathBuf, Option<Arc<String>>>;

fn cache() -> &'static Mutex<Cache> {
    static C: OnceLock<Mutex<Cache>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Take the icon folder from the preference (an empty text is none); a new folder forgets the
/// icons read so far.
pub fn set_icon_dir(p: Option<PathBuf>) {
    let p = p.filter(|p| !p.as_os_str().is_empty());
    let mut d = icon_dir_lock().write().unwrap();
    if *d != p {
        *d = p;
        cache().lock().unwrap().clear();
    }
}

pub fn icon_dir() -> Option<PathBuf> {
    icon_dir_lock().read().unwrap().clone()
}

/// An icon file of the folder as a data URL, read once (None when it cannot be read).
pub fn icon_url(dir: &Path, base: &str, file: &str) -> Option<Arc<String>> {
    if file.is_empty() {
        return None;
    }
    let path = dir.join(base).join(file);
    let mut c = cache().lock().unwrap();
    if let Some(hit) = c.get(&path) {
        return hit.clone();
    }
    let url = crate::picture::load_file(&path).ok().map(|(u, _, _)| Arc::new(u));
    c.insert(path, url.clone());
    url
}

/// The files of one icon folder, in name order (fs.readdirSync, icon-picker-dialog.js:127-131).
pub fn list_icons(dir: &Path, base: &str) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(dir.join(base)) else { return vec![] };
    let mut v: Vec<String> = rd.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).filter_map(|e| e.file_name().to_str().map(String::from)).collect();
    v.sort();
    v
}

/// An icon's label in the picker: its file name without ".svg", with "-" and "_" as spaces
/// (IconPickerDialog._toDataItem, icon-picker-dialog.js:49-58).
pub fn picker_label(file: &str) -> String {
    file.replace(".svg", "").replace(['-', '_'], " ")
}

/// The name Set Icon gives an element: the icon's file name without its extension, with "-" and
/// "_" as spaces (path.parse(file).name; aws-commands.js:45-50).
pub fn name_from_icon(file: &str) -> String {
    let stem = Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or(file);
    stem.replace(['-', '_'], " ")
}

/// The initials a placeholder shows: of the icon's words, without the vendors' names ("Amazon
/// EC2" → "EC2", "App Services" → "AS").
pub fn initials(file: &str) -> String {
    let name = name_from_icon(file);
    let words: Vec<&str> = name.split_whitespace().filter(|w| !matches!(w.to_ascii_lowercase().as_str(), "amazon" | "aws" | "azure" | "google" | "microsoft")).collect();
    match words.as_slice() {
        [] => String::new(),
        [w] if w.chars().count() <= 4 => w.to_string(),
        [w] => w.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default(),
        ws => ws.iter().take(3).filter_map(|w| w.chars().next()).flat_map(|c| c.to_uppercase()).collect(),
    }
}

/// Draw an icon over a box: the file from the icon folder; without the folder (or the file), a
/// rounded square in `tint` with its initials. No icon at all is a crossed box, as in the original
/// (loadAndDrawImage, elements.js:55-85).
pub fn draw_icon(c: &mut dyn Canvas, base: &str, file: &str, b: R, tint: Rgba, line: Rgba) {
    if file.is_empty() {
        let p = |x: f64, y: f64| P { x, y };
        c.polyline(&[p(b.x, b.y), p(b.x + b.w, b.y + b.h)], line, false);
        c.polyline(&[p(b.x + b.w, b.y), p(b.x, b.y + b.h)], line, false);
        c.rect(b, None, Some(line));
        return;
    }
    if let Some(url) = icon_dir().and_then(|d| icon_url(&d, base, file)) {
        c.image(b, &url);
        return;
    }
    let s = b.w.min(b.h);
    let sq = R { x: b.x + (b.w - s) / 2.0, y: b.y + (b.h - s) / 2.0, w: s, h: s };
    c.polygon(&crate::flowchart::round_box(sq, (s / 6.0).max(2.0)), Some(tint), None);
    let t = initials(file);
    if !t.is_empty() {
        let n = t.chars().count().max(2) as f64;
        let size = (s * 0.9 / n).min(s * 0.4).max(4.0);
        c.text(&t, R { x: sq.x, y: sq.y + (s - size) / 2.0, w: s, h: size }, 2, 1, size, crate::render::WHITE, false);
    }
}

// ------------------------------------------------------------------ the icon picker

/// The Icon Picker dialog ("Select Icon"): the element, the folder its icons come from and the
/// field the pick renames (icon-picker-dialog.js).
#[derive(Clone)]
pub struct IconPicker {
    pub model: Eid,
    pub base: &'static str,
    pub field: &'static str,
    pub filter: String,
    pub selected: Option<String>,
    /// How many icons are listed: 50 more each time the list is scrolled near its end.
    pub shown: usize,
    pub fresh: bool,
}

/// Icons listed at a time (itemsPerPage, icon-picker-dialog.js:37).
pub const PAGE: usize = 50;

/// The icons whose label or file name holds the filter, ignoring case (filterItems,
/// icon-picker-dialog.js:73-85).
pub fn filter_icons(files: &[String], filter: &str) -> Vec<String> {
    let f = filter.trim().to_lowercase();
    if f.is_empty() {
        return files.to_vec();
    }
    files.iter().filter(|x| picker_label(x).to_lowercase().contains(&f) || x.to_lowercase().contains(&f)).cloned().collect()
}

impl crate::app::App {
    /// Set Icon: the picker over the element's icon folder (aws:set-icon, gcp:set-product-icon).
    pub fn open_icon_picker(&mut self, m: Eid) {
        let Some((base, field)) = icon_base(self.repo.ty(m)) else { return };
        let cur = self.repo.str(m, "icon");
        self.icon_picker = Some(IconPicker { model: m, base, field, filter: String::new(), selected: (!cur.is_empty()).then_some(cur), shown: PAGE, fresh: true });
    }

    /// The picked icon: the element takes it and is renamed after it (handleSetIcon).
    pub fn set_icon(&mut self, m: Eid, field: &str, file: &str) {
        let name = name_from_icon(file);
        let file = file.to_string();
        let field = field.to_string();
        self.change(|a| {
            a.repo.set(m, "icon", json!(file));
            a.repo.set(m, &field, json!(name));
        });
    }

    pub fn icon_picker_ui(&mut self, ctx: &egui::Context) {
        let Some(mut p) = self.icon_picker.clone() else { return };
        if !self.repo.alive(p.model) {
            self.icon_picker = None;
            return;
        }
        let dir = icon_dir();
        let files = dir.as_ref().map(|d| list_icons(d, p.base)).unwrap_or_default();
        let (mut ok, mut cancel) = (false, false);
        let mut picked: Option<String> = None;
        egui::Window::new("Select Icon").id(egui::Id::new("icon-picker")).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            ui.set_width(520.0);
            if files.is_empty() {
                // no folder: the icon is typed by its file name
                let why = match &dir {
                    None => "The cloud icon sets are not bundled with UMLCraft. Choose a folder laid out like StarUML's resources/assets in Preferences › General › Cloud Icons to pick from them.".to_string(),
                    Some(d) => format!("No icons in {}.", d.join(p.base).display()),
                };
                ui.label(why);
                ui.add_space(4.0);
                ui.label(format!("Icon file name (in {}):", p.base));
                let mut t = p.selected.clone().unwrap_or_default();
                let r = ui.add(egui::TextEdit::singleline(&mut t).desired_width(f32::INFINITY));
                if p.fresh {
                    r.request_focus();
                }
                p.selected = (!t.trim().is_empty()).then(|| t.trim().to_string());
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    ok = true;
                }
            } else {
                let r = ui.add(egui::TextEdit::singleline(&mut p.filter).hint_text("Search").desired_width(f32::INFINITY));
                if p.fresh {
                    r.request_focus();
                }
                if r.changed() {
                    p.shown = PAGE;
                }
                let hits = filter_icons(&files, &p.filter);
                let dir = dir.clone().unwrap();
                let out = egui::ScrollArea::vertical().id_salt("icon-picker-list").max_height(360.0).auto_shrink([false, false]).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        for f in hits.iter().take(p.shown) {
                            let (rect, resp) = ui.allocate_exact_size(egui::vec2(96.0, 84.0), egui::Sense::click());
                            let sel = p.selected.as_deref() == Some(f.as_str());
                            if sel {
                                ui.painter().rect_filled(rect, 3.0, ui.visuals().selection.bg_fill);
                            } else if resp.hovered() {
                                ui.painter().rect_filled(rect, 3.0, ui.visuals().widgets.hovered.weak_bg_fill);
                            }
                            let ir = egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.min.y + 26.0), egui::vec2(40.0, 40.0));
                            match icon_texture(ui.ctx(), &dir, p.base, f) {
                                Some(t) => {
                                    ui.painter().image(t.id(), ir, egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                                }
                                None => {
                                    ui.painter().rect_stroke(ir, 3.0, ui.visuals().widgets.noninteractive.fg_stroke, egui::StrokeKind::Inside);
                                }
                            }
                            let label = picker_label(f);
                            let galley = ui.painter().layout(label, egui::FontId::proportional(11.0), ui.visuals().text_color(), 92.0);
                            let tp = egui::pos2(rect.center().x - galley.size().x / 2.0, rect.min.y + 50.0);
                            ui.painter().with_clip_rect(rect).galley(tp, galley, ui.visuals().text_color());
                            let resp = resp.on_hover_text(f);
                            if resp.clicked() {
                                p.selected = Some(f.clone());
                            }
                            if resp.double_clicked() {
                                picked = Some(f.clone());
                            }
                        }
                    });
                });
                // the next 50 when scrolled within 100 px of the end (setupInfiniteScroll)
                let near_end = out.state.offset.y + out.inner_rect.height() >= out.content_size.y - 100.0;
                if near_end && p.shown < hits.len() {
                    p.shown += PAGE;
                    ui.ctx().request_repaint();
                }
            }
            p.fresh = false;
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(p.selected.is_some(), egui::Button::new("OK")).clicked() {
                    ok = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel = true;
                }
            });
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                cancel = true;
            }
        });
        if ok {
            picked = p.selected.clone();
        }
        if let Some(f) = picked {
            self.icon_picker = None;
            self.set_icon(p.model, p.field, &f);
            return;
        }
        self.icon_picker = if cancel { None } else { Some(p) };
    }
}

/// A picker tile's picture, decoded once and kept in egui's memory.
fn icon_texture(ctx: &egui::Context, dir: &Path, base: &str, file: &str) -> Option<egui::TextureHandle> {
    let id = egui::Id::new(("cloud-icon", dir.join(base).join(file)));
    if let Some(t) = ctx.data_mut(|d| d.get_temp::<Option<egui::TextureHandle>>(id)) {
        return t;
    }
    let t = icon_url(dir, base, file).and_then(|u| crate::picture::decode(&u)).map(|img| {
        let size = [img.width() as usize, img.height() as usize];
        ctx.load_texture("cloud-icon", egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()), egui::TextureOptions::LINEAR)
    });
    ctx.data_mut(|d| d.insert_temp(id, t.clone()));
    t
}
