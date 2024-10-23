use crate::parse::nodes::{AConstant, AIdentifier, BinOp, Unop};

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
        DX,
        R10,
        R11,
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
        Binary(ASMBinary, Operand, Operand),
        IDiv(Operand),
        Cdq,
        Ret,
}

#[derive(Debug, Clone, Copy)]
pub enum ASMBinary {
        Add,
        Subtract,
        Multiply,
        LeftShift,
        RightShift,
        Or,
        XOr,
        And,
}
pub fn from_binop(binop: BinOp) -> Option<ASMBinary> {
        match binop {
                BinOp::Add => Some(ASMBinary::Add),
                BinOp::Multiply => Some(ASMBinary::Multiply),
                BinOp::Subtract => Some(ASMBinary::Subtract),
                BinOp::LeftShift => Some(ASMBinary::LeftShift),
                BinOp::RightShift => Some(ASMBinary::RightShift),
                BinOp::Or => Some(ASMBinary::Or),
                BinOp::XOr => Some(ASMBinary::XOr),
                BinOp::And => Some(ASMBinary::And),
                _ => None,
        }
}

#[derive(Debug)]
pub struct ASMProgram {
        pub function: ASMFunction,
}
#[derive(Debug, Clone)]
pub struct ASMFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<ASMInstruction>,
}
