import { asset, REPO_URL, SHOTS } from "@/lib/site";
import { Section } from "./section";

export function Gallery({ index }: { index: number }) {
  return (
    <Section id="gallery" index={index} label="Screenshots" annotation="Captured in UMLCraft">
      <div className="grid gap-px bg-line md:grid-cols-2">
        {SHOTS.map((shot, i) => (
          <figure key={shot.file} className="bg-bg">
            <a href={`${REPO_URL}/blob/main/docs/images/${shot.file}`} className="block bg-bg-sunken p-4 md:p-6">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={asset(`/images/${shot.file}`)}
                alt={shot.alt}
                width={1200}
                height={900}
                loading="lazy"
                className="w-full rounded-md shadow-xl shadow-black/15 transition-transform duration-300 hover:scale-[1.01]"
              />
            </a>
            <figcaption className="flex items-center justify-between gap-4 border-t border-line px-6 py-3">
              <span className="label text-muted">{shot.caption}</span>
              <span className="label text-faint">{String(i + 1).padStart(2, "0")}</span>
            </figcaption>
          </figure>
        ))}
      </div>
      <p className="border-t border-line px-6 py-4 text-sm text-muted md:px-10">
        Every screenshot is the real app, rendered with its <code className="font-mono text-ink">--screenshot</code>{" "}
        option from <code className="font-mono text-ink">examples/library.mdj</code>.
      </p>
    </Section>
  );
}
