use reqwest::Client;
use serde_json::json;
use anyhow::Result;

pub async fn get_openai_response(api_key: &str, system_prompt: &str, user_input: &str) -> Result<String> {
    let client = Client::new();

    let payload = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_input }
        ]
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let reply = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    Ok(reply)
}
