//! Core domain types and the top-level `parse_and_roll` convenience function.
//!
//! Everything in the dice engine revolves around three types:
//!
//! * [`DiceRoll`] — a fully-parsed dice expression ready to be executed.
//!   Produced by `parser::parse_dice_string`.
//! * [`RollResult`] — the outcome of executing one `DiceRoll`.
//!   Produced by `roller::roll_dice`.
//! * [`Modifier`] — an enum variant attached to a `DiceRoll` that changes how
//!   the roll is resolved (exploding, keep/drop, success counting, game-system
//!   specific logic, …).
//!
//! [`DiceGroup`] records the individual dice that belong to one "roll event"
//! (e.g. the base dice vs. an exploded pool) and is used by the formatter to
//! produce the per-dice breakdown shown in Discord messages.
//!
//! # Adding a new game system
//!
//! Follow this sequence — every step is required:
//!
//! 1. `aliases.rs`            — add alias expansion
//! 2. **this file**           — add a `Modifier` variant
//! 3. `parser.rs`             — add parsing in `split_combined_modifiers` and
//!    `parse_modifiers_from_part`
//! 4. `roller.rs`             — add roll execution logic
//! 5. `commands/roll.rs`      — add display / formatting logic
//! 6. `game_systems_tests.rs` — add table-driven tests
//! 7. `roll_syntax.md`        — document the new syntax
//!
//! # Modifier ordering invariant
//!
//! **Drop before explode.**  [`Modifier::Advantage`] / [`Modifier::Disadvantage`]
//! discard their surplus dice before any explosion runs, and a discarded die is
//! never reconsidered for explosion.  Open Legend requires this.
//!
//! `d#`/`k#` run the other way round: they are applied *after* explosions, so
//! exploded dice can themselves be dropped.  Both orderings are intentional and
//! covered by tests — do not change or merge them.

pub mod aliases;
pub mod parser;
pub mod rng;
pub mod roller;

use anyhow::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

pub use rng::{create_enhanced_rng, create_fast_rng, create_seeded_rng, get_dice_rng};

#[derive(Debug, Clone)]
pub struct DiceRoll {
    pub count: u32,
    pub sides: u32,
    pub modifiers: Vec<Modifier>,
    pub comment: Option<String>,
    pub label: Option<String>,
    pub private: bool,
    pub simple: bool,
    pub no_results: bool,
    pub unsorted: bool,
    pub original_expression: Option<String>, // The expression as the user typed it
}

#[derive(Debug, Clone)]
pub enum HeroSystemType {
    Normal,  // hsn - normal damage
    Killing, // hsk - killing damage
    Hit,     // hsh - to hit roll (3d6 roll-under)
}

/// Essence20 skill-die ranks, lowest to highest, as `(count, sides)`.
///
/// A skill test is `1d20` plus the character's rank.  With a *specialization*
/// the character rolls their own rank **and every rank below it**, keeping the
/// single highest result (Renegade Game Studios, Essence20 core rules).
pub const ESSENCE20_RANKS: [(u32, u32); 8] = [
    (1, 2),
    (1, 4),
    (1, 6),
    (1, 8),
    (1, 10),
    (1, 12),
    (2, 8),
    (3, 6),
];

/// 1-based rank number of `count`d`sides` in [`ESSENCE20_RANKS`], or `None` if
/// it is not a legal Essence20 skill die.
pub fn essence20_rank_number(count: u32, sides: u32) -> Option<u32> {
    ESSENCE20_RANKS
        .iter()
        .position(|&rank| rank == (count, sides))
        .map(|index| index as u32 + 1)
}

#[derive(Debug, Clone, PartialEq)]
pub enum LaserFeelingsType {
    Lasers,   // Roll <= target for success
    Feelings, // Roll >= target for success
}

impl fmt::Display for LaserFeelingsType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaserFeelingsType::Lasers => write!(f, "Lasers"),
            LaserFeelingsType::Feelings => write!(f, "Feelings"),
        }
    }
}

/// Highest effective WFRP target a difficulty modifier can produce. A base
/// Characteristic or Skill caps at 100; Easy (+20) and Very Easy (+40) tests
/// push past it, and beyond this everything short of an automatic failure
/// succeeds anyway.
pub const WFRP_MAX_TARGET: u32 = 200;

