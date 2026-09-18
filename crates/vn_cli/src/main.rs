use vn_script::{Instruction, Schema, compile_source};

use std::collections::HashMap;
use std::fs::read_to_string;
use std::process::ExitCode;

const USAGE: &str = "usage: vn dump <file.story>";

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
    let script = match read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ Failed to read {}: {}", file_path, e);
            return ExitCode::FAILURE;
        }
    };

    let program = compile_source(&script);

    println!(
        "{} scenes, {} instructions",
        program.scenes.len(),
        program.instructions.len()
    );

    let diagnostics = Schema::default().validate(&program);
    for diagnostic in &diagnostics {
        println!("{}", diagnostic.in_file(file_path));
    }

    let scene_starts: HashMap<usize, &str> = program
        .scenes
        .iter()
        .map(|(id, &start)| (start, id.as_str()))
        .collect();

    for (i, instr) in program.instructions.iter().enumerate() {
        if let Some(scene) = scene_starts.get(&i) {
            println!("\nscene {}:", scene);
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
