use crate::runtime::{
    ResourceManager,
    types::{GameContext, Screen, ScreenState},
};
use crate::script::provider::StoryProvider;
use raylib::prelude::*;

pub struct ScreenStateManager {
    pub current_screen: Box<dyn Screen>,
    pub current_state: ScreenState,
    pub factory: Box<dyn ScreenFactory>,
    pub resources: ResourceManager,
    pub story: StoryProvider,
}

pub trait ScreenFactory {
    fn create_screen(&self, state: &ScreenState) -> Box<dyn Screen>;
}

impl ScreenStateManager {
    pub fn new(
        initial_state: ScreenState,
        factory: Box<dyn ScreenFactory>,
        first_script: &str,
    ) -> Self {
        let resources = ResourceManager::default();
        let story = StoryProvider::from_source(first_script);
        let current_screen = factory.create_screen(&initial_state);

        Self {
            current_screen,
            current_state: initial_state,
            factory,
            resources,
            story,
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let next_state = {
            let ctx = GameContext {
                rl,
                thread,
                resources: &mut self.resources,
                story: &mut self.story,
            };
            self.current_screen.update(ctx)
        };

        if let Some(state) = next_state {
            self.transition_to(state);
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle) {
        self.current_screen
            .draw(d, &mut self.resources, &self.story);
    }

    fn transition_to(&mut self, next_state: ScreenState) {
        println!("Transitioning to: {:?}", next_state);
        self.current_state = next_state.clone();
        self.current_screen = self.factory.create_screen(&next_state);
    }
}
