use crate::database;
use crate::tags;
use crate::types;
use std::path::PathBuf;

pub fn command(args: &clap::ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("add", args)) => add(args.get_one::<PathBuf>("PATH").unwrap().clone()),
        Some(("del", args)) => del(args.get_one::<PathBuf>("PATH").unwrap().clone()),
        Some(("tag", args)) => add_tag(
            args.get_flag("force"),
            args.get_many::<String>("TAG")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect(),
        ),
        _ => panic!("Should be handled by clap itself"),
    }
}

fn path_to_project(path: PathBuf) -> types::DataType {
    let full_path = path.canonicalize().unwrap();
    let proj = types::Project {
        name: full_path.file_name().unwrap().to_str().unwrap().to_string(),
        path: full_path.to_str().unwrap().to_string(),
        collection: None,
        tags: Default::default(),
    };
    types::DataType::Project(proj)
}

fn list() {
    let projects = database::list_projects().unwrap();
    projects.iter().for_each(|proj| println!("{}", proj));
}

pub fn add(path: PathBuf) {
    let proj = path_to_project(path);
    let res = database::add(&proj);
    match res {
        Ok(_) => println!("Project added"),
        Err(_e) => {
            eprintln!("The project already exists");
            return;
        }
    }
    // TODO:Ask for tags
}

fn del(path: PathBuf) {
    let proj = path_to_project(path);
    let res = database::delete(&proj);
    match res {
        Ok(_) => println!("Deleted"),
        Err(_e) => {
            eprintln!("Error deleting collection");
            return;
        }
    }
}

fn add_tag(force: bool, tags: Vec<&str>) {
    tags.iter().for_each(|tag| {
        let tag = types::Tag {
            name: tag.to_string(),
        };
        let proj = path_to_project(std::env::current_dir().unwrap());
        let res = database::add_tag(&proj, tag.clone(), force);
        match res {
            Ok(_) => println!("Tag added: {}", tag.name),
            Err(e) => {
                match e {
                    types::NotFoundError::Tag => eprintln!("Tag not found"),
                    _ => eprintln!("Not currently in folder with project"),
                }
                return;
            }
        }
    });
}
