"use client";

import { Language, Parser, Query } from "web-tree-sitter";
import { type Grammar, highlightWith, type Token } from "@/highlight/lex";

export type { Token };

const BASE = process.env.NEXT_PUBLIC_BASE_PATH ?? "";

let loading: Promise<Grammar> | null = null;

async function load(): Promise<Grammar> {
  await Parser.init({
    locateFile: () => `${BASE}/highlight/web-tree-sitter.wasm`,
  });
  const language = await Language.load(
    `${BASE}/highlight/tree-sitter-story.wasm`,
  );
  const parser = new Parser();
  parser.setLanguage(language);
  const source = await fetch(`${BASE}/highlight/story.scm`).then((r) => {
    if (!r.ok) throw new Error(`story.scm: ${r.status}`);
    return r.text();
  });
  return { parser, query: new Query(language, source) };
}

export function ready(): Promise<Grammar> {
  loading ??= load();
  return loading;
}

export function highlightStory(grammar: Grammar, code: string): Token[][] {
  return highlightWith(grammar, code, "story");
}
