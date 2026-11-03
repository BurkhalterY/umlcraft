//! UMLCraft: a UML modeler that reads and writes StarUML .mdj project files.
//! Start-up: window, fonts, theme, a file to open and the `--screenshot <file.png>` option.

mod app;
mod canvas;
mod icons;
mod meta;
mod model;
mod parse;
mod render;
mod ui;
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
    let faces = [
        ("arial", DIAGRAM_FONT, format!("{lib}/LiberationSans-Regular.ttf")),
        ("arial-b", "Arial:bold", format!("{lib}/LiberationSans-Bold.ttf")),
        ("arial-i", "Arial:italic", format!("{lib}/LiberationSans-Italic.ttf")),
        ("arial-bi", "Arial:bold:italic", format!("{lib}/LiberationSans-BoldItalic.ttf")),
    ];
    let faces: Vec<(&str, Vec<String>)> = faces
        .into_iter()
        .map(|(name, pattern, known)| (name, system_font(pattern).into_iter().chain(std::iter::once(known)).collect()))
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
    ]) {
        fonts.font_data.insert("ui".into(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "ui".into());
    }
    ctx.set_fonts(fonts);
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
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--screenshot" {
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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
            let mut a = app::App::new(screenshot);
            if let Some(f) = file {
                a.open_path(f);
            }
            Ok(Box::new(a))
        }),
    )
}
