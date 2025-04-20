use std::time::Duration;

use indicatif::ProgressBar;
use rig::{
    agent::Agent,
    completion::{Completion, CompletionModel, PromptError},
    message::{AssistantContent, Message, ToolCall, ToolFunction, ToolResultContent, UserContent},
    OneOrMany,
};

use anyhow::Result;

use crate::utils::console::get_select_theme;

pub struct MultiTurnAgent<M: CompletionModel> {
    agent: Agent<M>,
    chat_history: Vec<Message>,
}

impl<M: CompletionModel> MultiTurnAgent<M> {
    pub fn new(agent: Agent<M>, chat_history: Vec<Message>) -> Self {
        Self {
            agent,
            chat_history,
        }
    }

    pub async fn multi_turn_prompt(
        &mut self,
        prompt: impl Into<Message> + Send,
        spinner: Option<&ProgressBar>,
    ) -> Result<String, PromptError> {
        let mut current_prompt: Message = prompt.into();
        loop {
            let res = self
                .agent
                .completion(current_prompt.to_owned(), self.chat_history.to_owned())
                .await?
                .send()
                .await?;

            let mut final_text = None;

            for content in res.choice.into_iter() {
                self.chat_history.push(current_prompt.to_owned());

                match content {
                    AssistantContent::Text(text) => {
                        final_text = Some(text.text.to_owned());

                        let response_message = Message::Assistant {
                            content: OneOrMany::one(AssistantContent::text(&text.text)),
                        };
                        self.chat_history.push(response_message);
                    }
                    AssistantContent::ToolCall(tool_call) => {
                        let tool_call_msg = AssistantContent::ToolCall(tool_call.to_owned());

                        self.chat_history.push(Message::Assistant {
                            content: OneOrMany::one(tool_call_msg),
                        });

                        let ToolCall {
                            id,
                            function: ToolFunction { name, arguments },
                        } = tool_call;

                        let (server_name, tool_name) =
                            name.split_once('-').unwrap_or((&name, &name));

                        let confirmation = confirm_tool_call(server_name, tool_name, spinner);

                        let tool_result = if confirmation {
                            match self
                                .agent
                                .tools
                                .call(tool_name, arguments.to_string())
                                .await
                            {
                                Ok(result) => result,
                                Err(_) => "Error calling tool".to_string(),
                            }
                        } else {
                            format!("User refused to run the {name} tool")
                        };

                        current_prompt = Message::User {
                            content: OneOrMany::one(UserContent::tool_result(
                                id,
                                OneOrMany::one(ToolResultContent::text(tool_result)),
                            )),
                        };

                        final_text = None;
                        break;
                    }
                }
            }

            if let Some(text) = final_text {
                return Ok(text);
            }
        }
    }
}

fn confirm_tool_call(server_name: &str, tool_name: &str, spinner: Option<&ProgressBar>) -> bool {
    if let Some(spinner) = spinner {
        spinner.disable_steady_tick();
        let _ = dialoguer::console::Term::stdout().clear_last_lines(2);
        println!();
    }

    let confirmation = dialoguer::Select::with_theme(&get_select_theme())
        .with_prompt(format!("Run tool '{tool_name}' from {server_name}"))
        .default(0)
        .items(&["Yes", "No"])
        .clear(true)
        .interact()
        .unwrap_or(0)
        == 0;

    if let Some(spinner) = spinner {
        spinner.enable_steady_tick(Duration::from_millis(100));
    }

    confirmation
}
