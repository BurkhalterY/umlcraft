# Agent guide

UMLCraft is a clean-room reimplementation of StarUML in Rust (egui). It reads and writes StarUML `.mdj` files. `README.md` describes the project and lists what each file in `src/` does. `MECHANICS.md` records the StarUML rules UMLCraft follows (file format, sizes, colours), each with its source.

## Build and test

```sh
cargo run --release -- examples/library.mdj
cargo test --release
```

On Linux, install the GTK 3, xkbcommon and OpenGL development packages first. On Debian and Ubuntu these are `libgtk-3-dev libxkbcommon-dev libgl1-mesa-dev`.

## The StarUML reference

On the development machine, StarUML 7.1.1 is installed in `/opt/StarUML`. Its JavaScript is the reference for every behaviour. To read it, extract it outside the repo:

```sh
npx -y @electron/asar extract /opt/StarUML/resources/app.asar <somewhere-outside-the-repo>/app
```

StarUML source paths in the docs (`src/ui/diagram-editor.js:2178`, `extensions/essential/uml/...`) are relative to that `app/` folder.

## Finding work: `docs/features/`

The work list is `docs/features/`, not GitHub issues. Do not open an issue for a missing feature.

It lists every StarUML feature that has been found, and each one has a UMLCraft status:

| Mark | Meaning |
|---|---|
| ✅ | Present in UMLCraft, close to StarUML's behaviour |
| 🟡 | Partial; the note says what is missing |
| ❌ | Missing |
| ➖ | Not applicable: licensing, trial, updates from staruml.io, Electron internals, or a StarUML bug not worth copying |

`docs/features/README.md` holds the legend and the count per file. The files are split by area:

| File | Area |
|---|---|
| `1-menus-and-shortcuts.md` | main menus, context menus and keymaps (tables) |
| `2-toolbox.md` | every toolbox item of every notation (table) |
| `3-preferences.md` | every preference key (table) |
| `4-canvas.md` | mouse, keyboard, selection, moving, edges, quick edit, clipboard, alignment |
| `5-panels-and-dialogs.md` | Model Explorer, Editors, Working Diagrams, status bar, dialogs |
| `6-engine-and-extensions.md` | files, export, print, CLI, HTTP API, validation rules, grammars, bundled extensions, notations |
| `7-website-and-docs.md` | what staruml.io and docs.staruml.io say; items tagged `[V8]` exist only in StarUML 8 |

There are two formats. In files 4–7 an item is a bullet, with the StarUML part first and the UMLCraft status after it:

```
- **Shift+click toggles**: Shift+left-click on a view adds it … (src/ui/diagram-editor.js:645-651) — **UMLCraft:** ✅ present Shift (or Ctrl) + click toggles membership (src/canvas.rs:514, 523-530)
```

In files 1–3 an item is a table row with a status column.

To pick work:

1. Read `docs/features/README.md`, choose a file, and look for ❌ and 🟡 items, for example with `grep -n '❌' docs/features/4-canvas.md`.
2. Prefer a coherent group, such as one `##` section or one feature together with its shortcut and menu entry, over scattered single lines. Class and use case diagrams come first, since UMLCraft only edits those so far.
3. Before writing code, read the StarUML source cited by the item, so that UMLCraft follows the real behaviour, not a guess. Note any rule worth keeping (sizes, defaults, key order) in `MECHANICS.md`, with its source.
4. An item marked as a StarUML bug or quirk is only copied on purpose. Say in the note which way you decided.

## Updating a status

Update the status in the same commit as the code.

- **Change only the UMLCraft part.** Leave the StarUML description and its source untouched. Change the mark and rewrite the note so it says what UMLCraft does, citing the code (`src/file.rs:line`). Keep 🟡 notes precise about what is still missing.
- **Update every copy.** One feature often appears in several places: a shortcut row, a main menu row, a context menu row, a canvas or panels bullet, and the website bullet. Search for the command id and the feature name across all of `docs/features/`, for example `grep -rn 'edit:copy\|Copy' docs/features/`, and update each one.
- **Refresh the citations you touch.** When an edit shifts lines in `src/`, the `src/*.rs:line` citations on the items you are updating must still point at the right code.
- **Recount.** Update the "Files" table in `docs/features/README.md` with the command shown below it.
- **New items.** If you find a StarUML feature that is not listed, add it to the right file and section in the same format, with its StarUML source.
- **Regressions.** If you remove or break something marked ✅, change its mark in the same commit.
