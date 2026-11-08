//! Command-line commands, as StarUML's (src/main-process/cli.js): `umlcraft ejs <file.mdj> -t
//! <template.ejs> …`, `umlcraft image <file.mdj> [-f png|jpeg|svg] [-s selector] [-o output]`,
//! `umlcraft html …`, `umlcraft pdf <file.mdj> …`, `umlcraft exec <file.mdj> -c <command> [-a arg]`,
//! `umlcraft mcp [file.mdj]` (src/mcp.rs), `--help` and `--version`. They run without a window.

use crate::app::App;
use crate::export::{filenamify, Format};
use std::path::PathBuf;

pub const USAGE: &str = "Usage: umlcraft [file.mdj] [--diagram <name>] [--select <name>]
       umlcraft ejs <file.mdj> -t <template.ejs> [-o output.txt] [-s @Project]
       umlcraft image <file.mdj> [-f png|jpeg|svg] [-s selector] [-o output]
       umlcraft pdf <file.mdj> [-s selector] [-o output.pdf] [-z A4] [-l landscape|portrait] [-n true|false]
       umlcraft html <file.mdj> [-o ./html-docs]
       umlcraft exec <file.mdj> -c <command-id> [-a arg] [-c <command-id> [-a arg]]... | --list
       umlcraft mcp [file.mdj] [--api-port=<port>]
       umlcraft --resave <in.mdj> <out.mdj>
       umlcraft --help | --version

  ejs     render the EJS template once for each element matching the selector (default
          @Project), with app, element and filenamify. -o (default output.txt) is itself a
          template, so each element can get its own file.
  image   export the diagrams matching the selector (default @Diagram) as pictures.
          -o is an EJS template of the file name, rendered per diagram.
          The default is filenamify(<name>.<format>).
  pdf     print the diagrams matching the selector (default @Diagram) to one PDF, one per page.
          -o defaults to output.pdf, -z to A4, -l to landscape; -n false hides the diagram names.
  html    write the model as HTML pages into the folder -o (default ./html-docs).
  exec    run commands on the model, in order (umlcraft exec --list shows them);
          -a is the command's argument: a string, or JSON for the factory commands.
          Nothing is saved unless a command saves (-c project:save [-a file.mdj]).
  mcp     serve the Model Context Protocol on stdin and stdout for AI tools. Without a file,
          the tools act on the running window through its API server (port 58321 unless
          --api-port is given); with a file, on that file, which is saved after each change.";

/// Run a command-line command when the arguments ask for one; returns the exit code, or None to start
/// the window.
pub fn run(args: &[String], ctx: impl Fn() -> egui::Context) -> Option<i32> {
    // `--help` after a command shows the help too, as yargs does (cli.js:291-303)
    let command = matches!(args.first().map(String::as_str), Some("ejs" | "image" | "html" | "pdf" | "exec" | "mcp"));
    if command && args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return Some(0);
    }
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
        Some("pdf") => Some(pdf(&args[1..], ctx())),
        Some("html") => Some(html(&args[1..], ctx())),
        Some("ejs") => Some(big_stack(args[1..].to_vec(), |a| ejs(&a))),
        Some("exec") => Some(crate::exec::cli(&args[1..], ctx())),
        Some("mcp") => Some(crate::mcp::cli(&args[1..], ctx)),
        _ => None,
    }
}

/// The value of an option, by its short flag or its long name (yargs aliases: -t/--template,
/// -o/--output, -s/--select, -f/--format, -z/--size, -l/--layout, -n/--showname, -c/--command,
/// -a/--arg), as `-o x`, `--output x` or `--output=x`.
pub(crate) fn opt<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let long = match flag {
        "-t" => "--template",
        "-o" => "--output",
        "-s" => "--select",
        "-f" => "--format",
        "-z" => "--size",
        "-l" => "--layout",
        "-n" => "--showname",
        "-c" => "--command",
        "-a" => "--arg",
        _ => flag,
    };
    for (i, a) in args.iter().enumerate() {
        if a == flag || a == long {
            return args.get(i + 1).map(String::as_str);
        }
        if let Some(v) = a.strip_prefix(long).and_then(|r| r.strip_prefix('=')) {
            return Some(v);
        }
    }
    None
}

/// Run a command on a thread with a large stack: templates may recurse deeply.
fn big_stack(args: Vec<String>, f: fn(Vec<String>) -> i32) -> i32 {
    std::thread::Builder::new().stack_size(512 * 1024 * 1024).spawn(move || f(args)).ok().and_then(|h| h.join().ok()).unwrap_or(1)
}

