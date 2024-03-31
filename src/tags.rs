use crate::database;
use crate::types;

pub fn command(args: &clap::ArgMatches) {
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
