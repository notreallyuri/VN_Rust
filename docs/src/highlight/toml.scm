(comment) @comment

(table
  (bare_key) @label)

(table
  (dotted_key) @label)

(table_array_element
  (bare_key) @label)

(table_array_element
  (dotted_key) @label)

(pair
  (bare_key) @variable)

(pair
  (dotted_key) @variable)

(quoted_key) @string

(string) @string

(escape_sequence) @escape

(boolean) @constant

[
  (integer)
  (float)
] @number

[
  (offset_date_time)
  (local_date_time)
  (local_date)
  (local_time)
] @number

"=" @operator

"," @punctuation.delimiter

[
  "["
  "]"
  "[["
  "]]"
  "{"
  "}"
] @punctuation.bracket
