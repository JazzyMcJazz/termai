pub trait Choice {}
pub trait ContentTrait {
    fn extract_content(&self) -> Option<String>;
}

pub trait ModelTrait {
    fn extract_models(&self) -> Vec<(String, String)>;
}
