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
pub enum AExpression {
        Constant(AConstant),
        Unop(Unop, Box<AExpression>),
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
