mod subcommand;

use clap::{Arg, ArgMatches, Command};
use subcommand::SubCommand;

#[derive(Clone)]
pub enum Args {
    Chat((&'static str, ChatArgs)),
    Suggest((&'static str, ChatArgs)),
    Explain((&'static str, ChatArgs)),
    Options,
    Changelog,
    None,
}

#[derive(Clone)]
pub struct ChatArgs(ArgMatches);

impl Args {
    pub fn new(version: &'static str) -> Args {
        // let model_arg = Arg::new("model")
        //     .short('m')
        //     .long("model")
        //     .action(ArgAction::SetTrue)
        //     .help("Specify the AI model to use");

        // let search_arg = Arg::new("search")
        //     .short('s')
        //     .long("search")
        //     .action(ArgAction::SetTrue)
        //     .help("Search the web (requires OpenAI or Perplexity API key)");

        let prompt_arg = Arg::new("prompt")
            .help("The prompt to send to the AI")
            .num_args(1..);

        let matches = Command::new("TermAI")
            .version(version)
            .about("Interact with AI models through different commands")
            .subcommand(
                Command::new(SubCommand::Chat)
                    .about(SubCommand::Chat.about())
                    // .arg(model_arg.to_owned())
                    // .arg(search_arg.to_owned())
                    .arg(prompt_arg.to_owned()),
            )
            .subcommand(
                Command::new(SubCommand::Suggest)
                    .about(SubCommand::Suggest.about())
                    // .arg(model_arg.to_owned())
                    .arg(prompt_arg.to_owned()),
            )
            .subcommand(
                Command::new(SubCommand::Explain)
                    .about(SubCommand::Explain.about())
                    // .arg(model_arg.to_owned())
                    .arg(prompt_arg.to_owned()),
            )
            .subcommand(Command::new(SubCommand::Options).about(SubCommand::Options.about()))
            .subcommand(Command::new(SubCommand::Changelog).about(SubCommand::Changelog.about()))
            .get_matches();

        match matches.subcommand_name() {
            Some("chat") => Args::Chat((SubCommand::Chat.as_str(), ChatArgs(matches))),
            Some("suggest") => Args::Suggest((SubCommand::Suggest.as_str(), ChatArgs(matches))),
            Some("explain") => Args::Explain((SubCommand::Explain.as_str(), ChatArgs(matches))),
            Some("options") => Args::Options,
            Some("changelog") => Args::Changelog,
            _ => Args::None,
        }
    }
}

impl ChatArgs {
    pub fn model(&self) -> bool {
        match self.0.subcommand() {
            Some((_, _args)) => false, // args.try_contains_id("model").is_ok() && args.get_flag("model"),
            None => false,
        }
    }

    pub fn search(&self) -> bool {
        match self.0.subcommand() {
            Some((_, _args)) => false, //args.try_contains_id("search").is_ok() && args.get_flag("search"),
            None => false,
        }
    }

    pub fn prompt(&self) -> Option<String> {
        match self.0.subcommand() {
            Some((_, args)) => {
                if args.try_contains_id("prompt").is_err() {
                    None
                } else if let Some(raw) = args.get_raw("prompt") {
                    let prompt = raw
                        .map(|s| s.to_str().unwrap_or_default())
                        .collect::<Vec<&str>>()
                        .join(" ");
                    Some(prompt)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
