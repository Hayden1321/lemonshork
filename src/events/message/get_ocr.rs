use url::Url;

use super::MessageError;
use crate::{Group, Paste};

pub async fn handler(group: &Group, url: String) -> Result<(String, Group), MessageError> {
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
        //TODO: DO THE ORC READING & RETURN
    }

    Err(MessageError::BadType)
}
