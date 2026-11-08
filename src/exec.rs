//! `umlcraft exec <file.mdj> -c <command-id> [-a arg]`: run commands on a model without a window
//! (src/main-process/cli.js:111-125, 261-286, src/engine/default-commands.js:1298-1313), and the
//! scriptable factory and engine commands (default-commands.js:1357-1387, 1688-1695).
//!
//! As in StarUML, the model is loaded, the commands run and nothing is saved unless a command saves.
//! Deliberate differences: several `-c` (each followed by its `-a`) run in order, so that a script
//! can create elements and then save; `-a` is parsed as JSON when it is a JSON object (StarUML passes
//! the string, which its factory commands cannot use); elements in the arguments are named by id,
//! `{"$ref": id}`, or `"$N"` for the element the N-th command of the call returned; the id of each
//! element a command creates is printed; commands that need a window or a selection are refused
//! with "Command needs a window"; the exit code is 1 when a command failed.

use crate::app::App;
use crate::views::Measure;
use crate::model::Eid;
use crate::views::{P, R};
use serde_json::{json, Value};
use std::path::PathBuf;

/// The commands `exec` runs, with what their argument is (StarUML's command ids; the last two are
/// UMLCraft's own names for the project manager's importFromJson and loadFromJson).
pub const COMMANDS: &[(&str, &str)] = &[
    ("application:log", "print the argument"),
    ("application:main-log", "print the argument"),
    ("application:quit", "stop (the commands after it are not run)"),
    ("project:new", "a new project; the argument: a template .mdj to start from"),
    ("project:open", "open the .mdj file of the argument"),
    ("project:save", "save, to the file of the argument if given"),
    ("project:save-as", "save to the file of the argument"),
    ("project:close", "stop (the commands after it are not run)"),
    ("project:import-fragment", "import the .mfj file of the argument into the project"),
    ("project:export-diagram-to-png", "the current diagram as PNG; the argument: the file"),
    ("project:export-diagram-to-jpeg", "the current diagram as JPEG; the argument: the file"),
    ("project:export-diagram-to-svg", "the current diagram as SVG; the argument: the file"),
    ("project:export-diagram-all-to-pngs", "every diagram as PNG into the folder of the argument"),
    ("project:export-diagram-all-to-jpegs", "every diagram as JPEG into the folder of the argument"),
    ("project:export-diagram-all-to-svgs", "every diagram as SVG into the folder of the argument"),
    ("project:print-to-pdf", "every diagram into the PDF of the argument (A4, landscape)"),
    ("html-export:export", "HTML docs into the folder of the argument"),
    ("edit:undo", "undo"),
    ("edit:redo", "redo"),
    ("mermaid:generate-diagram", "a diagram from the Mermaid code of the argument"),
    ("diagram-generator:overview", "generate an overview diagram"),
    ("diagram-generator:overview-expanded", "generate an expanded overview diagram"),
    ("diagram-generator:type-hierarchy", "generate a type hierarchy diagram"),
    ("diagram-generator:package-structure", "generate a package structure diagram"),
    ("diagram-layout:auto", "lay out the current diagram"),
    ("diagram-layout:top-bottom", "lay out the current diagram top to bottom"),
    ("diagram-layout:bottom-top", "lay out the current diagram bottom to top"),
    ("diagram-layout:left-right", "lay out the current diagram left to right"),
    ("diagram-layout:right-left", "lay out the current diagram right to left"),
    ("uml:apply-profile.uml-standard", "apply the UML Standard Profile"),
    ("factory:create-diagram", "{id, parent, diagram-init}: a diagram of type id"),
    ("factory:create-model", "{id, parent, field, model-init}: a model element of type id"),
    ("factory:create-model-and-view", "{id, diagram, x1, y1, x2, y2, tailView, headView, model-init, view-init}"),
    ("engine:set-property", "{model, property, value, set-model}: set a field"),
    ("project:import-from-json", "{parent, data}: insert an element tree, keeping its ids"),
    ("project:load-from-json", "{data}: replace the project with this one"),
];

/// Commands StarUML has that need a window, a dialog or a selection.
const WINDOW_ONLY: &[&str] = &[
    "application:preferences", "application:reload", "project:export-fragment", "edit:cut", "edit:copy", "edit:copy-diagram-as-image", "edit:paste", "edit:delete", "edit:delete-from-model", "edit:move-up", "edit:move-down",
    "edit:select-all", "edit:select-in-explorer", "edit:select-in-diagram", "edit:open-sub-diagram", "tools:extension-manager", "model:tag-editor", "help:about", "help:check-for-updates", "help:license-activation",
    "help:documentation", "help:forum", "help:release-notes", "mermaid:show-mermaid-dialog",
];

