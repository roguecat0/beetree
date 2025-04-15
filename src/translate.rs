use crate::file_handling;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;
use serde_json::Value;

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
    pub text: Option<String>,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub verbose: bool,
}

pub fn run(config: Config) {
    let text = if let Some(text) = config.text {
        text.to_string()
    } else {
        let path = config.input_file.unwrap();
        match file_handling::read_from_file(&path) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("{e}: could not read {path}");
                return;
            }
        }
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
        .post(config.host + "/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .body(serde_json::to_string(&request).unwrap())
        .send()
        .unwrap();
    let body = response.text().unwrap();
    let ai_response = get_ai_response(&body).unwrap();
    if let Some(output_file) = config.output_file {
        if let Err(e) = file_handling::write_to_file(&output_file, &ai_response) {
            eprintln!("{e}: couldn't write to {output_file}")
        }
    } else {
        println!("{ai_response}");
    }
}
pub fn get_ai_response(response: &str) -> Option<String> {
    let json: Value = serde_json::from_str(response).unwrap();
    let message = &json
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()?;
    Some(message.to_string())
}
