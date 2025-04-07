mod ai;
mod args;
mod client;
mod config;
mod program;
mod provider;
mod utils;

use program::Program;

fn main() {
    ctrlc::set_handler(move || {
        let term = console::Term::stdout();
        term.flush().expect("Failed to flush terminal");
        term.show_cursor().expect("Failed to show cursor");
        std::process::exit(0);
    })
    .expect("Failed to set Ctrl-C handler");

    Program::run();
}
