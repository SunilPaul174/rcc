use rcc::{initialize::initialize, lex::lex, parse::parse_program, tactile::TACTILE, toasm::asm};

fn main() {
        let res = initialize().unwrap();
        let res = lex(res).unwrap();
        let res = parse_program(res).unwrap();
        let res = TACTILE(res);
        let res = asm(res);
        dbg!(res);
}
