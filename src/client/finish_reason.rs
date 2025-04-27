use rig::providers::{anthropic, openai};

use super::{enums::StopReason, traits::CompetionResponseExt};

impl CompetionResponseExt for anthropic::completion::CompletionResponse {
    fn stop_reason(&self) -> StopReason {
        let Some(stop_reason) = self.stop_reason.as_ref() else {
            return StopReason::None;
        };

        match stop_reason.as_str() {
            "end_turn" => StopReason::Stop,
            "max_tokens" => StopReason::Length,
            "stop_sequence" => StopReason::ContentFilter,
            "tool_use" => StopReason::ToolCall,
            _ => StopReason::None,
        }
    }
}

impl CompetionResponseExt for openai::CompletionResponse {
    fn stop_reason(&self) -> StopReason {
        let mut stop_reason = None;

        for choice in &self.choices {
            if !choice.finish_reason.is_empty() {
                stop_reason = Some(choice.finish_reason.clone());
            }
        }

        let Some(stop_reason) = stop_reason else {
            return StopReason::None;
        };

        match stop_reason.as_str() {
            "stop" => StopReason::Stop,
            "length" => StopReason::Length,
            "content_filter" => StopReason::ContentFilter,
            "tool_calls" | "function_call" => StopReason::ToolCall,
            _ => StopReason::None,
        }
    }
}
