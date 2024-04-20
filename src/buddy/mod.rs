mod config;

use std::fs;
use std::path::{Path, PathBuf};
use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};
use crate::ais::assistant::{AssistantId, ThreadId};
use crate::ais::{assistant, new_op_client, OaClient};
use crate::buddy::config::Config;
use crate::Result;
use crate::utils::cli::ico_check;
use crate::utils::files::{ensure_dir, load_from_json, load_from_toml, read_to_string, save_to_json};

const BUDDY_TOML: &str = "buddy.toml";

#[derive(Debug)]
pub struct Buddy {
    dir: PathBuf,
    open_ai_client: OaClient,
    assistant_id: AssistantId,
    config: Config,
}

#[derive(Debug, From, Deref, Deserialize, Serialize)]
pub struct Conv {
    thread_id: ThreadId,
}

// Public Buddy functions
impl Buddy {
    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub async fn init_from_dir(
        dir: impl AsRef<Path>,
        recreate_asst: bool,
    ) -> Result<Self> {
        let dir = dir.as_ref();

        // -- Load from the directory
        let config: Config = load_from_toml(dir.join(BUDDY_TOML))?;

        // -- Get or Create the OpenAI Assistant
        let open_ai_client = new_op_client()?;
        let assistant_id =
            assistant::load_or_create_assistant(&open_ai_client, (&config).into(), recreate_asst).await?;

        // -- Create buddy
        let buddy = Buddy {
            dir: dir.to_path_buf(),
            open_ai_client,
            assistant_id,
            config,
        };

        // -- Upload instructions
        buddy.upload_instructions().await?;

        // -- Upload files
        buddy.upload_files(false).await?;

        Ok(buddy)
    }

    pub async fn upload_instructions(&self) -> Result<bool> {
        let file = self.dir.join(&self.config.instructions_file);
        if file.exists() {
            let inst_content = read_to_string(&file)?;
            assistant::upload_instructions(&self.open_ai_client, &self.assistant_id, inst_content)
                .await?;
            println!("{} Instructions uploaded", ico_check());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn load_or_create_conv(&self, recreate: bool) -> Result<Conv> {
        let conv_file = self.data_sir()?.join("conv.json");

        if recreate && conv_file.exist() {
            fs::remove_file(&conv_file)?;
        }

        let conv = if let Ok(conv) = load_from_json::<Conv>(&conv_file) {
            assistant::get_thread(&self.open_ai_client, &conv.thread_id)
                .await
                .map_err(|_| format!("Cannot find thread_id for {:?} ", conv))?;
            print!("{} Conversation loaded", ico_check());
            conv
        } else {
            let thread_id = assistant::create_thread(&self.open_ai_client).await?;
            println!("{} Conversation created", ico_check());
            let conv = thread_id.into();
            save_to_json(&conv_file, &conv)?;
            conv
        };

        Ok(conv)
    }
}

// Private Buddy functions
#[ allow(unused)]
impl Buddy {
    fn data_sir(&self) -> Result<PathBuf> {
        let data_dir = self.dir.join(".buddy");
        ensure_dir(&data_dir)?;
        Ok(data_dir)
    }

    fn data_files_sir(&self) -> Result<PathBuf> {
        let dir = self.data_sir()?.join("files");
        ensure_dir(&dir)?;
        Ok(dir)
    }
}