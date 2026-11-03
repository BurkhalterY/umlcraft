import { ArrowUpRightIcon, DownloadIcon } from "lucide-react";
import { formatSize, type Platform, type Release } from "@/lib/release";
import { APP, RELEASES_URL } from "@/lib/site";
import { onlyOn } from "./os-script";
import { Accent, Section } from "./section";

export function Downloads({ index, release }: { index: number; release: Release | null }) {
  const date = release ? new Date(release.date).toISOString().slice(0, 10) : null;
  return (
    <Section
      id="get-it"
      index={index}
      label={`Get ${APP.name}`}
      annotation="Free · Open source"
      title={
        <>
          Download <Accent>{APP.name}</Accent>.
        </>
      }
      lede={
        release ? (
          <>
            Version {release.version}, published {date}. A new version comes out every night when the code changed,
            so this is never more than a day behind.
          </>
        ) : (
          <>A new version comes out every night when the code changed. Every installer is on the releases page.</>
        )
      }
    >
      {release ? (
        <div className="grid gap-px border-t border-line bg-line md:grid-cols-2 xl:grid-cols-4">
          {APP.platforms.map((os, i) => (
            <PlatformCard key={os} os={os} number={i + 1} release={release} />
          ))}
        </div>
      ) : null}
      <div className="flex flex-wrap items-center justify-between gap-4 border-t border-line px-6 py-4 md:px-10">
        <p className="text-sm text-muted">
          Checksums for every file are in <code className="font-mono text-ink">SHA256SUMS.txt</code> on the release
          page.
        </p>
        <a
          href={release?.url ?? RELEASES_URL}
          className="inline-flex items-center gap-1.5 text-sm text-app-ink hover:underline"
        >
          {release ? `Release notes for ${release.version}` : "All releases"}
          <ArrowUpRightIcon className="size-4" />
        </a>
      </div>
    </Section>
  );
}

function PlatformCard({ os, number, release }: { os: Platform; number: number; release: Release }) {
  const files = release.downloads.filter((d) => d.platform === os);
  const primary = files.find((d) => d.primary) ?? files[0];
  const others = files.filter((d) => d !== primary);
  return (
    <div className="flex flex-col bg-bg p-6 md:p-8">
      <div className="flex h-6 items-center justify-between">
        <p className="label text-faint">Platform {String(number).padStart(2, "0")}</p>
        <p className={`${onlyOn(os)} label items-center rounded-full bg-app-soft px-2.5 py-0.5 text-app-ink`}>
          Your system
        </p>
      </div>
      <h3 className="display mt-4 text-3xl">{os}</h3>
      {primary ? (
        <>
          <p className="mt-2 text-sm text-ink">{primary.label}</p>
          <p className="text-sm text-muted">{primary.note}</p>
          <a
            href={primary.url}
            className="mt-6 inline-flex items-center justify-center gap-2 rounded-full bg-app px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-app-ink"
          >
            <DownloadIcon className="size-4" />
            Download for {os}
          </a>
          <p className="label mt-2 text-center text-faint">{formatSize(primary.size)}</p>
          {others.length > 0 && (
            <ul className="mt-6 divide-y divide-line border-t border-line">
              {others.map((d) => (
                <li key={d.file}>
                  <a href={d.url} className="group flex items-start justify-between gap-3 py-3">
                    <span>
                      <span className="block text-sm text-ink group-hover:text-app-ink">{d.label}</span>
                      <span className="block text-xs text-muted">{d.note}</span>
                      <span className="mt-1 block font-mono text-[0.6875rem] break-all text-faint">{d.file}</span>
                    </span>
                    <DownloadIcon className="mt-0.5 size-4 shrink-0 text-faint group-hover:text-app" />
                  </a>
                </li>
              ))}
            </ul>
          )}
        </>
      ) : (
        <a href={release.url} className="mt-6 text-sm text-app-ink hover:underline">
          No {os} build in this release; see the release page.
        </a>
      )}
    </div>
  );
}
