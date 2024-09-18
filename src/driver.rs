use crate::{codegen, lex, parse};
use std::{io, marker::PhantomData, path::PathBuf, process::Command};
use thiserror::Error;

pub struct _Initialized;
pub struct _Preprocessed;
pub struct _Lexed;
pub struct _Parsed;
pub struct _CodeGenerated;
pub struct _Compiled;

pub trait _CompilationState {}

impl _CompilationState for _Initialized {}
impl _CompilationState for _Preprocessed {}
impl _CompilationState for _Lexed {}
impl _CompilationState for _Parsed {}
impl _CompilationState for _CodeGenerated {}
impl _CompilationState for _Compiled {}

pub struct Program<S: _CompilationState> {
        operation: RequestedOperation,
        _state: PhantomData<S>,
        current_file: PathBuf,
}

pub enum RequestedOperation {
        Lex,
        Parse,
        Codegen,
        // No assemble + Link
        Emit,
        // Do everything
        Compile,
}

#[derive(Debug, Error)]
enum DriverError {
        #[error("The Request {0} cannot be fulfilled")]
        Request(String),
        #[error("The preprocessor exited")]
        PreProcessor(io::Error),
        #[error("The lexer failed with: {0}")]
        Lex(#[from] lex::LexError),
        #[error("The parser failed with: {0}")]
        Parse(#[from] parse::ParseError),
        #[error("Code generation failed with: {0}")]
        CodeGen(#[from] codegen::CodeGenError),
        #[error("Assembly and linkage failed with: {0}")]
        ASMLink(#[from] io::Error),
}

fn preprocess(
        program: Program<_Initialized>,
) -> Result<Program<_Preprocessed>, io::Error> {
        // gcc -E -P INPUT_FILE -o PREPROCESSED_FILE

        let input = program.current_file;
        let preprocessor_output = input.with_extension("s");
        let mut binding = Command::new("gcc");
        let preprocess = binding
                .args(["-E", "-P"])
                .arg(input)
                .arg("-o")
                .arg(&preprocessor_output);
        let _ = preprocess.output()?;

        Ok(Program {
                operation: program.operation,
                current_file: preprocessor_output,
                _state: std::marker::PhantomData,
        })
}

fn assemble_and_link(
        program: Program<_CodeGenerated>,
) -> Result<Program<_Compiled>, io::Error> {
        todo!()
}

fn get_request() -> Result<(RequestedOperation, PathBuf), String> {
        let mut args = std::env::args();

        let Some(file) = args.next() else {
                return Err(String::from("No args"));
        };
        let file = PathBuf::from(file);

        let Some(op) = args.next() else {
                return Err(String::from("No op"));
        };

        match op.as_str() {
                "--lex" => Ok((RequestedOperation::Lex, file)),
                "--parse" => Ok((RequestedOperation::Parse, file)),
                "--codegen" => Ok((RequestedOperation::Codegen, file)),
                "-S" => Ok((RequestedOperation::Emit, file)),
                _ => Ok((RequestedOperation::Compile, file)),
        }
}

fn drive() -> Result<(), DriverError> {
        let request = get_request();
        let Ok((operation, current_file)) = request else {
                return Err(DriverError::Request(request.err().unwrap()));
        };

        let program: Program<_Initialized> = Program {
                operation,
                current_file,
                _state: std::marker::PhantomData,
        };

        let program = preprocess(program);
        let Ok(program) = program else {
                return Err(DriverError::PreProcessor(program.err().unwrap()));
        };

        let program = lex::lex(program)?;
        let program = parse::parse(program)?;
        let program = codegen::codegen(program)?;
        let program = assemble_and_link(program)?;

        Ok(())
}
