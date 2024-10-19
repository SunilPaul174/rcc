use crate::{
        tactile::{TACTILEFunction, TACTILEInstruction, Value, TACTILE},
        Program, State,
};
use nodes::{ASMFunction, ASMInstruction, ASMProgram, Operand, Register};

pub mod nodes;

#[derive(Debug)]
pub struct Compiled {
        pub code: Vec<u8>,
        pub program: ASMProgram,
}
impl State for Compiled {}

pub fn asm(program: Program<TACTILE>) -> Program<Compiled> {
        let aprogram = program.state.program;
        let code = program.state.code;

        let func = ASMFunction::from(aprogram.function);
        let functions = vec![func];

        Program {
                operation: program.operation,
                state: Compiled {
                        code,
                        program: ASMProgram { functions },
                },
        }
}

fn from_tactile(value: TACTILEInstruction, stack_max: &mut usize) -> [ASMInstruction; 2] {
        let instructions;
        match value {
                TACTILEInstruction::Return(val) => {
                        instructions = [
                                ASMInstruction::Mov(val_to_op(val, stack_max), Operand::Register(Register::AX)),
                                ASMInstruction::Ret,
                        ];
                }
                TACTILEInstruction::Unary(unop, src, dst) => {
                        instructions = [
                                ASMInstruction::Mov(val_to_op(src, stack_max), val_to_op(dst, stack_max)),
                                ASMInstruction::Unary(unop, val_to_op(dst, stack_max)),
                        ]
                }
        }
        instructions
}

fn val_to_op(value: Value, stack_max: &mut usize) -> Operand {
        match value {
                Value::Constant(constant) => Operand::Imm(constant),
                Value::Var(identifier) => {
                        *stack_max += 1;
                        Operand::Pseudo(identifier.0)
                }
        }
}

impl From<TACTILEFunction> for ASMFunction {
        fn from(value: TACTILEFunction) -> Self {
                let identifier = value.identifier;
                let mut instructions = vec![];

                // negate and multiply by 4 to get the real value!!!
                let mut stack_max: usize = 1;

                let _: Vec<()> = value
                        .instructions
                        .iter()
                        .map(|&tactile| {
                                instructions.extend(from_tactile(tactile, &mut stack_max));
                        })
                        .collect();

                let instructions = instructions.iter().map(|&f| pseudo_pass(f, &mut stack_max)).collect();

                ASMFunction { identifier, instructions }
        }
}

fn pseudo_to_stack_operand(value: Operand, stack_max: &mut usize) -> Operand {
        match value {
                Operand::Pseudo(n) => {
                        if *stack_max < n {
                                *stack_max = n
                        }

                        Operand::Stack(n)
                }
                _ => value,
        }
}

fn pseudo_pass(value: ASMInstruction, stack_max: &mut usize) -> ASMInstruction {
        match value {
                ASMInstruction::Mov(src, dst) => {
                        let src = pseudo_to_stack_operand(src, stack_max);
                        let dst = pseudo_to_stack_operand(dst, stack_max);

                        ASMInstruction::Mov(src, dst)
                }
                ASMInstruction::Unary(unop, operand) => ASMInstruction::Unary(unop, pseudo_to_stack_operand(operand, stack_max)),
                _ => value,
        }
}
