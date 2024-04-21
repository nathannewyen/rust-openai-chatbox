use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;
use async_openai::types::{AssistantObject, AssistantToolsRetrieval, CreateAssistantFileRequest, CreateAssistantRequest, CreateFileRequest, CreateRunRequest, CreateThreadRequest, ModifyAssistantRequest, RunStatus, ThreadObject};
use console::Term;
use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use crate::ais::msg::{get_text_content, user_msg};
use crate::ais::OaClient;
use crate::Result;
use crate::utils::cli::{ico_check, ico_deleted_ok, ico_err, ico_uploaded, ico_uploading};
use crate::utils::files::XFile;

#[ allow(unused)]
const DEFAULT_QUERY: &[(&str, &str)] = &[("limit", "100")];
const POLLING_DURATION_MS: u64 = 500;
pub struct CreateConfig {
    pub(crate) name: String,
    pub(crate) model: String,
}

#[derive(Debug, From, Deref, Display)]
pub struct AssistantId(String);
#[derive(Debug, From, Deref, Display, Serialize, Deserialize)]
pub struct ThreadId(String);
#[derive(Debug, From, Deref, Display)]
pub struct FileId(String);
pub async fn create(open_ai_client: &OaClient, config: CreateConfig) -> Result<AssistantId> {
    let open_ai_assistant = open_ai_client.assistants();

    let asst_obj = open_ai_assistant
        .create(CreateAssistantRequest {
            model: config.model,
            name: Some(config.name),
            tools: Some(vec![AssistantToolsRetrieval::default().into()]),
            ..Default::default()
        }).await?;

    Ok(asst_obj.id.into())
}

#[ allow(unused)]
pub async fn load_or_create_assistant(
    open_ai_client: &OaClient,
    config: CreateConfig,
    recreate: bool,
) -> Result<AssistantId> {
    let assistant_obj = first_by_name(open_ai_client, &config.name).await?;
    let mut assistant_id = assistant_obj.map(|o| AssistantId(o.id));

    // Delete the assistant if it exists and recreate is true
    if let (true, Some(assistant_id_ref)) = (recreate, assistant_id.as_ref()) {
        delete(open_ai_client, assistant_id_ref).await?;
        assistant_id.take();
        println!("{} Assistant {} deleted", ico_deleted_ok(), config.name);
    }

    // Create if needed
    if let Some(assistant_id) = assistant_id {
        println!("{} Assistant {} loaded", ico_check(), config.name);
        Ok(assistant_id)
    } else {
        let assistant_name = config.name.clone();
        let assistant_id = create(open_ai_client, config).await?;
        println!("{} Assistant {} created", ico_check(), assistant_name);
        Ok(assistant_id)
    }
}

pub async fn first_by_name(open_ai_client: &OaClient, name: &str) -> Result<Option<AssistantObject>> {
    let open_ai_assistants = open_ai_client.assistants();

    let assistants = open_ai_assistants.list(DEFAULT_QUERY).await?.data;

    let assistant_obj = assistants
        .into_iter()
        .find(|a| a.name.as_ref().map(|n| n == name).unwrap_or(false));

    Ok(assistant_obj)
}

#[ allow(unused)]
pub async fn upload_instructions(
    open_ai_client: &OaClient,
    assistant_id: &AssistantId,
    inst_content: String,
) -> Result<()> {
    let open_ai_assistants = open_ai_client.assistants();
    let modif = ModifyAssistantRequest {
        instructions: Some(inst_content),
        ..Default::default()
    };
    open_ai_assistants.update(&assistant_id.0.as_str(), modif).await?;

    Ok(())
}

pub async fn delete(open_ai_client: &OaClient, assistant_id: &AssistantId) -> Result<()> {
    let open_ai_assistant = open_ai_client.assistants();

    // TODO: Delete files

    // Delete assistant
    open_ai_assistant.delete(&assistant_id.0.as_str()).await?;
    Ok(())
}

#[ allow(unused)]
pub async fn create_thread(open_ai_client: &OaClient) -> Result<ThreadId> {
    let open_ai_threads = open_ai_client.threads();

    let res = open_ai_threads
        .create(CreateThreadRequest {
            ..Default::default()
        })
        .await?;

    Ok(res.id.into())
}

#[ allow(unused)]
pub async fn get_thread(
    open_ai_client: &OaClient,
    thread_id: &ThreadId,
) -> Result<ThreadObject> {
    let open_ai_threads = open_ai_client.threads();

    let thread_obj = open_ai_threads.retrieve(&thread_id.0).await?;

    Ok(thread_obj)
}

