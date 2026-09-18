# Story DSL -- Script Structure

## 1. Scene Definition

A script is composed of scenes.
Each scene defines a linear sequence of instructions.

```story
scene <scene_id>:
```

### 1.1 Rules

- `<scene_id>` must be unique
- Scenes are jump targets
- Execution starts at a designated entry scene (e.g. start)

## 2. Character Presentation

### 2.1 Show a character

```story
show <character_id> <image_id>
```

### 2.2 Rules

- Displays a character using a specific expression / image
- `<character_id> and <image_id>` must exist in the engine registry
- Re-showing a character updates the image

---

### 2.3 Remove characters

```story
remove <character_id>
```

- Removes a single character from the scene

```story
clear
```

- Removes **all** characters from the scene

## 3. Dialogue & narration

### 3.1 Character Dialogue

```story
<character_id> "<text>"
```

### 3.2 Rules

- Displays dialogue attributed to a character
- Blocks execution until user input
- Character must exist

---

### 3.3 Anonymous narration

```story
"<text>"
```

### 3.4 Rules

- Displays narration with no speaker
- Rendering style is decided by the engine
- Also blocks execution until user input

This rule is explicit and intentional — not a fallback.

---

### 3.5 Variable interpolation

Dialogue text, narration, choice options and the speaker can show a variable's value
with `{<variable_id>}`:

```story
mary "Nice to meet you, {player_name}."
{player_name} "Nice to meet you, Mary."
"You have {coins} coins."
choice:
  "Tell {player_name}'s story":
    ...
```

### 3.6 Rules

- `<variable_id>` must exist (see section 8)
- The value is inserted as text: `true`/`false`, the number, the enum member, or the string
- `{{` and `}}` insert a literal `{` or `}`
- A variable with no value yet is shown as written (`{player_name}`), so the mistake is visible
- `{<variable_id>} "<text>"` uses the variable's value as the displayed speaker name

## 4. Choices (branching)

### 4.1 Choice block

```story
choice:
  "<option_text>":
    <block>
  "<option_text>":
    <block>
```

### 4.2 Rules

- choice: must open a block
- Each option must:
- Be a quoted string
- End with :
- Contain at least one instruction
- No fall through
- No implicit behavior

### 4.3 Example

```story
choice:
  "Agree":
    "You nod silently."
    jump next_scene

  "Disagree":
    "You shake your head."
    jump argument_scene
```

## 5. Scene transition

### 5.1 Jump to another scene

```story
jump <scene_id>
```

### 5.2 Rules

- Transfers execution to another scene
- Target scene must exist
- Ends execution of the current scene

## 6. Identifiers & formatting rules (Important!)

### 6.1 Identifiers

- Lowercase only
- Must match:

```css
[a-z_][a-z0-9_]*
```

- Applies to:
  - scene_id
  - character_id
  - image_id

This guarantees:

- No ambiguity
- Case-insensitive tooling
- Easier validation

---

### 6.2 Indentation

- Indentation defines scope
- Spaces only (recommended: 2 spaces)
- Tabs are invalid
- Blank lines are allowed
- Comments start with #

## 7. Full Minimal Example (Valid script)

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

## 8. Variable Interaction

This DSL does **not** allow variable declaration.
All variables are **declared, typed and owned by the Rust engine**

The script may **read and modify only pre-defined variables**

This ensures:

- Deterministic execution
- Strong validation
- Safe save/load behavior
- Editor tooling can detect errors early

---

### 8.1 Variable Registry (Engine-Side)

Variables are defined in the engine (or a schema file) and exposed to the script
runtime.

Example (conceptual):

- `affection` -> Integer
- `met_mary` -> boolean
- `route` -> enum (`good`, `bad`, `neutral`)
- `player_name` -> string

Scripts **cannot create or redefine variables**.

---

### 8.2 Setting a Variable

```story
set <variable_id> = <value>
```

#### 8.2.1 Values

| Literal | Type |
|---|---|
| `true`, `false` | boolean |
| `3`, `-2` | integer |
| `good` (a bare identifier) | enum member |
| `"Yuri"` (double-quoted) | string |

A string literal cannot contain `"`.

#### 8.2.2 Rules

- `<variable_id>` must exist
- Assigned value must match the variable's type
- Enum values must be valid members

#### 8.2.3 Examples

