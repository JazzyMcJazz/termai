use crate::provider::ProviderSettings;

use super::{
    constants::{CHAT_SYSTEM_MESSAGE, EXPLAIN_SYSTEM_PROMT, SUGGEST_SYSTEM_PROMT},
    models::{ChatMessage, ChatRequest, ChatResponse},
    ChatRole,
};

const URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAiClient;

impl OpenAiClient {
    pub fn chat(messages: &[ChatMessage], settings: &ProviderSettings) -> String {
        let (api_key, model) = settings.get();

        let mut msg = Vec::from([ChatMessage {
            role: ChatRole::System,
            content: CHAT_SYSTEM_MESSAGE.into(),
            refusal: None,
        }]);

        msg.extend(messages.iter().cloned());

        let client = reqwest::blocking::Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                model,
                messages: msg,
            });

        let res: ChatResponse = match req.send() {
            Ok(res) => match res.json() {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    return String::new();
                }
            },
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                return String::new();
            }
        };

        if let Some(choice) = res.choices.first() {
            choice.message.content.clone()
        } else {
            eprintln!("No response from server");
            String::new()
        }
    }

    pub fn suggest(query: &str, settings: &ProviderSettings) -> String {
        let (api_key, model) = settings.get();

        let client = reqwest::blocking::Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                model,
                messages: vec![
                    ChatMessage {
                        role: ChatRole::System,
                        content: SUGGEST_SYSTEM_PROMT.into(),
                        refusal: None,
                    },
                    ChatMessage {
                        role: ChatRole::User,
                        content: query.into(),
                        refusal: None,
                    },
                ],
            });

        let res: ChatResponse = match req.send() {
            Ok(res) => match res.json() {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    return String::new();
                }
            },
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                return String::new();
            }
        };

        if let Some(choice) = res.choices.first() {
            choice.message.content.clone()
        } else {
            eprintln!("No response from server");
            String::new()
        }
    }

    pub fn explain(command: &str, settings: &ProviderSettings) -> String {
        let (api_key, model) = settings.get();

        let client = reqwest::blocking::Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                model,
                messages: vec![
                    ChatMessage {
                        role: ChatRole::System,
                        content: EXPLAIN_SYSTEM_PROMT.into(),
                        refusal: None,
                    },
                    ChatMessage {
                        role: ChatRole::User,
                        content: command.into(),
                        refusal: None,
                    },
                ],
            });

        let res: ChatResponse = match req.send() {
            Ok(res) => match res.json() {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to parse response: {}", e);
                    return String::new();
                }
            },
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                return String::new();
            }
        };

        if let Some(choice) = res.choices.first() {
            choice.message.content.clone()
        } else {
            eprintln!("No response from server");
            String::new()
        }
    }
}
