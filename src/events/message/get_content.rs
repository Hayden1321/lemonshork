use super::MessageError;
use regex::Regex;
use serenity::model::channel::Message;

pub async fn handler(msg: Message) -> Result<Option<String>, MessageError> {
    if msg.attachments.len() > 0 {
        return Ok(Some(msg.attachments[0].clone().proxy_url));
    } else if msg.content.len() > 0 {
        let regex = match Regex::new(r"(?P<url>https?://.*)")
            .map_err(|_| MessageError::RegexCaptureError)?
            .captures(&msg.content)
        {
            Some(reg) => reg,
            None => return Ok(None),
        };

        match regex.name("url") {
            Some(res) => return Ok(Some(res.as_str().to_string())),
            None => return Ok(None),
        }
    }

    Err(MessageError::RegexCaptureError)
}
