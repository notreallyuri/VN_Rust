use std::rc::Rc;

use raylib::prelude::*;

use crate::action::Action;
use crate::context::{DrawContext, GameContext};
use crate::input::navigation::Focus;
use crate::overlay::{Overlay, OverlayAction};
use crate::ui;
use crate::ui::TextStyle;
use crate::ui::button::ButtonStyle;
use crate::ui::fonts::FontRole;
use crate::ui::shape::PanelStyle;

pub const CONFIRM_OVERLAY: &str = "confirm";

#[derive(Clone, Debug)]
pub struct Confirm {
    pub message: String,
    pub action: Action,
    pub confirm_label: Option<String>,
    pub cancel_label: Option<String>,
}

impl Confirm {
    pub fn new(message: impl Into<String>, action: Action) -> Self {
        Self {
            message: message.into(),
            action,
            confirm_label: None,
            cancel_label: None,
        }
    }

    pub fn confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = Some(label.into());
        self
    }

    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = Some(label.into());
        self
    }
}

impl From<Confirm> for Action {
    fn from(confirm: Confirm) -> Self {
        Action::Confirm(Box::new(confirm))
    }
}

#[derive(Clone, Debug)]
pub struct ConfirmConfig {
    pub message_text: TextStyle,
    pub confirm_label: String,
    pub cancel_label: String,
    pub confirm_button: ButtonStyle,
    pub cancel_button: ButtonStyle,
    pub panel_width: f32,
    pub padding: f32,
    pub panel: PanelStyle,
    pub backdrop: Color,
    pub confirm_keys: Vec<KeyboardKey>,
    pub cancel_keys: Vec<KeyboardKey>,
}

impl Default for ConfirmConfig {
    fn default() -> Self {
        Self {
            message_text: TextStyle::new(FontRole::Menu, 22.0, Color::RAYWHITE),
            confirm_label: "Yes".to_string(),
            cancel_label: "Cancel".to_string(),
            confirm_button: ButtonStyle::default()
                .size(150.0, 44.0)
                .color(Color::new(90, 40, 40, 255))
                .font_size(20.0),
            cancel_button: ButtonStyle::default().size(150.0, 44.0).font_size(20.0),
            panel_width: 460.0,
            padding: 28.0,
            panel: PanelStyle::new(Color::new(22, 22, 34, 250)).roundness(0.05),
            backdrop: Color::new(0, 0, 0, 160),
            confirm_keys: vec![KeyboardKey::KEY_ENTER, KeyboardKey::KEY_Y],
            cancel_keys: vec![KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_N],
        }
    }
}

impl ConfirmConfig {
    pub fn message_text(mut self, style: TextStyle) -> Self {
        self.message_text = style;
        self
    }

    pub fn confirm_label(mut self, label: impl Into<String>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn confirm_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.confirm_button = style(self.confirm_button);
        self
    }

    pub fn cancel_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.cancel_button = style(self.cancel_button);
        self
    }

    pub fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
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

    pub fn panel_roundness(mut self, roundness: f32) -> Self {
        self.panel = self.panel.roundness(roundness);
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn confirm_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.confirm_keys = keys.into_iter().collect();
        self
    }

    pub fn cancel_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.cancel_keys = keys.into_iter().collect();
        self
    }
}

struct Layout {
    panel: Rectangle,
    lines: Vec<String>,
    text_top: f32,
    confirm: Rectangle,
    cancel: Rectangle,
}

pub struct ConfirmDialog {
    config: Rc<ConfirmConfig>,
    request: Option<Confirm>,
    focus: Focus,
}

const CANCEL: usize = 0;
const CONFIRM: usize = 1;

impl ConfirmDialog {
    pub fn new(config: Rc<ConfirmConfig>) -> Self {
        Self {
            config,
            request: None,
            focus: Focus::default(),
        }
    }

