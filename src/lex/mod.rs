pub static INT: &[u8; 3] = b"int";
pub static VOID: &[u8; 3] = b"vod";
pub static RETURN: &[u8; 3] = b"ren";

use std::collections::HashMap;

use thiserror::Error;
use tokentype::{Token, TokenType};

use crate::{initialize::Initialized, Program, State};

pub mod tokentype;

#[derive(Debug)]
pub struct Lexed {
        pub code: Vec<u8>,
        pub tokens: Vec<Token>,
}
impl State for Lexed {}

#[derive(Debug, Error)]
pub enum LexError {
        #[error("There are no more valid tokens after {0}")]
        OutOfTokens(usize),
}

pub fn lex(program: Program<Initialized>) -> Result<Program<Lexed>, LexError> {
        let keyword_map = HashMap::from([(INT, TokenType::Int), (VOID, TokenType::Void), (RETURN, TokenType::Return)]);

        let mut left = 0;
        let tot_len = program.state.code.len();
        let code = program.state.code;
        let mut tokens = vec![];

        while left < tot_len {
                if code[left].is_ascii_whitespace() {
                        left += 1;
                        continue;
                }

                let Some(token) = get_largest_match(&code, left, &keyword_map) else {
                        return Err(LexError::OutOfTokens(left));
                };

                left += token.len;
                tokens.push(token);
        }

        Ok(Program {
                state: Lexed { code, tokens },
                ..program
        })
}

pub fn get_largest_match(code: &[u8], start: usize, keyword_map: &HashMap<&[u8; 3], TokenType>) -> Option<Token> {
        if let Some(token_type) = match code[start] {
                b'(' => Some(TokenType::OpenParen),
                b')' => Some(TokenType::CloseParen),
                b'{' => Some(TokenType::OpenBrace),
                b'}' => Some(TokenType::CloseBrace),
                b';' => Some(TokenType::SemiColon),
                b'-' => Some(TokenType::Minus),
                b'~' => Some(TokenType::Tilde),
                _ => None,
        } {
                return Some(Token { token_type, len: 1, start });
        }

        let is_identifier = !code[start].is_ascii_digit();
        let mut is_constant = code[start].is_ascii_digit();
        if !is_identifier && !is_constant {
                return None;
        }
        let mut len = 0;

        for &i in &code[start..] {
                let (is_char, is_digit) = (i.is_ascii_alphabetic(), i.is_ascii_digit());

                if !is_char && !is_digit {
                        break;
                }

                if is_constant && is_char {
                        is_constant = false;
                }
                if !is_identifier && !is_constant {
                        return None;
                }
                len += 1;
        }
        if !is_identifier && !is_constant {
                return None;
        }

        if is_constant {
                return Some(Token {
                        token_type: TokenType::Constant,
                        len,
                        start,
                });
        }

        let curr_slice = &code[start..start + len];
        let arr = &[curr_slice[0], *curr_slice.get(1)?, *curr_slice.last()?];

        if let Some(&token_type) = keyword_map.get(&arr) {
                return Some(Token { token_type, len, start });
        }

        Some(Token {
                token_type: TokenType::Identifier,
                len,
                start,
        })
}
