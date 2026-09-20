use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vn_script::{Diagnostic, SCHEMA_FILE_NAME, SchemaFile, StoryVm, compile_sources};

struct Target {
    stories: PathBuf,
    only: Option<PathBuf>,
    schema: Option<(PathBuf, SchemaFile)>,
}

pub fn check(path: &str, schema: Option<&str>, lang: Option<&str>) -> ExitCode {
    let target = match resolve(Path::new(path), schema.map(Path::new)) {
        Ok(target) => target,
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    };

    let sources = match crate::load_sources(&target.stories) {
        Ok(sources) if sources.is_empty() => {
            eprintln!("❌ no .story files in {}", target.stories.display());
            return ExitCode::FAILURE;
        }
        Ok(sources) => sources,
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    };

    let mut story = StoryVm::from_program(compile_sources(sources));
    let files = story.program().files.len();
    let scenes = story.program().scenes.len();

    let diagnostics = match &target.schema {
        Some((_, file)) => story.prepare(file.schema.clone(), file.entry_scene.as_deref()),
        None => story.validate(),
    };
    let diagnostics: Vec<Diagnostic> = diagnostics
        .into_iter()
        .filter(|d| target.only.as_ref().is_none_or(|only| is_under(d, only)))
        .collect();

    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }

    let errors = diagnostics.iter().filter(|d| d.is_error()).count();
    let warnings = diagnostics.len() - errors;
    let against = match &target.schema {
        Some((path, _)) => format!("against {}", path.display()),
        None => format!(
            "without a schema (syntax only; run the game with --export-schema to write {})",
            SCHEMA_FILE_NAME
        ),
    };
    let shown = match &target.only {
        Some(only) => format!(" in {}", only.display()),
        None => String::new(),
    };
    println!(
        "{} file{}, {} scene{} checked {}: {} error{}, {} warning{}{}",
        files,
        plural(files),
        scenes,
        plural(scenes),
        against,
        errors,
        plural(errors),
        warnings,
        plural(warnings),
        shown,
    );

    let translations = report_translations(&target, story.program(), lang);

    if errors > 0 || translations.is_err() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn report_translations(
    target: &Target,
    program: &vn_script::Program,
    lang: Option<&str>,
) -> Result<(), ()> {
    let Some((schema_path, file)) = &target.schema else {
        if lang.is_some() {
            eprintln!(
                "❌ no {} found, so there is nothing to check translations against",
                SCHEMA_FILE_NAME
            );
            return Err(());
        }
        return Ok(());
    };
    let root = schema_path.parent().unwrap_or(Path::new("."));

    let codes = match lang {
        Some(code) => vec![code.to_string()],
        None => crate::lang::codes(root),
    };
    if codes.is_empty() {
        if lang.is_some() {
            eprintln!("❌ no catalog to check");
            return Err(());
        }
        return Ok(());
    }

    let mut failed = false;
    for code in &codes {
        let status = match crate::lang::status(root, code, program, Some(&file.schema)) {
            Ok(status) => status,
            Err(e) => {
                eprintln!("❌ {}", e);
                failed = true;
                continue;
            }
        };

        if lang.is_some() {
            for line in &status.missing {
                println!(
                    "{}: missing {} translation: {:?}",
                    line.place(),
                    line.kind,
                    line.source
                );
            }
            for line in &status.stale {
                println!(
                    "{}: stale {} translation: {:?}",
                    line.place(),
                    line.kind,
                    line.source
                );
            }
        }

        println!(
            "{}: {}/{} translated, {} missing, {} stale",
            status.language,
            status.translated,
            status.total,
            status.missing.len(),
            status.stale.len(),
        );
        if lang.is_none() && !status.is_complete() {
            println!("  run `vn check --lang {}` to list them", status.language);
        }
    }

    if failed { Err(()) } else { Ok(()) }
}

fn resolve(path: &Path, schema: Option<&Path>) -> Result<Target, String> {
    if !path.exists() {
        return Err(format!("{}: no such file or directory", path.display()));
    }

    let is_schema = path.is_file() && path.extension().is_some_and(|ext| ext == "json");
    let schema_path = match schema {
        Some(schema) => Some(schema.to_path_buf()),
        None if is_schema => Some(path.to_path_buf()),
        None => find_schema(path),
    };

    let Some(schema_path) = schema_path else {
        return Ok(Target {
            stories: path.to_path_buf(),
            only: None,
            schema: None,
        });
    };

    let mut file = SchemaFile::read(&schema_path).map_err(|e| e.to_string())?;
    let root = schema_path.parent().unwrap_or(Path::new("."));
    let stories = root.join(&file.story_dir);
    let whole_project = is_schema || same_path(path, root) || same_path(path, &stories);

    if !whole_project && !is_inside(path, &stories) {
        file.entry_scene = None;
        return Ok(Target {
            stories: path.to_path_buf(),
            only: None,
            schema: Some((schema_path, file)),
        });
    }

    Ok(Target {
        stories,
        only: (!whole_project).then(|| path.to_path_buf()),
        schema: Some((schema_path, file)),
    })
}

pub(crate) fn find_schema(path: &Path) -> Option<PathBuf> {
    let absolute = fs::canonicalize(path).ok()?;
    let start = if absolute.is_dir() {
        absolute.as_path()
    } else {
        absolute.parent()?
    };

    let found = start
        .ancestors()
        .map(|dir| dir.join(SCHEMA_FILE_NAME))
        .find(|candidate| candidate.is_file())?;

    let relative = std::env::current_dir()
        .ok()
        .and_then(|cwd| found.strip_prefix(cwd).ok().map(Path::to_path_buf));
    Some(relative.unwrap_or(found))
}

fn same_path(a: &Path, b: &Path) -> bool {
    matches!((fs::canonicalize(a), fs::canonicalize(b)), (Ok(a), Ok(b)) if a == b)
}

fn is_inside(path: &Path, dir: &Path) -> bool {
    matches!((fs::canonicalize(path), fs::canonicalize(dir)), (Ok(path), Ok(dir)) if path.starts_with(&dir))
}

fn is_under(diagnostic: &Diagnostic, only: &Path) -> bool {
    diagnostic
        .file
        .as_ref()
        .is_some_and(|file| is_inside(Path::new(file), only))
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
