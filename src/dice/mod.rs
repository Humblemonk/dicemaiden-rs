pub mod aliases;
pub mod parser;
pub mod roller;

use anyhow::Result;
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
pub enum Modifier {
    Add(i32),
    Subtract(i32),
    Multiply(i32),
    Divide(i32),
    Explode(Option<u32>),           // e or e#
    ExplodeIndefinite(Option<u32>), // ie or ie#
    Drop(u32),                      // d#
    KeepHigh(u32),                  // k#
    KeepLow(u32),                   // kl#
    Reroll(u32),                    // r#
    RerollIndefinite(u32),          // ir#
    Target(u32),                    // t#
    Failure(u32),                   // f#
    Botch(Option<u32>),             // b or b#
    AddDice(DiceRoll),              // Additional dice
    SubtractDice(DiceRoll),         // Subtract dice result
    WrathGlory(Option<u32>, bool),  // Wrath & Glory: (difficulty, use_total_instead_of_successes)
    Godbound(bool),                 // gb (false) or gbs (true for straight damage)
}

#[derive(Debug, Clone)]
pub struct DiceGroup {
    pub _description: String, // Currently unused but kept for future debugging
    pub rolls: Vec<i32>,
    pub modifier_type: String, // "base", "add", "subtract"
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
}

impl RollResult {
    /// Format the dice roll display based on whether we have dice groups or kept rolls
    fn format_dice_display(&self) -> String {
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
                    _ => output.push(' '),
                }
            }
            output.push_str(&format!(
                "`[{}]`",
                group
                    .rolls
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
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
        if let Some(gb_damage) = self.godbound_damage {
            format!("**{}** damage", gb_damage)
        } else if let Some(successes) = self.successes {
            let mut result = format!("**{}** successes", successes);
            if let Some(failures) = self.failures {
                if failures > 0 {
                    result.push_str(&format!(" ({} failures)", failures));
                }
            }
            if let Some(botches) = self.botches {
                if botches > 0 {
                    result.push_str(&format!(" ({} botches)", botches));
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
            output.push_str(&format!("**{}**: ", label));
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
                output.push_str(&format!(" = {}", result_value));
            } else {
                output.push_str(&format!("= {}", result_value));
            }
        }

        output
    }

    /// Add comment and notes to the output
    fn add_comment_and_notes(&self, output: &mut String) {
        if let Some(comment) = &self.comment {
            output.push_str(&format!(" Reason: `{}`", comment));
        }

        for note in &self.notes {
            output.push_str(&format!("\n*Note: {}*", note));
        }
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
        write!(f, "{}", output)
    }
}

pub fn parse_and_roll(input: &str) -> Result<Vec<RollResult>> {
    let dice_expressions = parser::parse_dice_string(input)?;
    let mut results = Vec::new();

    for dice in dice_expressions {
        let result = roller::roll_dice(dice)?;
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

/// Format roll set results with totals
fn format_roll_set_results(results: &[RollResult]) -> String {
    let mut output = String::new();
    let mut total_sum = 0;

    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&result.to_string());

        // Sum based on what type of result this is
        total_sum += if let Some(gb_damage) = result.godbound_damage {
            gb_damage
        } else if let Some(successes) = result.successes {
            successes
        } else {
            result.total
        };
    }

    output.push_str(&format!("\n**Total: {}**", total_sum));
    output
}

/// Format semicolon-separated results showing individual requests
fn format_semicolon_separated_results(results: &[RollResult]) -> String {
    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }

        // Show the request for each individual roll
        if let Some(expr) = &result.original_expression {
            output.push_str(&format!("Request: `/roll {}` {}", expr, result));
        } else {
            output.push_str(&result.to_string());
        }
    }
    output
}

/// Format standard multiple results
fn format_standard_multiple_results(results: &[RollResult]) -> String {
    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&result.to_string());
    }
    output
}
