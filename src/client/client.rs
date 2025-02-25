use std::io::BufReader;

use reqwest::blocking::{Client, RequestBuilder};
use serde_json::json;

use crate::provider::Provider;

use super::{
    constants::{CHAT_SYSTEM_MESSAGE, EXPLAIN_SYSTEM_PROMT, SUGGEST_SYSTEM_PROMT},
    models::{AggregateChoice, ChatMessage, ChatResponse, ContentIterator},
    ChatRole,
};

pub struct AIClient;

impl AIClient {
    pub fn chat(messages: &[ChatMessage], provider: &Provider) -> String {
        let req = Self::build_request(messages, CHAT_SYSTEM_MESSAGE, provider, false);
        Self::handle_request(req)
    }

    pub fn chat_stream<'a>(
        messages: &[ChatMessage],
        provider: &'a Provider,
    ) -> ContentIterator<'a> {
        let req = Self::build_request(messages, CHAT_SYSTEM_MESSAGE, provider, true);

        let res = req.send().expect("Failed to send request");

        let reader = BufReader::new(res);
        ContentIterator::new(reader, provider)
    }

    pub fn suggest(query: &str, provider: &Provider) -> String {
        let req = Self::build_request(
            &[ChatMessage {
                role: ChatRole::User,
                content: query.into(),
                refusal: None,
            }],
            SUGGEST_SYSTEM_PROMT,
            provider,
            false,
        );

        Self::handle_request(req)
    }

    pub fn revise(command_to_revise: &str, query: &str, provider: &Provider) -> String {
        let req = Self::build_request(
            &[
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
            CHAT_SYSTEM_MESSAGE,
            provider,
            false,
        );

        Self::handle_request(req)
    }

    pub fn explain(command: &str, provider: &Provider) -> String {
        let req = Self::build_request(
            &[ChatMessage {
                role: ChatRole::User,
                content: command.into(),
                refusal: None,
            }],
            EXPLAIN_SYSTEM_PROMT,
            provider,
            false,
        );

        Self::handle_request(req)
    }

    fn build_request(
        messages: &[ChatMessage],
        system_message: &str,
        provider: &Provider,
        stream: bool,
    ) -> RequestBuilder {
        let client = Client::new();
        match provider {
            Provider::OpenAI(settings) => {
                let (base_url, api_key, model) = settings.get();

                let mut msg = Vec::from([ChatMessage {
                    role: ChatRole::System,
                    content: system_message.into(),
                    refusal: None,
                }]);

                msg.extend(messages.iter().cloned());

                client
                    .post(base_url)
                    .header("Content-Type", "application/json")
                    .bearer_auth(api_key)
                    .json(&json!({
                        "stream": stream,
                        "model": model,
                        "messages": msg,
                    }))
            }
            Provider::Anthropic(settings) => {
                let (base_url, api_key, model) = settings.get();

                client
                    .post(base_url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&json!({
                        "stream": stream,
                        "model": model,
                        "system": system_message,
                        "messages": messages,
                    }))
            }
        }
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
