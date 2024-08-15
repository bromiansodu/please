use crate::directory::{contains_git, get_name, read_dirs, Directory};
use crate::ERROR_WRITER;
use anyhow::{anyhow, Error};
use colored::Colorize;
use std::path::{Path, PathBuf};

pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub repos: Option<Vec<Directory>>,
}

pub fn scan(path: &Path) -> anyhow::Result<Vec<Project>, Error> {
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
    vec![Project {
        name: get_name(path),
        path: path.to_path_buf(),
        repos: None,
    }]
}

fn scan_deeper(
    parent_path: &Path,
    parent_dirs: Vec<Directory>,
) -> anyhow::Result<Vec<Project>, Error> {
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

pub fn print_projects(projects: Vec<Project>, mut writer: impl std::io::Write) {
    for project in projects {
        if let Some(repos) = project.repos {
            writeln!(writer,
                "\nProject {}, {:?}, with Git repositories:",
                project.name.bright_green(),
                project.path
            ).expect(ERROR_WRITER);
            for repo in repos {
                writeln!(writer, "  - {}", repo.name.yellow()).expect(ERROR_WRITER);
            }
        } else {
            writeln!(writer,
                "\nProject found: {}, {:?}",
                project.name.bright_green(),
                project.path
            ).expect(ERROR_WRITER);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_print_all_projects() {
        let projects = vec![make_project_with_two_repos(), make_project_with_one_repo()];

        let mut result = Vec::new();
        print_projects(projects, &mut result);

        assert_eq!(
            result,
            format!(
                "\nProject {}, \"{}\", with Git repositories:\n  - {}\n  - {}\n\
        \nProject {}, \"{}\", with Git repositories:\n  - {}\n",
                "Project1".bright_green(),
                "/some/path",
                "Repo1".yellow(),
                "Repo2".yellow(),
                "Project2".bright_green(),
                "/some/different/path",
                "DifferentRepo".yellow()
            )
                .as_bytes()
        );
    }

    #[test]
    fn should_print_one_project() {
        let projects = vec![make_project_with_two_repos()];

        let mut result = Vec::new();
        print_projects(projects, &mut result);

        assert_eq!(
            result,
            format!(
                "\nProject {}, \"{}\", with Git repositories:\n  - {}\n  - {}\n",
                "Project1".bright_green(),
                "/some/path",
                "Repo1".yellow(),
                "Repo2".yellow()
            )
                .as_bytes()
        );
    }

    #[test]
    fn should_print_project_without_repos() {
        let projects = vec![make_project_without_repos()];

        let mut result = Vec::new();
        print_projects(projects, &mut result);

        assert_eq!(
            result,
            format!(
                "\nProject found: {}, \"{}\"\n",
                "Project".bright_green(),
                "/some/path"
            )
                .as_bytes()
        );
    }

    fn make_project_with_two_repos() -> Project {
        Project {
            name: "Project1".to_string(),
            path: PathBuf::from("/some/path"),
            repos: Some(vec![
                Directory {
                    name: "Repo1".to_string(),
                    path: PathBuf::from("/some/path/repo1"),
                },
                Directory {
                    name: "Repo2".to_string(),
                    path: PathBuf::from("/some/path/repo2"),
                },
            ]),
        }
    }

    fn make_project_with_one_repo() -> Project {
        Project {
            name: "Project2".to_string(),
            path: PathBuf::from("/some/different/path"),
            repos: Some(vec![Directory {
                name: "DifferentRepo".to_string(),
                path: PathBuf::from("/some/different/path/repo"),
            }]),
        }
    }

    fn make_project_without_repos() -> Project {
        Project {
            name: "Project".to_string(),
            path: PathBuf::from("/some/path"),
            repos: None,
        }
    }

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
