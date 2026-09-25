# novn_macros

Two macros for the seam between a story and the Rust behind it: `#[command]`, which turns
a function into a command a story can `call`, and `#[derive(StoryWord)]`, which turns an
enum into the fixed set of words such a command accepts. `novn` re-exports both (and
has them in its prelude), so a game never names this crate in its `Cargo.toml`.

## `#[command]`

```rust
use novn::prelude::*;

#[command]
fn give_item(ctx: &mut GameContext, item: String, count: Option<u32>) -> Option<ScreenState> {
    ctx.state.get_mut::<Inventory>().add(&item, count.unwrap_or(1));
    None
}

VnApp::new("My Novel").command(give_item)
```

```story
call give_item verlaine_letter
call give_item candle 3
```

## What it writes

The function becomes a marker type of the same name carrying what the engine needs:

| Piece | From |
| --- | --- |
| `NAME` | The function's own name, or the one in `#[command("other_name")]` |
| `Args` | The parameters after the context, as the tuple `FromArgs` parses out of the line (`()` for none, `Vec<String>` on its own for "the rest of the line") |
| `run` | Unpacks that tuple and calls the body |
| `call` | The body as an ordinary associated function, so Rust code can run the command directly: `give_item::call(ctx, "candle".into(), Some(3))` |

The marker is what `VnApp::command` takes, which is why the name is written once and
checked by the compiler rather than repeated as a string. The story-facing signature
(argument count and kinds) is derived from the parameter types, exported to
`schema.json`, and checked against every `call` in the story — see
[novn's Commands](../novn/README.md#commands) for the types a parameter may
have.

## What it refuses

Each of these is a compile error at the offending span, not a surprise at startup:

- a name that a story could not write (`call` takes lowercase letters, digits and `_`,
  not starting with a digit)
- `async` functions, generic functions and methods (`self`)
- a borrowed argument: a command's arguments are parsed out of the story line, so they
  are owned — `String`, not `&str`
- a function with no return type; a command returns `Option<ScreenState>`, where `None`
  stays in the story and `Some(state)` switches screens

A closure has no name to take, so it stays with `VnApp::command_as("name", |ctx, ()| ...)`.

## `#[derive(StoryWord)]`

An argument a story writes as a word out of a fixed set is an enum:

```rust
#[derive(StoryWord, Clone, Copy, PartialEq, Eq)]
enum Note {
    DebtPaid,
    #[word("folio_41")]
    Folio41,
    HotWater,
}
```

| Piece | What it is |
| --- | --- |
| `as_str`, `Display` | The word: the variant in snake_case, or the `#[word("...")]` it was given |
| `from_word(&str)` | The variant, or `None` |
| `ALL` | Every variant, in the order they are written |
| `Default` | The first variant |
| `FromArg`, `Arg` | What lets `#[command] fn note(ctx: &mut GameContext, key: Note)` take it |
| `Serialize`, `Deserialize` | Through the word, so a save file holds the same string the story writes |

The set reaches the story's own tooling: an argument of this type is `ParamKind::Choice`,
which goes into `schema.json`, so `novn check` rejects a word outside the set (with a
"did you mean") and the LSP offers the words while a `call` line is typed. Deserializing
a word the enum no longer has is an error naming the ones it does have, rather than a
silent fallback.

Variant names convert on letter and digit boundaries — `EndingKeeper` → `ending_keeper`,
`Box14` → `box_14`, `Folio41` → `folio_41`. Where that is not the word you want,
`#[word("...")]` says so. Two variants may not claim the same word, a variant may not
carry data, and a word a story could not write (`call` takes lowercase letters, digits and
`_`) is a compile error.

## Tests

Both macros are exercised from `novn`, where the engine side of them lives:
`crates/novn/tests/commands.rs` (names, renaming, the signature flat parameters
produce, argument parsing, registration through the builder; and for story words: the
snake_case conversion, `#[word]`, the declared choices, the error for a word outside the
set, and the serde round trip).
