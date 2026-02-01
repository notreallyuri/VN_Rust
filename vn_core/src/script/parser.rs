use super::types::parser::{ChoiceOption, Condition, Node, Token, TokenKind};

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

    let condition_str = if_token
        .payload
        .trim_start_matches("if ")
        .trim_end_matches(':')
        .trim();
    let condition = Condition;

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
    let mut i = start + 1;
    let mut options = Vec::new();

    if i >= tokens.len() {
        return (Node::Choice { option: options }, i);
    }
    let option_indent = tokens[i].indent;

    if option_indent <= base_indent {
        return (Node::Choice { option: options }, i);
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

        options.push(ChoiceOption { text, body });
        i = next_i;
    }

    (Node::Choice { option: options }, i)
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

pub fn parse_blocks(tokens: &[Token], start: usize, indent: usize) -> (Vec<Node>, usize) {
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
            TokenKind::Scene => parse_scene(tokens, i),
            TokenKind::Choice => parse_choice(tokens, i),
            TokenKind::If => parse_if(tokens, i),
            _ => (parse_statement(token), i + 1),
        };

        nodes.push(node);
        i = next_i;
    }

    (nodes, i)
}
