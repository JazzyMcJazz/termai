use reqwest::{Client, RequestBuilder};
use rig::{
    completion::Chat,
    message::Message,
    streaming::{StreamingChat, StreamingResult},
};
use serde::de::DeserializeOwned;

use crate::{provider::Provider, utils::shell::detect_shell_environment};

use super::{
    constants::{CHAT_PREAMBLE, EXPLAIN_PREAMBLE, SUGGEST_PREAMBLE},
    models::{anthropic, openai},
    traits::ModelTrait,
};

pub struct AIClient;

impl AIClient {
    pub async fn chat_stream(
        prompt: &str,
        messages: Vec<Message>,
        provider: &Provider,
    ) -> StreamingResult {
        let stream = match provider {
            Provider::Anthropic(settings) => {
                let (_, api_key, model) = settings.get();

                let client = rig::providers::anthropic::ClientBuilder::new(&api_key).build();

                let agent = client.agent(&model).preamble(CHAT_PREAMBLE).build();

                agent.stream_chat(prompt, messages).await.unwrap()
            }
            Provider::OpenAI(settings) => {
                let (_, api_key, model) = settings.get();

                let agent = rig::providers::openai::Client::new(&api_key)
                    .agent(&model)
                    .preamble(CHAT_PREAMBLE)
                    .build();

                agent.stream_chat(prompt, messages).await.unwrap()
            }
        };

        stream
    }

    pub async fn chat(prompt: &str, messages: Vec<Message>, provider: &Provider) -> String {
        Self::chat_completion(prompt, CHAT_PREAMBLE, messages, provider).await
    }

    pub async fn suggest(prompt: &str, provider: &Provider) -> String {
        Self::chat_completion(prompt, SUGGEST_PREAMBLE, vec![], provider).await
    }

    pub async fn revise(prompt: &str, command_to_revise: &str, provider: &Provider) -> String {
        let messages = vec![Message::assistant(command_to_revise)];
        Self::chat_completion(prompt, SUGGEST_PREAMBLE, messages, provider).await
    }

    pub async fn explain(prompt: &str, provider: &Provider) -> String {
        Self::chat_completion(prompt, EXPLAIN_PREAMBLE, vec![], provider).await
    }

    pub async fn fetch_models(provider: &Provider) -> Vec<(String, String)> {
        let req = Self::build_models_request(provider);

        let res = match provider {
            Provider::OpenAI(_) => Self::handle_models_request::<openai::ModelResponse>(req).await,
            Provider::Anthropic(_) => {
                Self::handle_models_request::<anthropic::ModelResponse>(req).await
            }
        };

        match res {
            Ok(models) => models,
            Err(e) => {
                eprintln!("{} error: {}", provider.name(), e);
                Vec::new()
            }
        }
    }

    async fn chat_completion(
        prompt: &str,
        preamble: &str,
        messages: Vec<Message>,
        provider: &Provider,
    ) -> String {
        let mut preamble = preamble.to_string();

        if let Some((shell, os)) = detect_shell_environment() {
            preamble += format!("\n\nActive Shell: {} on {}", shell, os).as_str();
        }

        match provider {
            Provider::Anthropic(settings) => {
                let (_, api_key, model) = settings.get();

                let client = rig::providers::anthropic::ClientBuilder::new(&api_key).build();

                let agent = client.agent(&model).preamble(&preamble).build();

                agent.chat(prompt, messages).await.unwrap()
            }
            Provider::OpenAI(settings) => {
                let (_, api_key, model) = settings.get();

                let agent = rig::providers::openai::Client::new(&api_key)
                    .agent(&model)
                    .preamble(&preamble)
                    .build();

                agent.chat(prompt, messages).await.unwrap()
            }
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

    async fn handle_models_request<T: ModelTrait + DeserializeOwned>(
        req: RequestBuilder,
    ) -> Result<Vec<(String, String)>, String> {
        let res: T = match req.send().await {
            Ok(res) => match res.json().await {
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

// async fn get_tools() -> Option<Vec<rmcp::model::Tool>> {
//     let command = "deno";
//     let args = vec!["-A", "/home/lr/Development/mcp-playground/server/main.ts", "--stdio"];
//     let client = crate::mcp::McpClient::new(command, args).await;
//     let tools = client.tools().await;
//     client.cancel().await;

//     tools
// }
