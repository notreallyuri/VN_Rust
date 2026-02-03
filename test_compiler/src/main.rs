use vn_core::script::compiler::Compiler;
use vn_core::script::lexer::tokenize;
use vn_core::script::parser::parse_scene;
use vn_core::script::types::Instruction;

fn main() {
    let script = r#"
scene start:
    show gabriel happy
    gabriel "Hello!"
    show crownley serious
    "hello"
    crownley "Hello"
    choice:
        "Yes":
            if affection > 2:
                gabriel "character deep test"
                jump yes_scene
            else:
                gabriel "character deep test 2"
            "narration deep test"
        "No":
            "narration deep test 2"
            jump no_scene

scene yes_scene:
    "We jumped out!"
"#;

    println!("--- 1. LEXER ---");
    let tokens = tokenize(script);

    for t in tokens.iter().take(5) {
        println!("{:?} (Indent: {})", t.kind, t.indent);
    }

    let (node, _) = parse_scene(&tokens, 0);
    println!("Parsed 'start' scene successfully.");

    println!("\n--- 3. COMPILER (The Moment of Truth) ---");
    let compiler = Compiler::new();
    let instructions = compiler.compile(node);

    for (i, instr) in instructions.iter().enumerate() {
        let label = match instr {
            Instruction::Choice { options } => {
                // Formatting the choice to look nice
                let opts: Vec<String> = options
                    .iter()
                    .map(|(t, idx)| format!("'{}'->{}", t, idx))
                    .collect();
                format!("CHOICE [{}]", opts.join(", "))
            }
            Instruction::JumpIfFalse { jump_to_index, .. } => {
                format!("JUMP_IF_FALSE (goto {})", jump_to_index)
            }
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
            _ => format!("{:?}", instr),
        };

        println!("{:03}: {}", i, label);
    }
}
