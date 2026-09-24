# vn_live2d

Experimental first milestone for Live2D Cubism integration: a native adapter, a
raylib viewer, and the engine seam it registers through. A character is backed by a
model with `VnApp::character_visual`, its appearances join the schema so `show mary
happy` validates, `New Game` and hot reload drop the models, and loading a save or
rolling back restarts the ones still on screen. The C++ bridge now compiles and
renders against the proprietary Core, though the story-facing path has still only
been exercised through the viewer. Every failure falls back to the PNG character art.

The dependency runs one way: `vn_live2d` uses `vn_engine`, never the reverse. The
seam lives behind `vn_engine`'s `character-visuals` feature, which is off by
default, and this crate's `engine` feature turns it on. That seam now carries a second
backend, the engine's own [puppet](../vn_engine/README.md#character-visuals) — flat parts
moved by parameters, no SDK — which is what settled its shape.

The asset loader works without the Cubism SDK. The `native` feature enables a C ABI
bridge to the official C++ Framework and proprietary Core. This avoids porting
Cubism's motion blending, expressions, and physics to Rust.

## Current validation

- Workspace tests and SDK-independent asset tests pass.
- The engine seam compiles, lints and tests with `--features character-visuals`;
  that is not part of a default workspace build, so check it explicitly:
  `cargo test -p vn_engine --features character-visuals`, which now includes
  `tests/visuals.rs` over the seam's startup path.
- The bridge compiles against Cubism SDK for Native 5-r.5 (Core 6.0.1) and the viewer
  renders Haru, Mao, Hiyori and Natori fully textured, with clipping masks, on Mesa
  26.2 / AMD, OpenGL 4.6 core. No GL errors, and the host's red and green markers
  still bracket the model, so the state save/restore holds.
- Three faults only a real run could show, all fixed here: `SetIsPremultipliedAlpha`
  does not exist in 5-r.5 (the setter is an `IsPremultipliedAlpha` overload); the
  shader loader signature takes `csmSizeInt*`, not `int*`; and the Framework asks for
  `GL_LINEAR_MIPMAP_LINEAR` when it draws, so model textures without mipmaps are
  incomplete and sample as pure black. Textures are now mipmapped and filtered
  trilinearly, as the SDK's own sample does.
- The embedded shaders are ported from GLSL 1.20 to 3.30 core as they are copied.
  Mesa accepts the originals in a core context, but they are not legal there, and a
  stricter driver would leave the host's shader bound and draw a black silhouette.
- The seam carries a second backend: `vn_engine`'s puppet. Its probe was watched through
  a whole scene on Mesa 26.2 / AMD — five parts in order, an appearance swapping one of
  them, a parameter pushed from a frame hook and released — with no fallback to the PNGs.
  `CharacterVisual` needed one addition to take it, `set_parameter`, and no change to the
  rest, so the abstraction is no longer a hypothesis about a single backend.
- The SDK-independent GPU test passes with a real OpenGL 3.3 core context under
  Xvfb: drawing through VBOs, restoring host GL state (including on exceptions),
  and recreating the compatibility resources.
- **Still unverified:** motions, expressions and physics in motion (only the first
  frames have been watched), resize, failed loads, repeated destruction, several
  models at once, post-processing, and the story-facing seam behind Core — the puppet
  has now played a scene through it, a Cubism model has not, and neither has
  `set_parameter` against a real model. Nothing is checked automatically: there are no
  GPU regression tests yet.

## SDK setup

The initial target is **Linux x86_64, OpenGL 3.3, Cubism SDK for Native 5-r.5**.
Other platforms and SDK releases are deliberately rejected until validated.

