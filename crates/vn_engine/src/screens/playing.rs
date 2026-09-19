use std::rc::Rc;

use raylib::prelude::*;
use vn_script::{Event, Position};

use crate::screens::{LOG_OVERLAY, PAUSE_OVERLAY};
use crate::ui::{self, Background, ButtonStyle, TextStyle};
use crate::{
    Action, Anchor, DrawContext, Focus, FontRole, GameContext, Layout, NavInput, Screen,
    ScreenState, Stage, StyleOverride, background_path, character_path,
};
use crate::{LogEntry, PanelStyle};

#[derive(Clone, Debug, PartialEq)]
pub struct NamePlate {
    pub panel: PanelStyle,
    pub padding: Vector2,
    pub indent: f32,
    pub overlap: f32,
    pub min_width: f32,
}

impl Default for NamePlate {
    fn default() -> Self {
        Self {
            panel: PanelStyle::new(Color::new(0, 0, 0, 220)),
            padding: Vector2::new(18.0, 6.0),
            indent: 24.0,
            overlap: 12.0,
            min_width: 0.0,
        }
    }
}

impl NamePlate {
    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vector2::new(x, y);
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn overlap(mut self, overlap: f32) -> Self {
        self.overlap = overlap;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    pub fn rect(&self, dialogue_box: Rectangle, name_size: Vector2) -> Rectangle {
        let width = (name_size.x + self.padding.x * 2.0).max(self.min_width);
        let height = name_size.y + self.padding.y * 2.0;
        Rectangle::new(
            dialogue_box.x + self.indent,
            dialogue_box.y - height + self.overlap,
            width,
            height,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogueBoxStyle {
    pub height: f32,
    pub margin: f32,
    pub bottom: Option<f32>,
    pub max_width: Option<f32>,
    pub padding: f32,
    pub panel: PanelStyle,
    pub name_plate: Option<NamePlate>,
}

impl Default for DialogueBoxStyle {
    fn default() -> Self {
        Self {
            height: 170.0,
            margin: 40.0,
            bottom: None,
            max_width: None,
            padding: 24.0,
            panel: PanelStyle::new(Color::new(0, 0, 0, 200)),
            name_plate: None,
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

    pub fn bottom(mut self, distance: f32) -> Self {
        self.bottom = Some(distance);
        self
    }

    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.panel.color = color;
        self
    }

    pub fn roundness(mut self, roundness: f32) -> Self {
        self.panel = self.panel.roundness(roundness);
        self
    }

    pub fn panel(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.panel = style(self.panel);
        self
    }

    pub fn name_plate(mut self, plate: impl FnOnce(NamePlate) -> NamePlate) -> Self {
        self.name_plate = Some(plate(self.name_plate.unwrap_or_default()));
        self
    }

    pub fn rect(&self, screen: Vector2) -> Rectangle {
        let full = screen.x - self.margin * 2.0;
        let width = self.max_width.map_or(full, |max| max.min(full));
        Rectangle::new(
            (screen.x - width) / 2.0,
            screen.y - self.height - self.bottom.unwrap_or(self.margin),
            width,
            self.height,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayingKeys {
    pub hide: Vec<KeyboardKey>,
    pub skip_toggle: Vec<KeyboardKey>,
    pub skip_hold: Vec<KeyboardKey>,
    pub auto: Vec<KeyboardKey>,
    pub log: Vec<KeyboardKey>,
    pub screenshot: Vec<KeyboardKey>,
    pub fullscreen: Vec<KeyboardKey>,
    pub middle_click_hides: bool,
    pub right_click_pauses: bool,
}

impl Default for PlayingKeys {
    fn default() -> Self {
        Self {
            hide: vec![KeyboardKey::KEY_H],
            skip_toggle: vec![KeyboardKey::KEY_TAB],
            skip_hold: vec![
                KeyboardKey::KEY_LEFT_CONTROL,
                KeyboardKey::KEY_RIGHT_CONTROL,
            ],
            auto: vec![KeyboardKey::KEY_A],
            log: vec![KeyboardKey::KEY_L],
            screenshot: vec![KeyboardKey::KEY_S],
            fullscreen: vec![KeyboardKey::KEY_F],
            middle_click_hides: true,
            right_click_pauses: true,
        }
    }
}

impl PlayingKeys {
    pub fn hide(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.hide = keys.into_iter().collect();
        self
    }

    pub fn skip_toggle(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.skip_toggle = keys.into_iter().collect();
        self
    }

    pub fn skip_hold(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.skip_hold = keys.into_iter().collect();
        self
    }

    pub fn auto(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.auto = keys.into_iter().collect();
        self
    }

    pub fn log(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.log = keys.into_iter().collect();
        self
    }

    pub fn screenshot(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.screenshot = keys.into_iter().collect();
        self
    }

    pub fn fullscreen(mut self, keys: impl IntoIterator<Item = KeyboardKey>) -> Self {
        self.fullscreen = keys.into_iter().collect();
        self
    }

    pub fn middle_click_hides(mut self, enabled: bool) -> Self {
        self.middle_click_hides = enabled;
        self
    }

    pub fn right_click_pauses(mut self, enabled: bool) -> Self {
        self.right_click_pauses = enabled;
        self
    }
}

#[derive(Clone, Debug)]
pub struct HudButton {
    pub label: String,
    pub action: Action,
    pub tooltip: Option<String>,
    pub style: Option<StyleOverride>,
    pub group: Option<String>,
}

type ChoiceStyleFn = Rc<dyn Fn(usize, &str, ButtonStyle) -> ButtonStyle>;

#[derive(Clone)]
pub struct ChoiceStyle(ChoiceStyleFn);

impl std::fmt::Debug for ChoiceStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChoiceStyle(..)")
    }
}

impl HudButton {
    pub fn new(label: impl Into<String>, action: Action) -> Self {
        Self {
            label: label.into(),
            action,
            tooltip: None,
            style: None,
            group: None,
        }
    }

    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.group = Some(name.into());
        self
    }

    pub fn style(mut self, style: impl Fn(ButtonStyle) -> ButtonStyle + 'static) -> Self {
        self.style = Some(StyleOverride::new(style));
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}

#[derive(Clone, Debug)]
pub struct PlayingConfig {
    pub dialogue_box: DialogueBoxStyle,
    pub speaker_text: TextStyle,
    pub dialogue_text: TextStyle,
    pub choice_button: ButtonStyle,
    pub choice_style: Option<ChoiceStyle>,
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
    custom_hud: bool,
    pub keys: PlayingKeys,
    pub skip_interval: f64,
    pub auto_per_character: f64,
    pub indicator_text: TextStyle,
    pub indicator: PanelStyle,
    pub skip_label: String,
    pub auto_label: String,
    pub log_overlay: String,
    pub hud_button: ButtonStyle,
    pub hud_margin: f32,
    pub hud_layout: Layout,
    pub hud_groups: Vec<(String, Layout)>,
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
            choice_style: None,
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
            hud: vec![
                HudButton::new("Log", Action::overlay(LOG_OVERLAY))
                    .tooltip("Everything so far (L)"),
                HudButton::new("Auto", Action::ToggleAuto)
                    .tooltip("Advance on its own after each line (A)"),
                HudButton::new("Skip", Action::ToggleSkip)
                    .tooltip("Skip lines you've already seen (Tab, or hold Ctrl)"),
            ],
            custom_hud: false,
            keys: PlayingKeys::default(),
            skip_interval: 0.05,
            auto_per_character: 0.02,
            indicator_text: TextStyle::new(FontRole::Menu, 18.0, Color::RAYWHITE),
            indicator: PanelStyle::new(Color::new(0, 0, 0, 160)).roundness(0.3),
            skip_label: "Skip »".to_string(),
            auto_label: "Auto".to_string(),
            log_overlay: LOG_OVERLAY.to_string(),
            hud_button: ButtonStyle::default()
                .size(130.0, 40.0)
                .color(Color::new(20, 20, 30, 190))
                .font_size(18.0),
            hud_margin: 16.0,
            hud_layout: Layout::default()
                .row()
                .anchor(Anchor::TopRight)
                .spacing(10.0),
            hud_groups: Vec::new(),
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

    pub fn choice_button_for(
        mut self,
        style: impl Fn(usize, &str, ButtonStyle) -> ButtonStyle + 'static,
    ) -> Self {
        self.choice_style = Some(ChoiceStyle(Rc::new(style)));
        self
    }

    pub fn choice_style_for(&self, index: usize, text: &str) -> ButtonStyle {
        match &self.choice_style {
            Some(ChoiceStyle(style)) => style(index, text, self.choice_button.clone()),
            None => self.choice_button.clone(),
        }
    }

    pub fn hud_style(&self, index: usize) -> ButtonStyle {
        match self.hud.get(index).and_then(|button| button.style.as_ref()) {
            Some(style) => style.apply(&self.hud_button),
            None => self.hud_button.clone(),
        }
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

    pub fn hud_button(self, label: impl Into<String>, action: Action) -> Self {
        self.hud_item(HudButton::new(label, action))
    }

    pub fn hud_item(mut self, button: HudButton) -> Self {
        if !self.custom_hud {
            self.hud.clear();
            self.custom_hud = true;
        }
        self.hud.push(button);
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

    pub fn keys(mut self, keys: impl FnOnce(PlayingKeys) -> PlayingKeys) -> Self {
        self.keys = keys(self.keys);
        self
    }

    pub fn skip_interval(mut self, seconds: f64) -> Self {
        self.skip_interval = seconds.max(0.0);
        self
    }

    pub fn auto_per_character(mut self, seconds: f64) -> Self {
        self.auto_per_character = seconds.max(0.0);
        self
    }

    pub fn indicator_text(mut self, style: TextStyle) -> Self {
        self.indicator_text = style;
        self
    }

    pub fn indicator_color(mut self, color: Color) -> Self {
        self.indicator.color = color;
        self
    }

    pub fn indicator(mut self, style: impl FnOnce(PanelStyle) -> PanelStyle) -> Self {
        self.indicator = style(self.indicator);
        self
    }

    pub fn skip_label(mut self, text: impl Into<String>) -> Self {
        self.skip_label = text.into();
        self
    }

    pub fn auto_label(mut self, text: impl Into<String>) -> Self {
        self.auto_label = text.into();
        self
    }

    pub fn log_overlay(mut self, name: impl Into<String>) -> Self {
        self.log_overlay = name.into();
        self
    }

    pub fn auto_delay(&self, settings: &crate::Settings, text: &str) -> f64 {
        settings.auto_seconds() + text.chars().count() as f64 * self.auto_per_character
    }

    pub fn pause_overlay(mut self, name: impl Into<String>) -> Self {
        self.pause_overlay = name.into();
        self
    }

    pub fn hud_group(
        mut self,
        name: impl Into<String>,
        layout: impl FnOnce(Layout) -> Layout,
    ) -> Self {
        let name = name.into();
        let base = self.hud_layout.clone();
        match self.hud_groups.iter_mut().find(|(n, _)| *n == name) {
            Some((_, existing)) => *existing = layout(existing.clone()),
            None => self.hud_groups.push((name, layout(base))),
        }
        self
    }

    fn group_layout(&self, group: Option<&str>) -> &Layout {
        group
            .and_then(|name| self.hud_groups.iter().find(|(n, _)| n == name))
            .map_or(&self.hud_layout, |(_, layout)| layout)
    }

    pub fn hud_rects(&self, screen: Vector2) -> Vec<Rectangle> {
        let area = inset(screen, self.hud_margin);
        let mut rects = vec![Rectangle::new(0.0, 0.0, 0.0, 0.0); self.hud.len()];
        let mut groups: Vec<Option<&str>> = Vec::new();
        for button in &self.hud {
            if !groups.contains(&button.group.as_deref()) {
                groups.push(button.group.as_deref());
            }
        }
        for group in groups {
            let members: Vec<usize> = (0..self.hud.len())
                .filter(|&i| self.hud[i].group.as_deref() == group)
                .collect();
            let sizes: Vec<Vector2> = members
                .iter()
                .map(|&i| {
                    let style = self.hud_style(i);
                    Vector2::new(style.width, style.height)
                })
                .collect();
            let placed = self.group_layout(group).place(area, &sizes);
            for (i, rect) in members.into_iter().zip(placed) {
                rects[i] = rect;
            }
        }
        rects
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
    autosave_pending: bool,
    choice_focus: Focus,
    stage: Stage,
    hidden: bool,
    skipping: bool,
    last_skip: f64,
    ready_since: Option<f64>,
}

impl PlayingScreen {
    pub fn new(config: Rc<PlayingConfig>) -> Self {
        Self {
            config,
            current: None,
            typewriter: None,
            visible: None,
            autosave_pending: false,
            choice_focus: Focus::default(),
            stage: Stage::default(),
            hidden: false,
            skipping: false,
            last_skip: 0.0,
            ready_since: None,
        }
    }

    fn show(&mut self, event: Event, typed: Option<(u32, f64)>) {
        self.typewriter = match (&event, typed) {
            (Event::Say { text, .. }, Some((speed, now))) => {
                Some(Typewriter::start(text, speed, now))
            }
            _ => None,
        };
        self.choice_focus = Focus::default();
        self.current = Some(event);
    }

    fn typing(&self, now: f64) -> bool {
        self.typewriter
            .as_ref()
            .is_some_and(|typewriter| !typewriter.is_done(now))
    }

    fn advance(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        if let Some(key) = ctx.story.line_key() {
            ctx.seen.insert(key);
        }
        ctx.audio.stop_voice();
        self.ready_since = None;
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
                event @ (Event::Show { .. }
                | Event::Hide { .. }
                | Event::Clear { .. }
                | Event::Background { .. }) => {
                    let now = ctx.rl.get_time();
                    self.stage.apply(&event, ctx.story, &self.config, now);
                }
                Event::Sound { id } => ctx.play_sound(&id),
                Event::Voice { id } => ctx.audio.play_voice(&id),
                Event::SceneEnter { scene } => {
                    self.autosave_pending = true;
                    if let Some(next) = ctx.run_scene_hooks(&scene) {
                        return Some(next);
                    }
                }
                event if event.is_blocking() => {
                    if let Event::Say { speaker, text } = &event {
                        ctx.log.push(LogEntry::Line {
                            speaker: speaker.clone(),
                            text: text.clone(),
                        });
                    }
                    let typed = (ctx.settings.values.text_speed, ctx.rl.get_time());
                    self.show(event, Some(typed));
                    ctx.rollback
                        .record_with_log(ctx.story, ctx.state, Some(ctx.log.len()));
                    if std::mem::take(&mut self.autosave_pending) {
                        ctx.autosave();
                    }
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
                ctx.rollback
                    .record_with_log(ctx.story, ctx.state, Some(ctx.log.len()));
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
        let back = wheel > 0.0
            || ctx.nav.page_back
            || config.back_keys.iter().any(|&k| ctx.rl.is_key_pressed(k));
        let forward = wheel < 0.0
            || ctx.nav.page_forward
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
            self.stage.reset(ctx.story, &self.config);
            if let Some(len) = ctx.rollback.log_len() {
                ctx.log.show(len);
            }
            ctx.audio.stop_voice();
            ctx.modes.skip = false;
        }
        back || forward
    }

    fn auto_advance(&mut self, ctx: &mut GameContext, now: f64) -> Option<ScreenState> {
        let Some(Event::Say { text, .. }) = &self.current else {
            return None;
        };
        let waiting = self.typing(now) || self.stage.is_animating() || ctx.audio.voice_playing();
        if !ctx.modes.auto || waiting {
            self.ready_since = None;
            return None;
        }

        let delay = self.config.auto_delay(&ctx.settings.values, text);
        let since = *self.ready_since.get_or_insert(now);
        if now - since >= delay {
            self.advance(ctx)
        } else {
            None
        }
    }

    fn shows_hud(&self) -> bool {
        !matches!(self.current, Some(Event::End))
    }

    fn over_hud(&self, rl: &RaylibHandle) -> bool {
        self.shows_hud()
            && self
                .config
                .hud_rects(ui::screen_size(rl))
                .iter()
                .enumerate()
                .any(|(i, rect)| ui::button_hovered(rl, *rect, &self.config.hud_style(i)))
    }

    fn continue_pressed(&self, rl: &RaylibHandle, nav: &NavInput) -> bool {
        nav.accept
            || (rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && !self.over_hud(rl))
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
        if !self.stage.is_synced() {
            self.stage.sync(ctx.story, &self.config);
        }
        self.stage.expire(ctx.rl.get_time());

        let rl = &*ctx.rl;
        let pressed = |key: Option<KeyboardKey>| key.is_some_and(|key| rl.is_key_pressed(key));
        let [menu, pause, quick_save, quick_load] = [
            self.config.menu_key,
            self.config.pause_key,
            self.config.quick_save_key,
            self.config.quick_load_key,
        ]
        .map(pressed);

        let keys = &self.config.keys;
        let any = |list: &[KeyboardKey]| list.iter().any(|&key| rl.is_key_pressed(key));
        let held = |list: &[KeyboardKey]| list.iter().any(|&key| rl.is_key_down(key));
        let middle = self.config.keys.middle_click_hides
            && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE);
        let right = self.config.keys.right_click_pauses
            && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT);
        let hide = any(&keys.hide) || middle || ctx.nav.hide;
        let log = any(&keys.log) || ctx.nav.log;
        let screenshot = any(&keys.screenshot);
        let fullscreen = any(&keys.fullscreen);
        let skip_toggle = any(&keys.skip_toggle);
        let skip_hold = held(&keys.skip_hold) || ctx.nav.skip_held;
        let auto = any(&keys.auto);

        if menu {
            return Some(ScreenState::MainMenu);
        }

        if pause || ctx.nav.pause || right {
            self.hidden = false;
            ctx.modes.skip = false;
            ctx.open_overlay(self.config.pause_overlay.clone());
            return None;
        }

        if self.hidden {
            let wake = hide
                || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                || ctx.nav.accept
                || ctx.nav.any_key()
                || self
                    .config
                    .advance_keys
                    .iter()
                    .any(|&key| rl.is_key_pressed(key));
            if wake {
                self.hidden = false;
            }
            return None;
        }

        if screenshot {
            ctx.screenshot();
        }
        if fullscreen {
            ctx.settings.update(|s| s.fullscreen = !s.fullscreen);
        }

        if self.current.is_some() && hide {
            self.hidden = true;
            return None;
        }

        if self.current.is_some() && log {
            ctx.modes.skip = false;
            ctx.open_overlay(self.config.log_overlay.clone());
            return None;
        }

        let saying = matches!(self.current, Some(Event::Say { .. }));
        if saying && skip_toggle {
            ctx.modes.skip = !ctx.modes.skip;
        }
        if auto {
            ctx.modes.auto = !ctx.modes.auto;
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
            for (button, rect) in self.config.hud.iter().zip(&hud) {
                if let Some(text) = &button.tooltip {
                    ctx.tooltip(*rect, text.clone());
                }
            }
            let mut clicked = None;
            for (index, rect) in hud.iter().enumerate() {
                let style = self.config.hud_style(index);
                if ui::button_clicked(&mut ctx, *rect, &style) && clicked.is_none() {
                    clicked = Some(index);
                }
            }
            if let Some(index) = clicked {
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

                let mut clicked = None;
                let mut hovered = None;
                for (index, (rect, option)) in rects.iter().zip(options).enumerate() {
                    let style = self.config.choice_style_for(index, option);
                    if ui::button_hovered(ctx.rl, *rect, &style) {
                        hovered = Some(index);
                    }
                    if ui::button_clicked(&mut ctx, *rect, &style) && clicked.is_none() {
                        clicked = Some(index);
                    }
                }
                let enabled = vec![true; rects.len()];
                let accepted = self
                    .choice_focus
                    .update(&ctx.nav, &rects, &enabled, hovered);
                let clicked = clicked.or(accepted);

                match clicked {
                    Some(index) => {
                        let text = options[index].clone();
                        if let Err(e) = ctx.story.choose(index) {
                            eprintln!("⚠️ {}", e);
                            return None;
                        }
                        ctx.log.push(LogEntry::Choice { text: text.clone() });
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
            Some(Event::End) => {
                ctx.modes.skip = false;
                self.continue_pressed(ctx.rl, &ctx.nav)
                    .then(|| self.config.after_end.clone())
            }
            Some(_) => {
                let now = ctx.rl.get_time();
                let skipping = ctx.modes.skip || skip_hold;
                let seen = ctx
                    .story
                    .line_key()
                    .is_some_and(|key| ctx.seen.contains(key));
                let may_skip = seen || ctx.settings.values.skip_unseen;
                if skipping && !may_skip {
                    ctx.modes.skip = false;
                }

                if skipping && may_skip {
                    if now - self.last_skip >= self.config.skip_interval {
                        self.last_skip = now;
                        self.stage.finish();
                        self.advance(&mut ctx)
                    } else {
                        None
                    }
                } else if !self.continue_pressed(ctx.rl, &ctx.nav) {
                    self.auto_advance(&mut ctx, now)
                } else if self.typing(now) || self.stage.is_animating() {
                    if let Some(typewriter) = &mut self.typewriter {
                        typewriter.finish();
                    }
                    self.stage.finish();
                    None
                } else {
                    self.advance(&mut ctx)
                }
            }
        };

        if matches!(self.current, Some(Event::Choice { .. })) {
            ctx.modes.skip = false;
        }
        self.skipping = ctx.modes.skip || skip_hold;

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
        for path in self.stage.texture_paths() {
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }

        None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);

        let now = d.get_time();
        self.stage.draw(d, ctx.resources, ctx.story, config, now);
        if self.hidden {
            return;
        }

        let fonts = ctx.fonts();

        if self.shows_hud() {
            for (index, (button, rect)) in
                config.hud.iter().zip(config.hud_rects(screen)).enumerate()
            {
                let active = match button.action {
                    Action::ToggleAuto => ctx.modes.auto,
                    Action::ToggleSkip => self.skipping,
                    _ => false,
                };
                ui::Button::new(&button.label, &config.hud_style(index))
                    .active(active)
                    .draw(d, ctx, rect);
            }
        }

        let labels: Vec<&str> = [
            (self.skipping, config.skip_label.as_str()),
            (ctx.modes.auto && !self.skipping, config.auto_label.as_str()),
        ]
        .into_iter()
        .filter_map(|(on, label)| on.then_some(label))
        .collect();
        if let Some(label) = labels.first() {
            let style = &config.indicator_text;
            let size = fonts.measure(style.font, label, style.size);
            let panel = Rectangle::new(16.0, 16.0, size.x + 24.0, size.y + 12.0);
            config.indicator.draw(d, panel);
            ui::draw_text(d, fonts, label, Vector2::new(28.0, 22.0), style);
        }

        match &self.current {
            Some(Event::Say { speaker, text }) => {
                let rect = config.dialogue_box.rect(screen);
                let style = &config.dialogue_box;

                style.panel.draw(d, rect);

                let inner_x = rect.x + style.padding;
                let inner_width = rect.width - style.padding * 2.0;
                let mut y = rect.y + style.padding;

                if let Some(speaker) = speaker {
                    let name = ctx.characters.display_name(speaker, ctx.story);
                    let text = match ctx.characters.color(speaker) {
                        Some(color) => config.speaker_text.clone().color(color),
                        None => config.speaker_text.clone(),
                    };
                    match &style.name_plate {
                        Some(plate) => {
                            let size = fonts.measure(text.font, &name, text.size);
                            let plate_rect = plate.rect(rect, size);
                            plate.panel.draw(d, plate_rect);
                            let at = Vector2::new(
                                plate_rect.x + (plate_rect.width - size.x) / 2.0,
                                plate_rect.y + plate.padding.y,
                            );
                            ui::draw_text(d, fonts, &name, at, &text);
                        }
                        None => {
                            ui::draw_text(d, fonts, &name, Vector2::new(inner_x, y), &text);
                            y += config.speaker_text.size * 1.4;
                        }
                    }
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
                for (index, (option, rect)) in options.iter().zip(rects).enumerate() {
                    let style = config.choice_style_for(index, option);
                    ui::Button::new(option, &style)
                        .focused(ctx.shows_focus(&self.choice_focus, index))
                        .draw(d, ctx, rect);
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
