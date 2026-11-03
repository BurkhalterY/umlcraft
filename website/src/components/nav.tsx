import { asset, REPO_URL } from "@/lib/site";
import { GitHubIcon } from "./icons";

const LINKS = [
  ["Highlights", "#highlights"],
  ["Screenshots", "#gallery"],
  ["Download", "#get-it"],
  ["Progress", "#progress"],
  ["Build", "#start"],
];

export function Nav() {
  return (
    <nav className="sticky top-0 z-50 border-b border-line bg-bg/90 backdrop-blur-sm">
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between gap-6 border-x border-line px-6 md:px-10">
        <a href="#top" className="flex items-center gap-2.5" aria-label="UMLCraft, back to the top">
          {/* eslint-disable-next-line @next/next/no-img-element */}
          <img src={asset("/images/icon.png")} alt="" width={28} height={28} className="size-7 rounded-md" />
          <Wordmark className="text-lg" />
        </a>
        <ul className="hidden items-center gap-7 lg:flex">
          {LINKS.map(([label, href]) => (
            <li key={href}>
              <a href={href} className="text-sm text-muted transition-colors hover:text-ink-strong">
                {label}
              </a>
            </li>
          ))}
        </ul>
        <a
          href={REPO_URL}
          className="flex items-center gap-2 rounded-full border border-line px-3.5 py-1.5 text-sm text-ink transition-colors hover:border-app hover:text-ink-strong"
        >
          <GitHubIcon />
          <span>GitHub</span>
        </a>
      </div>
    </nav>
  );
}

/** "UML" in the display face, "Craft" in the serif italic accent. */
export function Wordmark({ className = "" }: { className?: string }) {
  return (
    <span className={`display tracking-[-0.03em] ${className}`}>
      UML<span className="accent">Craft</span>
    </span>
  );
}
