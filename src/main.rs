use std::{env, fs::File, io::BufReader};

use dotenv;
use serde::Deserialize;
use serenity::{Client, client::{EventHandler, Context}, async_trait, all::GatewayIntents};

use serenity::model::gateway::Ready;

mod events;

#[derive(Deserialize, Debug)]
pub struct Config {
    groups: Vec<Group>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Group {
    channels: Vec<String>,
    keywords: Vec<Keyword>,
    regex: Vec<Regex>,
    parsing: Parsing
}

#[derive(Deserialize, Debug, Clone)]
struct Keyword {
    keyword: String,
    reaction: Option<String>,
    response: String
}

#[derive(Deserialize, Debug, Clone)]
struct Regex {
    pattern: String,
    reaction: Option<String>,
    response: String
}

#[derive(Deserialize, Debug, Clone)]
struct Parsing {
    paste: Vec<Paste>
}

#[derive(Deserialize, Debug, Clone)]
struct Paste {
    domain: String,
    format: String
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
        if msg.author.bot { return; }
        events::message::message(self, ctx, msg).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");

    let file = File::open("./config.json").expect("Failed to open config file");
    let reader = BufReader::new(file);
    let cfg: Config = serde_json::from_reader(reader).expect("Failed to parse config file");

    let handler = Handler { cfg };

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&env::var("DISCORD_TOKEN").expect("Expected a token"), intents)
        .event_handler(handler)
        .await
        .expect("Err creating client");


    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
