use console::style;
use serde::{Deserialize, Serialize};

use crate::{
    mcp::McpClient,
    provider::{llm_models, Provider},
    utils::enums::ProviderName,
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    providers: Vec<Provider>,
    active_provider: Option<usize>,
    #[serde(default)]
    pub use_streaming: bool,
    /// **provider_name, model_id, display_name**
    #[serde(default)]
    available_models: Vec<(ProviderName, String, String)>,
    #[serde(default)]
    mcp_clients: Vec<McpClient>,
}

impl Config {
    pub fn load() -> Self {
        let mut cfg: Config = confy::load("termai", "config").unwrap_or_default();

        for provider in cfg.providers.iter_mut() {
            match provider.decrypt() {
                Ok(_) => {}
                Err(e) => {
                    let cross = style("✗").red().bold();
                    eprintln!("{cross} Failed to decrypt API key: {e}");
                    std::process::exit(1);
                }
            }
        }

        cfg
    }

    pub fn streaming(&self) -> bool {
        self.use_streaming
    }

    pub fn toggle_streaming(&mut self) {
        self.use_streaming = !self.use_streaming;
        self.save();
    }

    pub fn is_configured(&self, provider_name: ProviderName) -> bool {
        self.providers.iter().any(|p| p.name() == provider_name)
    }

    pub fn active_model(&self) -> Option<(String, String)> {
        if let Some(model) = self.active_provider().map(|p| p.model()) {
            let openai_models: &[(&str, &str)] = llm_models::OPENAI_MODELS;
            let anthropic_models: &[(&str, &str)] = llm_models::ANTHROPIC_MODELS;

            // Find the model in the list of available models
            let model = match self.active_provider()? {
                Provider::OpenAI(_) => openai_models.iter().find(|(id, _)| id == &model),
                Provider::Anthropic(_) => anthropic_models.iter().find(|(id, _)| id == &model),
            };

            model.map(|(id, name)| (id.to_string(), name.to_string()))
        } else {
            None
        }
    }

    pub fn active_provider(&self) -> Option<&Provider> {
        self.providers.get(self.active_provider?)
    }

    pub fn find_provider(&self, provider_name: &ProviderName) -> Option<&Provider> {
        self.providers.iter().find(|p| &p.name() == provider_name)
    }

    pub fn set_model(&mut self, provider_name: ProviderName, model: String) {
        self.active_provider = self
            .providers
            .iter()
            .position(|p| p.name() == provider_name);

        if let Some(provider) = self.active_provider() {
            let mut provider = provider.clone();
            provider.set_model(model);
            self.providers[self.active_provider.unwrap()] = provider;
            self.save();
        }
    }

    pub async fn refresh_available_models(&mut self) {
        let mut models: Vec<(ProviderName, String, String)> = Vec::new();

        let mut tasks = vec![];

        for provider in self.providers.iter() {
            let task = provider.fetch_available_models();
            tasks.push(task);
        }

        let results = futures::future::join_all(tasks).await;

        for (i, result) in results.into_iter().enumerate() {
            let provider = &self.providers[i];
            for (id, display_name) in result {
                models.push((provider.name(), id, display_name));
            }
        }

        self.available_models = models;
    }

    pub fn get_available_models(&self) -> &Vec<(ProviderName, String, String)> {
        &self.available_models
    }

    pub fn remove_provider(&mut self, provider_name: ProviderName) {
        let provider_index = self
            .providers
            .iter()
            .position(|p| p.name() == provider_name);

        self.providers.retain(|p| p.name() != provider_name);
        self.available_models
            .retain(|(p, _, _)| p != &provider_name);

        if let Some(index) = self.active_provider {
            if Some(index) == provider_index {
                self.active_provider = self.providers.first().map(|_| 0);
            } else if index > provider_index.unwrap_or(0) {
                self.active_provider = Some(index - 1);
            }
        };

        self.save();
    }

    pub async fn add_provider_api_key(&mut self, provider_name: ProviderName, api_key: String) {
        let provider_index = self
            .providers
            .iter()
            .position(|p| p.name() == provider_name);

        if let Some(index) = provider_index {
            let mut provider = self.providers[index].clone();
            provider.set_api_key(api_key);
            self.providers[index] = provider;
        } else {
            let provider = Provider::new(provider_name, api_key, None).await;
            let models = provider.fetch_available_models().await;
            for (id, display_name) in models {
                self.available_models
                    .push((provider_name, id, display_name));
            }
            self.providers.push(provider);
        }

        let provider_index = self
            .providers
            .iter()
            .position(|p| p.name() == provider_name);

        if self.active_provider().is_none() {
            self.active_provider = provider_index;
        }

        self.save();
    }

    pub fn mcp_clients(&mut self) -> &mut Vec<McpClient> {
        &mut self.mcp_clients
    }

    pub fn mcp_clients_mut(&mut self) -> &mut Vec<McpClient> {
        &mut self.mcp_clients
    }

    pub fn add_mcp_client(&mut self, mcp_client: McpClient) {
        self.mcp_clients.push(mcp_client);
        self.save();
    }

    pub fn save(&self) {
        let mut cfg = self.clone();

        cfg.providers.sort_by_key(|a| a.name());
        cfg.available_models.sort_by_key(|a| a.0);

        for provider in cfg.providers.iter_mut() {
            match provider.encrypt() {
                Ok(_) => {}
                Err(e) => {
                    let cross = style("✗").red().bold();
                    eprintln!("{cross} Failed to encrypt API key: {e}");
                    std::process::exit(1);
                }
            }
        }

        confy::store("termai", "config", cfg).expect("Failed to save configuration");
    }
}
