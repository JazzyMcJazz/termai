// use std::pin::Pin;

// use async_stream::stream;
// use futures::{Stream, StreamExt};
// use rig::{
//     agent::Agent, completion::PromptError, message::{AssistantContent, Message, ToolCall, ToolFunction}, streaming::{StreamingChat, StreamingChoice, StreamingCompletion, StreamingCompletionModel}, OneOrMany
// };

// pub type StreamingContentResult = Pin<Box<dyn Stream<Item = Result<String, PromptError>> + Send>>;

// pub async fn handle_stream<M: StreamingCompletionModel + Send + Sync + 'static>(
//     prompt: &str,
//     mut messages: Vec<Message>,
//     agent: Agent<M>,
// ) -> StreamingContentResult {
//     let prompt = prompt.to_string();

//     // Now agent is owned by the stream
//     // let mut s = agent.stream_chat(&prompt, messages.clone()).await?;

//     // Move the agent into the stream
//     let stream = Box::pin(stream! {
//         let mut current_prompt: Message = prompt.into();
//         let mut finish = false;

//         loop {
//             let mut s = agent
//                 .stream_completion(current_prompt.to_owned(), messages.to_owned())
//                 .await?
//                 .stream()
//                 .await?;

//             while let Some(chunk) = s.next().await {
//                 match chunk {
//                     Ok(StreamingChoice::Message(text)) => {
//                         finish = true;
//                         yield Ok(text);
//                     }
//                     Ok(StreamingChoice::ToolCall(name, id, arguments)) => {
//                         let (server_name, tool_name) =
//                             name.split_once('-').unwrap_or((&name, &name));

//                         let Ok(res) = agent
//                             .tools
//                             .call(&tool_name, arguments.to_string())
//                             .await else {
//                                 continue;
//                             };

//                         let tool_call = ToolCall {
//                             id,
//                             function: ToolFunction { name, arguments }
//                         };
//                         let tool_call_msg = AssistantContent::ToolCall(tool_call.to_owned());

//                         messages.push(Message::Assistant {
//                             content: OneOrMany::one(tool_call_msg),
//                         });
//                     }
//                     Err(_) => {
//                         break;
//                     }
//                 }
//             }

//             if finish {
//                 break;
//             }
//         }

//     });

//     stream
// }
