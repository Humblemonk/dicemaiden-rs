use crate::dice;
use anyhow::Result;
use serenity::{
    builder::{CreateCommand, CreateCommandOption},
    all::{CommandInteraction, CommandDataOptionValue, CommandOptionType},
    prelude::Context,
};
use sysinfo::{System, Pid};

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
        .description("Roll dice using standard RPG notation")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "dice",
                "Dice expression (e.g., 2d6+3, 4d6 k3, 3d10 t7)"
            )
            .required(true)
        )
}

pub fn register_r_alias() -> CreateCommand {
    CreateCommand::new("r")
        .description("Roll dice (short alias)")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "dice",
                "Dice expression (e.g., 2d6+3, 4d6 k3, 3d10 t7)"
            )
            .required(true)
        )
}

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<CommandResponse> {
    let options = &command.data.options;
    
    let dice_expr = options.first()
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("1d6");

    // Handle special commands
    if dice_expr.trim().to_lowercase() == "help" {
        return Ok(CommandResponse::public(generate_help_text()));
    }
    
    if dice_expr.trim().to_lowercase() == "help alias" {
        return Ok(CommandResponse::public(generate_alias_help()));
    }
    
    if dice_expr.trim().to_lowercase() == "help system" {
        return Ok(CommandResponse::public(generate_system_help()));
    }
    
    if dice_expr.trim().to_lowercase() == "donate" {
        return Ok(CommandResponse::public("Care to support the bot? You can donate via Patreon https://www.patreon.com/dicemaiden \n Another option is join the Dice Maiden Discord server and subscribe! https://discord.gg/4T3R5Cb".to_string()));
    }

    if dice_expr.trim().to_lowercase() == "bot-info" {
        let bot_info = generate_bot_info(ctx).await?;
        return Ok(CommandResponse::public(bot_info));
    }

    // Parse and roll dice
    match dice::parse_and_roll(dice_expr) {
        Ok(results) => {
            let formatted = dice::format_multiple_results(&results);
            
            // Check if any roll was marked as private
            let is_private = results.iter().any(|r| r.private);
            
            if is_private {
                Ok(CommandResponse::private(format!("🎲 **Private Roll** `/roll {}` {}", dice_expr, formatted)))
            } else {
                // Check if this has multiple results that are semicolon-separated
                let is_semicolon_separated = results.len() > 1 
                    && results.iter().any(|r| r.original_expression.is_some())
                    && !results.iter().all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));
                
                let content = if is_semicolon_separated {
                    // For semicolon-separated rolls, the formatted string already contains individual requests
                    format!("🎲 **{}** {}", command.user.name, formatted)
                } else if results.len() > 1 {
                    // For roll sets, add newline after request for multiple results
                    format!("🎲 **{}** Request: `/roll {}`\n{}", command.user.name, dice_expr, formatted)
                } else {
                    // Single result stays on same line
                    format!("🎲 **{}** Request: `/roll {}` {}", command.user.name, dice_expr, formatted)
                };
                
                Ok(CommandResponse::public(content))
            }
        }
        Err(e) => {
            let content = format!("🎲 **{}** used `/roll {}` - ❌ **Error**: {}", command.user.name, dice_expr, e);
            Ok(CommandResponse::public(content))
        }
    }
}

fn generate_help_text() -> String {
    r#"🎲 **Dice Maiden** 🎲

**Basic Usage:**
`/roll 2d6 + 3d10` - Roll two six-sided dice and three ten-sided dice
`/roll 3d6 + 5` - Roll three six-sided dice and add five
`/roll 4d6 k3` - Roll four six-sided dice and keep the highest 3

**Modifiers:**
• `e6` or `e` - Explode on 6s (or max value)
• `ie6` - Explode indefinitely on 6s
• `d2` - Drop lowest 2 dice
• `k3` - Keep highest 3 dice  
• `kl2` - Keep lowest 2 dice
• `r2` - Reroll dice ≤ 2 once
• `ir2` - Reroll dice ≤ 2 indefinitely
• `t7` - Count successes (≥ 7)
• `f1` - Count failures (≤ 1)
• `b1` - Count botches (≤ 1)

**Special Flags:**
• `p` - Private roll (only you see results)
• `s` - Simple output (no dice breakdown)
• `nr` - No results shown (just total)
• `ul` - Unsorted dice results

**Examples:**
• `/roll 10d6 e6 k8 +4` - Roll 10d6, explode 6s, keep 8 highest, add 4
• `/roll 6 4d6` - Roll 6 sets of 4d6
• `/roll 4d100 ; 3d10 k2` - Multiple separate rolls

Type `/roll help alias` for game system shortcuts!"#.to_string()
}

