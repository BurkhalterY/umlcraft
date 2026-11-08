//! Fonts for Print to PDF: the diagram faces and their fallbacks, found per character, and
//! embedded as subsets (Type0 fonts with Identity-H and a ToUnicode map, so any character prints
//! and the text can be selected and searched). The original registers every font file it knows
//! with PDFKit and picks a fallback font per character (diagram-export.js:321-324,
//! pdf-graphics.js:102-140, 990-1060).
//!
//! Only faces with TrueType outlines (a `glyf` table) are used: they are subset here by keeping
//! the glyph numbers and emptying the glyphs that are not printed.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// One font file (or one face of a collection) that the document may embed.
pub struct Face {
    data: Arc<Vec<u8>>,
    index: u32,
    pub upm: f64,
    pub ascender: f64,
    pub descender: f64,
    cap_height: f64,
    italic_angle: f64,
    bbox: [i16; 4],
    pub underline: (f64, f64),
    name: String,
    /// glyphs printed so far, with the text each one stands for
    pub used: BTreeMap<u16, String>,
}

impl Face {
    fn load(data: Arc<Vec<u8>>, index: u32) -> Option<Face> {
        let f = ttf_parser::Face::parse(&data, index).ok()?;
        f.tables().glyf?;
        let upm = f.units_per_em() as f64;
        let b = f.global_bounding_box();
        let name = f
            .names()
            .into_iter()
            .filter(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .find_map(|n| n.to_string())
            .unwrap_or_else(|| "Font".into());
        let name: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
        let ul = f.underline_metrics().map(|m| (m.position as f64, m.thickness as f64)).unwrap_or((-0.1 * upm, 0.05 * upm));
        Some(Face {
            upm,
            ascender: f.ascender() as f64,
            descender: f.descender() as f64,
            cap_height: f.capital_height().map(|c| c as f64).unwrap_or(0.7 * upm),
            italic_angle: f.italic_angle() as f64,
            bbox: [b.x_min, b.y_min, b.x_max, b.y_max],
            underline: ul,
            name: if name.is_empty() { "Font".into() } else { name },
            used: BTreeMap::new(),
            data,
            index,
        })
    }

    fn parsed(&self) -> Option<ttf_parser::Face<'_>> {
        ttf_parser::Face::parse(&self.data, self.index).ok()
    }

    /// The glyph of a character, if the face has one.
    fn glyph(&self, c: char) -> Option<u16> {
        self.parsed()?.glyph_index(c).map(|g| g.0).filter(|g| *g != 0)
    }

    /// The advance of a glyph in font units.
    pub fn advance(&self, g: u16) -> f64 {
        self.parsed().and_then(|f| f.glyph_hor_advance(ttf_parser::GlyphId(g))).unwrap_or(0) as f64
    }
}

/// The faces of one document: the four diagram styles, egui's fallback fonts (the ones the
/// screen falls back to), and system fonts found for characters none of those has.
pub struct Fonts {
    pub faces: Vec<Face>,
    /// the four styles of the current face (set_face), then of the Arial-compatible face
    styles: [Option<usize>; 4],
    arial: [Option<usize>; 4],
    /// the styles of each face used so far (lower case)
    by_face: HashMap<String, [Option<usize>; 4]>,
    fallbacks: Vec<usize>,
    system: Vec<usize>,
    files: HashMap<(String, u32), usize>,
    by_char: HashMap<(usize, char), (usize, u16)>,
    searched: HashMap<char, bool>,
}

impl Fonts {
    pub fn new() -> Fonts {
        let mut f = Fonts { faces: vec![], styles: [None; 4], arial: [None; 4], by_face: HashMap::new(), fallbacks: vec![], system: vec![], files: HashMap::new(), by_char: HashMap::new(), searched: HashMap::new() };
        for (i, (_, paths)) in crate::diagram_face_paths().into_iter().enumerate().take(4) {
            f.arial[i] = paths.iter().find_map(|p| f.file(p, 0));
        }
        f.styles = f.arial;
        let defs = egui::FontDefinitions::default();
        for name in defs.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default() {
            if let Some(fd) = defs.font_data.get(&name) {
                if let Some(face) = Face::load(Arc::new(fd.font.to_vec()), fd.index) {
                    f.faces.push(face);
                    f.fallbacks.push(f.faces.len() - 1);
                }
            }
        }
        f
    }

