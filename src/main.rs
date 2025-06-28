use anyhow::Result;
use dicemaiden_rs::{commands, database, DatabaseContainer, ShardManagerContainer};
use serenity::{
    all::*, async_trait, cache::Settings as CacheSettings, gateway::ShardManager, http::Http,
    model::gateway::Ready, prelude::*,
};
use std::{collections::HashSet, env, sync::Arc, time::Duration};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    time::interval,
};
use tracing::{error, info, warn};

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

        // Display the actual shard ID, not an adjusted one
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

        // Only do initial setup from shard 0 globally (not per-process)
        if ctx.shard_id.0 == 0 {
            info!("Bot activity set to 'Listening to /roll'");
            info!("Starting command registration...");
        }

        let guild_id = env::var("GUILD_ID")
            .ok()
            .and_then(|id| id.parse().ok())
            .map(GuildId::new);

        // Only register commands from shard 0 globally to avoid conflicts across all processes
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

        // Log when all shards in this process are ready
        if connected == self.shard_count {
            info!(
                "All {} shards in this process are now connected and ready!",
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

    // Read max_concurrency from environment, but use Discord's reported value if available
    let env_max_concurrency = env::var("MAX_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(1);

    // Configure the number of shards - use Discord's recommendation or manual override
    let manual_shard_count = env::var("SHARD_COUNT")
        .ok()
        .and_then(|s| s.parse::<u32>().ok());

    let use_autosharding = env::var("USE_AUTOSHARDING")
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false);

    info!("Shard configuration:");
    if let Some(manual_count) = manual_shard_count {
        info!("  Manual shard count: {}", manual_count);
    }
    if use_autosharding {
        info!("  Autosharding: enabled (will use Discord's recommendation)");
    }
    info!("  Environment max concurrency: {}", env_max_concurrency);

    let http = Http::new(&token);

    // Check what Discord says about gateway info
    match http.get_gateway().await {
        Ok(gateway_info) => {
            info!("Gateway URL: {}", gateway_info.url);
        }
        Err(e) => {
            error!("Failed to get gateway info: {}", e);
        }
    }

    // Get bot gateway info which includes session_start_limit
    let (actual_max_concurrency, recommended_shards) = match http.get_bot_gateway().await {
        Ok(bot_gateway) => {
            info!("Recommended shards from Discord: {}", bot_gateway.shards);
            let session_start_limit = bot_gateway.session_start_limit;
            info!("Session start limit info:");
            info!("  Total: {}", session_start_limit.total);
            info!("  Remaining: {}", session_start_limit.remaining);
            info!("  Reset after: {}ms", session_start_limit.reset_after);
            info!("  Max concurrency: {}", session_start_limit.max_concurrency);

            // Convert Discord's u64 to u32 for consistency with our code
            let discord_max_concurrency = session_start_limit.max_concurrency as u32;

            if discord_max_concurrency != env_max_concurrency {
                warn!(
                    "Environment MAX_CONCURRENCY ({}) differs from Discord's actual limit ({})",
                    env_max_concurrency, discord_max_concurrency
                );
            }

            if discord_max_concurrency == 1 {
                warn!("Discord reports max_concurrency=1. Your bot may not have large bot sharding approved.");
                warn!("Even verified bots need separate approval for higher concurrency limits.");
            } else {
                info!(
                    "Discord approved max_concurrency: {}",
                    discord_max_concurrency
                );
            }

            (discord_max_concurrency, bot_gateway.shards)
        }
        Err(e) => {
            error!("Failed to get bot gateway info: {}", e);
            warn!(
                "Using environment MAX_CONCURRENCY value: {}",
                env_max_concurrency
            );
            (env_max_concurrency, manual_shard_count.unwrap_or(1))
        }
    };

    // Determine final shard strategy
    let (shard_strategy, shard_count, shard_start, total_shards) = if use_autosharding {
        info!(
            "Using autosharding with Discord's recommended {} shards",
            recommended_shards
        );
        warn!(
            "PERFORMANCE WARNING: {} shards = ~{} minute startup time due to Serenity limitation",
            recommended_shards,
            (recommended_shards * 5) / 60
        );
        warn!("Consider using manual shard count with multi-process for faster startup");
        ("autoshard", recommended_shards, 0, recommended_shards)
    } else if let Some(manual_count) = manual_shard_count {
        // Check for multi-process environment variables
        let shard_start_env = env::var("SHARD_START")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        let total_shards_env = env::var("TOTAL_SHARDS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());

        match (shard_start_env, total_shards_env) {
            (Some(start), Some(total)) => {
                // Multi-process mode
                let process_shard_count = manual_count;
                info!("Multi-process mode detected:");
                info!(
                    "  This process handles shards {} to {} ({} shards)",
                    start,
                    start + process_shard_count + 1,
                    process_shard_count + 1
                );
                info!("  Total shards across all processes: {}", total);
                info!(
                    "  Estimated startup time: ~{} seconds",
                    process_shard_count * 5
                );
                ("multi_process", process_shard_count, start, total)
            }
            _ => {
                // Single process mode
                if manual_count != recommended_shards {
                    let startup_time_minutes = (manual_count * 5) / 60;
                    info!(
                        "Manual shard count ({}) differs from Discord's recommendation ({})",
                        manual_count, recommended_shards
                    );
                    info!("Estimated startup time: ~{} minutes", startup_time_minutes);
                    warn!("For faster startup, consider multi-process sharding:");
                    warn!("  Set SHARD_START and TOTAL_SHARDS environment variables");

                    if manual_count < recommended_shards {
                        let guilds_per_shard = if manual_count > 0 {
                            // Estimate based on typical large bot ratios
                            2500 * recommended_shards / manual_count
                        } else {
                            0
                        };

                        if guilds_per_shard > 2500 {
                            warn!(
                                "WARNING: {} shards may exceed Discord's 2,500 guilds per shard limit",
                                manual_count
                            );
                            warn!("Estimated: ~{} guilds per shard", guilds_per_shard);
                            warn!(
                                "Consider using at least {} shards",
                                recommended_shards * 2500 / 2500
                            );
                        } else {
                            info!(
                                "Estimated: ~{} guilds per shard (within Discord limits)",
                                guilds_per_shard
                            );
                        }
                    }
                }
                info!("Using manual shard count: {}", manual_count);
                ("manual", manual_count, 0, manual_count)
            }
        }
    } else {
        info!("No sharding configuration found, using single shard");
        ("single", 1, 0, 1)
    };

    // Use Discord's actual max_concurrency instead of environment variable
    let max_concurrency = actual_max_concurrency;

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
        Err(why) => panic!("Could not access application info: {why:?}"),
    };

    // Validate shard configuration for verified bots
    if max_concurrency > 1 {
        info!(
            "Verified bot detected with max_concurrency: {}",
            max_concurrency
        );
    } else {
        info!("Unverified bot mode: 1 shard per 5 seconds");
    }

    // Validate that shard count is a multiple of 16 for large bot sharding
    if shard_count >= 16 && shard_count % 16 != 0 {
        warn!("SHARD_COUNT ({}) is not a multiple of 16. This is required for Discord's large bot sharding.", shard_count);
        warn!(
            "Consider using 16, 32, 48, 64, etc. shards if you need large bot sharding approval."
        );
    }

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
    let client = Client::builder(&token, intents)
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

    // Clone shard manager for shutdown
    let shard_manager_for_shutdown = Arc::clone(&client.shard_manager);
    let client_shutdown_rx = shutdown_tx.subscribe();

    // Start the client with proper shard handling
    let client_handle = tokio::spawn(async move {
        let mut shutdown_rx = client_shutdown_rx;

        select! {
            result = start_client_with_shard_strategy(client, shard_strategy, shard_count, shard_start, total_shards) => {
                if let Err(why) = result {
                    error!("Client error: {:?}", why);
                }
                info!("Discord client stopped naturally");
            }
            _ = shutdown_rx.recv() => {
                info!("Discord client received shutdown signal");
                shard_manager_for_shutdown.shutdown_all().await;
                info!("All shards shut down");
            }
        }
    });

    // Wait for shutdown signal and provide accurate information
    if use_autosharding {
        info!(
            "Dice Maiden using autosharding with {} recommended shards",
            shard_count
        );
        info!(
            "Discord will automatically manage shard concurrency (max_concurrency: {})",
            max_concurrency
        );
    } else if shard_strategy == "multi_process" {
        info!("Dice Maiden starting in multi-process mode");
        info!(
            "This process handles shards {} to {} ({} shards)",
            shard_start,
            shard_start + shard_count,
            shard_count + 1
        );
        info!("Total shards across all processes: {}", total_shards);
        info!("Use 'pkill dicemaiden-rs' to stop all processes");
    } else if shard_count > 1 {
        info!("Dice Maiden starting {} shards manually", shard_count);
        info!(
            "Serenity will handle concurrency internally (max_concurrency: {})",
            max_concurrency
        );

        if max_concurrency > 1 {
            info!("Note: If shards still start sequentially, this may be a Serenity limitation");
            info!("Consider using multi-process sharding for faster startup:");
            info!("  Set SHARD_START and TOTAL_SHARDS environment variables");
        }
    } else {
        info!("Dice Maiden is running with 1 shard. Press Ctrl+C to shut down gracefully.");
    }

    shutdown_signal.await;

    info!("Shutdown signal received, initiating graceful shutdown...");

    // Broadcast shutdown to all tasks
    if let Err(e) = shutdown_tx.send(()) {
        warn!("Error broadcasting shutdown signal: {}", e);
    }

    // Wait for background tasks with a reasonable timeout
    let shutdown_timeout = Duration::from_secs(3);

    tokio::select! {
        _ = stats_handle => {
            info!("Statistics task finished");
        }
        _ = tokio::time::sleep(shutdown_timeout) => {
            warn!("Statistics task did not finish within timeout, continuing shutdown");
        }
    }

    // Don't wait for Discord client - it will shut down when the process exits
    // Just give it a moment to clean up, but don't block shutdown
    tokio::select! {
        _ = client_handle => {
            info!("Discord client finished");
        }
        _ = tokio::time::sleep(Duration::from_millis(500)) => {
            info!("Discord client shutdown - continuing with process exit");
        }
    }

    info!("Graceful shutdown completed");
    Ok(())
}

/// Start the client with proper shard strategy implementation
async fn start_client_with_shard_strategy(
    mut client: Client,
    strategy: &str,
    shard_count: u32,
    shard_start: u32,
    total_shards: u32,
) -> Result<(), SerenityError> {
    match strategy {
        "single" => {
            info!("Starting single shard");
            client.start().await
        }
        "autoshard" => {
            warn!(
                "Shards will start sequentially (~{} minutes total)",
                (shard_count * 5) / 60
            );
            warn!("For faster startup, consider reducing SHARD_COUNT to 64-128 shards");
            info!("Starting with autosharding - {} shards", shard_count);
            client.start_autosharded().await
        }
        "manual" => {
            warn!(
                "Shards will start sequentially (~{} minutes total)",
                (shard_count * 5) / 60
            );
            warn!("For faster startup, consider using multi-process sharding");
            info!(
                "Starting {} shards manually (0 to {})",
                shard_count,
                shard_count - 1
            );
            client.start_shard_range(0..shard_count, shard_count).await
        }
        "multi_process" => {
            // The range should be shard_start..(shard_start + shard_count)
            let end_shard = shard_start + shard_count;

            info!(
                "Starting shard range {} to {} ({} shards) out of {} total",
                shard_start,
                shard_start + shard_count + 1,
                shard_count + 1,
                total_shards
            );
            info!(
                "This process will take ~{} seconds to start",
                shard_count * 5
            );

            // This should create a range from shard_start to shard_start + shard_count (exclusive)
            // For example: 208..224 includes shards 208, 209, 210, ..., 223 (16 shards)
            client
                .start_shard_range(shard_start..end_shard, total_shards)
                .await
        }
        _ => {
            error!("Unknown shard strategy: {}", strategy);
            Err(SerenityError::Other("Unknown shard strategy"))
        }
    }
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

// Optimized statistics collection to support both single-process and multi-process sharding
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

    // Check if we're in multi-process mode
    let is_multi_process = env::var("SHARD_START").is_ok() && env::var("TOTAL_SHARDS").is_ok();

    // Generate a unique process ID for multi-process mode
    let process_id = if is_multi_process {
        let shard_start = env::var("SHARD_START").unwrap_or_default();
        let process_pid = std::process::id();
        format!("process_{shard_start}_{process_pid}")
    } else {
        "single_process".to_string()
    };

    info!(
        "Statistics collection mode: {}",
        if is_multi_process {
            "multi-process"
        } else {
            "single-process"
        }
    );
    if is_multi_process {
        info!("Process ID: {}", process_id);
    }

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
        system.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), false);

        let shard_info = shard_manager.runners.lock().await;
        if shard_info.is_empty() {
            continue;
        }

        let total_shards_in_process = shard_info.len();

        // Calculate memory once for all shards (they share the same process)
        let memory_usage = if let Some(process) = system.process(current_pid) {
            process.memory() as f64 / 1024.0 / 1024.0 // Convert from KB to MB
        } else {
            0.0
        };

        // Get total guild count more efficiently
        let total_guilds = cache.guilds().len() as i32;

        if is_multi_process {
            // Multi-process mode: Use process_stats table
            let shard_start = env::var("SHARD_START")
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            let shard_count = env::var("SHARD_COUNT")
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(total_shards_in_process as i32);
            let total_shards = env::var("TOTAL_SHARDS")
                .ok()
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(shard_count);

            // Update process stats
            if let Err(e) = db
                .update_process_stats(
                    &process_id,
                    shard_start,
                    shard_count,
                    total_shards,
                    total_guilds,
                    memory_usage,
                )
                .await
            {
                error!("Failed to update process stats: {}", e);
            }

            // In multi-process mode, we only use process_stats table
            // Individual shard stats are not needed since process_stats contains all the information

            // Cleanup old process stats every hour
            if let Err(e) = db.cleanup_old_process_stats().await {
                warn!("Failed to cleanup old process stats: {}", e);
            }

            // Log summary for this process
            info!(
                "Process stats: {} shards ({}-{}), {} servers, {:.2} MB memory",
                shard_count + 1,
                shard_start,
                shard_start + shard_count,
                total_guilds,
                memory_usage
            );
        } else {
            // Single-process mode: Use existing shard_stats table
            let guilds_per_shard = if total_shards_in_process > 0 {
                total_guilds / total_shards_in_process as i32
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
                    guilds_per_shard + (total_guilds % total_shards_in_process as i32)
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
                total_shards_in_process, total_guilds, memory_usage
            );
        }
    }

    info!("Statistics collection loop ended");
    Ok(())
}
