use crate::driver::{_Lexed, _Parsed, Program};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {}

pub fn parse(program: Program<_Lexed>) -> Result<Program<_Parsed>, ParseError> {
        todo!()
}
