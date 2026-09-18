# vn_cli

The `vn` command-line tool for working with `.story` files.

## Commands

### `vn dump <file.story | directory>`

Compiles a file, or every `.story` file under a directory (the way the engine loads a
story), and prints its program: file, scene and instruction counts, diagnostics that
don't need a registry (syntax errors, duplicate scenes, `jump`s to unknown scenes), then
every instruction with its absolute index, grouped under scene headers that show where
the scene is defined. Exits with a
failure code if there are errors.

```sh
cargo run -p vn_cli -- dump examples/god_is_watching/assets/story
```

```text
3 files, 28 scenes, 434 instructions

scene mary_breakfast:  (examples/god_is_watching/assets/story/01_mary.story:47)
034: SAY [NARRATOR]: "Two days later, ..."
036: SHOW hugo
037: CHOICE ['Ask about the candles'->38, 'Ask about the letters'->43]
038: ADD curiosity +1
042: GOTO 49
```

Planned: `check`, `new`, `run` (see TODO.md).
