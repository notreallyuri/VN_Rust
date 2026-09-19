# vn_cli

The `vn` command-line tool for working with `.story` files.

## Commands

### `vn check <path> [--schema <schema.json>]`

Checks stories without running the game: syntax, and, with a schema, the registries
(variables, characters, images, command arguments) and the entry scene. Prints every
diagnostic as `file:line: error: ...`, then a summary, and exits with a failure code if
there are errors (usable in CI and editor "run on save" setups).

```sh
cargo run -p vn_cli -- check examples/god_is_watching/assets
```

```text
examples/god_is_watching/assets/story/04_santa_ilde.story:12: error: unknown variable 'trsut'
examples/god_is_watching/assets/story/02_notebook.story:30: error: speaker: unknown character 'francs'; did you mean 'francis'?
6 files, 30 scenes checked against examples/god_is_watching/assets/schema.json: 2 errors, 0 warnings
```

A misspelled keyword or a name close to a registered one ends with a "did you mean"
suggestion (see vn_script's README, "Schema and validation").

The schema is the `schema.json` a game exports (see vn_engine's README, "Schema export").
What gets checked depends on `path`:

| `path` | Checks |
| --- | --- |
| A `schema.json`, or the directory holding it | The whole story (the schema's `story_dir`) |
| The story directory | The whole story |
| A file or subdirectory inside the story directory | The whole story (so jumps into other files resolve), showing only the diagnostics for `path` |
| A story file or directory outside it | Just that, against the schema's registries, without the entry scene |

Without `--schema`, `vn check` looks for `schema.json` in `path`'s directory and each of
its parents. With no schema anywhere it checks syntax and jump targets only, and says so.

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
6 files, 30 scenes, 503 instructions

scene box_the_letter:  (examples/god_is_watching/assets/story/01_box_14.story:24)
055: BACKGROUND archive_office
056: MUSIC archive
059: CHOICE ['Break the seal'->60, 'Leave it sealed'->68]
060: SET read_letter = true
061: SOUND page_turn
062: CALL give_item verlaine_letter
067: GOTO 70
```

Planned: `new`, `run`, `lsp`, `fmt` (see TODO.md).

## Tests

```sh
cargo test -p vn_cli
```

`tests/check.rs` runs the `vn` binary on temporary projects: valid projects from each
kind of path, registry and syntax errors with their files, one file checked inside its
project, the schema's entry scene, files outside the story directory, no schema,
`--schema`, unreadable inputs, and `dump` on a directory.
