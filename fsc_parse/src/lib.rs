mod ast;
mod lexer;
mod parser;

#[cfg(test)]
mod emit_tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::panic)]

    use crate::{lexer, parser};
    use std::fs::read_to_string;
    use std::path::PathBuf;
    use std::process;

    // temporary test for prototyping
    #[test]
    fn end_to_end() {
        let input = PathBuf::from("../scripts/add.fs");
        let _output = input.with_extension("fsb");

        let source = match read_to_string(&input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading file: {e}");
                process::exit(1);
            }
        };

        let tokens = match lexer::tokenize(&source) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("lex error: {e}");
                process::exit(1);
            }
        };

        let program = match parser::parse(tokens) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("parse error: {e}");
                process::exit(1);
            }
        };

        println!("{program:#?}");
    }
}
