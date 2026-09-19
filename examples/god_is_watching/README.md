# god_is_watching

The reference game: a short, complete visual novel that uses every feature of the story
language and the engine, so it doubles as their end-to-end test.

```sh
cargo run -p god_is_watching
```

## The story

October 1903. You are a new archivist of the House, named by you at the start. The
Registrar hands you two things that were never meant to be read together: Box 14, what
was recovered from the Von Lucis residence in 1894, and nineteen reports from the
House's field post at Santa Ilde, the last of which breaks off mid-word.

| Chapter | File | What happens |
| --- | --- | --- |
| Prologue | `00_archive.story` | The Archive. You sign the ledger and choose how to start |
| I. Box 14 | `01_box_14.story` | Mary Von Lucis's notebook, her maid's testimony, a sealed letter, and a register column you have to fill in, in ink |
| II. The Notebook | `02_notebook.story` | The Santa Ilde administrator's diary from 1894, a torn page, and a conversation from 1901 |
| III. Field Reports | `03_reports.story` | The House's reports, 203 to 221, up to the word that was never finished |
| IV. Santa Ilde | `04_santa_ilde.story` | You go there. An old administrator, a quiet sister, a boy with a wooden horse |
| V. The Report | `05_report.story` | You write report 222. Three endings |

The documents are history: every choice is the archivist's (open the letter or not,
what to write in the register, whether to hold the torn page to the lamp, how to speak
to the people at Santa Ilde, what to report). The third ending is only offered to a
player who found the clues and earned the administrator's trust; the tests in
`crates/vn_script/tests/vm.rs` play the story to all three endings.

## Feature tour

Where each feature is used, so the example can be read as a reference.

### Story language

| Feature | Where |
| --- | --- |
| Scenes, `jump` across files | Every chapter ends with a `jump` into the next file |
| `background <id>`, `background none` | Every scene; `background none` for the moment report 221 breaks off (`03_reports.story`) |
| `music <track>`, `music none` | Each place has its track: `archive` in the Archive, `von_lucis` in the 1894 house, `santa_ilde` at the orphanage and the field post, `ending` for all three endings |
| `sound <id>` | The box's latch (`archive_start`), turning pages, a door at Santa Ilde (`notebook_the_lady`), footsteps, closing the box (`ending_silence`) |
| `show … at <position>`, re-show keeping the spot | `show hugo tired at right` … `show mary afraid` keeps Mary at `left` (`01_box_14.story`) |
| `remove`, `clear` | Throughout; `clear` at the end of each remembered scene |
| Dialogue, narration | Throughout |
| `{variable}` in text and as the speaker | `registrar "{player_name}. Good…"`, and `{player_name} "…"` for the player's own lines |
| String escapes `\"` | The sealed letter (`"\"Von Lucis. Three days.\""`), choice options like `"Write \"a miracle\""` |
| `choice:` | Throughout |
| `choice final:` | The register column is written in ink (`box_the_register`) and the final report (`report_start`) |
| `commit` | Signing the visitors' book at Santa Ilde (`ilde_leaving`) |
| `if` / `else`, `&&`, `\|\|`, nested blocks | `if verdict == miracle \|\| verdict == sin:`, `if read_letter == true && saw_torn_page == true:`, a `choice` inside an `if` (`ilde_office`) |
| `set` for bool, enum and string values; `add` with `+=` / `-=` | `set read_letter = true`, `set approach = bold`, `add trust -= 1` |
| `call` with typed arguments | `call give_item field_report 19` (word + optional count), `call ask_name player_name`, `call note folio_41`, `call unlock ending_keeper` |

### Engine

