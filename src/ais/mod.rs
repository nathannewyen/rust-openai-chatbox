pub mod assistant;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use crate::Result;

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

pub type OaClient = Client<OpenAIConfig>;

pub fn new_op_client() -> Result<OaClient> {
    if std::env::var(ENV_OPENAI_API_KEY).is_ok() {
        Ok(Client::new())
    } else {
        print!("No {ENV_OPENAI_API_KEY} in env file");
        Err("No OpenAI api key in env file".into())
    }
}