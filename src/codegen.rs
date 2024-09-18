use thiserror::Error;

use crate::driver::{CodeGenerated, Parsed, Program};

#[derive(Debug, Error)]
pub enum CodeGenError {}

impl Program<Parsed> {
        pub fn codegen(&self) -> Result<Program<CodeGenerated>, CodeGenError> {
                todo!()
        }
}
