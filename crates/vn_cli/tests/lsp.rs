use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};
use vn_script::{CharacterDef, CommandSig, ParamKind, Schema, SchemaFile, VariableDef};

struct Project(PathBuf);

impl Project {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "vn_cli_lsp_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("story")).unwrap();
        fs::create_dir_all(root.join("backgrounds")).unwrap();
        fs::write(root.join("backgrounds/hall.png"), b"").unwrap();
        fs::write(root.join("backgrounds/study.png"), b"").unwrap();
        fs::create_dir_all(root.join("choices")).unwrap();
        fs::write(root.join("choices/door.png"), b"").unwrap();
        fs::create_dir_all(root.join("previews")).unwrap();
        fs::write(root.join("previews/hall_view.png"), b"").unwrap();

        let mut schema = Schema::default();
        schema.variables.insert("trust".into(), VariableDef::int(0));
        schema
            .variables
            .insert("met".into(), VariableDef::bool(false));
        schema.variables.insert(
            "route".into(),
            VariableDef::enumeration(["good", "bad"], "good"),
        );
        schema.characters.insert(
            "mary".into(),
            CharacterDef {
                name: "Mary".into(),
                images: vec!["neutral".into(), "tired".into()],
            },
        );
        schema.commands.insert(
            "give_item".into(),
            CommandSig {
                required: vec![ParamKind::Word],
                ..CommandSig::default()
            },
        );
        schema.commands.insert(
            "close_case".into(),
            CommandSig {
                required: vec![ParamKind::Choice(vec![
                    "report".into(),
                    "silence".into(),
                    "keeper".into(),
                ])],
                ..CommandSig::default()
            },
        );
        let mut file = SchemaFile::new("Test", "story", schema);
        file.entry_scene = Some("start".into());
        file.write(root.join("schema.json")).unwrap();
        Self(root.canonicalize().unwrap())
    }

    fn story(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join("story").join(name);
        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn uri(path: &Path) -> String {
    format!("file://{}", path.display()).replace(' ', "%20")
}

struct Client {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next_id: u64,
}

impl Client {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vn"))
            .arg("lsp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut client = Self {
            child,
            input,
            output,
            next_id: 1,
        };
        let init = client.request("initialize", json!({ "capabilities": {} }));
        assert_eq!(init["serverInfo"]["name"], "vn");
        client.notify("initialized", json!({}));
        client
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        write!(self.input, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.input.flush().unwrap();
    }

    fn receive(&mut self) -> Value {
        let mut length = 0;
        loop {
            let mut line = String::new();
            self.output.read_line(&mut line).unwrap();
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.strip_prefix("Content-Length:") {
                length = value.trim().parse().unwrap();
            }
        }
        let mut body = vec![0; length];
        self.output.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let message = self.receive();
            if message["id"] == id {
                return if message.get("error").is_some() {
                    message
                } else {
                    message["result"].clone()
                };
            }
        }
    }

    fn diagnostics(&mut self, count: usize) -> Vec<(String, Vec<Value>)> {
        (0..count)
            .map(|_| {
                let message = self.receive();
                assert_eq!(message["method"], "textDocument/publishDiagnostics");
                let params = &message["params"];
                (
                    params["uri"].as_str().unwrap().to_string(),
                    params["diagnostics"].as_array().unwrap().clone(),
                )
            })
            .collect()
    }

    fn open(&mut self, path: &Path, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": uri(path), "languageId": "story", "version": 1, "text": text } }),
        );
    }

    fn change(&mut self, path: &Path, text: &str) {
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri(path), "version": 2 }, "contentChanges": [{ "text": text }] }),
        );
    }

    fn at(&mut self, method: &str, path: &Path, line: u32, character: u32) -> Value {
        self.request(
            method,
            json!({ "textDocument": { "uri": uri(path) }, "position": { "line": line, "character": character } }),
        )
    }

    fn completion_labels(&mut self, path: &Path, line: u32, character: u32) -> Vec<String> {
        let result = self.at("textDocument/completion", path, line, character);
        let mut labels: Vec<String> = result
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["label"].as_str().unwrap().to_string())
            .collect();
        labels.sort();
        labels
    }

    fn stop(mut self) -> bool {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.child.wait().unwrap().success()
    }
}

