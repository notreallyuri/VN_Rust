import fs from "node:fs";
import path from "node:path";
import { Language, Parser, Query } from "web-tree-sitter";
import { type Grammar, highlightWith, type Token } from "@/highlight/lex";

export type { Token };

const here = path.join(process.cwd(), "src/highlight");

await Parser.init();

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

export const aliases: Record<string, string> = {
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

export function highlight(code: string, language?: string): Token[][] {
  const name = language ? aliases[language] : undefined;
  return highlightWith(name ? grammars[name] : undefined, code, name);
}
