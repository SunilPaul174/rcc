use crate::{
        parse::nodes::{AConstant, AIdentifier},
        toasm::{
                nodes::{ASMInstruction, Imm, Mov, Operand, Register},
                Compiled,
        },
        Program, State,
};

#[derive(Debug)]
pub struct ASM {
        pub code: Vec<u8>,
}
impl State for ASM {}

pub fn gen_asm(program: Program<Compiled>) -> Program<ASM> {
        let mut code = vec![];

        for i in program.state.program.functions {
                let AIdentifier { start, len } = i.identifier;

                let identifier = &program.state.code[start..start + len];
                code.extend_from_slice(&[b"\t.globl ", identifier, b"\n", identifier, b":\n"].concat());

                let instructions = i.instructions;

                for j in instructions {
                        code.extend_from_slice(&instruction_to_vec(&program.state.code, j));
                }
        }

        code.extend_from_slice(b".section .note.GNU-stack,\"\",@progbits");

        Program {
                operation: program.operation,
                state: ASM { code },
        }
}

fn instruction_to_vec(code: &[u8], instruction: ASMInstruction) -> Vec<u8> {
        match instruction {
                ASMInstruction::Mov(Mov { src, dest }) => {
                        let src = operand_to_slice(&code, src);
                        let src = [&[src.0], src.1].concat();

                        let dest = operand_to_slice(&code, dest);
                        let dest = [&[dest.0], dest.1].concat();

                        [b"\tmovl ", &src[..], b", ", &dest[..], b"\n"].concat()
                }
                ASMInstruction::Ret => vec![b'\t', b'r', b'e', b't', b'\n'],
        }
}

pub static DOLLAR: u8 = b'$';
pub static PERCENT: u8 = b'%';

fn operand_to_slice<'a>(code: &'a [u8], operand: Operand) -> (u8, &'a [u8]) {
        match operand {
                Operand::Imm(imm) => {
                        let Imm(AConstant { start, len }) = imm;
                        (DOLLAR, &code[start..start + len])
                }
                Operand::Register(reg) => match reg {
                        Register::EAX => (PERCENT, b"eax"),
                },
        }
}
