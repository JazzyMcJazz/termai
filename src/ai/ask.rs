use std::time::Duration;

use console::{style, Term};
use indicatif::ProgressBar;
use rustyline::DefaultEditor;
use termimad::MadSkin;

use crate::{
    client::{ChatMessage, ChatRole},
    config::Config,
    utils::console::get_spinner_style,
};

pub fn ask(term: &Term, cfg: &Config, mut message: Option<String>) {
    let provider = cfg.active_provider().unwrap_or_else(|| {
        eprintln!("No active provider");
        std::process::exit(1);
    });

    let ai = style("AI:").bold().green();
    let user = style("You:").bold().cyan();

    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    let skin = MadSkin::default();
    let spinner: ProgressBar;
    let spinner_style = get_spinner_style();

    let mut messages = vec![];
    if message.is_none() {
        println!("\n{ai}\nWhat do you want to ask?");
    }

    spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style.clone());

    println!("\n{user}");

    let mut input = String::new();
    if let Some(message) = message.take() {
        println!("{message}");
        input = message;
    } else {
        println!();
        while input.trim().is_empty() {
            term.clear_last_lines(1).expect("Failed to clear last line");
            input.clear();
            input = rl.readline("").unwrap_or("q".into());
        }
    }

    if input.trim().eq("q") {
        return;
    }

    println!();

    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(format!("{ai}"));

    messages.push(ChatMessage {
        role: ChatRole::User,
        content: input.trim().into(),
    });

    let response = provider.chat(&messages);

    spinner.finish_and_clear();
    println!("{ai}");
    skin.print_text(&response);
    println!();

    std::process::exit(0);
}
