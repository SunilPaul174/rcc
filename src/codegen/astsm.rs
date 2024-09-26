use crate::parse::nodes::{
        AConstant, AExpression, AFunction, AIdentifier, AProgram, AStatement,
        ReturnExpression,
};

// program = Program(function_definition)
pub(crate) struct ASMProgram {
        pub function: ASMFunction,
}
impl From<AProgram> for ASMProgram {
        fn from(value: AProgram) -> Self {
                ASMProgram {
                        function: ASMFunction::from(value.function),
                }
        }
}

// function_definition = Function(identifier name, instruction* instructions)
pub(crate) struct ASMFunction {
        pub(crate) identifier: ASMIdentifier,
        pub(crate) instructions: Vec<ASMReturnInstruction>,
}
impl From<AFunction> for ASMFunction {
        fn from(value: AFunction) -> Self {
                let AStatement::ReturnStatement(aexpression) =
                        value.statement_body;

                return ASMFunction {
                        identifier: ASMIdentifier::from(value.identifier),
                        instructions: vec![ASMReturnInstruction::from(
                                aexpression,
                        )],
                };
        }
}

pub(crate) struct ASMIdentifier {
        pub(crate) start: usize,
        pub(crate) len: usize,
}
impl From<AIdentifier> for ASMIdentifier {
        fn from(value: AIdentifier) -> Self {
                ASMIdentifier {
                        start: value.start,
                        len: value.len,
                }
        }
}

impl From<AExpression<ReturnExpression>> for ASMReturnInstruction {
        fn from(value: AExpression<ReturnExpression>) -> Self {
                let aconstant = value.state.constant;
                ASMReturnInstruction {
                        mov: (
                                Operand::from(aconstant),
                                Operand::Register(Register),
                        ),
                }
        }
}

// instruction = Mov(operand src, operand dst) | Ret
pub(crate) struct ASMReturnInstruction {
        pub(crate) mov: (Operand, Operand),
}

// operand = Imm(int) | Register
pub(crate) enum Operand {
        Imm(Imm),
        Register(Register),
}

impl From<AConstant> for Operand {
        fn from(value: AConstant) -> Self {
                Operand::Imm(Imm {
                        start: value.start,
                        len: value.len,
                })
        }
}

pub(crate) struct Register;

pub(crate) struct Imm {
        pub(crate) len: usize,
        pub(crate) start: usize,
}
