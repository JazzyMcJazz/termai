use console::{style, Term};
use dialoguer::{MultiSelect, Select};
use std::{env, time::Duration};

use crate::{
    ai::AI,
    args::{Args, ChatArgs},
    config::Config,
    mcp::{McpClient, McpClientConfig},
    utils::{
        changelog,
        console::{get_select_theme, get_spinner_style},
        enums::ProviderName,
    },
};

pub static VERSION: &str = env!("CARGO_PKG_VERSION");
static RELEASE_DATE: &str = env!("RELEASE_DATE");

pub struct Program {
    term: Term,
    cfg: Config,
    args: Args,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            term: Term::stdout(),
            cfg: Config::load(),
            args: Args::new(VERSION),
        }
    }
}

impl Program {
    pub async fn run() {
        let mut program = Program::default();

        let welome_msg = style("Welcome to TermAI - Your AI in the Terminal").bold();
        let version_msg = style(format!("version {} ({})", VERSION, RELEASE_DATE)).dim();
        println!("\n{welome_msg}\n{version_msg}");

        let model = program
            .cfg
            .active_model()
            .unwrap_or(("".into(), "None".into()));
        let active_model = format!(
            "{} {}",
            style("Active model:").bold(),
            style(model.1).cyan()
        );
        println!("\n{active_model}");

        match program.args {
            Args::None => program.main_menu().await,
            _ => program.handle_args().await,
        }
    }

    /////////////////////////////
    //          Menus          //
    /////////////////////////////

    async fn main_menu(&mut self) {
        println!();

        let items = if self.cfg.active_provider().is_some() {
            vec!["Chat", "Suggest", "Explain", "Options", "Exit"]
        } else {
            vec!["Options", "Exit"]
        };

        let mut selection = 0;
        loop {
            let Ok(s) = Select::with_theme(&get_select_theme())
                .with_prompt("What do you want to do?")
                .items(&items)
                .default(selection)
                .interact()
            else {
                std::process::exit(0);
            };

            selection = s;
            self.handle_main_menu_choice(&items[selection].to_lowercase(), None)
                .await;
        }
    }

    async fn options_menu(&mut self) {
        let mut selection = 0;
        loop {
            let items = if self.cfg.active_provider().is_some() {
                let stream_choice = if self.cfg.streaming() {
                    "Disable streaming"
                } else {
                    "Enable streaming (experimental)"
                };
                vec![
                    "Configure Provider",
                    "Change Model",
                    stream_choice,
                    "Model Context Protocol (MCP)",
                    "Changelog",
                    "Back",
                ]
            } else {
                vec!["Configure Provider", "Changelog", "Back"]
            };

            let _ = self.term.clear_last_lines(1);
            let Ok(s) = Select::with_theme(&get_select_theme())
                .with_prompt("Options")
                .items(&items)
                .default(selection)
                .interact()
            else {
                std::process::exit(0);
            };

            selection = s;
            let selected_option = items[selection].to_lowercase();
            let selected_option = selected_option.as_str();

            match selected_option {
                "configure provider" => self.provider_menu().await,
                "change model" => self.select_model_menu().await,
                "disable streaming" | "enable streaming (experimental)" => {
                    self.cfg.toggle_streaming()
                }
                "model context protocol (mcp)" => self.mcp_menu().await,
                "changelog" => {
                    changelog::print_latest();
                    std::process::exit(0);
                }
                "back" => {
                    let _ = self.term.clear_last_lines(1);
                    return;
                }
                _ => unreachable!(),
            }
        }
    }

    async fn provider_menu(&mut self) {
        let items = ProviderName::iter();

        let _ = self.term.clear_last_lines(1);
        let Ok(selection) = Select::with_theme(&get_select_theme())
            .with_prompt("Select provider")
            .items(&items)
            .default(0)
            .interact()
        else {
            std::process::exit(0);
        };

        self.provider_inner_menu(items[selection]).await;
    }

    async fn provider_inner_menu(&mut self, provider_name: ProviderName) {
        let is_configured = self.cfg.is_configured(provider_name);

        let items = if is_configured {
            vec!["Change API Key", "Remove provider", "Back"]
        } else {
            vec!["Add API Key", "Back"]
        };

        let _ = self.term.clear_last_lines(1);
        let Ok(selection) = Select::with_theme(&get_select_theme())
            .with_prompt(provider_name.to_string())
            .items(&items)
            .default(0)
            .interact()
        else {
            std::process::exit(0);
        };

        match selection {
            0 => self.configure_provider(provider_name).await,
            1 => {
                if is_configured {
                    self.cfg.remove_provider(provider_name)
                }
            }
            2 => (),
            _ => unreachable!(),
        }
    }

