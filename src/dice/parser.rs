use super::{DiceRoll, HeroSystemType, Modifier};
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;

// Pre-compile all regex patterns at startup to reduce memory allocations
static SET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)\s+(.+)$").expect("Failed to compile SET_REGEX"));

static DICE_ONLY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)?d(\d+|%)$").expect("Failed to compile DICE_ONLY_REGEX"));

static LABEL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\(([^)]*)\)\s*").expect("Failed to compile LABEL_REGEX"));

static COMMENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!\s*(.*)$").expect("Failed to compile COMMENT_REGEX"));

static OP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+\-*/])(\d+)$").expect("Failed to compile OP_REGEX"));

static DICE_MOD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+\-])(\d+)d(\d+)$").expect("Failed to compile DICE_MOD_REGEX"));

// Fixed label regex for nested parentheses
static LABEL_NESTED_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\(([^)]+)\)\s*").expect("Failed to compile LABEL_NESTED_REGEX"));

pub fn parse_dice_string(input: &str) -> Result<Vec<DiceRoll>> {
    let input = input.trim();

    // Check for multi-roll (semicolon separated) FIRST
    if input.contains(';') {
        let parts: Vec<&str> = input.split(';').collect();
        if parts.len() > 4 {
            return Err(anyhow!("Maximum of 4 separate rolls allowed"));
        }

        let mut results = Vec::with_capacity(parts.len());
        for part in parts {
            let part = part.trim();
            let mut sub_results = parse_dice_string(part)?;
            for dice in &mut sub_results {
                dice.original_expression = Some(part.to_string());
            }
            results.extend(sub_results);
        }
        return Ok(results);
    }

    // Check for roll sets SECOND
    if let Some(captures) = SET_REGEX.captures(input) {
        let count: u32 = captures[1]
            .parse()
            .map_err(|_| anyhow!("Invalid set count"))?;
        if !(2..=20).contains(&count) {
            return Err(anyhow!("Set count must be between 2 and 20"));
        }
        let dice_expr = &captures[2];

        let mut results = Vec::with_capacity(count as usize);
        for i in 0..count {
            let mut dice = parse_single_dice_expression(dice_expr)?;
            dice.label = Some(format!("Set {}", i + 1));
            results.push(dice);
        }
        return Ok(results);
    }

    // Parse single expression
    Ok(vec![parse_single_dice_expression(input)?])
}

fn parse_single_dice_expression(input: &str) -> Result<DiceRoll> {
    let mut dice = DiceRoll {
        count: 1,
        sides: 6,
        modifiers: Vec::new(),
        comment: None,
        label: None,
        private: false,
        simple: false,
        no_results: false,
        unsorted: false,
        original_expression: None,
    };

    // Normalize and handle the input
    let normalized_input = normalize_whitespace(input.trim());
    let mut remaining = normalized_input.as_str();

    // Parse flags, labels, and comments
    remaining = parse_flags(&mut dice, remaining);
    remaining = parse_label(&mut dice, remaining);
    remaining = parse_comment(&mut dice, remaining);
    remaining = remaining.trim();

    // Check for aliases after parsing flags/comments
    if let Some(expanded) = super::aliases::expand_alias(remaining) {
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;
        
        // Transfer flags and metadata
        expanded_dice.private = dice.private;
        expanded_dice.simple = dice.simple;
        expanded_dice.no_results = dice.no_results;
        expanded_dice.unsorted = dice.unsorted;
        expanded_dice.comment = dice.comment;
        expanded_dice.label = dice.label;

        return Ok(expanded_dice);
    }

    // Parse the expression into parts
    let parts = parse_expression_to_parts(remaining)?;

    if parts.is_empty() {
        return Err(anyhow!("No dice expression found"));
    }

    // Parse the base dice
    parse_base_dice(&mut dice, &parts[0])?;

    // Parse modifiers
    parse_all_modifiers(&mut dice, &parts[1..])?;

    Ok(dice)
}

fn normalize_whitespace(input: &str) -> String {
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    whitespace_regex.replace_all(input.trim(), " ").to_string()
}

