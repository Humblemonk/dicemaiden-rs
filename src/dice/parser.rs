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

static DICE_MOD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+\-])(\d+)d(\d+)$").expect("Failed to compile DICE_MOD_REGEX"));

// Add regex for advantage/disadvantage patterns with modifiers
static ADV_WITH_MOD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([+-])d(\d+|%)\s*([+-]\s*\d+.*)?$").expect("Failed to compile ADV_WITH_MOD_REGEX")
});

pub fn parse_dice_string(input: &str) -> Result<Vec<DiceRoll>> {
    let input = input.trim();

    // Check for advantage/disadvantage with modifiers FIRST before other alias expansion
    if let Some(captures) = ADV_WITH_MOD_REGEX.captures(input) {
        let advantage_sign = &captures[1];
        let sides = &captures[2];
        let modifier_part = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        // Expand the advantage/disadvantage part
        let adv_alias = format!("{}d{}", advantage_sign, sides);
        if let Some(expanded_adv) = super::aliases::expand_alias(&adv_alias) {
            // Combine with the modifier part
            let full_expression = if modifier_part.is_empty() {
                expanded_adv
            } else {
                format!("{} {}", expanded_adv, modifier_part)
            };
            return Ok(vec![parse_single_dice_expression(&full_expression)?]);
        }
    }

    // Check for aliases that expand to roll sets
    if let Some(expanded) = super::aliases::expand_alias(input) {
        if expanded.contains(' ') && !expanded.contains(';') {
            // Check if this is a roll set pattern
            if let Some(captures) = SET_REGEX.captures(&expanded) {
                return create_roll_set(&captures);
            }
        }
        // If alias doesn't expand to roll set, parse normally
        return Ok(vec![parse_single_dice_expression(&expanded)?]);
    }

    // Check for multi-roll (semicolon separated)
    if input.contains(';') {
        return parse_semicolon_separated_rolls(input);
    }

    // Check for roll sets
    if let Some(captures) = SET_REGEX.captures(input) {
        return create_roll_set(&captures);
    }

    // Parse single expression
    Ok(vec![parse_single_dice_expression(input)?])
}

// Helper function to create roll sets, eliminating duplication
fn create_roll_set(captures: &regex::Captures) -> Result<Vec<DiceRoll>> {
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
    Ok(results)
}

// Helper function to parse semicolon-separated rolls
fn parse_semicolon_separated_rolls(input: &str) -> Result<Vec<DiceRoll>> {
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
    Ok(results)
}

fn parse_single_dice_expression(input: &str) -> Result<DiceRoll> {
    let mut dice = create_default_dice_roll();

    // Normalize and handle the input
    let normalized_input = normalize_whitespace(input.trim());
    let mut remaining = normalized_input.as_str();

    // Parse flags, labels, and comments
    remaining = parse_flags(&mut dice, remaining);
    remaining = parse_label(&mut dice, remaining);
    remaining = parse_comment(&mut dice, remaining);
    remaining = remaining.trim();

    // Check for aliases after parsing flags/comments but before advantage/disadvantage check
    if let Some(expanded) = super::aliases::expand_alias(remaining) {
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;

        // Transfer flags and metadata
        transfer_dice_metadata(&dice, &mut expanded_dice);

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

// Helper function to create default dice roll, eliminating duplication
fn create_default_dice_roll() -> DiceRoll {
    DiceRoll {
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
    }
}

// Helper function to transfer metadata between dice rolls
fn transfer_dice_metadata(source: &DiceRoll, target: &mut DiceRoll) {
    target.private = source.private;
    target.simple = source.simple;
    target.no_results = source.no_results;
    target.unsorted = source.unsorted;
    target.comment = source.comment.clone();
    target.label = source.label.clone();
}

fn normalize_whitespace(input: &str) -> String {
    let whitespace_regex = Regex::new(r"\s+").unwrap();
    whitespace_regex.replace_all(input.trim(), " ").to_string()
}

// Handle both spaced and combined expressions
fn parse_expression_to_parts(input: &str) -> Result<Vec<String>> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    // Normalize whitespace first
    let normalized = normalize_whitespace(input);

    // Check if this is entirely without spaces and contains dice
    if !normalized.contains(' ') && normalized.contains('d') {
        return parse_combined_expression(&normalized);
    }

    // Handle mixed expressions (some spaces, some combined)
    let mut parts = Vec::new();
    let mut current_token = String::new();
    let mut chars = normalized.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' => {
                // Whitespace - finish current token if any
                if !current_token.is_empty() {
                    process_current_token(&mut parts, &mut current_token)?;
                }
                // Skip whitespace
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '+' | '-' | '*' | '/' => {
                // Mathematical operators - finish current token and add operator
                if !current_token.is_empty() {
                    process_current_token(&mut parts, &mut current_token)?;
                }
                parts.push(ch.to_string());
            }
            _ => {
                // Regular character - add to current token
                current_token.push(ch);
            }
        }
    }

    // Add final token if any
    if !current_token.is_empty() {
        process_final_token(&mut parts, &current_token)?;
    }

    Ok(parts)
}

