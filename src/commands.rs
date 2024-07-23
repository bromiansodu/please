use std::process::Stdio;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::project::{print_projects, scan};

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
    let projects = scan(&path_string)?;
    let project = projects.iter().find(|p| p.name.eq_ignore_ascii_case(&name))
        .with_context(|| format!("Project with given name '{}' was not found", &name))?;
    println!("Project {} found at {:?}", &project.name, &project.path);

    if let Some(repos) = &project.repos {
        repos.iter().for_each(|repo| {
            let cmd = std::process::Command::new("git")
                .arg("status")
                .current_dir(repo.path.as_path())
                .stdin(Stdio::inherit())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let cmd_output = cmd.wait_with_output().unwrap();
            match cmd_output.status.code() {
                Some(0) => println!("[{}]: {}",repo.name, String::from_utf8_lossy(&cmd_output.stdout)),
                Some(code) => println!("[{}] Error {}",repo.name, code),
                None => {}
            }
        });
    }
    Ok(())
}