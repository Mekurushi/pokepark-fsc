use crate::diagnostic::Diagnostic;
use crate::diagnostic::render::DiagnosticRenderer;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process;

mod ast;
mod diagnostic;
mod lexer;
mod parser;

// just for prototyping
pub fn compile() {
    let input = PathBuf::from("../scripts/add.fs");
    let _output = input.with_extension("fsb");

    let source = match read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading file: {e}");
            process::exit(1);
        }
    };
    let script_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let lex_output = lexer::tokenize(&source);
    let renderer = DiagnosticRenderer::new(&source, &script_name, &lex_output.line_starts);

    for e in lex_output.errors {
        eprint!("{}", renderer.render(&Diagnostic::from(e)));
    }

    let ast = parser::parse(lex_output.tokens, source.clone()).map_err(|e| {
        eprint!("{}", renderer.render(&Diagnostic::from(e)));
    });

    println!("{ast:#?}");
}

#[cfg(test)]
mod emit_tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    #![allow(clippy::panic)]

    use crate::compile;

    // temporary test for prototyping
    #[test]
    fn end_to_end() {
        compile();
    }
}
