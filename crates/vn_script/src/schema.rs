use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::suggest::did_you_mean;
use crate::template::referenced_variables;
use crate::{Condition, Diagnostic, Instruction, Program, Value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarType {
    Bool,
    Int,
    String,
    Enum(Vec<String>),
}

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarType::Bool => write!(f, "bool"),
            VarType::Int => write!(f, "int"),
            VarType::String => write!(f, "string"),
            VarType::Enum(members) => write!(f, "enum ({})", members.join(" | ")),
        }
    }
}

impl VarType {
    pub fn check(&self, value: &Value) -> Result<(), String> {
        match (self, value) {
            (VarType::Bool, Value::Bool(_))
            | (VarType::Int, Value::Int(_))
            | (VarType::String, Value::String(_)) => Ok(()),
            (VarType::Enum(members), Value::Enum(member)) => {
                if members.contains(member) {
                    Ok(())
                } else {
                    Err(format!(
                        "'{}' is not one of {}",
                        member,
                        members.join(" | ")
                    ))
                }
            }
            (VarType::String, Value::Enum(word)) => Err(format!(
                "expected a string, got the bare word `{}` (write \"{}\")",
                word, word
            )),
            (expected, found) => Err(format!("expected {}, got `{}`", expected, found.literal())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableDef {
    pub ty: VarType,
    pub default: Value,
}

impl VariableDef {
    pub fn bool(default: bool) -> Self {
        Self {
            ty: VarType::Bool,
            default: Value::Bool(default),
        }
    }

    pub fn int(default: i32) -> Self {
        Self {
            ty: VarType::Int,
            default: Value::Int(default),
        }
    }

    pub fn string(default: impl Into<String>) -> Self {
        Self {
            ty: VarType::String,
            default: Value::String(default.into()),
        }
    }

    pub fn enumeration<I, S>(members: I, default: &str) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let members: Vec<String> = members.into_iter().map(Into::into).collect();
        assert!(
            members.iter().any(|m| m == default),
            "enum default '{}' is not one of {}",
            default,
            members.join(" | ")
        );

        Self {
            ty: VarType::Enum(members),
            default: Value::Enum(default.to_string()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterDef {
    pub name: String,
    pub images: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamKind {
    Int,
    UInt,
    Float,
    Bool,
    Word,
}

impl fmt::Display for ParamKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ParamKind::Int => "an integer",
            ParamKind::UInt => "a non-negative integer",
            ParamKind::Float => "a number",
            ParamKind::Bool => "true or false",
            ParamKind::Word => "a word",
        };
        write!(f, "{}", name)
    }
}

impl ParamKind {
    pub fn accepts(self, arg: &str) -> bool {
        match self {
            ParamKind::Int => arg.parse::<i64>().is_ok(),
            ParamKind::UInt => arg.parse::<u64>().is_ok(),
            ParamKind::Float => arg.parse::<f64>().is_ok(),
            ParamKind::Bool => matches!(arg, "true" | "false"),
            ParamKind::Word => !arg.is_empty(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandSig {
    pub required: Vec<ParamKind>,
    pub optional: Vec<ParamKind>,
    pub rest: Option<ParamKind>,
}

impl CommandSig {
    pub fn usage(&self, name: &str) -> String {
        let mut parts = vec![format!("call {}", name)];
        parts.extend(self.required.iter().map(|k| format!("<{}>", short(*k))));
        parts.extend(self.optional.iter().map(|k| format!("[{}]", short(*k))));
        if let Some(rest) = self.rest {
            parts.push(format!("[{}...]", short(rest)));
        }
        parts.join(" ")
    }

    pub fn check(&self, name: &str, args: &[String]) -> Result<(), String> {
        let max = self.required.len() + self.optional.len();

        if args.len() < self.required.len() || (self.rest.is_none() && args.len() > max) {
            return Err(format!(
                "`call {}` takes {}, got {} (usage: {})",
                name,
                count(self.required.len(), max, self.rest.is_some()),
                args.len(),
                self.usage(name)
            ));
        }

        let kinds = self
            .required
            .iter()
            .chain(&self.optional)
            .copied()
            .chain(std::iter::repeat_n(self.rest, args.len()).flatten());

        for (i, (arg, kind)) in args.iter().zip(kinds).enumerate() {
            if !kind.accepts(arg) {
                return Err(format!(
                    "`call {}` argument {} should be {}, got `{}`",
                    name,
                    i + 1,
                    kind,
                    arg
                ));
            }
        }

        Ok(())
    }
}

fn short(kind: ParamKind) -> &'static str {
    match kind {
        ParamKind::Int => "int",
        ParamKind::UInt => "uint",
        ParamKind::Float => "number",
        ParamKind::Bool => "bool",
        ParamKind::Word => "word",
    }
}

fn count(min: usize, max: usize, rest: bool) -> String {
    let plural = |n: usize| if n == 1 { "argument" } else { "arguments" };
    if rest {
        format!("at least {} {}", min, plural(min))
    } else if min == max {
        format!("{} {}", min, plural(min))
    } else {
        format!("{} to {} arguments", min, max)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Schema {
    pub variables: BTreeMap<String, VariableDef>,
    pub characters: BTreeMap<String, CharacterDef>,
    pub commands: BTreeMap<String, CommandSig>,
}

impl Schema {
    pub fn defaults(&self) -> BTreeMap<String, Value> {
        self.variables
            .iter()
            .map(|(name, def)| (name.clone(), def.default.clone()))
            .collect()
    }

    pub fn validate(&self, program: &Program) -> Vec<Diagnostic> {
        let mut checker = Checker {
            schema: self,
            program,
            diagnostics: program.diagnostics.clone(),
            file: None,
        };

        for (index, instruction) in program.instructions.iter().enumerate() {
            checker.file = program.file(index);
            checker.instruction(program.line(index), instruction);
        }

        let mut diagnostics = checker.diagnostics;
        program.sort_diagnostics(&mut diagnostics);
        diagnostics
    }
}

struct Checker<'a> {
    schema: &'a Schema,
    program: &'a Program,
    diagnostics: Vec<Diagnostic>,
    file: Option<&'a str>,
}

impl<'a> Checker<'a> {
    fn error(&mut self, line: usize, message: String) {
        self.diagnostics
            .push(Diagnostic::error(line, message).with_file(self.file));
    }

    fn warn(&mut self, line: usize, message: String) {
        self.diagnostics
            .push(Diagnostic::warning(line, message).with_file(self.file));
    }

    fn instruction(&mut self, line: usize, instruction: &Instruction) {
        match instruction {
            Instruction::Jump { scene_id } => {
                if !self.program.scenes.contains_key(scene_id) {
                    self.error(
                        line,
                        format!(
                            "`jump {}`: no scene with that name{}",
                            scene_id,
                            did_you_mean(
                                scene_id,
                                self.program.scene_order.iter().map(String::as_str)
                            )
                        ),
                    );
                }
            }
            Instruction::Set { var_id, value } => {
                if let Some(def) = self.variable(line, var_id)
                    && let Err(e) = def.ty.check(value)
                {
                    self.error(line, format!("`set {}`: {}", var_id, e));
                }
            }
            Instruction::Add { var_id, .. } => {
                if let Some(def) = self.variable(line, var_id)
                    && def.ty != VarType::Int
                {
                    self.error(
                        line,
                        format!("`add {}`: variable is {}, not int", var_id, def.ty),
                    );
                }
            }
            Instruction::JumpIfFalse { condition, .. } => self.condition(line, condition),
            Instruction::Say { char_id, text } => {
                self.interpolations(line, text);
                if let Some(speaker) = char_id {
                    self.interpolations(line, speaker);
                    if !speaker.contains('{') {
                        self.character(line, speaker, "speaker");
                    }
                }
            }
            Instruction::Choice { options, .. } => {
                for arm in options {
                    self.interpolations(line, &arm.text);
                    if let Some(gate) = &arm.gate {
                        self.condition(line, &gate.condition);
                        if let Some(reason) = &gate.reason {
                            self.interpolations(line, reason);
                        }
                    }
                }
                if !options.is_empty()
                    && options
                        .iter()
                        .all(|arm| arm.gate.as_ref().is_some_and(|gate| gate.hides()))
                {
                    self.warn(
                        line,
                        "every option here can be hidden, which would leave the player with an empty choice; give at least one option a reason or no condition".to_string(),
                    );
                }
            }
            Instruction::Show {
                char_id, img_id, ..
            } => {
                if let Some(def) = self.character(line, char_id, "`show`")
                    && !def.images.is_empty()
                    && !def.images.contains(img_id)
                {
                    self.error(
                        line,
                        format!(
                            "`show {} {}`: '{}' has no image '{}' (images: {}){}",
                            char_id,
                            img_id,
                            char_id,
                            img_id,
                            def.images.join(", "),
                            did_you_mean(img_id, def.images.iter().map(String::as_str))
                        ),
                    );
                }
            }
            Instruction::Hide { char_id } => {
                self.character(line, char_id, "`remove`");
            }
            Instruction::Call { command, args } => {
                if self.schema.commands.is_empty() {
                    return;
                }
                match self.schema.commands.get(command) {
                    Some(sig) => {
                        if let Err(e) = sig.check(command, args) {
                            self.error(line, e);
                        }
                    }
                    None => self.error(
                        line,
                        format!(
                            "unknown command '{}'{}",
                            command,
                            did_you_mean(command, self.schema.commands.keys().map(String::as_str))
                        ),
                    ),
                }
            }
            Instruction::Clear
            | Instruction::Background { .. }
            | Instruction::Music { .. }
            | Instruction::Sound { .. }
            | Instruction::Voice { .. }
            | Instruction::Commit
            | Instruction::With(_)
            | Instruction::Pause
            | Instruction::Goto(_)
            | Instruction::End => {}
        }
    }

    fn variable(&mut self, line: usize, name: &str) -> Option<&'a VariableDef> {
        if self.schema.variables.is_empty() {
            return None;
        }
        let schema: &'a Schema = self.schema;
        let def = schema.variables.get(name);
        if def.is_none() {
            self.error(
                line,
                format!(
                    "unknown variable '{}'{}",
                    name,
                    did_you_mean(name, schema.variables.keys().map(String::as_str))
                ),
            );
        }
        def
    }

    fn character(&mut self, line: usize, id: &str, context: &str) -> Option<&'a CharacterDef> {
        if self.schema.characters.is_empty() {
            return None;
        }
        let schema: &'a Schema = self.schema;
        let def = schema.characters.get(id);
        if def.is_none() {
            self.error(
                line,
                format!(
                    "{}: unknown character '{}'{}",
                    context,
                    id,
                    did_you_mean(id, schema.characters.keys().map(String::as_str))
                ),
            );
        }
        def
    }

    fn interpolations(&mut self, line: usize, text: &str) {
        for name in referenced_variables(text) {
            self.variable(line, name);
        }
        self.diagnostics.extend(crate::markup::validate(text, line));
    }

    fn condition(&mut self, line: usize, condition: &Condition) {
        match condition {
            Condition::All(parts) | Condition::Any(parts) => {
                for part in parts {
                    self.condition(line, part);
                }
            }
            Condition::Test { var_id, op, value } => {
                let Some(def) = self.variable(line, var_id) else {
                    return;
                };

                if let Err(e) = def.ty.check(value) {
                    self.error(line, format!("`{} {} ...`: {}", var_id, op.symbol(), e));
                }
            }
        }
    }
}

pub const SCHEMA_FILE_NAME: &str = "schema.json";
pub const SCHEMA_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaFile {
    pub format_version: u32,
    pub game: String,
    pub story_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_scene: Option<String>,
    #[serde(flatten)]
    pub schema: Schema,
}

impl SchemaFile {
    pub fn new(game: impl Into<String>, story_dir: impl Into<String>, schema: Schema) -> Self {
        Self {
            format_version: SCHEMA_FORMAT_VERSION,
            game: game.into(),
            story_dir: story_dir.into(),
            entry_scene: None,
            schema,
        }
    }

    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("schemas serialize");
        json.push('\n');
        json
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        let file: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if file.format_version > SCHEMA_FORMAT_VERSION {
            return Err(format!(
                "schema format {} is newer than this tool supports ({})",
                file.format_version, SCHEMA_FORMAT_VERSION
            ));
        }
        Ok(file)
    }

    pub fn read(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let json = fs::read_to_string(path)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Self::from_json(&json).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", path.display(), e),
            )
        })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> io::Result<bool> {
        let path = path.as_ref();
        let json = self.to_json();
        if fs::read_to_string(path).is_ok_and(|existing| existing == json) {
            return Ok(false);
        }
        fs::write(path, json)
            .map_err(|e| io::Error::new(e.kind(), format!("{}: {}", path.display(), e)))?;
        Ok(true)
    }
}
