"use client";

import { useEffect, useRef, useState } from "react";

export function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const [able, setAble] = useState(false);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    setAble(typeof navigator !== "undefined" && !!navigator.clipboard);
    return () => clearTimeout(timer.current);
  }, []);

  if (!able) return null;

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      return;
    }
    setCopied(true);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => setCopied(false), 1600);
  }

  return (
    <button
      aria-label={copied ? "Copied" : "Copy to clipboard"}
      className="absolute top-2 right-2 rounded-md border border-line/70 bg-background/80 px-2 py-1 font-mono text-[0.68rem] text-muted uppercase tracking-widest opacity-0 backdrop-blur transition group-hover:opacity-100 hover:border-line hover:text-foreground focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-accent focus-visible:outline-offset-2"
      onClick={copy}
      type="button"
    >
      {copied ? "Copied" : "Copy"}
    </button>
  );
}
