// Shared help text module to eliminate duplication between commands
// This file should be placed at src/help_text.rs

pub fn generate_basic_help() -> String {
    r#"🎲 **Dice Maiden** 🎲

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
• If you experience a bug, please report the issue on GitHub!

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
• `gb` - Godbound damage chart (1-=0, 2-5=1, 6-9=2, 10+=4)
• `gbs` - Godbound straight damage (no chart)

**Special Flags:**
• `p` - Private roll (only you see results)
• `s` - Simple output (no dice breakdown)
• `nr` - No results shown (just total)
• `ul` - Unsorted dice results

**Examples:**
• `/roll 10d6 e6 k8 +4` - Roll 10d6, explode 6s, keep 8 highest, add 4
• `/roll 6 4d6` - Roll 6 sets of 4d6
• `/roll 4d100 ; 3d10 k2` - Multiple separate rolls

Type `/roll help alias` for game system shortcuts!"#
        .to_string()
}

pub fn generate_alias_help() -> String {
    r#"🎲 **Game System Aliases** 🎲\

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
• If you experience a bug, please report the issue on GitHub!

**Savage Worlds:**
• `sw8` → 1d8 ie8 + 1d6 ie6 k1 (d8 trait + d6 wild, keep highest)
• `sw10` → 1d10 ie10 + 1d6 ie6 k1 (d10 trait + d6 wild, keep highest)
• Snake Eyes: Critical failure when both dice roll natural 1

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
• `+d%` → Percentile advantage (roll-under systems)
• `-d%` → Percentile disadvantage (roll-under systems)

**Hero System 5th Edition:**
• `2hsn` → 2d6 hsn (normal damage)
• `3hsk` → 3d6 hsk (killing damage)
• `2.5hsk` → 2d6 + 1d3 hsk (2½ dice killing damage)
• `2hsk1` → 2d6 + 1d3 hsk (alternative fractional notation)
• `3hsh` → 3d6 hsh (to-hit roll)
• `hsn`, `hsk`, `hsh` → 1d6 hsn, 1d6 hsk, 3d6 hsh (single die versions)

**Godbound:**
• `gb` → 1d20 gb (basic d20 with damage chart)
• `gbs` → 1d20 gbs (basic d20 with straight damage)
• `gb 3d8` → 3d8 gb (3d8 with damage chart conversion)
• `gbs 2d10` → 2d10 gbs (2d10 straight damage)

**Warhammer 40k Wrath & Glory:**
• `wng 4d6` → 4d6 with wrath die and success counting
• `wng dn3 5d6` → 5d6 with difficulty 3 test (shows PASS/FAIL)
• `wng 4d6 !soak` → 4d6 without wrath die

**Other Systems:**
• `3df` → 3d3 fudge (Fudge dice showing +/blank/- symbols)
• `3wh4+` → 3d6 t4 (Warhammer 40k/AoS)
• `sr6` → 6d6 t5 (Shadowrun)
• `ex5` → 5d10 t7 t10 (Exalted)
• `6yz` → 6d6 t6 (Year Zero)
• `age` → 2d6 + 1d6 (AGE system)
• `dd34` → 1d3*10 + 1d4 (double-digit d66 style)

**Special Systems:**
• `ed15` → Earthdawn step 15
• `2hsn` → Hero System normal damage

Use `/roll help system` for specific examples!"#
        .to_string()
}

pub fn generate_system_help() -> String {
    r#"🎲 **Game System Examples** 🎲

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
• If you experience a bug, please report the issue on GitHub!

**Percentile Advantage/Disadvantage:**
• `/roll +d%` - Advantage (keeps lower tens die) for roll-under systems
• `/roll -d%` - Disadvantage (keeps higher tens die) for roll-under systems

**Fudge/FATE:**
• `/roll 3df` or `/roll 4df` - Fudge dice showing +/blank/- symbols
• Values: **+** = +1, (blank) = 0, **-** = -1

**Godbound:**
• `/roll gb` - d20 with damage chart (1-=0, 2-5=1, 6-9=2, 10+=4)
• `/roll gbs` - d20 straight damage (bypasses chart)
• `/roll gb 3d8` - Multi-die with chart conversion

**Hero System:**
• `/roll 2hsn` - 2d6 normal damage
• `/roll 3hsk` - 3d6 killing damage (BODY + STUN = BODY × 1d3)
• `/roll 2.5hsk` - 2½d6 killing (2d6 + 1d3)
• `/roll 3hsh` - 3d6 to-hit (target: 11 + OCV - DCV)

**Wrath & Glory:**
• `/roll wng 4d6` - Standard roll with wrath die
• `/roll wng dn2 4d6` - Difficulty 2 test (shows PASS/FAIL)
• `/roll wng 4d6 !soak` - Damage/soak roll (no wrath die)

**Other Systems:**
• `/roll dh 4d10` - Dark Heresy (righteous fury on 10s)
• `/roll ed15` - Earthdawn step 15 (steps 1-50 available)

**Multiple Rolls:**
• `/roll 4d6 ; 3d8 + 2 ; 1d20` - Up to 4 separate rolls
• `/roll 6 4d6` - Roll 6 sets of 4d6 (2-20 sets allowed)

Use `/help` for basic syntax and `/help alias` for more shortcuts!"#
        .to_string()
}
