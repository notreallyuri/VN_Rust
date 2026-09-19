# TODO

Organized as milestones. Each one should leave the workspace building and runnable.

## Direction

VN_Rust is a Ren'Py alternative for **Rust developers**: writers work in `.story` files,
while all logic, state and UI live in Rust and can be debugged with normal Rust tooling.

A new game should get going with: `cargo new` → add `vn_engine` → a short `main.rs` that
registers characters/variables/commands → drop `.story` files in `assets/` → `cargo run`.

Three levels of control, each optional:

1. **Defaults**: write stories, register variables, done.
2. **Override pieces**: replace the main menu, dialogue box, choice screen, etc. via traits; hooks.
3. **Bring your own frontend**: use `vn_script` alone and drive the VM, which emits events.

### Workspace layout

| Crate | Role |
| --- | --- |
| `crates/vn_script` | DSL lexer, parser, compiler, VM. **No rendering deps**, reusable by CLI/LSP |
| `crates/vn_engine` | raylib engine: screens, resources, game loop. Re-exports `raylib` and `vn_script` (as `script`) |
| `crates/vn_build` | Build-script helper: embeds a game's assets in release builds |
| `crates/vn_cli` | `vn` binary: `vn new`, `vn check`, `vn dump`, `vn lsp`; later `run`, `fmt` |
| `examples/god_is_watching` | The reference game, and the first real consumer of the engine API |

---

## M0: Reorganization

- [x] Split `vn_core` into `vn_script` (no raylib) and `vn_engine`
- [x] Move the game out of `runtime/` into `examples/god_is_watching`
- [x] `test_compiler` → `vn_cli` (`vn dump <file>`, no hardcoded path)
- [x] Rename `StoryProvider` → `StoryVm`
- [x] Drop the legacy JSON story types (`runtime/types/story.rs`) and unused deps
- [x] Shared `[workspace.package]` / `[workspace.dependencies]`

## M1: Example game setup

- [x] Commit an assets layout for the example: `examples/god_is_watching/assets/{story,characters,backgrounds}`. `.gitignore` now ignores only the root `/assets/`.
- [x] Story adapted from the source documents (rewritten in M7)
- [x] Sample story covering every DSL feature: `crates/vn_script/tests/fixtures/all_features.story` (lexer, parser and VM tests)
- [x] Placeholder character art: generated, color-coded, labeled fallback texture in `ResourceManager`
- [x] Fix: the engine passed the script **path** to `StoryVm::from_source`. Added `StoryVm::from_file`.
- [x] Resolve asset paths relative to the game crate (`ResourceManager` has an asset root; the example passes `CARGO_MANIFEST_DIR/assets`)
- [x] README: how to run the example

## M2: VM as an event stream

- [x] Design the VM API: `advance() -> Event` (`Say`, `Choice`, `Show`, `Hide`, `Clear`, `Call`, `End`), `choose(i)`, plus a read-only view of state (active characters, variables)
- [x] Screens render from events/state, not `instructions[ip]` (`playing_screen.rs`)
- [x] Remove the `Say` + `Pause` double-block (was two clicks per line, with a blank dialogue box on the second)
- [x] VM auto-starts: leading `show`s run before the first line is displayed.
- [x] Compile the whole file into one program with a scene table (absolute jump targets).
- [x] Implement `Jump { scene_id }`
- [x] Choices: `choose(index)`, plus rendering and click handling in the example
- [x] Parser: `set`, `add` (`+=`/`-=`). `remove`/`clear` done in M1.
- [x] Parse real conditions (`<var> <op> <literal>`, `&&`, `||`) into a `Condition` tree evaluated by the VM
- [x] String values (`"..."`) alongside enums, in `set` and conditions (tokenized condition parser)
- [x] `{variable}` interpolation in dialogue, narration, choices and the speaker (`{player_name} "..."`)
- [x] Compile `call` (emitted as `Event::Call`; no command registry yet)
- [x] `advance()` is a loop (was recursive `step()`), with a guard against jump loops that never produce an event.
- [x] New Game after returning to the menu reuses the old VM state. Add a reset.

