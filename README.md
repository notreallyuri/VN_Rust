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

## Writing Story Scripts

Story content is written using a custom DSL designed for clarity and structure.

📘 **[Story Script Specification](SCRIPT.md)**

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
