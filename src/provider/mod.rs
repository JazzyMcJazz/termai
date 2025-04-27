use indicatif::ProgressBar;
use rig::{completion::PromptError, message::Message};
use serde::{Deserialize, Serialize};

use crate::{
    client::{Client, StreamingContentResult},
    mcp::McpClient,
    utils::{encryption::Enc, enums::ProviderName},
};

pub mod llm_models;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    base_url: String,
    api_key: String,
    #[serde(rename = "model")]
    completion_model: String,
    #[serde(default)]
    search_model: Option<String>,
}

impl ProviderSettings {
    pub fn get(&self) -> (String, String, String, String) {
        (
            self.base_url.clone(),
            self.api_key.clone(),
            self.completion_model.clone(),
            self.search_model.clone().unwrap_or_default(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    OpenAI(ProviderSettings),
    Anthropic(ProviderSettings),
}

impl Provider {
    pub async fn new(provider_name: ProviderName, api_key: String) -> Self {
        let base_url = match provider_name {
            ProviderName::OpenAI => "https://api.openai.com".into(),
            ProviderName::Anthropic => "https://api.anthropic.com".into(),
        };

        let settings = ProviderSettings {
            base_url,
            api_key,
            completion_model: String::new(),
            search_model: None,
        };

        let mut provider = match provider_name {
            ProviderName::OpenAI => Provider::OpenAI(settings),
            ProviderName::Anthropic => Provider::Anthropic(settings),
        };

        let (compleltion_models, search_models) = provider.fetch_available_models().await;
        if let Some((model, _)) = compleltion_models.first() {
            provider.set_completion_model(model.clone());
        } else {
            eprint!("Failed to fetch models from {}", provider_name);
            std::process::exit(1);
        }

        if let Some((model, _)) = search_models.first() {
            provider.set_search_model(model.clone());
        }

        provider
    }

    pub fn name(&self) -> ProviderName {
        match self {
            Provider::OpenAI(_) => ProviderName::OpenAI,
            Provider::Anthropic(_) => ProviderName::Anthropic,
        }
    }

    pub fn set_api_key(&mut self, api_key: String) {
        match self {
            Provider::OpenAI(settings) => {
                settings.api_key = api_key;
            }
            Provider::Anthropic(settings) => {
                settings.api_key = api_key;
            }
        }
    }

    pub fn completion_model(&self) -> String {
        match self {
            Provider::OpenAI(settings) => settings.completion_model.clone(),
            Provider::Anthropic(settings) => settings.completion_model.clone(),
        }
    }

    pub fn set_completion_model(&mut self, model: String) {
        match self {
            Provider::OpenAI(settings) => {
                settings.completion_model = model;
            }
            Provider::Anthropic(settings) => {
                settings.completion_model = model;
            }
        }
    }

    pub fn search_model(&self) -> Option<String> {
        match self {
            Provider::OpenAI(settings) => settings.search_model.clone(),
            Provider::Anthropic(settings) => settings.search_model.clone(),
        }
    }

    pub fn set_search_model(&mut self, model: String) {
        match self {
            Provider::OpenAI(settings) => {
                settings.search_model = Some(model);
            }
            Provider::Anthropic(settings) => {
                settings.search_model = Some(model);
            }
        }
    }

    pub async fn chat(
        &self,
        prompt: &str,
        messages: Vec<Message>,
        mcp_clients: &mut Vec<McpClient>,
        spinner: &ProgressBar,
        search: bool,
    ) -> Result<String, PromptError> {
        Client::chat(prompt, messages, self, mcp_clients, spinner, search).await
    }

    pub async fn chat_stream(
        &self,
        prompt: &str,
        messages: Vec<Message>,
        mcp_clients: &mut Vec<McpClient>,
        search: bool,
    ) -> StreamingContentResult {
        Client::chat_stream(prompt, messages, self, mcp_clients, search).await
    }

    pub async fn suggest(&self, prompt: &str) -> Result<String, PromptError> {
        Client::suggest(prompt, self).await
    }

    pub async fn revise(
        &self,
        prompt: &str,
        command_to_revise: &str,
    ) -> Result<String, PromptError> {
        Client::revise(prompt, command_to_revise, self).await
    }

    pub async fn explain(&self, prompt: &str) -> Result<String, PromptError> {
        Client::explain(prompt, self).await
    }

    pub async fn fetch_available_models(&self) -> (Vec<(String, String)>, Vec<(String, String)>) {
        let models = match self {
            Provider::OpenAI(_) => {
                // Use available models from the API to filter supported models
                let models = Client::fetch_models(self).await;
                let completion_models = llm_models::OPENAI_COMPLETION_MODELS
                    .iter()
                    .filter(|(id, _)| models.iter().any(|(model, _)| model == id))
                    .map(|(id, name)| (id.to_string(), name.to_string()))
                    .collect();

                let search_models = llm_models::OPENAI_SEARCH_MODELS
                    .iter()
                    .filter(|(id, _)| models.iter().any(|(model, _)| model == id))
                    .map(|(id, name)| (id.to_string(), name.to_string()))
                    .collect::<Vec<_>>();

                (completion_models, search_models)
            }
            Provider::Anthropic(_) => {
                let models = Client::fetch_models(self).await;
                let completion_models = llm_models::ANTHROPIC_COMPLETION_MODELS
                    .iter()
                    .filter(|(id, _)| models.iter().any(|(model, _)| model == id))
                    .map(|(id, name)| (id.to_string(), name.to_string()))
                    .collect();

                let search_models = llm_models::ANTHROPIC_SEARCH_MODELS
                    .iter()
                    .filter(|(id, _)| models.iter().any(|(model, _)| model == id))
                    .map(|(id, name)| (id.to_string(), name.to_string()))
                    .collect::<Vec<_>>();

                (completion_models, search_models)
            }
        };

        models
    }

    pub fn encrypt(&mut self) -> Result<(), &'static str> {
        match self {
            Provider::OpenAI(settings) => {
                settings.api_key = Enc::encrypt(&settings.api_key)?;
            }
            Provider::Anthropic(settings) => {
                settings.api_key = Enc::encrypt(&settings.api_key)?;
            }
        }

        Ok(())
    }

    pub fn decrypt(&mut self) -> Result<(), &'static str> {
        match self {
            Provider::OpenAI(settings) => {
                settings.api_key = Enc::decrypt(&settings.api_key)?;
            }
            Provider::Anthropic(settings) => {
                settings.api_key = Enc::decrypt(&settings.api_key)?;
            }
        }

        Ok(())
    }
}
