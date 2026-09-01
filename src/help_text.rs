//! Static Discord-formatted help strings for the `/help` command.
//! Kept in a separate module so the text can be shared without duplication
//! if additional consumers are added (e.g. a web dashboard).
//!
//! # Adding a help topic
//!
//! Add the `generate_*_help` function, then add its name to [`HELP_TOPICS`] and
//! a match arm to [`generate_topic_help`].  Both `/help <topic>` and
//! `/roll help <topic>` dispatch through those two, and the slash-command
//! choices are built from `HELP_TOPICS`, so a topic added there shows up on
//! every surface at once — there is no second list to keep in sync.

/// Every valid `/help` topic, in the order they are offered as slash-command
/// choices.  Discord allows at most 25 choices per option.
pub const HELP_TOPICS: &[&str] = &[
    "basic",
    "alias",
    "system",
    "a5e",
    "aliens",
    "mothership",
    "ol",
    "ess",
    "tdh",
    "cpr",
    "wfrp",
];

/// Resolve a topic name to its help text, or `None` if the topic is unknown.
///
/// Callers decide what an unknown topic means: `/help` falls back to the basic
/// page, while `/roll help <topic>` lets it fall through to dice parsing so a
/// roll that merely starts with "help" still reaches the parser.
pub fn generate_topic_help(topic: &str) -> Option<String> {
    Some(match topic {
        "basic" => generate_basic_help(),
        "alias" => generate_alias_help(),
        "system" => generate_system_help(),
        "a5e" => generate_a5e_help(),
        "aliens" => generate_aliens_help(),
        "mothership" => generate_mothership_help(),
        "ol" => generate_open_legend_help(),
        "ess" => generate_essence20_help(),
        "tdh" => generate_darkest_house_help(),
        "cpr" => generate_cyberpunk_red_help(),
        "wfrp" => generate_wfrp_help(),
        _ => return None,
    })
}

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
• `i1` or `i` - Implode on 1s (or lower): subtract an extra die from the total
• `d2` - Drop lowest 2 dice
• `k3` - Keep highest 3 dice
• `kl2` - Keep lowest 2 dice
• `km2` - Keep middle 2 dice
• `adv1` - Advantage 1: roll 1 extra die, drop the lowest, then explode
• `dis1` - Disadvantage 1: roll 1 extra die, drop the highest, then explode
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
• `sw10` → 1d10 ie10 + 1d6 ie6 k1 (d10 trait + wild die)

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
• `+d%` / `-d%` → Percentile advantage / disadvantage (roll-under)

**Hero System 5th Edition:**
• `2hsn` → 2d6 hsn (normal damage)
• `3hsk` → 3d6 hsk (killing damage)
• `3hsh` → 3d6 hsh (to-hit roll)

**Godbound:**
• `gb` → 1d20 gb (d20 with damage chart)
• `gbs` → 1d20 gbs (d20 with straight damage)
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
• `cpr` / `cpd3` → Cyberpunk Red (`/help cpr`)
• `wfrp67` → WFRP 4e (`/help wfrp`)
• `conan tn12f3` → Conan (target 12, Focus 3)
• `sil3` → Silhouette (`/help system`)
• `ol5` → Open Legend (`/help ol`)
• `ess1d8` → Essence20 (`/help ess`)
• `tdh4` → The Darkest House (`/help tdh`)

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
• `/roll 3hsk` - 3d6 killing damage (BODY, and STUN = BODY × 1d6-1)
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

**Panic Table Results:**
• 1-6: Keeping it together (no effect)
• 7: Nervous Twitch - Stress +1 for you and nearby friends
• 8: Tremble - AGILITY rolls suffer -2 while panicked
• 9: Drop Item - Drop a weapon or key item, Stress +1
• 10: Freeze - Lose next slow action, Stress +1 nearby
• 11: Seek Cover - Next action must move you to safety
• 12: Scream - Lose a slow action, Stress -1, others panic
• 13: Flee - Flee to safety, Stress -1, witnesses panic
• 14: Berserk - Attack the nearest character, witnesses panic
• 15+: Catatonic - You collapse, unable to talk or move

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

