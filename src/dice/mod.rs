pub mod aliases;
pub mod parser;
pub mod roller;

use anyhow::Result;
use regex::Regex;
use std::fmt;

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
    pub original_expression: Option<String>, // Store the original expression
}

#[derive(Debug, Clone)]
pub enum HeroSystemType {
    Normal,  // hsn - normal damage
    Killing, // hsk - killing damage
    Hit,     // hsh - to hit roll (3d6 roll-under)
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

#[derive(Debug, Clone)]
pub enum Modifier {
    Add(i32),
    Subtract(i32),
    Multiply(i32),
    Divide(i32),
    Explode(Option<u32>),                   // e or e#
    ExplodeIndefinite(Option<u32>),         // ie or ie#
    Drop(u32),                              // d#
    KeepHigh(u32),                          // k#
    KeepLow(u32),                           // kl#
    KeepMiddle(u32),                        // km#
    Reroll(u32),                            // r#
    RerollIndefinite(u32),                  // ir#
    Target(u32),                            // t#  - count successes >= target
    RerollGreater(u32),                     // rg# - reroll >= threshold once
    RerollGreaterIndefinite(u32),           // irg# - reroll >= threshold indefinitely
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
    WrathGlory(Option<u32>, bool, u32), // Wrath & Glory: (difficulty, use_total_instead_of_successes)
    Godbound(bool),                     // gb (false) or gbs (true for straight damage)
    HeroSystem(HeroSystemType),         // Hero System damage/hit calculations
    Fudge,                              // df - Fudge dice with symbol display
    DarkHeresy,
    SavageWorlds(u32),
    D6System(u32, String),
    Shadowrun(u32),
    MarvelMultiverse(i32, i32), // (edges, troubles) - already calculated net values
    CyberpunkRed,
    Witcher,
    CypherSystem(u32),
    BraveNewWorld(u32),
    ConanSkill(u32),  // conan, conan3, conan4, conan5 - d20 skill rolls
    ConanCombat(u32), // cd, cd4, cd5 - combat dice interpretation
    Silhouette(u32),
    VampireMasquerade5(u32, u32),
    LaserFeelings(u32, u32, LaserFeelingsType), // (dice_count, target, roll_type)
    Alien,                                      // Basic alien roll (count 6s as successes)
    AlienStress(u32), // Stress dice (count 6s, track 1s for panic, stress level)
    ForgedDark,
    ForgedDarkZero,
}

#[derive(Debug, Clone)]
pub struct DiceGroup {
    pub _description: String, // Currently unused but kept for future debugging
    pub rolls: Vec<i32>,
    pub dropped_rolls: Vec<i32>, // Dropped dice for strikethrough display
    pub modifier_type: String,   // "base", "add", "subtract"
}

#[derive(Debug, Clone)]
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
    pub original_expression: Option<String>, // Store the original expression that generated this result
    pub simple: bool,                        // Add simple flag to control output formatting
    pub no_results: bool,                    // Add no_results flag
    pub private: bool,                       // Add private flag for ephemeral responses
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
}

impl RollResult {
    /// Format the dice roll display based on whether we have dice groups or kept rolls
    fn format_dice_display(&self) -> String {
        // Special handling for Fudge dice
        if let Some(ref symbols) = self.fudge_symbols {
            return format!("`[{}]`", symbols.join(", "));
        }

        // When there are dropped dice, always prioritize showing all original dice
        // This addresses the user's complaint that dice groups only show kept dice
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
        if let (Some(wrath_dice), Some(icons), Some(exalted_icons)) = (
            self.wng_wrath_dice.as_ref(),
            self.wng_icons,
            self.wng_exalted_icons,
        ) {
            let exalted_value = exalted_icons * 2;

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

            return format!(
                "{wrath_display} | TOTAL - Icons: `{icons}` Exalted Icons: `{exalted_icons}` (Value:{exalted_value})"
            );
        }

        // Forged in the Dark result formatting
        if let (Some(outcome), Some(highest_die)) = (&self.fitd_outcome, self.fitd_highest_die) {
            return format!("**{outcome}** (die: `{highest_die}`)");
        }

        if let Some(gb_damage) = self.godbound_damage {
            format!("**{gb_damage}** damage")
        } else if let Some(successes) = self.successes {
            let mut result = format!("**{successes}** successes");
            if let Some(failures) = self.failures {
                if failures > 0 {
                    result.push_str(&format!(" ({failures} failures)"));
                }
            }
            if let Some(botches) = self.botches {
                if botches > 0 {
                    result.push_str(&format!(" ({botches} botches)"));
                }
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
        if !self.suppress_comment {
            if let Some(comment) = &self.comment {
                output.push_str(&format!(" Reason: `{comment}`"));
            }
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
    let dice_expressions = crate::dice::parser::parse_dice_string(input)?;
    let mut results = Vec::new();

    for dice in dice_expressions {
        let result = crate::dice::roller::roll_dice(dice)?;
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

    // Check if this is a roll set (all results have "Set X" labels)
    let is_roll_set = results.len() > 1
        && results
            .iter()
            .all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));

    // Check if these are semicolon-separated rolls (have original expressions but no "Set X" labels)
    let is_semicolon_separated = results.len() > 1
        && !is_roll_set
        && results.iter().any(|r| r.original_expression.is_some());

    if is_roll_set {
        format_roll_set_results(results)
    } else if is_semicolon_separated {
        format_semicolon_separated_results(results)
    } else {
        format_standard_multiple_results(results)
    }
}

/// Format roll set results with totals and single comment display
fn format_roll_set_results(results: &[RollResult]) -> String {
    let mut output = String::new();
    let mut total_sum = 0;

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

        // Sum based on what type of result this is
        total_sum += calculate_result_value(result);
    }

    output.push_str(&format!("\n**Total: {total_sum}**"));

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

/// Extract the dice expression without the comment portion
fn strip_comment_from_expression(expr: &str) -> String {
    // Use regex to remove comment (everything after ! including the !)
    let comment_regex = Regex::new(r"\s*!\s*.*$").unwrap();
    comment_regex.replace(expr, "").trim().to_string()
}

const DISCORD_MESSAGE_LIMIT: usize = 2000;

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
        let is_roll_set = simplified_results.len() > 1
            && simplified_results
                .iter()
                .all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));

        if is_roll_set {
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
    let mut emergency_output = level3_output;
    if emergency_output.len() > DISCORD_MESSAGE_LIMIT - 3 {
        emergency_output.truncate(DISCORD_MESSAGE_LIMIT - 3);
        emergency_output.push_str("...");
    }
    emergency_output
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
        // Check if this is a roll set
        let is_roll_set = results.len() > 1
            && results
                .iter()
                .all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));

        if is_roll_set {
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

            let total_sum: i32 = results.iter().map(calculate_result_value).sum();
            format!("{}\n**Total: {}**", formatted_results.join(", "), total_sum)
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
        let is_roll_set = results.len() > 1
            && results
                .iter()
                .all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));

        if is_roll_set {
            let total: i32 = results.iter().map(calculate_result_value).sum();
            format!("{} sets, Total: **{}**", results.len(), total)
        } else {
            format!("{} results", results.len())
        }
    }
}
