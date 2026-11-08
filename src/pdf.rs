//! File > Print to PDF: one diagram per page, fitted to the page, with its name above it
//! (print-dialog.js, default-commands.js:541-585, diagram-export.js:318-362).
//!
//! UMLCraft writes the PDF itself, with the drawing code of the screen and the SVG export
//! (render.rs): lines, shapes and text are vector graphics, the text is set in the embedded diagram
//! fonts (pdf_font.rs), and pictures are images (pdf-graphics.js:216-1240).

use crate::app::App;
use crate::model::Eid;
use crate::pdf_font::Fonts;
use crate::render::{Canvas, Rgba, DASH};
use crate::views::{Measure, P, R};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;

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

/// A Flate-compressed (zlib) copy of a stream.
fn deflate(data: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    let _ = e.write_all(data);
    e.finish().unwrap_or_default()
}

/// The objects of the file, numbered from 1, written in order with the cross-reference table.
#[derive(Default)]
struct Doc {
    objs: Vec<Vec<u8>>,
}

impl Doc {
    fn reserve(&mut self) -> usize {
        self.objs.push(vec![]);
        self.objs.len()
    }
    fn set(&mut self, id: usize, body: impl Into<Vec<u8>>) {
        self.objs[id - 1] = body.into();
    }
    fn add(&mut self, body: impl Into<Vec<u8>>) -> usize {
        let id = self.reserve();
        self.set(id, body);
        id
    }
    /// A compressed stream object with these extra dictionary entries.
    fn stream(&mut self, dict: &str, data: &[u8]) -> usize {
        let z = deflate(data);
        let mut body = format!("<< {dict} /Filter /FlateDecode /Length {} >>\nstream\n", z.len()).into_bytes();
        body.extend_from_slice(&z);
        body.extend_from_slice(b"\nendstream");
        self.add(body)
    }
    fn finish(self, root: usize) -> Vec<u8> {
        let mut out: Vec<u8> = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
        let mut offsets = vec![];
        for (i, body) in self.objs.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(body);
            out.extend_from_slice(b"\nendobj\n");
        }
        let xref = out.len();
        let mut tail = format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1);
        for o in &offsets {
            let _ = writeln!(tail, "{o:010} 00000 n ");
        }
        let _ = write!(tail, "trailer\n<< /Size {} /Root {root} 0 R >>\nstartxref\n{xref}\n%%EOF\n", offsets.len() + 1);
        out.extend_from_slice(tail.as_bytes());
        out
    }
}

/// A picture as the PDF keeps it: a JPEG file as it is, anything else as RGB samples with an
/// alpha mask when it has transparent pixels.
enum Pixels {
    Jpeg { bytes: Vec<u8>, w: u32, h: u32, gray: bool },
    Raw { w: u32, h: u32, rgb: Vec<u8>, alpha: Option<Vec<u8>> },
}

/// The largest side, in pixels, an SVG picture is drawn at for the PDF.
const SVG_MAX_PX: f64 = 4096.0;

/// The pixels of a picture given as a data URL. An SVG picture is drawn at four times the size it
/// takes in the diagram (the original converts it into vector shapes with SVGtoPDF instead).
fn pixels(url: &str, b: R) -> Option<Pixels> {
    let (mime, bytes) = crate::picture::split_data_url(url)?;
    let rgba = if mime == "image/svg+xml" {
        use resvg::{tiny_skia, usvg};
        let tree = usvg::Tree::from_data(&bytes, &usvg::Options::default()).ok()?;
        let s = tree.size();
        let (sw, sh) = (s.width() as f64, s.height() as f64);
        let k = (4.0 * b.w.max(1.0) / sw).max(4.0 * b.h.max(1.0) / sh).max(1.0).min(SVG_MAX_PX / sw.max(sh));
        let (w, h) = ((sw * k).ceil().max(1.0) as u32, (sh * k).ceil().max(1.0) as u32);
        let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
        resvg::render(&tree, tiny_skia::Transform::from_scale(k as f32, k as f32), &mut pixmap.as_mut());
        let mut img = image::RgbaImage::new(w, h);
        for (p, c) in img.pixels_mut().zip(pixmap.pixels()) {
            let c = c.demultiply();
            *p = image::Rgba([c.red(), c.green(), c.blue(), c.alpha()]);
        }
        img
    } else {
        let img = image::load_from_memory(&bytes).ok()?;
        if bytes.starts_with(&[0xff, 0xd8]) {
            match img.color() {
                image::ColorType::Rgb8 => return Some(Pixels::Jpeg { w: img.width(), h: img.height(), bytes, gray: false }),
                image::ColorType::L8 => return Some(Pixels::Jpeg { w: img.width(), h: img.height(), bytes, gray: true }),
                _ => {}
            }
        }
        img.to_rgba8()
    };
    let (w, h) = rgba.dimensions();
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    let mut alpha = Vec::with_capacity((w * h) as usize);
    for p in rgba.pixels() {
        rgb.extend_from_slice(&p.0[..3]);
        alpha.push(p.0[3]);
    }
    let alpha = alpha.iter().any(|a| *a < 255).then_some(alpha);
    Some(Pixels::Raw { w, h, rgb, alpha })
}