## M3: App builder and registries (the "how do I get going" API)

- [x] `VnApp` builder in `vn_engine`: title, window size, assets, story directory and entry scene, fonts, `.run()`
- [x] Character registry (id → display name, color, image set); display names may use `{variable}`
- [x] Text input screen (`ctx.ask_text`), used by the example's `ask_name` command
- [x] Variable registry (id → typed default: bool, int, enum, string)
- [x] Command registry (`.command(name, handler)`)
- [x] Typed command args (`|ctx, (item, count): (String, Option<u32>)|`)
- [x] Validation against the registries at startup, with file:line diagnostics (`AppError::Script`)
- [x] Game state by type (`.state(T)`, `ctx.state.get_mut::<T>()`), reset on New Game
- [x] Overlays (`.overlay(name, ..)`, `Action::overlay`) and HUD buttons on the playing screen
- [x] Returning to the playing screen shows the same line (`StoryVm::current()`)
- [x] Default screens shipped by the engine (start, main menu, playing), configured through the builder (`start_screen`, `main_menu`, `playing`); the game overrides them per `ScreenState` with `.screen()`
- [x] Pause menu overlay on Esc (Resume, Save, Load, Quick Save, Quick Load, Main Menu, Quit) with Save/Load as overlays, an overlay stack, and engine-level notifications
- [x] Confirmation dialogs (`Action::confirm`): pause → Main Menu / Quit, overwriting a save, loading during play
- [x] Rollback (wheel / Page Up-Down) with barriers: `commit`, `choice final:`, `through_choices`, blocked commands
- [x] Rollback history in save files (rolling back after a load; history in edited scenes is dropped)
- [x] Confirm on the window close button during a game (raylib reports it for one frame; `request_close`)
- [x] Default settings screen (display, text speed; screen from the main menu, overlay from the pause menu), saved to `settings.json`
- [x] Engine-provided UI helpers (`ui::draw_button`, text, layout, backgrounds) and styles (`TextStyle`, `ButtonStyle`, `DialogueBoxStyle`)
- [x] Hooks: `on_scene_enter`, `on_choice` (VM `Event::SceneEnter`, opt-in)
- [x] Hot reload of `.story` files in debug builds (position kept or scene restarted, errors keep the old story)
- [x] `ScreenState` is now `StartScreen`, `MainMenu`, `Playing`, `Save`, `Load`, `TextInput`, `Settings`, `Custom(String)`, `Quit`

## M4: Validation and diagnostics

- [x] Replace `panic!`/`unwrap` in lexer/parser with `Diagnostic`s, with recovery so one broken line doesn't hide the rest (`compile_source`)
- [x] Validate against registries: characters/images, variables and types, command arity and types (M3)
- [x] Choice options non-empty (text and body), `choice:` has at least one option
- [x] Identifiers match `[a-z_][a-z0-9_]*` everywhere (scenes, characters, images, commands, variables, enum members); tabs in indentation are errors
- [x] `{variable}` in text must name a registered variable
- [x] Export the registry as a schema file (`schema.json`, refreshed in debug builds, `--export-schema`) so `vn check` and the LSP can validate without running the game
- [x] `vn check <path>`: whole story or one file in its project, schema found by walking up, exit code for CI
- [x] "Did you mean" suggestions: a first word close to a keyword (`remoe hugo` → `remove`), and unknown scenes, characters, images, variables, commands and entry scenes close to a known one (`marry` → `mary`)
- [x] Show script errors in the game (debug builds): a panel listing `file:line: message` when a hot reload fails (F2 collapses it), cleared by the next good reload
- [x] Parser edge cases:
  - [x] narration ending in `:` is lexed as `ChoiceOption` and panics
  - [x] an empty `if`/`else`/option body swallows the following siblings
  - [x] a stray `else:` or choice option outside its block panics
  - [x] `show` with a missing image id panics
  - [x] no string escaping: `\"` and `\\` (SCRIPT.md 3.7)
  - [x] a keyword-named character (`show "hi"`): keywords are reserved (SCRIPT.md 6.1)
