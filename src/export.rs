//! File > Export Diagram As: PNG, JPEG and SVG of the current diagram, or of every diagram into a folder
//! (default-commands.js:360-535, diagram-export.js:106-298); and StarUML 8's Export Image dialog
//! (Ctrl+Shift+E) with its range, format, background, scale and margin options.

use crate::app::App;
use crate::model::Eid;
use crate::views::Measure;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Format {
    Png,
    Jpeg,
    Svg,
}

impl Format {
    pub fn ext(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Jpeg => "jpg",
            Format::Svg => "svg",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Format::Png => "PNG",
            Format::Jpeg => "JPEG",
            Format::Svg => "SVG",
        }
    }
}

/// A file name safe on every system, as the `filenamify` package makes it: reserved characters and
/// control characters become "!".
pub fn filenamify(name: &str) -> String {
    let s: String = name.chars().map(|c| if "<>:\"/\\|?*".contains(c) || c.is_control() { '!' } else { c }).collect();
    let s = s.trim_end_matches(['.', ' ']).to_string();
    if s.is_empty() { "!".into() } else { s }
}

/// The picture of a diagram in this format. PNG keeps a transparent background; JPEG is flattened on
/// white (diagram-export.js:130-134).
pub fn render(app: &mut App, d: Eid, ms: &dyn Measure, f: Format, scale: f32) -> Result<Vec<u8>, String> {
    let info = crate::views::layout_diagram(&mut app.repo, ms, d);
    match f {
        Format::Svg => crate::render::to_svg(&app.repo, d, &info, ms).map(|s| s.into_bytes()).ok_or_else(|| "The diagram is empty.".into()),
        Format::Png => crate::render::to_png(&app.repo, d, &info, ms, scale),
        Format::Jpeg => {
            let png = crate::render::to_png(&app.repo, d, &info, ms, scale)?;
            let img = image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgba8();
            let mut rgb = image::RgbImage::new(img.width(), img.height());
            for (x, y, p) in img.enumerate_pixels() {
                let a = p.0[3] as u32;
                let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
                rgb.put_pixel(x, y, image::Rgb([mix(p.0[0]), mix(p.0[1]), mix(p.0[2])]));
            }
            let mut out = std::io::Cursor::new(Vec::new());
            rgb.write_to(&mut out, image::ImageFormat::Jpeg).map_err(|e| e.to_string())?;
            Ok(out.into_inner())
        }
    }
}

/// The Export Image dialog's options (https://docs.staruml.io/user-guide/managing-project/): Range,
/// Format, Fill background, Scale and Margin. Remembered in the preference `exportImage.options`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ImageOptions {
    /// Range: All Diagrams (true) or Current Diagram.
    pub all: bool,
    pub format: Format,
    /// Fill the background with the theme's background colour; JPEG is always filled.
    pub fill: bool,
    /// 1x or 2x; not used for SVG.
    pub scale: u8,
    /// Pixels around the drawing, 0 to 32.
    pub margin: u8,
}

pub const MAX_MARGIN: u8 = 32;

impl Default for ImageOptions {
    /// Current diagram, PNG, transparent, 1x, and the 10 px margin of StarUML 7's exports
    /// (diagram-export.js:33; the dialog's defaults are not documented).
    fn default() -> Self {
        ImageOptions { all: false, format: Format::Png, fill: false, scale: 1, margin: 10 }
    }
}

impl ImageOptions {
    /// Whether the background is filled: when asked, and always for JPEG.
    pub fn filled(&self) -> bool {
        self.fill || self.format == Format::Jpeg
    }
}

/// The picture of a diagram with the Export Image dialog's options: the bounds grown by the margin
/// (no extra width), the theme's `$background` behind it when filled, drawn at 1x or 2x.
pub fn render_with(app: &mut App, d: Eid, ms: &dyn Measure, o: &ImageOptions) -> Result<Vec<u8>, String> {
    let info = crate::views::layout_diagram(&mut app.repo, ms, d);
    let bg = o.filled().then(|| crate::render::parse_color("$background", crate::render::WHITE));
    let svg = crate::render::to_svg_framed(&app.repo, d, &info, ms, o.margin.min(MAX_MARGIN) as f64, 0.0, bg).ok_or("The diagram is empty.")?;
    match o.format {
        Format::Svg => Ok(svg.into_bytes()),
        Format::Png => crate::render::svg_to_png(&svg, o.scale.clamp(1, 2) as f32),
        Format::Jpeg => {
            let png = crate::render::svg_to_png(&svg, o.scale.clamp(1, 2) as f32)?;
            let img = image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgb8();
            let mut out = std::io::Cursor::new(Vec::new());
            img.write_to(&mut out, image::ImageFormat::Jpeg).map_err(|e| e.to_string())?;
            Ok(out.into_inner())
        }
    }
}

