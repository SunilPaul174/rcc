use std::{collections::HashMap, hash::BuildHasher};

use thiserror::Error;

use crate::{
        parse::{
                nodes::{AIdentifier, AProgram, Declaration},
                Parsed,
        },
        Program, State,
};

#[derive(Debug, Error)]
pub enum SemanticError {
        #[error("This variable was declared twice in the same scope.")]
        DefTwice,
}

pub struct SemanticallyAnalyzed {
        pub code: Vec<u8>,
        pub program: AProgram,
}

impl State for SemanticallyAnalyzed {}

impl TryFrom<Program<Parsed>> for Program<SemanticallyAnalyzed> {
        type Error = SemanticError;

        fn try_from(value: Program<Parsed>) -> Result<Self, Self::Error> { todo!() }
}

#[derive(Debug, Clone, Copy)]
pub struct Scope(u8);

fn resolve_declaration<S: BuildHasher>(declaration: Declaration, variable_map: HashMap<(AIdentifier, Scope), usize>) -> Declaration { todo!() }
