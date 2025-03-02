use console::{style, Style};
use dialoguer::theme::{ColorfulTheme, Theme};
use indicatif::ProgressStyle;

pub fn get_spinner_style() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template("{msg}\n{spinner}")
        .expect("Failed to create spinner style")
}

pub fn get_select_theme() -> impl Theme {
    ColorfulTheme {
        prompt_style: Style::new().for_stderr().bold(),
        prompt_prefix: style("?".to_string()).for_stderr().green().bold(),
        prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
        success_prefix: style("✔".to_string()).for_stderr().green(),
        success_suffix: style("›".to_string()).for_stderr().black().bright(),
        values_style: Style::new().for_stderr().dim(),
        active_item_prefix: style("›".to_string()).for_stderr().green(),
        ..Default::default()
    }
}
