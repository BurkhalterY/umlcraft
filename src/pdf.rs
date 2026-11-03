//! File > Print to PDF: one diagram per page, fitted to the page, with its name above it
//! (print-dialog.js, default-commands.js:541-585, diagram-export.js:318-362).
//!
//! UMLCraft writes the PDF itself: each diagram is drawn as a picture (JPEG, at twice the printed size)
//! rather than as vector shapes, and the name is set in the standard Helvetica font.

use crate::app::App;
use crate::model::Eid;
use crate::views::Measure;
use std::fmt::Write as _;

/// Page sizes in points (portrait), as the print dialog lists them (PDFKit's sizes).
pub const PAGE_SIZES: [(&str, f64, f64); 50] = [
    ("4A0", 4767.87, 6740.79), ("2A0", 3370.39, 4767.87), ("A0", 2383.94, 3370.39), ("A1", 1683.78, 2383.94), ("A2", 1190.55, 1683.78),
    ("A3", 841.89, 1190.55), ("A4", 595.28, 841.89), ("A5", 419.53, 595.28), ("A6", 297.64, 419.53), ("A7", 209.76, 297.64),
    ("A8", 147.40, 209.76), ("A9", 104.88, 147.40), ("A10", 73.70, 104.88), ("B0", 2834.65, 4008.19), ("B1", 2004.09, 2834.65),
    ("B2", 1417.32, 2004.09), ("B3", 1000.63, 1417.32), ("B4", 708.66, 1000.63), ("B5", 498.90, 708.66), ("B6", 354.33, 498.90),
    ("B7", 249.45, 354.33), ("B8", 175.75, 249.45), ("B9", 124.72, 175.75), ("B10", 87.87, 124.72), ("C0", 2599.37, 3676.54),
    ("C1", 1836.85, 2599.37), ("C2", 1298.27, 1836.85), ("C3", 918.43, 1298.27), ("C4", 649.13, 918.43), ("C5", 459.21, 649.13),
    ("C6", 323.15, 459.21), ("C7", 229.61, 323.15), ("C8", 161.57, 229.61), ("C9", 113.39, 161.57), ("C10", 79.37, 113.39),
    ("RA0", 2437.80, 3458.27), ("RA1", 1729.13, 2437.80), ("RA2", 1218.90, 1729.13), ("RA3", 864.57, 1218.90), ("RA4", 609.45, 864.57),
    ("SRA0", 2551.18, 3628.35), ("SRA1", 1814.17, 2551.18), ("SRA2", 1275.59, 1814.17), ("SRA3", 907.09, 1275.59), ("SRA4", 637.80, 907.09),
    ("Executive", 521.86, 756.00), ("Folio", 612.00, 936.00), ("Legal", 612.00, 1008.00), ("Letter", 612.00, 792.00), ("Tabloid", 792.00, 1224.00),
];

pub const MARGIN: f64 = 30.0;

/// The Print to PDF options (print.options; defaults all, landscape, A4, show name).
#[derive(Clone, Debug, PartialEq)]
pub struct PrintOptions {
    pub all: bool,
    pub landscape: bool,
    pub size: String,
    pub show_name: bool,
}

impl Default for PrintOptions {
    fn default() -> Self {
        PrintOptions { all: true, landscape: true, size: "A4".into(), show_name: true }
    }
}

impl PrintOptions {
    pub fn page(&self) -> (f64, f64) {
        let (_, w, h) = PAGE_SIZES.iter().find(|p| p.0.eq_ignore_ascii_case(&self.size)).copied().unwrap_or(("A4", 595.28, 841.89));
        if self.landscape { (h, w) } else { (w, h) }
    }
}

/// One page: the picture (JPEG bytes, pixel size), where it goes on the page, and the caption.
struct Page {
    jpeg: Vec<u8>,
    px: (u32, u32),
    at: (f64, f64, f64, f64),
    caption: Option<String>,
}

fn pdf_text(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '(' | ')' | '\\' => format!("\\{c}"),
            c if (c as u32) < 128 && !c.is_control() => c.to_string(),
            // Latin-1 letters as octal escapes, so the stream stays one byte per character
            c if (c as u32) >= 160 && (c as u32) < 256 => format!("\\{:03o}", c as u32),
            _ => "?".into(),
        })
        .collect()
}

