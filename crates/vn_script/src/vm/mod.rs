mod build;
mod error;
mod event;
mod snapshot;
mod state;
mod step;

pub use error::*;
pub use event::*;
pub use snapshot::*;

use std::collections::HashMap;

use crate::{Catalog, Position, Program, Schema, Transition, Value};

#[derive(Debug)]
pub struct StoryVm {
    program: Program,
    ip: usize,
    current_scene: Option<String>,
    variables: HashMap<String, Value>,
    active_characters: HashMap<String, String>,
    positions: HashMap<String, Position>,
    background: Option<String>,
    music: Option<String>,
    pending_transition: Option<Transition>,
    pending_choice: Option<usize>,
    current: Option<Event>,
    schema: Schema,
    entry: Option<String>,
    scene_events: bool,
    entered: bool,
    catalog: Option<Catalog>,
}
