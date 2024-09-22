#![feature(const_refs_to_static)]
pub mod codegen;
pub mod lex;
pub mod parse;

use lex::Token;
use std::{
        fs::{read, remove_file, File},
        io::{self, Write},
        path::PathBuf,
        process::Command,
        string::FromUtf8Error,
};
use thiserror::Error;

pub struct Initialized(PathBuf);
pub struct Preprocessed {
        pre_processor_output: Vec<u8>,
}
#[derive(Debug)]
pub struct Lexed {
        pre_processor_output: Vec<u8>,
        tokens: Vec<Token>,
}
pub struct Parsed;
pub struct CodeGenerated {
        code_generated: Vec<u8>,
}
pub struct Compiled;

pub trait CompilationState {}

impl CompilationState for Initialized {}
impl CompilationState for Preprocessed {}
impl CompilationState for Lexed {}
impl CompilationState for Parsed {}
impl CompilationState for CodeGenerated {}
impl CompilationState for Compiled {}

#[derive(Debug)]
pub struct Program<S: CompilationState> {
        operation: RequestedOperation,
        state: S,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RequestedOperation {
        Lex,
        Parse,
        CodeGen,
        // No assemble + Link
        Emit,
        // Do everything
        Compile,
}

#[derive(Debug, Error)]
pub enum DriverError {
        #[error("The Request {0} cannot be fulfilled")]
        Request(String),
        #[error("The preprocessor exited")]
        PreProcessor(PreProcessorError),
        #[error("The lexer failed with: {0}")]
        Lex(#[from] lex::InvalidTokenError),
        #[error("The parser failed with: {0}")]
        Parse(#[from] parse::ParseError),
        #[error("Code generation failed with: {0}")]
        CodeGen(#[from] codegen::CodeGenError),
        #[error("Assembly and linkage failed with: {0}")]
        ASMLink(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum PreProcessorError {
        #[error("Could not get output of preprocessor")]
        IoError(io::ErrorKind),
        #[error("Contains Invalid UTF8")]
        ReadError(FromUtf8Error),
}

impl From<FromUtf8Error> for PreProcessorError {
        fn from(value: FromUtf8Error) -> Self {
                PreProcessorError::ReadError(value)
        }
}

impl From<io::Error> for PreProcessorError {
        fn from(value: io::Error) -> Self {
                PreProcessorError::IoError(value.kind())
        }
}

impl Program<Initialized> {
        fn preprocess(
                self,
        ) -> Result<Program<Preprocessed>, PreProcessorError> {
                // cc -E -P INPUTFILE -o PREPROCESSEDFILE

                let input = self.state.0;
                let mut binding = Command::new("cc");
                let preprocessor =
                        binding.args(["-E", "-P"]).arg(input).args(["-o", "-"]);
                let pre_processor_output = preprocessor.output()?.stdout;

                // let pre_processor_output = read(input).unwrap();

                Ok(Program {
                        operation: self.operation,
                        state: Preprocessed {
                                pre_processor_output,
                        },
                })
        }
}

impl Program<CodeGenerated> {
        fn assemble_and_link(self) -> Result<Program<Compiled>, io::Error> {
                // cc ASSEMBLY_FILE -o OUTPUT_FILE

                let mut file = File::create("created_asm.s")?;
                file.write_all(&self.state.code_generated)?;
                let mut assembler_and_linker = Command::new("cc");
                let asm_and_link = assembler_and_linker.args([
                        "created_asm.s",
                        "-o",
                        "a.out",
                ]);
                let _ = asm_and_link.output()?;
                remove_file("created_asm.s")?;

                Ok(Program {
                        operation: self.operation,
                        state: Compiled,
                })
        }
}

fn get_request() -> Result<(RequestedOperation, PathBuf), String> {
        let mut args = std::env::args();
        args.next();

        let Some(file) = args.next() else {
                return Err(String::from("No file"));
        };
        let file = PathBuf::from(file);

        let Some(op) = args.next() else {
                return Ok((RequestedOperation::Compile, file));
        };

        match op.as_str() {
                "--lex" => Ok((RequestedOperation::Lex, file)),
                "--parse" => Ok((RequestedOperation::Parse, file)),
                "--codegen" => Ok((RequestedOperation::CodeGen, file)),
                "-S" => Ok((RequestedOperation::Emit, file)),
                _ => Ok((RequestedOperation::Compile, file)),
        }
}

pub fn drive() -> Result<(), DriverError> {
        let request = get_request();
        let Ok((operation, currentfile)) = request else {
                return Err(DriverError::Request(request.err().unwrap()));
        };
        let program: Program<Initialized> = Program {
                operation,
                state: Initialized(currentfile),
        };

        let program = program.preprocess();
        let Ok(program) = program else {
                return Err(DriverError::PreProcessor(program.err().unwrap()));
        };

        let program = program.lex()?;
        if program.operation == RequestedOperation::Lex {
                return Ok(());
        }
        let program = program.parse()?;
        if program.operation == RequestedOperation::Parse {
                return Ok(());
        }
        let program = program.code_gen()?;
        if program.operation == RequestedOperation::CodeGen {
                return Ok(());
        }
        let program = program.assemble_and_link()?;
        if program.operation == RequestedOperation::Emit {
                return Ok(());
        }
        Ok(())
}
