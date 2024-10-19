use crate::parse::nodes::{AConstant, AIdentifier, Unop};

impl From<AConstant> for Operand {
        fn from(value: AConstant) -> Self { Operand::Imm(value) }
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
        Imm(AConstant),
        Register(Register),
        //usize is number of temporary variable
        Pseudo(usize),
        //usize is stack size
        Stack(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum Register {
        AX,
        R10,
}
impl From<Register> for Operand {
        fn from(value: Register) -> Self { Operand::Register(value) }
}

#[derive(Debug, Clone, Copy)]
pub enum ASMInstruction {
        // src, dst
        Mov(Operand, Operand),
        Unary(Unop, Operand),
        AllocateStack(usize),
        Ret,
}

#[derive(Debug)]
pub struct ASMProgram {
        pub functions: Vec<ASMFunction>,
}
#[derive(Debug)]
pub struct ASMFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<ASMInstruction>,
}
