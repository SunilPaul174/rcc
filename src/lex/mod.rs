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
pub enum Error {
        #[error("There are no more valid tokens after {0}, {1:?}")]
        OutOfTokens(usize, Vec<Token>),
}

pub fn lex(program: Program<Initialized>) -> Result<Program<Lexed>, Error> {
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
                        return Err(Error::OutOfTokens(left, tokens));
                };

                left += token.len;
                tokens.push(token);
        }

        Ok(Program {
                state: Lexed { code, tokens },
                ..program
        })
}

pub fn get_largest_match<S: std::hash::BuildHasher>(code: &[u8], start: usize, keyword_map: &HashMap<&[u8; 3], TokenType, S>) -> Option<Token> {
        if let Some(token_type) = match code[start] {
                b'(' => Some(TokenType::OpenParen),
                b')' => Some(TokenType::CloseParen),
                b'{' => Some(TokenType::OpenBrace),
                b'}' => Some(TokenType::CloseBrace),
                b';' => Some(TokenType::SemiColon),
                b'-' => Some(TokenType::Minus),
                b'~' => Some(TokenType::Tilde),
                b'+' => Some(TokenType::Plus),
                b'*' => Some(TokenType::Asterisk),
                b'/' => Some(TokenType::ForwardSlash),
                b'%' => Some(TokenType::Percent),
                b'&' => Some(TokenType::BitwiseAnd),
                b'|' => Some(TokenType::BitwiseOr),
                b'^' => Some(TokenType::BitwiseXOr),
                b'=' => Some(TokenType::Equal),
                b'<' => Some(TokenType::LessThan),
                b'>' => Some(TokenType::MoreThan),
                b'!' => Some(TokenType::Not),
                _ => None,
        } {
                let temp = code.get(start + 1);
                if temp == None {
                        return Some(Token { token_type, len: 1, start });
                }

                if let Some(token_type) = match (token_type, temp.unwrap()) {
                        (TokenType::Minus, b'-') => Some(TokenType::Decrement),
                        (TokenType::BitwiseAnd, b'&') => Some(TokenType::LogicalAnd),
                        (TokenType::LessThan, b'<') => Some(TokenType::LeftShift),
                        (TokenType::LessThan, b'=') => Some(TokenType::LessThanOrEqual),
                        (TokenType::MoreThan, b'>') => Some(TokenType::RightShift),
                        (TokenType::MoreThan, b'=') => Some(TokenType::MoreThanOrEqual),
                        (TokenType::Equal, b'=') => Some(TokenType::EqualTo),
                        (TokenType::Not, b'=') => Some(TokenType::NotEqualTo),
                        (TokenType::BitwiseOr, b'|') => Some(TokenType::LogicalOr),
                        _ => None,
                } {
                        return Some(Token { token_type, len: 2, start });
                }
                return Some(Token { token_type, len: 1, start });
        }

        let is_constant = code[start].is_ascii_digit();
        if !(code[start].is_ascii_alphabetic() | code[start].is_ascii_digit() | (code[start] == b'_')) {
                return None;
        }
        let mut len = 0;

        for &i in &code[start..] {
                if !(i.is_ascii_alphabetic() | i.is_ascii_digit() | (i == b'_')) {
                        break;
                }
                if is_constant && (!i.is_ascii_digit()) {
                        return None;
                }
                len += 1;
        }

        if is_constant {
                return Some(Token {
                        token_type: TokenType::Constant,
                        len,
                        start,
                });
        }

        let curr_slice = &code[start..start + len];
        let first = *curr_slice.get(0)?;
        let second;
        if let Some(&res) = curr_slice.get(1) {
                second = res;
        } else {
                second = first;
        }
        let arr = &[first, second, *curr_slice.last()?];

        if let Some(&token_type) = keyword_map.get(&arr) {
                return Some(Token { token_type, len, start });
        }

        Some(Token {
                token_type: TokenType::Identifier,
                len,
                start,
        })
}
