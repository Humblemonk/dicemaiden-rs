use crate::dice;
use crate::help_text; // Import the shared help text module from src root
use crate::DatabaseContainer;
use anyhow::Result;
use regex::Regex;
use serenity::{
    all::{CommandDataOptionValue, CommandInteraction, CommandOptionType},
    builder::{CreateCommand, CreateCommandOption},
    prelude::Context,
};
use sysinfo::{Pid, System};

// Custom response type to include privacy information
#[derive(Debug)]
pub struct CommandResponse {
    pub content: String,
    pub ephemeral: bool,
}

impl CommandResponse {
    pub fn new(content: String, ephemeral: bool) -> Self {
        Self { content, ephemeral }
    }

    pub fn public(content: String) -> Self {
        Self::new(content, false)
    }

    pub fn private(content: String) -> Self {
        Self::new(content, true)
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("roll")
        .description("Ask Dice Maiden to roll some dice!")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "dice",
                "Dice expression (e.g., 2d6+3, 4d6 k3, 3d10 t7)",
            )
            .required(true),
        )
}

pub fn register_r_alias() -> CreateCommand {
    CreateCommand::new("r")
        .description("Roll dice (short alias)")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "dice",
                "Dice expression (e.g., 2d6+3, 4d6 k3, 3d10 t7)",
            )
            .required(true),
        )
}

// Helper function to get the display name (nickname or username)
fn get_display_name(command: &CommandInteraction) -> String {
    // Try to get the nickname from the member info (only available in guilds)
    if let Some(member) = &command.member {
        if let Some(nick) = &member.nick {
            return nick.clone();
        }
    }

    // Fall back to the user's global display name or username
    if !command
        .user
        .global_name
        .as_ref()
        .unwrap_or(&String::new())
        .is_empty()
    {
        command.user.global_name.as_ref().unwrap().clone()
    } else {
        command.user.name.clone()
    }
}

// Helper function to calculate server and user counts from cache
fn get_server_and_user_counts(ctx: &Context) -> (usize, usize) {
    let server_count = ctx.cache.guilds().len();
    let user_count: usize = ctx
        .cache
        .guilds()
        .iter()
        .map(|guild_id| {
            ctx.cache
                .guild(*guild_id)
                .map(|guild| guild.member_count as usize)
                .unwrap_or(0)
        })
        .sum();

    (server_count, user_count)
}

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<CommandResponse> {
    let options = &command.data.options;

    let dice_expr = options
        .first()
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("1d6");

    // Handle special commands using shared help text
    match dice_expr.trim().to_lowercase().as_str() {
        "help" => return Ok(CommandResponse::public(help_text::generate_basic_help())),
        "help alias" => return Ok(CommandResponse::public(help_text::generate_alias_help())),
        "help system" => return Ok(CommandResponse::public(help_text::generate_system_help())),
        "donate" => return Ok(CommandResponse::public(generate_donate_text())),
        "bot-info" => {
            let bot_info = generate_bot_info(ctx).await?;
            return Ok(CommandResponse::public(bot_info));
        }
        _ => {} // Continue with normal dice parsing
    }

    // Get the display name (nickname if available, otherwise username)
    let display_name = get_display_name(command);

    // Parse and roll dice
    match dice::parse_and_roll(dice_expr) {
        Ok(results) => {
            let formatted = dice::format_multiple_results_with_limit(&results);

            // Check if any roll was marked as private
            let is_private = results.iter().any(|r| r.private);

            if is_private {
                // For private rolls, strip comment from request display
                let clean_expr = strip_comment_from_expression(dice_expr);
                Ok(CommandResponse::private(format!(
                    "🎲 **Private Roll** `{clean_expr}` {formatted}"
                )))
            } else {
                // Check if this has multiple results that are semicolon-separated
                let is_semicolon_separated = results.len() > 1
                    && results.iter().any(|r| r.original_expression.is_some())
                    && !results
                        .iter()
                        .all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));

                let content = if is_semicolon_separated {
                    // For semicolon-separated rolls, the formatted string already contains individual requests
                    format!("🎲 **{display_name}** {formatted}")
                } else if results.len() > 1 {
                    // For roll sets, strip comment from request display
                    let clean_expr = strip_comment_from_expression(dice_expr);
                    format!("🎲 **{display_name}** Request: `{clean_expr}`\n{formatted}")
                } else {
                    // Single result, strip comment from request display
                    let clean_expr = strip_comment_from_expression(dice_expr);
                    format!("🎲 **{display_name}** Request: `{clean_expr}` {formatted}")
                };

                // Check if final content exceeds Discord limit
                if content.len() > 2000 {
                    // Final fallback - just show the simplified result
                    let clean_expr = strip_comment_from_expression(dice_expr);
                    let simplified_result = if results.len() == 1 {
                        let mut simplified = results[0].create_simplified();
                        simplified.simple = true; // Show only result, no dice breakdown
                        simplified.to_string()
                    } else {
                        format!(
                            "**{}** total results Reason: `Simplified roll due to character limit`",
                            results.len()
                        )
                    };

                    let simplified_content = format!(
                        "🎲 **{display_name}** Request: `{clean_expr}` {simplified_result}"
                    );
                    Ok(CommandResponse::public(simplified_content))
                } else {
                    Ok(CommandResponse::public(content))
                }
            }
        }
        Err(e) => {
            let clean_expr = strip_comment_from_expression(dice_expr);
            let content = format!("🎲 **{display_name}** used `{clean_expr}` - ❌ **Error**: {e}");
            Ok(CommandResponse::public(content))
        }
    }
}

