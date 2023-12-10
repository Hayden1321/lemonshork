use serenity::{client::Context, all::ReactionType};
use regex::Regex;
use url::Url;
use leptess::LepTess;

use crate::{Group, Keyword, Paste};

pub async fn message(handler: &crate::Handler, ctx: Context, msg: serenity::model::channel::Message) {
	let url = if msg.attachments.len() > 0 {
		Ok(msg.attachments[0].proxy_url.as_str())
	} else if msg.content.len() > 0 {
		if let Ok(reg) = Regex::new(r"(?P<url>https?://.*)") {
			if let Some(cap) = reg.captures(&msg.content) {
				Ok(cap.name("url").unwrap().as_str())
			} else {
				Err(())
			}
		} else {
			Err(())
		}
	} else {
		Err(())
	};


	let groups = handler.cfg.groups.iter().filter(|x| x.channels.contains(&msg.channel_id.to_string())).collect::<Vec<&Group>>();
	let group = *groups.get(0).unwrap();

	if url.is_ok() {
		let url = url.expect("Failed to get URL, idk why tho.");

		let request = group.parsing.paste.iter().filter(|x| x.domain == Url::parse(url).unwrap().domain().unwrap()).collect::<Vec<&Paste>>();
		if request.get(0).is_some() {
			let paste = *request.get(0).unwrap();
			let mut paste_url = url.to_string();

			if !url.contains("raw") {
				paste_url = paste.format.replace("&filename&", Url::parse(url).unwrap().path().trim_start_matches("/"));
			}

			let request = reqwest::get(paste_url).await.unwrap();
			if request.headers().get("Content-Type").expect("Failed to get content type").to_str().unwrap().contains("text/plain") {
				let text = request.text().await.unwrap();

				let match_type = match_text(text, group.clone()).await;

				if match_type.is_some() {
					let match_type = match_type.unwrap();
		
					match match_type {
						MessageMatchType::Keyword(keyword) => {
							msg.reply(&ctx.http, keyword.response.clone()).await.unwrap();
							msg.react(&ctx.http, ReactionType::Unicode(keyword.reaction.unwrap_or("👀".to_string()))).await.unwrap();
						},
						MessageMatchType::Regex(regex) => {
							msg.reply(&ctx.http, regex.response.clone()).await.unwrap();
							msg.react(&ctx.http, ReactionType::Unicode(regex.reaction.unwrap_or("👀".to_string()))).await.unwrap();
						}
					}
				}
			} else {
				msg.reply(&ctx.http, "Failed to get paste, invalid content type.").await.unwrap();
			}

			return			
		}
		
		
		let request = reqwest::get(url).await.unwrap();

	  if request.headers().get("Content-Type").expect("Failed to get content type").to_str().unwrap().contains("image/jpeg") || request.headers().get("Content-Type").expect("Failed to get content type").to_str().unwrap().contains("image/png") {			
			let mut tesseract = LepTess::new(Some("./tessdata"), "eng").unwrap();
		
			tesseract.set_image_from_mem(&request.bytes().await.unwrap()).expect("Failed to set image from memory");

			let output = tesseract.get_utf8_text().unwrap();

			let match_type = match_text(output, group.clone()).await;

			if match_type.is_some() {
				let match_type = match_type.unwrap();
	
				match match_type {
					MessageMatchType::Keyword(keyword) => {
						msg.reply(&ctx.http, keyword.response.clone()).await.unwrap();
						msg.react(&ctx.http, ReactionType::Unicode(keyword.reaction.unwrap_or("👀".to_string()))).await.unwrap();
					},
					MessageMatchType::Regex(regex) => {
						msg.reply(&ctx.http, regex.response.clone()).await.unwrap();
						msg.react(&ctx.http, ReactionType::Unicode(regex.reaction.unwrap_or("👀".to_string()))).await.unwrap();
					}
				}
			}
		}
	} else {
		let message = msg.content.clone();

		if message.len() == 0 { return }

		let match_type = match_text(message, group.clone()).await;

		if match_type.is_some() {
			let match_type = match_type.unwrap();

			match match_type {
				MessageMatchType::Keyword(keyword) => {
					msg.reply(&ctx.http, keyword.response.clone()).await.unwrap();
					msg.react(&ctx.http, ReactionType::Unicode(keyword.reaction.unwrap_or("👀".to_string()))).await.unwrap();
				},
				MessageMatchType::Regex(regex) => {
					msg.reply(&ctx.http, regex.response.clone()).await.unwrap();
					msg.react(&ctx.http, ReactionType::Unicode(regex.reaction.unwrap_or("👀".to_string()))).await.unwrap();
				}
			}
		}
	}

	//TODO: Get OCR to read image, return. (CHANGE FOR NATIVE VERSION)
}

enum MessageMatchType {
	Keyword(Keyword),
	Regex(crate::Regex)
}

async fn match_text(text: String, group: Group) -> Option<MessageMatchType> {
	for x in group.regex.iter() {
		let reg = Regex::new(&x.pattern).expect("Failed to compile regex");

		let cap = match reg.captures(&text) {
			Some(x) => x,
			None => continue
		};

		if cap.get(0).is_some() {
			return Some(MessageMatchType::Regex(x.clone()))
		}
	}


	let keywords = group.keywords.iter().filter(|x| text.trim().to_lowercase().split(" ").collect::<Vec<&str>>().contains(&x.keyword.as_str())).collect::<Vec<&Keyword>>();
	if keywords.get(0).is_some() {
		let keyword = *keywords.get(0).unwrap();

		return Some(MessageMatchType::Keyword(keyword.clone()))
	}
	
	None
}