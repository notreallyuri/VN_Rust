use crate::types::Value;
use crate::types::parser::{ChoiceOption, Node, Stmt, Token, TokenKind};

pub use crate::condition::parse_condition;

pub fn parse_program(tokens: &[Token]) -> Vec<Stmt> {
    let mut scenes = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].kind == TokenKind::Scene {
            let (scene, next_i) = parse_scene(tokens, i);
            scenes.push(Stmt {
                line: tokens[i].line,
                node: scene,
            });
            i = next_i;
        } else {
            i += 1;
        }
    }

    scenes
}

pub fn parse_scene(tokens: &[Token], start: usize) -> (Node, usize) {
    let token = &tokens[start];
    let id = token.payload.trim_end_matches(':').trim().to_string();

    if start + 1 >= tokens.len() {
        return (Node::Scene { id, body: vec![] }, start + 1);
    }

    let next_indent = tokens[start + 1].indent;

    if next_indent <= token.indent {
        return (Node::Scene { id, body: vec![] }, start + 1);
    }

    let (body, next_i) = parse_blocks(tokens, start + 1, next_indent);

    (Node::Scene { id, body }, next_i)
}

pub fn parse_dialogue(payload: &str) -> (Option<String>, String) {
    if let Some(first_quote) = payload.find('"') {
        let speaker_part = payload[..first_quote].trim();
        let text_part = payload[first_quote..].trim_matches('"');

        let speaker = if speaker_part.is_empty() {
            None
        } else {
            Some(speaker_part.to_string())
        };

        (speaker, text_part.to_string())
    } else {
        (None, payload.to_string())
    }
}

pub fn parse_if(tokens: &[Token], start: usize) -> (Node, usize) {
    let if_token = &tokens[start];
    let base_indent = if_token.indent;

    let condition = parse_condition(&if_token.payload, if_token.line);

    let then_indent = if start + 1 < tokens.len() {
        tokens[start + 1].indent
    } else {
        base_indent
    };
    let (then_branch, mut next_i) = parse_blocks(tokens, start + 1, then_indent);

    let mut else_branch = Vec::new();

    if next_i < tokens.len() {
        let next_token = &tokens[next_i];

        if next_token.kind == TokenKind::Else && next_token.indent == base_indent {
            let else_indent = if next_i + 1 < tokens.len() {
                tokens[next_i + 1].indent
            } else {
                base_indent
            };
            let (else_nodes, final_i) = parse_blocks(tokens, next_i + 1, else_indent);
            else_branch = else_nodes;
            next_i = final_i;
        }
    }

    (
        Node::If {
            condition,
            then_branch,
            else_branch,
        },
        next_i,
    )
}

pub fn parse_choice(tokens: &[Token], start: usize) -> (Node, usize) {
    let base_indent = tokens[start].indent;
    let final_choice = match tokens[start].payload.trim_end_matches(':').trim() {
        "" => false,
        "final" => true,
        other => panic!(
            "Unknown choice modifier `{}` at line {} (expected `choice:` or `choice final:`)",
            other, tokens[start].line
        ),
    };
    let mut i = start + 1;
    let mut options = Vec::new();

    if i >= tokens.len() {
        return (
            Node::ChoiceBlock {
                options,
                final_choice,
            },
            i,
        );
    }
    let option_indent = tokens[i].indent;

    if option_indent <= base_indent {
        return (
            Node::ChoiceBlock {
                options,
                final_choice,
            },
            i,
        );
    }

    while i < tokens.len() {
        let token = &tokens[i];

        if token.indent < option_indent {
            break;
        }

        if token.indent != option_indent {
            panic!(
                "Indentation mismatch in choice block at line {}. Expected {}, got {}",
                token.line, option_indent, token.indent
            );
        }

        let text = token
            .payload
            .trim_end_matches(':')
            .trim_matches('"')
            .to_string();

        let mut body_indent = option_indent;
        if i + 1 < tokens.len() {
            let next_indent = tokens[i + 1].indent;
            if next_indent > option_indent {
                body_indent = next_indent;
            }
        }

        let (body, next_i) = parse_blocks(tokens, i + 1, body_indent);

        options.push(ChoiceOption {
            text,
            line: token.line,
            body,
        });
        i = next_i;
    }

    (
        Node::ChoiceBlock {
            options,
            final_choice,
        },
        i,
    )
}

