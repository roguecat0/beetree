use beetree::translate::translate;
use clap::{command, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use std::env;

fn build_args_tree() -> ArgMatches {
    command!()
        .subcommand(build_translate_command())
        .subcommand(build_lang_command())
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .get_matches()
}
fn build_translate_command() -> Command {
    Command::new("translate")
        .about("translates input to (en,nl,fr)")
        .arg(Arg::new("text").help("text to be tranlated"))
        .arg(
            Arg::new("output_file")
                .short('o')
                .long("output")
                .help("path to output file"),
        )
        .arg(
            Arg::new("input_file")
                .short('i')
                .long("input")
                .help("path to input file"),
        )
        .group(
            ArgGroup::new("inputs")
                .required(true)
                .args(["text", "input_file"]),
        )
}
fn build_lang_command() -> Command {
    Command::new("lang").about("transfers language translations to their respective files")
}

fn main() {
    let matches = build_args_tree();
    if let Err(_) = dotenvy::dotenv() {
        return;
    }
    match matches.subcommand() {
        Some(("translate", args)) => {
            translate(args);
        }
        Some(subcommand) => println!("{subcommand:?}"),
        None => {}
    }
}
