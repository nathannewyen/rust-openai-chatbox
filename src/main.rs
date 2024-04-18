mod ais;
mod buddy;
mod error;

use crate::ais::assistant::CreateConfig;
use crate::ais::new_op_client;
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
    let assistant_id = ais::assistant::create(&open_ai_client, assistant_config).await?;
    println!("-> assistant_id: {assistant_id:?}" );

    Ok(())
}