/// What the pages of one document share: its fonts, pictures and opacities.
struct Shared {
    fonts: Fonts,
    images: Vec<Pixels>,
    image_of: HashMap<String, Option<usize>>,
    alphas: Vec<(u8, u8)>,
}

/// A number as the content stream writes it: at most three decimals, no trailing zeros.
fn num(x: f64) -> String {
    let s = format!("{:.3}", if x.abs() < 0.0005 { 0.0 } else { x });
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn rgb(c: Rgba) -> String {
    format!("{} {} {}", num(c.0 as f64 / 255.0), num(c.1 as f64 / 255.0), num(c.2 as f64 / 255.0))
}

/// One page's drawing: a content stream in diagram coordinates (y going down; the page's
/// transformation turns them into PDF space).
struct PdfCanvas<'a> {
    out: String,
    measure: &'a dyn Measure,
    sh: &'a mut Shared,
}

/// Text laid out in the PDF fonts: runs of glyphs, one per face, and the width.
struct Line {
    runs: Vec<(usize, Vec<u16>)>,
    width: f64,
    /// half the sum of the first face's ascender and descender, to centre the text on a line
    middle: f64,
    /// the face's underline: how far its top is above the baseline (negative: below), its thickness
    underline: (f64, f64),
}

impl PdfCanvas<'_> {
    /// Opens a graphics state with these fill and stroke opacities, when one is not 1.
    fn opacity(&mut self, fill: Option<Rgba>, stroke: Option<Rgba>) -> bool {
        let (fa, sa) = (fill.map(|c| c.3).unwrap_or(255), stroke.map(|c| c.3).unwrap_or(255));
        if fa == 255 && sa == 255 {
            return false;
        }
        let i = self.sh.alphas.iter().position(|x| *x == (fa, sa)).unwrap_or_else(|| {
            self.sh.alphas.push((fa, sa));
            self.sh.alphas.len() - 1
        });
        let _ = writeln!(self.out, "q /GS{i} gs");
        true
    }

    /// Paints a path: filled, stroked or both.
    fn paint(&mut self, fill: Option<Rgba>, stroke: Option<Rgba>, path: &str) {
        if fill.is_none() && stroke.is_none() {
            return;
        }
        let q = self.opacity(fill, stroke);
        if let Some(f) = fill {
            let _ = write!(self.out, "{} rg ", rgb(f));
        }
        if let Some(s) = stroke {
            let _ = write!(self.out, "{} RG ", rgb(s));
        }
        self.out += path;
        self.out += match (fill, stroke) {
            (Some(_), Some(_)) => " B\n",
            (Some(_), None) => " f\n",
            _ => " S\n",
        };
        if q {
            self.out += "Q\n";
        }
    }

    fn line(&mut self, t: &str, style: u8, size: f64) -> Line {
        let mut runs: Vec<(usize, Vec<u16>)> = vec![];
        let mut width = 0.0;
        for c in t.chars().filter(|c| !c.is_control()) {
            let Some((fi, g)) = self.sh.fonts.glyph(style, c) else { continue };
            let face = &mut self.sh.fonts.faces[fi];
            face.used.entry(g).or_insert_with(|| c.to_string());
            width += face.advance(g) * size / face.upm;
            match runs.last_mut() {
                Some((f, gs)) if *f == fi => gs.push(g),
                _ => runs.push((fi, vec![g])),
            }
        }
        let (middle, underline) = match self.sh.fonts.primary(style).map(|i| &self.sh.fonts.faces[i]) {
            Some(f) => {
                let k = size / f.upm;
                ((f.ascender + f.descender) / 2.0 * k, (f.underline.0 * k, f.underline.1 * k))
            }
            None => (0.35 * size, (-0.1 * size, 0.05 * size)),
        };
        Line { runs, width, middle, underline }
    }

    /// Writes laid-out text from this baseline point; `matrix` is where text space's x and y axes go.
    fn show(&mut self, l: &Line, size: f64, color: Rgba, at: P, matrix: [f64; 4]) {
        if l.runs.is_empty() {
            return;
        }
        let q = self.opacity(Some(color), None);
        let [a, b, c, d] = matrix;
        let _ = write!(self.out, "BT {} rg {} {} {} {} {} {} Tm", rgb(color), num(a), num(b), num(c), num(d), num(at.x), num(at.y));
        for (fi, gs) in &l.runs {
            let hex: String = gs.iter().map(|g| format!("{g:04X}")).collect();
            let _ = write!(self.out, " /F{fi} {} Tf <{hex}> Tj", num(size));
        }
        self.out += " ET\n";
        if q {
            self.out += "Q\n";
        }
    }

    /// Level black text with its baseline starting at this point (the page caption).
    fn caption(&mut self, t: &str, x: f64, baseline: f64, size: f64) {
        let l = self.line(t, 0, size);
        self.show(&l, size, crate::render::BLACK, P { x, y: baseline }, [1.0, 0.0, 0.0, -1.0]);
    }
}

