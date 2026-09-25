import Link from "next/link";

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-20 border-line border-b bg-background/85 backdrop-blur">
      <div className="mx-auto flex w-full max-w-6xl items-center justify-between px-6 py-4">
        <Link className="font-mono text-sm tracking-tight" href="/">
          novn
        </Link>
        <nav className="flex items-center gap-6 text-muted text-sm">
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