fn usage() -> i32 {
    eprintln!("{}", crate::cli::USAGE);
    2
}

/// `umlcraft exec`: returns the exit code.
pub fn cli(args: &[String], ctx: egui::Context) -> i32 {
    if args.iter().any(|a| a == "--list") {
        for (id, what) in COMMANDS {
            println!("{id:<38} {what}");
        }
        return 0;
    }
    let Some(file) = args.first().filter(|a| !a.starts_with('-')) else { return usage() };
    let path = PathBuf::from(file);
    if !path.exists() {
        eprintln!("[Error] File not found: {file}");
        return 1;
    }
    // the commands in order, each with the -a after it (or before it, for the first)
    let mut cmds: Vec<(String, Option<String>)> = vec![];
    let mut pending: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        let (flag, inline) = match a.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f, Some(v.to_string())),
            _ => (a.as_str(), None),
        };
        let value = || inline.clone().or_else(|| args.get(i + 1).cloned());
        match flag {
            "-c" | "--command" => {
                if let Some(v) = value() {
                    cmds.push((v, pending.take()));
                }
            }
            "-a" | "--arg" => {
                if let Some(v) = value() {
                    match cmds.last_mut() {
                        Some(last) if last.1.is_none() => last.1 = Some(v),
                        _ => pending = Some(v),
                    }
                }
            }
            _ => {
                i += 1;
                continue;
            }
        }
        i += if inline.is_some() { 1 } else { 2 };
    }
    let mut a = App::new(None);
    a.open_path(path);
    if let Some((t, d)) = a.message.take() {
        eprintln!("[Error] {t} {d}");
        return 1;
    }
    if cmds.is_empty() {
        println!("[UMLCraft] No command specified");
        return 0;
    }
    let ms = crate::canvas::EguiMeasure { ctx };
    let mut results: Vec<Option<Eid>> = vec![];
    let mut code = 0;
    for (id, arg) in cmds {
        if !COMMANDS.iter().any(|c| c.0 == id) {
            if WINDOW_ONLY.contains(&id.as_str()) || crate::palette::COMMANDS.iter().any(|c| c.id == id) {
                println!("[UMLCraft] Command needs a window: {id}");
            } else {
                println!("[UMLCraft] Command not found: {id}");
            }
            results.push(None);
            code = 1;
            continue;
        }
        if matches!(id.as_str(), "application:quit" | "project:close") {
            break;
        }
        match a.exec_command(&id, arg.as_deref(), &results, &ms) {
            Ok(made) => {
                if let Some(e) = made {
                    println!("[UMLCraft] {} {}", a.repo.ty(e), a.repo.id(e));
                }
                results.push(made);
            }
            Err(e) => {
                eprintln!("[Error] {e}");
                results.push(None);
                code = 1;
            }
        }
    }
    code
}

/// An element named by an argument: an id, `{"$ref": id}`, `{"_id": id}` or `"$N"` (the result of
/// the N-th command).
fn resolve(a: &App, v: Option<&Value>, results: &[Option<Eid>]) -> Result<Option<Eid>, String> {
    let id = match v {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(s)) => {
            if let Some(n) = s.strip_prefix('$').and_then(|n| n.parse::<usize>().ok()) {
                return results.get(n.wrapping_sub(1)).copied().flatten().map(Some).ok_or(format!("Command {n} made no element"));
            }
            s.clone()
        }
        Some(Value::Object(m)) => m.get("$ref").or_else(|| m.get("_id")).and_then(|x| x.as_str()).unwrap_or_default().to_string(),
        Some(other) => return Err(format!("Not an element: {other}")),
    };
    a.repo.by_id.get(&id).copied().filter(|e| a.repo.alive(*e)).map(Some).ok_or(format!("Element not found: {id}"))
}

