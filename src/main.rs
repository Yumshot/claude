use dotenv::dotenv;
use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    // Read the content of prompt.md
    let prompt_content = fs::read_to_string("src/data/prompt.md")?
        .replace("\n", " ") // Replace newlines with spaces
        .replace("\r", " ") // Replace carriage returns with spaces (for Windows compatibility)
        .trim() // Trim any leading or trailing whitespace
        .to_string(); // Ensure the result is a String

    // Get API key from environment variable
    let api_key = env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set in .env file");

    // Define the JSON data to be sent in the request body
    let data = format!(
        r#"
    {{
        "model": "claude-3-5-sonnet-20240620",
        "max_tokens": 1024,
        "messages": [
            {{"role": "user", "content": "{}"}}
        ]
    }}
    "#,
        prompt_content
    );

    // Construct the curl command
    let output = Command::new("curl")
        .arg("https://api.anthropic.com/v1/messages")
        .arg("--header")
        .arg(format!("x-api-key: {}", api_key))
        .arg("--header")
        .arg("anthropic-version: 2023-06-01")
        .arg("--header")
        .arg("content-type: application/json")
        .arg("--data")
        .arg(data.to_string()) // Convert Cow<'_, str> to String
        .output()?;

    println!("{:?}", output);

    // Check if the command was successful
    if output.status.success() {
        // Get the response
        let response = String::from_utf8_lossy(&output.stdout);

        // Parse the JSON response
        let json_response: Value = serde_json::from_str(&response)?;

        // Extract the content field from the JSON response
        let content = json_response["content"][0]["text"].as_str().unwrap_or("");

        // Check if the content length is less than 1
        let response_text = if content.len() < 1 {
            "Possible Error with Prompt".to_string()
        } else {
            content.to_string()
        };

        // Format the content into a Markdown format without breaking existing structure
        let formatted_response = format!(
            "### Question\n\n{}\n\n### Response\n\n{}",
            prompt_content, response_text
        );

        // Paths to the files
        let response_file_path = "src/data/response.md";
        let backlog_file_path = "src/data/backlog.md";

        // Read existing content from response.md
        let existing_response_content = if fs::metadata(response_file_path).is_ok() {
            fs::read_to_string(response_file_path)?
        } else {
            String::new()
        };

        // Append existing content to backlog.md
        if !existing_response_content.is_empty() {
            let mut backlog_file = fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(backlog_file_path)?;
            writeln!(backlog_file, "\n===\n{}", existing_response_content)?;
        }

        // Write the new formatted response to response.md, replacing any existing content
        fs::write(response_file_path, formatted_response)?;

        println!("Response saved to {}", response_file_path);
    } else {
        // Print the error
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
