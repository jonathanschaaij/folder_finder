use std::path::PathBuf;

use clap::{arg, value_parser, Command};
use fzf_wrapped::Fzf;

use crate::database;
use crate::projects;
use crate::tags;
use crate::types;

pub fn command() -> Command {
    Command::new("collection")
        .about("Manage collections / folders with projects")
        .arg_required_else_help(true)
        .subcommand(Command::new("list").about("List all collections"))
        .subcommand(
            Command::new("add").about("Add a new collection").arg(
                arg!(<PATH>)
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            ),
        )
        .subcommand(
            Command::new("del").about("Delete a collection").arg(
                arg!(<PATH>)
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            ),
        )
        .subcommand(
            Command::new("tag")
                .about("Tag the current collection")
                .arg(
                    arg!(<PATH>)
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(arg!(-f --force "Add tags if they don't exist'"))
                .arg(arg!(<TAG>).required(false).num_args(1..)),
        )
        .subcommand(Command::new("goto").about("Go to a collection"))
}

pub fn run(args: &clap::ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("goto", args)) => goto(),
        Some(("add", args)) => add(args.get_one::<PathBuf>("PATH").unwrap().clone(), true),
        Some(("del", args)) => del(args.get_one::<PathBuf>("PATH").unwrap().clone()),
        Some(("tag", args)) => add_tag(
            args.get_one::<PathBuf>("PATH").unwrap().clone(),
            args.get_flag("force"),
            args.get_many::<String>("TAG")
                .unwrap_or_default()
                .map(|name| types::Tag {
                    name: name.to_string(),
                })
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

fn goto() {
    let collections = database::list_collections().unwrap();
    let strings: Vec<String> = collections.iter().map(|proj| format!("{}", proj)).collect();
    let mut fzf = Fzf::default();
    fzf.run().expect("Failed to start fzf");
    fzf.add_items(&strings).expect("Failed to add items");
    let users_selection = fzf.output().expect("Failed to get the user's output");
    let index = strings.iter().position(|x| x == &users_selection).unwrap();
    let proj = &collections[index];

    println!("{}", proj.path);
}

fn add(path: PathBuf, ask_tags: bool) {
    let collection = path_to_collection(&path);
    let res = database::add(&collection);
    match res {
        Ok(_) => println!("Collection added"),
        Err(_e) => {
            eprintln!("The collection already exists");
            return;
        }
    }
    add_tag(path.clone(), true, tags::select_tags(true));

    // NOTE: Automatically add all subdirectories as projects
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
        let proj = projects::from_path(&subdir);
        projects::add(subdir.to_path_buf(), false);
        database::add_project_collection(&proj, &collection);
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

fn add_tag(path: PathBuf, force: bool, tags: Vec<types::Tag>) {
    let tags = if tags.is_empty() {
        tags::select_tags(force)
    } else {
        tags
    };

    tags.iter().for_each(|tag| {
        let col = path_to_collection(&path);
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
