import fs from "node:fs";
import path from "node:path";
import { Language, Parser, Query } from "web-tree-sitter";

export type Token = { text: string; kind: string | null };

const here = path.join(process.cwd(), "src/highlight");

await Parser.init();
const language = await Language.load(path.join(here, "tree-sitter-story.wasm"));
const parser = new Parser();
parser.setLanguage(language);
const query = new Query(
  language,
  fs.readFileSync(path.join(here, "highlights.scm"), "utf8"),
);

const kinds: Record<string, string> = {
  comment: "comment",
  keyword: "keyword",
  "keyword.function": "keyword",
  "keyword.conditional": "keyword",
  "keyword.operator": "operator",
  label: "label",
  type: "character",
  string: "string",
  "string.escape": "escape",
  constant: "constant",
  "constant.builtin": "constant",
  attribute: "constant",
  number: "number",
  boolean: "number",
  variable: "variable",
  "punctuation.delimiter": "punctuation",
  "punctuation.special": "operator",
};

function kindOf(capture: string): string | null {
  return kinds[capture] ?? null;
}

function lex(code: string): Token[][] {
  const tree = parser.parse(code);
  if (!tree) return [[{ text: code, kind: null }]];

  const spans: { start: number; end: number; kind: string }[] = [];
  for (const capture of query.captures(tree.rootNode)) {
    const kind = kindOf(capture.name);
    if (!kind) continue;
    const { startIndex: start, endIndex: end } = capture.node;
    const covering = spans.findIndex((s) => s.start <= start && s.end >= end);
    if (covering !== -1) spans.splice(covering, 1);
    if (!spans.some((s) => s.start < end && s.end > start)) {
      spans.push({ start, end, kind });
    }
  }
  spans.sort((a, b) => a.start - b.start);

  const flat: Token[] = [];
  let at = 0;
  for (const span of spans) {
    if (span.start > at)
      flat.push({ text: code.slice(at, span.start), kind: null });
    flat.push({ text: code.slice(span.start, span.end), kind: span.kind });
    at = span.end;
  }
  if (at < code.length) flat.push({ text: code.slice(at), kind: null });

  const lines: Token[][] = [[]];
  for (const token of flat) {
    const parts = token.text.split("\n");
    parts.forEach((part, index) => {
      if (index > 0) lines.push([]);
      if (part) lines[lines.length - 1].push({ text: part, kind: token.kind });
    });
  }
  return lines;
}

function parses(code: string): boolean {
  const tree = parser.parse(code);
  return tree !== null && !tree.rootNode.hasError;
}

const INDENT = "  ";

export function highlightStory(code: string): Token[][] {
  if (parses(code)) return lex(code);

  const wrapped = `scene __fragment:\n${code
    .split("\n")
    .map((line) => (line ? INDENT + line : line))
    .join("\n")}`;
  if (!parses(wrapped)) return lex(code);

  return lex(wrapped)
    .slice(1)
    .map((line) => {
      const [first, ...rest] = line;
      if (!first || !first.text.startsWith(INDENT)) return line;
      const text = first.text.slice(INDENT.length);
      return text ? [{ ...first, text }, ...rest] : rest;
    });
}
