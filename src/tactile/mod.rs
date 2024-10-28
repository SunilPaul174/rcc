use crate::{
        parse::{
                nodes::{AConstant, AExpression, AFactor, AIdentifier, AProgram, AStatement, BinOp, Unop},
                Parsed,
        },
        semanalysis::SemanticallyAnalyzed,
        Program, State,
};

#[derive(Debug, Clone, Copy)]
pub struct Identifier(pub usize);

#[derive(Debug)]
pub struct TACTILE {
        pub code: Vec<u8>,
        pub program: TACTILEProgram,
}
impl State for TACTILE {}

#[derive(Debug, Clone, Copy)]
pub enum Value {
        Constant(Constant),
        Var(Identifier),
}

#[derive(Debug, Clone, Copy)]
pub enum Constant {
        A(AConstant),
        S(i64),
}

#[derive(Debug, Clone, Copy)]
pub enum TACTILEInstruction {
        Return(Value),
        Unary(Unop, Value, Value),
        Binary(BinOp, Value, Value, Value),
        Copy(Value, Value),
        Jump(Label),
        JumpIfZero(Value, Label),
        JumpIfNotZero(Value, Label),
        Label(Label),
}

#[derive(Debug, Clone, Copy)]
pub struct Label(pub usize);

#[derive(Debug, Clone)]
pub struct TACTILEFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<TACTILEInstruction>,
}

#[derive(Debug, Clone)]
pub struct TACTILEProgram {
        pub function: TACTILEFunction,
}

fn emit_tacky(value: AExpression, instructions: &mut Vec<TACTILEInstruction>, max_identifier: &mut usize, max_label: &mut usize) -> Value {
        match value {
                AExpression::F(AFactor::Constant(n)) => Value::Constant(Constant::A(n)),
                AExpression::F(AFactor::Unop(unop, afactor)) => {
                        let src = emit_tacky(AExpression::F(*afactor), instructions, max_identifier, max_label);
                        let dst = Value::Var(Identifier(*max_identifier));
                        *max_identifier += 1;
                        instructions.push(TACTILEInstruction::Unary(unop, src, dst));
                        dst
                }
                AExpression::BinOp(binop, src, dst) => match binop {
                        BinOp::LogicalOr => {
                                let false_label = Label(*max_label);
                                *max_label += 1;
                                let end_label = Label(*max_label);
                                *max_label += 1;

                                let v1 = emit_tacky(*src, instructions, max_identifier, max_label);
                                instructions.push(TACTILEInstruction::JumpIfZero(v1, false_label));
                                let v2 = emit_tacky(*dst, instructions, max_identifier, max_label);
                                instructions.push(TACTILEInstruction::JumpIfZero(v2, false_label));

                                let dst = Value::Var(Identifier(*max_identifier));
                                *max_identifier += 1;

                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(1)), dst));
                                instructions.push(TACTILEInstruction::Jump(end_label));
                                instructions.push(TACTILEInstruction::Label(false_label));
                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(0)), dst));
                                instructions.push(TACTILEInstruction::Label(end_label));

                                dst
                        }
                        BinOp::LogicalAnd => {
                                let false_label = Label(*max_label);
                                *max_label += 1;
                                let end_label = Label(*max_label);
                                *max_label += 1;

                                let v1 = emit_tacky(*src, instructions, max_identifier, max_label);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v1, false_label));
                                let v2 = emit_tacky(*dst, instructions, max_identifier, max_label);
                                instructions.push(TACTILEInstruction::JumpIfNotZero(v2, false_label));

                                let dst = Value::Var(Identifier(*max_identifier));
                                *max_identifier += 1;

                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(1)), dst));
                                instructions.push(TACTILEInstruction::Jump(end_label));
                                instructions.push(TACTILEInstruction::Label(false_label));
                                instructions.push(TACTILEInstruction::Copy(Value::Constant(Constant::S(0)), dst));
                                instructions.push(TACTILEInstruction::Label(end_label));

                                dst
                        }
                        _ => {
                                let v1 = emit_tacky(*src, instructions, max_identifier, max_label);
                                let v2 = emit_tacky(*dst, instructions, max_identifier, max_label);
                                let dst = Value::Var(Identifier(*max_identifier));
                                *max_identifier += 1;
                                instructions.push(TACTILEInstruction::Binary(binop, v1, v2, dst));
                                dst
                        }
                },
                AExpression::F(AFactor::Expr(expr)) => emit_tacky(*expr, instructions, max_identifier, max_label),
                AExpression::Assignment(aexpression, aexpression1) => todo!(),
                AExpression::F(AFactor::Id(id)) => todo!(),
        }
}

fn tactile_program(program: AProgram) -> TACTILEProgram {
        TACTILEProgram {
                function: {
                        let value = program.functions;
                        let mut instructions = vec![];
                        let mut global_max_label = 1;
                        let mut global_max_identifier = 1;

                        let AStatement::Return(expr) = todo!() else { todo!() };

                        let val = emit_tacky(expr, &mut instructions, &mut global_max_identifier, &mut global_max_label);
                        instructions.push(TACTILEInstruction::Return(val));

                        TACTILEFunction {
                                identifier: value.identifier,
                                instructions,
                        }
                },
        }
}

pub fn TACTILE(program: Program<SemanticallyAnalyzed>) -> Program<TACTILE> {
        Program {
                operation: program.operation,
                state: TACTILE {
                        code: program.state.code,
                        program: tactile_program(program.state.program),
                },
        }
}
