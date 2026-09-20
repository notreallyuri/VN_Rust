use std::cell::Cell;
use std::rc::Rc;

use raylib::ffi;
use raylib::prelude::*;

use crate::PanelStyle;
use crate::ui::scroll::{Scroll, ScrollStyle};
use crate::ui::styled::{self, StyledLine, StyledText};
use crate::ui::{self, ButtonStyle, TextStyle};
use crate::{DrawContext, Focus, FontRole, GameContext, LogEntry, Overlay, OverlayAction};

pub const LOG_OVERLAY: &str = "log";

#[derive(Clone, Debug)]
pub struct LogConfig {
    pub title: String,
    pub title_text: TextStyle,
    pub speaker_text: TextStyle,
    pub line_text: TextStyle,
    pub narration_text: TextStyle,
    pub choice_text: TextStyle,
    pub choice_prefix: String,
    pub empty_label: String,
    pub panel_width: f32,
    pub entry_spacing: f32,
    pub panel: PanelStyle,
    pub backdrop: Color,
    pub back_button: ButtonStyle,
    pub back_label: String,
    pub close_keys: Vec<KeyboardKey>,
    pub scroll: ScrollStyle,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            title: "Log".to_string(),
            title_text: TextStyle::new(FontRole::Title, 40.0, Color::RAYWHITE),
            speaker_text: TextStyle::new(FontRole::Speaker, 20.0, Color::GOLD),
            line_text: TextStyle::new(FontRole::Dialogue, 20.0, Color::RAYWHITE),
            narration_text: TextStyle::new(FontRole::Dialogue, 20.0, Color::LIGHTGRAY),
            choice_text: TextStyle::new(FontRole::Menu, 19.0, Color::new(150, 190, 230, 255)),
            choice_prefix: "» ".to_string(),
            empty_label: "Nothing has happened yet.".to_string(),
            panel_width: 900.0,
            entry_spacing: 16.0,
            panel: PanelStyle::new(Color::new(14, 14, 22, 235)).roundness(0.03),
            backdrop: Color::new(0, 0, 0, 170),
            back_button: ButtonStyle::default().size(200.0, 46.0),
            back_label: "Back".to_string(),
            close_keys: vec![
                KeyboardKey::KEY_ESCAPE,
                KeyboardKey::KEY_BACKSPACE,
                KeyboardKey::KEY_L,
            ],
            scroll: ScrollStyle::default(),
        }
    }
}

impl LogConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn speaker_text(mut self, style: TextStyle) -> Self {
        self.speaker_text = style;
        self
    }

    pub fn line_text(mut self, style: TextStyle) -> Self {
        self.line_text = style;
        self
    }

    pub fn narration_text(mut self, style: TextStyle) -> Self {
        self.narration_text = style;
        self
    }

    pub fn choice_text(mut self, style: TextStyle) -> Self {
        self.choice_text = style;
        self
    }

    pub fn choice_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.choice_prefix = prefix.into();
        self
    }

    pub fn empty_label(mut self, text: impl Into<String>) -> Self {
        self.empty_label = text.into();
        self
    }

    pub fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width;
        self
    }

    pub fn entry_spacing(mut self, spacing: f32) -> Self {
        self.entry_spacing = spacing;
        self
    }

    pub fn panel_color(mut self, color: Color) -> Self {
        self.panel.color = color;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn back_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.back_button = style(self.back_button);
        self
    }

    pub fn back_label(mut self, text: impl Into<String>) -> Self {
        self.back_label = text.into();
        self
    }

    pub fn close_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.close_keys = keys.into_iter().collect();
        self
    }

    fn panel_rect(&self, screen: Vector2) -> Rectangle {
        let width = self.panel_width.min(screen.x - 40.0);
        Rectangle::new((screen.x - width) / 2.0, 24.0, width, screen.y - 48.0)
    }

    fn entries_area(&self, screen: Vector2) -> Rectangle {
        let panel = self.panel_rect(screen);
        let top = panel.y + self.title_text.size + 40.0;
        let bottom = self.back_rect(screen).y - 20.0;
        Rectangle::new(
            panel.x + 32.0,
            top,
            panel.width - 64.0,
            (bottom - top).max(0.0),
        )
    }

    fn back_rect(&self, screen: Vector2) -> Rectangle {
        let panel = self.panel_rect(screen);
        let style = &self.back_button;
        Rectangle::new(
            (screen.x - style.width) / 2.0,
            panel.y + panel.height - style.height - 20.0,
            style.width,
            style.height,
        )
    }
}

pub struct LogOverlay {
    config: Rc<LogConfig>,
    scroll: Scroll,
    content: Cell<f32>,
    focus: Focus,
}

impl LogOverlay {
    pub fn new(config: Rc<LogConfig>) -> Self {
        Self {
            config,
            scroll: Scroll::new(),
            content: Cell::new(0.0),
            focus: Focus::default(),
        }
    }
}

