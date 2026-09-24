use std::rc::Rc;

use raylib::prelude::*;
use vn_script::Event;

use super::{PlayingConfig, Typewriter};
use crate::action::Action;
use crate::context::{DrawContext, GameContext};
use crate::data::resources::{background_path, bust_path};
use crate::data::session::{LogEntry, Spoken};
use crate::frame::stage::Stage;
use crate::input::navigation::Focus;
use crate::screen::{Screen, ScreenState};
use crate::ui;
use crate::ui::styled::StyledText;

pub struct PlayingScreen {
    pub(super) config: Rc<PlayingConfig>,
    pub(super) current: Option<Event>,
    pub(super) typewriter: Option<Typewriter>,
    pub(super) visible: Option<usize>,
    pub(super) choice_focus: Focus,
    pub(super) stage: Stage,
    pub(super) hidden: bool,
    pub(super) skipping: bool,
    pub(super) last_skip: f64,
    pub(super) ready_since: Option<f64>,
}

impl PlayingScreen {
    pub fn new(config: Rc<PlayingConfig>) -> Self {
        Self {
            config,
            current: None,
            typewriter: None,
            visible: None,
            choice_focus: Focus::default(),
            stage: Stage::default(),
            hidden: false,
            skipping: false,
            last_skip: 0.0,
            ready_since: None,
        }
    }
}

