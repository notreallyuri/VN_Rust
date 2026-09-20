use raylib::prelude::*;
use vn_script::{Event, TransitionKind};

use super::{PlayingScreen, Typewriter};
use crate::ui;
use crate::{Focus, GameContext, LogEntry, NavInput, ScreenState};

impl PlayingScreen {
    pub(super) fn show(&mut self, event: Event, typed: Option<(u32, f64)>) {
        self.typewriter = match (&event, typed) {
            (Event::Say { text, .. }, Some((speed, now))) => {
                Some(Typewriter::start(text, speed, now))
            }
            _ => None,
        };
        self.choice_focus = Focus::default();
        self.current = Some(event);
    }

    pub(super) fn typing(&self, now: f64) -> bool {
        self.typewriter
            .as_ref()
            .is_some_and(|typewriter| !typewriter.is_done(now))
    }

    pub(super) fn advance(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
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
                    match crate::stage::effect_of(&event) {
                        Some((TransitionKind::Shake, seconds)) => ctx.shake(seconds),
                        Some((TransitionKind::Flash, seconds)) => ctx.flash(seconds),
                        _ => {}
                    }
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

    pub(super) fn resume(&mut self, ctx: &mut GameContext) -> Option<ScreenState> {
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

    pub(super) fn roll(&mut self, ctx: &mut GameContext) -> bool {
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

    pub(super) fn auto_advance(&mut self, ctx: &mut GameContext, now: f64) -> Option<ScreenState> {
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

    pub(super) fn shows_hud(&self) -> bool {
        !matches!(self.current, Some(Event::End))
    }

    pub(super) fn over_hud(&self, rl: &RaylibHandle) -> bool {
        self.shows_hud()
            && self
                .config
                .hud_rects(ui::screen_size(rl))
                .iter()
                .enumerate()
                .any(|(i, rect)| ui::button_hovered(rl, *rect, &self.config.hud_style(i)))
    }

    pub(super) fn continue_pressed(&self, rl: &RaylibHandle, nav: &NavInput) -> bool {
        nav.accept
            || (rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && !self.over_hud(rl))
            || self
                .config
                .advance_keys
                .iter()
                .any(|&key| rl.is_key_pressed(key))
    }
}
