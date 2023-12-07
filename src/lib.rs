pub mod common {
    pub struct HttpKey;

    impl songbird::typemap::TypeMapKey for HttpKey {
        type Value = reqwest::Client;
    }
}

pub mod util {
    use serenity::model::channel::Message;
    use serenity::Result as SerenityResult;

    pub fn check_msg(result: SerenityResult<Message>) {
        if let Err(why) = result {
            println!("Error sending message: {:?}", why);
        }
    }
}
