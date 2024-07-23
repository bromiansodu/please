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

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, tempdir_in};

    use super::*;

    #[test]
    fn test_from() {
        let source = PathBuf::from("/some/dir-name");
        let result = Directory::from(source.clone());

        assert_eq!("dir-name", result.name);
        assert_eq!(source, result.path);
    }

    #[test]
    fn test_contains_git() {
        let dirs = vec![Directory {
            name: "some-dir".to_string(),
            path: PathBuf::from("/some/path")
        }, Directory{
            name: ".git".to_string(),
            path: PathBuf::from("/some/.git")
        }];

        assert!(contains_git(&dirs));
    }

    #[test]
    fn contains_git_false() {
        let dirs = vec![Directory {
            name: "some-dir".to_string(),
            path: PathBuf::from("/some/path")
        }];

        assert!(!contains_git(&dirs));
    }

    #[test]
    fn test_read_dirs() {
        let temp_dir = tempdir().unwrap();
        let temp_sub_dir = tempdir_in(&temp_dir.path()).unwrap();

        let result = read_dirs(temp_dir.path()).unwrap();

        assert!(result.len().eq(&1));
        assert_eq!(temp_sub_dir.path().file_name().unwrap().to_str().unwrap(),
                   result.into_iter().nth(0).unwrap().name)
    }

    #[test]
    fn read_dirs_empty() {
        let temp_dir = tempdir().unwrap();
        let result = read_dirs(temp_dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    #[should_panic]
    fn read_dirs_error() {
        let result = read_dirs(Path::new("/not/existing")).unwrap();
        assert!(result.is_empty());
    }
}