| Feature | Where |
| --- | --- |
| Registries | `src/cast.rs`: 13 characters with colors and image lists, 10 variables (int, bool, enum, string) |
| Validation, `vn check`, schema export | The story is checked at startup; `assets/schema.json` is refreshed in debug builds for `vn check` |
| Commands with typed arguments | `give_item`, `ask_name`, `note`, `unlock` in `src/main.rs` |
| Game state by type | `Evidence` (`src/evidence.rs`) and `Journal` (`src/journal.rs`), saved with the game |
| Text input | `ask_name` opens the engine's text input screen, pre-filled with "Archivist" |
| Hooks | `on_scene_enter` announces each chapter as a notification; `on_choice` records every decision in the journal |
| Custom screens and overlays | Evidence (a screen, HUD button or E), Case file (an overlay: variables, notes, recent decisions), Credits (ending reached, achievements) |
| HUD buttons | Evidence, Case file, Save, Menu |
| Rollback and barriers | Wheel / Page Up-Down; `choice final:` and `commit` in the story; `unlock` is a blocked command, so an achievement can't be rolled back |
| Saves | Six slots in a 2-column grid with thumbnails and Delete, quick save/load (F5/F9), rollback history stored in saves, saved in the platform data directory (`~/.local/share/god_is_watching` on Linux) |
| Autosave and Continue | An autosave on every new scene and on quit; the main menu's Continue (`Action::Continue`) picks up the newest save after a restart |
| Audio | `.audio(\|a\| a.menu_music("title").fade_seconds(1.5))`: the title theme on the start screen and menu, story tracks crossfading in play; volume sliders in Settings, with `page_turn` as the sample sound |
| Confirmation dialogs | Exit from the main menu, the pause menu's Main Menu and Quit, overwriting a slot, closing the window mid-game |
| Settings | From the main menu and the pause menu, with sliders in the palette (`style::slider`); the text speed preview types one of Adelaide's lines |
| Buttons | `src/style.rs`: every button has a border that brightens on hover and a press scale, with a 0.12 s transition; menu buttons slide right on hover and click with `page_turn`; Exit skews on hover; choices use a nine-slice paper image (`ui/choice.png`, swapped for `choice_hover.png` on hover), left-aligned wrapped text; HUD buttons have icons (`ui/icon_*.png`) and a shadow; Continue is disabled until there is something to continue |
| Tooltips | On the HUD buttons (`HudButton::tooltip`), the menu's Continue (`MenuItem::tooltip`), the settings rows and the Delete button, styled with `.tooltips(...)` |
| Typewriter text | Every line, at the player's text speed |
| Layouts | Main menu `rows_of([1, 2, 2, 1])` with `Stretch`; pause menu and save slots as 2-column grids |
| Backgrounds and positions | See the story language table; sprites are scaled to the window height |
| Hot reload | Edit any `.story` file while playing (debug builds); a broken edit shows the error panel (F2 hides it) |
| `after_end` | The end screen leads to the credits instead of the main menu |

## Visual style

`src/style.rs` holds the palette (ink, panel, parchment, text, muted, oxblood) and the
text and button styles built from it; `main.rs` applies them to every default screen
(start, main menu, playing, pause menu, confirm dialog, save/load, settings, text input,
notifications), and the custom screens use the same functions. Titles, dialogue and
speaker names use Noto Serif; menus and buttons keep the engine's Noto Sans.

## Assets

The asset root is `CARGO_MANIFEST_DIR/assets`, resolved at compile time so
`cargo run -p god_is_watching` works from any directory. A shipped build would look for
assets next to the executable instead.

| Path | Contents |
|---|---|
| `story/` | The six chapter files, loaded together (the engine's default `story_dir`) |
| `backgrounds/<id>.png` | 11 backgrounds, 1280×720 |
| `characters/<character>/<image>.png` | 23 portraits, 600×900 with transparency |
| `fonts/` | Noto Serif (OFL, `fonts/OFL.txt`) |
| `ui/` | The choice panels (96×96, nine-slice with 16 px borders) and four 64×64 white HUD icons, generated |
| `music/<track>.ogg` | 5 tracks, CC0 (sources in `AUDIO_CREDITS.md`) |
| `sounds/<id>.ogg` | 7 sounds from Kenney's RPG Audio, CC0 (`AUDIO_CREDITS.md`) |
| `schema.json` | Exported registries (refreshed by debug runs, or `cargo run -p god_is_watching -- --export-schema`), used by `vn check` |

### Generated art

The art is generated, not drawn: `tools/generate_art.py` (Python 3 with Pillow and
NumPy) paints each background from shapes, glows, a color grade and a vignette, and each
portrait as a stylized silhouette in the character's color, with a rim light, a face
light and a few identifying details (Mary's hair, Adelaide's cap, Clara's veil,
Moriarty's glasses, hats, hoods, a child's proportions, and a doubled, blurred outline
for the people the story forgets). Expressions change the pose, lighting and color.

```sh
python3 examples/god_is_watching/tools/generate_art.py                 # everything
python3 examples/god_is_watching/tools/generate_art.py --only mary title  # some of it
python3 examples/god_is_watching/tools/generate_art.py --only ui          # the UI panels and icons
```

The output is deterministic (fixed random seed). It is placeholder-grade art with a
consistent style: replace any file with real art of the same name and size ratio, and
the game picks it up.

See [vn_engine's README](../../crates/vn_engine/README.md) for every option.
