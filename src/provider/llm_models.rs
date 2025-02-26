pub const OPENAI_MODELS: &[(&str, &str)] = &[
    ("gpt-4o", "gtp-4o"),
    ("gpt-4o-mini", "gtp-4o-mini"),
    // o-models are disabled for now because they do not support system role messages
    // and I don't want to refactor the code to support that

    // ("o1", "o1"),
    // ("o1-preview", "o1-preview"),
    // ("o1-mini", "o1-mini"),
    // ("o3-mini", "o3-mini"),
];

pub const ANTHROPIC_MODELS: &[(&str, &str)] = &[
    ("claude-3-7-sonnet-20250219", "Claude 3.7 Sonnet"),
    ("claude-3-5-haiku-20241022", "Claude 3.5 Haiku"),
    ("claude-3-opus-20240229", "Claude 3 Opus"),
];
