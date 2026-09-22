use std::io;
use std::rc::Rc;

use raylib::prelude::*;

use vn_script::{Event, RestoreOutcome, StoryVm};

use crate::action::Action;
use crate::context::{DrawContext, GameContext};
use crate::data::assets::Assets;
use crate::data::resources::ResourceManager;
use crate::data::rollback::Rollback;
use crate::data::saves::{Saves, THUMBNAIL_WIDTH};
use crate::data::session::{PlayModes, SeenLines, SessionLog};
use crate::data::settings::SettingsStore;
use crate::data::state::GameState;
use crate::game::audio::Audio;
use crate::game::characters::Characters;
use crate::game::commands::Commands;
use crate::game::hooks::Hooks;
use crate::game::script_errors::{SCRIPT_ERRORS_KEY, ScriptErrors};
use crate::input::navigation::{NavInput, Navigation};
use crate::overlay::{Overlay, OverlayAction, OverlayRequest};
use crate::screen::{Screen, ScreenState};
use crate::screens::confirm::{CONFIRM_OVERLAY, Confirm};
use crate::screens::keybinds::KEYBINDS_OVERLAY;
use crate::screens::text_input::TextRequest;
use crate::ui::toast::{Toast, ToastConfig};
use crate::ui::tooltip::{TooltipConfig, TooltipTimer};

pub const CLOSE_MESSAGE: &str = "Quit the game? Unsaved progress will be lost.";

const THUMBNAIL_INTERVAL: f64 = 1.0;

pub fn close_needs_confirmation(state: &ScreenState, story: &StoryVm) -> bool {
    let ended = matches!(story.current(), Some(Event::End));
    match state {
        ScreenState::StartScreen | ScreenState::MainMenu | ScreenState::Quit => false,
        ScreenState::Playing | ScreenState::TextInput => !ended,
        #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
        ScreenState::Video => !ended,
        _ => story.current().is_some() && !ended,
    }
}

pub trait ScreenFactory {
    fn create_screen(&self, state: &ScreenState) -> Option<Box<dyn Screen>>;

    fn create_overlay(&self, _name: &str) -> Option<Box<dyn Overlay>> {
        None
    }
}

pub struct World {
    pub story: StoryVm,
    pub state: GameState,
    pub saves: Saves,
    pub settings: SettingsStore,
    pub characters: Characters,
    pub rollback: Rollback,
    pub log: SessionLog,
    pub seen: SeenLines,
    pub persistent: crate::data::persistent::Persistent,
    pub modes: PlayModes,
}

pub struct Presentation {
    pub resources: ResourceManager,
    pub audio: Audio,
    pub navigation: Navigation,
    pub toast_config: ToastConfig,
    pub tooltip_config: TooltipConfig,
    pub(crate) effects: crate::frame::effects::ScreenEffects,
    pub(crate) weather: Option<crate::frame::scenery::Weather>,
    pub(crate) post: crate::frame::post::PostChain,
}

pub struct Screens {
    pub current: Box<dyn Screen>,
    pub showing: ScreenState,
    pub previous: Option<ScreenState>,
    pub factory: Box<dyn ScreenFactory>,
    overlays: Vec<(String, Box<dyn Overlay>)>,
    cursor_override: Option<(ScreenState, crate::ui::cursor::CursorKind)>,
}

#[derive(Default)]
pub struct Frame {
    nav: NavInput,
    toast: Option<Toast>,
    tooltip_timer: TooltipTimer,
    thumbnail: Option<Image>,
    thumbnail_at: Option<f64>,
    autosave: bool,
    screenshot: bool,
    cursor: crate::ui::cursor::CursorKind,
    changed: bool,
    quit: bool,
    closing: bool,
    fullscreen: bool,
}

pub struct ScreenStateManager {
    pub world: World,
    pub show: Presentation,
    pub screens: Screens,
    frame: Frame,
    pub commands: Rc<Commands>,
    pub hooks: Rc<Hooks>,
    pub close_confirmation: Option<String>,
    pub keybind_keys: Vec<KeyboardKey>,
    pub prompts: crate::input::prompts::Prompts,
    pub(crate) pointer: Option<crate::ui::cursor::Pointer>,
    script_errors: Option<ScriptErrors>,
    autosave_pending: bool,
    #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
    video_request: Option<crate::video::VideoRequest>,
    text_request: Option<TextRequest>,
    confirm_request: Option<Confirm>,
}

