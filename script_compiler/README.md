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

Variables are defined in the engine (or a schema file) and exposed to the script runtime.

Example (conceptual):

- `affection` -> Integer
- `met_mary` -> boolean
- `route` -> enum (`good`, `bad`, `neutral`)

Scripts **cannot create or redefine variables**.

---

### 8.2 Setting a Variable

```story
set <variable_id> = <value>
```

#### 8.2.1 Rules

- `<variable_id>` must exist
- Assigned value must match the variable's type
- Enum values must be valid members

#### 8.2.2 Examples

```story
set met_mary = true
set route = good
```

---

### 8.3 Modifying Numeric Variables

For integer variables only:

```story
add <variable_id> += <integer>
add <variable_id> -+ <integer>
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
  - `== != < > <= >=`
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

## 10. What this DSL intentionally does not include

By design:

- Variable declarations
- Functions
- Loops
- Inline expressions
- Script-defined assets

Those belong in **Rust**, not the script.

## 11. Why this structure is strong

- Extremely easy to parse (indent + first token)
- Writer-friendly
- Editor-friendly (Tree-sitter, LSP, formatter)
- Deterministic execution
- Safe validation before runtime

This is not _YAML_, not _Ren'py_, and that's a good thing
