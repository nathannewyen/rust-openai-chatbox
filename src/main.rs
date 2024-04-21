mod ais;
mod buddy;
mod error;
mod utils;

use textwrap::wrap;
use crate::buddy::Buddy;
use crate::utils::cli::{ico_err, ico_res, prompt, txt_res};
pub use self::error::{Error, Result};

#[tokio::main]
async fn main() {
    println!();
    match  start().await {
        Ok(_) => print!("\nSuccess!\n"),
        Err(e) => eprintln!("\nError: {}\n", e),
    }
}

const DEFAULT_DIR: &str = "buddy";

// Types

// Input command from the user
#[derive(Debug)]
enum Cmd {
    Quit,
    Chat(String),
    RefreshAll,
    RefreshConv,
    RefreshInst,
    RefreshFiles,
}

impl Cmd {
    fn from_input(input: impl Into<String>) -> Self {
        let input = input.into();

        if input == "/q" {
            Self::Quit
        } else if input == "/r" || input == "/ra" {
            Self::RefreshAll
        } else if input == "/ri" {
            Self::RefreshInst
        } else if input == "/rf" {
            Self::RefreshFiles
        } else if input == "/rc" {
            Self::RefreshConv
        } else {
            Self::Chat(input)
        }
    }
}

#[allow(unused)]
async fn start() -> Result<()> {
    let mut buddy = Buddy::init_from_dir(DEFAULT_DIR, false).await?;

    let mut conv = buddy.load_or_create_conv(false).await?;

    loop {
        println!();
        let input = prompt("Ask away")?;
        let cmd = Cmd::from_input(input);

        match cmd {
            Cmd::Quit => break,
            Cmd::Chat(msg) => {
                let res = buddy.chat(&conv, &msg).await?; // Borrow `msg` here
                let res = wrap(&res, 80).join("\n");
                println!(" -> {} {}", ico_res(), txt_res(res));
            }
            other => println!("{} command not supported {:?}", ico_err(), other),
        }
    }

    println!(" -> buddy {} - conv {:?}", buddy.name(), conv);

    Ok(())
}