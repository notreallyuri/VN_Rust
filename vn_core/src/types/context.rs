use crate::providers::StoryProvider;

pub struct GameContext<'a, U, R> {
    pub platform: &'a mut U,
    pub resources: &'a mut R,
    pub story: &'a mut StoryProvider,
}
