use crate::{
        parse::{
                nodes::{AExpression, AFactor, AProgram, AStatement},
                Parsed,
        },
        Program, State,
};

pub mod loop_labeling;
pub mod variable_resolution;

pub struct SemanticallyAnalyzed {
        pub code: Vec<u8>,
        pub program: AProgram,
}
impl State for SemanticallyAnalyzed {}

use loop_labeling::label_loops;
use thiserror::Error;
use variable_resolution::resolve_variables;

#[derive(Debug, Error)]
pub enum Error {
        #[error("Variable {0} was declared twice, second at {1}")]
        DeclaredTwice(String, usize),
        #[error("Invalid left side of assignment expr: \n{0}")]
        InvalidLValueExpr(AExpression),
        #[error("Variable {0} was not declared, at {1}")]
        UndeclaredVariable(String, usize),
        #[error("Invalid left side of assignment factor: \n{0:?}")]
        InvalidLValueFactor(AFactor),
        #[error("Break found outside loop: {0:?}")]
        BreakOutsideLoop(AStatement),
}

pub fn analyze(mut program: Program<Parsed>) -> Result<(Program<SemanticallyAnalyzed>, usize), Error> {
        () = resolve_variables(&program.state.code, &program.state.program)?;
        let max_label = label_loops(&mut program)?;

        Ok((
                Program {
                        operation: program.operation,
                        state: SemanticallyAnalyzed {
                                code: program.state.code,
                                program: program.state.program,
                        },
                },
                max_label,
        ))
}
