# god_is_watching

The reference game, and the first real user of the `vn_engine` API.

```sh
cargo run -p god_is_watching
```

## Assets

The asset root is `CARGO_MANIFEST_DIR/assets`, resolved at compile time so
`cargo run -p god_is_watching` works from any directory. A shipped build would look
for assets next to the executable instead.

| Path | Contents |
|---|---|
| `story/` | One file per POV: `01_mary`, `02_moriarty`, `03_field_post`, loaded together (the engine's default `story_dir`). Each ends with a `jump` into the next; the game starts from the first scene of `01_mary.story` |
| `characters/<character>/<image>.png` | Character art for `show <character> <image>`. Missing art uses a placeholder |
| `backgrounds/` | Reserved |
| `fonts/` | Noto Serif (OFL, `fonts/OFL.txt`), used for the title and dialogue |
| `schema.json` | Exported registries (refreshed by debug runs, or `cargo run -p god_is_watching -- --export-schema`), used by `vn check` |

## Setup

`main.rs` is a single `VnApp` builder chain. It uses the engine's default screens and
changes only:

- **Fonts:** `NotoSerif-Regular.ttf` for `FontRole::Title` and `FontRole::Dialogue`;
  menus, buttons, speaker names and choices keep the engine's default font.
- **Start screen:** an upper-case prompt and a `v0.1.0` footer.
- **Main menu:** 260×52 buttons: New Game, Continue (`Goto(Playing)`, resumes the story),
  Credits, and a red Exit (a per-button style override).
- **Playing screen:** a taller dialogue box with slightly rounded corners, a larger
  speaker name, and HUD buttons: **Inventory** (a screen), **Stats** (an overlay),
  **Save** (the engine's save overlay) and **Menu** (the pause menu, same as Esc).
- **Credits:** a custom screen registered with `.screen(ScreenState::Custom("credits"), ...)`,
  in `src/screens/credits.rs`. It's drawn with the engine's `ui` helpers and returns to
  the menu with its Back button, Esc or Backspace.

- **Inventory:** `Inventory` (`src/inventory.rs`) is registered with `.state()`. The
  story fills it through `call give_item <item> [count]`, handled by `give_item` in
  `main.rs`. `InventoryScreen` (`src/screens/inventory.rs`) lists items with counts;
  Back, Esc, Backspace or I returns to the story at the same line.
- **Stats:** `StatsOverlay` (`src/screens/stats.rs`) dims the playing screen and shows the
  current scene and every story variable. Click, Tab, Esc or Backspace closes it.

- **Rollback:** mouse wheel up / Page Up steps back through the story, wheel down / Page
  Down forward (engine defaults).
- **Save/Load:** saves go to `examples/god_is_watching/saves/` (git-ignored). Esc opens the
  engine's pause menu (Resume, Save, Load, Quick Save, Quick Load, Main Menu, Quit); the
  main menu has Load; F5/F9 quick save and quick load. `Inventory` derives `Serialize`/`Deserialize`, so it's saved with the story.

- **Cast and variables:** `src/cast.rs` registers every character from the three stories
  (display name, name color, the images each one uses) and the variables `curiosity`,
  `obedience`, `suspicion` (int) and `player_name` (string, default "Reader"). The stories
  are validated against them at startup.
- **Commands:** `give_item <item> [count]` and `ask_name <variable>` (opens the text input
  screen), both with typed arguments. The story doesn't call `ask_name` yet; add
  `call ask_name player_name` where the reader should name themselves.

`01_mary.story` sets `curiosity`, `obedience` and `suspicion` in its first two choices,
and "Read it" gives the `verlaine_letter`.

See [vn_engine's README](../../crates/vn_engine/README.md) for every option.
