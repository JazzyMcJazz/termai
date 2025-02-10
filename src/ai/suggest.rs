use std::time::Duration;

use console::style;
use indicatif::ProgressBar;

use crate::{
    ai,
    provider::Provider,
    utils::{commands::copy_to_clipboard, term_tools::get_spinner_style},
};

pub fn suggest(provider: &Provider, mut initial_query: Option<String>) {
    let mut last_suggestion = None::<String>;
    let mut should_exit = false;

    while !should_exit {
        // Prevent the loop from rerunning if Ctrl+C is pressed
        should_exit = true;

        let query = initial_query.clone().unwrap_or_else(|| {
            println!();
            let msg = if last_suggestion.is_some() {
                "How should this be revised?\n"
            } else {
                "What would you like the shell command to do?\n"
            };
            let prompt = format! {"{} {}", style("?").green().bold(), style(msg).bold()};
            dialoguer::Input::<String>::new()
                .with_prompt(prompt)
                .allow_empty(false)
                .interact()
                .unwrap_or_default()
        });

        initial_query = None;

        if query.trim().is_empty() {
            return;
        }

        println!();

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(get_spinner_style());
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(style("Thinking...").dim().bold().to_string());

        let suggested_command = if let Some(last_suggestion) = last_suggestion.clone() {
            provider.revise(&last_suggestion, &query)
        } else {
            provider.suggest(&query).clone()
        };

        spinner.finish_and_clear();

        let header = style("Suggestion:").bold();
        let suggestion = style(suggested_command.clone())
            .yellow()
            .on_color256(235)
            .bold();
        println!("{}\n\n  {}\n", header, suggestion);

        let options = vec![
            "Copy command to clipboard",
            "Explain command",
            "Revise command",
            "New command",
            "Exit",
        ];

        loop {
            let prompt =
                format! {"{} {}", style("?").green().bold(), style("Select an option").bold()};
            let Ok(selection) = dialoguer::Select::new()
                .with_prompt(prompt)
                .items(&options)
                .default(0)
                .interact()
            else {
                break;
            };

            match selection {
                0 => {
                    let _ = copy_to_clipboard(&suggested_command)
                        .map_err(|e| eprintln!("{} Error: {}\n", style("✗").red().bold(), e));
                    return; // Exit
                }
                1 => {
                    ai::explain(provider, Some(suggested_command.clone()));
                    std::thread::sleep(Duration::from_millis(500));
                    continue; // Continue to the next iteration of the inner loop
                }
                2 => {
                    last_suggestion = Some(suggested_command.clone());
                    should_exit = false;
                    break; // Continue to the next iteration of the outer loop
                }
                3 => {
                    last_suggestion = None;
                    should_exit = false;
                    break; // Continue to the next iteration of the outer loop
                }
                _ => {
                    return; // Exit
                }
            }
        }
    }
}
