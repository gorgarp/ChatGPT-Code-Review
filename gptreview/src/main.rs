use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
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

    let mut file = File::create("review.txt")?;

    for entry in WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match process_file(&client, &url, &api_key, &path, &mut file).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error processing file {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

async fn process_file(
    client: &Client,
    url: &str,
    api_key: &str,
    path: &std::path::Path,
    file: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    let code = fs::read_to_string(path)?;
    let prompt = format!("Review the following code:\n\n```\n{}\n```", code);
    let response = send_prompt(&client, &url, &api_key, &prompt).await?;
    let response_text = format!("File: {}\nReview:\n{}\n\n", path.display(), response.choices[0].message.content);
    println!("{}", response_text);
    file.write_all(response_text.as_bytes())?;
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
