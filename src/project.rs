use std::path::{Path, PathBuf};
use anyhow::{anyhow, Error};
use crate::directory::{contains_git, Directory, read_dirs, get_name};

pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub repos: Option<Vec<Directory>>,
}

pub fn scan(path_string: &String) -> anyhow::Result<Vec<Project>, Error> {
    let path = Path::new(path_string);

    let dirs = read_dirs(path)?;
    if contains_git(&dirs) {
        return Ok(parent_lvl_project(path));
    }

    let projects = scan_deeper(path, dirs)?;
    if projects.is_empty() {
        return Err(anyhow!("No projects found"));
    }
    Ok(projects)
}

fn parent_lvl_project(path: &Path) -> Vec<Project> {
    let mut project = Vec::new();
    project.push(Project {
        name: get_name(path),
        path: path.to_path_buf(),
        repos: None,
    });
    project
}

fn scan_deeper(parent_path: &Path, parent_dirs: Vec<Directory>)
               -> anyhow::Result<Vec<Project>, Error> {
    let mut projects = Vec::new();
    let mut repos = Vec::new();

    for dir in parent_dirs {
        let dirs = read_dirs(dir.path.as_path())?;
        if contains_git(&dirs) {
            repos.push(dir);
        } else {
            let mut sub_dirs = scan_deeper(&dir.path, dirs)?;
            if !sub_dirs.is_empty() {
                projects.append(&mut sub_dirs);
            }
        }
    }

    if !repos.is_empty() {
        projects.push(Project {
            name: get_name(parent_path),
            path: PathBuf::from(parent_path),
            repos: Some(repos),
        })
    }
    Ok(projects)
}

pub fn print_projects(projects: Vec<Project>) {
    for project in projects {
        if let Some(repos) = project.repos {
            println!("Project: {}, {:?}, has Git repositories:", project.name, project.path);
            for repo in repos {
                println!("Repository: {}, {:?}", repo.name, repo.path);
            }
        } else {
            println!("Project found: {}, {:?}", project.name, project.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_lvl_project() {
        let path = Path::new("/some/path/some-name");
        let result = parent_lvl_project(path);
        assert_eq!(1, result.len());

        let res_project = result.into_iter().nth(0).unwrap();
        assert_eq!("some-name", res_project.name);
        assert_eq!(path, res_project.path);
        assert!(res_project.repos.is_none());
    }
}