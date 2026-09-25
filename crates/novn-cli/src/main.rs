mod check;
mod dump;
mod fmt;
mod lang;
mod lsp;
mod new;
mod po;
mod translate;

use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use novn_script::{read_sources, story_files};

const USAGE: &str = "usage:
  novn check <path> [--schema <schema.json>] [--lang <code>]
  novn dump <file.story | directory>
  novn fmt <path> [--check]
  novn lsp
  novn translate <lang> [path] [--export | --import <file.po>]
  novn new <directory> [--title <title>] [--engine-path <dir> | --engine-git <url>]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.as_slice() {
        [cmd, path] if cmd == "dump" => dump::dump(path),
        [cmd] if cmd == "lsp" => lsp::lsp(),
        [cmd, path] if cmd == "fmt" => fmt::fmt(path, false),
        [cmd, path, flag] if cmd == "fmt" && flag == "--check" => fmt::fmt(path, true),
        [cmd, rest @ ..] if cmd == "translate" => match parse_translate(rest) {
            Ok((language, path, exchange)) => translate::translate(language, path, exchange),
            Err(e) => {
                eprintln!("❌ {e}\n\n{USAGE}");
                ExitCode::FAILURE
            }
        },
        [cmd, rest @ ..] if cmd == "check" => match parse_check(rest) {
            Ok((path, schema, lang)) => check::check(path, schema, lang),
            Err(e) => {
                eprintln!("❌ {e}\n\n{USAGE}");
                ExitCode::FAILURE
            }
        },
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

type CheckArgs<'a> = (&'a str, Option<&'a str>, Option<&'a str>);

fn parse_check(args: &[String]) -> Result<CheckArgs<'_>, String> {
    let (mut path, mut schema, mut lang) = (None, None, None);
    let mut rest = args.iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--schema" => schema = Some(value(&mut rest, "--schema")?),
            "--lang" => lang = Some(value(&mut rest, "--lang")?),
            _ if arg.starts_with("--") => return Err(format!("unknown option {arg}")),
            _ if path.is_none() => path = Some(arg.as_str()),
            _ => return Err(format!("unexpected argument {arg}")),
        }
    }

    Ok((path.ok_or("check needs a path")?, schema, lang))
}

fn value<'a>(args: &mut std::slice::Iter<'a, String>, flag: &str) -> Result<&'a str, String> {
    args.next()
        .map(String::as_str)
        .ok_or_else(|| format!("{flag} needs a value"))
}

type TranslateArgs<'a> = (&'a str, &'a str, translate::Exchange);

fn parse_translate(args: &[String]) -> Result<TranslateArgs<'_>, String> {
    let (mut language, mut path) = (None, None);
    let mut exchange = translate::Exchange::None;
    let mut rest = args.iter();

    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--export" => exchange = translate::Exchange::Export,
            "--import" => {
                exchange = translate::Exchange::Import(value(&mut rest, "--import")?.to_string())
            }
            _ if arg.starts_with("--") => return Err(format!("unknown option {arg}")),
            _ if language.is_none() => language = Some(arg.as_str()),
            _ if path.is_none() => path = Some(arg.as_str()),
            _ => return Err(format!("unexpected argument {arg}")),
        }
    }

    let language = language.ok_or("translate needs a language")?;
    Ok((language, path.unwrap_or("."), exchange))
}

pub fn load_sources(path: &Path) -> io::Result<Vec<(String, String)>> {
    let files = if path.is_dir() {
        story_files(path)?
    } else {
        vec![PathBuf::from(path)]
    };
    read_sources(&files)
}
