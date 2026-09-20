mod check;
mod dump;
mod fmt;
mod lsp;
mod new;
mod translate;

use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vn_script::{read_sources, story_files};

const USAGE: &str = "usage:
  vn check <path> [--schema <schema.json>]
  vn dump <file.story | directory>
  vn fmt <path> [--check]
  vn lsp
  vn translate <lang> [path]
  vn new <directory> [--title <title>] [--engine-path <dir> | --engine-git <url>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.as_slice() {
        [cmd, path] if cmd == "dump" => dump::dump(path),
        [cmd] if cmd == "lsp" => lsp::lsp(),
        [cmd, path] if cmd == "fmt" => fmt::fmt(path, false),
        [cmd, path, flag] if cmd == "fmt" && flag == "--check" => fmt::fmt(path, true),
        [cmd, path] if cmd == "check" => check::check(path, None),
        [cmd, language] if cmd == "translate" => translate::translate(language, "."),
        [cmd, language, path] if cmd == "translate" => translate::translate(language, path),
        [cmd, path, flag, schema] if cmd == "check" && flag == "--schema" => {
            check::check(path, Some(schema))
        }
        [cmd, rest @ ..] if cmd == "new" => match new::Options::parse(rest) {
            Ok(options) => new::new(options),
            Err(e) => {
                eprintln!("❌ {e}\n\n{USAGE}");
                ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

pub fn load_sources(path: &Path) -> io::Result<Vec<(String, String)>> {
    let files = if path.is_dir() {
        story_files(path)?
    } else {
        vec![PathBuf::from(path)]
    };
    read_sources(&files)
}