impl ScreenStateManager {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        initial_state: ScreenState,
        factory: Box<dyn ScreenFactory>,
        assets_root: impl Into<Assets>,
        story_dir: &str,
    ) -> io::Result<Self> {
        let assets_root = assets_root.into();
        let loader = crate::game::hot_reload::StoryLoader {
            #[cfg(feature = "character-visuals")]
            visuals: Default::default(),
            assets: assets_root.clone(),
            story_dir: story_dir.into(),
            schema: Default::default(),
            entry_scene: None,
            warn_missing_art: false,
        };
        let story = StoryVm::from_program(vn_script::compile_sources(loader.sources()?));
        Self::with_story(rl, thread, initial_state, factory, assets_root, story)
    }

    pub fn with_story(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        initial_state: ScreenState,
        factory: Box<dyn ScreenFactory>,
        assets_root: impl Into<Assets>,
        story: StoryVm,
    ) -> io::Result<Self> {
        let resources = ResourceManager::new(assets_root, rl, thread);
        let current_screen = factory.create_screen(&initial_state).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("no screen for the initial state {:?}", initial_state),
            )
        })?;

        Ok(Self {
            world: World {
                story,
                state: GameState::default(),
                saves: Saves::new("saves", "game"),
                settings: SettingsStore::in_memory(),
                characters: Characters::default(),
                rollback: Rollback::default(),
                log: SessionLog::default(),
                seen: SeenLines::in_memory(),
                persistent: crate::data::persistent::Persistent::in_memory(),
                modes: PlayModes::default(),
            },
            show: Presentation {
                resources,
                audio: Audio::silent(),
                navigation: Navigation::default(),
                toast_config: ToastConfig::default(),
                tooltip_config: TooltipConfig::default(),
                effects: crate::frame::effects::ScreenEffects::default(),
                weather: None,
                post: crate::frame::post::PostChain::new(),
            },
            screens: Screens {
                current: current_screen,
                showing: initial_state,
                previous: None,
                factory,
                overlays: Vec::new(),
                cursor_override: None,
            },
            frame: Frame::default(),
            commands: Rc::new(Commands::default()),
            hooks: Rc::new(Hooks::default()),
            close_confirmation: Some(CLOSE_MESSAGE.to_string()),
            keybind_keys: vec![KeyboardKey::KEY_F1],
            prompts: Default::default(),
            pointer: Some(crate::ui::cursor::Pointer::system()),
            script_errors: None,
            autosave_pending: false,
            #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
            video_request: None,
            text_request: None,
            confirm_request: None,
        })
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let now = rl.get_time();
        let keybinds_open = self.overlay_name() == Some(KEYBINDS_OVERLAY);
        self.show.resources.load_requested(rl, thread);
        self.frame.nav = self.show.navigation.read(rl);
        let mut requests = crate::request::Requests::default();

        self.screens
            .current
            .set_paused(!self.screens.overlays.is_empty(), now);

        let ctx = GameContext {
            rl,
            thread,
            resources: &mut self.show.resources,
            story: &mut self.world.story,
            state: &mut self.world.state,
            saves: &self.world.saves,
            previous: self.screens.previous.as_ref(),
            rollback: &mut self.world.rollback,
            settings: &mut self.world.settings,
            commands: Rc::clone(&self.commands),
            hooks: Rc::clone(&self.hooks),
            requests: &mut requests,
            effects: &mut self.show.effects,
            weather: &mut self.show.weather,
            post: &mut self.show.post,
            autosave_pending: &mut self.autosave_pending,
            #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
            video_request: &mut self.video_request,
            text_request: &mut self.text_request,
            confirm_request: &mut self.confirm_request,
            thumbnail: self.frame.thumbnail.as_ref(),
            audio: &mut self.show.audio,
            nav: self.frame.nav,
            prompts: &self.prompts,
            log: &mut self.world.log,
            modes: &mut self.world.modes,
            seen: &mut self.world.seen,
            persistent: &mut self.world.persistent,
        };

        let next_state = match self.screens.overlays.last_mut() {
            Some((_, overlay)) => match overlay.update(ctx) {
                OverlayAction::Stay => None,
                OverlayAction::Close => {
                    self.screens.overlays.pop();
                    None
                }
                OverlayAction::CloseAll => {
                    self.screens.overlays.clear();
                    None
                }
                OverlayAction::Goto(state) => Some(state),
            },
            None => self.screens.current.update(ctx),
        };

        let wants_keybinds = !keybinds_open
            && self.screens.showing != ScreenState::TextInput
            && self.keybind_keys.iter().any(|&key| rl.is_key_pressed(key));
        if wants_keybinds {
            self.world.modes.skip = false;
            self.open_overlay(KEYBINDS_OVERLAY);
        }

        let asked = requests.resolve();
        for overlay in asked.overlays {
            match overlay {
                OverlayRequest::Open(name) => self.open_overlay(&name),
                OverlayRequest::Close => self.close_overlay(),
                OverlayRequest::CloseAll => self.screens.overlays.clear(),
            }
        }
        self.frame.cursor = asked.cursor;
        if let Some(kind) = asked.cursor_override {
            self.screens.cursor_override = kind.map(|kind| (self.screens.showing.clone(), kind));
        }
        self.frame.autosave |= asked.autosave;
        self.frame.screenshot |= asked.screenshot;
        let (tooltip, toast) = (asked.tooltip, asked.toast);

        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        self.frame.tooltip_timer.update(tooltip, now, clicked);

        if let Some(mut toast) = toast {
            toast.shown_at = Some(now);
            self.frame.toast = Some(toast);
        } else if let Some(toast) = &mut self.frame.toast
            && toast.shown_at.is_none()
        {
            toast.shown_at = Some(now);
        } else if self
            .frame
            .toast
            .as_ref()
            .is_some_and(|toast| toast.expired(now, &self.show.toast_config))
        {
            self.frame.toast = None;
        }

        if let Some(state) = next_state {
            self.transition_to(state);
        }

        if self.frame.closing && self.overlay_name() != Some(CONFIRM_OVERLAY) {
            self.frame.closing = false;
        }

        self.screens
            .current
            .set_paused(!self.screens.overlays.is_empty(), now);
        self.update_music(rl.get_frame_time());
        self.world.persistent.save_if_due(now);

        if let Some(errors) = &mut self.script_errors
            && rl.is_key_pressed(SCRIPT_ERRORS_KEY)
        {
            errors.toggle();
        }

        let cursor = self.effective_cursor();
        if let Some(pointer) = &mut self.pointer {
            pointer.update(rl, cursor, self.frame.nav.device);
        }

        if self.world.settings.values.fullscreen != self.frame.fullscreen {
            rl.toggle_borderless_windowed();
            self.frame.fullscreen = self.world.settings.values.fullscreen;
        }
    }

    fn update_music(&mut self, dt: f32) {
        let wanted = match self.screens.showing {
            ScreenState::StartScreen | ScreenState::MainMenu => {
                self.show.audio.config().menu_music.clone()
            }
            ScreenState::Playing | ScreenState::TextInput => {
                self.world.story.music().map(str::to_string)
            }
            _ => self.show.audio.music().map(str::to_string),
        };

        let settings = &self.world.settings.values;
        let music_gain = settings.music_gain();
        #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
        let music_gain = if self.screens.showing == ScreenState::Video {
            0.0
        } else {
            music_gain
        };
        self.show
            .audio
            .set_volumes(music_gain, settings.sound_gain());
        self.show.audio.set_voice_volume(settings.voice_gain());
        self.show.audio.play_music(wanted.as_deref());
        self.show.audio.update(dt);
    }

    pub fn request_close(&mut self) {
        let confirm_open = self.overlay_name() == Some(CONFIRM_OVERLAY);

        let message = match &self.close_confirmation {
            Some(message)
                if !(self.frame.closing && confirm_open)
                    && close_needs_confirmation(&self.screens.showing, &self.world.story) =>
            {
                message.clone()
            }
            _ => {
                self.frame.quit = true;
                return;
            }
        };

        if confirm_open {
            self.screens.overlays.pop();
        }
        self.frame.closing = true;
        self.confirm(Confirm::new(message, Action::Quit).confirm_label("Quit"));
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        self.screens
            .current
            .draw(d, &self.draw_context(self.screens.overlays.is_empty()));
        if std::mem::take(&mut self.frame.screenshot) {
            self.take_screenshot(d, thread);
        }
        if std::mem::take(&mut self.frame.autosave) {
            self.frame.thumbnail_at = None;
            self.capture_thumbnail(d, thread);
            self.autosave();
        } else {
            self.capture_thumbnail(d, thread);
        }

        let top = self.screens.overlays.len().saturating_sub(1);
        for (index, (_, overlay)) in self.screens.overlays.iter().enumerate() {
            overlay.draw(d, &self.draw_context(index == top));
        }

        let ctx = self.draw_context(false);

        if let Some(errors) = &self.script_errors {
            errors.draw(d, ctx.fonts());
        }

        if let Some(toast) = &self.frame.toast {
            toast.draw(d, ctx.fonts(), &self.show.toast_config);
        }

        let config = &self.show.tooltip_config;
        if config.enabled
            && let Some(text) = self.frame.tooltip_timer.visible(d.get_time(), config.delay)
        {
            crate::ui::tooltip::draw_tooltip(d, ctx.fonts(), text, config);
        }
    }

    fn capture_thumbnail(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let now = d.get_time();
        let due = self
            .frame
            .thumbnail_at
            .is_none_or(|at| now - at >= THUMBNAIL_INTERVAL);
        let in_game = self.screens.showing == ScreenState::Playing
            && self.screens.overlays.is_empty()
            && self.world.story.current().is_some();
        if !due || !in_game {
            return;
        }

        flush_batch();
        let mut image = d.load_image_from_screen(thread);
        let height = THUMBNAIL_WIDTH * image.height().max(1) / image.width().max(1);
        image.resize(THUMBNAIL_WIDTH, height.max(1));
        self.frame.thumbnail = Some(image);
        self.frame.thumbnail_at = Some(now);
    }

    pub fn effects(&self) -> &crate::frame::effects::ScreenEffects {
        &self.show.effects
    }

    pub fn take_screen_changed(&mut self) -> bool {
        std::mem::take(&mut self.frame.changed)
    }

    pub fn thumbnail(&self) -> Option<&Image> {
        self.frame.thumbnail.as_ref()
    }

    pub fn autosave(&self) {
        #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
        if self.screens.showing == ScreenState::Video {
            return;
        }
        crate::data::saves::autosave(
            &self.world.saves,
            crate::data::saves::SaveParts {
                story: &self.world.story,
                state: &self.world.state,
                rollback: &self.world.rollback,
                log: self.world.log.entries(),
                thumbnail: self.frame.thumbnail.as_ref(),
            },
        );
    }

    fn take_screenshot(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        flush_batch();
        let image = d.load_image_from_screen(thread);
        let dir = self.world.saves.dir().join("screenshots");
        let path = dir.join(format!("screenshot-{}.png", crate::data::saves::now()));
        let written = std::fs::create_dir_all(&dir).is_ok() && {
            image.export_image(&path.to_string_lossy());
            path.exists()
        };
        if written {
            println!("Screenshot: {}", path.display());
            let text = crate::ui::labels::label(&self.world.story, "Screenshot saved").to_string();
            self.notify(Toast::info(text));
        } else {
            let text = crate::ui::labels::label(&self.world.story, "Could not save the screenshot")
                .to_string();
            self.notify(Toast::error(text));
        }
    }

    fn draw_context(&self, interactive: bool) -> DrawContext<'_> {
        DrawContext {
            resources: &self.show.resources,
            story: &self.world.story,
            state: &self.world.state,
            persistent: &self.world.persistent,
            saves: &self.world.saves,
            characters: &self.world.characters,
            settings: &self.world.settings.values,
            interactive,
            focus_visible: !self.frame.nav.pointer,
            log: &self.world.log,
            modes: self.world.modes,
            weather: self.show.weather,
            device: self.frame.nav.device,
            pad: self.frame.nav.pad,
            prompts: &self.prompts,
        }
    }

    pub fn show_script_errors(&mut self, errors: ScriptErrors) {
        let collapsed = self.script_errors.as_ref().is_some_and(|e| e.collapsed);
        self.script_errors = Some(ScriptErrors {
            collapsed,
            ..errors
        });
    }

    pub fn script_errors(&self) -> Option<&ScriptErrors> {
        self.script_errors.as_ref()
    }

    pub fn reload_story(&mut self, story: StoryVm) {
        self.script_errors = None;
        self.show.effects.clear();
        self.show.weather = None;
        match crate::game::hot_reload::swap_story(
            &mut self.world.story,
            story,
            &mut self.world.rollback,
        ) {
            Ok(RestoreOutcome::Exact) => {
                let text =
                    crate::ui::labels::label(&self.world.story, "Story reloaded").to_string();
                self.notify(Toast::info(text));
            }
            Ok(RestoreOutcome::SceneRestarted { scene }) => {
                let template = crate::ui::labels::label(
                    &self.world.story,
                    "Story reloaded; scene '{scene}' restarted",
                );
                let text = crate::ui::labels::fill(template, &[("scene", &scene)]);
                self.notify(Toast::info(text));
            }
            Err(e) => {
                eprintln!("⚠️ Story not reloaded: {}", e);
                let template =
                    crate::ui::labels::label(&self.world.story, "Story not reloaded: {reason}");
                let text = crate::ui::labels::fill(template, &[("reason", &e.to_string())]);
                self.notify(Toast::error(text));
                return;
            }
        }

        #[cfg(feature = "character-visuals")]
        self.show.resources.visuals.reset();

        #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
        if self.screens.showing == ScreenState::Video {
            self.video_request = None;
            self.transition_to(ScreenState::Playing);
        }

        if self.screens.showing == ScreenState::Playing
            && let Some(screen) = self.screens.factory.create_screen(&ScreenState::Playing)
        {
            self.screens.current = screen;
        }
    }

    pub fn confirm(&mut self, confirm: Confirm) {
        self.confirm_request = Some(confirm);
        self.open_overlay(CONFIRM_OVERLAY);
    }

    pub fn notify(&mut self, toast: Toast) {
        self.frame.toast = Some(toast);
    }

    pub fn quit_requested(&self) -> bool {
        self.frame.quit
    }

    pub fn overlay_name(&self) -> Option<&str> {
        self.screens.overlays.last().map(|(name, _)| name.as_str())
    }

    pub fn overlay_names(&self) -> impl Iterator<Item = &str> {
        self.screens.overlays.iter().map(|(name, _)| name.as_str())
    }

    pub fn open_overlay(&mut self, name: &str) {
        if self.overlay_name() == Some(name) {
            return;
        }
        match self.screens.factory.create_overlay(name) {
            Some(overlay) => self.screens.overlays.push((name.to_string(), overlay)),
            None => eprintln!("⚠️ No overlay registered as '{}'", name),
        }
    }

    pub fn close_overlay(&mut self) {
        self.screens.overlays.pop();
    }

    pub fn close_overlays(&mut self) {
        self.screens.overlays.clear();
    }

    fn effective_cursor(&self) -> crate::ui::cursor::CursorKind {
        crate::ui::cursor::effective(
            self.screens.cursor_override.as_ref(),
            &self.screens.showing,
            !self.screens.overlays.is_empty(),
            self.frame.cursor,
        )
    }

    pub fn cursor(&self) -> crate::ui::cursor::CursorKind {
        self.effective_cursor()
    }

    fn transition_to(&mut self, next_state: ScreenState) {
        if next_state == ScreenState::Quit {
            self.frame.quit = true;
            return;
        }

        match self.screens.factory.create_screen(&next_state) {
            Some(screen) => {
                self.world.modes.skip = false;
                self.frame.thumbnail_at = None;
                self.screens.current = screen;
                let previous = std::mem::replace(&mut self.screens.showing, next_state);
                self.screens.previous = Some(previous);
                self.screens.overlays.clear();
                self.show.effects.clear();
                self.show.weather = None;
                self.screens.cursor_override = None;
                self.frame.changed = true;
            }
            None => {
                eprintln!("⚠️ No screen registered for {:?}", next_state);
                #[cfg(any(feature = "video-portable", feature = "video-ffmpeg"))]
                if next_state == ScreenState::Video {
                    self.video_request = None;
                }
            }
        }
    }
}

fn flush_batch() {
    unsafe { raylib::ffi::rlDrawRenderBatchActive() };
}
