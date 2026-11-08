# StarUML feature inventory and UMLCraft parity

Every StarUML feature we could find (menus, shortcuts, toolbox items, preferences, mouse and keyboard tricks, quick-edit behaviour, dialogs, file formats, export, command line, HTTP API, validation rules, bundled extensions and notations), each checked against the UMLCraft code.

## Sources

- **StarUML 7.1.1 as installed** (`/opt/StarUML`): the JavaScript in `resources/app.asar`, extracted with `@electron/asar`. Files 1–6 come from it. Source paths such as `src/ui/diagram-editor.js:2178` or `extensions/essential/uml/...` are relative to the extracted `app/` folder.
- **staruml.io and docs.staruml.io**, read on 2026-11-03 (file 7). The docs now describe **StarUML 8.0** (released 2026-11-01, rewritten in Rust). Features found only in V8 are tagged **[V8]**. File 7 overlaps the others on purpose: it is the documented view of the same app.
- **UMLCraft** at commit `9816ceb`. Every status names the UMLCraft code it rests on (`src/*.rs:line`).

## Legend

| Mark | Meaning |
|---|---|
| ✅ | Present in UMLCraft, close to StarUML's behaviour |
| 🟡 | Partial: some of it, or different; the note says what is missing |
| ❌ | Missing |
| ➖ | Not applicable: licensing, trial, watermarks, updates from staruml.io, Electron internals, or a StarUML bug not worth copying ("upstream bug") |

StarUML quirks and bugs are kept in the list and marked as such, so a port can decide on purpose whether to reproduce them.

## Files

| File | ✅ | 🟡 | ❌ | ➖ | Items |
|---|---|---|---|---|---|
| [Menus and keyboard shortcuts](1-menus-and-shortcuts.md) | 356 | 14 | 66 | 8 | 444 |
| [Toolbox items](2-toolbox.md) | 235 | 0 | 106 | 0 | 341 |
| [Preferences](3-preferences.md) | 112 | 0 | 27 | 1 | 140 |
| [Diagram canvas and editing](4-canvas.md) | 203 | 29 | 4 | 8 | 244 |
| [Panels, views and dialogs](5-panels-and-dialogs.md) | 146 | 24 | 31 | 18 | 219 |
| [Engine, files, export, CLI, validation and bundled extensions](6-engine-and-extensions.md) | 258 | 52 | 40 | 20 | 370 |
| [Website and documentation](7-website-and-docs.md) | 142 | 72 | 67 | 34 | 315 |
| **All StarUML features** | **1452** | **191** | **341** | **89** | **2073** |
| [Beyond StarUML](8-beyond-staruml.md): UMLCraft's own features, not counted above | 0 | 1 | 0 | 0 | 1 |

Recount after editing (prints one line per mark). Use `[1-7]` for the StarUML totals, and `8` for UMLCraft's own features:

```sh
grep -ohE '^(- .*UMLCraft:\*\* |\| .*\| )(✅|🟡|❌|➖)' docs/features/[1-7]-*.md | grep -oE '(✅|🟡|❌|➖)$' | sort | uniq -c
```
