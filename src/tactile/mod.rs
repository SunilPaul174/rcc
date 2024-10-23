use crate::{
        parse::{
                nodes::{AConstant, AExpression, AFactor, AFunction, AIdentifier, AProgram, BinOp, Unop},
                Parsed,
        },
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
        Constant(AConstant),
        Var(Identifier),
}

#[derive(Debug, Clone, Copy)]
pub enum TACTILEInstruction {
        Return(Value),
        Unary(Unop, Value, Value),
        Binary(BinOp, Value, Value, Value),
}

#[derive(Debug, Clone)]
pub struct TACTILEFunction {
        pub identifier: AIdentifier,
        pub instructions: Vec<TACTILEInstruction>,
}

#[derive(Debug, Clone)]
pub struct TACTILEProgram {
        pub function: TACTILEFunction,
}

impl From<AFunction> for TACTILEFunction {
        fn from(value: AFunction) -> Self {
                let mut instructions = vec![];
                let mut global = 1;

                let expr = value.statement_body.expr;

                let val = emit_tacky(expr, &mut instructions, &mut global);
                instructions.push(TACTILEInstruction::Return(val));

                TACTILEFunction {
                        identifier: value.identifier,
                        instructions,
                }
        }
}

fn emit_tacky(value: AExpression, instructions: &mut Vec<TACTILEInstruction>, tmp: &mut usize) -> Value {
        match value {
                AExpression::Factor(AFactor::Constant(n)) => Value::Constant(n),
                AExpression::Factor(AFactor::Unop(unop, afactor)) => {
                        let src = emit_tacky(AExpression::Factor(*afactor), instructions, tmp);
                        let dst = Value::Var(Identifier(*tmp));
                        *tmp += 1;
                        instructions.push(TACTILEInstruction::Unary(unop, src, dst));
                        dst
                }
                AExpression::BinOp(bin_op, src, dst) => {
                        let v1 = emit_tacky(*src, instructions, tmp);
                        let v2 = emit_tacky(*dst, instructions, tmp);
                        let dst = Value::Var(Identifier(*tmp));
                        *tmp += 1;
                        instructions.push(TACTILEInstruction::Binary(bin_op, v1, v2, dst));
                        dst
                }
                AExpression::Factor(AFactor::Expr(expr)) => emit_tacky(*expr, instructions, tmp),
        }
}

fn tactile_program(program: AProgram) -> TACTILEProgram {
        TACTILEProgram {
                function: TACTILEFunction::from(program.functions),
        }
}

pub fn TACTILE(program: Program<Parsed>) -> Program<TACTILE> {
        Program {
                operation: program.operation,
                state: TACTILE {
                        code: program.state.code,
                        program: tactile_program(program.state.program),
                },
        }
}
