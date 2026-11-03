import { HIGHLIGHTS } from "@/lib/site";
import { Accent, Section } from "./section";

export function Highlights({ index }: { index: number }) {
  return (
    <Section
      id="highlights"
      index={index}
      label="Highlights"
      annotation="UML modeling"
      title={
        <>
          What&apos;s <Accent>inside</Accent>.
        </>
      }
      lede="A clean-room reimplementation of StarUML: the same layout, the same files, the same habits, in a small native program."
    >
      <ul className="grid gap-px border-t border-line bg-line sm:grid-cols-2 lg:grid-cols-3">
        {HIGHLIGHTS.map((h, i) => (
          <li key={h.title} className="bg-bg p-6 md:p-10">
            <p className="label text-app">{String(i + 1).padStart(2, "0")}</p>
            <h3 className="display mt-4 text-2xl md:text-3xl">{h.title}</h3>
            <p className="mt-3 leading-relaxed text-muted">{h.body}</p>
          </li>
        ))}
      </ul>
    </Section>
  );
}
