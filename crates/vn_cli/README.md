# vn_cli

The `vn` command-line tool for working with `.story` files.

## Commands

### `vn check <path> [--schema <schema.json>] [--lang <code>]`

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

#### Translations

When the project has a `lang/` directory, `vn check` ends with a line per catalog,
counted against the story as it is now rather than as it was at the last `vn translate`:

```text
6 files, 30 scenes checked against examples/god_is_watching/assets/schema.json: 0 errors, 0 warnings
pt-BR: 22/364 translated, 342 missing, 0 stale
  run `vn check --lang pt-BR` to list them
```

`--lang <code>` lists them, sorted by file and line, before that summary:

```text
00_archive.story:7: missing narration translation: "October, 1903. The Archive of the House, two floors below the street."
00_archive.story:10: missing dialogue translation: "You're the new one. Sit."
name registrar: missing name translation: "The Registrar"
screens: missing ui translation: "Resume"
```

Missing and stale translations don't fail the check — an untranslated line falls back to
the source text, so a partly translated game still runs. A catalog that can't be read
does fail, as does `--lang <code>` with no `<code>.json` to check.

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
6 files, 30 scenes, 591 instructions

scene box_the_letter:  (examples/god_is_watching/assets/story/01_box_14.story:24)
064: WITH dissolve
065: BACKGROUND archive_office
066: MUSIC archive
069: CHOICE ['Break the seal'->70, 'Leave it sealed'->78]
070: SET read_letter = true
071: SOUND page_turn
072: CALL give_item verlaine_letter
077: GOTO 80
```

### `vn fmt <path> [--check]`

Formats a file, or every `.story` file under a directory, in place. `--check` writes
nothing and only lists what would change, exiting with a failure code if anything would
(for CI and pre-commit hooks).

```sh
cargo run -p vn_cli -- fmt examples/god_is_watching/assets/story --check
```

```text
6 files, 0 to reformat
```

The rules:

| | |
| --- | --- |
| Indentation | Two spaces per level, taken from the nesting the file already has, not from how deep each line is written |
| Spacing | One space between words; `=`, `+=`, `-=` and the comparison operators in `if`, `set` and `add` get a space on each side |
| Comments | Kept as written and indented like the line below them |
| Blank lines | At most one in a row, exactly one before each `scene`, none at the start or end of the file |
| Strings | Never touched, including the spaces, `#`, `:` and `{variable}` inside them |

A file is only rewritten when it compiles and when the result compiles to the same
program, so formatting can't change what a story does; a file with errors is reported
and left alone.

### `vn translate <lang> [path]`

Extracts every translatable string into `lang/<lang>.json` next to the game's
`schema.json`: dialogue, narration, choice options, and the characters' display names.
Run it again whenever the story changes — it keeps the translations already written,
adds entries for new strings, and marks as `stale` the ones whose source has been edited
or deleted.

```sh
cargo run -p vn_cli -- translate pt-BR examples/god_is_watching/assets
```

```text
wrote examples/god_is_watching/assets/lang/pt-BR.json
263 strings (263 new), 0 translated, 263 missing, 0 stale
```

The screens' own labels ("Settings", "Back", "Saved to {slot}") come from `lang/ui.json`,
which the engine writes beside `schema.json` when a game runs in a debug build. They land
in the catalog's `ui` section alongside the story, so a translator has one file to fill.
Without that file `vn translate` says so and extracts the story only.

