/// Competion models for Anthropic
///
/// Tuple format: `(model_id, display_name)`
pub const ANTHROPIC_COMPLETION_MODELS: &[(&str, &str)] = &[
    ("claude-haiku-4-5-20251001", "Claude Haiku 4.5"),
    ("claude-sonnet-4-5-20250929", "Claude Sonnet 4.5"),
    ("claude-opus-4-6", "Claude Opus 4.6"),
];

/// Search models for Anthropic
///
/// Tuple format: `(model_id, display_name)`
pub const ANTHROPIC_SEARCH_MODELS: &[(&str, &str)] = &[];

/// Completion models for OpenAI
///
/// Tuple format: `(model_id, display_name)`
pub const OPENAI_COMPLETION_MODELS: &[(&str, &str)] = &[
    ("gpt-4.1", "GPT 4.1"),
    ("gpt-5-nano", "GPT 5 Nano"),
    ("gpt-5-mini", "GPT 5 Mini"),
    ("gpt-5.2", "GPT 5.2"),
    ("gpt-5.2-pro", "GPT 5.2 Pro"),
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
