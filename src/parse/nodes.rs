#[derive(Debug)]
pub struct AProgram {
        pub(crate) function: AFunction,
}

#[derive(Debug)]
pub(crate) struct AFunction {
        pub(crate) identifier: AIdentifier,
        pub(crate) statement_body: AStatement,
}

#[derive(Debug)]
pub(crate) struct AIdentifier {
        pub(crate) len: usize,
        pub(crate) start: usize,
}

#[derive(Debug)]
pub(crate) enum AStatement {
        ReturnStatement(AExpression<ReturnExpression>),
}

#[derive(Debug)]
pub(crate) struct AExpression<S: AReturnExpression> {
        pub(crate) state: S,
}

pub trait AReturnExpression {}

#[derive(Debug)]
pub(crate) struct ReturnExpression {
        pub(crate) constant: AConstant,
}

impl AReturnExpression for ReturnExpression {}

#[derive(Debug)]
pub(crate) struct AConstant {
        pub(crate) len: usize,
        pub(crate) start: usize,
}
