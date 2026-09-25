# novn

**novn** is a **full visual novel engine written in Rust**, designed around a
clear separation between **story definition** and **engine logic**

It provides:

- Story scripting via a custom, indentation-based DSL
- Scene and flow management
- Asset handling (textures, audio, etc.)
- Engine-side systems (state, inventory, stores, routes)
- Deterministic execution with strong validation

The scripting language is intentionally limited — all game logic lives in Rust.

## Project Goals

- Rust-first engine architecture
- Writer-friendly story scripting
- Scene and flow management
- Centralized state ownership
- Tooling-first design (parser, formatter, LSP)
- Scalable to large, multi-file projects
- No embedded general-purpose scripting language

## Engine Overview

novn is composed of **three clearly separated layers**, each with a distinct
role.

### Runtime Layer

Responsible for **platform-facing execution** and orchestration.

- Windowing and event handling
- Input processing
- Frame and update loop
- Backend integration (graphics, audio, OS)
- Driving execution of the Core Layer

This layer is intentionally thin and mostly delegates behavior to the Core Layer.

---

### Core Layer

Responsible for **all engine logic and state**.

- Scene management and execution
- Rendering orchestration
- Texture and audio management
- Game state and systems (inventory, stores, routes)
- Save / load handling
- Story validation and error reporting

The Core Layer owns **all authoritative game state**.

---

### Story Layer (DSL)

Responsible for **narrative description only**.

- Narrative flow
- Dialogue and narration
- Branching choices
- Conditional gates (engine-defined variables)
- Triggering engine-side commands

The **Story Layer cannot own logic or systems**.

It may only:

- Describe flow
- Read engine-defined state
- Request engine actions

## Workspace Layout

| Crate | Role |
|---|---|
| [`crates/vn_script`](crates/vn_script/README.md) | Story DSL: lexer, parser, compiler and VM. No rendering dependencies |
| [`crates/vn_engine`](crates/vn_engine/README.md) | raylib-based engine: `VnApp` builder, configurable default screens, UI helpers, resources, fonts. Re-exports `raylib` and `vn_script` |
| [`crates/vn_macros`](crates/vn_macros/README.md) | `#[command]`, the attribute that turns a Rust function into a story command. Re-exported by `vn_engine`, never depended on directly |
| [`crates/vn_build`](crates/vn_build/README.md) | Build-script helper (no dependencies) that embeds a game's assets in release builds |
| [`crates/vn_live2d`](crates/vn_live2d/README.md) | Experimental optional Cubism Native adapter and standalone viewer; not yet integrated into story rendering |
| [`crates/vn_cli`](crates/vn_cli/README.md) | `novn` command-line tool (`novn new <dir>`, `novn check <path>`, `novn dump <file.story | dir>`, `novn lsp`) |
| [`examples/god_is_watching`](examples/god_is_watching/README.md) | Reference game built on `vn_engine` |

Each crate documents its API and behavior in its own README.

Games depend only on `vn_engine`.

## Running the Example

`god_is_watching` is the reference game. Run it from anywhere in the workspace:

```sh
cargo run -p god_is_watching
```

raylib is built from source on the first run, so you need CMake and a C compiler.

`cargo build --release -p god_is_watching` embeds the assets in the executable, so
`target/release/god_is_watching` (or the `.exe`) can be sent to someone and played on its
own. See [vn_build](crates/vn_build/README.md).

Its assets live in `examples/god_is_watching/assets/`:

| Path | Contents |
|---|---|
| `story/` | Chapter scripts, all loaded as one story. Each chapter ends with a `jump` into the next, and the game starts from the first scene of `00_archive.story` |
| `characters/<character_id>/<image_id>.png` | Character art, as referenced by `show <character_id> <image_id>` |
| `backgrounds/<image_id>.png` | Backgrounds, as referenced by `background <image_id>` |
| `fonts/` | `.ttf`/`.otf` fonts, assigned to text roles with `VnApp::font` in `main.rs` |
| `schema.json` | The game's registries, exported by the game in debug builds, for `novn check` |

Missing art is not an error: the engine logs a warning and draws a labeled
placeholder card, so a story can be played before its art exists.

The engine ships Noto Sans as its default font, and a game can give each kind of text
(menu, dialogue, speaker names, ...) its own font from `assets/fonts/`. See
[vn_engine's README](crates/vn_engine/README.md#fonts).

To check the stories without starting the game, and to inspect how they compile:

```sh
cargo run -p vn_cli -- check examples/god_is_watching/assets
cargo run -p vn_cli -- dump examples/god_is_watching/assets/story
```

## Starting a Game

```sh
cargo run -p vn_cli -- new ../my-novel
cd ../my-novel
cargo run
```

`novn new` writes a small working game (a `main.rs` that registers a character, variables
and a command, and a two-scene story) that depends on this workspace's `vn_engine`. See
[vn_cli's README](crates/vn_cli/README.md#vn-new-directory---title-title---engine-path-dir----engine-git-url).

## Writing Story Scripts

Story content is written using a custom DSL designed for clarity and structure.

**[Story Script Specification](SCRIPT.md)**

## Design Intent

novn intentionally avoids:

- Embedded scripting languages
- Runtime-evaluated logic
- Script-defined systems
- Implicit behavior

This keeps stories:

- Predictable
- Easy to validate
- Easy to refactor
- Friendly to tooling and large projects

## Publishing

The crates go to crates.io as a family under `novn`. They depend on each other by path
*and* version, so the leaves have to be on the registry before anything that needs them
can even be packaged:

```sh
cargo publish -p novn-macros
cargo publish -p novn-script
cargo publish -p novn-build
cargo publish -p novn            # needs macros and script
cargo publish -p novn-cli        # needs script
cargo publish -p novn-live2d     # needs novn
```

Before any of that, `cargo package -p <crate> --allow-dirty` says what would be uploaded
and refuses anything the registry would. The examples carry `publish = false` and never
leave the repository.

Every crate inherits its version, edition, MSRV, repository, homepage and authors from
`[workspace.package]`, so a release bumps one number in the root manifest and the six
`version = "0.1.0"` lines under `[workspace.dependencies]` beside it.

`rust-version` is 1.98, which is what this is built and tested with rather than a floor
anyone has measured. Edition 2024 needs 1.85 at the least, so there is room to lower it
once an older toolchain has actually been tried.

## Project Status

Work in progress

Working today: the story DSL with validation and "did you mean" diagnostics, the VM, the
raylib engine (default screens, transitions, keyboard/gamepad navigation, saves with
autosave, thumbnails and migrations, rollback, settings, music, sound and voice, hot
reload, release builds with embedded assets), `novn new` / `novn check` / `novn dump` / `novn lsp`
(a language server for any editor), and the example game.

Planned (see TODO.md):

- Tree-sitter grammar, syntax highlighting and extensions for Neovim, VS Code and Zed, and
  a formatter
