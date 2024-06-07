/*In SQL spaces play an important role so we can't just skip whitespaces (free-form language)*/

use crate::{
    symbols::PreprocessedTokenTypes,
    tokens::{OperatorType, PreprocessedToken, Token, TokenType},
};

pub struct Lexer {
    pub client_string: String,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            client_string: input,
        }
    }

    ///This requires a mutable reference to self since we change the client
    /// String to trim any junk
    pub fn eat(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = vec![];
        self.client_string = self.client_string.trim_end().to_string();

        for (index, char) in self.client_string.char_indices() {
            if char == '=' {
                tokens.push(Token {
                    character: char,
                    position: index,
                    token_type: TokenType::Operator(OperatorType::Equal),
                })
            } else if char == ';' {
                tokens.push(Token {
                    character: char,
                    position: index,
                    token_type: TokenType::SemiColon,
                })
            } else if char == ' ' {
                tokens.push(Token {
                    character: char,
                    position: index,
                    token_type: TokenType::WhiteSpace,
                })
            } else {
                tokens.push(Token {
                    character: char,
                    position: index,
                    token_type: TokenType::Misc,
                })
            }
        }
        tokens
    }

    fn clear_buffer(&self, buff: &mut String, preproctokens: &mut Vec<PreprocessedToken>) {
        if !buff.is_empty() {
            let symbol = PreprocessedTokenTypes::from_string(buff);
            preproctokens.push(PreprocessedToken { value: symbol });
            *buff = String::new();
        }
    }

    pub fn parse(&mut self) -> Vec<PreprocessedToken> {
        let tokens = self.eat();
        let mut has_eof = false;
        let mut preprocessed_tokens: Vec<PreprocessedToken> = vec![];
        let mut buffer: String = String::new();

        for token in tokens {
            match token.token_type {
                TokenType::WhiteSpace | TokenType::SemiColon => {
                    self.clear_buffer(&mut buffer, &mut preprocessed_tokens);
                    if token.character == ';' {
                        has_eof = true;
                    }
                }
                _ => {
                    if token.token_type == TokenType::Operator(OperatorType::Equal) {
                        // We have an operator here, clear the buffer and add operator to it.
                        // To tackle something like: SELECT * FROM USERS WHERE NAME ='Aradhya';
                        // Since space would mean that we are already processing the it and clearing the buffer.
                        self.clear_buffer(&mut buffer, &mut preprocessed_tokens);
                        let operator = PreprocessedTokenTypes::Operator(OperatorType::Equal);
                        preprocessed_tokens.push(PreprocessedToken { value: operator });
                    } else {
                        buffer.push(token.character)
                    }
                }
            }
        }

        if !buffer.is_empty() {
            let symbol = PreprocessedTokenTypes::from_string(&buffer);
            preprocessed_tokens.push(PreprocessedToken { value: symbol });
        }

        if has_eof {
            let symbol = PreprocessedTokenTypes::from_string(&';'.to_string());
            preprocessed_tokens.push(PreprocessedToken { value: symbol });
        }
        preprocessed_tokens
    }
}
