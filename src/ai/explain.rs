use std::time::Duration;

use console::style;
use indicatif::ProgressBar;

use crate::{provider, utils::console::get_spinner_style};

pub fn explain(provider: &provider::Provider, query: Option<String>) {
    let query = query.unwrap_or_else(|| {
        println!();

        let prompt = format!{"{} {}", style("?").green().bold(), style("What shell command would you like explained?\n").bold()};
        dialoguer::Input::<String>::new()
            .with_prompt(prompt)
            .allow_empty(false)
            .interact().unwrap_or_default()
    });

    if query.trim().is_empty() {
        return;
    }

    println!();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(get_spinner_style());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(style("Thinking...").dim().bold().to_string());

    let explanation = provider.explain(&query);
    let explanation = explanation.replace(r"\x1b", "\x1b"); // Fix ANSI escape codes

    spinner.finish_and_clear();

    let header = style("Explanation:").bold();
    println!("{}\n\n{}\n", header, explanation);
}
