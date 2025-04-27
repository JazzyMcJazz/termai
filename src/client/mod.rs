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
    ) -> StreamingContentResult {
        let stream = match provider {
            Provider::Anthropic(settings) => {
                let (_, api_key, model) = settings.get();

                let client = rig::providers::anthropic::ClientBuilder::new(&api_key).build();

                let agent_builder: AgentBuilder<
                    rig::providers::anthropic::completion::CompletionModel,
                > = client.agent(&model).preamble(CHAT_PREAMBLE);

                let agent = Self::build_agent(agent_builder, Some(mcp_clients)).await;

                StreamingMultiTurnAgent::multi_turn_prompt(prompt, agent, messages.clone()).await
            }
            Provider::OpenAI(settings) => {
                let (_, api_key, model) = settings.get();

                let agent_builder = rig::providers::openai::Client::new(&api_key)
                    .agent(&model)
                    .preamble(CHAT_PREAMBLE);

                let agent = Self::build_agent(agent_builder, Some(mcp_clients)).await;

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
    ) -> Result<String, PromptError> {
        Self::chat_completion(
            prompt,
            CHAT_PREAMBLE,
            messages,
            provider,
            Some(mcp_clients),
            Some(spinner),
        )
        .await
    }

    pub async fn suggest(prompt: &str, provider: &Provider) -> Result<String, PromptError> {
        Self::chat_completion(prompt, SUGGEST_PREAMBLE, vec![], provider, None, None).await
    }

    pub async fn revise(
        prompt: &str,
        command_to_revise: &str,
        provider: &Provider,
    ) -> Result<String, PromptError> {
        let messages = vec![Message::assistant(command_to_revise)];
        Self::chat_completion(prompt, SUGGEST_PREAMBLE, messages, provider, None, None).await
    }

    pub async fn explain(prompt: &str, provider: &Provider) -> Result<String, PromptError> {
        Self::chat_completion(prompt, EXPLAIN_PREAMBLE, vec![], provider, None, None).await
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
    ) -> Result<String, PromptError> {
        let mut preamble = preamble.to_string();

        if let Some((shell, os)) = detect_shell_environment() {
            preamble += format!("\n\nActive Shell: {} on {}", shell, os).as_str();
        }

        match provider {
            Provider::Anthropic(settings) => {
                let (_, api_key, model) = settings.get();

                let client = rig::providers::anthropic::ClientBuilder::new(&api_key).build();

                let agent_builder = client.agent(&model).preamble(&preamble);

                let agent = Self::build_agent(agent_builder, mcp_clients).await;

                let mut agent = MultiTurnAgent::new(agent, messages.clone());

                agent.multi_turn_prompt(prompt, spinner).await
            }
            Provider::OpenAI(settings) => {
                let (_, api_key, model) = settings.get();

                let agent_builder = rig::providers::openai::Client::new(&api_key)
                    .agent(&model)
                    .preamble(&preamble);

                let agent = Self::build_agent(agent_builder, mcp_clients).await;

                let mut agent = MultiTurnAgent::new(agent, messages.clone());

                agent.multi_turn_prompt(prompt, spinner).await
            }
        }
    }

    async fn build_agent<M: CompletionModel>(
        agent_builder: AgentBuilder<M>,
        mcp_clients: Option<&mut Vec<McpClient>>,
    ) -> Agent<M> {
        if let Some(clients) = mcp_clients {
            let mut builder = agent_builder;
            for client in clients {
                // Add tool from MCP client if enabled and initialized
                if client.is_enabled() && client.initialize().await.is_ok() {
                    builder = client.add_tools(builder).await;
                }
            }

            builder.build()
        } else {
            agent_builder.build()
        }
    }

    fn build_models_request(provider: &Provider) -> RequestBuilder {
        let client = Reqwest::new();
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
            Err(e) => return Err(e.to_string()),
        };

        res.extract_models()
    }
}
