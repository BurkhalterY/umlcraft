import type { NextConfig } from "next";

// A static site: `npm run build` writes plain files to out/, which any static host can serve.
// BASE_PATH serves it from a sub-path, e.g. BASE_PATH=/umlcraft for a GitHub Pages project site.
const basePath = process.env.BASE_PATH ?? "";

const config: NextConfig = {
  output: "export",
  basePath,
  trailingSlash: true,
  images: { unoptimized: true },
  env: { NEXT_PUBLIC_BASE_PATH: basePath },
};

export default config;
