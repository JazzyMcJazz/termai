use console::{style, Term};
use dialoguer::Select;
use std::env;

use crate::{ai::AI, config::Config, utils::enums::ProviderName};

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
            vec!["Chat", "Suggest", "Explain", "Options", "Exit"]
        } else {
            vec!["Options", "Exit"]
        };

        let model = self.cfg.active_model().unwrap_or("None".into());
        let prompt = format!("{} {}", style("Active model:").bold(), style(model).cyan());

        let Ok(selection) = Select::new()
            .with_prompt(prompt)
            .items(&items)
            .default(0)
            .interact()
        else {
            return;
        };

        self.term
            .clear_last_lines(1)
            .expect("Failed to clear last line");

        self.select(&items[selection].to_lowercase());
    }

    fn options_menu(&mut self) {
        let items = vec!["Configure providers"];
        let Ok(selection) = Select::new().items(&items).default(0).interact() else {
            return;
        };

        match selection {
            0 => self.provider_menu(),
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
        let choices = if self.cfg.is_configured(provider) {
            if self.cfg.active_provider_name() == Some(provider) {
                vec!["Change API Key", "Change Model", "Remove provider"]
            } else {
                vec![
                    "Change API Key",
                    "Change Model",
                    "Remove provider",
                    "Set as active",
                ]
            }
        } else {
            vec!["Add API Key"]
        };

        let Ok(selection) = Select::new().items(&choices).default(0).interact() else {
            return;
        };

        match selection {
            0 => self.configure_provider(provider),
            1 => self.select_model_menu(provider),
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
                println!("CLAI v1.0.0");
                return;
            }
            "exit" => return,
            "chat" | "suggest" | "explain" => {}
            _ => {
                Program::help();
                return;
            }
        }

        let Some(provider) = self.cfg.active_provider() else {
            let cross = style("✗").red().bold();
            let msg = style("You need to configure a provider first").bold();
            println!("{cross} {msg} {cross}", cross = cross, msg = msg);
            println!("  Run `clai options` to configure a provider");
            return;
        };

        let arg = self.args.get(2).map(|arg| arg.to_owned());

        let ai = AI(&self.term);
        match choice {
            "chat" => ai.chat(&provider, arg),
            "suggest" => ai.suggest(&provider, arg),
            "explain" => ai.explain(&provider, arg),
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
        println!("Usage: clai [OPTION] [ARG]");
        println!();
        println!("Options:");
        println!("  chat    [ARG]  Chat with the AI (optional string argument)");
        println!("  suggest [ARG]  Get suggestions from the AI (optional string argument)");
        println!("  explain [ARG]  Get explanations from the AI (optional string argument)");
        println!("  options        Configure the AI");
        println!("  exit           Exit the program");
    }
}
