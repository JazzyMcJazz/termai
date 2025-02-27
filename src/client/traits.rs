pub trait Choice {}
pub trait ContentTrait {
    fn extract_content(&self) -> Option<String>;
}

pub trait ModelTrait {
    fn extract_models(&self) -> Result<Vec<(String, String)>, String>;
}
