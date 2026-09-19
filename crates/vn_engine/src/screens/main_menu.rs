use std::rc::Rc;

use raylib::prelude::*;

use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{
    Action, DrawContext, Focus, FontRole, GameContext, GameView, Screen, ScreenState, StyleOverride,
};
use crate::{Anchor, Layout};

type EnabledCheck = Rc<dyn Fn(&GameView) -> bool>;

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: Action,
    pub tooltip: Option<String>,
    style: Option<StyleOverride>,
    enabled: Option<EnabledCheck>,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
            tooltip: None,
            style: None,
            enabled: None,
        }
    }

    pub fn enabled_if(mut self, check: impl Fn(&GameView) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(check));
        self
    }

    pub fn is_enabled(&self, view: &GameView) -> bool {
        self.enabled.as_ref().is_none_or(|check| check(view))
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn style(mut self, style: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = Some(StyleOverride::new(style));
        self
    }

    pub(crate) fn resolve_style(&self, base: &ButtonStyle) -> ButtonStyle {
        match &self.style {
            Some(style) => style.apply(base),
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
    pub layout: Layout,
    pub margin: f32,
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
                MenuItem::new("Continue", Action::Continue).enabled_if(can_continue),
                MenuItem::new("Load", Action::Goto(ScreenState::Load)),
                MenuItem::new("Settings", Action::Goto(ScreenState::Settings)),
                MenuItem::new("Quit", Action::Quit),
            ],
            button: ButtonStyle::default(),
            buttons_y: 0.45,
            layout: Layout::default().anchor(Anchor::Top).spacing(18.0),
            margin: 40.0,
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
        self.layout = self.layout.spacing(spacing);
        self
    }

    pub fn layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.layout = layout(self.layout);
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn button_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        self.placement(screen)
            .into_iter()
            .map(|(rect, _)| rect)
            .collect()
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    fn placement(&self, screen: Vector2) -> Vec<(Rectangle, ButtonStyle)> {
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

        let bottom = screen.y - self.margin;
        let preferred = screen.y * self.buttons_y;
        let below_title = screen.y * self.title_y + self.title_text.size;
        let top = preferred
            .min(bottom - block.y)
            .max(below_title.min(preferred));
        let area = Rectangle::new(self.margin, top, screen.x - self.margin * 2.0, bottom - top);

        self.layout
            .place(area, &sizes)
            .into_iter()
            .zip(styles)
            .collect()
    }
}

pub fn can_continue(view: &GameView) -> bool {
    view.story.current().is_some() || view.saves.has_any()
}

pub(crate) fn clicked_item(
    ctx: &mut GameContext,
    items: &[MenuItem],
    placement: &[(Rectangle, ButtonStyle)],
    focus: &mut Focus,
) -> Option<usize> {
    let enabled: Vec<bool> = items
        .iter()
        .map(|item| item.is_enabled(&ctx.view()))
        .collect();
    let rects: Vec<Rectangle> = placement.iter().map(|(rect, _)| *rect).collect();
    let hovered = placement
        .iter()
        .position(|(rect, style)| ui::button_hovered(ctx.rl, *rect, style));

    let mut clicked = None;
    for (index, (rect, style)) in placement.iter().enumerate() {
        if enabled[index] && ui::button_clicked(ctx, *rect, style) && clicked.is_none() {
            clicked = Some(index);
        }
    }
    let accepted = focus.update(&ctx.nav, &rects, &enabled, hovered);
    clicked.or(accepted)
}

pub(crate) fn draw_items(
    d: &mut RaylibDrawHandle,
    ctx: &DrawContext,
    items: &[MenuItem],
    placement: Vec<(Rectangle, ButtonStyle)>,
    focus: &Focus,
) {
    let view = ctx.view();
    for (index, (item, (rect, style))) in items.iter().zip(placement).enumerate() {
        ui::Button::new(&item.label, &style)
            .disabled(!item.is_enabled(&view))
            .focused(ctx.shows_focus(focus, index))
            .draw(d, ctx, rect);
    }
}

pub(crate) fn register_tooltips(
    ctx: &mut GameContext,
    items: &[MenuItem],
    rects: impl Iterator<Item = Rectangle>,
) {
    for (item, rect) in items.iter().zip(rects) {
        if let Some(text) = &item.tooltip {
            ctx.tooltip(rect, text.clone());
        }
    }
}

pub struct MainMenuScreen {
    config: Rc<MainMenuConfig>,
    title: String,
    focus: Focus,
}

impl MainMenuScreen {
    pub fn new(config: Rc<MainMenuConfig>, app_title: &str) -> Self {
        let title = config
            .title
            .clone()
            .unwrap_or_else(|| app_title.to_string());
        Self {
            config,
            title,
            focus: Focus::default(),
        }
    }
}

impl Screen for MainMenuScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());

        let layout = self.config.placement(ui::screen_size(ctx.rl));
        register_tooltips(
            &mut ctx,
            &self.config.items,
            layout.iter().map(|(rect, _)| *rect),
        );
        let clicked = clicked_item(&mut ctx, &self.config.items, &layout, &mut self.focus)?;
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

        draw_items(d, ctx, &config.items, config.placement(screen), &self.focus);
    }
}
