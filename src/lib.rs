pub mod commands;
pub mod directory;
pub mod project;
pub mod git;

pub const DEFAULT_DEV_DIR_VAR: &'static str = "DEV_DIR";
pub const ERROR_WRITER: &'static str = "Failed to write to the output!";