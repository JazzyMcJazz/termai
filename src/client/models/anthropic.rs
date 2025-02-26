use serde::Deserialize;

use crate::client::traits::{ContentTrait, ModelTrait};

use super::shared::ChatRole;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: Option<String>,
    pub model: Option<String>,
    pub role: Option<ChatRole>,
    pub content: Option<Vec<Content>>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
    pub error: Option<ErrorObject>,
}

impl ContentTrait for ChatResponse {
    fn extract_content(&self) -> Option<String> {
        if self.r#type == "message" {
            self.content
                .as_ref()
                .map(|c| c.iter().map(|c| c.text.clone()).collect())
        } else if self.r#type == "error" {
            let message = self.error.as_ref().map(|e| e.message.clone());
            Some(format!("**Error**: {}", message.unwrap_or_default()))
        } else {
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Content {
    pub text: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorObject {
    #[serde(rename = "type")]
    pub r#type: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ChunkType {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "message_start")]
    MessageStart,
    #[serde(rename = "message_delta")]
    MessageDelta,
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "content_block_start")]
    ContentBlockStart,
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta,
    #[serde(rename = "content_block_stop")]
    ContentBlockStop,
    #[serde(rename = "error")]
    Error,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct StreamChunk {
    #[serde(rename = "type")]
    pub r#type: ChunkType,
    pub index: Option<i64>,
    pub delta: Option<Delta>,
    pub error: Option<ErrorChunk>,
    pub message: Option<ChatResponse>,
    pub content_block: Option<Delta>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub enum DeltaType {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "text_delta")]
    TextDelta,
    #[serde(rename = "input_json_delta")]
    InputJsonDelta,
    #[serde(rename = "thinking_delta")]
    ThinkingDelta,
    #[serde(rename = "signature_delta")]
    SignatureDelta,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    #[serde(rename = "type")]
    pub r#type: Option<DeltaType>,
    pub text: Option<String>,
    pub partial_json: Option<String>,
    pub thinking: Option<String>,
    pub signature: Option<String>,
}

impl Delta {
    pub fn extract_content(&self) -> Option<String> {
        match self.r#type {
            Some(DeltaType::TextDelta) => self.text.clone(),
            Some(DeltaType::InputJsonDelta) => self.partial_json.clone(),
            Some(DeltaType::ThinkingDelta) => self.thinking.clone(),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorChunk {
    #[serde(rename = "type")]
    pub r#type: String,
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    data: Vec<ModelData>,
    has_more: bool,
    first_id: Option<String>,
    last_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ModelData {
    #[serde(rename = "type")]
    r#type: String,
    id: String,
    display_name: String,
    created_at: String,
}

impl ModelTrait for ModelResponse {
    fn extract_models(&self) -> Vec<(String, String)> {
        self.data
            .iter()
            .map(|m| (m.id.clone(), m.display_name.clone()))
            .collect()
    }
}