- [x] `JumpIfFalse` handles only Int/Int and Bool/Bool, and ignores `op` for bools (now Int/Bool/Enum; ordering on bools/enums is a parse error)
- [x] `Add` on an unknown variable silently creates `Int(0)` (an error once variables are registered)
- [x] `ResourceManager::get_or_load` panics on a missing texture. Use a placeholder and log a warning. (M1)

## M5: Presentation

- [x] Character positions: `show mary happy at left` (five spots; unpositioned characters are spread evenly), sprites scaled to the window height
- [x] Backgrounds: `background <id>` / `background none`, drawn to cover the window, saved and rolled back with the story
- [x] Transitions: `with dissolve | fade | slide_left | slide_right [seconds]` on `show`, `background`, `remove`, `clear`; expression crossfades and position glides; non-blocking, a click finishes them
- [x] Fonts: built-in Noto Sans default, per-role fonts (`FontRole`) loaded from `assets/fonts/`
- [x] Text wrapping (`Fonts::wrap`, `ui::draw_text_wrapped`)
- [x] Typewriter effect (text speed setting; click shows the whole line)
- [x] Window-relative layout in the default screens
- [x] Layouts for button lists (`Layout`: column, row, grid, rows_of, custom; anchors, alignment, fitted spacing) in the main menu, pause menu, choices, HUD and save slots
- [x] Button styling: borders, shadows, images (stretched or nine-slice), icons, alignment, padding, overflow (ellipsis, shrink, wrap), transforms (scale, rotate, skew, offset), hovered/pressed/focused/disabled looks with transitions, hover and click sounds, clicks on release, per-button HUD and choice styles, `MenuItem::enabled_if`
- [x] Keyboard/gamepad navigation: spatial focus (columns, rows, grids, wrapping), Tab, key and stick repeat, focus that follows the mouse, gamepad A/B/X/Start/LB/RB, in every default screen
- [x] Settings screen (see M3): a display toggle, and sliders for text speed, music and sound volume
- [x] Tooltips (`MenuItem::tooltip`, `HudButton::tooltip`, `ctx.tooltip(rect, text)`), on by default in the settings rows and save slots
- [x] Clean exit (`ScreenState::Quit`)
- [x] Ren'Py playing controls: hide (H, middle click), skip (Ctrl held, Tab; read lines only unless set otherwise, remembered in `seen.json`), auto mode (A; waits for typing, transitions and voice), log (L, HUD button), screenshot (S), fullscreen (F), right click for the menu; gamepad equivalents
- [x] Controls overlay on F1, generated from the configured keys, with game-specific sections
- [x] Corner shapes (`Corners`: square, round, bevel, scoop, notch, per corner) and `PanelStyle` (fill or gradient, border, inner rule, shadow), with feathered curves; used by buttons, the dialogue box and every panel. Optional panels on the main menu (incl. a sidebar), settings, save/load and text input; a name plate for the speaker; the dialogue box's `bottom`/`max_width`; main menu title alignment and subtitle
- [x] Session log overlay (lines and choices; rolled back with the story, kept in saves)
- [x] `voice <id>` clips with a voice volume setting
- [x] Audio: `music <track>` / `music none` / `sound <id>` (SCRIPT.md 2.7); looping music with crossfades, part of the story state (saves, rollback, hot reload); menu music; volumes in settings

## M6: Save/load and tooling (save/load started early)

