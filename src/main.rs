mod ai;
mod args;
mod client;
mod config;
mod mcp;
mod program;
mod provider;
mod utils;

use program::Program;

#[tokio::main]
async fn main() {
    let Ok(_) = ctrlc::set_handler(move || {
        let term = console::Term::stdout();
        let _ = term.flush();
        let _ = term.show_cursor();
        std::process::exit(0);
    }) else {
        eprintln!("Failed to set Ctrl-C handler");
        std::process::exit(1);
    };

    Program::run().await;
}
