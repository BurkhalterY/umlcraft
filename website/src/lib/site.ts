// The words and pictures of the page, in one place. Wording follows the README.

export const REPO = "BurkhalterY/umlcraft";
export const REPO_URL = `https://github.com/${REPO}`;
export const RELEASES_URL = `${REPO_URL}/releases`;
export const SITE_URL = "https://burkhaltery.github.io/umlcraft";

export const APP = {
  name: "UMLCraft",
  prefix: "UML",
  category: "UML modeling",
  status: "Early alpha",
  /** [before, accent, after]: the accent word is set in the serif italic, in the app colour. */
  headline: ["The UML modeler you already ", "know", " how to use."] as const,
  pitch: "The open-source UML modeler you already know how to use.",
  lede: "Class and use case diagrams, a model explorer, a property editor and real .mdj files, in a native app written entirely in Rust. Open source, offline, and yours.",
  platforms: ["macOS", "Windows", "Linux", "FreeBSD"] as const,
};

export const HIGHLIGHTS = [
  {
    title: "Familiar by design",
    body: "Working Diagrams and the Toolbox on the left, the Model Explorer and Editors on the right, the same dark theme and the same shortcuts. If you know StarUML, you already know UMLCraft.",
  },
  {
    title: "Native and fast",
    body: "One small binary on egui and OpenGL that starts in a blink. No Electron, no web view, no waiting.",
  },
  {
    title: "Real .mdj files",
    body: "Open and save StarUML projects. Files come back with the same structure and key order, and anything UMLCraft does not edit is written back untouched.",
  },
  {
    title: "Type right on the diagram",
    body: "Double-click an attribute and write -id: int[1] = 0, or an operation and write +placeOrder(item: Order): boolean. Visibility, types, multiplicities and parameters are read from what you type.",
  },
  {
    title: "Every change undoable",
    body: "Moves, resizes, bend points, renames and deletes are each one step to undo and redo, with Ctrl+Z, Ctrl+Y or Ctrl+Shift+Z.",
  },
  {
    title: "Installs everywhere",
    body: "MSI, DMG, AppImage, deb, rpm and FreeBSD pkg, for x64 and ARM, built and published every night from the main branch.",
  },
];

export type Shot = { file: string; caption: string; alt: string };

/** The first one is the hero picture. Files come from docs/images (scripts/copy-assets.mjs). */
export const SHOTS: Shot[] = [
  {
    file: "class-diagram.png",
    caption: "A library domain model, with the Loan class selected",
    alt: "UMLCraft with a library class diagram open: an abstract LibraryItem with Book and DVD subclasses, a Loan with a note, a Member that owns its loans, a Lendable interface and a LoanStatus enumeration; the Loan class is selected and its properties are shown on the right",
  },
  {
    file: "usecase-diagram.png",
    caption: "Actors, a system boundary and an «include»",
    alt: "A use case diagram: Member and Librarian actors around a Library System subject with Borrow an item, Check the card and Return an item, and an include arrow",
  },
];

export const BUILD_COMMANDS = [
  "git clone https://github.com/BurkhalterY/umlcraft",
  "cd umlcraft",
  "cargo run --release -- examples/library.mdj",
];

/** Paths under public/, with the base path the site is served from. */
export function asset(path: string): string {
  return `${process.env.NEXT_PUBLIC_BASE_PATH ?? ""}${path}`;
}
