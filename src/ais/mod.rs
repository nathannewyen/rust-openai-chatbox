pub mod assistant;
pub mod msg;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use crate::Result;

use dotenv::dotenv;
use dotenv::var;

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

pub type OaClient = Client<OpenAIConfig>;

pub fn new_op_client() -> Result<OaClient> {
    dotenv().ok();
    if var(ENV_OPENAI_API_KEY).is_ok() {
        Ok(Client::new())
    } else {
        print!("No {} in env file", ENV_OPENAI_API_KEY);
        Err("No OpenAI api key in env file".into())
    }
}