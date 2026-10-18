//! Bot entry point and Discord gateway management.
//!
//! Initialises the SQLite database, resolves the shard strategy from environment
//! variables, builds the Serenity [`Client`], and starts the Discord gateway.
//!
//! # Shard strategies
//!
//! | `USE_AUTOSHARDING` | `SHARD_COUNT` | `SHARD_START` / `TOTAL_SHARDS` | Strategy          |
//! |--------------------|---------------|----------------------------------|-------------------|
//! | `true`             | —             | —                                | autoshard         |
//! | unset              | set           | both set                         | multi_process     |
//! | unset              | set           | unset                            | manual            |
//! | unset              | unset         | —                                | single            |
//!
//! In **multi-process** mode several bot processes each own a contiguous shard
//! range (`SHARD_START..=SHARD_START+SHARD_COUNT`, i.e. `SHARD_COUNT + 1` shards, because
//! Serenity's `start_shard_range` treats the range end as inclusive), sharing a single SQLite DB
//! via the `process_stats` table.  Single-process mode uses `shard_stats`.
//!
//! # Environment variables
//!
//! | Variable            | Required | Description                                      |
//! |---------------------|----------|--------------------------------------------------|
//! | `DISCORD_TOKEN`     | ✓        | Bot token from the Discord developer portal      |
//! | `DATABASE_URL`      |          | SQLite URL; defaults to `main.db` (see database) |
//! | `GUILD_ID`          |          | Register commands to one guild (dev/testing)     |
//! | `SHARD_COUNT`       |          | Shards to run; multi-process: one less than that |
//! | `SHARD_START`       |          | First shard ID in multi-process deployments      |
//! | `TOTAL_SHARDS`      |          | Total shards across all processes                |
//! | `USE_AUTOSHARDING`  |          | Let Discord decide the shard count (`true`)      |
//! | `MAX_CONCURRENCY`   |          | Hint; overridden by Discord's reported value     |
//! | `RUST_LOG`          |          | `tracing` filter; defaults to `info`             |
//!
//! # Event handler
//!
//! [`Handler::ready`] fires once per shard connection.  Only shard 0 registers
//! slash commands (`/roll`, `/r`, `/help`, `/purge`) to avoid duplicate
//! registrations when running many shards.
//!
//! [`Handler::interaction_create`] dispatches incoming slash-command interactions
//! to `commands::roll`, `commands::help`, or `commands::purge`.
//!
//! A background task collects per-shard guild counts and process memory usage
//! every 15 minutes and writes them to the database.  On SIGTERM, SIGINT or
//! Ctrl-C, `main` broadcasts a shutdown on a `broadcast` channel that both this
//! task and the Discord client listen on.

use anyhow::{Context as _, Result};
use dicemaiden_rs::{
    DatabaseContainer, GuildCountsContainer, ShardManagerContainer, commands, database,
    dice::truncate_to_message_limit, guild_counts::GuildCounts,
};
use serenity::{
    all::*, async_trait, gateway::ShardManager, http::Http, model::gateway::Ready, prelude::*,
};
use std::{env, sync::Arc, time::Duration};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::{
    select,
    signal::unix::{SignalKind, signal},
    sync::broadcast,
    time::interval,
};
use tracing::{error, info, warn};

struct Handler {
    shard_count: u32,
    guild_counts: Arc<GuildCounts>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        self.guild_counts
            .ready(ready.guilds.iter().map(|guild| guild.id));