struct Block {
    speaker: Option<(String, TextStyle)>,
    lines: Vec<StyledLine>,
    style: TextStyle,
}

impl Block {
    fn height(&self) -> f32 {
        let speaker = self
            .speaker
            .as_ref()
            .map_or(0.0, |(_, style)| style.size * 1.3);
        speaker
            + self
                .lines
                .iter()
                .map(|line| line.height(&self.style))
                .sum::<f32>()
    }
}

fn blocks(ctx: &DrawContext, config: &LogConfig, width: f32) -> Vec<Block> {
    let fonts = ctx.fonts();
    ctx.log
        .entries()
        .iter()
        .map(|entry| match entry {
            LogEntry::Line { speaker, text } => {
                let style = match speaker {
                    Some(_) => config.line_text.clone(),
                    None => config.narration_text.clone(),
                };
                let speaker = speaker.as_ref().map(|speaker| {
                    let name = ctx.characters.display_name(speaker, ctx.story);
                    let style = match ctx.characters.color(speaker) {
                        Some(color) => config.speaker_text.clone().color(color),
                        None => config.speaker_text.clone(),
                    };
                    (name, style)
                });
                let lines = styled::wrap(fonts, &StyledText::parse(text), &style, width);
                Block {
                    speaker,
                    lines,
                    style,
                }
            }
            LogEntry::Choice { text } => {
                let style = config.choice_text.clone();
                let text = format!("{}{}", config.choice_prefix, text);
                Block {
                    speaker: None,
                    lines: styled::wrap(fonts, &StyledText::parse(&text), &style, width),
                    style,
                }
            }
        })
        .collect()
}

impl Overlay for LogOverlay {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        let config = Rc::clone(&self.config);
        let screen = ui::screen_size(ctx.rl);
        let back = config.back_rect(screen);

        let key = config.close_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        let right_click = ctx
            .rl
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT);
        let accepted = self.focus.update(&ctx.nav, &[back], &[], None);
        if ui::button_clicked(&mut ctx, back, &config.back_button)
            || key
            || right_click
            || ctx.nav.back
            || ctx.nav.log
            || accepted.is_some()
        {
            return OverlayAction::Close;
        }

        let area = config.entries_area(screen);
        self.scroll.extent(area.height, self.content.get());
        self.scroll.input(ctx.rl, area, &config.scroll);

        let page = self.scroll.page();
        let mut delta = 0.0;
        if ctx.nav.up {
            delta += config.scroll.step;
        }
        if ctx.nav.down {
            delta -= config.scroll.step;
        }
        if ctx.nav.page_back || ctx.rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            delta += page;
        }
        if ctx.nav.page_forward || ctx.rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN) {
            delta -= page;
        }
        self.scroll.by(-delta);
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            self.scroll.to_start();
        }
        if ctx.rl.is_key_pressed(KeyboardKey::KEY_END) {
            self.scroll.to_end();
        }
        OverlayAction::Stay
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let panel = config.panel_rect(screen);
        let area = config.entries_area(screen);

        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, config.backdrop);
        config.panel.draw(d, panel);
        ui::draw_text_centered(
            d,
            fonts,
            ctx.label(&config.title),
            Vector2::new(
                screen.x / 2.0,
                panel.y + 20.0 + config.title_text.size / 2.0,
            ),
            &config.title_text,
        );

        let blocks = blocks(ctx, config, area.width);
        let content: f32 = blocks.iter().map(Block::height).sum::<f32>()
            + blocks.len().saturating_sub(1) as f32 * config.entry_spacing;
        self.content.set(content);

        if blocks.is_empty() {
            ui::draw_text_centered(
                d,
                fonts,
                ctx.label(&config.empty_label),
                Vector2::new(screen.x / 2.0, area.y + area.height / 2.0),
                &config.narration_text,
            );
        }

        unsafe {
            ffi::BeginScissorMode(
                area.x as i32,
                area.y as i32,
                area.width as i32,
                area.height as i32,
            );
        }
        let mut bottom = if content < area.height {
            area.y + content
        } else {
            area.y + area.height + self.scroll.from_end()
        };
        for block in blocks.iter().rev() {
            let top = bottom - block.height();
            if top < area.y + area.height && bottom > area.y {
                let mut y = top;
                if let Some((name, style)) = &block.speaker {
                    ui::draw_text(d, fonts, name, Vector2::new(area.x, y), style);
                    y += style.size * 1.3;
                }
                for line in &block.lines {
                    styled::draw_line(d, fonts, line, Vector2::new(area.x, y), &block.style);
                    y += line.height(&block.style);
                }
            }
            bottom = top - config.entry_spacing;
            if bottom < area.y {
                break;
            }
        }
        unsafe {
            ffi::EndScissorMode();
        }
        self.scroll.draw_bar(d, area, &config.scroll);

        ui::Button::new(ctx.label(&config.back_label), &config.back_button)
            .focused(ctx.shows_focus(&self.focus, 0))
            .draw(d, ctx, config.back_rect(screen));
    }
}