impl Measure for PdfCanvas<'_> {
    fn width(&self, text: &str, style: u8) -> f64 {
        self.measure.width(text, style)
    }
}

fn path_of(pts: &[P]) -> String {
    let mut path = String::new();
    for (i, p) in pts.iter().enumerate() {
        let _ = write!(path, "{}{} {} {}", if i > 0 { " " } else { "" }, num(p.x), num(p.y), if i == 0 { "m" } else { "l" });
    }
    path
}

impl Canvas for PdfCanvas<'_> {
    fn polyline(&mut self, pts: &[P], color: Rgba, dashed: bool) {
        if pts.len() < 2 {
            return;
        }
        if dashed {
            let _ = write!(self.out, "[{}] 0 d ", num(DASH));
        }
        self.paint(None, Some(color), &path_of(pts));
        if dashed {
            self.out += "[] 0 d\n";
        }
    }

    fn polygon(&mut self, pts: &[P], fill: Option<Rgba>, stroke: Option<Rgba>) {
        if pts.len() < 2 {
            return;
        }
        self.paint(fill, stroke, &(path_of(pts) + " h"));
    }

    fn rect(&mut self, r: R, fill: Option<Rgba>, stroke: Option<Rgba>) {
        self.paint(fill, stroke, &format!("{} {} {} {} re", num(r.x), num(r.y), num(r.w), num(r.h)));
    }

    /// Four Bézier curves, as PDFKit draws an ellipse (pdf-graphics.js:366-423).
    fn ellipse(&mut self, c: P, rx: f64, ry: f64, fill: Option<Rgba>, stroke: Option<Rgba>) {
        const K: f64 = 0.552_284_75;
        let (kx, ky) = (K * rx, K * ry);
        let p = |x: f64, y: f64| format!("{} {}", num(x), num(y));
        let path = format!(
            "{} m {} {} {} c {} {} {} c {} {} {} c {} {} {} c h",
            p(c.x + rx, c.y),
            p(c.x + rx, c.y + ky),
            p(c.x + kx, c.y + ry),
            p(c.x, c.y + ry),
            p(c.x - kx, c.y + ry),
            p(c.x - rx, c.y + ky),
            p(c.x - rx, c.y),
            p(c.x - rx, c.y - ky),
            p(c.x - kx, c.y - ry),
            p(c.x, c.y - ry),
            p(c.x + kx, c.y - ry),
            p(c.x + rx, c.y - ky),
            p(c.x + rx, c.y),
        );
        self.paint(fill, stroke, &path);
    }

    fn text(&mut self, t: &str, b: R, align: i64, style: u8, size: f64, color: Rgba, underline: bool) {
        let l = self.line(t, style, size);
        let x = match align {
            0 => b.x,
            1 => b.x + b.w - l.width,
            _ => b.x + (b.w - l.width) / 2.0,
        };
        let y = b.y + b.h / 2.0 + l.middle;
        self.show(&l, size, color, P { x, y }, [1.0, 0.0, 0.0, -1.0]);
        if underline && l.width > 0.0 {
            let r = R { x, y: y - l.underline.0, w: l.width, h: l.underline.1.max(0.5) };
            self.rect(r, Some(color), None);
        }
    }

    /// Text turned a quarter left, reading upwards (pdf-graphics.js textOut2 with a rotation).
    fn text_up(&mut self, t: &str, b: R, style: u8, size: f64, color: Rgba) {
        let l = self.line(t, style, size);
        let (cx, cy) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
        self.show(&l, size, color, P { x: cx + l.middle, y: cy + l.width / 2.0 }, [0.0, -1.0, -1.0, 0.0]);
    }

    fn image(&mut self, b: R, data_url: &str) {
        if b.w <= 0.0 || b.h <= 0.0 {
            return;
        }
        let i = match self.sh.image_of.get(data_url) {
            Some(i) => *i,
            None => {
                let i = pixels(data_url, b).map(|p| {
                    self.sh.images.push(p);
                    self.sh.images.len() - 1
                });
                self.sh.image_of.insert(data_url.to_string(), i);
                i
            }
        };
        if let Some(i) = i {
            let _ = writeln!(self.out, "q {} 0 0 {} {} {} cm /Im{i} Do Q", num(b.w), num(-b.h), num(b.x), num(b.y + b.h));
        }
    }
}

