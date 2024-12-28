use std::collections::HashMap;

use crate::{
        parse::nodes::{AExpression, AFactor, AIdentifier, AProgram, AStatement},
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
use type_checker::{type_check, FuncType, Type};

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
        #[error("Incompatible function definitions, one with of {0:?} and other with {1:?}")]
        IncompatibleFunctionDeclarations(FuncType, FuncType),
        #[error("Function is defined more than once")]
        FunctionDefinedMoreThanOnce(AIdentifier),
        #[error("Identifier is being used incorrectly. The identifier {0} is a {1:?} but was called as a {2:?}")]
        WrongType(String, Type, Type),
        #[error("Nested function declaration of {0} starting at {1}")]
        NestedFunctionDeclaration(String, usize),
}

// boolean to indicate if externally linked or not
type IdentifierMap<'b> = HashMap<(&'b [u8], usize), (Identifier, bool)>;

pub fn analyze<'b, 'a: 'b>(
        mut program: AProgram,
        code: &'a [u8],
) -> Result<(IdentifierMap<'b>, SemanticallyAnalyzed, usize), Error> {
        let identifier_map = resolve_identifiers(code, &program)?;
        let max_label = label_loops(&mut program)?;
        let () = type_check(&mut program, code)?;

        Ok((identifier_map, SemanticallyAnalyzed { program }, max_label))
}
