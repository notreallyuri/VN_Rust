use crate::types::{Action, Dialogue, Position, Scene};
use regex::Regex;
use std::str::FromStr;

pub fn parse_vn_script(content: &str) -> Scene {
    let mut lines = Vec::new();
    let mut bg_image = String::from("default_bg.png");
    let mut next_scene = None;

    let dialogue_re = Regex::new(
        r#"(?x)
        ^(?P<name>\w+)                 # Name
        (?:\s*\((?P<expr>\w+)\))?     # (Expression)
        (?:\s+(?P<pos>\w+))?          # Position
        :\s*"(?P<text>.*)"$            # : "Text"
    "#,
    )
    .unwrap();

    let command_re = Regex::new(
        r#"(?x)
    ^\[(?P<action>\w+)\]           # [ACTION]
    (?:\s+(?P<name>\w+))?          # Optional Name
    (?:\s*\((?P<expr>\w+)\))?     # Optional (Expression)
    (?:\s+(?P<pos>\w+))?          # Optional Position
    (?:\s*:\s*"(?P<text>.*)")?     # Optional : "Text"
"#,
    )
    .unwrap();

    let narrator_re = Regex::new(r#"(?x)^\s*"(?P<text>.*)"\s*$"#).unwrap();

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("BG:") {
            bg_image = rest.trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("NEXT:") {
            next_scene = Some(rest.trim().to_string());
        }
        // --- CHECK COMMANDS FIRST [ENTER], [LEAVE], [CLEAR] ---
        else if let Some(caps) = command_re.captures(trimmed) {
            lines.push(Dialogue {
                action: Action::from_str(caps.name("action").unwrap().as_str())
                    .unwrap_or(Action::Speak),
                character: caps.name("name").map(|m| m.as_str().to_string()),
                text: caps
                    .name("text")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
                expression: caps.name("expr").map(|m| m.as_str().to_string()),
                position: caps
                    .name("pos")
                    .and_then(|m| Position::from_str(m.as_str()).ok()),
            });
        }
        // --- CHECK DIALOGUE SECOND ---
        else if let Some(caps) = dialogue_re.captures(trimmed) {
            lines.push(Dialogue {
                action: Action::Speak,
                character: Some(caps.name("name").unwrap().as_str().to_string()),
                text: caps.name("text").unwrap().as_str().to_string(),
                expression: caps.name("expr").map(|m| m.as_str().to_string()),
                position: caps
                    .name("pos")
                    .and_then(|m| Position::from_str(m.as_str()).ok()),
            });
        }
        // --- CHECK NARRATOR LAST ---
        else if let Some(caps) = narrator_re.captures(trimmed) {
            lines.push(Dialogue {
                action: Action::Speak,
                character: None,
                text: caps.name("text").unwrap().as_str().to_string(),
                expression: None,
                position: None,
            });
        }
    }

    Scene {
        bg_image,
        next_scene,
        lines,
    }
}
