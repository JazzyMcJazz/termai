use clap::builder::Str;

pub enum SubCommand {
    Chat,
    Suggest,
    Explain,
    Options,
    Changelog,
}

impl std::fmt::Display for SubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SubCommand::Chat => write!(f, "chat"),
            SubCommand::Suggest => write!(f, "suggest"),
            SubCommand::Explain => write!(f, "explain"),
            SubCommand::Options => write!(f, "options"),
            SubCommand::Changelog => write!(f, "changelog"),
        }
    }
}

impl From<SubCommand> for Str {
    fn from(val: SubCommand) -> Self {
        match val {
            SubCommand::Chat => Str::from("chat"),
            SubCommand::Suggest => Str::from("suggest"),
            SubCommand::Explain => Str::from("explain"),
            SubCommand::Options => Str::from("options"),
            SubCommand::Changelog => Str::from("changelog"),
        }
    }
}

impl SubCommand {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubCommand::Chat => "chat",
            SubCommand::Suggest => "suggest",
            SubCommand::Explain => "explain",
            SubCommand::Options => "options",
            SubCommand::Changelog => "changelog",
        }
    }

    pub fn about(&self) -> &'static str {
        match self {
            SubCommand::Chat => "Start a chat with the AI",
            SubCommand::Suggest => "Get CLI command suggestions from the AI",
            SubCommand::Explain => "Get CLI command explanations from the AI",
            SubCommand::Options => "Open the options menu",
            SubCommand::Changelog => "Print the latest changelog",
        }
    }
}
