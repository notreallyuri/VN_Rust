# novn-live2d

Backs a character with a Live2D Cubism model instead of a PNG, through
[`novn`](../novn/README.md)'s character-visuals seam. Register one with
`VnApp::character_visual`, its appearances join the schema so `show mary happy` validates, and
every failure falls back to the PNG character art.

**Experimental.** The story path has been played end to end against the proprietary Core, with
two models on screen, saves, rollback, hot reload and shader passes over a model, but most of
that was watched by a person rather than checked by a test. Working and unguarded.

The dependency runs one way: this crate uses `novn`, never the reverse. The seam lives behind
`novn`'s `character-visuals` feature, which is off by default, and this crate's `engine` feature
turns it on.

**The guide is at <https://notreallyuri.github.io/novn/docs/crates/live2d>**: setting up the
SDK, both probes, picking a model that exercises the awkward paths, and how the bridge and the
renderer work. What the seam looks like from a game's side is at
<https://notreallyuri.github.io/novn/docs/engine/visuals>.

## Features

| Feature | Brings in |
| --- | --- |
| *(none)* | The asset loader only, which resolves and preflights a `.model3.json` without any SDK |
| `native` | The C ABI bridge to the official C++ Framework and the proprietary Core. Needs the SDK |
| `engine` | `native`, plus `novn/character-visuals`, which is the seam a game registers through |

## Building against the SDK

The target is Linux x86_64, OpenGL 3.3, **Cubism SDK for Native 5-r.5**. Other platforms and
SDK releases are refused rather than attempted, because nothing else has been validated.

Download the SDK from [Live2D's Native SDK page](https://www.live2d.com/en/sdk/download/native/),
install CMake, a C++17 compiler and the OpenGL and GLEW development libraries, then point
`CUBISM_SDK_ROOT` at the directory holding `Core/`, `Framework/` and the standard shaders.

```sh
export CUBISM_SDK_ROOT=/absolute/path/to/CubismSdkForNative-5-r.5
cargo run -p novn-live2d --features engine --example story
```

The build script makes no network requests, and neither installs the SDK nor accepts anything
on your behalf.

`examples/story.rs` plays a scene through the seam a game uses and needs no input. There is
also `examples/viewer.rs`, a bare model viewer. Both are developer probes rather than a
game-facing API; the guide has their flags.

## Licensing

Two things here are not covered by this crate's licence.

**Live2D's sample models** (Haru, Hiyori, Mao, Mark, Natori, Ren, Rice, Wanko) come under the
Free Material License, which forbids redistributing them. `ModelAssets::load` recognises all
eight by the fingerprint of their `.moc3`, whatever they have been renamed to. A debug build
warns and carries on, so they stay useful while you develop; a release build refuses to load
them, so they cannot leave inside a shipped game. `is_sample_model` is the same check if you
want it yourself. Use models you hold the rights to for anything you publish.

**The SDK** is a different matter. Core links statically and the Framework compiles in, so
players install nothing, and Core's static library is on Live2D's redistributable list. A Cubism
SDK Release License is required only for businesses with annual gross revenue of 10M JPY or
more. Nothing from the SDK is vendored in this repository. See `LICENSE.md` in the SDK, the
[Framework license](https://github.com/Live2D/CubismNativeFramework/blob/5-r.5/LICENSE.md) and
[Live2D's release licensing](https://www.live2d.com/en/sdk/license/).

Redistributing a binary still requires the applicable SDK notices and terms.

## Tests

Without Core:

```sh
cargo test -p novn-live2d
cargo clippy -p novn-live2d --all-targets -- -D warnings
```

`tests/gpu.rs` draws a model and compares it against a baseline. The sample models cannot be
redistributed, so there is no committed reference: the first run writes one under
`target/live2d-reference/` and later runs compare against it. It needs the SDK, a model and a
display, so it is `#[ignore]`d, and `VN_BLESS=1` rewrites the baseline.

```sh
xvfb-run -a cargo test -p novn-live2d --features engine --test gpu -- --ignored
```

The standalone graphics test covers the GL compatibility layer only, not Cubism. It needs GLFW
development libraries and a display:

```sh
c++ -std=c++17 -Wall -Wextra -Werror -Icrates/novn-live2d/native \
  crates/novn-live2d/native/core_profile.cpp \
  crates/novn-live2d/native/tests/core_profile.cpp \
  -lGLEW -lglfw -lGL -o /tmp/vn-live2d-core-profile-test
xvfb-run -a /tmp/vn-live2d-core-profile-test
```

`native/tools/moc_flags.c` reads a `.moc3` and reports which rendering paths it exercises,
which is how the model for the blend-mode checks was picked. It needs Core alone, no window and
no Framework; the guide has the command.

## Where the history went

The validation log and the milestone list that used to live here are in
[`TODO.md`](../../TODO.md), under *Animated character puppets*, which is where the rest of the
project's state is tracked.
