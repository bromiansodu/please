use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// List all Git repositories in default ENV variable (DEV_DIR) or given PATH
    List {
        /// Name of the project to pull (directory with Git repositories,
        /// which exists in DEFAULT_VAR (DEV_DIR
        name: Option<String>,

        /// Alternatively a path to a directory with Git repositories
        #[arg(short, long)]
        path: Option<String>
    }
}