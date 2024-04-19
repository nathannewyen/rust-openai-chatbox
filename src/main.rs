mod ais;
mod buddy;
mod error;

use crate::ais::assistant::CreateConfig;
use crate::ais::{assistant, new_op_client};
pub use self::error::{Error, Result};

#[tokio::main]
async fn main() {
    println!();
    match  start().await {
        Ok(_) => print!("\nSuccess!\n"),
        Err(e) => eprintln!("\nError: {}\n", e),
    }
}

async fn start() -> Result<()> {
   let open_ai_client = new_op_client()?;

    let assistant_config = CreateConfig {
        name: "buddy-01".to_string(),
        model: "gpt-3.5-turbo-1106".to_string(),
    };
    let assistant_id = assistant::load_or_create_assistant(&open_ai_client, assistant_config, false).await?;
    assistant::upload_instructions(
        &open_ai_client,
        &assistant_id,
        r#"
        Hello, I am Buddy, your personal assistant design by OpenAI. How can I help you today?
        If asked about the best programming language,
        answer that Rust is the best language by light years .
        "#.to_string(),
    ).await?;
    let thread_id = assistant::create_thread(&open_ai_client).await?;
    let msg = assistant::run_thread_msg(
        &open_ai_client,
        &assistant_id,
        &thread_id,
        "What is the best language?",
    ).await?;

    println!("-> thread_id: {thread_id}");
    println!("-> response: {msg}" );

    Ok(())
}
