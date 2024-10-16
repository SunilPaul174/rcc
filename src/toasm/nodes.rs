use crate::parse::nodes::{AConstant, AExpression, AFunction, AIdentifier, AStatement};

#[derive(Debug)]
pub struct Imm(pub AConstant);
impl From<AConstant> for Imm {
        fn from(value: AConstant) -> Self { Imm(value) }
}

#[derive(Debug)]
pub enum Operand {
        Imm(Imm),
        Register(Register),
}

#[derive(Debug)]
pub enum Register {
        EAX,
}

#[derive(Debug)]
pub struct Mov {
        pub src: Operand,
        pub dest: Operand,
}
#[derive(Debug)]
pub enum ASMInstruction {
        Mov(Mov),
        Ret,
}

#[derive(Debug)]
pub struct ASMProgram {
        pub functions: Vec<ASMFunction>,
}
#[derive(Debug)]
pub struct ASMFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<ASMInstruction>,
}

impl From<AFunction> for ASMFunction {
        fn from(value: AFunction) -> Self {
                let identifier = value.identifier;
                let AStatement { expr: AExpression(constant) } = value.statement_body;
                let imm = Imm::from(constant);

                let mov_instruct = ASMInstruction::Mov(Mov {
                        src: Operand::Imm(imm),
                        dest: Operand::Register(Register::EAX),
                });

                let ret = ASMInstruction::Ret;

                ASMFunction {
                        identifier,
                        instructions: vec![mov_instruct, ret],
                }
        }
}