#[ allow(unused)]
pub async fn run_thread_msg(
    open_ai_client: &OaClient,
    assistant_id: &AssistantId,
    thread_id: &ThreadId,
    msg: &str,
) -> Result<String> {
    let msg = user_msg(msg);

    // Attach message to thread
    let _message_obj = open_ai_client.threads().messages(&thread_id.0).create(msg).await?;

    // Create a run for the thread
    let run_request = CreateRunRequest {
        assistant_id: assistant_id.0.to_string(),
        ..Default::default()
    };
    let run = open_ai_client.threads().runs(&thread_id.0).create(run_request).await?;

    // Loop to get result
    let term = Term::stdout();
    loop {
        term.write_str(">")?;
        let run = open_ai_client.threads().runs(&thread_id.0).retrieve(&run.id).await?;
        term.write_str("< ")?;

        // Log the status of the run
        println!("Run status: {:?}", run.status);

        match run.status {
            RunStatus::Completed => {
                term.write_str("\n")?;
                return get_first_thread_msg_content(open_ai_client, thread_id).await;
            }
            RunStatus::InProgress => {
                // If the status is InProgress, continue polling
                sleep(Duration::from_millis(POLLING_DURATION_MS)).await;
                continue;
            }
            other => {
                term.write_str("\n")?;
                return Err(format!("ERROR WHILE RUN: {:?}", other).into());
            }
        };
    }
}

pub async fn get_first_thread_msg_content(
    open_ai_client: &OaClient,
    thread_id: &ThreadId,
) -> Result<String> {
    static QUERY: &[(&str, &str)] = &[("limit", "1")];

    let messages = open_ai_client.threads().messages(&thread_id.0).list(QUERY).await?;
    let msg = messages
        .data
        .into_iter()
        .next()
        .ok_or_else(|| "No message in thread".to_string())?;

    let text = get_text_content(msg)?;
    Ok(text)
}

pub async fn get_files_hashmap(
    open_ai_client: &OaClient,
    assistant_id: &AssistantId,
) -> Result<HashMap<String, FileId>> {
    // Get all asst files (files do not have .name)
    let oas_assts = open_ai_client.assistants();
    let oa_asst_files = oas_assts.files(&assistant_id.0); // Dereference `assistant_id` here
    let asst_files = oa_asst_files.list(DEFAULT_QUERY).await?.data;
    let asst_files_ids: HashSet<String> =
        asst_files.into_iter().map(|f| f.id).collect();

    // Get all files for org (those files have .filename)
    let oa_files = open_ai_client.files();
    let org_files = oa_files.list(DEFAULT_QUERY).await?.data; // Check the expected argument for `list` method

    // Build or file_name::file_id hashmap
    let file_id_by_name:HashMap<String, FileId> = org_files
        .into_iter()
        .filter(|org_file| asst_files_ids.contains(&org_file.id))
        .map(|org_file| (org_file.filename, org_file.id.into()))
        .collect();

    Ok(file_id_by_name)
}

/// Uploads a file to an assistant (first to the account, then attaches to asst)
/// - `force` is `false`, will not upload the file if already uploaded.
/// - `force` is `true`, it will delete existing file (account and asst), and upload.
///
/// Returns `(FileId, has_been_uploaded)`
pub async fn upload_file_by_name(
    oac: &OaClient,
    asst_id: &AssistantId,
    file: &Path,
    force: bool,
) -> Result<(FileId, bool)> {
    let file_name = file.x_file_name();
    let mut file_id_by_name = get_files_hashmap(oac, asst_id).await?;

    let file_id = file_id_by_name.remove(file_name);

    // -- If not force and file already created, return early.
    if !force {
        if let Some(file_id) = file_id {
            return Ok((file_id, false));
        }
    }

    // -- If we have old file_id, we delete the file.
    if let Some(file_id) = file_id {
        // -- Delete the org file
        let oa_files = oac.files();
        if let Err(err) = oa_files.delete(&file_id).await {
            println!(
                "{} Can't delete file '{}'\n    cause: {:?}",
                ico_err(),
                file.to_string_lossy(),
                err
            );
        }

        // -- Delete the asst_file association
        let oa_assts = oac.assistants();
        let oa_assts_files = oa_assts.files(^&asst_id.0);
        if let Err(err) = oa_assts_files.delete(&file_id.0).await {
            println!(
                "{} Can't remove assistant file '{}'\n    cause: {}",
                ico_err(),
                file.x_file_name(),
                err
            );
        }
    }

    // -- Upload and attach the file.
    let term = Term::stdout();

    // Print uploading.
    term.write_line(&format!(
        "{} Uploading file '{}'",
        ico_uploading(),
        file.x_file_name()
    ))?;

    // Upload file.
    let oa_files = oac.files();
    let oa_file = oa_files
        .create(CreateFileRequest {
            file: file.into(),
            purpose: "assistants".into(),
        })
        .await?;

    // Update print.
    term.clear_last_lines(1)?;
    term.write_line(&format!(
        "{} Uploaded file '{}'",
        ico_uploaded(),
        file.x_file_name()
    ))?;

    // Attach file to assistant.
    let oa_assts = oac.assistants();
    let oa_assts_files = oa_assts.files(&asst_id.0);
    let asst_file_obj = oa_assts_files
        .create(CreateAssistantFileRequest {
            file_id: oa_file.id.clone(),
        })
        .await?;

    // -- Assert warning.
    if oa_file.id != asst_file_obj.id {
        println!(
            "SHOULD NOT HAPPEN. File id not matching {} {}",
            oa_file.id, asst_file_obj.id
        )
    }

    Ok((asst_file_obj.id.into(), true))
}
