use crate::diagnostics::Diagnostic;
use crate::types::parser::{ChoiceOption, Node, Stmt, Token, TokenKind};

use super::parts::identifier;
use super::statement::{keyword_speaker, option_text, parse_statement};
use crate::condition::parse_condition;
use crate::lexer::keyword_name;

pub(super) struct Parser<'a> {
    pub(super) tokens: &'a [Token],
    pub(super) diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub(super) fn error(&mut self, line: usize, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(line, message));
    }

    pub(super) fn report<T>(&mut self, result: Result<T, Diagnostic>) -> Option<T> {
        result.map_err(|d| self.diagnostics.push(d)).ok()
    }

    pub(super) fn skip_nested(&self, start: usize) -> usize {
        let indent = self.tokens[start].indent;
        let mut i = start + 1;
        while i < self.tokens.len() && self.tokens[i].indent > indent {
            i += 1;
        }
        i
    }

    pub(super) fn indented(&self, start: usize) -> Option<usize> {
        let parent = self.tokens[start].indent;
        self.tokens
            .get(start + 1)
            .map(|next| next.indent)
            .filter(|&indent| indent > parent)
    }

    pub(super) fn body(&mut self, start: usize, what: &str) -> (Vec<Stmt>, usize) {
        match self.indented(start) {
            Some(indent) => self.blocks(start + 1, indent),
            None => {
                self.error(
                    self.tokens[start].line,
                    format!("{} needs an indented block below it", what),
                );
                (Vec::new(), start + 1)
            }
        }
    }

    pub(super) fn header(&mut self, token: &'a Token, what: &str) -> Option<&'a str> {
        let header = token.payload.strip_suffix(':').map(str::trim);
        if header.is_none() {
            self.error(token.line, format!("{} must end with `:`", what));
        }
        header
    }

    pub(super) fn program(&mut self) -> Vec<Stmt> {
        let mut scenes = Vec::new();
        let mut i = 0;

        while i < self.tokens.len() {
            let token = &self.tokens[i];

            if token.kind != TokenKind::Scene {
                if scenes.is_empty() || token.indent == 0 {
                    self.error(
                        token.line,
                        "this line is outside a scene; start the file with `scene <id>:`",
                    );
                    i = self.skip_nested(i);
                } else {
                    self.error(token.line, "unexpected indentation");
                    i += 1;
                    while i < self.tokens.len() && self.tokens[i].indent > 0 {
                        i += 1;
                    }
                }
                continue;
            }

            if token.indent > 0 {
                self.error(token.line, "`scene` can't be indented");
            }

            let (scene, next) = self.scene(i);
            scenes.extend(scene);
            i = next;
        }

        scenes
    }

    pub(super) fn scene(&mut self, start: usize) -> (Option<Stmt>, usize) {
        let token = &self.tokens[start];

        let id = self.header(token, "`scene <id>`").and_then(|id| {
            let id = identifier(id, "scene id", token.line);
            self.report(id)
        });

        let (body, next) = match self.indented(start) {
            Some(indent) => self.blocks(start + 1, indent),
            None => {
                if let Some(id) = &id {
                    self.diagnostics.push(Diagnostic::warning(
                        token.line,
                        format!("scene '{}' is empty", id),
                    ));
                }
                (Vec::new(), start + 1)
            }
        };

        let scene = id.map(|id| Stmt {
            line: token.line,
            node: Node::Scene { id, body },
        });
        (scene, next)
    }

    pub(super) fn blocks(&mut self, start: usize, indent: usize) -> (Vec<Stmt>, usize) {
        let mut nodes = Vec::new();
        let mut i = start;

        while i < self.tokens.len() {
            let token = &self.tokens[i];

            if token.indent < indent {
                break;
            }

            if token.indent > indent {
                self.error(token.line, "unexpected indentation");
                while i < self.tokens.len() && self.tokens[i].indent > indent {
                    i += 1;
                }
                continue;
            }

            let (node, next) = self.statement(i);
            nodes.extend(node.map(|node| Stmt {
                line: token.line,
                node,
            }));
            i = next;
        }

        (nodes, i)
    }

    pub(super) fn statement(&mut self, i: usize) -> (Option<Node>, usize) {
        let token = &self.tokens[i];

        if let Some(keyword) = keyword_name(token.kind)
            && token.payload.starts_with('"')
        {
            self.error(token.line, keyword_speaker(keyword));
            return (None, self.skip_nested(i));
        }

        match token.kind {
            TokenKind::ChoiceBlock => self.choice(i),
            TokenKind::If => self.if_block(i),
            TokenKind::Else => {
                self.error(token.line, "`else:` without a matching `if` above it");
                (None, self.skip_nested(i))
            }
            TokenKind::ChoiceOption => {
                self.error(
                    token.line,
                    "choice option outside a `choice:` block (narration doesn't end with `:`)",
                );
                (None, self.skip_nested(i))
            }
            TokenKind::Scene => {
                self.error(
                    token.line,
                    "`scene` can't be indented (scenes can't be nested)",
                );
                (None, self.skip_nested(i))
            }
            _ => {
                let node = parse_statement(token);
                (self.report(node), i + 1)
            }
        }
    }

    pub(super) fn if_block(&mut self, start: usize) -> (Option<Node>, usize) {
        let token = &self.tokens[start];

        let condition = self.header(token, "`if <condition>`").and_then(|expr| {
            let condition = parse_condition(expr, token.line);
            self.report(condition)
        });
        let (then_branch, mut next) = self.body(start, "`if`");

        let mut else_branch = Vec::new();
        if let Some(else_token) = self.tokens.get(next)
            && else_token.kind == TokenKind::Else
            && else_token.indent == token.indent
        {
            if else_token.payload != ":" {
                self.error(
                    else_token.line,
                    "write `else:` on its own line (for `else if`, put an `if` inside `else:`)",
                );
            }
            let (branch, after) = self.body(next, "`else:`");
            else_branch = branch;
            next = after;
        }

        let node = condition.map(|condition| Node::If {
            condition,
            then_branch,
            else_branch,
        });
        (node, next)
    }

    pub(super) fn choice(&mut self, start: usize) -> (Option<Node>, usize) {
        let token = &self.tokens[start];

        let final_choice = match self.header(token, "`choice`") {
            Some("final") => true,
            Some("") | None => false,
            Some(other) => {
                self.error(
                    token.line,
                    format!(
                        "unknown choice modifier `{}` (expected `choice:` or `choice final:`)",
                        other
                    ),
                );
                false
            }
        };

        let Some(option_indent) = self.indented(start) else {
            self.error(token.line, "`choice:` needs at least one option below it");
            return (None, start + 1);
        };

        let mut options = Vec::new();
        let mut i = start + 1;

        while i < self.tokens.len() && self.tokens[i].indent > token.indent {
            let option = &self.tokens[i];

            if option.indent != option_indent {
                self.error(option.line, "unexpected indentation in `choice:` block");
                i = self.skip_nested(i);
                continue;
            }

            if option.kind != TokenKind::ChoiceOption {
                self.error(
                    option.line,
                    "expected a choice option: quoted text ending with `:`, like `\"Agree\":`",
                );
                i = self.skip_nested(i);
                continue;
            }

            let text = self.report(option_text(option));
            let (body, next) = self.body(i, "choice option");
            options.extend(text.map(|text| ChoiceOption {
                text,
                line: option.line,
                body,
            }));
            i = next;
        }

        let node = Node::ChoiceBlock {
            options,
            final_choice,
        };
        (Some(node), i)
    }
}
