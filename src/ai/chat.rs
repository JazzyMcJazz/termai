use std::{
    io::{stdout, Write},
    time::Duration,
};

use console::{strip_ansi_codes, style, Term};
use futures::StreamExt;
use indicatif::{ProgressBar, TermLike};
use rig::message::Message;
use termimad::MadSkin;
use textwrap::wrap;

use crate::{
    ai::utils::{on_the_fly_change_model, on_the_fly_select_mcp_client, NO_MODELS_FOUND_MSG},
    client::StreamingContent,
    config::Config,
    editor::{CommandHint, Editor},
    utils::console::get_spinner_style,
};

pub async fn chat(
    term: &Term,
    cfg: &mut Config,
    mut initial_message: Option<String>,
    select_model: bool,
) {
    let mut provider = cfg
        .active_provider()
        .unwrap_or_else(|| {
            eprintln!("No active provider");
            std::process::exit(1);
        })
        .clone();

    if select_model {
        println!();
        if let Some(p) = on_the_fly_change_model(&mut cfg.clone(), Some(provider.model())).await {
            provider = p;
        } else {
            println!("{}", style(NO_MODELS_FOUND_MSG).red());
        }
    };

    let ai = style("AI:").bold().green();
    let user = style("You:").bold().cyan();

    let skin = MadSkin::default();
    let mut spinner: ProgressBar;
    let spinner_style = get_spinner_style();
    let mut streaming = cfg.streaming();

    // In memory message history
    let mut messages = vec![];

    if initial_message.is_none() {
        println!(
            "\n 🚀 {}: Type {} to see available commands\n\n{ai}\nWhat can I help with?\n",
            style("Quick Tip").bold().underlined(),
            style("/help").bold(),
        );
    } else {
        println!();
    }

    let Ok(mut editor) = Editor::new(hints()) else {
        eprintln!("Failed to create editor");
        std::process::exit(1);
    };

    loop {
        spinner = ProgressBar::new_spinner();
        spinner.set_style(spinner_style.clone());

        println!("{user}");

        // Get user input
        let mut input = String::new();
        if let Some(message) = initial_message.take() {
            println!("{message}");
            input = message;
        } else {
            println!();
            while input.trim().is_empty() {
                term.clear_last_lines(1).expect("Failed to clear last line");
                input.clear();
                input = editor.readline().unwrap_or("/quit".into());
            }
        }

        input = input.trim().to_string();

        let _ = editor.append_history(&input);

        if input.starts_with("/model") {
            println!();
            match on_the_fly_change_model(cfg, Some(provider.model())).await {
                Some(p) => provider = p,
                None => println!("{}", style(NO_MODELS_FOUND_MSG).red()),
            }
            println!();
            continue;
        }

        if input.starts_with("/clear") {
            term.clear_screen().expect("Failed to clear screen");
            messages.clear();
            println!("{ai}\nWhat can I help with?\n");
            continue;
        }

        if input.starts_with("/stream") || input.starts_with("/nostream") {
            streaming = input.starts_with("/stream");
            if streaming {
                println!("\nStreaming enabled\n")
            } else {
                println!("\nStreaming disabled\n")
            };
            continue;
        }

        if input.starts_with("/mcp") {
            println!();
            on_the_fly_select_mcp_client(cfg);
            println!();
            continue;
        }

        match editor.execute_command(&input) {
            Some(output) => input = output,
            None => {
                println!();
                continue;
            }
        }

        println!();

        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message(format!("{ai}"));

        if streaming {
            let mut response = String::new();
            let mut final_response = String::new();

            let mut stream = provider
                .chat_stream(&input, messages.clone(), cfg.mcp_clients())
                .await;

            let mut line_count = 0;

            let _ = term.hide_cursor();

            let mut clear = true;
            while let Some(content) = stream.next().await {
                let content = match content {
                    Ok(content) => content,
                    Err(e) => StreamingContent::Text(e.to_string()),
                };

                let content = match content {
                    StreamingContent::Text(text) => text,
                    StreamingContent::PauseSpinner => {
                        spinner.disable_steady_tick();
                        continue;
                    }
                    StreamingContent::StartSpinner => {
                        let _ = term.clear_last_lines(2);
                        if spinner.is_finished() {
                            spinner = ProgressBar::new_spinner();
                            spinner.set_style(spinner_style.clone());
                        }
                        spinner.enable_steady_tick(Duration::from_millis(100));
                        let _ = term.hide_cursor(); // confirmation prompt shows cursor after interaction
                        clear = true;
                        continue;
                    }
                };

                let term_width = term.width() as usize;

                if clear {
                    clear = false;
                    spinner.finish_and_clear();
                    println!("{ai}");
                }

                response.push_str(&content);
                final_response.push_str(&content);

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

            messages.push(Message::user(&input));
            messages.push(Message::assistant(final_response));

            let _ = term.show_cursor();
            let _ = term.flush();
        } else {
            let response = match provider
                .chat(&input, messages.clone(), cfg.mcp_clients(), &spinner)
                .await
            {
                Ok(response) => response,
                Err(e) => e.to_string(),
            };

            spinner.finish_and_clear();
            println!("{ai}");
            skin.print_text(&response);
            println!();

            messages.push(Message::user(&input));
            messages.push(Message::assistant(response));
        }
    }
}

fn hints() -> Vec<CommandHint> {
    vec![
        // Handled with custom logic (due to needing outside references)
        CommandHint::new("/model", "/model", Box::new(|_| None)),
        CommandHint::new("/clear", "/clear", Box::new(|_| None)),
        CommandHint::new("/stream", "/stream", Box::new(|_| None)),
        CommandHint::new("/nostream", "/nostream", Box::new(|_| None)),
        CommandHint::new("/mcp", "/mcp", Box::new(|_| None)),
        // Handled dynamically
        CommandHint::new(
            "/quit",
            "/quit",
            Box::new(|_| {
                println!("\n{}\nGoodbye! 👋\n", style("AI:").bold().green());
                std::process::exit(0);
            }),
        ),
        CommandHint::new(
            "/help",
            "/help",
            Box::new(|_| {
                let s = |s: String| style(s).bold();
                println!("\n{}", style("Slash Commands:").bold().underlined());
                println!("  {}    - Change the active model", s("/model".into()));
                println!("  {}      - Enable / disable MCP servers", s("/mcp".into()));
                println!(
                    "  {}    - Clear the screen and chat history",
                    s("/clear".into())
                ); // /clear
                println!("  {}   - Enable streaming", s("/stream".into()));
                println!("  {} - Disable streaming", s("/nostream".into()));
                println!("  {}     - Exit TermAI", s("/quit".into()));
                println!("  {}     - Show this help message", s("/help".into()));
                None
            }),
        ),
    ]
}

fn count_wrapped_lines(rendered: &str, width: usize) -> usize {
    let plain_text = strip_ansi_codes(rendered);

    let wrapped_lines = wrap(&plain_text, width);

    wrapped_lines.len()
}

fn clear_lines(num_lines: usize) {
    let mut out = stdout();
    for _ in 0..num_lines {
        let _ = write!(out, "\x1B[A"); // Move cursor up
        let _ = write!(out, "\x1B[2K"); // Clear line
    }
    let _ = out.flush();
}
