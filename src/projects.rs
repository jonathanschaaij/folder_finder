use clap::{arg, value_parser, Command};

use crate::auto_tags;
use crate::database;
use crate::tags;
use crate::types;
use std::path::PathBuf;

pub fn command() -> Command {
    Command::new("project")
        .about("Manage projects")
        .arg_required_else_help(true)
        .subcommand(Command::new("list").about("List all projects"))
        .subcommand(
            Command::new("add").about("Add a new project").arg(
                arg!(<PATH>)
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            ),
        )
        .subcommand(
            Command::new("del").about("Delete a project").arg(
                arg!(<PATH>)
                    .required(true)
                    .value_parser(value_parser!(PathBuf)),
            ),
        )
        .subcommand(
            Command::new("tag")
                .about("Tag the current project")
                .arg(
                    arg!(<PATH>)
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(arg!(-f --force "Add tags if they don't exist'"))
                .arg(arg!(<TAG>).required(true).num_args(1..)),
        )
}

pub fn run(args: &clap::ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
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

pub fn from_path(path: &PathBuf) -> types::DataType {
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

pub fn add(path: PathBuf, ask_tags: bool) {
    let proj = from_path(&path);
    let res = database::add(&proj);
    match res {
        Ok(_) => println!("Project added"),
        Err(_e) => {
            eprintln!("The project already exists");
            return;
        }
    }
    if ask_tags {
        add_tag(path.clone(), true, tags::select_tags(true));
    }

    auto_tags::auto_tag_project(&proj);
}

fn del(path: PathBuf) {
    let proj = from_path(&path);
    let res = database::delete(&proj);
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
        let proj = from_path(&path);
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