    async fn select_model_menu(&mut self) {
        self.cfg.refresh_available_models().await;
        let provider_models = self.cfg.get_available_models().to_owned();

        let active_model = if let Some(model) = self.cfg.active_model() {
            provider_models
                .iter()
                .position(|(_, m, _)| *m == model.0)
                .unwrap_or(0)
        } else {
            0
        };

        let items = provider_models
            .iter()
            .map(|(provider, _, display_name)| {
                let spaces: String = (0..26 - display_name.to_string().len())
                    .map(|_| ' ')
                    .collect();
                format!("{}{} {}", display_name, spaces, provider)
            })
            .collect::<Vec<String>>();

        let _ = self.term.clear_last_lines(1);
        let Ok(selection) = Select::with_theme(&get_select_theme())
            .with_prompt("Select model")
            .items(&items)
            .default(active_model)
            .interact()
        else {
            std::process::exit(0);
        };

        let Some((provider_name, model_id, model_name)) = provider_models.get(selection) else {
            println!("Invalid selection");
            return;
        };

        self.cfg.set_model(*provider_name, model_id.to_owned());

        let active_model = format!(
            "{} {}",
            style("Active model:").bold(),
            style(model_name).cyan()
        );

        let _ = self.term.clear_last_lines(3);
        println!("{active_model}\n\n");
    }

    async fn mcp_menu(&mut self) {
        let mut selection = 0;
        loop {
            let items = vec![
                "Configure new MCP server",
                "Enable/disable MCP servers",
                "Remove MCP server",
                "Back",
            ];

            let _ = self.term.clear_last_lines(1);
            selection = Select::with_theme(&get_select_theme())
                .with_prompt("Model Context Protocol (MCP)")
                .items(&items)
                .default(selection)
                .interact()
                .unwrap_or_else(|_| std::process::exit(0));

            match selection {
                0 => self.prompt_mcp_options().await,
                1 => self.select_enabled_mcp_servers(),
                2 => self.remove_mcp_client().await,
                _ => break,
            }
        }
    }

    /////////////////////////////////
    //           Helpers           //
    /////////////////////////////////

    async fn handle_args(&mut self) {
        match &self.args {
            Args::Chat((command, args)) => {
                self.handle_main_menu_choice(command, Some(args.to_owned()))
                    .await
            }
            Args::Suggest((command, args)) => {
                self.handle_main_menu_choice(command, Some(args.to_owned()))
                    .await
            }
            Args::Explain((command, args)) => {
                self.handle_main_menu_choice(command, Some(args.to_owned()))
                    .await
            }
            Args::Options => {
                println!();
                println!();
                self.options_menu().await
            }
            Args::Changelog => changelog::print_latest(),
            Args::None => unreachable!(),
        }
    }

    async fn handle_main_menu_choice(&mut self, choice: &str, args: Option<ChatArgs>) {
        match choice {
            "options" => {
                self.options_menu().await;
                return;
            }
            "changelog" => {
                changelog::print_latest();
                return;
            }
            "exit" => std::process::exit(0),
            "chat" | "suggest" | "explain" | "ask" => {}
            _ => {
                Program::help();
                return;
            }
        }

        // Check if a provider is configured
        if self.cfg.active_provider().is_none() {
            let cross = style("✗").red().bold();
            let msg = style("You need to configure a provider first").bold();
            println!("{cross} {msg} {cross}", cross = cross, msg = msg);
            println!("  Run `termai options` to configure a provider");
            std::process::exit(1);
        };

        let (prompt, model, search) = if let Some(args) = args {
            (args.prompt(), args.model(), args.search())
        } else {
            (None, false, false)
        };

        let mut ai = AI::new(&self.term, &mut self.cfg);
        match choice {
            "chat" => ai.chat(prompt, model, search).await,
            "suggest" => ai.suggest(prompt, model).await,
            "explain" => ai.explain(prompt, model).await,
            _ => Program::help(),
        }

        std::process::exit(0);
    }

    async fn configure_provider(&mut self, provider_name: ProviderName) {
        let Ok(api_key) = dialoguer::Password::new()
            .with_prompt(format!("Enter your {:?} API key", provider_name))
            .allow_empty_password(false)
            .interact()
        else {
            return;
        };

        self.cfg.add_provider_api_key(provider_name, api_key).await;
    }

