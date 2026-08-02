mod cli;

use clap::Parser;
use cli::{BuildArgs, Cli, Command};
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
    }
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