// FIXED: Better expression parsing that handles combined modifiers
fn parse_expression_to_parts(input: &str) -> Result<Vec<String>> {
    // Handle simple cases first
    if input.is_empty() {
        return Ok(vec![]);
    }
    
    // If the input contains spaces, handle as space-separated
    if input.contains(' ') {
        return parse_space_separated_expression(input);
    }
    
    // If no spaces, check if it's a dice expression with attached modifiers
    let dice_regex = Regex::new(r"^(\d*d\d+)(.*)$").unwrap();
    if let Some(captures) = dice_regex.captures(input) {
        let dice_part = captures[1].to_string();
        let modifiers_part = &captures[2];
        
        if modifiers_part.is_empty() {
            return Ok(vec![dice_part]);
        }
        
        // Parse the modifiers part
        let mut parts = vec![dice_part];
        let modifier_parts = split_combined_modifiers(modifiers_part)?;
        parts.extend(modifier_parts);
        
        return Ok(parts);
    }
    
    // If it's not a dice expression, treat as single part
    Ok(vec![input.to_string()])
}

// FIXED: Better space-separated parsing to handle complex cases
fn parse_space_separated_expression(input: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let words: Vec<&str> = input.split_whitespace().collect();
    
    let mut i = 0;
    while i < words.len() {
        let word = words[i];
        
        // Check if this word is an operator by itself
        if matches!(word, "+" | "-" | "*" | "/") {
            parts.push(word.to_string());
            i += 1;
            
            // Get the next word if it exists
            if i < words.len() {
                parts.push(words[i].to_string());
                i += 1;
            }
        } else {
            // This could be a dice expression, modifier, or combined expression
            
            // Check if it ends with an operator (like "e6k3+")
            if let Some(op_pos) = word.rfind(&['+', '-', '*', '/']) {
                let (main_part, op_part) = word.split_at(op_pos);
                
                if !main_part.is_empty() {
                    // Split the main part if it's a dice expression with modifiers
                    if main_part.contains("d") {
                        let split_parts = split_dice_and_modifiers_internal(main_part)?;
                        parts.extend(split_parts);
                    } else {
                        parts.push(main_part.to_string());
                    }
                }
                
                if !op_part.is_empty() {
                    parts.push(op_part.to_string());
                }
            } else if word.contains("d") {
                // This is a dice expression, possibly with modifiers
                let split_parts = split_dice_and_modifiers_internal(word)?;
                parts.extend(split_parts);
            } else {
                // Regular word/modifier
                parts.push(word.to_string());
            }
            i += 1;
        }
    }
    
    Ok(parts)
}

// Helper function for splitting dice and modifiers
fn split_dice_and_modifiers_internal(input: &str) -> Result<Vec<String>> {
    let dice_regex = Regex::new(r"^(\d*d\d+)(.*)$").unwrap();
    if let Some(captures) = dice_regex.captures(input) {
        let dice_part = captures[1].to_string();
        let modifiers_part = &captures[2];
        
        if modifiers_part.is_empty() {
            return Ok(vec![dice_part]);
        }
        
        let mut parts = vec![dice_part];
        let modifier_parts = split_combined_modifiers(modifiers_part)?;
        parts.extend(modifier_parts);
        
        Ok(parts)
    } else {
        Ok(vec![input.to_string()])
    }
}

// FIXED: Split combined modifiers like "e6k3" into ["e6", "k3"]
fn split_combined_modifiers(input: &str) -> Result<Vec<String>> {
    let mut modifiers = Vec::new();
    let mut remaining = input;
    
    while !remaining.is_empty() {
        let (modifier, rest) = extract_next_modifier(remaining)?;
        if modifier.is_empty() {
            // If we can't extract a modifier, treat the rest as one part
            if !remaining.is_empty() {
                modifiers.push(remaining.to_string());
            }
            break;
        }
        modifiers.push(modifier);
        remaining = rest;
    }
    
    if modifiers.is_empty() {
        // If we couldn't split anything, return the original
        Ok(vec![input.to_string()])
    } else {
        Ok(modifiers)
    }
}

