use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::json;
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::Path;
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
    let api_key = read_input("Please enter your API key:")?;
    let folder_path = read_input("Please enter the path to your code folder:")?;

    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let mut file = File::create("review.txt")?;

    for entry in WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match process_file(&client, &url, &api_key, path, &mut file).await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Error processing file {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

fn read_input(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().lock().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

async fn process_file(
    client: &Client,
    url: &str,
    api_key: &str,
    path: &Path,
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
        .json(&json!({ "model": "gpt-3.5-turbo", "messages": [{"role": "system", "content": "You are an AI that reviews code. Don't explain what it does. Just tell me any errors or improvements."}, {"role": "user", "content": prompt}], "max_tokens": 2000, "n": 1, "stop": null }))
        .send()
        .await?;

    let gpt_response: GptResponse = response.json().await?;
    Ok(gpt_response)
}
