use std::sync::RwLock;

use rand::Rng;
use serde_json::Value;
use serenity::{
    Client,
    all::{EventHandler, GatewayIntents, Mention, Message},
    async_trait,
};

#[derive(Default)]
struct Handler {
    grokking_out: RwLock<usize>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: serenity::prelude::Context, msg: Message) {
        let default_reponse = || async {
            msg.channel_id
                .say(&ctx.http, "yeah i rekon so".to_string())
                .await
                .unwrap();
        };

        if (msg.content.to_lowercase().contains("grok")
            || msg.content.to_lowercase().contains("gork"))
            && msg.author.display_name() != "grok-rs"
        {
            let contents_thing = std::fs::read_to_string("./responses.txt");

            let contents: Vec<String>;

            match contents_thing {
                Ok(value) => {
                    contents = value
                        .trim()
                        .split("\n")
                        .map(|x| x.to_owned())
                        .collect::<Vec<String>>();
                }
                Err(_) => {
                    default_reponse().await;
                    return;
                }
            }

            let index = rand::rng().random_range(0..contents.len() + (contents.len() / 2));

            if index >= contents.len() || msg.content.to_lowercase().contains("please answer") {
                let sys_prompt = if self.grokking_out.try_read().is_ok_and(|x| *x > 25) {
                    format!(
                        "{{\"model\": \"grok-rs\",\"prompt\":\"you are grok and you are GROKKING OUT! answer {}\", \"stream\":false}}",
                        msg.content
                    )
                } else {
                    format!(
                        "{{\"model\": \"grok-rs\",\"prompt\":\"you are grok and you think you are a very smart ai but you are very dumb, linkedin style. maybe the smartest. answer like an egotistical asshole. respond with at most a sentence. use emoji. {}\", \"stream\":false}}",
                        msg.content
                    )
                };
                ctx.http.broadcast_typing(msg.channel_id).await.unwrap();
                let client = reqwest::Client::new();
                let res = client
                    .post("http://192.168.1.44:11434/api/generate")
                    .body(sys_prompt)
                    .send()
                    .await;

                let response_text;

                if let Ok(response) = res {
                    response_text = response.text().await;
                } else {
                    default_reponse().await;
                    return ();
                }

                let json: Value = serde_json::from_str(&response_text.unwrap()).unwrap();
                //println!("{:?}", json);

                msg.channel_id
                    .say(
                        &ctx.http,
                        format!(
                            "{} {}",
                            Mention::from(msg.author.id),
                            json["response"].as_str().unwrap()
                        ),
                    )
                    .await
                    .unwrap();
            } else {
                msg.channel_id
                    .say(&ctx.http, &contents[index])
                    .await
                    .unwrap();
            }
            if let Ok(mut grok) = self.grokking_out.write() {
                if *grok > 25 {
                    *grok = 0;
                } else {
                    *grok += rand::rng().random_range(0..5)
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::fs::read_to_string("./token").unwrap();
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::default())
        .await
        .unwrap();

    client.start().await.unwrap();
}
