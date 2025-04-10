use serde::Deserialize;

use crate::client::traits::ModelTrait;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    data: Option<Vec<ModelData>>,
    has_more: Option<bool>,
    first_id: Option<String>,
    last_id: Option<String>,
    error: Option<ModelErrorObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelErrorObject {
    message: String,
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
    fn extract_models(&self) -> Result<Vec<(String, String)>, String> {
        if let Some(error) = &self.error {
            Err(error.message.clone())
        } else if let Some(data) = &self.data {
            Ok(data
                .iter()
                .map(|m| (m.id.clone(), m.display_name.clone()))
                .collect())
        } else {
            Err("No data found".to_string())
        }
    }
}
