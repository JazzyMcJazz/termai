use indicatif::ProgressStyle;

pub fn get_spinner_style() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template("{msg}\n{spinner}")
        .expect("Failed to create spinner style")
}
