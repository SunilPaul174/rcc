use crate::{CodeGenerated, Compiled, Initialized, Preprocessed, Program};

pub enum RequestedOperation {
        Lex,
        Parse,
        Codegen,
        Emit,
        Compile,
}

fn preprocess(program: Program<Initialized>) -> Program<Preprocessed> {
        todo!()
}

fn assemble_and_link(program: Program<CodeGenerated>) -> Program<Compiled> {
        todo!()
}

fn get_request() -> RequestedOperation {
        todo!()
}

fn drive() {
        todo!()
}
