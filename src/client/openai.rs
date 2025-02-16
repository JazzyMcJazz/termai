use std::io::BufReader;

use reqwest::blocking::{Client, RequestBuilder};

use crate::provider::ProviderSettings;

use super::{
    constants::{CHAT_SYSTEM_MESSAGE, EXPLAIN_SYSTEM_PROMT, SUGGEST_SYSTEM_PROMT},
    models::{AggregateChoice, ChatMessage, ChatRequest, ChatResponse, ContentIterator},
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

        let client = Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                stream: false,
                model,
                messages: msg,
            });

        Self::handle_request(req)
    }

    pub fn chat_stream(messages: &[ChatMessage], settings: &ProviderSettings) -> ContentIterator {
        let (api_key, model) = settings.get();

        let mut msg = Vec::from([ChatMessage {
            role: ChatRole::System,
            content: CHAT_SYSTEM_MESSAGE.into(),
            refusal: None,
        }]);

        msg.extend(messages.iter().cloned());

        let client = Client::new();
        let res = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                stream: true,
                model,
                messages: msg,
            })
            .send()
            .unwrap();

        let reader = BufReader::new(res);
        ContentIterator::new(reader)
    }

    pub fn suggest(query: &str, settings: &ProviderSettings) -> String {
        let (api_key, model) = settings.get();

        let client = reqwest::blocking::Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                stream: false,
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

        Self::handle_request(req)
    }

    pub fn revise(command_to_revise: &str, query: &str, settings: &ProviderSettings) -> String {
        let (api_key, model) = settings.get();

        let client = reqwest::blocking::Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                stream: false,
                model,
                messages: vec![
                    ChatMessage {
                        role: ChatRole::System,
                        content: SUGGEST_SYSTEM_PROMT.into(),
                        refusal: None,
                    },
                    ChatMessage {
                        role: ChatRole::Assistant,
                        content: command_to_revise.into(),
                        refusal: None,
                    },
                    ChatMessage {
                        role: ChatRole::User,
                        content: query.into(),
                        refusal: None,
                    },
                ],
            });

        Self::handle_request(req)
    }

    pub fn explain(command: &str, settings: &ProviderSettings) -> String {
        let (api_key, model) = settings.get();

        let client = reqwest::blocking::Client::new();
        let req = client
            .post(URL)
            .header("Content-Type", "application/json")
            .bearer_auth(api_key)
            .json(&ChatRequest {
                stream: false,
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

        Self::handle_request(req)
    }

    fn handle_request(req: RequestBuilder) -> String {
        let res: ChatResponse<AggregateChoice> = match req.send() {
            Ok(res) => match res.json() {
                Ok(json) => json,
                Err(e) => {
                    return format!("Failed to parse response: {}", e);
                }
            },
            Err(e) => {
                return format!("Failed to send request: {}", e);
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