    fn file(&mut self, path: &str, index: u32) -> Option<usize> {
        if let Some(i) = self.files.get(&(path.to_string(), index)) {
            return Some(*i);
        }
        let data = std::fs::read(path).ok()?;
        let face = Face::load(Arc::new(data), index)?;
        self.faces.push(face);
        let i = self.faces.len() - 1;
        self.files.insert((path.to_string(), index), i);
        Some(i)
    }

    /// The face of a style (1 bold, 2 italic), or the regular one, or the first fallback.
    pub fn primary(&self, style: u8) -> Option<usize> {
        self.styles[(style & 3) as usize].or(self.styles[0]).or(self.arial[(style & 3) as usize]).or(self.arial[0]).or(self.fallbacks.first().copied())
    }

    /// Print the texts that follow in a view's font face: its files (fonts.rs) when they have
    /// TrueType outlines, else the Arial-compatible face (pdf-graphics.js:102-140).
    pub fn set_face(&mut self, face: &str) {
        if crate::fonts::is_default(face) {
            self.styles = self.arial;
            return;
        }
        let key = face.trim().to_lowercase();
        if let Some(s) = self.by_face.get(&key) {
            self.styles = *s;
            return;
        }
        let mut s = [None; 4];
        for (i, slot) in s.iter_mut().enumerate() {
            if let Some((p, index)) = crate::fonts::face_file(face, i as u8) {
                *slot = self.file(&p.display().to_string(), index);
            }
        }
        if s[0].is_none() {
            s = self.arial;
        }
        self.by_face.insert(key, s);
        self.styles = s;
    }

    /// The face and glyph that print a character: the style's face, then egui's fallbacks, then a
    /// system font that has it (asked from fontconfig), else the missing-glyph box of the face.
    pub fn glyph(&mut self, style: u8, c: char) -> Option<(usize, u16)> {
        let primary = self.primary(style)?;
        if let Some(hit) = self.by_char.get(&(primary, c)) {
            return Some(*hit);
        }
        // a face without the character falls back to the Arial-compatible face (font-manager.js:114-126)
        let mut order: Vec<usize> = vec![primary];
        order.extend(self.arial[(style & 3) as usize].filter(|a| *a != primary));
        order.extend(self.fallbacks.iter().copied());
        order.extend(self.system.iter().copied());
        let mut hit = order.iter().find_map(|i| self.faces[*i].glyph(c).map(|g| (*i, g)));
        if hit.is_none() && !self.searched.contains_key(&c) {
            self.searched.insert(c, true);
            for (path, index) in system_fonts_for(c) {
                if let Some(i) = self.file(&path, index) {
                    if let Some(g) = self.faces[i].glyph(c) {
                        if !self.system.contains(&i) {
                            self.system.push(i);
                        }
                        hit = Some((i, g));
                        break;
                    }
                }
            }
        }
        let hit = hit.unwrap_or((primary, 0));
        self.by_char.insert((primary, c), hit);
        Some(hit)
    }
}

/// Font files that may have a character, best first, as fontconfig lists them.
fn system_fonts_for(c: char) -> Vec<(String, u32)> {
    let pattern = format!(":charset={:x}", c as u32);
    let run = |args: &[&str]| -> Vec<(String, u32)> {
        let Ok(out) = std::process::Command::new(args[0]).args(&args[1..]).output() else { return vec![] };
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| {
                let (f, i) = l.rsplit_once('|')?;
                Some((f.to_string(), i.parse().ok()?))
            })
            .collect()
    };
    let mut out = run(&["fc-match", "-f", "%{file}|%{index}\n", &format!("sans-serif{pattern}")]);
    out.extend(run(&["fc-list", &pattern, "-f", "%{file}|%{index}\n"]));
    out.dedup();
    out
}

