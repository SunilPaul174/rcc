use std::{io, path::PathBuf, process::Command};

use thiserror::Error;

use crate::{Program, State};

#[derive(Debug, Clone)]
pub struct Initialized {
        pub code: Vec<u8>,
}
impl State for Initialized {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
        Lex,
        ParseToCTree,
        Validate,
        ParseToASMTree,
        ParseToTACTILETree,
        GenerateASM,
        Compile,
}

#[derive(Debug, Error)]
pub enum Error {
        #[error("No file input")]
        NoFileInput,
        #[error("No operation input")]
        NoOperationInput,
        #[error("Malformed operation input")]
        MalformedOperationInput,
        #[error("IO Error {0}")]
        IoError(io::Error),
}
impl From<io::Error> for Error {
        fn from(value: io::Error) -> Self {
                Error::IoError(value)
        }
}

fn get_request() -> Result<(Operation, PathBuf, bool), Error> {
        let mut args = std::env::args();
        args.next();

        let first_two = (args.next(), args.next());

        if let (Some(string), None) = first_two {
                return Ok((Operation::Compile, PathBuf::from(string), false));
        }

        let Some(op) = first_two.0 else {
                return Err(Error::NoOperationInput);
        };

        let file = first_two.1.unwrap();
        let file = PathBuf::from(file);

        match op.as_str() {
                "--lex" => Ok((Operation::Lex, file, false)),
                "--parse" => Ok((Operation::ParseToCTree, file, false)),
                "--tacky" | "--tactile" => Ok((Operation::ParseToTACTILETree, file, false)),
                "--validate" => Ok((Operation::Validate, file, false)),
                "--codegen" => Ok((Operation::ParseToASMTree, file, false)),
                "-S" => Ok((Operation::GenerateASM, file, false)),
                "-C" => Ok((Operation::Compile, file, false)),
                "-c" => Ok((Operation::Compile, file, true)),
                _ => Err(Error::MalformedOperationInput),
        }
}

pub fn initialize() -> Result<Program<Initialized>, Error> {
        let (operation, path, obj) = get_request()?;
        let mut binding = Command::new("cc");
        let preprocessor = binding.args(["-E", "-P"]).arg(path).args(["-o", "-"]);
        let code = preprocessor.output()?.stdout;

        Ok(Program {
                operation,
                state: Initialized { code },
                obj,
        })
}
