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
• `km2` - Keep middle 2 dice
• `rg3` - Reroll dice ≥ 3
• `irg3` - Reroll ≥ 3 indefinitely
• `r2` - Reroll dice ≤ 2 once
• `ir2` - Reroll dice ≤ 2 indefinitely
• `t7` - Count successes (≥ 7)
• `t4ds6` - Count successes (≥ 4) and double success on 6 (defaults to target)
• `tl6` - Count successes (≤ 6)
• `tl6ds4` - Count successes (≤ 6) and double success on 4 (defaults to target)
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

Type `/roll help alias` for game system shortcuts!"#
        .to_string()
}

pub fn generate_alias_help() -> String {
    r#"🎲 **Game System Aliases** 🎲

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
• If you experience a bug, please report the issue on GitHub!

**Savage Worlds:**
• `sw8` → 1d8 ie8 + 1d6 ie6 k1 (d8 trait + d6 wild, keep highest)
• `sw10` → 1d10 ie10 + 1d6 ie6 k1 (d10 trait + d6 wild, keep highest)

**World/Chronicles of Darkness:**
• `4cod` → 4d10 t8 ie10 (Chronicles of Darkness standard)
• `4codr` → 4d10 t8 ie10 r7 (rote quality: reroll failures)
• `4wod8` → 4d10 f1 t8 (World of Darkness difficulty 8)
• `4wod8c` → 4d10 f1 t8 c (10s cancel 1s)

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
• `3hsh` → 3d6 hsh (to-hit roll)

**Godbound:**
• `gb` → 1d20 gb (basic d20 with damage chart)
• `gbs` → 1d20 gbs (basic d20 with straight damage)
• `gb 3d8` → 3d8 gb (3d8 with damage chart conversion)
• `gbs 2d10` → 2d10 gbs (2d10 straight damage)

**Other Systems:**
• `3df` → 3d3 fudge (Fudge dice showing +/blank/- symbols)
• `3wh4+` → 3d6 t4 (Warhammer 40k/AoS)
• `sr6` → 6d6 t5 (Shadowrun)
• `sp4` → 4d10 t8 ie10 (Storypath)
• `sp4t6` → 4d10 t6 ie10 (Storypath target change)
• `ex5` → 5d10 t7 t10 (Exalted)
• `6yz` → 6d6 t6 (Year Zero)
• `age` → 2d6 + 1d6 (AGE system)
• `dd34` → 1d3*10 + 1d4 (double-digit d66 style)
• `ed15` → Earthdawn step 15
• `cs 3` → Cypher System 1d20 cs3 (Level 3 task, target 9+)
• `cpr` → Cyberpunk Red 1d10 cpr (critical success on 10, critical failure on 1)
• `conan3` → 3d20 conan (3d20 skill roll)
• `sil#` → Silhouette system: roll #d6, keep highest, extra 6s add +1 (e.g., sil3, sil5)
• `dp` → d6 plot (Plot dice showing OPPORTUNITY/blank/COMPLICATION(+2/4) symbols)

Use `/roll help system` for specific examples!"#
        .to_string()
}