    async fn prompt_mcp_options(&mut self) {
        let selection = dialoguer::Select::with_theme(&get_select_theme())
            .with_prompt("Select connection type".to_string())
            .default(0)
            .items(&["Program", "SSE", "Cancel"])
            .clear(true)
            .interact()
            .unwrap_or_else(|_| std::process::exit(0));

        if selection == 2 {
            let _ = self.term.clear_last_lines(1);
            return;
        }

        let is_sse = selection == 1;

        let command_or_url_prompt = if is_sse { "URL" } else { "Command" };

        let command_or_url_prompt = format!(
            "{} {}",
            style("›").green(),
            style(command_or_url_prompt).bold()
        );

        let command_or_url = dialoguer::Input::<String>::new()
            .with_prompt(style(command_or_url_prompt).bold().to_string())
            .allow_empty(false)
            .interact()
            .unwrap_or_else(|_| std::process::exit(0));

        let args = if is_sse {
            None
        } else {
            let prompt = format!("{} {}", style("›").green(), style("Arguments").bold());
            Some(
                dialoguer::Input::<String>::new()
                    .with_prompt(style(prompt).bold().to_string())
                    .allow_empty(true)
                    .interact()
                    .unwrap_or_else(|_| std::process::exit(0))
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            )
        };

        let mut client: McpClient = match args {
            Some(args) => {
                McpClientConfig::StdIo(String::new(), String::new(), command_or_url, args, false)
            }
            None => McpClientConfig::Sse(String::new(), String::new(), command_or_url, false),
        }
        .into();

        println!();
        let message = format!(
            "{} {}",
            style("⏳").bold(),
            style("Testing connection...").bold()
        );
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_style(get_spinner_style());
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(100));

        tokio::time::sleep(Duration::from_millis(1000)).await;

        enum McpInit {
            Succes,
            Failure,
            Duplicate,
        }

        // Test the connection
        let mut result = match client.initialize().await {
            Ok(_) => McpInit::Succes,
            Err(_) => McpInit::Failure,
        };

        if self
            .cfg
            .mcp_clients()
            .iter()
            .filter(|c| c.name() == client.name())
            .count()
            > 0
        {
            result = McpInit::Duplicate;
        }

        spinner.finish_and_clear();
        self.term.clear_last_lines(1).unwrap_or(());

        // deno -A /home/lr/Development/mcp-playground/server/main.ts --stdio

        let message = match result {
            McpInit::Succes => {
                self.cfg.add_mcp_client(client);
                format!(
                    "{} {}",
                    style("✔").green(),
                    style("Connection successful. Configuration saved.").bold()
                )
            }
            McpInit::Duplicate => {
                format!(
                    "{} {} {}",
                    style("✗").red(),
                    style(client.name()).bold().italic(),
                    style("is already configured.").bold(),
                )
            }
            McpInit::Failure => {
                format!(
                    "{} {}",
                    style("✗").red(),
                    style("Connection failed.").bold()
                )
            }
        };

        println!("{message}\n\n");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    fn select_enabled_mcp_servers(&mut self) {
        let clients = self.cfg.mcp_clients_mut();

        let items = clients
            .iter()
            .map(|client| format!("{} ({})", client.name(), client.version()))
            .collect::<Vec<String>>();

        let defaults = clients
            .iter()
            .map(|client| client.is_enabled())
            .collect::<Vec<bool>>();

        let _ = self.term.clear_last_lines(1);
        let Ok(selections) = MultiSelect::with_theme(&get_select_theme())
            .with_prompt("Select MCP server (press Enter to confirm)")
            .items(&items[..])
            .defaults(&defaults[..])
            .interact()
        else {
            std::process::exit(0);
        };

        for (i, client) in clients.iter_mut().enumerate() {
            client.set_enabled(selections.contains(&i));
        }

        self.cfg.save();
    }

    async fn remove_mcp_client(&mut self) {
        let clients = self.cfg.mcp_clients_mut();

        let mut items = clients
            .iter()
            .map(|client| format!("{} ({})", client.name(), client.version()))
            .collect::<Vec<String>>();

        items.push("Cancel".to_string());

        let _ = self.term.clear_last_lines(1);
        let selection = Select::with_theme(&get_select_theme())
            .with_prompt("Select MCP server to remove")
            .items(&items[..])
            .default(0)
            .interact()
            .unwrap_or_else(|_| std::process::exit(0));

        if selection == items.len() - 1 {
            return;
        }

        clients.remove(selection);
        self.cfg.save();
    }

    fn help() {
        println!("\nUsage: termai [OPTION] [ARG]\n");
        println!("Options:");
        println!("  chat    [ARG]  Chat with the AI (optional string argument)");
        println!("  suggest [ARG]  Get suggestions from the AI (optional string argument)");
        println!("  explain [ARG]  Get explanations from the AI (optional string argument)");
        println!("  options        Configure TermAI");
    }
}
