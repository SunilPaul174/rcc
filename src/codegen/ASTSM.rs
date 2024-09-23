use crate::parse::nodes::{AConstant, AExpression, AFunction, AIdentifier, AProgram, AStatement, ReturnExpression};

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
        identifier: ASMIdentifier,
        instructions: Vec<ASMReturnInstruction>,
}
impl From<AFunction> for ASMFunction {
        fn from(value: AFunction) -> Self {
                let AStatement::ReturnStatement(aexpression) = value.statement_body;

                return ASMFunction {
                        identifier: ASMIdentifier::from(value.identifier),
                        instructions: vec![ASMReturnInstruction::from(aexpression)],
                };
        }
}

struct ASMIdentifier {
        start: usize,
        len: usize,
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
                        mov: Some((Operand::from(aconstant), Operand::Register(Register))),
                        ret: (),
                }
        }
}

// instruction = Mov(operand src, operand dst) | Ret
struct ASMReturnInstruction {
        mov: Option<(Operand, Operand)>,
        ret: (),
}

// operand = Imm(int) | Register
enum Operand {
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

struct Register;

struct Imm {
        len: usize,
        start: usize,
}
