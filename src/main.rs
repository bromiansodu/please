use clap::{Parser};

use please::commands::Commands;
use please::list::handle_list;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Override the default DEV_DIR environmental variable,
    /// which points to a folder with all projects / Git repositories
    #[arg(short, long)]
    default_var: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List { name, path }) =>
            handle_list(&cli.default_var, name, path),
        None =>
            println!("No command given. Use with --help or -h to see available commands and options")
    }
}
