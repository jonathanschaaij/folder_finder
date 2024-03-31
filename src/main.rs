use clap::{arg, command, value_parser, Command};
use std::path::PathBuf;

pub mod collections;
pub mod database;
pub mod projects;
pub mod tags;
pub mod types;

fn main() {
    let matches = command!()
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Setup the database")
                .arg(arg!(-r --reset "Reset the database")),
        )
        .subcommand(
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
                        .arg(arg!(-f --force "Add tags if they don't exist'"))
                        .arg(arg!(<TAG>).required(true).num_args(1..)),
                ),
        )
        .subcommand(
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
                        .arg(arg!(-f --force "Add tags if they don't exist'"))
                        .arg(arg!(<TAG>).required(true).num_args(1..)),
                ),
        )
        .subcommand(
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
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", args)) => init(args),
        Some(("tag", args)) => tags::command(args),
        Some(("collection", args)) => collections::command(args),
        Some(("project", args)) => projects::command(args),
        _ => panic!("Should be handled by clap itself"),
    }
}

fn init(args: &clap::ArgMatches) {
    let reset = args.get_flag("reset");
    database::init(reset);
}