pub fn parse_statement(token: &Token) -> Node {
    match token.kind {
        TokenKind::Show => {
            let mut parts = token.payload.split_whitespace();
            Node::Show {
                character: parts.next().unwrap().to_string(),
                image: parts.next().unwrap().to_string(),
            }
        }

        TokenKind::Remove => Node::Remove {
            character: token.payload.trim().to_string(),
        },

        TokenKind::Clear => Node::Clear,

        TokenKind::Commit => {
            if !token.payload.is_empty() {
                panic!("`commit` takes no arguments at line {}", token.line);
            }
            Node::Commit
        }

        TokenKind::Jump => Node::Jump {
            target: token.payload.trim().to_string(),
        },

        TokenKind::Call => {
            let mut parts = token.payload.split_whitespace();
            Node::Call {
                command: parts.next().unwrap().to_string(),
                args: parts.map(|s| s.to_string()).collect(),
            }
        }

        TokenKind::Set => parse_set(&token.payload, token.line),

        TokenKind::Add => parse_add(&token.payload, token.line),

        TokenKind::Dialogue => {
            let (speaker, text) = parse_dialogue(&token.payload);
            Node::Dialogue { speaker, text }
        }

        TokenKind::Narration => {
            let (speaker, text) = parse_dialogue(&token.payload);
            Node::Dialogue { speaker, text }
        }

        _ => panic!("Unhandled statement at line {}", token.line),
    }
}

pub fn parse_blocks(tokens: &[Token], start: usize, indent: usize) -> (Vec<Stmt>, usize) {
    let mut nodes = vec![];
    let mut i = start;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.indent < indent {
            break;
        }

        if token.indent > indent {
            panic!("Unexpected indentation at line {}", token.line);
        }

        let (node, next_i) = match token.kind {
            TokenKind::ChoiceBlock => parse_choice(tokens, i),
            TokenKind::If => parse_if(tokens, i),
            _ => (parse_statement(token), i + 1),
        };

        nodes.push(Stmt {
            line: token.line,
            node,
        });
        i = next_i;
    }

    (nodes, i)
}

pub fn parse_value(literal: &str, line: usize) -> Value {
    match literal {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        _ if literal.starts_with('"') => {
            let text = literal
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                .filter(|text| !text.contains('"'))
                .unwrap_or_else(|| panic!("Invalid string literal `{}` at line {}", literal, line));
            Value::String(text.to_string())
        }
        _ => match literal.parse::<i32>() {
            Ok(n) => Value::Int(n),
            Err(_) => Value::Enum(parse_identifier(literal, line)),
        },
    }
}

pub fn is_identifier(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

pub(crate) fn parse_identifier(text: &str, line: usize) -> String {
    if !is_identifier(text) {
        panic!("Invalid identifier `{}` at line {}", text, line);
    }

    text.to_string()
}

fn parse_set(payload: &str, line: usize) -> Node {
    let (var_id, literal) = payload
        .split_once('=')
        .unwrap_or_else(|| panic!("Expected `set <variable> = <value>` at line {}", line));

    Node::Set {
        var_id: parse_identifier(var_id.trim(), line),
        value: parse_value(literal.trim(), line),
    }
}

fn parse_add(payload: &str, line: usize) -> Node {
    let (var_id, sign, amount) = if let Some((var_id, amount)) = payload.split_once("+=") {
        (var_id, 1, amount)
    } else if let Some((var_id, amount)) = payload.split_once("-=") {
        (var_id, -1, amount)
    } else {
        panic!(
            "Expected `add <variable> += <integer>` or `-=` at line {}",
            line
        );
    };

    let amount: i32 = amount
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("`add` needs an integer amount at line {}", line));

    Node::Add {
        var_id: parse_identifier(var_id.trim(), line),
        amount: sign * amount,
    }
}
