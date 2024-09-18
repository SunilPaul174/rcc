use crate::{codegen, lex, parse};
use std::{
        fs::{remove_file, File},
        io::{self, Write},
        path::PathBuf,
        process::Command,
        str::FromStr,
};
use thiserror::Error;

pub struct Initialized(PathBuf);
pub struct Preprocessed;
pub struct Lexed;
pub struct Parsed;
pub struct CodeGenerated;
pub struct Compiled(PathBuf);

pub trait CompilationState {}

impl CompilationState for Initialized {}
impl CompilationState for Preprocessed {}
impl CompilationState for Lexed {}
impl CompilationState for Parsed {}
impl CompilationState for CodeGenerated {}
impl CompilationState for Compiled {}

pub struct Program<S: CompilationState> {
        operation: RequestedOperation,
        state: S,
        working_file: Vec<u8>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
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

impl Program<Initialized> {
        fn preprocess(self) -> Result<Program<Preprocessed>, io::Error> {
                // cc -E -P INPUTFILE -o PREPROCESSEDFILE

                let input = self.state.0;
                let mut binding = Command::new("cc");
                let preprocess =
                        binding.args(["-E", "-P"]).arg(input).args(["-o", "-"]);
                let pre_processor_output = preprocess.output()?.stdout;

                Ok(Program {
                        operation: self.operation,
                        working_file: pre_processor_output,
                        state: Preprocessed,
                })
        }
}

impl Program<CodeGenerated> {
        fn assemble_and_link(self) -> Result<Program<Compiled>, io::Error> {
                // cc ASSEMBLY_FILE -o OUTPUT_FILE

                let mut file = File::create("created_asm.s")?;
                file.write_all(&self.working_file)?;
                let mut assembler_and_linker = Command::new("cc");
                let asm_and_link = assembler_and_linker.args([
                        "created_asm.s",
                        "-o",
                        "a.out",
                ]);
                let _ = asm_and_link.output()?;
                remove_file("created_asm.s")?;

                Ok(Program {
                        working_file: vec![],
                        operation: self.operation,
                        state: Compiled(PathBuf::from("a.out")),
                })
        }
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
                "--codegen" => Ok((RequestedOperation::CodeGen, file)),
                "-S" => Ok((RequestedOperation::Emit, file)),
                _ => Ok((RequestedOperation::Compile, file)),
        }
}

fn drive() -> Result<(), DriverError> {
        let request = get_request();
        let Ok((operation, currentfile)) = request else {
                return Err(DriverError::Request(request.err().unwrap()));
        };

        let program: Program<Initialized> = Program {
                operation,
                working_file: vec![],
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
        let program = program.codegen()?;
        if program.operation == RequestedOperation::CodeGen {
                return Ok(());
        }
        let program = program.assemble_and_link()?;
        if program.operation == RequestedOperation::Emit {
                return Ok(());
        }
        Ok(())
}
