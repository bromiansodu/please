use std::env;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use clap::Parser;
use colored::Colorize;

use please::commands::{handle_clean, handle_list, handle_pull, handle_status, Commands};
use please::DEFAULT_DEV_DIR_VAR;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Override the default DEV_DIR environmental variable,
    /// which points to a folder with all projects / Git repositories
    #[arg(short, long)]
    override_default: Option<String>,

    /// Instead of using environmental variable, specify a path to a directory with Git repositories
    /// This option has a higher precedence than 'override_default'
    #[arg(short, long)]
    path: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    resolve_path(&cli.override_default, &cli.path).and_then(|path| match &cli.command {
        Some(Commands::List) => handle_list(&path, &mut std::io::stdout()),
        Some(Commands::Status { name }) => handle_status(&path, name),
        Some(Commands::Pull { name }) => handle_pull(&path, name),
        Some(Commands::Clean) => handle_clean(),
        None => {
            println!(
                "No command given. Use with --help or -h to see available commands and options"
            );
            Ok(())
        }
    })
}

fn resolve_path(
    override_default: &Option<String>,
    path_arg: &Option<PathBuf>,
) -> Result<PathBuf, Error> {
    match path_arg {
        Some(p) => Ok(p.clone()),
        None => match override_default {
            Some(var) => {
                let val =
                    env::var(var).with_context(|| format!("{} is not defined!", var.red()))?;
                Ok(PathBuf::from(val))
            }
            None => {
                let val = env::var(DEFAULT_DEV_DIR_VAR)
                    .with_context(|| format!("{} is not defined!", DEFAULT_DEV_DIR_VAR.red()))?;
                Ok(PathBuf::from(val))
            }
        },
    }
}
