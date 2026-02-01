pub enum Value {
    Bool(bool),
    Int(i32),
    String(String),
}

pub enum Comparison {
    Equal,
    NotEqual,
    Gte,
    Lte,
    Gt,
    Lt,
}

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
}

pub struct SceneInstruction {
    pub id: String,
    pub instructions: Vec<Instruction>,
}
