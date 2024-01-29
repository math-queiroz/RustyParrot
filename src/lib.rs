pub mod util {
    use serenity::builder::{CreateEmbed, CreateMessage};
    use serenity::client::Context;
    use serenity::model::channel::Message;
    use serenity::model::Colour;
    use serenity::Result as SerenityResult;

    fn check_msg(result: SerenityResult<Message>) {
        if let Err(why) = result {
            println!("Error sending message: {:?}", why);
        }
    }

    pub async fn stylized_reply(
        ctx: &Context,
        msg: &Message,
        content: &str,
        title: Option<String>,
    ) {
        let mut e = CreateEmbed::new()
            .color(Colour::new(0xDC8841))
            .description(content);
        if let Some(title) = title {
            e = e.title(title);
        }
        check_msg(
            msg.channel_id
                .send_message(&ctx.http, CreateMessage::new().add_embed(e))
                .await,
        );
    }
}

pub mod common {
    pub struct HttpKey;
    pub struct SqlitePoolKey;

    impl songbird::typemap::TypeMapKey for HttpKey {
        type Value = reqwest::Client;
    }

    impl songbird::typemap::TypeMapKey for SqlitePoolKey {
        type Value = sqlx::sqlite::SqlitePool;
    }
}
