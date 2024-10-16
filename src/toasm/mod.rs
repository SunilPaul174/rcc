use nodes::{ASMFunction, ASMProgram};

use crate::{parse::Parsed, Program, State};

pub mod nodes;

#[derive(Debug)]
pub struct Compiled {
        pub code: Vec<u8>,
        pub program: ASMProgram,
}
impl State for Compiled {}

pub fn asm(program: Program<Parsed>) -> Program<Compiled> {
        let aprogram = program.state.program;
        let code = program.state.code;
        let mut functions = vec![];

        for i in aprogram.functions {
                let func = ASMFunction::from(i);
                functions.push(func)
        }

        Program {
                operation: program.operation,
                state: Compiled {
                        code,
                        program: ASMProgram { functions },
                },
        }
}
