# UMLCraft mechanics sheet

UMLCraft is written in Rust with egui; the constants below appear under the same names in `src/`
(`views.rs` sizes, `render.rs` drawing and canvas, `ui.rs` window, `model.rs` file format, `app.rs` zoom).

Facts read from the original StarUML 7.1.1 (Electron 29) code in `resources/app.asar` (unpacked, readable JS) and
from a sample project the original saved itself (`checks/original/sample.mdj`, `sample2.mdj`).
Locations are `file:line` inside the unpacked asar, or a function address (`0x…`) in the native program
`/opt/StarUML/staruml`. Nothing here is pasted code; it is the rules, rewritten.

## Runtime (native program, decompiled)

`/opt/StarUML/staruml` (ELF x86-64, 176 MB) is the stock Electron 29.4.6 runtime (Chromium, V8, Node). The
decompiler finds no StarUML text in it: no `.mdj`, no `StarUML`, no menu or toolbox labels. Every modelling
rule below therefore comes from the JavaScript the runtime loads, which was read directly.

| Rule | Value | Source |
|---|---|---|
| Where the app's code is loaded from | first of `resources/app.asar`, `resources/app`, `resources/default_app.asar`; `appCodeLoaded` fires after | function 0x25398b0 (node bindings: `appSearchPaths`, `appSearchPathsOnlyLoadASAR`) |
| Sandbox helper | the file `chrome-sandbox` next to the program is used whenever it exists; if it is not root-owned with mode 4755 start-up aborts | function 0x743c260 (also reads `CHROME_DEVEL_SANDBOX`) |
| Turning the sandbox off | the environment variable `ELECTRON_DISABLE_SANDBOX` makes the runtime add its no-sandbox switch at start | function 0x23c1710 (electron_main_delegate, also `ELECTRON_ENABLE_STACK_DUMPING`) |
| Running as root | when the user id is 0 the runtime stops at start unless its no-sandbox switch is set; UMLCraft's launcher passes `--no-sandbox` as well as the variable, so this case starts too | function 0x23c1710 (getuid check after the variable; evidence ev_33d7ce65…) |
| Window background / page capture | `backgroundColor`, `paintWhenInitiallyHidden` and `capturePage` exist only as unreferenced names; the one referenced `BackgroundColor` belongs to Chromium's ScrollView widget, not to StarUML, so UMLCraft keeps the colour measured from the live window (#3a3f41, `ui.rs` LAYER0) | function 0x77149a0; string search ev_eb35a8c7… |
| Diagram font on Linux | the canvas asks for "Arial"; the runtime carries its own fontconfig (no `FcFontMatch` import) and takes the face fontconfig matches. On this system that is Liberation Sans in all four styles (`fc-match Arial`). UMLCraft resolves "Arial", "Arial:bold", "Arial:italic" and "Arial:bold:italic" the same way, falling back to Liberation Sans | string "Arial" 0xea6275 (only data-table references, e.g. 0xa000b8, 0xb0aae8); import search ev_d1e19b9a…; `src/main.rs` system_font |
| Files given at start-up | Linux and Windows pass the file on the command line (the original's menu entry runs `staruml %U` and registers no .mdj file type); macOS Finder sends an `open-file` event instead. UMLCraft reads the command line on every system; its installers register their own .mdj type (no clash, since the original registers none); the macOS `open-file` event is not received yet because winit has no such event | strings `open-file` 0xf14157, `second-instance` 0xf329ad, `requestSingleInstanceLock` 0xeabed9 (Electron API names, no StarUML code; evidence ev_ce76ef51…); `/usr/share/applications/staruml.desktop` |
| UMLCraft use | UMLCraft is a native Rust program (egui), so it needs no Electron runtime and no sandbox helper; the sandbox rules above describe the original only | — |

Ghidra could not analyse this 176 MB binary, so these readings come from the built-in decompiler (evidence
ev_9e3e4b9c…, ev_4a890849…, ev_a2f66936…; string searches ev_1a8da0e3… `.mdj` and ev_bf518789… `StarUML`
returned nothing).

## File format (.mdj)

| Rule | Value | Source |
|---|---|---|
| Encoding | UTF-8 JSON, pretty printed with one TAB per level | `src/core/repository.js:1586` (writeObject) |
| Root | a `Project` object; `documentVersion` 1 | `src/core/core.js:57` DOCUMENT_VERSION |
| Object key order | `_type` first, then fields in metamodel order, super-type fields first | `src/core/core.js:596` (Element.save) |
| Omitted fields | a field equal to its metamodel default; empty strings; null refs; empty arrays | `src/core/core.js:600-660` |
| Reference | `{"$ref": "<id>"}` | `src/core/repository.js:545` |
| Font | text `face;size;style` (style 0 normal, 1 bold, 2 italic, 3 bold italic), default `Arial;13;0` | `src/core/graphics.js:514` |
| Points | text `x:y;x:y;…` | `src/core/graphics.js:840` |
| Id | base64 of 14 bytes: 2 zero bytes, 8-byte ms timestamp, 2-byte counter, 2-byte random | `src/core/core.js:150` IdGenerator.generate; observed `AAAAAAGhohI4jHit75s=` |
| New project | Project "Untitled" › UMLModel "Model" › UMLClassDiagram "Main" (defaultDiagram) | `resources/templates/Default.mdj` |
| Unknown element kinds | kept exactly as read and written back unchanged (UMLCraft rule) | — |

## Where things go

| Rule | Source |
|---|---|
| New node model goes to `diagram._parent.ownedElements`, named `<Kind><n>` (Class1, Class2 …) | `src/engine/default-commands.js:1370` |
| Relationship model goes to the tail (source) model's `ownedElements` | `extensions/essential/uml/uml-factory.js:3519` |
| Directed relationship: `source` = tail model, `target` = head model | `src/engine/factory.js:720` |
| Association: `end1.reference` = tail, `end2.reference` = head; end1 decorates the tail | `src/engine/factory.js:635`, elements.js:5690 |
| Directed Association = end2 navigable; Aggregation = end2 shared; Composition = end2 composite | `extensions/essential/uml/toolbox/uml.json:52-87` |
| Views go to `diagram.ownedViews`; compartments/labels are `subViews` of their owner | sample.mdj |
| Classifier view sub-views in order: name, attribute, operation, reception (hidden), template-parameter (hidden) compartments | sample.mdj |
| Name compartment labels in order: stereotype, name (bold), namespace "(from X)", property (right aligned) | sample.mdj |
| Edge labels (general): name α=π/2 d=15, stereotype α=π/2 d=30, property α=−π/2 d=15, all at the middle | elements.js:5304 |
| Association end labels: tail role π/6·30, tail property π/4·40, tail multiplicity −π/6·25, head role −π/6·30, head property −π/4·40, head multiplicity π/6·25 | elements.js:5566 |
| Label placement: angle of the segment + α, `distance` away from its first point | `src/core/graphics.js:1207` Coord.getPointAwayLine |

## Drawing

| Name | Value | Source |
|---|---|---|
| Compartment padding (all sides) | 5 | elements.js:65-68 |
| Compartment item interval | 2 | elements.js:64 |
| Name compartment minimum height | 25 (sample) | sample.mdj |
| Empty list compartment height | 10 | sample.mdj |
| Shadow offset / alpha / colour | 7 / 0.2 / #C0C0C0 | elements.js:57-59 |
| Line, fill, font colour | #000000 / #ffffff / #000000 | `src/core/core.js:1475` |
| Package tab height / tab width | 15 / 2/5 of width | elements.js:87, 6760 |
| Package minimum size | 50×40 | elements.js:85-86 |
| Use case minimum | 70×32; ellipse; size ≥ √2 × name | elements.js:96-97, 8545 |
| Actor icon minimum | 26×55; head 16–84% wide, 0–34% high; arms at 46%; legs from 66% | elements.js:92-93, 8660 |
| Interface default display | icon (circle), attributes and operations hidden | sample.mdj, elements.js:5971 |
| Note folded corner | 10 | elements.js:14261 |
| Visibility symbols | public +, protected #, private -, package ~ | elements.js:375 |
| Attribute text | vis + name + `: Type` + `[mult]` + ` = default` | elements.js:957 |
| Operation text | vis + name(params) + `: Return` | elements.js:870 |
| Arrow end size | 11 long, ±22.5° (π/8) | `src/core/core.js:2738-2748` |
| Triangle / diamond length | 22 | `src/core/core.js:2749` |
| Dash pattern | [3] | `src/core/core.js` (lineMode DOT) |
| Edge end points | where the center-to-center line leaves each box grown by 1 | `src/core/core.js:2700`; sample points `359:110;221:110` |
| Default line style | 1 (oblique) | `resources/default/preferences/default.json:83` |
| Grid size / lines every / colour | 8 / 16 / #e0e0e0 | default.json:117-141 and screenshot |
| Canvas background | #ffffff, area outside #cccccc | default.json:135, live CSS |
| Zoom | 1.0, from 0.1 to 3, step 0.1 | `src/ui/diagram-editor.js:2396` |
| Selection colour / handle size | #4f99ff / 8 | `src/core/graphics.js:2723` |

## Window (dark theme, the default)

| Name | Value | Source |
|---|---|---|
| Window size | 1200×900 | `src/main-process/window.js:38` |
| Left sidebar width | 200 (Working Diagrams over Toolbox) | live layout |
| Right navigator width | 300 (Model Explorer over Editors) | live layout |
| Toolbar strip | 30 wide, #5d5f60 | live layout |
| Title strip / status bar | 14 / 27 high | live layout |
| Panel colours | #3a3f41 (explorer), #4a4c4e (toolbox/editors), #cccccc (diagram area) | live computed style |
| Text colour | #e3e3e3 | live computed style |
| UI font | 13px Source Sans Pro (UMLCraft uses an installed Source Sans Pro, else Noto Sans; diagrams use Liberation Sans, metric-compatible with Arial) | live computed style |
| Shortcuts | Ctrl+N/O/S, Ctrl+Shift+S, Ctrl+Z/Y, Delete, Ctrl+Delete, Ctrl+A | `resources/default/keymaps/linux.json` |

## Estimated / not taken over

* Label text widths come from the system's Arial-compatible font (Liberation Sans), so widths can differ by a pixel.
* The menu bar is drawn inside the window (22 px) instead of by the system, as egui apps do.
* Only Class and Use Case diagrams have a toolbox; other diagram kinds are drawn as plain boxes and lines (estimated look).
* Rectilinear routing is a simple elbow (estimated); the original's router was not read.
