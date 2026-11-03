// Copies the screenshots and the app icon from the repo into public/, so they live in one place
// (docs/images and packaging/icons) and the site always shows the current ones.
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const site = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(site, "..");

const files = [
  ["docs/images/class-diagram.png", "public/images/class-diagram.png"],
  ["docs/images/usecase-diagram.png", "public/images/usecase-diagram.png"],
  ["packaging/icons/umlcraft-256.png", "public/images/icon.png"],
  ["packaging/icons/umlcraft-512.png", "public/images/icon-512.png"],
  ["packaging/icons/umlcraft-32.png", "src/app/icon.png"],
  ["packaging/icons/umlcraft-256.png", "src/app/apple-icon.png"],
];

for (const [from, to] of files) {
  mkdirSync(dirname(join(site, to)), { recursive: true });
  copyFileSync(join(repo, from), join(site, to));
}
console.log(`copied ${files.length} files from the repo`);
