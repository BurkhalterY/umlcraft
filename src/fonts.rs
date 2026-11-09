//! Diagram font faces, as StarUML's font manager (src/engine/font-manager.js) gives them: the
//! faces it lists, user fonts from `<config>/fonts/<name>/font.json`, and the fonts that draw the
//! characters a face does not have.
//!
//! StarUML bundles its fonts; UMLCraft finds each face among the system's fonts (or a
//! metric-compatible stand-in, such as Liberation Sans for Arial) and among the user fonts. A
//! face is loaded the first time a view uses it; until then, and when it is not found, the
//! Arial-compatible face draws the text. The screen, the SVG and PNG pictures and the PDF use the
//! same files.

use egui::{FontData, FontDefinitions, FontFamily};
use resvg::usvg::fontdb;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// The faces StarUML lists in the Font dialog and the Style editor, in its order: the folders of
/// resources/fonts read in turn, each font.json's named fonts ("default" ones are not listed), the
/// PDFKit standard fonts last (font-manager.js:146-202, resources/fonts/*/font.json).
pub const FACES: [&str; 12] = ["Gloria Hallelujah", "Inter", "Lato", "Arial", "Times New Roman", "Courier New", "Roboto", "Source Code Pro", "Source Sans Pro", "Helvetica", "Times", "Courier"];

/// A user font: a `font.json` entry of a folder in `<config>/fonts` (font-manager.js:52-66, 146-161).
#[derive(Clone, Debug, PartialEq)]
pub struct UserFont {
    pub name: String,
    /// regular, bold, italic and bold italic (the style numbers 0-3 of a view's font)
    pub files: [PathBuf; 4],
    /// Unicode block numbers this font draws when it is a "default" font (unicode.js).
    pub ranges: Vec<u64>,
    /// a PDFKit standard font, which has no files
    pub predefined: bool,
}

/// The user fonts of a fonts folder: each sub-folder's `font.json`, an array of
/// `{name, regular, italic, bold, boldItalic, unicodeRanges?, predefined?}` with the files in that
/// sub-folder (font-manager.js:146-182). Folders and entries that do not parse are skipped.
pub fn read_user_fonts(dir: &Path) -> Vec<UserFont> {
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    let mut out = vec![];
    for d in dirs {
        let Some(list) = std::fs::read_to_string(d.join("font.json")).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) else { continue };
        for f in list.as_array().into_iter().flatten() {
            let s = |k: &str| f.get(k).and_then(|v| v.as_str()).map(str::to_string);
            let Some(name) = s("name") else { continue };
            let predefined = f.get("predefined").and_then(|v| v.as_bool()).unwrap_or(false);
            let Some(regular) = s("regular") else { continue };
            let file = |k: &str| d.join(s(k).unwrap_or_else(|| regular.clone()));
            let ranges = f.get("unicodeRanges").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_u64()).collect()).unwrap_or_default();
            out.push(UserFont { name, files: [file("regular"), file("bold"), file("italic"), file("boldItalic")], ranges, predefined });
        }
    }
    out
}

/// The user fonts folder: `fonts` next to the preferences file.
pub fn user_dir() -> Option<PathBuf> {
    crate::prefs::default_path().and_then(|p| p.parent().map(|d| d.join("fonts")))
}

/// The user fonts, read once.
pub fn user_fonts() -> &'static [UserFont] {
    static USER: OnceLock<Vec<UserFont>> = OnceLock::new();
    USER.get_or_init(|| if cfg!(test) { vec![] } else { user_dir().map(|d| read_user_fonts(&d)).unwrap_or_default() })
}

/// The files of the user fonts, for the PNG pictures.
pub fn user_font_files() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = user_fonts().iter().filter(|f| !f.predefined).flat_map(|f| f.files.iter().cloned()).collect();
    v.sort();
    v.dedup();
    v
}

/// The faces to offer in the Font dialog: StarUML's, then the user fonts (Object.keys of the
/// font manager's fonts, dialog-manager.js:538).
pub fn face_names() -> Vec<String> {
    let mut v: Vec<String> = FACES.iter().map(|s| s.to_string()).collect();
    for f in user_fonts() {
        if f.name != "default" && !v.iter().any(|x| x.eq_ignore_ascii_case(&f.name)) {
            v.push(f.name.clone());
        }
    }
    v
}

