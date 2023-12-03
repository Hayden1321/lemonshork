use std::env;

use dotenv;
use serenity::{Client, client::{EventHandler, Context}, async_trait, all::GatewayIntents};

use serenity::model::gateway::Ready;

mod events;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, _: Ready) {
        println!("blehhhh :3, it works.");
    }

    async fn message(&self, ctx: Context, msg: serenity::model::channel::Message) {
        events::message::message(self, ctx, msg).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&env::var("DISCORD_TOKEN").expect("Expected a token"), intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");


    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
