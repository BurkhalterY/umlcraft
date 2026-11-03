# Menus and keyboard shortcuts

Part of the [StarUML feature inventory](README.md); the legend and how to update a status are there.

StarUML 7.1.1 main menus, context menus and keymaps, from every `menus/*.json` and `keymaps/*.json` (Linux) in the app. One row per item; a command can appear in several tables.

| Shortcut | Command | UMLCraft | Notes | Source |
|---|---|---|---|---|
| `shift-alt-t` | `debug:show-devtools` | ➖ | Electron devtools | extensions/default/debug/keymaps/debug.json |
| `ctrl-r` | `debug:reload` | ➖ | Electron reload | extensions/default/debug/keymaps/debug.json |
| `ctrl-alt-t` | `diagram-thumbnails:toggle` | ❌ |  | extensions/default/diagram-thumbnails/keymaps/diagram-thumbnails.json |
| `ctrl-f` | `find:find` | ❌ |  | extensions/default/find/keymaps/find.json |
| `ctrl-alt-d` | `markdown-doc:toggle` | ❌ |  | extensions/default/markdown/keymaps/markdown.json |
| `ctrl-alt-m` | `minimap:toggle` | 🟡 | minimap always drawn in the canvas corner (src/canvas.rs:307), cannot be toggled | extensions/default/minimap/keymaps/minimap.json |
| `ctrl-alt-r` | `relationship-view:toggle` | ❌ |  | extensions/default/relationship-view/keymaps/relationship-view.json |
| `ctrl-shift-u` | `erd:suppress-columns` | ❌ |  | extensions/essential/erd/keymaps/erd.json |
| `ctrl-shift-0` | `format:stereotype-none` | 🟡 | the view's stereotypeDisplay flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-1` | `format:stereotype-label` | 🟡 | the view's stereotypeDisplay flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-2` | `format:stereotype-decoration` | ❌ | decoration display not drawn | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-3` | `format:stereotype-decoration-label` | ❌ | decoration display not drawn | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-4` | `format:stereotype-icon` | 🟡 | the view's stereotypeDisplay (icon only for interfaces, as a ball) flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-5` | `format:stereotype-icon-label` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-w` | `format:word-wrap` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-v` | `format:show-visibility` | 🟡 | the view's showVisibility flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-n` | `format:show-namespace` | 🟡 | the view's showNamespace flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-b` | `format:show-property` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-y` | `format:show-type` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-m` | `format:show-multiplicity` | 🟡 | the view's showMultiplicity flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-j` | `format:show-operation-signature` | 🟡 | the view's showOperationSignature flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-d` | `format:show-diagram-name` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-a` | `format:suppress-attributes` | 🟡 | the view's suppressAttributes flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-o` | `format:suppress-operations` | 🟡 | the view's suppressOperations flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-e` | `format:suppress-receptions` | ❌ |  | extensions/essential/uml/keymaps/uml.json |
| `ctrl-shift-t` | `format:suppress-literals` | 🟡 | the view's suppressLiterals flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/keymaps/uml.json |
| `ctrl-n` | `application:new` | ✅ | File › New, Ctrl+N (src/ui.rs:764, 913) | resources/default/keymaps/linux.json |
| `ctrl-o` | `project:open` | ✅ | File › Open, Ctrl+O (src/ui.rs:768, 916) | resources/default/keymaps/linux.json |
| `ctrl-s` | `project:save` | ✅ | Ctrl+S (src/ui.rs:773, 922) | resources/default/keymaps/linux.json |
| `ctrl-shift-s` | `project:save-as` | ✅ | Ctrl+Shift+S (src/ui.rs:777, 919) | resources/default/keymaps/linux.json |
| `ctrl-q` | `application:quit` | ✅ | File › Exit, Ctrl+Q, with unsaved-changes prompt (src/ui.rs:796, 925, 1065) | resources/default/keymaps/linux.json |
| `ctrl-z` | `edit:undo` | ✅ | Ctrl+Z (src/ui.rs:934) | resources/default/keymaps/linux.json |
| `ctrl-y` | `edit:redo` | ✅ | Ctrl+Y and Ctrl+Shift+Z (src/ui.rs:931) | resources/default/keymaps/linux.json |
| `ctrl-x` | `edit:cut` | ❌ | no clipboard | resources/default/keymaps/linux.json |
| `ctrl-c` | `edit:copy` | ❌ | no clipboard | resources/default/keymaps/linux.json |
| `ctrl-shift-c` | `edit:copy-diagram-as-image` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-v` | `edit:paste` | ❌ | no clipboard | resources/default/keymaps/linux.json |
| `delete` | `edit:delete` | ✅ | Delete key and Edit menu (src/canvas.rs:446, src/ui.rs:811) | resources/default/keymaps/linux.json |
| `ctrl-delete` | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:937) | resources/default/keymaps/linux.json |
| `ctrl-shift-up` | `edit:move-up` | ❌ | no reordering in the explorer | resources/default/keymaps/linux.json |
| `ctrl-shift-down` | `edit:move-down` | ❌ | no reordering in the explorer | resources/default/keymaps/linux.json |
| `ctrl-a` | `edit:select-all` | ✅ | Ctrl+A (src/ui.rs:940) | resources/default/keymaps/linux.json |
| `ctrl-e` | `edit:select-in-explorer` | ✅ | Ctrl+E (src/ui.rs:943) | resources/default/keymaps/linux.json |
| `ctrl-d` | `edit:select-in-diagram` | 🟡 | clicking an explorer row selects its view in the current diagram (src/app.rs:624); no Ctrl+D, no search across diagrams | resources/default/keymaps/linux.json |
| `ctrl-shift-d` | `edit:open-sub-diagram` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-shift-f` | `format:font` | ❌ | fonts read from files but not editable | resources/default/keymaps/linux.json |
| `ctrl-shift-i` | `format:fill-color` | 🟡 | fillColor is drawn (src/render.rs:136) but cannot be changed | resources/default/keymaps/linux.json |
| `ctrl-shift-l` | `format:line-color` | 🟡 | lineColor is drawn (src/render.rs:135) but cannot be changed | resources/default/keymaps/linux.json |
| `ctrl-l` | `format:linestyle-rectilinear` | ✅ | Ctrl+L (src/app.rs:632, src/ui.rs:948) | resources/default/keymaps/linux.json |
| `ctrl-b` | `format:linestyle-oblique` | ✅ | Ctrl+B (src/ui.rs:951) | resources/default/keymaps/linux.json |
| `ctrl-alt-l` | `format:linestyle-roundrect` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-alt-b` | `format:linestyle-curve` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-shift-r` | `format:auto-resize` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-shift-h` | `format:show-shadow` | ✅ | Ctrl+Shift+H (src/app.rs:648, src/ui.rs:954) | resources/default/keymaps/linux.json |
| `ctrl-shift-k` | `format:inherit-style` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-t` | `model:tag-editor` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-p` | `view:quick-find` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-shift-p` | `view:command-palette` | ❌ |  | resources/default/keymaps/linux.json |
| `f4` | `view:close-diagram` | ✅ | F4 (src/ui.rs:975) | resources/default/keymaps/linux.json |
| `ctrl-f4` | `view:close-other-diagrams` | ❌ |  | resources/default/keymaps/linux.json |
| `shift-f4` | `view:close-all-diagrams` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-shift-]` | `view:next-diagram` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-shift-[` | `view:previous-diagram` | ❌ |  | resources/default/keymaps/linux.json |
| `ctrl-plus` | `view:zoom-in` | ✅ | Ctrl+= / Ctrl++ (src/ui.rs:957) | resources/default/keymaps/linux.json |
| `ctrl-minus` | `view:zoom-out` | ✅ | Ctrl+- (src/ui.rs:960) | resources/default/keymaps/linux.json |
| `ctrl-0` | `view:actual-size` | ✅ | Ctrl+0 (src/ui.rs:963) | resources/default/keymaps/linux.json |
| `ctrl-9` | `view:fit-to-window` | ✅ | Ctrl+9 (src/canvas.rs:998) | resources/default/keymaps/linux.json |
| `ctrl-g` | `view:show-grid` | ✅ | Ctrl+G (src/ui.rs:969) | resources/default/keymaps/linux.json |
| `ctrl-shift-g` | `view:snap-to-grid` | ✅ | Ctrl+Shift+G (src/ui.rs:972) | resources/default/keymaps/linux.json |
| `ctrl-1` | `view:sidebar` | 🟡 | View menu toggle, no Ctrl+1 (src/ui.rs:893) | resources/default/keymaps/linux.json |
| `ctrl-2` | `view:navigator` | 🟡 | View menu toggle, no Ctrl+2 (src/ui.rs:894) | resources/default/keymaps/linux.json |
| `ctrl-3` | `view:toolbar` | ❌ | toolbar always shown, no Ctrl+3 | resources/default/keymaps/linux.json |
| `ctrl-4` | `view:statusbar` | 🟡 | View menu toggle, no Ctrl+4 (src/ui.rs:895) | resources/default/keymaps/linux.json |
| `ctrl-5` | `view:toolbox` | ❌ | toolbox cannot be hidden on its own, no Ctrl+5 | resources/default/keymaps/linux.json |
| `ctrl-6` | `view:editors` | ❌ | editors cannot be hidden on their own, no Ctrl+6 | resources/default/keymaps/linux.json |

