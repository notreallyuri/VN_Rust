use raylib::prelude::*;
use vn_script::markup::{self, Span};

use crate::ui::TextStyle;
use crate::ui::fonts::Fonts;

pub const LINE_SPACING: f32 = 1.3;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StyledText {
    spans: Vec<Span>,
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct StyledLine {
    pub pieces: Vec<Piece>,
}

impl StyledText {
    pub fn parse(text: &str) -> Self {
        Self {
            spans: markup::parse(text),
        }
    }

    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    pub fn plain(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }

    pub fn chars(&self) -> usize {
        self.spans
            .iter()
            .map(|span| span.text.chars().count())
            .sum()
    }

    pub fn pauses(&self) -> Vec<(usize, f32)> {
        let mut pauses = Vec::new();
        let mut at = 0;
        for span in &self.spans {
            if span.pause > 0.0 && at > 0 {
                pauses.push((at, span.pause));
            }
            at += span.text.chars().count();
        }
        pauses
    }
}

impl StyledLine {
    pub fn height(&self, style: &TextStyle) -> f32 {
        let tallest = self
            .pieces
            .iter()
            .map(|piece| piece.span.size.unwrap_or(style.size))
            .fold(style.size, f32::max);
        tallest * LINE_SPACING
    }

    pub fn chars(&self) -> usize {
        self.pieces
            .iter()
            .map(|piece| piece.text.chars().count())
            .sum()
    }

    fn take(&self, count: usize) -> StyledLine {
        let mut left = count;
        let mut pieces = Vec::new();
        for piece in &self.pieces {
            if left == 0 {
                break;
            }
            let length = piece.text.chars().count();
            if length <= left {
                left -= length;
                pieces.push(piece.clone());
            } else {
                pieces.push(Piece {
                    text: piece.text.chars().take(left).collect(),
                    span: piece.span.clone(),
                });
                left = 0;
            }
        }
        StyledLine { pieces }
    }
}

fn size_of(span: &Span, style: &TextStyle) -> f32 {
    span.size.unwrap_or(style.size)
}

fn color_of(span: &Span, style: &TextStyle) -> Color {
    match span.color {
        Some(rgb) => Color::new(
            ((rgb >> 16) & 0xff) as u8,
            ((rgb >> 8) & 0xff) as u8,
            (rgb & 0xff) as u8,
            style.color.a,
        ),
        None => style.color,
    }
}

fn measure(fonts: &Fonts, piece: &Piece, style: &TextStyle) -> f32 {
    fonts
        .measure_styled(
            style.font,
            &piece.text,
            size_of(&piece.span, style),
            style.spacing,
            piece.span.bold,
            piece.span.italic,
        )
        .x
}

struct Word {
    pieces: Vec<Piece>,
    newline: bool,
    space_before: bool,
}

fn split(pieces: Vec<Piece>) -> Vec<Vec<Piece>> {
    let text: String = pieces.iter().map(|piece| piece.text.as_str()).collect();
    let breaks = crate::ui::wrap::breaks(&text);
    if breaks.is_empty() {
        return vec![pieces];
    }

    let mut groups: Vec<Vec<Piece>> = Vec::new();
    let mut group: Vec<Piece> = Vec::new();
    let mut at = 0;
    let mut next = breaks.iter().copied().peekable();

    for piece in pieces {
        let mut rest = piece.text.as_str();
        while let Some(&stop) = next.peek() {
            let taken = rest.chars().count();
            if stop >= at + taken {
                break;
            }
            let cut: String = rest.chars().take(stop - at).collect();
            group.push(Piece {
                text: cut.clone(),
                span: piece.span.clone(),
            });
            groups.push(std::mem::take(&mut group));
            rest = &rest[cut.len()..];
            at = stop;
            next.next();
        }
        if !rest.is_empty() {
            at += rest.chars().count();
            group.push(Piece {
                text: rest.to_string(),
                span: piece.span.clone(),
            });
        }
    }

    if !group.is_empty() {
        groups.push(group);
    }
    groups
}

fn words(text: &StyledText) -> Vec<Word> {
    let mut words: Vec<Word> = Vec::new();
    let mut current: Vec<Piece> = Vec::new();
    let mut space = false;

    let push =
        |current: &mut Vec<Piece>, words: &mut Vec<Word>, space: &mut bool, newline: bool| {
            if current.is_empty() && !newline {
                return;
            }
            let mut space_before = std::mem::take(space);
            let mut groups = split(std::mem::take(current)).into_iter().peekable();
            if groups.peek().is_none() {
                words.push(Word {
                    pieces: Vec::new(),
                    newline,
                    space_before,
                });
                return;
            }
            while let Some(pieces) = groups.next() {
                words.push(Word {
                    pieces,
                    newline: newline && groups.peek().is_none(),
                    space_before,
                });
                space_before = false;
            }
        };

    for span in text.spans() {
        let mut piece = String::new();
        for c in span.text.chars() {
            if c == '\n' {
                if !piece.is_empty() {
                    current.push(Piece {
                        text: std::mem::take(&mut piece),
                        span: span.clone(),
                    });
                }
                push(&mut current, &mut words, &mut space, true);
            } else if c.is_whitespace() {
                if !piece.is_empty() {
                    current.push(Piece {
                        text: std::mem::take(&mut piece),
                        span: span.clone(),
                    });
                }
                push(&mut current, &mut words, &mut space, false);
                space = true;
            } else {
                piece.push(c);
            }
        }
        if !piece.is_empty() {
            current.push(Piece {
                text: piece,
                span: span.clone(),
            });
        }
    }
    push(&mut current, &mut words, &mut space, false);
    words
}

pub fn wrap(
    fonts: &Fonts,
    text: &StyledText,
    style: &TextStyle,
    max_width: f32,
) -> Vec<StyledLine> {
    let space = fonts
        .measure_spaced(style.font, " ", style.size, style.spacing)
        .x;
    let mut lines: Vec<StyledLine> = Vec::new();
    let mut line = StyledLine::default();
    let mut width = 0.0;

    for word in words(text) {
        let word_width: f32 = word
            .pieces
            .iter()
            .map(|piece| measure(fonts, piece, style))
            .sum();
        let spaced = if line.pieces.is_empty() || !word.space_before {
            0.0
        } else {
            space
        };

        if !line.pieces.is_empty() && width + spaced + word_width > max_width {
            lines.push(std::mem::take(&mut line));
            width = 0.0;
        } else if spaced > 0.0 {
            line.pieces.push(Piece {
                text: " ".to_string(),
                span: word
                    .pieces
                    .first()
                    .map(|piece| piece.span.clone())
                    .unwrap_or_default(),
            });
            width += spaced;
        }

        width += word_width;
        line.pieces.extend(word.pieces);

        if word.newline {
            lines.push(std::mem::take(&mut line));
            width = 0.0;
        }
    }

    if !line.pieces.is_empty() {
        lines.push(line);
    }
    lines
}

pub fn draw_line(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    line: &StyledLine,
    position: Vector2,
    style: &TextStyle,
) {
    let baseline = line.height(style) / LINE_SPACING;
    let mut x = position.x;

    for piece in &line.pieces {
        let size = size_of(&piece.span, style);
        let at = Vector2::new(x, position.y + (baseline - size));
        fonts.draw_styled(
            d,
            style.font,
            &piece.text,
            at,
            size,
            style.spacing,
            color_of(&piece.span, style),
            piece.span.bold,
            piece.span.italic,
        );
        x += measure(fonts, piece, style);
    }
}

pub fn draw(
    d: &mut RaylibDrawHandle,
    fonts: &Fonts,
    text: &StyledText,
    position: Vector2,
    max_width: f32,
    style: &TextStyle,
    visible: usize,
) -> f32 {
    let lines = wrap(fonts, text, style, max_width);
    let mut left = visible;
    let mut y = position.y;

    for line in &lines {
        let height = line.height(style);
        if left == 0 {
            break;
        }
        let shown = line.chars();
        let drawn = if shown <= left {
            line.clone()
        } else {
            line.take(left)
        };
        left = left.saturating_sub(shown);
        draw_line(d, fonts, &drawn, Vector2::new(position.x, y), style);
        y += height;
    }

    lines.iter().map(|line| line.height(style)).sum()
}

pub fn height(fonts: &Fonts, text: &StyledText, style: &TextStyle, max_width: f32) -> f32 {
    wrap(fonts, text, style, max_width)
        .iter()
        .map(|line| line.height(style))
        .sum()
}
