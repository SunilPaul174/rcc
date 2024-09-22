#[derive(Debug)]
pub struct AProgram {
        pub(super) function: AFunction,
}

#[derive(Debug)]
pub(super) struct AFunction {
        pub(super) identifier: AIdentifier,
        pub(super) statement_body: AStatement,
}

#[derive(Debug)]
pub(super) struct AIdentifier(pub(super) (usize, usize));

#[derive(Debug)]
pub(super) enum AStatement {
        ReturnStatement(AExpression),
}

#[derive(Debug)]
pub(super) enum AExpression {
        Constant(usize),
}