/// The highest stress level the Alien aliases track: the character sheet's ten
/// boxes. No rule found says what a push at this level does, so the bot
/// refuses it plainly rather than claim one (aliases::alien_push_refusal).
pub(crate) const ALIEN_MAX_STRESS: u32 = 10;

/// Highest Mothership target. A relevant Skill adds its bonus to the Stat
/// (Player's Survival Guide §5: Trained +10%, Expert +15%, Master +20%), so a
/// target can pass 99; every roll but 99 then succeeds.
pub const MOTHERSHIP_MAX_TARGET: u32 = 200;

/// Conan caps Expertise and Focus at 5, so a Focus above this is a typo rather
/// than a legal rating.
pub const CONAN_MAX_FOCUS: u32 = 5;

/// A Conan test uses at most 5d20, but assistance dice explicitly do not count
/// toward that limit and are capped at +4d20, so nine is the largest pool the
/// rules can produce.
pub const CONAN_MAX_POOL: u32 = 9;

/// The largest pool any roll may draw: written dice, advantage extras, a dice
/// operand, a `d6s` pool, Marvel edges or troubles. Each die or reroll is a step
/// of a loop on a shard worker, so an unbounded count hangs every shard in the
/// process.
pub(crate) const MAX_DICE: u32 = 500;

#[derive(Debug, Clone)]
pub enum Modifier {
    Add(i32),
    Subtract(i32),
    Multiply(i32),
    Divide(i32),
    /// `N/XdY`: N divided by the roll total. Always the first modifier.
    NumberDividedByRoll(i32),
    Explode(Option<u32>),                   // e or e#
    ExplodeIndefinite(Option<u32>),         // ie or ie#
    Implode(Option<u32>),                   // i or i#
    Drop(u32),                              // d#
    Advantage(u32),                         // adv# - drop # lowest before exploding
    Disadvantage(u32),                      // dis# - drop # highest before exploding
    KeepHigh(u32),                          // k#
    KeepLow(u32),                           // kl#
    KeepMiddle(u32),                        // km#
    Reroll(u32),                            // r#
    RerollIndefinite(u32),                  // ir#
    Target(u32),                            // t#  - count successes >= target
    RerollGreater(u32),                     // rg# - reroll >= threshold once
    RerollGreaterIndefinite(u32),           // irg# - reroll >= threshold indefinitely
    RerollExact(u32),                       // rx# - reroll a die showing exactly # once
    RerollExactIndefinite(u32),             // irx# - reroll a die until it stops showing #
    TargetLower(u32),                       // tl# - count successes <= target
    TargetWithDoubleSuccess(u32, u32),      // t{target}ds{double}
    TargetLowerWithDoubleSuccess(u32, u32), // tl{target}ds{double}
    Failure(u32),                           // f#
    Botch(Option<u32>),                     // b or b#
    Cancel,                                 // c
    AddDice(DiceRoll),                      // Additional dice
    SubtractDice(DiceRoll),                 // Subtract dice result
    MultiplyDice(DiceRoll),
    DivideDice(DiceRoll),
    WrathGlory(Option<u32>, bool, u32), // Wrath & Glory: (difficulty, use_total_instead_of_successes, wrath_dice)
    Godbound(bool),                     // gb (false) or gbs (true for straight damage)
    HeroSystem(HeroSystemType),         // Hero System damage/hit calculations
    Fudge,                              // df - Fudge dice with symbol display
    DarkHeresy,
    SavageWorlds(u32),
    D6System(u32),
    Shadowrun(u32),
    MarvelMultiverse(i32, i32), // (edges, troubles) - already calculated net values
    CyberpunkRed,
    CyberpunkRedDamage,      // cpd - Nd6 damage, 2+ sixes = Critical Injury
    Wfrp(u32),               // wfrp# - Warhammer Fantasy 4e roll-under with Success Levels
    BrokenEmpires(u32, u32), // tbe#e# - The Broken Empires (skill, Expertise; 0 = none)
    BrokenEmpiresFavor(u32), // favor# - spend 1-3 Favor for +10 skill per point
    Witcher,
    CypherSystem(u32),
    BraveNewWorld(u32),
    ConanSkill(u32),  // conan, conan1-conan9 - d20 skill rolls
    ConanCombat(u32), // cd, cd4, cd5 - combat dice interpretation
    /// `tn#`, `tn#f#`, `tn#f#c#` - Conan / 2d20 skill test. Each d20 at or under
    /// `target` (Attribute + Expertise) scores one success, at or under `focus`
    /// scores two, and at or over `complication` generates a Complication.
    ///
    /// `focus` is 0 when the user omits `f#`, which means no criticals. That is
    /// correct for Conan: unlike Star Trek Adventures, a natural 1 carries no
    /// special rule here and only ever crits by being at or under the Focus.
    TwoD20Test {
        target: u32,
        focus: u32,
        complication: u32,
    },
    Silhouette(u32),
    VampireMasquerade5(u32, u32),
    LaserFeelings(u32, u32, LaserFeelingsType), // (unused, always 0; target; roll_type)
    Alien,                                      // Basic alien roll (count 6s as successes)
    AlienStress(u32), // Stress dice (count 6s, track 1s for panic, stress level)
    ForgedDark,
    ForgedDarkZero,
    Daggerheart,                   // Daggerheart player roll (2d12 Hope/Fear)
    WildWords(Option<u32>),        // Wild Words RPG: None=basic, Some(n)=cut n highest dice
    Mothership(Option<u32>, bool), // Mothership RPG: (stat_target, is_advantage) - roll-under with doubles as crits
    MutantsMasterminds,            // Mutants & Masterminds degree system
    PlotDie,                       // Plotweaver system plot die
    Essence20(u32, bool),          // ess# / esss# - (skill rank 1-8, specialization)
    DarkestHouse(bool),            // tdh / tdhc - House Die (true = called upon the house)
}

