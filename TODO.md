# TODO

Organized as milestones. Each one should leave the workspace building and runnable.

## Direction

novn is a Ren'Py alternative for **Rust developers**: writers work in `.story` files,
while all logic, state and UI live in Rust and can be debugged with normal Rust tooling.

A new game should get going with: `cargo new` → add `novn` → a short `main.rs` that
registers characters/variables/commands → drop `.story` files in `assets/` → `cargo run`.

Three levels of control, each optional:

1. **Defaults**: write stories, register variables, done.
2. **Override pieces**: replace the main menu, dialogue box, choice screen, etc. via traits; hooks.
3. **Bring your own frontend**: use `novn_script` alone and drive the VM, which emits events.

### Workspace layout

| Crate | Role |
| --- | --- |
| `crates/novn-script` | DSL lexer, parser, compiler, VM. **No rendering deps**, reusable by CLI/LSP |
| `crates/novn` | raylib engine: screens, resources, game loop. Re-exports `raylib` and `novn_script` (as `script`) |
| `crates/novn-build` | Build-script helper: embeds a game's assets in release builds |
| `crates/novn-cli` | `novn` binary: `novn new`, `novn check`, `novn dump`, `novn lsp`; later `run`, `fmt` |
| `examples/god_is_watching` | The reference game, and the first real consumer of the engine API |

---

## M0: Reorganization

- [x] Split `novn_core` into `novn_script` (no raylib) and `novn`
- [x] Move the game out of `runtime/` into `examples/god_is_watching`
- [x] `test_compiler` → `novn_cli` (`novn dump <file>`, no hardcoded path)
- [x] Rename `StoryProvider` → `StoryVm`
- [x] Drop the legacy JSON story types (`runtime/types/story.rs`) and unused deps
- [x] Shared `[workspace.package]` / `[workspace.dependencies]`

## M1: Example game setup

- [x] Commit an assets layout for the example: `examples/god_is_watching/assets/{story,characters,backgrounds}`. `.gitignore` now ignores only the root `/assets/`.
- [x] Story adapted from the source documents (rewritten in M7)
- [x] Sample story covering every DSL feature: `crates/novn-script/tests/fixtures/all_features.story` (lexer, parser and VM tests)
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

