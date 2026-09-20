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

- [x] Inline text markup: `[b]`, `[i]`, `[color=#rrggbb]`, `[size=N]` and `[w]` waits, parsed into spans in `vn_script::markup` and validated by `vn check`; span-aware wrapping, drawing, typewriter and log in the engine, with bold/italic font variants (`VnApp::font_variant`). Square brackets because `{...}` is variable interpolation
  - [ ] Ruby text for furigana, left out of the first pass
- [x] Draw the game to a `RenderTexture` instead of straight to the screen (`RenderTarget` in `target.rs`), recreated on resize, with a fallback to drawing to the screen; screenshots and thumbnails still capture the game image
- [x] Screen transitions (`ScreenTransitionConfig`): crossfade (the default, 0.2s), fade through black and slide, built on a snapshot of the previous frame
- [x] A shared easing/tween helper (`Easing`, `Tween` in `ease.rs`), used by the stage and screen transitions; buttons still have their own timing to move over
- [x] A reusable scroll container (`Scroll`, `ScrollStyle`): wheel, thumb dragging, track clicks, keys and gamepad, with a scrollbar; the log screen now uses it
- [x] Resolution independence: `VnApp::design_size` fixes the layout size and the frame is scaled, centred and letterboxed; `viewport` maps mouse positions back so input follows the picture
- [x] Remove the debug `println!` on every screen change

### Presentation

- [x] Screen shake and flash (`with shake` / `with flash`, `ctx.shake`/`ctx.flash`): the change is instant and the frame is shaken by offsetting the render target, or washed with `flash_color`; `ScreenEffectsConfig` tunes it
- [x] Shader passes over the render target (`PostChain`, `VnApp::shader`, `ctx.shader`): named passes in registration order, with `amount`, `time` and `pixel` uniforms; `post::GRAIN`, `DESATURATE`, `BLUR` and `FXAA` ship with the engine
- [ ] Weather and particle overlays (rain, snow, dust motes) as part of `Scenery`
- [ ] NVL mode: full-screen text pages instead of the dialogue box, chosen per scene
- [ ] Dialogue box variants: a speaker portrait bust inside the box, and a per-character box style
- [ ] Choice presentation: images, disabled options with a reason, and hover previews
- [x] Sharper edges: `VnApp::render_scale` supersamples the frame (the render target is not multisampled, so window MSAA would not help), and corner segments scale with corner size and render scale; `post::FXAA` is the cheap alternative
- [ ] A custom mouse cursor, and prompts that show keyboard or gamepad glyphs depending on the last input used
- [ ] Animated character puppets (Live2D and friends). On the shelf; checked 2026-09-20, start with the seam when it comes off it
  - **The seam, first and separately (~1 day).** Characters are drawn in one place, `sprite()` in `stage.rs`: a texture looked up by `character_path(id, image)` and drawn with `draw_texture_pro`. Positions, transitions and alpha all pass through it, so a trait that draws itself into a rect at an alpha is all it takes. Every backend below needs this, and the layered rig is enough to prove it
  - **Rendering is not a problem.** raylib-rs exposes `rlBegin`/`rlVertex2f`/`rlTexCoord2f`/`rlSetTexture` for arbitrary textured triangles, `rlSetBlendFactors` and `BeginBlendMode` for multiply and add parts, and `LoadShaderFromMemory` for mask shaders (checked by compiling against them). A puppet renderer stays inside raylib: no second GL context, no `glow`, no context sharing. Clipping masks are practical now that frames go through a render target
  - **Layered sprite rig, in-house (~1 week): the one to build.** Parts as separate images with a pivot, parameters driving rotation, scale and offset, a small curve evaluator; `draw_texture_pro` already takes rotation and an origin, so there is no new drawing code. Breathing, blinking, mouth flap, head tilt and sway is most of the life a VN needs, with no licence and no FFI
  - **Cubism (3-6 weeks), as a separate optional crate, never a `vn_engine` dependency.** `live2d-cubism-core-sys` (v0.1.0, April 2026, SDK Native v5, MIT binding only) gives the Core: set parameters, read back vertices, UVs, indices, opacity, blend mode, masks and draw order. Rendering that is the easy half; the hard half is the C++ Cubism Framework nobody has ported, i.e. `motion3.json` playback with curve blending, `physics3.json` pendulums, expressions and lip sync. Needs Live2D's proprietary Core downloaded per game: free to develop with, free to release under 20M JPY a year, a Publication License Agreement above that, signed a month before release. `live2d-parser` reads model files in pure Rust if we only need to inspect them
  - **`inox2d` (2-3 weeks)**: Inochi2D in Rust, BSD-2, free tooling in Inochi Creator, no proprietary blob, but its README calls it a prototype, mesh groups and animations are missing, and we would write the rlgl renderer anyway. `cubism-rs` is stale and there is no `live2d-rs`
  - **Prerequisite for all of them**: art authored in parts (eyes, mouth, hair, body as layers). The example's art is flat PNGs, so it would have to be re-exported; Live2D additionally means an artist in the Cubism Editor
