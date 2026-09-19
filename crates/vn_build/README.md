# vn_build

Build-script helpers for games made with `vn_engine`. It has no dependencies, so using it
from `build.rs` doesn't compile raylib a second time.

## Embedding the assets

A release build that reads its assets from the game's source folder only runs on the
machine that built it. `vn_build` puts every file under `assets/` into the executable
instead, so a release build is a single file you can hand to someone.

`Cargo.toml`:

```toml
[build-dependencies]
vn_build = { path = "../VN_Rust/crates/vn_build" }
```

`build.rs`:

```rust
fn main() {
    vn_build::embed_assets("assets");
}
```

`src/main.rs`:

```rust
VnApp::new("My Game")
    .assets(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
    .embedded_assets(vn_engine::embedded_assets!())
```

`embedded_assets!()` includes the list the build script wrote (`$OUT_DIR/vn_assets.rs`):
a `&'static [(&str, &[u8])]` of paths relative to the assets folder, with `/` separators,
each with its bytes (`include_bytes!`). The engine then loads textures, fonts, music,
sounds and stories from memory (see vn_engine's README, "Assets").

| Build | What is embedded |
| --- | --- |
| `cargo build --release` (`PROFILE=release`) | Every file |
| Debug builds | Nothing: an empty list, so they compile fast, and the game reads the folder (hot reload, schema export) |
| `VN_EMBED_ASSETS=1` / `VN_EMBED_ASSETS=0` | Force it on or off for any profile |
| `Embed::new(dir).always(true)` | Always |

Hidden files (starting with `.`) are skipped. `Embed::new("assets").exclude(["schema.json",
"sources"]).run()` leaves out files or whole folders, given relative to the assets folder.
The build script reruns when anything under the folder changes.

## Tests

```sh
cargo test -p vn_build
```

`tests/embed.rs` checks the file list (sorted, `/` separators, hidden files and exclusions
left out) and that the generated code compiles to the expected table.
