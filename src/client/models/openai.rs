use serde::Deserialize;

use crate::client::traits::ModelTrait;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelResponse {
    pub data: Option<Vec<ModelData>>,
    pub error: Option<ModelErrorObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelErrorObject {
    pub message: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ModelData {
    pub id: String,
    pub object: String,
    pub created: i64,
}

impl ModelTrait for ModelResponse {
    fn extract_models(&self) -> Result<Vec<(String, String)>, String> {
        if let Some(error) = &self.error {
            Err(error.message.clone())
        } else if let Some(data) = &self.data {
            Ok(data.iter().map(|d| (d.id.clone(), d.id.clone())).collect())
        } else {
            Err("No data found".to_string())
        }
    }
}
