use crate::{
        parse::nodes::{AConstant, AIdentifier, Unop},
        toasm::{
                nodes::{ASMBinary, ASMFunction, ASMInstruction, Operand, Register},
                Compiled,
        },
        Program, State,
};

#[derive(Debug, Clone)]
pub struct Written {
        pub code: Vec<u8>,
}
impl State for Written {}
pub fn write(program: Program<Compiled>) -> Program<Written> {
        let code = func_to_vec(program.state.program.function, &program.state.code);

        Program {
                operation: program.operation,
                state: Written { code },
        }
}

pub static EAX: &[u8] = b"%eax";
pub static R10D: &[u8] = b"%r10d";
pub static DX: &[u8] = b"%edx";
pub static R11D: &[u8] = b"%r11d";

pub static PERCENT: u8 = b'%';
pub static DOLLAR: u8 = b'$';

pub static NOT: &[u8; 5] = b"\tnotl";
pub static NEG: &[u8; 5] = b"\tnegl";
pub static ADD: &[u8] = b"\taddl";
pub static SUB: &[u8] = b"\tsubl";
pub static MUL: &[u8] = b"\timull";
pub static DIV: &[u8; 6] = b"\tidivl";

pub static CDQ: &[u8; 5] = b"\tcdq\n";

fn the_real_stack(val: usize) -> i32 { -((val as i32) * 4) }

fn func_to_vec(function: ASMFunction, code: &[u8]) -> Vec<u8> {
        let mut instructions = Vec::new();

        instructions.extend_from_slice(b"\t.globl ");

        let AIdentifier { start, len } = function.identifier;
        let identifier = &code[start..start + len];
        instructions.extend_from_slice(identifier);
        instructions.push(b'\n');
        instructions.extend_from_slice(identifier);
        instructions.push(b':');
        instructions.push(b'\n');

        let extend_from_operand = |value, instructions: &mut Vec<u8>| match value {
                Operand::Imm(AConstant { start, len }) => {
                        instructions.push(DOLLAR);
                        instructions.extend_from_slice(&code[start..start + len]);
                }
                Operand::Register(register) => instructions.extend(match register {
                        Register::AX => EAX,
                        Register::R10 => R10D,
                        Register::DX => DX,
                        Register::R11 => R11D,
                }),
                Operand::Stack(stack_value) => {
                        let val = the_real_stack(stack_value).to_string().into_bytes();
                        instructions.extend(val);
                        instructions.extend_from_slice(b"(%rbp)");
                }
                Operand::Pseudo(_) => todo!(),
        };

        for i in function.instructions {
                instruction_to_extension(i, &mut instructions, extend_from_operand);
        }

        instructions.extend_from_slice(b".section .note.GNU-stack,\"\",@progbits");

        instructions
}

fn instruction_to_extension(i: ASMInstruction, instructions: &mut Vec<u8>, extend_from_operand: impl Fn(Operand, &mut Vec<u8>)) {
        match i {
                ASMInstruction::Mov(src, dst) => {
                        instructions.extend_from_slice(b"\tmovl ");
                        extend_from_operand(src, instructions);
                        instructions.push(b',');
                        extend_from_operand(dst, instructions);
                        instructions.push(b'\n');
                }
                ASMInstruction::Unary(unop, operand) => {
                        instructions.extend_from_slice(match unop {
                                Unop::Negate => NEG,
                                Unop::Complement => NOT,
                        });
                        instructions.push(b' ');
                        extend_from_operand(operand, instructions);
                        instructions.push(b'\n');
                }
                ASMInstruction::AllocateStack(n) => {
                        instructions.extend(b"\tpushq %rbp\n\tmovq %rsp, %rbp\n\tsubq $");
                        instructions.extend_from_slice(&(-the_real_stack(n)).to_string().into_bytes());
                        instructions.extend(b", %rsp\n");
                }
                ASMInstruction::Ret => instructions.extend_from_slice(b"\tmovq %rbp, %rsp\n\tpopq %rbp\n\tret\n"),
                ASMInstruction::Binary(asmbinary, src, dst) => {
                        instructions.extend(match asmbinary {
                                ASMBinary::Add => ADD,
                                ASMBinary::Subtract => SUB,
                                ASMBinary::Multiply => MUL,
                        });
                        instructions.push(b' ');
                        extend_from_operand(src, instructions);
                        instructions.push(b',');
                        extend_from_operand(dst, instructions);
                        instructions.push(b'\n');
                }
                ASMInstruction::IDiv(operand) => {
                        instructions.extend_from_slice(DIV);
                        instructions.push(b' ');
                        extend_from_operand(operand, instructions);
                        instructions.push(b'\n');
                }
                ASMInstruction::Cdq => instructions.extend_from_slice(CDQ),
        }
}
