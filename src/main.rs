mod ai;
mod args;
mod client;
mod config;
// mod mcp;
mod program;
mod provider;
mod utils;

use program::Program;

#[tokio::main]
async fn main() {
    ctrlc::set_handler(move || {
        let term = console::Term::stdout();
        term.flush().expect("Failed to flush terminal");
        term.show_cursor().expect("Failed to show cursor");
        std::process::exit(0);
    })
    .expect("Failed to set Ctrl-C handler");

    // let command = "deno";
    // let args = vec!["-A", "/home/lr/Development/mcp-playground/server/main.ts", "--stdio"];
    // let client = mcp::McpClient::new(command, args).await;
    // client.tools().await;
    // client.cancel().await;

    Program::run().await;
}
