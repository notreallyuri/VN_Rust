# vn_script

The story DSL: lexer, parser, compiler and VM. It has no rendering dependencies, so
the engine, the `vn` CLI and future tooling (LSP, formatter) can all share it.

The language itself is specified in [SCRIPT.md](../../SCRIPT.md).

## Pipeline

```text
source ──tokenize──▶ Vec<Token> ──parse_program──▶ Vec<Node::Scene> ──Compiler::compile──▶ Program ──▶ StoryVm
```

`compile_source(&str) -> Program` runs the whole pipeline on one source and collects
every stage's diagnostics into `Program::diagnostics`, sorted by line.
`compile_sources([(name, source), ...]) -> Program` does the same for several files
compiled into **one** program, in the order given; each diagnostic carries its file name.

## Multi-file stories

A story is usually a directory of `.story` files (one per chapter or POV):

- `story_files(dir)` lists every `*.story` file under `dir`, recursively, sorted by
  path. Other files are ignored. Name files so they sort in reading order
  (`00_archive.story`, `01_box_14.story`, ...).
- `read_sources(&paths)` reads them into `(path, source)` pairs for `compile_sources`.
- `StoryVm::from_dir(dir)` does both. `StoryVm::from_file(path)` is the one-file case.

Scene ids are global: `jump` works across files, and the default entry scene is the
first scene of the first file. A scene id defined in two files is an error on the
second one, naming where the first is (`scene 'a' is already defined at a.story:1`).
Saves are unaffected by how scenes are split into files: snapshots store a scene id, an
offset inside it and that scene's fingerprint.

| Stage | Entry point | File |
| --- | --- | --- |
| Lexer | `tokenize(&str) -> (Vec<Token>, Vec<Diagnostic>)` | `src/lexer.rs` |
| Parser | `parse_program(&[Token]) -> (Vec<Stmt>, Vec<Diagnostic>)` (every top-level `scene`) | `src/parser.rs` |
| Conditions | `parse_condition(&str, line) -> Result<Condition, Diagnostic>` | `src/condition.rs` |
| Compiler | `Compiler::new().compile(scenes) -> Program` | `src/compiler.rs` |
| Files | `story_files(dir)`, `read_sources(&paths)` | `src/files.rs` |
| VM | `StoryVm::from_dir` / `from_file` / `from_source` / `from_program` | `src/vm.rs` |
| Interpolation | `interpolate(&str, &variables) -> String` | `src/template.rs` |

## Program

A whole `.story` file compiles into one `Program`:

- `instructions`: one flat list for every scene in every file.
- `locations`: the `Location { file, line }` of each instruction, used by diagnostics.
  `program.line(i)` and `program.file(i)` read it; `file` indexes `files`.
- `files`: source file names, in compile order (empty for `compile_source`).
- `scene_locations`: where each scene is defined.
- `scenes`: scene id → index of the scene's first instruction.
- `scene_order`: scene ids in file order. The first is the default **entry scene**.
- `diagnostics`: problems found while lexing, parsing and compiling.

The parser keeps the source line of every statement (`Stmt { line, node }`), and the
compiler records it, with the current file, in `Program::locations`.
`Compiler::start_file(name)` + `compile_scenes(scenes)` (repeated) + `finish()` is the
multi-file form of `Compiler::new().compile(scenes)`.

Compilation rules:

- Every jump target (`Goto`, `Choice` options, `JumpIfFalse`) is an **absolute** index
  into `instructions`.
- Every scene ends with an `End` instruction. A scene with no `jump` ends the story;
  it does not run into the next scene in the file.
- `jump <scene>` stays a named `Jump` and is resolved at runtime through `scenes`.
  That keeps the current scene known to the VM (for hooks and saves later).
- A duplicate scene id is an error diagnostic; the first definition is kept.
- `Program::unknown_jump_targets()` lists `jump`s to scenes that don't exist
  (validation reports them as errors).
- A choice block compiles to `Choice { options: [(text, start)] }`, then each option's
  body followed by a `Goto` to the end of the block.
- `show <c> <i> at <position>` compiles to `Show { char_id, img_id, position: Some(..) }`;
  without `at`, `position` is `None` and isn't serialized, so scene fingerprints (and
  saves) from before positions existed stay valid. `background <id>` / `background none`
  compile to `Background { image }`, `music <id>` / `music none` to `Music { track }`, and
  `sound <id>` to `Sound { id }`.
