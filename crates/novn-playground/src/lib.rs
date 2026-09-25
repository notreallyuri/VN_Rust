#[cfg(target_arch = "wasm32")]
mod wasm;

use novn_script::{Event, Severity, StoryVm, compile_source};
use serde::{Deserialize, Serialize};

const MAX_STEPS: usize = 2000;

#[derive(Deserialize)]
pub struct Request {
    pub source: String,
    #[serde(default)]
    pub picks: Vec<usize>,
}

#[derive(Serialize, Debug)]
pub struct Note {
    pub severity: &'static str,
    pub line: usize,
    pub message: String,
}

#[derive(Serialize, Debug)]
pub struct Counts {
    pub scenes: usize,
    pub instructions: usize,
}

#[derive(Serialize, Debug)]
pub struct Step {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taken: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Option_ {
    pub index: usize,
    pub text: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub ok: bool,
    pub notes: Vec<Note>,
    pub listing: String,
    pub counts: Counts,
    pub steps: Vec<Step>,
    pub choices: Vec<Option_>,
    pub ended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stopped: Option<String>,
}

fn say(speaker: Option<String>, text: String) -> Step {
    Step {
        kind: if speaker.is_some() {
            "dialogue"
        } else {
            "narration"
        },
        speaker,
        text: Some(text),
        detail: None,
        taken: None,
    }
}

fn stage(kind: &'static str, detail: String) -> Step {
    Step {
        kind,
        speaker: None,
        text: None,
        detail: Some(detail),
        taken: None,
    }
}

fn describe(event: &Event) -> Option<Step> {
    Some(match event {
        Event::Say { speaker, text } => say(speaker.clone(), text.clone()),
        Event::Show {
            character, image, ..
        } => stage("show", format!("{character} {image}")),
        Event::Hide { character, .. } => stage("remove", character.clone()),
        Event::Clear { .. } => stage("clear", String::new()),
        Event::Background { image, .. } => {
            stage("background", image.clone().unwrap_or("none".into()))
        }
        Event::Music { track } => stage("music", track.clone().unwrap_or("none".into())),
        Event::Sound { id } => stage("sound", id.clone()),
        Event::Voice { id } => stage("voice", id.clone()),
        Event::Call { command, args } => {
            let mut detail = command.clone();
            for arg in args {
                detail.push(' ');
                detail.push_str(arg);
            }
            stage("call", detail)
        }
        Event::Commit => stage("commit", String::new()),
        _ => return None,
    })
}

fn options(event: &Event) -> Vec<Option_> {
    match event {
        Event::Choice { options } => options
            .iter()
            .map(|option| Option_ {
                index: option.index,
                text: option.text.clone(),
                enabled: option.enabled,
                reason: option.reason.clone(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub fn run(request: Request) -> Response {
    let program = compile_source(&request.source);

    let mut notes: Vec<Note> = program
        .diagnostics
        .iter()
        .map(|diagnostic| Note {
            severity: match diagnostic.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            line: diagnostic.line,
            message: diagnostic.message.clone(),
        })
        .collect();

    for scene in program.unknown_jump_targets() {
        notes.push(Note {
            severity: "error",
            line: 0,
            message: format!("jump to unknown scene '{scene}'"),
        });
    }

    let counts = Counts {
        scenes: program.scene_order.len(),
        instructions: program.instructions.len(),
    };
    let listing = program.listing();
    let ok = !notes.iter().any(|note| note.severity == "error");

    if !ok || counts.instructions == 0 {
        return Response {
            ok,
            notes,
            listing,
            counts,
            steps: Vec::new(),
            choices: Vec::new(),
            ended: false,
            stopped: None,
        };
    }

    let mut vm = StoryVm::from_program(program);
    let mut steps = Vec::new();
    let mut picks = request.picks.iter().copied();
    let mut stopped = None;
    let mut ended = false;
    let mut choices = Vec::new();

    for _ in 0..MAX_STEPS {
        let event = vm.advance();
        match &event {
            Event::End => {
                ended = true;
                break;
            }
            Event::Choice { .. } => {
                let Some(pick) = picks.next() else {
                    choices = options(&event);
                    break;
                };
                let chosen = options(&event)
                    .into_iter()
                    .find(|option| option.index == pick);
                match vm.choose(pick) {
                    Ok(()) => steps.push(Step {
                        kind: "chose",
                        speaker: None,
                        text: None,
                        detail: None,
                        taken: chosen.map(|option| option.text),
                    }),
                    Err(error) => {
                        stopped = Some(format!("{error:?}"));
                        break;
                    }
                }
            }
            other => {
                if let Some(step) = describe(other) {
                    steps.push(step);
                }
            }
        }
    }

    if steps.len() >= MAX_STEPS {
        stopped = Some(format!("stopped after {MAX_STEPS} steps"));
    }

    Response {
        ok,
        notes,
        listing,
        counts,
        steps,
        choices,
        ended,
        stopped,
    }
}

pub fn run_json(request: &str) -> String {
    let response = match serde_json::from_str::<Request>(request) {
        Ok(request) => run(request),
        Err(error) => Response {
            ok: false,
            notes: vec![Note {
                severity: "error",
                line: 0,
                message: format!("the playground could not read its request: {error}"),
            }],
            listing: String::new(),
            counts: Counts {
                scenes: 0,
                instructions: 0,
            },
            steps: Vec::new(),
            choices: Vec::new(),
            ended: false,
            stopped: None,
        },
    };
    serde_json::to_string(&response).unwrap_or_else(|error| {
        format!(r#"{{"ok":false,"notes":[{{"severity":"error","line":0,"message":"{error}"}}]}}"#)
    })
}
