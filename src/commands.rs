use std::env;
use std::io::stdout;
use std::path::PathBuf;
use std::process::{Child, Stdio};

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use crate::directory::Directory;
use crate::ERROR_WRITER;
use crate::project::{print_projects, Project, scan};

#[derive(Subcommand)]
pub enum Commands {
    /// List all Git repositories in directory pointed by default ENV variable (DEV_DIR) or given 'path' (option)
    List,

    /// Execute 'git status' on all repositories for given project 'name'
    Status {
        /// Name of the project to check status (directory with Git repositories,
        /// which exists in DEFAULT_VAR (DEV_DIR)
        /// 'all' can be used to execute command for all projects in DEV_DIR
        name: String,
    },

    /// Execute 'git pull' on all repositories for given project 'name'
    Pull {
        /// Name of the project to pull (directory with Git repositories,
        /// which exists in DEFAULT_VAR (DEV_DIR)
        /// 'all' can be used to execute command for all projects in DEV_DIR
        name: String,
    },
}

pub fn handle_list(path: &PathBuf, writer: impl std::io::Write) -> Result<()> {
    println!("Scanning in path {:?}", path);
    let projects = scan(&path)?;
    print_projects(projects, writer);
    Ok(())
}

pub fn handle_status(path: &PathBuf, name: &String) -> Result<()> {
    execute_git_cmd(path, name, "status")
}

pub fn handle_pull(path: &PathBuf, name: &String) -> Result<()> {
    execute_git_cmd(path, name, "pull")
}

fn execute_git_cmd(path: &PathBuf, name: &String, git_cmd: &str) -> Result<()> {
    let projects = scan(&path)?;

    if "all".eq_ignore_ascii_case(name) {
        projects.iter().for_each(|project| {
            for_project(git_cmd, project, &mut stdout())
        })
    } else {
        let project = projects.iter().find(|p| p.name.eq_ignore_ascii_case(&name))
            .with_context(|| format!("Project with given name '{}' was not found", &name.red()))?;
        for_project(git_cmd, project, &mut stdout());
    }
    Ok(())
}

fn for_project(arg: &str, project: &Project, mut writer: impl std::io::Write) {
    print_project(project, &mut writer);

    if let Some(repos) = &project.repos {
        repos.iter().for_each(|repo| {
            let cmd = std::process::Command::new(by_os())
                .arg(arg)
                .current_dir(repo.path.as_path())
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            print_cmd_out(repo, cmd, &mut writer);
        });
    }
}

fn print_project(project: &Project, mut writer: impl std::io::Write) {
    writeln!(&mut writer, "Project {} found at {:?}", &project.name.bright_green(), &project.path)
        .expect(ERROR_WRITER);
}

fn print_cmd_out(repo: &Directory, cmd: Child, mut writer: impl std::io::Write) {
    let cmd_output = cmd.wait_with_output().unwrap();
    match cmd_output.status.code() {
        Some(0) => writeln!(writer, "{} {}: {}",
                            "=>".bright_green(),
                            repo.name.yellow(),
                            String::from_utf8_lossy(&cmd_output.stdout))
            .expect(ERROR_WRITER),
        Some(code) => writeln!(writer, "{} {}: {} {}",
                               "=>".red(),
                               repo.name.yellow(),
                               "Error".red(), code)
            .expect(ERROR_WRITER),
        None => {}
    }
}

fn by_os() -> String {
    if env::consts::OS.eq("windows") {
        "git.exe".to_string()
    } else {
        "git".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, tempdir_in};

    #[test]
    fn should_print_git_error_128() {
        let temp_dir = tempdir().unwrap();
        let temp_sub_dir = tempdir_in(&temp_dir.path()).unwrap();
        let project = Project {
            name: "Project".to_string(),
            path: temp_dir.into_path(),
            repos: Some(vec![Directory {
                name: "Repo".to_string(),
                path: temp_sub_dir.into_path(),
            }]),
        };

        let mut result = Vec::new();
        for_project("status", &project, &mut result);

        assert_eq!(String::from_utf8_lossy(&result),
                   format!("Project {} found at {:?}\n{} {}: {} 128\n",
                           &project.name.bright_green(),
                           &project.path,
                           "=>".red(),
                           "Repo".yellow(),
                           "Error".red()));
    }

    #[test]
    fn test_execute_git_cmd_project_not_found() {
        let temp_dir = tempdir().unwrap();
        let _temp_sub_dir = tempdir_in(&temp_dir.path()).unwrap();
        let path = temp_dir.path().to_path_buf();
        let name = "nonexistent".to_string();
        let git_cmd = "status";

        let result = execute_git_cmd(&path, &name, git_cmd);
        assert!(result.is_err());
    }
}