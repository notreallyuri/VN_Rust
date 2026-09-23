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

### 1.2 Text mode

```story
scene <scene_id> nvl:
```

A scene is `adv` by default: one line at a time in the dialogue box. `nvl` reads the
scene as full-screen pages instead, each line printed below the last.

```story
scene ruins nvl:
  "The corridor narrows."
  mary "I have been here before."
  "Her voice does not echo."
```

- `adv` and `nvl` are the only modes; `adv` is what you get by writing nothing
- The page starts empty when the scene begins and turns over when it is full
- The mode belongs to the scene, so a jump changes it, and a save or a rollback restores
  it without storing anything of its own
- The page is built from the lines the player has read, so it survives a load and steps
  back with rollback

## 2. Character Presentation

### 2.1 Show a character

```story
show <character_id> <image_id>
```

### 2.2 Rules

- Displays a character using a specific expression / image
- `<character_id> and <image_id>` must exist in the engine registry
- Re-showing a character updates the image

### 2.3 Positions

```story
show <character_id> <image_id> at <position>
```

`<position>` is one of `far_left`, `left`, `center`, `right`, `far_right`.

- The character stands at that spot until it is removed
- Re-showing without `at` keeps the spot, so changing an expression doesn't move anyone
- Characters shown without a position are spread evenly by the engine
- `remove` and `clear` forget the spot

```story
show mary tired at left
show hugo neutral at right
mary "Father."
show mary angry
```

---

### 2.4 Remove characters

```story
remove <character_id>
```

- Removes a single character from the scene

```story
clear
```

- Removes **all** characters from the scene (the background stays)

### 2.5 Backgrounds

```story
background <image_id>
background none
```

- Replaces the whole backdrop with `<image_id>`; `background none` removes it
- The background stays until the next `background`, across `jump`s and scenes
- Characters on screen are not affected
- The engine decides where images live (for the default engine,
  `assets/backgrounds/<image_id>.png`)

```story
scene the_study:
  background study_night
  show hugo tired at center
  "Father burns them in the study fireplace."
```

### 2.6 Transitions

```story
show <character_id> <image_id> [at <position>] with <transition> [seconds]
remove <character_id> with <transition> [seconds]
clear with <transition> [seconds]
background <image_id> with <transition> [seconds]
```

| Transition | Characters | Backgrounds | Default length |
| --- | --- | --- | --- |
| `dissolve` | fade in, fade out, crossfade between expressions | crossfade | 0.5 s |
| `fade` | like `dissolve` | through black | 1 s |
| `slide_left` | enter from the right edge, leave to the left edge | the new one pushes the old one out to the left | 0.6 s |
| `slide_right` | enter from the left edge, leave to the right edge | pushed out to the right | 0.6 s |
| `shake` | the change is instant, and the screen shakes | the same | 0.4 s |
| `flash` | the change is instant, and the screen flashes | the same | 0.3 s |

- `with` goes last on the line; the length is in seconds (above 0, up to 30)
- Showing a character who is already on screen with a new expression and `with` crossfades
  the two; with a new `at` position, the character also moves there
- Without `with`, the change is instant (as before)
- Transitions don't pause the story: the next line starts while they play, and a click
  finishes them. Rollback and loading a save show the end result at once
- `shake` and `flash` are effects, not motions: whatever the line changes happens at once
  and the whole screen shakes or flashes for that long. `clear with shake` or
  `background storm with flash` are the usual ways to punctuate a moment
- `with` can't be used as a character or image id

```story
  background santa_ilde_courtyard with fade
  show moriarty older at right with dissolve
  show moriarty wary with dissolve
  show moriarty wary at left with dissolve 0.8
  remove moriarty with slide_left
```

### 2.7 Music and sound

```story
music <track_id>
music none
sound <sound_id>
voice <voice_id>
```

- `music` starts a looping track, replacing the one playing (the default engine crossfades);
  `music none` fades it out
- Like the background, the music stays across `jump`s and scenes, and is saved and rolled
  back with the story. A `music` line naming the track that is already playing does nothing
- `sound` plays a one-shot effect. It isn't saved, and it isn't replayed on rollback or load
- `voice` plays a voice clip for the next line of dialogue; it stops when the player moves
  on. Auto mode waits for it
- The engine decides where files live (for the default engine, `assets/music/<track_id>` and
  `assets/sounds/<sound_id>` and `assets/voice/<voice_id>`, as `.ogg`, `.mp3`, `.wav` or `.flac`); a missing file is a
  warning, and the game stays silent there

```story
scene the_door:
  background santa_ilde_door
  music santa_ilde
  sound door_open
  "August 9th. A young woman came to the door after midnight."
```

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

