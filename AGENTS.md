# Agent guide

UMLCraft is a clean-room reimplementation of StarUML in Rust (egui). It reads and writes StarUML `.mdj` files. `README.md` describes the project and lists what each file in `src/` does. `MECHANICS.md` records the StarUML rules UMLCraft follows (file format, sizes, colours), each with its source.

## Build and test

```sh
cargo run --release -- examples/library.mdj
cargo test --release
```

On Linux, install the GTK 3, xkbcommon and OpenGL development packages first. On Debian and Ubuntu these are `libgtk-3-dev libxkbcommon-dev libgl1-mesa-dev`.

## Website

The project website is in `website/`. It uses the same stack as the Crafting Apps pages on getartcraft.com: Next.js (App Router) with React, Tailwind CSS v4, TypeScript and `next/font`. It is exported as static files:

```sh
cd website
npm ci
npm run dev      # http://localhost:3000
npm run build    # static site in website/out/; BASE_PATH=/umlcraft npm run build serves it from a sub-path
```

- **Where the data comes from:**
  - the copy is in `website/src/lib/site.ts`, and follows the README;
  - the screenshots and icons are copied at build time from `docs/images/` and `packaging/icons/`, so change them there, not in `website/public/`;
  - the latest release is read from the GitHub API at build time;
  - the "Progress" figures come from files 1–7 of `docs/features/`.
- **Hosting:** `.github/workflows/website.yml` builds the site and publishes it on GitHub Pages (https://burkhaltery.github.io/umlcraft). It runs when `website/`, `docs/features/`, `docs/images/` or `packaging/icons/` change on `main`, after each successful release, and on demand.
- **Licensing:** the getartcraft.com source (`storytold/artcraft-services`) is under the ArtCraft licence, which only allows private personal use and forbids using the ArtCraft name and logo. Never copy its code, styles or images into this MIT repository. Follow its stack and layout, but write our own code.

## Commits

Always use [Conventional Commits](https://www.conventionalcommits.org). The version of every release is computed from the commit messages, so a wrong type gives a wrong version.

```
<type>(<optional scope>)<optional !>: <description>

<optional body>

<optional footers>
```

- **Types:**
  - `feat`: a new user-visible feature or behaviour
  - `fix`: a bug fix
  - `docs`: documentation only, including `docs/features/` status updates made on their own
  - `test`, `refactor`, `perf`, `build` (packaging and dependencies), `ci` (workflows), `chore`: everything else
- **Scopes** name the area: `canvas`, `explorer`, `editors`, `toolbox`, `menus`, `export`, `mdj`, `cli`, `packaging`, `release` and so on. A scope is optional, so leave it out rather than invent a vague one.
- **Description:** imperative, lower case, no final period. For example, `feat(canvas): drag a label to move it`.
- **Breaking change:** add `!` after the type or scope (`feat(mdj)!: …`) and explain it in a `BREAKING CHANGE: …` footer. UMLCraft is an application, so a change is breaking when one of these is true:
  - files saved by the new version no longer open in an older one, or in StarUML;
  - a command-line option is removed or its behaviour changes;
  - a feature, shortcut or menu command is removed, or changed so that existing habits break.

  A change that only adds something is never breaking.
- **One commit, one type.** If a change mixes a feature and an unrelated fix, make two commits.

## Releases

The release workflow (`.github/workflows/release.yml`) runs every night at 03:00 UTC. If `main` has commits since the last `vX.Y.Z` tag, it builds every installer, runs the tests and publishes the next version. `packaging/next-version.sh` computes that version from the commit messages:

| Commits since the last release | While 0.x | From 1.0 on |
|---|---|---|
| any breaking change | `0.1.4` → `0.2.0` | major |
| `feat` | `0.1.4` → `0.1.5` | minor |
| anything else | `0.1.4` → `0.1.5` | patch |

- **Never** create `v*` tags or GitHub releases by hand, and don't edit the version in `Cargo.toml` for a normal release. The workflow writes the computed version into `Cargo.toml` for the build only.
- A higher version written in `Cargo.toml` wins, so going to 1.0 means setting `version = "1.0.0"` there. That decision belongs to the project owner.
- A published release is never overwritten. A run that would reuse an existing version fails instead.
- To release right away, or to test the builds without publishing (the "dry run" box), use "Run workflow" on the Actions tab. Only do that when asked.
- To preview the next version and its notes: `packaging/next-version.sh --notes /tmp/notes.md`.

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
| `8-beyond-staruml.md` | UMLCraft's own features, which StarUML does not have (the project website, for example) |

Files 1–7 describe StarUML. Each of their items cites its StarUML source, and together they make up the parity totals. File 8 is the other kind of item: something UMLCraft adds. Its items cite their origin (who asked, and when) instead of a StarUML source, and they are counted separately. Put a new idea there, not in files 1–7. Don't use file 8 to change how a StarUML feature works: a deliberate difference from StarUML goes in the note of that feature's item in files 1–7.

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