#[test]
fn diagnostics_follow_the_unsaved_text() {
    let project = Project::new();
    let start = project.story("01.story", "scene start:\n  jump two\n");
    let two = project.story("02.story", "scene two:\n  mary \"hi\"\n");

    let mut client = Client::start();
    client.open(&start, "scene start:\n  set trsut = 1\n  jump two\n");
    let published = client.diagnostics(2);
    let (uri_start, errors) = published
        .iter()
        .find(|(u, _)| *u == uri(&start))
        .cloned()
        .unwrap();
    assert_eq!(uri_start, uri(&start));
    assert_eq!(errors.len(), 1, "{:?}", errors);
    assert_eq!(errors[0]["range"]["start"]["line"], 1);
    assert_eq!(errors[0]["range"]["start"]["character"], 2);
    assert_eq!(errors[0]["severity"], 1);
    let message = errors[0]["message"].as_str().unwrap();
    assert!(
        message.contains("trsut") && message.contains("trust"),
        "{}",
        message
    );
    assert!(
        published
            .iter()
            .any(|(u, d)| *u == uri(&two) && d.is_empty())
    );

    client.change(&start, "scene start:\n  set trust = 1\n  jump two\n");
    let published = client.diagnostics(2);
    assert!(
        published.iter().all(|(_, d)| d.is_empty()),
        "{:?}",
        published
    );
    assert!(client.stop());
}

#[test]
fn an_unsaved_file_can_define_a_scene_other_files_jump_to() {
    let project = Project::new();
    let start = project.story("01.story", "scene start:\n  jump later\n");
    let later = project.0.join("story/02.story");

    let mut client = Client::start();
    client.open(&start, "scene start:\n  jump later\n");
    let published = client.diagnostics(1);
    assert_eq!(published[0].1.len(), 1);

    client.open(&later, "scene later:\n  \"done\"\n");
    let published = client.diagnostics(2);
    assert!(
        published.iter().all(|(_, d)| d.is_empty()),
        "{:?}",
        published
    );
    assert!(client.stop());
}

#[test]
fn completion_knows_the_registries_and_the_scenes() {
    let project = Project::new();
    let text = "scene start:\n  jump \n  show \n  show mary \n  set \n  if trust \n  background \n  \"Hello {\n  set route = \n  call \n";
    let start = project.story("01.story", text);
    project.story("02.story", "scene second:\n  \"x\"\n");

    let mut client = Client::start();
    client.open(&start, text);
    client.diagnostics(2);

    assert_eq!(client.completion_labels(&start, 1, 7), ["second", "start"]);
    assert_eq!(client.completion_labels(&start, 2, 7), ["mary"]);
    assert_eq!(
        client.completion_labels(&start, 3, 12),
        ["neutral", "tired"]
    );
    assert_eq!(
        client.completion_labels(&start, 4, 6),
        ["met", "route", "trust"]
    );
    assert_eq!(
        client.completion_labels(&start, 5, 11),
        ["!=", "<", "<=", "==", ">", ">="]
    );
    assert_eq!(
        client.completion_labels(&start, 6, 13),
        ["hall", "none", "study"]
    );
    assert_eq!(
        client.completion_labels(&start, 7, 10),
        ["met", "route", "trust"]
    );
    assert_eq!(client.completion_labels(&start, 8, 15), ["bad", "good"]);
    assert_eq!(
        client.completion_labels(&start, 9, 7),
        ["close_case", "give_item"]
    );

    let first = client.completion_labels(&start, 1, 2);
    assert!(first.contains(&"jump".to_string()) && first.contains(&"mary".to_string()));
    assert!(client.stop());
}

