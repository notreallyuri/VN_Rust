mod analysis;
mod project;
mod rpc;
mod uri;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{Value, json};
use vn_script::Severity;

pub use analysis::{Context, ItemKind, complete, hover, scene_lines, word_at};
pub use project::Project;

const METHOD_NOT_FOUND: i64 = -32601;

pub fn lsp() -> ExitCode {
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let mut output = io::stdout().lock();
    let mut server = Server::default();

    loop {
        let message = match rpc::read_message(&mut input) {
            Ok(Some(message)) => message,
            Ok(None) => return ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("vn lsp: {}", e);
                return ExitCode::FAILURE;
            }
        };
        for reply in server.handle(&message) {
            if rpc::write_message(&mut output, &reply).is_err() {
                return ExitCode::FAILURE;
            }
        }
        if server.exit {
            return if server.shutdown {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    }
}

#[derive(Default)]
pub struct Server {
    open: HashMap<PathBuf, String>,
    published: HashMap<PathBuf, HashSet<String>>,
    shutdown: bool,
    exit: bool,
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

fn char_index(line: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (index, c) in line.chars().enumerate() {
        if units >= utf16 {
            return index;
        }
        units += c.len_utf16();
    }
    line.chars().count()
}

fn utf16_column(line: &str, chars: usize) -> usize {
    line.chars().take(chars).map(char::len_utf16).sum()
}

fn range(line: usize, start: usize, end: usize) -> Value {
    json!({
        "start": { "line": line, "character": start },
        "end": { "line": line, "character": end },
    })
}

fn completion_kind(kind: ItemKind) -> u32 {
    match kind {
        ItemKind::Keyword => 14,
        ItemKind::Scene => 9,
        ItemKind::Character => 7,
        ItemKind::Image => 20,
        ItemKind::Variable => 6,
        ItemKind::Value => 12,
        ItemKind::Command => 3,
        ItemKind::Operator => 24,
        ItemKind::Asset => 17,
    }
}

impl Server {
    pub fn handle(&mut self, message: &Value) -> Vec<Value> {
        let method = message["method"].as_str().unwrap_or_default();
        let params = &message["params"];
        let id = message.get("id").cloned();

        let result = match method {
            "initialize" => Some(capabilities()),
            "shutdown" => {
                self.shutdown = true;
                Some(Value::Null)
            }
            "exit" => {
                self.exit = true;
                None
            }
            "textDocument/didOpen" => {
                let doc = &params["textDocument"];
                if let (Some(path), Some(text)) = (self.path(&doc["uri"]), doc["text"].as_str()) {
                    self.open.insert(path.clone(), text.to_string());
                    return self.publish(&path);
                }
                None
            }
            "textDocument/didChange" => {
                let path = self.path(&params["textDocument"]["uri"]);
                let text = params["contentChanges"]
                    .as_array()
                    .and_then(|changes| changes.last())
                    .and_then(|change| change["text"].as_str());
                if let (Some(path), Some(text)) = (path, text) {
                    self.open.insert(path.clone(), text.to_string());
                    return self.publish(&path);
                }
                None
            }
            "textDocument/didSave" => {
                if let Some(path) = self.path(&params["textDocument"]["uri"]) {
                    return self.publish(&path);
                }
                None
            }
            "textDocument/didClose" => {
                if let Some(path) = self.path(&params["textDocument"]["uri"]) {
                    self.open.remove(&path);
                    return self.publish(&path);
                }
                None
            }
            "textDocument/completion" => Some(self.completion(params)),
            "textDocument/definition" => Some(self.definition(params)),
            "textDocument/hover" => Some(self.hover(params)),
            "textDocument/documentSymbol" => Some(self.symbols(params)),
            _ => {
                return match id {
                    Some(id) => vec![json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": METHOD_NOT_FOUND, "message": format!("unsupported method {}", method) },
                    })],
                    None => Vec::new(),
                };
            }
        };

        match (id, result) {
            (Some(id), Some(result)) => {
                vec![json!({ "jsonrpc": "2.0", "id": id, "result": result })]
            }
            (Some(id), None) => vec![json!({ "jsonrpc": "2.0", "id": id, "result": null })],
            _ => Vec::new(),
        }
    }

    fn path(&self, uri: &Value) -> Option<PathBuf> {
        uri.as_str()
            .and_then(uri::to_path)
            .map(|path| canonical(&path))
    }

    fn text(&self, path: &Path) -> String {
        self.open
            .get(path)
            .cloned()
            .or_else(|| fs::read_to_string(path).ok())
            .unwrap_or_default()
    }

    fn line(&self, path: &Path, line: usize) -> String {
        self.text(path)
            .lines()
            .nth(line)
            .unwrap_or_default()
            .to_string()
    }

    fn publish(&mut self, path: &Path) -> Vec<Value> {
        let project = Project::for_file(path);
        let analysis = project.analyze(&self.open);
        let key = project
            .single
            .clone()
            .unwrap_or_else(|| project.stories.clone());

        let mut by_file: HashMap<String, Vec<Value>> = analysis
            .files
            .iter()
            .map(|file| (uri::from_path(file), Vec::new()))
            .collect();

        for diagnostic in &analysis.diagnostics {
            let file = diagnostic
                .file
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| path.to_path_buf());
            let line = diagnostic.line.saturating_sub(1);
            let text = self.line(&canonical(&file), line);
            let indent = text.chars().take_while(|c| c.is_whitespace()).count();
            let severity = match diagnostic.severity {
                Severity::Error => 1,
                Severity::Warning => 2,
            };
            by_file.entry(uri::from_path(&file)).or_default().push(json!({
                "range": range(line, utf16_column(&text, indent), utf16_len(&text).max(indent + 1)),
                "severity": severity,
                "source": "vn",
                "message": diagnostic.message,
            }));
        }

        let previous = self.published.remove(&key).unwrap_or_default();
        for stale in previous {
            by_file.entry(stale).or_default();
        }
        self.published.insert(
            key,
            by_file
                .iter()
                .filter(|(_, list)| !list.is_empty())
                .map(|(uri, _)| uri.clone())
                .collect(),
        );

        let mut uris: Vec<String> = by_file.keys().cloned().collect();
        uris.sort();
        uris.into_iter()
            .map(|uri| {
                let diagnostics = by_file.remove(&uri).unwrap_or_default();
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": uri, "diagnostics": diagnostics },
                })
            })
            .collect()
    }

    fn position(&self, params: &Value) -> Option<(PathBuf, usize, String, usize)> {
        let path = self.path(&params["textDocument"]["uri"])?;
        let line = params["position"]["line"].as_u64()? as usize;
        let character = params["position"]["character"].as_u64()? as usize;
        let text = self.line(&path, line);
        let column = char_index(&text, character);
        Some((path, line, text, column))
    }

    fn completion(&self, params: &Value) -> Value {
        let Some((path, _, text, column)) = self.position(params) else {
            return json!([]);
        };
        let project = Project::for_file(&path);
        let analysis = project.analyze(&self.open);
        let mut scenes = analysis.program.scene_order.clone();
        scenes.sort();
        let audio = ["ogg", "mp3", "wav", "flac"];
        let ctx = Context {
            schema: project.schema.as_ref().map(|file| &file.schema),
            scenes,
            backgrounds: project.asset_ids("backgrounds", &["png"]),
            music: project.asset_ids("music", &audio),
            sounds: project.asset_ids("sounds", &audio),
            voices: project.asset_ids("voice", &audio),
        };
        let prefix: String = text.chars().take(column).collect();
        let items: Vec<Value> = complete(&prefix, &ctx)
            .into_iter()
            .map(|item| {
                let mut value = json!({
                    "label": item.label,
                    "kind": completion_kind(item.kind),
                });
                if let Some(detail) = item.detail {
                    value["detail"] = json!(detail);
                }
                value
            })
            .collect();
        json!(items)
    }

    fn scene_location(&self, path: &Path, word: &str) -> Option<(PathBuf, usize)> {
        let project = Project::for_file(path);
        let analysis = project.analyze(&self.open);
        let program = &analysis.program;
        let location = program.scene_locations.get(word)?;
        let file = program.file_name(location.file)?;
        Some((PathBuf::from(file), location.line.saturating_sub(1)))
    }

    fn definition(&self, params: &Value) -> Value {
        let Some((path, _, text, column)) = self.position(params) else {
            return Value::Null;
        };
        let Some((_, _, word)) = word_at(&text, column) else {
            return Value::Null;
        };
        match self.scene_location(&path, word) {
            Some((file, line)) => {
                let width = utf16_len(&self.line(&canonical(&file), line));
                json!({ "uri": uri::from_path(&file), "range": range(line, 0, width) })
            }
            None => Value::Null,
        }
    }

    fn hover(&self, params: &Value) -> Value {
        let Some((path, line, text, column)) = self.position(params) else {
            return Value::Null;
        };
        let Some((start, end, word)) = word_at(&text, column) else {
            return Value::Null;
        };
        let project = Project::for_file(&path);
        let ctx = Context {
            schema: project.schema.as_ref().map(|file| &file.schema),
            ..Context::default()
        };
        let scene = self.scene_location(&path, word);
        let scene = scene.as_ref().map(|(file, line)| {
            let name = file.file_name().map_or_else(
                || file.display().to_string(),
                |n| n.to_string_lossy().to_string(),
            );
            (name, line + 1)
        });
        match hover(word, &ctx, scene.as_ref().map(|(f, l)| (f.as_str(), *l))) {
            Some(markdown) => json!({
                "contents": { "kind": "markdown", "value": markdown },
                "range": range(line, utf16_column(&text, start), utf16_column(&text, end)),
            }),
            None => Value::Null,
        }
    }

    fn symbols(&self, params: &Value) -> Value {
        let Some(path) = self.path(&params["textDocument"]["uri"]) else {
            return json!([]);
        };
        let uri = uri::from_path(&path);
        let text = self.text(&path);
        let symbols: Vec<Value> = scene_lines(&text)
            .into_iter()
            .map(|(name, line)| {
                let width = utf16_len(text.lines().nth(line).unwrap_or_default());
                json!({
                    "name": name,
                    "kind": 2,
                    "location": { "uri": uri, "range": range(line, 0, width) },
                })
            })
            .collect();
        json!(symbols)
    }
}

fn capabilities() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": {
                "openClose": true,
                "change": 1,
                "save": { "includeText": false },
            },
            "completionProvider": { "triggerCharacters": [" ", "{"] },
            "definitionProvider": true,
            "hoverProvider": true,
            "documentSymbolProvider": true,
        },
        "serverInfo": { "name": "vn", "version": env!("CARGO_PKG_VERSION") },
    })
}