fn u16_at(d: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*d.get(i)?, *d.get(i + 1)?]))
}
fn u32_at(d: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_be_bytes([*d.get(i)?, *d.get(i + 1)?, *d.get(i + 2)?, *d.get(i + 3)?]))
}

/// A TrueType font file made of these tables (tag order, with checksums).
fn sfnt(mut tables: Vec<([u8; 4], Vec<u8>)>) -> Vec<u8> {
    tables.sort_by(|a, b| a.0.cmp(&b.0));
    let n = tables.len() as u16;
    let mut pow = 1u16;
    let mut sel = 0u16;
    while pow * 2 <= n {
        pow *= 2;
        sel += 1;
    }
    let mut out = vec![];
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(&(pow * 16).to_be_bytes());
    out.extend_from_slice(&sel.to_be_bytes());
    out.extend_from_slice(&(n * 16 - pow * 16).to_be_bytes());
    let checksum = |d: &[u8]| -> u32 { d.chunks(4).map(|c| { let mut b = [0u8; 4]; b[..c.len()].copy_from_slice(c); u32::from_be_bytes(b) }).fold(0u32, |a, x| a.wrapping_add(x)) };
    let mut offset = 12 + 16 * tables.len();
    let mut body = vec![];
    for (tag, data) in &tables {
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(data).to_be_bytes());
        out.extend_from_slice(&(offset as u32).to_be_bytes());
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        body.extend_from_slice(data);
        while body.len() % 4 != 0 {
            body.push(0);
        }
        offset = 12 + 16 * tables.len() + body.len();
    }
    out.extend_from_slice(&body);
    // head.checkSumAdjustment makes the whole file sum to 0xB1B0AFBA
    if let Some(pos) = tables.iter().position(|t| &t.0 == b"head") {
        let head_at = u32_at(&out, 12 + 16 * pos + 8).unwrap_or(0) as usize;
        let adj = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        out[head_at + 8..head_at + 12].copy_from_slice(&adj.to_be_bytes());
    }
    out
}

/// A copy of a TrueType face that keeps only these glyphs (and the parts of composite glyphs);
/// glyph numbers stay the same, so the PDF can name glyphs by their number.
pub fn subset(data: &[u8], index: u32, keep: &BTreeSet<u16>) -> Option<Vec<u8>> {
    let raw = ttf_parser::RawFace::parse(data, index).ok()?;
    let table = |tag: &[u8; 4]| raw.table(ttf_parser::Tag::from_bytes(tag));
    let (head, maxp, loca, glyf) = (table(b"head")?, table(b"maxp")?, table(b"loca")?, table(b"glyf")?);
    let long = u16_at(head, 50)? == 1;
    let n = u16_at(maxp, 4)? as usize;
    let at = |g: usize| -> Option<usize> { if long { u32_at(loca, 4 * g).map(|x| x as usize) } else { u16_at(loca, 2 * g).map(|x| 2 * x as usize) } };
    let mut keep: BTreeSet<u16> = keep.iter().copied().filter(|g| (*g as usize) < n).collect();
    keep.insert(0);
    let mut stack: Vec<u16> = keep.iter().copied().collect();
    while let Some(g) = stack.pop() {
        let (a, b) = (at(g as usize)?, at(g as usize + 1)?);
        let Some(d) = glyf.get(a..b) else { continue };
        if d.len() < 10 || (u16_at(d, 0)? as i16) >= 0 {
            continue;
        }
        // a composite glyph: each part names another glyph (flags in the 'glyf' spec)
        let mut p = 10;
        loop {
            let (Some(flags), Some(part)) = (u16_at(d, p), u16_at(d, p + 2)) else { break };
            if (part as usize) < n && keep.insert(part) {
                stack.push(part);
            }
            p += 4 + if flags & 1 != 0 { 4 } else { 2 };
            p += if flags & 8 != 0 { 2 } else if flags & 0x40 != 0 { 4 } else if flags & 0x80 != 0 { 8 } else { 0 };
            if flags & 0x20 == 0 {
                break;
            }
        }
    }
    let mut new_glyf = vec![];
    let mut new_loca = vec![];
    for g in 0..n {
        new_loca.extend_from_slice(&(new_glyf.len() as u32).to_be_bytes());
        if keep.contains(&(g as u16)) {
            if let Some(d) = at(g).zip(at(g + 1)).and_then(|(a, b)| glyf.get(a..b)) {
                new_glyf.extend_from_slice(d);
                while new_glyf.len() % 4 != 0 {
                    new_glyf.push(0);
                }
            }
        }
    }
    new_loca.extend_from_slice(&(new_glyf.len() as u32).to_be_bytes());
    let mut new_head = head.to_vec();
    new_head[8..12].copy_from_slice(&[0; 4]);
    new_head[50..52].copy_from_slice(&1u16.to_be_bytes());
    let mut tables = vec![(*b"head", new_head), (*b"loca", new_loca), (*b"glyf", new_glyf)];
    for tag in [b"hhea", b"maxp", b"hmtx", b"cvt ", b"fpgm", b"prep", b"OS/2"] {
        if let Some(t) = table(tag) {
            tables.push((*tag, t.to_vec()));
        }
    }
    Some(sfnt(tables))
}