pub fn generate_open_legend_help() -> String {
    r#"🎲 **Open Legend RPG** 🎲

**Note:**
- Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
- If you experience a bug, please report the issue on GitHub!

An action roll is 1d20 plus your attribute dice. Everything explodes on its maximum value.

**Basic Rolls:**
- `ol5` → 1d20 ie20 + 2d6 ie6 (attribute score 5)
- `ol0` → 1d20 ie20 (attribute score 0, no attribute dice)
- `3 ol5` → Roll 3 separate action rolls

**Attribute Dice:**
- 1 → 1d4 | 2 → 1d6 | 3 → 1d8 | 4 → 1d10 | 5 → 2d6
- 6 → 2d8 | 7 → 2d10 | 8 → 3d8 | 9 → 3d10 | 10 → 4d8

**Advantage / Disadvantage:**
- `ol5a2` → Advantage 2 (roll 2 extra attribute dice, drop the 2 lowest)
- `ol5d1` → Disadvantage 1 (roll 1 extra attribute die, drop the highest)
- The drop happens **before** exploding, so exploded dice are never dropped
- A die that survives the drop still explodes normally
- Advantage and disadvantage cancel out: `a2` with `d1` nets to advantage 1
- With attribute score 0 the d20 itself is rolled twice, capped at level 1

**Generic Modifiers:**
`adv#` and `dis#` work on any roll, not just Open Legend:
- `2d6 ie6 adv1` → Roll 3d6, drop the lowest, then explode what remains
- `4d10 ie10 dis2` → Roll 6d10, drop the 2 highest, then explode what remains
- Contrast with `d1`/`k3`, which are applied **after** exploding

Use `/help` for basic syntax and `/help alias` for more shortcuts!"#
        .to_string()
}

pub fn generate_wfrp_help() -> String {
    r#"🎲 **Warhammer Fantasy Roleplay 4e** 🎲

**Note:**
- Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
- If you experience a bug, please report the issue on GitHub!

Roll d100 against a Characteristic or Skill and succeed on equal or lower.

**Tests:**
- `wfrp67` → test a Characteristic or Skill of 67
- `wfrp67 + 20` → Easy test: the modifier adjusts the target, giving 87
- `wfrp67 - 30` → Very Hard test, target 37
- `3 wfrp45` → three separate tests

**Success Levels:**
- SL is the tens digit of the target minus the tens digit of the roll
- Target 67, roll 22 → +4 SL; roll 88 → -2 SL
- The result shown is the SL; the note carries the verdict
- `+0` scraped a success, `-0` missed on the ones digit alone

**Automatic Results:**
- 01-05 always succeeds, scoring at least +1 SL
- 96-00 always fails, scoring at most -1 SL
- The die is rolled 1-100, with 100 standing in for 00

**Doubles:**
- 11, 22 ... 99, 00 are an Astounding Success or Astounding Failure
- In combat that is a Critical Hit or a Fumble
- Independent of SL: both apply to the same roll"#
        .to_string()
}

pub fn generate_cyberpunk_red_help() -> String {
    r#"🎲 **Cyberpunk Red** 🎲

**Note:**
- Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
- If you experience a bug, please report the issue on GitHub!

**Skill Checks (1d10 + STAT + SKILL):**
- `cpr` → 1d10 with Critical Success and Critical Failure
- `cpr + 5` → add your STAT + SKILL total
- Critical Success (10): roll another d10 and add it
- Critical Failure (1): roll another d10 and subtract it
- Neither chains: a second 10 or 1 is just a number
- The same die built by hand is `1d10 i1 e10`

**Damage (Nd6):**
- `cpd3` → 3d6 damage, totalled straight
- `cpd4 + 2` → damage with a modifier
- `cpd2 * 3` → autofire: 2d6 multiplied by 3
- `cpd3 * 2` → aimed head shot: damage doubled
- `5 cpd6` → area attack: one roll per target

**Critical Injuries:**
- Two or more 6s on the damage dice inflict a Critical Injury
- It lands even if no damage got through the target's armor SP
- 2d6 is rolled for you: look it up on the Body table, or the Head table
  if you took an Aimed Shot at the head
- The 5 bonus damage goes direct to Hit Points, so it is NOT in the total:
  it ignores armor SP and hit location, while the total is what SP comes off"#
        .to_string()
}

