use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use raylib::prelude::*;

use vn_script::StoryVm;

use crate::{
    Characters, Commands, DrawContext, GameContext, GameState, Overlay, OverlayAction,
    OverlayRequest, ResourceManager, Rollback, Saves, Screen, ScreenState, TextRequest, Toast,
    ToastConfig,
};

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
    pub saves: Saves,
    pub previous_state: Option<ScreenState>,
    pub characters: Characters,
    pub rollback: Rollback,
    text_request: Option<TextRequest>,
    confirm_request: Option<crate::screens::Confirm>,
    pub toast_config: ToastConfig,
    toast: Option<Toast>,
    overlays: Vec<(String, Box<dyn Overlay>)>,
    quit_requested: bool,
}

impl ScreenStateManager {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        initial_state: ScreenState,
        factory: Box<dyn ScreenFactory>,
        assets_root: impl Into<PathBuf>,
        story_dir: &str,
    ) -> io::Result<Self> {
        let assets_root = assets_root.into();
        let story = StoryVm::from_dir(assets_root.join(story_dir))?;
        Self::with_story(rl, thread, initial_state, factory, assets_root, story)
    }

    pub fn with_story(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        initial_state: ScreenState,
        factory: Box<dyn ScreenFactory>,
        assets_root: impl Into<PathBuf>,
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
            saves: Saves::new("saves", "game"),
            previous_state: None,
            characters: Characters::default(),
            rollback: Rollback::default(),
            text_request: None,
            confirm_request: None,
            toast_config: ToastConfig::default(),
            toast: None,
            overlays: Vec::new(),
            quit_requested: false,
        })
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let now = rl.get_time();
        let mut overlay_requests = Vec::new();
        let mut toast = None;

        let ctx = GameContext {
            rl,
            thread,
            resources: &mut self.resources,
            story: &mut self.story,
            state: &mut self.state,
            saves: &self.saves,
            previous: self.previous_state.as_ref(),
            rollback: &mut self.rollback,
            commands: Rc::clone(&self.commands),
            overlay_requests: &mut overlay_requests,
            toast: &mut toast,
            text_request: &mut self.text_request,
            confirm_request: &mut self.confirm_request,
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
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        let ctx = DrawContext {
            resources: &self.resources,
            story: &self.story,
            state: &self.state,
            saves: &self.saves,
            characters: &self.characters,
        };

        self.current_screen.draw(d, &ctx);

        for (_, overlay) in &self.overlays {
            overlay.draw(d, &ctx);
        }

        if let Some(toast) = &self.toast {
            toast.draw(d, ctx.fonts(), &self.toast_config);
        }
    }

    pub fn confirm(&mut self, confirm: crate::screens::Confirm) {
        self.confirm_request = Some(confirm);
        self.open_overlay(crate::screens::CONFIRM_OVERLAY);
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
                println!("Transitioning to: {:?}", next_state);
                self.current_screen = screen;
                let previous = std::mem::replace(&mut self.current_state, next_state);
                self.previous_state = Some(previous);
                self.overlays.clear();
            }
            None => eprintln!("⚠️ No screen registered for {:?}", next_state),
        }
    }
}
