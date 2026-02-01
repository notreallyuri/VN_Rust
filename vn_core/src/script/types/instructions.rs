use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Int(i32),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Comparison {
    Equal,
    NotEqual,
    Gte,
    Lte,
    Gt,
    Lt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    Show {
        char_id: String,
        img_id: String,
    },
    Hide {
        char_id: String,
    },
    Clear,
    Say {
        char_id: Option<String>,
        text: String,
    },
    Jump {
        scene_id: String,
    },
    JumpIfFalse {
        var_id: String,
        op: Comparison,
        value: Value,
        jump_to_index: usize,
    },
    Choice {
        options: Vec<(String, usize)>,
    },
    Set {
        var_id: String,
        value: Value,
    },
    Add {
        var_id: String,
        amount: i32,
    },
    Call {
        command: String,
        args: Vec<String>,
    },
    Pause,
    Goto(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInstruction {
    pub id: String,
    pub instructions: Vec<Instruction>,
}
