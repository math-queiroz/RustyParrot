use lib::util::check_msg;
use serenity::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use songbird::input::YoutubeDl;
use songbird::{Event, EventContext, EventHandler, TrackEvent};

struct TrackErrorNotifier;

#[async_trait]
impl EventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(
                msg.reply(&ctx.http, "Must provide a URL to a video or audio")
                    .await,
            );

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(msg.reply(&ctx.http, "Must provide a valid URL").await);
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();
    let channel_id = msg
        .guild(&ctx.cache)
        .unwrap()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<lib::common::HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = manager.get(guild_id);
    if handler_lock.is_none() {
        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                check_msg(msg.reply(ctx, "Not in a voice channel").await);
                return Ok(());
            }
        };

        if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        }
    }

    match manager.get(guild_id) {
        Some(handler_lock) => {
            let mut handler = handler_lock.lock().await;
            let src = YoutubeDl::new(http_client, url);
            let _ = handler.play_input(src.into());
            check_msg(msg.reply(&ctx.http, "Playing song").await);
        }
        None => check_msg(
            msg.reply(&ctx.http, "Not in a voice channel to play in")
                .await,
        ),
    }

    Ok(())
}

#[command]
#[aliases(die)]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    match manager.get(guild_id) {
        Some(_) => {
            check_msg(msg.reply(&ctx.http, "Left voice channel").await);
            if let Err(e) = manager.remove(guild_id).await {
                check_msg(msg.reply(&ctx.http, format!("Failed: {:?}", e)).await);
            }
        }
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);
        }
    }

    Ok(())
}