- `with <transition> [seconds]` on `show`, `background`, `remove` or `clear` compiles to a
  `With(Transition { kind, millis })` instruction just before the one it applies to. The
  other instructions keep their shape, so scenes without `with` compile (and fingerprint)
  exactly as before. `TransitionKind` is `Dissolve`, `Fade`, `SlideLeft` or `SlideRight`;
  `millis` is `None` for the default length (`default_millis()`: 500, 1000, 600, 600) and
  `Transition::seconds()` resolves it.
- `commit` compiles to `Commit`; `choice final:` compiles like `choice:` with a `Commit`
  at the start of every option's body (SCRIPT.md 10).
- An `if` compiles to `JumpIfFalse { condition, else_start }` + then-branch + `Goto`
  over the else-branch. The VM evaluates the whole condition.

`vn dump <file.story>` prints a compiled program with scene headers, plus any
diagnostics that don't need a registry (unknown jump targets, duplicate scenes).

## Variables and conditions

| Statement | Node / instruction |
| --- | --- |
| `set <var> = <value>` | `Set { var_id, value }` |
| `add <var> += <n>` / `add <var> -= <n>` | `Add { var_id, amount }` (`-=` stores a negative amount) |
| `if <condition>:` ... `else:` | `JumpIfFalse { condition, .. }` |

Values (`parse_value`):

| Literal | `Value` |
| --- | --- |
| `true`, `false` | `Bool` |
| An `i32`, e.g. `3`, `-2` | `Int` |
| A bare identifier, e.g. `good` | `Enum("good")` |
| Double-quoted text, e.g. `"Yuri"` | `String("Yuri")` |

