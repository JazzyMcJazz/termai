use serde::{Deserialize, Serialize};

use crate::client::traits::{Choice, ContentTrait, ModelTrait};

use super::shared::ChatRole;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse<T: Choice> {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub model: Option<String>,
    pub choices: Option<Vec<T>>,
    pub usage: Option<Option<Usage>>,
    pub system_fingerprint: Option<String>,
    pub error: Option<ErrorObject>,
}

impl ContentTrait for ChatResponse<AggregateChoice> {
    fn extract_content(&self) -> Option<String> {
        if let Some(error) = &self.error {
            Some(error.message.clone())
        } else {
            self.choices.as_ref().map(|c| {
                c.iter()
                    .map(|c| c.message.content.clone())
                    .collect::<Vec<String>>()
                    .join("\n")
            })
        }
    }
}

impl ChatResponse<StreamChoice> {
    pub fn extract_content(&self) -> Option<String> {
        if let Some(error) = &self.error {
            Some(error.message.clone())
        } else {
            self.choices.as_ref().map(|c| {
                c.iter()
                    .map(|c| c.delta.content.clone().unwrap_or_default())
                    .collect::<Vec<String>>()
                    .join("\n")
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub refusal: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct AggregateChoice {
    pub index: i64,
    pub message: ChatMessage,
    pub finish_reason: String,
}

impl Choice for AggregateChoice {}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub index: i64,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

impl Choice for StreamChoice {}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    pub role: Option<ChatRole>,
    pub content: Option<String>,
    pub refusal: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub prompt_tokens_details: PromptTokensDetails,
    pub completion_tokens_details: CompletionTokensDetails,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct PromptTokensDetails {
    pub cached_tokens: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionTokensDetails {
    pub reasoning_tokens: i64,
    pub accepted_prediction_tokens: i64,
    pub rejected_prediction_tokens: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorObject {
    pub message: String,
    pub r#type: String,
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    pub data: Vec<ModelData>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ModelData {
    pub id: String,
    pub object: String,
    pub created: i64,
}

impl ModelTrait for ModelResponse {
    fn extract_models(&self) -> Vec<(String, String)> {
        self.data
            .iter()
            .map(|d| (d.id.clone(), d.id.clone()))
            .collect()
    }
}