impl Modifier {
    /// Splits math by position: math before the first target applies to each
    /// die, math after the last applies to the success count (when one of the
    /// targets sets a success count, see `sets_success_count`).
    pub fn is_target(&self) -> bool {
        self.sets_success_count() || matches!(self, Modifier::Failure(_) | Modifier::Botch(_))
    }

    /// Systems whose roller counts the pooled dice (Icons, sixes, the highest
    /// die) and applies no math to the result. An added dice operand joins the
    /// pool (apply_dice_operand puts its dice in kept_rolls, which these count),
    /// so `+ 1d6` works, as their rules phrase a bonus; a number or `- * /` is
    /// dropped by the roller, so the parser refuses it. Returns what the system
    /// does, for the error. W&G total mode (`wngt`) sums the dice, keeping math.
    pub fn counts_dice_without_math(&self) -> Option<&'static str> {
        match self {
            Modifier::WrathGlory(_, false, _) => Some("Wrath & Glory counts Icons"),
            Modifier::Alien | Modifier::AlienStress(_) => Some("Alien counts sixes"),
            Modifier::ForgedDark | Modifier::ForgedDarkZero => {
                Some("Forged in the Dark reads the highest die")
            }
            Modifier::WildWords(_) => Some("Wild Words reads the highest die"),
            Modifier::LaserFeelings(..) => {
                Some("Lasers & Feelings counts dice against your number")
            }
            _ => None,
        }
    }

    /// Targets that make the roller apply math after them to the success count.
    /// `f#` and `b#` don't: with only those, math after `f#` is ignored and math
    /// after `b#` goes on the total, unless math also comes before `b#`, which
    /// drops it (`4d1 + 1 b1 + 5` is 8).
    pub fn sets_success_count(&self) -> bool {
        matches!(
            self,
            Modifier::Target(_)
                | Modifier::TargetLower(_)
                | Modifier::TargetWithDoubleSuccess(..)
                | Modifier::TargetLowerWithDoubleSuccess(..)
                | Modifier::TwoD20Test { .. }
        )
    }
}

#[derive(Debug, Clone)]
pub struct DiceGroup {
    pub _description: String, // Never displayed; kept for debugging
    pub rolls: Vec<i32>,
    pub dropped_rolls: Vec<i32>, // Dropped dice for strikethrough display
    pub modifier_type: String,   // "base", "add", "subtract", "multiply", "divide", "result"
}

