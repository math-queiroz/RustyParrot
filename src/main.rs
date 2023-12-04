use std::env;

use serenity::{async_trait, Client};
use serenity::client::EventHandler;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;

mod commands;
use commands::*;
use serenity::model::gateway::GatewayIntents;

#[group]
#[commands(ping)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP)
        .configure(|c| c.prefix("~"));

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
