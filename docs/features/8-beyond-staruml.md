# Beyond StarUML

Part of the [StarUML feature inventory](README.md); the legend and how to update a status are there.

UMLCraft's own features and plans, which the original StarUML does not have. Files 1–7 describe StarUML. This file lists what UMLCraft adds on top, so each item cites its motivation instead of a StarUML source.

An item has the same format as in the other files, with **Origin** in place of the StarUML source:

```
  - **Name** — what it is and why. (Origin: who asked, when) — **UMLCraft:** ❌ not started
```

Add new ideas at the end of their section. When work starts, change the mark to 🟡 and say what is done and what is missing. When it is finished, change it to ✅ and cite the code.

## Project

- **Project website** — a public page for UMLCraft, in the style of the Crafting Apps pages (getartcraft.com/apps/photocraft). It has a hero with the latest download, the highlights, the screenshots, every installer of the latest release by platform, how far StarUML parity has come (from these files), and how to build it from source. It uses the same stack as those pages: Next.js (App Router), React, Tailwind CSS v4, TypeScript and `next/font`. It is exported as a static site, so any static host can serve it. (Origin: the project owner, 2026-11-03) — **UMLCraft:** 🟡 partial: the site is in `website/` and builds to static files (website/src/app/page.tsx). `.github/workflows/website.yml` publishes it on GitHub Pages (https://burkhaltery.github.io/umlcraft) when it changes on main and after each release. Missing: a first deploy (Pages must be enabled in the repository settings, source "GitHub Actions") and a custom domain

## Diagrams

- **Cloud icon folder** — StarUML draws its AWS, Azure and GCP diagrams with the vendors' icon sets, bundled under its resources/assets folder. Those icons are trademarked third-party artwork, so they are not copied into this MIT repository. Instead, Preferences › General › Cloud Icons › Cloud Icon Folder (`cloud.iconFolder`) names a folder laid out as StarUML's resources/assets (aws/service-icons, aws/group-icons, aws/resource-icons, aws/general-icons, azure, gcp/products, gcp/services, gcp/users-and-devices, gcp/expanded-icons). Icons are drawn from it and the Icon Picker lists its files; without it, an icon is a rounded square with its initials and the picker asks for the icon's file name. (Origin: owner request 2026-11-08, licensing) — **UMLCraft:** ✅ present the preference with a Browse… button (src/prefs.rs `Kind::Folder`), icons read once and kept (src/cloud.rs `icon_url`, `draw_icon`), and the picker over it (src/cloud.rs `icon_picker_ui`)
