"use client";

import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { Sidebar } from "@/components/sidebar";
import { Toc } from "@/components/toc";

export function MobileNav() {
  const [open, setOpen] = useState(false);
  const here = usePathname();

  // biome-ignore lint/correctness/useExhaustiveDependencies: the path is the point; a new route closes the drawer
  useEffect(() => setOpen(false), [here]);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) =>
      event.key === "Escape" && setOpen(false);
    const held = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    window.addEventListener("keydown", onKey);
    return () => {
      document.body.style.overflow = held;
      window.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <>
      <button
        aria-expanded={open}
        aria-label="Open the documentation menu"
        className="-ml-1.5 rounded-md p-1.5 text-muted transition-colors hover:text-foreground lg:hidden"
        onClick={() => setOpen(true)}
        type="button"
      >
        <svg
          aria-hidden="true"
          fill="none"
          height="18"
          stroke="currentColor"
          strokeLinecap="round"
          strokeWidth="1.5"
          viewBox="0 0 18 18"
          width="18"
        >
          <path d="M2 4.5h14M2 9h14M2 13.5h9" />
        </svg>
      </button>

      {open ? (
        <div className="fixed inset-0 z-40 lg:hidden">
          <button
            aria-label="Close the menu"
            className="absolute inset-0 h-full w-full bg-background/70 backdrop-blur-sm"
            onClick={() => setOpen(false)}
            tabIndex={-1}
            type="button"
          />
          <div className="absolute inset-y-0 left-0 flex w-[min(20rem,85vw)] flex-col border-line border-r bg-background shadow-2xl">
            <div className="flex items-center justify-between border-line border-b px-5 py-4">
              <span className="font-mono text-[0.7rem] text-muted uppercase tracking-[0.18em]">
                Documentation
              </span>
              <button
                aria-label="Close the menu"
                className="rounded-md p-1 text-muted transition-colors hover:text-foreground"
                onClick={() => setOpen(false)}
                type="button"
              >
                <svg
                  aria-hidden="true"
                  fill="none"
                  height="16"
                  stroke="currentColor"
                  strokeLinecap="round"
                  strokeWidth="1.5"
                  viewBox="0 0 16 16"
                  width="16"
                >
                  <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
                </svg>
              </button>
            </div>

            <div className="flex-1 overflow-y-auto px-5 py-6">
              <Sidebar />
              <div className="mt-8 border-line border-t pt-6 xl:hidden">
                <Toc />
              </div>
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}
