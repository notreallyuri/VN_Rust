#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
    Scene,
    Show,
    Remove,
    Clear,
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
pub struct ChoiceOption {
    pub text: String,
    pub body: Vec<Node>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Condition;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Node {
    Scene {
        id: String,
        body: Vec<Node>,
    },
    Show {
        character: String,
        image: String,
    },
    Dialogue {
        speaker: Option<String>,
        text: String,
    },
    ChoiceBlock {
        options: Vec<ChoiceOption>,
    },
    If {
        condition: Condition,
        then_branch: Vec<Node>,
        else_branch: Vec<Node>,
    },
    Jump {
        target: String,
    },
    Call {
        command: String,
        args: Vec<String>,
    },
}
