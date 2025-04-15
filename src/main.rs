use beetree::translate::{self, Config as TranslateConfig};
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
impl ToConfig<TranslateConfig> for ArgMatches {
    type Error = &'static str;
    fn to_config(&self) -> Result<TranslateConfig, Self::Error> {
        let api_key = env::var("B3_API_KEY").unwrap_or(String::from("dummy_key"));
        let model = env::var("B3_MODEL").map_err(|_| "no B3_MODEL env variable")?;
        let host = env::var("B3_HOST").map_err(|_| "no host env variable")?;
        let text = self.get_one::<String>("text").cloned();
        let input_file = self.get_one::<String>("input_file").cloned();
        let output_file = self.get_one::<String>("output_file").cloned();
        let verbose = self.get_flag("verbose");
        Ok(TranslateConfig {
            api_key,
            model,
            host,
            text,
            input_file,
            output_file,
            verbose,
        })
    }
}
trait ToConfig<T> {
    type Error;
    fn to_config(&self) -> Result<T, Self::Error>;
}

fn main() {
    let matches = build_args_tree();
    if let Err(_) = dotenvy::dotenv() {
        return;
    }
    match matches.subcommand() {
        Some(("translate", args)) => {
            let config = match args.to_config() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("error: {e}");
                    return;
                }
            };
            translate::run(config);
        }
        Some(subcommand) => println!("{subcommand:?}"),
        None => {}
    }
}
