use self::parse_content::MessageMatchType;
use crate::Group;
use serenity::{all::ReactionType, client::Context};
use std::{error::Error, fmt};

mod get_content;
mod get_ocr;
mod get_paste;
mod get_result;
mod parse_content;

#[derive(Debug)]
pub enum MessageError {
    RegexCaptureError,
    FilterError,
    HttpRequestError,
    ClientError,
    TesseractError,
    ImageLoadError,
    BadType,
}

impl Error for MessageError {}

impl fmt::Display for MessageError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::RegexCaptureError => fmt.write_str("Failed to capture regex"),
            MessageError::FilterError => fmt.write_str("Failed to filter groups"),
            MessageError::HttpRequestError => fmt.write_str("Failed to make HTTP request"),
            MessageError::ClientError => fmt.write_str("Failed to create client"),
            MessageError::TesseractError => fmt.write_str("Failed to read image"),
            MessageError::ImageLoadError => fmt.write_str("Failed to load image"),
            MessageError::BadType => fmt.write_str("File/Paste is a bad type"),
        }
    }
}

pub async fn message(
    handler: &crate::Handler,
    ctx: Context,
    msg: serenity::model::channel::Message,
) -> Result<(), MessageError> {
    let url = get_content::handler(msg.clone())
        .await
        .map_err(|_| return MessageError::RegexCaptureError)?;

    let group = *handler
        .cfg
        .groups
        .iter()
        .filter(|x| x.channels.contains(&msg.channel_id.to_string()))
        .collect::<Vec<&Group>>()
        .get(0)
        .ok_or(MessageError::FilterError)?;

    let result = get_result::handler(url, msg.content.clone(), group)
        .await
        .map_err(|_| MessageError::FilterError)?
        .ok_or(MessageError::FilterError)?;

    match result {
        MessageMatchType::Keyword(keyword) => {
            msg.reply(&ctx.http, keyword.response.clone())
                .await
                .map_err(|_| MessageError::ClientError)?;
            msg.react(
                &ctx.http,
                ReactionType::Unicode(keyword.reaction.unwrap_or("👀".to_string())),
            )
            .await
            .map_err(|_| MessageError::ClientError)?;
        }
        MessageMatchType::Regex(regex) => {
            msg.reply(&ctx.http, regex.response.clone()).await.unwrap();
            msg.react(
                &ctx.http,
                ReactionType::Unicode(regex.reaction.unwrap_or("👀".to_string())),
            )
            .await
            .map_err(|_| MessageError::ClientError)?;
        }
    }

    Ok(())
}
