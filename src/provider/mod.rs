use serde::{Deserialize, Serialize};

use crate::{
    client::{ChatMessage, Client},
    utils::{encryption::Enc, enums::ProviderName},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    api_key: String,
    model: String,
}

impl ProviderSettings {
    pub fn get(&self) -> (String, String) {
        (self.api_key.clone(), self.model.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    OpenAI(ProviderSettings),
}

impl Provider {
    pub fn new(provider: ProviderName, api_key: String, model: Option<String>) -> Self {
        let model = match model {
            Some(model) => model,
            None => match provider {
                ProviderName::OpenAI => "gpt-4o-mini".into(),
            },
        };

        match provider {
            ProviderName::OpenAI => Provider::OpenAI(ProviderSettings { api_key, model }),
        }
    }

    pub fn model(&self) -> String {
        match self {
            Provider::OpenAI(settings) => settings.model.clone(),
        }
    }

    pub fn set_model(&mut self, model: String) {
        match self {
            Provider::OpenAI(settings) => {
                settings.model = model;
            }
        }
    }

    pub fn chat(&self, messages: &[ChatMessage]) -> String {
        Client::chat(messages, self)
    }

    pub fn suggest(&self, query: &str) -> String {
        Client::suggest(query, self)
    }

    pub fn explain(&self, query: &str) -> String {
        Client::explain(query, self)
    }

    pub fn encrypt(&mut self) {
        match self {
            Provider::OpenAI(settings) => {
                settings.api_key = Enc::encrypt(&settings.api_key);
            }
        }
    }

    pub fn decrypt(&mut self) {
        match self {
            Provider::OpenAI(settings) => {
                settings.api_key = Enc::decrypt(&settings.api_key);
            }
        }
    }
}
