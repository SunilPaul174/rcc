use crate::parse::nodes::{AConstant, AFunction, AIdentifier, AProgram, AStatement, ReturnExpression};

impl<'a> AConstant {
        pub fn out(&'a self, output: &'a Vec<u8>) -> &'a [u8] { &output[self.start..(self.start + self.len)] }
}
impl<'a> AIdentifier {
        pub fn out(&'a self, output: &'a Vec<u8>) -> &'a [u8] { &output[self.start..(self.start + self.len)] }
}

impl<'a> ReturnExpression {
        pub fn out(&'a self, output: &'a Vec<u8>) -> String {
                let constant: String = self.constant.out(&output).iter().map(|f| *f as char).collect();
                format!("\"Return: {}\"", constant)
        }
}
impl<'a> AStatement {
        pub fn out(&'a self, output: &'a Vec<u8>) -> String {
                let AStatement::ReturnStatement(temp) = self;
                temp.out(&output)
        }
}

impl<'a> AFunction {
        pub fn out(&'a self, output: &'a Vec<u8>) -> String {
                let identifier = self.identifier.out(&output);
                let statement = self.statement_body.out(&output);
                let identifier = String::from_utf8(identifier.to_vec()).unwrap();
                // format!("{{\"{}\": {{{}}}}}", identifier, statement)
                format!("[\"{}\", {}]", identifier, statement)
        }
}

impl<'a> AProgram {
        pub fn out(&'a self, output: &'a Vec<u8>) -> String {
                let temp = self.function.out(&output);
                format!("{{\"Function\": {}}}", temp)
        }
}
