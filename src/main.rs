use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        application::interaction::{
            Interaction,
            InteractionResponseType,
            application_command::ApplicationCommandInteraction
        },
        prelude::*,
        application::command::CommandOptionType
    },
    prelude::*,
};

use songbird::SerenityInit;

use std::env;
use warp::Filter;

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, ctx: Context, ready: Ready) {

        println!("{} is online!", ready.user.name);

        for guild in ready.guilds {

            let _ = guild.id.set_application_commands(&ctx.http, |commands| {

                commands.create_application_command(|c| {

                    c.name("play")
                    .description("Play music")

                    .create_option(|o| {

                        o.name("song")
                        .description("Song name")
                        .kind(CommandOptionType::String)
                        .required(true)

                    })

                });

                commands

            }).await;

        }

    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::ApplicationCommand(command) = interaction {

            if command.data.name == "play" {

                run_play(&ctx, &command).await;

            }

        }

    }

}

async fn run_play(ctx: &Context, command: &ApplicationCommandInteraction) {

    let query = command.data.options[0]
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();

    let guild_id = command.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .unwrap()
        .clone();

    let channel_id = {

        let guild = guild_id
            .to_guild_cached(&ctx.cache)
            .unwrap();

        let voice_state = guild
            .voice_states
            .get(&command.user.id)
            .unwrap();

        voice_state.channel_id.unwrap()

    };

    let (handler_lock, _) = manager
        .join(guild_id, channel_id)
        .await;

    let mut handler = handler_lock.lock().await;

    let source = songbird::ytdl_search(query)
        .await
        .expect("Error sourcing ffmpeg");

    handler.play_source(source);

    let _ = command.create_interaction_response(&ctx.http, |r| {

        r.kind(InteractionResponseType::ChannelMessageWithSource)

        .interaction_response_data(|m| {

            m.content(format!("🎵 Playing: {}", query))

        })

    }).await;

}

async fn web_server() {

    let route = warp::path::end()
        .map(|| "Bot Running 24/7");

    warp::serve(route)
        .run(([0,0,0,0],3000))
        .await;

}

#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    tokio::spawn(web_server());

    let token = env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN not set");

    let intents =
        GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(token,intents)

        .event_handler(Handler)

        .register_songbird()

        .await

        .expect("Error creating client");

    if let Err(why) = client.start().await {

        println!("Client error: {:?}", why);

    }

}
