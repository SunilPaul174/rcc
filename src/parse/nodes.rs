use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct AProgram {
        pub functions: AFunction,
}

#[derive(Debug, Clone)]
pub struct AFunction {
        pub identifier: AIdentifier,
        pub function_body: Vec<BlockItem>,
}

#[derive(Debug, Clone, Copy)]
pub struct AIdentifier {
        pub start: usize,
        pub len: usize,
}
#[derive(Debug, Clone)]
pub enum BlockItem {
        D(Declaration),
        S(AStatement),
}
#[derive(Debug, Clone)]
pub struct Declaration {
        pub id: AIdentifier,
        pub init: Option<AExpression>,
}
#[derive(Debug, Clone)]
pub enum AStatement {
        Return(AExpression),
        Expr(AExpression),
        Nul,
}
#[derive(Debug, Clone)]
pub enum AFactor {
        Constant(AConstant),
        Unop(Unop, Box<AFactor>),
        Expr(Box<AExpression>),
        Id(AIdentifier),
}

#[derive(Debug, Clone)]
pub enum AExpression {
        F(AFactor),
        BinOp(BinOp, Box<AExpression>, Box<AExpression>),
        Assignment(Box<AExpression>, Box<AExpression>),
}
impl Display for AExpression {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                        AExpression::F(afactor) => write!(f, "{:?}", afactor),
                        AExpression::BinOp(_bin_op, _aexpression, _aexpression1) => write!(f, "{:?}", self),
                        AExpression::Assignment(aexpression, aexpression1) => write!(f, "left: {:?}, right: {:?}", aexpression, aexpression1),
                }
        }
}
#[derive(Debug, Clone, Copy)]
pub enum Unop {
        Negate,
        Complement,
        // !
        Not,
}
#[derive(Debug, Clone, Copy)]
pub struct AConstant {
        pub start: usize,
        pub len: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
        Add,
        Subtract,
        Multiply,
        Divide,
        Remainder,
        LeftShift,
        RightShift,
        BitwiseAnd,
        LogicalAnd,
        BitwiseOr,
        LogicalOr,
        EqualTo,
        NotEqualTo,
        LessThan,
        LessThanOrEqual,
        MoreThan,
        MoreThanOrEqual,
        BitwiseXOr,
        Equal,
}
