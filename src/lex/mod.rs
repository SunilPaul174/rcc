pub static INT: &[u8; 3] = b"int";
// pub static INT: u32 = 2780299380;
pub static VOID: &[u8; 3] = b"vod";
// pub static VOID: u32 = 3652976740;
pub static RETURN: &[u8; 3] = b"ren";
// pub static RETURN: u32 = 3381919854;

use std::{
        collections::HashMap,
        hash::{BuildHasher, BuildHasherDefault, Hasher},
};

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

#[derive(Debug, Clone, Copy)]
pub struct KeywordHash(pub u32);

impl Hasher for KeywordHash {
        fn finish(&self) -> u64 { self.0 as u64 }

        fn write(&mut self, bytes: &[u8]) {
                let len = bytes.len();
                let mut temp: u32 = 0;
                temp += (bytes[0] as u32) << 26;
                temp += (bytes[1] as u32) << 18;
                temp += bytes[len - 1] as u32;
                self.0 = temp;
        }
}

impl BuildHasher for KeywordHash {
        type Hasher = Self;

        fn build_hasher(&self) -> Self::Hasher { *self }
}

impl Default for KeywordHash {
        fn default() -> Self { KeywordHash(0) }
}
type KeywordHasher = BuildHasherDefault<KeywordHash>;

pub fn lex(program: Program<Initialized>) -> Result<Program<Lexed>, Error> {
        let mut keyword_map: HashMap<&[u8], TokenType, KeywordHasher> = HashMap::with_capacity_and_hasher(3, Default::default());
        // let keyword_map = HashMap::from([(INT, TokenType::Int), (VOID, TokenType::Void), (RETURN, TokenType::Return)]);
        keyword_map.entry(INT).or_insert(TokenType::Int);
        keyword_map.entry(VOID).or_insert(TokenType::Void);
        keyword_map.entry(RETURN).or_insert(TokenType::Return);

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

pub fn get_largest_match<S: BuildHasher>(code: &[u8], start: usize, keyword_map: &HashMap<&[u8], TokenType, S>) -> Option<Token> {
        // pub fn get_largest_match<S: BuildHasher>(code: &[u8], start: usize, keyword_map: &HashMap<&[u8; 3], TokenType, S>) -> Option<Token> {
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
                if temp.is_none() {
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

        if len <= 1 {
                return Some(Token {
                        token_type: TokenType::Identifier,
                        len: 1,
                        start,
                });
        }

        if let Some(&token_type) = keyword_map.get(curr_slice) {
                return Some(Token { token_type, len, start });
        }

        Some(Token {
                token_type: TokenType::Identifier,
                len,
                start,
        })
}
