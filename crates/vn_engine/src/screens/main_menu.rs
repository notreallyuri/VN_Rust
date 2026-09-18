use std::rc::Rc;

use raylib::prelude::*;

use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{Action, DrawContext, FontRole, GameContext, Screen, ScreenState};

type StyleOverride = Rc<dyn Fn(ButtonStyle) -> ButtonStyle>;

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: Action,
    style: Option<StyleOverride>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
            style: None,
        }
    }

    pub fn style(mut self, style: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = Some(Rc::new(style));
        self
    }

    pub(crate) fn resolve_style(&self, base: &ButtonStyle) -> ButtonStyle {
        match &self.style {
            Some(style) => style(base.clone()),
            None => base.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MainMenuConfig {
    pub title: Option<String>,
    pub title_text: TextStyle,
    pub title_y: f32,
    pub items: Vec<MenuItem>,
    pub button: ButtonStyle,
    pub buttons_y: f32,
    pub spacing: f32,
    pub background: Option<Background>,
    custom_items: bool,
}

impl Default for MainMenuConfig {
    fn default() -> Self {
        Self {
            title: None,
            title_text: TextStyle::new(FontRole::Title, 64.0, Color::RAYWHITE),
            title_y: 0.25,
            items: vec![
                MenuItem::new("New Game", Action::NewGame),
                MenuItem::new("Load", Action::Goto(ScreenState::Load)),
                MenuItem::new("Quit", Action::Quit),
            ],
            button: ButtonStyle::default(),
            buttons_y: 0.45,
            spacing: 18.0,
            background: None,
            custom_items: false,
        }
    }
}

impl MainMenuConfig {
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn title_text(mut self, style: TextStyle) -> Self {
        self.title_text = style;
        self
    }

    pub fn title_y(mut self, fraction: f32) -> Self {
        self.title_y = fraction;
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

    pub fn buttons_y(mut self, fraction: f32) -> Self {
        self.buttons_y = fraction;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    fn layout(&self, screen: Vector2) -> Vec<(Rectangle, ButtonStyle)> {
        let styles: Vec<ButtonStyle> = self
            .items
            .iter()
            .map(|item| item.resolve_style(&self.button))
            .collect();

        let mut y = screen.y * self.buttons_y;
        styles
            .into_iter()
            .map(|style| {
                let rect =
                    Rectangle::new((screen.x - style.width) / 2.0, y, style.width, style.height);
                y += style.height + self.spacing;
                (rect, style)
            })
            .collect()
    }
}

pub struct MainMenuScreen {
    config: Rc<MainMenuConfig>,
    title: String,
}

impl MainMenuScreen {
    pub fn new(config: Rc<MainMenuConfig>, app_title: &str) -> Self {
        let title = config
            .title
            .clone()
            .unwrap_or_else(|| app_title.to_string());
        Self { config, title }
    }
}

impl Screen for MainMenuScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());

        let layout = self.config.layout(ui::screen_size(ctx.rl));
        let clicked = layout
            .iter()
            .position(|(rect, _)| ui::is_clicked(ctx.rl, *rect))?;

        self.config.items[clicked].action.run(&mut ctx)
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);

        ui::draw_background(d, ctx.resources, config.background.as_ref());

        let fonts = ctx.fonts();
        ui::draw_text_centered(
            d,
            fonts,
            &self.title,
            Vector2::new(screen.x / 2.0, screen.y * config.title_y),
            &config.title_text,
        );

        for (item, (rect, style)) in config.items.iter().zip(config.layout(screen)) {
            ui::draw_button(d, fonts, rect, &item.label, &style);
        }
    }
}
