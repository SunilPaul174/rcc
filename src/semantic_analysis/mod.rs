use std::collections::HashMap;

use crate::{
        parse::nodes::{AExpression, AFactor, AProgram, AStatement},
        tactile::Identifier,
        State,
};

pub mod loop_labeling;
pub mod variable_resolution;

#[derive(Debug, Clone)]
pub struct SemanticallyAnalyzed {
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

type IdentifierMap<'b> = Result<(SemanticallyAnalyzed, usize, HashMap<(&'b [u8], usize), Identifier>), Error>;

pub fn analyze<'b, 'a: 'b>(mut program: AProgram, code: &'a [u8]) -> IdentifierMap<'b> {
        let variable_map = resolve_variables(code, &program)?;
        let max_label = label_loops(&mut program)?;

        Ok((SemanticallyAnalyzed { program }, max_label, variable_map))
}