// Helper function to process current token, reducing duplication
fn process_current_token(parts: &mut Vec<String>, current_token: &mut String) -> Result<()> {
    if current_token.contains('d') && has_attached_modifiers(current_token) {
        handle_combined_token_with_operator(parts, current_token)?;
    } else {
        parts.push(current_token.clone());
    }
    current_token.clear();
    Ok(())
}

// Helper function to handle combined tokens with trailing operators
fn handle_combined_token_with_operator(parts: &mut Vec<String>, current_token: &str) -> Result<()> {
    if current_token.ends_with(['+', '-', '*', '/']) {
        let op_pos = current_token.len() - 1;
        let main_part = &current_token[..op_pos];
        let op_part = &current_token[op_pos..];

        if main_part.contains('d') {
            let combined_parts = parse_combined_expression(main_part)?;
            parts.extend(combined_parts);
            parts.push(op_part.to_string());
        } else {
            parts.push(current_token.to_string());
        }
    } else {
        let combined_parts = parse_combined_expression(current_token)?;
        parts.extend(combined_parts);
    }
    Ok(())
}

// Helper function to process final token
fn process_final_token(parts: &mut Vec<String>, current_token: &str) -> Result<()> {
    if current_token.contains('d') && has_attached_modifiers(current_token) {
        handle_combined_token_with_operator(parts, current_token)?;
    } else {
        parts.push(current_token.to_string());
    }
    Ok(())
}

// Check if a dice expression has modifiers attached
fn has_attached_modifiers(input: &str) -> bool {
    // Check if it's just a simple dice expression
    if DICE_ONLY_REGEX.is_match(input) {
        return false;
    }

    // Check if it contains dice and additional characters that could be modifiers
    if let Some((_, modifiers_part)) = split_dice_and_modifiers(input) {
        !modifiers_part.is_empty()
    } else {
        false
    }
}

// Parse combined expressions like "4d6e6k3+2"
fn parse_combined_expression(input: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();

    // First, extract any mathematical operators at the end
    let (main_part, math_part) = split_math_operators(input);

    // Parse the main dice + modifiers part
    if let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(main_part) {
        parts.push(dice_part);

        // Split the modifiers part into individual modifiers
        let modifier_parts = split_combined_modifiers(&modifiers_part)?;
        parts.extend(modifier_parts);
    } else {
        // Not a dice expression, treat as single part
        parts.push(main_part.to_string());
    }

    // Add any mathematical parts
    parts.extend(math_part);

    Ok(parts)
}

// Split input into main part and mathematical operators
fn split_math_operators(input: &str) -> (&str, Vec<String>) {
    let mut split_pos = input.len();
    let mut math_parts = Vec::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = chars.len();

    // Work backwards to find mathematical operations
    while i > 0 {
        i -= 1;
        let ch = chars[i];

        match ch {
            '+' | '-' | '*' | '/' => {
                // Found operator, extract it and the operand that follows
                let op = ch.to_string();
                let start = i + 1;
                let mut end = start;

                // Find the complete operand (could be number or dice expression)
                while end < chars.len() {
                    let operand_char = chars[end];
                    if operand_char.is_ascii_digit() || operand_char == 'd' {
                        end += 1;
                    } else if operand_char.is_ascii_alphabetic() && end > start {
                        // Could be part of a modifier like "e6" in "1d4e4"
                        end += 1;
                    } else {
                        break;
                    }
                }

                if end > start {
                    let operand = chars[start..end].iter().collect::<String>();
                    math_parts.insert(0, operand);
                }
                math_parts.insert(0, op);
                split_pos = i;

                // Continue looking for more operators
                continue;
            }
            _ if ch.is_ascii_digit() || ch.is_ascii_alphabetic() => {
                // Part of an operand, continue
                continue;
            }
            _ => {
                // Not part of math operation, stop here
                break;
            }
        }
    }

    (&input[..split_pos], math_parts)
}

