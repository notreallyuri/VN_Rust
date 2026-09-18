use std::rc::Rc;

use raylib::prelude::*;
use vn_script::{Event, Position, StoryVm};

use crate::screens::PAUSE_OVERLAY;
use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{
    Action, Anchor, DrawContext, FontRole, GameContext, Layout, ResourceManager, Screen,
    ScreenState, background_path, character_path,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DialogueBoxStyle {
    pub height: f32,
    pub margin: f32,
    pub padding: f32,
    pub color: Color,
    pub roundness: f32,
}

impl Default for DialogueBoxStyle {
    fn default() -> Self {
        Self {
            height: 170.0,
            margin: 40.0,
            padding: 24.0,
            color: Color::new(0, 0, 0, 200),
            roundness: 0.0,
        }
    }
}

impl DialogueBoxStyle {
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn roundness(mut self, roundness: f32) -> Self {
        self.roundness = roundness.clamp(0.0, 1.0);
        self
    }

    fn rect(&self, screen: Vector2) -> Rectangle {
        Rectangle::new(
            self.margin,
            screen.y - self.height - self.margin,
            screen.x - self.margin * 2.0,
            self.height,
        )
    }
}

#[derive(Clone, Debug)]
pub struct HudButton {
    pub label: String,
    pub action: Action,
}

#[derive(Clone, Debug)]
pub struct PlayingConfig {
    pub dialogue_box: DialogueBoxStyle,
    pub speaker_text: TextStyle,
    pub dialogue_text: TextStyle,
    pub choice_button: ButtonStyle,
    pub choice_layout: Layout,
    pub end_title: String,
    pub end_title_text: TextStyle,
    pub end_hint: String,
    pub end_hint_text: TextStyle,
    pub advance_keys: Vec<KeyboardKey>,
    pub menu_key: Option<KeyboardKey>,
    pub after_end: ScreenState,
    pub background: Option<Background>,
    pub positions: [f32; 5],
    pub character_height: Option<f32>,
    pub hud: Vec<HudButton>,
    pub hud_button: ButtonStyle,
    pub hud_margin: f32,
    pub hud_layout: Layout,
    pub quick_save_key: Option<KeyboardKey>,
    pub quick_load_key: Option<KeyboardKey>,
    pub pause_key: Option<KeyboardKey>,
    pub pause_overlay: String,
}

impl Default for PlayingConfig {
    fn default() -> Self {
        Self {
            dialogue_box: DialogueBoxStyle::default(),
            speaker_text: TextStyle::new(FontRole::Speaker, 24.0, Color::GOLD),
            dialogue_text: TextStyle::new(FontRole::Dialogue, 26.0, Color::RAYWHITE),
            choice_button: ButtonStyle::default()
                .size(720.0, 56.0)
                .color(Color::new(30, 30, 45, 230))
                .font(FontRole::Choice),
            choice_layout: Layout::default().spacing(16.0),
            end_title: "The End".to_string(),
            end_title_text: TextStyle::new(FontRole::Title, 56.0, Color::RAYWHITE),
            end_hint: "Click to return to the menu".to_string(),
            end_hint_text: TextStyle::new(FontRole::Menu, 20.0, Color::GRAY),
            advance_keys: vec![KeyboardKey::KEY_SPACE, KeyboardKey::KEY_ENTER],
            menu_key: None,
            after_end: ScreenState::MainMenu,
            background: None,
            positions: [0.15, 0.3, 0.5, 0.7, 0.85],
            character_height: Some(0.8),
            hud: Vec::new(),
            hud_button: ButtonStyle::default()
                .size(130.0, 40.0)
                .color(Color::new(20, 20, 30, 190))
                .font_size(18.0),
            hud_margin: 16.0,
            hud_layout: Layout::default()
                .row()
                .anchor(Anchor::TopRight)
                .spacing(10.0),
            quick_save_key: Some(KeyboardKey::KEY_F5),
            quick_load_key: Some(KeyboardKey::KEY_F9),
            pause_key: Some(KeyboardKey::KEY_ESCAPE),
            pause_overlay: PAUSE_OVERLAY.to_string(),
        }
    }
}

impl PlayingConfig {
    pub fn dialogue_box(
        mut self,
        style: impl FnOnce(DialogueBoxStyle) -> DialogueBoxStyle,
    ) -> Self {
        self.dialogue_box = style(self.dialogue_box);
        self
    }

    pub fn speaker_text(mut self, style: TextStyle) -> Self {
        self.speaker_text = style;
        self
    }

    pub fn dialogue_text(mut self, style: TextStyle) -> Self {
        self.dialogue_text = style;
        self
    }

    pub fn choice_button(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.choice_button = style(self.choice_button);
        self
    }

    pub fn choice_spacing(mut self, spacing: f32) -> Self {
        self.choice_layout = self.choice_layout.spacing(spacing);
        self
    }

    pub fn choice_layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.choice_layout = layout(self.choice_layout);
        self
    }

    pub fn end_title(mut self, text: impl Into<String>) -> Self {
        self.end_title = text.into();
        self
    }

    pub fn end_title_text(mut self, style: TextStyle) -> Self {
        self.end_title_text = style;
        self
    }

    pub fn end_hint(mut self, text: impl Into<String>) -> Self {
        self.end_hint = text.into();
        self
    }

    pub fn end_hint_text(mut self, style: TextStyle) -> Self {
        self.end_hint_text = style;
        self
    }

    pub fn advance_keys(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.advance_keys = keys.into_iter().collect();
        self
    }

    pub fn menu_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.menu_key = key;
        self
    }

    pub fn after_end(mut self, state: ScreenState) -> Self {
        self.after_end = state;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    pub fn position(mut self, position: Position, x: f32) -> Self {
        self.positions[position_index(position)] = x;
        self
    }

    pub fn position_x(&self, position: Position) -> f32 {
        self.positions[position_index(position)]
    }

    pub fn character_height(mut self, fraction: Option<f32>) -> Self {
        self.character_height = fraction;
        self
    }

    pub fn hud_button(mut self, label: impl Into<String>, action: Action) -> Self {
        self.hud.push(HudButton {
            label: label.into(),
            action,
        });
        self
    }

    pub fn hud_button_style(mut self, style: impl FnOnce(ButtonStyle) -> ButtonStyle) -> Self {
        self.hud_button = style(self.hud_button);
        self
    }

    pub fn hud_margin(mut self, margin: f32) -> Self {
        self.hud_margin = margin;
        self
    }

    pub fn hud_spacing(mut self, spacing: f32) -> Self {
        self.hud_layout = self.hud_layout.spacing(spacing);
        self
    }

    pub fn hud_layout(mut self, layout: impl FnOnce(Layout) -> Layout) -> Self {
        self.hud_layout = layout(self.hud_layout);
        self
    }

    pub fn quick_save_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.quick_save_key = key;
        self
    }

    pub fn quick_load_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.quick_load_key = key;
        self
    }

    pub fn pause_key(mut self, key: Option<KeyboardKey>) -> Self {
        self.pause_key = key;
        self
    }

    pub fn pause_overlay(mut self, name: impl Into<String>) -> Self {
        self.pause_overlay = name.into();
        self
    }

    pub fn hud_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let button = &self.hud_button;
        let sizes = vec![Vector2::new(button.width, button.height); self.hud.len()];
        self.hud_layout
            .place(inset(screen, self.hud_margin), &sizes)
    }

    pub fn choice_rects(&self, count: usize, screen: Vector2) -> Vec<Rectangle> {
        let button = &self.choice_button;
        let sizes = vec![Vector2::new(button.width, button.height); count];
        self.choice_layout
            .place(inset(screen, self.dialogue_box.margin), &sizes)
    }
}

