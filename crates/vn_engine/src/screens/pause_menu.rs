use std::rc::Rc;

use raylib::prelude::*;

use crate::Layout;
use crate::screens::{MenuItem, SETTINGS_OVERLAY};
use crate::ui::{self, ButtonStyle, TextStyle};
use crate::{Action, DrawContext, FontRole, GameContext, Overlay, OverlayAction, ScreenState};

pub const PAUSE_OVERLAY: &str = "pause";
pub const SAVE_OVERLAY: &str = "save";
pub const LOAD_OVERLAY: &str = "load";

#[derive(Clone)]
pub struct PauseMenuConfig {
    pub title: String,
    pub title_text: TextStyle,
    pub items: Vec<MenuItem>,
    pub button: ButtonStyle,
    pub layout: Layout,
    pub padding: f32,
    pub panel_width: f32,
    pub panel_color: Color,
    pub panel_roundness: f32,
    pub backdrop: Color,
    pub close_keys: Vec<KeyboardKey>,
    custom_items: bool,
}

impl Default for PauseMenuConfig {
    fn default() -> Self {
        Self {
            title: "Paused".to_string(),
            title_text: TextStyle::new(FontRole::Title, 40.0, Color::RAYWHITE),
            items: vec![
                MenuItem::new("Resume", Action::Resume),
                MenuItem::new("Save", Action::overlay(SAVE_OVERLAY)),
                MenuItem::new("Load", Action::overlay(LOAD_OVERLAY)),
                MenuItem::new("Quick Save", Action::QuickSave),
                MenuItem::new("Quick Load", Action::QuickLoad),
                MenuItem::new("Settings", Action::overlay(SETTINGS_OVERLAY)),
                MenuItem::new(
                    "Main Menu",
                    Action::confirm(
                        "Return to the main menu? Unsaved progress will be lost.",
                        Action::Goto(ScreenState::MainMenu),
                    ),
                ),
                MenuItem::new(
                    "Quit",
                    Action::confirm(
                        "Quit the game? Unsaved progress will be lost.",
                        Action::Quit,
                    ),
                ),
            ],
            button: ButtonStyle::default().size(260.0, 44.0).font_size(20.0),
            layout: Layout::default().spacing(10.0),
            padding: 28.0,
            panel_width: 340.0,
            panel_color: Color::new(18, 18, 28, 240),
            panel_roundness: 0.04,
            backdrop: Color::new(0, 0, 0, 150),
            close_keys: vec![KeyboardKey::KEY_ESCAPE],
            custom_items: false,
        }
    }
}

impl PauseMenuConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        if !self.custom_items {
            self.items.clear();
            self.custom_items = true;
        }
        self.items.push(item);
        self
    }

    pub fn button(self, label: impl Into<String>, action: Action) -> Self {
        self.item(MenuItem::new(label, action))
    }

    pub fn button_style(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.button = style(self.button);
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.layout = self.layout.spacing(spacing);
        self
    }

    pub fn layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.layout = layout(self.layout);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width;
        self
    }

    pub fn panel_color(mut self, color: Color) -> Self {
        self.panel_color = color;
        self
    }

    pub fn panel_roundness(mut self, roundness: f32) -> Self {
        self.panel_roundness = roundness.clamp(0.0, 1.0);
        self
    }

    pub fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = color;
        self
    }

    pub fn close_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.close_keys = keys.into_iter().collect();
        self
    }

    pub fn panel(&self, screen: Vector2) -> Rectangle {
        self.placement(screen).0
    }

    pub fn button_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        self.placement(screen)
            .2
            .into_iter()
            .map(|(rect, _)| rect)
            .collect()
    }

    fn placement(&self, screen: Vector2) -> (Rectangle, f32, Vec<(Rectangle, ButtonStyle)>) {
        let styles: Vec<ButtonStyle> = self
            .items
            .iter()
            .map(|item| item.resolve_style(&self.button))
            .collect();

        let sizes: Vec<Vector2> = styles
            .iter()
            .map(|style| Vector2::new(style.width, style.height))
            .collect();
        let block = self.layout.block_size(&sizes);

        let title_height = self.title_text.size * 1.6;
        let width = self.panel_width.max(block.x + self.padding * 2.0);
        let height = self.padding * 2.0 + title_height + block.y;

        let panel = Rectangle::new(
            (screen.x - width) / 2.0,
            (screen.y - height) / 2.0,
            width,
            height,
        );
        let title_center_y = panel.y + self.padding + self.title_text.size * 0.5;

        let area = Rectangle::new(
            panel.x + self.padding,
            panel.y + self.padding + title_height,
            width - self.padding * 2.0,
            block.y,
        );
        let buttons = self
            .layout
            .place(area, &sizes)
            .into_iter()
            .zip(styles)
            .collect();

        (panel, title_center_y, buttons)
    }
}

pub struct PauseMenu {
    config: Rc<PauseMenuConfig>,
}

impl PauseMenu {
    pub fn new(config: Rc<PauseMenuConfig>) -> Self {
        Self { config }
    }
}

impl Overlay for PauseMenu {
    fn update(&mut self, mut ctx: GameContext) -> OverlayAction {
        let close = self
            .config
            .close_keys
            .iter()
            .any(|&key| ctx.rl.is_key_pressed(key));
        if close {
            return OverlayAction::CloseAll;
        }

        let (_, _, buttons) = self.config.placement(ui::screen_size(ctx.rl));
        let Some(index) = buttons
            .iter()
            .position(|(rect, _)| ui::is_clicked(ctx.rl, *rect))
        else {
            return OverlayAction::Stay;
        };

        match self.config.items[index].action.run(&mut ctx) {
            Some(state) => OverlayAction::Goto(state),
            None => OverlayAction::Stay,
        }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);
        let fonts = ctx.fonts();
        let (panel, title_y, buttons) = config.placement(screen);

        d.draw_rectangle(0, 0, screen.x as i32, screen.y as i32, config.backdrop);
        if config.panel_roundness > 0.0 {
            d.draw_rectangle_rounded(panel, config.panel_roundness, 8, config.panel_color);
        } else {
            d.draw_rectangle_rec(panel, config.panel_color);
        }

        ui::draw_text_centered(
            d,
            fonts,
            &config.title,
            Vector2::new(screen.x / 2.0, title_y),
            &config.title_text,
        );

        for (item, (rect, style)) in config.items.iter().zip(buttons) {
            ui::draw_button(d, fonts, rect, &item.label, &style);
        }
    }
}
