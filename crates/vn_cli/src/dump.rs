use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;

use vn_script::{Instruction, Schema, compile_sources};

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

    let scene_starts: HashMap<usize, &str> = program
        .scenes
        .iter()
        .map(|(id, &start)| (start, id.as_str()))
        .collect();

    for (i, instr) in program.instructions.iter().enumerate() {
        if let Some(scene) = scene_starts.get(&i) {
            let location = program.scene_locations[*scene];
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
            Instruction::Show {
                char_id,
                img_id,
                position,
            } => match position {
                Some(position) => format!("SHOW {} {} AT {}", char_id, img_id, position),
                None => format!("SHOW {} {}", char_id, img_id),
            },
            Instruction::Background { image } => {
                format!("BACKGROUND {}", image.as_deref().unwrap_or("none"))
            }
            Instruction::Music { track } => {
                format!("MUSIC {}", track.as_deref().unwrap_or("none"))
            }
            Instruction::Sound { id } => format!("SOUND {}", id),
            Instruction::Jump { scene_id } => format!("JUMP_SCENE '{}'", scene_id),
            Instruction::End => "END".to_string(),
            Instruction::Commit => "COMMIT".to_string(),
            Instruction::With(transition) => format!("WITH {}", transition),
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
