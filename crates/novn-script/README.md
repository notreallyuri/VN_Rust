# novn-script

The story language: lexer, parser, compiler and VM. It has no rendering dependencies, so
[`novn`](../novn/README.md), the [`novn` CLI](../novn-cli/README.md), the language server and
the formatter all share one implementation and cannot disagree about what a story means.

**The guide is in two pages.**
[Inside the compiler](https://notreallyuri.github.io/novn/docs/crates/script) is the pipeline,
the `Program`, values and conditions, error recovery and the schema.
[Driving the VM](https://notreallyuri.github.io/novn/docs/crates/script/vm) is the events, the
API, snapshots and translation at runtime, which is what you want if you are writing a frontend
of your own.

The language itself, for someone writing a story, is at
<https://notreallyuri.github.io/novn/docs/scenes>, with [SCRIPT.md](../../SCRIPT.md) as the
cheat sheet.

## The pipeline

```text
source ──tokenize──▶ Vec<Token> ──parse_program──▶ Vec<Node::Scene> ──Compiler::compile──▶ Program ──▶ StoryVm
```

```rust
let mut vm = StoryVm::from_dir("assets/story")?;
let diagnostics = vm.prepare(&schema, None);
let event = vm.advance_until_blocking();
```

`compile_source(&str) -> Program` runs the whole pipeline on one source and collects every
stage's diagnostics, sorted by line. `compile_sources([(name, source), ...])` does the same for
several files compiled into one program, each diagnostic carrying its file name.

## Source layout

| Path | Holds |
| --- | --- |
| `lexer.rs` | Tokens, indentation, string scanning |
| `parser/` | `blocks.rs` builds scenes from the indented token stream, `statement.rs` turns one line into a `Node`, `parts.rs` parses the pieces a line is made of |
| `condition.rs` | `if` conditions, shared by the parser and the schema |
| `compiler.rs` | `Node`s to a flat `Program` with jump targets |
| `vm/` | `mod.rs` is `StoryVm`, `build.rs` loads and validates, `step.rs` is the interpreter, `state.rs` the variables and scene accessors, `snapshot.rs` save and restore, `event.rs` and `error.rs` the types they trade in |
| `types/` | `Instruction`, `Node`, `Token`, `Value` and friends |
| `schema.rs` | The registry a game exports for `novn check` and the LSP |
| `translate.rs` | Extracting translatable strings, and the catalogue the VM reads them back from |
| `markup.rs`, `template.rs`, `format.rs`, `suggest.rs`, `files.rs`, `diagnostics.rs` | Text tags, `{variable}` interpolation, `novn fmt`, "did you mean", story file discovery, diagnostics |

Two invariants worth knowing before changing anything here. **Nothing in the lexer or parser
panics**: every error becomes a `Diagnostic` and parsing continues, so one broken line cannot
hide the rest of the file. And **a language addition must not move scene fingerprints**, because
a fingerprint is what decides whether a save resumes exactly or restarts the scene. Anything
optional is skipped when absent for that reason.

## Tests

```sh
cargo test -p novn-script
```

| File | Covers |
| --- | --- |
| `tests/lexer.rs` | Every token kind, comment and blank-line skipping |
| `tests/parser.rs` | Conditions (every operator, precedence, enum and string literals, operators inside strings), `set` and `add`, interpolated speakers, `choice final:`, the fixture compiling, a scene's mode, and an option's condition, reason and pictures |
| `tests/diagnostics.rs` | Every parse error with its exact line and message, recovery, string escapes, file names in multi-file diagnostics |
| `tests/suggest.rs` | Edit distance, `closest` limits, keyword hints, suggestions for unknown scenes, images and entry scenes |
| `tests/template.rs` | Interpolation of every value type, unset variables, `{{` and `}}`, malformed braces |
| `tests/snapshot.rs` | Snapshot round trips, edits to other scenes, edits to the saved scene, missing scenes |
| `tests/schema.rs` | Validation of every registry, line numbers inside branches, defaults, typed `set_variable`, entry scene, old saves with new variables, `prepare`, and schema files |
| `tests/vm.rs` | Scene entry, `current()`, jumps, choice branches, end of story, reset, `start_at`, the loop guard, condition evaluation, `set` and `add`, interpolation, music and sound, the example playing through all three chapters, `story_files`, gated options and scene modes |
| `tests/translate.rs` | What gets extracted and with which kind, how entries are keyed, file names normalised across loaders, an edited line going stale while the rest is kept, a renamed character, JSON round trips, and the VM reading lines out of a catalogue |
| `tests/spec.rs` | Every story example in the documentation, described below |
| `tests/fixtures/all_features.story` | Golden input covering every construct in the language |

### The documentation is a test suite

`tests/spec.rs` reads `SCRIPT.md` and every `page.mdx` under `docs/src/app/docs/`, pulls out
each fenced `story` block, and requires that it compiles without diagnostics. It then records
the compiled instructions and the events the VM produces, taking the first option of every
choice, into `tests/golden/examples.txt`.

Blocks containing `<placeholders>` are syntax templates and skipped. A bare fragment is wrapped
in a scene, and a `...` line becomes narration.

So an example on the website cannot drift from the language: change the compiler and the golden
file moves, write a broken snippet and the build fails naming the page, the line and the
heading. A separate test asserts the pages are actually being reached, so the extractor cannot
quietly stop finding them.

When a change is intended, regenerate and review the diff:

```sh
UPDATE_GOLDEN=1 cargo test -p novn-script --test spec
```
