use crate::{
    providers::StoryProvider,
    types::{GameContext, Screen, ScreenState},
};

pub struct ScreenStateManager<P, U, D, R>
where
    P: ScreenProvider<U, D, R>,
{
    pub current_screen: Box<dyn Screen<U, D, R>>,
    pub current_state: ScreenState,
    pub provider: P,
    pub resources: R,
    pub story: StoryProvider,
}

pub trait ScreenProvider<U, D, R> {
    fn create_screen(&self, state: &ScreenState) -> Box<dyn Screen<U, D, R>>;
}

impl<P, U, D, R> ScreenStateManager<P, U, D, R>
where
    P: ScreenProvider<U, D, R>,
    R: Default, // We assume resources can be created via Default::default()
{
    pub fn new(initial_state: ScreenState, provider: P, first_script: &str) -> Self {
        // We initialize the generic ResourceProvider here
        let resources = R::default();
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

    pub fn update(&mut self, platform_ctx: &mut U) {
        let next_state = {
            let ctx = GameContext {
                platform: platform_ctx,
                resources: &mut self.resources,
                story: &mut self.story,
            };
            self.current_screen.update(ctx)
        };

        if let Some(state) = next_state {
            self.transition_to(state);
        }
    }

    pub fn draw(&mut self, draw_target: &mut D) {
        self.current_screen
            .draw(draw_target, &mut self.resources, &self.story);
    }

    pub fn transition_to(&mut self, next_state: ScreenState) {
        self.current_state = next_state.clone();
        self.current_screen = self.provider.create_screen(&next_state);
    }
}
