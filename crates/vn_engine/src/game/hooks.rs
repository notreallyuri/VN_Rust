use crate::context::GameContext;
use crate::screen::ScreenState;

type SceneHook = Box<dyn Fn(&mut GameContext, &str) -> Option<ScreenState>>;
type ChoiceHook = Box<dyn Fn(&mut GameContext, usize, &str) -> Option<ScreenState>>;

#[derive(Default)]
pub struct Hooks {
    scene_enter: Vec<SceneHook>,
    choice: Vec<ChoiceHook>,
}

impl Hooks {
    pub fn on_scene_enter(
        &mut self,
        hook: impl Fn(&mut GameContext, &str) -> Option<ScreenState> + 'static,
    ) {
        self.scene_enter.push(Box::new(hook));
    }

    pub fn on_choice(
        &mut self,
        hook: impl Fn(&mut GameContext, usize, &str) -> Option<ScreenState> + 'static,
    ) {
        self.choice.push(Box::new(hook));
    }

    pub fn scene_enter_count(&self) -> usize {
        self.scene_enter.len()
    }

    pub fn choice_count(&self) -> usize {
        self.choice.len()
    }

    pub(crate) fn scene_entered(&self, ctx: &mut GameContext, scene: &str) -> Option<ScreenState> {
        self.scene_enter
            .iter()
            .fold(None, |next, hook| next.or(hook(ctx, scene)))
    }

    pub(crate) fn choice_made(
        &self,
        ctx: &mut GameContext,
        index: usize,
        text: &str,
    ) -> Option<ScreenState> {
        self.choice
            .iter()
            .fold(None, |next, hook| next.or(hook(ctx, index, text)))
    }
}