// Split dice expression from modifiers: "4d6e6k3" -> ("4d6", "e6k3")
fn split_dice_and_modifiers(input: &str) -> Option<(String, String)> {
    // Match basic dice pattern
    let dice_regex = Regex::new(r"^(\d*d\d+)(.*)$").unwrap();
    if let Some(captures) = dice_regex.captures(input) {
        let dice_part = captures[1].to_string();
        let modifiers_part = captures[2].to_string();

        if modifiers_part.is_empty() {
            Some((dice_part, String::new()))
        } else {
            Some((dice_part, modifiers_part))
        }
    } else {
        None
    }
}

// Split combined modifiers like "e6k3r1" into ["e6", "k3", "r1"]
fn split_combined_modifiers(input: &str) -> Result<Vec<String>> {
    if input.is_empty() {
        return Ok(vec![]);
    }

    // First, check if there's a trailing operator and separate it
    let (modifiers_part, _trailing_op) = if input.ends_with(['+', '-', '*', '/']) {
        let op_pos = input.len() - 1;
        (&input[..op_pos], Some(&input[op_pos..]))
    } else {
        (input, None)
    };

    let mut modifiers = Vec::new();
    let mut remaining = modifiers_part;

    while !remaining.is_empty() {
        let original_remaining = remaining;

        // Try to extract known modifier patterns
        let patterns = [
            r"^(ie\d*)",    // Indefinite explode first (longer pattern)
            r"^(ir\d+)",    // Indefinite reroll
            r"^(kl\d+)",    // Keep lowest
            r"^(e\d*)",     // Explode (after ie)
            r"^(k\d+)",     // Keep highest
            r"^(r\d+)",     // Reroll (after ir)
            r"^(d\d+)",     // Drop
            r"^(t\d+)",     // Target
            r"^(f\d+)",     // Failure
            r"^(b\d*)",     // Botch
            r"^(wng\d*t?)", // Wrath & Glory
            r"^(gb|gbs)",   // Godbound
            r"^(hs[nkh])",  // Hero System
            r"^(dh)",       // Dark Heresy
            r"^(fudge|df)", // Fudge
        ];

        let mut found = false;
        for pattern in &patterns {
            let regex = Regex::new(pattern).unwrap();
            if let Some(captures) = regex.captures(remaining) {
                let modifier = captures[1].to_string();
                modifiers.push(modifier.clone());
                remaining = &remaining[modifier.len()..];
                found = true;
                break;
            }
        }

        if !found {
            // If we can't parse any more modifiers, treat the rest as one piece
            if !remaining.is_empty() {
                modifiers.push(remaining.to_string());
            }
            break;
        }

        // Safety check to prevent infinite loops
        if remaining == original_remaining {
            break;
        }
    }

    Ok(modifiers)
}

// Better flag parsing with proper whitespace handling
fn parse_flags<'a>(dice: &mut DiceRoll, mut remaining: &'a str) -> &'a str {
    let flags = ["p", "s", "nr", "ul"];

    let mut changed = true;
    while changed {
        changed = false;
        remaining = remaining.trim_start();

        for &flag in &flags {
            if remaining.starts_with(flag) {
                // Check if it's a complete flag (followed by space or end of string)
                let after_flag = &remaining[flag.len()..];
                if after_flag.is_empty()
                    || after_flag.starts_with(' ')
                    || after_flag.starts_with('\t')
                {
                    match flag {
                        "p" => dice.private = true,
                        "s" => dice.simple = true,
                        "nr" => dice.no_results = true,
                        "ul" => dice.unsorted = true,
                        _ => {}
                    }
                    remaining = remaining[flag.len()..].trim_start();
                    changed = true;
                    break;
                }
            }
        }
    }

    remaining
}

fn parse_label<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if let Some(captures) = LABEL_REGEX.captures(remaining) {
        let label_content = &captures[1];
        let trimmed = label_content.trim();
        dice.label = Some(trimmed.to_string());
        return &remaining[captures.get(0).unwrap().end()..];
    }
    remaining
}

