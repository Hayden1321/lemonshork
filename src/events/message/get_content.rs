use super::MessageError;
use regex::Regex;
use serenity::model::channel::Message;

pub async fn handler(msg: Message) -> Result<String, MessageError> {
    if msg.attachments.len() > 0 {
        return Ok(msg.attachments[0].clone().proxy_url);
    } else if msg.content.len() > 0 {
        return Ok(Regex::new(r"(?P<url>https?://.*)")
            .map_err(|_| MessageError::RegexCaptureError)?
            .captures(&msg.content)
            .ok_or(MessageError::RegexCaptureError)?
            .name("url")
            .ok_or(MessageError::RegexCaptureError)?
            .as_str()
            .to_string());
    }

    Err(MessageError::RegexCaptureError)
}
