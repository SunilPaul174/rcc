use crate::{Lexed, Preprocessed, Program};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexError {}

impl Program<Preprocessed> {
        pub fn lex(&self) -> Result<Program<Lexed>, LexError> {
                todo!()
        }
}
