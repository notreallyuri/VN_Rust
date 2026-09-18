mod check;
mod dump;

use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vn_script::{read_sources, story_files};

const USAGE: &str = "usage:
  vn check <path> [--schema <schema.json>]
  vn dump <file.story | directory>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.as_slice() {
        [cmd, path] if cmd == "dump" => dump::dump(path),
        [cmd, path] if cmd == "check" => check::check(path, None),
        [cmd, path, flag, schema] if cmd == "check" && flag == "--schema" => {
            check::check(path, Some(schema))
        }
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
