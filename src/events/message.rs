use serenity::client::Context;
use regex::Regex;

pub async fn message(handler: &crate::Handler, ctx: Context, msg: serenity::model::channel::Message) {
	let reg = Regex::new(r"(?P<url>https?://.*)").expect("Failed to compile regex");

	let cap = reg.captures(&msg.content).expect("Failed to capture regex");

	println!("URL: {}", cap.name("url").unwrap().as_str());


}