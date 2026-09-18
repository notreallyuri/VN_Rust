# VN_Rust

**VN_Rust** is a **full visual novel engine written in Rust**, designed around a
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

VN_Rust is composed of **three clearly separated layers**, each with a distinct
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
| [`crates/vn_cli`](crates/vn_cli/README.md) | `vn` command-line tool (`vn check <path>`, `vn dump <file.story | dir>`) |
| [`examples/god_is_watching`](examples/god_is_watching/README.md) | Reference game built on `vn_engine` |

Each crate documents its API and behavior in its own README.

Games depend only on `vn_engine`.

## Running the Example

`god_is_watching` is the reference game. Run it from anywhere in the workspace:

```sh
cargo run -p god_is_watching
```

raylib is built from source on the first run, so you need CMake and a C compiler.

Its assets live in `examples/god_is_watching/assets/`:

| Path | Contents |
|---|---|
| `story/` | Chapter scripts, all loaded as one story. Each chapter ends with a `jump` into the next, and the game starts from the first scene of `01_mary.story` |
| `characters/<character_id>/<image_id>.png` | Character art, as referenced by `show <character_id> <image_id>` |
| `backgrounds/` | Reserved; backgrounds are not in the DSL yet |
| `fonts/` | `.ttf`/`.otf` fonts, assigned to text roles with `VnApp::font` in `main.rs` |
| `schema.json` | The game's registries, exported by the game in debug builds, for `vn check` |

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

## Writing Story Scripts

Story content is written using a custom DSL designed for clarity and structure.

**[Story Script Specification](SCRIPT.md)**

## Design Intent

VN_Rust intentionally avoids:

- Embedded scripting languages
- Runtime-evaluated logic
- Script-defined systems
- Implicit behavior

This keeps stories:

- Predictable
- Easy to validate
- Easy to refactor
- Friendly to tooling and large projects

## Project Status

Work in progress

Current focus:

- Core engine systems
- Story DSL parsing and validation
- Runtime interpreter
- Error diagnostics

Planned:

- Tree-sitter grammar
- Editor tooling (Neovim, VS Code, Zed)
- Save/load integration
- Example project
