use std::io::Cursor;

use rusty_tesseract::{Args, Image};
use serenity::client::Context;
use regex::Regex;
use reqwest;

pub async fn message(handler: &crate::Handler, ctx: Context, msg: serenity::model::channel::Message) {
	let url: Result<&str, _> = if &msg.attachments.len() > &0 {
		Ok(&msg.attachments[0].proxy_url.as_str())
	} else if &msg.content.len() > &0 {
		let reg = Regex::new(r"(?P<url>https?://.*)").expect("Failed to compile regex");

		let cap = reg.captures(&msg.content).expect("Failed to capture regex");

		Ok(cap.name("url").unwrap().as_str())
		
	} else { 
		Err(())
	};

	if url.is_ok() {
		let url = url.expect("Failed to get URL, idk why tho.");
		let body = reqwest::get(url).await.unwrap().bytes().await.unwrap();

		let image = image::load_from_memory(&body).unwrap();
		
		let default_args = Args::default();
	
		let output = rusty_tesseract::image_to_string(&Image::from_dynamic_image(&image)
		.unwrap(), &default_args).unwrap();
	
		println!("{:?}", output);
	}
	
	// println!("URL: {}", cap.name("url").unwrap().as_str());
	// if let Some(url) =  {
		
	// };

	//TODO: Get OCR to read image, return.
	//TODO: Check against config (add support for pastebins and regular text), return result.
}