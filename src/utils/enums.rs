use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderName {
    OpenAI,
    Anthropic,
}

impl ProviderName {
    pub fn iter() -> Vec<ProviderName> {
        vec![ProviderName::OpenAI, ProviderName::Anthropic]
    }
}

impl Display for ProviderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderName::OpenAI => write!(f, "OpenAI"),
            ProviderName::Anthropic => write!(f, "Anthropic"),
        }
    }
}

impl PartialOrd for ProviderName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProviderName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ProviderName::OpenAI, ProviderName::OpenAI) => std::cmp::Ordering::Equal,
            (ProviderName::OpenAI, _) => std::cmp::Ordering::Less,
            (_, ProviderName::OpenAI) => std::cmp::Ordering::Greater,
            (ProviderName::Anthropic, ProviderName::Anthropic) => std::cmp::Ordering::Equal,
        }
    }
}

impl Copy for ProviderName {}
