#![feature(type_changing_struct_update)]

use initialize::Operation;

pub mod initialize;
pub mod lex;
pub mod parse;
pub mod toasm;
pub mod write;

#[derive(Debug)]
pub struct Program<S: State> {
        pub operation: Operation,
        pub state: S,
}

pub trait State {}
