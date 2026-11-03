import { RELEASES_URL, REPO_URL } from "@/lib/site";
import { Wordmark } from "./nav";

const COLUMNS: [string, [string, string][]][] = [
  [
    "UMLCraft",
    [
      ["Download", "#get-it"],
      ["Releases", RELEASES_URL],
      ["Source code", REPO_URL],
      ["Feature list", `${REPO_URL}/tree/main/docs/features`],
    ],
  ],
  [
    "Related",
    [
      ["The Crafting Apps", "https://getartcraft.com/apps"],
      ["Made with Craft Anything", "https://github.com/storytold/craft-anything"],
      ["StarUML", "https://staruml.io"],
    ],
  ],
];

export function Footer() {
  return (
    <footer>
      <div className="mx-auto grid max-w-7xl gap-px border-x border-line bg-line md:grid-cols-[1.4fr_1fr_1fr]">
        <div className="bg-bg p-6 md:p-10">
          <Wordmark className="text-2xl" />
          <p className="mt-4 max-w-sm text-sm leading-relaxed text-muted">
            An open-source, clean-room reimplementation of StarUML in Rust. StarUML is a product of MKLabs; UMLCraft is an
            independent project and is not affiliated with it.
          </p>
        </div>
        {COLUMNS.map(([title, links]) => (
          <nav key={title} className="bg-bg p-6 md:p-10" aria-label={title}>
            <p className="label text-faint">{title}</p>
            <ul className="mt-4 space-y-2.5">
              {links.map(([label, href]) => (
                <li key={label}>
                  <a href={href} className="text-sm text-ink hover:text-app-ink">
                    {label}
                  </a>
                </li>
              ))}
            </ul>
          </nav>
        ))}
      </div>
      <div className="mx-auto flex max-w-7xl flex-wrap justify-between gap-4 border-x border-t border-line px-6 py-4 md:px-10">
        <p className="label text-faint">MIT License</p>
        <p className="label text-faint">macOS · Windows · Linux · FreeBSD</p>
      </div>
    </footer>
  );
}
