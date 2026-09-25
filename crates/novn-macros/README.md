# novn-macros

Two macros for the seam between a story and the Rust behind it: `#[command]`, which turns a
function into a command a story can `call`, and `#[derive(StoryWord)]`, which turns an enum into
the fixed set of words such a command accepts.

[`novn`](../novn/README.md) re-exports both and has them in its prelude, so a game never names
this crate in its `Cargo.toml`.

**The guide is at <https://notreallyuri.github.io/novn/docs/crates/macros>**: what each macro
writes, what it refuses and why, and how the set of words reaches `novn check` and the LSP.

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

## Tests

Both macros are exercised from `novn`, where the engine side of them lives, in
`crates/novn/tests/commands.rs`: names, renaming, the signature flat parameters produce,
argument parsing, registration through the builder, and for story words the snake_case
conversion, `#[word]`, the declared choices, the error for a word outside the set, and the serde
round trip.
