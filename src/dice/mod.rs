pub mod parser;
pub mod roller;
pub mod aliases;

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
    Explode(Option<u32>),      // e or e#
    ExplodeIndefinite(Option<u32>), // ie or ie#
    Drop(u32),                 // d#
    KeepHigh(u32),            // k#
    KeepLow(u32),             // kl#
    Reroll(u32),              // r#
    RerollIndefinite(u32),    // ir#
    Target(u32),              // t#
    Failure(u32),             // f#
    Botch(Option<u32>),       // b or b#
    AddDice(DiceRoll),        // Additional dice
    SubtractDice(DiceRoll),   // Subtract dice result
    WrathGlory(Option<u32>, bool), // Wrath & Glory: (difficulty, use_total_instead_of_successes)
}

#[derive(Debug, Clone)]
pub struct DiceGroup {
    pub _description: String,  // Currently unused but kept for future debugging
    pub rolls: Vec<i32>,
    pub modifier_type: String,  // "base", "add", "subtract"
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
    pub simple: bool, // Add simple flag to control output formatting
    pub no_results: bool, // Add no_results flag
    pub private: bool, // Add private flag for ephemeral responses
}

impl fmt::Display for RollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        
        if let Some(label) = &self.label {
            output.push_str(&format!("**{}**: ", label));
        }
        
        // Check if no_results flag is set - show dice tally but no result total
        if self.no_results {
            // Show dice breakdown but no total/results
            if !self.dice_groups.is_empty() {
                output.push_str("Roll: ");
                for (i, group) in self.dice_groups.iter().enumerate() {
                    if i > 0 {
                        match group.modifier_type.as_str() {
                            "add" => output.push_str(" + "),
                            "subtract" => output.push_str(" - "),
                            _ => output.push(' '),
                        }
                    }
                    output.push_str(&format!("`[{}]`", 
                        group.rolls.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            } else if !self.kept_rolls.is_empty() {
                output.push_str(&format!("Roll: `[{}]`", 
                    self.kept_rolls.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            
            // Add dropped dice if any
            if !self.dropped_rolls.is_empty() {
                output.push_str(&format!(" ~~[{}]~~", 
                    self.dropped_rolls.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            // Note: No total/results shown for nr flag
        } else if self.simple {
            // Simple output: show only the total/successes without dice breakdown
            if let Some(successes) = self.successes {
                output.push_str(&format!("= **{}** successes", successes));
                if let Some(failures) = self.failures {
                    if failures > 0 {
                        output.push_str(&format!(" ({} failures)", failures));
                    }
                }
                if let Some(botches) = self.botches {
                    if botches > 0 {
                        output.push_str(&format!(" ({} botches)", botches));
                    }
                }
            } else if let Some(botches) = self.botches {
                output.push_str(&format!("= **{}** total, **{}** botches", self.total, botches));
            } else {
                output.push_str(&format!("= **{}**", self.total));
            }
        } else {
            // Full output: show dice breakdown
            // Add dice groups if we have them, otherwise use kept_rolls
            if !self.dice_groups.is_empty() {
                output.push_str("Roll: ");
                for (i, group) in self.dice_groups.iter().enumerate() {
                    if i > 0 {
                        match group.modifier_type.as_str() {
                            "add" => output.push_str(" + "),
                            "subtract" => output.push_str(" - "),
                            _ => output.push(' '),
                        }
                    }
                    output.push_str(&format!("`[{}]`", 
                        group.rolls.iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            } else if !self.kept_rolls.is_empty() {
                output.push_str(&format!("Roll: `[{}]`", 
                    self.kept_rolls.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            
            // Add dropped dice if any
            if !self.dropped_rolls.is_empty() {
                output.push_str(&format!(" ~~[{}]~~", 
                    self.dropped_rolls.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            
            // Add results after the tally
            if let Some(successes) = self.successes {
                output.push_str(&format!(" = **{}** successes", successes));
                if let Some(failures) = self.failures {
                    if failures > 0 {
                        output.push_str(&format!(" ({} failures)", failures));
                    }
                }
                if let Some(botches) = self.botches {
                    if botches > 0 {
                        output.push_str(&format!(" ({} botches)", botches));
                    }
                }
            } else if let Some(botches) = self.botches {
                // Show botches even without success system
                output.push_str(&format!(" = **{}** total, **{}** botches", self.total, botches));
            } else {
                output.push_str(&format!(" = **{}**", self.total));
            }
        }
        
        if let Some(comment) = &self.comment {
            output.push_str(&format!(" Reason: `{}`", comment));
        }
        
        for note in &self.notes {
            output.push_str(&format!("\n*Note: {}*", note));
        }
        
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
        && results.iter().all(|r| r.label.as_ref().is_some_and(|l| l.starts_with("Set ")));
    
    // Check if these are semicolon-separated rolls (have original expressions but no "Set X" labels)
    let is_semicolon_separated = results.len() > 1 
        && !is_roll_set
        && results.iter().any(|r| r.original_expression.is_some());
    
    if is_roll_set {
        // Special formatting for roll sets - no "Request:" prefix
        let mut output = String::new();
        let mut total_sum = 0;
        
        for (i, result) in results.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&result.to_string());
            
            if let Some(successes) = result.successes {
                total_sum += successes;
            } else {
                total_sum += result.total;
            }
        }
        
        output.push_str(&format!("\n**Total: {}**", total_sum));
        output
    } else if is_semicolon_separated {
        // Special formatting for semicolon-separated rolls - show individual requests
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
    } else {
        // Original formatting for other multiple results
        let mut output = String::new();
        for (i, result) in results.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&result.to_string());
        }
        output
    }
}
