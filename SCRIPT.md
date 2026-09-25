# The story language

Stories are written in a small indentation-based language. It describes narrative flow and
nothing else: no logic, no systems, no expressions to evaluate. Everything a story can name
is registered in Rust first, and every name it uses is checked before the window opens.

**The guide is at <https://notreallyuri.github.io/novn/docs/scenes>.** This file is the cheat
sheet. Each row below links to the page that explains it.

## A complete story

```story
scene start:
  show gabriel serious

  gabriel "This is a really interesting format."

  show mary serious
  mary "Is the parser working?"

  choice:
    "Yes":
      "The room goes silent."
      jump second_scene

    "No":
      clear


scene second_scene:
  show gabriel worried
  "Things just got darker."
```

That is a valid story. A speaker before a quoted line makes it dialogue, a quoted line on its
own is narration, and a scene runs top to bottom until it jumps or ends.

## Every statement

| Statement | Does | Page |
| --- | --- | --- |
| `scene <name>:` | Opens a scene. Everything under it is indented | [Scenes and characters](https://notreallyuri.github.io/novn/docs/scenes) |
| `show <char> <image>` | Puts a character on stage, or changes the image they are showing | [Scenes and characters](https://notreallyuri.github.io/novn/docs/scenes) |
| `remove <char>` | Takes one character off | [Scenes and characters](https://notreallyuri.github.io/novn/docs/scenes) |
| `clear` | Takes everyone off | [Scenes and characters](https://notreallyuri.github.io/novn/docs/scenes) |
| `background <image>` | Changes the background | [Scenes and characters](https://notreallyuri.github.io/novn/docs/scenes) |
| `music <name>`, `sound <name>`, `voice <name>` | Audio. Music loops and carries across scenes, sound fires once, voice belongs to the line after it | [Scenes and characters](https://notreallyuri.github.io/novn/docs/scenes) |
| `<char> "..."` | Dialogue | [Dialogue and narration](https://notreallyuri.github.io/novn/docs/dialogue) |
| `"..."` | Narration | [Dialogue and narration](https://notreallyuri.github.io/novn/docs/dialogue) |
| `choice:` | Branches. Each option is a quoted label with an indented body | [Choices and jumps](https://notreallyuri.github.io/novn/docs/choices) |
| `jump <scene>` | Goes to another scene and does not come back | [Choices and jumps](https://notreallyuri.github.io/novn/docs/choices) |
| `set <var> = <value>` | Assigns a registered variable | [Variables and conditions](https://notreallyuri.github.io/novn/docs/variables) |
| `add <var> += <value>` | Moves a number, with `+=` or `-=` | [Variables and conditions](https://notreallyuri.github.io/novn/docs/variables) |
| `if <cond>:` / `else:` | Conditional flow | [Variables and conditions](https://notreallyuri.github.io/novn/docs/variables) |
| `call <command> <args>` | Runs a Rust command | [Engine commands](https://notreallyuri.github.io/novn/docs/commands) |
| `commit` | Drops the rollback history to here | [Rollback](https://notreallyuri.github.io/novn/docs/rollback) |

Modifiers that attach to the statements above: `at` and `with` for placement and transitions,
`image` and `preview` for an option's pictures, `when` and `unless` for gating an option, and
`final` for a choice the player cannot roll back through.

## The rules worth knowing up front

- **Indentation is two spaces**, and it is structure rather than style. A body is what sits
  indented under the line that opens it.
- **Identifiers are lowercase**, letters, digits and `_`, not starting with a digit. That goes
  for scenes, characters, images, variables and commands alike.
- **Nothing is declared in the story.** Characters, variables and commands are registered in
  Rust, and the story may only name what is already there.
- **A typo is an error before the window opens**, with the file, the line, and a suggestion of
  the nearest name that does exist.

[Identifiers and layout](https://notreallyuri.github.io/novn/docs/syntax) has the whole of it,
including what the language deliberately leaves out and why.

## Checking a story

```sh
novn check path/to/assets      # validate against the game's schema.json
novn dump path/to/story        # see how it compiles
```

Both are in [`novn-cli`](crates/novn-cli/README.md).
