//! UMLCraft: a UML modeler that reads and writes StarUML .mdj project files.
//! Start-up: window, fonts, theme, a file to open and the `--screenshot <file.png>` option.

// no console window next to the app on Windows (release builds)
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod align;
mod app;
mod cli;
mod clipboard;
mod export;
mod find;
mod canvas;
mod icons;
mod meta;
mod model;
mod palette;
mod parse;
mod prefs;
mod related;
mod render;
mod selector;
mod style;
mod tags;
mod ui;
mod validate;
mod views;
#[cfg(test)]
mod tests;

use egui::{FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::sync::Arc;

/// The original's diagram font face (View font "Arial;13;0", core.js:1475).
const DIAGRAM_FONT: &str = "Arial";

fn first_existing(paths: &[&str]) -> Option<Vec<u8>> {
    paths.iter().find_map(|p| std::fs::read(p).ok())
}

/// Diagram text uses an Arial-compatible face (the original's default font is Arial 13);
/// the panels use a Source-Sans-like UI face when one is installed.
/// The face the system gives for a font request, the way the original's engine resolves the
/// diagram font "Arial" on Linux (through fontconfig; MECHANICS.md "Runtime"). `None` without fontconfig.
fn system_font(pattern: &str) -> Option<String> {
    let out = std::process::Command::new("fc-match").args(["-f", "%{file}", pattern]).output().ok()?;
    let file = String::from_utf8(out.stdout).ok()?;
    std::path::Path::new(&file).exists().then_some(file)
}

pub(crate) fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let lib = "/usr/share/fonts/truetype/liberation";
    // where Arial (or a metric-compatible face) lives on Linux/FreeBSD, Windows and macOS
    let win = "C:\\Windows\\Fonts";
    let mac = "/System/Library/Fonts/Supplemental";
    let faces = [
        ("arial", DIAGRAM_FONT, vec![format!("{lib}/LiberationSans-Regular.ttf"), "/usr/local/share/fonts/Liberation/LiberationSans-Regular.ttf".into(), format!("{win}\\arial.ttf"), format!("{mac}/Arial.ttf"), "/Library/Fonts/Arial.ttf".into()]),
        ("arial-b", "Arial:bold", vec![format!("{lib}/LiberationSans-Bold.ttf"), "/usr/local/share/fonts/Liberation/LiberationSans-Bold.ttf".into(), format!("{win}\\arialbd.ttf"), format!("{mac}/Arial Bold.ttf"), "/Library/Fonts/Arial Bold.ttf".into()]),
        ("arial-i", "Arial:italic", vec![format!("{lib}/LiberationSans-Italic.ttf"), "/usr/local/share/fonts/Liberation/LiberationSans-Italic.ttf".into(), format!("{win}\\ariali.ttf"), format!("{mac}/Arial Italic.ttf"), "/Library/Fonts/Arial Italic.ttf".into()]),
        ("arial-bi", "Arial:bold:italic", vec![format!("{lib}/LiberationSans-BoldItalic.ttf"), "/usr/local/share/fonts/Liberation/LiberationSans-BoldItalic.ttf".into(), format!("{win}\\arialbi.ttf"), format!("{mac}/Arial Bold Italic.ttf"), "/Library/Fonts/Arial Bold Italic.ttf".into()]),
    ];
    let faces: Vec<(&str, Vec<String>)> = faces
        .into_iter()
        .map(|(name, pattern, known)| (name, system_font(pattern).into_iter().chain(known).collect()))
        .collect();
    let fallback: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (name, paths) in faces {
        let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
        let mut chain = vec![];
        if let Some(bytes) = first_existing(&refs) {
            fonts.font_data.insert(name.to_string(), Arc::new(FontData::from_owned(bytes)));
            chain.push(name.to_string());
        }
        chain.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(name.into()), chain);
    }
    if let Some(bytes) = first_existing(&[
        "/usr/share/fonts/opentype/source-sans-pro/SourceSansPro-Regular.otf",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/local/share/fonts/noto/NotoSans-Regular.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
    ]) {
        fonts.font_data.insert("ui".into(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "ui".into());
    }
    ctx.set_fonts(fonts);
}