fn norm(face: &str) -> String {
    face.trim().trim_matches(|c| c == '\'' || c == '"').to_lowercase()
}

/// True for the face every diagram starts with, drawn by the Arial-compatible font.
pub fn is_default(face: &str) -> bool {
    matches!(norm(face).as_str(), "" | "arial" | "default")
}

/// The system families that stand in for a face, best first: the face itself, then metric-compatible
/// faces for the ones StarUML bundles as Liberation (resources/fonts/Liberation/font.json) and for
/// the PDFKit standard fonts.
fn stand_ins(face: &str) -> Vec<String> {
    let extra: &[&str] = match norm(face).as_str() {
        "arial" | "helvetica" => &["Liberation Sans", "Arimo", "Arial", "Helvetica", "Nimbus Sans", "TeX Gyre Heros"],
        "times new roman" | "times" => &["Liberation Serif", "Tinos", "Times New Roman", "Times", "Nimbus Roman", "TeX Gyre Termes"],
        "courier new" | "courier" => &["Liberation Mono", "Cousine", "Courier New", "Courier", "Nimbus Mono PS", "TeX Gyre Cursor"],
        _ => &[],
    };
    let mut v = vec![face.trim().to_string()];
    v.extend(extra.iter().map(|s| s.to_string()));
    v
}

/// The system's fonts, loaded once (on first use, as it takes a moment).
fn db() -> &'static fontdb::Database {
    static DB: OnceLock<fontdb::Database> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    })
}

/// The font file and face index that draw a face in a style (1 bold, 2 italic): a user font of
/// that name, else the system's face of that family (or a stand-in) nearest to the style.
pub fn face_file(face: &str, style: u8) -> Option<(PathBuf, u32)> {
    let key = norm(face);
    if let Some(u) = user_fonts().iter().find(|u| !u.predefined && norm(&u.name) == key) {
        let p = u.files[(style & 3) as usize].clone();
        if p.exists() {
            return Some((p, 0));
        }
    }
    let db = db();
    for fam in stand_ins(face) {
        let lower = fam.to_lowercase();
        // the family as the font names it (fontdb compares names exactly, CSS ignores case)
        let Some(name) = db.faces().flat_map(|f| f.families.iter()).find(|(n, _)| n.to_lowercase() == lower).map(|(n, _)| n.clone()) else { continue };
        let q = fontdb::Query {
            families: &[fontdb::Family::Name(&name)],
            weight: if style & 1 != 0 { fontdb::Weight::BOLD } else { fontdb::Weight::NORMAL },
            stretch: fontdb::Stretch::Normal,
            style: if style & 2 != 0 { fontdb::Style::Italic } else { fontdb::Style::Normal },
        };
        let Some(id) = db.query(&q) else { continue };
        match db.face_source(id) {
            Some((fontdb::Source::File(p), i)) | Some((fontdb::Source::SharedFile(p, _), i)) => return Some((p, i)),
            _ => {}
        }
    }
    None
}

// ------------------------------------------------------------------ the screen's fonts

/// The fonts given to egui: the start-up ones plus every face and fallback loaded since. Bumping
/// `generation` tells each egui context to take them again.
struct Registry {
    defs: FontDefinitions,
    /// face (lower case) → found
    faces: HashMap<String, bool>,
    generation: u64,
}

fn registry() -> &'static Mutex<Option<Registry>> {
    static REG: Mutex<Option<Registry>> = Mutex::new(None);
    &REG
}

/// The Unicode fallbacks (CJK) have been looked for.
static CJK_DONE: AtomicBool = AtomicBool::new(false);

const GEN_ID: &str = "umlcraft-fonts-generation";

/// Give egui these start-up fonts, and keep them as the base of the faces loaded later. The
/// user's "default" fonts are added as fallbacks of every diagram face (font-manager.js:84-126).
pub fn install(ctx: &egui::Context, mut defs: FontDefinitions) {
    let mut fallbacks = vec![];
    for (i, u) in user_fonts().iter().enumerate().filter(|(_, u)| u.name == "default" && !u.predefined) {
        if let Ok(bytes) = std::fs::read(&u.files[0]) {
            let name = format!("user-default-{i}");
            defs.font_data.insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
            fallbacks.push(name);
        }
    }
    add_fallbacks(&mut defs, &fallbacks, 0);
    let mut reg = registry().lock().unwrap();
    let generation = reg.as_ref().map(|r| r.generation + 1).unwrap_or(1);
    let faces = reg.take().map(|r| r.faces).unwrap_or_default();
    // faces loaded by an earlier context stay loaded
    let mut r = Registry { defs, faces: HashMap::new(), generation };
    for (k, found) in faces {
        if found {
            load_face(&mut r.defs, &k);
        }
        r.faces.insert(k, found);
    }
    ctx.set_fonts(r.defs.clone());
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(GEN_ID), generation));
    *reg = Some(r);
    drop(reg);
    // a CJK system language gets its fallbacks at once
    let lang = std::env::var("LC_ALL").or_else(|_| std::env::var("LANG")).unwrap_or_default();
    if ["zh", "ja", "ko"].iter().any(|l| lang.starts_with(l)) {
        need_glyphs(ctx, "中");
    }
}

