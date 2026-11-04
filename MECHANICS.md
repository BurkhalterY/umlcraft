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

* Word wrap: a view with `wordWrap` on breaks its labels between words to the label width, and its minimum width is the widest single word plus padding (`src/core/core.js:3705-3727, 3769-3790`). UMLCraft applies this to the name and the attribute, operation and literal rows of class-like boxes; notes and text wrap only while their `wordWrap` is on (default on).

* Robustness: a class stereotyped «boundary», «entity» or «control» with Stereotype Display Icon (or Icon with Label) is drawn as the UML robustness icon with its name below; the Robustness tools create such a class with attributes and operations suppressed (`extensions/default/robustness/main.js:17`). UMLCraft draws its own icons in a 50×36 area (estimated).

* Ball and socket: a dependency on an interface shown as a ball (icon display) is drawn as a solid line ending in a half circle around the ball, 5 px outside it, open towards the ball (UMLCraft's estimate of the documented socket notation, docs.staruml.io class-diagram).

* Component and deployment diagrams (UMLCraft's estimates of the standard UML shapes): a component is a box with the component icon (14×16 with two 8×4 tabs) at its top right; an artifact is a box with a page icon, or in icon display (the default, `uml.artifact.stereotypeDisplay`) a 30×40 page with the name under it; a node is a box with a 10 px depth above and to the right; a deployment is a dashed arrow labelled «deploy»; a communication path is drawn and edited like an association. Components and nodes start with attributes and operations suppressed (`extensions/essential/uml/preferences/uml.json`).

* Ownership dot: an association end with an `ownerAttribute` gets a filled dot of radius 3, 6 px from its tip along the line (UML 2.5 notation; size and place estimated).

## Exporting pictures

* JPEG is the PNG flattened on white; Export All writes `<folder>/<png|jpg|svg>/<filenamify(path)>_<index>.<ext>`, the path joining owner names with "::" and filenamify turning `<>:"/\|?*` and control characters into "!" (`src/engine/default-commands.js:452-535`, `src/engine/diagram-export.js:130-134, 275-298`).

| Rule | Value | Source |
|---|---|---|
| Margin around the drawing | the diagram's bounds grown by 10 on every side | `src/engine/diagram-export.js:33` BOUNDING_BOX_EXPAND, `src/core/graphics.js:381` expand |
| PNG size | grown bounds + 30 in width and height, at zoom 1, times the screen's pixel ratio | `diagram-export.js:113-127` getImageData |
| PNG background | transparent (white only for JPEG); no grid | `diagram-export.js:130-134` |
| SVG size and background | grown bounds, no extra, no background | `diagram-export.js:178-189` getSVGImageData |
| Native side | `toPNG` 0xec2c6e/0x101d688 and `image/png` are Electron API table entries only; no function to follow | native_strings_find ev_b82b90eb… |
| UMLCraft | draws the SVG off-screen with resvg for PNG export, so scrolled-out parts and the grid do not matter; trial watermarks of the original are not copied | `src/render.rs` to_png, to_svg_sized |

## Element Picker and type names

* A name typed into a `type` or `stereotype` property is looked up among elements of the field's type (UMLClassifier or UMLStereotype): first the owner's namespace, then each enclosing namespace, then the whole project; a match is stored as a reference, otherwise the text (`views/property-editor-view.js:604-622`, `core/repository.js:2353-2361`).
* The Element Picker ("Select Element") opens with the current value selected and its owners expanded, or "Do not specify" checked; a choice of the wrong type gives "Selected element (X) is not an instance of T" (`dialogs/element-picker-dialog.js:289-346`).

## Layout

| Rule | Value | Source |
|---|---|---|
| Menu | Format › Layout › Auto, Top to Bottom, Bottom to Top, Left to Right, Right to Left; Auto is top to bottom (estimated) | `extensions/default/diagram-layout/main.js:30-40` |
| Undo | the whole layout is one "layout diagram" step | `src/engine/engine.js:1513-1549` |
| UMLCraft | own layered layout instead of dagre: a line's head goes one rank before its tail; node, edge and rank separations 50, 10 and 50 (estimated); rectilinear lines are left to route themselves; boxes lying inside a package or subject move with it | `src/layout.rs` |

## Diagram Generator

* Tools › Diagram Generator works on the package selected in the Explorer ("To generate … diagram, select a Package." otherwise), adds the diagram in one undo step, lays it out and opens it. Overview: owned elements with compartments suppressed and interfaces as icons; Type Hierarchy: classes, interfaces, enumerations, signals and data types at any depth with only generalizations and realizations, left to right, separations node 10, edge 10, rank 100; Package Structure: nested packages joined by containment lines (`extensions/default/diagram-generator/main.js:58-303`). UMLCraft draws the containment's circle-plus at the parent package, radius 7 (estimated).

## Mermaid import

* Only `classDiagram` code is generated (the original also reads sequence, state, flowchart, ER, requirement and mindmap code, which UMLCraft cannot edit). The container is a new "Package by Mermaid" unless a package is the base; the diagram is "Class Diagram by Mermaid". Members are `Type name`; visibility `+ # - ~`, none is package. `<|--` generalization, `..|>` realization, `()--` lollipop realization, `o` shared, `*` composite, `<`/`>` navigable, other dashed links dependencies. Layout uses `direction` (default TB) with separations 50 (`extensions/default/mermaid/factory/class-factory.js:40-540`, `utils.js:1-21`).

## HTML Docs

* File › Export › HTML Docs writes `<folder>/html-docs`: `index.html` (frameset 25%/75%), `contents/navigation.html`, `contents/diagrams.html`, `contents/element_index.html`, `contents/<md5(_id)>.html` per element and `diagrams/<md5(_id)>.svg` per diagram; the diagram page's image map is offset by the SVG's 10 px margin; the navigation tree goes 8 levels deep (`extensions/default/html-export/main.js:159-224`, `templates/navigation.ejs`, `templates/content.ejs:42`). UMLCraft writes its own HTML and style sheet.

## Print to PDF

| Rule | Value | Source |
|---|---|---|
| Options and defaults | range all, layout landscape, size A4, show name on; saved to `print.options` on each change | `src/dialogs/print-dialog.js:33-76` |
| Page layout | one diagram per page, 30 pt margin, scale `min(pageW / bounds.x2, pageH / bounds.y2)` capped at 1 | `src/engine/diagram-export.js:318-362` |
| Caption | the diagram's path name, Helvetica 10, above the diagram | `diagram-export.js:318-362` |
| Page sizes | PDFKit's: 4A0, 2A0, A0–A10, B0–B10, C0–C10, RA0–RA4, SRA0–SRA4, Executive, Folio, Legal, Letter, Tabloid | `static/html-contents/print-dialog.html` |
| UMLCraft | writes the PDF itself, with each diagram as a JPEG at twice the printed size instead of vector shapes | `src/pdf.rs` |

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
| Shortcuts | Ctrl+N/O/S, Ctrl+Shift+S, Ctrl+Z/Y, Delete, Ctrl+Delete, Ctrl+A; Format toggles Ctrl+Shift+H/V/N/M/J/A/O/T | `resources/default/keymaps/linux.json` |

## HTTP API

* A local server (127.0.0.1, port 58321) starts with the window, not for command-line commands; `settings.json` keys `apiServer` and `apiServerPort` turn it off or move it. `GET /` greets; `POST /generate_diagram {code}`, `/get_all_diagrams_info`, `/get_current_diagram_info` (records `{id, type, name, description}`) and `/get_diagram_image_by_id {diagramId}` (unknown id: 500 "Diagram not found or invalid ID") (`src/main-process/api-server.js:12-154`, `main.js:33-38`). UMLCraft's image answer is `{success, data, mimeType}` (estimated) and its greeting names UMLCraft.

## Keymap

* `keymap.json` in the settings folder maps keystroke descriptors (`cmdctrl-alt-g`: modifiers then the key, case-insensitive, `cmdctrl` = Cmd on macOS and Ctrl elsewhere) to command ids; the user's bindings override the defaults for the same keystroke, and a bad file is logged and ignored (`ui/keymap-manager.js:49-54, 154-292`, `app-context.js:760-764`). UMLCraft's settings folder is `UMLCraft` instead of `StarUML`, and its command ids are the ones the Command Palette lists.

## Snapping

* While a moved or resized coordinate stays in the grid cell it started in, it keeps its original value; once it crosses into another cell it snaps to the nearest grid multiple (`src/ui/diagram-editor.js:1201-1323`). UMLCraft applies this to box moves and resizes, not to bend points.

## Editing commands

Read from the StarUML source through the feature inventory (`docs/features/`), which quotes these lines.

| Rule | Source |
|---|---|
| Ctrl/Cmd + left-drag pans (move cursor) and blocks every tool, so Ctrl+click never selects; middle-drag pans the same way | `src/ui/diagram-editor.js:2178-2193, 2236-2251` |
| Shift+click toggles a view and never starts a drag; a click without moving on a member of a group keeps only that view | `src/ui/diagram-editor.js:645-660, 901-912` |
| Right-click on empty canvas deselects everything | `src/ui/diagram-editor.js:701-704` |
| Rubber band: clamped at x, y ≥ 0; selects visible views whose bounding box overlaps it, a selected view counting 5 px larger; Select All takes the visible, enabled top-level views | `src/ui/diagram-editor.js:747-765`, `src/core/core.js:4162-4196`, `src/core/graphics.js:1011-1018` |
| Toolbox: a click on the active item locks it (padlock), a click on the locked one unlocks and returns to Select; Esc anywhere and opening or closing a group return to Select; the diagram type's group opens by itself; nothing shows without a diagram | `src/views/toolbox-view.js:245-310, 444-479` |
| Double-click on empty canvas: a Text "Text", 100 wide, at the click point, with its inline edit open | `src/app-context.js:1397-1402` |
| Dragging a line's label stores the new centre as alpha/distance from its anchor segment (the inverse of Coord.getPointAwayLine), so it follows the line | `src/ui/diagram-editor.js:2051-2106`, `src/engine/engine.js:754-795` |
| Tab selects the next top-level view in `ownedViews`, wrapping; none selected → the first; no reverse | `src/ui/diagram-manager.js:726-754` |
| Shift+arrow nudges by the grid size, or 1 with snap off | `src/ui/diagram-manager.js:761-810` |
| Format toggles (Show Shadow, Show Visibility/Namespace/Multiplicity/Operation Signature, Suppress Attributes/Operations/Literals): invert the merged value on the selected views that have the flag; a mixed selection gets `true` | `src/engine/default-commands.js:1013-1029`, `extensions/essential/uml/uml-commands.js:271-279, 4596-4651` |
| Clipboard: JSON text on the system clipboard (UMLCraft's own layout). Views paste as new views of the same models, only if every model still exists, centred on the screen centre and selected; a model pastes as a deep copy with new ids into the selected element, in the field it was copied from | `src/engine/clipboard-manager.js:68-237`, `src/engine/default-commands.js:614-774` |
| Inline edit keys: Ctrl+Enter adds an attribute (literal on an enumeration) or a sibling item, Ctrl+Shift+Enter an operation, Ctrl+Delete deletes the item, Ctrl+Up/Down reorders, Up/Down move between items; a new item un-suppresses its compartment and is edited at once. In notes Enter commits and Shift+Enter is a new line | `extensions/essential/uml/quickedits/uml.json`, `extensions/essential/uml/uml-commands.js:784-1024`, `src/ui/quickedit-manager.js:326-348` |
| Name, attribute and operation expressions take a leading `<<stereotype>>` (linked to a UMLStereotype of that name, else a string; none clears it) and a visibility sign; types naming a classifier are linked; operation parameters are updated by position and no parentheses removes them | `extensions/essential/uml/grammars/uml-*.pegjs`, `extensions/essential/uml/uml-commands.js:288-616` |
| Line editing: dragging an end reconnects it (same rules as creation; refused with the toast "X cannot be connected to Y"), updating source/target or end1/end2.reference; rectilinear segments move across; on release points within 5 px of a neighbour (oblique, curve) and points in a straight run are dropped; bend points clamp at 0 | `src/ui/diagram-editor.js:1742-2044`, `src/engine/engine.js:940-1024`, `extensions/essential/uml/main.js:44-82`, `src/core/graphics.js:737-773` |
| Hover cursors on the selected line: hand on its ends, move on bend points, ns/ew-resize on rectilinear segments, crosshair on other segments | `src/ui/diagram-editor.js:344-411` |
| Explorer drop: a hidden view of the model is shown again; otherwise a view at the drop point, with showNamespace when the model is owned elsewhere than the diagram, and lines for its relationships to every view of the other ends. A dropped relationship links the existing end views, creating missing ends at the drop point (second at y + 100), or alerts "Relationship View is already existed in this Diagram." | `src/app-context.js:1145-1186`, `src/engine/factory.js:871-1180` |
| Containment: while moving boxes with containerChangeable that share a container, the package or subject under the cursor that can hold them is outlined; on release they get it as containerView (listed in its containedViews), come to the front, and their models move into the package's model (or to the diagram's owner when dropped on empty canvas). Never into the box itself or something it contains. Contained views follow their container, recursively | `src/ui/diagram-editor.js:429-493, 1640-1657`, `src/core/core.js:1961-1986`, `src/engine/engine.js:633-654, 1036-1185`, `extensions/essential/uml/main.js:24-41` |
| Colours: Fill Color, Line Color and the Styles swatches start from the colour all selected views share (transparent swatch when mixed) and set it on every selected view that has the field, written `#rrggbb`. The Styles section comes before Properties and shows only with a view selection | `src/engine/default-commands.js:966-991`, `src/views/style-editor-view.js:229-278, 549-592, 666-668` |
| Font: Format > Font and the Styles face and size fields start from the merged values (blank when mixed) and set face and size on the selected views, keeping each view's style digit. Labels inside a view take its face and size (as parentStyle labels do), rows and labels are as tall as the size, and widths scale with it. Faces are stored but drawn with the Arial-compatible font | `src/engine/default-commands.js:940-964`, `src/core/core.js:1659-1666`, `src/views/style-editor-view.js:180-227` |
| Model Explorer: the header settings button opens Sort By Added / Sort By Name, Show Stereotype Text (on by default) and Expand All / Collapse All; quick search lists up to 99 names containing the text with " — parent", and picking one reveals it in the tree | `src/views/model-explorer-view.js:455-693` |
| Preferences are a JSON file in the per-user configuration folder (`~/.config/UMLCraft`, `~/Library/Application Support/UMLCraft`, `%APPDATA%\UMLCraft`), with the original's key names (`diagramEditor.showGrid`, `explorer.sortBy`, `_viewState.<panel>`, `_workingDiagrams.<full path>`); screenshots ignore it | `src/core/preference-manager.js`, `src/ui/diagram-manager.js:152-196` |
| New views take the view.* preferences (font face and size, fill, line and font colour, line style, shadow), then the uml.<kind>.* ones (fill colour, stereotype display, suppressed compartments); a value equal to the file default is not written | `resources/default/preferences/default.json`, `extensions/essential/uml/preferences/uml.json` |
| Opening a file opens every default diagram (the first one current), then the diagrams remembered for that file | `src/app-context.js:983-999` |
| Tag Editor: the stereotype's attributes (inherited ones too) come first, sorted by name, as slots; then the other tags by name. A tag stores only its kind's fields: checked (boolean), number, reference, value + options (enum), value (string). A slot's kind and default come from the attribute's type | `src/dialogs/tag-editor-dialog.js:56-153`, `src/dialogs/tag-value-editor-dialog.js:186-227` |
| Command Palette lists named commands matched by a case-insensitive substring of id or name and runs the pick after the palette closes; Quick Find lists up to 99 models by name, each followed by its views per diagram, and jumps to the pick | `src/ui/command-palette.js:46-137`, `src/ui/quick-find.js:46-160` |
| Validation runs after open and save while validation.validateModel is on; UMLCraft checks UML001-UML004, UML006-UML015 and UML017-UML020 (rules.js); results go to the Validation Results panel and turn the status-bar mark red | `extensions/essential/uml/rules.js:27-297`, `src/ui/validation-panel.js:43-142` |
| Add related element: the new box goes 70 px below (sub), 100 px above (super) or 70 px to the right (associated; actors to the left), or 10 px beyond the last box already joined that way on that side; coordinates ≥ 0; one undo step, then the new box's name edit opens | `src/ui/quickedit-manager.js:894-1036`, `extensions/essential/uml/uml-commands.js:1040-2104` |
| Alignment and sizing act on selected node views only; lines and labels are skipped; each command is one undo step | `extensions/default/alignment/main.js:26-34, 370-376` |
| Align Right/Bottom: right = left + width − 1, new left = max right − width + 1, so right edges meet (UMLCraft: right = left + width, same edge) | `extensions/default/alignment/main.js:96-115` |
| Align Center/Middle: centre on the midpoint of the selection's overall extent | `extensions/default/alignment/main.js:117-142` |
| Space Equally: sort by left (top), keep the outermost extent, equal gaps; no-op when the sizes fill the extent | `extensions/default/alignment/main.js:213-293` |
| Set Width/Height/Size Equally: the largest width and/or height | `extensions/default/alignment/main.js:295-368` |
| Zoom In/Out (±0.1) and Actual Size (1.0) keep the diagram point at the screen centre; Ctrl+wheel keeps the origin | `src/engine/default-commands.js:1063-1084`, `src/ui/diagram-editor.js:2273-2280` |
| Fit to Window: zoom = min(view w / box w, view h / box h) of the diagram bounding box, at most 1.0, then centre on the box (UMLCraft rounds down to a tenth) | `src/engine/default-commands.js:1086-1101` |
| Closing the current diagram: the one that slid into its index becomes current, else the first; closing any diagram clears the selection | `src/ui/diagram-manager.js:106-138`, `src/app-context.js:1310-1317` |
| Next/Previous Diagram wrap around the working diagrams (open order); Close Others keeps only the current one | `src/ui/diagram-manager.js:201-216, 271-292` |
| Select In Diagram: the model's views directly in diagrams; one → open, select, centre; several → picker "Select Diagram to show"; none → toast "No diagrams to show" | `src/engine/default-commands.js:877-907` |
| Scroll-to-view: centre on the view's bounding box, at the current zoom, clamped at 0 | `src/ui/diagram-manager.js:340-348, 484-490` |
| Each diagram keeps its scroll origin (`_originX/Y`, not saved in the file); a new one starts at (0,0) | `src/ui/diagram-editor.js:2384-2387`, `src/ui/diagram-manager.js:218-228` |
| Toasts stack upward inside the content area and hide after 5 s | `src/ui/toast-manager.js:36-72` |
| Bring to Front / Send to Back: move the selected top-level views to the end / start of `diagram.ownedViews` (drawing and hit order) | `extensions/default/alignment/main.js:36-73` |

## Estimated / not taken over

* Label text widths come from the system's Arial-compatible font (Liberation Sans), so widths can differ by a pixel.
* The menu bar is drawn inside the window (22 px) instead of by the system, as egui apps do.
* Only Class and Use Case diagrams have a toolbox; other diagram kinds are drawn as plain boxes and lines (estimated look).
* Rectilinear routing is a simple elbow (estimated); the original's router was not read.
* Template parameters are drawn in a dashed box whose right and top edges reach 10 px past the class's top-right corner (estimated; `views.rs` TEMPLATE_OUT); their text is `«stereotype» name: type = default`.
* Line styles 2 (rounded rectilinear) and 3 (curve) are drawn from the points: rounded corners of radius 10, and quadratic curves through the segment midpoints that bend towards each point (estimated, `views.rs` drawn_points).
