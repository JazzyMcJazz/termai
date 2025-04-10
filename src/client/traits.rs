pub trait ModelTrait {
    fn extract_models(&self) -> Result<Vec<(String, String)>, String>;
}
