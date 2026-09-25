# The documentation site

The guide for VN_Rust: the story language, the engine, and the tools around them. It is a
hand-built Next.js app rather than a docs framework, because the design is owned here and
because the code block has a compiler behind it — see `TODO.md` in the repository root,
milestone 11, for the plan and the decisions behind it.

Nothing has moved into it yet. The prose still lives in the READMEs beside the code; this
is the scaffold and the pipeline that will carry it.

```sh
pnpm install
pnpm dev      # http://localhost:3000/VN_Rust — the prefix is not optional
pnpm build    # a static export in out/
pnpm lint     # biome
pnpm format   # biome, writing
```

`pnpm dev` prints that URL after Next's own banner, because Next's banner says
`http://localhost:3000` and `/` is not a route here: the site lives under its `basePath`
in development exactly as it does once deployed.

`basePath` is `/VN_Rust` in development as well as in the build, so a hand-written link
that forgets it breaks here rather than only once deployed. The build is a static export
(`output: "export"`), served from GitHub Pages at
`https://notreallyuri.github.io/VN_Rust`; `public/.nojekyll` is what stops Pages hiding
the `_next` directory.

Content will live in `src/content` as MDX, one page per subsystem, and the crate READMEs
will shrink to front doors that link into it. Until that move happens, edit the READMEs.
