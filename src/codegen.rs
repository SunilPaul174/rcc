use thiserror::Error;

use crate::driver::{Program, _CodeGenerated, _Parsed};

#[derive(Debug, Error)]
pub enum CodeGenError {}

pub fn codegen(
        program: Program<_Parsed>,
) -> Result<Program<_CodeGenerated>, CodeGenError> {
        todo!()
}
