use std::{io, path::PathBuf, process::Command};

use thiserror::Error;

use crate::{Program, State};

#[derive(Debug)]
pub struct Initialized {
        pub code: Vec<u8>,
}
impl State for Initialized {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
        Lex,
        ParseToCTree,
        ParseToASMTree,
        ParseToTACTILETree,
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

        let first_two = (args.next(), args.next());

        if let (Some(string), None) = first_two {
                return Ok((Operation::Compile, PathBuf::from(string)));
        }

        let Some(op) = first_two.0 else {
                return Err(InitializationError::NoOperationInput);
        };

        let file = first_two.1.unwrap();
        let file = PathBuf::from(file);

        match op.as_str() {
                "--lex" => Ok((Operation::Lex, file)),
                "--parse" => Ok((Operation::ParseToCTree, file)),
                "--tacky" => Ok((Operation::ParseToTACTILETree, file)),
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