#[derive(Debug, Clone, Default)]
pub struct RollResult {
    pub individual_rolls: Vec<i32>,
    pub kept_rolls: Vec<i32>,
    pub dropped_rolls: Vec<i32>,
    pub total: i32,
    pub successes: Option<i32>,
    pub failures: Option<i32>,
    pub botches: Option<i32>,
    pub comment: Option<String>,
    pub label: Option<String>,
    pub notes: Vec<String>,
    pub dice_groups: Vec<DiceGroup>,
    pub original_expression: Option<String>, // The expression that generated this result
    pub simple: bool,                        // `s` flag: result only, no dice breakdown
    pub no_results: bool,                    // `nr` flag: dice breakdown only, no result
    pub private: bool,                       // `p` flag: ephemeral response
    pub godbound_damage: Option<i32>,        // Store converted Godbound damage
    pub fudge_symbols: Option<Vec<String>>,  // Store Fudge dice symbols
    // Wrath & Glory specific fields
    pub wng_wrath_die: Option<i32>, // Value of the wrath die (first die)
    pub wng_icons: Option<i32>,     // Count of icons (4-5 results)
    pub wng_exalted_icons: Option<i32>, // Count of exalted icons (6 results)
    pub wng_wrath_dice: Option<Vec<i32>>, // All wrath dice values (for multiple dice)
    pub suppress_comment: bool,
    pub alien_stress_level: Option<u32>, // Current stress level for Alien RPG
    pub alien_panic_roll: Option<i32>,   // Panic roll result (1d6 + stress level)
    pub alien_stress_ones: Option<i32>,  // Count of 1s rolled on stress dice
    pub fitd_outcome: Option<String>, // "SUCCESS", "PARTIAL SUCCESS", "FAILURE", "CRITICAL SUCCESS"
    pub fitd_result: Option<String>,  // Description of what the outcome means
    pub fitd_highest_die: Option<i32>, // The key die used for the result
    pub plot_symbols: Option<Vec<String>>, // Store Plot dice symbols
    // Imploded dice (`i` / `i#`): rolled extras stored as positive values and
    // subtracted from the total. Kept out of `kept_rolls` so they are never
    // dropped, kept, or counted as successes.
    pub implosion_rolls: Vec<i32>,
}

impl RollResult {
    /// Empty result carrying the display metadata of the roll that produced it.
    ///
    /// Every game-system handler starts from this, so a field added to
    /// `RollResult` gets its per-roll default in exactly one place instead of
    /// being forgotten in one handler out of a dozen.
    pub fn from_dice(dice: &DiceRoll) -> Self {
        Self {
            comment: dice.comment.clone(),
            label: dice.label.clone(),
            original_expression: dice.original_expression.clone(),
            simple: dice.simple,
            no_results: dice.no_results,
            private: dice.private,
            ..Self::default()
        }
    }

