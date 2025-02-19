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

        let welome_msg = style("Welcome to TermAI - Your AI in the Terminal").bold();
        let version_msg = style(format!("version {} ({})", VERSION, RELEASE_DATE)).dim();
        println!("\n{welome_msg}\n{version_msg}");

        let model = program.cfg.active_model().unwrap_or("None".into());
        let active_model = format!("{} {}", style("Active model:").bold(), style(model).cyan());
        println!("\n{active_model}");

        if let Some(choice) = program.args.get(1) {
            program.select(&choice.to_owned());
        } else {
            program.main_menu();
        };
    }

    /////////////////////////////
    //          Menus          //
    /////////////////////////////

    fn main_menu(&mut self) {
        let items = if self.cfg.active_provider_name().is_some() {
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
        let items = if self.cfg.active_provider_name().is_some() {
            let stream_choice = if self.cfg.streaming() {
                "Disable streaming"
            } else {
                "Enable streaming (experimental)"
            };
            vec!["Providers", "Change Model", "Changelog", stream_choice]
        } else {
            vec!["Providers", "Changelog"]
        };

        let Ok(selection) = Select::new().items(&items).default(0).interact() else {
            return;
        };

        let selected_option = items[selection].to_lowercase();
        let selected_option = selected_option.as_str();

        match selected_option {
            "providers" => self.provider_menu(),
            "change model" => self.select_model_menu(self.cfg.active_provider_name().unwrap()),
            "Disable streaming" | "Enable streaming (experimental)" => self.cfg.toggle_streaming(),
            "changelog" => changelog::print_latest(),
            _ => unreachable!(),
        }
    }

    fn provider_menu(&mut self) {
        let items = vec![ProviderName::OpenAI];
        let Ok(selection) = Select::new().items(&items).default(0).interact() else {
            return;
        };

        self.provider_inner_menu(items[selection]);
    }

    fn provider_inner_menu(&mut self, provider: ProviderName) {
        let items = if self.cfg.is_configured(provider) {
            if self.cfg.active_provider_name() == Some(provider) {
                vec!["Change API Key", "Remove provider"]
            } else {
                vec!["Change API Key", "Remove Provider", "Set as active"]
            }
        } else {
            vec!["Add API Key"]
        };

        let Ok(selection) = Select::new().items(&items).default(0).interact() else {
            return;
        };

        match selection {
            0 => self.configure_provider(provider),
            2 => self.cfg.remove_provider(provider),
            3 => self.cfg.set_active_provider(provider),
            _ => unreachable!(),
        }
    }

    fn select_model_menu(&mut self, provider: ProviderName) {
        let items = match provider {
            ProviderName::OpenAI => vec!["gpt-4o", "gpt-4o-mini"],
        };

        let Ok(selection) = Select::new().items(&items).default(0).interact() else {
            return;
        };

        self.cfg.set_model(provider, items[selection].into());
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
            "version" | "--version" => {
                println!("TermAI v{} ({})", VERSION, RELEASE_DATE);
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
            "chat" => ai.chat(&provider, rest_args, self.cfg.streaming()),
            "suggest" => ai.suggest(&provider, rest_args),
            "explain" => ai.explain(&provider, rest_args),
            "ask" => ai.ask(&provider, rest_args),
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
        println!("  options        Configure TermAI");
    }
}
