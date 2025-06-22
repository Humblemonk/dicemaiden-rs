mod commands;
mod database;
mod dice;
mod help_text;

use anyhow::Result;
use serenity::{
    all::*, async_trait, gateway::ShardManager, http::Http, model::gateway::Ready, prelude::*,
};
use std::{collections::HashSet, env, sync::Arc, time::Duration};
use sysinfo::{Pid, System};
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    time::interval,
};
use tracing::{error, info, warn};

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
        info!(
            "{} is connected on shard {}!",
            ready.user.name, ctx.shard_id
        );

        // Set the bot's activity status to "Listening to /roll"
        let activity = ActivityData::listening("/roll");
        ctx.set_activity(Some(activity));

        // Only log this once from shard 0 to avoid spam
        if ctx.shard_id.0 == 0 {
            info!("Bot activity set to 'Listening to /roll'");
        }

        let guild_id = env::var("GUILD_ID")
            .ok()
            .and_then(|id| id.parse().ok())
            .map(GuildId::new);

        // Only register commands from shard 0 to avoid conflicts
        if ctx.shard_id.0 == 0 {
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
                    info!("Registered {} slash commands", commands.len());
                }
                Err(e) => {
                    error!("Failed to register slash commands: {}", e);
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response = match command.data.name.as_str() {
                "roll" => commands::roll::run(&ctx, &command).await,
                "r" => commands::roll::run(&ctx, &command).await,
                "help" => match commands::help::run(&ctx, &command).await {
                    Ok(content) => Ok(commands::CommandResponse::public(content)),
                    Err(e) => Err(e),
                },
                "purge" => match commands::purge::run(&ctx, &command).await {
                    Ok(content) => Ok(commands::CommandResponse::public(content)),
                    Err(e) => Err(e),
                },
                _ => Ok(commands::CommandResponse::public(
                    "Unknown command".to_string(),
                )),
            };

            let (response_content, ephemeral) = match response {
                Ok(cmd_response) => (cmd_response.content, cmd_response.ephemeral),
                Err(e) => {
                    error!("Error executing command: {}", e);
                    (
                        "An error occurred while executing the command.".to_string(),
                        false,
                    )
                }
            };

            let mut response_message = serenity::builder::CreateInteractionResponseMessage::new()
                .content(response_content);

            if ephemeral {
                response_message = response_message.ephemeral(true);
            }

            if let Err(why) = command
                .create_response(
                    &ctx.http,
                    serenity::builder::CreateInteractionResponse::Message(response_message),
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

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Configure the number of shards (must be a multiple of 16 for large bot sharding)
    let shard_count = env::var("SHARD_COUNT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1); // Default to 1 shard for smaller bots

    // Validate that shard count is a multiple of 16 for large bot sharding
    if shard_count % 16 != 0 {
        warn!("SHARD_COUNT ({}) is not a multiple of 16. This is required for Discord's large bot sharding.", shard_count);
        warn!(
            "Consider using 16, 32, 48, 64, etc. shards if you need large bot sharding approval."
        );
    }

    info!("Configuring bot with {} shards", shard_count);

    let http = Http::new(&token);
    let (_owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(owner) = &info.owner {
                owners.insert(owner.id);
            }
            if let Some(team) = info.team {
                for member in &team.members {
                    owners.insert(member.user.id);
                }
            }
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let intents = GatewayIntents::GUILDS;

    // Create client with explicit shard configuration
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // Configure sharding after client creation
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<DatabaseContainer>(Arc::clone(&db));
    }

    // Create shutdown broadcast channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let shard_manager = Arc::clone(&client.shard_manager);
    let db_clone = Arc::clone(&db);
    let cache_clone = Arc::clone(&client.cache);
    let stats_shutdown_rx = shutdown_tx.subscribe();

    // Start the statistics collection task with graceful shutdown
    let stats_handle = tokio::spawn(async move {
        if let Err(e) = collect_shard_stats_with_shutdown(
            db_clone,
            cache_clone,
            shard_manager,
            stats_shutdown_rx,
        )
        .await
        {
            error!("Statistics collection error: {}", e);
        }
        info!("Statistics collection task stopped");
    });

    // Setup signal handlers for graceful shutdown
    let shutdown_signal = setup_signal_handlers(shutdown_tx.clone());

    // Run the client with graceful shutdown
    let client_handle = tokio::spawn(async move {
        if let Err(why) = client.start_shards(shard_count).await {
            error!("Client error: {:?}", why);
        }
        info!("Discord client stopped");
    });

    // Wait for shutdown signal
    info!(
        "Dice Maiden is running with {} shards. Press Ctrl+C to shut down gracefully.",
        shard_count
    );
    shutdown_signal.await;

    info!("Shutdown signal received, initiating graceful shutdown...");

    // Broadcast shutdown to all tasks
    if let Err(e) = shutdown_tx.send(()) {
        warn!("Error broadcasting shutdown signal: {}", e);
    }

    // Wait for background tasks with timeout (longer for multiple shards)
    let shutdown_timeout = Duration::from_secs(30); // Increased from 10 to 30 seconds

    tokio::select! {
        _ = stats_handle => {
            info!("Statistics task finished");
        }
        _ = tokio::time::sleep(shutdown_timeout) => {
            warn!("Statistics task did not finish within timeout, continuing shutdown");
        }
    }

    // Shutdown Discord client
    tokio::select! {
        _ = client_handle => {
            info!("Discord client finished");
        }
        _ = tokio::time::sleep(shutdown_timeout) => {
            warn!("Discord client did not finish within timeout, continuing shutdown");
        }
    }

    info!("Graceful shutdown completed");
    Ok(())
}

async fn setup_signal_handlers(shutdown_tx: broadcast::Sender<()>) {
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");

    select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C");
        }
    }

    // Send shutdown signal to all tasks
    if let Err(e) = shutdown_tx.send(()) {
        error!("Failed to send shutdown signal: {}", e);
    }
}

