# example_vn

A small work-in-progress example: one scene, three speakers, and Kaede backed by a
Live2D model when the `live2d` feature is on.

```sh
cargo run -p example_vn                     # PNG path, no SDK needed
cargo run -p example_vn --features live2d   # Kaede as a Cubism model
```

## Kaede

`kaede` is registered twice: as an ordinary `Character` with the appearances
`neutral`, `cheerful` and `annoyed`, and, behind the `live2d` feature, as a
`Live2dCharacter` whose appearances start different idle motions on the model. The
story says `show kaede annoyed` either way; without the feature the engine looks for
`assets/characters/kaede/annoyed.png`, and with it the model plays that motion.

The model is **not in this repository**. Live2D's sample models are not
redistributable, so `assets/characters/kaede/` is git-ignored and you copy one in:

```sh
export CUBISM_SDK_ROOT=/path/to/CubismSdkForNative-5-r.5
cp -r "$CUBISM_SDK_ROOT/Samples/Resources/Hiyori" \
      examples/example_vn/assets/characters/kaede
```

Hiyori is one of Live2D's sample models, so it is for development only: a release
build refuses to load it (see `crates/novn-live2d/README.md`), and `build.rs` keeps
`characters/kaede/` out of the embedded assets. Ship a model you hold the rights to.

Hiyori has no expressions, only nine idle motions, so the three appearances pick
motions 0, 4 and 7. A model with named expressions (Natori has Angry, Smile, Sad and
others) would map onto `Appearance::new().expression("Smile")` instead. `size(w, h)`
sets how large the model is drawn; without it the model's own canvas size is used.

Building with `live2d` needs the Cubism SDK, nasm-free but CMake- and GLEW-dependent;
see `crates/novn-live2d/README.md`. Without the feature, and without PNGs for those
appearances, the engine draws its placeholder and logs a warning.

## Still missing

Backgrounds, music, the rest of the story, and art for the other two speakers. The
name prompt (`call remember_name player_name`) writes into the `player_name`
variable, which the story interpolates as a speaker.
