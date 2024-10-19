// TACTILE ADSL
// program = Program(function_definition)
// function_definition = Function(identifier, list instruction body)
// instruction = Return(val) | Unary(unary_operator, val src, val dst)
// val = Constant(int) | Var(identifier)
// unary_operator = Complement | Negate

use crate::{
        parse::{
                nodes::{AConstant, AExpression, AFunction, AIdentifier, AProgram, Unop},
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
                let mut global = 0;

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
        if let AExpression::Constant(n) = value {
                return Value::Constant(n);
        }

        let AExpression::Unop(unop, aexpression) = value else {
                panic!("BUGGGGGGGGG")
        };

        let src = emit_tacky(*aexpression, instructions, tmp);

        let dst_name = *tmp;
        *tmp += 1;

        let dst = Value::Var(Identifier(dst_name));
        instructions.push(TACTILEInstruction::Unary(unop, src, dst));

        dst
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
