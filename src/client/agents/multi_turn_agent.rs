use indicatif::ProgressBar;
use rig::{
    agent::Agent,
    completion::{Completion, CompletionModel, PromptError},
    message::{AssistantContent, Message, ToolCall, ToolFunction, ToolResultContent, UserContent},
    OneOrMany,
};

use anyhow::Result;

use super::util::confirm_tool_call;

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

            self.chat_history.push(current_prompt.to_owned());

            let mut final_text = None;

            for content in res.choice.into_iter() {
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

                        let confirmation = confirm_tool_call(&name, spinner);

                        let tool_result = if confirmation {
                            match self.agent.tools.call(&name, arguments.to_string()).await {
                                Ok(result) => result,
                                Err(_) => "Error calling tool".to_string(),
                            }
                        } else {
                            "Cancelled by user".to_string()
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
