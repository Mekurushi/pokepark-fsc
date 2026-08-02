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
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,
}
