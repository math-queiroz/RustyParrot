use std::env;

use serenity::all::Ready;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::standard::{Configuration, StandardFramework};
use serenity::{async_trait, Client};

mod commands;
use commands::*;
use serenity::model::gateway::GatewayIntents;
use songbird::SerenityInit;

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(play, pause, leave, queue, clear, skip, volume)]
struct Music;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(3)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("rusty-parrot.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database file");

    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");

    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP)
        .group(&MUSIC_GROUP);

    framework.configure(Configuration::new().prefix("."));

    let token =
        env::var("DISCORD_TOKEN").expect("No key DISCORD_TOKEN found in env vars or .env file");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<lib::common::SqlitePoolKey>(database)
        .type_map_insert::<lib::common::HttpKey>(reqwest::Client::new())
        .await
        .expect("Error while creating the client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
