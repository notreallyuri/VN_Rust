use std::path::Path;
use std::process::ExitCode;

use novn_script::{Schema, compile_sources};

pub fn dump(file_path: &str) -> ExitCode {
    let sources = match crate::load_sources(Path::new(file_path)) {
        Ok(sources) => sources,
        Err(e) => {
            eprintln!("❌ {}", e);
            return ExitCode::FAILURE;
        }
    };

    let program = compile_sources(sources);

    println!(
        "{} files, {} scenes, {} instructions",
        program.files.len(),
        program.scenes.len(),
        program.instructions.len()
    );

    let diagnostics = Schema::default().validate(&program);
    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }

    print!("{}", program.listing());

    if diagnostics.iter().any(|d| d.is_error()) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