fn generate_donate_text() -> String {
    "Care to support the bot? You can donate via Patreon https://www.patreon.com/dicemaiden \n Another option is join the Dice Maiden Discord server and subscribe! https://discord.gg/4T3R5Cb".to_string()
}

async fn generate_bot_info(ctx: &Context) -> Result<String> {
    let mut system = System::new_all();
    system.refresh_all();

    // Get memory usage
    let current_pid = std::process::id();
    let memory_usage = if let Some(process) = system.process(Pid::from_u32(current_pid)) {
        process.memory() as f64 / 1024.0 / 1024.0 // Convert from KB to MB
    } else {
        0.0
    };

    // Format memory with appropriate unit
    let memory_display = if memory_usage > 1000.0 {
        format!("{:.2} GB", memory_usage / 1024.0)
    } else {
        format!("{memory_usage:.2} MB")
    };

    // Get database stats first to determine if we should show partial or total stats
    let db_stats_result = get_database_stats(ctx).await;

    // Check if we're in multi-process mode by looking for environment variables
    let is_multi_process =
        std::env::var("SHARD_START").is_ok() && std::env::var("TOTAL_SHARDS").is_ok();

    let stats_section = if is_multi_process {
        // Multi-process mode: Show process-specific stats and reference database for totals
        let shard_start = std::env::var("SHARD_START")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let shard_count = std::env::var("SHARD_COUNT")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        let total_shards = std::env::var("TOTAL_SHARDS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(shard_count);

        // Get current process's server and user counts using helper function
        let (process_server_count, process_user_count) = get_server_and_user_counts(ctx);

        format!(
            r#"**Current Process Stats:**
• Process Shards: {} to {} ({} shards of {} total)
• Process Servers: {}
• Process Users: ~{}
• Process Memory: {}"#,
            shard_start,
            shard_start + shard_count + 1,
            shard_count + 1,
            total_shards,
            process_server_count,
            process_user_count,
            memory_display
        )
    } else {
        // Single process mode: Show normal stats using helper function
        let (server_count, user_count) = get_server_and_user_counts(ctx);

        format!(
            r#"**Current Stats:**
• Servers: {server_count}
• Estimated Users: {user_count}
• Memory Usage: {memory_display}"#
        )
    };

    Ok(format!(
        r#"🤖 **Dice Maiden Bot Info** 🤖

{stats_section}

{db_stats_result}"#
    ))
}

async fn get_database_stats(ctx: &Context) -> String {
    let data = ctx.data.read().await;
    if let Some(db) = data.get::<DatabaseContainer>() {
        // Check if we're in multi-process mode
        let is_multi_process =
            std::env::var("SHARD_START").is_ok() && std::env::var("TOTAL_SHARDS").is_ok();

        if is_multi_process {
            // Multi-process mode: Use process_stats table
            match db.get_all_process_stats().await {
                Ok(process_stats) => {
                    if process_stats.is_empty() {
                        return "**Total Stats (All Processes):** No data yet\n".to_string();
                    }

                    // Get the most recent timestamp
                    let most_recent_timestamp = process_stats
                        .first()
                        .map(|s| s.timestamp.as_str())
                        .unwrap_or("Unknown");

                    // Sum server counts from all active processes
                    let total_servers: i32 = process_stats.iter().map(|s| s.server_count).sum();

                    // Sum memory usage from all processes
                    let total_memory: f64 = process_stats.iter().map(|s| s.memory_mb).sum();

                    // Count active processes
                    let active_processes = process_stats.len();

                    // Get total shards from any process (they should all have the same total_shards)
                    let total_shards = process_stats.first().map(|s| s.total_shards).unwrap_or(0);

                    // Format memory with appropriate unit
                    let memory_display = if total_memory > 1000.0 {
                        format!("{:.2} GB", total_memory / 1024.0)
                    } else {
                        format!("{total_memory:.2} MB")
                    };

                    format!(
                        "**Total Stats (All Processes):**\n• Last update: {most_recent_timestamp}\n• Total servers: {total_servers} (across {total_shards} shards)\n• Total memory: {memory_display}\n• Active processes: {active_processes} (multi-process sharding)\n"
                    )
                }
                Err(_) => "**Total Stats (All Processes):** Error reading data\n".to_string(),
            }
        } else {
            // Single-process mode: Use shard_stats table
            match db.get_all_shard_stats().await {
                Ok(stats) => {
                    if stats.is_empty() {
                        return "**Database Stats:** No data yet\n".to_string();
                    }

                    // Get the most recent timestamp from any shard
                    let most_recent_timestamp = stats
                        .first()
                        .map(|s| s.timestamp.as_str())
                        .unwrap_or("Unknown");

                    // Sum server counts from all shards
                    let total_servers: i32 = stats.iter().map(|s| s.server_count).sum();

                    // Get memory usage specifically from shard 0 (only shard that records actual memory)
                    let shard_0_memory = stats
                        .iter()
                        .find(|s| s.shard_id == 0)
                        .map(|s| s.memory_mb)
                        .unwrap_or(0.0);

                    // Format recorded memory with appropriate unit
                    let memory_display = if shard_0_memory > 1000.0 {
                        format!("{:.2} GB", shard_0_memory / 1024.0)
                    } else {
                        format!("{shard_0_memory:.2} MB")
                    };

                    // Count how many shards we have data for
                    let shard_count = stats.len();

                    format!(
                        "**Database Stats:**\n• Last update: {most_recent_timestamp}\n• Total recorded servers: {total_servers} (across {shard_count} shards)\n• Recorded memory: {memory_display}\n"
                    )
                }
                Err(_) => "**Database Stats:** Error reading data\n".to_string(),
            }
        }
    } else {
        "**Database Stats:** Not available\n".to_string()
    }
}

fn strip_comment_from_expression(expr: &str) -> String {
    // Use regex to remove comment (everything after ! including the !)
    let comment_regex = Regex::new(r"\s*!\s*.*$").unwrap();
    comment_regex.replace(expr, "").trim().to_string()
}
