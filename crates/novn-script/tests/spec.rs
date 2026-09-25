use std::fs;
use std::path::{Path, PathBuf};

use novn_script::{Event, StoryVm, compile_source};

const GOLDEN: &str = "tests/golden/examples.txt";
const MAX_EVENTS: usize = 100;
const MIN_EXAMPLES: usize = 15;

struct Source {
    label: String,
    text: String,
}

struct Block {
    source: String,
    line: usize,
    heading: String,
    text: String,
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pages(dir: &Path, route: &str, out: &mut Vec<Source>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{} cannot be read: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            pages(&path, &format!("{route}/{name}"), out);
        } else if path.file_name().is_some_and(|n| n == "page.mdx") {
            out.push(Source {
                label: format!("docs{route}"),
                text: fs::read_to_string(&path).unwrap(),
            });
        }
    }
}

fn sources() -> Option<Vec<Source>> {
    let root = repo();
    let script = root.join("SCRIPT.md");
    if !script.is_file() {
        return None;
    }

    let mut out = vec![Source {
        label: "SCRIPT.md".to_string(),
        text: fs::read_to_string(&script).unwrap(),
    }];

    let docs = root.join("docs/src/app/docs");
    assert!(
        docs.is_dir(),
        "{} is missing. The story examples live in the documentation pages, and this \
         test compiles them; if the pages moved, point it at where they went rather \
         than leaving them unchecked.",
        docs.display()
    );
    pages(&docs, "", &mut out);
    Some(out)
}

fn blocks() -> Option<Vec<Block>> {
    let mut blocks = Vec::new();

    for source in sources()? {
        let mut heading = String::new();
        let mut current: Option<Block> = None;

        for (index, line) in source.text.lines().enumerate() {
            match &mut current {
                Some(block) if line.trim_start().starts_with("```") => {
                    blocks.push(std::mem::replace(block, placeholder()));
                    current = None;
                }
                Some(block) => {
                    block.text.push_str(line);
                    block.text.push('\n');
                }
                None if is_story_fence(line.trim_start()) => {
                    current = Some(Block {
                        source: source.label.clone(),
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
    }
    Some(blocks)
}

fn is_story_fence(line: &str) -> bool {
    line == "```story" || line == "```story-play"
}

fn placeholder() -> Block {
    Block {
        source: String::new(),
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

fn render() -> Option<(String, Vec<String>, usize)> {
    let mut out = String::from(
        "# Every story example in SCRIPT.md and the documentation pages: the story as\n\
         # compiled, its instructions and the events the VM produces (taking the first\n\
         # option of every choice).\n\
         # Regenerate with: UPDATE_GOLDEN=1 cargo test -p novn-script --test spec\n",
    );
    let mut failures = Vec::new();
    let mut examples = 0;

    for block in blocks()?.iter().filter(|block| !is_template(&block.text)) {
        examples += 1;
        let source = as_story(&block.text);
        let program = compile_source(&source);
        for diagnostic in &program.diagnostics {
            failures.push(format!(
                "{}:{} ({}): {}",
                block.source, block.line, block.heading, diagnostic
            ));
        }

        out.push_str(&format!(
            "\n======== {}:{}  {}\n",
            block.source, block.line, block.heading
        ));
        out.push_str(&source);
        out.push_str("-------- instructions");
        out.push_str(&program.listing());
        out.push_str("-------- events\n");
        out.push_str(&trace(&source));
    }
    Some((out, failures, examples))
}

fn outside_the_repository() -> bool {
    eprintln!("skipped: SCRIPT.md is not beside the crate, so this is not the repository");
    true
}

#[test]
fn every_example_compiles_without_diagnostics() {
    let Some((_, failures, _)) = render() else {
        assert!(outside_the_repository());
        return;
    };
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn examples_match_the_golden_file() {
    let Some((actual, _, _)) = render() else {
        assert!(outside_the_repository());
        return;
    };
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
             If the change is intended: UPDATE_GOLDEN=1 cargo test -p novn-script --test spec",
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

    let Some((_, _, examples)) = render() else {
        assert!(outside_the_repository());
        return;
    };
    assert!(
        examples >= MIN_EXAMPLES,
        "only {examples} examples found, expected at least {MIN_EXAMPLES}. The story \
         examples are the documentation's test suite; if they thinned out this much, \
         something moved without this test following it."
    );
}

#[test]
fn every_page_with_story_blocks_is_reached() {
    let Some(blocks) = blocks() else {
        assert!(outside_the_repository());
        return;
    };
    let mut labels: Vec<&str> = blocks.iter().map(|block| block.source.as_str()).collect();
    labels.sort_unstable();
    labels.dedup();

    assert!(
        labels.contains(&"SCRIPT.md"),
        "SCRIPT.md contributed no examples"
    );
    let pages = labels.iter().filter(|l| l.starts_with("docs/")).count();
    assert!(
        pages >= 8,
        "only {pages} documentation pages contributed examples: {labels:?}"
    );
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
