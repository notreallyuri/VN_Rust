#include "tree_sitter/array.h"
#include "tree_sitter/parser.h"

#include <stdint.h>
#include <string.h>

enum TokenType {
    NEWLINE,
    INDENT,
    DEDENT,
};

typedef struct {
    Array(uint16_t) indents;
} Scanner;

static inline void skip(TSLexer *lexer) { lexer->advance(lexer, true); }

void *tree_sitter_story_external_scanner_create(void) {
    Scanner *scanner = ts_calloc(1, sizeof(Scanner));
    array_init(&scanner->indents);
    array_push(&scanner->indents, 0);
    return scanner;
}

void tree_sitter_story_external_scanner_destroy(void *payload) {
    Scanner *scanner = (Scanner *)payload;
    array_delete(&scanner->indents);
    ts_free(scanner);
}

unsigned tree_sitter_story_external_scanner_serialize(void *payload, char *buffer) {
    Scanner *scanner = (Scanner *)payload;
    unsigned size = 0;
    for (uint32_t i = 1; i < scanner->indents.size; i++) {
        if (size + 2 > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) {
            break;
        }
        uint16_t indent = *array_get(&scanner->indents, i);
        buffer[size++] = (char)(indent & 0xff);
        buffer[size++] = (char)(indent >> 8);
    }
    return size;
}

void tree_sitter_story_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    Scanner *scanner = (Scanner *)payload;
    array_clear(&scanner->indents);
    array_push(&scanner->indents, 0);
    for (unsigned i = 0; i + 1 < length; i += 2) {
        uint16_t indent = (uint8_t)buffer[i] | ((uint16_t)(uint8_t)buffer[i + 1] << 8);
        array_push(&scanner->indents, indent);
    }
}

bool tree_sitter_story_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    Scanner *scanner = (Scanner *)payload;
    bool error_recovery = valid_symbols[NEWLINE] && valid_symbols[INDENT] && valid_symbols[DEDENT];

    lexer->mark_end(lexer);

    bool found_end_of_line = false;
    uint32_t indent = 0;
    for (;;) {
        if (lexer->lookahead == '\n') {
            found_end_of_line = true;
            indent = 0;
            skip(lexer);
        } else if (lexer->lookahead == ' ') {
            indent++;
            skip(lexer);
        } else if (lexer->lookahead == '\t') {
            indent += 8;
            skip(lexer);
        } else if (lexer->lookahead == '\r' || lexer->lookahead == '\f') {
            indent = 0;
            skip(lexer);
        } else if (lexer->lookahead == '#' && (found_end_of_line || lexer->get_column(lexer) == indent)) {
            while (lexer->lookahead && lexer->lookahead != '\n') {
                skip(lexer);
            }
        } else if (lexer->eof(lexer)) {
            indent = 0;
            found_end_of_line = true;
            break;
        } else {
            break;
        }
    }

    if (!found_end_of_line) {
        return false;
    }

    uint16_t current = *array_back(&scanner->indents);

    if (valid_symbols[INDENT] && indent > current && !error_recovery) {
        array_push(&scanner->indents, (uint16_t)indent);
        lexer->result_symbol = INDENT;
        return true;
    }

    if ((valid_symbols[DEDENT] || !valid_symbols[NEWLINE]) && indent < current && scanner->indents.size > 1) {
        array_pop(&scanner->indents);
        lexer->result_symbol = DEDENT;
        return true;
    }

    if (valid_symbols[NEWLINE] && !error_recovery) {
        lexer->result_symbol = NEWLINE;
        return true;
    }

    return false;
}
