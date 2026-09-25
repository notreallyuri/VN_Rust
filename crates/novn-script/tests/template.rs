use std::collections::HashMap;

use novn_script::{Value, interpolate};

fn vars() -> HashMap<String, Value> {
    HashMap::from([
        ("player_name".to_string(), Value::String("Yuri".into())),
        ("coins".to_string(), Value::Int(12)),
        ("met_mary".to_string(), Value::Bool(true)),
        ("route".to_string(), Value::Enum("good".into())),
    ])
}

#[test]
fn replaces_every_value_type() {
    assert_eq!(
        interpolate(
            "{player_name} has {coins} coins, {met_mary}, {route}.",
            &vars()
        ),
        "Yuri has 12 coins, true, good."
    );
}

#[test]
fn unset_variables_stay_as_written() {
    assert_eq!(interpolate("Hi, {nobody}.", &vars()), "Hi, {nobody}.");
}

#[test]
fn doubled_braces_are_literal() {
    assert_eq!(
        interpolate("{{player_name}} is {player_name}}}", &vars()),
        "{player_name} is Yuri}"
    );
}

#[test]
fn non_identifiers_are_left_alone() {
    assert_eq!(
        interpolate("{ player_name } {Player_Name}", &vars()),
        "{ player_name } {Player_Name}"
    );
}

#[test]
fn unclosed_brace_is_left_alone() {
    assert_eq!(interpolate("{player_name", &vars()), "{player_name");
}

#[test]
fn text_without_braces_is_unchanged() {
    assert_eq!(
        interpolate("Plain line, ünïcode — fine.", &vars()),
        "Plain line, ünïcode — fine."
    );
}
