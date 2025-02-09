use std::time::Duration;

use console::style;
use indicatif::ProgressBar;

use crate::{
    ai,
    provider::Provider,
    utils::{commands::copy_to_clipboard, term_tools::get_spinner_style},
};

pub fn suggest(provider: &Provider, query: Option<String>) {
    let query = query.unwrap_or_else(|| {
        let prompt = format!{"{} {}", style("?").green().bold(), style("What would you like the shell command to to?\n").bold()};
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

    let suggested_command = provider.suggest(&query).clone();
    // let suggested_command = "git commit -m \"Add new feature\"";

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
        // "Revise command",
        // "New command",
        "Exit",
    ];

    loop {
        let prompt = format! {"{} {}", style("?").green().bold(), style("Select an option").bold()};
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
                break;
            }
            1 => {
                ai::explain(provider, Some(suggested_command.clone()));
                std::thread::sleep(Duration::from_millis(500));
            }
            // 2 => {
            //     term.write_line("[unimplemented] Command revised").expect("Failed to write line");
            //     break;
            // }
            // 3 => {
            //     term.write_line("[unimplemented] New command").expect("Failed to write line");
            //     break;
            // }
            _ => {
                break;
            }
        }
    }
}
