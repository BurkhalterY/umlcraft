import { ArrowUpRightIcon } from "lucide-react";
import { total, type Area } from "@/lib/parity";
import { REPO_URL } from "@/lib/site";
import { Accent, Section } from "./section";

// StarUML parity, from docs/features/ at build time.
export function Progress({ index, areas }: { index: number; areas: Area[] }) {
  const all = total(areas);
  const pct = (n: number, a: Area) => (100 * n) / Math.max(1, a.present + a.partial + a.missing);
  return (
    <Section
      id="progress"
      index={index}
      label="Progress"
      annotation="StarUML parity"
      title={
        <>
          How far <Accent>along</Accent>.
        </>
      }
      lede={
        <>
          Every StarUML feature we found, from its menus and shortcuts to its quick edits and validation rules, is listed
          and checked against the code. This is early alpha: class and use case diagrams work today, and the list says
          what comes next.
        </>
      }
    >
      <div className="border-t border-line px-6 py-8 md:px-10">
        <div className="flex flex-wrap items-baseline gap-x-10 gap-y-3">
          <Figure value={all.present} label="present" />
          <Figure value={all.partial} label="partial" />
          <Figure value={all.missing} label="still to do" />
        </div>
        <Bar area={all} pct={pct} tall />
      </div>
      <ul className="grid gap-px border-t border-line bg-line md:grid-cols-2">
        {areas.map((a, i) => (
          <li
            key={a.file}
            className={`bg-bg px-6 py-5 md:px-10 ${i === areas.length - 1 && areas.length % 2 === 1 ? "md:col-span-2" : ""}`}
          >
            <div className="flex items-baseline justify-between gap-4">
              <a
                href={`${REPO_URL}/blob/main/docs/features/${a.file}`}
                className="text-sm text-ink hover:text-app-ink"
              >
                {a.title}
              </a>
              <span className="label text-faint">
                {a.present} / {a.present + a.partial + a.missing}
              </span>
            </div>
            <Bar area={a} pct={pct} />
          </li>
        ))}
      </ul>
      <div className="flex flex-wrap items-center justify-between gap-4 border-t border-line px-6 py-4 md:px-10">
        <p className="flex items-center gap-4 text-xs text-muted">
          <Swatch className="bg-app" /> present
          <Swatch className="bg-app/35" /> partial
          <Swatch className="bg-bg-sunken" /> missing
        </p>
        <a
          href={`${REPO_URL}/tree/main/docs/features`}
          className="inline-flex items-center gap-1.5 text-sm text-app-ink hover:underline"
        >
          The full list
          <ArrowUpRightIcon className="size-4" />
        </a>
      </div>
    </Section>
  );
}

function Figure({ value, label }: { value: number; label: string }) {
  return (
    <p>
      <span className="display text-4xl md:text-5xl">{value.toLocaleString("en-US")}</span>
      <span className="label ml-2 text-muted">{label}</span>
    </p>
  );
}

function Bar({ area, pct, tall = false }: { area: Area; pct: (n: number, a: Area) => number; tall?: boolean }) {
  return (
    <div
      className={`mt-3 flex overflow-hidden rounded-full bg-bg-sunken ${tall ? "h-3" : "h-1.5"}`}
      role="img"
      aria-label={`${area.present} present, ${area.partial} partial, ${area.missing} missing`}
    >
      <div className="bg-app" style={{ width: `${pct(area.present, area)}%` }} />
      <div className="bg-app/35" style={{ width: `${pct(area.partial, area)}%` }} />
    </div>
  );
}

function Swatch({ className }: { className: string }) {
  return <span className={`inline-block size-2.5 rounded-full ${className}`} />;
}
