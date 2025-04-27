use std::time::Duration;

use console::style;
use indicatif::ProgressBar;

use crate::{
    ai::utils::{on_the_fly_change_model, NO_MODELS_FOUND_MSG},
    config::Config,
    utils::console::get_spinner_style,
};

pub async fn explain(cfg: &Config, query: Option<String>, select_model: bool) {
    let mut provider = cfg
        .active_provider()
        .unwrap_or_else(|| {
            eprintln!("No active provider");
            std::process::exit(1);
        })
        .clone();

    if select_model {
        println!();
        if let Some(p) =
            on_the_fly_change_model(&mut cfg.clone(), Some(provider.completion_model()), false)
                .await
        {
            provider = p;
        } else {
            println!("{}", style(NO_MODELS_FOUND_MSG).red());
        }
    };

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

    let explanation = match provider.explain(&query).await {
        Ok(result) => result.replace(r"\x1b", "\x1b"), // Fix ANSI escape codes
        Err(e) => e.to_string(),
    };

    spinner.finish_and_clear();

    let header = style("Explanation:").bold();
    println!("{}\n\n{}\n", header, explanation);
}
