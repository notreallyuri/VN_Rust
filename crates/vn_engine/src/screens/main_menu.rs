use std::rc::Rc;

use raylib::prelude::*;

use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{
    Action, DrawContext, Focus, FontRole, GameContext, GameView, Screen, ScreenState, StyleOverride,
};
use crate::{Anchor, Corners, Fonts, Layout, PanelStyle, Scenery, TextAlign};

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
    pub title_align: TextAlign,
    pub subtitle: Option<String>,
    pub subtitle_text: TextStyle,
    pub items: Vec<MenuItem>,
    pub button: ButtonStyle,
    pub buttons_y: f32,
    pub layout: Layout,
    pub margin: f32,
    pub background: Option<Background>,
    pub panel: Option<PanelStyle>,
    pub panel_padding: f32,
    pub panel_sidebar: bool,
    pub scenery: Scenery,
    pub buttons_in_bar: bool,
    pub separator: Option<(PanelStyle, f32)>,
    pub intro: f64,
    custom_items: bool,
}

impl Default for MainMenuConfig {
    fn default() -> Self {
        Self {
            title: None,
            title_text: TextStyle::new(FontRole::Title, 64.0, Color::RAYWHITE),
            title_y: 0.25,
            title_align: TextAlign::Center,
            subtitle: None,
            subtitle_text: TextStyle::new(FontRole::Menu, 20.0, Color::LIGHTGRAY),
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
            panel: None,
            panel_padding: 40.0,
            panel_sidebar: false,
            scenery: Scenery::default(),
            buttons_in_bar: false,
            separator: None,
            intro: 0.0,
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

    pub fn title_align(mut self, align: TextAlign) -> Self {
        self.title_align = align;
        self
    }

    pub fn subtitle(mut self, text: impl Into<String>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    pub fn subtitle_text(mut self, style: TextStyle) -> Self {
        self.subtitle_text = style;
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = Some(style(self.panel.unwrap_or_default()));
        self
    }

    pub fn panel_padding(mut self, padding: f32) -> Self {
        self.panel_padding = padding;
        self
    }

    pub fn panel_sidebar(mut self, sidebar: bool) -> Self {
        self.panel_sidebar = sidebar;
        self
    }

    pub fn scenery(mut self, scenery: impl FnOnce(Scenery) -> Scenery) -> Self {
        self.scenery = scenery(self.scenery);
        self
    }

    pub fn buttons_in_bar(mut self, in_bar: bool) -> Self {
        self.buttons_in_bar = in_bar;
        self
    }

    pub fn separator(mut self, size: f32, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        let panel = style(PanelStyle::default().corners(Corners::bevel(size / 2.0)));
        self.separator = Some((panel, size));
        self
    }

    pub fn intro(mut self, seconds: f64) -> Self {
        self.intro = seconds.max(0.0);
        self
    }

    pub fn separator_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let Some((_, size)) = self.separator else {
            return Vec::new();
        };
        self.button_rects(screen)
            .windows(2)
            .filter(|pair| (pair[0].y - pair[1].y).abs() < 0.5 && pair[1].x > pair[0].x)
            .map(|pair| {
                let x = (pair[0].x + pair[0].width + pair[1].x) / 2.0;
                let y = pair[0].y + pair[0].height / 2.0;
                Rectangle::new(x - size / 2.0, y - size / 2.0, size, size)
            })
            .collect()
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

    fn block(&self, screen: Vector2) -> Rectangle {
        let rects = self.button_rects(screen);
        let left = rects.iter().map(|r| r.x).fold(f32::MAX, f32::min);
        let top = rects.iter().map(|r| r.y).fold(f32::MAX, f32::min);
        let right = rects.iter().map(|r| r.x + r.width).fold(f32::MIN, f32::max);
        let bottom = rects
            .iter()
            .map(|r| r.y + r.height)
            .fold(f32::MIN, f32::max);
        if rects.is_empty() {
            return Rectangle::new(screen.x / 2.0, screen.y * self.buttons_y, 0.0, 0.0);
        }
        Rectangle::new(left, top, right - left, bottom - top)
    }

    pub fn title_rects(&self, screen: Vector2, fonts: &Fonts, title: &str) -> Vec<Rectangle> {
        let block = self.block(screen);
        let place = |size: Vector2, center_y: f32| {
            let x = match self.title_align {
                TextAlign::Left => block.x,
                TextAlign::Center => (screen.x - size.x) / 2.0,
                TextAlign::Right => block.x + block.width - size.x,
            };
            Rectangle::new(x, center_y - size.y / 2.0, size.x, size.y)
        };
        let title_style = &self.title_text;
        let title_size = ui::measure_text(fonts, title, title_style);
        let title_rect = place(title_size, screen.y * self.title_y);
        let mut rects = vec![title_rect];
        if let Some(subtitle) = &self.subtitle {
            let style = &self.subtitle_text;
            let size = ui::measure_text(fonts, subtitle, style);
            let center = title_rect.y + title_rect.height + size.y * 0.9;
            rects.push(place(size, center));
        }
        rects
    }

    pub fn panel_rect(&self, screen: Vector2, fonts: &Fonts, title: &str) -> Rectangle {
        let block = self.block(screen);
        let mut left = block.x;
        let mut top = block.y;
        let mut right = block.x + block.width;
        let mut bottom = block.y + block.height;
        for rect in self.title_rects(screen, fonts, title) {
            left = left.min(rect.x);
            top = top.min(rect.y);
            right = right.max(rect.x + rect.width);
            bottom = bottom.max(rect.y + rect.height);
        }
        let pad = self.panel_padding;
        if self.panel_sidebar {
            const BLEED: f32 = 8.0;
            let (x, width) = if (left + right) / 2.0 <= screen.x / 2.0 {
                (-BLEED, right + pad + BLEED)
            } else {
                (left - pad, screen.x - left + pad + BLEED)
            };
            Rectangle::new(x, -BLEED, width, screen.y + BLEED * 2.0)
        } else {
            Rectangle::new(
                left - pad,
                top - pad,
                right - left + pad * 2.0,
                bottom - top + pad * 2.0,
            )
        }
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
        let bar = self
            .buttons_in_bar
            .then(|| self.scenery.bottom_bar(screen, f64::MAX))
            .flatten();
        if let Some(bar) = bar {
            let area = Rectangle::new(self.margin, bar.y, screen.x - self.margin * 2.0, bar.height);
            return self
                .layout
                .place(area, &sizes)
                .into_iter()
                .zip(styles)
                .collect();
        }

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
        ui::Button::new(ctx.label(&item.label), &style)
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
    opened: Option<(f64, Intro)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Intro {
    None,
    Buttons,
    All,
}

fn ease(t: f64) -> f32 {
    let t = t.clamp(0.0, 1.0);
    (t * t * (3.0 - 2.0 * t)) as f32
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
            opened: None,
        }
    }

    fn fades(&self, now: f64) -> (f32, f32) {
        let Some((opened, intro)) = self.opened else {
            return (0.0, 0.0);
        };
        let seconds = self.config.intro;
        if seconds <= 0.0 || intro == Intro::None {
            return (1.0, 1.0);
        }
        let since = now - opened;
        match intro {
            Intro::All => (
                ease(since / (seconds * 0.6)),
                ease((since - seconds * 0.45) / (seconds * 0.55)),
            ),
            _ => (1.0, ease(since / seconds)),
        }
    }
}

impl Screen for MainMenuScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());
        if self.opened.is_none() {
            let intro = match ctx.previous {
                None => Intro::All,
                Some(ScreenState::StartScreen) => Intro::Buttons,
                Some(_) => Intro::None,
            };
            self.opened = Some((ctx.rl.get_time(), intro));
        }

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

        config
            .scenery
            .draw_background(d, ctx.resources, config.background.as_ref());
        let since_open = self.opened.map_or(0.0, |(at, _)| d.get_time() - at);
        config.scenery.draw_frame(d, since_open);
        let (title_fade, buttons_fade) = self.fades(d.get_time());

        let fonts = ctx.fonts();
        if let Some(panel) = &config.panel {
            panel.draw(d, config.panel_rect(screen, fonts, &self.title));
        }

        let rects = config.title_rects(screen, fonts, &self.title);
        let texts = [
            (Some(&self.title), &config.title_text),
            (config.subtitle.as_ref(), &config.subtitle_text),
        ];
        for (rect, (text, style)) in rects.iter().zip(texts) {
            if let Some(text) = text.map(|text| ctx.label(text)) {
                let style = style
                    .clone()
                    .color(style.color.alpha(title_fade * style.color.a as f32 / 255.0));
                ui::draw_text(d, fonts, text, Vector2::new(rect.x, rect.y), &style);
            }
        }

        if let Some((separator, _)) = &config.separator {
            for rect in config.separator_rects(screen) {
                separator.draw_faded(d, rect, buttons_fade);
            }
        }

        let view = ctx.view();
        for (index, (item, (rect, style))) in config
            .items
            .iter()
            .zip(config.placement(screen))
            .enumerate()
        {
            ui::Button::new(ctx.label(&item.label), &style)
                .disabled(!item.is_enabled(&view))
                .focused(ctx.shows_focus(&self.focus, index))
                .opacity(buttons_fade)
                .draw(d, ctx, rect);
        }
    }
}
