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

**Percentile Advantage/Disadvantage:**
• `/roll +d%` - Percentile advantage for roll-under systems (Call of Cthulhu, etc.)
• `/roll -d%` - Percentile disadvantage for roll-under systems

**Percentile System Details:**
For roll-under systems (like Call of Cthulhu), percentile advantage/disadvantage works by:
• **Advantage (+d%)**: Roll 2d10 for tens digit, keep lowest + 1d10 for ones
• **Disadvantage (-d%)**: Roll 2d10 for tens digit, keep highest + 1d10 for ones
• Lower results are better in roll-under systems, so advantage keeps the lower tens die

**Fudge/FATE Dice:**
• `/roll 3df` - 3 Fudge dice showing symbols: + (plus), (blank), - (minus)
• `/roll 4df` - Standard FATE roll (4 Fudge dice)
• `/roll 4d3 fudge` - Alternative syntax for Fudge dice

**Fudge Dice Values:**
Each Fudge die shows a symbol representing:
• **+** (plus) = +1 to total
• ` `(blank) = 0 to total  
• **-** (minus) = -1 to total

**Godbound:**
• `/roll gb` - Basic d20 with damage chart (1-=0, 2-5=1, 6-9=2, 10+=4)
• `/roll gbs` - Basic d20 with straight damage (no chart)
• `/roll gb 3d8` - 3d8 with damage chart conversion
• `/roll gbs 2d10 +5` - 2d10 straight damage +5
• `/roll 5d6 gb` - 5d6 using damage chart
• `/roll 2d12 gbs` - 2d12 straight damage

**Damage Chart Conversion:**
The Godbound damage chart converts each die individually:
• 1 or less → 0 damage
• 2-5 → 1 damage  
• 6-9 → 2 damage
• 10+ → 4 damage

Use `gbs` for "straight damage" that bypasses the chart.

**Hero System:**
• `/roll 2hsn` - 2d6 normal damage
• `/roll 3hsk` - 3d6 killing damage (shows BODY and STUN)
• `/roll 2.5hsk` - 2d6 + 1d3 killing damage with fractional dice
• `/roll 3hsh` - 3d6 to-hit roll (roll-under mechanics)

**Hero System Damage:**
• **Normal damage (hsn)**: Standard dice total
• **Killing damage (hsk)**: BODY = dice total, STUN = BODY × 1d3 multiplier
• **To-hit (hsh)**: 3d6 roll-under (target: 11 + OCV - DCV)

**Warhammer 40k Wrath & Glory:**
• `/roll wng 4d6` - 4d6 with wrath die
• `/roll wng dn2 4d6` - 4d6 with difficulty 2 test (shows PASS/FAIL)
• `/roll wng 4d6 !soak` - 4d6 without wrath die
• `/roll wng dn4 6d6 !exempt` - 6d6 difficulty 4 test without wrath die

**Dark Heresy 2nd Edition:**
• `/roll dh 4d10` - 4d10 with righteous fury on 10s

**Earthdawn:**
• `/roll ed1` through `/roll ed50` - Step numbers 1-50

**Multiple Rolls:**
Maximum 4 separate rolls with semicolons:
• `/roll 4d6 ; 3d8 + 2 ; 1d20 ; 2d10 t7`

**Roll Sets:**
• `/roll 6 4d6` - Roll 6 sets of 4d6 (2-20 sets allowed)

Use `/help` for basic syntax!"#
        .to_string()
}
