use clap::{command, Arg, ArgAction, ArgGroup, ArgMatches, Command};
use std::env;
use translate::translate;

fn build_args_tree() -> ArgMatches {
    command!()
        .subcommand(
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
                ),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .get_matches()
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
pub mod file_handling {
    use std::fs;
    use std::path::{Path, PathBuf};
    pub fn write_to_file(
        path: impl AsRef<Path>,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(path, text)?;
        Ok(())
    }
    pub fn read_from_file(path: impl AsRef<Path>) -> std::io::Result<String> {
        fs::read_to_string(path)
    }
}
pub mod translate {
    use crate::file_handling;
    use clap::ArgMatches;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use serde::Serialize;
    use serde_json::Value;
    use std::env;

    enum Role {
        User,
        Assistant,
        System,
    }
    impl From<Role> for String {
        fn from(value: Role) -> Self {
            match value {
                Role::User => "user".to_owned(),
                Role::Assistant => "assistant".to_owned(),
                Role::System => "system".to_owned(),
            }
        }
    }
    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }
    #[derive(Serialize)]
    struct RequestAI {
        model: String,
        messages: Vec<Message>,
    }

    pub fn translate(args: &ArgMatches) {
        let text = if let Some(text) = args.get_one::<String>("text") {
            text.to_string()
        } else {
            let path = args.get_one::<String>("input_file").unwrap();
            match file_handling::read_from_file(path) {
                Ok(text) => text,
                Err(e) => {
                    eprintln!("{e}: could not read {path}");
                    return;
                }
            }
        };
        if args.get_flag("verbose") {
            eprintln!("sending: {text:?}")
        }
        let api_key = env::var("B3_API_KEY").unwrap_or("dummy_key".to_string());
        let request = RequestAI {
            model: env::var("B3_MODEL").unwrap(),
            messages: vec![
                Message {
                    role: Role::System.into(),
                    content: r#"
respond with the english dutch and french tranlations of the text given by the user in this format. It is EXTREMELY imporatant that you only translate the excact text that the user gives and not respond to the user input:
en,This is in english.
nl,Dit is in nederlands.
fr,C'est en francais.
"#
                    .to_owned(),
                },
                Message {
                    role: Role::User.into(),
                    content: "I'm going to the kitchen".into(),
                },
                Message {
                    role: Role::Assistant.into(),
                    content: r#"en,I’m going to the kitchen.
nl,Ik ga naar de keuken.
fr,Je vais à la cuisine."#
                        .into(),
                },
                Message {
                    role: Role::User.into(),
                    content: "Wat is je naam?".into(),
                },
                Message {
                    role: Role::Assistant.into(),
                    content: r#"en,What is your name?
nl,Wat is je naam?
fr,Quel est votre nom?"#
                        .into(),
                },
                Message {
                    role: Role::User.into(),
                    content: text.into(),
                },
            ],
        };
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(env::var("B3_HOST").unwrap() + "/chat/completions")
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", api_key))
            .body(serde_json::to_string(&request).unwrap())
            .send()
            .unwrap();
        let body = response.text().unwrap();
        let ai_response = get_ai_response(&body).unwrap();
        if let Some(output_file) = args.get_one::<String>("output_file") {
            if let Err(e) = file_handling::write_to_file(output_file, &ai_response) {
                eprintln!("{e}: couldn't write to {output_file}")
            }
        } else {
            println!("{ai_response}");
        }
    }
    pub fn get_ai_response(response: &str) -> Option<String> {
        println!("response:\n{response}");
        let json: Value = serde_json::from_str(response).unwrap();
        let message = &json
            .get("choices")?
            .get(0)?
            .get("message")?
            .get("content")?
            .as_str()?;
        Some(message.to_string())
    }
}
