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
| `with <transition>` | Every chapter opens with `background … with fade`; other backgrounds, `show`, `remove` and `clear` dissolve (expression changes crossfade, moves glide); Clara leaves `with slide_right`; the third shadow arrives `with slide_left` and everyone goes `with dissolve 1.5` (`reports_the_shadows`) |
| `music <track>`, `music none` | Each place has its track: `archive` in the Archive, `von_lucis` in the 1894 house, `santa_ilde` at the orphanage and the field post, `ending` for all three endings |
| `sound <id>` | The box's latch (`archive_start`), turning pages, a door at Santa Ilde (`notebook_the_lady`), footsteps, closing the box (`ending_silence`) |
| `show … at <position>`, re-show keeping the spot | `show hugo tired at right` … `show mary afraid` keeps Mary at `left` (`01_box_14.story`) |
| `remove`, `clear` | Throughout; `clear` at the end of each remembered scene |
| Dialogue, narration | Throughout |
| `{variable}` in text and as the speaker | `registrar "{player_name}. Good…"`, and `{player_name} "…"` for the player's own lines |
| String escapes `\"` | The sealed letter (`"\"Von Lucis. Three days.\""`), choice options like `"Write \"a miracle\""` |
| `choice:` | Throughout |
| `choice final:` | The register column is written in ink (`box_the_register`) and the final report (`report_start`) |
| `when` on an option, with a reason | "Ask about folio 41" is offered greyed out, with "You never got to the folio the page was torn from" as its tooltip, until the torn page has been read (`ilde_office`) |
| `when` on an option, without a reason | The third answer to the Registrar is not there at all until the archivist has recognised Clara and earned the boy's trust (`report_start`) |
| `preview <id>` on an option | Each of the three final answers shows where it ends (`report_start`) |
| `scene <id> nvl:` | The administrator's notebook opens as a full-screen page (`notebook_start`) |
| `commit` | Signing the visitors' book at Santa Ilde (`ilde_leaving`) |
| `if` / `else`, `&&`, `\|\|`, nested blocks | `if verdict == miracle \|\| verdict == sin:`, `if read_letter == true && saw_torn_page == true:`, `if believed_moriarty == true:` inside a choice option (`ilde_office`) |
| `set` for bool, enum and string values; `add` with `+=` / `-=` | `set read_letter = true`, `set approach = bold`, `add trust -= 1` |
| `call` with typed arguments | `call give_item field_report 19` (word + optional count), `call ask_name player_name`, `call note folio_41`, `call unlock ending_keeper`, `call search study` and `call assemble`, which open a screen and come back to the same line |

### Engine