### 3.7 Strings

Dialogue, narration, choice options and string values (`set`, conditions) are written in
double quotes. Two escapes are allowed inside them:

| Write | To get |
|---|---|
| `\"` | `"` |
| `\\` | `\` |

```story
mary "The sign says \"Closed\"."
```

Any other `\` is an error. Nothing may follow the closing quote (except the `:` of a
choice option).

### 3.8 Text tags

Dialogue, narration and choice text can carry tags in square brackets. Variables stay in
braces (`{name}`), so the two never collide.

| Tag | Effect |
|---|---|
| `[b]…[/b]` | Bold |
| `[i]…[/i]` | Italic |
| `[color=#c8a165]…[/color]` | Colour, six hex digits |
| `[size=34]…[/size]` | Text size in pixels |
| `[w]` / `[w=0.75]` | Hold the typewriter there, half a second by default |

```story
registrar "October, [b]1903[/b]. The [color=#c8a165]Archive of the House[/color][w=0.5], two floors below the street."
```

Tags nest, and `[[` writes a literal `[`. An unknown tag, a bad colour or size, and a
closing tag that closes nothing are errors; a tag left open is a warning and ends with
the line. Choice buttons show the text with its tags stripped.

Italic needs an italic font registered for the role (`VnApp::font_variant`); without one
the text is drawn in the regular face. Bold uses a bold font the same way, and is drawn
twice with a small offset when there is none.

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
- A choice must have at least one option
- Each option must:
  - Be a quoted string, not empty
  - End with :
  - Contain at least one instruction
- No fall through
- No implicit behavior
- A quoted line ending in `:` outside a `choice:` block is an error, not narration
- Between the option's text and its `:` come the modifiers of 4.4 and 4.5, each at most
  once, in any order. `when`, `unless`, `image` and `preview` mean what they say only
  there; everywhere else they are ordinary words

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

### 4.4 Conditions on an option

```story
choice:
  "<option_text>" when <condition>:
    <block>
  "<option_text>" unless <condition> "<reason>":
    <block>
```

- `when` offers the option only while the condition holds; `unless` is its opposite
- Without a reason, an option whose condition fails is not shown at all
- With a reason, it is shown but cannot be taken, and the reason is what the player reads
  when they point at it. Reasons are translated like any other line
- The conditions are the ones `if` takes (8.4), variables included
- `vn check` warns when every option of a choice can be hidden: a choice with nothing
  left to offer is skipped at runtime

```story
set has_key = true
choice:
  "Open the door" when has_key == true "The door is locked":
    "The lock gives."
  "Look through the window" unless curtains == true:
    "You see a table, and nothing else."
  "Turn back":
    "You leave the way you came."
```

### 4.5 Pictures on an option

```story
choice:
  "<option_text>" image <image_id>:
    <block>
  "<option_text>" preview <image_id>:
    <block>
```

- `image` draws `choices/<image_id>.png` on the option itself; the game decides whether
  that fills the button or sits beside the label as an icon
- `preview` shows `previews/<image_id>.png` while the option is under the pointer or has
  the focus
- Both are ids like every other asset (6.1), not paths

```story
choice:
  "The north road" image north preview north_view:
    "Stones, and then trees."
  "The river path" image river:
    "Water, loud enough to hide a voice."
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
  - image_id (characters and backgrounds)
  - track_id and sound_id
  - variable_id, enum members
  - command_id

A character_id can't be a keyword, because a line's first word decides what it is
(`show "Hi"` is a broken `show`, not dialogue). The keywords are:

`scene` `show` `background` `music` `sound` `voice` `remove` `clear` `choice` `commit` `jump` `if` `else` `call` `set` `add`

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
- Comments start with # (whole lines only)
- `scene` lines are not indented; everything else belongs to a scene
- Every `if`, `else:` and choice option needs an indented block below it
- A scene with no lines is allowed, with a warning

### 6.3 Errors

Every mistake in a script is reported with its line number, and the game doesn't start
until they are fixed. One broken line doesn't hide the errors after it.

A name that is close to a known one gets a suggestion: a misspelled keyword
(`remoe hugo` → did you mean `remove`?) and unknown scenes, characters, images,
variables and commands (`marry` → did you mean `mary`?).

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

Quotes inside a string are escaped with `\"` (section 3.7).

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

`else:` is optional and stands alone on its line. There is no `else if`: put an `if`
inside `else:` instead.

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
- An argument is outside the set of words the command takes (a command may declare that
  one of its arguments is one of a fixed list, e.g. `report`, `silence` or `keeper`)
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
