/// LLM models for OpenAI
///
/// Tuple format: `(model_id, display_name)`
pub const OPENAI_MODELS: &[(&str, &str)] = &[
    ("gpt-4o", "GPT 4o"),
    ("gpt-4o-mini", "GPT 4o mini"),
    ("gpt-4.1", "GPT 4.1"),
    ("gpt-4.1-mini", "GPT 4.1 mini"),
    ("gpt-4.1-nano", "GPT 4.1 nano"),
    // ("gpt-4.5-preview", "GPT 4.5-preview"),
    // ("o1", "o1"),
    ("o3-mini", "o3 mini"),
    ("o4-mini", "o4 mini"),
];

/// LLM models for Anthropic
///
/// Tuple format: `(model_id, display_name)`
pub const ANTHROPIC_MODELS: &[(&str, &str)] = &[
    ("claude-3-7-sonnet-20250219", "Claude 3.7 Sonnet"),
    ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku"),
    ("claude-3-opus-20240229", "Claude 3 Opus"),
];