impl App {
    pub fn image_options(&self) -> ImageOptions {
        let v = self.prefs.get("exportImage.options");
        let d = ImageOptions::default();
        let get = |k: &str| v.and_then(|v| v.get(k));
        ImageOptions {
            all: get("range").and_then(|x| x.as_str()).map(|x| x == "all").unwrap_or(d.all),
            format: match get("format").and_then(|x| x.as_str()) {
                Some("jpeg") => Format::Jpeg,
                Some("svg") => Format::Svg,
                Some("png") => Format::Png,
                _ => d.format,
            },
            fill: get("fillBackground").and_then(|x| x.as_bool()).unwrap_or(d.fill),
            scale: get("scale").and_then(|x| x.as_u64()).filter(|s| *s == 1 || *s == 2).map(|s| s as u8).unwrap_or(d.scale),
            margin: get("margin").and_then(|x| x.as_u64()).map(|m| m.min(MAX_MARGIN as u64) as u8).unwrap_or(d.margin),
        }
    }

    /// Each change of the options is kept straight away ("The options are remembered for the next time").
    pub fn set_image_options(&mut self, o: &ImageOptions) {
        let v = serde_json::json!({
            "range": if o.all { "all" } else { "current" },
            "format": match o.format { Format::Png => "png", Format::Jpeg => "jpeg", Format::Svg => "svg" },
            "fillBackground": o.fill,
            "scale": o.scale,
            "margin": o.margin,
        });
        self.prefs.set("exportImage.options", v);
        self.prefs.flush();
    }

    /// File › Export Image... (Ctrl+Shift+E).
    pub fn open_export_image(&mut self) {
        self.export_image_open = true;
        self.export_image_result = None;
    }

    /// Write every diagram with the dialog's options into a folder, named as Export All names them.
    pub fn export_all_with(&mut self, folder: &std::path::Path, ms: &dyn Measure, o: &ImageOptions) -> Result<usize, String> {
        self.export_all_by(folder, o.format, |a, d| render_with(a, d, ms, o))
    }

