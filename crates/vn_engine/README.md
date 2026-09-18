# vn_engine

A raylib-based visual novel engine built on [`vn_script`](../vn_script/README.md).

It re-exports `raylib` and `vn_script` (as `vn_engine::script`), so games build against
the same versions the engine uses and only need `vn_engine` as a dependency.

## Quick start

A game is a `VnApp` built from defaults, with only the parts it wants changed:

```rust
use vn_engine::raylib::prelude::*;
use vn_engine::{FontRole, Action, MenuItem, ScreenState, TextStyle, VnApp};

fn main() -> std::io::Result<()> {
    VnApp::new("My Novel")
        .size(1280, 720)
        .assets(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
        .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
        .main_menu(|m| {
            m.button_style(|b| b.size(260.0, 52.0).roundness(0.3))
                .button("New Game", Action::NewGame)
                .button("Continue", Action::Goto(ScreenState::Playing))
                .item(MenuItem::new("Quit", Action::Quit).style(|b| b.color(Color::MAROON)))
        })
        .playing(|p| p.dialogue_box(|b| b.height(180.0).color(Color::new(10, 10, 20, 220))))
        .run()
}
```

`run()` loads and validates the story, opens the window, loads the fonts and the
[settings](#settings), and runs the loop until the player closes the window (see
[Closing the window](#closing-the-window)) or a screen returns `ScreenState::Quit`. It returns an
`AppError` without opening a window if the story can't be read or has errors.

## VnApp

| Method | Default | Purpose |
|---|---|---|
| `new(title)` | | Window title; also the main menu title unless the menu sets one |
| `size(w, h)` | 1280×720 | Window size |
| `target_fps(fps)` | 60 | |
| `clear_color(color)` | black | Cleared every frame, before the screen draws |
| `assets(path)` | `assets` | Asset root; every other path is relative to it |
| `story_dir(path)` | `story` | Directory of `.story` files; every one under it (recursively) is loaded into one story, in path order |
| `initial_screen(state)` | `StartScreen` | First screen, e.g. `MainMenu` to skip the start screen |
| `font(role, file)` | | Font from `<assets>/fonts/` for a `FontRole` (see [Fonts](#fonts)) |
| `start_screen(\|s\| ...)` | | Configure the default start screen |
| `main_menu(\|m\| ...)` | | Configure the default main menu |
| `playing(\|p\| ...)` | | Configure the default playing screen |
| `screen(state, build)` | | Use your own `Screen` for `state` (see [Custom screens](#custom-screens)) |
| `overlay(name, build)` | | Register an overlay, or replace a built-in one (see [Overlays](#overlays)) |
| `pause_menu(\|p\| ...)` | | Configure the pause menu (see [Pause menu](#pause-menu)) |
| `confirm_dialog(\|c\| ...)` | | Configure confirmation dialogs (see [Confirmation dialogs](#confirmation-dialogs)) |
| `rollback(\|r\| ...)` | on | Configure rollback (see [Rollback](#rollback)) |
| `settings(\|s\| ...)` | | Configure the settings screen (see [Settings](#settings)) |
| `confirm_on_close(Option<&str>)` | "Quit the game? Unsaved progress will be lost." | Message shown when the window's close button is clicked during a game; `None` quits right away (see [Closing the window](#closing-the-window)) |
| `toast(\|t\| ...)` | | Configure notifications (see [Notifications](#notifications)) |
| `exit_key(Option<key>)` | `None` | A key that closes the window. Off by default, so Esc can open the pause menu |
| `state(value)` | | Register game state (see [Game state](#game-state)) |
| `command(name, handler)` | | Handle `call <name> ...` from stories (see [Commands](#commands)) |
| `variable(name, VariableDef)` | | Register a story variable (see [Registries and validation](#registries-and-validation)) |
| `character(id, Character)` | | Register a character |
| `entry_scene(id)` | first scene of the first file | Scene the story starts from |
| `warn_missing_art(bool)` | `true` | Warn at startup about `show`s with no image file |
| `schema_file(Option<&str>)` | `Some("schema.json")` | Where the schema is exported, relative to the assets (see [Schema export](#schema-export)); `None` turns the export off |
| `text_input(\|t\| ...)` | | Configure the default text input screen |
| `saves_dir(path)` | `saves` | Where save files go (see [Save and load](#save-and-load)) |
| `save_menu(\|s\| ...)` | | Configure the default Save/Load screens |

Each configure closure receives the current config and returns the changed one, so calls
chain and unset options keep their defaults.

## Default screens

The engine ships the behavior of each screen; a game only changes text, sizes, colors,
fonts, positions and actions.

Layout is relative to the window: `*_y` values are fractions of the window height
(`0.5` is the middle), sizes are in pixels, and everything is centered horizontally.

### Start screen (`StartScreenConfig`)

Any key or click goes to `next`.

| Option | Default |
|---|---|
| `prompt(text)`, `prompt_text(style)`, `prompt_y(f)` | "Press any key to start", Menu font 30 px, 0.5 |
| `footer(text)`, `footer_text(style)` | none (bottom-right, e.g. a version), Menu font 14 px gray |
| `background(bg)` | none |
| `next(state)` | `MainMenu` |

### Main menu (`MainMenuConfig`)

| Option | Default |
|---|---|
| `title(text)`, `title_text(style)`, `title_y(f)` | app title, Title font 64 px, 0.25 |
| `button(label, action)`, `item(MenuItem)` | New Game, Load, Settings, Quit |
| `button_style(\|b\| ...)` | `ButtonStyle::default()` (240×52) |
| `buttons_y(f)`, `spacing(px)` | 0.45, 18 |
| `bottom_margin(px)` | 40: a menu that would reach closer to the bottom moves up (not into the title), then tightens its spacing |
| `background(bg)` | none |

The first `button`/`item` call replaces the default list; later calls append.
`MenuItem::new(label, action).style(|b| ...)` changes one button's style on top of
`button_style`.

`Action` (shared by the main menu, the pause menu and HUD buttons):

| Action | Behavior |
|---|---|
| `NewGame` | Reset the story and game state, then `Playing` |
| `Goto(state)` | Switch to `state` (closes any overlays). `Goto(Playing)` continues the current story |
| `Action::overlay(name)` | Open an overlay on top of the current screen (e.g. `PAUSE_OVERLAY`, `SAVE_OVERLAY`, `LOAD_OVERLAY`) |
| `Resume` | Close every open overlay |
| `QuickSave` | Save to the `quick` slot and show a notification |
| `QuickLoad` | Load the `quick` slot and continue playing, or show the error |
| `Action::confirm(message, action)` | Ask first; run `action` only if the player confirms (see [Confirmation dialogs](#confirmation-dialogs)) |
| `Quit` | Close the app |
| `Action::custom(\|ctx\| ...)` | Run your code with the `GameContext`; return the next state or `None` to stay |

### Playing screen (`PlayingConfig`)

Plays the story from the VM's current position. Coming back to it (e.g. from an
inventory screen) shows the same line or choice again, using `StoryVm::current()`.

- `Say`: the dialogue box, with the speaker name above word-wrapped text. A registered
  character's display name and color are used for its id. New lines type out at the
  player's [text speed](#settings); clicking (or an advance key) while a line is typing
  shows all of it, the next click continues. Lines shown again (coming back to the
  screen, rollback, loading) appear whole. Words are wrapped for the full line up front,
  so they don't jump between lines while typing.
- `Choice`: one button per option, centered vertically.
- `End`: an end title and hint; continuing goes to `after_end`.
- Characters on screen are spread evenly across the width (sorted by id), bottom-aligned.
- Click or an advance key continues. **Esc** (`pause_key`) opens the [pause menu](#pause-menu).
  `menu_key` (off by default) jumps straight to the main menu.
- `call` runs the matching [command](#commands) handler; unknown commands are logged.
- F5 / F9 run `Action::QuickSave` / `Action::QuickLoad`; the result shows as a
  [notification](#notifications).
- HUD buttons sit in a row at the top-right, hidden on the end screen. Clicking one runs
  its `Action` and doesn't advance the dialogue.

| Option | Default |
|---|---|
| `dialogue_box(\|b\| ...)` | `DialogueBoxStyle`: height 170, margin 40, padding 24, black at 200 alpha, square corners |
| `speaker_text(style)` | Speaker font 24 px gold |
| `dialogue_text(style)` | Dialogue font 26 px white |
| `choice_button(\|b\| ...)`, `choice_spacing(px)` | 720×56, Choice font; 16 |
| `end_title(text)`, `end_title_text(style)` | "The End", Title font 56 px |
| `end_hint(text)`, `end_hint_text(style)` | "Click to return to the menu", Menu font 20 px gray |
| `advance_keys(keys)` | Space, Enter |
| `pause_key(Option<key>)`, `pause_overlay(name)` | `Some(Escape)`, `PAUSE_OVERLAY` |
| `menu_key(Option<key>)` | `None` |
| `after_end(state)` | `MainMenu` |
| `background(bg)` | none |
| `hud_button(label, action)` | none |
| `quick_save_key(Option<key>)`, `quick_load_key(Option<key>)` | `Some(F5)`, `Some(F9)` (the `quick` slot) |
| `hud_button_style(\|b\| ...)`, `hud_margin(px)`, `hud_spacing(px)` | 130×40, dark translucent, 18 px text; 16; 10 |

raylib's "Esc closes the window" is turned off (see `VnApp::exit_key`).

## Styles

| Type | Fields / builder methods |
|---|---|
| `TextStyle::new(font, size, color)` | `.font()`, `.size()`, `.color()` |
| `ButtonStyle::default()` | `.size(w, h)`, `.color(c)` (hover becomes a lighter shade), `.hover_color(c)`, `.roundness(0..1)`, `.text(style)`, `.font(role)`, `.font_size(px)`, `.text_color(c)` |
| `DialogueBoxStyle::default()` | `.height()`, `.margin()`, `.padding()`, `.color()`, `.roundness()` |
| `Background` | `Color(color)` fills the screen; `Image(path)` stretches an asset over it |

## Custom screens

Replace any screen, or add new ones under `ScreenState::Custom(name)`:

```rust
VnApp::new("My Novel")
    .main_menu(|m| m.button("Credits", Action::Goto(ScreenState::Custom("credits".into()))))
    .screen(ScreenState::Custom("credits".into()), CreditsScreen::new)
```

`screen(state, build)` takes precedence over the default for that state; `build` runs each
time the screen is entered. A custom screen implements `Screen`:

| Type | Role |
|---|---|
| `ScreenState` | `StartScreen`, `MainMenu`, `Playing`, `Save`, `Load`, `TextInput`, `Settings`, `Custom(String)`, `Quit` |
| `Screen` | `update(ctx) -> Option<ScreenState>` returns the next state; `draw(d, &DrawContext)` |
| `GameContext` | What `update` gets: `rl`, `thread`, `resources`, `story`, `state`, `saves`, `previous` (the screen before this one); plus `open_overlay(name)`, `run_command(name, args)`, `ask_text(request)`, `save(slot)` and `load(slot)` |
| `DrawContext` | What `draw` gets: `resources`, `story`, `state`, `saves`, `characters`, and `fonts()` |

There is no default `Settings` screen yet. Switching to a state with no screen logs a
warning and stays on the current one.

The `ui` module has the pieces the default screens use, for custom screens to reuse:

| Function | Purpose |
|---|---|
| `draw_button(d, fonts, rect, label, &ButtonStyle)` | Button with hover color and centered label |
| `is_clicked(rl, rect)`, `is_hovered(rl, rect)` | Mouse hit tests |
| `draw_text`, `draw_text_centered`, `draw_text_wrapped` | Text with a `TextStyle`; wrapped returns the height used |
| `stacked_rects(count, w, h, spacing, center_x, top)` | A centered column of rectangles |
| `screen_size(rl)` | Window size as a `Vector2` |
| `load_background(ctx, bg)`, `draw_background(d, resources, bg)` | `Background` support (load in `update`, draw in `draw`) |

See `examples/god_is_watching/src/screens/credits.rs` and `inventory.rs`.

## Pause menu

Esc on the playing screen opens the pause menu, an overlay over the game:

| Button | Action |
| --- | --- |
| Resume | `Action::Resume` (also Esc) |
| Save | `Action::overlay(SAVE_OVERLAY)`: the save slots as a panel over the game |
| Load | `Action::overlay(LOAD_OVERLAY)`: the load slots as a panel over the game |
| Quick Save | `Action::QuickSave` |
| Quick Load | `Action::QuickLoad` |
| Settings | `Action::overlay(SETTINGS_OVERLAY)`: the [settings](#settings) as a panel over the game |
| Main Menu | `Action::confirm(.., Goto(MainMenu))` ("Unsaved progress will be lost"); `Goto(Playing)` from the main menu resumes |
| Quit | `Action::confirm(.., Quit)` |

Save, Load and Settings open on top of the pause menu; their Back button, Esc or
Backspace return to it. Loading a slot closes everything and continues playing from the save.

`PauseMenuConfig` (`.pause_menu(|p| ...)`):

| Option | Default |
| --- | --- |
| `title(text)`, `title_text(style)` | "Paused", Title font 40 px |
| `button(label, action)`, `item(MenuItem)` | the buttons above; the first call replaces the list |
| `button_style(\|b\| ...)`, `spacing(px)` | 260×44, 20 px text; 10 |
| `panel_width(px)`, `padding(px)`, `panel_color(c)`, `panel_roundness(r)` | 340, 28, dark translucent, 0.04 |
| `backdrop(c)` | black at 150 alpha, over the game |
| `close_keys(keys)` | Esc |

To use your own pause menu, register an overlay under the same name:
`.overlay(PAUSE_OVERLAY, MyPauseMenu::new)`.

## Confirmation dialogs

`Action::confirm(message, action)` opens a dialog over whatever is on screen and runs
`action` only if the player confirms:

```rust
.main_menu(|m| m.button("Quit", Action::confirm("Leave the game?", Action::Quit)))
```

`Confirm::new(message, action).confirm_label("Overwrite").cancel_label("Keep")` sets
per-dialog button labels (`Action::from(confirm)` / `.into()` to use it as an action), and
`ctx.confirm(confirm)` opens one from code.

Where the engine asks by default:

| When | Message | Turn off |
| --- | --- | --- |
| Pause menu → Main Menu | Return to the main menu? Unsaved progress will be lost. | replace the pause menu buttons |
| Pause menu → Quit | Quit the game? Unsaved progress will be lost. | replace the pause menu buttons |
| The window's close button, during a game | Quit the game? Unsaved progress will be lost. | `.confirm_on_close(None)` |
| Saving over a used slot | Overwrite Slot N? | `.save_menu(\|s\| s.confirm_overwrite(false))` |
| Loading from the in-game Load panel | Load Slot N? Unsaved progress will be lost. | `.save_menu(\|s\| s.confirm_load_in_game(false))` |

Loading from the main menu's Load screen and quick save/load don't ask.

The dialog is an overlay (`CONFIRM_OVERLAY`) on top of the current stack: Cancel, Esc or N
close it; the confirm button, Enter or Y run the action. `ConfirmConfig`
(`.confirm_dialog(|c| ...)`): `message_text`, `confirm_label` ("Yes"), `cancel_label`
("Cancel"), `confirm_button` (red), `cancel_button`, `panel_width` (460), `padding`,
`panel_color`, `panel_roundness`, `backdrop`, `confirm_keys`, `cancel_keys`.

### Closing the window

raylib reports a click on the window's close button (or the exit key, if one is set)
through `window_should_close()` for a single frame, so the loop hands it to
`ScreenStateManager::request_close()` instead of stopping:

- On the start screen or main menu, or once the story has ended, the game quits.
- On the playing screen or text input, and on any other screen once a story has started
  (Settings, Load, a custom inventory screen, ...), the confirmation dialog opens with
  `Action::Quit` behind its "Quit" button. `close_needs_confirmation(state, story)` is
  the rule.
- Clicking the close button again while that dialog is open quits, so the window can
  always be closed.

## Settings

Player preferences, separate from saves: `Settings { fullscreen, text_speed }`, stored as
`settings.json` in the saves directory and written as soon as something changes (via a
temporary file, like saves). A missing or unreadable file gives the defaults (windowed,
40 characters per second), with a warning for an unreadable one; missing fields take
their default, so new settings don't break old files.

- **Display:** windowed or fullscreen (borderless, at the monitor's resolution). Applied
  at startup and whenever it changes.
- **Text speed:** characters per second for the typewriter on the playing screen; `0` is
  instant.

The default settings screen shows one row per setting, whose button cycles through the
values, and a sample line that types out at the chosen speed. It exists as a screen
(`ScreenState::Settings`, in the default main menu) and as an overlay
(`SETTINGS_OVERLAY`, in the pause menu). Back, Esc or Backspace return to where it was
opened from.

`SettingsConfig` (`.settings(|s| ...)`):

| Option | Default |
| --- | --- |
| `title(text)`, `title_text(style)` | "Settings", Title font 44 px |
| `label_text(style)` | Menu font 24 px |
| `value_button(\|b\| ...)` | 220×46, 20 px text |
| `row_width(px)`, `row_spacing(px)` | 560, 16 |
| `text_speeds([(label, chars_per_second)])` | Slow 20, Normal 40, Fast 80, Instant 0 (a value not in the list shows as "N chars/s") |
| `sample_text(text)`, `sample_text_style(style)` | "This is how fast the story's text appears.", Dialogue font 22 px |
| `back_button(\|b\| ...)`, `back_label(text)`, `back_keys(keys)` | 200×48, "Back", Esc and Backspace |
| `backdrop(c)` | black at 200 alpha, behind the overlay |
| `background(bg)` | none, behind the screen |

Labels (`display_label`, `windowed_label`, `fullscreen_label`, `text_speed_label`) are
public fields. From code, `ctx.settings.values` reads the settings and
`ctx.settings.update(|s| ...)` changes and saves them; `DrawContext::settings` is the
read-only view. `Typewriter::start(text, chars_per_second, now)` / `visible(now)` /
`finish()` is the timing the playing screen and the preview use.

## Rollback

On the playing screen, **mouse wheel up / Page Up** steps back one line and **wheel down /
Page Down** steps forward again. Stepping back restores the story (line, variables,
characters) and every [game state](#game-state) value, so an item given by a command is
taken back too. Continuing (click/Space) after rolling back plays on from there and
forgets the lines that were ahead.

What can't be undone is decided at three levels:

| Level | How | Effect |
| --- | --- | --- |
| Game default | `.rollback(\|r\| r.through_choices(false))` | Every choice is permanent once made (default: choices can be undone) |
| Story | `choice final:` | This choice is permanent, whatever option is picked |
| Story | `commit` | A point of no return anywhere; inside one option it makes only that option permanent (SCRIPT.md 10) |
| Commands | `.rollback(\|r\| r.block_command("unlock_achievement"))` | Running this command is a point of no return, for effects that can't be taken back |

Rollback stops at the first line after a barrier. It also resets on New Game and on every
load, and isn't stored in save files.

`RollbackConfig` (`.rollback(|r| ...)`):

| Option | Default |
| --- | --- |
| `enabled(bool)` | `true` |
| `max_steps(n)` | 100 lines of history |
| `through_choices(bool)` | `true` |
| `block_command(name)` | none |
| `mouse_wheel(bool)` | `true` |
| `back_keys(keys)`, `forward_keys(keys)` | Page Up, Page Down |

Each step stores a story snapshot and the game state as JSON (the same data as a save), so
large state types make history more expensive; lower `max_steps` if needed. From code:
`ctx.rollback.back(story, state)`, `forward`, `can_go_back()`, `steps_back()`,
`mark_barrier()`, `clear()`.

## Notifications

`ctx.notify(text)` / `ctx.notify_error(text)` show a short message at the top-left, above
every screen and overlay (quick save/load use them). `ToastConfig` (`.toast(|t| ...)`):
`text(style)`, `error_text(style)`, `background(color)`, `seconds(s)` (2.5),
`margin(px)` (16).

## Overlays

An overlay draws on top of the current screen (which keeps drawing underneath) and takes
all input until it closes; the screen below doesn't update meanwhile.

```rust
VnApp::new("My Novel")
    .playing(|p| p.hud_button("Stats", Action::overlay("stats")))
    .overlay("stats", StatsOverlay::new)
```

```rust
pub trait Overlay {
    fn update(&mut self, ctx: GameContext) -> OverlayAction;
    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext);
}
```

Overlays form a **stack**: opening one while another is open puts it on top (pause →
save → confirm). Only the top one updates; all of them draw, bottom to top, so each one's
backdrop dims the ones below. Opening the overlay that's already on top
does nothing. Changing screens closes them all.

| `OverlayAction` | Effect |
| --- | --- |
| `Stay` | Keep it open |
| `Close` | Close this one, back to the one below (or the screen) |
| `CloseAll` | Close every overlay |
| `Goto(state)` | Switch screens (closes every overlay) |

From any screen, overlay, action or command: `ctx.open_overlay(name)`, `ctx.close_overlay()`,
`ctx.close_overlays()`. The built-in overlays are `PAUSE_OVERLAY` (`"pause"`),
`SAVE_OVERLAY` (`"save"`), `LOAD_OVERLAY` (`"load"`) and `CONFIRM_OVERLAY` (`"confirm"`);
`.overlay(name, ..)` with one of
those names replaces it. An unknown name logs a warning. See
`examples/god_is_watching/src/screens/stats.rs`.

## Game state

Rust-owned state (inventory, relationships, ...) is registered by type and reached from
any screen, overlay or command:

```rust
#[derive(Clone, Default, Serialize, Deserialize)]
struct Inventory { /* ... */ }

VnApp::new("My Novel").state(Inventory::default())

ctx.state.get_mut::<Inventory>()   // in update / commands
ctx.state.get::<Inventory>()       // in draw
```

State types must also derive `Serialize` and `Deserialize`: they're saved with the game
(see [Save and load](#save-and-load)).

`get`/`get_mut` panic with the type name if it wasn't registered; `try_get`/`try_get_mut`
return `Option`. `Action::NewGame` resets every registered value to the one passed to
`state()` (hence `Clone`).

Story variables (`set`, `add`, conditions) live in the VM instead: `ctx.story.variables()`.

## Commands

`call <name> <args...>` in a story runs the handler registered under that name. The
handler's second parameter declares the arguments as a tuple; they're parsed and checked
for you:

```story
call give_item verlaine_letter
call give_item candle 3
```

```rust
fn give_item(ctx: &mut GameContext, (item, count): (String, Option<u32>)) -> Option<ScreenState> {
    ctx.state.get_mut::<Inventory>().add(&item, count.unwrap_or(1));
    None
}

VnApp::new("My Novel").command("give_item", give_item)
```

| Rust type | Accepts |
| --- | --- |
| `String` | any word |
| `i8`…`i64` | an integer |
| `u8`…`u64`, `usize` | a non-negative integer |
| `f32`, `f64` | a number |
| `bool` | `true` / `false` |
| `Option<T>` | an optional trailing argument |
| `()` | no arguments |
| `Vec<String>` | any number of words (unchecked) |

Tuples take up to 5 arguments; required ones must come before `Option`s (checked when
the command is registered). The signature is part of the schema, so every `call` in the
story is checked at startup (argument count and kinds), and again when it runs.

Returning `Some(state)` switches screens (e.g. to a text input); the story resumes from
the next instruction when the playing screen is entered again.

## Registries and validation

Variables and characters are registered on the app; together with the command
signatures they form a `Schema` the story is checked against at startup:

```rust
VnApp::new("My Novel")
    .variable("affection", VariableDef::int(0))
    .variable("route", VariableDef::enumeration(["good", "bad", "neutral"], "neutral"))
    .variable("player_name", VariableDef::string("Reader"))
    .character("mary", Character::new("Mary").color(Color::GOLD).images(["neutral", "happy"]))
    .character("player", Character::new("{player_name}").color(Color::SKYBLUE))
    .command("give_item", give_item)
```

- **Variables** start at their default on New Game (no more "unset" values), and
  `set_variable` from Rust is type-checked.
- **Characters**: `name` is what the dialogue box shows for that speaker (it may use
  `{variable}`, so `player "Hi."` shows the player's chosen name), `color` tints the
  name, and `images` (optional) lists the valid `show <id> <image>` images.
- An **empty registry turns its checks off**, so prototypes can skip them.

On errors, `run()` returns `AppError::Script` before opening the window; its `Display`
lists every error with file and line. Syntax errors (`show mary` with no image, a stray
`else:`, a tab in the indentation, ...) are reported the same way, next to registry
errors; nothing in the script can make the game panic at startup:

```text
❌ .../assets/story has 2 errors:
  .../story/01_mary.story:12: error: unknown variable 'curiosty'
  .../story/02_moriarty.story:40: error: speaker: unknown character 'marry'
```

Warnings (currently: a `show` whose image file doesn't exist; a placeholder is drawn) are
printed and the game starts. `VnApp::check()` runs the same checks without a window,
returning the loaded story and the warnings (useful in tests), and `VnApp::schema()`
returns the schema.

### Schema export

The registries only exist in Rust, so tools that don't run the game (`vn check`, the
LSP) read them from `<assets>/schema.json` (a `vn_script::SchemaFile`: title, story
directory, entry scene, variables, characters and command signatures):

- **Debug builds** refresh it on every `run()`, before validating the story, and print
  `Updated <path>` when it changed. Unchanged schemas aren't rewritten, so the file only
  shows up in a diff when a registry actually changes. Commit it.
- **`--export-schema`**: `cargo run -- --export-schema` writes it (also in release builds,
  and to `<assets>/schema.json` even when `schema_file(None)`) and exits without opening
  a window or validating the story, so it works while the story has errors.
- `schema_export()` returns the `SchemaFile`, `schema_path()` where it goes, and
  `export_schema()` writes it (`Ok(Some(path))` when the file changed).

| `AppError` | When |
| --- | --- |
| `Story { path, source }` | The story directory (`path`) or a file in it can't be read, or it has no `.story` files |
| `Script { path, errors }` | Parsing or validation found errors (or the entry scene doesn't exist); each error names its file |
| `Schema { path, source }` | `--export-schema` couldn't write the schema file |
| `Screen(io::Error)` | No screen is registered for the initial `ScreenState` |

## Text input

`ctx.ask_text(TextRequest::new(variable, prompt))` switches to the default text input
screen; Enter stores the text in the (string) variable and returns to `Playing`. It's
meant to be called from a command:

```rust
fn ask_name(ctx: &mut GameContext, (variable,): (String,)) -> Option<ScreenState> {
    ctx.ask_text(TextRequest::new(variable, "What is your name?").max_len(20))
}
```

```story
call ask_name player_name
player "Nice to meet you."
```

`TextRequest`: `.initial(text)`, `.max_len(n)` (default 24 characters), `.allow_empty(bool)`
(default false: Enter on an empty field shows "Please type something."). Asking for a
non-string registered variable logs a warning and does nothing.

`TextInputConfig` (`.text_input(|t| ...)`): `prompt_text`, `input_text`, `hint`,
`hint_text`, `box_size`, `box_color`, `box_border`, `background`.

## Save and load

A save holds everything needed to continue: the story position, story variables, the
characters on screen, and every registered [game state](#game-state) value.

### Where saves live

One JSON file per slot in `saves_dir` (default `./saves`): `1.json` … `6.json` for the
numbered slots and `quick.json` for quick save. Slot names may only contain ASCII
letters, digits, `_` and `-`, so a slot can't point outside the directory. Files are
written to `<slot>.json.tmp` and then renamed, so a crash or full disk mid-save leaves the
previous save intact.

```json
{
  "format_version": 1,
  "game": "God Is Watching",
  "saved_at": 1789000000,
  "summary": "mary: The hallway candles were lit when I came down.",
  "story": {
    "scene": "mary_breakfast",
    "offset": 7,
    "scene_fingerprint": 1234567890,
    "pending_choice": false,
    "current": { "Say": { "speaker": "mary", "text": "..." } },
    "variables": { "curiosity": { "Int": 2 } },
    "active_characters": { "hugo": "neutral" }
  },
  "state": { "Inventory": { "items": { "verlaine_letter": 1 } } }
}
```

### Story position

The position is saved as *scene + offset within the scene*, not an absolute instruction
index, plus a fingerprint of that scene's compiled instructions. So:

- Editing **other** scenes (even adding lines before the saved one) doesn't affect a save.
- If the **saved scene itself** changed, the load restarts that scene from its start,
  keeping the saved variables, characters and state, and reports
  `LoadWarning::SceneRestarted`.
- If the saved scene **no longer exists**, the load fails with `SaveError::Story`.

### Game state

Every value registered with `.state(T)` is saved under its type name (`Inventory` for
`my_game::inventory::Inventory`), so `T` must implement `Serialize` and `Deserialize`.
Two registered types with the same name panic at startup. Renaming a type changes its
key, so older saves then report it as missing (see below).

### Loading is all-or-nothing

A load reads and checks everything (file, format, game, scene, every state value) before
changing anything. If any step fails, the running game is left exactly as it was.

### Errors

`SaveError` covers everything that stops a save or load:

| Variant | When | `player_message()` |
|---|---|---|
| `Empty { slot }` | No file for the slot | This slot is empty. |
| `Io { path, source }` | Can't read/write (permissions, disk full, ...) | Could not access the save file (...). |
| `Corrupt { path, source }` | Not valid JSON / not a save (e.g. truncated) | This save file is damaged and can't be loaded. |
| `NewerFormat { found, supported }` | `format_version` newer than this build | This save was made by a newer version of the game. |
| `OtherGame { found, expected }` | Saved by a game with another title | This save belongs to a different game. |
| `Story(VmError)` | The saved scene doesn't exist anymore | This save points to a part of the story that no longer exists. |
| `State(StateError)` | A state value doesn't match its type anymore (names the key) | This save's game data doesn't match this version of the game. |
| `InvalidSlot(name)` | Slot name with characters other than `[A-Za-z0-9_-]` | Invalid save slot. |

`Display` gives the developer-facing message (with paths and serde errors), which the
default screens print to stderr; `player_message()` is what they show on screen.

Non-fatal issues are returned as `LoadReport::warnings` (and printed by `ctx.load`):

| `LoadWarning` | Meaning |
|---|---|
| `SceneRestarted { scene }` | The saved scene was edited; it restarted from its beginning |
| `MissingState { key }` | A state type registered now wasn't in the save; it keeps its initial value |
| `UnknownState { key }` | The save has state for a type that's no longer registered; ignored |

### Default screens

- **Save** and **Load**, as full screens (`ScreenState::Save` / `Load`) or as overlays over
  the game (`SAVE_OVERLAY` / `LOAD_OVERLAY`, opened from the pause menu): a list of slots showing
  how long ago each was saved and the line that was on screen, "Empty", or the error in
  red for unreadable files. Load also lists the quick save when there is one. Saving over
  a used slot and loading during play ask first (see [Confirmation
  dialogs](#confirmation-dialogs)); results and errors show as notifications; a successful
  load goes to `Playing`. The list refreshes whenever a save is written. Back (or Esc / Backspace) returns to the previous
  screen, or to the pause menu for the overlays.
- The main menu has a **Load** button and the pause menu has Save, Load, Quick Save and
  Quick Load. A HUD button can open the save panel directly:
  `.hud_button("Save", Action::overlay(SAVE_OVERLAY))`.
- **Quick save / load**: F5 / F9 on the playing screen, or the pause menu.

`SaveMenuConfig` (`.save_menu(|s| ...)`):

| Option | Default |
|---|---|
| `slots(n)` | 6 |
| `save_title(text)`, `load_title(text)`, `title_text(style)` | "Save Game", "Load Game", Title font 44 px |
| `slot_size(w, h)`, `slot_spacing(px)`, `slot_color(c)` | 760×64 (shrinks to fit), 10, dark blue |
| `slot_title_text`, `slot_summary_text`, `error_text` | Menu 20 px, Dialogue 17 px gray, Menu 17 px red |
| `confirm_overwrite(bool)`, `confirm_load_in_game(bool)` | `true`, `true` |
| `empty_label(text)` | "Empty" |
| `back_button(\|b\| ...)`, `back_label(text)`, `back_keys(keys)` | 200×46, "Back", Esc and Backspace |
| `backdrop(c)` | near-black at 235 alpha (overlay version only) |
| `background(bg)` | none |

### From code

```rust
ctx.save("1")?;                          // in a screen, overlay or command
let report = ctx.load("quick")?;         // Err(SaveError), or Ok with warnings
ctx.saves.slot("2")                      // SlotInfo { slot, save: Result<SaveFile, SaveError> }
ctx.saves.latest()                       // most recent readable save, e.g. for "Continue"
ctx.saves.delete("3")?;
```

`Saves` doesn't need a window, so it can also be used from tools and tests
(`Saves::new(dir, title).save(slot, &story, &state)`). The VM part is
`StoryVm::snapshot()` / `restore()` in `vn_script`.

## Lower level: ScreenStateManager

`VnApp` is built on `ScreenStateManager`, which a game can drive itself for a custom loop:

```rust
let (mut rl, thread) = raylib::init().size(1280, 720).build();
let mut manager = ScreenStateManager::new(&mut rl, &thread, ScreenState::StartScreen, factory, assets, "story")?;

while !manager.quit_requested() {
    if rl.window_should_close() {
        manager.request_close();
    }
    manager.update(&mut rl, &thread);
    let mut d = rl.begin_drawing(&thread);
    manager.draw(&mut d);
}
```

`factory` is any `ScreenFactory` (`create_screen(&state) -> Option<Box<dyn Screen>>`, and
optionally `create_overlay(name)`); `DefaultScreens` is the one `VnApp` uses. The manager's
`state`, `commands` and `saves` fields hold what `VnApp::state`/`command` register, and
`open_overlay`/`close_overlay`/`confirm`/`notify` control overlays and notifications
directly, and `rollback` holds the history. `settings` starts in memory
(`SettingsStore::in_memory()`, never written); use `SettingsStore::load(path)` to keep
them in a file. `close_confirmation` is the message for `request_close()`. Textures and fonts live on the GPU, so the
manager must be created after the window, and dropped before it (declare it after `rl`).

## Resources

`ResourceManager` resolves every path relative to the asset root passed to
`ScreenStateManager::new`.

### Textures

`get_or_load(path, rl, thread)` loads a texture once and caches it by path. A missing or
unreadable file logs a warning and is replaced by a generated placeholder: a
300×500 card labeled with the path, with a color derived from the path so each
character/expression keeps the same color between runs. A story can be played before
its art exists.

### Fonts

The engine ships **Noto Sans** (SIL Open Font License 1.1, `assets/fonts/OFL.txt`) and
compiles it into the binary as the default font.

Each kind of text has a `FontRole`:

`Default`, `Title`, `Menu`, `Button`, `Dialogue`, `Speaker`, `Choice`, `Custom(&'static str)`

A game assigns files from `<assets>/fonts/` to roles:

```rust
VnApp::new("My Novel")
    .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
    .font(FontRole::Custom("journal"), "Caveat.ttf")
```

(or `resources.set_font(&mut rl, &thread, role, file)` when driving `ScreenStateManager`
directly). Screens draw through `resources.fonts`, or through `TextStyle` and the `ui`
helpers:

```rust
fonts.draw(d, FontRole::Dialogue, text, Vector2::new(x, y), 26.0, Color::RAYWHITE);
let size = fonts.measure(FontRole::Button, text, 22.0);
let lines = fonts.wrap(FontRole::Dialogue, text, 26.0, max_width);
```

Behavior:

- Lookup order: the role's font, then the `Default` role's font, then the built-in font.
- Each file loads once, however many roles use it.
- A missing or unreadable font logs a warning and the role keeps its fallback.
- Supported formats: `.ttf` and `.otf`.
- Glyphs are rasterized once at 64 px and scaled when drawn, with mipmaps and trilinear
  filtering so small sizes stay smooth.
- Glyph coverage: ASCII, Latin-1, Latin Extended-A, general punctuation
  (U+2010–U+2027: dashes, curly quotes, ellipsis) and €. Other characters draw as `?`.
- Fonts are loaded through raylib's C `LoadFontFromMemory` rather than raylib-rs'
  wrapper. The wrapper passes the glyph string's byte length as the codepoint count,
  which reads out of bounds for non-ASCII glyphs.
