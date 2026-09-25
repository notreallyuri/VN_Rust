"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useRef, useState } from "react";
import { type Hit, parts, ready, type Section, search } from "@/search";

function useShortcut(open: () => void) {
  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const typing =
        event.target instanceof HTMLElement &&
        (event.target.tagName === "INPUT" ||
          event.target.tagName === "TEXTAREA" ||
          event.target.isContentEditable);
      if (
        (event.key === "k" && (event.metaKey || event.ctrlKey)) ||
        (event.key === "/" && !typing)
      ) {
        event.preventDefault();
        open();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open]);
}

export function Search() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [sections, setSections] = useState<Section[] | null>(null);
  const [broken, setBroken] = useState(false);
  const [at, setAt] = useState(0);
  const field = useRef<HTMLInputElement>(null);
  const router = useRouter();

  const show = useCallback(() => setOpen(true), []);
  useShortcut(show);

  useEffect(() => {
    if (!open) return;
    ready()
      .then(setSections)
      .catch(() => setBroken(true));
    field.current?.focus();
  }, [open]);

  useEffect(() => setAt(0), []);

  const hits: Hit[] = sections ? search(sections, query) : [];

  function go(hit: Hit) {
    setOpen(false);
    setQuery("");
    router.push(hit.a ? `${hit.r}#${hit.a}` : hit.r);
  }

  function onKey(event: React.KeyboardEvent) {
    if (event.key === "Escape") return setOpen(false);
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setAt((was) => Math.min(was + 1, hits.length - 1));
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setAt((was) => Math.max(was - 1, 0));
    }
    if (event.key === "Enter" && hits[at]) {
      event.preventDefault();
      go(hits[at]);
    }
  }

  return (
    <>
      <button
        aria-label="Search the documentation"
        className="flex items-center gap-2 rounded-md border border-line px-2.5 py-1 text-muted text-xs transition-colors hover:border-line hover:text-foreground"
        onClick={show}
        type="button"
      >
        Search
        <kbd className="hidden font-mono text-[0.65rem] text-muted/60 sm:inline">
          ⌘K
        </kbd>
      </button>

      {open ? (
        // biome-ignore lint/a11y/noStaticElementInteractions: the backdrop dismisses
        // biome-ignore lint/a11y/useKeyWithClickEvents: Escape is handled on the field
        <div
          className="fixed inset-0 z-50 flex items-start justify-center bg-background/70 px-4 pt-[12vh] backdrop-blur-sm"
          onClick={(event) =>
            event.target === event.currentTarget && setOpen(false)
          }
        >
          <div className="w-full max-w-xl overflow-hidden rounded-xl border border-line bg-background shadow-2xl">
            <input
              aria-label="Search"
              className="w-full border-line border-b bg-transparent px-4 py-3 text-base outline-none placeholder:text-muted/60"
              onChange={(event) => {
                setQuery(event.target.value);
                setAt(0);
              }}
              onKeyDown={onKey}
              placeholder="Search the documentation…"
              ref={field}
              value={query}
            />

            <div className="max-h-[55vh] overflow-y-auto">
              {broken ? (
                <p className="px-4 py-6 text-muted text-sm">
                  the search index did not load
                </p>
              ) : !sections ? (
                <p className="px-4 py-6 text-muted text-sm">loading…</p>
              ) : query.trim() === "" ? (
                <p className="px-4 py-6 text-muted text-sm">
                  Type to search {new Set(sections.map((s) => s.r)).size} pages.
                  Arrows to move, Enter to open, Escape to close.
                </p>
              ) : hits.length === 0 ? (
                <p className="px-4 py-6 text-muted text-sm">
                  nothing matches “{query}”
                </p>
              ) : (
                <ul>
                  {hits.map((hit, index) => (
                    <li key={`${hit.r}#${hit.a}`}>
                      <button
                        className={`block w-full px-4 py-3 text-left transition-colors ${
                          index === at ? "bg-faint" : "hover:bg-faint/60"
                        }`}
                        onClick={() => go(hit)}
                        onMouseEnter={() => setAt(index)}
                        type="button"
                      >
                        <span className="flex items-baseline gap-2">
                          <span className="font-medium text-sm">
                            {hit.h || hit.p}
                          </span>
                          {hit.h ? (
                            <span className="text-muted text-xs">{hit.p}</span>
                          ) : null}
                        </span>
                        <span className="mt-1 block text-muted text-xs leading-5">
                          {parts(hit.excerpt, query).map((piece, part) =>
                            part % 2 === 1 ? (
                              <mark
                                className="bg-transparent text-foreground"
                                // biome-ignore lint/suspicious/noArrayIndexKey: a fixed split
                                key={part}
                              >
                                {piece}
                              </mark>
                            ) : (
                              piece
                            ),
                          )}
                        </span>
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}
