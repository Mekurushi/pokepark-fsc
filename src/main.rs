mod lexer;

use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: fsc <file.fsc> [output.fsb]");
        process::exit(1);
    }

    let input = PathBuf::from(&args[1]);

    match input.extension().and_then(|e| e.to_str()) {
        Some("fsc") => {}
        Some(ext) => {
            eprintln!("Error: expected a .fsc file, got .{ext}");
            process::exit(1);
        }
        None => {
            eprintln!("Error: file has no extension — expected .fsc");
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

    let _output = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        input.with_extension("fsb")
    };

    // TODO: connecting assembling
}
