use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "fsc",
    version,
    about = "Compile PokéPark FSC source files",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Compile an FSC source file")]
    Build(BuildArgs),

    #[command(about = "Patch functions in an existing FSB file")]
    Patch(PatchArgs),
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct PatchArgs {
    #[arg(value_name = "PATCH")]
    pub patch: PathBuf,

    #[arg(value_name = "ORIGINAL")]
    pub original: PathBuf,

    #[arg(long, value_name = "SYMBOLS.toml", required = true)]
    pub symbols: PathBuf,

    #[arg(short, long, value_name = "OUTPUT", required = true)]
    pub output: PathBuf,

    #[arg(long, value_name = "SYMBOLS.toml", required = true)]
    pub symbols_output: PathBuf,
}
