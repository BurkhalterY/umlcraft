// The latest GitHub release, read once at build time, with its installers sorted by platform.
// Without network access (or with the API rate-limited) the page falls back to the releases page.
import { REPO, RELEASES_URL } from "./site";

export type Platform = "macOS" | "Windows" | "Linux" | "FreeBSD";

export type Download = {
  platform: Platform;
  label: string;
  note: string;
  file: string;
  url: string;
  size: number;
  /** The one the platform card offers first. */
  primary: boolean;
};

export type Release = {
  version: string;
  url: string;
  date: string;
  downloads: Download[];
};

type Kind = { platform: Platform; label: string; note: string; primary?: boolean; test: RegExp };

// Asset names come from packaging/*/package.* (see the README's Downloads tables).
const KINDS: Kind[] = [
  { platform: "macOS", label: "Disk image (Universal)", note: "Apple silicon and Intel Macs.", primary: true, test: /-macos-universal\.dmg$/ },
  { platform: "macOS", label: "Zip (Universal)", note: "The app bundle, zipped.", test: /-macos-universal\.zip$/ },
  { platform: "Windows", label: "Installer (x64)", note: "Most Windows 10 and 11 PCs.", primary: true, test: /-windows-x64\.msi$/ },
  { platform: "Windows", label: "Portable (x64)", note: "Unzip and run, no install.", test: /-windows-x64-portable\.zip$/ },
  { platform: "Windows", label: "Installer (ARM64)", note: "Windows 11 on ARM laptops.", test: /-windows-arm64\.msi$/ },
  { platform: "Windows", label: "Portable (ARM64)", note: "Unzip and run on Windows on ARM.", test: /-windows-arm64-portable\.zip$/ },
  { platform: "Linux", label: "AppImage (x86_64)", note: "Any distro. Mark it executable and run.", primary: true, test: /-x86_64\.AppImage$/ },
  { platform: "Linux", label: "Debian package (x86_64)", note: "Debian, Ubuntu and derivatives.", test: /_amd64\.deb$/ },
  { platform: "Linux", label: "RPM package (x86_64)", note: "Fedora, RHEL and openSUSE.", test: /\.x86_64\.rpm$/ },
  { platform: "Linux", label: "Tarball (x86_64)", note: "Plain binaries to unpack anywhere.", test: /-linux-x86_64\.tar\.gz$/ },
  { platform: "Linux", label: "AppImage (ARM64)", note: "Any distro on 64-bit ARM.", test: /-aarch64\.AppImage$/ },
  { platform: "Linux", label: "Debian package (ARM64)", note: "Debian, Ubuntu and derivatives on ARM.", test: /_arm64\.deb$/ },
  { platform: "Linux", label: "RPM package (ARM64)", note: "Fedora, RHEL and openSUSE on ARM.", test: /\.aarch64\.rpm$/ },
  { platform: "Linux", label: "Tarball (ARM64)", note: "Plain binaries for 64-bit ARM.", test: /-linux-aarch64\.tar\.gz$/ },
  { platform: "FreeBSD", label: "Package (x86_64)", note: "Install with pkg add.", primary: true, test: /^umlcraft-[0-9.]+\.pkg$/ },
  { platform: "FreeBSD", label: "Tarball (x86_64)", note: "Plain binaries for FreeBSD.", test: /-freebsd-x86_64\.tar\.gz$/ },
];

type ApiAsset = { name: string; browser_download_url: string; size: number };
type ApiRelease = { tag_name: string; html_url: string; published_at: string; assets: ApiAsset[] };

export async function latestRelease(): Promise<Release | null> {
  const headers: Record<string, string> = { Accept: "application/vnd.github+json" };
  if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers, cache: "force-cache" });
    if (!res.ok) throw new Error(`GitHub API ${res.status}`);
    const r = (await res.json()) as ApiRelease;
    const downloads: Download[] = [];
    for (const kind of KINDS) {
      const a = r.assets.find((x) => kind.test.test(x.name));
      if (a) {
        downloads.push({
          platform: kind.platform,
          label: kind.label,
          note: kind.note,
          file: a.name,
          url: a.browser_download_url,
          size: a.size,
          primary: kind.primary ?? false,
        });
      }
    }
    return { version: r.tag_name.replace(/^v/, ""), url: r.html_url, date: r.published_at, downloads };
  } catch (e) {
    console.warn(`latest release unavailable, linking to ${RELEASES_URL} instead: ${(e as Error).message}`);
    return null;
  }
}

export function formatSize(bytes: number): string {
  return `${(bytes / 1_000_000).toFixed(1)} MB`;
}
