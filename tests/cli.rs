use assert_cmd::Command;
use predicates::prelude::predicate;

#[test]
fn dev_dir_not_defined() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("please")?;
    cmd.env_remove("DEV_DIR");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("DEV_DIR is not defined!"));
    Ok(())
}

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("please")?;

    cmd.arg("help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage: please"));
    Ok(())
}

#[test]
fn no_command_given() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("please")?;
    cmd.env("DEV_DIR", "/home");
    cmd.assert().success().stdout(predicate::str::contains(
        "No command given. Use with --help or -h to see available commands and options",
    ));

    Ok(())
}