/// The app icon shown by the window manager (packaging/icons, drawn for UMLCraft).
fn window_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(include_bytes!("../packaging/icons/umlcraft-256.png")).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width, height })
}

fn setup_style(ctx: &egui::Context) {
    use ui::*;
    // the original's default theme is "dark" (index.html loads staruml-dark.css)
    ctx.set_visuals(egui::Visuals::dark());
    ctx.global_style_mut(|s| {
        let v = &mut s.visuals;
        v.panel_fill = LAYER0;
        v.window_fill = LAYER1;
        v.extreme_bg_color = INPUT;
        v.override_text_color = Some(TEXT);
        v.selection.bg_fill = ACCENT;
        v.widgets.inactive.bg_fill = INPUT;
        v.widgets.inactive.weak_bg_fill = INPUT;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, INPUT_BORDER);
        v.widgets.hovered.weak_bg_fill = HOVER_ROW;
        v.widgets.active.weak_bg_fill = ACCENT;
        s.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        s.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        s.spacing.button_padding = egui::vec2(8.0, 3.0);
    });
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // command-line commands (image, --help, --version) run without a window
    let headless = || {
        let ctx = egui::Context::default();
        setup_fonts(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        ctx
    };
    if let Some(code) = cli::run(&args, headless) {
        std::process::exit(code);
    }
    // `--resave <in.mdj> <out.mdj>`: open a model and save it again without a window
    if let Some(i) = args.iter().position(|a| a == "--resave") {
        let (Some(src), Some(dst)) = (args.get(i + 1), args.get(i + 2)) else {
            eprintln!("usage: umlcraft --resave <in.mdj> <out.mdj>");
            std::process::exit(2);
        };
        let ctx = egui::Context::default();
        setup_fonts(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let mut a = app::App::new(None);
        a.open_path(PathBuf::from(src));
        if let Some((t, d)) = a.message.take() {
            eprintln!("{t} {d}");
            std::process::exit(1);
        }
        let ok = a.save_to(PathBuf::from(dst), &canvas::EguiMeasure { ctx });
        std::process::exit(if ok { 0 } else { 1 });
    }
    let mut screenshot: Option<PathBuf> = None;
    let mut file: Option<PathBuf> = None;
    let mut diagram: Option<String> = None;
    let mut select: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--diagram" || args[i] == "--select" {
            // `--diagram <name>` opens that diagram, `--select <name>` selects that element in it
            let v = args.get(i + 1).cloned();
            if args[i] == "--diagram" { diagram = v } else { select = v }
            i += 1;
        } else if args[i] == "--screenshot" {
            screenshot = Some(PathBuf::from(args.get(i + 1).cloned().unwrap_or_else(|| "screenshot.png".into())));
            i += 1;
        } else if args[i].ends_with(".mdj") && std::path::Path::new(&args[i]).exists() {
            file = Some(PathBuf::from(&args[i]));
        }
        i += 1;
    }
    if let Some(p) = &screenshot {
        if p.is_relative() {
            screenshot = std::env::current_dir().ok().map(|d| d.join(p));
        }
    }

    let mut viewport = egui::ViewportBuilder::default();
    if let Some(icon) = window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport: viewport
            .with_title("Untitled — UMLCraft")
            .with_app_id("umlcraft")
            .with_inner_size([ui::WINDOW_WIDTH, ui::WINDOW_HEIGHT]),
        ..Default::default()
    };
    eframe::run_native(
        "UMLCraft",
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            let shot = screenshot.is_some();
            let mut a = app::App::new(screenshot);
            // screenshots start from the defaults, so they do not depend on this machine's settings
            if !shot {
                a.prefs = prefs::Prefs::load(prefs::default_path());
                a.apply_prefs();
            }
            match file {
                Some(f) => {
                    a.open_path(f);
                }
                // with no file given: the backup after a crash, else the last file when asked to
                None if !shot => {
                    if !a.recover_backup() {
                        a.auto_load_working_file();
                    }
                }
                None => {}
            }
            a.show_by_name(diagram.as_deref(), select.as_deref());
            Ok(Box::new(a))
        }),
    )
}