fn parse_comment<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if remaining.trim() == "!" {
        dice.comment = Some("".to_string());
        return "";
    }

    if let Some(captures) = COMMENT_REGEX.captures(remaining) {
        let comment_content = captures[1].trim();

        // Handle semicolon in comments - take everything before first semicolon
        if comment_content.contains(';') {
            if let Some(first_part) = comment_content.split(';').next() {
                let trimmed = first_part.trim();
                dice.comment = Some(trimmed.to_string());
            } else {
                dice.comment = Some("".to_string());
            }
        } else {
            dice.comment = Some(comment_content.to_string());
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

    // Check if this is a simple dice expression
    if DICE_ONLY_REGEX.is_match(part) {
        return parse_simple_dice_part(dice, part);
    }

    // If it's not a simple dice expression, it might be a combined expression
    // that needs to be split further
    if part.contains('d') {
        if let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(part) {
            // Parse the dice part
            parse_simple_dice_part(dice, &dice_part)?;

            // Parse the modifiers part if any
            parse_modifiers_from_part(dice, &modifiers_part)?;
            return Ok(());
        }
    }

    Err(anyhow!("Invalid dice expression: {}", part))
}

// Helper function to parse modifiers from a part, reducing duplication
fn parse_modifiers_from_part(dice: &mut DiceRoll, modifiers_part: &str) -> Result<()> {
    if !modifiers_part.is_empty() {
        let modifier_parts = split_combined_modifiers(modifiers_part)?;
        for modifier_part in modifier_parts {
            let modifier = parse_single_modifier(&modifier_part)?;
            dice.modifiers.push(modifier);
        }
    }
    Ok(())
}

fn parse_simple_dice_part(dice: &mut DiceRoll, part: &str) -> Result<()> {
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

        // Check if this part is combined modifiers before parsing as single
        let part = &parts[i];
        if is_combined_modifiers_token(part) {
            let modifier_parts = split_combined_modifiers(part)?;

            for modifier_part in modifier_parts {
                let modifier = parse_single_modifier(&modifier_part)?;
                dice.modifiers.push(modifier);
            }
        } else {
            // Parse single modifier
            let modifier = parse_single_modifier(part)?;
            dice.modifiers.push(modifier);
        }
        i += 1;
    }
    Ok(())
}

// Helper function to check if a token is combined modifiers
fn is_combined_modifiers_token(input: &str) -> bool {
    if input.is_empty() || input.contains('d') {
        return false;
    }

    // Check if it starts with common modifier patterns and has more after the first one
    let modifier_patterns = [
        r"^(e\d*)",
        r"^(ie\d*)",
        r"^(k\d+)",
        r"^(kl\d+)",
        r"^(d\d+)",
        r"^(r\d+)",
        r"^(ir\d+)",
        r"^(t\d+)",
        r"^(f\d+)",
        r"^(b\d*)",
    ];

    for pattern in &modifier_patterns {
        let regex = Regex::new(pattern).unwrap();
        if let Some(captures) = regex.captures(input) {
            let first_modifier = &captures[1];

            // If the match is the entire string, it's a single modifier, not combined
            if first_modifier.len() == input.len() {
                return false;
            }

            // If there's more after the first modifier, check if it looks like more modifiers
            let remaining = &input[first_modifier.len()..];
            return is_modifier_start(remaining);
        }
    }

    false
}

// Check if a string starts with modifier patterns
fn is_modifier_start(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    let modifier_starts = [
        r"^e\d*",
        r"^ie\d*",
        r"^k\d+",
        r"^kl\d+",
        r"^d\d+",
        r"^r\d+",
        r"^ir\d+",
        r"^t\d+",
        r"^f\d+",
        r"^b\d*",
        r"^wng",
        r"^gb",
        r"^gbs",
        r"^hs[nkh]",
        r"^dh",
        r"^fudge",
        r"^df",
    ];

    for pattern in &modifier_starts {
        let regex = Regex::new(pattern).unwrap();
        if regex.is_match(input) {
            return true;
        }
    }

    false
}

