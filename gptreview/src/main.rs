use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::json;
use std::fs;
use std::io::{self, Write}; // added Write trait
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct GptResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Please enter your API key:");
    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    api_key = api_key.trim().to_string();

    println!("Please enter the path to your code folder:");
    let mut folder_path = String::new();
    io::stdin().read_line(&mut folder_path)?;
    folder_path = folder_path.trim().to_string();

    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let mut file = fs::File::create("review.txt")?; // open the file for writing

    for entry in WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let code = fs::read_to_string(path)?;
            let prompt = format!("Review the following code:\n\n```\n{}\n```", code);
            let response = send_prompt(&client, &url, &api_key, &prompt).await?;

            // write the response to the file instead of printing it
            writeln!(file, "File: {}\nReview:\n{}\n", path.display(), response.choices[0].message.content)?;
        }
    }

    Ok(())
}

async fn send_prompt(
    client: &Client,
    url: &str,
    api_key: &str,
    prompt: &str,
) -> Result<GptResponse, Box<dyn std::error::Error>> {
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&json!({ "model": "gpt-3.5-turbo", "messages": [{"role": "system", "content": "You are an AI that reviews code."}, {"role": "user", "content": prompt}], "max_tokens": 150, "n": 1, "stop": null }))
        .send()
        .await?;

    let gpt_response: GptResponse = response.json().await?;
    Ok(gpt_response)
}
