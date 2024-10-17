use rcc::{initialize::initialize, lex::lex, parse::parse_program, toasm::asm, write::gen_asm};

fn main() {
        let res = initialize().unwrap();
        println!("{:#?}", res);
        let res = lex(res).unwrap();
        println!("{:#?}", res);
        let res = parse_program(res).unwrap();
        println!("{:#?}", res);
        let res = asm(res);
        println!("{:#?}", res);
}
