#[derive(Debug)]
pub struct AProgram {
        pub functions: Vec<AFunction>,
}

#[derive(Debug)]
pub struct AFunction {
        pub identifier: AIdentifier,
        pub statement_body: AStatement,
}

#[derive(Debug)]
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
pub struct AExpression(pub AConstant);
#[derive(Debug)]
pub struct AConstant {
        pub start: usize,
        pub len: usize,
}
