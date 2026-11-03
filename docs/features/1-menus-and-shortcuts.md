# Menus and keyboard shortcuts

Part of the [StarUML feature inventory](README.md); the legend and how to update a status are there.

StarUML 7.1.1 main menus, context menus and keymaps, from every `menus/*.json` and `keymaps/*.json` (Linux) in the app. One row per item; a command can appear in several tables.

| Shortcut | Command | UMLCraft | Notes | Source |
|---|---|---|---|---|
| `shift-alt-t` | `debug:show-devtools` | ➖ | Electron devtools | extensions/default/debug/keymaps/debug.json |
| `ctrl-r` | `debug:reload` | ➖ | Electron reload | extensions/default/debug/keymaps/debug.json |
| `ctrl-alt-t` | `diagram-thumbnails:toggle` | ❌ |  | extensions/default/diagram-thumbnails/keymaps/diagram-thumbnails.json |
| `ctrl-f` | `find:find` | ✅ | Ctrl+F and Model › Find... open the Find dialog; results go to a Find Results panel (src/ui.rs:1582, 1134; src/find.rs:49) | extensions/default/find/keymaps/find.json |
| `ctrl-alt-d` | `markdown-doc:toggle` | ❌ |  | extensions/default/markdown/keymaps/markdown.json |
| `ctrl-alt-m` | `minimap:toggle` | 🟡 | minimap always drawn in the canvas corner (src/canvas.rs:426), cannot be toggled | extensions/default/minimap/keymaps/minimap.json |
| `ctrl-alt-r` | `relationship-view:toggle` | ❌ |  | extensions/default/relationship-view/keymaps/relationship-view.json |
| `ctrl-shift-u` | `erd:suppress-columns` | ❌ |  | extensions/essential/erd/keymaps/erd.json |
| `ctrl-shift-0` | `format:stereotype-none` | ✅ | sets stereotypeDisplay to none on the selected views; the stereotype is hidden (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-1` | `format:stereotype-label` | ✅ | sets stereotypeDisplay to label; the «stereotype» line is drawn (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-2` | `format:stereotype-decoration` | 🟡 | the value is set, but UMLCraft draws no stereotype decoration icon (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-3` | `format:stereotype-decoration-label` | 🟡 | the value is set and the label drawn, but no decoration icon (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-4` | `format:stereotype-icon` | 🟡 | the value is set; only interfaces have an icon (the ball) (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-5` | `format:stereotype-icon-label` | 🟡 | the value is set; only interfaces have an icon (the ball), and stereotype icons are not drawn (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-w` | `format:word-wrap` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-v` | `format:show-visibility` | 🟡 | Ctrl+Shift+V toggles showVisibility; egui turns Ctrl+Shift+V into a paste, so the key works only while the clipboard holds text (src/ui.rs:1456-1474; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-n` | `format:show-namespace` | ✅ | Ctrl+Shift+N toggles showNamespace on the selected views that have it (src/ui.rs:1456; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-b` | `format:show-property` | ✅ | Ctrl+Shift+B toggles showProperty, which shows property strings ({readOnly, ordered…}) of attributes and association ends, on the selected views that have it (src/ui.rs:1476; src/app.rs:1145-1159) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-y` | `format:show-type` | ✅ | Ctrl+Shift+Y toggles showType, which shows the types of attributes, parameters and return values, on the selected views that have it (src/ui.rs:1476; src/app.rs:1145-1159) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-m` | `format:show-multiplicity` | ✅ | Ctrl+Shift+M toggles showMultiplicity on the selected views that have it (src/ui.rs:1456; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-j` | `format:show-operation-signature` | ✅ | Ctrl+Shift+J toggles showOperationSignature on the selected views that have it (src/ui.rs:1456; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-d` | `format:show-diagram-name` | ❌ | no Show Diagram Name; in UMLCraft Ctrl+Shift+D is Open Sub-Diagram, from the default keymap (src/ui.rs:1538) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-a` | `format:suppress-attributes` | ✅ | Ctrl+Shift+A toggles suppressAttributes on the selected views that have it (src/ui.rs:1456; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-o` | `format:suppress-operations` | ✅ | Ctrl+Shift+O toggles suppressOperations on the selected views that have it (src/ui.rs:1456; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-e` | `format:suppress-receptions` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-t` | `format:suppress-literals` | ✅ | Ctrl+Shift+T toggles suppressLiterals on the selected views that have it (src/ui.rs:1456; src/app.rs:1131) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-n` | `application:new` | ✅ | File › New, Ctrl+N (src/ui.rs:1001, 1476) | resources/default/keymaps/linux.json |
| `ctrl-o` | `project:open` | ✅ | File › Open, Ctrl+O (src/ui.rs:1005, 1479) | resources/default/keymaps/linux.json |
| `ctrl-s` | `project:save` | ✅ | Ctrl+S (src/ui.rs:1021, 1485) | resources/default/keymaps/linux.json |
| `ctrl-shift-s` | `project:save-as` | ✅ | Ctrl+Shift+S (src/ui.rs:1025, 1482) | resources/default/keymaps/linux.json |
| `ctrl-q` | `application:quit` | ✅ | File › Exit, Ctrl+Q, with unsaved-changes prompt (src/ui.rs:1061, 1488, 1894) | resources/default/keymaps/linux.json |
| `ctrl-z` | `edit:undo` | ✅ | Ctrl+Z (src/ui.rs:1502) | resources/default/keymaps/linux.json |
| `ctrl-y` | `edit:redo` | ✅ | Ctrl+Y and Ctrl+Shift+Z (src/ui.rs:1499) | resources/default/keymaps/linux.json |
| `ctrl-x` | `edit:cut` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:1344, 1509) | resources/default/keymaps/linux.json |
| `ctrl-c` | `edit:copy` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:1344, 1509) | resources/default/keymaps/linux.json |
| `ctrl-shift-c` | `edit:copy-diagram-as-image` | ✅ | Ctrl+Shift+C and Edit menu: deselects, draws the diagram as PNG export does and puts the picture on the system clipboard, white background outside macOS (src/clipboard.rs:165; src/ui.rs:1354, 1520, 1678) | resources/default/keymaps/linux.json |
| `ctrl-v` | `edit:paste` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:1344, 1509) | resources/default/keymaps/linux.json |
| `delete` | `edit:delete` | ✅ | Delete key and Edit menu (src/canvas.rs:577, src/ui.rs:1077) | resources/default/keymaps/linux.json |
| `ctrl-delete` | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:1526) | resources/default/keymaps/linux.json |
| `ctrl-shift-up` | `edit:move-up` | ✅ | Ctrl+Shift+Up, Edit and explorer menus: swaps with the previous sibling of the same explorer group (src/app.rs:1002; src/ui.rs:1331, 1544) | resources/default/keymaps/linux.json |
| `ctrl-shift-down` | `edit:move-down` | ✅ | Ctrl+Shift+Down, Edit and explorer menus: swaps with the next sibling of the same explorer group (src/app.rs:1002; src/ui.rs:1331, 1544) | resources/default/keymaps/linux.json |
| `ctrl-a` | `edit:select-all` | ✅ | Ctrl+A (src/ui.rs:1529) | resources/default/keymaps/linux.json |
| `ctrl-e` | `edit:select-in-explorer` | ✅ | Ctrl+E (src/ui.rs:1532) | resources/default/keymaps/linux.json |
| `ctrl-d` | `edit:select-in-diagram` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1042; src/ui.rs:1541, 1096, 354; src/canvas.rs:1575) | resources/default/keymaps/linux.json |
| `ctrl-shift-d` | `edit:open-sub-diagram` | ✅ | Ctrl+Shift+D, Edit and canvas menus: opens the first diagram the element owns, or toasts "No sub-diagram in the selected element" (src/app.rs:1020; src/ui.rs:1100, 1538; src/canvas.rs:1580) | resources/default/keymaps/linux.json |
| `ctrl-shift-f` | `format:font` | ✅ | Ctrl+Shift+F and Format › Font... (main and canvas menus) open the Font window: face, size and colour pre-filled with the merged values. The face is stored but drawn with the Arial-compatible font (src/ui.rs:1447, 1108, 1793-1850; src/style.rs:74) | resources/default/keymaps/linux.json |
| `ctrl-shift-i` | `format:fill-color` | ✅ | Ctrl+Shift+I opens a colour window; Format › Fill Color (main and canvas menus) has a colour picker; both apply the merged colour to the selected views (src/ui.rs:1456, 1299, 1860; src/style.rs:35) | resources/default/keymaps/linux.json |
| `ctrl-shift-l` | `format:line-color` | ✅ | Ctrl+Shift+L opens a colour window; Format › Line Color has a colour picker (src/ui.rs:1456, 1299, 1860; src/style.rs:35) | resources/default/keymaps/linux.json |
| `ctrl-l` | `format:linestyle-rectilinear` | ✅ | Ctrl+L and a check item in Format › Line Style (main and canvas menus) (src/app.rs:1100; src/ui.rs:1314, 1559) | resources/default/keymaps/linux.json |
| `ctrl-b` | `format:linestyle-oblique` | ✅ | Ctrl+B and a check item in Format › Line Style (src/app.rs:1100; src/ui.rs:1314, 1562) | resources/default/keymaps/linux.json |
| `ctrl-alt-l` | `format:linestyle-roundrect` | ✅ | Ctrl+Alt+L and Format › Line Style › Rounded Rectilinear: routed like rectilinear, drawn with rounded corners (radius estimated) (src/ui.rs:1314, 1553; src/views.rs:912) | resources/default/keymaps/linux.json |
| `ctrl-alt-b` | `format:linestyle-curve` | ✅ | Ctrl+Alt+B and Format › Line Style › Curve: drawn as a smooth curve bending towards each point (curve shape estimated) (src/ui.rs:1314, 1556; src/views.rs:912) | resources/default/keymaps/linux.json |
| `ctrl-shift-r` | `format:auto-resize` | ✅ | Ctrl+Shift+R and a check item in Format (main and canvas menus) and Styles: inverts the shared autoResize, or sets it when mixed (src/ui.rs:1461, 1938; src/app.rs:1109-1140) | resources/default/keymaps/linux.json |
| `ctrl-shift-h` | `format:show-shadow` | ✅ | Ctrl+Shift+H and a check item in Format (main and canvas menus): inverts the shared value, or sets it when mixed (src/app.rs:1131, 1147; src/ui.rs:1565, 1399) | resources/default/keymaps/linux.json |
| `ctrl-shift-k` | `format:inherit-style` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-t` | `model:tag-editor` | ✅ | Ctrl+T, Model › Tag Editor..., and the canvas and explorer menus (src/ui.rs:1139, 1576; src/tags.rs:171-245) | resources/default/keymaps/linux.json |
| `ctrl-p` | `view:quick-find` | ✅ | Ctrl+P and View › Quick Find... (src/ui.rs:1156, 1579; src/palette.rs:127-162, 273) | resources/default/keymaps/linux.json |
| `ctrl-shift-p` | `view:command-palette` | ✅ | Ctrl+Shift+P and View › Command Palette... (src/ui.rs:1462; src/palette.rs:36-119, 172-271) | resources/default/keymaps/linux.json |
| `f4` | `view:close-diagram` | ✅ | F4, View menu and Working Diagrams menu; the diagram that slides into its place, or the first, becomes current (src/ui.rs:1610, 1365; src/app.rs:944) | resources/default/keymaps/linux.json |
| `ctrl-f4` | `view:close-other-diagrams` | ✅ | Ctrl+F4, View menu and Working Diagrams menu (src/ui.rs:1607, 1365; src/app.rs:966) | resources/default/keymaps/linux.json |
| `shift-f4` | `view:close-all-diagrams` | ✅ | Shift+F4, View menu and Working Diagrams menu (src/ui.rs:1597, 1365; src/app.rs:971) | resources/default/keymaps/linux.json |
| `ctrl-shift-]` | `view:next-diagram` | ✅ | Ctrl+Shift+] (also reported as Ctrl+Shift+}), wraps to the first (src/ui.rs:1614, 1168; src/app.rs:981) | resources/default/keymaps/linux.json |
| `ctrl-shift-[` | `view:previous-diagram` | ✅ | Ctrl+Shift+[ (also reported as Ctrl+Shift+{), wraps to the last (src/ui.rs:1617, 1172; src/app.rs:981) | resources/default/keymaps/linux.json |
| `ctrl-plus` | `view:zoom-in` | ✅ | Ctrl+= / Ctrl++, keeps the screen centre (src/ui.rs:1568; src/canvas.rs:1654) | resources/default/keymaps/linux.json |
| `ctrl-minus` | `view:zoom-out` | ✅ | Ctrl+-, keeps the screen centre (src/ui.rs:1571; src/canvas.rs:1654) | resources/default/keymaps/linux.json |
| `ctrl-0` | `view:actual-size` | ✅ | Ctrl+0, keeps the screen centre (src/ui.rs:1574; src/canvas.rs:1654) | resources/default/keymaps/linux.json |
| `ctrl-9` | `view:fit-to-window` | ✅ | Ctrl+9, centred on the bounding box (src/ui.rs:1586; src/canvas.rs:1666) | resources/default/keymaps/linux.json |
| `ctrl-g` | `view:show-grid` | ✅ | Ctrl+G (src/ui.rs:1593) | resources/default/keymaps/linux.json |
| `ctrl-shift-g` | `view:snap-to-grid` | ✅ | Ctrl+Shift+G, checked before Ctrl+G so it is not taken as Show Grid (src/ui.rs:1590) | resources/default/keymaps/linux.json |
| `ctrl-1` | `view:sidebar` | ✅ | View › Sidebar check item and Ctrl+1; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/keymaps/linux.json |
| `ctrl-2` | `view:navigator` | ✅ | View › Navigator check item and Ctrl+2; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/keymaps/linux.json |
| `ctrl-3` | `view:toolbar` | ✅ | View › Toolbar check item and Ctrl+3; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/keymaps/linux.json |
| `ctrl-4` | `view:statusbar` | ✅ | View › Statusbar check item and Ctrl+4; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/keymaps/linux.json |
| `ctrl-5` | `view:toolbox` | ✅ | View › Toolbox check item and Ctrl+5; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/keymaps/linux.json |
| `ctrl-6` | `view:editors` | ✅ | View › Editors check item and Ctrl+6; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/keymaps/linux.json |

## Main menu items

| Menu path | Command | Shortcut | UMLCraft | Notes | Source |
|---|---|---|---|---|---|
| Format › Alignment › Send To Back | `alignment:send-to-back` |  | ✅ | Format › Alignment › Send To Back moves the selected top-level views to the start of ownedViews (src/align.rs:146; src/ui.rs:1414) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Bring To Front | `alignment:bring-to-front` |  | ✅ | Format › Alignment › Bring To Front moves the selected top-level views to the end of ownedViews (src/align.rs:146; src/ui.rs:1414) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Left | `alignment:align-left` |  | ✅ | left of every selected node set to the minimum left (src/align.rs:55) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Right | `alignment:align-right` |  | ✅ | right edges set to the maximum right (src/align.rs:58) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Center | `alignment:align-center` |  | ✅ | nodes centred on the midpoint of the horizontal extent (src/align.rs:60) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Top | `alignment:align-top` |  | ✅ | top of every selected node set to the minimum top (src/align.rs:56) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Bottom | `alignment:align-bottom` |  | ✅ | bottom edges set to the maximum bottom (src/align.rs:59) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Middle | `alignment:align-middle` |  | ✅ | nodes centred on the midpoint of the vertical extent (src/align.rs:64) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Space Equally, Horizontally | `alignment:space-equally-horizontally` |  | ✅ | sorted by left, equal gaps inside the outer extent; nothing when they overlap (src/align.rs:68) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Space Equally, Vertically | `alignment:space-equally-vertically` |  | ✅ | sorted by top, equal gaps inside the outer extent; nothing when they overlap (src/align.rs:68) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Set Width Equally | `alignment:set-width-equally` |  | ✅ | largest width in the selection (src/align.rs:92) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Set Height Equally | `alignment:set-height-equally` |  | ✅ | largest height in the selection (src/align.rs:93) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Set Size Equally | `alignment:set-size-equally` |  | ✅ | largest width and height in the selection (src/align.rs:94) | extensions/default/alignment/menus/alignment.json |
| Debug › Show DevTools | `debug:show-devtools` | `shift-alt-t` | ➖ | Electron devtools | extensions/default/debug/menus/debug.json |
| Debug › Reload | `debug:reload` | `ctrl-r` | ➖ | Electron reload | extensions/default/debug/menus/debug.json |
| Tools › Diagram Generator › Overview | `diagram-generator:overview` |  | ❌ |  | extensions/default/diagram-generator/menus/diagram-generator.json |
| Tools › Diagram Generator › Overview Expanded | `diagram-generator:overview-expanded` |  | ❌ |  | extensions/default/diagram-generator/menus/diagram-generator.json |
| Tools › Diagram Generator › Type Hierarchy | `diagram-generator:type-hierarchy` |  | ❌ |  | extensions/default/diagram-generator/menus/diagram-generator.json |
| Tools › Diagram Generator › Package Structure | `diagram-generator:package-structure` |  | ❌ |  | extensions/default/diagram-generator/menus/diagram-generator.json |
| Format › Layout › Auto | `diagram-layout:auto` |  | ❌ |  | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Top to Bottom | `diagram-layout:top-bottom` |  | ❌ |  | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Bottom to Top | `diagram-layout:bottom-top` |  | ❌ |  | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Left to Right | `diagram-layout:left-right` |  | ❌ |  | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Right to Left | `diagram-layout:right-left` |  | ❌ |  | extensions/default/diagram-layout/menus/diagram-layout.json |
| View › Diagram Thumbnails | `diagram-thumbnails:toggle` | `ctrl-alt-t` | ❌ |  | extensions/default/diagram-thumbnails/menus/diagram-thumbnails.json |
| Model › Find | `find:find` | `ctrl-f` | ✅ | Ctrl+F and Model › Find... open the Find dialog; results go to a Find Results panel (src/ui.rs:1582, 1134; src/find.rs:49) | extensions/default/find/menus/find.json |
| File › Export › HTML Docs... | `html-export:export` |  | ❌ |  | extensions/default/html-export/menus/html-export.json |
| View › Markdown Documentation | `markdown-doc:toggle` | `ctrl-alt-d` | ❌ |  | extensions/default/markdown/menus/markdown.json |
| Tools › Generate Diagram by Mermaid... | `mermaid:show-mermaid-dialog` |  | ❌ |  | extensions/default/mermaid/menus/mermaid.json |
| View › Minimap | `minimap:toggle` | `ctrl-alt-m` | 🟡 | minimap always drawn in the canvas corner (src/canvas.rs:426), cannot be toggled | extensions/default/minimap/menus/minimap.json |
| View › Relationships | `relationship-view:toggle` | `ctrl-alt-r` | ❌ |  | extensions/default/relationship-view/menus/relationship-view.json |
| File › Import › StarUML 1 File (.uml)... | `staruml-v1:import` |  | ❌ |  | extensions/default/staruml-v1/menus/staruml-v1.json |
| File › New From Template › AWS Architecture | `aws:new-from-template` |  | ❌ | no templates | extensions/essential/aws/menus/aws.json |
| Model › Add Diagram › AWS Architecture Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/aws/menus/aws.json |
| Model › Add › AWS Model | `factory:create-model` |  | ❌ |  | extensions/essential/aws/menus/aws.json |
| File › New From Template › Azure Architecture | `azure:new-from-template` |  | ❌ | no templates | extensions/essential/azure/menus/azure.json |
| Model › Add Diagram › Azure Architecture Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/azure/menus/azure.json |
| Model › Add › Azure Model | `factory:create-model` |  | ❌ |  | extensions/essential/azure/menus/azure.json |
| File › New From Template › BPMN Business Process Modeling | `bpmn:new-from-template` |  | ❌ | no templates | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add Diagram › BPMN Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Business Process | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Collaboration | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Choreography | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Global Conversation | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Task | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Sub Process | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Gateway | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Start Event | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Intermediate Event (Throw) | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Intermediate Event (Catch) | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › End Event | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Compensate Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Cancel Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Error Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Link Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Signal Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Timer Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Escalation Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Message Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Terminate Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| Model › Add › Conditional Event Definition | `factory:create-model` |  | ❌ |  | extensions/essential/bpmn/menus/bpmn.json |
| File › New From Template › C4 Model | `c4:new-from-template` |  | ❌ | no templates | extensions/essential/c4/menus/c4.json |
| Model › Add Diagram › C4 Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/c4/menus/c4.json |
| Model › Add › C4 Model | `factory:create-model` |  | ❌ |  | extensions/essential/c4/menus/c4.json |
| Model › Add › C4 Person | `factory:create-model` |  | ❌ |  | extensions/essential/c4/menus/c4.json |
| Model › Add › C4 Software System | `factory:create-model` |  | ❌ |  | extensions/essential/c4/menus/c4.json |
| Model › Add › C4 Container | `factory:create-model` |  | ❌ |  | extensions/essential/c4/menus/c4.json |
| Model › Add › C4 Component | `factory:create-model` |  | ❌ |  | extensions/essential/c4/menus/c4.json |
| Model › Add › Tag | `common:tag` |  | ✅ | Model › Add › Tag and the explorer Add menu create a string tag "Tag<n>" (src/ui.rs:304-313) | extensions/essential/common/menus/common.json |
| Model › Add Diagram › Data Flow Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/dfd/menus/dfd.json |
| Model › Add › Data Flow Model | `factory:create-model` |  | ❌ |  | extensions/essential/dfd/menus/dfd.json |
| Model › Add › External Entity | `factory:create-model` |  | ❌ |  | extensions/essential/dfd/menus/dfd.json |
| Model › Add › Process | `factory:create-model` |  | ❌ |  | extensions/essential/dfd/menus/dfd.json |
| Model › Add › Data Store | `factory:create-model` |  | ❌ |  | extensions/essential/dfd/menus/dfd.json |
| File › New From Template › Data Model | `erd:new-from-template` |  | ❌ | no templates | extensions/essential/erd/menus/erd.json |
| Format › Suppress Columns | `erd:suppress-columns` | `ctrl-shift-u` | ❌ |  | extensions/essential/erd/menus/erd.json |
| Model › Add Diagram › ER Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/erd/menus/erd.json |
| Model › Add › Data Model | `factory:create-model` |  | ❌ |  | extensions/essential/erd/menus/erd.json |
| Model › Add › Entity | `factory:create-model` |  | ❌ |  | extensions/essential/erd/menus/erd.json |
| Model › Add › Column | `factory:create-model` |  | ❌ |  | extensions/essential/erd/menus/erd.json |
| Model › Add Diagram › Flowchart Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/flowchart/menus/flowchart.json |
| File › New From Template › GCP Architecture | `gcp:new-from-template` |  | ❌ | no templates | extensions/essential/gcp/menus/gcp.json |
| Model › Add Diagram › GCP Architecture Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/gcp/menus/gcp.json |
| Model › Add › GCP Model | `factory:create-model` |  | ❌ |  | extensions/essential/gcp/menus/gcp.json |
| Model › Add Diagram › Mindmap Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/mindmap/menus/mindmap.json |
| Model › Add › Mindmap | `factory:create-model` |  | ❌ |  | extensions/essential/mindmap/menus/mindmap.json |
| Model › Add Diagram › Requirement Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/sysml/menus/sysml.json |
| Model › Add Diagram › Block Definition Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/sysml/menus/sysml.json |
| Model › Add Diagram › Internal Block Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/sysml/menus/sysml.json |
| Model › Add Diagram › Parametric Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Requirement | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Block | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Value Type | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Interface Block | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Constraint Block | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Part | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Reference | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Value | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Port | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Property | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Constraint Property | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Flow Property | `factory:create-model` |  | ❌ |  | extensions/essential/sysml/menus/sysml.json |
| Model › Add › Parameter | `factory:create-model` |  | ✅ | explorer Add (operations), Editors button (src/ui.rs:278) | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Property Values | `format:suppress-property-values` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Constraints | `format:suppress-constraints` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Parts | `format:suppress-parts` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Ports | `format:suppress-ports` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress References | `format:suppress-references` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Values | `format:suppress-values` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Properties | `format:suppress-properties` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Flow Properties | `format:suppress-flow-properties` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| File › New From Template › UML Minimal | `uml:new-from-template` |  | ❌ | no templates | extensions/essential/uml/menus/uml.json |
| File › New From Template › UML Conventional | `uml:new-from-template` |  | ❌ | no templates | extensions/essential/uml/menus/uml.json |
| File › New From Template › 4+1 View Model | `uml:new-from-template` |  | ❌ | no templates | extensions/essential/uml/menus/uml.json |
| File › New From Template › Rational | `uml:new-from-template` |  | ❌ | no templates | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › None | `format:stereotype-none` | `ctrl-shift-0` | ✅ | sets stereotypeDisplay to none on the selected views; the stereotype is hidden (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Label | `format:stereotype-label` | `ctrl-shift-1` | ✅ | sets stereotypeDisplay to label; the «stereotype» line is drawn (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Decoration | `format:stereotype-decoration` | `ctrl-shift-2` | 🟡 | the value is set, but UMLCraft draws no stereotype decoration icon (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Decoration with Label | `format:stereotype-decoration-label` | `ctrl-shift-3` | 🟡 | the value is set and the label drawn, but no decoration icon (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Icon | `format:stereotype-icon` | `ctrl-shift-4` | 🟡 | the value is set; only interfaces have an icon (the ball) (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Icon with Label | `format:stereotype-icon-label` | `ctrl-shift-5` | 🟡 | the value is set; only interfaces have an icon (the ball), and stereotype icons are not drawn (src/ui.rs:1384, 1450; src/style.rs:110) | extensions/essential/uml/menus/uml.json |
| Format › Word Wrap | `format:word-wrap` | `ctrl-shift-w` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Show Visibility | `format:show-visibility` | `ctrl-shift-v` | ✅ | check item in Format (main and canvas menus): inverts the shared showVisibility of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Format › Show Namespace | `format:show-namespace` | `ctrl-shift-n` | ✅ | check item in Format (main and canvas menus): inverts the shared showNamespace of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Format › Show Property | `format:show-property` | `ctrl-shift-b` | ✅ | check item in Format (main and canvas menus): inverts the shared showProperty of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1262, 1412, 1968-1980; src/app.rs:1131-1159; src/views.rs:386-410) | extensions/essential/uml/menus/uml.json |
| Format › Show Type | `format:show-type` | `ctrl-shift-y` | ✅ | check item in Format (main and canvas menus): inverts the shared showType of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1262, 1412, 1968-1980; src/app.rs:1131-1159; src/views.rs:386-410) | extensions/essential/uml/menus/uml.json |
| Format › Show Multiplicity | `format:show-multiplicity` | `ctrl-shift-m` | ✅ | check item in Format (main and canvas menus): inverts the shared showMultiplicity of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Format › Show Operation Signature | `format:show-operation-signature` | `ctrl-shift-j` | ✅ | check item in Format (main and canvas menus): inverts the shared showOperationSignature of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Format › Show Diagram Name | `format:show-diagram-name` | `ctrl-shift-d` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Suppress Attributes | `format:suppress-attributes` | `ctrl-shift-a` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressAttributes of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Format › Suppress Operations | `format:suppress-operations` | `ctrl-shift-o` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressOperations of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Format › Suppress Receptions | `format:suppress-receptions` | `ctrl-shift-e` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Suppress Literals | `format:suppress-literals` | `ctrl-shift-t` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressLiterals of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Class Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:336) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Package Diagram | `factory:create-diagram` |  | ✅ | Model › Add Diagram and the explorer menu; the toolbox shows Packages and Annotations (src/ui.rs:116, 113) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Object Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Composite Structure Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Component Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Deployment Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Use Case Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:336) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Sequence Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Communication Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Timing Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Interaction Overview Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Information Flow Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Statechart Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Activity Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Profile Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add › Template Parameter | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu on a classifier; drawn in a dashed box over its top-right corner (src/ui.rs:298; src/views.rs:799) | extensions/essential/uml/menus/uml.json |
| Model › Add › Template Parameter Substitution | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Enumeration Literal | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Attribute | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Operation | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Reception | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Extension Point | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Slot | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Interaction Operand | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Model | `factory:create-model` |  | ✅ | explorer Add (Project only) (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Subsystem | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Package | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Profile | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:279-300) | extensions/essential/uml/menus/uml.json |
| Model › Add › Class | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Interface | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Signal | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:279-300) | extensions/essential/uml/menus/uml.json |
| Model › Add › DataType | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › PrimitiveType | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:279-300) | extensions/essential/uml/menus/uml.json |
| Model › Add › Enumeration | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Artifact | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Component | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Node | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › UseCase | `factory:create-model` |  | ✅ | explorer Add (Use Case) (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Actor | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Stereotype | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:279-300) | extensions/essential/uml/menus/uml.json |
| Model › Add › Information Item | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Interaction | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › State Machine | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Activity | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Opaque Behavior | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › State | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Region | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Entry Activity | `uml:create-model.entry-activity` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Do Activity | `uml:create-model.do-activity` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Exit Activity | `uml:create-model.exit-activity` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Trigger | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Effect | `uml:create-model.effect` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Action | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Constraint | `uml:create-model.constraint` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Apply Profile › UML Standard Profile (v2) | `uml:apply-profile.uml-standard` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| File › New From Template › Wireframe Model | `wireframe:new-from-template` |  | ❌ | no templates | extensions/essential/wireframe/menus/wireframe.json |
| Model › Add Diagram › Wireframe Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/wireframe/menus/wireframe.json |
| Model › Add › Wireframe | `factory:create-model` |  | ❌ |  | extensions/essential/wireframe/menus/wireframe.json |
| File › New | `application:new` | `ctrl-n` | ✅ | File › New, Ctrl+N (src/ui.rs:1001, 1476) | resources/default/menus/linux.json |
| File › New From Template | `application:new-from-template` |  | ❌ | no templates | resources/default/menus/linux.json |
| File › Open... | `project:open` | `ctrl-o` | ✅ | File › Open, Ctrl+O (src/ui.rs:1005, 1479) | resources/default/menus/linux.json |
| File › Open Recent |  |  | ✅ | the 10 most recent files, kept in the preferences file (src/ui.rs:1011; src/prefs.rs:118-130) | resources/default/menus/linux.json |
| File › Save | `project:save` | `ctrl-s` | ✅ | Ctrl+S (src/ui.rs:1021, 1485) | resources/default/menus/linux.json |
| File › Save As... | `project:save-as` | `ctrl-shift-s` | ✅ | Ctrl+Shift+S (src/ui.rs:1025, 1482) | resources/default/menus/linux.json |
| File › Close | `project:close` |  | 🟡 | File › Close starts a new blank project instead of closing (src/ui.rs:1030) | resources/default/menus/linux.json |
| File › Import › Fragment... | `project:import-fragment` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export › Fragment... | `project:export-fragment` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export Diagram As › PNG... | `project:export-diagram-to-png` |  | ✅ | File › Export Diagram As › PNG... and the toolbar button (src/ui.rs:1035-1050, 1683-1698; src/export.rs:42) | resources/default/menus/linux.json |
| File › Export Diagram As › JPEG... | `project:export-diagram-to-jpeg` |  | ✅ | File › Export Diagram As › JPEG..., on a white background (src/ui.rs:1035-1050, 1683; src/export.rs:42-61) | resources/default/menus/linux.json |
| File › Export Diagram As › SVG... | `project:export-diagram-to-svg` |  | ✅ | File › Export Diagram As › SVG... (src/ui.rs:1035-1050, 1683; src/export.rs:42) | resources/default/menus/linux.json |
| File › Export Diagram As › All to PNGs... | `project:export-diagram-all-to-pngs` |  | ✅ | every diagram into <folder>/png/ as filenamify(path)_<index> (src/ui.rs:1701-1714; src/export.rs:73-98) | resources/default/menus/linux.json |
| File › Export Diagram As › All to JPEGs... | `project:export-diagram-all-to-jpegs` |  | ✅ | every diagram into <folder>/jpg/ as filenamify(path)_<index> (src/ui.rs:1701-1714; src/export.rs:73-98) | resources/default/menus/linux.json |
| File › Export Diagram As › All to SVGs... | `project:export-diagram-all-to-svgs` |  | ✅ | every diagram into <folder>/svg/ as filenamify(path)_<index> (src/ui.rs:1701-1714; src/export.rs:73-98) | resources/default/menus/linux.json |
| File › Preferences... | `application:preferences` |  | ✅ | File › Preferences... opens the Preferences window (src/ui.rs:1056; src/prefs.rs:316) | resources/default/menus/linux.json |
| File › Print to PDF... | `project:print-to-pdf` |  | ✅ | opens the Print to PDF dialog (src/ui.rs:1051-1052; src/pdf.rs:210) | resources/default/menus/linux.json |
| File › Exit |  |  | ✅ | File › Exit, Ctrl+Q, with unsaved-changes prompt (src/ui.rs:1061, 1488, 1894) | resources/default/menus/linux.json |
| Edit › Undo | `edit:undo` | `ctrl-z` | ✅ | Ctrl+Z (src/ui.rs:1502) | resources/default/menus/linux.json |
| Edit › Redo | `edit:redo` | `ctrl-y` | ✅ | Ctrl+Y and Ctrl+Shift+Z (src/ui.rs:1499) | resources/default/menus/linux.json |
| Edit › Cut | `edit:cut` | `ctrl-x` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:1344, 1509) | resources/default/menus/linux.json |
| Edit › Copy | `edit:copy` | `ctrl-c` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:1344, 1509) | resources/default/menus/linux.json |
| Edit › Copy Diagram As Image | `edit:copy-diagram-as-image` | `ctrl-shift-c` | ✅ | Ctrl+Shift+C and Edit menu: deselects, draws the diagram as PNG export does and puts the picture on the system clipboard, white background outside macOS (src/clipboard.rs:165; src/ui.rs:1354, 1520, 1678) | resources/default/menus/linux.json |
| Edit › Paste | `edit:paste` | `ctrl-v` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:1344, 1509) | resources/default/menus/linux.json |
| Edit › Delete | `edit:delete` | `delete` | ✅ | Delete key and Edit menu (src/canvas.rs:577, src/ui.rs:1077) | resources/default/menus/linux.json |
| Edit › Delete From Model | `edit:delete-from-model` | `ctrl-delete` | ✅ | Ctrl+Delete (src/ui.rs:1526) | resources/default/menus/linux.json |
| Edit › Move Up | `edit:move-up` | `ctrl-shift-up` | ✅ | Ctrl+Shift+Up, Edit and explorer menus: swaps with the previous sibling of the same explorer group (src/app.rs:1002; src/ui.rs:1331, 1544) | resources/default/menus/linux.json |
| Edit › Move Down | `edit:move-down` | `ctrl-shift-down` | ✅ | Ctrl+Shift+Down, Edit and explorer menus: swaps with the next sibling of the same explorer group (src/app.rs:1002; src/ui.rs:1331, 1544) | resources/default/menus/linux.json |
| Edit › Select All | `edit:select-all` | `ctrl-a` | ✅ | Ctrl+A (src/ui.rs:1529) | resources/default/menus/linux.json |
| Edit › Select In Explorer | `edit:select-in-explorer` | `ctrl-e` | ✅ | Ctrl+E (src/ui.rs:1532) | resources/default/menus/linux.json |
| Edit › Select In Diagram | `edit:select-in-diagram` | `ctrl-d` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1042; src/ui.rs:1541, 1096, 354; src/canvas.rs:1575) | resources/default/menus/linux.json |
| Edit › Open Sub-Diagram | `edit:open-sub-diagram` | `ctrl-shift-d` | ✅ | Ctrl+Shift+D, Edit and canvas menus: opens the first diagram the element owns, or toasts "No sub-diagram in the selected element" (src/app.rs:1020; src/ui.rs:1100, 1538; src/canvas.rs:1580) | resources/default/menus/linux.json |
| Format › Font... | `format:font` | `ctrl-shift-f` | ✅ | Ctrl+Shift+F and Format › Font... (main and canvas menus) open the Font window: face, size and colour pre-filled with the merged values. The face is stored but drawn with the Arial-compatible font (src/ui.rs:1447, 1108, 1793-1850; src/style.rs:74) | resources/default/menus/linux.json |
| Format › Fill Color... | `format:fill-color` | `ctrl-shift-i` | ✅ | Ctrl+Shift+I opens a colour window; Format › Fill Color (main and canvas menus) has a colour picker; both apply the merged colour to the selected views (src/ui.rs:1456, 1299, 1860; src/style.rs:35) | resources/default/menus/linux.json |
| Format › Line Color... | `format:line-color` | `ctrl-shift-l` | ✅ | Ctrl+Shift+L opens a colour window; Format › Line Color has a colour picker (src/ui.rs:1456, 1299, 1860; src/style.rs:35) | resources/default/menus/linux.json |
| Format › Line Style › Rectilinear | `format:linestyle-rectilinear` | `ctrl-l` | ✅ | Ctrl+L and a check item in Format › Line Style (main and canvas menus) (src/app.rs:1100; src/ui.rs:1314, 1559) | resources/default/menus/linux.json |
| Format › Line Style › Oblique | `format:linestyle-oblique` | `ctrl-b` | ✅ | Ctrl+B and a check item in Format › Line Style (src/app.rs:1100; src/ui.rs:1314, 1562) | resources/default/menus/linux.json |
| Format › Line Style › RoundRect | `format:linestyle-roundrect` | `ctrl-alt-l` | ✅ | Ctrl+Alt+L and Format › Line Style › Rounded Rectilinear: routed like rectilinear, drawn with rounded corners (radius estimated) (src/ui.rs:1314, 1553; src/views.rs:912) | resources/default/menus/linux.json |
| Format › Line Style › Curve | `format:linestyle-curve` | `ctrl-alt-b` | ✅ | Ctrl+Alt+B and Format › Line Style › Curve: drawn as a smooth curve bending towards each point (curve shape estimated) (src/ui.rs:1314, 1556; src/views.rs:912) | resources/default/menus/linux.json |
| Format › Auto Resize | `format:auto-resize` | `ctrl-shift-r` | ✅ | Ctrl+Shift+R and a check item in Format (main and canvas menus) and Styles: inverts the shared autoResize, or sets it when mixed (src/ui.rs:1461, 1938; src/app.rs:1109-1140) | resources/default/menus/linux.json |
| Format › Show Shadow | `format:show-shadow` | `ctrl-shift-h` | ✅ | Ctrl+Shift+H and a check item in Format (main and canvas menus): inverts the shared value, or sets it when mixed (src/app.rs:1131, 1147; src/ui.rs:1565, 1399) | resources/default/menus/linux.json |
| Format › Inherit Style | `format:inherit-style` | `ctrl-shift-k` | ❌ |  | resources/default/menus/linux.json |
| Model › Add Diagram | `model:add-diagram` |  | 🟡 | Model › Add Diagram and the explorer menu offer Class, Package and Use Case diagrams, the kinds UMLCraft edits (src/ui.rs:116, 335, 1125) | resources/default/menus/linux.json |
| Model › Add | `model:add` |  | 🟡 | Model › Add and the explorer Add menu offer Model, Package, Profile, Class, Interface, Signal, DataType, PrimitiveType, Enumeration, Stereotype, Actor, Use Case and the features that apply (src/ui.rs:279-324) | resources/default/menus/linux.json |
| Model › Tag Editor... | `model:tag-editor` | `ctrl-t` | ✅ | Ctrl+T, Model › Tag Editor..., and the canvas and explorer menus (src/ui.rs:1139, 1576; src/tags.rs:171-245) | resources/default/menus/linux.json |
| Tools › Extension Manager... | `tools:extension-manager` |  | ➖ | UMLCraft has no extension system | resources/default/menus/linux.json |
| View › Quick Find... | `view:quick-find` | `ctrl-p` | ✅ | Ctrl+P and View › Quick Find... (src/ui.rs:1156, 1579; src/palette.rs:127-162, 273) | resources/default/menus/linux.json |
| View › Command Palette... | `view:command-palette` | `ctrl-shift-p` | ✅ | Ctrl+Shift+P and View › Command Palette... (src/ui.rs:1462; src/palette.rs:36-119, 172-271) | resources/default/menus/linux.json |
| View › Close Diagram | `view:close-diagram` | `f4` | ✅ | F4, View menu and Working Diagrams menu; the diagram that slides into its place, or the first, becomes current (src/ui.rs:1610, 1365; src/app.rs:944) | resources/default/menus/linux.json |
| View › Close Other Diagrams | `view:close-other-diagrams` | `ctrl-f4` | ✅ | Ctrl+F4, View menu and Working Diagrams menu (src/ui.rs:1607, 1365; src/app.rs:966) | resources/default/menus/linux.json |
| View › Close All Diagrams | `view:close-all-diagrams` | `shift-f4` | ✅ | Shift+F4, View menu and Working Diagrams menu (src/ui.rs:1597, 1365; src/app.rs:971) | resources/default/menus/linux.json |
| View › Next Diagram | `view:next-diagram` | `ctrl-shift-]` | ✅ | Ctrl+Shift+] (also reported as Ctrl+Shift+}), wraps to the first (src/ui.rs:1614, 1168; src/app.rs:981) | resources/default/menus/linux.json |
| View › Previous Diagram | `view:previous-diagram` | `ctrl-shift-[` | ✅ | Ctrl+Shift+[ (also reported as Ctrl+Shift+{), wraps to the last (src/ui.rs:1617, 1172; src/app.rs:981) | resources/default/menus/linux.json |
| View › Zoom In | `view:zoom-in` | `ctrl-plus` | ✅ | Ctrl+= / Ctrl++, keeps the screen centre (src/ui.rs:1568; src/canvas.rs:1654) | resources/default/menus/linux.json |
| View › Zoom Out | `view:zoom-out` | `ctrl-minus` | ✅ | Ctrl+-, keeps the screen centre (src/ui.rs:1571; src/canvas.rs:1654) | resources/default/menus/linux.json |
| View › Actual Size | `view:actual-size` | `ctrl-0` | ✅ | Ctrl+0, keeps the screen centre (src/ui.rs:1574; src/canvas.rs:1654) | resources/default/menus/linux.json |
| View › Fit To Window | `view:fit-to-window` | `ctrl-9` | ✅ | Ctrl+9, centred on the bounding box (src/ui.rs:1586; src/canvas.rs:1666) | resources/default/menus/linux.json |
| View › Show Grid | `view:show-grid` | `ctrl-g` | ✅ | Ctrl+G (src/ui.rs:1593) | resources/default/menus/linux.json |
| View › Snap to Grid | `view:snap-to-grid` | `ctrl-shift-g` | ✅ | Ctrl+Shift+G, checked before Ctrl+G so it is not taken as Show Grid (src/ui.rs:1590) | resources/default/menus/linux.json |
| View › Sidebar | `view:sidebar` | `ctrl-1` | ✅ | View › Sidebar check item and Ctrl+1; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/menus/linux.json |
| View › Navigator | `view:navigator` | `ctrl-2` | ✅ | View › Navigator check item and Ctrl+2; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/menus/linux.json |
| View › Toolbar | `view:toolbar` | `ctrl-3` | ✅ | View › Toolbar check item and Ctrl+3; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/menus/linux.json |
| View › Statusbar | `view:statusbar` | `ctrl-4` | ✅ | View › Statusbar check item and Ctrl+4; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/menus/linux.json |
| View › Toolbox | `view:toolbox` | `ctrl-5` | ✅ | View › Toolbox check item and Ctrl+5; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/menus/linux.json |
| View › Editors | `view:editors` | `ctrl-6` | ✅ | View › Editors check item and Ctrl+6; kept in the preferences file (src/ui.rs:1194-1199, 1600-1606; src/prefs.rs:84-90) | resources/default/menus/linux.json |
| Window › Minimize |  |  | 🟡 | left to the window manager | resources/default/menus/linux.json |
| Window › Full Screen |  |  | 🟡 | left to the window manager | resources/default/menus/linux.json |
| Window › Bring All to Front |  |  | 🟡 | left to the window manager | resources/default/menus/linux.json |
| Help › About StarUML | `help:about` |  | ✅ | Help › About UMLCraft (src/ui.rs:1201) | resources/default/menus/linux.json |
| Help › Check For Updates... | `help:check-for-updates` |  | ➖ | StarUML licence/update server | resources/default/menus/linux.json |
| Help › License Activation... | `help:license-activation` |  | ➖ | StarUML licensing | resources/default/menus/linux.json |
| Help › Documentation... | `help:documentation` |  | ❌ |  | resources/default/menus/linux.json |
| Help › Forum... | `help:forum` |  | ➖ |  | resources/default/menus/linux.json |

## Context menus

| Context | Item | Command | UMLCraft | Notes |
|---|---|---|---|---|
| diagram-canvas | *(the menu as a whole)* |  | 🟡 | the canvas menu has Cut, Copy, Copy Diagram As Image, Paste, Edit, Add Literal/Attribute/Operation, Line Style (four styles), Format (show/suppress toggles and Alignment), Select In Explorer, Select In Diagram, Open Sub-Diagram, Delete, Delete from Model; on empty canvas Paste, Select All and Fit To Window. Missing: the font, colour, stereotype-display, auto-resize and add-related items (src/canvas.rs:1519) |
| diagram-canvas | Format › Alignment › Send To Back | `alignment:send-to-back` | ✅ | Format › Alignment › Send To Back moves the selected top-level views to the start of ownedViews (src/align.rs:146; src/ui.rs:1414) |
| diagram-canvas | Format › Alignment › Bring To Front | `alignment:bring-to-front` | ✅ | Format › Alignment › Bring To Front moves the selected top-level views to the end of ownedViews (src/align.rs:146; src/ui.rs:1414) |
| diagram-canvas | Format › Alignment › Align Left | `alignment:align-left` | ✅ | left of every selected node set to the minimum left (src/align.rs:55) |
| diagram-canvas | Format › Alignment › Align Right | `alignment:align-right` | ✅ | right edges set to the maximum right (src/align.rs:58) |
| diagram-canvas | Format › Alignment › Align Center | `alignment:align-center` | ✅ | nodes centred on the midpoint of the horizontal extent (src/align.rs:60) |
| diagram-canvas | Format › Alignment › Align Top | `alignment:align-top` | ✅ | top of every selected node set to the minimum top (src/align.rs:56) |
| diagram-canvas | Format › Alignment › Align Bottom | `alignment:align-bottom` | ✅ | bottom edges set to the maximum bottom (src/align.rs:59) |
| diagram-canvas | Format › Alignment › Align Middle | `alignment:align-middle` | ✅ | nodes centred on the midpoint of the vertical extent (src/align.rs:64) |
| diagram-canvas | Format › Alignment › Space Equally, Horizontally | `alignment:space-equally-horizontally` | ✅ | sorted by left, equal gaps inside the outer extent; nothing when they overlap (src/align.rs:68) |
| diagram-canvas | Format › Alignment › Space Equally, Vertically | `alignment:space-equally-vertically` | ✅ | sorted by top, equal gaps inside the outer extent; nothing when they overlap (src/align.rs:68) |
| diagram-canvas | Format › Alignment › Set Width Equally | `alignment:set-width-equally` | ✅ | largest width in the selection (src/align.rs:92) |
| diagram-canvas | Format › Alignment › Set Height Equally | `alignment:set-height-equally` | ✅ | largest height in the selection (src/align.rs:93) |
| diagram-canvas | Format › Alignment › Set Size Equally | `alignment:set-size-equally` | ✅ | largest width and height in the selection (src/align.rs:94) |
| diagram-canvas | Model › Add › Tag | `common:tag` | ✅ | Model › Add › Tag and the explorer Add menu create a string tag "Tag<n>" (src/ui.rs:304-313) |
| diagram-canvas | Format › Suppress Columns | `erd:suppress-columns` | ❌ |  |
| diagram-canvas | Format › Suppress Property Values | `format:suppress-property-values` | ❌ | SysML |
| diagram-canvas | Format › Suppress Constraints | `format:suppress-constraints` | ❌ | SysML |
| diagram-canvas | Format › Suppress Parts | `format:suppress-parts` | ❌ | SysML |
| diagram-canvas | Format › Suppress Ports | `format:suppress-ports` | ❌ | SysML |
| diagram-canvas | Format › Suppress References | `format:suppress-references` | ❌ | SysML |
| diagram-canvas | Format › Suppress Values | `format:suppress-values` | ❌ | SysML |
| diagram-canvas | Format › Suppress Properties | `format:suppress-properties` | ❌ | SysML |
| diagram-canvas | Format › Suppress Flow Properties | `format:suppress-flow-properties` | ❌ | SysML |
| diagram-canvas | Model › Add › Entry Activity | `uml:create-model.entry-activity` | ❌ |  |
| diagram-canvas | Model › Add › Do Activity | `uml:create-model.do-activity` | ❌ |  |
| diagram-canvas | Model › Add › Exit Activity | `uml:create-model.exit-activity` | ❌ |  |
| diagram-canvas | Model › Add › Effect | `uml:create-model.effect` | ❌ |  |
| diagram-canvas | Model › Add › Constraint | `uml:create-model.constraint` | ❌ |  |
| diagram-canvas | Format › Stereotype Display › None | `format:stereotype-none` | ✅ | sets stereotypeDisplay to none on the selected views; the stereotype is hidden (src/ui.rs:1384, 1450; src/style.rs:110) |
| diagram-canvas | Format › Stereotype Display › Label | `format:stereotype-label` | ✅ | sets stereotypeDisplay to label; the «stereotype» line is drawn (src/ui.rs:1384, 1450; src/style.rs:110) |
| diagram-canvas | Format › Stereotype Display › Decoration | `format:stereotype-decoration` | 🟡 | the value is set, but UMLCraft draws no stereotype decoration icon (src/ui.rs:1384, 1450; src/style.rs:110) |
| diagram-canvas | Format › Stereotype Display › Decoration with Label | `format:stereotype-decoration-label` | 🟡 | the value is set and the label drawn, but no decoration icon (src/ui.rs:1384, 1450; src/style.rs:110) |
| diagram-canvas | Format › Stereotype Display › Icon | `format:stereotype-icon` | 🟡 | the value is set; only interfaces have an icon (the ball) (src/ui.rs:1384, 1450; src/style.rs:110) |
| diagram-canvas | Format › Stereotype Display › Icon with Label | `format:stereotype-icon-label` | 🟡 | the value is set; only interfaces have an icon (the ball), and stereotype icons are not drawn (src/ui.rs:1384, 1450; src/style.rs:110) |
| diagram-canvas | Format › Word Wrap | `format:word-wrap` | ❌ |  |
| diagram-canvas | Format › Show Visibility | `format:show-visibility` | ✅ | check item in Format (main and canvas menus): inverts the shared showVisibility of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Format › Show Namespace | `format:show-namespace` | ✅ | check item in Format (main and canvas menus): inverts the shared showNamespace of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Format › Show Property | `format:show-property` | ✅ | check item in Format (main and canvas menus): inverts the shared showProperty of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1262, 1412, 1968-1980; src/app.rs:1131-1159; src/views.rs:386-410) |
| diagram-canvas | Format › Show Type | `format:show-type` | ✅ | check item in Format (main and canvas menus): inverts the shared showType of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1262, 1412, 1968-1980; src/app.rs:1131-1159; src/views.rs:386-410) |
| diagram-canvas | Format › Show Multiplicity | `format:show-multiplicity` | ✅ | check item in Format (main and canvas menus): inverts the shared showMultiplicity of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Format › Show Operation Signature | `format:show-operation-signature` | ✅ | check item in Format (main and canvas menus): inverts the shared showOperationSignature of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Format › Show Diagram Name | `format:show-diagram-name` | ❌ |  |
| diagram-canvas | Format › Suppress Attributes | `format:suppress-attributes` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressAttributes of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Format › Suppress Operations | `format:suppress-operations` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressOperations of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Format › Suppress Receptions | `format:suppress-receptions` | ❌ |  |
| diagram-canvas | Format › Suppress Literals | `format:suppress-literals` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressLiterals of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1399-1411; src/app.rs:1117-1144) |
| diagram-canvas | Add | `model:add` | 🟡 | Model › Add and the explorer Add menu offer Model, Package, Profile, Class, Interface, Signal, DataType, PrimitiveType, Enumeration, Stereotype, Actor, Use Case and the features that apply (src/ui.rs:279-324) |
| diagram-canvas | Add Diagram | `model:add-diagram` | 🟡 | Model › Add Diagram and the explorer menu offer Class, Package and Use Case diagrams, the kinds UMLCraft edits (src/ui.rs:116, 335, 1125) |
| diagram-canvas | Format › Font... | `format:font` | ✅ | Ctrl+Shift+F and Format › Font... (main and canvas menus) open the Font window: face, size and colour pre-filled with the merged values. The face is stored but drawn with the Arial-compatible font (src/ui.rs:1447, 1108, 1793-1850; src/style.rs:74) |
| diagram-canvas | Format › Fill Color... | `format:fill-color` | ✅ | Ctrl+Shift+I opens a colour window; Format › Fill Color (main and canvas menus) has a colour picker; both apply the merged colour to the selected views (src/ui.rs:1456, 1299, 1860; src/style.rs:35) |
| diagram-canvas | Format › Line Color... | `format:line-color` | ✅ | Ctrl+Shift+L opens a colour window; Format › Line Color has a colour picker (src/ui.rs:1456, 1299, 1860; src/style.rs:35) |
| diagram-canvas | Format › Line Style › Rectilinear | `format:linestyle-rectilinear` | ✅ | Ctrl+L and a check item in Format › Line Style (main and canvas menus) (src/app.rs:1100; src/ui.rs:1314, 1559) |
| diagram-canvas | Format › Line Style › Oblique | `format:linestyle-oblique` | ✅ | Ctrl+B and a check item in Format › Line Style (src/app.rs:1100; src/ui.rs:1314, 1562) |
| diagram-canvas | Format › Line Style › RoundRect | `format:linestyle-roundrect` | ✅ | Ctrl+Alt+L and Format › Line Style › Rounded Rectilinear: routed like rectilinear, drawn with rounded corners (radius estimated) (src/ui.rs:1314, 1553; src/views.rs:912) |
| diagram-canvas | Format › Line Style › Curve | `format:linestyle-curve` | ✅ | Ctrl+Alt+B and Format › Line Style › Curve: drawn as a smooth curve bending towards each point (curve shape estimated) (src/ui.rs:1314, 1556; src/views.rs:912) |
| diagram-canvas | Format › Auto Resize | `format:auto-resize` | ✅ | Ctrl+Shift+R and a check item in Format (main and canvas menus) and Styles: inverts the shared autoResize, or sets it when mixed (src/ui.rs:1461, 1938; src/app.rs:1109-1140) |
| diagram-canvas | Format › Show Shadow | `format:show-shadow` | ✅ | Ctrl+Shift+H and a check item in Format (main and canvas menus): inverts the shared value, or sets it when mixed (src/app.rs:1131, 1147; src/ui.rs:1565, 1399) |
| diagram-canvas | Format › Inherit Style | `format:inherit-style` | ❌ |  |
| diagram-canvas | Cut | `edit:cut` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:1344, 1509) |
| diagram-canvas | Copy | `edit:copy` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:1344, 1509) |
| diagram-canvas | Paste | `edit:paste` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:1344, 1509) |
| diagram-canvas | Select All | `edit:select-all` | ✅ | Ctrl+A (src/ui.rs:1529) |
| diagram-canvas | Delete | `edit:delete` | ✅ | Delete key and Edit menu (src/canvas.rs:577, src/ui.rs:1077) |
| diagram-canvas | Delete From Model | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:1526) |
| diagram-canvas | Select In Explorer | `edit:select-in-explorer` | ✅ | Ctrl+E (src/ui.rs:1532) |
| diagram-canvas | Select In Diagram | `edit:select-in-diagram` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1042; src/ui.rs:1541, 1096, 354; src/canvas.rs:1575) |
| diagram-canvas | Open Sub-Diagram | `edit:open-sub-diagram` | ✅ | Ctrl+Shift+D, Edit and canvas menus: opens the first diagram the element owns, or toasts "No sub-diagram in the selected element" (src/app.rs:1020; src/ui.rs:1100, 1538; src/canvas.rs:1580) |
| diagram-canvas | Tag Editor... | `model:tag-editor` | ✅ | Ctrl+T, Model › Tag Editor..., and the canvas and explorer menus (src/ui.rs:1139, 1576; src/tags.rs:171-245) |
| mermaid-example-dropdown | Class Diagram | `mermaid:example-class-diagram` | ❌ |  |
| mermaid-example-dropdown | Sequence Diagram | `mermaid:example-sequence-diagram` | ❌ |  |
| mermaid-example-dropdown | State Diagram | `mermaid:example-state-diagram` | ❌ |  |
| mermaid-example-dropdown | Flowchart | `mermaid:example-flowchart` | ❌ |  |
| mermaid-example-dropdown | Entity Relationship Diagram | `mermaid:example-er-diagram` | ❌ |  |
| mermaid-example-dropdown | Requirement Diagram | `mermaid:example-requirement-diagram` | ❌ |  |
| mermaid-example-dropdown | Mindmap | `mermaid:example-mindmap` | ❌ |  |
| #relationship-view div.listview | Select Related Element | `relationship-view:select-related-element` | ❌ |  |
| #relationship-view div.listview | Select Relationship | `relationship-view:select-relationship` | ❌ |  |
| model-explorer | *(the menu as a whole)* |  | 🟡 | the explorer menu has Add, Add Diagram, Open Diagram, Delete from Model (src/ui.rs:278) |
| model-explorer | Model › Add › Tag | `common:tag` | ✅ | Model › Add › Tag and the explorer Add menu create a string tag "Tag<n>" (src/ui.rs:304-313) |
| model-explorer | Model › Add › Entry Activity | `uml:create-model.entry-activity` | ❌ |  |
| model-explorer | Model › Add › Do Activity | `uml:create-model.do-activity` | ❌ |  |
| model-explorer | Model › Add › Exit Activity | `uml:create-model.exit-activity` | ❌ |  |
| model-explorer | Model › Add › Effect | `uml:create-model.effect` | ❌ |  |
| model-explorer | Model › Add › Constraint | `uml:create-model.constraint` | ❌ |  |
| model-explorer | Add Diagram | `model:add-diagram` | 🟡 | Model › Add Diagram and the explorer menu offer Class, Package and Use Case diagrams, the kinds UMLCraft edits (src/ui.rs:116, 335, 1125) |
| model-explorer | Add | `model:add` | 🟡 | Model › Add and the explorer Add menu offer Model, Package, Profile, Class, Interface, Signal, DataType, PrimitiveType, Enumeration, Stereotype, Actor, Use Case and the features that apply (src/ui.rs:279-324) |
| model-explorer | Cut | `edit:cut` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:1344, 1509) |
| model-explorer | Copy | `edit:copy` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:1344, 1509) |
| model-explorer | Paste | `edit:paste` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:1344, 1509) |
| model-explorer | Delete From Model | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:1526) |
| model-explorer | Move Up | `edit:move-up` | ✅ | Ctrl+Shift+Up, Edit and explorer menus: swaps with the previous sibling of the same explorer group (src/app.rs:1002; src/ui.rs:1331, 1544) |
| model-explorer | Move Down | `edit:move-down` | ✅ | Ctrl+Shift+Down, Edit and explorer menus: swaps with the next sibling of the same explorer group (src/app.rs:1002; src/ui.rs:1331, 1544) |
| model-explorer | Select In Diagram | `edit:select-in-diagram` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1042; src/ui.rs:1541, 1096, 354; src/canvas.rs:1575) |
| working-diagrams | *(the menu as a whole)* |  | 🟡 | right-clicking a row opens Close Diagram, Close Other Diagrams and Close All Diagrams, acting on the current diagram; no Rename (src/ui.rs:813, 1365) |
| working-diagrams | Close Diagram | `view:close-diagram` | ✅ | F4, View menu and Working Diagrams menu; the diagram that slides into its place, or the first, becomes current (src/ui.rs:1610, 1365; src/app.rs:944) |
| working-diagrams | Close Other Diagrams | `view:close-other-diagrams` | ✅ | Ctrl+F4, View menu and Working Diagrams menu (src/ui.rs:1607, 1365; src/app.rs:966) |
| working-diagrams | Close All Diagrams | `view:close-all-diagrams` | ✅ | Shift+F4, View menu and Working Diagrams menu (src/ui.rs:1597, 1365; src/app.rs:971) |
| working-diagrams | Rename | `view:rename-diagram` | ❌ | the Working Diagrams menu has no Rename and there is no rename dialog (src/ui.rs:1365) |
| model-explorer-settings | *(the menu as a whole)* |  | ✅ | the header settings button opens Sort By Added, Sort By Name, Show Stereotype Text, Expand All and Collapse All (src/ui.rs:385, 422-444) |
| model-explorer-settings | Sort By Added | `explorer:sort-by-added` | ✅ | radio item in the settings menu (src/ui.rs:423-426) |
| model-explorer-settings | Sort By Name | `explorer:sort-by-name` | ✅ | radio item in the settings menu (src/ui.rs:427-430, 150) |
| model-explorer-settings | Show Stereotype Text | `explorer:show-stereotype-text` | ✅ | check item, on by default; tree labels get «stereotype» first (src/ui.rs:432-434, 199-204) |
| model-explorer-settings | Expand All | `explorer:expand-all` | ✅ | opens every element that has children (src/ui.rs:436-439, 447-454) |
| model-explorer-settings | Collapse All | `explorer:collapse-all` | ✅ | closes every element, leaving only the project row (src/ui.rs:440-443) |