/// The diagram families (the Arial ones and the loaded faces) whose chains get the fallbacks, and
/// the panels' fonts when `ui` is set.
fn add_fallbacks(defs: &mut FontDefinitions, names: &[String], skip_ui: u8) {
    if names.is_empty() {
        return;
    }
    for (fam, chain) in defs.families.iter_mut() {
        let diagram = matches!(fam, FontFamily::Name(n) if n.starts_with("arial") || n.starts_with("face:"));
        if diagram || (skip_ui == 1 && matches!(fam, FontFamily::Proportional | FontFamily::Monospace)) {
            for n in names {
                if !chain.contains(n) {
                    chain.push(n.clone());
                }
            }
        }
    }
}

fn family_name(key: &str, style: u8) -> String {
    format!("face:{key}:{}", style & 3)
}

/// Load the four styles of a face into the definitions; false when the face is not found.
fn load_face(defs: &mut FontDefinitions, key: &str) -> bool {
    let mut any = false;
    for style in 0..4u8 {
        let base = defs.families.get(&crate::canvas::diagram_family(style)).cloned().unwrap_or_default();
        let mut chain = vec![];
        if let Some((path, index)) = face_file(key, style) {
            let data_name = format!("file:{}#{index}", path.display());
            if !defs.font_data.contains_key(&data_name) {
                if let Ok(bytes) = std::fs::read(&path) {
                    let mut fd = FontData::from_owned(bytes);
                    fd.index = index;
                    defs.font_data.insert(data_name.clone(), Arc::new(fd));
                }
            }
            if defs.font_data.contains_key(&data_name) {
                chain.push(data_name);
                any = true;
            }
        }
        let first = chain.first().cloned();
        chain.extend(base.into_iter().filter(|n| Some(n) != first.as_ref()));
        defs.families.insert(FontFamily::Name(family_name(key, style).into()), chain);
    }
    if !any {
        for style in 0..4u8 {
            defs.families.remove(&FontFamily::Name(family_name(key, style).into()));
        }
    }
    any
}

/// Give this context the registry's fonts if it does not have them yet (they become active at its
/// next pass).
fn sync(ctx: &egui::Context, r: &Registry) {
    let id = egui::Id::new(GEN_ID);
    if ctx.data(|d| d.get_temp::<u64>(id)) != Some(r.generation) {
        ctx.set_fonts(r.defs.clone());
        ctx.data_mut(|d| d.insert_temp(id, r.generation));
        ctx.request_repaint();
    }
}

/// The egui family that draws a face in a style (1 bold, 2 italic). A face not loaded yet is
/// looked for now and drawn from the next frame on; until then, and when it is missing, the
/// Arial-compatible family draws it.
pub fn family(ctx: &egui::Context, face: &str, style: u8) -> FontFamily {
    if is_default(face) {
        return crate::canvas::diagram_family(style);
    }
    let key = norm(face);
    let fam = FontFamily::Name(family_name(&key, style).into());
    if ctx.fonts(|f| f.definitions().families.contains_key(&fam)) {
        return fam;
    }
    let mut reg = registry().lock().unwrap();
    let Some(r) = reg.as_mut() else { return crate::canvas::diagram_family(style) };
    if !r.faces.contains_key(&key) {
        let found = load_face(&mut r.defs, &key);
        r.faces.insert(key, found);
        if found {
            r.generation += 1;
        }
    }
    sync(ctx, r);
    crate::canvas::diagram_family(style)
}

/// True when a face is found among the user's and the system's fonts (the Arial face always is).
pub fn available(face: &str) -> bool {
    is_default(face) || face_file(face, 0).is_some()
}

