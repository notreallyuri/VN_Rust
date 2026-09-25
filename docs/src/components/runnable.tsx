"use client";

import { type ReactNode, useEffect, useState } from "react";
import { Notes, Played } from "@/components/playground";
import { type Answer, ask } from "@/playground/client";

type View = "code" | "run" | "listing";

export function Runnable({
  code,
  children,
}: {
  code: string;
  children: ReactNode;
}) {
  const [view, setView] = useState<View>("code");
  const [picks, setPicks] = useState<number[]>([]);
  const [answer, setAnswer] = useState<Answer | null>(null);
  const [broken, setBroken] = useState<string | null>(null);

  useEffect(() => {
    if (view === "code") return;
    let current = true;
    ask(code, picks)
      .then((next) => current && setAnswer(next))
      .catch(
        (error: unknown) =>
          current &&
          setBroken(error instanceof Error ? error.message : String(error)),
      );
    return () => {
      current = false;
    };
  }, [view, code, picks]);

  return (
    <div className="my-6 overflow-hidden rounded-lg border border-line bg-faint">
      <div className="flex items-center justify-between border-line border-b px-2 py-1.5">
        <span className="flex items-center gap-1">
          {(["code", "run", "listing"] as View[]).map((which) => (
            <button
              aria-pressed={view === which}
              className={`rounded px-2 py-1 font-mono text-[0.68rem] uppercase tracking-widest transition ${
                view === which
                  ? "bg-line/60 text-foreground"
                  : "text-muted hover:text-foreground"
              }`}
              key={which}
              onClick={() => setView(which)}
              type="button"
            >
              {which}
            </button>
          ))}
        </span>
        {view !== "code" && answer ? (
          <span className="px-2 font-mono text-[0.68rem] text-muted/70">
            {answer.counts.scenes} scene{answer.counts.scenes === 1 ? "" : "s"},{" "}
            {answer.counts.instructions} instructions
          </span>
        ) : null}
      </div>

      {view === "code" ? (
        <div className="[&>figure]:my-0 [&>figure]:rounded-none [&>figure]:border-0">
          {children}
        </div>
      ) : broken ? (
        <p className="px-4 py-4 font-mono text-[0.75rem] text-amber-500/90">
          the playground did not load: {broken}
        </p>
      ) : !answer ? (
        <p className="px-4 py-4 font-mono text-[0.75rem] text-muted/60">
          compiling…
        </p>
      ) : view === "listing" ? (
        <pre className="overflow-x-auto px-4 py-4 font-mono text-[0.75rem] text-muted leading-6">
          {answer.listing.trim() || "nothing to compile"}
        </pre>
      ) : (
        <Played
          answer={answer}
          onPick={(index) => setPicks([...picks, index])}
        />
      )}

      {view !== "code" && answer ? (
        <>
          {answer.wrapped ? (
            <p className="border-line border-t px-4 py-2 font-mono text-[0.7rem] text-muted/60">
              a fragment, so it runs inside a scene called example
            </p>
          ) : null}
          <Notes notes={answer.notes} />
          {picks.length > 0 ? (
            <div className="border-line border-t px-4 py-2">
              <button
                className="font-mono text-[0.7rem] text-muted uppercase tracking-widest transition hover:text-foreground"
                onClick={() => setPicks([])}
                type="button"
              >
                restart
              </button>
            </div>
          ) : null}
        </>
      ) : null}
    </div>
  );
}
