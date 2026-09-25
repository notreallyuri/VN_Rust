"use client";

import Link from "next/link";
import { type ReactNode, useEffect, useState } from "react";
import { Search } from "@/components/search";

export function SiteHeader({
  width = "max-w-7xl",
  leading,
}: {
  width?: string;
  leading?: ReactNode;
}) {
  const [lifted, setLifted] = useState(false);

  useEffect(() => {
    const onScroll = () => setLifted(window.scrollY > 4);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={`sticky top-0 z-20 border-b transition-colors duration-200 ${
        lifted
          ? "border-line bg-background/85 backdrop-blur"
          : "border-transparent bg-transparent"
      }`}
    >
      <div
        className={`mx-auto flex w-full items-center justify-between px-6 py-4 ${width}`}
      >
        <span className="flex items-center gap-2">
          {leading}
          <Link className="font-mono text-sm tracking-tight" href="/">
            novn
          </Link>
        </span>
        <nav className="flex items-center gap-5 text-muted text-sm">
          <Search />
          <Link
            className="transition-colors hover:text-foreground"
            href="/docs"
          >
            Docs
          </Link>
          <a
            className="transition-colors hover:text-foreground"
            href="https://github.com/notreallyuri/novn"
          >
            GitHub
          </a>
        </nav>
      </div>
    </header>
  );
}