Download and extract the SDK from the [official Native SDK page](https://www.live2d.com/en/sdk/download/native/).
Point `CUBISM_SDK_ROOT` at the directory containing:

```text
Core/include/Live2DCubismCore.h
Core/lib/linux/x86_64/libLive2DCubismCore.a
Framework/CMakeLists.txt
Framework/src/Rendering/OpenGL/Shaders/Standard/
```

Install CMake, a C++17 compiler, OpenGL development libraries, and GLEW development
libraries. The build script makes no network requests and does not install or
accept terms for the SDK. Core, Framework, and model assets retain their own
licenses; none is vendored here. See the [Framework license](https://github.com/Live2D/CubismNativeFramework/blob/5-r.5/LICENSE.md)
and [Live2D release licensing](https://www.live2d.com/en/sdk/license/).

```sh
export CUBISM_SDK_ROOT=/absolute/path/to/CubismSdkForNative-5-r.5
cargo run -p vn_live2d --features native --example viewer -- \
  "$CUBISM_SDK_ROOT/Samples/Resources" Haru/Haru.model3.json
```

The viewer draws into the engine's `RenderTarget`, then composites it with raylib.
It draws a red square before Cubism and a green square afterward. Resize the
window; press M for motions, E for expressions, Space to pause, and R to destroy
and reload the model. An optional final integer limits the number of frames for
smoke testing, and `VN_SHOT=/path/frame.png` writes frame 120 to a file, which is how
the renders above were checked. This is a developer probe, not the final game-facing API.

## Design

- `ModelAssets::load` resolves `.model3.json` references through `Assets`, including
  textures, motions, expressions, physics, and pose files. Folder and embedded
  assets use the same loader. It checks version, names, paths, file availability,
  JSON syntax, and sizes before native loading. Use trusted, SDK-compatible model
  exports; this preflight is not a validator for every Cubism file format.
- `with_cubism` scopes SDK initialization to the lifetime of a raylib window. Models
  borrow that session and cannot escape its closure or move across threads.
  Destruction releases native model resources before textures and the SDK.
- `Model` exposes motion selection, expression selection, persistent parameter
  overrides, restart, update, and drawing. Physics and pose evaluation follow motion and
  expression updates. Animation time is capped at 100 ms per update after a stall.
  Audio referenced by motion files, automatic breathing, and voice-driven lip sync
  are not implemented in this milestone.
- Native exceptions become Rust errors. No C++ object layouts cross the C ABI.
- The Framework's OpenGL renderer uses CPU vertex/index arrays. Its two drawing
  source files are copied into Cargo's build directory and their submission calls
  redirected to a small VBO/VAO compatibility layer. The user's SDK is untouched.
  This is why the Framework version is pinned. This straightforward streaming
  implementation prioritizes correctness; it has not been performance-tuned.
- A draw flushes raylib's batch, uses a separate VAO, then restores framebuffer,
  viewport, shader, buffer, texture-unit, blend, and other modified GL state.
- Framework shader files are embedded at build time; execution does not depend on
  the SDK's location or the process working directory. Redistributing a binary
  still requires the applicable SDK notices and terms.

## Sample models are development-only

Live2D's sample models (Haru, Hiyori, Mao, Mark, Natori, Ren, Rice, Wanko) come under
the Free Material License, which forbids redistributing them. `ModelAssets::load`
recognises all eight by the fingerprint of their `.moc3`, whatever they have been
renamed to. A debug build warns and carries on, so they stay useful while developing;
a release build refuses to load them, so they cannot leave in a shipped game. Use
models you hold the rights to for anything you publish. `is_sample_model` exposes the
same check.

The SDK itself is different: Core links statically into the binary and the Framework
compiles into it, so players install nothing, and Core's static library is on Live2D's
redistributable list. A Cubism SDK Release License is required only for businesses
with annual gross revenue of 10M JPY or more; see `LICENSE.md` in the SDK.

## Tests

Without Core:

```sh
cargo test -p vn_live2d
cargo clippy -p vn_live2d --all-targets -- -D warnings
```

The standalone graphics test additionally needs GLFW development libraries and a
display (or Xvfb). It tests only the compatibility layer, not Cubism itself:

```sh
c++ -std=c++17 -Wall -Wextra -Werror -Icrates/vn_live2d/native \
  crates/vn_live2d/native/core_profile.cpp \
  crates/vn_live2d/native/tests/core_profile.cpp \
  -lGLEW -lglfw -lGL -o /tmp/vn-live2d-core-profile-test
xvfb-run -a /tmp/vn-live2d-core-profile-test
```

## Next milestone

1. Continue the first real run. Four models draw correctly with masks; still to check
   are inverted masks and blending modes, expressions, motions and physics over time,
   multiple models at once, resize, failed loads, repeated destruction, screenshots
   and post-processing. Add model-dependent GPU regression checks, now that `VN_SHOT`
   makes a frame comparable.
2. ~~Prove the seam carries a second backend.~~ Done: the engine's puppet backend draws
   through the same `CharacterVisualFactory`, and the one thing it wanted that the seam
   lacked is now there — `CharacterVisual::set_parameter`, with `ctx.visual_parameter`
   and an `on_frame` hook behind it, which is the channel voice-driven lip sync needs.
   A push is remembered per character, so it follows the character into the next
   appearance it loads. Live2D implements it against `Model::set_parameter`, but that
   path has not been run against Core yet, and nothing here does lip sync: the engine
   has no channel from a playing voice line to a parameter.
3. ~~Replace the reset-everything rollback.~~ Done: `CharacterVisual::restart` returns a
   model to its appearance's preset (expression, motion, parameters) without touching
   the moc, textures or physics. Rollback and loading a save restart the models that
   stay on screen instead of dropping them; New Game and hot reload still drop
   everything, since the assets themselves may have changed. A model that fails to
   restart falls back to its PNG, like any other failure. The default implementation
   does nothing, so a backend without transient state need not implement it. Not yet
   run against Core.
4. Save logical appearance/parameter state and restore it on load and rollback,
   initially restarting transient idle animation and physics. Define pause, skip,
   hot reload, and custom-screen behavior before claiming story integration.
