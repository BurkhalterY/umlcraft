// StarUML parity, read at build time from docs/features/: the same counts as its README.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

export type Area = {
  file: string;
  title: string;
  present: number;
  partial: number;
  missing: number;
};

const FEATURES = join(process.cwd(), "..", "docs", "features");
const MARK = /^(?:- .*UMLCraft:\*\* |\| .*?\| )(✅|🟡|❌|➖)/u;

/** Files 1–7 describe StarUML; 8 lists UMLCraft's own features and is not counted as parity. */
export function parityAreas(): Area[] {
  const files = readdirSync(FEATURES)
    .filter((f) => /^[1-7]-.*\.md$/.test(f))
    .sort();
  return files.map((file) => {
    const text = readFileSync(join(FEATURES, file), "utf8");
    const title = text.match(/^# (.+)$/m)?.[1] ?? file;
    const area: Area = { file, title, present: 0, partial: 0, missing: 0 };
    for (const line of text.split("\n")) {
      // the status cell of table rows, or the mark after "UMLCraft:" in bullets
      const bullet = line.match(/UMLCraft:\*\* (✅|🟡|❌|➖)/u);
      const cell = line.startsWith("| ") ? line.match(/\| (✅|🟡|❌|➖)/u) : null;
      const m = line.startsWith("- ") ? bullet : cell;
      if (!m || !MARK.test(line)) continue;
      if (m[1] === "✅") area.present++;
      else if (m[1] === "🟡") area.partial++;
      else if (m[1] === "❌") area.missing++;
    }
    return area;
  });
}

export function total(areas: Area[]): Area {
  return areas.reduce(
    (t, a) => ({ ...t, present: t.present + a.present, partial: t.partial + a.partial, missing: t.missing + a.missing }),
    { file: "", title: "All", present: 0, partial: 0, missing: 0 },
  );
}
