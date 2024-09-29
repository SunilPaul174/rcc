pub mod tokentype;

use std::{
        collections::HashMap,
        fmt::{Debug, Display},
};

use crate::lex::tokentype::{INT, RETURN, VOID};
use thiserror::Error;
use tokentype::TokenType;

use crate::{Lexed, Preprocessed, Program};

#[derive(Debug, Clone, Copy)]
pub struct Token {
        pub token_type: TokenType,
        pub start: usize,
}

impl Display for Token {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} at {}", format!("{:?}", self.token_type), self.start)
        }
}

#[derive(Debug, Error)]
pub enum InvalidTokenError {
        #[error("invalid token at {0}")]
        At(usize),
}

impl Program<Preprocessed> {
        // Identifier [a-zA-Z_]\w*\b
        // Constant [0-9]+\b
        // int int\b
        // void void\b
        // return return\b
        // Open parenthesis \(
        // Close parenthesis \)
        // Open brace {
        // Close brace }
        // Semicolon ;

        pub fn lex(self) -> Result<Program<Lexed>, InvalidTokenError> {
                let pre_processor_output = self.state.pre_processor_output;

                let mut curr_slice;
                let mut left = 0;
                let total_len = pre_processor_output.len();
                let mut tokens = vec![];

                while left < total_len {
                        curr_slice = &pre_processor_output[left..];
                        let first = curr_slice[0];
                        if first.is_ascii_whitespace() {
                                left += 1;
                                continue;
                        }

                        let Some((token, len)) = get_largest_match(curr_slice, left, first) else {
                                return Err(InvalidTokenError::At(left));
                        };

                        tokens.push(token);
                        left += len;
                        continue;
                }

                Ok(Program {
                        operation: self.operation,
                        state: Lexed {
                                pre_processor_output,
                                tokens,
                        },
                })
        }
}

fn get_largest_match(curr_slice: &[u8], start: usize, first: u8) -> Option<(Token, usize)> {
        if let Some(value) = is_symbol(first, start) {
                return Some(value);
        }

        let mut is_numeric = true;
        let mut is_alphabetic = true;
        let mut len = 0;

        // check if these are valid anything
        for i in curr_slice {
                let curr_alpha = i.is_ascii_alphabetic();
                let curr_digit = i.is_ascii_digit();

                if (is_numeric) & (!curr_digit) & (curr_alpha) & (is_alphabetic) {
                        is_numeric = false;
                } else if (is_numeric) & (!is_alphabetic) & (curr_alpha) {
                        return None;
                }

                if (is_alphabetic) & (!curr_alpha) & (curr_digit) & (is_numeric) {
                        is_alphabetic = false;
                } else if (is_alphabetic) & (!is_numeric) & (curr_digit) {
                        return None;
                }

                if !(curr_alpha) & !(curr_digit) {
                        break;
                }
                len += 1;
        }

        if is_numeric {
                return Some((
                        Token {
                                token_type: TokenType::Constant(len),
                                start,
                        },
                        len,
                ));
        }

        let slice = &curr_slice[..len];
        let arr = [slice[0], slice[1], slice[slice.len() - 1]];

        let keyword_map = HashMap::from([
                (INT, TokenType::KeywordInt),
                (VOID, TokenType::KeywordVoid),
                (RETURN, TokenType::KeywordReturn),
        ]);

        if let Some(&token_type) = keyword_map.get(&arr) {
                return Some((Token { token_type, start }, len));
        };

        Some((
                Token {
                        token_type: TokenType::Identifier(len),
                        start,
                },
                len,
        ))
}

fn is_symbol(char: u8, start: usize) -> Option<(Token, usize)> {
        let token_type = match char {
                b'(' => TokenType::OpenParanthesis,
                b')' => TokenType::CloseParanthesis,
                b'{' => TokenType::OpenBrace,
                b'}' => TokenType::CloseBrace,
                b';' => TokenType::SemiColon,
                _ => return None,
        };

        Some((Token { token_type, start }, 1))
}
