use std::time::Duration;

use console::{style, Term};
use indicatif::ProgressBar;
use rustyline::DefaultEditor;
use termimad::MadSkin;

use crate::{
    client::{ChatMessage, ChatRole},
    provider::Provider,
    utils::term_tools::get_spinner_style,
};

pub fn chat(
    term: &Term,
    provider: &Provider,
    mut initial_message: Option<String>,
    streaming: bool,
) {
    let exit_words = [
        "exit".into(),
        "q".into(),
        "quit".into(),
        "goodbye".into(),
        "thanks".into(),
    ];

    let ai = style("AI:").bold().green();
    let user = style("You:").bold().cyan();

    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    let skin = MadSkin::default();
    let mut spinner: ProgressBar;
    let spinner_style = get_spinner_style();

    let mut messages = vec![];
    if initial_message.is_none() {
        println!("\n{ai}\nWhat can I help with?");
    }

    loop {
        spinner = ProgressBar::new_spinner();
        spinner.set_style(spinner_style.clone());

        println!("\n{user}");

        let mut input = String::new();
        if let Some(message) = initial_message.take() {
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

        println!();

        if exit_words.contains(&input.trim().to_lowercase()) {
            if input.trim().to_lowercase() == "thanks" {
                println!("{ai}\nYou're welcome! 😊\nGoodbye! 👋\n");
            } else {
                println!("{ai}\nGoodbye! 👋\n");
            }
            return;
        }

        if input.trim() == "clear" {
            term.clear_screen().expect("Failed to clear screen");
            messages.clear();
            println!("{ai}\nWhat can I help with?");
            continue;
        }

        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(format!("{ai}"));

        messages.push(ChatMessage {
            role: ChatRole::User,
            content: input.trim().into(),
            refusal: None,
        });

        if streaming {
            let mut response = String::new();
            let content_iter = provider.chat_stream(&messages);
            let mut line_count = 0;
            let term_width = term.size().1 as usize;

            term.hide_cursor().expect("Failed to hide cursor");

            for (i, content) in content_iter.enumerate() {
                if i == 0 {
                    spinner.finish_and_clear();
                    println!("{ai}");
                }

                response.push_str(&content);

                let text = skin.text(&response, None);
                let mut new_line_count = text.lines.len();

                for line in text.to_string().lines() {
                    if line.len() > term_width {
                        new_line_count += line.len() / term_width;
                    }
                }

                if line_count > 0 {
                    term.clear_last_lines(line_count)
                        .expect("Failed to clear lines");
                }

                skin.print_text(&response);

                line_count = new_line_count;
            }

            term.show_cursor().expect("Failed to show cursor");
            term.flush().expect("Failed to flush terminal");
        } else {
            let response = provider.chat(&messages);

            spinner.finish_and_clear();
            println!("{ai}");
            skin.print_text(&response);
        }
    }
}