## Main menu items

| Menu path | Command | Shortcut | UMLCraft | Notes | Source |
|---|---|---|---|---|---|
| Format › Alignment › Send To Back | `alignment:send-to-back` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Bring To Front | `alignment:bring-to-front` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Left | `alignment:align-left` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Right | `alignment:align-right` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Center | `alignment:align-center` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Top | `alignment:align-top` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Bottom | `alignment:align-bottom` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Align Middle | `alignment:align-middle` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Space Equally, Horizontally | `alignment:space-equally-horizontally` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Space Equally, Vertically | `alignment:space-equally-vertically` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Set Width Equally | `alignment:set-width-equally` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Set Height Equally | `alignment:set-height-equally` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
| Format › Alignment › Set Size Equally | `alignment:set-size-equally` |  | ❌ |  | extensions/default/alignment/menus/alignment.json |
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
| Model › Find | `find:find` | `ctrl-f` | ❌ |  | extensions/default/find/menus/find.json |
| File › Export › HTML Docs... | `html-export:export` |  | ❌ |  | extensions/default/html-export/menus/html-export.json |
| View › Markdown Documentation | `markdown-doc:toggle` | `ctrl-alt-d` | ❌ |  | extensions/default/markdown/menus/markdown.json |
| Tools › Generate Diagram by Mermaid... | `mermaid:show-mermaid-dialog` |  | ❌ |  | extensions/default/mermaid/menus/mermaid.json |
| View › Minimap | `minimap:toggle` | `ctrl-alt-m` | 🟡 | minimap always drawn in the canvas corner (src/canvas.rs:307), cannot be toggled | extensions/default/minimap/menus/minimap.json |
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
| Model › Add › Tag | `common:tag` |  | 🟡 | Tags are listed in the explorer ("name = value", src/ui.rs:169) but cannot be added | extensions/essential/common/menus/common.json |
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
| Model › Add › Parameter | `factory:create-model` |  | ✅ | explorer Add (operations), Editors button (src/ui.rs:243) | extensions/essential/sysml/menus/sysml.json |
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
| Format › Stereotype Display › None | `format:stereotype-none` | `ctrl-shift-0` | 🟡 | the view's stereotypeDisplay flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Label | `format:stereotype-label` | `ctrl-shift-1` | 🟡 | the view's stereotypeDisplay flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Decoration | `format:stereotype-decoration` | `ctrl-shift-2` | ❌ | decoration display not drawn | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Decoration with Label | `format:stereotype-decoration-label` | `ctrl-shift-3` | ❌ | decoration display not drawn | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Icon | `format:stereotype-icon` | `ctrl-shift-4` | 🟡 | the view's stereotypeDisplay (icon only for interfaces, as a ball) flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Stereotype Display › Icon with Label | `format:stereotype-icon-label` | `ctrl-shift-5` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Word Wrap | `format:word-wrap` | `ctrl-shift-w` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Show Visibility | `format:show-visibility` | `ctrl-shift-v` | 🟡 | the view's showVisibility flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Show Namespace | `format:show-namespace` | `ctrl-shift-n` | 🟡 | the view's showNamespace flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Show Property | `format:show-property` | `ctrl-shift-b` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Show Type | `format:show-type` | `ctrl-shift-y` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Show Multiplicity | `format:show-multiplicity` | `ctrl-shift-m` | 🟡 | the view's showMultiplicity flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Show Operation Signature | `format:show-operation-signature` | `ctrl-shift-j` | 🟡 | the view's showOperationSignature flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Show Diagram Name | `format:show-diagram-name` | `ctrl-shift-d` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Suppress Attributes | `format:suppress-attributes` | `ctrl-shift-a` | 🟡 | the view's suppressAttributes flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Suppress Operations | `format:suppress-operations` | `ctrl-shift-o` | 🟡 | the view's suppressOperations flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Format › Suppress Receptions | `format:suppress-receptions` | `ctrl-shift-e` | ❌ |  | extensions/essential/uml/menus/uml.json |
| Format › Suppress Literals | `format:suppress-literals` | `ctrl-shift-t` | 🟡 | the view's suppressLiterals flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Class Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:275) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Package Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Object Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Composite Structure Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Component Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Deployment Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Use Case Diagram | `factory:create-diagram` |  | ✅ | (src/ui.rs:275) | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Sequence Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Communication Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Timing Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Interaction Overview Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Information Flow Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Statechart Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Activity Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add Diagram › Profile Diagram | `factory:create-diagram` |  | ❌ | diagram kind not editable | extensions/essential/uml/menus/uml.json |
| Model › Add › Template Parameter | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Template Parameter Substitution | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Enumeration Literal | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Attribute | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Operation | `factory:create-model` |  | ✅ | explorer Add, Model menu, Editors button (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Reception | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Extension Point | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Slot | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Interaction Operand | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Model | `factory:create-model` |  | ✅ | explorer Add (Project only) (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Subsystem | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Package | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Profile | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Class | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Interface | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Signal | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › DataType | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › PrimitiveType | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Enumeration | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Artifact | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Component | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › Node | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
| Model › Add › UseCase | `factory:create-model` |  | ✅ | explorer Add (Use Case) (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Actor | `factory:create-model` |  | ✅ | explorer Add (src/ui.rs:243) | extensions/essential/uml/menus/uml.json |
| Model › Add › Stereotype | `factory:create-model` |  | ❌ |  | extensions/essential/uml/menus/uml.json |
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
| File › New | `application:new` | `ctrl-n` | ✅ | File › New, Ctrl+N (src/ui.rs:764, 913) | resources/default/menus/linux.json |
| File › New From Template | `application:new-from-template` |  | ❌ | no templates | resources/default/menus/linux.json |
| File › Open... | `project:open` | `ctrl-o` | ✅ | File › Open, Ctrl+O (src/ui.rs:768, 916) | resources/default/menus/linux.json |
| File › Open Recent |  |  | ❌ |  | resources/default/menus/linux.json |
| File › Save | `project:save` | `ctrl-s` | ✅ | Ctrl+S (src/ui.rs:773, 922) | resources/default/menus/linux.json |
| File › Save As... | `project:save-as` | `ctrl-shift-s` | ✅ | Ctrl+Shift+S (src/ui.rs:777, 919) | resources/default/menus/linux.json |
| File › Close | `project:close` |  | 🟡 | File › Close starts a new blank project instead of closing (src/ui.rs:782) | resources/default/menus/linux.json |
| File › Import › Fragment... | `project:import-fragment` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export › Fragment... | `project:export-fragment` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export Diagram As › PNG... | `project:export-diagram-to-png` |  | ✅ | File › Export Diagram As PNG and toolbar button (src/ui.rs:787, src/render.rs:453) | resources/default/menus/linux.json |
| File › Export Diagram As › JPEG... | `project:export-diagram-to-jpeg` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export Diagram As › SVG... | `project:export-diagram-to-svg` |  | ✅ | (src/ui.rs:791, src/render.rs:446) | resources/default/menus/linux.json |
| File › Export Diagram As › All to PNGs... | `project:export-diagram-all-to-pngs` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export Diagram As › All to JPEGs... | `project:export-diagram-all-to-jpegs` |  | ❌ |  | resources/default/menus/linux.json |
| File › Export Diagram As › All to SVGs... | `project:export-diagram-all-to-svgs` |  | ❌ |  | resources/default/menus/linux.json |
| File › Preferences... | `application:preferences` |  | ❌ | no preferences dialog | resources/default/menus/linux.json |
| File › Print to PDF... | `project:print-to-pdf` |  | ❌ |  | resources/default/menus/linux.json |
| File › Exit |  |  | ✅ | File › Exit, Ctrl+Q, with unsaved-changes prompt (src/ui.rs:796, 925, 1065) | resources/default/menus/linux.json |
| Edit › Undo | `edit:undo` | `ctrl-z` | ✅ | Ctrl+Z (src/ui.rs:934) | resources/default/menus/linux.json |
| Edit › Redo | `edit:redo` | `ctrl-y` | ✅ | Ctrl+Y and Ctrl+Shift+Z (src/ui.rs:931) | resources/default/menus/linux.json |
| Edit › Cut | `edit:cut` | `ctrl-x` | ❌ | no clipboard | resources/default/menus/linux.json |
| Edit › Copy | `edit:copy` | `ctrl-c` | ❌ | no clipboard | resources/default/menus/linux.json |
| Edit › Copy Diagram As Image | `edit:copy-diagram-as-image` | `ctrl-shift-c` | ❌ |  | resources/default/menus/linux.json |
| Edit › Paste | `edit:paste` | `ctrl-v` | ❌ | no clipboard | resources/default/menus/linux.json |
| Edit › Delete | `edit:delete` | `delete` | ✅ | Delete key and Edit menu (src/canvas.rs:446, src/ui.rs:811) | resources/default/menus/linux.json |
| Edit › Delete From Model | `edit:delete-from-model` | `ctrl-delete` | ✅ | Ctrl+Delete (src/ui.rs:937) | resources/default/menus/linux.json |
| Edit › Move Up | `edit:move-up` | `ctrl-shift-up` | ❌ | no reordering in the explorer | resources/default/menus/linux.json |
| Edit › Move Down | `edit:move-down` | `ctrl-shift-down` | ❌ | no reordering in the explorer | resources/default/menus/linux.json |
| Edit › Select All | `edit:select-all` | `ctrl-a` | ✅ | Ctrl+A (src/ui.rs:940) | resources/default/menus/linux.json |
| Edit › Select In Explorer | `edit:select-in-explorer` | `ctrl-e` | ✅ | Ctrl+E (src/ui.rs:943) | resources/default/menus/linux.json |
| Edit › Select In Diagram | `edit:select-in-diagram` | `ctrl-d` | 🟡 | clicking an explorer row selects its view in the current diagram (src/app.rs:624); no Ctrl+D, no search across diagrams | resources/default/menus/linux.json |
| Edit › Open Sub-Diagram | `edit:open-sub-diagram` | `ctrl-shift-d` | ❌ |  | resources/default/menus/linux.json |
| Format › Font... | `format:font` | `ctrl-shift-f` | ❌ | fonts read from files but not editable | resources/default/menus/linux.json |
| Format › Fill Color... | `format:fill-color` | `ctrl-shift-i` | 🟡 | fillColor is drawn (src/render.rs:136) but cannot be changed | resources/default/menus/linux.json |
| Format › Line Color... | `format:line-color` | `ctrl-shift-l` | 🟡 | lineColor is drawn (src/render.rs:135) but cannot be changed | resources/default/menus/linux.json |
| Format › Line Style › Rectilinear | `format:linestyle-rectilinear` | `ctrl-l` | ✅ | Ctrl+L (src/app.rs:632, src/ui.rs:948) | resources/default/menus/linux.json |
| Format › Line Style › Oblique | `format:linestyle-oblique` | `ctrl-b` | ✅ | Ctrl+B (src/ui.rs:951) | resources/default/menus/linux.json |
| Format › Line Style › RoundRect | `format:linestyle-roundrect` | `ctrl-alt-l` | ❌ |  | resources/default/menus/linux.json |
| Format › Line Style › Curve | `format:linestyle-curve` | `ctrl-alt-b` | ❌ |  | resources/default/menus/linux.json |
| Format › Auto Resize | `format:auto-resize` | `ctrl-shift-r` | ❌ |  | resources/default/menus/linux.json |
| Format › Show Shadow | `format:show-shadow` | `ctrl-shift-h` | ✅ | Ctrl+Shift+H (src/app.rs:648, src/ui.rs:954) | resources/default/menus/linux.json |
| Format › Inherit Style | `format:inherit-style` | `ctrl-shift-k` | ❌ |  | resources/default/menus/linux.json |
| Model › Add Diagram | `model:add-diagram` |  | 🟡 | Class and Use Case diagrams only (src/ui.rs:274, 848) | resources/default/menus/linux.json |
| Model › Add | `model:add` |  | 🟡 | explorer Add submenu with 8 kinds plus features (src/ui.rs:243) | resources/default/menus/linux.json |
| Model › Tag Editor... | `model:tag-editor` | `ctrl-t` | ❌ |  | resources/default/menus/linux.json |
| Tools › Extension Manager... | `tools:extension-manager` |  | ➖ | UMLCraft has no extension system | resources/default/menus/linux.json |
| View › Quick Find... | `view:quick-find` | `ctrl-p` | ❌ |  | resources/default/menus/linux.json |
| View › Command Palette... | `view:command-palette` | `ctrl-shift-p` | ❌ |  | resources/default/menus/linux.json |
| View › Close Diagram | `view:close-diagram` | `f4` | ✅ | F4 (src/ui.rs:975) | resources/default/menus/linux.json |
| View › Close Other Diagrams | `view:close-other-diagrams` | `ctrl-f4` | ❌ |  | resources/default/menus/linux.json |
| View › Close All Diagrams | `view:close-all-diagrams` | `shift-f4` | ❌ |  | resources/default/menus/linux.json |
| View › Next Diagram | `view:next-diagram` | `ctrl-shift-]` | ❌ |  | resources/default/menus/linux.json |
| View › Previous Diagram | `view:previous-diagram` | `ctrl-shift-[` | ❌ |  | resources/default/menus/linux.json |
| View › Zoom In | `view:zoom-in` | `ctrl-plus` | ✅ | Ctrl+= / Ctrl++ (src/ui.rs:957) | resources/default/menus/linux.json |
| View › Zoom Out | `view:zoom-out` | `ctrl-minus` | ✅ | Ctrl+- (src/ui.rs:960) | resources/default/menus/linux.json |
| View › Actual Size | `view:actual-size` | `ctrl-0` | ✅ | Ctrl+0 (src/ui.rs:963) | resources/default/menus/linux.json |
| View › Fit To Window | `view:fit-to-window` | `ctrl-9` | ✅ | Ctrl+9 (src/canvas.rs:998) | resources/default/menus/linux.json |
| View › Show Grid | `view:show-grid` | `ctrl-g` | ✅ | Ctrl+G (src/ui.rs:969) | resources/default/menus/linux.json |
| View › Snap to Grid | `view:snap-to-grid` | `ctrl-shift-g` | ✅ | Ctrl+Shift+G (src/ui.rs:972) | resources/default/menus/linux.json |
| View › Sidebar | `view:sidebar` | `ctrl-1` | 🟡 | View menu toggle, no Ctrl+1 (src/ui.rs:893) | resources/default/menus/linux.json |
| View › Navigator | `view:navigator` | `ctrl-2` | 🟡 | View menu toggle, no Ctrl+2 (src/ui.rs:894) | resources/default/menus/linux.json |
| View › Toolbar | `view:toolbar` | `ctrl-3` | ❌ | toolbar always shown, no Ctrl+3 | resources/default/menus/linux.json |
| View › Statusbar | `view:statusbar` | `ctrl-4` | 🟡 | View menu toggle, no Ctrl+4 (src/ui.rs:895) | resources/default/menus/linux.json |
| View › Toolbox | `view:toolbox` | `ctrl-5` | ❌ | toolbox cannot be hidden on its own, no Ctrl+5 | resources/default/menus/linux.json |
| View › Editors | `view:editors` | `ctrl-6` | ❌ | editors cannot be hidden on their own, no Ctrl+6 | resources/default/menus/linux.json |
| Window › Minimize |  |  | 🟡 | left to the window manager | resources/default/menus/linux.json |
| Window › Full Screen |  |  | 🟡 | left to the window manager | resources/default/menus/linux.json |
| Window › Bring All to Front |  |  | 🟡 | left to the window manager | resources/default/menus/linux.json |
| Help › About StarUML | `help:about` |  | ✅ | Help › About UMLCraft (src/ui.rs:897) | resources/default/menus/linux.json |
| Help › Check For Updates... | `help:check-for-updates` |  | ➖ | StarUML licence/update server | resources/default/menus/linux.json |
| Help › License Activation... | `help:license-activation` |  | ➖ | StarUML licensing | resources/default/menus/linux.json |
| Help › Documentation... | `help:documentation` |  | ❌ |  | resources/default/menus/linux.json |
| Help › Forum... | `help:forum` |  | ➖ |  | resources/default/menus/linux.json |

## Context menus

| Context | Item | Command | UMLCraft | Notes |
|---|---|---|---|---|
| diagram-canvas | *(the menu as a whole)* |  | 🟡 | the canvas menu has Edit, Add Literal/Attribute/Operation, Line Style (two styles), Select In Explorer, Delete, Delete from Model; on empty canvas Select All and Fit To Window (src/canvas.rs:934) |
| diagram-canvas | Format › Alignment › Send To Back | `alignment:send-to-back` | ❌ |  |
| diagram-canvas | Format › Alignment › Bring To Front | `alignment:bring-to-front` | ❌ |  |
| diagram-canvas | Format › Alignment › Align Left | `alignment:align-left` | ❌ |  |
| diagram-canvas | Format › Alignment › Align Right | `alignment:align-right` | ❌ |  |
| diagram-canvas | Format › Alignment › Align Center | `alignment:align-center` | ❌ |  |
| diagram-canvas | Format › Alignment › Align Top | `alignment:align-top` | ❌ |  |
| diagram-canvas | Format › Alignment › Align Bottom | `alignment:align-bottom` | ❌ |  |
| diagram-canvas | Format › Alignment › Align Middle | `alignment:align-middle` | ❌ |  |
| diagram-canvas | Format › Alignment › Space Equally, Horizontally | `alignment:space-equally-horizontally` | ❌ |  |
| diagram-canvas | Format › Alignment › Space Equally, Vertically | `alignment:space-equally-vertically` | ❌ |  |
| diagram-canvas | Format › Alignment › Set Width Equally | `alignment:set-width-equally` | ❌ |  |
| diagram-canvas | Format › Alignment › Set Height Equally | `alignment:set-height-equally` | ❌ |  |
| diagram-canvas | Format › Alignment › Set Size Equally | `alignment:set-size-equally` | ❌ |  |
| diagram-canvas | Model › Add › Tag | `common:tag` | 🟡 | Tags are listed in the explorer ("name = value", src/ui.rs:169) but cannot be added |
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
| diagram-canvas | Format › Stereotype Display › None | `format:stereotype-none` | 🟡 | the view's stereotypeDisplay flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Stereotype Display › Label | `format:stereotype-label` | 🟡 | the view's stereotypeDisplay flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Stereotype Display › Decoration | `format:stereotype-decoration` | ❌ | decoration display not drawn |
| diagram-canvas | Format › Stereotype Display › Decoration with Label | `format:stereotype-decoration-label` | ❌ | decoration display not drawn |
| diagram-canvas | Format › Stereotype Display › Icon | `format:stereotype-icon` | 🟡 | the view's stereotypeDisplay (icon only for interfaces, as a ball) flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Stereotype Display › Icon with Label | `format:stereotype-icon-label` | ❌ |  |
| diagram-canvas | Format › Word Wrap | `format:word-wrap` | ❌ |  |
| diagram-canvas | Format › Show Visibility | `format:show-visibility` | 🟡 | the view's showVisibility flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Show Namespace | `format:show-namespace` | 🟡 | the view's showNamespace flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Show Property | `format:show-property` | ❌ |  |
| diagram-canvas | Format › Show Type | `format:show-type` | ❌ |  |
| diagram-canvas | Format › Show Multiplicity | `format:show-multiplicity` | 🟡 | the view's showMultiplicity flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Show Operation Signature | `format:show-operation-signature` | 🟡 | the view's showOperationSignature flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Show Diagram Name | `format:show-diagram-name` | ❌ |  |
| diagram-canvas | Format › Suppress Attributes | `format:suppress-attributes` | 🟡 | the view's suppressAttributes flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Suppress Operations | `format:suppress-operations` | 🟡 | the view's suppressOperations flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Format › Suppress Receptions | `format:suppress-receptions` | ❌ |  |
| diagram-canvas | Format › Suppress Literals | `format:suppress-literals` | 🟡 | the view's suppressLiterals flag from a file is honoured when drawing (src/views.rs) but there is no command or shortcut to change it |
| diagram-canvas | Add | `model:add` | 🟡 | explorer Add submenu with 8 kinds plus features (src/ui.rs:243) |
| diagram-canvas | Add Diagram | `model:add-diagram` | 🟡 | Class and Use Case diagrams only (src/ui.rs:274, 848) |
| diagram-canvas | Format › Font... | `format:font` | ❌ | fonts read from files but not editable |
| diagram-canvas | Format › Fill Color... | `format:fill-color` | 🟡 | fillColor is drawn (src/render.rs:136) but cannot be changed |
| diagram-canvas | Format › Line Color... | `format:line-color` | 🟡 | lineColor is drawn (src/render.rs:135) but cannot be changed |
| diagram-canvas | Format › Line Style › Rectilinear | `format:linestyle-rectilinear` | ✅ | Ctrl+L (src/app.rs:632, src/ui.rs:948) |
| diagram-canvas | Format › Line Style › Oblique | `format:linestyle-oblique` | ✅ | Ctrl+B (src/ui.rs:951) |
| diagram-canvas | Format › Line Style › RoundRect | `format:linestyle-roundrect` | ❌ |  |
| diagram-canvas | Format › Line Style › Curve | `format:linestyle-curve` | ❌ |  |
| diagram-canvas | Format › Auto Resize | `format:auto-resize` | ❌ |  |
| diagram-canvas | Format › Show Shadow | `format:show-shadow` | ✅ | Ctrl+Shift+H (src/app.rs:648, src/ui.rs:954) |
| diagram-canvas | Format › Inherit Style | `format:inherit-style` | ❌ |  |
| diagram-canvas | Cut | `edit:cut` | ❌ | no clipboard |
| diagram-canvas | Copy | `edit:copy` | ❌ | no clipboard |
| diagram-canvas | Paste | `edit:paste` | ❌ | no clipboard |
| diagram-canvas | Select All | `edit:select-all` | ✅ | Ctrl+A (src/ui.rs:940) |
| diagram-canvas | Delete | `edit:delete` | ✅ | Delete key and Edit menu (src/canvas.rs:446, src/ui.rs:811) |
| diagram-canvas | Delete From Model | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:937) |
| diagram-canvas | Select In Explorer | `edit:select-in-explorer` | ✅ | Ctrl+E (src/ui.rs:943) |
| diagram-canvas | Select In Diagram | `edit:select-in-diagram` | 🟡 | clicking an explorer row selects its view in the current diagram (src/app.rs:624); no Ctrl+D, no search across diagrams |
| diagram-canvas | Open Sub-Diagram | `edit:open-sub-diagram` | ❌ |  |
| diagram-canvas | Tag Editor... | `model:tag-editor` | ❌ |  |
| mermaid-example-dropdown | Class Diagram | `mermaid:example-class-diagram` | ❌ |  |
| mermaid-example-dropdown | Sequence Diagram | `mermaid:example-sequence-diagram` | ❌ |  |
| mermaid-example-dropdown | State Diagram | `mermaid:example-state-diagram` | ❌ |  |
| mermaid-example-dropdown | Flowchart | `mermaid:example-flowchart` | ❌ |  |
| mermaid-example-dropdown | Entity Relationship Diagram | `mermaid:example-er-diagram` | ❌ |  |
| mermaid-example-dropdown | Requirement Diagram | `mermaid:example-requirement-diagram` | ❌ |  |
| mermaid-example-dropdown | Mindmap | `mermaid:example-mindmap` | ❌ |  |
| #relationship-view div.listview | Select Related Element | `relationship-view:select-related-element` | ❌ |  |
| #relationship-view div.listview | Select Relationship | `relationship-view:select-relationship` | ❌ |  |
| model-explorer | *(the menu as a whole)* |  | 🟡 | the explorer menu has Add, Add Diagram, Open Diagram, Delete from Model (src/ui.rs:243) |
| model-explorer | Model › Add › Tag | `common:tag` | 🟡 | Tags are listed in the explorer ("name = value", src/ui.rs:169) but cannot be added |
| model-explorer | Model › Add › Entry Activity | `uml:create-model.entry-activity` | ❌ |  |
| model-explorer | Model › Add › Do Activity | `uml:create-model.do-activity` | ❌ |  |
| model-explorer | Model › Add › Exit Activity | `uml:create-model.exit-activity` | ❌ |  |
| model-explorer | Model › Add › Effect | `uml:create-model.effect` | ❌ |  |
| model-explorer | Model › Add › Constraint | `uml:create-model.constraint` | ❌ |  |
| model-explorer | Add Diagram | `model:add-diagram` | 🟡 | Class and Use Case diagrams only (src/ui.rs:274, 848) |
| model-explorer | Add | `model:add` | 🟡 | explorer Add submenu with 8 kinds plus features (src/ui.rs:243) |
| model-explorer | Cut | `edit:cut` | ❌ | no clipboard |
| model-explorer | Copy | `edit:copy` | ❌ | no clipboard |
| model-explorer | Paste | `edit:paste` | ❌ | no clipboard |
| model-explorer | Delete From Model | `edit:delete-from-model` | ✅ | Ctrl+Delete (src/ui.rs:937) |
| model-explorer | Move Up | `edit:move-up` | ❌ | no reordering in the explorer |
| model-explorer | Move Down | `edit:move-down` | ❌ | no reordering in the explorer |
| model-explorer | Select In Diagram | `edit:select-in-diagram` | 🟡 | clicking an explorer row selects its view in the current diagram (src/app.rs:624); no Ctrl+D, no search across diagrams |
| working-diagrams | *(the menu as a whole)* |  | ❌ | no context menu on Working Diagrams rows; only an × close button (src/ui.rs:619) |
| working-diagrams | Close Diagram | `view:close-diagram` | ✅ | F4 (src/ui.rs:975) |
| working-diagrams | Close Other Diagrams | `view:close-other-diagrams` | ❌ |  |
| working-diagrams | Close All Diagrams | `view:close-all-diagrams` | ❌ |  |
| working-diagrams | Rename | `view:rename-diagram` | ❌ | working-diagrams context menu missing |
| model-explorer-settings | *(the menu as a whole)* |  | 🟡 | only a sort-by-name toggle button (src/ui.rs:309) |
| model-explorer-settings | Sort By Added | `explorer:sort-by-added` | ✅ | sort toggle in the explorer header (src/ui.rs:309) |
| model-explorer-settings | Sort By Name | `explorer:sort-by-name` | ✅ | (src/ui.rs:143, 309) |
| model-explorer-settings | Show Stereotype Text | `explorer:show-stereotype-text` | ❌ |  |
| model-explorer-settings | Expand All | `explorer:expand-all` | ❌ |  |
| model-explorer-settings | Collapse All | `explorer:collapse-all` | ❌ |  |