    fn layout(&self, fonts: &crate::ui::fonts::Fonts, screen: Vector2) -> Layout {
        let config = &self.config;
        let message = self.request.as_ref().map_or("", |r| r.message.as_str());
        let style = &config.message_text;

        let lines = fonts.wrap(
            style.font,
            message,
            style.size,
            config.panel_width - config.padding * 2.0,
        );
        let text_height = lines.len() as f32 * style.size * 1.3;
        let button_height = config
            .confirm_button
            .height
            .max(config.cancel_button.height);
        let height = config.padding * 3.0 + text_height + button_height;

        let panel = Rectangle::new(
            (screen.x - config.panel_width) / 2.0,
            (screen.y - height) / 2.0,
            config.panel_width,
            height,
        );

        let buttons_y = panel.y + config.padding * 2.0 + text_height;
        let gap = 16.0;
        let total = config.confirm_button.width + gap + config.cancel_button.width;
        let left = (screen.x - total) / 2.0;

        Layout {
            panel,
            lines,
            text_top: panel.y + config.padding,
            cancel: Rectangle::new(
                left,
                buttons_y,
                config.cancel_button.width,
                config.cancel_button.height,
            ),
            confirm: Rectangle::new(
                left + config.cancel_button.width + gap,
                buttons_y,
                config.confirm_button.width,
                config.confirm_button.height,
            ),
        }
    }
}

impl Overlay for ConfirmDialog {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        if self.request.is_none() {
            self.request = ctx.take_confirm_request();
            if self.request.is_none() {
                return OverlayAction::Close;
            }
            return OverlayAction::Stay;
        }

        let pressed = |keys: &[KeyboardKey]| keys.iter().any(|&key| ctx.rl.is_key_pressed(key));
        let (confirm_key, cancel_key) = (
            pressed(&self.config.confirm_keys),
            pressed(&self.config.cancel_keys),
        );

        let layout = self.layout(&ctx.resources.fonts, ui::screen_size(ctx.rl));

        let config = Rc::clone(&self.config);
        let cancel_clicked =
            ui::button::button_clicked(&mut ctx, layout.cancel, &config.cancel_button);
        let confirm_clicked =
            ui::button::button_clicked(&mut ctx, layout.confirm, &config.confirm_button);

        let rects = [layout.cancel, layout.confirm];
        let hovered = [
            ui::button::button_hovered(ctx.rl, layout.cancel, &config.cancel_button),
            ui::button::button_hovered(ctx.rl, layout.confirm, &config.confirm_button),
        ]
        .iter()
        .position(|&hovered| hovered);
        let had_focus = self.focus.index().is_some();
        let accepted = self.focus.update(&ctx.nav, &rects, &[true, true], hovered);
        let confirm_key = confirm_key && !(had_focus && ctx.nav.accept);

        if cancel_key || cancel_clicked || ctx.nav.back || accepted == Some(CANCEL) {
            return OverlayAction::Close;
        }

        if confirm_key || confirm_clicked || accepted == Some(CONFIRM) {
            let action = self.request.take().map(|r| r.action);
            ctx.close_overlay();
            if let Some(state) = action.and_then(|action| action.run(&mut ctx)) {
                return OverlayAction::Goto(state);
            }
        }

        OverlayAction::Stay
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let Some(request) = &self.request else {
            return;
        };
        let config = &self.config;
        let fonts = ctx.fonts();
        let screen = ui::screen_size(d);
        let layout = self.layout(fonts, screen);

        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, config.backdrop);
        config.panel.draw(d, layout.panel);

        let style = &config.message_text;
        for (i, line) in layout.lines.iter().enumerate() {
            let y = layout.text_top + i as f32 * style.size * 1.3 + style.size / 2.0;
            ui::draw_text_centered(d, fonts, line, Vector2::new(screen.x / 2.0, y), style);
        }

        let confirm_label = request
            .confirm_label
            .as_deref()
            .unwrap_or(&config.confirm_label);
        let confirm_label = ctx.label(confirm_label);
        let cancel_label = request
            .cancel_label
            .as_deref()
            .unwrap_or(&config.cancel_label);
        let cancel_label = ctx.label(cancel_label);

        ui::button::Button::new(cancel_label, &config.cancel_button)
            .focused(ctx.shows_focus(&self.focus, CANCEL))
            .draw(d, ctx, layout.cancel);
        ui::button::Button::new(confirm_label, &config.confirm_button)
            .focused(ctx.shows_focus(&self.focus, CONFIRM))
            .draw(d, ctx, layout.confirm);
    }
}
