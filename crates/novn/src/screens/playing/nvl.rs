use raylib::prelude::*;

use super::PlayingConfig;
use crate::context::DrawContext;
use crate::data::session::LogEntry;
use crate::ui::shape::PanelStyle;
use crate::ui::styled::{self, StyledText};
use crate::ui::{self, TextStyle};

const SPEAKER_SPACING: f32 = 1.3;

#[derive(Clone, Debug, PartialEq)]
pub struct NvlStyle {
    pub margin: f32,
    pub padding: f32,
    pub max_width: Option<f32>,
    pub entry_spacing: f32,
    pub panel: PanelStyle,
}

impl Default for NvlStyle {
    fn default() -> Self {
        Self {
            margin: 60.0,
            padding: 40.0,
            max_width: Some(1100.0),
            entry_spacing: 14.0,
            panel: PanelStyle::new(Color::new(0, 0, 0, 200)),
        }
    }
}

impl NvlStyle {
    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn max_width(mut self, width: Option<f32>) -> Self {
        self.max_width = width;
        self
    }

    pub fn entry_spacing(mut self, spacing: f32) -> Self {
        self.entry_spacing = spacing;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.panel.color = color;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn rect(&self, screen: Vector2) -> Rectangle {
        let full = screen.x - self.margin * 2.0;
        let width = self.max_width.map_or(full, |max| max.min(full));
        Rectangle::new(
            (screen.x - width) / 2.0,
            self.margin,
            width,
            (screen.y - self.margin * 2.0).max(0.0),
        )
    }

    pub fn page_start(&self, heights: &[f32], height: f32) -> usize {
        let mut start = 0;
        let mut used = 0.0;

        for (index, entry) in heights.iter().enumerate() {
            let gap = if index == start {
                0.0
            } else {
                self.entry_spacing
            };
            if index > start && used + gap + entry > height {
                start = index;
                used = *entry;
            } else {
                used += gap + entry;
            }
        }
        start
    }

    pub fn text_area(&self, rect: Rectangle) -> Rectangle {
        Rectangle::new(
            rect.x + self.padding,
            rect.y + self.padding,
            (rect.width - self.padding * 2.0).max(0.0),
            (rect.height - self.padding * 2.0).max(0.0),
        )
    }
}

pub(super) struct Entry {
    speaker: Option<(String, TextStyle)>,
    text: StyledText,
    style: TextStyle,
    height: f32,
}

pub(super) fn entries(ctx: &DrawContext, config: &PlayingConfig, width: f32) -> Vec<Entry> {
    let fonts = ctx.fonts();
    let scene = ctx.story.current_scene().unwrap_or_default();
    let log = ctx.log.entries();
    let start = log
        .iter()
        .rposition(|entry| entry.said().scene != scene)
        .map_or(0, |at| at + 1);

    log[start..]
        .iter()
        .filter_map(|entry| {
            let LogEntry::Line { speaker, said } = entry else {
                return None;
            };
            let speaker = speaker.as_ref().map(|speaker| {
                let name = ctx.characters.display_name(speaker, ctx.story);
                let style = match ctx.characters.color(speaker) {
                    Some(color) => crate::ui::reading::text(&config.speaker_text).color(color),
                    None => crate::ui::reading::text(&config.speaker_text),
                };
                (name, style)
            });
            let style = crate::ui::reading::text(&config.dialogue_text);
            let text = StyledText::parse(&said.text(ctx.story.catalog()));
            let height = speaker
                .as_ref()
                .map_or(0.0, |(_, style)| style.size * SPEAKER_SPACING)
                + styled::height(fonts, &text, &style, width);
            Some(Entry {
                speaker,
                text,
                style,
                height,
            })
        })
        .collect()
}

pub(super) fn draw(
    d: &mut RaylibDrawHandle,
    ctx: &DrawContext,
    config: &PlayingConfig,
    screen: Vector2,
    visible: Option<usize>,
) {
    let style = &config.nvl;
    let rect = style.rect(screen);
    let area = style.text_area(rect);

    let entries = entries(ctx, config, area.width);
    if entries.is_empty() {
        return;
    }

    style.panel.draw(d, rect);

    let fonts = ctx.fonts();
    let heights: Vec<f32> = entries.iter().map(|entry| entry.height).collect();
    let start = style.page_start(&heights, area.height);
    let last = entries.len() - 1;
    let mut y = area.y;

    for (index, entry) in entries.iter().enumerate().skip(start) {
        if index > start {
            y += style.entry_spacing;
        }
        if let Some((name, name_style)) = &entry.speaker {
            ui::draw_text(d, fonts, name, Vector2::new(area.x, y), name_style);
            y += name_style.size * SPEAKER_SPACING;
        }
        let shown = match index == last {
            true => visible.unwrap_or(usize::MAX),
            false => usize::MAX,
        };
        y += styled::draw(
            d,
            fonts,
            &entry.text,
            Vector2::new(area.x, y),
            area.width,
            &entry.style,
            shown,
        );
    }
}
