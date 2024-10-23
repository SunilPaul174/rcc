#[derive(Debug)]
pub struct AProgram {
        pub functions: AFunction,
}

#[derive(Debug)]
pub struct AFunction {
        pub identifier: AIdentifier,
        pub statement_body: AStatement,
}

#[derive(Debug, Clone, Copy)]
pub struct AIdentifier {
        pub start: usize,
        pub len: usize,
}
#[derive(Debug)]
pub struct AReturnStatement {
        pub statement: AStatement,
}
#[derive(Debug)]
pub struct AStatement {
        pub expr: AExpression,
}
#[derive(Debug)]
pub enum AFactor {
        Constant(AConstant),
        Unop(Unop, Box<AFactor>),
        Expr(Box<AExpression>),
}
#[derive(Debug)]
pub enum AExpression {
        Factor(AFactor),
        BinOp(BinOp, Box<AExpression>, Box<AExpression>),
}
#[derive(Debug, Clone, Copy)]
pub enum Unop {
        Negate,
        Complement,
}
#[derive(Debug, Clone, Copy)]
pub struct AConstant {
        pub start: usize,
        pub len: usize,
}
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
        Add,
        Subtract,
        Multiply,
        Divide,
        Remainder,
        LeftShift,
        RightShift,
        And,
        Or,
        XOr,
}