- [ ] Video playback (openings, endings, in-scene cutscenes): raylib has no decoder, so this needs ffmpeg or a pure-Rust decoder feeding frames into a texture, behind an optional feature so a game that doesn't use it doesn't pay for it

### Interaction

- [x] Image maps / point-and-click scenes (`ImageMap`, `Hotspot`): named hotspots over a background, each with a label, tooltip, hover look and action. Shapes are fractions of the picture, so they follow it when the window doesn't match its aspect
- [x] Drag and drop (`DragBoard`, `Draggable`, `DropTarget`): items dragged onto targets that accept or refuse them, with the pointer or the keyboard. It shares `hit.rs` with image maps: `Shape` (rect, circle, polygon), picking the topmost, `Highlight` and `LabelStyle`

### Extras

- [ ] Persistent data across playthroughs, next to the existing `seen.json`: unlocked endings, CG and tracks
- [ ] A CG gallery and a music room, as default screens a game can enable
- [ ] Achievements with a toast on unlock, reusing `ScreenManager::notify`

### Accessibility

- [ ] A UI scale and text size setting, applied through the style system
- [ ] A reduce-motion setting: skip transitions, hold scenery still, no shake
- [ ] Readability options: text outline or a dimmed backdrop behind dialogue over bright backgrounds
- [ ] A self-voicing hook (the engine hands the current line to a game-provided reader)

## M10: Authoring and dev tools

Not a visual UI editor: the point of the project is that logic stays in Rust and is
debugged with Rust tooling, and a WYSIWYG editor is what Godot and Unity already do
better. Ren'Py reached the same conclusion, and answers it with hot reload plus live dev
tools instead, which is what this milestone copies. Everything here is debug-build only
and sits beside the existing hot reload (`VnApp::hot_reload`, on by default in debug
builds), F1 (controls) and F2 (script errors).

- [ ] Debug overlay on F3: layout rectangles, focus order, the style and rect of the widget under the cursor, and frame timing. The equivalent of Ren'Py's inspector, and the fastest way to find a layout bug
- [ ] Live style tweaking: hot-reload the style values a game defines (the example keeps them in `src/style.rs`), or edit them in an overlay with sliders and colour pickers that writes the tweaked values back out as Rust to paste. This is the part of a UI editor people actually want
- [ ] Position picker: drag a sprite in the running game and get the `at` position, or exact coordinates, to paste into the `.story` line. Ren'Py's image location picker
- [ ] Director: while playing, pick `show`, `background`, `music`, `sound` or `voice` from a menu, see it applied live, and write the line into the `.story` file at the current point. `.story` is line-oriented and `vn fmt` normalizes whatever a tool writes, so generated lines can't drift from hand-written style
- [ ] Scene jump: a debug menu listing every scene, to jump straight to one with its variables set, instead of replaying to reach it
- [ ] A screenshot and GIF capture key for bug reports and devlogs, writing next to the existing screenshot key

## Ongoing

- [x] Tests: golden tests over the SCRIPT.md examples (`crates/vn_script/tests/spec.rs`: each example compiles cleanly, its listing and VM events match `tests/golden/script_md.txt`); `Program::listing()` and `Display for Instruction` shared with `vn dump`
- [x] SCRIPT.md §8.3 typo: `-+` → `-=`
- [x] SCRIPT.md: specify audio (backgrounds, positions and string escaping done)
