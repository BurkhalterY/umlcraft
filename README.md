# UMLCraft — source code guide

UMLCraft is written in **Rust**, like the other CraftApps. The window and widgets use **egui** (through
eframe, with an OpenGL renderer), and .mdj files are read and written with **serde_json**, keeping every
field in the original order. File dialogs come from **rfd** and PNG pictures are saved with **image**.

All of the source code is in this folder (`app/`):

| File | What it does |
|---|---|
| `Cargo.toml` | The project and its libraries |
| `src/main.rs` | Start-up: window, fonts, dark theme, a file to open, `--screenshot <file.png>` and `--resave <in> <out>` |
| `src/meta.rs` | The element kinds and fields of the .mdj file format, in file order, with defaults |
| `src/model.rs` | Reading, editing and writing .mdj project files (anything unknown is kept as it was) |
| `src/views.rs` | Building and sizing boxes, compartments, labels and lines |
| `src/render.rs` | Drawing diagrams (shapes, arrows, shadows) on screen and into SVG files |
| `src/canvas.rs` | The diagram canvas: select, move, resize, create, inline editing, zoom, mini-map |
| `src/parse.rs` | Reading typed lines such as `-id: int[1]` or `+pay(amount: int): boolean` |
| `src/app.rs` | Project state, undo/redo, open/save and model operations |
| `src/ui.rs` | Menus, Working Diagrams, Toolbox, Model Explorer, Editors, status bar |
| `src/icons.rs` | The small icons |
| `MECHANICS.md`, `mechanics.json` | The rules and numbers taken from the original, with their sources |

Build with `cargo build --release`; run with `cargo run --release -- [file.mdj]`.

## Was StarUML's code copied?

No. StarUML's JavaScript was read to learn its rules: the .mdj file layout, field names and defaults,
sizes such as the 5-pixel compartment padding, arrow shapes, colours and window layout. Those rules are
written down with their sources in `MECHANICS.md`, and every file here was written from scratch from that
sheet; no StarUML source file, function, icon or logo was pasted in. The icons are simple drawings made
for UMLCraft, and the fonts are the ones installed on the computer (Liberation Sans for diagrams).

## Version history

This folder is a git repository. Build output (`target/`) and test pictures (`shots/`) are not tracked.
