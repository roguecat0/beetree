use anyhow::bail;
use anyhow::Context;
use beetree::lang;
use beetree::translate;
use clap::{command, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use std::env;
use std::path::PathBuf;

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
    Command::new("lang")
        .about("transfers language translations to their respective files")
        .arg(Arg::new("new_var").help("the variable name the language tags will get"))
        .arg(Arg::new("text").help("text to be transfered"))
        .arg(
            Arg::new("input_file")
                .short('i')
                .long("input")
                .help("path to input file"),
        )
        .arg(
            Arg::new("base_path")
                .required(true)
                .short('b')
                .long("base_path")
                .help("path to branching language directory"),
        )
        .arg(
            Arg::new("append_file")
                .short('a')
                .long("append")
                .help("path to file (per language) to append translations to"),
        )
        .arg(
            Arg::new("find_var")
                .short('f')
                .long("find")
                .help("variable name that will be scanned.\nthe translations will be inserted the line before the var"),
        )
        .group(
            ArgGroup::new("search_type")
                .required(true)
                .args(["append_file", "find_var"]),
        )
        .group(
            ArgGroup::new("inputs")
                .required(true)
                .args(["text", "input_file"]),
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
        Ok(lang::Config {
            text: self.get_one::<String>("text").cloned(),
            input_file: self.get_one::<String>("input_file").cloned(),
            append_file: self.get_one::<String>("append_file").cloned(),
            find_var: self.get_one::<String>("find_var").cloned(),
            base_path: self
                .get_one::<String>("base_path")
                .cloned()
                .expect("guaranteed by clap"),
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
    // todo: change to warning message in verbose
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
            let config = args.to_config().map_err(|_e| anyhow::anyhow!("lol"))?;
            lang::run(config).map_err(|_e| anyhow::anyhow!("lol"))?;
        }
        Some(subcommand) => println!("{subcommand:?}"),
        None => {}
    }
    Ok(())
}