// FIXED: Extract the next modifier from a combined string
fn extract_next_modifier(input: &str) -> Result<(String, &str)> {
    if input.is_empty() {
        return Ok((String::new(), input));
    }
    
    // Try different modifier patterns in order of preference
    let patterns = [
        r"^(ie\d*)",    // Indefinite explode
        r"^(ir\d+)",    // Indefinite reroll
        r"^(kl\d+)",    // Keep lowest
        r"^(e\d*)",     // Explode
        r"^(k\d+)",     // Keep highest
        r"^(r\d+)",     // Reroll
        r"^(d\d+)",     // Drop
        r"^(t\d+)",     // Target
        r"^(f\d+)",     // Failure
        r"^(b\d*)",     // Botch
        r"^(wng\d*t?)", // Wrath & Glory
        r"^(gb|gbs)",   // Godbound
        r"^(hs[nkh])",  // Hero System
        r"^(dh)",       // Dark Heresy
        r"^(fudge|df)", // Fudge
        r"^([+\-*/]\d+)", // Mathematical operators
    ];
    
    for pattern in &patterns {
        let regex = Regex::new(pattern).unwrap();
        if let Some(captures) = regex.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // If no pattern matched, return empty
    Ok((String::new(), input))
}

// FIXED: Flag parsing with better handling of combined expressions
fn parse_flags<'a>(dice: &mut DiceRoll, mut remaining: &'a str) -> &'a str {
    let flags = ["p", "s", "nr", "ul"];
    
    let mut changed = true;
    while changed {
        changed = false;
        let original = remaining;
        
        for &flag in &flags {
            // Check for flag followed by space
            let flag_with_space = format!("{} ", flag);
            if remaining.starts_with(&flag_with_space) {
                match flag {
                    "p" => dice.private = true,
                    "s" => dice.simple = true,
                    "nr" => dice.no_results = true,
                    "ul" => dice.unsorted = true,
                    _ => {}
                }
                remaining = &remaining[flag_with_space.len()..];
                changed = true;
                break;
            }
            // Check for flag at end or followed by non-alphabetic character
            else if remaining == flag {
                match flag {
                    "p" => dice.private = true,
                    "s" => dice.simple = true,
                    "nr" => dice.no_results = true,
                    "ul" => dice.unsorted = true,
                    _ => {}
                }
                remaining = "";
                changed = true;
                break;
            }
            // Check for flag followed by digit (like "ul4d6")
            else if remaining.starts_with(flag) && remaining.len() > flag.len() {
                let next_char = remaining.chars().nth(flag.len()).unwrap();
                if next_char.is_ascii_digit() {
                    match flag {
                        "p" => dice.private = true,
                        "s" => dice.simple = true,
                        "nr" => dice.no_results = true,
                        "ul" => dice.unsorted = true,
                        _ => {}
                    }
                    remaining = &remaining[flag.len()..];
                    changed = true;
                    break;
                }
            }
        }
        
        if remaining == original {
            break;
        }
        
        remaining = remaining.trim_start();
    }
    
    remaining
}

fn parse_label<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    // Fixed: Handle labels with nested parentheses more carefully
    if let Some(captures) = LABEL_REGEX.captures(remaining) {
        let label_content = &captures[1];
        if !label_content.is_empty() {
            // Don't allow nested parentheses in labels
            if !label_content.contains('(') && !label_content.contains(')') {
                dice.label = Some(label_content.to_string());
                return &remaining[captures.get(0).unwrap().end()..];
            }
        }
    }
    remaining
}

fn parse_comment<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if remaining.trim() == "!" {
        return "";
    }
    
    if let Some(captures) = COMMENT_REGEX.captures(remaining) {
        let comment_content = captures[1].trim();
        if !comment_content.is_empty() {
            // Handle semicolon in comments - keep everything before first semicolon
            if comment_content.contains(';') {
                if let Some(first_part) = comment_content.split(';').next() {
                    if !first_part.trim().is_empty() {
                        dice.comment = Some(first_part.trim().to_string());
                    }
                }
            } else {
                dice.comment = Some(comment_content.to_string());
            }
        }
        remaining[..captures.get(0).unwrap().start()].trim()
    } else {
        remaining
    }
}

