use novn::prelude::*;
use novn::raylib::prelude::Color;

fn person(name: &str, color: (u8, u8, u8), images: &[&str]) -> Character {
    Character::new(name)
        .color(Color::new(color.0, color.1, color.2, 255))
        .images(images.iter().copied())
}

pub fn register(app: VnApp) -> VnApp {
    app.character(
        "registrar",
        person("The Registrar", (170, 182, 205), &["neutral", "stern"])
            .box_style(|b| b.color(crate::style::PANEL_DEEP).roundness(0.0)),
    )
    .character(
        "mary",
        person("Mary", (226, 200, 150), &["tired", "afraid", "resolved"]),
    )
    .character(
        "hugo",
        person("Hugo Von Lucis", (170, 190, 220), &["neutral", "tired"]),
    )
    .character(
        "adelaide",
        person("Adelaide", (205, 175, 205), &["neutral", "afraid"]),
    )
    .character(
        "clara",
        person(
            "Sister Clara",
            (180, 210, 170),
            &["neutral", "guarded", "unveiled"],
        ),
    )
    .character("francis", person("Francis", (215, 180, 150), &["neutral"]))
    .character(
        "moriarty",
        person("Moriarty", (215, 135, 120), &["neutral", "older", "wary"]),
    )
    .character(
        "house_envoy",
        person("Envoy of the House", (185, 185, 190), &["neutral"]),
    )
    .character(
        "man_in_black",
        person("The Man in Black", (150, 150, 165), &["neutral"]),
    )
    .character("shadow", person("Shadow", (160, 148, 200), &["neutral"]))
    .character(
        "shadow_third",
        person("The Third Shadow", (160, 148, 200), &["neutral"]),
    )
    .character("guest", person("The Guest", (160, 160, 160), &["neutral"]))
    .character(
        "gabriel",
        person("Gabriel", (240, 210, 160), &["neutral", "curious"]),
    )
    .variable("player_name", VariableDef::string("Archivist"))
    .variable(
        "approach",
        VariableDef::enumeration(["careful", "bold"], "careful"),
    )
    .variable(
        "verdict",
        VariableDef::enumeration(["unwritten", "miracle", "sin", "unknown"], "unwritten"),
    )
    .variable("trust", VariableDef::int(0))
    .variable("suspicion", VariableDef::int(0))
    .variable("read_letter", VariableDef::bool(false))
    .variable("saw_torn_page", VariableDef::bool(false))
    .variable("believed_moriarty", VariableDef::bool(false))
    .variable("recognized_clara", VariableDef::bool(false))
    .variable(
        "ending",
        VariableDef::enumeration(["none", "report", "silence", "keeper"], "none"),
    )
    .variable("cases_closed", VariableDef::int(0))
}
