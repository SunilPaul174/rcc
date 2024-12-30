use crate::parse::nodes::{AConstant, AIdentifier, Binop, Unop};

use super::Identifier;

#[derive(Debug, Clone, Copy)]
pub enum Value {
        Constant(Constant),
        Var(Identifier),
}

#[derive(Debug, Clone, Copy)]
pub enum Constant {
        A(AConstant),
        S(i64),
}

#[derive(Debug, Clone)]
pub enum TACTILEInstruction {
        Return(Value),
        Unary(Unop, Value, Value),
        Binary(Binop, Value, Value, Value),
        Copy(Value, Value),
        Jump(Label),
        JumpIfZero(Value, Label),
        JumpIfNotZero(Value, Label),
        L(Label),
        F(FunctionCall),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
        id: AIdentifier,
        args: Option<Vec<Value>>,
        dst: Value,
}

#[derive(Debug, Clone, Copy)]
pub struct Label(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct TACTILELoopLabel {
        pub begin: usize,
        pub break_label: usize,
        pub continue_label: usize,
}

#[derive(Debug, Clone)]
pub enum TACTILELabel {
        T(TACTILELoopLabel),
        S(SwitchLabel),
}

#[derive(Debug, Clone)]
pub struct SwitchLabel {
        pub label: Label,
}
