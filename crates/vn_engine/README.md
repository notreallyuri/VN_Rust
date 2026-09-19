# vn_engine

A raylib-based visual novel engine built on [`vn_script`](../vn_script/README.md).

It re-exports `raylib` and `vn_script` (as `vn_engine::script`), so games build against
the same versions the engine uses and only need `vn_engine` as a dependency.

## Quick start

A game is a `VnApp` built from defaults, with only the parts it wants changed:

```rust
use vn_engine::raylib::prelude::*;
use vn_engine::{FontRole, Action, MenuItem, TextStyle, VnApp};

fn main() -> std::io::Result<()> {
    VnApp::new("My Novel")
        .size(1280, 720)
        .assets(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"))
        .font(FontRole::Dialogue, "NotoSerif-Regular.ttf")
        .main_menu(|m| {
            m.button_style(|b| b.size(260.0, 52.0).roundness(0.3))
                .button("New Game", Action::NewGame)
                .button("Continue", Action::Continue)
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
| `tooltips(\|t\| ...)` | on | Configure tooltips (see [Tooltips](#tooltips)) |
| `exit_key(Option<key>)` | `None` | A key that closes the window. Off by default, so Esc can open the pause menu |
| `state(value)` | | Register game state (see [Game state](#game-state)) |
| `command(name, handler)` | | Handle `call <name> ...` from stories (see [Commands](#commands)) |
| `variable(name, VariableDef)` | | Register a story variable (see [Registries and validation](#registries-and-validation)) |
| `character(id, Character)` | | Register a character |
| `entry_scene(id)` | first scene of the first file | Scene the story starts from |
| `warn_missing_art(bool)` | `true` | Warn at startup about `show`s, `background`s, `music` and `sound`s with no file |
| `audio(\|a\| ...)` | on | Configure music and sound (see [Audio](#audio)) |
| `on_scene_enter(\|ctx, scene\| ...)`, `on_choice(\|ctx, index, text\| ...)` | | Run Rust code when the story enters a scene or the player picks an option (see [Hooks](#hooks)) |
| `hot_reload(bool)` | on in debug builds | Reload the story when a `.story` file changes (see [Hot reload](#hot-reload)) |
| `schema_file(Option<&str>)` | `Some("schema.json")` | Where the schema is exported, relative to the assets (see [Schema export](#schema-export)); `None` turns the export off |
| `text_input(\|t\| ...)` | | Configure the default text input screen |
| `saves_dir(path)` | the platform data directory | Where save files and settings go (see [Where saves live](#where-saves-live)); `saves_path()` returns the directory in use |
| `autosave(bool)` | `true` | Save to the `auto` slot on entering a scene and on quitting (see [Autosave](#autosave)) |
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
| `button(label, action)`, `item(MenuItem)` | New Game, Continue, Load, Settings, Quit |
| `button_style(\|b\| ...)` | `ButtonStyle::default()` (240×52) |
| `buttons_y(f)` | 0.45: top of the button area, as a fraction of the window height |
| `layout(\|l\| ...)`, `spacing(px)` | a column anchored at the top of the button area, 18 apart (see [Layouts](#layouts)) |
| `margin(px)` | 40 from the sides and bottom. A menu that would reach closer to the bottom moves up (not into the title), then tightens its spacing |
| `background(bg)` | none |

The first `button`/`item` call replaces the default list; later calls append.
`MenuItem::new(label, action).style(|b| ...)` changes one button's style on top of
`button_style`, and `.tooltip(text)` gives it a [tooltip](#tooltips) (main and pause menus).

`Action` (shared by the main menu, the pause menu and HUD buttons):

| Action | Behavior |
|---|---|
| `NewGame` | Reset the story and game state, then `Playing` |
| `Continue` | Back to the game in progress; after a restart, load the most recent save (usually the [autosave](#autosave)). With neither, a notification says there is no saved game |
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
- The story's `background` fills the window (scaled to cover it, cropping the edges if
  the aspect ratio differs); without one, the `background` option is drawn.
- Characters stand at their `at` position (a fraction of the window width, see
  `position`), bottom-aligned and scaled to `character_height` of the window height.
  Characters without a position are spread evenly (sorted by id).
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
| `choice_layout(\|l\| ...)` | a column centered in the window, inside the dialogue box's margin |
| `end_title(text)`, `end_title_text(style)` | "The End", Title font 56 px |
| `end_hint(text)`, `end_hint_text(style)` | "Click to return to the menu", Menu font 20 px gray |
| `advance_keys(keys)` | Space, Enter |
| `pause_key(Option<key>)`, `pause_overlay(name)` | `Some(Escape)`, `PAUSE_OVERLAY` |
| `menu_key(Option<key>)` | `None` |
| `after_end(state)` | `MainMenu` |
| `background(bg)` | none; drawn when the story has no `background` |
| `position(Position, x)` | `far_left` 0.15, `left` 0.3, `center` 0.5, `right` 0.7, `far_right` 0.85 of the window width (the character's center) |
| `character_height(Option<fraction>)` | `Some(0.8)`: sprites are scaled to 80% of the window height, keeping their aspect ratio. `None` draws them at their pixel size |
| `hud_button(label, action)`, `hud_item(HudButton::new(label, action).tooltip(text))` | none |
| `quick_save_key(Option<key>)`, `quick_load_key(Option<key>)` | `Some(F5)`, `Some(F9)` (the `quick` slot) |
| `hud_button_style(\|b\| ...)`, `hud_margin(px)`, `hud_spacing(px)` | 130×40, dark translucent, 18 px text; 16; 10 |
| `hud_layout(\|l\| ...)` | a row anchored top-right, inside `hud_margin` |

raylib's "Esc closes the window" is turned off (see `VnApp::exit_key`).

## Styles

| Type | Fields / builder methods |
|---|---|
| `TextStyle::new(font, size, color)` | `.font()`, `.size()`, `.color()` |
| `ButtonStyle::default()` | See [Buttons](#buttons) |
| `SliderStyle::default()` | `.track_height()`, `.knob_radius()`, `.track_color()`, `.fill_color()`, `.knob_color()`, `.step_marks(bool)` |
| `DialogueBoxStyle::default()` | `.height()`, `.margin()`, `.padding()`, `.color()`, `.roundness()` |
| `Background` | `Color(color)` fills the screen; `Image(path)` covers it with an asset (scaled, cropping the edges if the aspect ratio differs) |

## Buttons

Every button the engine draws (menus, HUD, choices, dialogs, settings, save screens)
takes a `ButtonStyle`. Custom screens draw with the same function.

```rust
ButtonStyle::default()
    .size(260.0, 52.0)
    .color(Color::new(38, 33, 42, 240))
    .roundness(0.18)
    .border(1.0, Color::new(110, 100, 86, 255))
    .shadow(0.0, 3.0, Color::new(0, 0, 0, 120))
    .icon(ButtonIcon::new("ui/icon_save.png").size(16.0))
    .hovered(|l| l.border(1.5, PARCHMENT).transform(|t| t.offset(6.0, 0.0)))
    .pressed(|l| l.transform(|t| t.scale(0.97)))
    .transition(0.12)
    .click_sound("page_turn")
