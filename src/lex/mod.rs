use thiserror::Error;

use crate::{Lexed, Preprocessed, Program};

static INT: [u8; 3] = [b'i', b'n', b't'];
static VOID: [u8; 4] = [b'v', b'o', b'i', b'd'];
static RETURN: [u8; 6] = [b'r', b'e', b't', b'u', b'r', b'n'];

#[derive(Debug, Copy, Clone)]
pub enum TokenType {
        // usizes are lengths
        Identifier(usize),
        Constant(usize),
        KeywordInt,
        KeywordVoid,
        KeywordReturn,
        OpenBrace,
        CloseBrace,
        OpenParanthesis,
        CloseParanthesis,
        SemiColon,
}

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
                // let slice_left = || {
                //         curr_slice.as_ptr() as usize
                //                 - pre_processor_output.as_ptr() as usize
                // };
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

fn get_largest_match(
        curr_slice: &[u8],
        start: usize,
) -> Option<(Token, usize)> {
        match curr_slice[0] {
                b'(' => {
                        return Some((
                                Token {
                                        token_type: TokenType::OpenParanthesis,
                                        start,
                                },
                                1,
                        ))
                }
                b')' => {
                        return Some((
                                Token {
                                        token_type: TokenType::CloseParanthesis,
                                        start,
                                },
                                1,
                        ))
                }
                b'{' => {
                        return Some((
                                Token {
                                        token_type: TokenType::OpenBrace,
                                        start,
                                },
                                1,
                        ))
                }
                b'}' => {
                        return Some((
                                Token {
                                        token_type: TokenType::CloseBrace,
                                        start,
                                },
                                1,
                        ))
                }
                b';' => {
                        return Some((
                                Token {
                                        token_type: TokenType::SemiColon,
                                        start,
                                },
                                1,
                        ))
                }
                _ => {}
        }
        let mut is_numeric = true;
        let mut is_alphabetic = true;
        let mut len = 0;

        // check if these are valid anything
        for i in curr_slice {
                let curr_alpha = i.is_ascii_alphabetic();
                let curr_digit = i.is_ascii_digit();
                // TODO fix yo bool here

                if (is_numeric) & (!curr_digit) & (curr_alpha) & (is_alphabetic)
                {
                        is_numeric = false;
                } else if (is_numeric) & (!is_alphabetic) & (curr_alpha) {
                        return None;
                }

                if (is_alphabetic) & (!curr_alpha) & (curr_digit) & (is_numeric)
                {
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

        let temp = &curr_slice[..len];
        if temp == INT {
                return Some((
                        Token {
                                token_type: TokenType::KeywordInt,
                                start,
                        },
                        len,
                ));
        } else if temp == VOID {
                return Some((
                        Token {
                                token_type: TokenType::KeywordVoid,
                                start,
                        },
                        len,
                ));
        } else if temp == RETURN {
                return Some((
                        Token {
                                token_type: TokenType::KeywordReturn,
                                start,
                        },
                        len,
                ));
        }

        Some((
                Token {
                        token_type: TokenType::Identifier(len),
                        start,
                },
                len,
        ))
}