fn parse_base_dice(dice: &mut DiceRoll, part: &str) -> Result<()> {
    // Try alias expansion first
    if let Some(expanded) = super::aliases::expand_alias(part) {
        let expanded_dice = parse_single_dice_expression(&expanded)?;
        dice.count = expanded_dice.count;
        dice.sides = expanded_dice.sides;
        dice.modifiers.extend(expanded_dice.modifiers);
        return Ok(());
    }

    if let Some(captures) = DICE_ONLY_REGEX.captures(part) {
        dice.count = captures
            .get(1)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);

        if &captures[2] == "%" {
            dice.sides = 100;
        } else {
            dice.sides = captures[2]
                .parse()
                .map_err(|_| anyhow!("Invalid dice sides"))?;
        }

        if dice.count > 500 {
            return Err(anyhow!("Maximum 500 dice allowed"));
        }
        if dice.sides < 1 {
            return Err(anyhow!("Dice must have at least 1 side"));
        }
        if dice.sides > 1000 {
            return Err(anyhow!("Maximum 1000 sides allowed"));
        }
        
        Ok(())
    } else {
        Err(anyhow!("Invalid dice expression: {}", part))
    }
}

fn parse_all_modifiers(dice: &mut DiceRoll, parts: &[String]) -> Result<()> {
    let mut i = 0;
    while i < parts.len() {
        // Try parsing operator + operand pairs
        if i + 1 < parts.len() {
            if let Some(consumed) = try_parse_operator_pair(dice, &parts[i], &parts[i + 1])? {
                i += consumed;
                continue;
            }
        }

        // Parse single modifier
        let modifier = parse_single_modifier(&parts[i])?;
        dice.modifiers.push(modifier);
        i += 1;
    }
    Ok(())
}

fn try_parse_operator_pair(dice: &mut DiceRoll, first: &str, second: &str) -> Result<Option<usize>> {
    match first {
        "+" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Add(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::AddDice(dice_roll));
                return Ok(Some(2));
            }
        }
        "-" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Subtract(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::SubtractDice(dice_roll));
                return Ok(Some(2));
            }
        }
        "*" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Multiply(num));
                return Ok(Some(2));
            }
        }
        "/" => {
            if let Ok(num) = second.parse::<i32>() {
                if num == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                dice.modifiers.push(Modifier::Divide(num));
                return Ok(Some(2));
            }
        }
        _ => {}
    }
    
    Ok(None)
}

