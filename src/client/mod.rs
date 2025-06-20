mod agents;
mod constants;
mod enums;
mod finish_reason;
mod models;
mod streaming;
mod traits;

use indicatif::ProgressBar;
use reqwest::{Client as Reqwest, RequestBuilder};
use rig::{
    agent::{Agent, AgentBuilder},
    client::CompletionClient,
    completion::{CompletionModel, PromptError},
    message::Message,
};
use serde::de::DeserializeOwned;

use crate::{mcp::McpClient, provider::Provider, utils::shell::detect_shell_environment};

use agents::{MultiTurnAgent, StreamingMultiTurnAgent};
use constants::{CHAT_PREAMBLE, EXPLAIN_PREAMBLE, SUGGEST_PREAMBLE};
use models::{anthropic, openai};
use traits::ModelTrait;

pub use agents::{StreamingContent, StreamingContentResult};

pub struct Client;

impl Client {
    pub async fn chat_stream(
        prompt: &str,
        messages: Vec<Message>,
        provider: &Provider,
        mcp_clients: &mut Vec<McpClient>,
        search: bool,
    ) -> StreamingContentResult {
        let stream = match provider {
            Provider::Anthropic(settings) => {
                let (_, api_key, cm, sm) = settings.get();

                let model = choose_model(search, &cm, &sm);

                let client = rig::providers::anthropic::ClientBuilder::new(&api_key).build();

                let agent_builder: AgentBuilder<
                    rig::providers::anthropic::completion::CompletionModel,
                > = client
                    .agent(&model)
                    .max_tokens(get_max_tokens(&model))
                    .preamble(CHAT_PREAMBLE);

                let agent = Self::build_agent(agent_builder, Some(mcp_clients), search).await;

                StreamingMultiTurnAgent::multi_turn_prompt(prompt, agent, messages.clone()).await
            }
            Provider::OpenAI(settings) => {
                let (_, api_key, cm, sm) = settings.get();

                let model = choose_model(search, &cm, &sm);

                let agent_builder = rig::providers::openai::Client::new(&api_key)
                    .agent(&model)
                    .preamble(CHAT_PREAMBLE);

                let agent = Self::build_agent(agent_builder, Some(mcp_clients), search).await;

                StreamingMultiTurnAgent::multi_turn_prompt(prompt, agent, messages.clone()).await
            }
        };

        stream
    }

    pub async fn chat(
        prompt: &str,
        messages: Vec<Message>,
        provider: &Provider,
        mcp_clients: &mut Vec<McpClient>,
        spinner: &ProgressBar,
        search: bool,
    ) -> Result<String, PromptError> {
        Self::chat_completion(
            prompt,
            CHAT_PREAMBLE,
            messages,
            provider,
            Some(mcp_clients),
            Some(spinner),
            search,
        )
        .await
    }

    pub async fn suggest(prompt: &str, provider: &Provider) -> Result<String, PromptError> {
        Self::chat_completion(
            prompt,
            SUGGEST_PREAMBLE,
            vec![],
            provider,
            None,
            None,
            false,
        )
        .await
    }

    pub async fn revise(
        prompt: &str,
        command_to_revise: &str,
        provider: &Provider,
    ) -> Result<String, PromptError> {
        let messages = vec![Message::assistant(command_to_revise)];
        Self::chat_completion(
            prompt,
            SUGGEST_PREAMBLE,
            messages,
            provider,
            None,
            None,
            false,
        )
        .await
    }

    pub async fn explain(prompt: &str, provider: &Provider) -> Result<String, PromptError> {
        Self::chat_completion(
            prompt,
            EXPLAIN_PREAMBLE,
            vec![],
            provider,
            None,
            None,
            false,
        )
        .await
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
        mcp_clients: Option<&mut Vec<McpClient>>,
        spinner: Option<&ProgressBar>,
        search: bool,
    ) -> Result<String, PromptError> {
        let mut preamble = preamble.to_string();

        if let Some((shell, os)) = detect_shell_environment() {
            preamble += format!("\n\nActive Shell: {} on {}", shell, os).as_str();
        }

        match provider {
            Provider::Anthropic(settings) => {
                let (_, api_key, cm, sm) = settings.get();

                let model = choose_model(search, &cm, &sm);

                let client = rig::providers::anthropic::ClientBuilder::new(&api_key).build();

                let agent_builder = client
                    .agent(&model)
                    .max_tokens(get_max_tokens(&model))
                    .preamble(&preamble);

                let agent = Self::build_agent(agent_builder, mcp_clients, search).await;

                let mut agent = MultiTurnAgent::new(agent, messages.clone());

                agent.multi_turn_prompt(prompt, spinner).await
            }
            Provider::OpenAI(settings) => {
                let (_, api_key, cm, sm) = settings.get();

                let model = choose_model(search, &cm, &sm);

                let agent_builder = rig::providers::openai::Client::new(&api_key)
                    .agent(&model)
                    .preamble(&preamble);

                let agent = Self::build_agent(agent_builder, mcp_clients, search).await;

                let mut agent = MultiTurnAgent::new(agent, messages.clone());

                agent.multi_turn_prompt(prompt, spinner).await
            }
        }
    }

    async fn build_agent<M: CompletionModel>(
        mut agent_builder: AgentBuilder<M>,
        mcp_clients: Option<&mut Vec<McpClient>>,
        skip_tools: bool,
    ) -> Agent<M> {
        if skip_tools {
            return agent_builder.build();
        }

        if let Some(clients) = mcp_clients {
            for client in clients {
                // Add tool from MCP client if enabled and initialized
                if client.is_enabled() && client.initialize().await.is_ok() {
                    agent_builder = client.add_tools(agent_builder).await;
                }
            }

            agent_builder.build()
        } else {
            agent_builder.build()
        }
    }

    fn build_models_request(provider: &Provider) -> RequestBuilder {
        let client = Reqwest::new();
        match provider {
            Provider::OpenAI(settings) => {
                let (base_url, api_key, _, _) = settings.get();
                let url = format!("{base_url}/v1/models");

                client
                    .get(url)
                    .header("Content-Type", "application/json")
                    .bearer_auth(api_key)
            }
            Provider::Anthropic(settings) => {
                let (base_url, api_key, _, _) = settings.get();
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
            Err(e) => return Err(e.to_string()),
        };

        res.extract_models()
    }
}

fn choose_model(search: bool, completion_model: &str, search_model: &str) -> String {
    if search {
        search_model.to_string()
    } else {
        completion_model.to_string()
    }
}

fn get_max_tokens(model: &str) -> u64 {
    match model {
        "claude-sonnet-4-20250514" | "claude-3-7-sonnet-20250219" => 64_000,
        "claude-opus-4-20250514" => 32_000,
        "claude-3-5-sonnet-20241022" => 8192,
        "claude-3-5-haiku-20241022" | "claude-3-opus-20240229" => 4096,
        _ => 4096,
    }
}
