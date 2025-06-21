mod dice;
mod commands;
mod database;

use anyhow::Result;
use serenity::{
    async_trait,
    gateway::ShardManager,
    http::Http,
    model::gateway::Ready,
    prelude::*,
    all::*,
};
use std::{
    collections::HashSet,
    env,
    sync::Arc,
    time::Duration,
};
use tracing::{error, info};
use tokio::time::interval;
use sysinfo::{System, Pid};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}

pub struct DatabaseContainer;

impl TypeMapKey for DatabaseContainer {
    type Value = Arc<database::Database>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = env::var("GUILD_ID")
            .ok()
            .and_then(|id| id.parse().ok())
            .map(GuildId::new);

        // Register slash commands globally (or to a specific guild for testing)
        let commands = if let Some(guild_id) = guild_id {
            // Guild-specific commands for testing
            let commands = vec![
                commands::roll::register(),
                commands::roll::register_r_alias(),
                commands::help::register(),
                commands::purge::register(),
            ];
            
            guild_id.set_commands(&ctx.http, commands).await
        } else {
            // Global commands
            let commands = vec![
                commands::roll::register(),
                commands::roll::register_r_alias(),
                commands::help::register(),
                commands::purge::register(),
            ];
            
            Command::set_global_commands(&ctx.http, commands).await
        };

        match commands {
            Ok(commands) => {
                info!(
                    "Registered {} slash commands",
                    commands.len()
                );
            }
            Err(e) => {
                error!("Failed to register slash commands: {}", e);
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response = match command.data.name.as_str() {
                "roll" => commands::roll::run(&ctx, &command).await,
                "r" => commands::roll::run(&ctx, &command).await,
                "help" => {
                    // Convert help command response to the new format
                    match commands::help::run(&ctx, &command).await {
                        Ok(content) => Ok(commands::CommandResponse::public(content)),
                        Err(e) => Err(e),
                    }
                }
                "purge" => {
                    // Convert purge command response to the new format
                    match commands::purge::run(&ctx, &command).await {
                        Ok(content) => Ok(commands::CommandResponse::public(content)),
                        Err(e) => Err(e),
                    }
                }
                _ => Ok(commands::CommandResponse::public("Unknown command".to_string())),
            };

            let (response_content, ephemeral) = match response {
                Ok(cmd_response) => (cmd_response.content, cmd_response.ephemeral),
                Err(e) => {
                    error!("Error executing command: {}", e);
                    ("An error occurred while executing the command.".to_string(), false)
                }
            };

            // Create the response with appropriate ephemeral setting
            let mut response_message = serenity::builder::CreateInteractionResponseMessage::new()
                .content(response_content);
            
            if ephemeral {
                response_message = response_message.ephemeral(true);
            }

            if let Err(why) = command
                .create_response(&ctx.http, 
                    serenity::builder::CreateInteractionResponse::Message(response_message)
                )
                .await
            {
                error!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt::init();

    // Initialize database
    let db = Arc::new(database::Database::new().await?);
    db.init().await?;

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected DISCORD_TOKEN in environment");

    let http = Http::new(&token);
    let (_owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            match info.team {
                Some(team) => {
                    for member in &team.members {
                        owners.insert(member.user.id);
                    }
                }
                None => {}
            }
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<DatabaseContainer>(Arc::clone(&db));
    }

    let _shard_manager = Arc::clone(&client.shard_manager);
    let db_clone = Arc::clone(&db);
    let cache_clone = Arc::clone(&client.cache);

    // Start the statistics collection task
    tokio::spawn(async move {
        collect_shard_stats(db_clone, cache_clone).await;
    });

    let _shard_manager = Arc::clone(&client.shard_manager);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        _shard_manager.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    Ok(())
}

async fn collect_shard_stats(db: Arc<database::Database>, cache: Arc<serenity::cache::Cache>) {
    let mut interval = interval(Duration::from_secs(300)); // 5 minutes
    let mut system = System::new_all();
    
    loop {
        interval.tick().await;
        
        // Refresh system information
        system.refresh_all();
        
        // Get current process memory usage in MB
        let current_pid = std::process::id();
        let memory_usage = if let Some(process) = system.process(Pid::from_u32(current_pid)) {
            process.memory() as f64 / 1024.0 / 1024.0 // Convert from KB to MB
        } else {
            0.0
        };
        
        // Get server count from cache
        let server_count = cache.guilds().len() as i32;
        
        // For now, we'll use shard_id 0 since this is likely a single-shard bot
        // In a multi-shard setup, you'd iterate through all shards
        let shard_id = 0;
        
        if let Err(e) = db.update_shard_stats(shard_id, server_count, memory_usage).await {
            error!("Failed to update shard stats: {}", e);
        } else {
            info!("Updated shard {} stats: {} servers, {:.2} MB memory", 
                shard_id, server_count, memory_usage);
        }
    }
}
