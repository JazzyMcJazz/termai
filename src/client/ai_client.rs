use std::io::BufReader;

use reqwest::blocking::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::provider::Provider;

use super::{
    constants::{CHAT_SYSTEM_MESSAGE, EXPLAIN_SYSTEM_PROMT, SUGGEST_SYSTEM_PROMT},
    models::{
        anthropic, openai,
        shared::{ChatMessage, ChatRole, ContentIterator},
    },
    traits::{ContentTrait, ModelTrait},
};

pub struct AIClient;

impl AIClient {
    pub fn chat(messages: &[ChatMessage], provider: &Provider) -> String {
        let req = Self::build_chat_request(messages, CHAT_SYSTEM_MESSAGE, provider, false);

        match provider {
            Provider::OpenAI(_) => {
                Self::handle_chat_request::<openai::ChatResponse<openai::AggregateChoice>>(req)
            }
            Provider::Anthropic(_) => Self::handle_chat_request::<anthropic::ChatResponse>(req),
        }
    }

    pub fn chat_stream<'a>(
        messages: &[ChatMessage],
        provider: &'a Provider,
    ) -> ContentIterator<'a> {
        let req = Self::build_chat_request(messages, CHAT_SYSTEM_MESSAGE, provider, true);

        let res = req.send().expect("Failed to send request");

        let reader = BufReader::new(res);
        ContentIterator::new(reader, provider)
    }

    pub fn suggest(query: &str, provider: &Provider) -> String {
        let req = Self::build_chat_request(
            &[ChatMessage {
                role: ChatRole::User,
                content: query.into(),
            }],
            SUGGEST_SYSTEM_PROMT,
            provider,
            false,
        );

        match provider {
            Provider::OpenAI(_) => {
                Self::handle_chat_request::<openai::ChatResponse<openai::AggregateChoice>>(req)
            }
            Provider::Anthropic(_) => Self::handle_chat_request::<anthropic::ChatResponse>(req),
        }
    }

    pub fn revise(command_to_revise: &str, query: &str, provider: &Provider) -> String {
        let req = Self::build_chat_request(
            &[
                ChatMessage {
                    role: ChatRole::System,
                    content: SUGGEST_SYSTEM_PROMT.into(),
                },
                ChatMessage {
                    role: ChatRole::Assistant,
                    content: command_to_revise.into(),
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: query.into(),
                },
            ],
            CHAT_SYSTEM_MESSAGE,
            provider,
            false,
        );

        match provider {
            Provider::OpenAI(_) => {
                Self::handle_chat_request::<openai::ChatResponse<openai::AggregateChoice>>(req)
            }
            Provider::Anthropic(_) => Self::handle_chat_request::<anthropic::ChatResponse>(req),
        }
    }

    pub fn explain(command: &str, provider: &Provider) -> String {
        let req = Self::build_chat_request(
            &[ChatMessage {
                role: ChatRole::User,
                content: command.into(),
            }],
            EXPLAIN_SYSTEM_PROMT,
            provider,
            false,
        );

        match provider {
            Provider::OpenAI(_) => {
                Self::handle_chat_request::<openai::ChatResponse<openai::AggregateChoice>>(req)
            }
            Provider::Anthropic(_) => Self::handle_chat_request::<anthropic::ChatResponse>(req),
        }
    }

    pub fn fetch_models(provider: &Provider) -> Vec<(String, String)> {
        let req = Self::build_models_request(provider);

        let res = match provider {
            Provider::OpenAI(_) => Self::handle_models_request::<openai::ModelResponse>(req),
            Provider::Anthropic(_) => Self::handle_models_request::<anthropic::ModelResponse>(req),
        };

        match res {
            Ok(models) => models,
            Err(e) => {
                eprintln!("{} error: {}", provider.name(), e);
                Vec::new()
            }
        }
    }

    fn build_chat_request(
        messages: &[ChatMessage],
        system_message: &str,
        provider: &Provider,
        stream: bool,
    ) -> RequestBuilder {
        let client = Client::new();
        match provider {
            Provider::OpenAI(settings) => {
                let (base_url, api_key, model) = settings.get();
                let url = format!("{base_url}/v1/chat/completions");

                let mut msg = Vec::from([ChatMessage {
                    role: ChatRole::System,
                    content: system_message.into(),
                }]);

                msg.extend(messages.iter().cloned());

                client
                    .post(url)
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
                let url = format!("{base_url}/v1/messages");

                client
                    .post(url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&json!({
                        "max_tokens": 4096,
                        "stream": stream,
                        "model": model,
                        "system": system_message,
                        "messages": messages,
                    }))
            }
        }
    }

    fn handle_chat_request<T: ContentTrait + DeserializeOwned>(req: RequestBuilder) -> String {
        let res: T = match req.send() {
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

        if let Some(content) = res.extract_content() {
            content
        } else {
            eprintln!("No response from server");
            String::new()
        }
    }

    fn build_models_request(provider: &Provider) -> RequestBuilder {
        let client = Client::new();
        match provider {
            Provider::OpenAI(settings) => {
                let (base_url, api_key, _) = settings.get();
                let url = format!("{base_url}/v1/models");

                client
                    .get(url)
                    .header("Content-Type", "application/json")
                    .bearer_auth(api_key)
            }
            Provider::Anthropic(settings) => {
                let (base_url, api_key, _) = settings.get();
                let url = format!("{base_url}/v1/models");

                client
                    .get(url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
            }
        }
    }

    fn handle_models_request<T: ModelTrait + DeserializeOwned>(
        req: RequestBuilder,
    ) -> Result<Vec<(String, String)>, String> {
        // let res = req.send().expect("Failed to send request");
        // let text = res.text().expect("Failed to parse response");
        // dbg!(text);

        // return Ok(Vec::new());

        let res: T = match req.send() {
            Ok(res) => match res.json() {
                Ok(json) => json,
                Err(_) => {
                    return Err("failed to parse response".to_string());
                }
            },
            Err(e) => {
                eprintln!("Failed to send request: {}", e);
                return Err("failed to send request".to_string());
            }
        };

        res.extract_models()
    }
}
