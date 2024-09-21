pub mod tokentype;
use tokentype::TokenType;

use core::str;
use std::collections::HashMap;

use thiserror::Error;

use crate::{Lexed, Preprocessed, Program};

#[derive(Debug)]
pub struct Token {
        pub token_type: TokenType,
        pub start: usize,
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
                let mut tokens: Vec<Token> = vec![];

                while left < total_len {
                        curr_slice = &pre_processor_output[left..];
                        if curr_slice[0].is_ascii_whitespace() {
                                left += 1;
                                continue;
                        }

                        let res = get_largest_match(curr_slice, left);

                        if let Some((token, len)) = res {
                                tokens.push(token);
                                left += len;
                                continue;
                        } else {
                                return Err(InvalidTokenError::At(left));
                        }
                }

                dbg!(Ok(Program {
                        operation: self.operation,
                        state: Lexed {
                                pre_processor_output,
                                tokens,
                        },
                }))
        }
}

fn get_largest_match(curr_slice: &[u8], start: usize) -> Option<(Token, usize)> {
        if let Some(value) = check_for_symbols(curr_slice, start) {
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

        let curr_identifier = str::from_utf8(&curr_slice[..len]).unwrap();

        let keyword_map = HashMap::from([
                ("auto", TokenType::KeywordAuto),
                ("break", TokenType::KeywordBreak),
                ("case", TokenType::KeywordCase),
                ("char", TokenType::KeywordChar),
                ("const", TokenType::KeywordConst),
                ("continue", TokenType::KeywordContinue),
                ("default", TokenType::KeywordDefault),
                ("do", TokenType::KeywordDo),
                ("double", TokenType::KeywordDouble),
                ("else", TokenType::KeywordElse),
                ("enum", TokenType::KeywordEnum),
                ("extern", TokenType::KeywordExtern),
                ("float", TokenType::KeywordFloat),
                ("for", TokenType::KeywordFor),
                ("goto", TokenType::KeywordGoto),
                ("if", TokenType::KeywordIf),
                ("long", TokenType::KeywordLong),
                ("register", TokenType::KeywordRegister),
                ("return", TokenType::KeywordReturn),
                ("short", TokenType::KeywordShort),
                ("signed", TokenType::KeywordSigned),
                ("sizeof", TokenType::KeywordSizeof),
                ("static", TokenType::KeywordStatic),
                ("struct", TokenType::KeywordStruct),
                ("switch", TokenType::KeywordSwitch),
                ("typedef", TokenType::KeywordTypedef),
                ("union", TokenType::KeywordUnion),
                ("unsigned", TokenType::KeywordUnsigned),
                ("void", TokenType::KeywordVoid),
                ("volatile", TokenType::KeywordVolatile),
                ("while", TokenType::KeywordWhile),
                ("int", TokenType::KeywordInt),
        ]);

        let Some(&token_type) = keyword_map.get(curr_identifier) else {
                return Some((
                        Token {
                                token_type: TokenType::Identifier(len),
                                start,
                        },
                        len,
                ));
        };

        Some((Token { token_type, start }, len))
}

fn check_for_symbols(curr_slice: &[u8], start: usize) -> Option<(Token, usize)> {
        let token_type = match curr_slice[0] {
                b'(' => TokenType::OpenParanthesis,
                b')' => TokenType::CloseParanthesis,
                b'{' => TokenType::OpenBrace,
                b'}' => TokenType::CloseBrace,
                b';' => TokenType::SemiColon,
                _ => return None,
        };

        Some((Token { token_type, start }, 1))
}
