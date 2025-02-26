use serde::{Deserialize, Serialize};

use crate::{
    client::{ChatMessage, Client, ContentIterator},
    utils::{encryption::Enc, enums::ProviderName},
};

pub mod llm_models;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    base_url: String,
    api_key: String,
    model: String,
}

impl ProviderSettings {
    pub fn get(&self) -> (String, String, String) {
        (
            self.base_url.clone(),
            self.api_key.clone(),
            self.model.clone(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    OpenAI(ProviderSettings),
    Anthropic(ProviderSettings),
}

impl Provider {
    pub fn new(provider_name: ProviderName, api_key: String, model: Option<String>) -> Self {
        let base_url = match provider_name {
            ProviderName::OpenAI => "https://api.openai.com".into(),
            ProviderName::Anthropic => "https://api.anthropic.com".into(),
        };

        let settings = ProviderSettings {
            base_url,
            api_key,
            model: model.to_owned().unwrap_or_default(),
        };

        let mut provider = match provider_name {
            ProviderName::OpenAI => Provider::OpenAI(settings),
            ProviderName::Anthropic => Provider::Anthropic(settings),
        };

        if model.is_none() {
            let models = Client::fetch_models(&provider);
            if let Some((model, _)) = models.first() {
                provider.set_model(model.clone());
            } else {
                eprint!("Failed to fetch models from {}", provider_name);
                std::process::exit(1);
            }
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

    pub fn model(&self) -> String {
        match self {
            Provider::OpenAI(settings) => settings.model.clone(),
            Provider::Anthropic(settings) => settings.model.clone(),
        }
    }

    pub fn set_model(&mut self, model: String) {
        match self {
            Provider::OpenAI(settings) => {
                settings.model = model;
            }
            Provider::Anthropic(settings) => {
                settings.model = model;
            }
        }
    }

    pub fn chat(&self, messages: &[ChatMessage]) -> String {
        Client::chat(messages, self)
    }

    pub fn chat_stream<'a>(&'a self, messages: &[ChatMessage]) -> ContentIterator<'a> {
        Client::chat_stream(messages, self)
    }

    pub fn suggest(&self, query: &str) -> String {
        Client::suggest(query, self)
    }

    pub fn revise(&self, command_to_revise: &str, query: &str) -> String {
        Client::revise(command_to_revise, query, self)
    }

    pub fn explain(&self, query: &str) -> String {
        Client::explain(query, self)
    }

    pub fn fetch_models(&self) -> Vec<(String, String)> {
        let models = match self {
            Provider::OpenAI(_) => {
                // Use available models from the API to filter supported models
                let models = Client::fetch_models(self);
                llm_models::OPENAI_MODELS
                    .iter()
                    .filter(|(id, _)| models.iter().any(|(model, _)| model == id))
                    .map(|(id, name)| (id.to_string(), name.to_string()))
                    .collect()
            }
            // No need to filter models for Anthropic since all models are available for all users
            Provider::Anthropic(_) => llm_models::ANTHROPIC_MODELS
                .iter()
                .map(|(id, name)| (id.to_string(), name.to_string()))
                .collect(),
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
