use serenity::model::channel::Message;
use serenity::{
    client::{Client, Context, EventHandler},
    model::id::ChannelId,
};
// use serenity::model::prelude::*;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult, StandardFramework,
};
use serenity::prelude::*;
use serenity::{async_trait, framework::standard::Args};

use std::{fs, sync::Arc};

struct TargetWrap;

impl TypeMapKey for TargetWrap {
    type Value = Arc<RwLock<Option<ChannelId>>>;
}

#[group]
#[commands(setup)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.channel(&ctx).await.unwrap().private().is_some() {
            let lock = {
                let data = ctx.data.read().await;
                data.get::<TargetWrap>()
                    .expect("TargetWrap not initialized")
                    .clone()
            };

            {
                let chan = lock.read().await;
                let chan = chan.expect("Target not set. Use the setup command");
                let _ = chan.say(&ctx, msg.content.to_owned()).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = fs::read_to_string("DISCORD_TOKEN").expect("Error when reading the token");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<TargetWrap>(Arc::new(RwLock::new(None)));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
async fn setup(ctx: &Context, _: &Message, mut args: Args) -> CommandResult {
    let chan = args.single::<ChannelId>()?;
    {
        let mut data = ctx.data.write().await;
        data.insert::<TargetWrap>(Arc::new(RwLock::new(Some(chan))));
    }
    Ok(())
}