fn generate_alias_help() -> String {
    r#"🎲 **Game System Aliases** 🎲

**World/Chronicles of Darkness:**
• `4cod` → 4d10 t8 ie10 (Chronicles of Darkness)
• `4cod8` → 4d10 t7 ie10 (8-again)
• `4wod8` → 4d10 f1 ie10 t8 (World of Darkness difficulty 8)

**D&D/Pathfinder:**
• `dndstats` → 6 4d6 k3 (ability score generation)
• `attack +5` → 1d20 +5
• `skill -2` → 1d20 -2
• `save +3` → 1d20 +3
• `+d20` → 2d20 k1 (advantage)
• `-d20` → 2d20 kl1 (disadvantage)

**Warhammer 40k Wrath & Glory:**
• `wng 4d6` → 4d6 with wrath die and success counting
• `wng dn3 5d6` → 5d6 with difficulty 3 test (shows PASS/FAIL)
• `wng 4d6 !soak` → 4d6 without wrath die

**Other Systems:**
• `3df` → 3d3 t3 f1 (Fudge dice)
• `3wh4+` → 3d6 t4 (Warhammer 40k/AoS)
• `sr6` → 6d6 t5 (Shadowrun)
• `ex5` → 5d10 t7 t10 (Exalted)
• `6yz` → 6d6 t6 (Year Zero)
• `age` → 2d6 + 1d6 (AGE system)
• `dd34` → 1d3*10 + 1d4 (double-digit d66 style)

**Special Systems:**
• `ed15` → Earthdawn step 15
• `2hsn` → Hero System normal damage

Use `/roll help system` for specific examples!"#.to_string()
}

fn generate_system_help() -> String {
    r#"🎲 **Game System Examples** 🎲

**Warhammer 40k Wrath & Glory:**
• `/roll wng 4d6` - 4d6 with wrath die
• `/roll wng dn2 4d6` - 4d6 with difficulty 2 test (shows PASS/FAIL)
• `/roll wng 4d6 !soak` - 4d6 without wrath die
• `/roll wng dn4 6d6 !exempt` - 6d6 difficulty 4 test without wrath die

**Dark Heresy 2nd Edition:**
• `/roll dh 4d10` - 4d10 with righteous fury on 10s

**Hero System:**
• `/roll 2hsn` - 2d6 normal damage
• `/roll 5hsk1 +1d3` - 5½d6 killing damage +1 stun
• `/roll 3hsh` - 3d6 healing (11 + 3 - 3d6)

**Earthdawn:**
• `/roll ed1` through `/roll ed50` - Step numbers 1-50

**Multiple Rolls:**
Maximum 4 separate rolls with semicolons:
• `/roll 4d6 ; 3d8 + 2 ; 1d20 ; 2d10 t7`

**Roll Sets:**
• `/roll 6 4d6` - Roll 6 sets of 4d6 (2-20 sets allowed)

Use `/help` for basic syntax!"#.to_string()
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
    let user_count: usize = ctx.cache.guilds()
        .iter()
        .map(|guild_id| {
            ctx.cache.guild(*guild_id)
                .map(|guild| guild.member_count as usize)
                .unwrap_or(0)
        })
        .sum();
    
    // Get database stats if available
    let db_stats = {
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
                Err(_) => "**Database Stats:** Error reading data\n".to_string()
            }
        } else {
            "**Database Stats:** Not available\n".to_string()
        }
    };
    
    let bot_info = format!(
        r#"🤖 **Dice Maiden Bot Info** 🤖

**Current Stats:**
• Servers: {}
• Estimated Users: {}
• Memory Usage: {:.2} MB

{}"#,
        server_count,
        user_count,
        memory_usage,
        db_stats
    );
    
    Ok(bot_info)
}
