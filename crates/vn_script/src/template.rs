use std::collections::HashMap;

use crate::parser::is_identifier;
use crate::types::Value;

pub fn interpolate(text: &str, variables: &HashMap<String, Value>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(pos) = rest.find(['{', '}']) {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];

        if let Some(after) = tail.strip_prefix("{{") {
            out.push('{');
            rest = after;
        } else if let Some(after) = tail.strip_prefix("}}") {
            out.push('}');
            rest = after;
        } else if let Some(after) = tail.strip_prefix('}') {
            out.push('}');
            rest = after;
        } else if let Some(close) = tail.find('}') {
            let name = &tail[1..close];
            match variables.get(name) {
                Some(value) if is_identifier(name) => out.push_str(&value.to_string()),
                _ => out.push_str(&tail[..=close]),
            }
            rest = &tail[close + 1..];
        } else {
            out.push_str(tail);
            rest = "";
        }
    }

    out.push_str(rest);
    out
}

pub fn referenced_variables(text: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = text;

    while let Some(pos) = rest.find('{') {
        let tail = &rest[pos..];

        if let Some(after) = tail.strip_prefix("{{") {
            rest = after;
        } else if let Some(close) = tail.find('}') {
            let name = &tail[1..close];
            if is_identifier(name) {
                names.push(name);
            }
            rest = &tail[close + 1..];
        } else {
            break;
        }
    }

    names
}