- [x] Save/load: VM snapshot (scene + offset + scene fingerprint, variables, characters, current line), game state by type name, JSON slots with atomic writes, all-or-nothing loads, `SaveError` / `LoadWarning`, default Save/Load screens, quick save/load
- [x] Autosave (on scene change / on quit), save thumbnails, delete-slot and overwrite confirmation in the default screens; `Action::Continue` loads the newest save
- [x] Platform save directory (e.g. `~/.local/share/<game>`) instead of `./saves`
- [x] Save format migrations: engine steps for `SAVE_FORMAT_VERSION`, and game steps (`.save_version(n)`, `.migrate_save(from, ..)`) that rename/edit state and variables in the save and its rollback history
- [x] Release builds that run anywhere: `Assets` (a folder or files embedded in the executable) behind every load, `vn_build` embedding `assets/` from `build.rs` in release builds, `embedded_assets!()`, and an `assets` folder next to the executable as a fallback
- [x] `vn new` project template (Cargo.toml, `main.rs`, a two-scene story, a matching `schema.json`)
- [x] Multi-file projects: every `.story` under `story_dir` is one program; diagnostics name their file; the example's chapters jump into each other
- [ ] Editor support for `.story` files:
  - [x] `tree-sitter-story` grammar (`editors/tree-sitter-story`), with an external scanner for indentation (INDENT/DEDENT, like tree-sitter-python), `highlights.scm` and `folds.scm`
  - [ ] Corpus tests from `all_features.story` (`test/corpus/` is still empty; they would have caught the EOF loop that made parsing run out of memory)
  - [x] Neovim (`editors/nvim`): filetype detection, the grammar and queries through nvim-treesitter, folds, and indentation as an `indentexpr` (tree-sitter indent queries don't work well while typing in an indentation-sensitive grammar)
  - [ ] Zed, Helix and VS Code: not a focus for now. Zed and Helix reuse the grammar and queries; VS Code needs a TextMate grammar
  - [x] LSP (`vn lsp`, reusing `vn_script` diagnostics and the exported schema): errors as you type (unsaved buffers included), go to scene definition, completion of scenes, characters, images, variables, values, commands, positions, transitions and asset ids, hover, scene outline. Setup for Neovim and Helix in vn_cli's README
  - [x] Formatter (`vn fmt <path> [--check]`, `vn_script::format`): two-space indentation, spacing around operators, comments and blank lines normalized, strings untouched; only rewrites a file when the result compiles to the same program

## M7: Example overhaul

`examples/god_is_watching` is the showcase and the end-to-end test of every feature.

- [x] Rework the plot: the player is an archivist of the House in 1903, reading the 1894 documents and going to Santa Ilde; three endings, one hidden
- [x] Use every DSL feature in the story itself (see the example README's feature tour)
- [x] Use every engine feature (registries, commands, state, text input, hooks, overlays, custom screens, saves, rollback barriers, settings, confirmations, layouts, backgrounds, positions, hot reload)
- [x] A consistent visual style for the UI (`src/style.rs`, applied to every default screen)
- [x] Generated art (`tools/generate_art.py`): 11 backgrounds, 23 portraits
- [x] A feature tour in the example README
- [x] Real art to replace the generated placeholders (Will change to proper styling later)
- [x] Music and sound: 5 CC0 tracks and 7 CC0 sounds (`assets/AUDIO_CREDITS.md`)
- [x] Cinematic title and menu: `Scenery` (background motion, vignette, sliding letterbox) on the start screen and main menu, a title card on the start screen, menu buttons in the letterbox bar with diamond separators and an intro fade, `TextStyle::spacing`, `ButtonLook::underline`, `Button::opacity`; HUD groups (`HudButton::group`, `hud_group`)
- [x] UI overhaul: one visual system in `style.rs` taken from the art (warm ink, brass, parchment, oxblood for irreversible actions), scooped frames on every panel, beveled buttons with hover changes limited to fill and rule, a sidebar main menu, a framed dialogue box with a name plate and the HUD as a row beneath it, panels on the custom screens

## M8: Localization

Worth doing before the engine grows further: it touches the DSL, every default screen,
fonts and saves at once, and each of those is cheaper to change now than later.

- [ ] `vn translate <lang>`: extract every translatable string (dialogue, narration, choice options, and the display names in `schema.json`) into a per-language table keyed by file and a hash of the source text, so edits show up as stale entries instead of silently keeping the old translation
- [ ] Look translations up at runtime through the VM's `Say`/`Choice` events, falling back to the source text when one is missing
- [ ] A language setting in `settings.json`, changed from the settings screen without restarting
- [ ] Translatable UI labels: the default screens ship their English strings as a table a game can replace or extend
- [ ] Fonts per language with a fallback chain, and wrapping for scripts without spaces (CJK)
- [ ] `vn check` reports missing and stale translations for a language
- [ ] Saves stay language-independent: store ids, not translated text, so the session log and save slots re-render in whatever language is active

## M9: UI and presentation

Split by what gets harder to add later. The first group changes code that everything
else draws through, so it is cheaper now; the rest is additive.

### Foundations

- [ ] Inline text markup (`{b}`, `{i}`, `{color=...}`, `{size=...}`, `{w}` waits, ruby text for furigana), parsed into styled spans in `vn_script` so `vn check` validates tags. Touches `Fonts::wrap`, `ui::draw_text_wrapped`, the typewriter (reveal across spans) and the log at once, and translated lines carry the same tags, so it belongs with M8
- [ ] Draw the game to a `RenderTexture` instead of straight to the screen. Nothing does this today, and it is what the next three items need
- [ ] Screen transitions: `ScreenManager::transition_to` swaps instantly; crossfade, fade through black and slide between screens, reusing the transition kinds stories already have
- [ ] A shared easing/tween helper (`stage.rs` has a private `ease`, buttons roll their own), used by sprites, buttons, screen transitions and scenery
- [ ] A reusable scroll container (wheel, drag, scrollbar, keyboard and gamepad through `navigation.rs`), today only inside the log screen; needed by settings, keybindings, save lists and a gallery
- [ ] Resolution independence: design at one size, letterbox or pillarbox the render target on other aspect ratios (ultrawide included)
- [ ] Remove the debug `println!` on every screen change (`screen_manager.rs:491`)

### Presentation

- [ ] Screen shake and flash, as `with shake` / `with flash` on story lines
- [ ] Shader passes over the render target: blur behind panels, grain or CRT for flashbacks, a desaturation pass for endings
- [ ] Weather and particle overlays (rain, snow, dust motes) as part of `Scenery`
- [ ] NVL mode: full-screen text pages instead of the dialogue box, chosen per scene
- [ ] Dialogue box variants: a speaker portrait bust inside the box, and a per-character box style
- [ ] Choice presentation: images, disabled options with a reason, and hover previews
- [ ] A custom mouse cursor, and prompts that show keyboard or gamepad glyphs depending on the last input used
- [ ] Video playback (openings, endings, in-scene cutscenes): raylib has no decoder, so this needs ffmpeg or a pure-Rust decoder feeding frames into a texture, behind an optional feature so a game that doesn't use it doesn't pay for it

### Interaction

- [ ] Image maps / point-and-click scenes: named hotspots over a background, each with a hover look and an action, used for examining a room or picking a destination
- [ ] Drag and drop: draggable items with drop targets and a validity rule, for inventory puzzles and minigames; needs the same hit-testing as image maps, so the two share a base

### Extras

- [ ] Persistent data across playthroughs, next to the existing `seen.json`: unlocked endings, CG and tracks
- [ ] A CG gallery and a music room, as default screens a game can enable
- [ ] Achievements with a toast on unlock, reusing `ScreenManager::notify`

### Accessibility

- [ ] A UI scale and text size setting, applied through the style system
- [ ] A reduce-motion setting: skip transitions, hold scenery still, no shake
- [ ] Readability options: text outline or a dimmed backdrop behind dialogue over bright backgrounds
- [ ] A self-voicing hook (the engine hands the current line to a game-provided reader)

## Ongoing

- [x] Tests: golden tests over the SCRIPT.md examples (`crates/vn_script/tests/spec.rs`: each example compiles cleanly, its listing and VM events match `tests/golden/script_md.txt`); `Program::listing()` and `Display for Instruction` shared with `vn dump`
- [x] SCRIPT.md §8.3 typo: `-+` → `-=`
- [x] SCRIPT.md: specify audio (backgrounds, positions and string escaping done)