```

**Base look**

| Builder | Default | Meaning |
| --- | --- | --- |
| `size(w, h)` | 240×52 | |
| `color(c)` | dark blue | Fill. Also sets the hovered fill to a lighter shade and the pressed fill back to `c` |
| `hover_color(c)`, `pressed_color(c)` | | Fill in those states |
| `roundness(0..1)` | 0 | Rounded corners (fill, border and shadow) |
| `text(style)`, `font(role)`, `font_size(px)`, `text_color(c)`, `hover_text_color(c)` | Button font 22 px white | Label |
| `align(TextAlign)` | `Center` | `Left`, `Center` or `Right` inside the padding |
| `padding(x, y)` | 12, 6 | Space between the edge and the label/icon |
| `overflow(TextOverflow)` | `Ellipsis` | A label wider than the button: `Overflow` (draw past the edge), `Ellipsis` (cut with …), `Shrink` (smaller font, down to 8 px), `Wrap` (several lines, centered vertically) |
| `border(width, c)`, `no_border()` | none | An outline |
| `shadow(x, y, c)` | none | A copy of the shape, offset and drawn behind |
| `image(ButtonImage)` | none | A texture instead of the fill (see below) |
| `icon(ButtonIcon)` | none | An image next to the label (see below) |
| `transform(\|t\| ...)`, `skew(x°, y°)`, `rotate(°)`, `scale(f)` | identity | See Transforms |

**States.** `hovered`, `pressed`, `focused` and `disabled` are each a `ButtonLook`, a
list of overrides on top of the base: `fill`, `text_color`, `border(width, c)`,
`shadow(x, y, c)`, `image(..)`, `transform(|t| ...)` and `opacity(0..1)`. Unset fields keep
the base value. They stack in the order focused → hovered → pressed → disabled.

| State | When | Default |
| --- | --- | --- |
| `hovered(\|l\| ...)` | The pointer is over the button | a lighter fill |
| `pressed(\|l\| ...)` | The left button is held on it | the base fill |
| `focused(\|l\| ...)` | `Button::focused(true)` (for keyboard navigation) | a 2 px white border |
| `disabled(\|l\| ...)` | `Button::disabled(true)`, or a `MenuItem` whose `enabled_if` is false | 45% opacity; it ignores the pointer |

`transition(seconds)` animates between states: colors, border width, shadow, transform
and opacity blend, and the image switches halfway. 0 (the default) switches at once.

**Images.** `ButtonImage::new(path)` (relative to the assets) replaces the fill. It is
stretched to the button, or cut into nine pieces with `.nine_slice(left, top, right,
bottom)` / `.slice_all(px)` so the corners keep their size. `.tint(c)` colors it. A state
can swap the image (`hovered(|l| l.image(ButtonImage::new("ui/hover.png")))`). Images load
the first time a button is drawn; a missing file shows the usual placeholder.

**Icons.** `ButtonIcon::new(path)`: `.size(height)` (width follows the image's aspect),
`.side(IconSide::Left | Right)`, `.gap(px)`, `.tint(c)` (default: the label color, so it
follows state colors; draw icons white). With an empty label the icon is centered alone.

**Transforms.** `Transform` has `scale` (`scale(f)` / `scale_xy(x, y)`), `rotate(degrees)`,
`skew(x°, y°)` (x leans the top to the left for positive values), `offset(x, y)` in pixels and
`origin(fx, fy)`, the pivot as a fraction of the button (default the center). Everything
drawn for the button, including its label, goes through it. Clicks follow the base
transform: a rotated or skewed button responds where it is drawn. State transforms
(a hover offset, a press scale) don't move the click area, so a button can't slide away
from the pointer.

**Sounds.** `hover_sound(id)` plays when the pointer enters the button, `click_sound(id)`
on a click (see [Audio](#audio) for where sounds live).

**Clicks** happen on release: press on the button, then release on it. Pressing and
dragging off cancels, like a desktop button.

**Per-button styles.** `MenuItem::style`, `HudButton::style` and
`PlayingConfig::choice_button_for(|index, text, style| ...)` change one button on top of the
group's style (the example marks options starting with `[` differently). Each HUD button
is laid out at its own size. `MenuItem::enabled_if(|view| ...)` disables an item: it
draws with the disabled look and can't be clicked, but its tooltip still shows. The
default main menu's Continue uses `can_continue` (a story in progress, or any save).
`GameView` (`ctx.view()` from either context) is the read-only story, state, saves and
settings these checks receive.

**In a custom screen:**

```rust
// update
if ui::button_clicked(&mut ctx, rect, &style) { /* ... */ }
// draw
ui::draw_button(d, ctx, rect, "Back", &style);
ui::Button::new("Gallery", &style).disabled(locked).focused(selected).draw(d, ctx, rect);
```

`ui::button_hovered(rl, rect, &style)` is the hit test on its own. Only the layer that
receives input shows hover and pressed looks: a screen under an overlay, or a pause menu
under the settings overlay, draws its buttons at rest. Custom drawing that reacts to the
pointer should use `ctx.pointer_over(d, rect)` rather than `ui::is_hovered` for the same
reason. Resolving a look
(`style.look(StateAmounts { hover, press, focus, disabled })`), `Transform::apply` /
`invert` / `contains` and `step_amount` are public for custom widgets and tests.

## Layouts

Every list of buttons the engine draws (main menu, pause menu, choices, HUD, save slots)
is placed by a `Layout`, set with that screen's `layout` option. The option receives the
current layout, so changing one part keeps the rest:

```rust
.main_menu(|m| m.layout(|l| l.rows_of([1, 2, 2, 1]).align(Align::Stretch)))
.main_menu(|m| m.layout(|l| l.grid(2).anchor(Anchor::Bottom)))
.pause_menu(|p| p.layout(|l| l.grid(2)))
.playing(|p| p.choice_layout(|l| l.anchor(Anchor::Left).align(Align::Start)))
.save_menu(|s| s.slot_size(560.0, 72.0).slot_layout(|l| l.grid(2)))
```

| Builder | Meaning |
| --- | --- |
| `.column()`, `.row()`, `.grid(columns)` | How items are arranged. A grid fills rows left to right; each column is as wide as its widest item, each row as tall as its tallest |
| `.rows_of([counts])` | The number of items in each row, e.g. `[1, 2, 2, 1]`: one button, two rows of two, one button. Items past the list continue in rows of the last count; zeros are ignored, and an empty list is a column. Unlike a grid, each row is sized on its own and placed by `align`, so a single button is centered over (or aligned with) the rows below it |
| `.custom(\|area, sizes\| rects)` | Your own placement: gets the available `Rectangle` and each item's size, returns one `Rectangle` per item |
| `.anchor(Anchor)` | Where the block of items sits in the available area: `TopLeft`, `Top`, `TopRight`, `Left`, `Center`, `Right`, `BottomLeft`, `Bottom`, `BottomRight` |
| `.align(Align)` | Where an item narrower than its column sits in it (or, for `rows_of`, where a narrower row sits): `Start`, `Center` (default), `End`. `Stretch` makes items fill instead: a column's items take the widest item's width, a grid's take their column's width, and a short `rows_of` row spreads the spare width over its items so every row is as wide as the widest (with `[1, 2, 2, 1]`, the single buttons span both columns) |
| `.spacing(px)`, `.spacing_xy(x, y)` | Gaps between columns (x) and rows (y). They shrink (down to 0) when the block doesn't fit the area |

What "the available area" is depends on the screen:

| Screen | Area | Default |
| --- | --- | --- |
| Main menu | From `buttons_y` to the bottom `margin`, inside the side margins | column, top, 18 |
| Pause menu | Inside the panel, below the title. The panel widens to fit a grid | column, 10 |
| Choices | The window inside the dialogue box's margin | column, center, 16 |
| HUD | The window inside `hud_margin` | row, top-right, 10 |
| Save slots | Between the title and the Back button. Slots get shorter to fit the rows | column, top, 10 |

`Layout::place(area, &sizes)`, `block_size(&sizes)`, `row_counts(count)`, `columns(count)`
and `rows(count)` are public, for custom screens. `PlayingConfig::choice_rects`/`hud_rects`,
`MainMenuConfig::button_rects` and `PauseMenuConfig::button_rects`/`panel` give the
rectangles a default screen uses.

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
| `GameContext` | What `update` gets: `rl`, `thread`, `resources`, `story`, `state`, `saves`, `rollback`, `settings`, `previous` (the screen before this one); plus `open_overlay(name)`, `run_command(name, args)`, `ask_text(request)`, `save(slot)`, `load(slot)`, `tooltip(rect, text)`, `play_sound(id)` and `view()` |
| `DrawContext` | What `draw` gets: `resources`, `story`, `state`, `saves`, `characters`, `settings`, `fonts()` and `view()`; `interactive` is true only for the layer that receives input (the top overlay, or the screen when no overlay is open), and `pointer_over(d, rect)` is a hover test that is false on the other layers |

Switching to a state with no screen logs a warning and stays on the current one.

The `ui` module has the pieces the default screens use, for custom screens to reuse:

| Function | Purpose |
|---|---|
| `draw_button(d, ctx, rect, label, &ButtonStyle)`, `Button::new(label, &style)` | A button with every state, image, icon and transform (see [Buttons](#buttons)) |
| `button_clicked(&mut ctx, rect, &style)`, `button_hovered(rl, rect, &style)` | Button hit tests that follow the transform; `button_clicked` fires on release and plays the style's sounds |
| `is_clicked(rl, rect)`, `is_hovered(rl, rect)` | Plain rectangle hit tests (on press) |
| `draw_slider(d, area, fraction, steps, highlighted, &SliderStyle)`, `slider_fraction`, `slider_step` | The settings sliders |
| `fit_text(fonts, style, text, width)` | Cut text with … to fit a width |
| `draw_text`, `draw_text_centered`, `draw_text_wrapped` | Text with a `TextStyle`; wrapped returns the height used |
| `draw_text_wrapped_visible(.., visible)` | Wrapped text showing only the first `visible` characters (the typewriter) |
| `screen_size(rl)` | Window size as a `Vector2` |
| `load_background(ctx, bg)`, `draw_background(d, resources, bg)` | `Background` support (load in `update`, draw in `draw`) |
| `draw_texture_cover(d, texture, rect)` | Draw a texture covering `rect`, cropping to keep its aspect ratio |

See `examples/god_is_watching/src/screens/credits.rs` and `evidence.rs`.

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
| `layout(\|l\| ...)` | a column (see [Layouts](#layouts)); a grid widens the panel |
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

Player preferences, separate from saves: `Settings { fullscreen, text_speed, music_volume, sound_volume }`, stored as
`settings.json` in the saves directory and written as soon as something changes (via a
temporary file, like saves). A missing or unreadable file gives the defaults (windowed,
40 characters per second), with a warning for an unreadable one; missing fields take
their default, so new settings don't break old files.

- **Display:** windowed or fullscreen (borderless, at the monitor's resolution). Applied
  at startup and whenever it changes.
- **Text speed:** characters per second for the typewriter on the playing screen; `0` is
  instant.
- **Music volume**, **Sound volume:** percentages (defaults 70 and 80); `music_gain()` and
  `sound_gain()` give them as 0.0–1.0. See [Audio](#audio).

The default settings screen shows one row per setting (`SettingsRow::Display`,
`TextSpeed`, `MusicVolume`, `SoundVolume`). Display is a button that toggles windowed and
fullscreen; the others are sliders: drag the knob or click anywhere on the track, or hover
the row and press ←/→ for one step. Text speed stops at each entry of `text_speeds`
(Slow → Normal → Fast → Instant); the volumes move in `volume_step` steps, with the value
("Normal", "70%", "Off") to the right. Each row has a [tooltip](#tooltips). There is also and a sample line that types out at the chosen speed. It exists as a screen
(`ScreenState::Settings`, in the default main menu) and as an overlay
(`SETTINGS_OVERLAY`, in the pause menu). Back, Esc or Backspace return to where it was
opened from.

`SettingsConfig` (`.settings(|s| ...)`):

| Option | Default |
| --- | --- |
| `title(text)`, `title_text(style)` | "Settings", Title font 44 px |
| `label_text(style)` | Menu font 24 px |
| `value_button(\|b\| ...)` | 260×46, 20 px text; also the size of every row's control |
| `value_text(style)` | Menu font 18 px light gray: the value next to a slider |
| `slider(\|s\| ...)` | `SliderStyle::default()`: a 6 px track, a 10 px knob, dots at each stop when there are 12 or fewer |
| `row_width(px)`, `row_spacing(px)` | 600, 16 |
| `text_speeds([(label, chars_per_second)])` | Slow 20, Normal 40, Fast 80, Instant 0 (a value not in the list shows as "N chars/s") |
| `audio_rows(bool)` | `true`: show the two volume rows |
| `volume_step(percent)` | 5 |
| `sample_sound(id)` | none: a sound played when the sound slider is released (or stepped with the keys), so the player hears the new volume |
| `tooltip(row, Option<&str>)` | a short description per row; `None` removes it |
| `sample_text(text)`, `sample_text_style(style)` | "This is how fast the story's text appears.", Dialogue font 22 px |
| `back_button(\|b\| ...)`, `back_label(text)`, `back_keys(keys)` | 200×48, "Back", Esc and Backspace |
| `backdrop(c)` | black at 200 alpha, behind the overlay |
| `background(bg)` | none, behind the screen |

Labels (`display_label`, `windowed_label`, `fullscreen_label`, `text_speed_label`,
`music_volume_label`, `sound_volume_label`) are
public fields. `set_fraction(row, &mut settings, 0.0..=1.0)`, `fraction(row, &settings)`,
`step(row, &mut settings, ±1)` and `value_name(row, &settings)` are the mapping the sliders
use, public for custom settings screens. From code, `ctx.settings.values` reads the settings and
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

Rollback stops at the first line after a barrier. It resets on New Game. The history is
stored in save files (see [Save and load](#rollback-history)), so a player can roll back
after loading, up to the same barriers.

`RollbackConfig` (`.rollback(|r| ...)`):

| Option | Default |
| --- | --- |
| `enabled(bool)` | `true` |
| `max_steps(n)` | 100 lines of history |
| `through_choices(bool)` | `true` |
| `block_command(name)` | none |
| `save_history(bool)` | `true`: store the history in save files |
| `mouse_wheel(bool)` | `true` |
| `back_keys(keys)`, `forward_keys(keys)` | Page Up, Page Down |

Each step stores a story snapshot and the game state as JSON (the same data as a save), so
large state types make history more expensive; lower `max_steps` if needed. From code:
`ctx.rollback.back(story, state)`, `forward`, `can_go_back()`, `steps_back()`,
`mark_barrier()`, `clear()`; `history()` (the `Checkpoint`s a save stores),
`restore_history(checkpoints, story)` and `retain_valid(story)` (drop what no longer fits
the story).

## Notifications

`ctx.notify(text)` / `ctx.notify_error(text)` show a short message at the top-left, above
every screen and overlay (quick save/load use them). `ToastConfig` (`.toast(|t| ...)`):
`text(style)`, `error_text(style)`, `background(color)`, `seconds(s)` (2.5),
`margin(px)` (16).

## Tooltips

Hovering a control for `delay` seconds shows a small box of text next to the pointer. It
moves to the other side near the window's edges, wraps at `max_width`, and hides on click
until the pointer leaves and comes back.

Built in: the settings rows, the save slots' Delete button, `MenuItem::tooltip` in the
main and pause menus, and HUD buttons:

```rust
.playing(|p| p.hud_item(HudButton::new("Save", Action::overlay(SAVE_OVERLAY)).tooltip("F5 quick saves")))
```

A custom screen or overlay registers one in `update`:

```rust
ctx.tooltip(rect, "Everything you have filed so far");
```

It only counts while the pointer is over `rect`, and the last one registered in a frame
wins. The manager draws it on top of everything.

`TooltipConfig` (`.tooltips(|t| ...)`):

| Option | Default |
| --- | --- |
| `enabled(bool)` | `true` |
| `delay(seconds)` | 0.5 |
| `text(style)` | Menu font 16 px white |
| `background(c)`, `border(Option<c>)` | near-black, a gray 1 px border |
| `padding(px)`, `max_width(px)`, `offset(x, y)` | 8, 360, (14, 20) from the pointer |

`TooltipTimer` is the timing on its own (`update(hovered, now, clicked)`,
`visible(now, delay)`), and `draw_tooltip(d, fonts, text, config)` draws one.

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

## Hooks

Hooks run Rust code at points in the story without a `call` in the script:

```rust
VnApp::new("My Novel")
    .on_scene_enter(|ctx, scene| {
        ctx.state.get_mut::<Journal>().visited(scene);
        None
    })
    .on_choice(|ctx, index, text| {
        println!("picked option {} ({})", index, text);
        None
    })
```

| Hook | Runs |
| --- | --- |
| `on_scene_enter(\|ctx, scene\| ...)` | When the story enters a scene: at the start (New Game), on every `jump` (including one to the same scene), and when a load or hot reload restarts an edited scene. Not when a save or rollback puts the story back in the middle of a scene |
| `on_choice(\|ctx, index, text\| ...)` | Right after the player picks an option, before the option's lines run. `text` is the option as shown (interpolated) |

Hooks are registered in order and all run; like commands, returning `Some(state)` switches
screens (the first one returned wins), and the story continues when the playing screen
is entered again. They see the same `GameContext` as commands. Rolling back and playing
forward again runs them again, so a hook with a permanent effect (an achievement) should
check whether it already happened, or the story should `commit` first.

In `vn_script`, scene entries are `Event::SceneEnter { scene }` events, off by default
(`StoryVm::set_scene_events(true)`; the engine turns them on).

## Hot reload

In debug builds, editing a `.story` file while the game runs reloads the story without
restarting: every half second the engine checks the files' modification times (and
added or removed files) under `story_dir`. The new story is validated exactly like at
startup (same schema, entry scene and art warnings), then the player's position moves
into it the way a save loads:

| Change | Result |
| --- | --- |
| Other scenes | The same line stays on screen; rollback history in unchanged scenes is kept. Notification: "Story reloaded" |
| The current scene | The scene restarts from its top, keeping variables, characters and game state; rollback history is cleared. Notification: "Story reloaded; scene '…' restarted" |
| The current scene was deleted | Nothing changes; the error is shown as a notification |
| Errors in the story | Nothing changes; the diagnostics go to the console and to a red panel at the top of the window (below) |

The error panel lists every diagnostic as `file:line: error: message`, with paths
relative to the story directory, under "Story not reloaded: N errors (still running the
previous version)". It stays until a reload succeeds; F2 (`SCRIPT_ERRORS_KEY`) collapses it
to its title and back. Lines that don't fit in 60% of the window end with "... and N more
lines". The game keeps running the previous story underneath. `ScriptErrors::from_error(&error,
story_dir)` builds it and `ScreenStateManager::show_script_errors` shows it, for custom
loops.

`.hot_reload(false)` turns it off; `.hot_reload(true)` turns it on in release builds too.
The pieces are public for custom loops: `StoryLoader` (what `VnApp::loader()` returns:
load and validate), `StoryWatcher` (`poll(now)` / `changed()`),
`swap_story(current, fresh, rollback)` and `ScreenStateManager::reload_story(story)`.

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
  .../story/04_santa_ilde.story:12: error: unknown variable 'trsut'
  .../story/01_box_14.story:40: error: speaker: unknown character 'marry'
```

Warnings (currently: a `show` or `background` whose image file doesn't exist; a
placeholder is drawn) are
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

By default saves go to the platform's data directory, in a folder named after the game
title (`slug("God Is Watching")` is `god_is_watching`):

| Platform | Directory |
| --- | --- |
| Linux | `$XDG_DATA_HOME/<game>`, or `~/.local/share/<game>` |
| macOS | `~/Library/Application Support/<game>` |
| Windows | `%APPDATA%\<game>` |

`default_saves_dir(title)` computes it (falling back to `./saves` when no home directory
is set), `.saves_dir(path)` picks another directory, and debug builds print the directory
in use at startup. `settings.json` lives in the same directory.

One JSON file per slot: `1.json` … `6.json` for the numbered slots, `quick.json` for quick
save and `auto.json` for the autosave, each with an optional `<slot>.png` thumbnail. Slot names may only contain ASCII
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
    "variables": { "trust": { "Int": 2 } },
    "active_characters": { "hugo": "neutral" }
  },
  "state": { "Inventory": { "items": { "verlaine_letter": 1 } } }
}
```

### Autosave

With `.autosave(true)` (the default) the engine saves to the `auto` slot (`AUTO_SLOT`):

- when the story enters a scene, once its first line is on screen (so the thumbnail and
  the summary show the new scene), and
- when the game closes, if a story is in progress (not before it starts, not at the end).

Autosaves are silent; failures are only logged. They include the rollback history like
any other save. `ctx.autosave()` requests one from a command or screen; it's written
during that frame's draw, after the thumbnail is taken. The Load screen lists the
autosave at the top, and `Action::Continue` loads the newest save, which is usually it.

### Thumbnails

While the playing screen is showing with no overlay open, the engine grabs the frame at
most once a second, scaled down to `THUMBNAIL_WIDTH` (320) pixels wide. Every save writes
that picture to `<slot>.png` next to the JSON, so saving from the pause menu shows the
game, not the menu. A save made with no picture yet removes the old one, so a slot never
shows a stale thumbnail. `Saves::write_thumbnail(slot, Option<&Image>)` and
`thumbnail_path(slot)` are public; `Saves::delete` removes both files.

### Story position

The position is saved as *scene + offset within the scene*, not an absolute instruction
index, plus a fingerprint of that scene's compiled instructions. So:

- Editing **other** scenes (even adding lines before the saved one) doesn't affect a save.
- If the **saved scene itself** changed, the load restarts that scene from its start,
  keeping the saved variables, characters and state, and reports
  `LoadWarning::SceneRestarted`.
- If the saved scene **no longer exists**, the load fails with `SaveError::Story`.

### Rollback history

A save made by `ctx.save` (the save menus, quick save) also stores the rollback history
(`SaveFile::rollback`, a list of `Checkpoint`s), unless `.rollback(|r|
r.save_history(false))`. After a load, the player can roll back as they could before
saving. History that no longer fits the story is dropped: every checkpoint before one in
an edited or deleted scene, and all of it when the saved scene itself was restarted.
Saves without history (older ones, or `Saves::save`) load with none. It's the largest
part of a save file (up to `max_steps` snapshots).

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
  red for unreadable files, with the slot's thumbnail on the left. Load also lists the
  autosave and the quick save when they exist. Hovering a used slot shows a **Delete**
  button (the Delete key works too), which asks first. Saving over
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
| `slot_layout(\|l\| ...)` | a column at the top (see [Layouts](#layouts)) |
| `slot_title_text`, `slot_summary_text`, `error_text` | Menu 20 px, Dialogue 17 px gray, Menu 17 px red |
| `confirm_overwrite(bool)`, `confirm_load_in_game(bool)` | `true`, `true` |
| `thumbnails(bool)`, `thumbnail_color(c)` | `true`, near-black behind a slot with no picture |
| `allow_delete(bool)`, `confirm_delete(bool)` | `true`, `true` |
| `delete_button(\|b\| ...)`, `delete_label(text)`, `delete_keys(keys)` | 76×26 dark red, "Delete", the Delete key |
| `empty_label(text)` | "Empty" |
| `back_button(\|b\| ...)`, `back_label(text)`, `back_keys(keys)` | 200×46, "Back", Esc and Backspace |
| `backdrop(c)` | near-black at 235 alpha (overlay version only) |
| `background(bg)` | none |

### From code

```rust
ctx.save("1")?;                          // in a screen, overlay or command
let report = ctx.load("quick")?;         // Err(SaveError), or Ok with warnings
ctx.saves.slot("2")                      // SlotInfo { slot, save: Result<SaveFile, SaveError> }
ctx.saves.latest()                       // most recent readable save (what Action::Continue loads)
ctx.autosave();                          // write the auto slot this frame (when autosave is on)
ctx.saves.delete("3")?;
```

`Saves` doesn't need a window, so it can also be used from tools and tests
(`Saves::new(dir, title).save(slot, &story, &state)`). The VM part is
`StoryVm::snapshot()` / `restore()` in `vn_script`.

## Audio

Stories choose the music and sounds (`music <track>`, `music none`, `sound <id>`, see
SCRIPT.md 2.6); the engine plays them.

- Files live in `<assets>/music/<track>` and `<assets>/sounds/<id>`, as `.ogg`, `.mp3`,
  `.wav` or `.flac` (tried in that order; `music_path` and `sound_path` find them). A
  missing file is a startup warning and the game stays silent there.
- Music loops. A new track crossfades with the old one, and `music none` fades out.
- The music follows the screen: the start screen and main menu play the menu music (if
  any); the playing screen and text input play the story's track (`StoryVm::music()`);
  other screens (Save, Load, Settings, custom screens) keep whatever was playing. Since
  the track is part of the story's state, loading a save, rolling back, New Game and hot
  reload all switch to the right music with no extra code.
- Sounds play once, when the story reaches them. Rolling back or loading doesn't replay
  them.
- Volumes come from the [settings](#settings).
- If no audio device can be opened, a warning is printed and the game runs silently.

`AudioConfig` (`.audio(|a| ...)`):

| Option | Default |
| --- | --- |
| `menu_music(track)` | none |
| `fade_seconds(s)` | 1.0; 0 switches tracks at once |
| `enabled(bool)` | `true`; `false` never opens the audio device |

From code, `ctx.play_sound(id)` plays a sound (from a command, say) and `ctx.music()` is the
track playing. `Audio` holds the device, streams and cached sounds; the audio device is
opened once and kept for the whole program.

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
    manager.draw(&mut d, &thread);
}
manager.autosave();
```

