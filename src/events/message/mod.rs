use crate::{Group, Keyword, Paste};
use leptess::LepTess;
use regex::Regex;
use serenity::{all::ReactionType, client::Context};
use std::{error::Error, fmt, io::Cursor};
use url::Url;

mod get_content;
mod get_ocr;
mod get_paste;
mod parse_content;

#[derive(Debug)]
pub enum MessageError {
    RegexCaptureError,
    FilterError,
    HttpRequestError,
    UrlParseError,
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
            MessageError::UrlParseError => fmt.write_str("Failed to parse URL"),
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

    match get_paste::handler(group, url).await {
        Ok((g, url)) => {
            parse_content::handler().await; // Make this take http and then reply to the user
        }
        Err(MessageError::BadType) => {
            // IGNORE THIS
        }
        Err(_) => {}
    }

    //TODO: Handle OCR & Regular Text

    Ok(())
}
