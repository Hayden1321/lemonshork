use super::{
    get_ocr, get_paste,
    parse_content::{self, MessageMatchType},
    MessageError,
};
use crate::Group;

pub async fn handler(
    url: Option<String>,
    content: String,
    group: &Group,
) -> Result<Option<MessageMatchType>, MessageError> {
    match url {
        Some(url) => {
            match get_paste::handler(group, &url).await {
                Ok(output) => {
                    match parse_content::handler(output.to_lowercase(), group)
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
                    match parse_content::handler(output.to_lowercase(), group)
                        .await
                        .map_err(|_| MessageError::RegexCaptureError)?
                    {
                        Some(r) => return Ok(Some(r)),
                        None => {}
                    };
                }
                Err(_) => return Err(MessageError::TesseractError),
            }

            return Ok(None);
        }
        None => {
            match parse_content::handler(content.to_lowercase(), group)
                .await
                .map_err(|_| MessageError::RegexCaptureError)?
            {
                Some(r) => return Ok(Some(r)),
                None => return Ok(None),
            };
        }
    };
}
