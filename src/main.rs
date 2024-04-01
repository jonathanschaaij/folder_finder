use clap::{arg, command, Command};

pub mod auto_tags;
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
        .subcommand(collections::command())
        .subcommand(projects::command())
        .subcommand(tags::command())
        .get_matches();

    match matches.subcommand() {
        Some(("init", args)) => init(args),
        Some(("tag", args)) => tags::run(args),
        Some(("collection", args)) => collections::run(args),
        Some(("project", args)) => projects::run(args),
        _ => panic!("Should be handled by clap itself"),
    }
}

fn init(args: &clap::ArgMatches) {
    let reset = args.get_flag("reset");
    database::init(reset);
}
