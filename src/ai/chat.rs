use std::{
    io::{stdout, Write},
    time::Duration,
};

use console::{strip_ansi_codes, style, Term};
use indicatif::{ProgressBar, TermLike};
use rustyline::DefaultEditor;
use termimad::MadSkin;
use textwrap::wrap;

use crate::{
    ai::utils::{on_the_fly_change_model, NO_MODELS_FOUND_MSG},
    client::{ChatMessage, ChatRole},
    config::Config,
    utils::console::get_spinner_style,
};

pub fn chat(term: &Term, cfg: &Config, mut initial_message: Option<String>, select_model: bool) {
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

    let exit_words = ["exit".into(), "q".into(), "quit".into(), "goodbye".into()];

    let ai = style("AI:").bold().green();
    let user = style("You:").bold().cyan();

    let mut rl = DefaultEditor::new().expect("Failed to create editor");
    let skin = MadSkin::default();
    let mut spinner: ProgressBar;
    let spinner_style = get_spinner_style();
    let mut select_model = false;

    // In memory message history
    let mut messages = vec![];

    if initial_message.is_none() {
        println!("\n{ai}\nWhat can I help with?\n");
    } else {
        println!();
    }

    loop {
        spinner = ProgressBar::new_spinner();
        spinner.set_style(spinner_style.clone());

        if select_model {
            select_model = false;
            if let Some(p) = on_the_fly_change_model(&mut cfg.clone(), Some(provider.model())) {
                provider = p;
                println!();
            } else {
                println!("{}", style(NO_MODELS_FOUND_MSG).red());
            }
        }

        println!("{user}");

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

        if input.trim().eq("/model") {
            select_model = true;
            println!();
            continue;
        }

        println!();

        if exit_words.contains(&input.trim().to_lowercase()) {
            println!("{ai}\nGoodbye! 👋\n");
            std::process::exit(0);
        }

        if input.trim() == "clear" {
            term.clear_screen().expect("Failed to clear screen");
            messages.clear();
            println!("{ai}\nWhat can I help with?\n");
            continue;
        }

        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(format!("{ai}"));

        messages.push(ChatMessage {
            role: ChatRole::User,
            content: input.trim().into(),
        });

        if cfg.streaming() {
            let mut response = String::new();
            let content_iter = provider.chat_stream(&messages);
            let mut line_count = 0;

            term.hide_cursor().expect("Failed to hide cursor");

            for (i, content) in content_iter.enumerate() {
                let term_width = term.width() as usize;

                if i == 0 {
                    spinner.finish_and_clear();
                    println!("{ai}");
                }

                response.push_str(&content);

                let text = format!("{}", skin.text(&response, Some(term_width)));

                if line_count > 0 {
                    clear_lines(line_count);
                }

                println!("{}", text);

                if response.ends_with("\n\n") {
                    response.clear();
                    line_count = 1;
                } else {
                    line_count = count_wrapped_lines(&text, term_width);
                }
            }

            term.show_cursor().expect("Failed to show cursor");
            term.flush().expect("Failed to flush terminal");
        } else {
            let response = provider.chat(&messages);

            spinner.finish_and_clear();
            println!("{ai}");
            skin.print_text(&response);
            println!();
        }
    }
}

fn count_wrapped_lines(rendered: &str, width: usize) -> usize {
    let plain_text = strip_ansi_codes(rendered);

    let wrapped_lines = wrap(&plain_text, width);

    wrapped_lines.len()
}

fn clear_lines(num_lines: usize) {
    let mut out = stdout();
    for _ in 0..num_lines {
        write!(out, "\x1B[A").unwrap(); // Move cursor up
        write!(out, "\x1B[2K").unwrap(); // Clear line
    }
    out.flush().unwrap();
}
