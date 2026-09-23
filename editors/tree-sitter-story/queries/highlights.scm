(comment) @comment @spell

[
  "scene"
  "show"
  "remove"
  "clear"
  "background"
  "music"
  "sound"
  "voice"
  "set"
  "add"
  "commit"
] @keyword

[
  "jump"
  "call"
] @keyword.function

[
  "if"
  "else"
  "choice"
  "final"
  "when"
  "unless"
] @keyword.conditional

[
  "at"
  "with"
  "image"
  "preview"
] @keyword.operator

[
  "="
  "+="
  "-="
  "=="
  "!="
  ">="
  "<="
  ">"
  "<"
  "&&"
  "||"
] @operator

":" @punctuation.delimiter

(scene
  name: (identifier) @label)

(scene_mode) @keyword.modifier

(option_image
  image: (identifier) @constant)

(option_preview
  image: (identifier) @constant)

(jump_statement
  scene: (identifier) @label)

(call_statement
  command: (identifier) @function.call)

(call_statement
  argument: (identifier) @variable.parameter)

(word) @string.special

(show_statement
  character: (identifier) @type)

(remove_statement
  character: (identifier) @type)

(dialogue
  speaker: (identifier) @type)

(show_statement
  image: (identifier) @constant)

(show_statement
  position: (identifier) @constant.builtin)

(background_statement
  image: (identifier) @constant)

(music_statement
  track: (identifier) @constant)

(sound_statement
  sound: (identifier) @constant)

(voice_statement
  clip: (identifier) @constant)

(transition
  kind: (identifier) @attribute)

(set_statement
  variable: (identifier) @variable)

(add_statement
  variable: (identifier) @variable)

(comparison
  variable: (identifier) @variable)

(string) @string

(string_content) @spell

(escape_sequence) @string.escape

(interpolation
  [
    "{"
    "}"
  ] @punctuation.special)

(interpolation
  variable: (identifier) @variable)

(number) @number

(boolean) @boolean

(none) @constant.builtin
