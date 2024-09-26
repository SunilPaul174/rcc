use crate::{
        codegen::astsm::{ASMFunction, ASMIdentifier, ASMProgram, Imm, Operand, Register},
        ASMASTGenerated, Compiled, Program,
};

impl Program<ASMASTGenerated> {
        pub fn compile(self) -> Program<Compiled> {
                let mut machine_code_out: Vec<u8> = vec![];

                let ASMProgram {
                        function:
                                ASMFunction {
                                        identifier: ASMIdentifier { start, len },
                                        instructions,
                                },
                } = self.state.asm_program;

                let identifier = &self.state.pre_processor_output[start..start + len];
                let mut instructions_out = vec![];
                for i in instructions {
                        let (Operand::Imm(Imm { len, start }), Operand::Register(Register)) = i.mov else {
                                todo!();
                        };
                        let slice = &self.state.pre_processor_output[start..start + len];

                        instructions_out.push(b'\t');
                        instructions_out.extend([b"movl $", slice, b", %eax\nret"].concat());
                }

                machine_code_out.extend([
                        br#".globl "#,
                        identifier,
                        &[b'\n'],
                        identifier,
                        &[b':'],
                        &instructions_out[..],
                        &[b'\n'],
                ]
                .concat());

                machine_code_out.extend(br#".section  .note.GNU-stack, "", @progbits"#);

                Program {
                        state: Compiled {
                                code_generated: machine_code_out,
                        },
                        ..self
                }
        }
}
