# novn

A raylib-based visual novel engine, built on [`novn-script`](../novn-script/README.md).

It re-exports `raylib` and `novn_script` (as `novn::script`), so a game builds against the
same versions the engine uses and needs only `novn` as a dependency, plus
[`novn-build`](../novn-build/README.md) as a build dependency to ship a release build.

**The guide lives at <https://notreallyuri.github.io/novn/docs/engine/app>.** This file is the
front door: what the crate is, how to start it, what the features do, and how to work on it.
Everything about *using* the engine is on the site, and the table at the bottom says which
page covers what.

## Quick start

A game is a `VnApp` built from defaults, with only the parts it wants changed:

```rust
use novn::prelude::*;
use novn::raylib::prelude::*;
use novn::screens::main_menu::MenuItem;

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

`run()` loads and validates the story, opens the window, loads the fonts and the settings,
and runs the loop until the player closes the window or a screen returns `ScreenState::Quit`.
It returns an `AppError` without opening a window if the story cannot be read or has errors.

`novn new <dir>` writes a working version of the above. See
[`novn-cli`](../novn-cli/README.md).

## Features

All off by default, so a plain build stays small and fast.

| Feature | Brings in |
| --- | --- |
| `character-visuals` | The animated-character seam, plus the built-in puppet rig. What [`novn-live2d`](../novn-live2d/README.md) registers through |
| `video` | Video playback in pure Rust, with assembly paths on |
| `video-portable` | The same, without the assembly, for targets that cannot take it |
| `video-ffmpeg` | Video through FFmpeg instead, for the codecs the portable decoder does not read |

An off-by-default feature takes its targets out of the default build, examples included, so
each one wants checking on its own before a commit. See Tests below.

## Source layout

The crate root holds what everything else hangs off, the app, the screen manager and the
contexts a screen is handed, and the rest is grouped by what it does:

| Path | Holds |
| --- | --- |
| `lib.rs`, `app/`, `screen_manager.rs`, `context.rs`, `screen.rs`, `overlay.rs`, `action.rs` | The loop, and what it hands a screen each frame |
| `ui/` | Drawing. `mod.rs` is the helpers themselves (text, backgrounds, sliders, hit tests), beside `button/`, `shape.rs`, `layout.rs`, `styled.rs`, `fonts.rs`, `ease.rs`, `scroll.rs`, `theme.rs`, `tooltip.rs`, `toast.rs` |
| `input/` | `navigation.rs` (focus, keys, gamepad), `hit.rs` (shapes and picking), `image_map.rs`, `drag.rs` |
| `frame/` | The picture: `target.rs`, `viewport.rs`, `post.rs`, `effects.rs`, `screen_transition.rs`, `scenery.rs`, `stage.rs` |
| `request.rs` | What a screen asks the manager to do once `update` returns |
| `data/` | What persists: `saves/`, `session.rs`, `rollback.rs`, `state.rs`, `settings.rs`, `assets.rs`, `resources.rs` |
| `game/` | What a game registers, and the story's side effects: `characters.rs`, `commands.rs`, `hooks.rs`, `audio.rs`, `hot_reload.rs`, `script_errors.rs`, and behind `character-visuals` both `visuals.rs` (the seam) and `puppet.rs` (the built-in rig) |
| `screens/` | One module per default screen |

Nothing is re-exported at the crate root, so every item is reached at its own path:
`novn::ui::button::ButtonStyle`, `novn::data::saves::Saves`, `novn::frame::post::GRAIN`. Each
group also has a `prelude` for files working inside it, and `novn::prelude` is the short list
a game file wants.

Modules that grew past a few hundred lines are folders whose `mod.rs` wires the parts
together and re-exports them:

| Path | Parts |
| --- | --- |
| `app/` | `builder.rs` is the `VnApp` builder, `check.rs` the schema export and story/art validation, `run.rs` the window and the game loop, `screens.rs` the default `ScreenFactory`, `error.rs` `AppError` |
| `ui/button/` | `style.rs` (`ButtonStyle`, `ButtonLook`), `decor.rs` (borders, shadows, images, icons), `transform.rs` (scale/rotate/skew and its hit test), `look.rs` (the blended `Look`), `anim.rs` (hover and press state), `draw.rs` (`Button`, `draw_button`) |
| `data/saves/` | `store.rs` is `Saves` (slots, files, thumbnails), `file.rs` the `SaveFile` and applying one, `migrate.rs` the version steps, `dirs.rs` where saves live, `error.rs` the errors and warnings |
| `screens/playing/` | `config.rs`, `screen.rs`, `flow.rs`, `typewriter.rs`, `keys.rs`, `style.rs` (the dialogue box), `choice.rs` (option pictures and previews), `nvl.rs` (full-screen pages) |
| `screens/settings/` | `config.rs`, `values.rs`, `layout.rs`, `menu.rs`, `screen.rs` |
| `screens/save_menu/` | `config.rs`, `menu.rs`, `actions.rs`, `screen.rs` |

Each screen's `config.rs` is the builder a game configures; `menu.rs` and `screen.rs` are what
run.

## Tests

```sh
cargo test -p novn
```

The tests run without a window. Drawing and input are checked by playing the example. A few
checks that need a GPU are `#[ignore]`d and run with `cargo test -p novn -- --ignored`.

