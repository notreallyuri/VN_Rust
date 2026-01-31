use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(
    Debug, Deserialize, Serialize, JsonSchema, Display, EnumString, Clone, PartialEq, Eq, Hash,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE", ascii_case_insensitive)] // Add this!
pub enum Action {
    Leave,
    Clear,
    Speak,
    Enter,
}

#[derive(
    Debug, Deserialize, Serialize, JsonSchema, Display, EnumString, Clone, PartialEq, Eq, Hash,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE", ascii_case_insensitive)] // Add this!
pub enum Position {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Dialogue {
    pub action: Action,
    pub character: Option<String>,
    pub text: String,
    pub expression: Option<String>,
    pub position: Option<Position>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct Scene {
    pub bg_image: String,
    pub next_scene: Option<String>,
    pub lines: Vec<Dialogue>,
}