/// `$N` and ids inside a JSON value (model-init, view-init, set-property values) as `{"$ref": id}`.
fn refs_in(a: &App, v: &Value, results: &[Option<Eid>]) -> Value {
    match v {
        Value::String(s) if s.starts_with('$') && s[1..].parse::<usize>().is_ok() => match resolve(a, Some(v), results) {
            Ok(Some(e)) => a.repo.ref_to(e),
            _ => v.clone(),
        },
        Value::Object(m) if !m.contains_key("$ref") => Value::Object(m.iter().map(|(k, x)| (k.clone(), refs_in(a, x, results))).collect()),
        Value::Array(xs) => Value::Array(xs.iter().map(|x| refs_in(a, x, results)).collect()),
        _ => v.clone(),
    }
}

impl App {
    /// Assign an init object to an element: nested objects go into the element's embedded objects
    /// (an association's ends), other values are set (factory.js:139-150 assignInitObject).
    fn assign_init(&mut self, e: Eid, init: Option<&Value>) {
        let Some(Value::Object(m)) = init else { return };
        for (k, v) in m {
            match (v, self.repo.child(e, k)) {
                (Value::Object(o), Some(c)) if !o.contains_key("$ref") => self.assign_init(c, Some(v)),
                _ => self.repo.set(e, k, v.clone()),
            }
        }
    }

    /// Run one command of `exec`; returns the element it made, if any.
    pub fn exec_command(&mut self, id: &str, arg: Option<&str>, results: &[Option<Eid>], ms: &dyn Measure) -> Result<Option<Eid>, String> {
        use crate::export::Format;
        let opts: Value = match arg {
            Some(t) if t.trim_start().starts_with('{') => serde_json::from_str(t).map_err(|e| format!("The argument is not valid JSON: {e}"))?,
            Some(t) => Value::String(t.to_string()),
            None => Value::Null,
        };
        let text = arg.unwrap_or("");
        let need = |what: &str| -> Result<&str, String> { if text.is_empty() { Err(format!("{id} needs {what} (-a)")) } else { Ok(text) } };
        let current = |a: &App| a.diagram.filter(|d| a.repo.alive(*d)).ok_or_else(|| "There is no current diagram".to_string());
        match id {
            "application:log" | "application:main-log" => println!("{text}"),
            "project:new" => {
                let repo = if text.is_empty() { crate::model::Repo::blank() } else { crate::model::Repo::parse(&std::fs::read_to_string(text).map_err(|e| e.to_string())?)? };
                self.set_project(repo, None);
            }
            "project:open" => {
                self.open_path(PathBuf::from(need("a file")?));
                if let Some((t, d)) = self.message.take() {
                    return Err(format!("{t} {d}"));
                }
            }
            "project:save" | "project:save-as" => {
                let p = if text.is_empty() && id == "project:save" { self.filename.clone().ok_or("The project has no file: give one with -a")? } else { PathBuf::from(need("a file")?) };
                if !self.save_to(p.clone(), ms) {
                    return Err(self.message.take().map(|(t, d)| format!("{t} {d}")).unwrap_or_else(|| format!("Could not save {}", p.display())));
                }
                println!("[UMLCraft] {}", p.display());
            }
            "project:import-fragment" => return self.import_fragment(std::path::Path::new(need("a file")?)).map(Some),
            "project:export-diagram-to-png" | "project:export-diagram-to-jpeg" | "project:export-diagram-to-svg" => {
                let f = match id {
                    "project:export-diagram-to-png" => Format::Png,
                    "project:export-diagram-to-jpeg" => Format::Jpeg,
                    _ => Format::Svg,
                };
                let d = current(self)?;
                let out = if text.is_empty() { PathBuf::from(self.export_name(d, f)) } else { PathBuf::from(text) };
                let bytes = crate::export::render(self, d, ms, f, 1.0)?;
                std::fs::write(&out, bytes).map_err(|e| e.to_string())?;
                println!("[UMLCraft] {}", out.display());
            }
            "project:export-diagram-all-to-pngs" | "project:export-diagram-all-to-jpegs" | "project:export-diagram-all-to-svgs" => {
                let f = match id {
                    "project:export-diagram-all-to-pngs" => Format::Png,
                    "project:export-diagram-all-to-jpegs" => Format::Jpeg,
                    _ => Format::Svg,
                };
                let n = self.export_all(std::path::Path::new(need("a folder")?), ms, f, 1.0)?;
                println!("[UMLCraft] Total {n} diagrams were exported");
            }
            "project:print-to-pdf" => {
                let out = PathBuf::from(if text.is_empty() { "output.pdf" } else { text });
                let o = crate::pdf::PrintOptions { all: true, landscape: true, size: "A4".into(), show_name: true };
                let diagrams = self.repo.all_of("Diagram");
                let bytes = self.pdf_of(&diagrams, &o, ms)?;
                std::fs::write(&out, bytes).map_err(|e| e.to_string())?;
                println!("[UMLCraft] {}", out.display());
            }
            "html-export:export" => {
                let out = PathBuf::from(need("a folder")?);
                self.export_html(&out, ms)?;
                println!("HTML docs were exported on {}", out.display());
            }
            "edit:undo" => self.do_undo(),
            "edit:redo" => self.do_redo(),
            // the code alone, or `{"code": …, "base": element}` for the command's optional base
            // (mermaid/main.js:155-185)
            "mermaid:generate-diagram" => {
                let code = need("Mermaid code")?;
                if let Ok(o @ Value::Object(_)) = serde_json::from_str::<Value>(code) {
                    let code = o.get("code").and_then(|c| c.as_str()).ok_or("Mermaid code is missing")?.to_string();
                    let base = resolve(self, o.get("base"), results)?;
                    return self.generate_mermaid_in(&code, base, ms).map(Some);
                }
                return self.generate_mermaid(code, ms).map(Some);
            }
            "diagram-generator:overview" | "diagram-generator:overview-expanded" | "diagram-generator:type-hierarchy" | "diagram-generator:package-structure" => {
                use crate::generate::Generator;
                let g = match id {
                    "diagram-generator:overview" => Generator::Overview,
                    "diagram-generator:overview-expanded" => Generator::OverviewExpanded,
                    "diagram-generator:type-hierarchy" => Generator::TypeHierarchy,
                    _ => Generator::PackageStructure,
                };
                return Ok(self.generate_diagram(g, ms));
            }
            "diagram-layout:auto" | "diagram-layout:top-bottom" | "diagram-layout:bottom-top" | "diagram-layout:left-right" | "diagram-layout:right-left" => {
                use crate::layout::Dir;
                current(self)?;
                self.layout_diagram(match id {
                    "diagram-layout:bottom-top" => Dir::BottomTop,
                    "diagram-layout:left-right" => Dir::LeftRight,
                    "diagram-layout:right-left" => Dir::RightLeft,
                    _ => Dir::TopBottom,
                });
            }
            "uml:apply-profile.uml-standard" => self.apply_standard_profile(),
            "factory:create-diagram" => return self.factory_create_diagram(&opts, results).map(Some),
            "factory:create-model" => return self.factory_create_model(&opts, results).map(Some),
            "factory:create-model-and-view" => return self.factory_create_model_and_view(&opts, results).map(Some),
            "engine:set-property" => self.engine_set_property(&opts, results)?,
            "project:import-from-json" => return self.import_from_json(&opts, results).map(Some),
            "project:load-from-json" => {
                let data = opts.get("data").cloned().unwrap_or(opts);
                let repo = crate::model::Repo::parse(&data.to_string())?;
                self.set_project(repo, None);
            }
            _ => return Err(format!("Command not found: {id}")),
        }
        Ok(None)
    }