There is one file per subject under `tests/`, named after it: `saves.rs`, `rollback.rs`,
`navigation.rs`, `stage.rs`, `layout.rs`, and so on for the rest.

Because the optional features take their targets out of the default build, each wants its own
run:

```sh
cargo test -p novn --features character-visuals
cargo clippy -p novn --features character-visuals --all-targets -- -D warnings
cargo clippy -p novn --features video-ffmpeg --example video -- -D warnings
```

### The GPU test

`tests/gpu.rs` compares what a backend actually draws against reference pictures in
`tests/fixtures/puppet/reference/`. It needs a window, so it is `#[ignore]`d, and a virtual
display is enough: the same frame comes out of llvmpipe and of an AMD card, pixel for pixel.

```sh
xvfb-run -a cargo test -p novn --features character-visuals --test gpu -- --ignored
```

Every parameter is held at a value for the shot, so the frame is a function of the rig and its
pictures alone. A run that differs by more than 0.02% of the frame fails and writes what it
drew to the temporary directory, to be looked at beside the reference. For scale, moving one
bound parameter by 0.2 moves 0.17% of the frame.

A change that is meant is blessed with `VN_BLESS=1`, which rewrites the references instead of
comparing them. Look at the diff before you do that.

## The guide

| Page | Covers |
| --- | --- |
| [Building an app](https://notreallyuri.github.io/novn/docs/engine/app) | `VnApp`, the registries, the schema, where things live |
| [The screens you get](https://notreallyuri.github.io/novn/docs/engine/screens) | Start screen, main menu, actions, the playing screen |
| [Menus, settings and overlays](https://notreallyuri.github.io/novn/docs/engine/menus) | Pause menu, dialogs, settings, notifications, controls, log |
| [Story integration](https://notreallyuri.github.io/novn/docs/engine/story) | Commands, story words, hooks, game state, hot reload |
| [Look and feel](https://notreallyuri.github.io/novn/docs/engine/look) | Theme, corners, panels, buttons, layouts |
| [The picture](https://notreallyuri.github.io/novn/docs/engine/picture) | Render target, resolution, shaders, effects, transitions, scenery |
| [Input](https://notreallyuri.github.io/novn/docs/engine/input) | Keyboard and gamepad, prompts, image maps, dragging, tooltips |
| [Audio and video](https://notreallyuri.github.io/novn/docs/engine/media) | Music that follows the screen, voice, cutscenes, codecs |
| [Languages](https://notreallyuri.github.io/novn/docs/engine/languages) | Catalogues, the screens' own labels, what stays in ids |
| [Screens of your own](https://notreallyuri.github.io/novn/docs/engine/custom-screens) | Screens, overlays, text input, what the manager holds |
| [Character visuals](https://notreallyuri.github.io/novn/docs/engine/visuals) | The seam, puppets, parameters, lip sync, writing a backend |
| [Saving and rollback](https://notreallyuri.github.io/novn/docs/engine/saving) | Save files, autosave, thumbnails, rollback, migrations |
| [Assets and fonts](https://notreallyuri.github.io/novn/docs/engine/assets) | Folders or embedded, textures, placeholders, fonts |

For the story language itself, start at
<https://notreallyuri.github.io/novn/docs/scenes>.
