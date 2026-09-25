"use client";

import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

type Heading = { id: string; text: string; level: number };

const OFFSET = 96;

export function Toc() {
  const pathname = usePathname();
  const [headings, setHeadings] = useState<Heading[]>([]);
  const [active, setActive] = useState("");

  // biome-ignore lint/correctness/useExhaustiveDependencies: this stays mounted across routes, so the path is what re-reads the headings
  useEffect(() => {
    const found = Array.from(
      document.querySelectorAll<HTMLHeadingElement>("main h2[id], main h3[id]"),
    ).map((h) => ({
      id: h.id,
      text: h.textContent ?? "",
      level: h.tagName === "H3" ? 3 : 2,
    }));
    setHeadings(found);

    let frame = 0;
    const update = () => {
      frame = 0;
      let current = found[0]?.id ?? "";
      for (const heading of found) {
        const element = document.getElementById(heading.id);
        if (element && element.getBoundingClientRect().top <= OFFSET) {
          current = heading.id;
        }
      }
      setActive(current);
    };
    const onScroll = () => {
      if (frame === 0) frame = requestAnimationFrame(update);
    };

    update();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll);
    return () => {
      if (frame !== 0) cancelAnimationFrame(frame);
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
    };
  }, [pathname]);

  if (headings.length < 2) return null;

  return (
    <nav aria-label="On this page" className="text-sm">
      <p className="mb-3 font-mono text-[0.7rem] text-muted uppercase tracking-[0.18em]">
        On this page
      </p>
      <ul className="space-y-2">
        {headings.map((heading) => (
          <li
            key={heading.id}
            className={heading.level === 3 ? "pl-3" : undefined}
          >
            <a
              href={`#${heading.id}`}
              aria-current={active === heading.id ? "location" : undefined}
              className={
                active === heading.id
                  ? "text-accent transition-colors"
                  : "text-muted transition-colors hover:text-foreground"
              }
            >
              {heading.text}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  );
}