    /// `factory:create-diagram`: a diagram of type `id` in `parent` (the current selection or the
    /// project by default) (default-commands.js:1357-1362, factory.js:183-193).
    pub fn factory_create_diagram(&mut self, o: &Value, results: &[Option<Eid>]) -> Result<Eid, String> {
        let ty = o.get("diagramType").or_else(|| o.get("id")).and_then(|v| v.as_str()).unwrap_or("");
        if !crate::meta::is_kind_of(ty, "Diagram") {
            return Err(format!("Factory.createDiagram(): No such function of options.id: {ty}"));
        }
        let parent = resolve(self, o.get("parent"), results)?.or(self.focus.filter(|f| self.repo.alive(*f))).unwrap_or(self.repo.root);
        let ty = ty.to_string();
        let init = o.get("diagram-init").map(|v| refs_in(self, v, results));
        Ok(self.change(|a| {
            let d = a.add_diagram(Some(parent), &ty);
            a.assign_init(d, init.as_ref());
            d
        }))
    }

    /// `factory:create-model`: a model element of type `id` in `parent[field]` (field
    /// "ownedElements" by default; the parent defaults to the selection) (default-commands.js:1364-1368,
    /// factory.js:202-214, 466-505).
    pub fn factory_create_model(&mut self, o: &Value, results: &[Option<Eid>]) -> Result<Eid, String> {
        let ty = o.get("modelType").or_else(|| o.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
        if !crate::meta::known(&ty) || crate::meta::is_kind_of(&ty, "View") || crate::meta::is_kind_of(&ty, "Diagram") {
            return Err(format!("Factory.createModel(): No such function of options.id: {ty}"));
        }
        let parent = resolve(self, o.get("parent"), results)?.or(self.focus.filter(|f| self.repo.alive(*f))).ok_or("factory:create-model needs a parent")?;
        let field = o.get("field").and_then(|v| v.as_str()).map(str::to_string);
        let init = o.get("model-init").map(|v| refs_in(self, v, results));
        let generate = o.get("generate-name").and_then(|v| v.as_str()).map(str::to_string);
        Ok(self.change(|a| {
            let m = match field.as_deref() {
                None | Some("ownedElements") if !a.can_add_feature(parent, &ty) => a.add_model(parent, &ty),
                None => a.add_feature(parent, &ty).unwrap_or_else(|| a.add_model(parent, &ty)),
                Some(f) => {
                    let name = a.new_name(parent, &ty);
                    a.repo.create(&ty, &[("name", json!(name))], Some((parent, f)))
                }
            };
            if let Some(g) = generate {
                let name = a.new_name(parent, &g);
                a.repo.set(m, "name", json!(name));
            }
            a.assign_init(m, init.as_ref());
            m
        }))
    }

    /// `factory:create-model-and-view`: a model element and its view on `diagram` (the current diagram
    /// by default) from (x1, y1) to (x2, y2); a relationship joins `tailView` and `headView` (or the
    /// views of `tailModel` and `headModel` on the diagram). The element goes in the diagram's owner,
    /// as the command sets it (default-commands.js:1370-1375, factory.js:222-294, 517-590).
    pub fn factory_create_model_and_view(&mut self, o: &Value, results: &[Option<Eid>]) -> Result<Eid, String> {
        let id = o.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let d = resolve(self, o.get("diagram"), results)?.or(self.diagram.filter(|d| self.repo.alive(*d))).ok_or("There is no current diagram")?;
        if !self.is_diagram(d) {
            return Err("diagram is not a diagram".into());
        }
        let dty = self.repo.ty(d).to_string();
        let Some(tool) = crate::ui::tool_for(&dty, &id) else {
            return Err(format!("Factory.createModelAndView(): No such function of id: {id}"));
        };
        let num = |k: &str| o.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let (x1, y1, x2, y2) = (num("x1"), num("y1"), num("x2"), num("y2"));
        let model_init = o.get("model-init").map(|v| refs_in(self, v, results));
        let view_init = o.get("view-init").map(|v| refs_in(self, v, results));
        let view_on = |a: &App, key_view: &str, key_model: &str| -> Result<Option<Eid>, String> {
            if let Some(v) = resolve(a, o.get(key_view), results)? {
                return Ok(Some(v));
            }
            let Some(m) = resolve(a, o.get(key_model), results)? else { return Ok(None) };
            Ok(a.repo.list(d, "ownedViews").into_iter().find(|v| a.repo.refd(*v, "model") == Some(m)))
        };
        let prev = self.diagram;
        self.diagram = Some(d);
        let made: Result<Eid, String> = if tool.line {
            let tail = view_on(self, "tailView", "tailModel")?;
            let head = view_on(self, "headView", "headModel")?;
            match (tail, head) {
                (Some(t), Some(h)) => {
                    if !self.can_connect(&tool, t, h) {
                        Err(format!("Invalid connection ({})", tool.kind.trim_start_matches("UML")))
                    } else {
                        self.change(|a| a.create_edge(&tool, t, h)).ok_or_else(|| format!("Could not create {id}"))
                    }
                }
                _ => Err(format!("{id} needs tailView and headView")),
            }
        } else if tool.kind == "Frame" || !self.robustness_profile_ready(tool.kind) {
            Err(format!("Factory.createModelAndView(): No such function of id: {id}"))
        } else {
            let rect = if x2 - x1 < 10.0 || y2 - y1 < 10.0 {
                let (w, h) = crate::views::default_size(tool.kind);
                R { x: x1, y: y1, w, h }
            } else {
                R { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
            };
            let kind = tool.kind;
            if matches!(kind, "UMLPort" | "UMLPart" | "UMLInputPin" | "UMLOutputPin" | "UMLInputExpansionNode" | "UMLOutputExpansionNode" | "UMLConnectionPointReference") {
                self.change(|a| a.create_feature_box(kind, P { x: x1, y: y1 }, rect))
            } else {
                self.change(|a| a.create_node(kind, rect)).ok_or_else(|| format!("Could not create {id} on this diagram"))
            }
        };
        self.diagram = prev;
        let v = made?;
        let m = self.repo.refd(v, "model");
        if let Some(m) = m {
            self.assign_init(m, model_init.as_ref());
        }
        self.assign_init(v, view_init.as_ref());
        Ok(v)
    }

    /// `engine:set-property`: set `property` of `model` (or of the element the lodash path
    /// `set-model` leads to from it) to `value` (default-commands.js:1383-1387).
    pub fn engine_set_property(&mut self, o: &Value, results: &[Option<Eid>]) -> Result<(), String> {
        let mut e = resolve(self, o.get("model"), results)?.ok_or("engine:set-property needs a model")?;
        if let Some(path) = o.get("set-model").and_then(|v| v.as_str()) {
            e = self.follow_path(e, path)?;
        }
        let prop = o.get("property").and_then(|v| v.as_str()).ok_or("engine:set-property needs a property")?.to_string();
        if matches!(prop.as_str(), "_id" | "_parent" | "_type") || matches!(self.repo.e(e).fields.get(&prop), Some(crate::model::F::One(_) | crate::model::F::Many(_))) {
            return Err(format!("{prop} cannot be set"));
        }
        let value = refs_in(self, o.get("value").unwrap_or(&Value::Null), results);
        self.change(|a| a.repo.set(e, &prop, value));
        Ok(())
    }

    /// The element a lodash path such as `model`, `end1.reference` or `attributes[0]` leads to.
    fn follow_path(&self, from: Eid, path: &str) -> Result<Eid, String> {
        use crate::model::F;
        enum Cur {
            E(Eid),
            L(Vec<Eid>),
        }
        let mut cur = Cur::E(from);
        let steps = path.replace('[', ".").replace(']', "");
        let none = || format!("No element at {path}");
        for step in steps.split('.').filter(|s| !s.is_empty()) {
            cur = match cur {
                Cur::L(items) => Cur::E(step.parse::<usize>().ok().and_then(|i| items.get(i).copied()).ok_or_else(none)?),
                Cur::E(e) if step == "_parent" => Cur::E(self.repo.parent(e).ok_or_else(none)?),
                Cur::E(e) => match self.repo.e(e).fields.get(step) {
                    Some(F::One(c)) => Cur::E(*c),
                    Some(F::Many(v)) => Cur::L(v.clone()),
                    Some(F::V(Value::Array(_))) => Cur::L(self.repo.ref_list(e, step)),
                    Some(F::V(v)) => Cur::E(self.repo.by_ref(v).ok_or_else(none)?),
                    None => return Err(none()),
                },
            };
        }
        match cur {
            Cur::E(e) => Ok(e),
            Cur::L(_) => Err(none()),
        }
    }

    /// `project:import-from-json`: insert an element tree under `parent` (the project by default) in
    /// its owned elements, keeping the ids (project-manager.js:264-283 importFromJson). Ids already in
    /// the project are refused, so that two elements never share one.
    pub fn import_from_json(&mut self, o: &Value, results: &[Option<Eid>]) -> Result<Eid, String> {
        let (data, parent) = match o.get("data") {
            Some(d) => (d.clone(), resolve(self, o.get("parent"), results)?),
            None => (o.clone(), None),
        };
        if data.get("_type").and_then(|t| t.as_str()).is_none() {
            return Err("The data is not an element (no _type)".into());
        }
        fn ids(v: &Value, out: &mut Vec<String>) {
            match v {
                Value::Object(m) => {
                    if let (Some(Value::String(id)), true) = (m.get("_id"), m.contains_key("_type")) {
                        out.push(id.clone());
                    }
                    m.values().for_each(|x| ids(x, out));
                }
                Value::Array(a) => a.iter().for_each(|x| ids(x, out)),
                _ => {}
            }
        }
        let mut all = vec![];
        ids(&data, &mut all);
        if let Some(dup) = all.iter().find(|id| self.repo.by_id.get(*id).map(|e| self.repo.alive(*e)).unwrap_or(false)) {
            return Err(format!("An element with the id {dup} is already in the project"));
        }
        let parent = parent.unwrap_or(self.repo.root);
        let e = self.change(|a| a.repo.insert_json(data, parent, "ownedElements"));
        Ok(e)
    }
}
