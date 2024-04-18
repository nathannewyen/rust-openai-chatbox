use async_openai::types::{AssistantObject, AssistantToolsRetrieval, CreateAssistantRequest, CreateThreadRequest, ModifyAssistantRequest, ThreadObject};
use derive_more::{Deref, Display, From};
use crate::ais::OaClient;
use crate::Result;

const DEFAULT_QUERY: &[(&str, &str)] = &[("limit", "100")];

pub struct CreateConfig {
    pub(crate) name: String,
    pub(crate) model: String,
}

#[derive(Debug, From, Deref, Display)]
pub struct AssistantId(String);
#[derive(Debug, From, Deref, Display)]
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

#[allow(dead_code)]
#[allow(unused_variables)]
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
        println!("Assistant {} deleted", config.name);
    }

    // Create if needed
    if let Some(assistant_id) = assistant_id {
        println!("Assistant {} loaded", config.name);
        Ok(assistant_id)
    } else {
        let assistant_name = config.name.clone();
        let assistant_id = create(open_ai_client, config).await?;
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

pub async fn create_thread(open_ai_client: &OaClient) -> Result<ThreadId> {
    let open_ai_threads = open_ai_client.threads();

    let res = open_ai_threads
        .create(CreateThreadRequest {
            ..Default::default()
        })
        .await?;

    Ok(res.id.into())
}

pub async fn get_thread(
    open_ai_client: &OaClient,
    thread_id: &ThreadId,
) -> Result<ThreadObject> {
    let open_ai_threads = open_ai_client.threads();

    let thread_obj = open_ai_threads.retrieve(&thread_id.0).await?;

    Ok(thread_obj)
}