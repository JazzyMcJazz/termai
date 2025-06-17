use async_stream::stream;
use futures::{Stream, StreamExt};
use rig::{
    agent::Agent,
    completion::{CompletionModel, PromptError},
    message::{AssistantContent, Message, Text, ToolResultContent, UserContent},
    streaming::StreamingCompletion,
    OneOrMany,
};
use std::pin::Pin;

use anyhow::Result;

use super::util::confirm_tool_call;

pub type StreamingContentResult =
    Pin<Box<dyn Stream<Item = Result<StreamingContent, PromptError>> + Send>>;

#[derive(Debug)]
pub enum StreamingContent {
    Text(String),
    PauseSpinner,
    StartSpinner,
}

pub struct StreamingMultiTurnAgent;

impl StreamingMultiTurnAgent {
    pub async fn multi_turn_prompt<M>(
        prompt: impl Into<Message> + Send,
        agent: Agent<M>,
        mut chat_history: Vec<Message>,
    ) -> StreamingContentResult
    where
        M: CompletionModel + 'static,
        <M as CompletionModel>::StreamingResponse: std::marker::Send,
    {
        let prompt: Message = prompt.into();

        (Box::pin(stream! {
            let mut current_prompt = prompt;
            let mut did_write_message = false;
            let mut did_call_tool = false;
            let mut finish = false;

            'outer: loop {
                let mut stream = agent
                    .stream_completion(current_prompt.to_owned(), chat_history.to_owned())
                    .await?
                    .stream()
                    .await?;

                chat_history.push(current_prompt.to_owned());

                let mut tool_calls = vec![];
                let mut tool_results = vec![];

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(AssistantContent::Text(Text { text })) => {
                            if did_call_tool && did_write_message {
                                yield Ok(StreamingContent::Text("\n".to_string()));
                            }

                            yield Ok(StreamingContent::Text(text));
                            did_write_message = true;
                            did_call_tool = false;
                            finish = true;
                        }
                        Ok(AssistantContent::ToolCall(tool_call)) => {
                            if did_write_message {
                                yield Ok(StreamingContent::Text("\n".to_string()));
                            }

                            did_call_tool = true;
                            finish = false;

                            yield Ok(StreamingContent::PauseSpinner);
                            let confirmation = confirm_tool_call(&tool_call.function.name, None);
                            yield Ok(StreamingContent::StartSpinner);

                            let tool_result = if confirmation {
                                match agent.tools.call(&tool_call.function.name, tool_call.function.arguments.to_string()).await {
                                    Ok(res) => res,
                                    Err(e) => e.to_string(),
                                }
                            } else {
                                "Cancelled by user".to_string()
                            };

                            let tool_call_msg = AssistantContent::ToolCall(tool_call.to_owned());
                            tool_calls.push(tool_call_msg);
                            tool_results.push((tool_call.id, tool_result));
                        }
                        Err(_) => {
                            break 'outer;
                        }
                    }
                }

                // Add tool calls to chat history
                if !tool_calls.is_empty() {
                    chat_history.push(Message::Assistant {
                        content: OneOrMany::many(tool_calls).expect("Impossible EmptyListError"),
                    });
                }

                // Add tool results to chat history
                for (id, tool_result) in tool_results {
                    chat_history.push(Message::User {
                        content: OneOrMany::one(UserContent::tool_result(
                            id,
                            OneOrMany::one(ToolResultContent::text(tool_result)),
                        )),
                    });
                }

                current_prompt = chat_history.pop().unwrap();

                if finish {
                    break 'outer;
                }
            }
        })) as _
    }
}