/// UTF-16BE hex of a text, for the ToUnicode map.
fn utf16_hex(s: &str) -> String {
    s.encode_utf16().map(|u| format!("{u:04X}")).collect()
}

/// The parts of one embedded face: the font file, the glyph widths (in 1/1000 em) and the
/// ToUnicode map, and its descriptor numbers.
pub struct Embedded {
    pub base_font: String,
    pub file: Vec<u8>,
    pub widths: String,
    pub to_unicode: String,
    pub descriptor: String,
}

impl Face {
    /// What the PDF needs to embed this face with the glyphs used; `tag` is the six-letter subset
    /// prefix of its name.
    pub fn embed(&self, tag: &str) -> Option<Embedded> {
        let keep: BTreeSet<u16> = self.used.keys().copied().collect();
        let file = subset(&self.data, self.index, &keep)?;
        let k = 1000.0 / self.upm;
        let mut widths = String::from("[");
        for g in &keep {
            widths += &format!("{g} [{:.0}] ", self.advance(*g) * k);
        }
        widths.push(']');
        let entries: Vec<(u16, &String)> = self.used.iter().filter(|(_, t)| !t.is_empty()).map(|(g, t)| (*g, t)).collect();
        let mut cmap = String::from(
            "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
        );
        for chunk in entries.chunks(100) {
            cmap += &format!("{} beginbfchar\n", chunk.len());
            for (g, t) in chunk {
                cmap += &format!("<{g:04X}> <{}>\n", utf16_hex(t));
            }
            cmap += "endbfchar\n";
        }
        cmap += "endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n";
        let base_font = format!("{tag}+{}", self.name);
        let flags = 4 | if self.italic_angle != 0.0 { 64 } else { 0 };
        let descriptor = format!(
            "/Type /FontDescriptor /FontName /{base_font} /Flags {flags} /FontBBox [{:.0} {:.0} {:.0} {:.0}] /ItalicAngle {:.1} /Ascent {:.0} /Descent {:.0} /CapHeight {:.0} /StemV 80",
            self.bbox[0] as f64 * k,
            self.bbox[1] as f64 * k,
            self.bbox[2] as f64 * k,
            self.bbox[3] as f64 * k,
            self.italic_angle,
            self.ascender * k,
            self.descender * k,
            self.cap_height * k
        );
        Some(Embedded { base_font, file, widths, to_unicode: cmap, descriptor })
    }
}
