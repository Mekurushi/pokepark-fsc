mod cli;

use clap::Parser;
use cli::{BuildArgs, Cli, Command, PatchArgs};
use fsc_compiler::{CompileRequest, compile};
use fsc_diagnostics::render_diagnostics;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    if run(Cli::parse()).is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run(cli: Cli) -> Result<(), ()> {
    match cli.command {
        Command::Build(args) => build(args),
        Command::Patch(args) => patch(args),
    }
}

fn patch(args: PatchArgs) -> Result<(), ()> {
    let patch_source = fs::read_to_string(&args.patch).map_err(|error| {
        eprintln!(
            "error: could not read patch source {}: {error}",
            args.patch.display()
        );
    })?;
    let original_binary = fs::read(&args.original).map_err(|error| {
        eprintln!(
            "error: could not read original FSB {}: {error}",
            args.original.display()
        );
    })?;
    let symbols_source = fs::read_to_string(&args.symbols).map_err(|error| {
        eprintln!(
            "error: could not read symbol table {}: {error}",
            args.symbols.display()
        );
    })?;
    let symbols = fsc_patcher::parse_symbol_table(&symbols_source).map_err(|error| {
        eprintln!(
            "error: could not parse symbol table {}: {error}",
            args.symbols.display()
        );
    })?;

    let request = fsc_patcher::PatchRequest::new(&patch_source, &original_binary, &symbols);
    let _artifact = fsc_patcher::patch(request).map_err(|error| {
        eprintln!("error: {error}");
    })?;

    Ok(())
}

fn build(args: BuildArgs) -> Result<(), ()> {
    let input = args.input;
    let output = args.output.unwrap_or_else(|| input.with_extension("fsb"));
    let source = fs::read_to_string(&input).map_err(|error| {
        eprintln!("error: could not read {}: {error}", input.display());
    })?;
    let Some(source_name) = input.file_name().and_then(|name| name.to_str()) else {
        eprintln!("error: input path does not have a valid UTF-8 file name");
        return Err(());
    };
    let Some(script_name) = input.file_stem().and_then(|name| name.to_str()) else {
        eprintln!("error: input path does not have a valid UTF-8 file stem");
        return Err(());
    };

    let request = CompileRequest::new(&source, script_name);
    let artifact = compile(request).map_err(|failure| {
        eprint!(
            "{}",
            render_diagnostics(failure.diagnostics(), source_name, &source)
        );
    })?;

    let rendered = render_diagnostics(artifact.diagnostics(), source_name, &source);
    if !rendered.is_empty() {
        eprint!("{rendered}");
    }
    fs::write(&output, artifact.bytes()).map_err(|error| {
        eprintln!("error: could not write {}: {error}", output.display());
    })?;
    eprintln!(
        "compiled {} → {}  ({} bytes)",
        input.display(),
        output.display(),
        artifact.bytes().len(),
    );
    Ok(())
}
