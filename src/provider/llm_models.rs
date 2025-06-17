/// Competion models for Anthropic
///
/// Tuple format: `(model_id, display_name)`
pub const ANTHROPIC_COMPLETION_MODELS: &[(&str, &str)] = &[
    ("claude-sonnet-4-20250514", "Claude Sonnet 4"),
    ("claude-opus-4-20250514", "Claude Opus 4"),
    ("claude-3-7-sonnet-20250219", "Claude 3.7 Sonnet"),
    ("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet"),
    ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku"),
    ("claude-3-opus-20240229", "Claude 3 Opus"),
];

/// Search models for Anthropic
///
/// Tuple format: `(model_id, display_name)`
pub const ANTHROPIC_SEARCH_MODELS: &[(&str, &str)] = &[];

/// Completion models for OpenAI
///
/// Tuple format: `(model_id, display_name)`
pub const OPENAI_COMPLETION_MODELS: &[(&str, &str)] = &[
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

/// Search models for OpenAI
///
/// Tuple format: `(model_id, display_name)`
pub const OPENAI_SEARCH_MODELS: &[(&str, &str)] = &[
    ("gpt-4o-search-preview", "GPT 4o Search"),
    ("gpt-4o-mini-search-preview", "GPT 4o-mini Search"),
];

// /// Completion models for Perplexity
// ///
// /// Tuple format: `(model_id, display_name)`
// pub const PERPLEXITY_COMPLETION_MODELS: &[(&str, &str)] = &[];

// /// Search models for Perplexity
// ///
// /// Tuple format: `(model_id, display_name)`
// pub const PERPLEXITY_SEARCH_MODELS: &[(&str, &str)] = &[
//     ("sonar", "Sonar"),
//     ("sonar-pro", "Sonar Pro"),
// ];
