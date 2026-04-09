use serenity::{
    async_trait,
    model::{
        application::interaction::{Interaction, InteractionResponseType},
        gateway::Ready,
    },
    prelude::*,
};

use songbird::{SerenityInit,input::YoutubeDl};
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, ctx: Context, ready: Ready) {

        println!("{} is online!", ready.user.name);

        let guilds = ready.guilds;

        for guild in guilds {

            let id = guild.id;

            let _ = id.set_application_commands(&ctx.http, |commands| {

                commands.create_application_command(|c| {
                    c.name("play")
                    .description("Play music")
                    .create_option(|o| {
                        o.name("song")
                        .description("Song name")
                        .kind(serenity::model::prelude::command::CommandOptionType::String)
                        .required(true)
                    })
                });

                commands.create_application_command(|c| {
                    c.name("skip")
                    .description("Skip current song")
                });

                commands.create_application_command(|c| {
                    c.name("stop")
                    .description("Stop music")
                });

                commands

            }).await;
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::ApplicationCommand(command) = interaction {

            let guild_id = command.guild_id.unwrap();

            let manager = songbird::get(&ctx).await.unwrap().clone();

            if command.data.name == "play" {

                let query = command.data.options[0].value
                    .as_ref()
                    .unwrap()
                    .as_str()
                    .unwrap();

                let channel_id = command.member
                    .as_ref()
                    .unwrap()
                    .voice
                    .as_ref()
                    .unwrap()
                    .channel_id
                    .unwrap();

                let handler = manager.join(guild_id, channel_id).await;

                if let Ok(handler_lock) = handler {

                    let mut handler = handler_lock.lock().await;

                    let source = YoutubeDl::new(query.to_string());

                    handler.play_source(source.into());

                }

                let _ = command.create_interaction_response(&ctx.http, |r| {

                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| {

                        m.content(format!("🎵 Playing {}", query))

                    })

                }).await;
            }

            if command.data.name == "skip" {

                if let Some(handler_lock) = manager.get(guild_id) {

                    let handler = handler_lock.lock().await;

                    handler.queue().skip().unwrap();

                }

                let _ = command.create_interaction_response(&ctx.http, |r| {

                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| {

                        m.content("⏭ Skipped")

                    })

                }).await;
            }

            if command.data.name == "stop" {

                if let Some(handler_lock) = manager.get(guild_id) {

                    let handler = handler_lock.lock().await;

                    handler.queue().stop();

                }

                let _ = command.create_interaction_response(&ctx.http, |r| {

                    r.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| {

                        m.content("⏹ Music stopped")

                    })

                }).await;
            }

        }

    }

}

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Token missing");

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {

        println!("Client error: {:?}", why);

    }

}
