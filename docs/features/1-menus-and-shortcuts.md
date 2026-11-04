# Menus and keyboard shortcuts

Part of the [StarUML feature inventory](README.md); the legend and how to update a status are there.

StarUML 7.1.1 main menus, context menus and keymaps, from every `menus/*.json` and `keymaps/*.json` (Linux) in the app. One row per item; a command can appear in several tables.

| Shortcut | Command | UMLCraft | Notes | Source |
|---|---|---|---|---|
| `shift-alt-t` | `debug:show-devtools` | ➖ | Electron devtools | extensions/default/debug/keymaps/debug.json |
| `ctrl-r` | `debug:reload` | ➖ | Electron reload | extensions/default/debug/keymaps/debug.json |
| `ctrl-alt-t` | `diagram-thumbnails:toggle` | ✅ | Ctrl+Alt+T shows or hides the Diagram Thumbnails panel (src/ui.rs:2297-2299) | extensions/default/diagram-thumbnails/keymaps/diagram-thumbnails.json |
| `ctrl-f` | `find:find` | ✅ | Ctrl+F and Model › Find... open the Find dialog; results go to a Find Results panel (src/ui.rs:2306, 1733; src/find.rs:49) | extensions/default/find/keymaps/find.json |
| `ctrl-alt-d` | `markdown-doc:toggle` | ✅ | Ctrl+Alt+D shows or hides the Markdown Documentation panel (src/ui.rs:2260-2262) | extensions/default/markdown/keymaps/markdown.json |
| `ctrl-alt-m` | `minimap:toggle` | ✅ | Ctrl+Alt+M shows or hides the minimap (src/ui.rs:2294-2296) | extensions/default/minimap/keymaps/minimap.json |
| `ctrl-alt-r` | `relationship-view:toggle` | ✅ | Ctrl+Alt+R shows or hides the Relationships panel (src/ui.rs:2291-2299) | extensions/default/relationship-view/keymaps/relationship-view.json |
| `ctrl-shift-u` | `erd:suppress-columns` | ❌ |  | extensions/essential/erd/keymaps/erd.json |
| `ctrl-shift-0` | `format:stereotype-none` | ✅ | sets stereotypeDisplay to none on the selected views; the stereotype is hidden (src/ui.rs:2080, 2147; src/style.rs:176) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-1` | `format:stereotype-label` | ✅ | sets stereotypeDisplay to label; the «stereotype» line is drawn (src/ui.rs:2080, 2147; src/style.rs:176) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-2` | `format:stereotype-decoration` | 🟡 | the value is set; interfaces and «boundary», «entity» and «control» classes get a small decoration icon at the top right, other stereotypes none (src/ui.rs:2080, 2147; src/style.rs:176; src/render.rs:370-398) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-3` | `format:stereotype-decoration-label` | 🟡 | the value is set and the label drawn; interfaces and «boundary», «entity» and «control» classes get a small decoration icon at the top right, other stereotypes none (src/ui.rs:2080, 2147; src/style.rs:176; src/render.rs:370-398) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-4` | `format:stereotype-icon` | 🟡 | the value is set; only interfaces (the ball) and «boundary», «entity» and «control» classes have an icon; profile stereotype icons are not drawn (src/ui.rs:2080, 2147; src/style.rs:176; src/views.rs:29-52) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-5` | `format:stereotype-icon-label` | 🟡 | the value is set; only interfaces (the ball) and «boundary», «entity» and «control» classes have an icon; profile stereotype icons are not drawn, and stereotype icons are not drawn (src/ui.rs:2080, 2147; src/style.rs:176; src/views.rs:29-52) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-w` | `format:word-wrap` | ✅ | Ctrl+Shift+W toggles wordWrap on the selected views that have it (src/ui.rs:2181; src/app.rs:1684-1698) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-v` | `format:show-visibility` | 🟡 | Ctrl+Shift+V toggles showVisibility; egui turns Ctrl+Shift+V into a paste, so the key works only while the clipboard holds text (src/ui.rs:2157-2179; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-n` | `format:show-namespace` | ✅ | Ctrl+Shift+N toggles showNamespace on the selected views that have it (src/ui.rs:2157; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-b` | `format:show-property` | ✅ | Ctrl+Shift+B toggles showProperty, which shows property strings ({readOnly, ordered…}) of attributes and association ends, on the selected views that have it (src/ui.rs:2181; src/app.rs:1684-1698) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-y` | `format:show-type` | ✅ | Ctrl+Shift+Y toggles showType, which shows the types of attributes, parameters and return values, on the selected views that have it (src/ui.rs:2181; src/app.rs:1684-1698) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-m` | `format:show-multiplicity` | ✅ | Ctrl+Shift+M toggles showMultiplicity on the selected views that have it (src/ui.rs:2157; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-j` | `format:show-operation-signature` | ✅ | Ctrl+Shift+J toggles showOperationSignature on the selected views that have it (src/ui.rs:2157; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-d` | `format:show-diagram-name` | ✅ | Format › Show Diagram Name (main and canvas menus) toggles showDiagramName on the selected frames, which then end their name with `[diagram]`. Ctrl+Shift+D does it only when frames are selected: StarUML's UML keymap takes the key from Open Sub-Diagram, but UMLCraft keeps Open Sub-Diagram for every other selection (src/ui.rs:2233-2242, 2819; src/views.rs:547-567) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-a` | `format:suppress-attributes` | ✅ | Ctrl+Shift+A toggles suppressAttributes on the selected views that have it (src/ui.rs:2157; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-o` | `format:suppress-operations` | ✅ | Ctrl+Shift+O toggles suppressOperations on the selected views that have it (src/ui.rs:2157; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-e` | `format:suppress-receptions` | ✅ | Ctrl+Shift+E toggles suppressReceptions on the selected views that have it (src/ui.rs:2181; src/app.rs:1684) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-t` | `format:suppress-literals` | ✅ | Ctrl+Shift+T toggles suppressLiterals on the selected views that have it (src/ui.rs:2157; src/app.rs:1670) | extensions/essential/uml/keymaps/uml.json |
| `ctrl-n` | `application:new` | ✅ | File › New, Ctrl+N (src/ui.rs:1555, 2181) | resources/default/keymaps/linux.json |
| `ctrl-o` | `project:open` | ✅ | File › Open, Ctrl+O (src/ui.rs:1570, 2184) | resources/default/keymaps/linux.json |
| `ctrl-s` | `project:save` | ✅ | Ctrl+S (src/ui.rs:1586, 2190) | resources/default/keymaps/linux.json |
| `ctrl-shift-s` | `project:save-as` | ✅ | Ctrl+Shift+S (src/ui.rs:1590, 2187) | resources/default/keymaps/linux.json |
| `ctrl-q` | `application:quit` | ✅ | File › Exit, Ctrl+Q, with unsaved-changes prompt (src/ui.rs:1650, 2193, 2756) | resources/default/keymaps/linux.json |
| `ctrl-z` | `edit:undo` | ✅ | Ctrl+Z (src/ui.rs:2207) | resources/default/keymaps/linux.json |
| `ctrl-y` | `edit:redo` | ✅ | Ctrl+Y and Ctrl+Shift+Z (src/ui.rs:2204) | resources/default/keymaps/linux.json |
| `ctrl-x` | `edit:cut` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:2040, 2214) | resources/default/keymaps/linux.json |
| `ctrl-c` | `edit:copy` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:2040, 2214) | resources/default/keymaps/linux.json |
| `ctrl-shift-c` | `edit:copy-diagram-as-image` | ✅ | Ctrl+Shift+C and Edit menu: deselects, draws the diagram as PNG export does and puts the picture on the system clipboard, white background outside macOS (src/clipboard.rs:165; src/ui.rs:2050, 2225, 2404) | resources/default/keymaps/linux.json |
| `ctrl-v` | `edit:paste` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:2040, 2214) | resources/default/keymaps/linux.json |
| `delete` | `edit:delete` | ✅ | Delete key and Edit menu (src/canvas.rs:684, src/ui.rs:1666) | resources/default/keymaps/linux.json |
| `ctrl-delete` | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:2231) | resources/default/keymaps/linux.json |
| `ctrl-shift-up` | `edit:move-up` | ✅ | Ctrl+Shift+Up, Edit and explorer menus: swaps with the previous sibling of the same explorer group (src/app.rs:1536; src/ui.rs:2027, 2249) | resources/default/keymaps/linux.json |
| `ctrl-shift-down` | `edit:move-down` | ✅ | Ctrl+Shift+Down, Edit and explorer menus: swaps with the next sibling of the same explorer group (src/app.rs:1536; src/ui.rs:2027, 2249) | resources/default/keymaps/linux.json |
| `ctrl-a` | `edit:select-all` | ✅ | Ctrl+A (src/ui.rs:2234) | resources/default/keymaps/linux.json |
| `ctrl-e` | `edit:select-in-explorer` | ✅ | Ctrl+E (src/ui.rs:2237) | resources/default/keymaps/linux.json |
| `ctrl-d` | `edit:select-in-diagram` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1576; src/ui.rs:2246, 1685, 552; src/canvas.rs:1971) | resources/default/keymaps/linux.json |
| `ctrl-shift-d` | `edit:open-sub-diagram` | ✅ | Ctrl+Shift+D, Edit and canvas menus: opens the first diagram the element owns, or toasts "No sub-diagram in the selected element" (src/app.rs:1554; src/ui.rs:1689, 2243; src/canvas.rs:1976) | resources/default/keymaps/linux.json |
| `ctrl-shift-f` | `format:font` | ✅ | Ctrl+Shift+F and Format › Font... (main and canvas menus) open the Font window: face, size and colour pre-filled with the merged values. The face is stored but drawn with the Arial-compatible font (src/ui.rs:2144, 1697, 2559-2638; src/style.rs:140) | resources/default/keymaps/linux.json |
| `ctrl-shift-i` | `format:fill-color` | ✅ | Ctrl+Shift+I opens the Color dialog (picker with opacity, hex field, preset palette, Transparent; OK applies); Format › Fill Color (main and canvas menus) has a colour picker; both apply to the selected views (src/ui.rs:2176-2179, 2007, 2661-2751; src/style.rs:77-84) | resources/default/keymaps/linux.json |
| `ctrl-shift-l` | `format:line-color` | ✅ | Ctrl+Shift+L opens the Color dialog; Format › Line Color has a colour picker (src/ui.rs:2176-2179, 2007, 2661-2751; src/style.rs:77-84) | resources/default/keymaps/linux.json |
| `ctrl-l` | `format:linestyle-rectilinear` | ✅ | Ctrl+L and a check item in Format › Line Style (main and canvas menus) (src/app.rs:1639; src/ui.rs:2010, 2274) | resources/default/keymaps/linux.json |
| `ctrl-b` | `format:linestyle-oblique` | ✅ | Ctrl+B and a check item in Format › Line Style (src/app.rs:1639; src/ui.rs:2010, 2277) | resources/default/keymaps/linux.json |
| `ctrl-alt-l` | `format:linestyle-roundrect` | ✅ | Ctrl+Alt+L and Format › Line Style › Rounded Rectilinear: routed like rectilinear, drawn with rounded corners (radius estimated) (src/ui.rs:2010, 2258; src/views.rs:1447) | resources/default/keymaps/linux.json |
| `ctrl-alt-b` | `format:linestyle-curve` | ✅ | Ctrl+Alt+B and Format › Line Style › Curve: drawn as a smooth curve bending towards each point (curve shape estimated) (src/ui.rs:2010, 2267; src/views.rs:1447) | resources/default/keymaps/linux.json |
| `ctrl-shift-r` | `format:auto-resize` | ✅ | Ctrl+Shift+R and a check item in Format (main and canvas menus) and Styles: inverts the shared autoResize, or sets it when mixed (src/ui.rs:2166, 2801; src/app.rs:1648-1679) | resources/default/keymaps/linux.json |
| `ctrl-shift-h` | `format:show-shadow` | ✅ | Ctrl+Shift+H and a check item in Format (main and canvas menus): inverts the shared value, or sets it when mixed (src/app.rs:1670, 1686; src/ui.rs:2280, 2095) | resources/default/keymaps/linux.json |
| `ctrl-shift-k` | `format:inherit-style` | ✅ | Ctrl+Shift+K and a check item in Format (main and canvas menus) after Show Shadow: sets parentStyle, inverting the shared value, or `true` when mixed (src/app.rs:1684; src/ui.rs:2181, 2841) | resources/default/keymaps/linux.json |
| `ctrl-t` | `model:tag-editor` | ✅ | Ctrl+T, Model › Tag Editor..., and the canvas and explorer menus (src/ui.rs:1744, 2300; src/tags.rs:171-275) | resources/default/keymaps/linux.json |
| `ctrl-p` | `view:quick-find` | ✅ | Ctrl+P and View › Quick Find... (src/ui.rs:1775, 2303; src/palette.rs:182-217, 346) | resources/default/keymaps/linux.json |
| `ctrl-shift-p` | `view:command-palette` | ✅ | Ctrl+Shift+P and View › Command Palette... (src/ui.rs:2167; src/palette.rs:63-174, 227-344) | resources/default/keymaps/linux.json |
| `f4` | `view:close-diagram` | ✅ | F4, View menu and Working Diagrams menu; the diagram that slides into its place, or the first, becomes current (src/ui.rs:2334, 2061; src/app.rs:1437) | resources/default/keymaps/linux.json |
| `ctrl-f4` | `view:close-other-diagrams` | ✅ | Ctrl+F4, View menu and Working Diagrams menu (src/ui.rs:2331, 2061; src/app.rs:1498) | resources/default/keymaps/linux.json |
| `shift-f4` | `view:close-all-diagrams` | ✅ | Shift+F4, View menu and Working Diagrams menu (src/ui.rs:2321, 2061; src/app.rs:1503) | resources/default/keymaps/linux.json |
| `ctrl-shift-]` | `view:next-diagram` | ✅ | Ctrl+Shift+] (also reported as Ctrl+Shift+}), wraps to the first (src/ui.rs:2338, 1787; src/app.rs:1513) | resources/default/keymaps/linux.json |
| `ctrl-shift-[` | `view:previous-diagram` | ✅ | Ctrl+Shift+[ (also reported as Ctrl+Shift+{), wraps to the last (src/ui.rs:2341, 1791; src/app.rs:1513) | resources/default/keymaps/linux.json |
| `ctrl-plus` | `view:zoom-in` | ✅ | Ctrl+= / Ctrl++, keeps the screen centre (src/ui.rs:2283; src/canvas.rs:2050) | resources/default/keymaps/linux.json |
| `ctrl-minus` | `view:zoom-out` | ✅ | Ctrl+-, keeps the screen centre (src/ui.rs:2286; src/canvas.rs:2050) | resources/default/keymaps/linux.json |
| `ctrl-0` | `view:actual-size` | ✅ | Ctrl+0, keeps the screen centre (src/ui.rs:2289; src/canvas.rs:2050) | resources/default/keymaps/linux.json |
| `ctrl-9` | `view:fit-to-window` | ✅ | Ctrl+9, centred on the bounding box (src/ui.rs:2310; src/canvas.rs:2062) | resources/default/keymaps/linux.json |
| `ctrl-g` | `view:show-grid` | ✅ | Ctrl+G (src/ui.rs:2317) | resources/default/keymaps/linux.json |
| `ctrl-shift-g` | `view:snap-to-grid` | ✅ | Ctrl+Shift+G, checked before Ctrl+G so it is not taken as Show Grid (src/ui.rs:2314) | resources/default/keymaps/linux.json |
| `ctrl-1` | `view:sidebar` | ✅ | View › Sidebar check item and Ctrl+1; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/keymaps/linux.json |
| `ctrl-2` | `view:navigator` | ✅ | View › Navigator check item and Ctrl+2; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/keymaps/linux.json |
| `ctrl-3` | `view:toolbar` | ✅ | View › Toolbar check item and Ctrl+3; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/keymaps/linux.json |
| `ctrl-4` | `view:statusbar` | ✅ | View › Statusbar check item and Ctrl+4; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/keymaps/linux.json |
| `ctrl-5` | `view:toolbox` | ✅ | View › Toolbox check item and Ctrl+5; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/keymaps/linux.json |
| `ctrl-6` | `view:editors` | ✅ | View › Editors check item and Ctrl+6; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/keymaps/linux.json |

## Main menu items

| Menu path | Command | Shortcut | UMLCraft | Notes | Source |
|---|---|---|---|---|---|
| Format › Alignment › Send To Back | `alignment:send-to-back` |  | ✅ | Format › Alignment › Send To Back moves the selected top-level views to the start of ownedViews (src/align.rs:146; src/ui.rs:2111) | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Bring To Front | `alignment:bring-to-front` |  | ✅ | Format › Alignment › Bring To Front moves the selected top-level views to the end of ownedViews (src/align.rs:146; src/ui.rs:2111) | extensions/default/alignment/menus/alignment.json |
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
| Tools › Diagram Generator › Overview | `diagram-generator:overview` |  | ✅ | generates the Overview diagram of the package selected in the Explorer, laid out and opened (src/ui.rs:1760-1772; src/generate.rs:42) | extensions/default/diagram-generator/menus/diagram-generator.json |
| Tools › Diagram Generator › Overview Expanded | `diagram-generator:overview-expanded` |  | ✅ | generates the Overview Expanded diagram of the package selected in the Explorer, laid out and opened (src/ui.rs:1760-1772; src/generate.rs:42) | extensions/default/diagram-generator/menus/diagram-generator.json |
| Tools › Diagram Generator › Type Hierarchy | `diagram-generator:type-hierarchy` |  | ✅ | generates the Type Hierarchy diagram of the package selected in the Explorer, laid out and opened (src/ui.rs:1760-1772; src/generate.rs:42) | extensions/default/diagram-generator/menus/diagram-generator.json |
| Tools › Diagram Generator › Package Structure | `diagram-generator:package-structure` |  | ✅ | generates the Package Structure diagram of the package selected in the Explorer, laid out and opened (src/ui.rs:1760-1772; src/generate.rs:42) | extensions/default/diagram-generator/menus/diagram-generator.json |
| Format › Layout › Auto | `diagram-layout:auto` |  | ✅ | lays the current diagram out in ranks, top to bottom, in one undo step; UMLCraft's own layered layout, not dagre (src/ui.rs:1706-1716; src/layout.rs:356) | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Top to Bottom | `diagram-layout:top-bottom` |  | ✅ | lays the current diagram out in ranks, super-classes and other line heads above, in one undo step; UMLCraft's own layered layout, not dagre (src/ui.rs:1706-1716; src/layout.rs:356) | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Bottom to Top | `diagram-layout:bottom-top` |  | ✅ | lays the current diagram out in ranks, heads below, in one undo step; UMLCraft's own layered layout, not dagre (src/ui.rs:1706-1716; src/layout.rs:356) | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Left to Right | `diagram-layout:left-right` |  | ✅ | lays the current diagram out in ranks, heads on the left, in one undo step; UMLCraft's own layered layout, not dagre (src/ui.rs:1706-1716; src/layout.rs:356) | extensions/default/diagram-layout/menus/diagram-layout.json |
| Format › Layout › Right to Left | `diagram-layout:right-left` |  | ✅ | lays the current diagram out in ranks, heads on the right, in one undo step; UMLCraft's own layered layout, not dagre (src/ui.rs:1706-1716; src/layout.rs:356) | extensions/default/diagram-layout/menus/diagram-layout.json |
| View › Diagram Thumbnails | `diagram-thumbnails:toggle` | `ctrl-alt-t` | ✅ | check item in the View menu; the visibility is kept in `view.diagramThumbnails.visibility` (src/ui.rs:1815; src/prefs.rs:94, 121) | extensions/default/diagram-thumbnails/menus/diagram-thumbnails.json |
| Model › Find | `find:find` | `ctrl-f` | ✅ | Ctrl+F and Model › Find... open the Find dialog; results go to a Find Results panel (src/ui.rs:2306, 1733; src/find.rs:49) | extensions/default/find/menus/find.json |
| File › Export › HTML Docs... | `html-export:export` |  | ✅ | asks for a folder and writes `<folder>/html-docs`; toasts "HTML docs exported successfully." (src/ui.rs:1626-1639; src/html.rs:487-493) | extensions/default/html-export/menus/html-export.json |
| View › Markdown Documentation | `markdown-doc:toggle` | `ctrl-alt-d` | ✅ | check item in the View menu; the visibility is kept in `view.markdown.visibility` (src/ui.rs:1814; src/prefs.rs:96, 123) | extensions/default/markdown/menus/markdown.json |
| Tools › Generate Diagram by Mermaid... | `mermaid:show-mermaid-dialog` |  | ✅ | opens the Mermaid dialog; class diagrams only (src/ui.rs:1761-1764; src/mermaid.rs:463) | extensions/default/mermaid/menus/mermaid.json |
| View › Minimap | `minimap:toggle` | `ctrl-alt-m` | ✅ | check item in the View menu; the state is kept in `view.minimap.visibility`, on by default (src/ui.rs:1813; src/prefs.rs:95, 122; src/canvas.rs:510, 738) | extensions/default/minimap/menus/minimap.json |
| View › Relationships | `relationship-view:toggle` | `ctrl-alt-r` | ✅ | check item in the View menu; the panel's visibility is kept in `view.relationships.visibility` (src/ui.rs:1816; src/prefs.rs:93, 120) | extensions/default/relationship-view/menus/relationship-view.json |
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
| Model › Add › Tag | `common:tag` |  | ✅ | Model › Add › Tag and the explorer Add menu create a string tag "Tag<n>" (src/ui.rs:489-498) | extensions/essential/common/menus/common.json |
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
| Model › Add › Parameter | `factory:create-model` |  | ✅ | explorer Add (operations), Editors button (src/ui.rs:442) | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Property Values | `format:suppress-property-values` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Constraints | `format:suppress-constraints` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Parts | `format:suppress-parts` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Ports | `format:suppress-ports` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress References | `format:suppress-references` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Values | `format:suppress-values` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Properties | `format:suppress-properties` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| Format › Suppress Flow Properties | `format:suppress-flow-properties` |  | ❌ | SysML | extensions/essential/sysml/menus/sysml.json |
| File › New From Template › UML Minimal | `uml:new-from-template` |  | ✅ | starts a new project with the template's models and empty diagrams, after the save prompt; with the UML Standard Profile's stereotypes (src/ui.rs:1559-1569; src/templates.rs:91-105) | extensions/essential/uml/menus/uml.json |
| File › New From Template › UML Conventional | `uml:new-from-template` |  | ✅ | starts a new project with the template's models and empty diagrams, after the save prompt; with the UML Standard Profile's stereotypes (src/ui.rs:1559-1569; src/templates.rs:91-105) | extensions/essential/uml/menus/uml.json |
| File › New From Template › 4+1 View Model | `uml:new-from-template` |  | 🟡 | starts a new project with the template's models and empty diagrams, after the save prompt; with the UML Standard Profile's stereotypes (src/ui.rs:1559-1569; src/templates.rs:91-105) | extensions/essential/uml/menus/uml.json |
| File › New From Template › Rational | `uml:new-from-template` |  | 🟡 | starts a new project with the template's models and empty diagrams, after the save prompt; with the UML Standard Profile's stereotypes (src/ui.rs:1559-1569; src/templates.rs:91-105) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › None | `format:stereotype-none` | `ctrl-shift-0` | ✅ | sets stereotypeDisplay to none on the selected views; the stereotype is hidden (src/ui.rs:2080, 2147; src/style.rs:176) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Label | `format:stereotype-label` | `ctrl-shift-1` | ✅ | sets stereotypeDisplay to label; the «stereotype» line is drawn (src/ui.rs:2080, 2147; src/style.rs:176) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Decoration | `format:stereotype-decoration` | `ctrl-shift-2` | 🟡 | the value is set; interfaces and «boundary», «entity» and «control» classes get a small decoration icon at the top right, other stereotypes none (src/ui.rs:2080, 2147; src/style.rs:176; src/render.rs:370-398) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Decoration with Label | `format:stereotype-decoration-label` | `ctrl-shift-3` | 🟡 | the value is set and the label drawn; interfaces and «boundary», «entity» and «control» classes get a small decoration icon at the top right, other stereotypes none (src/ui.rs:2080, 2147; src/style.rs:176; src/render.rs:370-398) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Icon | `format:stereotype-icon` | `ctrl-shift-4` | 🟡 | the value is set; only interfaces (the ball) and «boundary», «entity» and «control» classes have an icon; profile stereotype icons are not drawn (src/ui.rs:2080, 2147; src/style.rs:176; src/views.rs:29-52) | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Icon with Label | `format:stereotype-icon-label` | `ctrl-shift-5` | 🟡 | the value is set; only interfaces (the ball) and «boundary», «entity» and «control» classes have an icon; profile stereotype icons are not drawn, and stereotype icons are not drawn (src/ui.rs:2080, 2147; src/style.rs:176; src/views.rs:29-52) | extensions/essential/uml/menus/uml.json |
| Format › Word Wrap | `format:word-wrap` | `ctrl-shift-w` | ✅ | check item in Format (main and canvas menus) and Styles: inverts the shared wordWrap of the selected views, or sets it when mixed. Class-like boxes then wrap their name and rows to the box width and may shrink to the widest word; notes and text stop wrapping when it is off (src/ui.rs:1944, 2108, 2842; src/views.rs:107-139, 922-1049; src/render.rs:79-102, 144) | extensions/essential/uml/menus/uml.json |
| Format › Show Visibility | `format:show-visibility` | `ctrl-shift-v` | ✅ | check item in Format (main and canvas menus): inverts the shared showVisibility of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Format › Show Namespace | `format:show-namespace` | `ctrl-shift-n` | ✅ | check item in Format (main and canvas menus): inverts the shared showNamespace of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Format › Show Property | `format:show-property` | `ctrl-shift-b` | ✅ | check item in Format (main and canvas menus): inverts the shared showProperty of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1944, 2108, 2838-2854; src/app.rs:1670-1698; src/views.rs:700-724) | extensions/essential/uml/menus/uml.json |
| Format › Show Type | `format:show-type` | `ctrl-shift-y` | ✅ | check item in Format (main and canvas menus): inverts the shared showType of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1944, 2108, 2838-2854; src/app.rs:1670-1698; src/views.rs:700-724) | extensions/essential/uml/menus/uml.json |
| Format › Show Multiplicity | `format:show-multiplicity` | `ctrl-shift-m` | ✅ | check item in Format (main and canvas menus): inverts the shared showMultiplicity of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Format › Show Operation Signature | `format:show-operation-signature` | `ctrl-shift-j` | ✅ | check item in Format (main and canvas menus): inverts the shared showOperationSignature of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Format › Show Diagram Name | `format:show-diagram-name` | `ctrl-shift-d` | ✅ | Format › Show Diagram Name (main and canvas menus) toggles showDiagramName on the selected frames, which then end their name with `[diagram]`. Ctrl+Shift+D does it only when frames are selected: StarUML's UML keymap takes the key from Open Sub-Diagram, but UMLCraft keeps Open Sub-Diagram for every other selection (src/ui.rs:2233-2242, 2819; src/views.rs:547-567) | extensions/essential/uml/menus/uml.json |
| Format › Suppress Attributes | `format:suppress-attributes` | `ctrl-shift-a` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressAttributes of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Format › Suppress Operations | `format:suppress-operations` | `ctrl-shift-o` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressOperations of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Format › Suppress Receptions | `format:suppress-receptions` | `ctrl-shift-e` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressReceptions of the selected views, or sets it when mixed; with it off, a Receptions compartment is drawn under the operations (src/ui.rs:2852; src/app.rs:1670-1698; src/views.rs:1209-1220) | extensions/essential/uml/menus/uml.json |
| Format › Suppress Literals | `format:suppress-literals` | `ctrl-shift-t` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressLiterals of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Class Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:534) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Package Diagram | `factory:create-diagram` |  | ✅ | Model › Add Diagram and the explorer menu; the toolbox shows Packages and Annotations (src/ui.rs:231, 228) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Object Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:234) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Composite Structure Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Component Diagram | `factory:create-diagram` |  | ✅ | Model › Add Diagram › Component Diagram; components, artifacts, interfaces and nodes can be drawn on it (src/ui.rs:231-240) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Deployment Diagram | `factory:create-diagram` |  | ✅ | Model › Add Diagram › Deployment Diagram; components, artifacts, interfaces and nodes can be drawn on it (src/ui.rs:231-240) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Use Case Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:534) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Sequence Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Communication Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Timing Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Interaction Overview Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Information Flow Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:238) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Statechart Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Activity Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Profile Diagram | `factory:create-diagram` |  | ✅ | in a profile, or in a new profile "Profile" (src/ui.rs:239; src/app.rs:1285) | extensions/essential/uml/menus/uml.json |
| Model › Add › Template Parameter | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu on a classifier; drawn in a dashed box over its top-right corner (src/ui.rs:483; src/views.rs:1221) | extensions/essential/uml/menus/uml.json |
| Model › Add › Template Parameter Substitution | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu on a template binding, into its parameterSubstitutions; its formal and actual are in Properties (src/ui.rs:476-483; src/app.rs:1249, 1261) | extensions/essential/uml/menus/uml.json |
| Model › Add › Enumeration Literal | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Attribute | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Operation | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Reception | `factory:create-model` |  | ✅ | explorer Add and Model menu, on classifiers that take attributes; drawn as «signal» name (src/ui.rs:476; src/app.rs:1239-1253) | extensions/essential/uml/menus/uml.json |
| Model › Add › Extension Point | `factory:create-model` |  | ✅ | explorer Add and Model menu on a use case; drawn under its name as "name: location" (src/ui.rs:476; src/app.rs:1244, 1262) | extensions/essential/uml/menus/uml.json |
| Model › Add › Slot | `factory:create-model` |  | ✅ | explorer Add and Model menu on an object, and Ctrl+Enter in its name edit (src/ui.rs:476; src/canvas.rs:1456) | extensions/essential/uml/menus/uml.json |
| Model › Add › Interaction Operand | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Model | `factory:create-model` |  | ✅ | explorer Add (Project only) (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Subsystem | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu; drawn like a package, holding boxes (src/ui.rs:453; src/views.rs:175-176) | extensions/essential/uml/menus/uml.json |
| Model › Add › Package | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Profile | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:443-485) | extensions/essential/uml/menus/uml.json |
| Model › Add › Class | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Interface | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Signal | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:443-485) | extensions/essential/uml/menus/uml.json |
| Model › Add › DataType | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › PrimitiveType | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:443-485) | extensions/essential/uml/menus/uml.json |
| Model › Add › Enumeration | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Artifact | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu, under packages and the project (src/ui.rs:453) | extensions/essential/uml/menus/uml.json |
| Model › Add › Component | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu, under packages and the project (src/ui.rs:453) | extensions/essential/uml/menus/uml.json |
| Model › Add › Node | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu, under packages and the project (src/ui.rs:453) | extensions/essential/uml/menus/uml.json |
| Model › Add › UseCase | `factory:create-model` |  | ✅ | explorer Add (Use Case) (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Actor | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:442) | extensions/essential/uml/menus/uml.json |
| Model › Add › Stereotype | `factory:create-model` |  | ✅ | in Model › Add and the explorer Add menu under a package or the project (src/ui.rs:443-485) | extensions/essential/uml/menus/uml.json |
| Model › Add › Information Item | `factory:create-model` |  | ✅ | explorer Add and Model menu under packages (src/ui.rs:454) | extensions/essential/uml/menus/uml.json |
| Model › Add › Interaction | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu under a classifier; shown in the explorer with its properties (src/ui.rs:463-471, 833-834; src/app.rs:1270-1278) | extensions/essential/uml/menus/uml.json |
| Model › Add › State Machine | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu under a classifier or a package, with one region; shown in the explorer with its properties (src/ui.rs:463-471, 833-834; src/app.rs:1270-1278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Activity | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu under a classifier or a package; shown in the explorer with its properties (src/ui.rs:463-471, 833-834; src/app.rs:1270-1278) | extensions/essential/uml/menus/uml.json |
| Model › Add › Opaque Behavior | `factory:create-model` |  | ✅ | Model › Add and the explorer Add menu under a classifier; shown in the explorer with its properties (src/ui.rs:463-471, 833-834; src/app.rs:1270-1278) | extensions/essential/uml/menus/uml.json |
| Model › Add › State | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Region | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Entry Activity | `uml:create-model.entry-activity` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Do Activity | `uml:create-model.do-activity` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Exit Activity | `uml:create-model.exit-activity` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Trigger | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Effect | `uml:create-model.effect` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Action | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Constraint | `uml:create-model.constraint` |  | ✅ | Model › Add › Constraint in the Model menu and the explorer Add menu, on any model element; the constraint is owned by it (src/ui.rs:472-475) | extensions/essential/uml/menus/uml.json |
| Model › Apply Profile › UML Standard Profile (v2) | `uml:apply-profile.uml-standard` |  | 🟡 | adds a UMLStandardProfile with the 36 stereotypes (no metaclasses, icons or diagrams); refused with a toast when it exists (src/ui.rs:1737-1742; src/app.rs:698-713) | extensions/essential/uml/menus/uml.json |
| File › New From Template › Wireframe Model | `wireframe:new-from-template` |  | ❌ | no templates | extensions/essential/wireframe/menus/wireframe.json |
| Model › Add Diagram › Wireframe Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/wireframe/menus/wireframe.json |
| Model › Add › Wireframe | `factory:create-model` |  | ❌ |  | extensions/essential/wireframe/menus/wireframe.json |
| File › New | `application:new` | `ctrl-n` | ✅ | File › New, Ctrl+N (src/ui.rs:1555, 2181) | resources/default/menus/linux.json |
| File › New From Template | `application:new-from-template` |  | 🟡 | the File menu has New From Template with the four UML templates; the other notations' templates are missing (src/ui.rs:1559-1569) | resources/default/menus/linux.json |
| File › Open... | `project:open` | `ctrl-o` | ✅ | File › Open, Ctrl+O (src/ui.rs:1570, 2184) | resources/default/menus/linux.json |
| File › Open Recent |  |  | ✅ | the 10 most recent files, kept in the preferences file (src/ui.rs:1576; src/prefs.rs:138-150) | resources/default/menus/linux.json |
| File › Save | `project:save` | `ctrl-s` | ✅ | Ctrl+S (src/ui.rs:1586, 2190) | resources/default/menus/linux.json |
| File › Save As... | `project:save-as` | `ctrl-shift-s` | ✅ | Ctrl+Shift+S (src/ui.rs:1590, 2187) | resources/default/menus/linux.json |
| File › Close | `project:close` |  | 🟡 | File › Close starts a new blank project instead of closing (src/ui.rs:1595) | resources/default/menus/linux.json |
| File › Import › Fragment... | `project:import-fragment` |  | ✅ | picks a `.mfj` and adds it to the project (src/ui.rs:1615-1620; src/fragment.rs:34-40) | resources/default/menus/linux.json |
| File › Export › Fragment... | `project:export-fragment` |  | ✅ | saves the selected element, or one picked in the Element Picker, as `Fragment.mfj` (src/ui.rs:1626-1630; src/fragment.rs:43-66) | resources/default/menus/linux.json |
| File › Export Diagram As › PNG... | `project:export-diagram-to-png` |  | ✅ | File › Export Diagram As › PNG... and the toolbar button (src/ui.rs:1600-1639, 2409-2424; src/export.rs:42) | resources/default/menus/linux.json |
| File › Export Diagram As › JPEG... | `project:export-diagram-to-jpeg` |  | ✅ | File › Export Diagram As › JPEG..., on a white background (src/ui.rs:1600-1639, 2409; src/export.rs:42-61) | resources/default/menus/linux.json |
| File › Export Diagram As › SVG... | `project:export-diagram-to-svg` |  | ✅ | File › Export Diagram As › SVG... (src/ui.rs:1600-1639, 2409; src/export.rs:42) | resources/default/menus/linux.json |
| File › Export Diagram As › All to PNGs... | `project:export-diagram-all-to-pngs` |  | ✅ | every diagram into <folder>/png/ as filenamify(path)_<index> (src/ui.rs:2427-2440; src/export.rs:73-96) | resources/default/menus/linux.json |
| File › Export Diagram As › All to JPEGs... | `project:export-diagram-all-to-jpegs` |  | ✅ | every diagram into <folder>/jpg/ as filenamify(path)_<index> (src/ui.rs:2427-2440; src/export.rs:73-96) | resources/default/menus/linux.json |
| File › Export Diagram As › All to SVGs... | `project:export-diagram-all-to-svgs` |  | ✅ | every diagram into <folder>/svg/ as filenamify(path)_<index> (src/ui.rs:2427-2440; src/export.rs:73-96) | resources/default/menus/linux.json |
| File › Preferences... | `application:preferences` |  | ✅ | File › Preferences... opens the Preferences window (src/ui.rs:1645; src/prefs.rs:357) | resources/default/menus/linux.json |
| File › Print to PDF... | `project:print-to-pdf` |  | ✅ | opens the Print to PDF dialog (src/ui.rs:1640-1641; src/pdf.rs:210) | resources/default/menus/linux.json |
| File › Exit |  |  | ✅ | File › Exit, Ctrl+Q, with unsaved-changes prompt (src/ui.rs:1650, 2193, 2756) | resources/default/menus/linux.json |
| Edit › Undo | `edit:undo` | `ctrl-z` | ✅ | Ctrl+Z (src/ui.rs:2207) | resources/default/menus/linux.json |
| Edit › Redo | `edit:redo` | `ctrl-y` | ✅ | Ctrl+Y and Ctrl+Shift+Z (src/ui.rs:2204) | resources/default/menus/linux.json |
| Edit › Cut | `edit:cut` | `ctrl-x` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:2040, 2214) | resources/default/menus/linux.json |
| Edit › Copy | `edit:copy` | `ctrl-c` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:2040, 2214) | resources/default/menus/linux.json |
| Edit › Copy Diagram As Image | `edit:copy-diagram-as-image` | `ctrl-shift-c` | ✅ | Ctrl+Shift+C and Edit menu: deselects, draws the diagram as PNG export does and puts the picture on the system clipboard, white background outside macOS (src/clipboard.rs:165; src/ui.rs:2050, 2225, 2404) | resources/default/menus/linux.json |
| Edit › Paste | `edit:paste` | `ctrl-v` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:2040, 2214) | resources/default/menus/linux.json |
| Edit › Delete | `edit:delete` | `delete` | ✅ | Delete key and Edit menu (src/canvas.rs:684, src/ui.rs:1666) | resources/default/menus/linux.json |
| Edit › Delete From Model | `edit:delete-from-model` | `ctrl-delete` | ✅ | Ctrl+Delete (src/ui.rs:2231) | resources/default/menus/linux.json |
| Edit › Move Up | `edit:move-up` | `ctrl-shift-up` | ✅ | Ctrl+Shift+Up, Edit and explorer menus: swaps with the previous sibling of the same explorer group (src/app.rs:1536; src/ui.rs:2027, 2249) | resources/default/menus/linux.json |
| Edit › Move Down | `edit:move-down` | `ctrl-shift-down` | ✅ | Ctrl+Shift+Down, Edit and explorer menus: swaps with the next sibling of the same explorer group (src/app.rs:1536; src/ui.rs:2027, 2249) | resources/default/menus/linux.json |
| Edit › Select All | `edit:select-all` | `ctrl-a` | ✅ | Ctrl+A (src/ui.rs:2234) | resources/default/menus/linux.json |
| Edit › Select In Explorer | `edit:select-in-explorer` | `ctrl-e` | ✅ | Ctrl+E (src/ui.rs:2237) | resources/default/menus/linux.json |
| Edit › Select In Diagram | `edit:select-in-diagram` | `ctrl-d` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1576; src/ui.rs:2246, 1685, 552; src/canvas.rs:1971) | resources/default/menus/linux.json |
| Edit › Open Sub-Diagram | `edit:open-sub-diagram` | `ctrl-shift-d` | ✅ | Ctrl+Shift+D, Edit and canvas menus: opens the first diagram the element owns, or toasts "No sub-diagram in the selected element" (src/app.rs:1554; src/ui.rs:1689, 2243; src/canvas.rs:1976) | resources/default/menus/linux.json |
| Format › Font... | `format:font` | `ctrl-shift-f` | ✅ | Ctrl+Shift+F and Format › Font... (main and canvas menus) open the Font window: face, size and colour pre-filled with the merged values. The face is stored but drawn with the Arial-compatible font (src/ui.rs:2144, 1697, 2559-2638; src/style.rs:140) | resources/default/menus/linux.json |
| Format › Fill Color... | `format:fill-color` | `ctrl-shift-i` | ✅ | Ctrl+Shift+I opens the Color dialog (picker with opacity, hex field, preset palette, Transparent; OK applies); Format › Fill Color (main and canvas menus) has a colour picker; both apply to the selected views (src/ui.rs:2176-2179, 2007, 2661-2751; src/style.rs:77-84) | resources/default/menus/linux.json |
| Format › Line Color... | `format:line-color` | `ctrl-shift-l` | ✅ | Ctrl+Shift+L opens the Color dialog; Format › Line Color has a colour picker (src/ui.rs:2176-2179, 2007, 2661-2751; src/style.rs:77-84) | resources/default/menus/linux.json |
| Format › Line Style › Rectilinear | `format:linestyle-rectilinear` | `ctrl-l` | ✅ | Ctrl+L and a check item in Format › Line Style (main and canvas menus) (src/app.rs:1639; src/ui.rs:2010, 2274) | resources/default/menus/linux.json |
| Format › Line Style › Oblique | `format:linestyle-oblique` | `ctrl-b` | ✅ | Ctrl+B and a check item in Format › Line Style (src/app.rs:1639; src/ui.rs:2010, 2277) | resources/default/menus/linux.json |
| Format › Line Style › RoundRect | `format:linestyle-roundrect` | `ctrl-alt-l` | ✅ | Ctrl+Alt+L and Format › Line Style › Rounded Rectilinear: routed like rectilinear, drawn with rounded corners (radius estimated) (src/ui.rs:2010, 2258; src/views.rs:1447) | resources/default/menus/linux.json |
| Format › Line Style › Curve | `format:linestyle-curve` | `ctrl-alt-b` | ✅ | Ctrl+Alt+B and Format › Line Style › Curve: drawn as a smooth curve bending towards each point (curve shape estimated) (src/ui.rs:2010, 2267; src/views.rs:1447) | resources/default/menus/linux.json |
| Format › Auto Resize | `format:auto-resize` | `ctrl-shift-r` | ✅ | Ctrl+Shift+R and a check item in Format (main and canvas menus) and Styles: inverts the shared autoResize, or sets it when mixed (src/ui.rs:2166, 2801; src/app.rs:1648-1679) | resources/default/menus/linux.json |
| Format › Show Shadow | `format:show-shadow` | `ctrl-shift-h` | ✅ | Ctrl+Shift+H and a check item in Format (main and canvas menus): inverts the shared value, or sets it when mixed (src/app.rs:1670, 1686; src/ui.rs:2280, 2095) | resources/default/menus/linux.json |
| Format › Inherit Style | `format:inherit-style` | `ctrl-shift-k` | ✅ | Ctrl+Shift+K and a check item in Format (main and canvas menus) after Show Shadow: sets parentStyle, inverting the shared value, or `true` when mixed (src/app.rs:1684; src/ui.rs:2181, 2841) | resources/default/menus/linux.json |
| Model › Add Diagram | `model:add-diagram` |  | 🟡 | Model › Add Diagram and the explorer menu offer Class, Package, Component, Deployment and Use Case diagrams, the kinds UMLCraft edits (src/ui.rs:231, 533, 1724) | resources/default/menus/linux.json |
| Model › Add | `model:add` |  | 🟡 | Model › Add and the explorer Add menu offer Model, Package, Profile, Class, Interface, Signal, DataType, PrimitiveType, Enumeration, Stereotype, Actor, Use Case, the behaviours (Interaction, State Machine, Activity, Opaque Behavior) and the features that apply (src/ui.rs:443-522) | resources/default/menus/linux.json |
| Model › Tag Editor... | `model:tag-editor` | `ctrl-t` | ✅ | Ctrl+T, Model › Tag Editor..., and the canvas and explorer menus (src/ui.rs:1744, 2300; src/tags.rs:171-275) | resources/default/menus/linux.json |
| Tools › Extension Manager... | `tools:extension-manager` |  | ➖ | UMLCraft has no extension system | resources/default/menus/linux.json |
| View › Quick Find... | `view:quick-find` | `ctrl-p` | ✅ | Ctrl+P and View › Quick Find... (src/ui.rs:1775, 2303; src/palette.rs:182-217, 346) | resources/default/menus/linux.json |
| View › Command Palette... | `view:command-palette` | `ctrl-shift-p` | ✅ | Ctrl+Shift+P and View › Command Palette... (src/ui.rs:2167; src/palette.rs:63-174, 227-344) | resources/default/menus/linux.json |
| View › Close Diagram | `view:close-diagram` | `f4` | ✅ | F4, View menu and Working Diagrams menu; the diagram that slides into its place, or the first, becomes current (src/ui.rs:2334, 2061; src/app.rs:1437) | resources/default/menus/linux.json |
| View › Close Other Diagrams | `view:close-other-diagrams` | `ctrl-f4` | ✅ | Ctrl+F4, View menu and Working Diagrams menu (src/ui.rs:2331, 2061; src/app.rs:1498) | resources/default/menus/linux.json |
| View › Close All Diagrams | `view:close-all-diagrams` | `shift-f4` | ✅ | Shift+F4, View menu and Working Diagrams menu (src/ui.rs:2321, 2061; src/app.rs:1503) | resources/default/menus/linux.json |
| View › Next Diagram | `view:next-diagram` | `ctrl-shift-]` | ✅ | Ctrl+Shift+] (also reported as Ctrl+Shift+}), wraps to the first (src/ui.rs:2338, 1787; src/app.rs:1513) | resources/default/menus/linux.json |
| View › Previous Diagram | `view:previous-diagram` | `ctrl-shift-[` | ✅ | Ctrl+Shift+[ (also reported as Ctrl+Shift+{), wraps to the last (src/ui.rs:2341, 1791; src/app.rs:1513) | resources/default/menus/linux.json |
| View › Zoom In | `view:zoom-in` | `ctrl-plus` | ✅ | Ctrl+= / Ctrl++, keeps the screen centre (src/ui.rs:2283; src/canvas.rs:2050) | resources/default/menus/linux.json |
| View › Zoom Out | `view:zoom-out` | `ctrl-minus` | ✅ | Ctrl+-, keeps the screen centre (src/ui.rs:2286; src/canvas.rs:2050) | resources/default/menus/linux.json |
| View › Actual Size | `view:actual-size` | `ctrl-0` | ✅ | Ctrl+0, keeps the screen centre (src/ui.rs:2289; src/canvas.rs:2050) | resources/default/menus/linux.json |
| View › Fit To Window | `view:fit-to-window` | `ctrl-9` | ✅ | Ctrl+9, centred on the bounding box (src/ui.rs:2310; src/canvas.rs:2062) | resources/default/menus/linux.json |
| View › Show Grid | `view:show-grid` | `ctrl-g` | ✅ | Ctrl+G (src/ui.rs:2317) | resources/default/menus/linux.json |
| View › Snap to Grid | `view:snap-to-grid` | `ctrl-shift-g` | ✅ | Ctrl+Shift+G, checked before Ctrl+G so it is not taken as Show Grid (src/ui.rs:2314) | resources/default/menus/linux.json |
| View › Sidebar | `view:sidebar` | `ctrl-1` | ✅ | View › Sidebar check item and Ctrl+1; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/menus/linux.json |
| View › Navigator | `view:navigator` | `ctrl-2` | ✅ | View › Navigator check item and Ctrl+2; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/menus/linux.json |
| View › Toolbar | `view:toolbar` | `ctrl-3` | ✅ | View › Toolbar check item and Ctrl+3; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/menus/linux.json |
| View › Statusbar | `view:statusbar` | `ctrl-4` | ✅ | View › Statusbar check item and Ctrl+4; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/menus/linux.json |
| View › Toolbox | `view:toolbox` | `ctrl-5` | ✅ | View › Toolbox check item and Ctrl+5; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/menus/linux.json |
| View › Editors | `view:editors` | `ctrl-6` | ✅ | View › Editors check item and Ctrl+6; kept in the preferences file (src/ui.rs:1818-1823, 2324-2330; src/prefs.rs:85-91) | resources/default/menus/linux.json |
| Window › Minimize |  |  | ✅ | Window › Minimize (src/ui.rs:1826-1829) | resources/default/menus/linux.json |
| Window › Full Screen |  |  | ✅ | Window › Full Screen, toggling (src/ui.rs:1830-1834) | resources/default/menus/linux.json |
| Window › Bring All to Front |  |  | ✅ | Window › Bring All to Front focuses the window (src/ui.rs:1836-1839) | resources/default/menus/linux.json |
| Help › About StarUML | `help:about` |  | ✅ | Help › About UMLCraft, also in the Command Palette (src/ui.rs:1852; src/palette.rs:163, 338) | resources/default/menus/linux.json |
| Help › Check For Updates... | `help:check-for-updates` |  | ➖ | StarUML licence/update server | resources/default/menus/linux.json |
| Help › License Activation... | `help:license-activation` |  | ➖ | StarUML licensing | resources/default/menus/linux.json |
| Help › Documentation... | `help:documentation` |  | ✅ | Help › Documentation... opens the project README on GitHub, also in the Command Palette (src/ui.rs:1843-1846; src/palette.rs:161, 336) | resources/default/menus/linux.json |
| Help › Forum... | `help:forum` |  | ➖ |  | resources/default/menus/linux.json |

## Context menus

| Context | Item | Command | UMLCraft | Notes |
|---|---|---|---|---|
| diagram-canvas | *(the menu as a whole)* |  | 🟡 | the canvas menu has Cut, Copy, Copy Diagram As Image, Paste, Edit, Add Literal/Attribute/Operation, Line Style (four styles), Format (show/suppress toggles and Alignment), Select In Explorer, Select In Diagram, Open Sub-Diagram, Delete, Delete from Model; on empty canvas Paste, Select All and Fit To Window. Missing: the font, colour, stereotype-display, auto-resize and add-related items (src/canvas.rs:1915) |
| diagram-canvas | Format › Alignment › Send To Back | `alignment:send-to-back` | ✅ | Format › Alignment › Send To Back moves the selected top-level views to the start of ownedViews (src/align.rs:146; src/ui.rs:2111) |
| diagram-canvas | Format › Alignment › Bring To Front | `alignment:bring-to-front` | ✅ | Format › Alignment › Bring To Front moves the selected top-level views to the end of ownedViews (src/align.rs:146; src/ui.rs:2111) |
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
| diagram-canvas | Model › Add › Tag | `common:tag` | ✅ | Model › Add › Tag and the explorer Add menu create a string tag "Tag<n>" (src/ui.rs:489-498) |
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
| diagram-canvas | Model › Add › Constraint | `uml:create-model.constraint` | ✅ | Model › Add › Constraint in the Model menu and the explorer Add menu, on any model element; the constraint is owned by it (src/ui.rs:472-475)
| diagram-canvas | Format › Stereotype Display › None | `format:stereotype-none` | ✅ | sets stereotypeDisplay to none on the selected views; the stereotype is hidden (src/ui.rs:2080, 2147; src/style.rs:176) |
| diagram-canvas | Format › Stereotype Display › Label | `format:stereotype-label` | ✅ | sets stereotypeDisplay to label; the «stereotype» line is drawn (src/ui.rs:2080, 2147; src/style.rs:176) |
| diagram-canvas | Format › Stereotype Display › Decoration | `format:stereotype-decoration` | 🟡 | the value is set; interfaces and «boundary», «entity» and «control» classes get a small decoration icon at the top right, other stereotypes none (src/ui.rs:2080, 2147; src/style.rs:176; src/render.rs:370-398) |
| diagram-canvas | Format › Stereotype Display › Decoration with Label | `format:stereotype-decoration-label` | 🟡 | the value is set and the label drawn; interfaces and «boundary», «entity» and «control» classes get a small decoration icon at the top right, other stereotypes none (src/ui.rs:2080, 2147; src/style.rs:176; src/render.rs:370-398) |
| diagram-canvas | Format › Stereotype Display › Icon | `format:stereotype-icon` | 🟡 | the value is set; only interfaces (the ball) and «boundary», «entity» and «control» classes have an icon; profile stereotype icons are not drawn (src/ui.rs:2080, 2147; src/style.rs:176; src/views.rs:29-52) |
| diagram-canvas | Format › Stereotype Display › Icon with Label | `format:stereotype-icon-label` | 🟡 | the value is set; only interfaces (the ball) and «boundary», «entity» and «control» classes have an icon; profile stereotype icons are not drawn, and stereotype icons are not drawn (src/ui.rs:2080, 2147; src/style.rs:176; src/views.rs:29-52) |
| diagram-canvas | Format › Word Wrap | `format:word-wrap` | ✅ | check item in Format (main and canvas menus) and Styles: inverts the shared wordWrap of the selected views, or sets it when mixed. Class-like boxes then wrap their name and rows to the box width and may shrink to the widest word; notes and text stop wrapping when it is off (src/ui.rs:1944, 2108, 2842; src/views.rs:107-139, 922-1049; src/render.rs:79-102, 144) |
| diagram-canvas | Format › Show Visibility | `format:show-visibility` | ✅ | check item in Format (main and canvas menus): inverts the shared showVisibility of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Format › Show Namespace | `format:show-namespace` | ✅ | check item in Format (main and canvas menus): inverts the shared showNamespace of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Format › Show Property | `format:show-property` | ✅ | check item in Format (main and canvas menus): inverts the shared showProperty of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1944, 2108, 2838-2854; src/app.rs:1670-1698; src/views.rs:700-724) |
| diagram-canvas | Format › Show Type | `format:show-type` | ✅ | check item in Format (main and canvas menus): inverts the shared showType of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:1944, 2108, 2838-2854; src/app.rs:1670-1698; src/views.rs:700-724) |
| diagram-canvas | Format › Show Multiplicity | `format:show-multiplicity` | ✅ | check item in Format (main and canvas menus): inverts the shared showMultiplicity of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Format › Show Operation Signature | `format:show-operation-signature` | ✅ | check item in Format (main and canvas menus): inverts the shared showOperationSignature of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Format › Show Diagram Name | `format:show-diagram-name` | ✅ | Format › Show Diagram Name (main and canvas menus) toggles showDiagramName on the selected frames, which then end their name with `[diagram]`. Ctrl+Shift+D does it only when frames are selected: StarUML's UML keymap takes the key from Open Sub-Diagram, but UMLCraft keeps Open Sub-Diagram for every other selection (src/ui.rs:2233-2242, 2819; src/views.rs:547-567) |
| diagram-canvas | Format › Suppress Attributes | `format:suppress-attributes` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressAttributes of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Format › Suppress Operations | `format:suppress-operations` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressOperations of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Format › Suppress Receptions | `format:suppress-receptions` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressReceptions of the selected views, or sets it when mixed; with it off, a Receptions compartment is drawn under the operations (src/ui.rs:2852; src/app.rs:1670-1698; src/views.rs:1209-1220)
| diagram-canvas | Format › Suppress Literals | `format:suppress-literals` | ✅ | check item in Format (main and canvas menus): inverts the shared suppressLiterals of the selected views, or sets it when mixed; checked when all are on, disabled when no selected view has it (src/ui.rs:2095-2107; src/app.rs:1656-1683) |
| diagram-canvas | Add | `model:add` | 🟡 | Model › Add and the explorer Add menu offer Model, Package, Profile, Class, Interface, Signal, DataType, PrimitiveType, Enumeration, Stereotype, Actor, Use Case, the behaviours (Interaction, State Machine, Activity, Opaque Behavior) and the features that apply (src/ui.rs:443-522) |
| diagram-canvas | Add Diagram | `model:add-diagram` | 🟡 | Model › Add Diagram and the explorer menu offer Class, Package, Component, Deployment and Use Case diagrams, the kinds UMLCraft edits (src/ui.rs:231, 533, 1724) |
| diagram-canvas | Format › Font... | `format:font` | ✅ | Ctrl+Shift+F and Format › Font... (main and canvas menus) open the Font window: face, size and colour pre-filled with the merged values. The face is stored but drawn with the Arial-compatible font (src/ui.rs:2144, 1697, 2559-2638; src/style.rs:140) |
| diagram-canvas | Format › Fill Color... | `format:fill-color` | ✅ | Ctrl+Shift+I opens the Color dialog (picker with opacity, hex field, preset palette, Transparent; OK applies); Format › Fill Color (main and canvas menus) has a colour picker; both apply to the selected views (src/ui.rs:2176-2179, 2007, 2661-2751; src/style.rs:77-84)
| diagram-canvas | Format › Line Color... | `format:line-color` | ✅ | Ctrl+Shift+L opens the Color dialog; Format › Line Color has a colour picker (src/ui.rs:2176-2179, 2007, 2661-2751; src/style.rs:77-84)
| diagram-canvas | Format › Line Style › Rectilinear | `format:linestyle-rectilinear` | ✅ | Ctrl+L and a check item in Format › Line Style (main and canvas menus) (src/app.rs:1639; src/ui.rs:2010, 2274) |
| diagram-canvas | Format › Line Style › Oblique | `format:linestyle-oblique` | ✅ | Ctrl+B and a check item in Format › Line Style (src/app.rs:1639; src/ui.rs:2010, 2277) |
| diagram-canvas | Format › Line Style › RoundRect | `format:linestyle-roundrect` | ✅ | Ctrl+Alt+L and Format › Line Style › Rounded Rectilinear: routed like rectilinear, drawn with rounded corners (radius estimated) (src/ui.rs:2010, 2258; src/views.rs:1447) |
| diagram-canvas | Format › Line Style › Curve | `format:linestyle-curve` | ✅ | Ctrl+Alt+B and Format › Line Style › Curve: drawn as a smooth curve bending towards each point (curve shape estimated) (src/ui.rs:2010, 2267; src/views.rs:1447) |
| diagram-canvas | Format › Auto Resize | `format:auto-resize` | ✅ | Ctrl+Shift+R and a check item in Format (main and canvas menus) and Styles: inverts the shared autoResize, or sets it when mixed (src/ui.rs:2166, 2801; src/app.rs:1648-1679) |
| diagram-canvas | Format › Show Shadow | `format:show-shadow` | ✅ | Ctrl+Shift+H and a check item in Format (main and canvas menus): inverts the shared value, or sets it when mixed (src/app.rs:1670, 1686; src/ui.rs:2280, 2095) |
| diagram-canvas | Format › Inherit Style | `format:inherit-style` | ✅ | Ctrl+Shift+K and a check item in Format (main and canvas menus) after Show Shadow: sets parentStyle, inverting the shared value, or `true` when mixed (src/app.rs:1684; src/ui.rs:2181, 2841) |
| diagram-canvas | Cut | `edit:cut` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:2040, 2214) |
| diagram-canvas | Copy | `edit:copy` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:2040, 2214) |
| diagram-canvas | Paste | `edit:paste` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:2040, 2214) |
| diagram-canvas | Select All | `edit:select-all` | ✅ | Ctrl+A (src/ui.rs:2234) |
| diagram-canvas | Delete | `edit:delete` | ✅ | Delete key and Edit menu (src/canvas.rs:684, src/ui.rs:1666) |
| diagram-canvas | Delete From Model | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:2231) |
| diagram-canvas | Select In Explorer | `edit:select-in-explorer` | ✅ | Ctrl+E (src/ui.rs:2237) |
| diagram-canvas | Select In Diagram | `edit:select-in-diagram` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1576; src/ui.rs:2246, 1685, 552; src/canvas.rs:1971) |
| diagram-canvas | Open Sub-Diagram | `edit:open-sub-diagram` | ✅ | Ctrl+Shift+D, Edit and canvas menus: opens the first diagram the element owns, or toasts "No sub-diagram in the selected element" (src/app.rs:1554; src/ui.rs:1689, 2243; src/canvas.rs:1976) |
| diagram-canvas | Tag Editor... | `model:tag-editor` | ✅ | Ctrl+T, Model › Tag Editor..., and the canvas and explorer menus (src/ui.rs:1744, 2300; src/tags.rs:171-275) |
| mermaid-example-dropdown | Class Diagram | `mermaid:example-class-diagram` | ✅ | an "Example: Class" button fills in a class diagram example (src/mermaid.rs:468-470) |
| mermaid-example-dropdown | Sequence Diagram | `mermaid:example-sequence-diagram` | ❌ |  |
| mermaid-example-dropdown | State Diagram | `mermaid:example-state-diagram` | ❌ |  |
| mermaid-example-dropdown | Flowchart | `mermaid:example-flowchart` | ❌ |  |
| mermaid-example-dropdown | Entity Relationship Diagram | `mermaid:example-er-diagram` | ❌ |  |
| mermaid-example-dropdown | Requirement Diagram | `mermaid:example-requirement-diagram` | ❌ |  |
| mermaid-example-dropdown | Mindmap | `mermaid:example-mindmap` | ❌ |  |
| #relationship-view div.listview | Select Related Element | `relationship-view:select-related-element` | ✅ | the Relationships panel row's context menu selects the element at the other end (src/relview.rs:85-89) |
| #relationship-view div.listview | Select Relationship | `relationship-view:select-relationship` | ✅ | the Relationships panel row's context menu selects the relationship (src/relview.rs:90-93) |
| model-explorer | *(the menu as a whole)* |  | 🟡 | the explorer menu has Add (with Constraint, Reception and Extension Point where they apply), Add Diagram, Open Diagram, Tag Editor, Select In Diagram, Cut, Copy, Paste, Move Up, Move Down and Delete from Model; of the behaviour items only Interaction, State Machine, Activity and Opaque Behavior (src/ui.rs:512)
| model-explorer | Model › Add › Tag | `common:tag` | ✅ | Model › Add › Tag and the explorer Add menu create a string tag "Tag<n>" (src/ui.rs:489-498) |
| model-explorer | Model › Add › Entry Activity | `uml:create-model.entry-activity` | ❌ |  |
| model-explorer | Model › Add › Do Activity | `uml:create-model.do-activity` | ❌ |  |
| model-explorer | Model › Add › Exit Activity | `uml:create-model.exit-activity` | ❌ |  |
| model-explorer | Model › Add › Effect | `uml:create-model.effect` | ❌ |  |
| model-explorer | Model › Add › Constraint | `uml:create-model.constraint` | ✅ | Model › Add › Constraint in the Model menu and the explorer Add menu, on any model element; the constraint is owned by it (src/ui.rs:472-475)
| model-explorer | Add Diagram | `model:add-diagram` | 🟡 | Model › Add Diagram and the explorer menu offer Class, Package, Component, Deployment and Use Case diagrams, the kinds UMLCraft edits (src/ui.rs:231, 533, 1724) |
| model-explorer | Add | `model:add` | 🟡 | Model › Add and the explorer Add menu offer Model, Package, Profile, Class, Interface, Signal, DataType, PrimitiveType, Enumeration, Stereotype, Actor, Use Case, the behaviours (Interaction, State Machine, Activity, Opaque Behavior) and the features that apply (src/ui.rs:443-522) |
| model-explorer | Cut | `edit:cut` | ✅ | Ctrl+X, Edit, canvas and explorer menus: copies, then removes the views (canvas) or deletes the model (explorer) (src/clipboard.rs:198; src/ui.rs:2040, 2214) |
| model-explorer | Copy | `edit:copy` | ✅ | Ctrl+C, Edit, canvas and explorer menus: views or one model as JSON on the system clipboard, in UMLCraft's own layout (src/clipboard.rs:108, 191; src/ui.rs:2040, 2214) |
| model-explorer | Paste | `edit:paste` | ✅ | Ctrl+V, Edit, canvas and explorer menus: views as new views of the same models, centred on screen; a model as a deep copy into the selected element (src/clipboard.rs:251-332; src/ui.rs:2040, 2214) |
| model-explorer | Delete From Model | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:2231) |
| model-explorer | Move Up | `edit:move-up` | ✅ | Ctrl+Shift+Up, Edit and explorer menus: swaps with the previous sibling of the same explorer group (src/app.rs:1536; src/ui.rs:2027, 2249) |
| model-explorer | Move Down | `edit:move-down` | ✅ | Ctrl+Shift+Down, Edit and explorer menus: swaps with the next sibling of the same explorer group (src/app.rs:1536; src/ui.rs:2027, 2249) |
| model-explorer | Select In Diagram | `edit:select-in-diagram` | ✅ | Ctrl+D, Edit menu, canvas and explorer menus: one view opens its diagram, selects and centres it; several open the "Select Diagram to show" picker; none toasts "No diagrams to show" (src/app.rs:1576; src/ui.rs:2246, 1685, 552; src/canvas.rs:1971) |
| working-diagrams | *(the menu as a whole)* |  | ✅ | right-clicking a row opens Close Diagram, Close Other Diagrams, Close All Diagrams and Rename, acting on the current diagram (src/ui.rs:1309-1317, 2073) |
| working-diagrams | Close Diagram | `view:close-diagram` | ✅ | F4, View menu and Working Diagrams menu; the diagram that slides into its place, or the first, becomes current (src/ui.rs:2334, 2061; src/app.rs:1437) |
| working-diagrams | Close Other Diagrams | `view:close-other-diagrams` | ✅ | Ctrl+F4, View menu and Working Diagrams menu (src/ui.rs:2331, 2061; src/app.rs:1498) |
| working-diagrams | Close All Diagrams | `view:close-all-diagrams` | ✅ | Shift+F4, View menu and Working Diagrams menu (src/ui.rs:2321, 2061; src/app.rs:1503) |
| working-diagrams | Rename | `view:rename-diagram` | ✅ | opens "Enter diagram name" pre-filled with the current diagram's name; OK renames it as one undo step (src/ui.rs:1312-1315; src/input.rs:28-64) |
| model-explorer-settings | *(the menu as a whole)* |  | ✅ | the header settings button opens Sort By Added, Sort By Name, Show Stereotype Text, Expand All and Collapse All (src/ui.rs:587, 672-694) |
| model-explorer-settings | Sort By Added | `explorer:sort-by-added` | ✅ | radio item in the settings menu (src/ui.rs:673-676) |
| model-explorer-settings | Sort By Name | `explorer:sort-by-name` | ✅ | radio item in the settings menu (src/ui.rs:677-680, 274) |
| model-explorer-settings | Show Stereotype Text | `explorer:show-stereotype-text` | ✅ | check item, on by default; tree labels get «stereotype» first (src/ui.rs:682-684, 336-341) |
| model-explorer-settings | Expand All | `explorer:expand-all` | ✅ | opens every element that has children (src/ui.rs:686-689, 697-704) |
| model-explorer-settings | Collapse All | `explorer:collapse-all` | ✅ | closes every element, leaving only the project row (src/ui.rs:690-693) |