`path` is the project (a `schema.json`, or any directory at or under it — the same search
as `vn check`); it defaults to the working directory. A story with errors is reported and
nothing is extracted, so a broken edit can't half-rewrite the catalog. The file format,
how entries are keyed, and how a game loads one are in
[vn_script's README](../vn_script/README.md#translation).

### `vn new <directory> [--title <title>] [--engine-path <dir> | --engine-git <url>]`

Creates a game project that runs with `cargo run`:

```sh
cargo run -p vn_cli -- new ../my-novel
```

| File | Contents |
| --- | --- |
| `Cargo.toml` | A package named after the directory (`my-novel` → `my_novel`), depending on `vn_engine` (and `vn_build` for the build script, from the same place), with an empty `[workspace]` so it builds on its own even inside another workspace |
| `src/main.rs` | A `VnApp` with one character (`guide`), two variables and a `give_item` command, and the embedded assets |
| `build.rs` | Embeds `assets/` in release builds with `vn_build`, so `cargo build --release` gives an executable that runs anywhere |
| `assets/story/start.story` | Two scenes using dialogue, a choice, `set`, `if`, `call` and `jump` |
| `assets/schema.json` | The registries of `main.rs`, so `vn check assets` works before the first run (debug builds rewrite it) |
| `assets/{characters,backgrounds,fonts}/` | Empty; missing art is drawn as a placeholder |
| `README.md`, `.gitignore` | |

The window title is `--title`, or the directory name in title case (`My Novel`). The
engine dependency is `--engine-path` (written relative to the project when they share a
parent directory), `--engine-git`, or by default the `vn_engine` next to the `vn_cli` that
was built, falling back to the GitHub repository. The directory may exist, but must be
empty.

### `vn lsp`

A language server for `.story` files over stdin/stdout, for any editor with LSP support.
Install the binary once (`cargo install --path crates/vn_cli`), then point the editor at
`vn lsp` for `*.story` files.

| Feature | Behavior |
| --- | --- |
| Diagnostics | On open, change, save and close, for every file of the project, using the text of unsaved buffers: syntax errors, registries, unknown scenes, "did you mean" hints. The same checks as `vn check` |
| Completion | By context: keywords and characters at the start of a line; scenes after `jump`; characters after `show`/`remove` and their images after `show <character>`; `at`/`with`, positions and transitions; background, music, sound and voice ids from the asset folders (and `none`); variables after `set`, `add` (ints only), `if`, `&&`, `||` and inside `{` in a string; `true`/`false` or enum members after `set x =` and `x ==`; operators; commands after `call`, with their usage |
| Go to definition | On a scene name (in a `jump` or anywhere): the `scene` line, in whichever file |
| Hover | Characters (display name, images), variables (type and default), commands (usage), scenes (file and line) |
| Document symbols | The scenes of the file (an outline) |

The project is found like `vn check` does: the nearest `schema.json` above the file, and
its `story_dir`. A file outside the story directory is checked alone against the
schema's registries; a file with no schema above it is checked with the other `.story`
files in its folder, for syntax and jumps only.

Neovim (0.11+):

```lua
vim.filetype.add({ extension = { story = "story" } })
vim.lsp.config("vn", { cmd = { "vn", "lsp" }, filetypes = { "story" }, root_markers = { "schema.json" } })
vim.lsp.enable("vn")
```

Helix (`languages.toml`):

```toml
[language-server.vn]
command = "vn"
args = ["lsp"]

[[language]]
name = "story"
scope = "source.story"
file-types = ["story"]
roots = ["schema.json"]
language-servers = ["vn"]
```

VS Code and Zed need a small extension to start it (planned, see TODO.md).

Planned: `run` (see TODO.md).

## Tests

```sh
cargo test -p vn_cli
```

`tests/check.rs` runs the `vn` binary on temporary projects: valid projects from each
kind of path, registry and syntax errors with their files, one file checked inside its
project, the schema's entry scene, files outside the story directory, no schema,
`--schema`, unreadable inputs, and `dump` on a directory. `tests/new.rs` runs `vn new`:
the files it writes pass `vn check`, the package name and engine path, `--title` and
`--engine-git` (with quotes in the title), non-empty directories, names that aren't crate
names, and bad arguments.
`tests/translate.rs` runs `vn translate`: the catalog it writes and where, a second run
that changes nothing, an edited line going stale while the rest keeps its translation, a
story with errors, a language tag that isn't one, and a project with no schema.
`tests/lsp.rs` runs `vn lsp` over pipes like an editor: diagnostics following unsaved
edits (and clearing), a scene defined in an unsaved file, completion in every context,
definition across files, hover, symbols, a file with no project, unknown requests, and
shutdown.
