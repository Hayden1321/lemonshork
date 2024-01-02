use std::{error::Error, fmt, io::Cursor};

use leptess::LepTess;
use regex::Regex;
use serenity::{all::ReactionType, client::Context};
use url::Url;

use crate::{Group, Keyword, Paste};

#[derive(Debug)]
pub enum MessageError {
    RegexCaptureError,
    FilterError,
    HttpRequestError,
    UrlParseError,
    ClientError,
    TesseractError,
    ImageLoadError,
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
        }
    }
}

pub async fn message(
    handler: &crate::Handler,
    ctx: Context,
    msg: serenity::model::channel::Message,
) -> Result<(), MessageError> {
    let url = if msg.attachments.len() > 0 {
        Ok(msg.attachments[0].proxy_url.as_str())
    } else if msg.content.len() > 0 {
        if let Ok(reg) = Regex::new(r"(?P<url>https?://.*)") {
            if let Some(cap) = reg.captures(&msg.content) {
                match cap.name("url") {
                    Some(x) => Ok(x.as_str()),
                    None => Err(MessageError::RegexCaptureError),
                }
            } else {
                Err(MessageError::RegexCaptureError)
            }
        } else {
            Err(MessageError::RegexCaptureError)
        }
    } else {
        Err(MessageError::RegexCaptureError)
    }?;

    let groups = handler
        .cfg
        .groups
        .iter()
        .filter(|x| x.channels.contains(&msg.channel_id.to_string()))
        .collect::<Vec<&Group>>();
    let group = *groups.get(0).unwrap();

    let request = group
        .parsing
        .paste
        .iter()
        .filter(|x| {
            x.domain
                == match Url::parse(url) {
                    Ok(x) => match x.domain() {
                        Some(x) => x.to_string(),
                        None => return false,
                    },
                    Err(_) => return false,
                }
        })
        .collect::<Vec<&Paste>>();

    if request.get(0).is_some() {
        let paste = match request.get(0) {
            Some(x) => *x,
            None => return Err(MessageError::HttpRequestError),
        };
        let mut paste_url = url.to_string();

        if !url.contains("raw") {
            paste_url = paste.format.replace(
                "&filename&",
                Url::parse(url)
                    .map_err(|_| MessageError::RegexCaptureError)?
                    .path()
                    .trim_start_matches("/"),
            );
        }

        let request = reqwest::get(paste_url).await.unwrap();
        if request
            .headers()
            .get("Content-Type")
            .expect("Failed to get content type")
            .to_str()
            .map_err(|_| MessageError::HttpRequestError)?
            .contains("text/plain")
        {
            let text = request
                .text()
                .await
                .map_err(|_| MessageError::HttpRequestError)?;

            let match_type = match_text(text, group.clone()).await;

            if match_type.is_some() {
                let match_type = match match_type {
                    Some(x) => x,
                    None => return Err(MessageError::FilterError),
                };

                match match_type {
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
            }
        } else {
            msg.reply(&ctx.http, "Failed to get paste, invalid content type.")
                .await
                .map_err(|_| MessageError::UrlParseError)?;
        }

        return Ok(());
    }

    let request = reqwest::get(url)
        .await
        .map_err(|_| MessageError::HttpRequestError)?;

    if request
        .headers()
        .get("Content-Type")
        .expect("Failed to get content type")
        .to_str()
        .map_err(|_| MessageError::HttpRequestError)?
        .contains("image/jpeg")
        || request
            .headers()
            .get("Content-Type")
            .expect("Failed to get content type")
            .to_str()
            .map_err(|_| MessageError::HttpRequestError)?
            .contains("image/png")
    {
        let image = image::load_from_memory(
            &request
                .bytes()
                .await
                .map_err(|_| MessageError::ImageLoadError)?,
        )
        .map_err(|_| MessageError::ImageLoadError)?;
        let mut tiff_buff = Vec::new();

        image
            .write_to(
                &mut Cursor::new(&mut tiff_buff),
                image::ImageOutputFormat::Tiff,
            )
            .map_err(|_| MessageError::ImageLoadError)?;

        let mut tesseract = match LepTess::new(Some("./tessdata"), "eng") {
            Ok(x) => x,
            Err(_) => return Err(MessageError::TesseractError),
        };

        tesseract
            .set_image_from_mem(&tiff_buff)
            .expect("Failed to set image from memory");
        tesseract.set_source_resolution(70);

        let output = tesseract
            .get_utf8_text()
            .map_err(|_| MessageError::TesseractError)?;

        let match_type = match_text(output, group.clone()).await;

        if match_type.is_some() {
            let match_type = match match_type {
                Some(x) => x,
                None => return Err(MessageError::FilterError),
            };

            match match_type {
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
        }
        Ok(())
    } else {
        let message = msg.content.clone();

        if message.len() == 0 {
            return Err(MessageError::FilterError);
        }

        let match_type = match_text(message, group.clone()).await;

        if match_type.is_some() {
            let match_type = match match_type {
                Some(x) => x,
                None => return Err(MessageError::FilterError),
            };

            match match_type {
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
        };

        Ok(())
    }
}

enum MessageMatchType {
    Keyword(Keyword),
    Regex(crate::Regex),
}

async fn match_text(text: String, group: Group) -> Option<MessageMatchType> {
    for x in group.regex.iter() {
        let reg = Regex::new(&x.pattern).expect("Failed to compile regex");

        let cap = match reg.captures(&text) {
            Some(x) => x,
            None => continue,
        };

        if cap.get(0).is_some() {
            return Some(MessageMatchType::Regex(x.clone()));
        }
    }

    let keywords = group
        .keywords
        .iter()
        .filter(|x| {
            text.trim()
                .to_lowercase()
                .split(" ")
                .collect::<Vec<&str>>()
                .contains(&x.keyword.as_str())
        })
        .collect::<Vec<&Keyword>>();
    if keywords.get(0).is_some() {
        let keyword = *keywords.get(0).unwrap();

        return Some(MessageMatchType::Keyword(keyword.clone()));
    }

    None
}
