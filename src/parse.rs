use crate::{Lexed, Parsed, Program};
use thiserror::Error;

#[derive(Error, Debug)]
enum ParseError {}

fn parse(program: Program<Lexed>) -> Result<Program<Parsed>, ParseError> {
        todo!()
}
