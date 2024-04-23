use super::{
    get_ocr, get_paste,
    parse_content::{self, MessageMatchType},
    MessageError,
};
use crate::Group;

pub async fn handler(
    url: String,
    content: String,
    group: &Group,
) -> Result<Option<MessageMatchType>, MessageError> {
    match get_paste::handler(group, &url).await {
        Ok(output) => {
            match parse_content::handler(output, group)
                .await
                .map_err(|_| MessageError::RegexCaptureError)?
            {
                Some(r) => return Ok(Some(r)),
                None => {}
            };
        }
        Err(MessageError::BadType) => {}
        Err(_) => {}
    }

    match get_ocr::handler(&url).await {
        Ok(output) => {
            match parse_content::handler(output, group)
                .await
                .map_err(|_| MessageError::RegexCaptureError)?
            {
                Some(r) => return Ok(Some(r)),
                None => {}
            };
        }
        Err(_) => {}
    }

    match parse_content::handler(content, group)
        .await
        .map_err(|_| MessageError::RegexCaptureError)?
    {
        Some(r) => return Ok(Some(r)),
        None => {}
    };

    Ok(None)
}
