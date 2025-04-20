use std::time::Duration;

use indicatif::ProgressBar;

use crate::utils::console::get_select_theme;

pub fn confirm_tool_call(name: &str, spinner: Option<&ProgressBar>) -> bool {
    if let Some(spinner) = spinner {
        spinner.disable_steady_tick();
        let _ = dialoguer::console::Term::stdout().clear_last_lines(2);
        println!();
    }

    let confirmation = dialoguer::Select::with_theme(&get_select_theme())
        .with_prompt(format!("Run tool '{name}'?"))
        .default(0)
        .items(&["Yes", "No"])
        .clear(true)
        .interact()
        .unwrap_or(0)
        == 0;

    if let Some(spinner) = spinner {
        spinner.enable_steady_tick(Duration::from_millis(100));
    }

    confirmation
}
