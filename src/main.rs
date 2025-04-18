use anyhow::{bail, Context};
use beetree::lang::{Action, FindSpecified};
use beetree::translate;
use beetree::{lang, Input};
use clap::error::ErrorKind;
use clap::{arg, command, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use std::env;
use std::io::IsTerminal;
use std::path::PathBuf;

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
        .arg(arg!(output_file: -o --output <FILE> "path to output file"))
        .arg(arg!(input_file: -i --input <FILE> "path to input file"))
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
    let group = ArgGroup::new("actions").args(["prepend_var"]);
    let new_var = Arg::new("new_var").help("the variable name the language tags will get");
    let old_var = Arg::new("old_var").help("variable to be searched for");
    let input_file = Arg::new("input_file")
        .short('i')
        .long("input")
        .help("path to input file");
    let search_file = Arg::new("search_file")
        .short('f')
        .long("file")
        .help("path to file (per language) to specify seach.\nwill append to this file if no actions flags included");

    Command::new("lang")
        .about("transfers language translations to their respective files")
        .subcommand_negates_reqs(true)
        .arg(
            Arg::new("base_path")
                .short('b')
                .long("base_path")
                .default_value(".")
                .global(true)
                .value_parser(value_parser!(PathBuf))
                .help("path to branching language directory"),
        )
        .arg(&new_var)
        .arg(input_file)
        .arg(&search_file)
        .arg(
            Arg::new("prepend_var")
                .short('p')
                .long("prepend")
                .help("the translations will be inserted the line before the variable"),
        )
        .group(group)
        .subcommand(Command::new("remove")
            .about("deletes the variable of the file it appears in\nonly one line variables supported\nonly deletes first appearance")
            .arg(&search_file)
            .arg(Arg::new("languages").required(true).help("comma separated list of the languages the file are split into"))
            .arg(old_var.clone().required(true))
        )
}
trait ToConfig<T> {
    type Error;
    fn to_config(&self) -> Result<T, Self::Error>;
}
impl ToConfig<translate::Config> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<translate::Config, Self::Error> {
        let api_key = self
            .get_one::<String>("api_key")
            .expect("default")
            .to_string();
        let model = self
            .get_one::<String>("model")
            .expect("required")
            .to_string();
        let host = self
            .get_one::<String>("host")
            .expect("required")
            .to_string();
        let output_file = self.get_one::<String>("output_file").cloned();
        let input = if let Some(text) = self.get_one::<String>("text") {
            beetree::Input::Text(text.to_string())
        } else {
            let file = self.get_one::<String>("input_file").expect("clap handles");
            let file = PathBuf::from(file);
            beetree::Input::File(file)
        };
        let verbose = self.get_flag("verbose");
        let languages = self
            .get_one::<String>("languages")
            .expect("required")
            .to_string();
        Ok(translate::Config {
            api_key,
            model,
            languages,
            host,
            input,
            output_file,
            verbose,
        })
    }
}
impl ToConfig<lang::Config> for ArgMatches {
    type Error = &'static str;
    fn to_config(&self) -> Result<lang::Config, Self::Error> {
        let search_file = self.get_one::<String>("search_file").map(|s| s.into());
        let action = if let Some(needle) = self.get_one::<String>("prepend_var") {
            Action::PrependFile(FindSpecified {
                needle: needle.to_string(),
                file: search_file,
            })
        } else {
            Action::Append(search_file.expect("guaranteed by clap"))
        };
        Ok(lang::Config {
            text: self.get_one::<String>("text").cloned(),
            input_file: self.get_one::<String>("input_file").map(|s| s.into()),
            action,
            base_path: self
                .get_one::<String>("base_path")
                .expect("guaranteed by clap")
                .into(),
            new_var: self
                .get_one::<String>("new_var")
                .cloned()
                .expect("guaranteed by clap"),
            verbose: self.get_flag("verbose"),
        })
    }
}
fn get_terminal_pipe_input(
    cmd: &mut Command,
    args: &ArgMatches,
    arg_id: &str,
) -> anyhow::Result<String> {
    let input = args
        .get_one::<String>(arg_id)
        .ok_or(anyhow::anyhow!("arg id {arg_id} not present"))?;
    let text = if input == "-" {
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
        input.to_string()
    };
    Ok(text)
}

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let mut cmd = cli();
    let matches = cmd.get_matches_mut();
    match matches.subcommand() {
        Some(("translate", args)) => {
            let mut config: translate::Config = args.to_config()?;
            if let None = args.get_one::<String>("input_file") {
                let text = get_terminal_pipe_input(
                    &mut cmd.find_subcommand_mut("translate").expect("is subcommand"),
                    args,
                    "text",
                )
                .expect("required");
                config.input = Input::Text(text);
            }
            translate::run(config)?;
        }
        Some(("lang", args)) => {
            println!("args: {args:?}");
            bail!("end");
            let config = if let Some((_, subargs)) = args.subcommand() {
                subargs
                    .to_config()
                    .map_err(|e| anyhow::anyhow!("error: {e}"))?
            } else {
                args.to_config()
                    .map_err(|e| anyhow::anyhow!("error: {e}"))?
            };
            lang::run(config).map_err(|e| anyhow::anyhow!("error: {e}"))?;
        }
        Some((subcommand, _)) => panic!("clap handles invaled subommand: {subcommand:?}"),
        None => {}
    }
    Ok(())
}