```story
set met_mary = true
set route = good
set player_name = "Yuri"
```

Values the player provides (e.g. typing a name) are set by the engine, usually in
response to a command (section 9):

```story
call ask_name player_name
mary "Nice to meet you, {player_name}."
```

---

### 8.3 Modifying Numeric Variables

For integer variables only:

```story
add <variable_id> += <integer>
add <variable_id> -= <integer>
```

#### 8.3.1 Rules

- Variable must exist
- Variable must be numeric
- Only constant integer deltas are allowed

#### 8.3.2 Examples

```story
add affection += 1
add affection -= 2
```

---

### 8.4 Conditional Branching

Scripts may branch based on engine-defined variables.

```story
if <condition>:
  <block>
else:
  <block>
```

#### 8.4.1 Condition Rules

Allowed:

- Variable references
- Literal values
- Comparison operators:
  - `== != < > <= >=` for integers
  - `== !=` for booleans, enums and strings
- Boolean operators:
  - `&& ||`

Not allowed:

- Function calls
- Assignments
- Arithmetic expressions
- Nested expressions

#### 8.4.2 Example

```story
if affection >= 3 && met_mary == true:
  mary "You remembered."
else:
  mary "You forgot."
```

### 8.5 Execution Model

- Conditions are evaluated by the interpreter using the current engine state
- Variable mutations are validated and applied by the engine
- Invalid variable access is a compile-time error, not a runtime error

### 8.6 Validation Rules

A script is invalid if:

- A referenced variable does not exist
- A variable is assigned a value of the wrong type
- A non-numeric variable is modified with add
- A condition uses unsupported operators or expressions

### 8.7 Design Intent

This system intentionally avoids becoming a general-purpose scripting language.

- Scripts **describe story flow**
- Rust **owns logic and state**
- Variables act as **gates**, not computation tools

This keeps DSL:

- Simple
- Safe
- Predictable
- Easy to tool

### 8.8 Minimal Example (Variables + Choices)

```story
scene start:
  if met_mary == true:
    mary "Nice to see you again."
  else:
    mary "Hello."

  choice:
    "Be kind":
      add affection += 1
      jump next_scene

    "Say nothing":
      jump next_scene
```

## 9. Engine Commands (Side effects)

The script may invoke _engine-defined commands_.

```story
call <command_id> [args...]
```

### 9.1 Rules

- Commands are registered by the engine
- Scripts cannot define commands
- Commands:
  - May mutate engine state
  - May trigger effects (inventory, sounds, flags)
- Commands do **not** return values
- Commands do **not** affect control flow

### 9.2 Validation

A script is invalid if:

- The command does not exist
- Argument count or types do not match
- Arguments reference unknown identifiers

### 9.3 Example

```story
call give_item stick 1
call unlock_route good
```

## 10. Rollback (points of no return)

Players can roll back to earlier lines (mouse wheel up / Page Up) and forward again. The
engine decides the defaults (SCRIPT.md does not require rollback); the script marks the
moments that must not be undone.

### 10.1 Commit

```story
commit
```

A point of no return: the player can't roll back to anything before it. Rolling back
stops at the first line after the `commit`.

### 10.2 Final choices

```story
choice final:
  "Keep the letter":
    ...
  "Burn the letter":
    ...
```

Whatever option is picked, the choice can't be undone. Same as starting every option's
body with `commit`.

### 10.3 Rules

- `commit` takes no arguments
- `choice final:` is the only choice modifier
- Plain `choice:` blocks follow the engine's default (rollback through choices is allowed
  unless the game turns it off)
- Put `commit` inside a single option to make only that option permanent:

```story
choice:
  "Spare him":
    "He runs into the night."
  "Pull the trigger":
    commit
    "It is done."
```

- Engine commands can also be declared as barriers on the Rust side (for example one that
  unlocks an achievement)

## 11. What this DSL intentionally does not include

By design:

- Variable declarations
- Functions
- Loops
- Inline expressions
- Script-defined assets

Those belong in **Rust**, not the script.

## 12. Why this structure is strong

- Extremely easy to parse (indent + first token)
- Writer-friendly
- Editor-friendly (Tree-sitter, LSP, formatter)
- Deterministic execution
- Safe validation before runtime

This is not _YAML_, not _Ren'py_, and that's a good thing
