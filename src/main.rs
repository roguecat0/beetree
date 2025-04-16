use anyhow::{bail, Context};
use beetree::lang;
use beetree::lang::{Action, FindSpecified};
use beetree::translate;
use clap::{command, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use std::env;
use std::path::PathBuf;

fn build_args_tree() -> ArgMatches {
    command!()
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(build_translate_command())
        .subcommand(build_lang_command())
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("verbose mode"),
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
    let group = ArgGroup::new("actions").args(["prepend_var"]);
    let new_var = Arg::new("new_var")
                .help("the variable name the language tags will get");
    let old_var = Arg::new("old_var")
                .help("variable to be searched for");
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
impl ToConfig<translate::Config> for ArgMatches {
    type Error = anyhow::Error;
    fn to_config(&self) -> Result<translate::Config, Self::Error> {
        let api_key = env::var("B3_API_KEY").unwrap_or(String::from("dummy_key"));
        let model = env::var("B3_MODEL").with_context(|| "No B3_MODEL")?;
        let host = env::var("B3_HOST").with_context(|| "no B3_HOST")?;
        let output_file = self.get_one::<String>("output_file").cloned();
        let input = if let Some(text) = self.get_one::<String>("text") {
            beetree::Input::Text(text.to_string())
        } else {
            let file = self.get_one::<String>("input_file").expect("clap handles");
            let file = PathBuf::from(file);
            beetree::Input::File(file)
        };
        let verbose = self.get_flag("verbose");
        Ok(translate::Config {
            api_key,
            model,
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
trait ToConfig<T> {
    type Error;
    fn to_config(&self) -> Result<T, Self::Error>;
}

fn main() -> anyhow::Result<()> {
    let matches = build_args_tree();
    if let (Err(_), true) = (dotenvy::dotenv(), matches.get_flag("verbose")) {
        eprintln!("no .env file found");
    }
    match matches.subcommand() {
        Some(("translate", args)) => {
            let config = args.to_config()?;
            translate::run(config)?;
        }
        Some(("lang", args)) => {
            println!("args: {args:?}");
            bail!("end");
            let config = if let Some((_,subargs)) = args.subcommand() {
                subargs
                    .to_config()
                    .map_err(|e| anyhow::anyhow!("error: {e}"))?
            } else {
                args
                    .to_config()
                    .map_err(|e| anyhow::anyhow!("error: {e}"))?
            };
            lang::run(config).map_err(|e| anyhow::anyhow!("error: {e}"))?;
        }
        Some(subcommand) => println!("sub command: {subcommand:?} not supported"),
        None => {}
    }
    Ok(())
}

