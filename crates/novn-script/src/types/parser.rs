use super::instructions::{Condition, OptionGate, Position, SceneMode, Transition, Value};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Scene,
    Show,
    Background,
    Music,
    Sound,
    Voice,
    Remove,
    Clear,
    Commit,
    Dialogue,
    Narration,
    ChoiceBlock,
    ChoiceOption,
    Jump,
    If,
    Else,
    Call,
    Set,
    Add,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub indent: usize,
    pub payload: String,
    pub line: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OptionNode {
    pub text: String,
    pub line: usize,
    pub body: Vec<Stmt>,
    pub gate: Option<OptionGate>,
    pub image: Option<String>,
    pub preview: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Stmt {
    pub line: usize,
    pub node: Node,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Scene {
        id: String,
        mode: SceneMode,
        body: Vec<Stmt>,
    },
    Show {
        character: String,
        image: String,
        position: Option<Position>,
        transition: Option<Transition>,
    },
    Background {
        image: Option<String>,
        transition: Option<Transition>,
    },
    Music {
        track: Option<String>,
    },
    Sound {
        id: String,
    },
    Voice {
        id: String,
    },
    Remove {
        character: String,
        transition: Option<Transition>,
    },
    Clear {
        transition: Option<Transition>,
    },
    Commit,
    Dialogue {
        speaker: Option<String>,
        text: String,
    },
    ChoiceBlock {
        options: Vec<OptionNode>,
        final_choice: bool,
    },
    If {
        condition: Condition,
        then_branch: Vec<Stmt>,
        else_branch: Vec<Stmt>,
    },
    Jump {
        target: String,
    },
    Call {
        command: String,
        args: Vec<String>,
    },
    Set {
        var_id: String,
        value: Value,
    },
    Add {
        var_id: String,
        amount: i32,
    },
}
