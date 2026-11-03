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
| `packaging/` | Installer scripts and files for Windows, macOS, Linux and FreeBSD, and the app icon |
| `.github/workflows/release.yml` | Builds every installer on GitHub |
| `MECHANICS.md`, `mechanics.json` | The rules and numbers taken from the original, with their sources |

Build with `cargo build --release`; run with `cargo run --release -- [file.mdj]`.

## Installers

`.github/workflows/release.yml` builds every installer on GitHub:

| System | Files |
|---|---|
| Windows x64 and ARM64 | `.msi` installer (Start menu entry, opens `.mdj` files) and a portable `.zip` |
| macOS (Apple Silicon and Intel) | universal `.dmg` and `.zip` |
| Linux x86_64 and aarch64 | `.AppImage`, `.deb`, `.rpm` and a portable `.tar.gz` |
| FreeBSD x86_64 | `.pkg` (`pkg add`) and a portable `.tar.gz` |

Run it from the Actions tab ("Release", Run workflow) to build them as downloadable artifacts, or push
to the `release` branch or a `v1.2.3` tag to also publish them as a GitHub Release with a SHA256SUMS
file. The scripts in `packaging/` can also be run by hand on each system.

Signing is optional and turns on when these repository secrets exist: `APPLE_CERTIFICATE`,
`APPLE_CERTIFICATE_PASSWORD`, `KEYCHAIN_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`
(macOS signing and notarization) and `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` (Windows).
Without them, Windows shows an "unknown publisher" warning and macOS asks to confirm the first start
(right-click, Open). On macOS, double-clicking a `.mdj` file starts UMLCraft but does not open that file
yet; use File > Open.

## Was StarUML's code copied?

No. StarUML's JavaScript was read to learn its rules: the .mdj file layout, field names and defaults,
sizes such as the 5-pixel compartment padding, arrow shapes, colours and window layout. Those rules are
written down with their sources in `MECHANICS.md`, and every file here was written from scratch from that
sheet; no StarUML source file, function, icon or logo was pasted in. The icons are simple drawings made
for UMLCraft, and the fonts are the ones installed on the computer (Liberation Sans for diagrams).

## Version history

This folder is a git repository. Build output (`target/`) and test pictures (`shots/`) are not tracked.
