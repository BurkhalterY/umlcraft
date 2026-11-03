import type { ReactNode } from "react";

// A page section: a hairline frame, a "02 / Highlights" eyebrow row, then an optional title and lede.
export function Section({
  id,
  index,
  label,
  annotation,
  title,
  lede,
  children,
}: {
  id: string;
  index: number;
  label: string;
  annotation?: string;
  title?: ReactNode;
  lede?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section id={id} className="border-b border-line">
      <div className="mx-auto max-w-7xl border-x border-line">
        <div className="flex items-center justify-between gap-4 border-b border-line px-6 py-3 md:px-10">
          <p className="label text-muted">
            <span className="text-app">{String(index).padStart(2, "0")}</span>
            <span className="mx-2 text-faint">/</span>
            {label}
          </p>
          {annotation && <p className="label text-faint">{annotation}</p>}
        </div>
        {(title || lede) && (
          <div className="grid gap-6 px-6 pt-12 pb-10 md:grid-cols-[1.2fr_1fr] md:items-end md:px-10 md:pt-16">
            {title && <h2 className="display text-4xl sm:text-5xl md:text-6xl">{title}</h2>}
            {lede && <p className="max-w-xl text-base leading-relaxed text-muted md:justify-self-end">{lede}</p>}
          </div>
        )}
        {children}
      </div>
    </section>
  );
}

export function Accent({ children }: { children: ReactNode }) {
  return <span className="accent">{children}</span>;
}
