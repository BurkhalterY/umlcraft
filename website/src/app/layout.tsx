import type { Metadata, Viewport } from "next";
import { Archivo, Geist_Mono, Instrument_Serif, Inter } from "next/font/google";
import type { ReactNode } from "react";
import { OsScript } from "@/components/os-script";
import { APP, SITE_URL } from "@/lib/site";
import "./globals.css";

const archivo = Archivo({ subsets: ["latin"], variable: "--font-archivo", display: "swap" });
const instrument = Instrument_Serif({
  subsets: ["latin"],
  weight: "400",
  style: ["normal", "italic"],
  variable: "--font-instrument",
  display: "swap",
});
const inter = Inter({ subsets: ["latin"], variable: "--font-inter", display: "swap" });
const geistMono = Geist_Mono({ subsets: ["latin"], variable: "--font-geist-mono", display: "swap" });

const title = `${APP.name}: ${APP.headline.join("")}`;

export const metadata: Metadata = {
  metadataBase: new URL(process.env.SITE_URL ?? SITE_URL),
  title,
  description: APP.lede,
  openGraph: {
    title,
    description: APP.pitch,
    type: "website",
    images: [{ url: "images/class-diagram.png", width: 1200, height: 900, alt: "UMLCraft editing a class diagram" }],
  },
  twitter: { card: "summary_large_image", title, description: APP.pitch },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f3f2ef" },
    { media: "(prefers-color-scheme: dark)", color: "#111215" },
  ],
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html
      lang="en"
      className={`${archivo.variable} ${instrument.variable} ${inter.variable} ${geistMono.variable}`}
      suppressHydrationWarning
    >
      <head>
        <OsScript />
      </head>
      <body>{children}</body>
    </html>
  );
}
