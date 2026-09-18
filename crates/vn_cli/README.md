# vn_cli

The `vn` command-line tool for working with `.story` files.

## Commands

### `vn dump <file.story>`

Compiles a file and prints its program: scene and instruction counts, diagnostics that
don't need a registry (syntax errors, duplicate scenes, `jump`s to unknown scenes), then
every instruction with its absolute index, grouped under scene headers. Exits with a
failure code if there are errors.

```sh
cargo run -p vn_cli -- dump examples/god_is_watching/assets/story/01_mary.story
```

```text
8 scenes, 144 instructions

scene mary_breakfast:
031: SAY [NARRATOR]: "Two days later, ..."
034: CHOICE ['Ask about the candles'->35, 'Ask about the letters'->39]
038: GOTO 44
048: JUMP_SCENE 'mary_the_wind'
049: END
```

Planned: `check`, `new`, `run` (see TODO.md).
