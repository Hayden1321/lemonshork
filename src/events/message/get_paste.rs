use super::MessageError;
use crate::{Group, Paste};
use url::Url;

pub async fn handler(group: &Group, url: &String) -> Result<String, MessageError> {
    let paste_config = *group
        .parsing
        .paste
        .iter()
        .filter(|x| {
            x.domain
                == match Url::parse(url.as_str()) {
                    Ok(x) => match x.domain() {
                        Some(x) => x.to_string(),
                        None => return false,
                    },
                    Err(_) => return false,
                }
        })
        .collect::<Vec<&Paste>>()
        .get(0)
        .ok_or(MessageError::FilterError)?;

    let mut paste_url = url.to_string();

    if !url.contains("raw") {
        paste_url = paste_config.format.replace(
            "&filename&",
            Url::parse(url.as_str())
                .map_err(|_| MessageError::RegexCaptureError)?
                .path()
                .trim_start_matches("/"),
        );
    }

    let request = reqwest::get(paste_url)
        .await
        .map_err(|_| MessageError::HttpRequestError)?;

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

        return Ok(text);
    }

    Err(MessageError::BadType)
}
