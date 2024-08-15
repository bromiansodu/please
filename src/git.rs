use std::env::consts::OS;
use std::path::Path;
use std::process::{Child, Stdio};

use anyhow::{anyhow, Error};
use colored::Colorize;

const GIT_EXEC: &str = "git";
const GIT_EXEC_WINDOWS: &str = "git.exe";
pub const GIT_PULL: &str = "pull";
pub const GIT_STATUS: &str = "status";
pub const GIT_CHECKOUT: &str = "checkout";
pub const GIT_BRANCH: &str = "branch";

pub fn checkout(target: String) -> anyhow::Result<()> {
    let cmd_output = two_args_cmd(GIT_CHECKOUT, &target).wait_with_output().unwrap();

    match cmd_output.status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!("Unable to checkout to {} code[{}]", target, code)),
        None => Err(anyhow!("Git checkout to {} errored", target))
    }
}

pub fn pull() -> anyhow::Result<()> {
    let cmd_output = one_arg_cmd(GIT_PULL).wait_with_output().unwrap();

    match cmd_output.status.code() {
        Some(0) => {
            println!("Pulled the latest changes");
            Ok(())
        },
        Some(code) => Err(anyhow!("Git pull errored. Code[{}]", code)),
        None => Err(anyhow!("Unable to pull"))
    }
}

pub fn delete(branch: String) -> anyhow::Result<()> {
    let cmd_output = three_args_cmd(GIT_BRANCH, "-d", &branch)
        .wait_with_output().unwrap();

    match cmd_output.status.code() {
        Some(0) => {
            println!("{} has been deleted", branch.yellow());
            Ok(())
        },
        Some(code) => Err(anyhow!("Deleting branch {} failed. Code[{}]", branch, code)),
        None => Err(anyhow!("Deleting branch {} failed", branch))
    }
}

pub fn get_curr_branch() -> anyhow::Result<String, Error> {
    let cmd_output = two_args_cmd(GIT_BRANCH, "--show-current")
        .wait_with_output().unwrap();

    match cmd_output.status.code() {
        Some(0) => Ok(String::from_utf8_lossy(&cmd_output.stdout).trim().to_string()),
        Some(code) => Err(anyhow!("Unable to read current branch. Code[{}]", code)),
        None => Err(anyhow!("Unable to read current branch"))
    }
}

pub fn get_branches() -> anyhow::Result<Vec<String>, Error> {
    let cmd_output = one_arg_cmd(GIT_BRANCH).wait_with_output().unwrap();
    match cmd_output.status.code() {
        Some(0) => {
            let sanitized = String::from_utf8_lossy(&cmd_output.stdout)
                .to_string()
                .replace("*", "");
            let branches = sanitized
                .trim_end()
                .split("\n")
                .map(|s| s.trim_start())
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            Ok(branches)
        }
        Some(code) => Err(anyhow!("Unable to read branches. Code[{}]", code)),
        None => Err(anyhow!("Unable to read branches"))
    }
}

fn one_arg_cmd(arg: &str) -> Child {
    std::process::Command::new(by_os())
        .arg(arg)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

fn two_args_cmd(arg1: &str, arg2: &str) -> Child {
    std::process::Command::new(by_os())
        .arg(arg1).arg(arg2)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

fn three_args_cmd(arg1: &str, arg2: &str, arg3: &str) -> Child {
    std::process::Command::new(by_os())
        .arg(arg1).arg(arg2).arg(arg3)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

pub fn custom_cwd_cmd(arg: &str, path: &Path) -> Child {
    std::process::Command::new(by_os())
        .arg(arg)
        .current_dir(path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

fn by_os() -> &'static str {
    if OS.eq("windows") {
        GIT_EXEC_WINDOWS
    } else {
        GIT_EXEC
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::process::Output;

    use tempfile::{tempdir, TempDir};

    use super::*;

    fn init_git() -> TempDir {
        let temp_dir = tempdir().unwrap();
        println!("temp dir path: {:?}", &temp_dir.path());
        assert!(env::set_current_dir(temp_dir.path()).is_ok());

        assert!(one_arg_cmd("init").wait().is_ok());
        temp_dir
    }

    #[test]
    fn should_init_git() {
        let temp_dir = tempdir().unwrap();
        println!("temp dir path: {:?}", &temp_dir.path());
        assert!(env::set_current_dir(temp_dir.path()).is_ok());

        assert!(one_arg_cmd("init").wait().is_ok());

        let check = one_arg_cmd(GIT_STATUS).wait_with_output().unwrap();
        validate_correct_branch(&check, "On branch main");
    }

    #[test]
    fn should_get_current_branch() {
        let temp_dir = tempdir().unwrap();
        println!("temp dir path: {:?}", &temp_dir.path());
        assert!(env::set_current_dir(temp_dir.path()).is_ok());

        assert!(one_arg_cmd("init").wait().is_ok());
        let current = get_curr_branch().unwrap();
        assert_eq!(current, "main");
    }

    #[test]
    fn should_error_pull() {
        let _temp_dir = init_git();
        assert!(pull().is_err());
    }

    #[test]
    fn should_error_delete() {
        let _temp_dir = init_git();
        assert!(delete("main".to_string()).is_err());
    }

    #[test]
    fn should_error_checkout() {
        let _temp_dir = init_git();
        assert!(checkout("main".to_string()).is_err());
    }

    #[test]
    fn should_init_git_with_custom_cwd() {
        let temp_dir = tempdir().unwrap();
        assert!(custom_cwd_cmd("init", temp_dir.path()).wait().is_ok());

        assert!(env::set_current_dir(temp_dir.path()).is_ok());
        let check = one_arg_cmd(GIT_STATUS).wait_with_output().unwrap();
        validate_correct_branch(&check, "On branch main");
    }

    fn validate_correct_branch(out: &Output, expected: &str) {
        match out.status.code() {
            Some(0) => {
                let out = String::from_utf8_lossy(&out.stdout);
                println!("output: {}", &out);
                assert!(out.contains(expected));
            },
            Some(code) => {
                let out = String::from_utf8_lossy(&out.stdout);
                println!("code: {}, output: {}", code, &out);
                assert!(out.contains(expected));
            },
            _ => panic!("Checking status has failed")
        }
    }
}