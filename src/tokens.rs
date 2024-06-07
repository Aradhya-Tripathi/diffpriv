use crate::symbols::PreprocessedTokenTypes;

#[derive(Debug)]
pub struct Token {
    pub character: char,
    pub position: usize,
    pub token_type: TokenType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    WhiteSpace,
    SemiColon,
    Operator(OperatorType),
    Misc,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OperatorType {
    Equal,
    NotEqual,
}

impl Token {
    pub fn new(character: char, position: usize, token_type: TokenType) -> Self {
        Token {
            character,
            position,
            token_type,
        }
    }
}

#[derive(Debug)]
pub struct PreprocessedToken {
    pub value: PreprocessedTokenTypes,
}
