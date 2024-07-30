use std::env;
use std::process::Stdio;

use anyhow::{Context, Result};
use clap::Subcommand;

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

pub fn handle_list(path_string: String) -> Result<()> {
    println!("Scanning in path {path_string}");
    // { //debug
    //     let paths = fs::read_dir(p)
    //         .with_context(|| format!("Failed to read given path: {p}"))?;
    //     for path in paths {
    //         println!("Name: {}", path.unwrap().path().display());
    //     }
    // }
    let projects = scan(&path_string)?;
    print_projects(projects);
    Ok(())
}

pub fn handle_status(path_string: String, name: &String) -> Result<()> {
    execute_git_cmd(path_string, name, "status")
}

pub fn handle_pull(path_string: String, name: &String) -> Result<()> {
    execute_git_cmd(path_string, name, "pull")
}

fn execute_git_cmd(path_string: String, name: &String, git_cmd: &str) -> Result<()> {
    let projects = scan(&path_string)?;

    if "all".eq_ignore_ascii_case(name) {
        projects.iter().for_each(|project| {
            for_project(git_cmd, project)
        })
    } else {
        let project = projects.iter().find(|p| p.name.eq_ignore_ascii_case(&name))
            .with_context(|| format!("Project with given name '{}' was not found", &name))?;
        println!("Project {} found at {:?}", &project.name, &project.path);

        for_project(git_cmd, project);
    }
    Ok(())
}

fn for_project(arg: &str, project: &Project) {
    if let Some(repos) = &project.repos {
        repos.iter().for_each(|repo| {
            let cmd = std::process::Command::new(by_os())
                .arg(arg)
                .current_dir(repo.path.as_path())
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let cmd_output = cmd.wait_with_output().unwrap();
            match cmd_output.status.code() {
                Some(0) => println!("[{}]: {}", repo.name, String::from_utf8_lossy(&cmd_output.stdout)),
                Some(code) => println!("[{}] Error {}", repo.name, code),
                None => {}
            }
        });
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
    fn test_execute_git_cmd_project_not_found() {
        let temp_dir = tempdir().unwrap();
        let _temp_sub_dir = tempdir_in(&temp_dir.path()).unwrap();
        let path_string = temp_dir.path().to_str().unwrap().to_string();
        let name = "nonexistent".to_string();
        let git_cmd = "status";

        let result = execute_git_cmd(path_string.clone(), &name, git_cmd);
        assert!(result.is_err());
    }
}