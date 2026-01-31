use crate::{
    providers::{ResourceProvider, StoryProvider},
    types::{GameContext, Screen, ScreenState},
};
use raylib::prelude::*;

pub struct ScreenStateManager<P: ScreenProvider> {
    pub current_screen: Box<dyn Screen>,
    pub current_state: ScreenState,
    pub provider: P,
    pub resources: ResourceProvider,
    pub story: StoryProvider,
}

pub trait ScreenProvider {
    fn create_screen(&self, state: &ScreenState) -> Box<dyn Screen>;
}

impl<P: ScreenProvider> ScreenStateManager<P> {
    pub fn new(initial_state: ScreenState, provider: P, first_script: &str) -> Self {
        let resources = ResourceProvider::new();
        let story = StoryProvider::new(first_script);

        let current_screen = provider.create_screen(&initial_state);

        Self {
            current_screen,
            current_state: initial_state,
            provider,
            resources,
            story,
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let next_state = {
            let mut ctx = GameContext {
                rl,
                thread,
                resources: &mut self.resources,
                story: &mut self.story,
            };
            self.current_screen.update(&mut ctx)
        };

        if let Some(state) = next_state {
            self.transition_to(state);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread) {
        self.current_screen
            .draw(d, thread, &mut self.resources, &self.story);
    }

    pub fn transition_to(&mut self, next_state: ScreenState) {
        self.current_state = next_state.clone();
        self.current_screen = self.provider.create_screen(&next_state);
    }
}
