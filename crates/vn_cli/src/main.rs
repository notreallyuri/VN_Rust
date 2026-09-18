use vn_script::{Instruction, Schema, compile_sources, read_sources, story_files};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: vn dump <file.story | directory>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.as_slice() {
        [cmd, file_path] if cmd == "dump" => dump(file_path),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn dump(file_path: &str) -> ExitCode {
    let path = Path::new(file_path);
    let files = if path.is_dir() {
        story_files(path)
    } else {
        Ok(vec![PathBuf::from(path)])
    };

    let sources = match files.and_then(|files| read_sources(&files)) {
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

    let scene_starts: HashMap<usize, &str> = program
        .scenes
        .iter()
        .map(|(id, &start)| (start, id.as_str()))
        .collect();

    for (i, instr) in program.instructions.iter().enumerate() {
        if let Some(scene) = scene_starts.get(&i) {
            let location = program.location(i);
            let file = program.file_name(location.file).unwrap_or_default();
            println!("\nscene {}:  ({}:{})", scene, file, location.line);
        }

        let label = match instr {
            Instruction::Choice { options } => {
                let opts: Vec<String> = options
                    .iter()
                    .map(|(t, idx)| format!("'{}'->{}", t, idx))
                    .collect();
                format!("CHOICE [{}]", opts.join(", "))
            }
            Instruction::JumpIfFalse {
                condition,
                jump_to_index,
            } => format!("JUMP_IF_FALSE {} (goto {})", condition, jump_to_index),
            Instruction::Goto(idx) => format!("GOTO {}", idx),
            Instruction::Pause => "PAUSE".to_string(),
            Instruction::Say { char_id, text } => {
                if let Some(name) = char_id {
                    format!("SAY [{}]: \"{}\"", name, text)
                } else {
                    format!("SAY [NARRATOR]: \"{}\"", text)
                }
            }
            Instruction::Show { char_id, .. } => format!("SHOW {}", char_id),
            Instruction::Jump { scene_id } => format!("JUMP_SCENE '{}'", scene_id),
            Instruction::End => "END".to_string(),
            Instruction::Commit => "COMMIT".to_string(),
            Instruction::Hide { char_id } => format!("HIDE {}", char_id),
            Instruction::Clear => "CLEAR".to_string(),
            Instruction::Set { var_id, value } => format!("SET {} = {}", var_id, value.literal()),
            Instruction::Add { var_id, amount } => format!("ADD {} {:+}", var_id, amount),
            Instruction::Call { command, args } => format!("CALL {} {}", command, args.join(" ")),
        };

        println!("{:03}: {}", i, label);
    }

    if diagnostics.iter().any(|d| d.is_error()) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
