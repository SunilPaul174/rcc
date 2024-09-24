pub mod astsm;

use astsm::ASMProgram;
use thiserror::Error;

use crate::{ASMASTGenerated, Parsed, Program};

#[derive(Debug, Error)]
pub enum CodeGenError {}

impl Program<Parsed> {
        pub fn code_gen(
                self,
        ) -> Result<Program<ASMASTGenerated>, CodeGenError> {
                let asm_program = ASMProgram::from(self.state.program);
                Ok(Program {
                        state: ASMASTGenerated {
                                asm_program,
                                pre_processor_output: self
                                        .state
                                        .pre_processor_output,
                        },
                        ..self
                })
        }
}
