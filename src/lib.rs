use std::{marker::PhantomData, path::PathBuf};

use driver::RequestedOperation;
mod codegen;
mod driver;
mod lex;
mod parse;

pub struct Initialized;
pub struct Preprocessed;
pub struct Lexed;
pub struct Parsed;
pub struct CodeGenerated;
pub struct Compiled;

pub trait CompilationState {}

impl CompilationState for Initialized {}
impl CompilationState for Preprocessed {}
impl CompilationState for Lexed {}
impl CompilationState for Parsed {}
impl CompilationState for CodeGenerated {}
impl CompilationState for Compiled {}

pub struct Program<S: CompilationState> {
        operation: RequestedOperation,
        _state: PhantomData<S>,
        current_file: PathBuf,
}
