import { DownloadIcon } from "lucide-react";
import type { Release } from "@/lib/release";
import { APP, asset, REPO_URL, SHOTS } from "@/lib/site";
import { GitHubIcon } from "./icons";
import { onlyOn, unknownOs } from "./os-script";
import { Accent } from "./section";

export function Hero({ release }: { release: Release | null }) {
  const [before, accent, after] = APP.headline;
  const shot = SHOTS[0];
  const primary = (os: string) => release?.downloads.find((d) => d.platform === os && d.primary);
  const button =
    "items-center gap-2 rounded-full bg-app px-5 py-3 text-sm font-medium text-white transition-colors hover:bg-app-ink";
  return (
    <section id="top" className="border-b border-line">
      <div className="mx-auto max-w-7xl border-x border-line">
        <div className="flex items-center justify-between gap-4 border-b border-line px-6 py-3 md:px-10">
          <p className="label text-muted">
            <span className="text-app">01</span>
            <span className="mx-2 text-faint">/</span>
            {APP.name}
          </p>
          <p className="label text-faint">
            {APP.category} · {APP.status}
          </p>
        </div>

        <div className="px-6 pt-14 pb-12 md:px-10 md:pt-20">
          <div className="flex items-center gap-4">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img src={asset("/images/icon-512.png")} alt="" width={64} height={64} className="size-14 rounded-xl md:size-16" />
            <p className="display text-3xl md:text-4xl">
              UML<span className="accent">Craft</span>
            </p>
          </div>
          <h1 className="display mt-10 max-w-5xl text-5xl sm:text-6xl md:text-7xl lg:text-8xl">
            {before}
            <Accent>{accent}</Accent>
            {after}
          </h1>
          <p className="mt-8 max-w-2xl text-lg leading-relaxed text-muted">{APP.lede}</p>

          <div className="mt-10 flex flex-wrap items-center gap-3">
            {APP.platforms.map((os) => {
              const d = primary(os);
              return (
                <a key={os} href={d?.url ?? "#get-it"} className={`${onlyOn(os)} ${button}`}>
                  <DownloadIcon className="size-4" />
                  Download for {os}
                </a>
              );
            })}
            <a href="#get-it" className={`${unknownOs} inline-flex ${button}`}>
              <DownloadIcon className="size-4" />
              Download
            </a>
            <a
              href={REPO_URL}
              className="inline-flex items-center gap-2 rounded-full border border-line px-5 py-3 text-sm text-ink transition-colors hover:border-app hover:text-ink-strong"
            >
              <GitHubIcon />
              View on GitHub
            </a>
          </div>
          <p className="label mt-6 text-faint">
            {APP.platforms.join(" · ")} · Free and open source (MIT)
            {release && <> · v{release.version}</>}
          </p>
        </div>

        <figure className="border-t border-line">
          <div className="bg-bg-sunken p-3 md:p-6">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              src={asset(`/images/${shot.file}`)}
              alt={shot.alt}
              width={1200}
              height={900}
              className="mx-auto w-full max-w-5xl rounded-md shadow-2xl shadow-black/20"
              fetchPriority="high"
            />
          </div>
          <figcaption className="flex items-center justify-between gap-4 border-t border-line px-6 py-3 md:px-10">
            <span className="label text-muted">{shot.caption}</span>
            <span className="label text-faint">Screenshot 01</span>
          </figcaption>
        </figure>
      </div>
    </section>
  );
}
