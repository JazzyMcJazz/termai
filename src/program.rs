use console::{style, Term};
use dialoguer::Select;
use std::env;

use crate::{
    ai::AI,
    config::Config,
    utils::{changelog, enums::ProviderName},
};

static VERSION: &str = env!("CARGO_PKG_VERSION");
static RELEASE_DATE: &str = env!("RELEASE_DATE");

pub struct Program {
    term: Term,
    cfg: Config,
    args: Vec<String>,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            term: Term::stdout(),
            cfg: Config::load(),
            args: env::args().collect(),
        }
    }
}

impl Program {
    pub fn run() {
        let mut program = Program::default();
        let choice = program.args.get(1);

        if choice == Some(&"--version".to_string()) {
            println!("TermAI v{} ({})", VERSION, RELEASE_DATE);
            return;
        }

        let welome_msg = style("Welcome to TermAI - Your AI in the Terminal").bold();
        let version_msg = style(format!("version {} ({})", VERSION, RELEASE_DATE)).dim();
        println!("\n{welome_msg}\n{version_msg}");

        let model = program.cfg.active_model().unwrap_or("None".into());
        let active_model = format!("{} {}", style("Active model:").bold(), style(model).cyan());
        println!("\n{active_model}");

        if let Some(choice) = choice {
            program.select(&choice.to_owned());
        } else {
            program.main_menu();
        };
    }

    /////////////////////////////
    //          Menus          //
    /////////////////////////////

    fn main_menu(&mut self) {
        let items = if self.cfg.active_provider().is_some() {
            vec!["Chat", "Suggest", "Explain", "Ask", "Options", "Exit"]
        } else {
            vec!["Options", "Exit"]
        };

        let prompt =
            format! {"\n{} {}", style("?").green().bold(), style("What do you want to do?").bold()};

        let Ok(selection) = Select::new()
            .with_prompt(prompt)
            .items(&items)
            .default(0)
            .interact()
        else {
            return;
        };

        self.select(&items[selection].to_lowercase());
    }

    fn options_menu(&mut self) {
        let items = if self.cfg.active_provider().is_some() {
            let stream_choice = if self.cfg.streaming() {
                "Disable streaming"
            } else {
                "Enable streaming (experimental)"
            };
            vec![
                "Configure Provider",
                "Change Model",
                "Changelog",
                stream_choice,
            ]
        } else {
            vec!["Configure Provider", "Changelog"]
        };

        let prompt =
            format! {"\n{} {}", style("?").green().bold(), style("What do you want to do?").bold()};

        let Ok(selection) = Select::new()
            .with_prompt(prompt)
            .items(&items)
            .default(0)
            .interact()
        else {
            return;
        };

        let selected_option = items[selection].to_lowercase();
        let selected_option = selected_option.as_str();

        match selected_option {
            "configure provider" => self.provider_menu(),
            "change model" => self.select_model_menu(),
            "disable streaming" | "enable streaming (experimental)" => self.cfg.toggle_streaming(),
            "changelog" => changelog::print_latest(),
            _ => unreachable!(),
        }
    }

    fn provider_menu(&mut self) {
        let items = ProviderName::iter();

        let prompt = format! {"\n{} {}", style("?").green().bold(), style("Provider").bold()};

        let Ok(selection) = Select::new()
            .with_prompt(prompt)
            .items(&items)
            .default(0)
            .interact()
        else {
            return;
        };

        self.provider_inner_menu(items[selection]);
    }

    fn provider_inner_menu(&mut self, provider_name: ProviderName) {
        let items = if self.cfg.is_configured(provider_name) {
            vec!["Change API Key", "Remove provider"]
        } else {
            vec!["Add API Key"]
        };

        let prompt =
            format! {"\n{} {}", style("?").green().bold(), style("What do you want to do?").bold()};

        let Ok(selection) = Select::new()
            .with_prompt(prompt)
            .items(&items)
            .default(0)
            .interact()
        else {
            return;
        };

        match selection {
            0 => self.configure_provider(provider_name),
            1 => self.cfg.remove_provider(provider_name),
            _ => unreachable!(),
        }
    }

    fn select_model_menu(&mut self) {
        let provider_models = self.cfg.fetch_available_models();

        let items = provider_models
            .iter()
            .map(|(provider, _, display_name)| {
                let spaces: String = (0..18 - display_name.to_string().len())
                    .map(|_| ' ')
                    .collect();
                format!("{}{} {}", display_name, spaces, provider)
            })
            .collect::<Vec<String>>();

        let Ok(selection) = Select::new().items(&items).default(0).interact() else {
            return;
        };

        let Some((provider_name, model, _)) = provider_models.get(selection) else {
            println!("Invalid selection");
            return;
        };

        self.cfg.set_model(*provider_name, model.to_owned());
    }

    /////////////////////////////////
    //           Helpers           //
    /////////////////////////////////

    fn select(&mut self, choice: &str) {
        match choice {
            "options" => {
                self.options_menu();
                return;
            }
            "changelog" => {
                changelog::print_latest();
                return;
            }
            "exit" => return,
            "chat" | "suggest" | "explain" | "ask" => {}
            _ => {
                Program::help();
                return;
            }
        }

        let Some(provider) = self.cfg.active_provider() else {
            let cross = style("✗").red().bold();
            let msg = style("You need to configure a provider first").bold();
            println!("{cross} {msg} {cross}", cross = cross, msg = msg);
            println!("  Run `termai options` to configure a provider");
            return;
        };

        let rest_args = if self.args.len() > 2 {
            Some(self.args[2..].join(" "))
        } else {
            None
        };

        let ai = AI(&self.term);
        match choice {
            "chat" => ai.chat(provider, rest_args, self.cfg.streaming()),
            "suggest" => ai.suggest(provider, rest_args),
            "explain" => ai.explain(provider, rest_args),
            "ask" => ai.ask(provider, rest_args),
            _ => Program::help(),
        }
    }

    fn configure_provider(&mut self, provider_name: ProviderName) {
        let Ok(api_key) = dialoguer::Password::new()
            .with_prompt(format!("Enter your {:?} API key", provider_name))
            .allow_empty_password(false)
            .interact()
        else {
            return;
        };

        self.cfg.store(provider_name, api_key);
    }

    fn help() {
        println!("\nUsage: termai [OPTION] [ARG]\n");
        println!("Options:");
        println!("  chat    [ARG]  Chat with the AI (optional string argument)");
        println!("  suggest [ARG]  Get suggestions from the AI (optional string argument)");
        println!("  explain [ARG]  Get explanations from the AI (optional string argument)");
        println!("  ask     [ARG]  Similar to chat, but the program exits after one response (optional string argument)");
        println!("  options        Configure TermAI");
    }
}
