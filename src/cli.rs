//! Command-line commands, as StarUML's (src/main-process/cli.js): `umlcraft image <file.mdj>
//! [-f png|jpeg|svg] [-s selector] [-o output]`, `--help` and `--version`. They run without a window.

use crate::app::App;
use crate::export::{filenamify, Format};
use std::path::PathBuf;

pub const USAGE: &str = "Usage: umlcraft [file.mdj] [--diagram <name>] [--select <name>]
       umlcraft image <file.mdj> [-f png|jpeg|svg] [-s selector] [-o output]
       umlcraft --resave <in.mdj> <out.mdj>
       umlcraft --help | --version

  image   export the diagrams matching the selector (default @Diagram) as pictures.
          -o is a file name; <%= element.name %> and <%= filenamify(element.name) %> in it
          are replaced per diagram. The default is filenamify(<name>.<format>).";

/// Run a command-line command when the arguments ask for one; returns the exit code, or None to start
/// the window.
pub fn run(args: &[String], ctx: impl Fn() -> egui::Context) -> Option<i32> {
    match args.first().map(String::as_str) {
        Some("--help") | Some("-h") => {
            println!("{USAGE}");
            Some(0)
        }
        Some("--version") | Some("-v") => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Some(0)
        }
        Some("image") => Some(image(&args[1..], ctx())),
        Some(cmd @ ("ejs" | "html" | "pdf" | "exec")) => {
            eprintln!("[Error] The {cmd} command is not supported by UMLCraft.");
            Some(2)
        }
        _ => None,
    }
}

fn opt<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str)
}

/// `image`: export the diagrams the selector matches (cli.js:164-195, default-commands.js:1217-1260).
fn image(args: &[String], ctx: egui::Context) -> i32 {
    let Some(file) = args.first().filter(|a| !a.starts_with('-')) else {
        eprintln!("{USAGE}");
        return 2;
    };
    let path = PathBuf::from(file);
    if !path.exists() {
        eprintln!("[Error] File not found: {file}");
        return 1;
    }
    let format = match opt(args, "-f").unwrap_or("png") {
        "png" => Format::Png,
        "jpeg" | "jpg" => Format::Jpeg,
        "svg" => Format::Svg,
        other => {
            eprintln!("[Error] Unknown format: {other}");
            return 2;
        }
    };
    let mut a = App::new(None);
    a.open_path(path);
    if let Some((t, d)) = a.message.take() {
        eprintln!("[Error] {t} {d}");
        return 1;
    }
    let selected = match crate::selector::select(&a.repo, opt(args, "-s").unwrap_or("@Diagram")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Error] {e}");
            return 2;
        }
    };
    let diagrams: Vec<_> = selected.into_iter().filter(|e| a.is_diagram(*e)).collect();
    let ms = crate::canvas::EguiMeasure { ctx };
    let mut n = 0;
    for d in diagrams {
        let name = a.repo.name(d);
        let out = match opt(args, "-o") {
            Some(o) => o.replace("<%= filenamify(element.name) %>", &filenamify(&name)).replace("<%= element.name %>", &name),
            None => filenamify(&format!("{name}.{}", match format {
                Format::Png => "png",
                Format::Jpeg => "jpeg",
                Format::Svg => "svg",
            })),
        };
        let out = PathBuf::from(out);
        if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
            let _ = std::fs::create_dir_all(dir);
        }
        match crate::export::render(&mut a, d, &ms, format, 1.0).and_then(|b| std::fs::write(&out, b).map_err(|e| e.to_string())) {
            Ok(()) => {
                println!("[UMLCraft] {}", out.display());
                n += 1;
            }
            Err(e) => eprintln!("[Error] {name}: {e}"),
        }
    }
    println!("Total {n} diagrams were exported");
    0
}