impl PlayingScreen {
    fn update_story(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
        ui::load_background(ctx, self.config.background.as_ref());
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

        if self.current.is_some() && self.roll(ctx) {
            return None;
        }

        if self.current.is_some() {
            if quick_save {
                return Action::QuickSave.run(ctx);
            }
            if quick_load {
                return Action::QuickLoad.run(ctx);
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
                if ui::button::button_clicked(ctx, *rect, &style) && clicked.is_none() {
                    clicked = Some(index);
                }
            }
            if let Some(index) = clicked {
                let action = self.config.hud[index].action.clone();
                return action.run(ctx);
            }
        }

        let next = match &self.current {
            None => self.resume(ctx),
            Some(Event::Choice { options }) => {
                let rects = self
                    .config
                    .choice_rects(options.len(), ui::screen_size(ctx.rl));

                let mut clicked = None;
                let mut hovered = None;
                for (at, (rect, option)) in rects.iter().zip(options).enumerate() {
                    let style = self.config.option_style(option);
                    if ui::button::button_hovered(ctx.rl, *rect, &style) {
                        hovered = Some(at);
                        if !option.enabled {
                            ctx.cursor(crate::ui::cursor::CursorKind::NotAllowed);
                            if let Some(reason) = &option.reason {
                                ctx.tooltip_text(reason.clone());
                            }
                        }
                    }
                    if option.enabled
                        && ui::button::button_clicked(ctx, *rect, &style)
                        && clicked.is_none()
                    {
                        clicked = Some(at);
                    }
                }
                let enabled: Vec<bool> = options.iter().map(|option| option.enabled).collect();
                let accepted = self
                    .choice_focus
                    .update(&ctx.nav, &rects, &enabled, hovered);
                let clicked = clicked.or(accepted);

                match clicked {
                    Some(at) => {
                        let index = options[at].index;
                        let text = options[at].text.clone();
                        let said = ctx
                            .story
                            .choice_source(index)
                            .map(|source| Spoken::capture(ctx.story, source));
                        if let Err(e) = ctx.story.choose(index) {
                            eprintln!("⚠️ {}", e);
                            return None;
                        }
                        if let Some(said) = said {
                            ctx.log.push(LogEntry::Choice { said });
                        }
                        if !ctx.rollback.config().through_choices {
                            ctx.rollback.mark_barrier();
                        }
                        match ctx.run_choice_hooks(index, &text) {
                            Some(next) => Some(next),
                            None => self.advance(ctx),
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
                        self.advance(ctx)
                    } else {
                        None
                    }
                } else if !self.continue_pressed(ctx.rl, &ctx.nav) {
                    self.auto_advance(ctx, now)
                } else if self.typing(now) || self.stage.is_animating() {
                    if let Some(typewriter) = &mut self.typewriter {
                        typewriter.finish();
                    }
                    self.stage.finish();
                    None
                } else {
                    self.advance(ctx)
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

        None
    }
}

impl PlayingScreen {
    fn previewed<'a>(
        &self,
        rl: &RaylibHandle,
        options: &'a [vn_script::ChoiceOption],
        rects: &[Rectangle],
    ) -> Option<&'a str> {
        let under_mouse = rects.iter().position(|rect| ui::is_hovered(rl, *rect));
        let at = under_mouse.or_else(|| self.choice_focus.index())?;
        options.get(at)?.preview.as_deref()
    }
}

fn draw_preview(
    d: &mut RaylibDrawHandle,
    ctx: &DrawContext,
    style: &super::ChoicePreviewStyle,
    preview: &str,
    screen: Vector2,
) {
    let path = crate::data::resources::preview_path(preview);
    let Some(texture) = ctx.resources.texture(&path) else {
        return;
    };

    let panel = style.rect(screen);
    style.panel.draw(d, panel);

    let natural = Vector2::new(texture.width as f32, texture.height as f32);
    d.draw_texture_pro(
        texture,
        Rectangle::new(0.0, 0.0, natural.x, natural.y),
        style.picture_rect(panel, natural),
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}

impl Screen for PlayingScreen {
    fn update(&mut self, mut ctx: GameContext) -> Option<ScreenState> {
        let next = self.update_story(&mut ctx);
        if next.is_some() {
            return next;
        }
        ctx.run_frame_hooks(ctx.rl.get_frame_time());

        #[cfg(feature = "character-visuals")]
        for key in self.stage.visual_keys(ctx.story) {
            ctx.show_visual(&key.character, &key.appearance);
        }
        #[cfg(not(feature = "character-visuals"))]
        for (name, image) in ctx.story.active_characters() {
            ctx.resources.get_or_load(
                &crate::data::resources::character_path(name, image),
                ctx.rl,
                ctx.thread,
            );
        }
        if let Some(image) = ctx.story.background() {
            ctx.resources
                .get_or_load(&background_path(image), ctx.rl, ctx.thread);
        }
        if let Some(Event::Choice { options }) = &self.current {
            for path in self.config.option_pictures(options) {
                ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
            }
        }
        for path in self.stage.texture_paths() {
            #[cfg(feature = "character-visuals")]
            if path.starts_with("characters/") {
                continue;
            }
            ctx.resources.get_or_load(&path, ctx.rl, ctx.thread);
        }
        None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, ctx: &DrawContext) {
        let config = &self.config;
        let screen = ui::screen_size(d);

        let now = d.get_time();
        self.stage.draw(d, ctx.resources, ctx.story, config, now);
        if let Some(weather) = ctx.weather {
            weather.draw(d, now, screen);
        }
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
                ui::button::Button::new(ctx.label(&button.label), &config.hud_style(index))
                    .active(active)
                    .draw(d, ctx, rect);
            }
        }

        let labels: Vec<&str> = [
            (self.skipping, ctx.label(&config.skip_label)),
            (
                ctx.modes.auto && !self.skipping,
                ctx.label(&config.auto_label),
            ),
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

        let nvl = ctx.story.scene_mode().is_nvl();
        if nvl {
            let visible = matches!(self.current, Some(Event::Say { .. })).then_some(self.visible);
            super::nvl::draw(d, ctx, config, screen, visible.flatten());
        }

        match &self.current {
            Some(Event::Say { .. }) if nvl => {}
            Some(Event::Say { speaker, text }) => {
                let style = ctx
                    .characters
                    .dialogue_box(speaker.as_deref(), &config.dialogue_box);
                let style = &style;
                let rect = style.rect(screen);

                style.panel.draw(d, rect);

                if let (Some(bust), Some(speaker)) = (&style.bust, speaker.as_deref())
                    && let Some(file) = ctx.characters.bust(speaker)
                {
                    let path = bust_path(file);
                    if let Some(texture) = ctx.resources.texture(&path) {
                        let natural = Vector2::new(texture.width as f32, texture.height as f32);
                        let at = bust.rect(rect, natural);
                        d.draw_texture_pro(
                            texture,
                            Rectangle::new(0.0, 0.0, natural.x, natural.y),
                            at,
                            Vector2::zero(),
                            0.0,
                            Color::WHITE,
                        );
                    }
                }

                let area = style.text_area(rect);
                let inner_x = area.x;
                let inner_width = area.width;
                let mut y = area.y;

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

                crate::ui::styled::draw(
                    d,
                    fonts,
                    &StyledText::parse(text),
                    Vector2::new(inner_x, y),
                    inner_width,
                    &config.dialogue_text,
                    self.visible.unwrap_or(usize::MAX),
                );
            }
            Some(Event::Choice { options }) => {
                let rects = config.choice_rects(options.len(), screen);
                for (at, (option, rect)) in options.iter().zip(&rects).enumerate() {
                    let style = config.option_style(option);
                    let label = vn_script::markup::plain(&option.text);
                    ui::button::Button::new(&label, &style)
                        .focused(ctx.shows_focus(&self.choice_focus, at))
                        .disabled(!option.enabled)
                        .draw(d, ctx, *rect);
                }
                if let Some(preview) = self.previewed(d, options, &rects) {
                    draw_preview(d, ctx, &config.choice_preview, preview, screen);
                }
            }
            Some(Event::End) => {
                ui::draw_text_centered(
                    d,
                    fonts,
                    ctx.label(&config.end_title),
                    Vector2::new(screen.x / 2.0, screen.y * 0.42),
                    &config.end_title_text,
                );
                ui::draw_text_centered(
                    d,
                    fonts,
                    &ctx.prompt(&config.end_hint),
                    Vector2::new(screen.x / 2.0, screen.y * 0.52),
                    &config.end_hint_text,
                );
            }
            _ => {}
        }
    }
}
