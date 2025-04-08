use std::time::Duration;

use console::style;
use indicatif::ProgressBar;

use crate::{
    ai::{
        self,
        utils::{on_the_fly_change_model, NO_MODELS_FOUND_MSG},
    },
    config::Config,
    utils::{
        commands::copy_to_clipboard,
        console::{get_select_theme, get_spinner_style},
    },
};

pub fn suggest(cfg: &Config, mut initial_query: Option<String>, select_model: bool) {
    let mut provider = cfg
        .active_provider()
        .unwrap_or_else(|| {
            eprintln!("No active provider");
            std::process::exit(1);
        })
        .clone();

    if select_model {
        println!();
        if let Some(p) = on_the_fly_change_model(&mut cfg.clone(), Some(provider.model())) {
            provider = p;
        } else {
            println!("{}", style(NO_MODELS_FOUND_MSG).red());
        }
    };

    let mut last_suggestion = None::<String>;

    'outer: loop {
        println!();

        let query = initial_query.clone().unwrap_or_else(|| {
            let msg = if last_suggestion.is_some() {
                "How should this be revised?\n"
            } else {
                "What would you like the shell command to do?\n"
            };
            let prompt = format! {"{} {}", style("?").green().bold(), style(msg).bold()};
            let query = dialoguer::Input::<String>::new()
                .with_prompt(prompt)
                .allow_empty(false)
                .interact()
                .unwrap_or_default();

            println!();
            query
        });

        initial_query = None;

        if query.trim().is_empty() {
            return;
        }

        // println!();

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

        'inner: loop {
            let Ok(selection) = dialoguer::Select::with_theme(&get_select_theme())
                .with_prompt("Select an option")
                .items(&options)
                .default(0)
                .interact()
            else {
                std::process::exit(0);
            };

            match selection {
                0 => {
                    let _ = copy_to_clipboard(&suggested_command)
                        .map_err(|e| eprintln!("{} Error: {}\n", style("✗").red().bold(), e));
                    break 'outer; // Exit
                }
                1 => {
                    ai::explain(cfg, Some(suggested_command.clone()), false);
                    std::thread::sleep(Duration::from_millis(500));
                    continue; // Continue to the next iteration of the inner loop
                }
                2 => {
                    last_suggestion = Some(suggested_command.clone());
                    // let _ = term.clear_last_lines(1);
                    break 'inner; // Continue to the next iteration of the outer loop
                }
                3 => {
                    last_suggestion = None;
                    break 'inner; // Continue to the next iteration of the outer loop
                }
                _ => {
                    break 'outer; // Exit
                }
            }
        }
    }

    std::process::exit(0);
}
