mod ais;
mod buddy;
mod error;

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
   print!("Hello World!");
    Ok(())
}
