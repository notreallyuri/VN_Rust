import fs from "node:fs";
import path from "node:path";
import { Language, Parser, Query } from "web-tree-sitter";

export type Token = { text: string; kind: string | null };

const here = path.join(process.cwd(), "src/highlight");

await Parser.init();

const kinds: Record<string, string> = {
  comment: "comment",
  "comment.documentation": "comment",
  keyword: "keyword",
  "keyword.function": "keyword",
  "keyword.conditional": "keyword",
  "keyword.operator": "operator",
  operator: "operator",
  label: "label",
  type: "character",
  "type.builtin": "character",
  constructor: "character",
  string: "string",
  "string.escape": "escape",
  escape: "escape",
  constant: "constant",
  "constant.builtin": "constant",
  attribute: "constant",
  number: "number",
  boolean: "number",
  variable: "variable",
  "variable.builtin": "variable",
  "variable.parameter": "variable",
  property: "variable",
  function: "label",
  "function.macro": "label",
  "function.method": "label",
  "punctuation.delimiter": "punctuation",
  "punctuation.bracket": "punctuation",
  "punctuation.special": "operator",
};

type Span = { start: number; end: number; kind: string };

function split(around: Span, inner: Span): Span[] {
  const parts: Span[] = [];
  if (around.start < inner.start) {
    parts.push({ start: around.start, end: inner.start, kind: around.kind });
  }
  parts.push(inner);
  if (inner.end < around.end) {
    parts.push({ start: inner.end, end: around.end, kind: around.kind });
  }
  return parts;
}

type Grammar = {
  parser: Parser;
  query: Query;
  wrap?: (code: string) => string;
};

async function grammar(name: string, queries: string): Promise<Grammar> {
  const language = await Language.load(
    path.join(here, `tree-sitter-${name}.wasm`),
  );
  const parser = new Parser();
  parser.setLanguage(language);
  const query = new Query(
    language,
    fs.readFileSync(path.join(here, queries), "utf8"),
  );
  return { parser, query };
}

const grammars: Record<string, Grammar> = {
  story: await grammar("story", "story.scm"),
  rust: await grammar("rust", "rust.scm"),
  bash: await grammar("bash", "bash.scm"),
  toml: await grammar("toml", "toml.scm"),
};

const aliases: Record<string, string> = {
  story: "story",
  rust: "rust",
  rs: "rust",
  sh: "bash",
  bash: "bash",
  shell: "bash",
  console: "bash",
  toml: "toml",
  "story-play": "story",
};

const INDENT = "  ";

function lex({ parser, query }: Grammar, code: string): Token[][] {
  const tree = parser.parse(code);
  if (!tree) return [[{ text: code, kind: null }]];

  const spans: Span[] = [];
  for (const capture of query.captures(tree.rootNode)) {
    const kind = kinds[capture.name];
    if (!kind) continue;
    const { startIndex: start, endIndex: end } = capture.node;
    if (end <= start) continue;

    const outer = spans.findIndex((s) => s.start <= start && s.end >= end);
    if (outer !== -1) {
      const around = spans[outer];
      if (around.start === start && around.end === end) {
        around.kind = kind;
        continue;
      }
      spans.splice(outer, 1, ...split(around, { start, end, kind }));
      continue;
    }
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

function parses(grammar: Grammar, code: string): boolean {
  const tree = grammar.parser.parse(code);
  return tree !== null && !tree.rootNode.hasError;
}

function plain(code: string): Token[][] {
  return code.split("\n").map((line) => [{ text: line, kind: null }]);
}

export function highlight(code: string, language?: string): Token[][] {
  const name = language ? aliases[language] : undefined;
  const grammar = name ? grammars[name] : undefined;
  if (!grammar) return plain(code);

  if (parses(grammar, code)) return lex(grammar, code);

  if (name !== "story") return lex(grammar, code);

  const wrapped = `scene __fragment:\n${code
    .split("\n")
    .map((line) => (line ? INDENT + line : line))
    .join("\n")}`;
  if (!parses(grammar, wrapped)) return lex(grammar, code);

  return lex(grammar, wrapped)
    .slice(1)
    .map((line) => {
      const [first, ...rest] = line;
      if (!first || !first.text.startsWith(INDENT)) return line;
      const text = first.text.slice(INDENT.length);
      return text ? [{ ...first, text }, ...rest] : rest;
    });
}