/// Characters drawn from the CJK fallbacks: the blocks StarUML's Noto Sans KR, JP and SC subsets
/// cover (resources/fonts/NotoSans/font.json, unicode.js), with the full-width forms.
pub fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF)
}

/// The system fonts to draw CJK text with, best first, as StarUML picks Noto Sans SC for Chinese
/// characters, JP for kana and KR for Hangul (resources/fonts/NotoSans/font.json).
const CJK_FAMILIES: [&str; 22] = [
    "Noto Sans CJK SC", "Noto Sans SC", "Source Han Sans SC", "Microsoft YaHei", "PingFang SC", "Hiragino Sans GB", "WenQuanYi Zen Hei", "WenQuanYi Micro Hei", "Noto Sans CJK JP", "Noto Sans JP", "Yu Gothic",
    "Meiryo", "Hiragino Sans", "IPAGothic", "IPAPGothic", "MS Gothic", "Noto Sans CJK KR", "Noto Sans KR", "Malgun Gothic", "Apple SD Gothic Neo", "NanumGothic", "Droid Sans Fallback",
];

/// The CJK fallback fonts: for each of 中, あ and 한, the first font that has it, unless an
/// earlier pick has it too.
pub fn cjk_files() -> Vec<(PathBuf, u32)> {
    let db = db();
    let has = |id: fontdb::ID, c: char| db.with_face_data(id, |d, i| ttf_parser::Face::parse(d, i).ok().and_then(|f| f.glyph_index(c)).is_some()).unwrap_or(false);
    let mut picks: Vec<fontdb::ID> = vec![];
    for c in ['中', 'あ', '한'] {
        if picks.iter().any(|id| has(*id, c)) {
            continue;
        }
        let named = CJK_FAMILIES.iter().find_map(|fam| db.faces().find(|f| f.families.iter().any(|(n, _)| n == fam) && f.weight == fontdb::Weight::NORMAL && f.style == fontdb::Style::Normal && has(f.id, c)).map(|f| f.id));
        let any = || db.faces().find(|f| f.style == fontdb::Style::Normal && has(f.id, c)).map(|f| f.id);
        if let Some(id) = named.or_else(any) {
            picks.push(id);
        }
    }
    picks
        .into_iter()
        .filter_map(|id| match db.face_source(id) {
            Some((fontdb::Source::File(p), i)) | Some((fontdb::Source::SharedFile(p, _), i)) => Some((p, i)),
            _ => None,
        })
        .collect()
}

/// Text about to be drawn or measured: when it has CJK characters, the CJK fallbacks are loaded
/// (once) for the diagram faces and the panels (font-manager.js:84-126).
pub fn need_glyphs(ctx: &egui::Context, text: &str) {
    if CJK_DONE.load(Ordering::Relaxed) || !text.chars().any(is_cjk) {
        return;
    }
    CJK_DONE.store(true, Ordering::Relaxed);
    let files = cjk_files();
    let mut reg = registry().lock().unwrap();
    let Some(r) = reg.as_mut() else { return };
    let mut names = vec![];
    for (path, index) in files {
        let name = format!("file:{}#{index}", path.display());
        if !r.defs.font_data.contains_key(&name) {
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let mut fd = FontData::from_owned(bytes);
            fd.index = index;
            r.defs.font_data.insert(name.clone(), Arc::new(fd));
        }
        names.push(name);
    }
    if names.is_empty() {
        return;
    }
    add_fallbacks(&mut r.defs, &names, 1);
    r.generation += 1;
    sync(ctx, r);
}

/// For a window-less command: load the faces and fallbacks the project's views use, and run a
/// pass so they are active before the diagrams are laid out.
pub fn preload(ctx: &egui::Context, repo: &crate::model::Repo) {
    let mut faces: Vec<String> = vec![];
    let mut cjk = false;
    for e in repo.all() {
        let font = repo.str(e, "font");
        let face = font.split(';').next().unwrap_or("").to_string();
        if !is_default(&face) && !faces.contains(&face) {
            faces.push(face);
        }
        if !cjk {
            cjk = ["name", "text"].iter().any(|k| repo.str(e, k).chars().any(is_cjk));
        }
    }
    for f in &faces {
        family(ctx, f, 0);
    }
    if cjk {
        need_glyphs(ctx, "中");
    }
    let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
    out.textures_delta.clear();
}
