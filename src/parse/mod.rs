use crate::{Lexed, Parsed, Program};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {}

impl Program<Lexed> {
        pub fn parse(&self) -> Result<Program<Parsed>, ParseError> {
                todo!()
        }
}
