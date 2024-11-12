use std::process::exit;

use rcc::{
        initialize::{initialize, Operation},
        lex::lex,
        parse::parse_program,
        semantic_analysis::analyze,
        tactile::tactile,
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
        let mut res = parse_program(res).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::ParseToCTree {
                return;
        }
        let (res, max_label, variable_map) = analyze(&mut res).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::Validate {
                return;
        }
        let res = tactile(res, max_label, variable_map);
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
