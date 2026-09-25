use novn_script::Event;
use novn_script::markup::plain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    Line {
        speaker: Option<String>,
        text: String,
    },
    Choices {
        options: Vec<String>,
    },
    Option {
        text: String,
    },
    Notice {
        text: String,
    },
}

impl Reading {
    pub fn of(event: &Event, name: impl Fn(&str) -> String) -> Option<Reading> {
        match event {
            Event::Say { speaker, text } => Some(Reading::Line {
                speaker: speaker.as_deref().map(name),
                text: plain(text),
            }),
            Event::Choice { options } => Some(Reading::Choices {
                options: options
                    .iter()
                    .filter(|option| option.enabled)
                    .map(|option| plain(&option.text))
                    .collect(),
            }),
            _ => None,
        }
    }

    pub fn text(&self) -> String {
        match self {
            Reading::Line {
                speaker: Some(speaker),
                text,
            } => format!("{speaker}: {text}"),
            Reading::Line { text, .. } => text.clone(),
            Reading::Choices { options } => options
                .iter()
                .map(|option| sentence(option))
                .collect::<Vec<_>>()
                .join(" "),
            Reading::Option { text } | Reading::Notice { text } => text.clone(),
        }
    }
}

fn sentence(text: &str) -> String {
    let text = text.trim();
    match text.chars().last() {
        Some('.' | '!' | '?' | '…' | ':' | ';') | None => text.to_string(),
        Some(_) => format!("{text}."),
    }
}
