use crate::{
        parse::nodes::{AIdentifier, BinOp, Unop},
        tactile::{Constant, Label},
};

impl From<Constant> for Operand {
        fn from(value: Constant) -> Self { Operand::Imm(value) }
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
        Imm(Constant),
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
        Cmp(Operand, Operand),
        Binary(ASMBinary, Operand, Operand),
        IDiv(Operand),
        Cdq,
        Jmp(Label),
        JmpCC(CondCode, Label),
        SetCC(CondCode, Operand),
        Label(Label),
        Ret,
}

#[derive(Debug, Clone, Copy)]
pub enum CondCode {
        E,
        NE,
        G,
        GE,
        L,
        LE,
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
                BinOp::BitwiseOr => Some(ASMBinary::Or),
                BinOp::BitwiseXOr => Some(ASMBinary::XOr),
                BinOp::BitwiseAnd => Some(ASMBinary::And),
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
