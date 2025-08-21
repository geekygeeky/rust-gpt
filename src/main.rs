use dotenv::dotenv;
use hyper::body::Buf;
use hyper::{Body, Client, Request, header};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::env;
use std::io::{Write, stdin, stdout};

#[derive(Debug, Deserialize)]
struct OAIResponse {
    id: Option<String>,
    // object: Option<String>,
    created: Option<u64>,
    model: Option<String>,
    choices: Vec<OAIChoices>,
}

#[derive(Debug, Deserialize)]
struct OAIChoices {
    message: OAIMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OAIRequest<'a> {
    model: &'a str,
    messages: Vec<OAIMessage>,
    temperature: f32,
    max_tokens: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load env
    dotenv().ok();
    let model = "google/gemini-2.0-flash";

    // Create a HttpConnector -> hyper
    let https = HttpsConnector::new();

    // Create a client
    let client = Client::builder().build(https);
    let uri = "https://api.aimlapi.com/v1/chat/completions";
    let preamble = "You are a helpful assistant. Go straight to the point, no intro and outro. Make the response concise and short as much as possible.";
    let openai_key = env::var("API_KEY").unwrap();
    let auth_header = format!("Bearer {}", openai_key);
    println!("{esc}c", esc = 27 as char);

    loop {
        print!(">");
        stdout().flush().unwrap();
        let mut user_input = String::new();

        stdin()
            .read_line(&mut user_input)
            .expect("Failed to read input");
        println!("");

        // Remove any newline characters or extra whitespace
        let user_input = user_input.trim();

        if user_input.to_lowercase() == "exit" {
            break;
        }

        // Spinner -> wait for response
        let mut spinner = Spinner::new(Spinners::Dots12, "\tGPT is thinking...".into());
        let request_body = OAIRequest {
            model,
            max_tokens: 200,
            temperature: 0.7,
            messages: vec![
                OAIMessage {
                    role: String::from("system"),
                    content: preamble.to_string(),
                },
                OAIMessage {
                    role: String::from("user"),
                    content: user_input.to_string(),
                },
            ],
        };
        let parsed_body = Body::from(serde_json::to_vec(&request_body)?);
        let request = Request::post(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, &auth_header)
            .body(parsed_body)
            .unwrap();

        let res = client.request(request).await?;
        let body = hyper::body::aggregate(res).await?;
        let json: OAIResponse = serde_json::from_reader(body.reader())?;

        spinner.stop();

        println!("");
        println!("ID -> {}", json.id.expect("ID must be string"));
        println!("Model -> {}", json.model.expect("Model not present"));
        println!("Created -> {}", json.created.expect("Date not present"));
        println!(
            "---------Content---------\n{}",
            json.choices[0].message.content
        );
    }
    Ok(())
}