/// The subset prefix of the n-th font's name: six capital letters.
fn subset_tag(n: usize) -> String {
    let mut x = n + 0x5EED;
    (0..6)
        .map(|_| {
            let c = (b'A' + (x % 26) as u8) as char;
            x /= 26;
            c
        })
        .collect()
}

/// Writes the fonts, pictures and opacities the pages used, and the resource dictionary they share.
fn write_resources(doc: &mut Doc, sh: &Shared, res: usize) {
    let mut fonts = String::new();
    for (i, face) in sh.fonts.faces.iter().enumerate() {
        if face.used.is_empty() {
            continue;
        }
        let Some(e) = face.embed(&subset_tag(i)) else { continue };
        let file = doc.stream(&format!("/Length1 {}", e.file.len()), &e.file);
        let descriptor = doc.add(format!("<< {} /FontFile2 {file} 0 R >>", e.descriptor));
        let cid = doc.add(format!(
            "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {descriptor} 0 R /DW 1000 /W {} /CIDToGIDMap /Identity >>",
            e.base_font, e.widths
        ));
        let to_unicode = doc.stream("", e.to_unicode.as_bytes());
        let font = doc.add(format!(
            "<< /Type /Font /Subtype /Type0 /BaseFont /{} /Encoding /Identity-H /DescendantFonts [{cid} 0 R] /ToUnicode {to_unicode} 0 R >>",
            e.base_font
        ));
        let _ = write!(fonts, "/F{i} {font} 0 R ");
    }
    let mut images = String::new();
    for (i, p) in sh.images.iter().enumerate() {
        let id = match p {
            Pixels::Jpeg { bytes, w, h, gray } => {
                let cs = if *gray { "/DeviceGray" } else { "/DeviceRGB" };
                let mut body = format!(
                    "<< /Type /XObject /Subtype /Image /Width {w} /Height {h} /ColorSpace {cs} /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
                    bytes.len()
                )
                .into_bytes();
                body.extend_from_slice(bytes);
                body.extend_from_slice(b"\nendstream");
                doc.add(body)
            }
            Pixels::Raw { w, h, rgb, alpha } => {
                let image = format!("/Type /XObject /Subtype /Image /Width {w} /Height {h} /BitsPerComponent 8");
                let smask = alpha.as_ref().map(|a| format!(" /SMask {} 0 R", doc.stream(&format!("{image} /ColorSpace /DeviceGray"), a))).unwrap_or_default();
                doc.stream(&format!("{image} /ColorSpace /DeviceRGB{smask}"), rgb)
            }
        };
        let _ = write!(images, "/Im{i} {id} 0 R ");
    }
    let mut states = String::new();
    for (i, (fa, sa)) in sh.alphas.iter().enumerate() {
        let id = doc.add(format!("<< /Type /ExtGState /ca {} /CA {} >>", num(*fa as f64 / 255.0), num(*sa as f64 / 255.0)));
        let _ = write!(states, "/GS{i} {id} 0 R ");
    }
    doc.set(res, format!("<< /ProcSet [/PDF /Text /ImageB /ImageC] /Font << {fonts}>> /XObject << {images}>> /ExtGState << {states}>> >>"));
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

    /// The PDF of these diagrams, one per page. Each page's drawing is put 30 pt from the top left
    /// corner and scaled by min(width / box right, height / box bottom) of the room inside the
    /// margins, never enlarged; the path name goes 8 pt above that room, in the diagram font at
    /// 10 pt (the original sets it in Helvetica 10; diagram-export.js:318-362).
    pub fn pdf_of(&mut self, diagrams: &[Eid], o: &PrintOptions, ms: &dyn Measure) -> Result<Vec<u8>, String> {
        let diagrams = diagrams.to_vec();
        if diagrams.is_empty() {
            return Err("No current diagram.".into());
        }
        let page = o.page();
        let (cw, ch) = (page.0 - 2.0 * MARGIN, page.1 - 2.0 * MARGIN);
        let mut doc = Doc::default();
        let (catalog, pages, res) = (doc.reserve(), doc.reserve(), doc.reserve());
        let mut sh = Shared { fonts: Fonts::new(), images: vec![], image_of: HashMap::new(), alphas: vec![] };
        let mut kids = vec![];
        for d in diagrams {
            let info = crate::views::layout_diagram(&mut self.repo, ms, d);
            // an empty diagram prints an empty page, at scale 1
            let scale = crate::render::bounds(&self.repo, d)
                .map(|b| (cw / (b.x + b.w)).min(ch / (b.y + b.h)))
                .filter(|s| s.is_finite() && *s > 0.0)
                .unwrap_or(1.0)
                .min(1.0);
            let mut c = PdfCanvas { out: String::new(), measure: ms, sh: &mut sh };
            // PDF's origin is the bottom-left corner: turn the page so y goes down, as in the diagram
            let _ = writeln!(c.out, "1 0 0 -1 0 {} cm 1 w 0 J 0 j", num(page.1));
            let _ = writeln!(c.out, "q {} 0 0 {} {MARGIN} {MARGIN} cm", num(scale), num(scale));
            crate::render::draw_diagram(&mut c, &self.repo, d, &info);
            c.out += "Q\n";
            if o.show_name {
                let name = self.path_name(d);
                c.caption(&name, MARGIN, MARGIN - 8.0, 10.0);
            }
            let content = doc.stream("", c.out.as_bytes());
            kids.push(doc.add(format!(
                "<< /Type /Page /Parent {pages} 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources {res} 0 R /Contents {content} 0 R >>",
                page.0, page.1
            )));
        }
        write_resources(&mut doc, &sh, res);
        let list: String = kids.iter().map(|k| format!("{k} 0 R ")).collect();
        doc.set(pages, format!("<< /Type /Pages /Kids [{list}] /Count {} >>", kids.len()));
        doc.set(catalog, format!("<< /Type /Catalog /Pages {pages} 0 R >>"));
        Ok(doc.finish(catalog))
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
        crate::dialogs::dialog(ctx, "print", "Print to PDF", |ui| {
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
            // Enter saves and Esc cancels, unless a size list is open (dialog-manager.js:148-162)
            if !egui::Popup::is_any_open(ui.ctx()) {
                save |= ui.input(|i| i.key_pressed(egui::Key::Enter));
                cancel |= ui.input(|i| i.key_pressed(egui::Key::Escape));
            }
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