`Enum` is a closed set the game defines (`route = good | bad | neutral`); `String` is free
text only known at runtime, like a name the player types. Strings use the same escapes as
dialogue (`\"`, `\\`, see [Parse errors](#parse-errors)).

`Value` implements `Display` as player-facing text (`Yuri`, `3`, `true`, `good`), which
interpolation uses. `Value::literal()` gives the DSL form (`"Yuri"`), which `vn dump` uses.
Conditions (`parse_condition`) are comparisons joined by `&&` and `||`, with `&&`
binding tighter and no parentheses (SCRIPT.md 8.4 allows no nesting). The condition is
tokenized first, so operators inside a string literal (`name == "a && b"`) are just text.
Spaces between tokens are optional (`affection>=-2`). They parse into a `Condition` tree:

```text
a == 1 || b == 2 && c == true
  ⇒ Any([Test(a == 1), All([Test(b == 2), Test(c == true)])])
```

A single comparison is a bare `Test`; `All`/`Any` only appear with two or more parts.
`Condition` implements `Display`, which `vn dump` uses.

Condition errors: no comparison operator, a single `=`, anything but `&&`/`||` between
comparisons, `<`/`>`/`<=`/`>=` against a non-integer literal, an unterminated string, an
integer outside `i32`, or a name that isn't an identifier.

Comparison rules in the VM:

| Variable | Literal | Result |
| --- | --- | --- |
| `Int` | `Int` | All six operators |
| `Bool` | `Bool` | `==` / `!=` |
| `Enum` | `Enum` | `==` / `!=` (by member name) |
| `String` | `String` | `==` / `!=` (exact, case-sensitive) |
| Different types | | `false`, for every operator |
| Unset | `Bool` / `Int` / `String` | Compared as `false` / `0` / `""` |
| Unset | `Enum` | Matches no member: `==` is `false`, `!=` is `true` |

With an empty variable registry, any variable name is accepted and types aren't checked
(handy for prototyping). Once variables are registered, see [Schema and
validation](#schema-and-validation).

Values the player provides are set from Rust with `vm.set_variable`, typically when the
game handles a `call` (e.g. `call ask_name player_name` → show an input screen →
`set_variable("player_name", Value::String(input))`).

## Parse errors

Nothing in the lexer or parser panics. Every error becomes an error `Diagnostic` with its
line, and parsing goes on so one broken line doesn't hide the rest.

The lexer:

- classifies each line by its first word, with a trailing `:` stripped (`else:` and
  `choice:` are keywords). A token's `payload` is the rest of the line. The keyword
  table (`keyword`, `keyword_name`, `is_keyword`) is shared with the parser.
- reads quoted text with `scan_string`, which handles the `\"` and `\\` escapes. A quoted
  line followed by `:` is a `ChoiceOption`; anything else quoted is `Narration`, so
  narration whose text ends in `:` stays narration.
- reports tabs in indentation and drops that line, so the lines after it still parse in
  the right block.

The parser (a `Parser` struct that collects diagnostics) recovers this way:

| Problem | Recovery |
| --- | --- |
| A broken simple statement (`show mary`, `jump`, bad identifier, bad string) | The statement is dropped |
| A line indented deeper than its block | One error; the whole over-indented run is skipped |
| A stray `else:`, choice option, or indented `scene` | Skipped with everything nested under it |
| A line outside any scene | Skipped with everything nested under it |
| `if` or a scene with a broken header | The body is still parsed (and checked), then dropped |
| `if`, `else:` or an option with no indented block | Error; the body is empty and the following siblings are not swallowed |
| `choice:` with no options, or a non-option line inside one | Error; that line and its nested lines are skipped |
| A scene with no lines | Warning only |

A line that doesn't start with a keyword and isn't valid dialogue gets a hint when its
first word is close to a keyword: `remoe hugo` ends with "did you mean `remove`?". The hint
is appended to the usual message, so an unrelated guess (`and` → `add`) still reads
sensibly.

Rules checked here rather than in the schema: `scene`, `if` and `choice` headers end with
`:`; `else` is exactly `else:`; `show` has exactly a character and an image; `remove` and
`jump` take one identifier; `clear` and `commit` take nothing; choice option text isn't
empty; every scene, character, image, variable, command and enum id is an identifier, and
character ids aren't keywords (`show "Hi"` is an error, not dialogue); nothing follows
a closing quote; `add` amounts fit in `i32`.

## Interpolation

`{variable}` in dialogue text, narration, choice options or the speaker is replaced with
the variable's value (SCRIPT.md 3.5):

```story
mary "Nice to meet you, {player_name}."
{player_name} "Nice to meet you, Mary."
```

- The VM interpolates when it emits `Say` and `Choice`, using the values at that moment,
  so every frontend receives finished text. `Event::Say.speaker` is the display name.
- The compiled program keeps the text as written (`vn dump` shows `{player_name}`).
- `{{` and `}}` produce literal braces.
- An unset variable, or braces around something that isn't an identifier
  (`{ name }`, `{Name}`), is left as written, so mistakes stay visible.
- `interpolate` is public, for frontends that build their own text.

## Schema and validation

A `Schema` describes what the game defines in Rust, so scripts can be checked before
they run (SCRIPT.md 8.6, 9.2):

| Registry | Type | Checks |
| --- | --- | --- |
| `variables` | name → `VariableDef { ty: VarType, default }` | `set`/`add`/conditions/`{var}` use known variables of the right type; enum members exist; `add` only on ints |
| `characters` | id → `CharacterDef { name, images }` | speakers, `show` and `remove` use known ids; `show` images are in `images` (when non-empty) |
| `commands` | name → `CommandSig { required, optional, rest }` | `call` names a known command, with the right number and kinds of arguments |

`VariableDef::bool(false)`, `::int(0)`, `::string("...")`,
`::enumeration(["good", "bad"], "good")` (panics if the default isn't a member).
`ParamKind` is `Int`, `UInt`, `Float`, `Bool` or `Word`.

**An empty registry turns its checks off.** A game can start with no registries and add
them as the story grows; once a registry has an entry, every use is checked.

`schema.validate(&program)` (or `vm.validate()`) returns `Vec<Diagnostic>` sorted by
file (in compile order) and line, including compile diagnostics and unknown `jump`
targets (always checked):

```text
story/04_santa_ilde.story:12: error: unknown variable 'trsut'
story/01_box_14.story:30: error: `set route`: 'great' is not one of good | bad | neutral
story/00_archive.story:41: error: `call give_item` argument 2 should be a non-negative integer, got `many`
```

`Diagnostic { severity, file, line, message }`: `Display` gives `path:N: error: ...` (the
form above) when `file` is set, `line N: error: ...` when it isn't, and leaves out the
line when it is 0 (problems not tied to a line, like a missing entry scene).
`with_file(Option<&str>)` sets the file. A speaker written as
`{variable}` isn't checked against characters, only the variable is.

Unknown scenes, characters, images, variables, commands and entry scenes end with a
suggestion when a known name is close:

```text
story/02_notebook.story:4: error: speaker: unknown character 'marry'; did you mean 'mary'?
```

The helpers are public in `suggest`: `edit_distance(a, b)` (insertions, deletions,
substitutions and swaps of neighbours each cost 1), `closest(word, candidates)` (the
nearest candidate within a third of the word's length, at least 1) and
`did_you_mean(word, candidates)` (the `; did you mean '...'?` suffix, or nothing).

### Schema files

A game writes its registries to a `schema.json` (`SCHEMA_FILE_NAME`) next to its assets,
so tools can validate stories without running the game (`vn check`, and later the LSP):

```json
{
  "format_version": 1,
  "game": "God Is Watching",
  "story_dir": "story",
  "entry_scene": "start",
  "variables": { "trust": { "ty": "Int", "default": { "Int": 0 } } },
  "characters": { "mary": { "name": "Mary", "images": ["tired"] } },
  "commands": { "give_item": { "required": ["Word"], "optional": ["UInt"], "rest": null } }
}
```

`SchemaFile { format_version, game, story_dir, entry_scene, schema }` (the schema's
fields are inlined). `story_dir` is relative to the schema file; `entry_scene` is left
out when the game uses the default.

| Method | Purpose |
| --- | --- |
| `SchemaFile::new(game, story_dir, schema)` | Current format version, no entry scene |
| `to_json()` / `from_json(&str)` | Pretty JSON with a trailing newline, so the file diffs cleanly. A newer `format_version` than `SCHEMA_FORMAT_VERSION` is an error |
| `read(path)` / `write(path) -> io::Result<bool>` | `write` returns `false` and leaves the file alone when nothing changed |

Every field except `format_version`, `game` and `story_dir` may be left out (an empty
registry, which turns its checks off), as may a character's `images` and a command's
`optional`/`rest`.

## StoryVm

The VM emits one `Event` per `advance()` call. The frontend decides how to present it.

| Event | Blocking | Meaning |
| --- | --- | --- |
| `Say { speaker, text }` | yes | A line of dialogue (`speaker: None` is narration) |
| `Choice { options }` | yes | Waits for `choose(index)`; `advance` returns the same choice until then |
| `End` | yes | The story is over; `advance` keeps returning `End` until `reset` |
| `Show { character, image, position, transition }` | no | Already applied to `active_characters()` (and `position()` when `at` was used). `transition` is the `with` of that line, if any |
| `Background { image, transition }` | no | Already applied to `background()`; `None` for `background none` |
| `Music { track }` | no | Already applied to `music()`; `None` for `music none` |
| `Sound { id }` | no | A one-shot sound for the frontend to play; not part of the state |
| `Voice { id }` | no | A voice clip for the next line; not part of the state |
| `Hide { character, transition }` | no | Already applied |
| `Clear { transition }` | no | Already applied |
| `Call { command, args }` | no | For the game to handle |
| `Commit` | no | A `commit` (or the start of a `choice final:` option): a rollback barrier, for the frontend |
| `SceneEnter { scene }` | no | The story entered a scene: at the start, on every `jump`, or when a restore restarted an edited scene. Only with `set_scene_events(true)` (off by default, so simple frontends never see it); not emitted when a restore returns to the exact position |

`Event::is_blocking()` tells whether the frontend should wait for the player.

### API

| Method | Purpose |
| --- | --- |
| `from_dir(dir)`, `from_file(path)`, `from_source(src)`, `from_program(program)` | Load. The VM starts at the entry scene, ready to run |
| `advance() -> Event` | Run to the next event |
| `advance_until_blocking() -> Event` | Run to the next `Say`/`Choice`/`End`, applying presentation events on the way |
| `choose(index) -> Result<(), VmError>` | Answer the pending choice (0-based) |
| `set_schema(schema)` | Use a schema: variables start at their defaults, `set_variable` is type-checked. Resets the story |
| `validate()` | The schema's diagnostics for this program (plus a missing entry scene) |
| `prepare(schema, entry_scene) -> Vec<Diagnostic>` | `set_schema` + `set_entry_scene` + `validate` in one call, with an error for an unknown entry scene. What the engine and `vn check` run |
| `set_entry_scene(id)`, `entry_scene()` | Start from a scene other than the first. Resets the story |
| `reset()` | Restart from the entry scene; variables back to their defaults, no characters |
| `start_at(scene_id) -> Result<(), VmError>` | Restart from a given scene |
| `current()` | The blocking event (`Say`/`Choice`/`End`) waiting for the player, if any. Lets a frontend leave and come back without skipping a line; cleared by the next `advance`, `choose` and `reset` |
| `current_scene()` | Scene the story is in (`None` for an empty file) |
| `active_characters()` | Character id → image id for everyone on screen |
| `position(character)` | The `Position` (`FarLeft`, `Left`, `Center`, `Right`, `FarRight`) given with `at`, if any. Kept when the character is shown again without `at`; forgotten by `remove` and `clear` |
| `background()` | The current background image id, if any. `clear` doesn't touch it |
| `music()` | The current music track id, if any |
| `line_key()` | A stable id for the line on screen (a hash of the scene, speaker and text as written), for "read" tracking; `None` unless a line is showing |
| `variables()`, `variable(id)`, `variable_type(id)` | Story variables |
| `set_variable(id, value) -> Result<(), VmError>` | Set from Rust. With registered variables, unknown names and wrong types are errors |
| `program()` | The compiled program |
| `set_scene_events(bool)` | Emit `Event::SceneEnter` (off by default) |

`VmError` values: `UnknownScene(id)`, `NoChoicePending`, `ChoiceOutOfRange { index, options }`,
`UnknownVariable(id)`, `TypeMismatch { variable, message }`.

A typical frontend loop:

```rust
let mut current = vm.advance_until_blocking();

// on click / key press:
match current {
    Event::Say { .. } => current = vm.advance_until_blocking(),
    Event::Choice { .. } => {
        vm.choose(picked)?;
        current = vm.advance_until_blocking();
    }
    Event::End => { /* back to the menu */ }
    _ => {}
}
```

Use `advance()` directly instead when the frontend wants to animate `Show`/`Hide`/`Clear`
or handle `Call`. With `advance_until_blocking()` those events are applied to the VM's
state but not returned.

### Snapshots (save/load)

`snapshot() -> StorySnapshot` captures the story (serde-serializable):

| Field | Meaning |
|---|---|
| `scene`, `offset` | Current scene and the position inside it (not an absolute index) |
| `scene_fingerprint` | Hash of that scene's compiled instructions, with jump targets made relative to the scene |
| `pending_choice`, `current` | Whether a choice is waiting, and the event on screen |
| `variables`, `active_characters` | Sorted maps, so save files are stable |
| `positions`, `background` | Character spots and the background (both missing in older snapshots, which restore with none) |
| `music` | The music track; left out of the JSON when there is none, and missing in older snapshots |

`restore(&snapshot) -> Result<RestoreOutcome, VmError>`:

Restoring starts from the schema's defaults and then applies the saved variables, so a
variable registered after the save was made gets its default.

- `Exact`: same scene, unchanged fingerprint. Continues exactly where it was, showing the
  same line or choice.
- `SceneRestarted { scene }`: the scene exists but was edited. Starts it from the top,
  keeping variables and characters.
- `Err(UnknownScene)`: the scene is gone. The VM is left untouched.

`check_restore` does the same checks without changing anything. Other scenes can change
freely: the fingerprint only covers the saved scene, and offsets are scene-relative.
`Program::scene_range(id)` and `Program::scene_fingerprint(id)` are the helpers behind it.

### Runtime behavior

- `advance` is a loop, not recursion, so long runs of non-blocking instructions can't
  overflow the stack.
- If one `advance` runs 100 000 instructions without producing an event (for example
  `scene a: jump b` / `scene b: jump a`), the VM logs a warning and ends the story
  instead of hanging.
- A `jump` to an unknown scene logs a warning and ends the story.
- A `choice:` with no options is skipped.
- `add` on an unset variable starts it at `Int(0)`. On a `Bool` or `Enum` variable it logs
  a warning and leaves the value unchanged. Overflow saturates.

## Tests

```sh
cargo test -p vn_script
```

| File | Covers |
| --- | --- |
| `tests/lexer.rs` | Every token kind, comment and blank-line skipping |
| `tests/parser.rs` | Conditions (every operator, `&&`/` | | ` precedence, enum and string literals, operators inside strings), `set`/`add`, interpolated speakers, `choice final:`, and the fixture compiling |
| `tests/diagnostics.rs` | Every parse error with its exact line and message, recovery (no follow-on errors, empty blocks don't swallow siblings, tabs), string escapes, file names in multi-file diagnostics |
| `tests/suggest.rs` | Edit distance, `closest` limits, keyword hints, suggestions for unknown scenes, images and entry scenes |
| `tests/template.rs` | Interpolation of every value type, unset variables, `{{`/`}}`, malformed braces |
| `tests/snapshot.rs` | Snapshot round trips (mid-scene, at a choice, JSON), edits to other scenes, edits to the saved scene, missing scenes |
| `tests/schema.rs` | Validation of every registry (unknown names, types, enum members, images, command arity and kinds), line numbers inside branches, defaults, typed `set_variable`, entry scene, old saves with new variables, and the example story directory, `prepare`, schema files (round trip, unchanged writes, missing fields, newer formats) |
| `tests/vm.rs` | Scene entry, `current()`, jumps, choice branches, end of story, reset, `start_at`, loop guard, condition evaluation (including strings), `set`/`add`, interpolation in text, speakers and choices, music state and sound events (and music in snapshots), the fixture playing through, the example playing through all three chapters, `story_files` (recursive, sorted, `.story` only) |
| `tests/fixtures/all_features.story` | Golden input covering every construct in SCRIPT.md |
