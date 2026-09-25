# The documentation site

The guide for novn: the story language, the engine, and the tools around them. It is a
hand-built Next.js app rather than a docs framework, because the design is owned here and
because the code block has a compiler behind it — see `TODO.md` in the repository root,
milestone 11, for the plan and the decisions behind it.

Nothing has moved into it yet. The prose still lives in the READMEs beside the code; this
is the scaffold and the pipeline that will carry it.

```sh
pnpm install
pnpm dev      # http://localhost:3000
pnpm build    # a static export in out/
pnpm lint     # biome, and the absolute-path guard CI runs
pnpm format   # biome, writing
```

The build is a static export (`output: "export"`) served from GitHub Pages at
`https://notreallyuri.github.io/novn`, so the deployed site lives under a `basePath` of
`/novn`. Development runs at the root instead, because typing the prefix a hundred
times a day to catch a mistake made once is the wrong trade. What catches it instead:
`next/link` and `next/image` add the prefix themselves, and CI greps for the paths that
would skip it — a hand-written `<a href="/engine">`, an `<img src="/…">`, a `url(/…)` in
CSS. Write those through the components or relative, and the prefix takes care of itself.

`public/.nojekyll` is what stops Pages hiding the `_next` directory.

Code blocks are highlighted by tree-sitter at build time, so no highlighter ships to the
browser. `.story` uses the same grammar nvim and the LSP use; Rust and shell use the
grammars from `tree-sitter-rust` and `tree-sitter-bash`, whose wasm and queries are copied
into `src/highlight/` from `node_modules`. A shell block is drawn with a prompt in the
gutter, and that prompt is not selectable, so copying gives the commands alone.

`src/highlight/tree-sitter-story.wasm` and `story.scm` are copies of what lives in
`editors/tree-sitter-story`; CI diffs the queries so they cannot drift, and the wasm is
rebuilt with:

```sh
cd editors/tree-sitter-story && tree-sitter build --wasm
cp tree-sitter-story.wasm ../../docs/src/highlight/
```

Content will live in `src/content` as MDX, one page per subsystem, and the crate READMEs
will shrink to front doors that link into it. Until that move happens, edit the READMEs.
