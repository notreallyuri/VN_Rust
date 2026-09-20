use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vn_script::translate::{self, Catalog};
use vn_script::{
    LANG_DIR, SCHEMA_FILE_NAME, SchemaFile, UI_STRINGS_FILE, UiStrings, compile_sources,
};

pub fn translate(language: &str, path: &str) -> ExitCode {
    if !is_language_tag(language) {
        eprintln!(
            "❌ '{}' is not a language tag (letters, digits and dashes, like pt-BR)",
            language
        );
        return ExitCode::FAILURE;
    }

    let path = Path::new(path);
    let Some(schema_path) = crate::check::find_schema(path) else {
        eprintln!(
            "❌ no {} found in {} or above it",
            SCHEMA_FILE_NAME,
            path.display()
        );
        return ExitCode::FAILURE;
    };

    let file = match SchemaFile::read(&schema_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    };

    let root = schema_path.parent().unwrap_or(Path::new("."));
    let stories = root.join(&file.story_dir);
    let sources = match crate::load_sources(&stories) {
        Ok(sources) if sources.is_empty() => {
            eprintln!("❌ no .story files in {}", stories.display());
            return ExitCode::FAILURE;
        }
        Ok(sources) => sources,
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    };

    let program = compile_sources(sources);
    let errors: Vec<&vn_script::Diagnostic> = program
        .diagnostics
        .iter()
        .filter(|d| d.is_error())
        .collect();
    if !errors.is_empty() {
        for diagnostic in &errors {
            println!("{}", diagnostic);
        }
        eprintln!(
            "❌ {} error{} in the story; nothing extracted",
            errors.len(),
            plural(errors.len())
        );
        return ExitCode::FAILURE;
    }

    let strings = translate::extract(&program);
    let names = translate::extract_names(&file.schema);

    let catalog_path = catalog_path(root, language);
    let mut catalog = if catalog_path.exists() {
        match Catalog::read(&catalog_path) {
            Ok(catalog) => catalog,
            Err(e) => {
                eprintln!("❌ {}", e);
                return ExitCode::FAILURE;
            }
        }
    } else {
        Catalog::new(language)
    };
    catalog.language = language.to_string();

    let ui = root.join(LANG_DIR).join(UI_STRINGS_FILE);
    let ui = match ui.exists() {
        true => match UiStrings::read(&ui) {
            Ok(strings) => strings.strings,
            Err(e) => {
                eprintln!("⚠️ {}; the screens' own labels are not in the catalog", e);
                Vec::new()
            }
        },
        false => Vec::new(),
    };
    let ui_added = catalog.refresh_ui(&ui);

    let refresh = catalog.refresh(&strings, &names);
    match catalog.write(&catalog_path) {
        Ok(written) => {
            let status = if written { "wrote" } else { "unchanged:" };
            println!("{} {}", status, catalog_path.display());
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    }

    if ui.is_empty() {
        println!(
            "no {}/{} yet: run the game once in a debug build to list the screens' own labels",
            LANG_DIR, UI_STRINGS_FILE
        );
    }
    println!(
        "{} string{} ({} new), {} translated, {} missing, {} stale",
        refresh.total,
        plural(refresh.total),
        refresh.added + ui_added,
        refresh.translated,
        refresh.missing(),
        refresh.stale,
    );
    ExitCode::SUCCESS
}

pub fn catalog_path(root: &Path, language: &str) -> PathBuf {
    root.join(LANG_DIR).join(format!("{}.json", language))
}

fn is_language_tag(language: &str) -> bool {
    !language.is_empty()
        && language
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        && language.starts_with(|c: char| c.is_ascii_alphabetic())
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
