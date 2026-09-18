# vn_script

The story DSL: lexer, parser, compiler and VM. It has no rendering dependencies, so
the engine, the `vn` CLI and future tooling (LSP, formatter) can all share it.

The language itself is specified in [SCRIPT.md](../../SCRIPT.md).

## Pipeline

```text
source ──tokenize──▶ Vec<Token> ──parse_program──▶ Vec<Node::Scene> ──Compiler::compile──▶ Program ──▶ StoryVm
```

| Stage | Entry point | File |
| --- | --- | --- |
| Lexer | `tokenize(&str) -> Vec<Token>` | `src/lexer.rs` |
| Parser | `parse_program(&[Token]) -> Vec<Node>` (every top-level `scene`) | `src/parser.rs` |
| Conditions | `parse_condition(&str, line) -> Condition` | `src/condition.rs` |
| Compiler | `Compiler::new().compile(scenes) -> Program` | `src/compiler.rs` |
| VM | `StoryVm::from_file` / `from_source` / `from_program` | `src/vm.rs` |
| Interpolation | `interpolate(&str, &variables) -> String` | `src/template.rs` |

## Program

A whole `.story` file compiles into one `Program`:

- `instructions`: one flat list for every scene in the file.
- `lines`: the source line of each instruction, used by diagnostics.
- `scenes`: scene id → index of the scene's first instruction.
- `scene_order`: scene ids in file order. The first is the default **entry scene**.
- `diagnostics`: problems found while compiling (duplicate scenes).

The parser keeps the source line of every statement (`Stmt { line, node }`), and the
compiler copies it into `Program::lines`.

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
text only known at runtime, like a name the player types. A string literal can't contain
`"` (no escaping yet, same as dialogue).

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

Parse errors (these panic with the line number until M4 adds diagnostics):

- no comparison operator, a single `=`, or a missing `:` after the condition
- anything but `&&`/`||` between comparisons
- `<`, `>`, `<=`, `>=` against a `Bool`, `Enum` or `String` literal
- an unterminated string, or an integer outside `i32`
- a variable or enum name that isn't `[a-z_][a-z0-9_]*` (SCRIPT.md 6.1)
- `add` without `+=`/`-=`, or with a non-integer amount

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
line, including compile diagnostics and unknown `jump` targets (always checked):

```text
story/01_mary.story:12: error: unknown variable 'curiosty'
story/01_mary.story:30: error: `set route`: 'great' is not one of good | bad | neutral
story/01_mary.story:41: error: `call give_item` argument 2 should be a non-negative integer, got `many`
```

`Diagnostic { severity, line, message }`: `Display` gives `line N: error: ...`,
`in_file(path)` gives the `path:N: error: ...` form above. A speaker written as
`{variable}` isn't checked against characters, only the variable is.

Schemas are serde-serializable, so they can later be exported for `vn check` and editor
tooling (TODO.md, M4).

## StoryVm

The VM emits one `Event` per `advance()` call. The frontend decides how to present it.

| Event | Blocking | Meaning |
| --- | --- | --- |
| `Say { speaker, text }` | yes | A line of dialogue (`speaker: None` is narration) |
| `Choice { options }` | yes | Waits for `choose(index)`; `advance` returns the same choice until then |
| `End` | yes | The story is over; `advance` keeps returning `End` until `reset` |
| `Show { character, image }` | no | Already applied to `active_characters()` |
| `Hide { character }` | no | Already applied |
| `Clear` | no | Already applied |
| `Call { command, args }` | no | For the game to handle |
| `Commit` | no | A `commit` (or the start of a `choice final:` option): a rollback barrier, for the frontend |

`Event::is_blocking()` tells whether the frontend should wait for the player.

### API

| Method | Purpose |
| --- | --- |
| `from_file(path)`, `from_source(src)`, `from_program(program)` | Load. The VM starts at the entry scene, ready to run |
| `advance() -> Event` | Run to the next event |
| `advance_until_blocking() -> Event` | Run to the next `Say`/`Choice`/`End`, applying presentation events on the way |
| `choose(index) -> Result<(), VmError>` | Answer the pending choice (0-based) |
| `set_schema(schema)` | Use a schema: variables start at their defaults, `set_variable` is type-checked. Resets the story |
| `validate()` | The schema's diagnostics for this program (plus a missing entry scene) |
| `set_entry_scene(id)`, `entry_scene()` | Start from a scene other than the first. Resets the story |
| `reset()` | Restart from the entry scene; variables back to their defaults, no characters |
| `start_at(scene_id) -> Result<(), VmError>` | Restart from a given scene |
| `current()` | The blocking event (`Say`/`Choice`/`End`) waiting for the player, if any. Lets a frontend leave and come back without skipping a line; cleared by the next `advance`, `choose` and `reset` |
| `current_scene()` | Scene the story is in (`None` for an empty file) |
| `active_characters()` | Character id → image id for everyone on screen |
| `variables()`, `variable(id)`, `variable_type(id)` | Story variables |
| `set_variable(id, value) -> Result<(), VmError>` | Set from Rust. With registered variables, unknown names and wrong types are errors |
| `program()` | The compiled program |

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
| `tests/parser.rs` | Conditions (every operator, `&&`/` | | ` precedence, enum and string literals, operators inside strings, errors), `set`/`add`, interpolated speakers, and the fixture compiling |
| `tests/template.rs` | Interpolation of every value type, unset variables, `{{`/`}}`, malformed braces |
| `tests/snapshot.rs` | Snapshot round trips (mid-scene, at a choice, JSON), edits to other scenes, edits to the saved scene, missing scenes |
| `tests/schema.rs` | Validation of every registry (unknown names, types, enum members, images, command arity and kinds), line numbers inside branches, defaults, typed `set_variable`, entry scene, old saves with new variables, and the example stories |
| `tests/vm.rs` | Scene entry, `current()`, jumps, choice branches, end of story, reset, `start_at`, loop guard, condition evaluation (including strings), `set`/`add`, interpolation in text, speakers and choices, the fixture and every example story playing to the end |
| `tests/fixtures/all_features.story` | Golden input covering every construct in SCRIPT.md |
