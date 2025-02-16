use std::io::{BufRead, BufReader};

use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

pub trait Choice {}

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub stream: bool,
    pub model: String,
    pub messages: Vec<ChatMessage>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse<T: Choice> {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<T>,
    pub usage: Option<Usage>,
    pub system_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRole {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
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

pub struct ContentIterator {
    reader: BufReader<Response>,
}

impl ContentIterator {
    pub fn new(reader: BufReader<Response>) -> Self {
        Self { reader }
    }
}

impl Iterator for ContentIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        // Read the next line from the response.
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // End of stream
            Ok(_) => {
                // Sometimes the line might be empty or whitespace; skip those.
                line = line.trim_start_matches("data: ").trim().to_string();
                if line.is_empty() || line.eq("[DONE]") {
                    return self.next();
                }
                // Parse the line as JSON.
                let res: ChatResponse<StreamChoice> = match serde_json::from_str(&line) {
                    Ok(json) => json,
                    Err(e) => {
                        return Some(format!("\n\nError: {}\nLine: {}", e, line));
                    }
                };

                match res.choices.first() {
                    Some(choice) => {
                        if let Some(ref content) = choice.delta.content {
                            Some(content.clone())
                        } else {
                            Some("".into())
                        }
                    }
                    None => Some("".into()),
                }
            }
            Err(e) => Some(format!("\n\nError: {}\n\n", e)),
        }
    }
}
