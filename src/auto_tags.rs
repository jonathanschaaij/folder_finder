use crate::{database, types};

pub fn auto_tag_project(proj: &types::DataType) -> () {
    let proj = proj.project().expect("Expected a project");
    let mut tags = vec![];
    tags.push(check_git(proj));
    tags.push(check_rust(proj));
    tags.push(check_svelte(proj));
    // TODO: Add more checks here, e.g. for Python, JS, HTML, Go, etc.
    // TODO: Feature to add collection of file extensions with corresponding tags e.g. .js -> JS, .py -> Python
    tags.iter().for_each(|tag| match tag {
        Some(tag) => {
            let _ = database::add_tag(&types::DataType::Project(proj.clone()), tag.clone(), true);
        }
        None => (),
    });
    ()
}

fn check_git(proj: &types::Project) -> Option<types::Tag> {
    let git_tag = types::Tag {
        name: "git".to_string(),
    };
    if proj.tags.contains(&git_tag) {
        return None;
    }
    //check if .git directory exists
    let git_path = std::path::Path::new(&proj.path).join(".git");
    if git_path.exists() {
        return Some(git_tag);
    } else {
        return None;
    }
}

fn check_rust(proj: &types::Project) -> Option<types::Tag> {
    let rust_tag = types::Tag {
        name: "rust".to_string(),
    };
    if proj.tags.contains(&rust_tag) {
        return None;
    }
    //check if Cargo.toml exists
    let cargo_path = std::path::Path::new(&proj.path).join("Cargo.toml");
    if cargo_path.exists() {
        return Some(rust_tag);
    } else {
        return None;
    }
}

fn check_svelte(proj: &types::Project) -> Option<types::Tag> {
    let svelte_tag = types::Tag {
        name: "svelte".to_string(),
    };
    if proj.tags.contains(&svelte_tag) {
        return None;
    }
    //check if svelte.config.js exists
    let svelte_path = std::path::Path::new(&proj.path).join("svelte.config.js");
    if svelte_path.exists() {
        return Some(svelte_tag);
    } else {
        return None;
    }
}
