# vn_live2d

Experimental first milestone for Live2D Cubism integration: a native adapter, a
raylib viewer, and the engine seam it registers through. A character is backed by a
model with `VnApp::character_visual`, its appearances join the schema so `show mary
happy` validates, and `New Game`, loading and rollback reset the models. **The C++
bridge has not been compiled or run against the proprietary Core, so none of that
path is verified end to end.** Every failure falls back to the PNG character art.

The dependency runs one way: `vn_live2d` uses `vn_engine`, never the reverse. The
seam lives behind `vn_engine`'s `character-visuals` feature, which is off by
default, and this crate's `engine` feature turns it on.

The asset loader works without the Cubism SDK. The `native` feature enables a C ABI
bridge to the official C++ Framework and proprietary Core. This avoids porting
Cubism's motion blending, expressions, and physics to Rust.

## Current validation

- Workspace tests and SDK-independent asset tests pass.
- The engine seam compiles, lints and tests with `--features character-visuals`;
  that is not part of a default workspace build, so check it explicitly:
  `cargo test -p vn_engine --features character-visuals --lib`.
- Rust native bindings and the viewer have been type-checked without linking Core.
- The SDK-independent GPU test passes with a real OpenGL 3.3 core context under
  Xvfb: drawing through VBOs, restoring host GL state (including on exceptions),
  and recreating the compatibility resources.
- **The complete C++ bridge has not yet been compiled or run against Core.** A
  local SDK is required for that next step. Model rendering, clipping, physics,
  expression playback, and the viewer remain unverified end to end.

## SDK setup

The initial target is **Linux x86_64, OpenGL 3.3, Cubism SDK for Native 5-r.5**.
Other platforms and SDK releases are deliberately rejected until validated.

Download and extract the SDK from the [official Native SDK page](https://www.live2d.com/en/sdk/download/native/).
Point `CUBISM_SDK_ROOT` at the directory containing:

```text
Core/include/Live2DCubismCore.h
Core/include/Live2DCubismCore.hpp
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
smoke testing. This is a developer probe, not the final game-facing API.

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
  overrides, update, and drawing. Physics and pose evaluation follow motion and
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

1. Compile and run the full bridge with the pinned SDK and representative models.
   Check masks, inverted masks, blending, expressions, motions, and physics;
   multiple models; resize; failed loads; repeated destruction; screenshots; and
   post-processing. Add model-dependent GPU regression checks once verified.
2. Prove the seam carries a second backend. It has only ever had this one behind
   it, so `CharacterVisual` is still a hypothesis about the abstraction. It also
   has no channel for per-frame parameters, which is what voice-driven lip sync
   would need.
3. Replace the reset-everything rollback. `flow.rs` drops every instance on each
   rollback step, so stepping back reloads each model's moc, textures, motions and
   physics. Correct, and far too slow to keep.
4. Save logical appearance/parameter state and restore it on load and rollback,
   initially restarting transient idle animation and physics. Define pause, skip,
   hot reload, and custom-screen behavior before claiming story integration.
