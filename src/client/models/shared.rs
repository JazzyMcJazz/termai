use std::io::{BufRead, BufReader};

use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};

use crate::provider::Provider;

use super::{anthropic, openai};

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
}

pub struct ContentIterator<'a> {
    reader: BufReader<Response>,
    provider: &'a Provider,
}

impl<'a> ContentIterator<'a> {
    pub fn new(reader: BufReader<Response>, provider: &'a Provider) -> Self {
        Self { reader, provider }
    }

    fn parse(&self, line: &str) -> Option<String> {
        match self.provider {
            Provider::OpenAI(_) => {
                let res: openai::ChatResponse<openai::StreamChoice> =
                    match serde_json::from_str(line) {
                        Ok(json) => json,
                        Err(_) => return Some(line.into()),
                    };

                res.extract_content()
            }
            Provider::Anthropic(_) => {
                let res: anthropic::StreamChunk = match serde_json::from_str(line) {
                    Ok(json) => json,
                    Err(e) => {
                        return Some(format!("\n\nError: {}\nLine: {}", e, line));
                    }
                };

                match res.r#type {
                    anthropic::ChunkType::ContentBlockStart => {
                        res.content_block.as_ref().map(|c| c.text.clone())?
                    }
                    anthropic::ChunkType::ContentBlockDelta => {
                        res.delta.as_ref().map(|c| c.extract_content())?
                    }
                    anthropic::ChunkType::Error => {
                        let message = res.error.as_ref().map(|e| e.message.clone());
                        Some(format!("**Error**: {}", message.unwrap_or_default()))
                    }
                    _ => Some("".into()),
                }
            }
        }
    }
}

impl Iterator for ContentIterator<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        // Read the next line from the response.
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // End of stream
            Ok(_) => {
                // Sometimes the line might be empty or whitespace; skip those.
                line = line.trim_start_matches("data: ").trim().to_string();
                if line.is_empty() || line.eq("[DONE]") || line.starts_with("event: ") {
                    return self.next();
                }

                // Parse the line as JSON.
                // Some(line)
                self.parse(&line)
            }
            Err(e) => Some(format!("\n\nError: {}\n\n", e)),
        }
    }
}
