# vn_live2d

Experimental first milestone for Live2D Cubism integration: a native adapter, a
raylib viewer, and the engine seam it registers through. A character is backed by a
model with `VnApp::character_visual`, its appearances join the schema so `show mary
happy` validates, `New Game` and hot reload drop the models, and loading a save or
rolling back restarts the ones still on screen. The C++ bridge compiles and renders
against the proprietary Core, and a scene has now been played through the story path
with two models on screen at once. Every failure falls back to the PNG character art.

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
- **The story path itself has now been run behind Core**, by `examples/story.rs`: a
  scene that shows a model, changes its appearance, brings a second character up beside
  it, pushes parameters, saves, loads, and removes one again — in auto mode, so it plays
  itself. What it settled, all on the same Mesa 26.2 / AMD machine:
  - Two models draw at once, each running its own motion, and a third is loaded and
    dropped as the first character changes appearance. No leak of GL state between them
    and no fallback.
  - `ctx.visual_parameter` reaches a live model over a running motion, every frame:
    the probe holds Hiyori's eyes shut and drives her mouth from an `on_frame` hook,
    and `None` hands both back to the motion on the next frame. The bridge applies
    stored overrides after the motion and expression update and before physics and
    pose, which is what makes that work.
  - Loading a save restarts the models on screen: they keep drawing, with the
    appearance's motion started again, and neither the moc nor the textures are
    reloaded. An earlier version of the probe looped the load and did it four times
    over, which the models survived.
  - Failure behind Core behaves as designed: `--break` truncates kaede's `.moc3`, Core
    rejects it (`Failed to CubismMoc::Create()`), and the engine prints one line and
    draws her PNG while the other character's model keeps running.
  - Toggling fullscreen mid-scene resized the window from 1280×720 to 1920×1080 with
    both models still correct, so a resize with live models is no longer unverified.
  - Mao draws correctly, which is the model that exercises the awkward paths: 15
    additive drawables, 8 multiplicative and 10 inverted masks, out of 262. The glow
    beside its hat is one of the additive parts and composites as a glow, not a box, and
    nothing is clipped to the wrong side of a mask. `native/tools/moc_flags.c` is how
    that model was picked; of the SDK's samples only Mao, Rice and Ren use inverted
    masks at all, and Haru, Hiyori, Mark and Wanko exercise none of it.
  - Shader passes run over the models: with grain and desaturation on, the whole
    composited frame carries them, models included, and the models themselves still draw
    correctly underneath. The screenshot that shows this is one the engine had to be
    taught to take — see below.
  - A push survives a save. The probe now saves while it is holding the eyes shut,
    releases them, and loads: the save file carries
    `"visuals":{"one":{"ParamEyeLOpen":0.0,...}}`, and the frame after the load has the
    eyes shut again with nothing pushing them. That is `character-visuals` state going
    through the save and coming back onto a live model.
  - Motion and physics hold up over a long run. `--watch=90` left Mao animating for ten
    minutes after the scene ended, sampling a frame every ninety seconds: still moving,
    still correctly drawn, no drift into a broken pose, and not one warning in the log.
  - Hot reload works with models on screen. Editing a line of the story while two models
    were up dropped both and loaded them again, and the scene restarted, which is what
    the engine documents for a reload.
  - Expressions work, checked with Natori, whose `.exp3.json` files are named: each
    appearance carries its own (`Normal` and `Smile` here), the bridge applies it at
    load beside the motion, and `restart` puts it back after a load.
  - Rolling back with Page Up restores the appearance a line was shown with — the model
    goes back to the earlier appearance's expression and motion — and a rollback past a
    `remove` brings that character's model back, loading it again. No fallback, no GL
    error. Two things to know when watching it: auto mode advances again the moment a
    rollback lands, so the probe turns auto off at the end of the scene, and a save
    written with `Saves::save` carries neither rollback history nor pushed parameters, so
    it is `ctx.save(slot)` a game wants — loading the bare kind leaves only the steps
    played after it.
  - A 64-byte corruption in the middle of a `.moc3` loaded without complaint. The
    Framework's consistency check is enabled (`LoadModel(..., true)`), but it validates
    structure, not content, so it is not an integrity check for untrusted files.
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
- **Still unverified:** nothing from the first milestone's list is left untried, but
  nothing on it is checked automatically either. Both probes are watched by a person, and
  there are no GPU regression tests, so none of this would catch a regression tomorrow.
