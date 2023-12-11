use lib::util::stylized_reply;

use reqwest::Client;
use serenity::async_trait;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use songbird::input::{Input, YoutubeDl};
use songbird::tracks::{PlayMode, Track};
use songbird::{Event, EventContext, EventHandler, Songbird, TrackEvent};

use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::sync::Arc;

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

async fn get_http_client(ctx: &Context) -> Client {
    let data = ctx.data.read().await;
    data.get::<lib::common::HttpKey>()
        .cloned()
        .expect("Typemap entry exists as from client initialisation")
}

async fn get_songbird_manager(ctx: &Context) -> Arc<Songbird> {
    songbird::get(ctx)
        .await
        .expect("Songbird's voice client exists as from client initialisation")
        .clone()
}

async fn parse_source(http_client: Client, query: &str) -> Result<(Input, Track), String> {
    let mut q = query;
    if q.starts_with('<') && q.ends_with('>') {
        q = &q[1..q.len() - 1];
    }

    let src = if q.starts_with("http") {
        YoutubeDl::new(http_client, q.to_string())
    } else {
        YoutubeDl::new(http_client, format!("ytsearch:{}", q))
    };
    let track = Track::from(src.clone());
    if !track.input.is_playable() {
        Ok((Input::from(src), track))
    } else {
        Err("There was an error playing this query".to_string())
    }
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.rest();
    if query.is_empty() {
        stylized_reply(
            ctx,
            msg,
            "Must provide a query URL/name to a video or audio",
            None,
        )
        .await;
        return Ok(());
    };
    let http_client = get_http_client(ctx).await;
    let (mut input, track) = match parse_source(http_client, query).await {
        Ok(track) => track,
        Err(reason) => {
            stylized_reply(
                ctx,
                msg,
                format!("Error parsing query: {}", reason).as_str(),
                None,
            )
            .await;
            return Ok(());
        }
    };
    let guild_id = msg.guild_id.unwrap();
    let channel_id = msg
        .guild(&ctx.cache)
        .unwrap()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);
    let manager = get_songbird_manager(ctx).await;
    let handler_lock = manager.get(guild_id);
    if handler_lock.is_none() {
        let voice_channel_id = match channel_id {
            Some(channel) => channel,
            None => {
                stylized_reply(ctx, msg, "Not in a voice channel", None).await;
                return Ok(());
            }
        };
        if let Ok(handler_lock) = manager.join(guild_id, voice_channel_id).await {
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        }
    }
    match manager.get(guild_id) {
        Some(handler_lock) => {
            let mut handler = handler_lock.lock().await;
            let (_track, meta) = futures::join!(handler.enqueue(track), input.aux_metadata());
            if let Ok(meta) = meta {
                let title = meta.title.unwrap_or("song".to_string());
                let source_url = meta
                    .source_url
                    .unwrap_or("<http://youtube.com>".to_string());
                stylized_reply(
                    ctx,
                    msg,
                    format!("Added [{}]({}) to queue!", title, source_url).as_str(),
                    None,
                )
                .await;
            }
        }
        None => stylized_reply(ctx, msg, "Not in a voice channel to play in", None).await,
    }
    Ok(())
}

#[command]
#[aliases(stop, resume)]
#[only_in(guilds)]
async fn pause(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = get_songbird_manager(ctx).await;
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let _ = match handler.queue().current() {
            Some(track) => match track.get_info().await.map(|i| i.playing) {
                Ok(PlayMode::Play) => track.pause(),
                Ok(PlayMode::Pause) => track.play(),
                Ok(_) | Err(_) => {
                    stylized_reply(ctx, msg, "Could not change song state...", None).await;
                    return Ok(());
                }
            },
            None => {
                stylized_reply(ctx, msg, "There is no song playing", None).await;
                return Ok(());
            }
        };
    } else {
        stylized_reply(ctx, msg, "Not in a voice channel to play in", None).await;
    }
    Ok(())
}

#[command]
#[aliases(die)]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = get_songbird_manager(ctx).await;
    match manager.get(guild_id) {
        Some(_) => {
            stylized_reply(ctx, msg, "Left voice channel", None).await;
            if let Err(e) = manager.remove(guild_id).await {
                stylized_reply(ctx, msg, format!("Failed: {:?}", e).as_str(), None).await;
            }
        }
        None => {
            stylized_reply(ctx, msg, "Not in a voice channel", None).await;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn queue(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = get_songbird_manager(ctx).await;
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue().current_queue();
        stylized_reply(
            ctx,
            msg,
            format!(
                "There are {} queued songs",
                if queue.len() < 2 {
                    "no".to_string()
                } else {
                    (queue.len() - 1).to_string()
                }
            )
            .as_str(),
            None,
        )
        .await;
    } else {
        stylized_reply(ctx, msg, "Not in a voice channel to play in", None).await;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn clear(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = get_songbird_manager(ctx).await;
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        handler
            .queue()
            .modify_queue(|q| q.drain(1..).collect::<VecDeque<_>>());
        stylized_reply(ctx, msg, "Cleared the queue", None).await;
    } else {
        stylized_reply(ctx, msg, "Not in a voice channel to play in", None).await;
    }
    Ok(())
}

#[command]
#[aliases(next)]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = get_songbird_manager(ctx).await;
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();
        stylized_reply(
            ctx,
            msg,
            format!(
                "Song skipped! {} left in queue",
                if queue.len() > 1 {
                    (queue.len() - 1).to_string()
                } else {
                    "No songs".to_string()
                }
            )
            .as_str(),
            None,
        )
        .await;
    } else {
        stylized_reply(ctx, msg, "Not in a voice channel to play in", None).await;
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let target_volume = match args.borrow_mut().single::<isize>() {
        Ok(vol) => {
            if vol < 0 || vol > 100 {
                stylized_reply(ctx, msg, "Must provide a value between 0 and 100", None).await;
                return Ok(());
            } else {
                vol
            }
        }
        Err(_) => {
            stylized_reply(ctx, msg, "Must provide a valid number", None).await;
            return Ok(());
        }
    };
    let guild_id = msg.guild_id.unwrap();
    let manager = get_songbird_manager(ctx).await;
    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        match handler.queue().current() {
            Some(track) => {
                let _ = track.set_volume(target_volume as f32 / 100f32);
            }
            None => {
                stylized_reply(ctx, msg, "There is no song playing", None).await;
                return Ok(());
            }
        }
    } else {
        stylized_reply(ctx, msg, "Not in a voice channel to play in", None).await;
    }
    Ok(())
}
