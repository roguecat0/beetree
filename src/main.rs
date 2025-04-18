use crate::config_parse::ToConfig;
use beetree::translate;
use beetree::{lang, Input};
use clap::error::ErrorKind;
use clap::{arg, command, value_parser, Arg, ArgAction, Command};
use std::env;
use std::io::IsTerminal;
use std::path::PathBuf;
mod config_parse;

fn cli() -> Command {
    command!()
        .arg_required_else_help(true)
        .about("general utily cli for working on the beetree webapplication\nreads .env files")
        .subcommand_required(true)
        .subcommand(build_translate_command())
        .subcommand(build_lang_command())
        .arg(
            arg!(-v --verbose "Execute in verbose mode")
                .global(true)
                .action(ArgAction::SetTrue),
        )
}
fn build_translate_command() -> Command {
    Command::new("translate")
        .about("translates input to (en,nl,fr)\n")
        .arg(
            arg!([text] "text to be tranlated\nadd '-' when passing through stdin")
                .required_unless_present("input_file"),
        )
        .arg(
            arg!(output_file: -o --output <FILE> "path to output file")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(input_file: -i --input <FILE> "path to input file")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--host <ADDR> "the address of the server running the llm")
                .env("B3_HOST")
                .required(true),
        )
        .arg(
            arg!(api_key: --"api-key" <KEY> "api key for llm server")
                .env("B3_KEY")
                .default_value("dummy_key"),
        )
        .arg(
            arg!(--model <MODEL> "chosen model")
                .env("B3_MODEL")
                .required(true),
        )
        .arg(
            arg!(--languages <LANGS> "list of the languages to translate to")
                .env("B3_LANGUAGES")
                .default_value("nl,fr,en"),
        )
}
fn build_lang_command() -> Command {
    let src_tag = arg!([source_tag] "tag name of the created language binding");
    let dest_tag = arg!([destination_tag] "tag to be searched for");
    let input_file = arg!(input_file: -i --input <FILE> "path to input file")
        .value_parser(value_parser!(PathBuf));
    let _output_file = arg!(output_file: -o --output <FILE> "path to output file")
        .value_parser(value_parser!(PathBuf));
    let search_file =
        arg!(search_file: -f --file <FILE> "path to file (per language) to specify search.")
            .value_parser(value_parser!(PathBuf));
    let text = arg!([text] "tranlations to be parsed to chosen location")
        .required_unless_present("input_file");

    Command::new("lang")
        .about("transfers language translations to their respective files")
        .subcommand_negates_reqs(true)
        .arg(
            arg!(base_path: --"base" <DIR> "path to branching language directory")
                .default_value(".")
                .global(true)
                .value_parser(value_parser!(PathBuf))
        )
        .arg(&src_tag)
        .arg(&search_file)
        .arg(
            Arg::new("prepend_var")
                .short('p')
                .long("prepend")
                .help("the translations will be inserted the line before the variable"),
        )
        .subcommand(Command::new("append")
            .about("append the translations to the chosen file")
            .arg(src_tag.clone().required(true))
            .arg(&text)
            .arg(&input_file)
            .arg(search_file.clone().required(true))
        )
        .subcommand(Command::new("insert")
            .about("inserts the translations before the destination tag")
            .arg(src_tag.clone().required(true))
            .arg(dest_tag.clone().required(true))
            .arg(&text)
            .arg(&input_file)
            .arg(&search_file)
        )
        .subcommand(Command::new("remove")
            .about("deletes the variable of the file it appears in\nonly one line variables supported\nonly deletes first appearance")
            .arg(&search_file)
            .arg(arg!(--languages <LANGS> "list of the languages to translate to")
                .env("B3_LANGUAGES")
                .default_value("nl,fr,en"))
            .arg(dest_tag.clone().required(true))
        )
}
fn get_terminal_pipe_input(cmd: &mut Command, arg_id: &str, text: String) -> String {
    if text == "-" {
        if !std::io::stdin().is_terminal() {
            std::io::read_to_string(std::io::stdin()).unwrap()
        } else {
            cmd.error(
                ErrorKind::ArgumentConflict,
                format!("needs to pipe text through stdin when '-' provided to [{arg_id}]"),
            )
            .exit();
        }
    } else {
        text
    }
}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let mut cmd = cli();
    let matches = cmd.get_matches_mut();
    match matches.subcommand() {
        Some(("translate", args)) => {
            let cmd = cmd.find_subcommand_mut("translate").expect("curr scmd");
            let mut config: translate::Config = args.to_config()?;
            if let Input::Text(text) = config.input {
                let text = get_terminal_pipe_input(cmd, "text", text);
                config.input = Input::Text(text);
            }
            translate::run(config)?;
        }
        Some(("lang", args)) => {
            let cmd = cmd.find_subcommand_mut("lang").expect("curr scmd");
            match args.subcommand() {
                Some(("remove", args)) => {
                    let config: lang::RemoveConfig = args.to_config()?;
                    lang::remove(config)?;
                }
                Some(("append", args)) => {
                    let cmd = cmd.find_subcommand_mut("append").expect("curr scmd");
                    let mut config: lang::AppendConfig = args.to_config()?;
                    if let Input::Text(text) = config.input {
                        let text = get_terminal_pipe_input(cmd, "text", text);
                        config.input = Input::Text(text);
                    }
                    lang::append(config)?;
                }
                Some(("insert", args)) => {
                    let cmd = cmd.find_subcommand_mut("insert").expect("curr scmd");
                    let mut config: lang::InsertConfig = args.to_config()?;
                    if let Input::Text(text) = config.input {
                        let text = get_terminal_pipe_input(cmd, "text", text);
                        config.input = Input::Text(text);
                    }
                    lang::insert(config)?;
                }
                _ => todo!(),
            }
        }
        Some((subcommand, _)) => panic!("clap handles invaled subommand: {subcommand:?}"),
        None => {}
    }
    Ok(())
}
