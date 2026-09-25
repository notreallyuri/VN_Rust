"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { nav } from "@/nav";

const pages = nav.flatMap((section) =>
  section.items.map((item) => ({ ...item, section: section.title })),
);

export function PageNav() {
  const here = usePathname()?.replace(/\/$/, "") || "/docs";
  const at = pages.findIndex((page) => page.href === here);
  if (at === -1) return null;

  const previous = pages[at - 1];
  const next = pages[at + 1];
  if (!previous && !next) return null;

  return (
    <nav className="mt-16 grid gap-3 border-line border-t pt-8 sm:grid-cols-2">
      {previous ? (
        <Link
          href={previous.href}
          className="group rounded-lg border border-line px-4 py-3 transition-colors hover:border-muted/40"
        >
          <span className="font-mono text-[0.7rem] text-muted uppercase tracking-[0.18em]">
            Previous
          </span>
          <span className="mt-1 block text-foreground text-sm">
            {previous.title}
          </span>
        </Link>
      ) : (
        <span />
      )}
      {next ? (
        <Link
          href={next.href}
          className="group rounded-lg border border-line px-4 py-3 text-right transition-colors hover:border-muted/40 sm:col-start-2"
        >
          <span className="font-mono text-[0.7rem] text-muted uppercase tracking-[0.18em]">
            Next
          </span>
          <span className="mt-1 block text-foreground text-sm">
            {next.title}
          </span>
        </Link>
      ) : null}
    </nav>
  );
}
