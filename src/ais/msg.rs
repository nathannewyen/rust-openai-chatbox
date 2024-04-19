use async_openai::types::{CreateMessageRequest, MessageContent, MessageObject};
use crate::Result;

#[ allow(unused)]
pub fn user_msg(content: impl Into<String>) -> CreateMessageRequest {
    CreateMessageRequest {
        role: "user".to_string(),
        content: content.into(),
        ..Default::default()
    }
}

#[ allow(unused)]
pub fn get_text_content(msg: MessageObject) -> Result<String> {
    let msg_content = msg
            .content
            .into_iter()
            .next()
            .ok_or_else(|| "No content in message".to_string())?;

    let txt = match msg_content {
        MessageContent::Text(text) => text.text.value,
        MessageContent::ImageFile(_) => {
            return Err("Image content not supported".into());
        }
    };

    Ok(txt)
}