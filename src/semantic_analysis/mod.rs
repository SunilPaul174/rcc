use std::{
        collections::HashMap,
        hash::{BuildHasher, RandomState},
};

use crate::{
        parse::nodes::{AExpression, AFactor, AProgram, AStatement},
        tactile::Identifier,
        State,
};

pub mod identifier_resolution;
pub mod loop_labeling;
pub mod type_checker;

#[derive(Debug, Clone)]
pub struct SemanticallyAnalyzed {
        pub program: AProgram,
}
impl State for SemanticallyAnalyzed {}

use identifier_resolution::resolve_identifiers;
use loop_labeling::label_loops;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
        #[error("Identifier {0} was declared twice, second at {1}")]
        DeclaredTwice(String, usize),
        #[error("Invalid left side of assignment expr: \n{0}")]
        InvalidLValueExpr(AExpression),
        #[error("Identifier {0} was not declared, at {1}")]
        UndeclaredIdentifier(String, usize),
        #[error("Invalid left side of assignment factor: \n{0:?}")]
        InvalidLValueFactor(AFactor),
        #[error("Break found outside loop: {0:?}")]
        BreakOutsideLoop(AStatement),
}

type IdentifierMap<'b> = HashMap<(&'b [u8], usize), (Identifier, bool)>;

pub fn analyze<'b, 'a: 'b>(mut program: AProgram, code: &'a [u8]) -> Result<(IdentifierMap<'b>, SemanticallyAnalyzed, usize), Error> {
        let variable_map = resolve_identifiers(code, &program)?;
        let max_label = label_loops(&mut program)?;

        Ok((variable_map, SemanticallyAnalyzed { program }, max_label))
}
