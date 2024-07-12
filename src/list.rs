pub fn handle_list(default: &Option<String>, name: &Option<String>, path: &Option<String>) {
    match name {
        Some(n) => for_project(default, n),
        None => match path {
            Some(p) => for_path(p),
            None => println!("No name or path given")
        }
    }
}

fn for_project(default: &Option<String>, n: &String) {
    match default {
        Some(var) => {
            println!("Scanning for name {n} in directory pointed by {var} env var");
        }
        None => println!("Default directory is not defined! (DEV_DIR env var or default-var option)")
    }
}

fn for_path(p: &String) {
    println!("Scanning in path {p}");
}