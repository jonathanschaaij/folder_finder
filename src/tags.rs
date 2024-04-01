use crate::database;
use crate::types;

use clap::{arg, Command};

pub fn command() -> Command {
    Command::new("tag")
        .about("Manage tags")
        .arg_required_else_help(true)
        .subcommand(Command::new("list").about("List all tags"))
        .subcommand(
            Command::new("add")
                .about("Add a new tag")
                .arg(arg!(<TAG>).required(true).num_args(1..)),
        )
        .subcommand(
            Command::new("del")
                .about("Delete a tag")
                .arg(arg!(<TAG>).required(true).num_args(1..)),
        )
}

pub fn run(args: &clap::ArgMatches) {
    match args.subcommand() {
        Some(("list", _)) => list(),
        Some(("add", args)) => add(args
            .get_many::<String>("TAG")
            .unwrap_or_default()
            .map(|v| v.as_str())
            .collect()),
        Some(("del", args)) => del(args
            .get_many::<String>("TAG")
            .unwrap_or_default()
            .map(|v| v.as_str())
            .collect()),
        _ => panic!("Should be handled by clap itself"),
    }
}

fn word_to_tag(word: &str) -> types::DataType {
    let tag = types::Tag {
        name: word.to_string().to_lowercase(),
    };
    types::DataType::Tag(tag)
}

fn list() {
    let tags = database::list_tags();
    tags.iter().for_each(|tag| println!("{}", tag.name));
}

fn add(words: Vec<&str>) {
    for word in words {
        let tag = word_to_tag(word);
        let res = database::add(&tag);
        match res {
            Ok(_) => println!("Tag added"),
            Err(_e) => {
                eprintln!("The tag already exists");
                return;
            }
        }
    }
}

fn del(words: Vec<&str>) {
    for word in words {
        let tag = word_to_tag(word);
        let res = database::delete(&tag);
        match res {
            Ok(_) => println!("Tag deleted"),
            Err(_e) => {
                eprintln!("Error deleting tag");
                return;
            }
        }
    }
}

fn query_new() -> Vec<types::Tag> {
    let mut tags = Vec::new();
    loop {
        let tag = dialoguer::Input::<String>::new()
            .with_prompt("Enter a tag")
            .allow_empty(true)
            .interact()
            .unwrap();
        if tag.is_empty() {
            break tags;
        } else {
            tags.push(types::Tag { name: tag });
        }
    }
}

pub fn select_tags(allow_new: bool) -> Vec<types::Tag> {
    //Option<types::Tag> {
    let tags = database::list_tags();
    let mut tag_names = tags.iter().map(|tag| tag.name.clone()).collect::<Vec<_>>();
    let mut index_offset = 0;
    if allow_new {
        tag_names.insert(0, "Add more tags".to_string());
        index_offset = 1;
    }

    let selected = dialoguer::MultiSelect::new()
        .items(&tag_names)
        .interact()
        .unwrap();

    let query = selected.contains(&0);
    let mut selected_tags = selected
        .iter()
        .filter(|&&i| i != 0)
        .map(|&i| tags[i - index_offset].clone())
        .collect::<Vec<_>>();

    if query {
        query_new()
            .iter()
            .for_each(|tag| selected_tags.push(tag.clone()));
    }
    selected_tags
}