async fn collect_shard_stats_with_shutdown(
    db: Arc<database::Database>,
    cache: Arc<serenity::cache::Cache>,
    shard_manager: Arc<ShardManager>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(300)); // 5 minutes
    let mut system = System::new_all();

    loop {
        select! {
            _ = interval.tick() => {
                // Continue with stats collection
            }
            _ = shutdown_rx.recv() => {
                info!("Statistics collection received shutdown signal");
                break;
            }
        }

        // Refresh system information
        system.refresh_all();

        // Get current process memory usage in MB
        let current_pid = std::process::id();
        let memory_usage = if let Some(process) = system.process(Pid::from_u32(current_pid)) {
            process.memory() as f64 / 1024.0 / 1024.0 // Convert from KB to MB
        } else {
            0.0
        };

        // Get shard information
        let shard_info = match shard_manager.runners.lock().await {
            shard_runners => shard_runners,
        };

        if shard_info.is_empty() {
            // No shards running yet, wait for next interval
            continue;
        }

        let total_shards = shard_info.len();

        // Iterate through all active shards
        for (&shard_id, shard_runner) in shard_info.iter() {
            // Check if this shard is connected (skip if shutting down)
            if shard_runner.stage != serenity::gateway::ConnectionStage::Connected {
                continue;
            }

            // Get guilds for this specific shard (with error handling for shutdown)
            let shard_guild_count = cache
                .guilds()
                .iter()
                .filter(|guild_id| {
                    // Calculate which shard this guild belongs to
                    // Discord uses: (guild_id >> 22) % num_shards
                    let guild_shard_id = (guild_id.get() >> 22) % total_shards as u64;
                    guild_shard_id == shard_id.0 as u64
                })
                .count() as i32;

            // Update stats for this shard (continue on error during shutdown)
            if let Err(e) = db
                .update_shard_stats(shard_id.0 as i32, shard_guild_count, memory_usage)
                .await
            {
                // During shutdown, database errors are expected, so just log them at debug level
                if shard_runner.stage == serenity::gateway::ConnectionStage::Connected {
                    error!("Failed to update shard {} stats: {}", shard_id.0, e);
                } else {
                    warn!(
                        "Shard {} stats update failed during shutdown: {}",
                        shard_id.0, e
                    );
                }
            } else {
                info!(
                    "Updated shard {} stats: {} servers, {:.2} MB memory",
                    shard_id.0, shard_guild_count, memory_usage
                );
            }
        }

        // Also log total stats
        let total_guilds = cache.guilds().len();
        info!(
            "Total stats: {} shards, {} servers, {:.2} MB memory",
            total_shards, total_guilds, memory_usage
        );
    }

    info!("Statistics collection loop ended");
    Ok(())
}
