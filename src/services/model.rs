use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, ChatCompletionMessage};
use openai_api_rs::v1::common::GPT4_O_LATEST; // Select model as per your use case
use std::env;
use anyhow::{Result, Context};

use crate::utils::structs::Prompt;

pub async fn get_openai_response(
    system_prompt: &str,
    messages: Option<Vec<Prompt>>,
    user_input: &str
) -> Result<String> {
    // Retrieve the OpenAI API key from the environment securely
    let api_key = env::var("OPENAI_API_KEY")
        .context("Missing OPENAI_API_KEY environment variable")?;

    // Initialize the OpenAI client with the API key
    let client = OpenAIClient::builder().with_api_key(api_key).build().unwrap();

    let mut msgs = vec![];
    if system_prompt.is_empty() {
        msgs.push(
            ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: chat_completion::Content::Text(system_prompt.to_owned()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        );
    }

    if messages.is_some() {
        for msg in messages.unwrap().iter() {
            msgs.push(
                ChatCompletionMessage {
                    role: chat_completion::MessageRole::user,
                    content: chat_completion::Content::Text(msg.content.to_owned()),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }
            );

            msgs.push(
                ChatCompletionMessage {
                    role: chat_completion::MessageRole::assistant,
                    content: chat_completion::Content::Text(msg.output.to_owned()),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                }
            );
        }
    }

    msgs.push(
        ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(user_input.to_owned()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        },
    );

    // Construct the chat completion request with the system and user messages
    let req = ChatCompletionRequest::new(
        GPT4_O_LATEST.to_string(), // Replace this with your desired model
        msgs,
    );

    let result = client.chat_completion(req).await.unwrap();
    let answer = result.choices[0].message.content.clone().unwrap();

    Ok(answer)
}