    /// Format the dice roll display based on whether we have dice groups or kept rolls
    fn format_dice_display(&self) -> String {
        // Special handling for Fudge dice
        if let Some(ref symbols) = self.fudge_symbols {
            return format!("`[{}]`", symbols.join(", "));
        }

        // Special handling for a Plot die
        if let Some(ref symbols) = self.plot_symbols {
            return format!("`[{}]`", symbols.join(", "));
        }

        // When there are dropped dice, show every original die, not only the kept ones
        if !self.dropped_rolls.is_empty() {
            if !self.dice_groups.is_empty() {
                // For complex expressions with dice groups AND dropped dice:
                // Use dice groups, which should contain all original dice from each group
                // (If dice groups were modified by keep operations, this will still work
                // because format_dropped_dice() will show what was dropped separately)
                return self.format_dice_groups();
            } else {
                // Simple case: no dice groups, but there are dropped dice
                // Show all original dice (kept + dropped) as a single group
                let mut all_original_dice = self.individual_rolls.clone();
                all_original_dice.extend(self.dropped_rolls.clone());
                return format!(
                    "`[{}]`",
                    all_original_dice
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }

        // Standard behavior for cases without dropped dice
        if !self.dice_groups.is_empty() {
            self.format_dice_groups()
        } else if !self.kept_rolls.is_empty() {
            format!(
                "`[{}]`",
                self.kept_rolls
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            String::new()
        }
    }

    /// Format dice groups with proper separators
    fn format_dice_groups(&self) -> String {
        let mut output = String::new();
        for (i, group) in self.dice_groups.iter().enumerate() {
            if i > 0 {
                match group.modifier_type.as_str() {
                    "add" => output.push_str(" + "),
                    "subtract" => output.push_str(" - "),
                    "multiply" => output.push_str(" * "),
                    "divide" => output.push_str(" / "),
                    "result" => output.push_str(" → "),
                    _ => output.push(' '),
                }
            }
            let formatted_rolls: Vec<String> = group
                .rolls
                .iter()
                .map(|&roll| {
                    if roll == -1 {
                        "**M**".to_string() // Marvel logo in bold
                    } else {
                        roll.to_string()
                    }
                })
                .collect();

            output.push_str(&format!("`[{}]`", formatted_rolls.join(", ")));

            if !group.dropped_rolls.is_empty() {
                let dropped_formatted: Vec<String> = group
                    .dropped_rolls
                    .iter()
                    .map(|&roll| roll.to_string())
                    .collect();
                output.push_str(&format!(" ~~[{}]~~", dropped_formatted.join(", ")));
            }
        }
        output
    }

    /// Format dropped dice if any exist
    fn format_dropped_dice(&self) -> String {
        if self.dropped_rolls.is_empty() {
            String::new()
        } else {
            format!(
                " ~~[{}]~~",
                self.dropped_rolls
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    /// Format the result value (damage, successes, or total)
    fn format_result_value(&self) -> String {
        // Special handling for Wrath & Glory
        if let (Some(icons), Some(exalted_icons)) = (self.wng_icons, self.wng_exalted_icons) {
            let exalted_value = exalted_icons * 2;
            let icon_totals = format!(
                "TOTAL - Icons: `{icons}` Exalted Icons: `{exalted_icons}` (Value:{exalted_value})"
            );

            // Damage, Determination and exempt rolls have no Wrath Die
            let Some(wrath_dice) = self.wng_wrath_dice.as_ref() else {
                return icon_totals;
            };

            // Format wrath dice display
            let wrath_display = if wrath_dice.len() == 1 {
                format!("Wrath: `{}`", wrath_dice[0])
            } else {
                format!(
                    "Wrath: `[{}]`",
                    wrath_dice
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };

            return format!("{wrath_display} | {icon_totals}");
        }

        // Forged in the Dark result formatting
        if let (Some(outcome), Some(highest_die)) = (&self.fitd_outcome, self.fitd_highest_die) {
            return format!("**{outcome}** (die: `{highest_die}`)");
        }

        if let Some(gb_damage) = self.godbound_damage {
            format!("**{gb_damage}** damage")
        } else if let Some(successes) = self.successes {
            let mut result = format!("**{successes}** successes");
            if let Some(failures) = self.failures
                && failures > 0
            {
                result.push_str(&format!(" ({failures} failures)"));
            }
            if let Some(botches) = self.botches
                && botches > 0
            {
                result.push_str(&format!(" ({botches} botches)"));
            }
            result
        } else if let Some(botches) = self.botches {
            format!("**{}** total, **{}** botches", self.total, botches)
        } else {
            format!("**{}**", self.total)
        }
    }

    /// Format the complete output based on the result type
    fn format_output(&self, show_dice: bool, show_result: bool) -> String {
        let mut output = String::new();

        if let Some(label) = &self.label {
            output.push_str(&format!("**{label}**: "));
        }

        if show_dice {
            let dice_display = self.format_dice_display();
            if !dice_display.is_empty() {
                output.push_str("Roll: ");
                output.push_str(&dice_display);
                output.push_str(&self.format_dropped_dice());
            }
        }

        if show_result {
            let result_value = self.format_result_value();
            if show_dice && !self.format_dice_display().is_empty() {
                output.push_str(&format!(" = {result_value}"));
            } else {
                output.push_str(&format!("= {result_value}"));
            }
        }

        output
    }

    /// Add comment and notes to the output
    fn add_comment_and_notes(&self, output: &mut String) {
        // Only add comment if not suppressed
        if !self.suppress_comment
            && let Some(comment) = &self.comment
        {
            output.push_str(&format!(" Reason: `{comment}`"));
        }

        for note in &self.notes {
            output.push_str(&format!("\n*Note: {note}*"));
        }
    }

    /// Create a simplified copy of the roll result with suppressed comment
    pub fn create_simplified(&self) -> RollResult {
        let mut simplified = self.clone();

        // Set suppress_comment but REPLACE the comment
        simplified.suppress_comment = true;
        simplified.comment = Some("Simplified roll due to character limit".to_string());

        simplified
    }
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = if self.no_results {
            // Show dice breakdown but no total/results
            self.format_output(true, false)
        } else if self.simple {
            // Simple output: show only the total/successes without dice breakdown
            self.format_output(false, true)
        } else {
            // Full output: show dice breakdown and results
            self.format_output(true, true)
        };

        self.add_comment_and_notes(&mut output);
        write!(f, "{output}")
    }
}

pub fn parse_and_roll(input: &str) -> Result<Vec<RollResult>> {
    parse_and_roll_with_rng(input, &mut rng::get_dice_rng())
}

/// Parse and roll `input` drawing every die from `rng`.
///
/// With a seeded generator this is fully reproducible, which is how the test
/// suite pins the exact output of systems whose dice are fixed by the system
/// itself and so cannot be made deterministic through notation.
pub fn parse_and_roll_with_rng(input: &str, rng: &mut impl rand::Rng) -> Result<Vec<RollResult>> {
    let dice_expressions = crate::dice::parser::parse_dice_string(input)?;
    let mut results = Vec::new();

    for dice in dice_expressions {
        let result = crate::dice::roller::roll_dice_with_rng(dice, rng)?;
        results.push(result);
    }

    Ok(results)
}

pub fn format_multiple_results(results: &[RollResult]) -> String {
    if results.is_empty() {
        return "No dice to roll!".to_string();
    }

    if results.len() == 1 {
        return results[0].to_string();
    }

    if is_roll_set(results) {
        format_roll_set_results(results)
    } else if is_semicolon_separated(results) {
        format_semicolon_separated_results(results)
    } else {
        format_standard_multiple_results(results)
    }
}

/// Sum of a roll set's results. In `i64` because each result can be near
/// `i32::MAX` (`2 1d1 +2147483646`), and a formatter has no error to return.
fn roll_set_total(results: &[RollResult]) -> i64 {
    results
        .iter()
        .map(|result| i64::from(calculate_result_value(result)))
        .sum()
}

/// Format roll set results with totals and single comment display
fn format_roll_set_results(results: &[RollResult]) -> String {
    let mut output = String::new();

    // Extract the comment from the first roll to show once for the entire set
    let set_comment = results.first().and_then(|r| r.comment.as_ref());

    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }

        // Create a copy with suppressed comment for individual display
        let mut display_result = result.clone();
        display_result.suppress_comment = true;

        output.push_str(&display_result.to_string());
    }

    output.push_str(&format!("\n**Total: {}**", roll_set_total(results)));

    // Add the comment once for the entire set
    if let Some(comment) = set_comment {
        output.push_str(&format!(" Reason: `{comment}`"));
    }

    output
}

/// Helper function to calculate the appropriate value for a result (reducing duplication)
fn calculate_result_value(result: &RollResult) -> i32 {
    if let Some(gb_damage) = result.godbound_damage {
        gb_damage
    } else if let Some(successes) = result.successes {
        successes
    } else {
        result.total
    }
}

/// Helper function to format multiple results with a custom formatter
fn format_results_with_separator<F>(results: &[RollResult], formatter: F) -> String
where
    F: Fn(&RollResult) -> String,
{
    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&formatter(result));
    }
    output
}

/// Format semicolon-separated results showing individual requests
fn format_semicolon_separated_results(results: &[RollResult]) -> String {
    format_results_with_separator(results, |result| {
        // Show the request for each individual roll (without /roll prefix and without comment)
        if let Some(expr) = &result.original_expression {
            let clean_expr = strip_comment_from_expression(expr);
            format!("Request: `{clean_expr}` {result}")
        } else {
            result.to_string()
        }
    })
}

/// Format standard multiple results
fn format_standard_multiple_results(results: &[RollResult]) -> String {
    format_results_with_separator(results, |result| result.to_string())
}

static COMMENT_SUFFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s*!\s*.*$").expect("Failed to compile COMMENT_SUFFIX_REGEX"));

/// Extract the dice expression without the comment portion
pub(crate) fn strip_comment_from_expression(expr: &str) -> String {
    // Use regex to remove comment (everything after ! including the !)
    COMMENT_SUFFIX_REGEX.replace(expr, "").trim().to_string()
}

/// A roll set: several results, each labelled "Set N".
pub(crate) fn is_roll_set(results: &[RollResult]) -> bool {
    results.len() > 1
        && results
            .iter()
            .all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")))
}

/// Semicolon-separated rolls: several results carrying their own expressions,
/// and not a roll set.
pub(crate) fn is_semicolon_separated(results: &[RollResult]) -> bool {
    results.len() > 1
        && !is_roll_set(results)
        && results.iter().any(|r| r.original_expression.is_some())
}

pub(crate) const DISCORD_MESSAGE_LIMIT: usize = 2000;

/// Cut `content` to fit one Discord message, marking the cut with `...`. A send
/// over the limit fails outright and the user sees "interaction failed", so
/// every reply passes through this before it goes out.
pub fn truncate_to_message_limit(mut content: String) -> String {
    if content.len() > DISCORD_MESSAGE_LIMIT {
        // Cut on a char boundary: `truncate` panics inside a multi-byte char (emoji)
        content.truncate(content.floor_char_boundary(DISCORD_MESSAGE_LIMIT - 3));
        content.push_str("...");
    }
    content
}

pub fn format_multiple_results_with_limit(results: &[RollResult]) -> String {
    let full_output = format_multiple_results(results);

    if full_output.len() <= DISCORD_MESSAGE_LIMIT {
        return full_output;
    }

    // Level 1: Try simplified format
    let simplified_results: Vec<RollResult> =
        results.iter().map(|r| r.create_simplified()).collect();

    let level1_output = if simplified_results.len() == 1 {
        let mut simplified = simplified_results[0].clone();
        simplified.simple = true;
        simplified.to_string()
    } else {
        if is_roll_set(&simplified_results) {
            format_roll_set_results(&simplified_results)
        } else {
            format_results_with_separator(&simplified_results, |result| {
                let mut simple_result = result.clone();
                simple_result.simple = true;
                simple_result.to_string()
            })
        }
    };

    if level1_output.len() <= DISCORD_MESSAGE_LIMIT {
        return level1_output;
    }

    // Level 2: Ultra-compact format (totals only)
    let level2_output = format_ultra_compact_results(results);
    if level2_output.len() <= DISCORD_MESSAGE_LIMIT {
        return level2_output;
    }

    // Level 3: Summary format
    let level3_output = format_summary_results(results);
    if level3_output.len() <= DISCORD_MESSAGE_LIMIT {
        return level3_output;
    }

    // Level 4: Emergency truncation
    truncate_to_message_limit(level3_output)
}

/// Format ultra-compact results (totals only, minimal text)
fn format_ultra_compact_results(results: &[RollResult]) -> String {
    if results.is_empty() {
        return "No results".to_string();
    }

    if results.len() == 1 {
        let result = &results[0];
        if let Some(gb_damage) = result.godbound_damage {
            format!("**{gb_damage}** damage")
        } else if let Some(successes) = result.successes {
            format!("**{successes}** successes")
        } else {
            format!("**{}**", result.total)
        }
    } else {
        if is_roll_set(results) {
            // Show individual set results AND combined total
            let formatted_results: Vec<String> = results
                .iter()
                .map(|result| {
                    let value = calculate_result_value(result);
                    if let Some(gb_damage) = result.godbound_damage {
                        format!("**{gb_damage}**")
                    } else if let Some(successes) = result.successes {
                        format!("**{successes}**")
                    } else {
                        format!("**{value}**")
                    }
                })
                .collect();

            format!(
                "{}\n**Total: {}**",
                formatted_results.join(", "),
                roll_set_total(results)
            )
        } else {
            // Multiple separate rolls (semicolon-separated)
            let formatted_results: Vec<String> = results
                .iter()
                .map(|result| {
                    if let Some(gb_damage) = result.godbound_damage {
                        format!("**{gb_damage}** dmg")
                    } else if let Some(successes) = result.successes {
                        format!("**{successes}** succ")
                    } else {
                        format!("**{}**", result.total)
                    }
                })
                .collect();
            formatted_results.join(", ")
        }
    }
}

/// Format summary results (very minimal information)
fn format_summary_results(results: &[RollResult]) -> String {
    if results.is_empty() {
        return "No results".to_string();
    }

    if results.len() == 1 {
        format!("Result: **{}**", calculate_result_value(&results[0]))
    } else {
        if is_roll_set(results) {
            format!(
                "{} sets, Total: **{}**",
                results.len(),
                roll_set_total(results)
            )
        } else {
            format!("{} results", results.len())
        }
    }
}