- One engine change came out of this: `ctx.screenshot()` used to photograph the render
  target from inside it, so a screenshot never showed a shader pass, the screen shake or
  a flash. It now runs after the frame is composited, which is both what a player expects
  from the S key and what made the shader-pass check above possible. Save thumbnails
  still come from the render target.

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

The story probe plays a scene instead, through the engine seam a game uses, and needs no
input: it turns auto mode on and plays itself.

```sh
cargo run -p vn_live2d --features engine --example story
```

It builds a temporary assets folder, symlinks a model folder into it as two characters,
and plays a scene that shows one, changes her appearance, brings the second up beside
her, pushes parameters from a frame hook and releases them, saves, loads, and removes
one. It takes screenshots as it goes, into its own saves folder, and prints each step.
Without arguments it uses `example_vn`'s Hiyori; pass a model folder to use another — it
reads that model's `.model3.json` for its motion count and expression names, so a model
with expressions gets one per appearance and a model without falls back to motions alone:

```sh
cargo run -p vn_live2d --features engine --example story -- \
  "$CUBISM_SDK_ROOT/Samples/Resources/Natori"
```

`--break` truncates that model's `.moc3` in the copy, so Core rejects it and the run
shows the PNG fallback beside a model that still works. `--post` leaves a grain and a
desaturate pass on for the whole scene. `--watch=<seconds>` keeps screenshotting that
often once the scene has finished, for watching motion and physics over a long run.
`--smoke` quits at the end.

A model's `.moc3` says which rendering paths it actually exercises, which is how Mao was
picked for the blend-mode and inverted-mask check. The tool needs Core alone — no window,
no Framework:

```sh
cc -std=c11 -O1 -I"$CUBISM_SDK_ROOT/Core/include" \
  crates/vn_live2d/native/tools/moc_flags.c \
  "$CUBISM_SDK_ROOT/Core/lib/linux/x86_64/libLive2DCubismCore.a" -o /tmp/moc-flags -lm
/tmp/moc-flags "$CUBISM_SDK_ROOT"/Samples/Resources/*/*.moc3
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

1. ~~Continue the first real run.~~ Done: motions and expressions, multiple models at
   once, repeated loading and destruction, resize, failed loads and the PNG fallback,
   screenshots, rollback, blend modes and inverted masks, shader passes over a model,
   hot reload with models on screen, and the story path end to end (see the validation
   list above). What is left of it is automation: every one of those was watched by a
   person, so the next step is model-dependent GPU regression checks, now that both
   probes write comparable frames and `moc_flags` says which model exercises which
   path.
2. ~~Prove the seam carries a second backend.~~ Done: the engine's puppet backend draws
   through the same `CharacterVisualFactory`, and the one thing it wanted that the seam
   lacked is now there — `CharacterVisual::set_parameter`, with `ctx.visual_parameter`
   and an `on_frame` hook behind it, which is the channel voice-driven lip sync needs.
   A push is remembered per character, so it follows the character into the next
   appearance it loads. Live2D implements it against `Model::set_parameter`, and that
   path has now been run against Core. What is still missing is lip sync itself: the
   engine has no channel from a playing voice line to a parameter, so a game has to
   compute the value it pushes.
3. ~~Replace the reset-everything rollback.~~ Done: `CharacterVisual::restart` returns a
   model to its appearance's preset (expression, motion, parameters) without touching
   the moc, textures or physics. Rollback and loading a save restart the models that
   stay on screen instead of dropping them; New Game and hot reload still drop
   everything, since the assets themselves may have changed. A model that fails to
   restart falls back to its PNG, like any other failure. The default implementation
   does nothing, so a backend without transient state need not implement it. Run against
   Core through a save and load; rollback by key shares the same call.
4. ~~Save logical appearance/parameter state and restore it on load and rollback.~~
   Done: appearance was already story state, and a pushed parameter is now recorded with
   the line it was pushed on, so a save carries it and a rollback puts back the values
   that line was shown with. Transient state is still restarted rather than restored —
   idle animation, motions and physics begin again from the appearance's preset and the
   push is applied over the top. Pause, skip, hot reload and custom screens are written
   down in [the engine's README](../vn_engine/README.md#while-the-story-is-not-playing);
   a custom screen can show a backend-drawn character now too: screens ask for the
   appearances they want each frame and the manager prepares that set. What this seam is
   still short of is a channel from a playing voice line to a parameter.
