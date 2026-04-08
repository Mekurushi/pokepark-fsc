use fsc_assembler::Assembler;
use fsc_codegen::check::check_func;
use fsc_codegen::lower;
use fsc_parse::diagnostic::render::DiagnosticRenderer;
use fsc_parse::diagnostic::Diagnostic;
use fsc_parse::lexer;
use fsc_parse::parser;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::process;

fn main() {
    let input = PathBuf::from("scripts/add.fs");
    let output = input.with_extension("fsb");

    let source = match read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading file: {e}");
            process::exit(1);
        }
    };
    let file_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let lex_output = lexer::tokenize(&source);
    let renderer = DiagnosticRenderer::new(&source, &file_name, &lex_output.line_starts);

    for e in lex_output.errors {
        eprint!("{}", renderer.render(&Diagnostic::from(e)));
    }

    let ast = parser::parse(lex_output.tokens, source.clone()).map_err(|e| {
        eprint!("{}", renderer.render(&Diagnostic::from(e)));
    });

    let mut asm = Assembler::new();

    match ast {
        Ok(ast) => {
            for ast in ast.iter() {
                println!("{:?}", check_func(&ast));
                match lower::lower_func(ast, &mut asm) {
                    Ok(_) => {}
                    Err(e) => {}
                }
            }
        }
        Err(_) => todo!(),
    }
    let binary = match asm.finalize(
        input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
    ) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("binary error: {e}");
            process::exit(1);
        }
    };

    let bytes = match binary.serialize() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("serialize error: {e}");
            process::exit(1);
        }
    };

    write(output, bytes);
}
