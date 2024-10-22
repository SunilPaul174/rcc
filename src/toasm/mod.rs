use crate::{
        parse::nodes::BinOp,
        tactile::{TACTILEFunction, TACTILEInstruction, Value, TACTILE},
        Program, State,
};
use nodes::{from_binop, ASMBinary, ASMFunction, ASMInstruction, ASMProgram, Operand, Register};

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

fn from_tactile(value: TACTILEInstruction) -> Vec<ASMInstruction> {
        match value {
                TACTILEInstruction::Return(val) => vec![ASMInstruction::Mov(val_to_op(val), Operand::Register(Register::AX)), ASMInstruction::Ret],
                TACTILEInstruction::Unary(unop, src, dst) => vec![ASMInstruction::Mov(val_to_op(src), val_to_op(dst)), ASMInstruction::Unary(unop, val_to_op(dst))],
                TACTILEInstruction::Binary(binop, src1, src2, dst) => match binop {
                        BinOp::Add | BinOp::Subtract | BinOp::Multiply => {
                                vec![
                                        ASMInstruction::Mov(val_to_op(src1), val_to_op(dst)),
                                        ASMInstruction::Binary(from_binop(binop).unwrap(), val_to_op(src2), val_to_op(dst)),
                                ]
                        }

                        BinOp::Divide => vec![
                                ASMInstruction::Mov(val_to_op(src1), Operand::Register(Register::AX)),
                                ASMInstruction::Cdq,
                                ASMInstruction::IDiv(val_to_op(src2)),
                                ASMInstruction::Mov(Operand::Register(Register::AX), val_to_op(dst)),
                        ],
                        BinOp::Remainder => vec![
                                ASMInstruction::Mov(val_to_op(src1), Operand::Register(Register::AX)),
                                ASMInstruction::Cdq,
                                ASMInstruction::IDiv(val_to_op(src2)),
                                ASMInstruction::Mov(Operand::Register(Register::DX), val_to_op(dst)),
                        ],
                },
        }
}

fn val_to_op(value: Value) -> Operand {
        match value {
                Value::Constant(constant) => Operand::Imm(constant),
                Value::Var(identifier) => Operand::Pseudo(identifier.0),
        }
}

impl From<TACTILEFunction> for ASMFunction {
        fn from(value: TACTILEFunction) -> Self {
                let identifier = value.identifier;
                let mut temp_instructions = vec![];
                temp_instructions.push(ASMInstruction::AllocateStack(0));

                let mut stack_max: usize = 0;

                let _: Vec<()> = value
                        .instructions
                        .iter()
                        .map(|&tactile| {
                                temp_instructions.extend(from_tactile(tactile));
                        })
                        .collect();

                let mut temp_instructions: Vec<ASMInstruction> = temp_instructions.iter().map(|&f| pseudo_pass(f, &mut stack_max)).collect();
                temp_instructions[0] = ASMInstruction::AllocateStack(stack_max);

                let mut instructions = Vec::with_capacity(temp_instructions.len() * 2);
                for i in temp_instructions {
                        last_pass(i, &mut instructions);
                }

                ASMFunction { identifier, instructions }
        }
}

fn last_pass(i: ASMInstruction, instructions: &mut Vec<ASMInstruction>) {
        match i {
                ASMInstruction::Mov(Operand::Stack(src), Operand::Stack(dest)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Stack(src), Operand::Register(Register::R10)));
                        instructions.push(ASMInstruction::Mov(Operand::Register(Register::R10), Operand::Stack(dest)));
                }
                ASMInstruction::IDiv(Operand::Imm(aconstant)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Imm(aconstant), Operand::Register(Register::R10)));
                        instructions.push(ASMInstruction::IDiv(Operand::Register(Register::R10)));
                }
                ASMInstruction::Binary(ASMBinary::Add, Operand::Stack(src), Operand::Stack(dst)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Stack(src), Operand::Register(Register::R10)));
                        instructions.push(ASMInstruction::Binary(ASMBinary::Add, Operand::Register(Register::R10), Operand::Stack(dst)));
                }
                ASMInstruction::Binary(ASMBinary::Subtract, Operand::Stack(src), Operand::Stack(dst)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Stack(src), Operand::Register(Register::R10)));
                        instructions.push(ASMInstruction::Binary(ASMBinary::Subtract, Operand::Register(Register::R10), Operand::Stack(dst)));
                }
                ASMInstruction::Binary(ASMBinary::Multiply, src, Operand::Stack(dst)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Stack(dst), Operand::Register(Register::R11)));
                        instructions.push(ASMInstruction::Binary(ASMBinary::Multiply, src, Operand::Register(Register::R11)));
                        instructions.push(ASMInstruction::Mov(Operand::Register(Register::R11), Operand::Stack(dst)));
                }
                _ => instructions.push(i),
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
                ASMInstruction::Binary(binop, src, dst) => ASMInstruction::Binary(binop, pseudo_to_stack_operand(src, stack_max), pseudo_to_stack_operand(dst, stack_max)),
                ASMInstruction::IDiv(src) => ASMInstruction::IDiv(pseudo_to_stack_operand(src, stack_max)),
                _ => value,
        }
}

fn pseudo_to_stack_operand(value: Operand, stack_max: &mut usize) -> Operand {
        match value {
                Operand::Pseudo(n) => {
                        if *stack_max < n {
                                *stack_max = n;
                        } else if *stack_max == 0 {
                                *stack_max = 1;
                        }

                        Operand::Stack(n)
                }
                _ => value,
        }
}
