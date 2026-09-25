use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use novn_script::{Program, compile_sources, format, story_files};

pub fn fmt(path: &str, check: bool) -> ExitCode {
    let path = Path::new(path);
    let files = match files(path) {
        Ok(files) if files.is_empty() => {
            eprintln!("❌ no .story files in {}", path.display());
            return ExitCode::FAILURE;
        }
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    };

    let mut changed = 0;
    let mut failed = 0;

    for file in &files {
        match reformat(file) {
            Err(e) => {
                eprintln!("❌ {}", e);
                failed += 1;
            }
            Ok(None) => {}
            Ok(Some(formatted)) => {
                changed += 1;
                if check {
                    println!("would reformat {}", file.display());
                } else if let Err(e) = fs::write(file, formatted) {
                    eprintln!("❌ {}: {}", file.display(), e);
                    failed += 1;
                } else {
                    println!("formatted {}", file.display());
                }
            }
        }
    }

    let files = files.len();
    println!(
        "{} file{}, {} {}",
        files,
        plural(files),
        changed,
        if check { "to reformat" } else { "reformatted" }
    );

    if failed > 0 || (check && changed > 0) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn files(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    if path.is_dir() {
        story_files(path)
    } else {
        Ok(vec![path.to_path_buf()])
    }
}

fn reformat(file: &Path) -> Result<Option<String>, String> {
    let name = file.display().to_string();
    let source = fs::read_to_string(file).map_err(|e| format!("{}: {}", name, e))?;

    let before = compile_sources([(name.clone(), source.as_str())]);
    if before.diagnostics.iter().any(|d| d.is_error()) {
        return Err(format!("{}: not formatted, it has errors", name));
    }

    let formatted = format(&source);
    if formatted == source {
        return Ok(None);
    }

    let after = compile_sources([(name.clone(), formatted.as_str())]);
    if after.diagnostics.iter().any(|d| d.is_error()) || story(&after) != story(&before) {
        return Err(format!(
            "{}: not formatted, the result would not be the same story (please report this)",
            name
        ));
    }

    Ok(Some(formatted))
}

fn story(program: &Program) -> String {
    let mut scenes: Vec<_> = program.scenes.iter().collect();
    scenes.sort();
    format!(
        "{:?}{:?}{:?}",
        program.instructions, program.scene_order, scenes
    )
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