fn position_index(position: Position) -> usize {
    Position::ALL
        .iter()
        .position(|p| *p == position)
        .expect("every position is in Position::ALL")
}

fn inset(screen: Vector2, margin: f32) -> Rectangle {
    Rectangle::new(
        margin,
        margin,
        (screen.x - margin * 2.0).max(0.0),
        (screen.y - margin * 2.0).max(0.0),
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typewriter {
    started: f64,
    chars_per_second: u32,
    total: usize,
    finished: bool,
}

impl Typewriter {
    pub fn start(text: &str, chars_per_second: u32, now: f64) -> Self {
        Self {
            started: now,
            chars_per_second,
            total: text.chars().count(),
            finished: chars_per_second == 0,
        }
    }

    pub fn finished(text: &str) -> Self {
        Self {
            started: 0.0,
            chars_per_second: 0,
            total: text.chars().count(),
            finished: true,
        }
    }

    pub fn visible(&self, now: f64) -> usize {
        if self.finished {
            return self.total;
        }
        let elapsed = (now - self.started).max(0.0);
        ((elapsed * self.chars_per_second as f64) as usize).min(self.total)
    }

    pub fn is_done(&self, now: f64) -> bool {
        self.visible(now) >= self.total
    }

    pub fn finish(&mut self) {
        self.finished = true;
    }
}

pub struct PlayingScreen {
    config: Rc<PlayingConfig>,
    current: Option<Event>,
    typewriter: Option<Typewriter>,
    visible: Option<usize>,
}

impl PlayingScreen {
    pub fn new(config: Rc<PlayingConfig>) -> Self {
        Self {
            config,
            current: None,
            typewriter: None,
            visible: None,
        }
    }

    fn show(&mut self, event: Event, typed: Option<(u32, f64)>) {
        self.typewriter = match (&event, typed) {
            (Event::Say { text, .. }, Some((speed, now))) => {
                Some(Typewriter::start(text, speed, now))
            }
            _ => None,
        };
        self.current = Some(event);
    }

    fn typing(&self, now: f64) -> bool {
        self.typewriter
            .as_ref()
            .is_some_and(|typewriter| !typewriter.is_done(now))
    }

    fn advance(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        loop {
            match ctx.story.advance() {
                Event::Call { command, args } => {
                    if ctx.rollback.blocks_command(&command) {
                        ctx.rollback.mark_barrier();
                    }
                    if let Some(next) = ctx.run_command(&command, &args) {
                        return Some(next);
                    }
                }
                Event::Commit => ctx.rollback.mark_barrier(),
                Event::SceneEnter { scene } => {
                    if let Some(next) = ctx.run_scene_hooks(&scene) {
                        return Some(next);
                    }
                }
                event if event.is_blocking() => {
                    let typed = (ctx.settings.values.text_speed, ctx.rl.get_time());
                    self.show(event, Some(typed));
                    ctx.rollback.record(ctx.story, ctx.state);
                    return None;
                }
                _ => {}
            }
        }
    }

    fn resume(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        match ctx.story.current() {
            Some(event) => {
                self.show(event.clone(), None);
                ctx.rollback.record(ctx.story, ctx.state);
                None
            }
            None => self.advance(ctx),
        }
    }

    fn roll(&mut self, ctx: &mut GameContext) -> bool {
        let config = ctx.rollback.config();
        let wheel = if config.mouse_wheel {
            ctx.rl.get_mouse_wheel_move()
        } else {
            0.0
        };
        let back = wheel > 0.0 || config.back_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        let forward = wheel < 0.0
            || config
                .forward_keys
                .iter()
                .any(|&k| ctx.rl.is_key_pressed(k));

        let moved = if back {
            ctx.rollback.back(ctx.story, ctx.state)
        } else if forward {
            ctx.rollback.forward(ctx.story, ctx.state)
        } else {
            false
        };

        if moved {
            self.current = ctx.story.current().cloned();
            self.typewriter = None;
        }
        back || forward
    }

    fn shows_hud(&self) -> bool {
        !matches!(self.current, Some(Event::End))
    }

    fn continue_pressed(&self, rl: &RaylibHandle) -> bool {
        rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || self
                .config
                .advance_keys
                .iter()
                .any(|&key| rl.is_key_pressed(key))
    }
}

impl Screen for PlayingScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        ui::load_background(&mut ctx, self.config.background.as_ref());

        let rl = &*ctx.rl;
        let pressed = |key: Option<KeyboardKey>| key.is_some_and(|key| rl.is_key_pressed(key));
        let [menu, pause, quick_save, quick_load] = [
            self.config.menu_key,
            self.config.pause_key,
            self.config.quick_save_key,
            self.config.quick_load_key,
        ]
        .map(pressed);

        if menu {
            return Some(ScreenState::MainMenu);
        }

        if pause {
            ctx.open_overlay(self.config.pause_overlay.clone());
            return None;
        }

        if self.current.is_some() && self.roll(&mut ctx) {
            return None;
        }

        if self.current.is_some() {
            if quick_save {
                return Action::QuickSave.run(&mut ctx);
            }
            if quick_load {
                return Action::QuickLoad.run(&mut ctx);
            }
        }

        if self.shows_hud() {
            let hud = self.config.hud_rects(ui::screen_size(ctx.rl));
            if let Some(index) = hud.iter().position(|rect| ui::is_clicked(ctx.rl, *rect)) {
                let action = self.config.hud[index].action.clone();
                return action.run(&mut ctx);
            }
        }

        let next = match &self.current {
            None => self.resume(&mut ctx),
            Some(Event::Choice { options }) => {
                let rects = self
                    .config
                    .choice_rects(options.len(), ui::screen_size(ctx.rl));

                match rects.iter().position(|rect| ui::is_clicked(ctx.rl, *rect)) {
                    Some(index) => {
                        let text = options[index].clone();
                        if let Err(e) = ctx.story.choose(index) {
                            eprintln!("⚠️ {}", e);
                            return None;
                        }
                        if !ctx.rollback.config().through_choices {
                            ctx.rollback.mark_barrier();
                        }
                        match ctx.run_choice_hooks(index, &text) {
                            Some(next) => Some(next),
                            None => self.advance(&mut ctx),
                        }
                    }
                    None => None,
                }
            }
            Some(Event::End) => self
                .continue_pressed(ctx.rl)
                .then(|| self.config.after_end.clone()),
            Some(_) => {
                let now = ctx.rl.get_time();
                if !self.continue_pressed(ctx.rl) {
                    None
                } else if self.typing(now) {
                    if let Some(typewriter) = &mut self.typewriter {
                        typewriter.finish();
                    }
                    None
                } else {
                    self.advance(&mut ctx)
                }
            }
        };

        let now = ctx.rl.get_time();
        self.visible = self
            .typewriter
            .as_ref()
            .map(|typewriter| typewriter.visible(now));

        if next.is_some() {
            return next;
        }

        for (name, image) in ctx.story.active_characters() {
            let path = character_path(name, image);
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }
        if let Some(image) = ctx.story.background() {
            let path = background_path(image);
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }

        None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);

        match ctx.story.background() {
            Some(image) => match ctx.resources.textures.get(&background_path(image)) {
                Some(texture) => {
                    ui::draw_texture_cover(d, texture, Rectangle::new(0.0, 0.0, screen.x, screen.y))
                }
                None => ui::draw_background(d, ctx.resources, config.background.as_ref()),
            },
            None => ui::draw_background(d, ctx.resources, config.background.as_ref()),
        }
        draw_characters(d, ctx.resources, ctx.story, config, screen);

        let fonts = ctx.fonts();

        if self.shows_hud() {
            for (button, rect) in config.hud.iter().zip(config.hud_rects(screen)) {
                ui::draw_button(d, fonts, rect, &button.label, &config.hud_button);
            }
        }

        match &self.current {
            Some(Event::Say { speaker, text }) => {
                let rect = config.dialogue_box.rect(screen);
                let style = &config.dialogue_box;

                if style.roundness > 0.0 {
                    d.draw_rectangle_rounded(rect, style.roundness, 8, style.color);
                } else {
                    d.draw_rectangle_rec(rect, style.color);
                }

                let inner_x = rect.x + style.padding;
                let inner_width = rect.width - style.padding * 2.0;
                let mut y = rect.y + style.padding;

                if let Some(speaker) = speaker {
                    let name = ctx.characters.display_name(speaker, ctx.story);
                    let style = match ctx.characters.color(speaker) {
                        Some(color) => config.speaker_text.clone().color(color),
                        None => config.speaker_text.clone(),
                    };
                    ui::draw_text(d, fonts, &name, Vector2::new(inner_x, y), &style);
                    y += config.speaker_text.size * 1.4;
                }

                ui::draw_text_wrapped_visible(
                    d,
                    fonts,
                    text,
                    Vector2::new(inner_x, y),
                    inner_width,
                    &config.dialogue_text,
                    self.visible.unwrap_or(usize::MAX),
                );
            }
            Some(Event::Choice { options }) => {
                let rects = config.choice_rects(options.len(), screen);
                for (option, rect) in options.iter().zip(rects) {
                    ui::draw_button(d, fonts, rect, option, &config.choice_button);
                }
            }
            Some(Event::End) => {
                ui::draw_text_centered(
                    d,
                    fonts,
                    &config.end_title,
                    Vector2::new(screen.x / 2.0, screen.y * 0.42),
                    &config.end_title_text,
                );
                ui::draw_text_centered(
                    d,
                    fonts,
                    &config.end_hint,
                    Vector2::new(screen.x / 2.0, screen.y * 0.52),
                    &config.end_hint_text,
                );
            }
            _ => {}
        }
    }
}

fn draw_characters(
    d: &mut RaylibDrawHandle,
    resources: &ResourceManager,
    story: &StoryVm,
    config: &PlayingConfig,
    screen: Vector2,
) {
    let mut characters: Vec<_> = story.active_characters().iter().collect();
    characters.sort();

    let unplaced = characters
        .iter()
        .filter(|(name, _)| story.position(name).is_none())
        .count();
    let mut next_unplaced = 0;

    for (name, image) in characters {
        let Some(texture) = resources.textures.get(&character_path(name, image)) else {
            continue;
        };

        let center_x = match story.position(name) {
            Some(position) => screen.x * config.position_x(position),
            None => {
                next_unplaced += 1;
                screen.x * next_unplaced as f32 / (unplaced as f32 + 1.0)
            }
        };

        let (w, h) = (texture.width as f32, texture.height as f32);
        let scale = config
            .character_height
            .map_or(1.0, |fraction| screen.y * fraction / h);
        let dest = Rectangle::new(
            center_x - w * scale / 2.0,
            screen.y - h * scale,
            w * scale,
            h * scale,
        );
        d.draw_texture_pro(
            texture,
            Rectangle::new(0.0, 0.0, w, h),
            dest,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}
