use core::fmt;
use std::{env, error::Error, fs::File, io::BufReader};

use dotenv;
use serde::Deserialize;
use serenity::{
    all::GatewayIntents,
    async_trait,
    client::{Context, EventHandler},
    Client,
};

use serenity::model::gateway::Ready;

mod events;

#[derive(Debug)]
enum DiscordClientError {
    TokenFailed,
    ClientError,
    ConfigReadFailure,
    ConfigParseFailure,
    EnvError,
}

impl Error for DiscordClientError {}

impl fmt::Display for DiscordClientError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscordClientError::TokenFailed => fmt.write_str("Failed to get token"),
            DiscordClientError::ClientError => fmt.write_str("Failed to create client"),
            DiscordClientError::ConfigReadFailure => fmt.write_str("Failed to read config file"),
            DiscordClientError::ConfigParseFailure => fmt.write_str("Failed to parse config file"),
            DiscordClientError::EnvError => fmt.write_str("Failed to read .env file"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Config {
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
    dotenv::dotenv().map_err(|_| DiscordClientError::EnvError)?;

    let file = File::open("./config.json").map_err(|_| DiscordClientError::ConfigReadFailure)?;
    let reader = BufReader::new(file);
    let cfg: Config =
        serde_json::from_reader(reader).map_err(|_| DiscordClientError::ConfigParseFailure)?;

    let handler = Handler { cfg };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(
        &env::var("DISCORD_TOKEN").map_err(|_| DiscordClientError::TokenFailed)?,
        intents,
    )
    .event_handler(handler)
    .await
    .map_err(|_| DiscordClientError::ClientError)?;

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
