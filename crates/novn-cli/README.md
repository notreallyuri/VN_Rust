# novn-cli

The `novn` command-line tool for working with `.story` files: check them, see what they compile
to, format them, extract translations, start a project, and serve a language server.

```sh
cargo install novn-cli
```

One binary comes out, called `novn`.

**The guide is at <https://notreallyuri.github.io/novn/docs/cli>**, in three pages: the commands,
[translating a game](https://notreallyuri.github.io/novn/docs/cli/translate), and
[editors and the language server](https://notreallyuri.github.io/novn/docs/cli/editors).

## The commands

| Command | Does |
| --- | --- |
| `novn check <path> [--schema <file>] [--lang <code>]` | Validates a story against the game's `schema.json`: syntax, registries, command arguments, the entry scene, and translation counts. Fails the exit code on errors, so it belongs in CI |
| `novn dump <file.story \| dir>` | Compiles and prints the program, every instruction with its absolute index under scene headers |
| `novn fmt <path> [--check]` | Formats in place. Only rewrites a file that compiles to the same program, so it cannot change what a story does |
| `novn translate <lang> [path] [--export \| --import <file.po>]` | Extracts translatable strings into `lang/<lang>.json`, and moves them to and from a `.po` for Poedit, Weblate, Crowdin or OmegaT |
| `novn new <dir> [--title <t>] [--engine-path <dir> \| --engine-git <url>]` | Writes a project that runs with `cargo run` |
| `novn lsp` | A language server over stdin and stdout, for any editor that speaks LSP |

Running from inside this repository, without installing:

```sh
cargo run -p novn-cli -- check examples/god_is_watching/assets
cargo run -p novn-cli -- dump examples/god_is_watching/assets/story
```

`run` is planned; see [`TODO.md`](../../TODO.md).

## Tests

```sh
cargo test -p novn-cli
```

One file per command, each running the real binary against temporary projects:

| File | Covers |
| --- | --- |
| `tests/check.rs` | Valid projects from each kind of path, registry and syntax errors with their files, one file checked inside its project, the entry scene, files outside the story directory, no schema, `--schema`, unreadable inputs, and `dump` on a directory |
| `tests/new.rs` | The files it writes passing `novn check`, the package name and engine path, `--title` and `--engine-git` including quotes in the title, non-empty directories, names that are not crate names, and bad arguments |
| `tests/translate.rs` | The catalogue it writes and where, a second run changing nothing, an edited line going stale while the rest keeps its translation, a story with errors, a language tag that is not one, and a project with no schema |
| `tests/lsp.rs` | The server over pipes, the way an editor drives it: diagnostics following unsaved edits and clearing again, a scene defined in an unsaved file, completion in every context, definition across files, hover, symbols, a file with no project, unknown requests, and shutdown |
