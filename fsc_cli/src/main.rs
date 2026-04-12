use fsc_assembler::Assembler;
use fsc_parse::diagnostic::Diagnostic;
use fsc_parse::diagnostic::render::DiagnosticRenderer;
use fsc_parse::lexer;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("build") => {
            let (input, output) = parse_build_args(&args);
            match compile(&input) {
                Ok(bytes) => {
                    if let Err(e) = fs::write(&output, &bytes) {
                        eprintln!("error: could not write {}: {e}", output.display());
                        process::exit(1);
                    }
                    eprintln!(
                        "compiled {} → {}  ({} bytes)",
                        input.display(),
                        output.display(),
                        bytes.len(),
                    );
                }
                Err(()) => process::exit(1),
            }
        }

        Some(cmd) => {
            eprintln!("error: unknown command `{cmd}`");
            print_usage();
            process::exit(1);
        }

        None => {
            print_usage();
            process::exit(1);
        }
    }
}

fn parse_build_args(args: &[String]) -> (PathBuf, PathBuf) {
    let input = if let Some(p) = args.get(2) {
        PathBuf::from(p)
    } else {
        eprintln!("error: `fsc build` requires an input file");
        eprintln!("usage: fsc build <input.fs> [-o <output.fsb>]");
        process::exit(1);
    };

    let output = if let Some(o_flag) = args.get(3) {
        if o_flag == "-o" {
            if let Some(p) = args.get(4) {
                PathBuf::from(p)
            } else {
                eprintln!("error: `-o` requires an output path");
                process::exit(1);
            }
        } else {
            input.with_extension("fsb")
        }
    } else {
        input.with_extension("fsb")
    };

    (input, output)
}

fn compile(input: &Path) -> Result<Vec<u8>, ()> {
    let source = fs::read_to_string(input).map_err(|e| {
        eprintln!("error: could not read {}: {e}", input.display());
    })?;

    let Some(file_name) = input.file_name().and_then(|s| s.to_str()) else {
        eprintln!("error: could not find file name");

        return Err(());
    };

    let Some(script_name) = input.file_stem().and_then(|s| s.to_str()) else {
        eprintln!("error: could not find file name");
        return Err(());
    };

    let lex_output = lexer::tokenize(&source);
    let renderer = DiagnosticRenderer::new(&source, file_name, &lex_output.line_starts);

    // --- parse ---
    let script = match fsc_parse::parse(&source) {
        Ok(s) => s,
        Err(e) => {
            eprint!("{}", renderer.render(&Diagnostic::from(e)));
            return Err(());
        }
    };

    // --- sema ---
    let hir = match fsc_sema::analyze(&script) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("error: {e}");
            return Err(());
        }
    };

    // --- codegen ---
    let mut asm = Assembler::new();
    if let Err(e) = fsc_codegen::compile(&hir, &mut asm) {
        eprintln!("error: {e}");
        return Err(());
    }

    // --- finalize ---
    let binary = asm.finalize(script_name.to_string()).map_err(|e| {
        eprintln!("error: {e}");
    })?;

    binary.serialize().map_err(|e| {
        eprintln!("error: {e}");
    })
}

fn print_usage() {
    eprintln!("usage: fsc <command> [options]");
    eprintln!();
    eprintln!("commands:");
    eprintln!("  build <input.fs> [-o <output.fsb>]   compile a script");
}
