/**
 * @file Story scripts for VN_Rust (SCRIPT.md)
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = { or: 1, and: 2, compare: 3 };

module.exports = grammar({
  name: 'story',

  externals: $ => [$._newline, $._indent, $._dedent],

  extras: $ => [/[ \t\r\f\n]/, $.comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($.scene),

    comment: _ => token(seq('#', /[^\n]*/)),

    scene: $ => seq('scene', field('name', $.identifier), ':', $._newline, optional($.block)),

    block: $ => seq($._indent, repeat1($._statement), $._dedent),

    _statement: $ => choice(
      $.show_statement,
      $.remove_statement,
      $.clear_statement,
      $.background_statement,
      $.music_statement,
      $.sound_statement,
      $.voice_statement,
      $.jump_statement,
      $.call_statement,
      $.set_statement,
      $.add_statement,
      $.commit_statement,
      $.dialogue,
      $.narration,
      $.choice_statement,
      $.if_statement,
    ),

    _line: $ => $._newline,

    transition: $ => seq('with', field('kind', $.identifier), optional(field('seconds', $.number))),

    show_statement: $ => seq(
      'show',
      field('character', $.identifier),
      field('image', $.identifier),
      optional(seq('at', field('position', $.identifier))),
      optional($.transition),
      $._line,
    ),

    remove_statement: $ => seq('remove', field('character', $.identifier), optional($.transition), $._line),

    clear_statement: $ => seq('clear', optional($.transition), $._line),

    background_statement: $ => seq(
      'background',
      field('image', choice($.none, $.identifier)),
      optional($.transition),
      $._line,
    ),

    music_statement: $ => seq('music', field('track', choice($.none, $.identifier)), $._line),

    sound_statement: $ => seq('sound', field('sound', $.identifier), $._line),

    voice_statement: $ => seq('voice', field('clip', $.identifier), $._line),

    jump_statement: $ => seq('jump', field('scene', $.identifier), $._line),

    call_statement: $ => seq(
      'call',
      field('command', $.identifier),
      repeat(field('argument', choice($.number, $.boolean, $.identifier, $.word))),
      $._line,
    ),

    set_statement: $ => seq(
      'set',
      field('variable', $.identifier),
      '=',
      field('value', $._value),
      $._line,
    ),

    add_statement: $ => seq(
      'add',
      field('variable', $.identifier),
      field('operator', choice('+=', '-=')),
      field('amount', $.number),
      $._line,
    ),

    commit_statement: $ => seq('commit', $._line),

    dialogue: $ => seq(field('speaker', choice($.identifier, $.interpolation)), field('text', $.string), $._line),

    narration: $ => seq(field('text', $.string), $._line),

    choice_statement: $ => seq(
      'choice',
      optional(field('final', 'final')),
      ':',
      $._newline,
      $._indent,
      repeat1($.choice_option),
      $._dedent,
    ),

    choice_option: $ => seq(field('text', $.string), ':', $._newline, optional($.block)),

    if_statement: $ => seq(
      'if',
      field('condition', $._condition),
      ':',
      $._newline,
      optional(field('consequence', $.block)),
      optional(field('alternative', $.else_clause)),
    ),

    else_clause: $ => seq('else', ':', $._newline, optional($.block)),

    _condition: $ => choice($.comparison, $.and_condition, $.or_condition),

    and_condition: $ => prec.left(PREC.and, seq(
      field('left', $._condition),
      '&&',
      field('right', $._condition),
    )),

    or_condition: $ => prec.left(PREC.or, seq(
      field('left', $._condition),
      '||',
      field('right', $._condition),
    )),

    comparison: $ => prec(PREC.compare, seq(
      field('variable', $.identifier),
      field('operator', choice('==', '!=', '>=', '<=', '>', '<')),
      field('value', $._value),
    )),

    _value: $ => choice($.boolean, $.number, $.string, $.identifier),

    string: $ => seq(
      '"',
      repeat(choice($.string_content, $.escape_sequence, $.interpolation)),
      token.immediate('"'),
    ),

    string_content: _ => token.immediate(prec(1, /[^"\\{\n]+/)),

    escape_sequence: _ => token.immediate(seq('\\', /["\\]/)),

    interpolation: $ => seq(
      choice('{', token.immediate('{')),
      field('variable', alias(token.immediate(/[a-z_][a-z0-9_]*/), $.identifier)),
      token.immediate('}'),
    ),

    boolean: _ => choice('true', 'false'),

    none: _ => 'none',

    number: _ => /-?\d+(\.\d+)?/,

    identifier: _ => /[a-z_][a-z0-9_]*/,

    word: _ => /[^\s"#]+/,
  },
});
