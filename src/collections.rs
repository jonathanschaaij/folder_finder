use std::path::PathBuf;

use crate::database;
use crate::projects;
use crate::types;

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

fn path_to_collection(path: &PathBuf) -> types::DataType {
    let full_path = path.canonicalize().unwrap();
    let collection = types::Collection {
        path: full_path.to_str().unwrap().to_string(),
        tags: Default::default(),
    };
    types::DataType::Collection(collection)
}

fn list() {
    let collections = database::list_collections().unwrap();
    collections
        .iter()
        .for_each(|collection| println!("{}", collection));
}

fn add(path: PathBuf) {
    let collection = path_to_collection(&path);
    let res = database::add(&collection);
    match res {
        Ok(_) => println!("Collection added"),
        Err(_e) => {
            eprintln!("The collection already exists");
            return;
        }
    }

    // TODO:Ask for tags
    // TODO: ASK for auto import maybe using a flag
    let dir = std::fs::read_dir(&path).unwrap();
    let subdirs = dir
        .filter_map(|entry| {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    subdirs.iter().for_each(|subdir| {
        projects::add(subdir.to_path_buf());
    });
}

fn del(path: PathBuf) {
    let collection = path_to_collection(&path);
    let res = database::delete(&collection);
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
        let col = path_to_collection(&std::env::current_dir().unwrap());
        let res = database::add_tag(&col, tag.clone(), force);
        match res {
            Ok(_) => println!("Tag added: {}", tag.name),
            Err(e) => {
                match e {
                    types::NotFoundError::Tag => eprintln!("Tag not found"),
                    _ => eprintln!("Not currently in folder with collection"),
                }
                return;
            }
        }
    });
}
