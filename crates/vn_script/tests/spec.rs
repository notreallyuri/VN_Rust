use std::fs;
use std::path::PathBuf;

use vn_script::{Event, StoryVm, compile_source};

const SCRIPT_MD: &str = include_str!("../../../SCRIPT.md");
const GOLDEN: &str = "tests/golden/script_md.txt";
const MAX_EVENTS: usize = 100;

struct Block {
    line: usize,
    heading: String,
    text: String,
}

fn blocks() -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut heading = String::new();
    let mut current: Option<Block> = None;

    for (index, line) in SCRIPT_MD.lines().enumerate() {
        match &mut current {
            Some(block) if line.trim_start().starts_with("```") => {
                blocks.push(std::mem::replace(block, placeholder()));
                current = None;
            }
            Some(block) => {
                block.text.push_str(line);
                block.text.push('\n');
            }
            None if line.trim_start() == "```story" => {
                current = Some(Block {
                    line: index + 1,
                    heading: heading.clone(),
                    text: String::new(),
                });
            }
            None if line.starts_with('#') => {
                heading = line.trim_start_matches('#').trim().to_string();
            }
            None => {}
        }
    }
    blocks
}

fn placeholder() -> Block {
    Block {
        line: 0,
        heading: String::new(),
        text: String::new(),
    }
}

fn is_template(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.iter().enumerate().any(|(i, &b)| {
        b == b'<'
            && bytes[i + 1..]
                .iter()
                .position(|&c| c == b'>')
                .is_some_and(|end| {
                    end > 0
                        && bytes[i + 1..i + 1 + end]
                            .iter()
                            .all(|c| c.is_ascii_lowercase() || *c == b'_')
                })
    })
}

fn as_story(text: &str) -> String {
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            if line.trim() == "..." {
                let indent = line.len() - line.trim_start().len();
                format!("{}\"...\"", " ".repeat(indent))
            } else {
                line.to_string()
            }
        })
        .collect();

    if lines.iter().any(|line| line.starts_with("scene ")) {
        return lines.join("\n") + "\n";
    }

    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    let body: Vec<String> = lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("  {}", &line[indent..])
            }
        })
        .collect();
    format!("scene example:\n{}\n", body.join("\n"))
}

fn trace(source: &str) -> String {
    let mut vm = StoryVm::from_source(source);
    let mut out = String::new();
    for _ in 0..MAX_EVENTS {
        let event = vm.advance();
        out.push_str(&format!("> {:?}\n", event));
        match event {
            Event::End => return out,
            Event::Choice { .. } => {
                if let Err(e) = vm.choose(0) {
                    out.push_str(&format!("! choose(0): {:?}\n", e));
                    return out;
                }
            }
            _ => {}
        }
    }
    out.push_str("! stopped after the event limit\n");
    out
}

fn render() -> (String, Vec<String>) {
    let mut out = String::from(
        "# Every example in SCRIPT.md: the story as compiled, its instructions and the\n\
         # events the VM produces (taking the first option of every choice).\n\
         # Regenerate with: UPDATE_GOLDEN=1 cargo test -p vn_script --test spec\n",
    );
    let mut failures = Vec::new();

    for block in blocks().iter().filter(|block| !is_template(&block.text)) {
        let source = as_story(&block.text);
        let program = compile_source(&source);
        for diagnostic in &program.diagnostics {
            failures.push(format!(
                "SCRIPT.md:{} ({}): {}",
                block.line, block.heading, diagnostic
            ));
        }

        out.push_str(&format!(
            "\n======== SCRIPT.md:{}  {}\n",
            block.line, block.heading
        ));
        out.push_str(&source);
        out.push_str("-------- instructions");
        out.push_str(&program.listing());
        out.push_str("-------- events\n");
        out.push_str(&trace(&source));
    }
    (out, failures)
}

#[test]
fn every_example_compiles_without_diagnostics() {
    let (_, failures) = render();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn examples_match_the_golden_file() {
    let (actual, _) = render();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN);

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &actual).unwrap();
        return;
    }

    let expected = fs::read_to_string(&path).unwrap_or_default();
    if expected != actual {
        let line = expected
            .lines()
            .zip(actual.lines())
            .position(|(a, b)| a != b)
            .unwrap_or_else(|| expected.lines().count().min(actual.lines().count()));
        let show = |text: &str| text.lines().nth(line).unwrap_or("<end>").to_string();
        panic!(
            "{} is out of date at line {}:\n  expected: {}\n  actual:   {}\n\
             If the change is intended: UPDATE_GOLDEN=1 cargo test -p vn_script --test spec",
            GOLDEN,
            line + 1,
            show(&expected),
            show(&actual)
        );
    }
}

#[test]
fn templates_are_told_apart_from_examples() {
    assert!(is_template("show <character_id> <image_id>\n"));
    assert!(is_template("scene <scene_id>:\n"));
    assert!(!is_template("if affection < 3 && trust > 1:\n"));
    assert!(!is_template("show mary tired at left\n"));

    let examples = blocks()
        .iter()
        .filter(|block| !is_template(&block.text))
        .count();
    assert!(examples >= 15, "only {} examples found", examples);
}

#[test]
fn fragments_are_wrapped_in_a_scene() {
    assert_eq!(
        as_story("  show mary tired\n  ...\n"),
        "scene example:\n  show mary tired\n  \"...\"\n"
    );
    assert_eq!(
        as_story("scene start:\n  \"hi\"\n"),
        "scene start:\n  \"hi\"\n"
    );
}
