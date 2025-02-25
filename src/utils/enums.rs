use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProviderName {
    OpenAI,
    Anthropic,
}

impl Display for ProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderName::OpenAI => write!(f, "OpenAI"),
            ProviderName::Anthropic => write!(f, "Anthropic"),
        }
    }
}

impl Copy for ProviderName {}
