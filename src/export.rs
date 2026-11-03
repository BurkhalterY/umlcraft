//! File > Export Diagram As: PNG, JPEG and SVG of the current diagram, or of every diagram into a folder
//! (default-commands.js:360-535, diagram-export.js:106-298).

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

impl App {
    /// The default file name of a diagram picture: the diagram's name, or "diagram".
    pub fn export_name(&self, d: Eid, f: Format) -> String {
        let n = self.repo.name(d);
        format!("{}.{}", filenamify(if n.is_empty() { "diagram" } else { &n }), f.ext())
    }

    /// Write every diagram of the project into `<folder>/<png|jpg|svg>/` as
    /// `filenamify(path)_<index>.<ext>`, where the path joins the owners' names with "::"
    /// (default-commands.js:452-535). Returns how many were written.
    pub fn export_all(&mut self, folder: &std::path::Path, ms: &dyn Measure, f: Format, scale: f32) -> Result<usize, String> {
        let dir = folder.join(match f {
            Format::Png => "png",
            Format::Jpeg => "jpg",
            Format::Svg => "svg",
        });
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let diagrams = self.repo.all_of("Diagram");
        let mut n = 0;
        for (i, d) in diagrams.into_iter().enumerate() {
            let path: Vec<String> = self.repo.path(d).into_iter().skip(1).map(|e| self.repo.name(e)).collect();
            let file = dir.join(format!("{}_{i}.{}", filenamify(&path.join("::")), f.ext()));
            match render(self, d, ms, f, scale) {
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
