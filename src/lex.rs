use crate::{Lexed, Preprocessed, Program};
use thiserror::Error;

#[derive(Error, Debug)]
enum LexError {}

fn lex(program: Program<Preprocessed>) -> Result<Program<Lexed>, LexError> {
        todo!()
}
