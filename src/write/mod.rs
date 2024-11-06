use crate::{
        parse::nodes::{AConstant, AIdentifier, Unop},
        tactile::Constant,
        toasm::{
                nodes::{ASMBinary, ASMFunction, ASMInstruction, CondCode, Operand, Register},
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
pub static AX: &[u8] = b"%eax";

pub static R10D: &[u8] = b"%r10d";
pub static R10B: &[u8] = b"%r10b";

pub static DX: &[u8] = b"%edx";
pub static DL: &[u8] = b"%dl";

pub static R11D: &[u8] = b"%r11d";
pub static R11B: &[u8] = b"%r11b";

pub static PERCENT: u8 = b'%';
pub static DOLLAR: u8 = b'$';

pub static NOTL: &[u8] = b"\tnotl ";
pub static NEGL: &[u8] = b"\tnegl ";
pub static ADDL: &[u8] = b"\taddl ";
pub static SUBL: &[u8] = b"\tsubl ";
pub static IMULL: &[u8] = b"\timull ";
pub static LEFTSHIFTL: &[u8] = b"\tshll ";
pub static RIGHTSHIFTL: &[u8] = b"\tshrl ";
pub static ANDL: &[u8] = b"\tandl ";
pub static ORL: &[u8] = b"\torl ";
pub static XORL: &[u8] = b"\txorl ";
pub static CMPL: &[u8] = b"\tcmpl ";
pub static JMP: &[u8] = b"\tjmp ";
pub static INC: &[u8] = b"\tinc ";
pub static DEC: &[u8] = b"\tdec ";

pub static DIVL: &[u8] = b"\tidivl ";

pub static CDQ: &[u8] = b"\tcdq\n";

pub static TEARDOWN: &[u8] = b"\tmovq %rbp, %rsp\n\tpopq %rbp\n\tret\n";

// we hold the stack in the tactile stage as the number of variables on the stack. However, the all the variables are QWords, so you need to subtract from top of stack
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
fn the_real_stack(val: usize) -> i32 {
        -((val as i32) * 4)
}

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

        let extend_from_operand = |value, instructions: &mut Vec<u8>, setcc: bool| match value {
                Operand::Imm(Constant::A(AConstant { start, len })) => {
                        instructions.push(DOLLAR);
                        instructions.extend_from_slice(&code[start..start + len]);
                }
                Operand::Imm(Constant::S(n)) => {
                        instructions.push(DOLLAR);
                        instructions.extend_from_slice(&n.to_string().into_bytes());
                }
                Operand::Register(register) => instructions.extend(match (register, setcc) {
                        (Register::AX, false) => EAX,
                        (Register::R10, false) => R10D,
                        (Register::DX, _) => DX,
                        (Register::R11, false) => R11D,
                        (Register::AX, true) => AX,
                        (Register::R10, true) => R10B,
                        (Register::R11, true) => R11B,
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

fn instruction_to_extension(i: ASMInstruction, instructions: &mut Vec<u8>, extend_from_operand: impl Fn(Operand, &mut Vec<u8>, bool)) {
        match i {
                ASMInstruction::Mov(src, dst) => {
                        instructions.extend_from_slice(b"\tmovl ");
                        extend_from_operand(src, instructions, false);
                        instructions.push(b',');
                        extend_from_operand(dst, instructions, false);
                        instructions.push(b'\n');
                }
                ASMInstruction::Unary(unop, operand) => {
                        let op = match unop {
                                Unop::Negate => NEGL,
                                Unop::Complement => NOTL,
                                Unop::IncrementPre => INC,
                                Unop::IncrementPost => INC,
                                Unop::DecrementPre => DEC,
                                Unop::DecrementPost => DEC,
                                Unop::Not => panic!("not possible; removed in tactile->abstractasm stage."),
                        };
                        instructions.extend_from_slice(op);
                        extend_from_operand(operand, instructions, false);
                        instructions.push(b'\n');
                }
                ASMInstruction::AllocateStack(n) => {
                        if n == 0 {
                                return;
                        }
                        instructions.extend(b"\tpushq %rbp\n\tmovq %rsp, %rbp\n\tsubq $");
                        instructions.extend_from_slice(&(-the_real_stack(n)).to_string().into_bytes());
                        instructions.extend(b", %rsp\n");
                }
                ASMInstruction::Ret => instructions.extend_from_slice(TEARDOWN),
                ASMInstruction::Binary(asmbinary, src, dst) => {
                        instructions.extend(match asmbinary {
                                ASMBinary::Add => ADDL,
                                ASMBinary::Subtract => SUBL,
                                ASMBinary::Multiply => IMULL,
                                ASMBinary::LeftShift => LEFTSHIFTL,
                                ASMBinary::RightShift => RIGHTSHIFTL,
                                ASMBinary::Or => ORL,
                                ASMBinary::XOr => XORL,
                                ASMBinary::And => ANDL,
                                ASMBinary::AddAssign => ADDL,
                                ASMBinary::SubtractAssign => SUBL,
                                ASMBinary::MultiplyAssign => IMULL,
                                ASMBinary::LeftShiftAssign => LEFTSHIFTL,
                                ASMBinary::RightShiftAssign => RIGHTSHIFTL,
                                ASMBinary::BitwiseAndAssign => ANDL,
                                ASMBinary::BitwiseOrAssign => ORL,
                                ASMBinary::BitwiseXOrAssign => XORL,
                        });
                        extend_from_operand(src, instructions, false);
                        instructions.push(b',');
                        extend_from_operand(dst, instructions, false);
                        instructions.push(b'\n');
                }
                ASMInstruction::IDiv(operand) => {
                        instructions.extend_from_slice(DIVL);
                        extend_from_operand(operand, instructions, false);
                        instructions.push(b'\n');
                }
                ASMInstruction::Cdq => instructions.extend_from_slice(CDQ),
                ASMInstruction::Cmp(op1, op2) => {
                        instructions.extend_from_slice(CMPL);
                        extend_from_operand(op1, instructions, false);
                        instructions.push(b',');
                        extend_from_operand(op2, instructions, false);
                        instructions.push(b'\n');
                }
                ASMInstruction::Jmp(label) => {
                        instructions.extend_from_slice(JMP);
                        instructions.extend_from_slice(b".L");
                        instructions.extend_from_slice(&label.0.to_string().into_bytes());
                }
                ASMInstruction::JmpCC(cond_code, label) => {
                        instructions.extend_from_slice(b"\tj");
                        instructions.extend_from_slice(cond_code_to_slice(cond_code));
                        instructions.extend_from_slice(b".L");
                        instructions.extend_from_slice(&label.0.to_string().into_bytes());
                        instructions.push(b'\n');
                }
                ASMInstruction::SetCC(cond_code, op1) => {
                        instructions.extend_from_slice(b"\tset");
                        instructions.extend_from_slice(cond_code_to_slice(cond_code));
                        extend_from_operand(op1, instructions, true);
                        instructions.push(b'\n');
                }
                ASMInstruction::Label(label) => {
                        instructions.extend_from_slice(b"\n.L");
                        instructions.extend_from_slice(&label.0.to_string().into_bytes());
                        instructions.extend_from_slice(b":\n");
                }
        }
}

fn cond_code_to_slice(cond_code: CondCode) -> &'static [u8] {
        match cond_code {
                CondCode::E => b"e ",
                CondCode::NE => b"ne ",
                CondCode::G => b"g ",
                CondCode::GE => b"ge ",
                CondCode::L => b"l ",
                CondCode::LE => b"le ",
        }
}
