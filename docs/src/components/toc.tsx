"use client";

import { useEffect, useState } from "react";

type Heading = { id: string; text: string; level: number };

export function Toc() {
  const [headings, setHeadings] = useState<Heading[]>([]);
  const [active, setActive] = useState<string>("");

  useEffect(() => {
    const found = Array.from(
      document.querySelectorAll<HTMLHeadingElement>("main h2, main h3"),
    )
      .filter((h) => h.id)
      .map((h) => ({
        id: h.id,
        text: h.textContent ?? "",
        level: h.tagName === "H3" ? 3 : 2,
      }));
    setHeadings(found);

    const observer = new IntersectionObserver(
      (entries) => {
        const visible = entries.filter((e) => e.isIntersecting);
        if (visible.length > 0) setActive(visible[0].target.id);
      },
      { rootMargin: "-80px 0px -70% 0px" },
    );
    for (const h of found) {
      const el = document.getElementById(h.id);
      if (el) observer.observe(el);
    }
    return () => observer.disconnect();
  }, []);

  if (headings.length < 2) return null;

  return (
    <div className="text-sm">
      <p className="mb-3 font-mono text-[0.7rem] text-muted uppercase tracking-[0.18em]">
        On this page
      </p>
      <ul className="space-y-2">
        {headings.map((h) => (
          <li key={h.id} className={h.level === 3 ? "pl-3" : undefined}>
            <a
              href={`#${h.id}`}
              className={
                active === h.id
                  ? "text-foreground transition-colors"
                  : "text-muted transition-colors hover:text-foreground"
              }
            >
              {h.text}
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
}
