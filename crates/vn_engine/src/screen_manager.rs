use std::io;
use std::rc::Rc;

use raylib::prelude::*;

use vn_script::{Event, RestoreOutcome, StoryVm};

use crate::screens::{CONFIRM_OVERLAY, Confirm, KEYBINDS_OVERLAY};
use crate::{
    Action, Assets, Audio, Characters, Commands, DrawContext, GameContext, GameState, Hooks,
    NavInput, Navigation, Overlay, OverlayAction, OverlayRequest, ResourceManager, Rollback,
    SCRIPT_ERRORS_KEY, Saves, Screen, ScreenState, ScriptErrors, SettingsStore, THUMBNAIL_WIDTH,
    TextRequest, Toast, ToastConfig, TooltipConfig, TooltipTimer,
};
use crate::{PlayModes, SeenLines, SessionLog};

pub const CLOSE_MESSAGE: &str = "Quit the game? Unsaved progress will be lost.";

const THUMBNAIL_INTERVAL: f64 = 1.0;

pub fn close_needs_confirmation(state: &ScreenState, story: &StoryVm) -> bool {
    let ended = matches!(story.current(), Some(Event::End));
    match state {
        ScreenState::StartScreen | ScreenState::MainMenu | ScreenState::Quit => false,
        ScreenState::Playing | ScreenState::TextInput => !ended,
        _ => story.current().is_some() && !ended,
    }
}

pub trait ScreenFactory {
    fn create_screen(&self, state: &ScreenState) -> Option<Box<dyn Screen>>;

    fn create_overlay(&self, _name: &str) -> Option<Box<dyn Overlay>> {
        None
    }
}