`factory` is any `ScreenFactory` (`create_screen(&state) -> Option<Box<dyn Screen>>`, and
optionally `create_overlay(name)`); `DefaultScreens` is the one `VnApp` uses. The manager's
`state`, `commands` and `saves` fields hold what `VnApp::state`/`command` register, and
`open_overlay`/`close_overlay`/`confirm`/`notify` control overlays and notifications
directly, `rollback` holds the history, and `hooks` holds the [hooks](#hooks).
`reload_story(story)` swaps in a reloaded story (see [Hot reload](#hot-reload)). `settings` starts in memory
(`SettingsStore::in_memory()`, never written); use `SettingsStore::load(path)` to keep
them in a file. `close_confirmation` is the message for `request_close()`. `audio` starts silent
(`Audio::silent()`); set `Audio::new(assets, AudioConfig::default())` to hear the story.
`draw` needs the thread to take [thumbnails](#thumbnails), and `autosave()` writes the
auto slot (what `VnApp` does on quit). Textures and fonts live on the GPU, so the
manager must be created after the window, and dropped before it (declare it after `rl`).

## Resources

`ResourceManager` resolves every path relative to the asset root passed to
`ScreenStateManager::new`.

### Textures

`get_or_load(path, rl, thread)` loads a texture once and caches it by path. A missing or
unreadable file logs a warning and is replaced by a generated placeholder: a
300×500 card labeled with the path, with a color derived from the path so each
character/expression keeps the same color between runs, or, for a background, a dark
1280×720 image labeled with the path. A story can be played before its art exists.

`texture(path)` returns a loaded texture without loading anything (it works from `draw`),
and remembers the paths it didn't find; the manager loads those at the start of the next
update (`load_requested`). Button images and icons load this way.

Story art lives at `character_path(character, image)` (`characters/<id>/<image>.png`) and
`background_path(image)` (`backgrounds/<image>.png`), relative to the assets.

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
  (U+2010–U+2027: dashes, curly quotes, ellipsis), € and the arrows ← ↑ → ↓. Other
  characters draw as `?`.
- Fonts are loaded through raylib's C `LoadFontFromMemory` rather than raylib-rs'
  wrapper. The wrapper passes the glyph string's byte length as the codepoint count,
  which reads out of bounds for non-ASCII glyphs.

## Tests

```sh
cargo test -p vn_engine
```

The tests run without a window; drawing and input are checked by playing the example.

| File | Covers |
| --- | --- |
| `tests/app.rs` | `VnApp::check` (validation, missing art, story directories, errors with their file), entry scene, typed command arguments, schema export, hook registration |
| `tests/saves.rs` | Save/load round trips, file format and errors, all-or-nothing loads, edited or missing scenes, state added or removed, stored rollback history, thumbnails (scaled, replaced, deleted with the slot), autosave switch, save directory names |
| `tests/button.rs` | Transforms (round trips, rotated and skewed hit tests), state blending and defaults, image switching and listed paths, transition steps, per-button HUD and choice styles, disabled menu items and `can_continue` |
| `tests/audio.rs` | Finding audio files by extension, fades, the music a silent `Audio` tracks, `AudioConfig` |
| `tests/rollback.rs` | Back/forward, barriers (`commit`, final choices, blocked commands, `through_choices`), history limits, history across a save and load |
| `tests/settings.rs` | Settings files, the typewriter, text speeds, slider positions and arrow-key steps for each row, slider math, tooltip timing, when closing the window asks |
| `tests/layout.rs` | Every layout arrangement, anchor and alignment, fitting, and the default screens' positions |
| `tests/hot_reload.rs` | The file watcher, swapping in a reloaded story, the story loader, the error panel's lines |