fn try_parse_operator_pair(
    dice: &mut DiceRoll,
    first: &str,
    second: &str,
) -> Result<Option<usize>> {
    match first {
        "+" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Add(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::AddDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // This might be a complex dice expression like "1d4e4"
                let dice_roll = parse_complex_dice_expression(second)?;
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
            } else if second.contains('d') {
                // This might be a complex dice expression like "1d4e4"
                let dice_roll = parse_complex_dice_expression(second)?;
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

// Parse complex dice expressions that might have modifiers attached
fn parse_complex_dice_expression(input: &str) -> Result<DiceRoll> {
    if is_dice_expression(input) {
        // Simple dice expression
        return parse_dice_expression_only(input);
    }

    // Complex expression with modifiers - parse as a mini dice expression
    let mut dice = create_default_dice_roll();

    if let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(input) {
        // Parse the dice part
        parse_simple_dice_part(&mut dice, &dice_part)?;

        // Parse the modifiers part if any
        parse_modifiers_from_part(&mut dice, &modifiers_part)?;
        Ok(dice)
    } else {
        Err(anyhow!("Invalid complex dice expression: {}", input))
    }
}

// Helper function to parse exploding dice values, reducing duplication
fn parse_explode_value(stripped: &str, part: &str, modifier_name: &str) -> Result<Option<u32>> {
    if stripped.is_empty() {
        Ok(None)
    } else {
        let val = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid {} value in '{}'", modifier_name, part))?;
        if val == 0 {
            return Err(anyhow!("Cannot explode on 0"));
        }
        Ok(Some(val))
    }
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

    // Check for invalid characters before parsing numbers
    if part.contains(['+', '-', '*', '/']) {
        return Err(anyhow!("Invalid modifier '{}' - contains operator", part));
    }

    // Handle exploding dice
    if let Some(stripped) = part.strip_prefix("ie") {
        let num = parse_explode_value(stripped, part, "explode")?;
        return Ok(Modifier::ExplodeIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix('e') {
        let num = parse_explode_value(stripped, part, "explode")?;
        return Ok(Modifier::Explode(num));
    }

    // Handle drop modifier
    if let Some(stripped) = part.strip_prefix('d') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid drop value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot drop 0 dice"));
        }
        return Ok(Modifier::Drop(num));
    }

    // Continue with other modifiers...
    if let Some(stripped) = part.strip_prefix("ir") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::RerollIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix('r') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::Reroll(num));
    }

    if let Some(stripped) = part.strip_prefix("kl") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid keep low value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot keep 0 dice"));
        }
        return Ok(Modifier::KeepLow(num));
    }

    if let Some(stripped) = part.strip_prefix('k') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid keep value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot keep 0 dice"));
        }
        return Ok(Modifier::KeepHigh(num));
    }

    if let Some(stripped) = part.strip_prefix('t') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid target value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }
        return Ok(Modifier::Target(num));
    }

    if let Some(stripped) = part.strip_prefix('f') {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid failure value in '{}'", part))?;
        return Ok(Modifier::Failure(num));
    }

    if let Some(stripped) = part.strip_prefix('b') {
        let num = if stripped.is_empty() {
            None
        } else {
            Some(
                stripped
                    .parse()
                    .map_err(|_| anyhow!("Invalid botch value in '{}'", part))?,
            )
        };
        return Ok(Modifier::Botch(num));
    }

    // Wrath & Glory handling
    if let Some(stripped) = part.strip_prefix("wng") {
        if stripped.is_empty() {
            return Ok(Modifier::WrathGlory(None, false));
        } else if stripped == "t" {
            return Ok(Modifier::WrathGlory(None, true));
        } else if let Some(dn_part) = stripped.strip_prefix("dn") {
            if let Some(dn_str) = dn_part.strip_suffix('t') {
                let dn = dn_str
                    .parse()
                    .map_err(|_| anyhow!("Invalid difficulty value in '{}'", part))?;
                return Ok(Modifier::WrathGlory(Some(dn), true));
            } else {
                let dn = dn_part
                    .parse()
                    .map_err(|_| anyhow!("Invalid difficulty value in '{}'", part))?;
                return Ok(Modifier::WrathGlory(Some(dn), false));
            }
        } else if let Some(dn_str) = stripped.strip_suffix('t') {
            if let Ok(dn) = dn_str.parse::<u32>() {
                return Ok(Modifier::WrathGlory(Some(dn), true));
            }
        } else if let Ok(dn) = stripped.parse::<u32>() {
            return Ok(Modifier::WrathGlory(Some(dn), false));
        }

        return Err(anyhow!("Invalid Wrath & Glory modifier: {}", part));
    }

    // Additional dice modifiers
    if let Some(captures) = DICE_MOD_REGEX.captures(part) {
        let count: u32 = captures[2].parse()?;
        let sides: u32 = captures[3].parse()?;
        let dice_roll = DiceRoll {
            count,
            sides,
            modifiers: Vec::new(),
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };
        return match &captures[1] {
            "+" => Ok(Modifier::AddDice(dice_roll)),
            "-" => Ok(Modifier::SubtractDice(dice_roll)),
            _ => Err(anyhow!("Invalid dice modifier")),
        };
    }

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
        let count = captures
            .get(1)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);
        let sides = if &captures[2] == "%" {
            100
        } else {
            captures[2].parse()?
        };

        Ok(DiceRoll {
            count,
            sides,
            modifiers: Vec::new(),
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        })
    } else {
        Err(anyhow!("Invalid dice expression: {}", input))
    }
}