    /// The Export Image dialog: it stays open and shows what its last button did; Close or Esc closes it.
    pub fn export_image_ui(&mut self, ctx: &egui::Context) {
        if !self.export_image_open {
            return;
        }
        let mut o = self.image_options();
        let before = o;
        let (mut export, mut export_all, mut copy, mut close) = (false, false, false, false);
        egui::Window::new("Export Image").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            egui::Grid::new("export-image-grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Range");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut o.all, false, "Current Diagram");
                    ui.radio_value(&mut o.all, true, "All Diagrams");
                });
                ui.end_row();
                ui.label("Format");
                ui.horizontal(|ui| {
                    for f in [Format::Png, Format::Jpeg, Format::Svg] {
                        ui.radio_value(&mut o.format, f, f.label());
                    }
                });
                ui.end_row();
                ui.label("");
                let jpeg = o.format == Format::Jpeg;
                let mut fill = o.filled();
                ui.add_enabled(!jpeg, egui::Checkbox::new(&mut fill, "Fill background")).on_hover_text("Fill with the background color of the current theme. JPEG images are always filled.");
                if !jpeg {
                    o.fill = fill;
                }
                ui.end_row();
                ui.label("Scale");
                ui.add_enabled_ui(o.format != Format::Svg, |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut o.scale, 1, "1x");
                        ui.radio_value(&mut o.scale, 2, "2x");
                    });
                });
                ui.end_row();
                ui.label("Margin");
                ui.add(egui::Slider::new(&mut o.margin, 0..=MAX_MARGIN).suffix(" px"));
                ui.end_row();
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                export = ui.add_enabled(!o.all, egui::Button::new("Export...")).clicked();
                export_all = ui.add_enabled(o.all, egui::Button::new("Export All...")).clicked();
                copy = ui.button("Copy to Clipboard").clicked();
                close = ui.button("Close").clicked();
            });
            if let Some(r) = &self.export_image_result {
                ui.add_space(4.0);
                ui.label(r);
            }
            close |= ui.input(|i| i.key_pressed(egui::Key::Escape));
        });
        if o != before {
            self.set_image_options(&o);
        }
        if close {
            self.export_image_open = false;
            return;
        }
        let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
        if export {
            let Some(d) = self.diagram else {
                self.export_image_result = Some("No diagram to export.".into());
                return;
            };
            self.select(vec![]);
            let f = o.format;
            let Some(p) = rfd::FileDialog::new().set_title("Export Image").set_file_name(self.export_name(d, f)).add_filter(f.label(), &[f.ext()]).save_file() else { return };
            self.export_image_result = Some(match render_with(self, d, &ms, &o).and_then(|b| std::fs::write(&p, b).map_err(|e| e.to_string())) {
                Ok(()) => format!("Exported to {}", p.display()),
                Err(e) => format!("Failed to export the diagram: {e}"),
            });
        }
        if export_all {
            if self.repo.all_of("Diagram").is_empty() {
                self.export_image_result = Some("No diagram to export.".into());
                return;
            }
            let Some(folder) = rfd::FileDialog::new().set_title("Select a folder where all diagrams to be exported").pick_folder() else { return };
            self.select(vec![]);
            self.export_image_result = Some(match self.export_all_with(&folder, &ms, &o) {
                Ok(n) => format!("{n} diagrams exported to {}", folder.join(o.format.ext()).display()),
                Err(e) => format!("Failed to export the diagrams: {e}"),
            });
        }
        if copy {
            self.copy_image_with(&ms, &o);
        }
    }

    /// Copy to Clipboard: the current diagram as a picture with the dialog's options (always a bitmap;
    /// an SVG format is copied as PNG).
    pub fn copy_image_with(&mut self, ms: &dyn Measure, o: &ImageOptions) {
        let Some(d) = self.diagram else {
            self.export_image_result = Some("No diagram to copy.".into());
            return;
        };
        let png = render_with(self, d, ms, &ImageOptions { format: Format::Png, fill: o.filled(), ..*o });
        let before = self.message.clone();
        self.copy_diagram_image(png);
        self.export_image_result = Some(if self.message != before { "Failed to copy the diagram.".into() } else { "Copied to the clipboard.".into() });
    }

    /// The default file name of a diagram picture: the diagram's name, or "diagram".
    pub fn export_name(&self, d: Eid, f: Format) -> String {
        let n = self.repo.name(d);
        format!("{}.{}", filenamify(if n.is_empty() { "diagram" } else { &n }), f.ext())
    }

    /// Write every diagram of the project into `<folder>/<png|jpg|svg>/` as
    /// `filenamify(path)_<index>.<ext>`, where the path joins the owners' names with "::"
    /// (default-commands.js:452-535). Returns how many were written.
    pub fn export_all(&mut self, folder: &std::path::Path, ms: &dyn Measure, f: Format, scale: f32) -> Result<usize, String> {
        self.export_all_by(folder, f, |a, d| render(a, d, ms, f, scale))
    }

    fn export_all_by(&mut self, folder: &std::path::Path, f: Format, mut picture: impl FnMut(&mut App, Eid) -> Result<Vec<u8>, String>) -> Result<usize, String> {
        let dir = folder.join(f.ext());
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let diagrams = self.repo.all_of("Diagram");
        let mut n = 0;
        for (i, d) in diagrams.into_iter().enumerate() {
            let path: Vec<String> = self.repo.path(d).into_iter().skip(1).map(|e| self.repo.name(e)).collect();
            let file = dir.join(format!("{}_{i}.{}", filenamify(&path.join("::")), f.ext()));
            match picture(self, d) {
                Ok(bytes) => {
                    std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
                    n += 1;
                }
                // an empty diagram has no picture; the others are still written
                Err(_) => continue,
            }
        }
        Ok(n)
    }
}
