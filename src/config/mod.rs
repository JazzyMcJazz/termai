use serde::{Deserialize, Serialize};

use crate::{provider::Provider, utils::enums::ProviderName};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    openai: Option<Provider>,
    active_provider: Option<ProviderName>,
}

impl Config {
    pub fn load() -> Self {
        let mut cfg: Config = confy::load("clai", "config").unwrap_or_default();

        if let Some(openai) = cfg.openai.as_mut() {
            openai.decrypt();
        }

        cfg
    }

    pub fn is_configured(&self, provider_name: ProviderName) -> bool {
        match provider_name {
            ProviderName::OpenAI => self.openai.is_some(),
        }
    }

    pub fn active_model(&self) -> Option<String> {
        match self.active_provider {
            Some(ProviderName::OpenAI) => self.openai.as_ref().map(|p| p.model()),
            _ => None,
        }
    }

    pub fn active_provider(&self) -> Option<Provider> {
        match self.active_provider {
            Some(ProviderName::OpenAI) => self.openai.clone(),
            _ => None,
        }
    }

    pub fn active_provider_name(&self) -> Option<ProviderName> {
        // Account for the possibility of a provider being removed
        self.active_provider
            .filter(|&active_provider| self.is_configured(active_provider))
    }

    pub fn set_active_provider(&mut self, provider_name: ProviderName) {
        let exists = match provider_name {
            ProviderName::OpenAI => self.openai.is_some(),
        };

        if exists {
            self.active_provider = Some(provider_name);
            self.save();
        }
    }

    pub fn set_model(&mut self, provider_name: ProviderName, model: String) {
        match provider_name {
            ProviderName::OpenAI => {
                if let Some(openai) = self.openai.as_mut() {
                    openai.set_model(model);
                }
            }
        }

        self.save();
    }

    pub fn remove_provider(&mut self, provider_name: ProviderName) {
        match provider_name {
            ProviderName::OpenAI => self.openai = None,
        }

        if self.active_provider == Some(provider_name) {
            self.active_provider = None;
        }

        self.save();
    }

    pub fn store(&mut self, provider_name: ProviderName, api_key: String) {
        let provider = match provider_name {
            ProviderName::OpenAI => self.openai.as_ref(),
        };

        let model = provider.map(|p| p.model());

        let provider = Provider::new(provider_name, api_key, model);
        match provider_name {
            ProviderName::OpenAI => self.openai = Some(provider),
        }

        if self.active_provider.is_none() {
            self.active_provider = Some(provider_name);
        }

        self.save();
    }

    fn save(&self) {
        let mut cfg = self.clone();

        if let Some(openai) = cfg.openai.as_mut() {
            openai.encrypt();
        }

        confy::store("clai", "config", cfg).expect("Failed to save configuration");
    }
}
