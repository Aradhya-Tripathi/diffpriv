use crate::{builtins::Symbols, tokens::OperatorType};

/// Higher level Token types
#[derive(Debug)]
pub enum PreprocessedTokenTypes {
    Symbol(Symbols),
    Identifier(String),
    Operator(OperatorType),
}

impl PreprocessedTokenTypes {
    pub fn from_string(buffer: &String) -> Self {
        let case_insensitive_buffer = buffer.to_ascii_lowercase();
        match case_insensitive_buffer.as_str() {
            "select" => PreprocessedTokenTypes::Symbol(Symbols::SELECT),
            "where" => PreprocessedTokenTypes::Symbol(Symbols::WHERE),
            "*" => PreprocessedTokenTypes::Symbol(Symbols::WILDCARD),
            _ => PreprocessedTokenTypes::Identifier(buffer.to_string()),
        }
    }
}
