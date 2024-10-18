use rcc::{initialize::initialize, lex::lex, parse::parse_program, toasm::asm};

fn main() {
        let res = initialize().unwrap();
        let res = lex(res).unwrap();
        let res = parse_program(res).unwrap();
        let _ = asm(res);
}
