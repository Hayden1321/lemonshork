use core::fmt;
use error_stack::{Result, ResultExt};
use serde::Deserialize;
use serenity::model::gateway::Ready;
use serenity::{
    all::GatewayIntents,
    async_trait,
    client::{Context, EventHandler},
    Client,
};
use std::{fs::File, io::BufReader};

mod events;

#[derive(Debug)]
struct DiscordClientError;

impl fmt::Display for DiscordClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("App start error")
    }
}

impl error_stack::Context for DiscordClientError {}

#[derive(Deserialize, Debug)]
pub struct Config {
    token: String,
    groups: Vec<Group>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Group {
    channels: Vec<String>,
    keywords: Vec<Keyword>,
    regex: Vec<Regex>,
    parsing: Parsing,
}

#[derive(Deserialize, Debug, Clone)]
struct Keyword {
    keyword: String,
    reaction: Option<String>,
    response: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Regex {
    pattern: String,
    reaction: Option<String>,
    response: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Parsing {
    paste: Vec<Paste>,
}

#[derive(Deserialize, Debug, Clone)]
struct Paste {
    domain: String,
    format: String,
}

pub struct Handler {
    pub cfg: Config,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, _: Ready) {
        println!("blehhhh :3, it works.");
    }

    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        if msg.author.bot {
            return;
        }
        match events::message::message(self, ctx, msg).await {
            Ok(_) => (),
            Err(e) => println!("Error: {:?}", e),
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), DiscordClientError> {
    let file = File::open("./config.json")
        .attach_printable("Failed to open config.json")
        .change_context(DiscordClientError)?;
    let reader = BufReader::new(file);
    let cfg: Config = serde_json::from_reader(reader)
        .attach_printable("Failed to serialize config (Is it valid?).")
        .change_context(DiscordClientError)?;

    let token = cfg.token.clone();

    let handler = Handler { cfg };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .attach_printable("Failed to build client.")
        .change_context(DiscordClientError)?;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
