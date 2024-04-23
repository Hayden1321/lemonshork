use super::MessageError;
use crate::{Group, Keyword};
use regex::Regex;

pub async fn handler(
    content: String,
    group: &Group,
) -> Result<Option<MessageMatchType>, MessageError> {
    for x in group.regex.iter() {
        let reg = Regex::new(&x.pattern).map_err(|_| MessageError::RegexCaptureError)?;

        let cap = match reg.captures(&content) {
            Some(x) => x,
            None => continue,
        };

        if cap.get(0).is_some() {
            return Ok(Some(MessageMatchType::Regex(x.clone())));
        }
    }

    let keyword = match group
        .keywords
        .iter()
        .filter(|x| {
            content
                .trim()
                .to_lowercase()
                .split(" ")
                .collect::<Vec<&str>>()
                .contains(&x.keyword.as_str())
        })
        .collect::<Vec<&Keyword>>()
        .get(0)
    {
        Some(t) => *t,
        None => return Ok(None),
    };

    Ok(Some(MessageMatchType::Keyword(keyword.clone())))
}
