# novn-playground

Compiles and plays a `.story` source for the documentation site's playground. Built to
WebAssembly and run in the reader's browser, so the site's examples are checked by the same
lexer, parser, compiler and VM the engine runs.

Not published: it exists for <https://notreallyuri.github.io/novn/docs/playground>.

## Why it is a separate crate

[`novn-script`](../novn-script/README.md) has no idea this exists. Keeping the wasm entry
points here means the compiler never grows a `wasm-bindgen` dependency, or any opinion about
where it is running.

There is no `wasm-bindgen` here either. The whole interface is one string in and one string
out, which is small enough to hand-roll, and doing so means no `wasm-pack`, no
`wasm-bindgen-cli` version to keep in step with a crate version, and no generated glue.

## Building it

```sh
sh scripts/build_playground_wasm.sh
```

That builds with the workspace's `wasm` profile and copies the result to
`docs/public/novn-playground.wasm`, which is gitignored. `pnpm build` and `pnpm dev` in
`docs/` run it first, so the site always has a current one.

## The interface

Three exports, and `memory`.

| Export | Does |
| --- | --- |
| `novn_alloc(len) -> ptr` | An exact allocation of `len` bytes for the caller to write a request into |
| `novn_run(ptr, len) -> ptr` | Reads a JSON request, returns a buffer holding a little-endian `u32` length followed by that many bytes of JSON |
| `novn_free(ptr, len)` | Frees either of the above. The result buffer's `len` is `4 + the length in its header` |

The request is `{"source": "...", "picks": [0, 1]}`.

**`picks` is why there is no session state.** Rather than holding a VM across calls, the
whole story is replayed from the start with those choices applied, so every call is a pure
function of its request and the JS side never has a handle to leak. The VM is fast enough
that replaying on every click costs nothing.

The response carries `ok`, `notes` (the diagnostics, with line numbers), `listing` (what
`novn dump` prints), `counts`, `steps` (what the story did), `choices` (what is waiting, with
`enabled` and the `reason` a gated option is shut), `ended`, and `stopped` if the replay could
not continue.

A story with errors is not played, so `steps` is empty and `notes` says why.

## What it does not have

No schema, because there is no Rust to export one. Characters, images and variables are
accepted because nothing knows better, which makes the playground *looser* than `novn check`
on a real project. Everything it does report is a genuine error.

## Tests

```sh
cargo test -p novn-playground
```

`tests/playground.rs` covers it natively, without a browser: compiling, stopping at a choice,
replaying through a pick, a gated option's reason, an option hidden rather than disabled,
errors with their line and suggestion, unknown jumps, the listing, a loop that stops, a pick
that is not on offer, and rubbish in the JSON path.
