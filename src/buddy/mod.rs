mod config;

use std::path::{Path, PathBuf};
use derive_more::{Deref, From};
use serde::{Deserialize, Serialize};
use crate::ais::assistant::{AssistantId, ThreadId};
use crate::ais::{assistant, new_op_client, OaClient};
use crate::buddy::config::Config;
use crate::Result;
use crate::utils::files::{ensure_dir, load_from_toml};

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
        reccreate_assistant: bool,
    ) -> Result<Self> {
        let dir = dir.as_ref();

        // Load from the directory
        let config: Config = load_from_toml(dir.join(BUDDY_TOML))?;

        // Get or Create the OpenAI Assistant
        let open_ai_client = new_op_client();
        let assistant_id =
            assistant::load_or_create_assistant(&open_ai_client, (&config).into(), reccreate_assistant).await?;

        // Create buddy
        let buddy = Buddy {
            dir: dir.to_path_buf(),
            open_ai_client,
            assistant_id,
            config,
        };

        buddy.upload_intsructions().await?;

        Ok(buddy)
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