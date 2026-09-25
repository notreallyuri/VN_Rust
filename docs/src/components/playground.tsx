"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { type Answer, ask } from "@/playground/client";

type View = "play" | "listing";

const STAGE: Record<string, string> = {
  show: "show",
  remove: "remove",
  clear: "clear",
  background: "background",
  music: "music",
  sound: "sound",
  voice: "voice",
  call: "call",
  commit: "commit",
};

function Gutter({ notes, lines }: { notes: Answer["notes"]; lines: number }) {
  const bad = new Set(notes.filter((n) => n.line > 0).map((n) => n.line));
  return (
    <div
      aria-hidden
      className="select-none py-3 pr-2 pl-3 text-right font-mono text-[0.78rem] text-muted/40 leading-6"
    >
      {Array.from({ length: lines }, (_, at) => (
        <div
          className={bad.has(at + 1) ? "text-red-500/80" : undefined}
          // biome-ignore lint/suspicious/noArrayIndexKey: the index is the line number
          key={at}
        >
          {at + 1}
        </div>
      ))}
    </div>
  );
}

export function Notes({ notes }: { notes: Answer["notes"] }) {
  if (notes.length === 0) return null;
  return (
    <ul className="space-y-1.5 border-line border-t px-4 py-3 font-mono text-[0.75rem] leading-5">
      {notes.map((note) => (
        <li key={`${note.line}:${note.message}`}>
          <span
            className={
              note.severity === "error"
                ? "text-red-500/90"
                : "text-amber-500/90"
            }
          >
            {note.line > 0 ? `line ${note.line}: ` : ""}
            {note.severity}:
          </span>{" "}
          <span className="text-muted">{note.message}</span>
        </li>
      ))}
    </ul>
  );
}

export function Played({
  answer,
  onPick,
}: {
  answer: Answer;
  onPick: (index: number) => void;
}) {
  return (
    <div className="space-y-2.5 px-4 py-4 text-[0.82rem] leading-6">
      {answer.steps.map((step, at) => {
        const key = `${at}-${step.kind}`;
        if (step.kind === "dialogue") {
          return (
            <p key={key}>
              <span className="text-accent">{step.speaker}</span>{" "}
              <span className="text-foreground">{step.text}</span>
            </p>
          );
        }
        if (step.kind === "narration") {
          return (
            <p className="text-muted italic" key={key}>
              {step.text}
            </p>
          );
        }
        if (step.kind === "chose") {
          return (
            <p className="font-mono text-[0.75rem] text-accent/80" key={key}>
              → {step.taken}
            </p>
          );
        }
        return (
          <p className="font-mono text-[0.72rem] text-muted/60" key={key}>
            {STAGE[step.kind] ?? step.kind}
            {step.detail ? ` ${step.detail}` : ""}
          </p>
        );
      })}

      {answer.choices.length > 0 ? (
        <div className="space-y-2 pt-2">
          {answer.choices.map((choice) => (
            <button
              className="block w-full rounded-md border border-line px-3 py-2 text-left text-[0.8rem] transition enabled:hover:border-accent/60 enabled:hover:text-foreground disabled:cursor-not-allowed disabled:opacity-45"
              disabled={!choice.enabled}
              key={choice.index}
              onClick={() => onPick(choice.index)}
              title={choice.reason ?? undefined}
              type="button"
            >
              {choice.text}
              {choice.reason ? (
                <span className="ml-2 text-muted text-xs">
                  ({choice.reason})
                </span>
              ) : null}
            </button>
          ))}
        </div>
      ) : null}

      {answer.ended ? (
        <p className="pt-2 font-mono text-[0.72rem] text-muted/60">
          the story ended
        </p>
      ) : null}

      {answer.stopped ? (
        <p className="pt-2 font-mono text-[0.72rem] text-amber-500/90">
          {answer.stopped}
        </p>
      ) : null}
    </div>
  );
}

export function Playground({ code, name }: { code: string; name?: string }) {
  const [source, setSource] = useState(code.replace(/\n$/, ""));
  const [picks, setPicks] = useState<number[]>([]);
  const [answer, setAnswer] = useState<Answer | null>(null);
  const [broken, setBroken] = useState<string | null>(null);
  const [view, setView] = useState<View>("play");
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      ask(source, picks)
        .then((next) => {
          setAnswer(next);
          setBroken(null);
        })
        .catch((error: unknown) =>
          setBroken(error instanceof Error ? error.message : String(error)),
        );
    }, 140);
    return () => clearTimeout(timer.current);
  }, [source, picks]);

  const lines = useMemo(() => source.split("\n").length, [source]);

  function edit(next: string) {
    setSource(next);
    setPicks([]);
  }

  return (
    <figure className="my-6 overflow-hidden rounded-lg border border-line bg-faint">
      <figcaption className="flex items-center justify-between border-line border-b px-4 py-2 font-mono text-[0.7rem] text-muted uppercase tracking-widest">
        <span>{name ?? "playground"}</span>
        <span className="flex items-center gap-1">
          {answer && !answer.ok ? (
            <span className="mr-2 text-red-500/90 normal-case tracking-normal">
              {answer.notes.filter((n) => n.severity === "error").length} error
              {answer.notes.filter((n) => n.severity === "error").length === 1
                ? ""
                : "s"}
            </span>
          ) : answer ? (
            <span className="mr-2 text-muted/60 normal-case tracking-normal">
              {answer.counts.scenes} scene
              {answer.counts.scenes === 1 ? "" : "s"},{" "}
              {answer.counts.instructions} instructions
            </span>
          ) : null}
          {(["play", "listing"] as View[]).map((which) => (
            <button
              aria-pressed={view === which}
              className={`rounded px-2 py-1 uppercase tracking-widest transition ${
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
      </figcaption>

      <div className="grid lg:grid-cols-2 lg:divide-x lg:divide-line">
        <div className="flex min-w-0">
          <Gutter notes={answer?.notes ?? []} lines={Math.max(lines, 10)} />
          <textarea
            aria-label="story source"
            className="flex-1 resize-none overflow-x-auto bg-transparent py-3 pr-4 font-mono text-[0.78rem] text-foreground leading-6 outline-none focus-visible:bg-background/40"
            onChange={(event) => edit(event.target.value)}
            rows={Math.max(lines, 10)}
            spellCheck={false}
            value={source}
            wrap="off"
          />
        </div>

        <div className="border-line border-t lg:border-t-0">
          {broken ? (
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
        </div>
      </div>

      <Notes notes={answer?.notes ?? []} />

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
    </figure>
  );
}
