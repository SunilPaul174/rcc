use crate::{
        parse::nodes::{BinOp, Unop},
        tactile::{Constant, TACTILEFunction, TACTILEInstruction, Value, TACTILE},
        Program, State,
};
use nodes::{from_binop, ASMBinary, ASMFunction, ASMInstruction, ASMProgram, CondCode, Operand, Register};

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

        let function = ASMFunction::from(aprogram.function);

        Program {
                operation: program.operation,
                state: Compiled {
                        code,
                        program: ASMProgram { function },
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

                let from_tactile = |&value| match value {
                        TACTILEInstruction::Return(val) => {
                                temp_instructions.extend([ASMInstruction::Mov(val_to_op(val), Operand::Register(Register::AX)), ASMInstruction::Ret])
                        }
                        TACTILEInstruction::Unary(unop, src, dst) => {
                                match unop {
                                        Unop::Not => {
                                                temp_instructions.extend([
                                                        ASMInstruction::Cmp(Operand::Imm(Constant::S(0)), val_to_op(src)),
                                                        ASMInstruction::Cmp(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                                        ASMInstruction::SetCC(CondCode::E, val_to_op(dst)),
                                                ]);
                                                return;
                                        }
                                        _ => {}
                                }
                                temp_instructions.extend([ASMInstruction::Mov(val_to_op(src), val_to_op(dst)), ASMInstruction::Unary(unop, val_to_op(dst))])
                        }

                        TACTILEInstruction::Binary(binop, src1, src2, dst) => match binop {
                                BinOp::Divide => temp_instructions.extend([
                                        ASMInstruction::Mov(val_to_op(src1), Operand::Register(Register::AX)),
                                        ASMInstruction::Cdq,
                                        ASMInstruction::IDiv(val_to_op(src2)),
                                        ASMInstruction::Mov(Operand::Register(Register::AX), val_to_op(dst)),
                                ]),
                                BinOp::Remainder => temp_instructions.extend([
                                        ASMInstruction::Mov(val_to_op(src1), Operand::Register(Register::AX)),
                                        ASMInstruction::Cdq,
                                        ASMInstruction::IDiv(val_to_op(src2)),
                                        ASMInstruction::Mov(Operand::Register(Register::DX), val_to_op(dst)),
                                ]),
                                BinOp::MoreThan => temp_instructions.extend([
                                        ASMInstruction::Cmp(val_to_op(src2), val_to_op(dst)),
                                        ASMInstruction::Mov(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                        ASMInstruction::SetCC(CondCode::G, val_to_op(dst)),
                                ]),
                                BinOp::MoreThanOrEqual => temp_instructions.extend([
                                        ASMInstruction::Cmp(val_to_op(src2), val_to_op(dst)),
                                        ASMInstruction::Mov(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                        ASMInstruction::SetCC(CondCode::GE, val_to_op(dst)),
                                ]),
                                BinOp::EqualTo => temp_instructions.extend([
                                        ASMInstruction::Cmp(val_to_op(src2), val_to_op(dst)),
                                        ASMInstruction::Mov(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                        ASMInstruction::SetCC(CondCode::E, val_to_op(dst)),
                                ]),
                                BinOp::NotEqualTo => temp_instructions.extend([
                                        ASMInstruction::Cmp(val_to_op(src2), val_to_op(dst)),
                                        ASMInstruction::Mov(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                        ASMInstruction::SetCC(CondCode::NE, val_to_op(dst)),
                                ]),
                                BinOp::LessThan => temp_instructions.extend([
                                        ASMInstruction::Cmp(val_to_op(src2), val_to_op(dst)),
                                        ASMInstruction::Mov(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                        ASMInstruction::SetCC(CondCode::L, val_to_op(dst)),
                                ]),
                                BinOp::LessThanOrEqual => temp_instructions.extend([
                                        ASMInstruction::Cmp(val_to_op(src2), val_to_op(dst)),
                                        ASMInstruction::Mov(Operand::Imm(Constant::S(0)), val_to_op(dst)),
                                        ASMInstruction::SetCC(CondCode::LE, val_to_op(dst)),
                                ]),

                                _ => temp_instructions.extend([
                                        ASMInstruction::Mov(val_to_op(src1), val_to_op(dst)),
                                        ASMInstruction::Binary(from_binop(binop).unwrap(), val_to_op(src2), val_to_op(dst)),
                                ]),
                        },
                        TACTILEInstruction::Jump(label) => temp_instructions.push(ASMInstruction::Jmp(label)),
                        TACTILEInstruction::Copy(src, dst) => temp_instructions.push(ASMInstruction::Mov(val_to_op(src), val_to_op(dst))),
                        TACTILEInstruction::Label(label) => temp_instructions.push(ASMInstruction::Label(label)),
                        TACTILEInstruction::JumpIfZero(value, label) => temp_instructions.extend([
                                ASMInstruction::Cmp(Operand::Imm(Constant::S(0)), val_to_op(value)),
                                ASMInstruction::JmpCC(CondCode::E, label),
                        ]),
                        TACTILEInstruction::JumpIfNotZero(value, label) => temp_instructions.extend([
                                ASMInstruction::Cmp(Operand::Imm(Constant::S(0)), val_to_op(value)),
                                ASMInstruction::JmpCC(CondCode::NE, label),
                        ]),
                };

                let mut stack_max: usize = 0;

                let _: Vec<()> = value.instructions.iter().map(from_tactile).collect();

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
                ASMInstruction::Cmp(Operand::Stack(op1), Operand::Stack(op2)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Stack(op1), Operand::Register(Register::R10)));
                        instructions.push(ASMInstruction::Cmp(Operand::Register(Register::R10), Operand::Stack(op2)));
                }
                ASMInstruction::Cmp(op1, Operand::Imm(op2)) => {
                        instructions.push(ASMInstruction::Mov(Operand::Imm(op2), Operand::Register(Register::R11)));
                        instructions.push(ASMInstruction::Cmp(op1, Operand::Register(Register::R11)));
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
                ASMInstruction::Binary(binop, src, dst) => {
                        ASMInstruction::Binary(binop, pseudo_to_stack_operand(src, stack_max), pseudo_to_stack_operand(dst, stack_max))
                }
                ASMInstruction::IDiv(src) => ASMInstruction::IDiv(pseudo_to_stack_operand(src, stack_max)),
                _ => value,
        }
}

fn pseudo_to_stack_operand(value: Operand, stack_max: &mut usize) -> Operand {
        match value {
                Operand::Pseudo(n) => {
                        if *stack_max < n {
                                *stack_max = n;
                        }
                        Operand::Stack(n)
                }
                _ => value,
        }
}
