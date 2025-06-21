use anyhow::Result;
use serenity::{
    builder::{CreateCommand, CreateCommandOption},
    all::{CommandInteraction, CommandOptionType, CommandDataOptionValue},
    prelude::Context,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("help")
        .description("Show help information for Dice Maiden")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "topic",
                "Help topic (basic, alias, system)"
            )
            .required(false)
            .add_string_choice("basic", "basic")
            .add_string_choice("alias", "alias")
            .add_string_choice("system", "system")
        )
}

pub async fn run(
    _ctx: &Context,
    command: &CommandInteraction,
) -> Result<String> {
    let topic = command.data.options.first()
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::String(s) => Some(s.as_str()),
            _ => None,
        })
        .unwrap_or("basic");

    match topic {
        "alias" => Ok(generate_alias_help()),
        "system" => Ok(generate_system_help()),
        _ => Ok(generate_basic_help()),
    }
}

fn generate_basic_help() -> String {
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

Use `/help topic:alias` for game system shortcuts!"#.to_string()
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

Use `/help topic:system` for specific examples!"#.to_string()
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