pub fn generate_darkest_house_help() -> String {
    r#"🎲 **The Darkest House (Monte Cook Games)** 🎲

**Note:**
- Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
- If you experience a bug, please report the issue on GitHub!

Everything is 2d6 plus your Rating, aiming for 7 plus the Rating of the task or opponent.

**Basic Rolls:**
- `tdh4` → 2d6 + Rating 4, plus the House Die
- `tdh4 +1` → any flat modifier folds into the Rating
- `3 tdh4` → three separate checks

**Boons and Banes:**
- `tdh4b` → Boon: roll 3d6 and discard the lowest
- `tdh4n` → Bane: roll 3d6 and discard the highest
- Boons and Banes cancel each other out, and never add more than one die

**The House Die:**
- Rolled with every action, but never with damage
- It does not affect success or failure
- If it is higher than the dice used for the action, 🏚️ the house acts
- A tie is not higher, so the house waits
- With a Boon or Bane the discarded die does not count

**Calling Upon the House (add `c`):**
- `tdh4c` → the House Die is added to your result
- The house then acts automatically and you gain 1 Doom
- Combines with Boons and Banes: `tdh4bc`

**Damage (no House Die):**
- Wound Rating = 1d6 + attack Rating - defense Rating → `1d6 + 4 - 2`
- Boon or Bane on damage: `1d6 adv1 + 4 - 2` / `1d6 dis1 + 4 - 2`
- A result of 0 or less is a scratch with no mechanical effect

Use `/help` for basic syntax and `/help alias` for more game systems!"#
        .to_string()
}

pub fn generate_essence20_help() -> String {
    r#"🎲 **Essence20 (Renegade Game Studios)** 🎲

**Note:**
- Additional support can be found on GitHub `https://github.com/Humblemonk/dicemaiden-rs`
- If you experience a bug, please report the issue on GitHub!

A skill test is 1d20 plus your skill die. Used by the Power Rangers, G.I. Joe and Transformers RPGs.

**Basic Rolls:**
- `ess1d8` → 1d20 + 1d8 (d8 skill die)
- `essd8` → same thing, the dice count is optional
- `ess1d8 + 3` → skill test with a flat +3
- `3 ess1d8` → three separate skill tests

**Skill Die Ladder:**
- d2 → d4 → d6 → d8 → d10 → d12 → 2d8 → 3d6

**Specializations (add `s`):**
- `ess1d8s` → rolls d8, d6, d4 and d2, adding only the highest to the d20
- The dice that lost are shown struck through
- 2d8 and 3d6 count as a single result each (their dice are summed)

**Edge and Snag:**
- `+ess1d8` → Edge: roll 2d20, keep the higher
- `-ess1d8` → Snag: roll 2d20, keep the lower
- Both combine with specializations: `+ess1d8s`

**Critical Success:**
- Any skill die showing its maximum is a critical success, even if a bigger die
  was the one added — a d4 showing 4 crits while the d8 supplies the total
- Every skill die counts, down to the d2 on a 2; 2d8 and 3d6 crit on 16 and 18
- The d20 is not a skill die, so a natural 20 is reported separately

Use `/help` for basic syntax and `/help alias` for more game systems!"#
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
3. **Failure** (non-doubles > stat)
4. **Critical Failure** (doubles > stat) - WORST
- Within same category: prefer lower roll

**Selection Logic for Disadvantage:**
When rolling with disadvantage, the worse roll is selected by priority:
1. **Critical Failure** (doubles > stat) - WORST
2. **Failure** (non-doubles > stat)
3. **Success** (non-doubles ≤ stat)
4. **Critical Success** (doubles ≤ stat) - BEST
- Within same category: prefer higher roll

Use `/help` for basic syntax and `/help alias` for more game systems!"#
        .to_string()
}
