use std::process::exit;

use rcc::{
        initialize::{initialize, Operation},
        lex::lex,
        parse::parse_program,
        semanalysis::analyze,
        tactile::TACTILE,
        toasm::asm,
        write::write,
};

fn main() {
        let res = initialize().unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        let res = lex(res).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::Lex {
                return;
        }
        let res = dbg!(parse_program(res)).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::ParseToCTree {
                return;
        }
        let res = analyze(res).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::Validate {
                return;
        }
        let res = TACTILE(res);
        if res.operation == Operation::ParseToTACTILETree {
                return;
        }
        let res = asm(res);
        if res.operation == Operation::ParseToASMTree {
                return;
        }
        let res = write(res);
        println!("{}", String::from_utf8(res.state.code).unwrap());
}
