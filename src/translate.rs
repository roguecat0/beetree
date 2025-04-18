use crate::{file_handling, Input};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    FileError(#[from] file_handling::Error),
    #[error("invalid ai response: {0}")]
    AiResponse(String),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

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

pub struct Config {
    pub host: String,
    pub api_key: String,
    pub model: String,
    pub input: Input,
    pub output_file: Option<String>,
    pub languages: String,
    pub verbose: bool,
}

pub fn run(config: Config) -> Result<(), Error> {
    let text = match config.input {
        Input::File(path) => file_handling::read_from_file(&path)?,
        Input::Text(text) => text,
    };
    if config.verbose {
        eprintln!("sending: {text:?}")
    }
    let api_key = config.api_key;
    let request = RequestAI {
            model: config.model,
            messages: vec![
                Message {
                    role: Role::System.into(),
                    content: r#"
respond with the translation of the user inputed text, in the languages given by the user, represented by a list of abreviations. 
For example languages:en,nl,fr would represent english, dutch and french. and you need to provide tranlations of the text given by the user in this format. 
It is EXTREMELY imporatant that you only translate the excact text that the user gives and not respond to the user input:
en,This is in english.
nl,Dit is in nederlands.
fr,C'est en francais.
"#
                    .to_owned(),
                },
                Message {
                    role: Role::User.into(),
                    content: "languages:en,nl,fr\nI'm going to the kitchen".into(),
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
                    content: "languages:nl,fr\nWat is je naam?".into(),
                },
                Message {
                    role: Role::Assistant.into(),
                    content: r#"nl,Wat is je naam?
fr,Quel est votre nom?"#
                        .into(),
                },
                Message {
                    role: Role::User.into(),
                    content: format!("languages:{}\n{text}",config.languages),
                },
            ],
        };
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(config.host + "/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body(serde_json::to_string(&request)?)
        .send()?;
    let body = response.text()?;
    let ai_response = get_ai_response(&body)?;
    if let Some(output_file) = config.output_file {
        file_handling::write_to_file(&output_file, &ai_response)?;
    } else {
        println!("{ai_response}");
    }
    Ok(())
}
pub fn get_ai_response(response: &str) -> Result<String, Error> {
    let json: Value = serde_json::from_str(response)?;
    let message = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or(Error::AiResponse(response.to_owned()))?;
    Ok(message.to_string())
}