/// Assemble the PDF file.
fn write_pdf(pages: &[Page], page: (f64, f64)) -> Vec<u8> {
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
    let mut offsets: Vec<usize> = vec![];
    let obj = |out: &mut Vec<u8>, offsets: &mut Vec<usize>, body: &[u8]| {
        offsets.push(out.len());
        let n = offsets.len();
        out.extend_from_slice(format!("{n} 0 obj\n").as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    // 1 catalog, 2 pages, 3 font, then per page: page, content, image
    let n = pages.len();
    let kids: String = (0..n).map(|i| format!("{} 0 R ", 4 + i * 3)).collect();
    obj(&mut out, &mut offsets, b"<< /Type /Catalog /Pages 2 0 R >>");
    obj(&mut out, &mut offsets, format!("<< /Type /Pages /Kids [{kids}] /Count {n} >>").as_bytes());
    obj(&mut out, &mut offsets, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>");
    for (i, p) in pages.iter().enumerate() {
        let (_page_id, content_id, image_id) = (4 + i * 3, 5 + i * 3, 6 + i * 3);
        obj(
            &mut out,
            &mut offsets,
            format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /Font << /F1 3 0 R >> /XObject << /Im{i} {image_id} 0 R >> >> /Contents {content_id} 0 R >>", page.0, page.1).as_bytes(),
        );
        let mut c = String::new();
        let (x, y, w, h) = p.at;
        // PDF's origin is the bottom-left corner
        let _ = write!(c, "q {w:.3} 0 0 {h:.3} {x:.3} {:.3} cm /Im{i} Do Q\n", page.1 - y - h);
        if let Some(cap) = &p.caption {
            let _ = write!(c, "BT /F1 10 Tf {MARGIN:.2} {:.2} Td ({}) Tj ET\n", page.1 - MARGIN + 8.0, pdf_text(cap));
        }
        obj(&mut out, &mut offsets, format!("<< /Length {} >>\nstream\n{c}endstream", c.len()).as_bytes());
        let mut img = format!("<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n", p.px.0, p.px.1, p.jpeg.len()).into_bytes();
        img.extend_from_slice(&p.jpeg);
        img.extend_from_slice(b"\nendstream");
        obj(&mut out, &mut offsets, &img);
    }
    let xref = out.len();
    let mut tail = format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
    for o in &offsets {
        let _ = writeln!(tail, "{o:010} 00000 n ");
    }
    let _ = write!(tail, "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1);
    out.extend_from_slice(tail.as_bytes());
    out
}

impl App {
    /// The diagrams to print: every diagram in project order, or the current one.
    pub fn print_diagrams(&self, all: bool) -> Vec<Eid> {
        if all {
            let mut out = vec![];
            fn walk(a: &App, e: Eid, out: &mut Vec<Eid>) {
                if a.is_diagram(e) {
                    out.push(e);
                }
                for (_, c) in a.repo.children(e) {
                    if !a.is_view(c) {
                        walk(a, c, out);
                    }
                }
            }
            walk(self, self.repo.root, &mut out);
            out
        } else {
            self.diagram.into_iter().collect()
        }
    }

    /// The PDF bytes for these options. Each diagram is scaled by min(page width / box right, page height /
    /// box bottom) inside 30 pt margins, never enlarged; its path name goes above it when asked.
    pub fn print_pdf(&mut self, o: &PrintOptions, ms: &dyn Measure) -> Result<Vec<u8>, String> {
        let diagrams = self.print_diagrams(o.all);
        self.pdf_of(&diagrams, o, ms)
    }

    /// The PDF of these diagrams, one per page.
    pub fn pdf_of(&mut self, diagrams: &[Eid], o: &PrintOptions, ms: &dyn Measure) -> Result<Vec<u8>, String> {
        let diagrams = diagrams.to_vec();
        if diagrams.is_empty() {
            return Err("No current diagram.".into());
        }
        let page = o.page();
        let (cw, ch) = (page.0 - 2.0 * MARGIN, page.1 - 2.0 * MARGIN);
        let mut pages = vec![];
        for d in diagrams {
            let info = crate::views::layout_diagram(&mut self.repo, ms, d);
            let Some(b) = crate::render::bounds(&self.repo, d) else { continue };
            let scale = (cw / (b.x + b.w)).min(ch / (b.y + b.h)).min(1.0);
            // a picture at twice the printed size, flattened on white
            let png = crate::render::to_png(&self.repo, d, &info, ms, (2.0 * scale) as f32)?;
            let img = image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgba8();
            let mut rgb = image::RgbImage::new(img.width(), img.height());
            for (x, y, p) in img.enumerate_pixels() {
                let a = p.0[3] as u32;
                let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
                rgb.put_pixel(x, y, image::Rgb([mix(p.0[0]), mix(p.0[1]), mix(p.0[2])]));
            }
            let mut jpeg = vec![];
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg, 92).encode_image(&rgb).map_err(|e| e.to_string())?;
            // the PNG starts 10 px before the bounds (diagram-export.js:106-128)
            let (x, y) = (MARGIN + (b.x - 10.0).max(0.0) * scale, MARGIN + (b.y - 10.0).max(0.0) * scale);
            let (w, h) = (img.width() as f64 / 2.0, img.height() as f64 / 2.0);
            let caption = o.show_name.then(|| self.path_name(d));
            pages.push(Page { jpeg, px: (img.width(), img.height()), at: (x, y, w, h), caption });
        }
        if pages.is_empty() {
            return Err("No current diagram.".into());
        }
        Ok(write_pdf(&pages, page))
    }
}

impl App {
    pub fn print_options(&self) -> PrintOptions {
        let v = self.prefs.get("print.options");
        let d = PrintOptions::default();
        let get = |k: &str| v.and_then(|v| v.get(k));
        PrintOptions {
            all: get("range").and_then(|x| x.as_str()).map(|x| x != "current").unwrap_or(d.all),
            landscape: get("layout").and_then(|x| x.as_str()).map(|x| x != "portrait").unwrap_or(d.landscape),
            size: get("size").and_then(|x| x.as_str()).map(String::from).unwrap_or(d.size),
            show_name: get("showName").and_then(|x| x.as_bool()).unwrap_or(d.show_name),
        }
    }

    /// Each change of the print options is kept straight away (print-dialog.js:33-76).
    pub fn set_print_options(&mut self, o: &PrintOptions) {
        let v = serde_json::json!({
            "range": if o.all { "all" } else { "current" },
            "layout": if o.landscape { "landscape" } else { "portrait" },
            "size": o.size,
            "showName": o.show_name,
        });
        self.prefs.set("print.options", v);
        self.prefs.flush();
    }

    /// The Print to PDF dialog: Print Range, Page Layout, Page Size, Show Diagram Name; Save asks for the
    /// file, `filenamify(project name).pdf` by default (default-commands.js:541-585).
    pub fn print_dialog_ui(&mut self, ctx: &egui::Context) {
        if !self.print_open {
            return;
        }
        let mut o = self.print_options();
        let before = o.clone();
        let (mut save, mut cancel) = (false, false);
        egui::Window::new("Print to PDF").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
            egui::Grid::new("print-grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Print Range");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut o.all, true, "All Diagrams");
                    ui.radio_value(&mut o.all, false, "Current Diagram");
                });
                ui.end_row();
                ui.label("Page Layout");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut o.landscape, true, "Landscape");
                    ui.radio_value(&mut o.landscape, false, "Portrait");
                });
                ui.end_row();
                ui.label("Page Size");
                egui::ComboBox::from_id_salt("print-size").selected_text(o.size.clone()).show_ui(ui, |ui| {
                    for (n, ..) in PAGE_SIZES {
                        ui.selectable_value(&mut o.size, n.to_string(), n);
                    }
                });
                ui.end_row();
                ui.label("");
                ui.checkbox(&mut o.show_name, "Show Diagram Name");
                ui.end_row();
            });
            ui.horizontal(|ui| {
                save = ui.button("Save").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        if o != before {
            self.set_print_options(&o);
        }
        if cancel {
            self.print_open = false;
        }
        if save {
            self.print_open = false;
            let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
            let name = format!("{}.pdf", crate::export::filenamify(&self.repo.name(self.repo.root)));
            let Some(mut p) = rfd::FileDialog::new().set_title("Print to PDF").set_file_name(name).add_filter("PDF", &["pdf"]).save_file() else { return };
            if p.extension().map(|e| e != "pdf").unwrap_or(true) {
                p.set_extension("pdf");
            }
            match self.print_pdf(&o, &ms).and_then(|b| std::fs::write(&p, b).map_err(|e| e.to_string())) {
                Ok(()) => {}
                Err(e) if e == "No current diagram." => self.toast(&e),
                Err(e) => self.message = Some(("Failed to print the diagrams.".into(), e)),
            }
        }
    }
}
