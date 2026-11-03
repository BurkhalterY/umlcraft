# Beyond StarUML

Part of the [StarUML feature inventory](README.md); the legend and how to update a status are there.

UMLCraft's own features and plans, which the original StarUML does not have. Files 1–7 describe StarUML. This file lists what UMLCraft adds on top, so each item cites its motivation instead of a StarUML source.

An item has the same format as in the other files, with **Origin** in place of the StarUML source:

```
  - **Name** — what it is and why. (Origin: who asked, when) — **UMLCraft:** ❌ not started
```

Add new ideas at the end of their section. When work starts, change the mark to 🟡 and say what is done and what is missing. When it is finished, change it to ✅ and cite the code.

## Project

- **Project website** — a public page for UMLCraft, in the style of the Crafting Apps pages (getartcraft.com/apps/photocraft). It has a hero with the latest download, the highlights, the screenshots, every installer of the latest release by platform, how far StarUML parity has come (from these files), and how to build it from source. It uses the same stack as those pages: Next.js (App Router), React, Tailwind CSS v4, TypeScript and `next/font`. It is exported as a static site, so any static host can serve it. (Origin: the project owner, 2026-11-03) — **UMLCraft:** 🟡 partial: the site is in `website/` and builds to static files (`npm run build` → `website/out/`). It has no hosting or deploy workflow yet, and it shows the latest release only as of the last build (website/src/app/page.tsx)