/// `ejs`: render the template for each selected element (cli.js:18-40, 132-163,
/// default-commands.js:1184-1215).
fn ejs(args: &[String]) -> i32 {
    let Some(file) = args.first().filter(|a| !a.starts_with('-')) else {
        eprintln!("{USAGE}");
        return 2;
    };
    if !PathBuf::from(file).exists() {
        eprintln!("[Error] File not found: {file}");
        return 1;
    }
    let Some(template) = opt(args, "-t") else {
        eprintln!("Missing required argument: t");
        return 2;
    };
    let tpath = PathBuf::from(template);
    if !tpath.exists() {
        eprintln!("[Error] File not found: {template}");
        return 1;
    }
    let mut a = App::new(None);
    a.open_path(PathBuf::from(file));
    if let Some((t, d)) = a.message.take() {
        eprintln!("[Error] {t} {d}");
        return 1;
    }
    let text = match std::fs::read_to_string(&tpath) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("[Error] {e}");
            return 1;
        }
    };
    let output = opt(args, "-o").unwrap_or("output.txt");
    let filename = a.filename.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
    let dir = tpath.parent().map(|d| if d.as_os_str().is_empty() { std::path::Path::new(".") } else { d });
    let run = || -> Result<usize, String> {
        let elements = crate::selector::select(&a.repo, opt(args, "-s").unwrap_or("@Project"))?;
        for e in &elements {
            let rendered = crate::ejs::render(&text, &a.repo, &filename, Some(*e), dir)?;
            let out = crate::ejs::render(output, &a.repo, &filename, Some(*e), None)?;
            let out = PathBuf::from(out);
            if let Some(d) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
                std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            std::fs::write(&out, rendered).map_err(|e| format!("{}: {e}", out.display()))?;
            println!("[UMLCraft] {}", out.display());
        }
        Ok(elements.len())
    };
    match run() {
        Ok(n) => {
            println!("[UMLCraft] Total {n} files were generated");
            0
        }
        Err(e) => {
            eprintln!("[Error] {e}");
            1
        }
    }
}

/// `image`: export the diagrams the selector matches (cli.js:164-195, default-commands.js:1217-1260).
fn image(args: &[String], ctx: egui::Context) -> i32 {
    let format = match opt(args, "-f").unwrap_or("png") {
        "png" => Format::Png,
        "jpeg" | "jpg" => Format::Jpeg,
        "svg" => Format::Svg,
        other => {
            eprintln!("[Error] Unknown format: {other}");
            return 2;
        }
    };
    let (mut a, diagrams) = match open_diagrams(args) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let ms = crate::canvas::EguiMeasure { ctx };
    let mut n = 0;
    for d in diagrams {
        let name = a.repo.name(d);
        let out = match opt(args, "-o") {
            Some(o) => match crate::ejs::render(o, &a.repo, &a.filename.as_ref().map(|p| p.display().to_string()).unwrap_or_default(), Some(d), None) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("[Error] {e}");
                    return 1;
                }
            },
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

/// Open the file and select the diagrams; an error is printed and gives the exit code.
fn open_diagrams(args: &[String]) -> Result<(App, Vec<crate::model::Eid>), i32> {
    let Some(file) = args.first().filter(|a| !a.starts_with('-')) else {
        eprintln!("{USAGE}");
        return Err(2);
    };
    let path = PathBuf::from(file);
    if !path.exists() {
        eprintln!("[Error] File not found: {file}");
        return Err(1);
    }
    let mut a = App::new(None);
    a.open_path(path);
    if let Some((t, d)) = a.message.take() {
        eprintln!("[Error] {t} {d}");
        return Err(1);
    }
    let selected = crate::selector::select(&a.repo, opt(args, "-s").unwrap_or("@Diagram")).map_err(|e| {
        eprintln!("[Error] {e}");
        2
    })?;
    let diagrams = selected.into_iter().filter(|e| a.is_diagram(*e)).collect();
    Ok((a, diagrams))
}

/// `pdf`: print the selected diagrams to one PDF (cli.js:216-260, default-commands.js:1278-1296).
fn pdf(args: &[String], ctx: egui::Context) -> i32 {
    let (mut a, diagrams) = match open_diagrams(args) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let size = opt(args, "-z").unwrap_or("A4");
    if !crate::pdf::PAGE_SIZES.iter().any(|p| p.0.eq_ignore_ascii_case(size)) {
        eprintln!("[Error] Unknown page size: {size}");
        return 2;
    }
    let landscape = match opt(args, "-l").unwrap_or("landscape") {
        "landscape" => true,
        "portrait" => false,
        other => {
            eprintln!("[Error] Unknown page layout: {other}");
            return 2;
        }
    };
    let show_name = !args.iter().any(|x| x == "--no-showname") && opt(args, "-n").map(|n| n != "false").unwrap_or(true);
    let o = crate::pdf::PrintOptions { all: true, landscape, size: size.to_string(), show_name };
    let out = PathBuf::from(opt(args, "-o").unwrap_or("output.pdf"));
    if let Some(dir) = out.parent().filter(|d| !d.as_os_str().is_empty()) {
        let _ = std::fs::create_dir_all(dir);
    }
    let ms = crate::canvas::EguiMeasure { ctx };
    match a.pdf_of(&diagrams, &o, &ms).and_then(|b| std::fs::write(&out, b).map_err(|e| e.to_string())) {
        Ok(()) => {
            println!("[UMLCraft] {}", out.display());
            println!("Total {} diagrams were printed", diagrams.len());
            0
        }
        Err(e) => {
            eprintln!("[Error] {e}");
            1
        }
    }
}

/// `html`: write the HTML docs into the output folder (cli.js:196-215, default-commands.js:1262-1276).
fn html(args: &[String], ctx: egui::Context) -> i32 {
    let (mut a, _) = match open_diagrams(args) {
        Ok(x) => x,
        Err(code) => return code,
    };
    let out = PathBuf::from(opt(args, "-o").unwrap_or("./html-docs"));
    let ms = crate::canvas::EguiMeasure { ctx };
    match a.export_html(&out, &ms) {
        Ok(()) => {
            println!("HTML docs were exported on {}", out.display());
            0
        }
        Err(e) => {
            eprintln!("[Error] {e}");
            1
        }
    }
}
