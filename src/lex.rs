use crate::driver::{Program, _Lexed, _Preprocessed};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexError {}

pub fn lex(
        program: Program<_Preprocessed>,
) -> Result<Program<_Lexed>, LexError> {
        todo!()
}