pub struct ScreenStateManager {
    pub current_screen: Box<dyn Screen>,
    pub current_state: ScreenState,
    pub factory: Box<dyn ScreenFactory>,
    pub resources: ResourceManager,
    pub story: StoryVm,
    pub state: GameState,
    pub commands: Rc<Commands>,
    pub hooks: Rc<Hooks>,
    pub saves: Saves,
    pub previous_state: Option<ScreenState>,
    pub characters: Characters,
    pub rollback: Rollback,
    pub settings: SettingsStore,
    pub close_confirmation: Option<String>,
    text_request: Option<TextRequest>,
    confirm_request: Option<Confirm>,
    pub toast_config: ToastConfig,
    toast: Option<Toast>,
    script_errors: Option<ScriptErrors>,
    thumbnail: Option<Image>,
    thumbnail_at: Option<f64>,
    screen_changed: bool,
    pub(crate) effects: crate::ScreenEffects,
    pub(crate) post: crate::PostChain,
    autosave_request: bool,
    pub audio: Audio,
    pub tooltip_config: TooltipConfig,
    pub navigation: Navigation,
    nav: NavInput,
    pub log: SessionLog,
    pub modes: PlayModes,
    pub seen: SeenLines,
    screenshot_request: bool,
    pub keybind_keys: Vec<KeyboardKey>,
    tooltip_timer: TooltipTimer,
    overlays: Vec<(String, Box<dyn Overlay>)>,
    quit_requested: bool,
    closing: bool,
    fullscreen: bool,
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
        let loader = crate::StoryLoader {
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
            current_screen,
            current_state: initial_state,
            factory,
            resources,
            story,
            state: GameState::default(),
            commands: Rc::new(Commands::default()),
            hooks: Rc::new(Hooks::default()),
            saves: Saves::new("saves", "game"),
            previous_state: None,
            characters: Characters::default(),
            rollback: Rollback::default(),
            settings: SettingsStore::in_memory(),
            close_confirmation: Some(CLOSE_MESSAGE.to_string()),
            text_request: None,
            confirm_request: None,
            toast_config: ToastConfig::default(),
            toast: None,
            script_errors: None,
            thumbnail: None,
            thumbnail_at: None,
            screen_changed: false,
            effects: crate::ScreenEffects::default(),
            post: crate::PostChain::new(),
            autosave_request: false,
            audio: Audio::silent(),
            tooltip_config: TooltipConfig::default(),
            navigation: Navigation::default(),
            nav: NavInput::default(),
            log: SessionLog::default(),
            modes: PlayModes::default(),
            seen: SeenLines::in_memory(),
            screenshot_request: false,
            keybind_keys: vec![KeyboardKey::KEY_F1],
            tooltip_timer: TooltipTimer::default(),
            overlays: Vec::new(),
            quit_requested: false,
            closing: false,
            fullscreen: false,
        })
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let now = rl.get_time();
        let keybinds_open = self.overlay_name() == Some(KEYBINDS_OVERLAY);
        self.resources.load_requested(rl, thread);
        self.nav = self.navigation.read(rl);
        let mut overlay_requests = Vec::new();
        let mut toast = None;
        let mut tooltip = None;

        let ctx = GameContext {
            rl,
            thread,
            resources: &mut self.resources,
            story: &mut self.story,
            state: &mut self.state,
            saves: &self.saves,
            previous: self.previous_state.as_ref(),
            rollback: &mut self.rollback,
            settings: &mut self.settings,
            commands: Rc::clone(&self.commands),
            hooks: Rc::clone(&self.hooks),
            overlay_requests: &mut overlay_requests,
            toast: &mut toast,
            effects: &mut self.effects,
            post: &mut self.post,
            text_request: &mut self.text_request,
            confirm_request: &mut self.confirm_request,
            thumbnail: self.thumbnail.as_ref(),
            autosave_request: &mut self.autosave_request,
            audio: &mut self.audio,
            tooltip: &mut tooltip,
            nav: self.nav,
            log: &mut self.log,
            modes: &mut self.modes,
            seen: &mut self.seen,
            screenshot_request: &mut self.screenshot_request,
        };

        let next_state = match self.overlays.last_mut() {
            Some((_, overlay)) => match overlay.update(ctx) {
                OverlayAction::Stay => None,
                OverlayAction::Close => {
                    self.overlays.pop();
                    None
                }
                OverlayAction::CloseAll => {
                    self.overlays.clear();
                    None
                }
                OverlayAction::Goto(state) => Some(state),
            },
            None => self.current_screen.update(ctx),
        };

        let wants_keybinds = !keybinds_open
            && self.current_state != ScreenState::TextInput
            && self.keybind_keys.iter().any(|&key| rl.is_key_pressed(key));
        if wants_keybinds {
            self.modes.skip = false;
            self.open_overlay(KEYBINDS_OVERLAY);
        }

        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        self.tooltip_timer.update(tooltip, now, clicked);

        for request in overlay_requests {
            match request {
                OverlayRequest::Open(name) => self.open_overlay(&name),
                OverlayRequest::Close => self.close_overlay(),
                OverlayRequest::CloseAll => self.overlays.clear(),
            }
        }

        if let Some(mut toast) = toast {
            toast.shown_at = Some(now);
            self.toast = Some(toast);
        } else if let Some(toast) = &mut self.toast
            && toast.shown_at.is_none()
        {
            toast.shown_at = Some(now);
        } else if self
            .toast
            .as_ref()
            .is_some_and(|toast| toast.expired(now, &self.toast_config))
        {
            self.toast = None;
        }

        if let Some(state) = next_state {
            self.transition_to(state);
        }

        if self.closing && self.overlay_name() != Some(CONFIRM_OVERLAY) {
            self.closing = false;
        }

        self.update_music(rl.get_frame_time());

        if let Some(errors) = &mut self.script_errors
            && rl.is_key_pressed(SCRIPT_ERRORS_KEY)
        {
            errors.toggle();
        }

        if self.settings.values.fullscreen != self.fullscreen {
            rl.toggle_borderless_windowed();
            self.fullscreen = self.settings.values.fullscreen;
        }
    }

    fn update_music(&mut self, dt: f32) {
        let wanted = match self.current_state {
            ScreenState::StartScreen | ScreenState::MainMenu => {
                self.audio.config().menu_music.clone()
            }
            ScreenState::Playing | ScreenState::TextInput => self.story.music().map(str::to_string),
            _ => self.audio.music().map(str::to_string),
        };

        let settings = &self.settings.values;
        self.audio
            .set_volumes(settings.music_gain(), settings.sound_gain());
        self.audio.set_voice_volume(settings.voice_gain());
        self.audio.play_music(wanted.as_deref());
        self.audio.update(dt);
    }

    pub fn request_close(&mut self) {
        let confirm_open = self.overlay_name() == Some(CONFIRM_OVERLAY);

        let message = match &self.close_confirmation {
            Some(message)
                if !(self.closing && confirm_open)
                    && close_needs_confirmation(&self.current_state, &self.story) =>
            {
                message.clone()
            }
            _ => {
                self.quit_requested = true;
                return;
            }
        };

        if confirm_open {
            self.overlays.pop();
        }
        self.closing = true;
        self.confirm(Confirm::new(message, Action::Quit).confirm_label("Quit"));
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        self.current_screen
            .draw(d, &self.draw_context(self.overlays.is_empty()));
        if std::mem::take(&mut self.screenshot_request) {
            self.take_screenshot(d, thread);
        }
        if std::mem::take(&mut self.autosave_request) {
            self.thumbnail_at = None;
            self.capture_thumbnail(d, thread);
            self.autosave();
        } else {
            self.capture_thumbnail(d, thread);
        }

        let top = self.overlays.len().saturating_sub(1);
        for (index, (_, overlay)) in self.overlays.iter().enumerate() {
            overlay.draw(d, &self.draw_context(index == top));
        }

        let ctx = self.draw_context(false);

        if let Some(errors) = &self.script_errors {
            errors.draw(d, ctx.fonts());
        }

        if let Some(toast) = &self.toast {
            toast.draw(d, ctx.fonts(), &self.toast_config);
        }

        let config = &self.tooltip_config;
        if config.enabled
            && let Some(text) = self.tooltip_timer.visible(d.get_time(), config.delay)
        {
            crate::draw_tooltip(d, ctx.fonts(), text, config);
        }
    }

    fn capture_thumbnail(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let now = d.get_time();
        let due = self
            .thumbnail_at
            .is_none_or(|at| now - at >= THUMBNAIL_INTERVAL);
        let in_game = self.current_state == ScreenState::Playing
            && self.overlays.is_empty()
            && self.story.current().is_some();
        if !due || !in_game {
            return;
        }

        flush_batch();
        let mut image = d.load_image_from_screen(thread);
        let height = THUMBNAIL_WIDTH * image.height().max(1) / image.width().max(1);
        image.resize(THUMBNAIL_WIDTH, height.max(1));
        self.thumbnail = Some(image);
        self.thumbnail_at = Some(now);
    }

    pub fn effects(&self) -> &crate::ScreenEffects {
        &self.effects
    }

    pub fn take_screen_changed(&mut self) -> bool {
        std::mem::take(&mut self.screen_changed)
    }

    pub fn thumbnail(&self) -> Option<&Image> {
        self.thumbnail.as_ref()
    }

    pub fn autosave(&self) {
        crate::saves::autosave(
            &self.saves,
            crate::saves::SaveParts {
                story: &self.story,
                state: &self.state,
                rollback: &self.rollback,
                log: self.log.entries(),
                thumbnail: self.thumbnail.as_ref(),
            },
        );
    }

    fn take_screenshot(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        flush_batch();
        let image = d.load_image_from_screen(thread);
        let dir = self.saves.dir().join("screenshots");
        let path = dir.join(format!("screenshot-{}.png", crate::saves::now()));
        let written = std::fs::create_dir_all(&dir).is_ok() && {
            image.export_image(&path.to_string_lossy());
            path.exists()
        };
        if written {
            println!("Screenshot: {}", path.display());
            self.notify(Toast::info("Screenshot saved"));
        } else {
            self.notify(Toast::error("Could not save the screenshot"));
        }
    }

    fn draw_context(&self, interactive: bool) -> DrawContext<'_> {
        DrawContext {
            resources: &self.resources,
            story: &self.story,
            state: &self.state,
            saves: &self.saves,
            characters: &self.characters,
            settings: &self.settings.values,
            interactive,
            focus_visible: !self.nav.pointer,
            log: &self.log,
            modes: self.modes,
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
        self.effects.clear();
        match crate::swap_story(&mut self.story, story, &mut self.rollback) {
            Ok(RestoreOutcome::Exact) => self.notify(Toast::info("Story reloaded")),
            Ok(RestoreOutcome::SceneRestarted { scene }) => self.notify(Toast::info(format!(
                "Story reloaded; scene '{}' restarted",
                scene
            ))),
            Err(e) => {
                eprintln!("⚠️ Story not reloaded: {}", e);
                self.notify(Toast::error(format!("Story not reloaded: {}", e)));
                return;
            }
        }

        if self.current_state == ScreenState::Playing
            && let Some(screen) = self.factory.create_screen(&ScreenState::Playing)
        {
            self.current_screen = screen;
        }
    }

    pub fn confirm(&mut self, confirm: Confirm) {
        self.confirm_request = Some(confirm);
        self.open_overlay(CONFIRM_OVERLAY);
    }

    pub fn notify(&mut self, toast: Toast) {
        self.toast = Some(toast);
    }

    pub fn quit_requested(&self) -> bool {
        self.quit_requested
    }

    pub fn overlay_name(&self) -> Option<&str> {
        self.overlays.last().map(|(name, _)| name.as_str())
    }

    pub fn overlay_names(&self) -> impl Iterator<Item = &str> {
        self.overlays.iter().map(|(name, _)| name.as_str())
    }

    pub fn open_overlay(&mut self, name: &str) {
        if self.overlay_name() == Some(name) {
            return;
        }
        match self.factory.create_overlay(name) {
            Some(overlay) => self.overlays.push((name.to_string(), overlay)),
            None => eprintln!("⚠️ No overlay registered as '{}'", name),
        }
    }

    pub fn close_overlay(&mut self) {
        self.overlays.pop();
    }

    pub fn close_overlays(&mut self) {
        self.overlays.clear();
    }

    fn transition_to(&mut self, next_state: ScreenState) {
        if next_state == ScreenState::Quit {
            self.quit_requested = true;
            return;
        }

        match self.factory.create_screen(&next_state) {
            Some(screen) => {
                self.modes.skip = false;
                self.thumbnail_at = None;
                self.current_screen = screen;
                let previous = std::mem::replace(&mut self.current_state, next_state);
                self.previous_state = Some(previous);
                self.overlays.clear();
                self.effects.clear();
                self.screen_changed = true;
            }
            None => eprintln!("⚠️ No screen registered for {:?}", next_state),
        }
    }
}

fn flush_batch() {
    unsafe { raylib::ffi::rlDrawRenderBatchActive() };
}
