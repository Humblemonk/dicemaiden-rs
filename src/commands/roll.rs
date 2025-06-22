use crate::dice;
use crate::help_text; // Import the shared help text module from src root
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

    // Parse and roll dice
    match dice::parse_and_roll(dice_expr) {
        Ok(results) => {
            let formatted = dice::format_multiple_results(&results);

            // Check if any roll was marked as private
            let is_private = results.iter().any(|r| r.private);

            if is_private {
                // For private rolls, strip comment from request display
                let clean_expr = strip_comment_from_expression(dice_expr);
                Ok(CommandResponse::private(format!(
                    "🎲 **Private Roll** `{}` {}",
                    clean_expr, formatted
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
                    format!("🎲 **{}** {}", command.user.name, formatted)
                } else if results.len() > 1 {
                    // For roll sets, strip comment from request display
                    let clean_expr = strip_comment_from_expression(dice_expr);
                    format!(
                        "🎲 **{}** Request: `{}`\n{}",
                        command.user.name, clean_expr, formatted
                    )
                } else {
                    // Single result, strip comment from request display
                    let clean_expr = strip_comment_from_expression(dice_expr);
                    format!(
                        "🎲 **{}** Request: `{}` {}",
                        command.user.name, clean_expr, formatted
                    )
                };

                Ok(CommandResponse::public(content))
            }
        }
        Err(e) => {
            let clean_expr = strip_comment_from_expression(dice_expr);
            let content = format!(
                "🎲 **{}** used `{}` - ❌ **Error**: {}",
                command.user.name, clean_expr, e
            );
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

    // Get server count
    let server_count = ctx.cache.guilds().len();

    // Get user count (approximate)
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

    // Get database stats if available
    let db_stats = get_database_stats(ctx).await;

    Ok(format!(
        r#"🤖 **Dice Maiden Bot Info** 🤖

**Current Stats:**
• Servers: {}
• Estimated Users: {}
• Memory Usage: {:.2} MB

{}"#,
        server_count, user_count, memory_usage, db_stats
    ))
}

async fn get_database_stats(ctx: &Context) -> String {
    let data = ctx.data.read().await;
    if let Some(db) = data.get::<crate::DatabaseContainer>() {
        match db.get_all_shard_stats().await {
            Ok(stats) => {
                if let Some(latest) = stats.first() {
                    format!("**Database Stats:**\n• Last update: {}\n• Recorded servers: {}\n• Recorded memory: {:.2} MB\n", 
                        latest.timestamp, latest.server_count, latest.memory_mb)
                } else {
                    "**Database Stats:** No data yet\n".to_string()
                }
            }
            Err(_) => "**Database Stats:** Error reading data\n".to_string(),
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
