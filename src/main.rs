mod commands;
mod database;
mod dice;
mod help_text;

use anyhow::Result;
use serenity::{
    all::*, async_trait, cache::Settings as CacheSettings, gateway::ShardManager, http::Http,
    model::gateway::Ready, prelude::*,
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

struct Handler {
    shard_count: u32,
    connected_shards: Arc<std::sync::atomic::AtomicU32>,
}

impl Handler {
    fn new(shard_count: u32) -> Self {
        Self {
            shard_count,
            connected_shards: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let connected = self
            .connected_shards
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;

        info!(
            "Shard {} connected! ({}/{} shards ready)",
            ctx.shard_id.0, connected, self.shard_count
        );

        // Log progress milestones
        if connected % 25 == 0 || connected == self.shard_count {
            info!(
                "Startup progress: {}/{} shards connected ({:.1}%)",
                connected,
                self.shard_count,
                (connected as f32 / self.shard_count as f32) * 100.0
            );
        }

        // Set the bot's activity status to "Listening to /roll"
        let activity = ActivityData::listening("/roll");
        ctx.set_activity(Some(activity));

        // Only do initial setup from shard 0
        if ctx.shard_id.0 == 0 {
            info!("Bot activity set to 'Listening to /roll'");
            info!("Starting command registration...");
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
                    info!("Successfully registered {} slash commands", commands.len());
                }
                Err(e) => {
                    error!("Failed to register slash commands: {}", e);
                }
            }
        }

        // Log when all shards are ready
        if connected == self.shard_count {
            info!(
                "All {} shards are now connected and ready!",
                self.shard_count
            );
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

    // Read max_concurrency from environment
    let max_concurrency = env::var("MAX_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1); // Default to 1 for unverified bots

    // Configure the number of shards (must be a multiple of 16 for large bot sharding)
    let shard_count = env::var("SHARD_COUNT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1); // Default to 1 shard for smaller bots

    // Validate shard configuration for verified bots
    if max_concurrency > 1 {
        info!(
            "Verified bot detected with max_concurrency: {}",
            max_concurrency
        );

        // Calculate optimal startup
        let startup_batches = shard_count.div_ceil(max_concurrency);
        let estimated_startup = startup_batches * 5;

        info!(
            "Startup config: {} shards in {} batches (~{}s)",
            shard_count, startup_batches, estimated_startup
        );
    } else {
        info!("Unverified bot mode: 1 shard per 5 seconds");
    }

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

    // Configure minimal cache settings to dramatically reduce memory usage
    let mut cache_settings = CacheSettings::default();

    // Disable message caching completely (biggest memory saver for a dice bot)
    cache_settings.max_messages = 0;

    // Disable caching of unnecessary data for a dice bot
    cache_settings.cache_guilds = true; // Keep guild info for stats, but minimal data
    cache_settings.cache_channels = false; // Don't cache channel data - we don't need it
    cache_settings.cache_users = false; // Don't cache user data - we don't need it for dice rolling

    // Set TTL for temporary data (reduces memory over time)
    cache_settings.time_to_live = Duration::from_secs(3600); // 1 hour TTL

    info!("Configured minimal cache settings for reduced memory usage");

    // Create client with explicit shard configuration and optimized cache
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler::new(shard_count))
        .cache_settings(cache_settings) // Apply optimized cache settings
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
    if shard_count > 1 {
        info!("Dice Maiden is starting with {} shards. This may take a few minutes for all shards to connect...", shard_count);
        info!(
            "Expected startup time: ~{} minutes for {} shards",
            (shard_count as f32 / 60.0).ceil() as u32,
            shard_count
        );
    } else {
        info!(
            "Dice Maiden is running with {} shard. Press Ctrl+C to shut down gracefully.",
            shard_count
        );
    }

    shutdown_signal.await;

    info!("Shutdown signal received, initiating graceful shutdown...");

    // Broadcast shutdown to all tasks
    if let Err(e) = shutdown_tx.send(()) {
        warn!("Error broadcasting shutdown signal: {}", e);
    }

    // Wait for background tasks with timeout (longer for multiple shards)
    let shutdown_timeout = Duration::from_secs(5);

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

// Optimized statistics collection to reduce memory usage and system overhead
async fn collect_shard_stats_with_shutdown(
    db: Arc<database::Database>,
    cache: Arc<serenity::cache::Cache>,
    shard_manager: Arc<ShardManager>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(900)); // 15 minutes

    // Create system info once and only refresh our specific process
    let mut system = System::new();
    let current_pid = Pid::from_u32(std::process::id());

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

        // Only refresh our specific process memory usage instead of scanning all system processes
        system.refresh_process(current_pid);

        let shard_info = shard_manager.runners.lock().await;
        if shard_info.is_empty() {
            continue;
        }

        let total_shards = shard_info.len();

        // Calculate memory once for all shards (they share the same process)
        let memory_usage = if let Some(process) = system.process(current_pid) {
            process.memory() as f64 / 1024.0 / 1024.0 // Convert from KB to MB
        } else {
            0.0
        };

        // Get total guild count more efficiently
        let total_guilds = cache.guilds().len() as i32;
        let guilds_per_shard = if total_shards > 0 {
            total_guilds / total_shards as i32
        } else {
            0
        };

        // Process shards more efficiently
        for (&shard_id, shard_runner) in shard_info.iter() {
            if shard_runner.stage != serenity::gateway::ConnectionStage::Connected {
                continue;
            }

            // Approximate guild distribution instead of expensive per-shard calculations
            let shard_guild_count = if shard_id.0 == 0 {
                // Shard 0 gets any remainder
                guilds_per_shard + (total_guilds % total_shards as i32)
            } else {
                guilds_per_shard
            };

            // Only shard 0 reports memory usage to avoid duplication in database
            let shard_memory = if shard_id.0 == 0 { memory_usage } else { 0.0 };

            // Update stats with simplified error handling during shutdown
            if let Err(e) = db
                .update_shard_stats(shard_id.0 as i32, shard_guild_count, shard_memory)
                .await
            {
                if shard_runner.stage == serenity::gateway::ConnectionStage::Connected {
                    error!("Failed to update shard {} stats: {}", shard_id.0, e);
                } else {
                    warn!(
                        "Shard {} stats update failed during shutdown: {}",
                        shard_id.0, e
                    );
                }
            }
        }

        // Log summary every 15 minutes (same as stats collection interval)
        info!(
            "Stats summary: {} shards, {} servers, {:.2} MB memory",
            total_shards, total_guilds, memory_usage
        );
    }

    info!("Statistics collection loop ended");
    Ok(())
}
