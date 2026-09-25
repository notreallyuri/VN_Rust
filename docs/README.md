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
browser. Four grammars: `.story` uses the same one nvim and the LSP use, and Rust, shell
and TOML come from `tree-sitter-rust`, `tree-sitter-bash` and
`@tree-sitter-grammars/tree-sitter-toml`, whose wasm and queries are copied into
`src/highlight/` from `node_modules`. One capture-to-kind map covers all four, so a colour
is decided once.

`toml.scm` is the one query that is adapted rather than copied. Upstream captures a whole
`pair` as `@property`, which overlaps the key, the `=` and the value, and it gives table
headers and keys the same capture, so `[features]` and the keys under it would come out one
colour. Ours captures the table name as `@label` and the key as `@variable`, and takes the
whole key node so a dotted key like `workspace.package` stays one span; `dotted_key` nests
left-recursively, so matching its parts by depth would miss all but the last. `.` is left
out of the punctuation rule for the same reason.

Overlapping captures resolve by splitting, not discarding: a capture inside another cuts
the outer one in two and keeps its colour on both sides. That is what makes a `\"` inside a
string, a `{variable}` inside a `.story` line and a `$VAR` inside a quoted shell path keep
the string colour around them. Two captures on the exact same range keep the last one,
which is the precedence `rust.scm` is written against.

A shell block is drawn with a prompt in the gutter, and that prompt is `select-none` and
`aria-hidden`, so both selecting and the copy button give the commands alone. The copy
button is the only client component in the site; it renders nothing until it has seen a
`navigator.clipboard`, and it stays on `Copy` if the write is refused rather than claiming
a copy that did not happen.

`src/highlight/tree-sitter-story.wasm` and `story.scm` are copies of what lives in
`editors/tree-sitter-story`; CI diffs the queries so they cannot drift, and the wasm is
rebuilt with:

```sh
cd editors/tree-sitter-story && tree-sitter build --wasm
cp tree-sitter-story.wasm ../../docs/src/highlight/
```

The other three ship their own wasm, so upgrading one is a recopy from `node_modules`:

```sh
cp node_modules/tree-sitter-rust/tree-sitter-rust.wasm src/highlight/
cp node_modules/tree-sitter-rust/queries/highlights.scm src/highlight/rust.scm
```

Content lives in `src/app/docs/` as `page.mdx`, one page per subject, with `src/nav.ts` as
the manifest that drives the sidebar and the previous/next links. The crate READMEs are
front doors that link into it, so prose belongs here and not there.

The playground on `/docs/playground` is [`novn-playground`](../crates/novn-playground/README.md)
built to WebAssembly and run in the reader's browser: the real compiler and VM, not a
recording. `pnpm build` and `pnpm dev` compile it first, through `scripts/build_playground_wasm.sh`,
so working on the site needs a Rust toolchain with `wasm32-unknown-unknown`. The `.wasm` lands
in `public/` and is gitignored, since it is a build product.

A fenced block becomes one by asking for `story-play` instead of `story`. That marker has to
live in the language tag rather than the fence's meta, because remark drops the meta before it
reaches a component and Turbopack cannot take a rehype plugin as a function to put it back.
`crates/novn-script/tests/spec.rs` reads both spellings, so a playable example is still
compiled and golden-tested like every other one.

Fetching the wasm needs the `basePath` that `next/link` adds for free elsewhere, so the config
exposes it as `NEXT_PUBLIC_BASE_PATH` and `src/playground/client.ts` builds the URL from it.

`scripts/check_links.py` at the repository root checks relative links and anchors across
the markdown, the `/docs/...` links inside MDX against the routes the export actually
produces, and braces in MDX prose that would be read as JSX. CI runs it after the build,
since two of the three need `out/`.
