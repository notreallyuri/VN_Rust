"use client";

import { useEffect, useRef, useState } from "react";
import { highlightStory, ready } from "@/highlight/browser";
import type { Grammar, Token } from "@/highlight/lex";

const BOX = "px-0 py-3 font-mono text-[0.78rem] leading-6";

export function Editor({
  value,
  onChange,
  rows,
}: {
  value: string;
  onChange: (next: string) => void;
  rows: number;
}) {
  const [grammar, setGrammar] = useState<Grammar | null>(null);
  const behind = useRef<HTMLPreElement>(null);
  const front = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    let live = true;
    ready()
      .then((loaded) => live && setGrammar(loaded))
      .catch(() => {});
    return () => {
      live = false;
    };
  }, []);

  const lines: Token[][] | null = grammar
    ? highlightStory(grammar, value)
    : null;

  function sync() {
    const from = front.current;
    const to = behind.current;
    if (!from || !to) return;
    to.scrollTop = from.scrollTop;
    to.scrollLeft = from.scrollLeft;
  }

  return (
    <div className="relative flex-1">
      {lines ? (
        <pre
          aria-hidden
          className={`${BOX} pointer-events-none absolute inset-0 overflow-hidden whitespace-pre pr-4 text-foreground`}
          ref={behind}
        >
          <code>
            {lines.map((tokens, index) => (
              <span
                className="block"
                // biome-ignore lint/suspicious/noArrayIndexKey: the index is the line number
                key={index}
              >
                {tokens.length === 0
                  ? "\n"
                  : tokens.map((token, at) => (
                      <span
                        className={token.kind ? `tok-${token.kind}` : undefined}
                        // biome-ignore lint/suspicious/noArrayIndexKey: a fixed listing
                        key={at}
                      >
                        {token.text}
                      </span>
                    ))}
                {tokens.length > 0 ? "\n" : null}
              </span>
            ))}
          </code>
        </pre>
      ) : null}

      <textarea
        aria-label="story source"
        className={`${BOX} relative block w-full resize-none overflow-auto whitespace-pre bg-transparent pr-4 outline-none ${
          lines ? "text-transparent caret-foreground" : "text-foreground"
        }`}
        onChange={(event) => onChange(event.target.value)}
        onScroll={sync}
        ref={front}
        rows={rows}
        spellCheck={false}
        value={value}
        wrap="off"
      />
    </div>
  );
}
