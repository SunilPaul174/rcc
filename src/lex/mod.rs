use std::{
        collections::HashMap,
        hash::{BuildHasher, BuildHasherDefault, Hasher},
};

use thiserror::Error;
use tokentype::{Token, TokenType};

use crate::{initialize::Initialized, Program, State};

pub mod tokentype;

#[derive(Debug, Clone)]
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

pub static INT: &[u8] = b"int";
pub static VOID: &[u8] = b"void";
pub static RETURN: &[u8] = b"return";
pub static IF: &[u8] = b"if";
pub static ELSE: &[u8] = b"else";
pub static DO: &[u8] = b"do";
pub static WHILE: &[u8] = b"while";
pub static FOR: &[u8] = b"for";
pub static BREAK: &[u8] = b"break";
pub static CONTINUE: &[u8] = b"continue";
pub static SWITCH: &[u8] = b"switch";
pub static CASE: &[u8] = b"case";
pub static DEFAULT: &[u8] = b"default";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeywordHash(pub u32);

impl Hasher for KeywordHash {
        fn finish(&self) -> u64 {
                self.0 as u64
        }

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

        fn build_hasher(&self) -> Self::Hasher {
                *self
        }
}

type KeywordHasher = BuildHasherDefault<KeywordHash>;

pub fn lex(program: Program<Initialized>) -> Result<Program<Lexed>, Error> {
        let mut keyword_map: HashMap<&[u8], TokenType, KeywordHasher> =
                HashMap::with_capacity_and_hasher(3, BuildHasherDefault::default());
        keyword_map.entry(INT).or_insert(TokenType::Int);
        keyword_map.entry(VOID).or_insert(TokenType::Void);
        keyword_map.entry(RETURN).or_insert(TokenType::Return);
        keyword_map.entry(IF).or_insert(TokenType::If);
        keyword_map.entry(ELSE).or_insert(TokenType::Else);
        keyword_map.entry(DO).or_insert(TokenType::Do);
        keyword_map.entry(WHILE).or_insert(TokenType::While);
        keyword_map.entry(FOR).or_insert(TokenType::For);
        keyword_map.entry(BREAK).or_insert(TokenType::Break);
        keyword_map.entry(CONTINUE).or_insert(TokenType::Continue);
        keyword_map.entry(SWITCH).or_insert(TokenType::Switch);
        keyword_map.entry(CASE).or_insert(TokenType::Case);
        keyword_map.entry(DEFAULT).or_insert(TokenType::Default);

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

pub fn get_largest_match<S: BuildHasher>(
        code: &[u8],
        start: usize,
        keyword_map: &HashMap<&[u8], TokenType, S>,
) -> Option<Token> {
        if let Some(value) = match_symbol(code, start) {
                return Some(value);
        }

        let is_constant = code[start].is_ascii_digit();
        let is_identifier = code[start].is_ascii_alphabetic() | (code[start] == b'_');

        if !(is_identifier | is_constant) {
                return None;
        }

        let mut len = 0;

        for &i in &code[start..] {
                let is_digit = i.is_ascii_digit();
                let is_alpha = i.is_ascii_alphabetic();
                if is_constant && !is_digit {
                        if is_alpha {
                                return None;
                        }
                        break;
                }
                if !((i == b'_') | is_alpha | is_digit) {
                        break;
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

        if (len <= 1) | (len > 8) {
                return Some(Token {
                        token_type: TokenType::Identifier,
                        len,
                        start,
                });
        }

        let curr_slice = &code[start..start + len];

        if let Some(&token_type) = keyword_map.get(curr_slice) {
                return Some(Token { token_type, len, start });
        }

        Some(Token {
                token_type: TokenType::Identifier,
                len,
                start,
        })
}

fn match_symbol(code: &[u8], start: usize) -> Option<Token> {
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
                b'?' => Some(TokenType::Ternary),
                b':' => Some(TokenType::Colon),
                b',' => Some(TokenType::Comma),
                _ => None,
        } {
                let Some(curr) = code.get(start + 1) else {
                        return Some(Token {
                                token_type,
                                len: 1,
                                start,
                        });
                };

                if let Some(token_type) = match (token_type, curr) {
                        (TokenType::Minus, b'-') => Some(TokenType::DoubleMinus),
                        (TokenType::Minus, b'=') => Some(TokenType::SubtractAssign),
                        (TokenType::Plus, b'+') => Some(TokenType::DoublePlus),
                        (TokenType::Plus, b'=') => Some(TokenType::AddAssign),
                        (TokenType::Asterisk, b'=') => Some(TokenType::MultiplyAssign),
                        (TokenType::ForwardSlash, b'=') => Some(TokenType::DivideAssign),
                        (TokenType::Percent, b'=') => Some(TokenType::RemainderAssign),
                        (TokenType::BitwiseAnd, b'&') => Some(TokenType::LogicalAnd),
                        (TokenType::BitwiseOr, b'|') => Some(TokenType::LogicalOr),
                        (TokenType::LessThan, b'<') => Some(TokenType::LeftShift),
                        (TokenType::LessThan, b'=') => Some(TokenType::LessThanOrEqual),
                        (TokenType::MoreThan, b'>') => Some(TokenType::RightShift),
                        (TokenType::MoreThan, b'=') => Some(TokenType::MoreThanOrEqual),
                        (TokenType::Equal, b'=') => Some(TokenType::EqualTo),
                        (TokenType::Not, b'=') => Some(TokenType::NotEqualTo),
                        (TokenType::BitwiseAnd, b'=') => Some(TokenType::BitwiseAndAssign),
                        (TokenType::BitwiseOr, b'=') => Some(TokenType::BitwiseOrAssign),
                        (TokenType::BitwiseXOr, b'=') => Some(TokenType::BitwiseXOrAssign),
                        _ => None,
                } {
                        let Some(curr) = code.get(start + 2) else {
                                return Some(Token {
                                        token_type,
                                        len: 2,
                                        start,
                                });
                        };

                        if let Some(token_type) = match (token_type, curr) {
                                (TokenType::RightShift, b'=') => Some(TokenType::RightShiftAssign),
                                (TokenType::LeftShift, b'=') => Some(TokenType::LeftShiftAssign),
                                _ => None,
                        } {
                                return Some(Token {
                                        token_type,
                                        len: 3,
                                        start,
                                });
                        }

                        return Some(Token {
                                token_type,
                                len: 2,
                                start,
                        });
                }
                return Some(Token {
                        token_type,
                        len: 1,
                        start,
                });
        }
        None
}