pub fn generate_system_help() -> String {
    r#"🎲 **Game System Examples** 🎲

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`

**Percentile Advantage/Disadvantage:**
• `/roll +d%` - Advantage (keeps lower tens die) for roll-under systems
• `/roll -d%` - Disadvantage (keeps higher tens die) for roll-under systems

**Fudge/FATE:**
• `/roll 3df` or `/roll 4df` - Fudge dice showing +/blank/- symbols
• Values: **+** = +1, (blank) = 0, **-** = -1

**D6 Legends System:**
• Regular dice: Count successes on 4-6
• Wild die: Counts successes on 4-6, explodes on 6, failures (1) subtract 1 success
• `/roll 8d6l` → 7 regular dice + 1 wild die

**Godbound:**
• `/roll gb` - d20 with damage chart (1-=0, 2-5=1, 6-9=2, 10+=4)
• `/roll gbs` - d20 straight damage (bypasses chart)
• `/roll gb 3d8` - Multi-die with chart conversion

**Hero System:**
• `/roll 2hsn` - 2d6 normal damage
• `/roll 3hsk` - 3d6 killing damage (BODY + STUN = BODY × 1d3)
• `/roll 3hsh` - 3d6 to-hit (target: 11 + OCV - DCV)

**Wrath & Glory:**
• `/roll wng 4d6` - Standard roll with wrath die
• `/roll wng w2 4d6` - Standard roll with 2 wrath dice
• `/roll wng dn2 4d6` - Difficulty 2 test (shows PASS/FAIL)
• `/roll wng 4d6 !soak` - Damage/soak roll (no wrath die)

**Marvel Multiverse:**
• `/roll mm` - Basic 3d6 roll (Marvel die in middle)
• `/roll mm 2e` - 3d6 with 2 edges
• `/roll mm 3t` - 3d6 with 3 troubles

**Witcher d10 System:**
• `wit` → 1d10 wit (basic Witcher skill check)
• `wit + 5` → 1d10 wit with +5 modifier

**Brave New World**
• `bnw3` → 3d6 pool, take highest die, 6s explode into new results
• `bnw5 + 2` → 5-die pool with +2 modifier (applied after taking highest)

**Plotweaver/Cosmere RPG:**
• `/roll dp` or `/roll 1dp` - Plot dice showing OPPORTUNITY/blank/COMPLICATION(+2/4) symbols
• Values: **OPPORTUNITY** = 0, (blank) = 0, **COMPLICATION (+2)** = 2

**Other Systems:**
• `/roll dh 4d10` - Dark Heresy (righteous fury on 10s)
• `/roll fitd4` - Forged in the Dark 4d6 action roll

**Multiple Rolls:**
• `/roll 4d6 ; 3d8 + 2 ; 1d20` - Up to 4 separate rolls
• `/roll 6 4d6` - Roll 6 sets of 4d6 (2-20 sets allowed)

Use `/help` for basic syntax and `/help alias` for more shortcuts!"#
        .to_string()
}

pub fn generate_a5e_help() -> String {
    r#"🎲 **Level Up: Advanced 5th Edition (A5E) System** 🎲

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
• If you experience a bug, please report the issue on GitHub!

A5E uses expertise dice that add to d20 rolls. Multiple expertise sources don't stack as additional dice, but increase the die size:

• **1 source**: +1d4 expertise die
• **2 sources**: +1d6 expertise die  
• **3+ sources**: +1d8 expertise die (maximum)

**Concise A5E Syntax (assumes d20):**
• `a5e +5 ex1` → 1d20+5 + 1d4 (attack +5 with expertise level 1)
• `a5e ex2` → 1d20 + 1d6 (no modifier, expertise level 2)
• `a5e -2 ex3` → 1d20-2 + 1d8 (penalty -2, expertise level 3)

**Expertise Levels:**
• `ex1` = 1d4 (one expertise source)
• `ex2` = 1d6 (two expertise sources)  
• `ex3` = 1d8 (three or more sources)

**Explicit Dice Sizes:**
• `ex4`, `ex6`, `ex8` (standard)
• `ex10`, `ex12`, `ex20`, `ex100` (house rules)

**Advantage/Disadvantage (only d20 rolled twice):**
• `+a5e +5 ex1` → 2d20 kh1+5 + 1d4 (advantage + expertise)
• `-a5e +5 ex1` → 2d20 kl1+5 + 1d4 (disadvantage + expertise)
• `+a5e ex2` → 2d20 kh1 + 1d6 (advantage, no modifier)

**Common Usage Examples:**
• `a5e +7 ex1` - Attack roll with proficiency bonus and one expertise source
• `+a5e +3 ex2` - Advantage on ability check with two expertise sources  
• `-a5e +5 ex3` - Disadvantage on saving throw with maximum expertise
• `a5e +12 ex6` - High-level attack with explicit d6 expertise die

Use `/help` for basic syntax and `/help alias` for more shortcuts!"#
        .to_string()
}

pub fn generate_aliens_help() -> String {
    r#"🎲 **Alien RPG (Year Zero Engine) System** 🎲

**Note:**
• Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
• If you experience a bug, please report the issue on GitHub!

The Alien RPG uses the Year Zero Engine with **Base Dice** (safe) and **Stress Dice** (dangerous but powerful).

**Basic Syntax:**
• `alien4` → 4 base dice (attribute + skill roll)
• `alien5s2` → 5 base dice + 2 stress dice
• `alien3s1p` → Push roll (increases stress by 1)

**Base Dice (Safe):**
• Roll d6s equal to **Attribute + Skill**
• Count 6s as successes - no negative effects

**Stress Dice (Powerful but Dangerous):**
• Add extra d6s to your roll for more successes
• 6s = successes (just like base dice)
• 1s = **PANIC RISK** - triggers automatic panic roll
• Stress level ranges from 1-10

**Panic System:**
When stress dice show **1s**, you must make a panic roll:
• Panic Roll = `1d6 + Current Stress Level`
• Higher stress = worse panic effects

**Panic Table Results:**
• 1-6: Keeping it together (no effect)
• 7: Tremble - Shaky hands (-2 to next roll)
• 8: Drop Item - You drop a weapon or important item
• 9: Freeze - You lose your next turn
• 10: Seek Cover - You must move to safety immediately
• 11: Scream - Everyone who hears you must make a Panic Roll
• 12: Flee - You must move away from the threat
• 13: Berserk - You attack the nearest person or creature
• 14: Catatonic - You become unresponsive for one turn
• 15+: Heart Attack - You suffer a heart attack and become Broken

**Push Mechanics:**
• Add 'p' to stress aliases to push: `alien4s2p` becomes `alien4s3`
• Cannot push if you rolled any 1s on stress dice
• Pushing adds +1 to your stress level
• Risk vs. reward - more successes but higher panic risk

**Stress Level Guidelines:**
• 1-3: Low stress, manageable risk
• 4-6: Moderate stress, noticeable panic effects
• 7-10: High stress, severe consequences likely

Use `/help` for basic syntax and `/help alias` for more shortcuts!"#
        .to_string()
}

pub fn generate_mothership_help() -> String {
    r#"🎲 **Mothership RPG System** 🎲

**Note:**
- Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
- If you experience a bug, please report the issue on GitHub!

Mothership uses a percentile (d100) roll-under system where rolling doubles (11, 22, 33, etc.) results in critical successes or critical failures. The advantage/disadvantage system uses sophisticated selection logic.

**Basic Rolls:**
- `ms` → 1d100 roll-under (default target 50)
- `ms45` → 1d100 roll-under with Strength stat 45
- `ms30` → 1d100 roll-under with Speed stat 30

**Advantage/Disadvantage:**
- `+ms45` → Roll 2d100, select better result using Mothership logic
- `-ms45` → Roll 2d100, select worse result using Mothership logic
- `+ms` → Advantage with default target 50
- `-ms` → Disadvantage with default target 50

**Critical System:**
Doubles (11, 22, 33, ..., 99, 00) are critical rolls:
- If you succeed (roll ≤ stat), doubles = **Critical Success**
- If you fail (roll > stat), doubles = **Critical Failure**

**Selection Logic for Advantage:**
When rolling with advantage, the better roll is selected by priority:
1. **Critical Success** (doubles ≤ stat) - BEST
2. **Success** (non-doubles ≤ stat)
3. **Critical Failure** (doubles > stat)
4. **Failure** (non-doubles > stat) - WORST
- Within same category: prefer lower roll

**Selection Logic for Disadvantage:**
When rolling with disadvantage, the worse roll is selected by priority:
1. **Failure** (non-doubles > stat) - WORST
2. **Critical Failure** (doubles > stat)
3. **Success** (non-doubles ≤ stat)
4. **Critical Success** (doubles ≤ stat) - BEST
- Within same category: prefer higher roll

Use `/help` for basic syntax and `/help alias` for more game systems!"#
        .to_string()
}
