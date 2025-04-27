use super::enums::StopReason;

pub trait ModelTrait {
    fn extract_models(&self) -> Result<Vec<(String, String)>, String>;
}
pub trait CompetionResponseExt {
    fn stop_reason(&self) -> StopReason;
}
