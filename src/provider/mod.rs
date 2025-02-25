use serde::{Deserialize, Serialize};

use crate::{
    client::{ChatMessage, Client, ContentIterator},
    utils::{encryption::Enc, enums::ProviderName},
};

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
    pub fn new(provider: ProviderName, api_key: String, model: Option<String>) -> Self {
        let model = match model {
            Some(model) => model,
            None => match provider {
                ProviderName::OpenAI => "gpt-4o-mini".into(),
                ProviderName::Anthropic => todo!(),
            },
        };

        let base_url = match provider {
            ProviderName::OpenAI => "https://api.openai.com/v1/chat/completions".into(),
            ProviderName::Anthropic => "https://api.anthropic.com/v1/messages".into(),
        };

        match provider {
            ProviderName::OpenAI => Provider::OpenAI(ProviderSettings {
                base_url,
                api_key,
                model,
            }),
            ProviderName::Anthropic => Provider::Anthropic(ProviderSettings {
                base_url,
                api_key,
                model,
            }),
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