| Feature | Where |
| --- | --- |
| Registries | `src/cast.rs`: 13 characters with colors and image lists, 10 variables (int, bool, enum, string) |
| Validation, `vn check`, schema export | The story is checked at startup; `assets/schema.json` is refreshed in debug builds for `vn check` |
| Commands with typed arguments | `give_item`, `ask_name`, `note`, `unlock`, `search` and `assemble` in `src/main.rs` |
| Game state by type | `Evidence` (`src/evidence.rs`), `Journal` (`src/journal.rs`) and `Desk` (`src/desk.rs`: what has been examined, and what goes in the report), saved with the game |
| Gallery and music room | `GALLERY` in the main menu (`src/screens/gallery.rs`, with its table in `src/gallery.rs`): People, Places and Music tabs, "3 of 8 found" per tab, locked entries as "???", and tracks that play when clicked. The engine records what the story shows (`char:mary/tired`, `bg:archive_office`, `music:archive`); "The desk, examined" is unlocked by hand with `ctx.mark_seen` when that hotspot is examined in the study, and is drawn as a crop of the study art. Tabs, titles, crops and the locked look are all the game's, not the engine's |
| Persistent storage | Two types in `src/journal.rs`, registered with `.persistent(...)`, shared by every save slot, kept through New Game and loading, and written to `persistent.json` beside the saves. `Achievements` are the rewards `unlock` announces. `CaseLedger` records every ending reached (`call close_case report` in `05_report.story`) and is read three ways: `call remember_cases` copies it into the story variable `cases_closed`, so the Registrar notices a returning player in the prologue; Credits lists every ending found across playthroughs, unfound ones as "???"; and the main menu's Credits stays disabled, with a tooltip, until the first ending (`enabled_if` on `GameView::persistent`) |
| Text input | `ask_name` opens the engine's text input screen, pre-filled with "Archivist" |
| Hooks | `on_scene_enter` announces each chapter as a notification; `on_choice` records every decision in the journal |
| Custom screens and overlays | Evidence (a screen, HUD button or E), Case file (an overlay: variables, notes, recent decisions, what is going in the report), Credits (ending reached, achievements), the study (`src/screens/search.rs`) and the report desk (`src/screens/report_desk.rs`) |
| Image map | `call search study` opens the study as a point-and-click photograph (`src/screens/search.rs`): six hotspots over `von_lucis_study.png`, the desk a polygon and the lamp a circle, each with a label at the pointer and a line for the caption panel; a brass dot marks what has already been examined (`ImageMap::frame`) |
| Drag and drop | `call assemble` opens the report desk (`src/screens/report_desk.rs`): every filed item is a card to drag into "Report 222" or "Back in Box 14". The report tray refuses Box 14 itself (`DropTarget::accepts`), and says so; the cards' places come from `Desk`, so a drop is a change to the game's own state |
| HUD buttons | Evidence and Case file as small plates at the top-right (a `hud_group("top", ..)`), Log, Auto and Menu as a quiet row of icon labels under the dialogue box (`hud_layout` anchored at the bottom) |
| Title card and main menu | One shot for both: the title background slowly pushing in and drifting (`Scenery::motion`, `pan`), a vignette, letterbox bars that slide in on the start screen, "GOD IS WATCHING" in tracked capitals (`TextStyle::spacing`). "PRESS ANY KEY" pulses in the bottom bar; pressing it keeps the shot and fades the menu into the same bar (`buttons_in_bar`, `intro`): text links with an underline on hover and brass diamonds between them (`separator`) |
| Rollback and barriers | Wheel / Page Up-Down; `choice final:` and `commit` in the story; `unlock` is a blocked command, so rollback stops at the moment an achievement is earned (the achievement itself lives outside the save, so rollback couldn't undo it anyway) |
| Saves | Six slots in a 2-column grid with thumbnails and Delete, quick save/load (F5/F9), rollback history stored in saves, saved in the platform data directory (`~/.local/share/god_is_watching` on Linux) |
| Autosave and Continue | An autosave on every new scene and on quit; the main menu's Continue (`Action::Continue`) picks up the newest save after a restart |
| Audio | `.audio(\|a\| a.menu_music("title").fade_seconds(1.5))`: the title theme on the start screen and menu, story tracks crossfading in play; volume sliders in Settings, with `page_turn` as the sample sound |
| Confirmation dialogs | Exit from the main menu, the pause menu's Main Menu and Quit, overwriting a slot, closing the window mid-game |
| Languages | `.source_language("English")` and `.language("pt-BR", "Português (BR)")`, with a partial demo catalog in `assets/lang/pt-BR.json`: the settings screen's own labels are translated, the story is not, so it shows what a half-finished translation looks like. `assets/lang/ui.json` is the label list the engine writes in debug builds |
| Settings | From the main menu and the pause menu, with sliders in the palette (`style::slider`); the text speed preview types one of Adelaide's lines |
| Buttons | `src/style.rs`: beveled corners and a brass border that brightens on hover, with a 0.12 s transition; main menu links are text only, underlined on hover, and click with `page_turn`; Exit, Delete and Yes use the oxblood variant; choices use a nine-slice beveled frame (`ui/choice.png`, swapped for `choice_hover.png` on hover), left-aligned wrapped text; HUD buttons are borderless labels with icons (`ui/icon_*.png`); Continue is disabled until there is something to continue |
| Shapes and panels | Scooped (inward) corners with a double brass rule on every panel (`style::frame`), beveled corners on buttons, slots and plates, diamond separators on the main menu |
| Dialogue box | A framed box with a name plate on its top edge for the speaker (`name_plate`), narrower than the window (`max_width`) and raised off the bottom (`bottom`) to leave room for the HUD row |
| Keyboard and gamepad | Every screen, including the example's Evidence, Credits and Case file (B closes them), the study's hotspots and the report desk, where Enter lifts a card, the arrows move it between the trays that will take it and B puts it back; `style.rs` gives every button a `focused` look matching its hover look |
| Playing controls | Ren'Py's keys (H, Ctrl, Tab, A, L, S, F, middle and right click); the HUD has Log and Auto with icons; the log is styled like the rest (`Decided:` before choices); the voice row is hidden (`voice_row(false)`), since the story has no voice clips |
| Controls overlay (F1) | Styled like the rest, with an extra "In the Archive" section for the Evidence, Case file, study and report desk keys (`KeySection`) |
| Tooltips | On the HUD buttons (`HudButton::tooltip`), the menu's Continue (`MenuItem::tooltip`), the settings rows and the Delete button, styled with `.tooltips(...)` |
| Typewriter text | Every line, at the player's text speed |
| Layouts | Main menu as a centered row in the letterbox bar, each link sized to its label (`MenuItem::style`); pause menu and save slots as 2-column grids, the slots centered in their panel |
| Backgrounds and positions | See the story language table; sprites are scaled to the window height |
| Hot reload | Edit any `.story` file while playing (debug builds); a broken edit shows the error panel (F2 hides it) |
| `after_end` | The end screen leads to the credits instead of the main menu |

## Visual style

The UI takes its materials from the art: candlelit ink browns, brass, parchment, and
oxblood kept for the few actions that can't be undone. `src/style.rs` is the whole
system, and `main.rs` and the custom screens only use it:

| Piece | Look | Used for |
| --- | --- | --- |
| Palette | `INK`, `PANEL`, `RAISED`, `BRASS`, `BRASS_DIM`, `PARCHMENT`, `TEXT`, `MUTED`, `OXBLOOD`, `BACKDROP` | Everything |
| `frame` | Scooped corners, a brass border and a faint inner rule, a soft shadow | The dialogue box, pause menu, confirm dialog, log, controls, settings, save/load, text input, case file, evidence, credits |
| `scenery`, `title_card`, `menu_link` | Motion, vignette and letterbox; the tracked title; underlined text links | The start screen and the main menu |
| `hud_chip` | A translucent beveled plate | The Evidence and Case file buttons |
| `inset` | Beveled, darker than the frame, a dim rule | Save slots, the evidence list, the settings sample, the name field |
| `plate` | Small beveled panel | Speaker names, tooltips, notifications, the skip/auto indicator |
| `button`, `menu_button`, `danger_button`, `choice_button`, `hud_button` | Beveled; hover and focus only change the fill and the rule, never the position or size | Every button |
| `heading`, `body`, `label`, `section` | Noto Serif for titles and dialogue, Noto Sans for menus; section labels in brass | Every text |

The game runs at `render_scale(1.5)`: the frame is drawn at 1920×1080 and scaled down to
the window, so the scooped corners, the brass rules and the text come out smoother. The
layout is still 1280×720, so nothing moves. Drop it to 1.0 on a slow machine.

## Assets

The asset root is `CARGO_MANIFEST_DIR/assets`, resolved at compile time so
`cargo run -p god_is_watching` works from any directory. Release builds carry the assets
inside the executable: `build.rs` embeds the folder with `vn_build` (leaving out
`AUDIO_CREDITS.md` and `schema.json`), and `main.rs` passes
`vn_engine::embedded_assets!()`. So `cargo build --release -p god_is_watching` gives a
single file (about 40 MB) that runs on any machine, even one without the source:

```sh
cargo build --release -p god_is_watching
./target/release/god_is_watching
# or for Windows, from Linux:
cargo build --release -p god_is_watching --target x86_64-pc-windows-gnu
```

| Path | Contents |
|---|---|
| `story/` | The six chapter files, loaded together (the engine's default `story_dir`) |
| `backgrounds/<id>.png` | 11 backgrounds, 1280×720, generated with ChatGPT |
| `characters/<character>/<image>.png` | 23 cartoon portraits, 1024×1536 with transparency, generated with ChatGPT |
| `fonts/` | Noto Serif (OFL, `fonts/OFL.txt`) |
| `ui/` | The choice frames (96×96, nine-slice with 16 px borders) and the 64×64 white HUD icons, drawn by `tools/generate_art.py` |
| `previews/<id>.png` | What the three final answers lead to, 480×270, cut down from the backgrounds by `tools/generate_art.py` |
| `music/<track>.ogg` | 5 tracks, CC0 (sources in `AUDIO_CREDITS.md`) |
| `sounds/<id>.ogg` | 7 sounds from Kenney's RPG Audio, CC0 (`AUDIO_CREDITS.md`) |
| `schema.json` | Exported registries (refreshed by debug runs, or `cargo run -p god_is_watching -- --export-schema`), used by `vn check` |

### Art

The backgrounds and portraits were generated with ChatGPT's image model. Replace any file
with art of the same name and aspect ratio and the game picks it up.

The character art uses bold ink outlines, expressive cartoon faces, angular shapes,
flat shadows, and gritty painted texture. Mary's approved cartoon sample is the style
reference for the cast. Expressions preserve each character's costume and framing;
Gabriel has extra transparent space above his head to keep him shorter than the adults.
The sprites retain their generated alpha and 2:3 aspect ratio at 1024×1536.

The September 2026 restyle's original portraits are backed up under
`output/imagegen/cartoon-cast/originals/` at the workspace root. The same folder's
`gallery.html` compares the installed portraits with the originals, and
`validation.json` records the image dimensions, transparency checks, and hashes.

`tools/generate_art.py` (Python 3 with Pillow and NumPy) draws the UI images in the
palette of `style.rs`:

```sh
python3 examples/god_is_watching/tools/generate_art.py                   # the UI frames, icons and previews
python3 examples/god_is_watching/tools/generate_art.py --only choice     # some of them
python3 examples/god_is_watching/tools/generate_art.py --only previews   # the ending previews, from the backgrounds
```

It also still has the painted placeholder backgrounds and silhouette portraits the game
used before; `--placeholders` writes them over the current art (for example to test the
engine without the real images).

See [vn_engine's README](../../crates/vn_engine/README.md) for every option.
