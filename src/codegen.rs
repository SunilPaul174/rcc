use crate::{CodeGenerated, Parsed, Program};
use thiserror::Error;

#[derive(Error, Debug)]
enum CodeGenError {}

fn generate_code(
        program: Program<Parsed>,
) -> Result<Program<CodeGenerated>, CodeGenError> {
        todo!()
}
