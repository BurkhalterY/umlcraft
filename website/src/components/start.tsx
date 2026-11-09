import { BUILD_COMMANDS, REPO_URL } from "@/lib/site";
import { CopyButton } from "./copy-button";
import { GitHubIcon } from "./icons";
import { Accent, Section } from "./section";

export function Start({ index }: { index: number }) {
  return (
    <Section
      id="start"
      index={index}
      label="Build it"
      annotation="From source"
      title={
        <>
          Build it <Accent>yourself</Accent>.
        </>
      }
      lede={
        <>
          All it takes is a recent Rust toolchain. On Linux, install the GTK 3, xkbcommon and OpenGL development packages
          first (on Debian and Ubuntu: <code className="font-mono text-ink">libgtk-3-dev libxkbcommon-dev libgl1-mesa-dev</code>).
        </>
      }
    >
      {/* minmax(0, …) columns, so the long commands scroll inside the terminal instead of widening the page */}
      <div className="grid grid-cols-1 gap-px border-t border-line bg-line md:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
        <div className="min-w-0 bg-bg-sunken p-6 md:p-10">
          <div className="flex items-center justify-between">
            <p className="label text-faint">Terminal</p>
            <CopyButton text={BUILD_COMMANDS.join("\n")} />
          </div>
          <pre className="mt-4 overflow-x-auto font-mono text-sm leading-7 text-ink">
            {BUILD_COMMANDS.map((c) => (
              <span key={c} className="block">
                <span className="text-faint select-none">$ </span>
                {c}
              </span>
            ))}
          </pre>
        </div>
        <div className="flex flex-col justify-between gap-6 bg-bg p-6 md:p-10">
          <p className="leading-relaxed text-muted">
            UMLCraft is MIT-licensed. The code, the StarUML feature list and the rules behind the file format are all in the
            repository; pick an open item in <code className="font-mono text-ink">docs/features/</code> to help.
          </p>
          <a
            href={REPO_URL}
            className="inline-flex w-fit items-center gap-2 rounded-full bg-ink-strong px-5 py-3 text-sm font-medium text-bg transition-opacity hover:opacity-85"
          >
            <GitHubIcon />
            BurkhalterY/umlcraft
          </a>
        </div>
      </div>
    </Section>
  );
}
