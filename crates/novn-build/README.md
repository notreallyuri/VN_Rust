# novn-build

Build-script helper for games made with [`novn`](../novn/README.md). It embeds a game's assets
in the executable, so a release build is a single file you can hand to someone instead of one
that only runs on the machine that built it.

It has no dependencies, so using it from `build.rs` does not compile raylib a second time.

**The guide is at <https://notreallyuri.github.io/novn/docs/crates/build>**: when it embeds and
when it does not, leaving files out, and what the generated table looks like.

## Setting it up

```toml
# Cargo.toml
[build-dependencies]
novn-build = "0.1"
```

```rust
// build.rs
fn main() {
    novn_build::embed_assets("assets");
}
```

```rust
// src/main.rs
VnApp::new("My Game")
    .assets(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
    .embedded_assets(novn::embedded_assets!())
```

Release builds embed everything, debug builds embed nothing and read the folder, which is what
keeps hot reload and the schema export working while you develop. `VN_EMBED_ASSETS=1` or `=0`
forces it either way.

## Tests

```sh
cargo test -p novn-build
```

`tests/embed.rs` covers the file list, sorted, with `/` separators, hidden files and exclusions
left out, and that the generated code compiles to the expected table.
