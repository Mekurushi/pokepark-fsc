use fsc_asm::ast_walker::AstWalker;
use fsc_asm::{lexer, parser};
use fsc_assembler::Assembler;
use std::fs::{read_to_string, write};
use std::path::PathBuf;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fsc <file.fsa> [output.fsb]");
        process::exit(1);
    }

    let input = PathBuf::from(&args[1]);

    match input.extension().and_then(|e| e.to_str()) {
        Some("fsa") => {}
        Some(ext) => {
            eprintln!("Error: expected a .fsa file, got .{ext}");
            process::exit(1);
        }
        None => {
            eprintln!("Error: file has no extension — expected .fsa");
            process::exit(1);
        }
    }

    if !input.exists() {
        eprintln!("Error: file not found: {}", input.display());
        process::exit(1);
    }
    if !input.is_file() {
        eprintln!("Error: path is not a file: {}", input.display());
        process::exit(1);
    }

    let output = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        input.with_extension("fsb")
    };

    let script_name = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

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
    let mut core = Assembler::new();
    let mut walker = AstWalker::new(&mut core);

    match walker.walk(&program) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("assembler error: {e}");
            process::exit(1);
        }
    }

    let binary = match core.finalize(script_name) {
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

    write(output, bytes)?;
    Ok(())
}
