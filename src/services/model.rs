use anyhow::{Context, Result};
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionMessage, ChatCompletionRequest};
use std::env;

use crate::utils::{error::AppError, logger::log_error, structs::Prompt};

pub struct Question {
    pub system_prompt: Option<String>,
    pub messages: Option<Vec<Prompt>>,
    pub user_input: String,
}

/// This function is used to query using the openai API
async fn get_openai_response(question: Question, model: &str) -> Result<String> {
    // Retrieve the OpenAI API key from the environment securely
    let api_key =
        env::var("OPENAI_API_KEY").context("Missing OPENAI_API_KEY environment variable")?;

    // Initialize the OpenAI client with the API key
    let client = OpenAIClient::builder()
        .with_api_key(api_key)
        .build()
        .unwrap();

    let mut msgs = vec![];
    if question.system_prompt.is_some() {
        msgs.push(ChatCompletionMessage {
            role: chat_completion::MessageRole::system,
            content: chat_completion::Content::Text(question.system_prompt.unwrap()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        });
    }

    if question.messages.is_some() {
        for msg in question.messages.unwrap().iter() {
            msgs.push(ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(msg.content.to_owned()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            });

            msgs.push(ChatCompletionMessage {
                role: chat_completion::MessageRole::assistant,
                content: chat_completion::Content::Text(msg.output.to_owned()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }

    let usr_input = if question.user_input.is_empty() {
        String::from(".")
    } else {
        question.user_input.to_owned()
    };

    msgs.push(ChatCompletionMessage {
        role: chat_completion::MessageRole::user,
        content: chat_completion::Content::Text(usr_input),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    });

    // Construct the chat completion request with the system and user messages
    let req = ChatCompletionRequest::new(
        model.to_string(), // Replace this with your desired model
        msgs,
    );

    let result = client.chat_completion(req).await.map_err(|e| {
        log_error(&format!(
            "Failed to receive answer from {}. With error: {}",
            model, e
        ));
        AppError::ModelError {
            model_name: model.to_owned(),
            failure_str: e.to_string(),
        }
    })?;
    let answer = result.choices[0].message.content.clone().unwrap();

    Ok(answer)
}

pub async fn ask_question(llm: &str, model: &str, question: Question) -> Result<String> {
    match llm {
        "openai" => {
            let ans = get_openai_response(question, model).await?;
            Ok(ans)
        }
        _ => Ok(String::from("Not finished")),
    }
}
