use std::{fs, io};
use std::path::{Path, PathBuf};

pub const GIT_DIR: &'static str = ".git";
pub const NAME_UNAVAILABLE: &'static str = "Name_Unavailable";

pub struct Directory {
    pub name: String,
    pub path: PathBuf,
}

impl Directory {
    pub fn from(p: PathBuf) -> Directory {
        Directory {
            name: get_name(&p),
            path: p,
        }
    }
}

pub fn read_dirs(path: &Path) -> anyhow::Result<Vec<Directory>, io::Error> {
    // let mut dirs = Vec::new();
    // let r = fs::read_dir(path)?;
    //
    // for dir in r {
    //     let p = dir.unwrap().path();
    //     if p.is_dir() {
    //         dirs.push(Directory {
    //             name: get_name(&p),
    //             path: p,
    //         });
    //     }
    // }
    //
    // Ok(dirs)
    let dirs = fs::read_dir(path)?
        .filter(|r| r.is_ok())
        .into_iter()
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir())
        .map(Directory::from)
        .collect();
    Ok(dirs)
}

pub fn contains_git(dirs: &Vec<Directory>) -> bool {
    dirs.iter()
        .any(|x| { GIT_DIR.eq(&x.name) })
}

pub fn get_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(NAME_UNAVAILABLE.to_string().as_ref()).to_str()
        .unwrap_or(NAME_UNAVAILABLE).to_string()
}