- [x] `VnApp` builder in `novn`: title, window size, assets, story directory and entry scene, fonts, `.run()`
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
- [x] Export the registry as a schema file (`schema.json`, refreshed in debug builds, `--export-schema`) so `novn check` and the LSP can validate without running the game
- [x] `novn check <path>`: whole story or one file in its project, schema found by walking up, exit code for CI
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
- [x] Release builds that run anywhere: `Assets` (a folder or files embedded in the executable) behind every load, `novn_build` embedding `assets/` from `build.rs` in release builds, `embedded_assets!()`, and an `assets` folder next to the executable as a fallback
- [x] `novn new` project template (Cargo.toml, `main.rs`, a two-scene story, a matching `schema.json`)
- [x] Multi-file projects: every `.story` under `story_dir` is one program; diagnostics name their file; the example's chapters jump into each other
- [ ] Editor support for `.story` files:
  - [x] `tree-sitter-story` grammar (`editors/tree-sitter-story`), with an external scanner for indentation (INDENT/DEDENT, like tree-sitter-python), `highlights.scm` and `folds.scm`
  - [ ] Corpus tests from `all_features.story` (`test/corpus/` is still empty; they would have caught the EOF loop that made parsing run out of memory)
  - [x] Neovim (`editors/nvim`): filetype detection, the grammar and queries through nvim-treesitter, folds, and indentation as an `indentexpr` (tree-sitter indent queries don't work well while typing in an indentation-sensitive grammar)
  - [ ] Zed, Helix and VS Code: not a focus for now. Zed and Helix reuse the grammar and queries; VS Code needs a TextMate grammar
  - [x] LSP (`novn lsp`, reusing `novn_script` diagnostics and the exported schema): errors as you type (unsaved buffers included), go to scene definition, completion of scenes, characters, images, variables, values, commands, positions, transitions and asset ids, hover, scene outline. Setup for Neovim and Helix in novn_cli's README
  - [x] Formatter (`novn fmt <path> [--check]`, `novn_script::format`): two-space indentation, spacing around operators, comments and blank lines normalized, strings untouched; only rewrites a file when the result compiles to the same program

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

- [x] `novn translate <lang> [path]`: extracts every translatable string (dialogue, narration, choice options, and the display names in `schema.json`) into `lang/<lang>.json` beside the schema, keyed by the story file's name and a hash of the source text. Re-running keeps the translations already written, adds what is new, and marks what was edited or deleted `stale` (with the old text to work from) instead of silently keeping it. A story with errors extracts nothing
- [x] Look translations up at runtime through the VM's `Say`/`Choice` events, falling back to the source text when one is missing (`novn_script::translate::Catalog`, `StoryVm::set_catalog`, `language()`). The lookup runs before interpolation, so `{variable}` works inside a translation
- [x] The engine side: `VnApp::source_language` / `language(code, label)`, `lang/<code>.json` read through `Assets` (folder or embedded), a `language` setting saved in `settings.json` and applied at startup, a Language row in the settings screen when a game ships more than one, `ctx.set_language` / `apply_language`, and a hot reload that keeps it. Changing language re-renders the line being read
- [x] Translatable UI labels: every string the default screens show goes through `ctx.label` / `ctx.message` and is looked up in the catalog's `ui` section by its English text, so a game's own labels and notifications are translatable with nothing to declare. A debug build writes `lang/ui.json` (what the game actually shows, read from the live configs) and `novn translate` folds it into the catalog; `VnApp::ui_text` adds strings a game builds itself
- [x] Fonts per language with a fallback chain (`VnApp::language_font`; language+role → language+Default → role → Default → built-in), a glyph set read from the story and every catalog so a CJK atlas covers what the game shows and no more, and wrapping between characters for scripts without spaces, with kinsoku rules (`ui::wrap`)
- [x] `novn check` reports missing and stale translations for a language: a line per catalog in `lang/`, counted against the story as it is now, and `--lang <code>` lists each one with its file and line
- [x] `novn translate <lang> --export` / `--import <file.po>`: the hash-keyed JSON stays the runtime format, and a PO file beside it is what a translator edits (Poedit, Weblate, Crowdin, OmegaT). Exported in story order with the speaker and file:line as comments, `msgctxt` carrying the catalog key; `stale` travels as `#~` obsolete and a translator's own `fuzzy` flag as `#, fuzzy`, which `Entry::translated()` withholds until it is cleared
- [x] Saves stay language-independent: the log and the save slots store the speaker's id, the story file, the line as written and the values it was read with (`Spoken`, `SavePoint`), and re-render through the active catalog; `time_ago` returns an `Elapsed` the screens fill in with `ctx.message`. `SAVE_FORMAT_VERSION` 2; older saves show their stored summary

## M9: UI and presentation

Split by what gets harder to add later. The first group changes code that everything
else draws through, so it is cheaper now; the rest is additive.

### Foundations

- [x] Inline text markup: `[b]`, `[i]`, `[color=#rrggbb]`, `[size=N]` and `[w]` waits, parsed into spans in `novn_script::markup` and validated by `novn check`; span-aware wrapping, drawing, typewriter and log in the engine, with bold/italic font variants (`VnApp::font_variant`). Square brackets because `{...}` is variable interpolation
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
- [x] Weather and particle overlays (`Weather::rain`/`snow`/`dust`): `Scenery::weather` on the menus, `ctx.weather` during a scene. Each particle's position comes from the clock and its index like `Motion::at`, so there is no simulation state, nothing in a save and nothing for rollback to desynchronise; not restored by a load, the same as `ctx.shader`
- [x] NVL mode: full-screen text pages instead of the dialogue box, chosen per scene
  (`scene <id> nvl:`, `Program::scene_modes` / `StoryVm::scene_mode`, `NvlStyle`). The
  page is rebuilt each frame from the log's lines for the current scene, so it is saved,
  rolled back and retranslated with everything else and the engine stores no page of its
  own; it turns over when the next line no longer fits. The notebook chapter of the
  example opens in NVL
- [x] Dialogue box variants: `Character::box_style(|b| ...)` layers a character's own box over the game's base, inheriting what it does not name; `DialogueBoxStyle::bust` plus `Character::bust(file)` draws a portrait inside the box and moves the text out of its way, keeping its aspect ratio and standing on the box's floor, with `rise`/`sink` to break the edges
- [x] Choice presentation: `"text" when <cond> "reason"` / `unless` gates an option in the
  story — without a reason it is not offered at all, with one it is drawn disabled, with
  the reason as its tooltip, the `NotAllowed` cursor and no focus. `image <id>` and
  `preview <id>` name `choices/<id>.png` and `previews/<id>.png`; `ChoiceImageStyle` fills
  the button or puts an icon beside the label, `ChoicePreviewStyle` draws the hovered or
  focused option's preview. `Event::Choice` now carries a `ChoiceOption` per offered
  option (its own `index`, `enabled`, `reason`, `image`, `preview`), and `choose` refuses
  an option whose condition fails; `novn check` warns when every option of a choice can be
  hidden. Save format 3, migrating the old choices in mid-choice saves
- [x] Sharper edges: `VnApp::render_scale` supersamples the frame (the render target is not multisampled, so window MSAA would not help), and corner segments scale with corner size and render scale; `post::FXAA` is the cheap alternative
- [x] Cursor states wherever they mean something — `Text` over the text field, `Hand` over anything clickable including image-map hotspots and save slots, `Grab`/`Grabbing` for drag boards, sliders and scroll thumbs, `NotAllowed` over disabled items, empty load slots and refusing drop targets — merged by priority, falling back through the pictures a game supplies, with a hotspot per picture. Without pictures they drive the system's own shapes. Games choose their own per element (`ButtonStyle::cursor`, `Hotspot::cursor`, `CursorKind::Custom`), per frame (`ctx.cursor`) or regardless (`ctx.override_cursor`, owned by the screen that set it)
- [x] A custom mouse cursor (`VnApp::cursor`, `CursorStyle`: hotspot, size, an optional hand over anything clickable, hidden while a gamepad is in use) and prompts that name the right control: `Navigation` now tracks the last `InputDevice`, and `ctx.prompt("{advance} to go on")` fills tokens from the same config the controls overlay reads, after translation so the token can move
- [x] Optional native video cutscenes, before M9 Extras: `video` selects rav1d + Matroska demuxing + Vorbis; `video-ffmpeg` selects FFmpeg, including when both features are enabled. Neither is enabled by default
  - [x] Shared `ctx.play_video(VideoRequest)` API and `ScreenState::Video`, launched through a registered command; blocking playback with skip, pause, overlay suspension, aspect-preserving letterboxing and return to the story. Voice stops, music is muted while continuing underneath, movie audio uses sound volume, and starting a cutscene marks a rollback barrier. No mid-movie position in saves; autosave is suppressed during playback, and hot reload cancels the movie
  - [x] Worker-thread decoding, bounded frame/audio queues, timestamped RGBA uploads, streamed stereo audio with a callback-driven sample clock, silent playback, EOF draining, cancellation and error notifications. Folder and embedded assets work; FFmpeg stages embedded media in a temporary file. Colour conversion is on the CPU for this first pass
  - [x] Desktop H.264/H.265 rejection by codec ID, independent of extension or the linked FFmpeg build. The policy reserves these codecs for browser WASM targets; browser playback is not implemented yet
  - [x] Generated AV1/Vorbis, VP9/Opus, silent and rejected-codec fixtures; decoder, audio timeline and cancellation tests; a runnable `novn` video example and usage/packaging notes in its README
  - [x] End-to-end playback on Linux (Ryzen 5 5600X, release, SIMD): 60 s 1080p30 AV1/Vorbis at 1.1 and 28 Mbit/s, on all cores and pinned to two. The queue never ran dry; CPU averaged 31–34% of one core at 1.1 Mbit/s and ~90% at 28 Mbit/s; peak RSS ~250 MiB against ~100 MiB for a tiny clip; texture upload averaged 0.6 ms on the main thread. The audio clock stood still in 18% of ticks and jumped up to 37 ms, dropping up to 100 of 1800 frames; interpolating it between callbacks brought that to 0–2
  - [ ] Release validation on Windows/macOS and genuinely low-end hardware (two pinned desktop cores is not an old laptop); real footage instead of generated clips, and shipped size. The earlier conversation's synthetic decoder-only 66/250/341 fps figures are not end-to-end acceptance results. FFmpeg currently requires externally supplied matching development/runtime libraries; automated trimmed LGPL builds and distribution remain separate work
  - [ ] Later extensions: GPU conversion after profiling, looping menu/background video, seeking and browser playback. Keep M9 Extras focused on its existing persistence/seen-record primitives

#### Animated character puppets (Live2D and friends)

Started 2026-09-20. The seam, both backends and their state exist; what is left is listed
at the end. The APIs are documented in
[the guide](https://notreallyuri.github.io/novn/docs/engine/visuals) for the seam and the
puppet, and [`crates/novn-live2d`](crates/novn-live2d/README.md) for Cubism. Four layers,
bottom to top, then the gaps.

- [x] **The seam** (`novn`, behind the off-by-default `character-visuals` feature).
  `CharacterVisualFactory` is what a game registers with `VnApp::character_visual`: it
  names its appearances, validates its assets without a window, and loads one instance.
  `CharacterVisual` is that instance: natural size, update, draw into a rect at an alpha,
  restart, set_parameter. A backend's appearances join the schema, so `show mary happy`
  validates against a rig exactly as against a folder of PNGs, and `novn check` names a
  missing part before a window opens. One instance per character *and* appearance, kept
  while it fades out and dropped when it leaves. Nothing about a backend is load-bearing:
  any failure prints one line and falls back to `characters/<id>/<appearance>.png`,
  without retrying every frame
- [x] **The puppet backend** (`novn`'s own `game::puppet`, no SDK and no FFI). A
  character cut into flat parts, each placed by a pivot in the rig's own coordinate space
  and moved by parameters through rotation, offset, scale or opacity. An appearance is a
  pose: parts swapped for another picture, parts hidden, parameters held. Breathing, sway
  and blink come from the instance's clock, so there is no simulation state and a restart
  is the clock going back to zero. It is also what proved the seam was an abstraction
  rather than one backend's shape: taking it cost the trait one method, `set_parameter`,
  and changed nothing else
- [x] **The Cubism backend** (`novn_live2d`, optional, and never a `novn` dependency —
  the dependency runs one way). Asset preflight that resolves a `.model3.json` through
  `Assets` and checks it before any native call, a C ABI bridge to the official Framework
  5-r.5 and the proprietary Core, model lifetime and control APIs, a renderer hosted
  inside raylib that restores every GL state it touches, and a refusal to load Live2D's
  sample models in a release build. Verified against Core in a played scene on
  2026-09-24: two models at once, motions and expressions, appearance changes, resize,
  rollback, hot reload, blend modes and inverted masks, shader passes over a model, ten
  minutes of continuous physics, and a rejected `.moc3` falling back to its PNG beside a
  model that kept working
- [x] **State.** Appearance was always story state; a pushed parameter now is too. It is
  recorded with the line it was pushed on, so rolling back puts back the values that line
  was shown with — releasing whatever was pushed after it — and a save carries them onto
  a live model when it is loaded. Transient state is restarted rather than restored: idle
  animation, motions and physics begin again from the appearance's preset, with the push
  applied over the top. `ctx.visual_parameter` pushes and releases one, `on_frame` is the
  per-frame channel behind it, and what visuals do while a menu is up, while skipping, on
  New Game and on a hot reload is written down in the engine's README
- [ ] **What is left**, in the order it matters
  - [x] A custom screen can show one. Screens ask for the appearances they want each
    frame (`ctx.show_visual`), the manager prepares exactly that set once per frame, and
    a screen that asks for none leaves the instances alone — which is what freezes them
    under a menu. The puppet probe's `portrait` command is a screen of its own drawing
    the rig
  - [x] Lip sync from the voice line. `ctx.voice_level()` is the clip's own loudness this
    instant, from an envelope taken at load (10 ms buckets, normalised to the clip's
    loudest moment, so a quiet recording opens the mouth as wide as a loud one), and
    pushing it at a mouth parameter from `on_frame` is the whole of it
  - [x] Frames are checked against references. `novn`'s `tests/gpu.rs` holds every
    parameter still, draws the rig and compares against committed pictures; llvmpipe under
    Xvfb and an AMD card produce the same frame pixel for pixel, so it runs without a GPU.
    `novn_live2d`'s `tests/gpu.rs` does the same for a model against a baseline it writes on
    first run under `target/`, since the sample models cannot be redistributed
  - [ ] Both probes are still watched by a person for anything the two reference frames do
    not cover: motions over time, physics, masks, a scene's worth of loading and dropping
  - [x] A parameter the model does not have is reported once for that character and
    ignored from then on, rather than costing the instance its model for the rest of the
    scene. A typo in a game's code is not a broken model
  - [ ] Platform: Linux x86_64, OpenGL 3.3 and Cubism SDK 5-r.5 only. Other platforms and
    SDK releases are deliberately refused until someone validates them
  - [ ] Art has to be authored in parts for either backend. The example's art is flat
    PNGs and would have to be re-exported; Cubism additionally means an artist in the
    Cubism Editor

Routes not taken, for the record: porting the Framework's motion blending, expressions and
physics to Rust, which is the reason the bridge exists at all; `inox2d`, whose licence and
free tooling are attractive but whose README calls it a prototype, and which would still
need an rlgl renderer written; and `cubism-rs`, which is stale. `live2d-parser` reads model
files in pure Rust if we ever need only to inspect them.

### Interaction

- [x] Image maps / point-and-click scenes (`ImageMap`, `Hotspot`): named hotspots over a background, each with a label, tooltip, hover look and action. Shapes are fractions of the picture, so they follow it when the window doesn't match its aspect
- [x] Drag and drop (`DragBoard`, `Draggable`, `DropTarget`): items dragged onto targets that accept or refuse them, with the pointer or the keyboard. It shares `hit.rs` with image maps: `Shape` (rect, circle, polygon), picking the topmost, `Highlight` and `LabelStyle`

### Extras

Galleries, music rooms and achievements are game features: every game designs its own,
and the engine already has the pieces (custom screens, image maps, buttons, audio,
`ctx.notify`). What the engine owes them is the two things a game can't build properly
on its own — somewhere to keep data that outlives a playthrough, and a record of what the
player has seen.

- [x] Storage that persists across playthroughs, typed like `GameState` but global:
  `VnApp::persistent(T::default())` and `ctx.persistent.get_mut::<T>()`. It lives next
  to `seen.json`, is written atomically, survives New Game and loading, is never part of
  a save and is never rolled back. Today there is nowhere to put such data, which is why
  the example's achievements are wrong: they live in `Journal`, registered with
  `.state(...)`, so they are saved per slot and `Action::NewGame` wipes them with
  `ctx.state.reset()`. `seen.json` becomes the engine's own use of the same mechanism
  rather than a one-off. Done: `.persistent(T)` and `ctx.persistent`, stored in
  `persistent.json`; `seen.json` keeps its format and shares the atomic writer; the
  example's `Achievements` moved onto it
- [x] Records of what has ever been shown, kept by the engine the way it keeps seen
  lines: backgrounds, character images and music tracks, in any playthrough, readable by
  games. The engine sees every `show`, `background` and `music` as it happens; a game
  would have to intercept each one itself. A gallery or a music room is then a custom
  screen asking "has this been seen?". Done: `SeenArt` in the persistent store, written
  as `bg:`/`char:`/`music:` keys, plus `ctx.mark_seen` for art a game shows itself, since
  most galleries unlock at a chosen moment. The engine keeps no categories, order or
  unlock rules: that structure differs in every game and belongs to it
- [x] Gallery, music room and achievements in the example, built from those two, as proof
  the primitives are enough — and as recipes for the documentation site. Moving the
  achievements onto persistent storage fixes them

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
- [ ] Director: while playing, pick `show`, `background`, `music`, `sound` or `voice` from a menu, see it applied live, and write the line into the `.story` file at the current point. `.story` is line-oriented and `novn fmt` normalizes whatever a tool writes, so generated lines can't drift from hand-written style
- [ ] Scene jump: a debug menu listing every scene, to jump straight to one with its variables set, instead of replaying to reach it
- [ ] A screenshot and GIF capture key for bug reports and devlogs, writing next to the existing screenshot key

## M11: Documentation site

A React site rather than generated API docs. The audience splits: someone writing
`.story` files will never open `cargo doc` and should not have to, and
`Instruction::Say { char_id, text }` tells them nothing. The comparator is Ren'Py's
documentation, not a crate on docs.rs. Most of the prose exists already — see the table
below — so the work is structure, navigation and search, not writing.

The model is [ui.shadcn.com](https://ui.shadcn.com): a hand-built Next.js app rather than
a configured docs framework, which is the whole reason it does not look like everything
else. Its beauty is restraint — near-monochrome, one accent, generous whitespace, a
strict sidebar / content / on-this-page layout, almost nothing decorative — and its
signature is the Preview/Code tab, the real component beside its real source. Copy the
method, not the stack.

**Decided 2026-09-24.** The content moves into the site and the READMEs shrink to front
doors, so there is one copy of every explanation. The site is plain Next + MDX, hand
built, no docs framework: the design is owned, and the code block has a compiler behind it
so it was never going to be someone else's component. It is a static export served from
GitHub Pages. The playground comes straight after the pages, because it reuses the code
block the pages need anyway.

Still true and load-bearing, re-checked 2026-09-24: `cargo check -p novn_script --target
wasm32-unknown-unknown` is clean, so the playground needs no port; the tree-sitter grammar
and its queries are in `editors/tree-sitter-story`; and `crates/novn-script/tests/spec.rs`
already compiles every fenced block in SCRIPT.md and golden-tests the result.

What there is to move, as it stands today: 5226 lines of prose, 3033 of them in the engine
guide.

| Source | Lines | Becomes |
| --- | --- | --- |
| `README.md` | 179 | Landing page, project goals, the three-layer architecture |
| `SCRIPT.md` | 746 | The DSL reference: the writer's half of the site, and where the playground earns the most |
| `crates/novn/README.md` | 3033 | The engine guide, as roughly twelve pages. Already a book squeezed into one file |
| `crates/novn-script/README.md` | 540 | Internals, for contributors |
| `crates/novn-cli/README.md` | 287 | Tooling reference (`novn new`, `check`, `fmt`, `translate`, `lsp`) |
| `crates/novn-live2d`, `crates/novn-build`, `crates/novn-macros` | 441 | Short pages under "release and optional backends". The dated verification logs in the Live2D README stay in this file instead — they are a record of what was run, not documentation |

**Phase 0 — the ground.** Nothing here is about documentation; it is what the rest needs
to stand on.

- [x] CI, in `.github/workflows/ci.yml`. A `check` job: `cargo fmt --check`, clippy over
  the workspace and again with `character-visuals`, both test suites, and every
  `#[ignore]`d window test under `xvfb-run` on llvmpipe — seven of them, including the
  reference frames. A separate `video` job for the FFmpeg backend, whose apt list is
  half the install time and whose failures should not look like the engine's. Neither
  can build `novn_live2d`'s native bridge: that needs the proprietary Core, which cannot be
  put in CI, so the crate is only covered in its SDK-independent form
- [x] `docs/` is committed and built by CI: a `docs` job runs `pnpm install
  --frozen-lockfile`, Biome, and the static export, which typechecks on the way through.
  The scaffold's demo page and Vercel artwork are gone; the rest is Next 16, React 19,
  Tailwind 4, Biome and pnpm as it came
- [x] `basePath` is `/novn` in the build, where Pages serves a project site, and empty
  in development, where the root is what anyone types. `next/link` and `next/image` add
  the prefix themselves; CI greps for the hand-written paths that would skip it. Also
  `output: "export"`, `trailingSlash: true`, unoptimized images (the default loader needs
  a server), and `public/.nojekyll`, without which Pages hides `_next`. A domain would
  delete the prefix and the grep together
- [x] Deploying to Pages. The `docs` job uploads `docs/out` on a push to main and a
  `deploy` job publishes it, in its own concurrency group without cancel-in-progress,
  since a half-finished deployment is worse than a queued one. **The repository's Pages
  source has to be set to GitHub Actions by hand, in Settings, or the deploy job fails
  with nowhere to put the artifact.** The `docs` job itself was broken from the start:
  `pnpm/action-setup` is a `uses:` step, so `defaults.run.working-directory` never reached
  it and it looked for a package.json at the repository root

**Phase 1 — the pages.** The site is useful at the end of this, and the README problem is
solved by the same work rather than twice.

- [x] The shell: a sticky header shared with the landing page, a sidebar from a nav
  manifest that marks where you are, the content column, and an on-this-page built at
  runtime from the headings `rehype-slug` gives ids to. MDX through `@next/mdx`, with
  `remark-gfm` for the tables the READMEs are full of — plugins named as strings, since
  Turbopack cannot take a JavaScript function through its Rust side
- [x] The page tree, as `page.mdx` under `src/app/docs/`. SCRIPT.md moved in full, as
  seven pages plus its complete example on the getting-started page, and `README.md`
  became the landing page and "what novn is". The engine guide moved as thirteen pages,
  grouped by what a reader is after rather than by the order its 47 sections happened to
  sit in: building an app, the screens you get, menus and settings, story integration,
  look and feel, the picture, input, audio and video, languages, screens of your own,
  character visuals, saving and rollback, assets and fonts
- [x] A fourth section, the other crates: embedding assets (`novn-build`), the command
  macros (`novn-macros`), and Live2D Cubism (`novn-live2d`)
- [x] A fifth section, the tools: the command line (`check`, `dump`, `fmt`, `new`),
  translating a game, and editors and the language server. Three pages rather than one,
  because writing a story, handing strings to a translator and setting up an editor are
  three different sittings. The editor snippets said `vn` where the binary has been `novn`
  since the rename, so anyone who copied them got a language server that never started
- [x] `novn-script` moved as two pages: inside the compiler (the pipeline, the `Program`,
  values and conditions, error recovery, the schema) and driving the VM (events, the API,
  snapshots, translation at runtime). Split by whether you are changing the compiler or
  writing a frontend on top of it
- [x] Every crate README is a front door now: what the crate is, a quick start, the
  features, how to work on it, and a link to its guide. 5,776 lines of markdown became
  803. `novn` went from 3,035 to 154, `novn-script` from 540 to 93, `novn-cli` from 287
  to 49, and SCRIPT.md from 746 to a cheat sheet. Live2D kept its licensing in full,
  since that has to travel inside the `.crate` rather than live on a website, and sent
  its validation log here instead
- [x] The rename reached the docs. Every `crates/vn_*` path and `vn_engine`-era crate name
  in README.md and TODO.md was stale from the move to `novn`, so every link in the
  workspace table was broken; four anchors into sections that no longer exist now point at
  the guide
- [x] A link checker in CI, `scripts/check_links.py`, run by the `docs` job after the
  export exists. Three checks: relative links and anchors across the repository's
  markdown, the `/docs/...` links inside MDX against the routes the export actually
  produces, and braces in MDX prose that would be read as JSX. All three earned their
  place this session: the shrink turned up four anchors into sections that no longer
  existed, and a bare `{slot}` in a page failed a build

**Phase 2 — the code block.** One component, built knowing what is coming next.

- [x] `.story` highlighting from `editors/tree-sitter-story`, built to WASM with
  `tree-sitter build --wasm` and run through `web-tree-sitter` — in Node, during
  `next build`, so the spans are in the exported HTML and no highlighter reaches the
  browser. The grammar's own `highlights.scm` maps to the token palette, and CI diffs the
  copy against the source so they cannot drift. `web-tree-sitter` has to be in
  `serverExternalPackages`: bundled, Turbopack rewrites its runtime wasm to a URL that
  does not exist on disk at build time
- [x] Rust and shell through the same component and the same token palette, from
  `tree-sitter-rust` and `tree-sitter-bash` on npm rather than a vendored grammar. One
  capture-to-kind map covers all three, so a colour is decided once. `sh`, `bash`, `shell`,
  `console`, `rs` and `rust` are aliases onto the two. A shell block gets a prompt gutter
  that is `select-none` and `aria-hidden`, so copying a command does not take a `$` with it.
  Rust is the most common fence on the site now, 46 blocks to `.story`'s 31
- [x] TOML as a fourth grammar, from `@tree-sitter-grammars/tree-sitter-toml`, which ships
  its own wasm. Four fences today, all manifest stanzas, which on its own would not have
  been worth it; it is in because a TOML configuration layer would make it load-bearing
  (see the idea below). Its query is the one that is adapted rather than copied: upstream
  captures a whole `pair` as `@property`, overlapping the key, the `=` and the value, and
  gives table headers and keys the same capture, so `[features]` and the keys under it came
  out one colour
- [x] Overlapping captures resolve by splitting instead of discarding. A capture inside
  another used to delete the outer one, so `"a \" quote"` lost its string colour entirely
  and kept it only on the escape. It now cuts the outer span in two and keeps its colour on
  both sides, which fixed 71 tokens across the site with nothing changing colour: strings
  around a `{variable}` in a `.story` line, strings around a `$VAR` in a quoted shell path,
  and `#[derive(...)]` attributes. Two captures on the exact same range still keep the last
  one, which is the precedence `rust.scm` is written against; getting that backwards
  recoloured every method name from `label` to `variable`
- [x] A copy button on every code block, the site's only client component. It copies the
  code without the shell prompt, since the prompt is gutter rather than content, and it
  stays on `Copy` if the write is refused rather than claiming a copy that did not happen.
  Verified over CDP against the real hydrated page: 50 blocks across eight pages, every
  label flipping and no prompt reaching the clipboard
- [x] A Code/Run/Listing tab strip on every `.story` block, which is what "preview" turned
  out to mean here. The honest preview for this site is not a picture of a rendered screen,
  which would need the engine in a browser and would drift from it the moment it did not;
  it is what the snippet *compiles to* and what it *does*, which the playground's wasm
  already answers. It was indeed cheaper after phase 3: the tab strip is a thin client
  wrapper around the server-rendered block, and the wasm is fetched on the first click
  rather than on page load, so a reader who never opens a tab pays nothing

**Phase 3 — the playground.** The thing that makes the site worth looking at.

- [x] A thin `crates/novn-playground` wasm crate over `novn_script`, which never learns about
  `wasm-bindgen` and neither does the playground: the whole interface is one string in and one
  string out, small enough to hand-roll, so there is no `wasm-pack`, no `wasm-bindgen-cli`
  version to keep in step, and no generated glue. Three exports and `memory`. 263 KiB on the
  `wasm` profile. `picks` in the request is what removes session state: the story is replayed
  from the start with those choices applied, so every call is a pure function of its request
  and the JS side has no handle to leak. Fuzzed through 29 hostile sources and 5 malformed
  requests with no trap, and memory flat at 45 pages over 4000 calls
- [x] An editable `.story` block showing, live as it is typed: the real diagnostics with
  their "did you mean" suggestions, the compiled listing (`novn dump`) behind a tab, and the
  story played out with its choices clickable, including a gated option shown shut with the
  reason a player would read. `/docs/playground` is the dedicated page and the getting-started
  example is playable in place. Driven in a real browser over CDP rather than trusted from the
  markup: replay, the gate, the listing tab, and typing `remoe` producing the engine's own
  diagnostic at the right line
- [x] Every `.story` sample in the site is runnable in place, 21 across the five language
  pages with not one compile error between them. Three accommodations made it true, all of
  them the playground admitting it is looking at a documentation snippet rather than a
  project: a fragment with no `scene` is wrapped in one and says so; a `jump` to a scene the
  snippet does not define is a warning rather than an error, which is also what the VM itself
  does at runtime; and a lone `...` becomes a narration line, exactly as `spec.rs` has always
  done. The first two are only defensible because the playground has no schema and never
  claimed to be `novn check`
- [x] The editor is highlighted, by the usual trick: the spans in a `pre` behind a textarea
  whose text is transparent and whose caret is not. It is the same tree-sitter grammar and the
  same `story.scm` the static blocks use, with the lex code split into `highlight/lex.ts` so
  both a Node and a browser entry share one implementation, which is the only reason the
  editor cannot come out a different colour from the block beside it. Checked by dumping the
  token stream from both and comparing: five sources, including a broken one and one with
  blank lines, match tag for tag and character for character.
  The alignment is the fiddly half, and it is measured rather than eyeballed: the two boxes
  are pixel-identical, the fourth line sits at 84px in both, and `scrollLeft` follows. The
  cost is about 380 KiB, the `web-tree-sitter` runtime and the grammar, fetched on demand and
  only on a page that has an editor. A failed load leaves a plain textarea with visible text

**Phase 4 — keeping it honest.**

- [x] The `spec.rs` extractor follows the content into MDX, so fenced `.story` blocks stay
  compiled and golden-tested. Brought forward out of this phase by force: shrinking
  SCRIPT.md pulled the content out from under it and the suite went red, finding one
  example where it wanted fifteen. It now reads SCRIPT.md and every `page.mdx`, checks 32
  examples across 13 sources, and names the page, the line and the heading on a failure. A
  companion test asserts the pages are still being reached, so it cannot quietly stop
  finding them the way it just did
- [ ] The same for Rust snippets, which nothing checks today: an extractor that generates
  a compile-only crate from the site's fenced `rust` blocks. An engine whose character is
  catching mistakes before they run should not ship examples that do not compile
- [ ] Search, ours since there is no framework to inherit it from: an index generated at
  build time from the MDX, and a small client over it. Algolia DocSearch is the fallback
  if a build-time index proves too coarse for 5000 lines
- [ ] One-line `///` pointers on public items, linking into the site rather than repeating
  it: `/// Switch to the text input screen. See <docs/engine/text-input>.` Typing `ctx.`
  in an editor shows a list of names and nothing else today, and the LSP can only surface
  what is in the source. Pointers, not prose, so documentation still lives in one place.
  Last, because the links need somewhere to point

Not in this milestone: running the **whole engine** in the browser. That means raylib
through emscripten and `raylib-rs` on wasm, which is its own project. The script-level
playground above is cheap precisely because `novn_script` has no rendering dependencies;
do not let it sell the much larger one.

## Not committed to: a TOML configuration layer

Noted 2026-09-25, out of adding the TOML grammar to the site. Not scheduled, and worth
arguing about before it is.

**The case for it.** A game's *look* is Rust today: window size, fonts, the theme, corner
shapes, panel and button styles, the dialogue box, scenery, default transition lengths. All
of it is a recompile away from being tried, while a story reloads on save. Someone tuning a
dialogue box against real art is doing the slowest possible edit loop for the most visual
part of the engine. A `novn.toml` read at startup and watched by the existing hot reload
would put look on the same footing as prose, and it is the kind of file a person can hand to
someone who does not write Rust.

**What would go in it**, if anything does: window and resolution, fonts by role, the theme
and its colours, shapes and panel styles, the default transition lengths, the scenery, and
the default keybinds. All of it description, none of it behaviour.

**What must not.** Characters, variables, commands, hooks, screens. Those are the registries
the whole validation story rests on: a story may only name what Rust already declared, and
`schema.json` is an export of that rather than a source. Moving any of it into a config file
would mean a story could name something no compiler ever saw, which is the property this
engine exists to have. The line is that a config file may say what things *look* like and
never what *exists*.

**The tension to resolve first.** A config file is untyped until something parses it, and
this project's character is catching mistakes before a window opens. So it only earns its
place with the same treatment everything else gets: a schema, `novn check` reading it,
unknown keys as errors with a "did you mean", and a bad value naming its file and line.
A TOML layer that fails at runtime with a default silently substituted would be a hole in
the design, not a feature. Worth noting `suggest::did_you_mean` is already public and
generic over candidates, so the diagnostics half is mostly there.

**The cheap first slice**, if it happens: the theme alone, since it is the most visual, the
most iterated, and the part with no behaviour attached at all. If that is not pleasant to
use, nothing else in the list will be.

## Ongoing

- [x] Tests: golden tests over every story example in the documentation (`crates/novn-script/tests/spec.rs`: each one compiles cleanly, its listing and VM events match `tests/golden/examples.txt`); `Program::listing()` and `Display for Instruction` shared with `novn dump`
- [x] SCRIPT.md §8.3 typo: `-+` → `-=`
- [x] SCRIPT.md: specify audio (backgrounds, positions and string escaping done)
