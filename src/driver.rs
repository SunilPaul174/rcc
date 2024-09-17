use thiserror::Error;

use crate::{CodeGenerated, Compiled, Initialized, Preprocessed, Program};

pub enum RequestedOperation {
        Lex,
        Parse,
        Codegen,
        Emit,
        Compile,
}

#[derive(Error, Debug)]
enum DriverError {}

fn preprocess(
        program: Program<Initialized>,
) -> Result<Program<Preprocessed>, String> {
        todo!()
}

fn assemble_and_link(
        program: Program<CodeGenerated>,
) -> Result<Program<Compiled>, String> {
        todo!()
}

fn get_request() -> Result<RequestedOperation, String> {
        todo!()
}

fn drive() -> Result<(), DriverError> {
        todo!()
}
