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
        ReturnStatement(ReturnExpression),
}

#[derive(Debug)]
pub(crate) struct ReturnExpression {
        pub(crate) constant: AConstant,
}

#[derive(Debug)]
pub(crate) struct AConstant {
        pub(crate) len: usize,
        pub(crate) start: usize,
}
