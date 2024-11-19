use crate::{
        parse::nodes::{AIdentifier, Binop},
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
        Unary(ASMUnary, Operand),
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
        AddAssign,
        SubtractAssign,
        MultiplyAssign,
        LeftShiftAssign,
        RightShiftAssign,
        BitwiseAndAssign,
        BitwiseOrAssign,
        BitwiseXOrAssign,
}
#[derive(Debug, Clone, Copy)]
pub enum ASMUnary {
        Increment,
        Decrement,
        Not,
        Negate,
        Complement,
}

#[derive(Debug)]
pub struct NotASMBinary;
impl TryFrom<Binop> for ASMBinary {
        type Error = NotASMBinary;

        fn try_from(value: Binop) -> Result<Self, Self::Error> {
                match value {
                        Binop::Add => Ok(ASMBinary::Add),
                        Binop::Multiply => Ok(ASMBinary::Multiply),
                        Binop::Subtract => Ok(ASMBinary::Subtract),
                        Binop::LeftShift => Ok(ASMBinary::LeftShift),
                        Binop::RightShift => Ok(ASMBinary::RightShift),
                        Binop::BitwiseOr => Ok(ASMBinary::Or),
                        Binop::BitwiseXOr => Ok(ASMBinary::XOr),
                        Binop::BitwiseAnd => Ok(ASMBinary::And),
                        Binop::AddAssign => Ok(ASMBinary::AddAssign),
                        Binop::SubtractAssign => Ok(ASMBinary::SubtractAssign),
                        Binop::MultiplyAssign => Ok(ASMBinary::MultiplyAssign),
                        Binop::LeftShiftAssign => Ok(ASMBinary::LeftShiftAssign),
                        Binop::RightShiftAssign => Ok(ASMBinary::RightShiftAssign),
                        Binop::BitwiseAndAssign => Ok(ASMBinary::BitwiseAndAssign),
                        Binop::BitwiseOrAssign => Ok(ASMBinary::BitwiseOrAssign),
                        Binop::BitwiseXOrAssign => Ok(ASMBinary::BitwiseXOrAssign),
                        _ => Err(NotASMBinary),
                }
        }
}

#[derive(Debug, Clone)]
pub struct ASMProgram {
        pub functions: Vec<ASMFunction>,
}
#[derive(Debug, Clone)]
pub struct ASMFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<ASMInstruction>,
}
