use std::{io, path::PathBuf, process::Command};

use thiserror::Error;

use crate::{Program, State};

#[derive(Debug)]
pub struct Initialized {
        pub code: Vec<u8>,
}
impl State for Initialized {}

#[derive(Debug)]
pub enum Operation {
        Lex,
        ParseToCTree,
        ParseToASMTree,
        GenerateASM,
        Compile,
}

#[derive(Debug, Error)]
pub enum InitializationError {
        #[error("No file input")]
        NoFileInput,
        #[error("No operation input")]
        NoOperationInput,
        #[error("Malformed operation input")]
        MalformedOperationInput,
        #[error("IO Error {0}")]
        IoError(io::Error),
}
impl From<io::Error> for InitializationError {
        fn from(value: io::Error) -> Self { InitializationError::IoError(value) }
}

fn get_request() -> Result<(Operation, PathBuf), InitializationError> {
        let mut args = std::env::args();
        args.next();

        let Some(op) = args.next() else {
                return Err(InitializationError::NoFileInput);
        };

        let Some(file) = args.next() else {
                return Err(InitializationError::NoOperationInput);
        };
        let file = PathBuf::from(file);

        match op.as_str() {
                "--lex" => Ok((Operation::Lex, file)),
                "--parse" => Ok((Operation::ParseToCTree, file)),
                "--codegen" => Ok((Operation::ParseToASMTree, file)),
                "-S" => Ok((Operation::GenerateASM, file)),
                "-C" => Ok((Operation::Compile, file)),
                _ => Err(InitializationError::MalformedOperationInput),
        }
}

pub fn initialize() -> Result<Program<Initialized>, InitializationError> {
        let (operation, path) = get_request()?;
        let mut binding = Command::new("cc");
        let preprocessor = binding.args(["-E", "-P"]).arg(path).args(["-o", "-"]);
        let code = preprocessor.output()?.stdout;

        Ok(Program {
                operation,
                state: Initialized { code },
        })
}
