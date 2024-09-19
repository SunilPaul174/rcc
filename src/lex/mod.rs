use crate::{Lexed, Preprocessed, Program};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexError {}

enum TokenType {
        Identifier,
        Constant,
        KeywordInt,
        KeywordVoid,
        KeywordReturn,
        OpenParenthesis,
        CloseParenthesis,
        OpenBrace,
        CloseBrace,
        SemiColon,
}

struct Token {
        left: usize,
        right: usize,
        r#type: TokenType,
}

impl Program<Preprocessed> {
        pub fn lex(&self) -> Result<Program<Lexed>, LexError> {
                todo!()
        }
}
