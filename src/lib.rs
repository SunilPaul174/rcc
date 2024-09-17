use std::marker::PhantomData;

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

pub trait FileState {}

impl FileState for Initialized {}
impl FileState for Preprocessed {}
impl FileState for Lexed {}
impl FileState for Parsed {}
impl FileState for CodeGenerated {}
impl FileState for Compiled {}

pub struct Program<S: FileState> {
        operation: RequestedOperation,
        _state: PhantomData<S>,
}