#[test]
fn completion_knows_what_a_choice_option_takes() {
    let project = Project::new();
    let text = "scene start nvl:\n  choice:\n    \"Open\" \n    \"Open\" when \n    \"Open\" when trust \n    \"Open\" image \n    \"Open\" preview \n";
    let start = project.story("01.story", text);

    let mut client = Client::start();
    client.open(&start, text);
    client.diagnostics(1);

    assert_eq!(
        client.completion_labels(&start, 2, 11),
        ["image", "preview", "unless", "when"]
    );
    assert_eq!(
        client.completion_labels(&start, 3, 16),
        ["met", "route", "trust"]
    );
    assert_eq!(
        client.completion_labels(&start, 4, 22),
        ["!=", "<", "<=", "==", ">", ">="]
    );
    assert_eq!(client.completion_labels(&start, 5, 17), ["door"]);
    assert_eq!(client.completion_labels(&start, 6, 19), ["hall_view"]);
    assert!(client.stop());
}

#[test]
fn completion_offers_the_words_a_command_takes() {
    let project = Project::new();
    let text = "scene start:\n  call close_case \n  call give_item \n";
    let start = project.story("01.story", text);

    let mut client = Client::start();
    client.open(&start, text);
    client.diagnostics(1);

    assert_eq!(
        client.completion_labels(&start, 1, 18),
        ["keeper", "report", "silence"],
        "an enum argument offers its words"
    );
    assert!(
        client.completion_labels(&start, 2, 17).is_empty(),
        "a plain word has nothing to offer"
    );
    assert!(client.stop());
}

#[test]
fn completion_offers_the_scene_modes() {
    let project = Project::new();
    let text = "scene start \n  \"one\"\n";
    let start = project.story("01.story", text);

    let mut client = Client::start();
    client.open(&start, text);
    client.diagnostics(1);

    assert_eq!(client.completion_labels(&start, 0, 12), ["adv:", "nvl:"]);
    assert!(client.stop());
}

#[test]
fn definition_hover_and_symbols() {
    let project = Project::new();
    let start = project.story("01.story", "scene start:\n  mary \"hi\"\n  jump two\n");
    let two = project.story("02.story", "# second file\nscene two:\n  set trust = 1\n");

    let mut client = Client::start();
    client.open(&start, &fs::read_to_string(&start).unwrap());
    client.diagnostics(2);

    let definition = client.at("textDocument/definition", &start, 2, 8);
    assert_eq!(definition["uri"], uri(&two));
    assert_eq!(definition["range"]["start"]["line"], 1);

    let hover = client.at("textDocument/hover", &start, 1, 3);
    let text = hover["contents"]["value"].as_str().unwrap();
    assert!(
        text.contains("Mary") && text.contains("neutral, tired"),
        "{}",
        text
    );
    assert_eq!(hover["range"]["start"]["character"], 2);
    assert_eq!(hover["range"]["end"]["character"], 6);

    let hover = client.at("textDocument/hover", &two, 2, 7);
    assert!(hover["contents"]["value"].as_str().unwrap().contains("int"));
    assert!(client.at("textDocument/hover", &start, 0, 1)["contents"].is_null());

    let symbols = client.request(
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": uri(&two) } }),
    );
    assert_eq!(symbols[0]["name"], "two");
    assert_eq!(symbols[0]["location"]["range"]["start"]["line"], 1);
    assert!(client.stop());
}

#[test]
fn files_outside_a_project_are_checked_alone() {
    let dir = std::env::temp_dir().join(format!("vn_cli_lsp_alone_{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let file = dir.canonicalize().unwrap().join("note.story");
    fs::write(&file, "scene a:\n  jump nowhere\n").unwrap();

    let mut client = Client::start();
    client.open(&file, "scene a:\n  jump nowhere\n");
    let published = client.diagnostics(1);
    assert_eq!(published[0].1.len(), 1);
    assert!(
        client
            .completion_labels(&file, 1, 7)
            .contains(&"a".to_string())
    );
    assert!(client.stop());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unknown_requests_get_an_error() {
    let mut client = Client::start();
    let reply = client.request("textDocument/rename", json!({}));
    assert_eq!(reply["error"]["code"], -32601);
    assert!(client.stop());
}
