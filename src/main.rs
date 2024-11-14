use std::process::exit;

use rcc::{
        initialize::{initialize, Operation},
        lex::lex,
        parse::parse_program,
        semantic_analysis::analyze,
        tactile::tactile,
        toasm::asm,
        write::write,
        Program,
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
        let res = parse_program(res).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::ParseToCTree {
                return;
        }
        let code = res.state.code;
        let (semanal, max_label, variable_map) = analyze(res.state.program, &code).unwrap_or_else(|f| {
                eprintln!("{f}");
                exit(1);
        });
        if res.operation == Operation::Validate {
                return;
        }

        let res = Program {
                operation: res.operation,
                state: semanal,
        };

        let res = tactile(res, max_label, variable_map, &code);
        if res.operation == Operation::ParseToTACTILETree {
                return;
        }
        let asm = asm(res.state);
        if res.operation == Operation::ParseToASMTree {
                return;
        }
        let res = Program {
                operation: res.operation,
                state: asm,
        };
        let code = write(res.state, &code).code;
        println!("{}", String::from_utf8(code).unwrap());
}