fn parse_single_modifier(part: &str) -> Result<Modifier> {
    // System modifiers
    match part {
        "dh" => return Ok(Modifier::DarkHeresy),
        "hsn" => return Ok(Modifier::HeroSystem(HeroSystemType::Normal)),
        "hsk" => return Ok(Modifier::HeroSystem(HeroSystemType::Killing)),
        "hsh" => return Ok(Modifier::HeroSystem(HeroSystemType::Hit)),
        "fudge" | "df" => return Ok(Modifier::Fudge),
        "gb" => return Ok(Modifier::Godbound(false)),
        "gbs" => return Ok(Modifier::Godbound(true)),
        _ => {}
    }

    // Operator with number (e.g., "+5", "-3")
    if let Some(captures) = OP_REGEX.captures(part) {
        let num: i32 = captures[2].parse().map_err(|_| anyhow!("Invalid number"))?;
        return match &captures[1] {
            "+" => Ok(Modifier::Add(num)),
            "-" => Ok(Modifier::Subtract(num)),
            "*" => Ok(Modifier::Multiply(num)),
            "/" => {
                if num == 0 {
                    Err(anyhow!("Cannot divide by zero"))
                } else {
                    Ok(Modifier::Divide(num))
                }
            }
            _ => Err(anyhow!("Unknown operator")),
        };
    }

    // Wrath & Glory
    if let Some(stripped) = part.strip_prefix("wng") {
        let use_total = part.ends_with('t');
        let number_part = if use_total { &stripped[..stripped.len()-1] } else { stripped };
        let difficulty = if number_part.is_empty() { 
            None 
        } else { 
            Some(number_part.parse().map_err(|_| anyhow!("Invalid difficulty"))?) 
        };
        return Ok(Modifier::WrathGlory(difficulty, use_total));
    }

    // Numbered modifiers
    if let Some(stripped) = part.strip_prefix("ie") {
        let num = if stripped.is_empty() { None } else { Some(stripped.parse()?) };
        return Ok(Modifier::ExplodeIndefinite(num));
    }
    
    if let Some(stripped) = part.strip_prefix('e') {
        let num = if stripped.is_empty() { None } else { Some(stripped.parse()?) };
        return Ok(Modifier::Explode(num));
    }
    
    if let Some(stripped) = part.strip_prefix("ir") {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid reroll value"))?;
        return Ok(Modifier::RerollIndefinite(num));
    }
    
    if let Some(stripped) = part.strip_prefix('r') {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid reroll value"))?;
        return Ok(Modifier::Reroll(num));
    }
    
    if let Some(stripped) = part.strip_prefix("kl") {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid keep low value"))?;
        return Ok(Modifier::KeepLow(num));
    }
    
    if let Some(stripped) = part.strip_prefix('k') {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid keep value"))?;
        return Ok(Modifier::KeepHigh(num));
    }
    
    if let Some(stripped) = part.strip_prefix('d') {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid drop value"))?;
        if num == 0 {
            return Err(anyhow!("Cannot drop 0 dice"));
        }
        return Ok(Modifier::Drop(num));
    }
    
    if let Some(stripped) = part.strip_prefix('t') {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid target value"))?;
        if num == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }
        return Ok(Modifier::Target(num));
    }
    
    if let Some(stripped) = part.strip_prefix('f') {
        let num = stripped.parse().map_err(|_| anyhow!("Invalid failure value"))?;
        return Ok(Modifier::Failure(num));
    }
    
    if let Some(stripped) = part.strip_prefix('b') {
        let num = if stripped.is_empty() { None } else { Some(stripped.parse()?) };
        return Ok(Modifier::Botch(num));
    }

    // Additional dice modifiers
    if let Some(captures) = DICE_MOD_REGEX.captures(part) {
        let count: u32 = captures[2].parse()?;
        let sides: u32 = captures[3].parse()?;
        let dice_roll = DiceRoll {
            count, sides,
            modifiers: Vec::new(),
            comment: None, label: None,
            private: false, simple: false, no_results: false, unsorted: false,
            original_expression: None,
        };
        return match &captures[1] {
            "+" => Ok(Modifier::AddDice(dice_roll)),
            "-" => Ok(Modifier::SubtractDice(dice_roll)),
            _ => Err(anyhow!("Invalid dice modifier")),
        };
    }

    // Plain numbers
    if let Ok(num) = part.parse::<i32>() {
        return Ok(Modifier::Add(num));
    }

    Err(anyhow!("Unknown modifier: {}", part))
}

fn is_dice_expression(input: &str) -> bool {
    DICE_ONLY_REGEX.is_match(input)
}

fn parse_dice_expression_only(input: &str) -> Result<DiceRoll> {
    if let Some(captures) = DICE_ONLY_REGEX.captures(input) {
        let count = captures.get(1).map(|m| m.as_str().parse().unwrap_or(1)).unwrap_or(1);
        let sides = if &captures[2] == "%" { 100 } else { captures[2].parse()? };
        
        Ok(DiceRoll {
            count, sides,
            modifiers: Vec::new(),
            comment: None, label: None,
            private: false, simple: false, no_results: false, unsorted: false,
            original_expression: None,
        })
    } else {
        Err(anyhow!("Invalid dice expression: {}", input))
    }
}
