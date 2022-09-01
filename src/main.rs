use linkify::{LinkFinder, LinkKind};
use url::Url;

use std::env;
use std::time::Duration;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use tokio::time::sleep;

struct Handler;

struct LinkFinderUrl;

impl TypeMapKey for LinkFinderUrl {
    type Value = Arc<LinkFinder>; // gonna pretend this thing doesn't need an RwLock
}

const NITTER_CUNNYCON: &str = "https://nitter.cunnycon.org";
const PIXIV_CUNNYCON: &str = "https://pixiv.cunnycon.org";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, mut msg: Message) {
        let finder = {
            let data = ctx.data.read().await;
            data.get::<LinkFinderUrl>().expect("Expected LinkFinderSingle in TypeMap.").clone()
        };

        let mut new_urls = Vec::<String>::new();
        
        for link in finder.links(&msg.content).flat_map(|x| Url::parse(x.as_str())) {
            if let Some(domain) = link.domain() {
                if domain.contains("twitter.com") {
                    new_urls.push(format!("{}{}", NITTER_CUNNYCON, link.path()));
                }
                else if domain.contains("pixiv.net") {
                    new_urls.push(format!("{}{}", PIXIV_CUNNYCON, link.path()));
                }
            }
        }

        if !new_urls.is_empty() {
            let message = new_urls.join(" | ");
            if let Err(why) = msg.reply(&ctx.http, message.as_str()).await {
                println!("Error sending message: {:?}", why);
            }
            // TODO: move this to another task which will periodically check if an embed exists, up to a maximum of X seconds
            tokio::spawn(async move {
                sleep(Duration::from_millis(500)).await;
                if let Err(why) = msg.suppress_embeds(&ctx.http).await {
                    println!("Error suppressing message embeds: {:?}", why);
                }
            });
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;

        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);

        data.insert::<LinkFinderUrl>(Arc::new(finder));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
