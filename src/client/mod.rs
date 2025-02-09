use crate::provider::Provider;

pub use models::{ChatMessage, ChatRole};

mod constants;
mod models;
mod openai;

pub struct Client;

impl Client {
    pub fn chat(messages: &[ChatMessage], provider: &Provider) -> String {
        match provider {
            Provider::OpenAI(settings) => openai::OpenAiClient::chat(messages, settings),
        }
    }
    pub fn suggest(query: &str, provider: &Provider) -> String {
        match provider {
            Provider::OpenAI(settings) => openai::OpenAiClient::suggest(query, settings),
        }
    }
    pub fn explain(command: &str, provider: &Provider) -> String {
        match provider {
            Provider::OpenAI(settings) => openai::OpenAiClient::explain(command, settings),
        }
    }
}
