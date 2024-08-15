use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::Child;

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::{ERROR_WRITER, git};
use crate::directory::Directory;
use crate::git::{GIT_PULL, GIT_STATUS};
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

    /// Checkout to develop > master > main branch and delete previous branch
    /// Applied to current working dir (CWD)
    Clean,
}

pub fn handle_list(path: &Path, writer: impl Write) -> Result<()> {
    println!("Scanning in path {:?}", path);
    let projects = scan(path)?;
    print_projects(projects, writer);
    Ok(())
}

pub fn handle_status(path: &Path, name: &String) -> Result<()> {
    execute_git_cmd(path, name, GIT_STATUS)
}

pub fn handle_pull(path: &Path, name: &String) -> Result<()> {
    execute_git_cmd(path, name, GIT_PULL)
}

fn execute_git_cmd(path: &Path, name: &String, git_cmd: &str) -> Result<()> {
    let projects = scan(path)?;

    if "all".eq_ignore_ascii_case(name) {
        projects
            .iter()
            .for_each(|project| for_project(git_cmd, project, &mut stdout()))
    } else {
        let project = projects
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
            .with_context(|| format!("Project with given name '{}' was not found", &name.red()))?;
        for_project(git_cmd, project, &mut stdout());
    }
    Ok(())
}

fn for_project(arg: &str, project: &Project, mut writer: impl Write) {
    print_project(project, &mut writer);

    if let Some(repos) = &project.repos {
        repos.iter().for_each(|repo| {
            let cmd = git::custom_cwd_cmd(arg, repo.path.as_path());
            print_repository(repo, cmd, &mut writer);
        });
    }
}

fn print_project(project: &Project, mut writer: impl Write) {
    writeln!(
        &mut writer,
        "Project {} found at {:?}",
        &project.name.bright_green(),
        &project.path
    ).expect(ERROR_WRITER);
}

fn print_repository(repo: &Directory, cmd: Child, mut writer: impl Write) {
    let cmd_output = cmd.wait_with_output().unwrap();
    match cmd_output.status.code() {
        Some(0) => writeln!(
            writer,
            "{} {}: {}",
            "=>".bright_green(),
            repo.name.yellow(),
            String::from_utf8_lossy(&cmd_output.stdout)
        ).expect(ERROR_WRITER),
        Some(code) => writeln!(
            writer,
            "{} {}: {} {}",
            "=>".red(),
            repo.name.yellow(),
            "Error".red(),
            code
        ).expect(ERROR_WRITER),
        None => {}
    }
}

pub fn handle_clean() -> Result<()> {
    let current = git::get_curr_branch()?;
    let branches = git::get_branches()?;
    clean(current, branches, stdout())
}

fn clean(current: String, branches: Vec<String>, mut writer: impl Write) -> Result<()> {
    match determine_target(branches) {
        Some(target) => {
            if target.eq(&current) {
                writeln!(writer, "Current branch is already {}", current)
                    .expect(ERROR_WRITER);
                Ok(())
            } else {
                writeln!(writer, "Branch will be changed to {} and branch {} will be deleted",
                         target.bright_green(), &current.bright_red())
                    .expect(ERROR_WRITER);
                writeln!(writer, "Continue? (y / N and hit Enter)")
                    .expect(ERROR_WRITER);

                if user_confirmed(&get_user_input()) {
                    git::checkout(target)
                        .and_then(|_| { git::pull() })
                        .and_then(|_| { git::delete(current) })
                } else {
                    writeln!(writer, "Aborting").expect(ERROR_WRITER);
                    Ok(())
                }
            }
        }
        None => {
            writeln!(writer, "Unable to determine target branch to checkout to")
                .expect(ERROR_WRITER);
            Ok(())
        }
    }
}

fn get_user_input() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input)
        .expect("Failed to read user input");
    input
}

fn user_confirmed(input: &str) -> bool {
    input.trim().eq_ignore_ascii_case("y") ||
        input.trim().eq_ignore_ascii_case("yes")
}

fn determine_target(branches: Vec<String>) -> Option<String> {
    let develop = "develop".to_string();
    if branches.contains(&develop) { return Some(develop) }

    let main = "main".to_string();
    if branches.contains(&main) { return Some(main) }

    let master = "master".to_string();
    if branches.contains(&master) { return Some(master) };

    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempfile::{tempdir, tempdir_in};

    use super::*;

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

        assert_eq!(
            String::from_utf8_lossy(&result),
            format!(
                "Project {} found at {:?}\n{} {}: {} 128\n",
                &project.name.bright_green(),
                &project.path,
                "=>".red(),
                "Repo".yellow(),
                "Error".red()
            )
        );
    }

    #[test]
    fn should_print_project() {
        let project = Project {
            name: "Project".to_string(),
            path: PathBuf::from("/some/path"),
            repos: Some(vec![Directory {
                name: "Repo".to_string(),
                path: PathBuf::from("/some/path/sub"),
            }]),
        };

        let mut result = Vec::new();
        print_project(&project, &mut result);

        assert_eq!(
            String::from_utf8_lossy(&result),
            format!(
                "Project {} found at {:?}\n",
                &project.name.bright_green(),
                &project.path
            )
        );
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

    #[test]
    fn should_determine_develop() {
        let branches = vec!["main".to_string(), "test".to_string(), "develop".to_string()];
        let result = determine_target(branches);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "develop");
    }

    #[test]
    fn should_determine_main() {
        let branches = vec!["test".to_string(), "main".to_string(), "test2".to_string()];
        let result = determine_target(branches);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "main");
    }

    #[test]
    fn should_determine_master() {
        let branches = vec!["test".to_string(), "master".to_string(), "test2".to_string()];
        let result = determine_target(branches);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "master");
    }

    #[test]
    fn should_fail_to_determine() {
        let branches = vec!["test".to_string(), "some-branch".to_string(), "test2".to_string()];
        let result = determine_target(branches);
        assert!(result.is_none());
    }

    #[test]
    fn should_return_true_user_confirmed() {
        let mut input = "y".to_string();
        assert!(user_confirmed(&input));
        input = "yes".to_string();
        assert!(user_confirmed(&input));
        input = "Y".to_string();
        assert!(user_confirmed(&input));
        input = "YeS".to_string();
        assert!(user_confirmed(&input));
    }

    #[test]
    fn should_return_false_user_confirmed() {
        let mut input = "n".to_string();
        assert!(!user_confirmed(&input));
        input = "no".to_string();
        assert!(!user_confirmed(&input));
        input = "N".to_string();
        assert!(!user_confirmed(&input));
        input = "NO".to_string();
        assert!(!user_confirmed(&input));
        input = "Anything   ".to_string();
        assert!(!user_confirmed(&input));
        input = "   ".to_string();
        assert!(!user_confirmed(&input));
    }

    #[test]
    fn clean_should_find_current_is_same_as_target() {
        let current = "master".to_string();
        let branches = vec!["test".to_string(), "master".to_string(), "test2".to_string()];
        let mut result = Vec::new();

        clean(current, branches, &mut result).unwrap();

        assert_eq!(String::from_utf8_lossy(&result), "Current branch is already master\n");
    }

    #[test]
    fn clean_should_be_unable_to_determine_target() {
        let current = "test2".to_string();
        let branches = vec!["test".to_string(), "some-branch".to_string(), "test2".to_string()];
        let mut result = Vec::new();

        clean(current, branches, &mut result).unwrap();

        assert_eq!(String::from_utf8_lossy(&result), "Unable to determine target branch to checkout to\n");
    }
}