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

    #[arg(
        long,
        value_name = "ADDRESS",
        required = true,
        value_parser = parse_hex_address
    )]
    pub base_address: u32,

    #[arg(short, long, value_name = "OUTPUT", required = true)]
    pub output: PathBuf,
}

// TODO: check if this is really necessary only to strip prefixes
fn parse_hex_address(value: &str) -> Result<u32, String> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .ok_or_else(|| "address must start with `0x`".to_owned())?;

    u32::from_str_radix(digits, 16).map_err(|_| format!("invalid hexadecimal address `{value}`"))
}