        let connected = get_connected_shard_count(&ctx).await;

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

    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        self.guild_counts.create(guild.id, guild.member_count);
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        incomplete: UnavailableGuild,
        _full: Option<Guild>,
    ) {
        self.guild_counts
            .delete(incomplete.id, incomplete.unavailable);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // `/purge` can fall back to deleting up to 100 messages one at a
            // time, which outlasts Discord's 3-second response window, so it
            // acknowledges first and edits the result in afterwards.
            let deferred = command.data.name == "purge";
            if deferred && let Err(why) = command.defer(&ctx.http).await {
                error!("Cannot defer purge command: {}", why);
                return;
            }

            let response = match command.data.name.as_str() {
                "roll" => commands::roll::run(&ctx, &command).await,
                "r" => commands::roll::run(&ctx, &command).await,
                "help" => commands::help::run(&ctx, &command).await,
                "purge" => commands::purge::run(&ctx, &command)
                    .await
                    .map(commands::CommandResponse::public),
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
            // Each command fits its own output where it can do so gracefully;
            // this is the backstop for every reply, so none fails to send.
            let response_content = truncate_to_message_limit(response_content);

            let sent = if deferred {
                // A deferred response's visibility was fixed by the defer
                command
                    .edit_response(
                        &ctx.http,
                        serenity::builder::EditInteractionResponse::new()
                            .content(response_content)
                            .allowed_mentions(user_mentions_only()),
                    )
                    .await
                    .map(|_| ())
            } else {
                let mut response_message =
                    serenity::builder::CreateInteractionResponseMessage::new()
                        .content(response_content)
                        .allowed_mentions(user_mentions_only());

                if ephemeral {
                    response_message = response_message.ephemeral(true);
                }

                command
                    .create_response(
                        &ctx.http,
                        serenity::builder::CreateInteractionResponse::Message(response_message),
                    )
                    .await
            };

            if let Err(why) = sent {
                error!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

// Replies echo user text (labels, comments, nicknames, expressions in error
// messages), so `/roll (@everyone) 1d6` pinged the whole server. A roll may
// ping the users it names (`/roll (@player) 1d20`); @everyone, @here and role
// mentions stay inert.
fn user_mentions_only() -> serenity::builder::CreateAllowedMentions {
    serenity::builder::CreateAllowedMentions::new().all_users(true)
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // Initialize database
    let db = Arc::new(database::Database::new().await?);
    db.init().await?;

    let token = env::var("DISCORD_TOKEN").context("Expected DISCORD_TOKEN in environment")?;

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
                warn!(
                    "Discord reports max_concurrency=1. Your bot may not have large bot sharding approved."
                );
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
                    start + process_shard_count,
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
                        // Estimate based on typical large bot ratios; zero if manual_count is 0
                        let guilds_per_shard = (2500 * recommended_shards)
                            .checked_div(manual_count)
                            .unwrap_or(0);

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
        warn!(
            "SHARD_COUNT ({}) is not a multiple of 16. This is required for Discord's large bot sharding.",
            shard_count
        );
        warn!(
            "Consider using 16, 32, 48, 64, etc. shards if you need large bot sharding approval."
        );
    }

    let intents = GatewayIntents::GUILDS;

    // No Serenity cache (the `cache` feature is off): it held every server's full
    // data, ~50 KB each, and only the server and member counts were ever read.
    let guild_counts = Arc::new(GuildCounts::default());

    let client = Client::builder(&token, intents)
        // Sent in each shard's IDENTIFY, so no extra presence update on every reconnect
        .activity(ActivityData::listening("/roll"))
        .event_handler(Handler {
            shard_count,
            guild_counts: Arc::clone(&guild_counts),
        })
        .await
        .context("Error creating client")?;

    // Share the shard manager, database and guild counts with the command handlers
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<DatabaseContainer>(Arc::clone(&db));
        data.insert::<GuildCountsContainer>(Arc::clone(&guild_counts));
    }

    // Create shutdown broadcast channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    let shard_manager = Arc::clone(&client.shard_manager);
    let db_clone = Arc::clone(&db);
    let stats_shutdown_rx = shutdown_tx.subscribe();

    // Start the statistics collection task with graceful shutdown
    let stats_handle = tokio::spawn(async move {
        if let Err(e) = collect_shard_stats_with_shutdown(
            db_clone,
            guild_counts,
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

    // Log how this process is running, then wait for a shutdown signal
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
            if shard_count == 1 {
                info!("Starting single shard");
                client.start().await
            } else {
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
                // Not start_shard_range: its inclusive end would also boot an invalid shard
                // `shard_count` (see the multi_process arm). start_shards boots 0..shard_count.
                client.start_shards(shard_count).await
            }
        }
        "multi_process" => {
            // Serenity's `start_shard_range` takes a `Range` but treats `range.end` as
            // inclusive (`init = end - start + 1`; its docs say `4..7` starts "shards 4
            // through 7"), so this boots shard_start..=end_shard, which is
            // SHARD_COUNT + 1 shards. Deployments set SHARD_COUNT one lower to compensate.
            // E.g. SHARD_START=208, SHARD_COUNT=15 boots 208..=223 (16 shards).
            let end_shard = shard_start + shard_count;

            info!(
                "Starting shard range {} to {} ({} shards) out of {} total",
                shard_start,
                end_shard,
                shard_count + 1,
                total_shards
            );
            info!(
                "This process will take ~{} seconds to start",
                shard_count * 5
            );

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
    let (mut sigterm, mut sigint) = match (
        signal(SignalKind::terminate()),
        signal(SignalKind::interrupt()),
    ) {
        (Ok(sigterm), Ok(sigint)) => (sigterm, sigint),
        (Err(e), _) | (_, Err(e)) => {
            // Returning would start a shutdown (main awaits this), so keep
            // running and fall back to Ctrl+C; if even that can't be awaited,
            // wait for the process to be killed.
            error!("Failed to register signal handlers: {}", e);
            if let Err(e) = tokio::signal::ctrl_c().await {
                error!("Failed to listen for Ctrl+C: {}", e);
                std::future::pending::<()>().await;
            }
            info!("Received Ctrl+C");
            if let Err(e) = shutdown_tx.send(()) {
                error!("Failed to send shutdown signal: {}", e);
            }
            return;
        }
    };

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

/// Every 15 minutes, write this process's server count and memory use to the
/// database: one `process_stats` row in multi-process mode, one `shard_stats`
/// row per connected shard otherwise. Runs until a shutdown is broadcast.
async fn collect_shard_stats_with_shutdown(
    db: Arc<database::Database>,
    guild_counts: Arc<GuildCounts>,
    shard_manager: Arc<ShardManager>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    let mut interval = interval(Duration::from_secs(900)); // 15 minutes

    // Create system info once and only refresh our specific process
    let mut system = System::new();
    let current_pid = Pid::from_u32(std::process::id());

    // Check if we're in multi-process mode
    let is_multi_process = env::var("SHARD_START").is_ok() && env::var("TOTAL_SHARDS").is_ok();

    // One row per shard range, stable across restarts. It held the PID, so a
    // restarted container added a second row while the old one lived on until
    // the 30-minute cleanup, and bot-info's totals counted that process twice.
    let process_id = if is_multi_process {
        let shard_start = env::var("SHARD_START").unwrap_or_default();
        format!("process_{shard_start}")
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
    // Rows from before the stable ID (`process_<start>_<pid>`) are removed
    // after this process's first write, not at startup: removing them first
    // left the range out of bot-info and topgg.sh until that write.
    let mut stale_rows_removed = false;

    // The first tick fires at once, before the guild counts have loaded; it
    // replaced this range's full count with a near-empty one. Skip it, so the
    // last full count stands until a real one replaces it.
    interval.tick().await;

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

        // Copy what the stats need and release the runners lock before any
        // database await: the ShardManager and every `ready` handler wait on it.
        let (total_shards_in_process, connected_shards) = {
            let runners = shard_manager.runners.lock().await;
            // Only single-process mode writes per-shard rows
            let connected: Vec<ShardId> = if is_multi_process {
                Vec::new()
            } else {
                runners
                    .iter()
                    .filter(|(_, runner)| {
                        runner.stage == serenity::gateway::ConnectionStage::Connected
                    })
                    .map(|(&shard_id, _)| shard_id)
                    .collect()
            };
            (runners.len(), connected)
        };
        if total_shards_in_process == 0 {
            continue;
        }

        // Calculate memory once for all shards (they share the same process)
        let memory_usage = if let Some(process) = system.process(current_pid) {
            process.memory() as f64 / 1024.0 / 1024.0 // Convert from KB to MB
        } else {
            0.0
        };

        // Get total guild count more efficiently
        let total_guilds = guild_counts.totals().0 as i32;

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
            } else if !stale_rows_removed {
                match db.remove_stale_process_rows(&process_id, shard_start).await {
                    Ok(()) => stale_rows_removed = true,
                    Err(e) => warn!("Failed to remove stale process stats: {}", e),
                }
            }

            // In multi-process mode, we only use process_stats table
            // Individual shard stats are not needed since process_stats contains all the information

            // Prune rows from processes that stopped writing (45 minutes unwritten)
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
            for shard_id in connected_shards {
                // Approximate guild distribution instead of expensive per-shard calculations
                let shard_guild_count = if shard_id.0 == 0 {
                    // Shard 0 gets any remainder
                    guilds_per_shard + (total_guilds % total_shards_in_process as i32)
                } else {
                    guilds_per_shard
                };

                // Only shard 0 reports memory usage to avoid duplication in database
                let shard_memory = if shard_id.0 == 0 { memory_usage } else { 0.0 };

                if let Err(e) = db
                    .update_shard_stats(shard_id.0 as i32, shard_guild_count, shard_memory)
                    .await
                {
                    error!("Failed to update shard {} stats: {}", shard_id.0, e);
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

/// Helper function to get the actual count of connected shards from the shard manager
async fn get_connected_shard_count(ctx: &Context) -> u32 {
    // Access the shard manager from the context data
    let data = ctx.data.read().await;

    if let Some(shard_manager) = data.get::<ShardManagerContainer>() {
        let runners = shard_manager.runners.lock().await;

        // Count shards that are actually in Connected state
        runners
            .values()
            .filter(|runner| runner.stage == serenity::gateway::ConnectionStage::Connected)
            .count() as u32
    } else {
        // Fallback if we can't access the shard manager
        warn!("Could not access shard manager to count connected shards");
        0
    }
}
