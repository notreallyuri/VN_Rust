use raylib::{RaylibHandle, RaylibThread};

use crate::runtime::ResourceManager;
use crate::script::provider::StoryProvider;

pub struct GameContext<'a> {
    pub rl: &'a mut RaylibHandle,
    pub thread: &'a RaylibThread,
    pub resources: &'a mut ResourceManager,
    pub story: &'a mut StoryProvider,
}
