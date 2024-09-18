use thiserror::Error;

use crate::{CodeGenerated, Parsed, Program};

#[derive(Debug, Error)]
pub enum CodeGenError {}

impl Program<Parsed> {
        pub fn code_gen(&self) -> Result<Program<CodeGenerated>, CodeGenError> {
                todo!()
        }
}
