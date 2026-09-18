use vn_engine::raylib::prelude::Color;
use vn_engine::{Character, VariableDef, VnApp};

pub fn register(app: VnApp) -> VnApp {
    app.character(
        "mary",
        Character::new("Mary")
            .color(Color::new(222, 196, 140, 255))
            .images(["tired"]),
    )
    .character(
        "hugo",
        Character::new("Hugo Von Lucis")
            .color(Color::new(170, 190, 220, 255))
            .images(["neutral", "tired"]),
    )
    .character(
        "adelaide",
        Character::new("Adelaide")
            .color(Color::new(200, 170, 200, 255))
            .images(["neutral", "afraid"]),
    )
    .character(
        "clara",
        Character::new("Clara")
            .color(Color::new(190, 215, 180, 255))
            .images(["neutral"]),
    )
    .character(
        "francis",
        Character::new("Francis")
            .color(Color::new(210, 180, 150, 255))
            .images(["neutral"]),
    )
    .character(
        "moriarty",
        Character::new("Moriarty")
            .color(Color::new(200, 120, 110, 255))
            .images(["neutral"]),
    )
    .character(
        "house_envoy",
        Character::new("Envoy of the House")
            .color(Color::new(180, 180, 180, 255))
            .images(["neutral"]),
    )
    .character(
        "man_in_black",
        Character::new("The Man in Black")
            .color(Color::new(150, 150, 160, 255))
            .images(["neutral"]),
    )
    .character(
        "shadow",
        Character::new("Shadow")
            .color(Color::new(130, 120, 150, 255))
            .images(["neutral"]),
    )
    .character(
        "shadow_third",
        Character::new("The Third Shadow")
            .color(Color::new(130, 120, 150, 255))
            .images(["neutral"]),
    )
    .character("guest", Character::new("Guest").images(["neutral"]))
    .variable("curiosity", VariableDef::int(0))
    .variable("obedience", VariableDef::int(0))
    .variable("suspicion", VariableDef::int(0))
    .variable("player_name", VariableDef::string("Reader"))
}
