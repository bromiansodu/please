use std::env;

use anyhow::{Context, Result};

use crate::DEFAULT_DEV_DIR_VAR;
use crate::project::{print_projects, scan};

pub fn handle_list(override_default: &Option<String>, path: &Option<String>)
                   -> Result<()> {
    match path {
        Some(p) => for_path(p),
        None => for_dev_dir(&override_default),
    }
}

fn for_dev_dir(override_default: &Option<String>) -> Result<()> {
    match override_default {
        Some(var) => {
            let val = env::var(var)
                .with_context(|| format!("{} is not defined!", var))?;
            for_path(&val)
        }
        None => {
            let dir = env::var(DEFAULT_DEV_DIR_VAR)
                .with_context(|| format!("{DEFAULT_DEV_DIR_VAR} is not defined!"))?;
            for_path(&dir)
        }
    }
}

fn for_path(p: &String) -> Result<()> {
    println!("Scanning in path {p}");
    // { //debug
    //     let paths = fs::read_dir(p)
    //         .with_context(|| format!("Failed to read given path: {p}"))?;
    //     for path in paths {
    //         println!("Name: {}", path.unwrap().path().display());
    //     }
    // }

    let projects = scan(p)?;
    print_projects(projects);
    Ok(())
}