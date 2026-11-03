import { Downloads } from "@/components/downloads";
import { Footer } from "@/components/footer";
import { Gallery } from "@/components/gallery";
import { Hero } from "@/components/hero";
import { Highlights } from "@/components/highlights";
import { Nav } from "@/components/nav";
import { Progress } from "@/components/progress";
import { Start } from "@/components/start";
import { parityAreas } from "@/lib/parity";
import { latestRelease } from "@/lib/release";

// One page, rendered at build time: the latest release comes from the GitHub API and the parity
// figures from docs/features/, as they are when the site is built.
export default async function Home() {
  const release = await latestRelease();
  const areas = parityAreas();
  return (
    <>
      <Nav />
      <main>
        <Hero release={release} />
        <Highlights index={2} />
        <Gallery index={3} />
        <Downloads index={4} release={release} />
        <Progress index={5} areas={areas} />
        <Start index={6} />
      </main>
      <Footer />
    </>
  );
}
