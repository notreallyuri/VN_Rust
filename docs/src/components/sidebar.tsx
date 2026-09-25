"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { nav } from "@/nav";

export function Sidebar() {
  const here = usePathname()?.replace(/\/$/, "") || "/docs";
  return (
    <nav className="space-y-8 text-sm">
      {nav.map((section) => (
        <div key={section.title}>
          <p className="mb-3 font-mono text-[0.7rem] text-muted uppercase tracking-[0.18em]">
            {section.title}
          </p>
          <ul className="space-y-1 border-line border-l">
            {section.items.map((item) => {
              const current = here === item.href;
              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    aria-current={current ? "page" : undefined}
                    className={
                      current
                        ? "-ml-px block border-accent border-l py-1.5 pl-4 font-medium text-foreground"
                        : "-ml-px block border-transparent border-l py-1.5 pl-4 text-muted transition-colors hover:border-line hover:text-foreground"
                    }
                  >
                    {item.title}
                  </Link>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
    </nav>
  );
}
