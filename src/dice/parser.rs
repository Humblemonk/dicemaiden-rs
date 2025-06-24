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
    Lazy::new(|| Regex::new(r"^\(([^)]+)\)\s*").expect("Failed to compile LABEL_REGEX"));

static COMMENT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"!\s*(.+)$").expect("Failed to compile COMMENT_REGEX"));

static OP_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+\-*/])(\d+)$").expect("Failed to compile OP_REGEX"));

static DICE_MOD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+\-])(\d+)d(\d+)$").expect("Failed to compile DICE_MOD_REGEX"));

static WNG_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(wng\d*t?)").expect("Failed to compile WNG_PATTERN"));

static IE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(ie\d*)").expect("Failed to compile IE_PATTERN"));

static IR_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(ir\d+)").expect("Failed to compile IR_PATTERN"));

static KL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(kl\d+)").expect("Failed to compile KL_PATTERN"));

static E_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(e\d*)").expect("Failed to compile E_PATTERN"));

static K_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(k\d+)").expect("Failed to compile K_PATTERN"));

static R_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(r\d+)").expect("Failed to compile R_PATTERN"));

static D_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(d\d+)").expect("Failed to compile D_PATTERN"));

static T_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(t\d+)").expect("Failed to compile T_PATTERN"));

static F_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(f\d+)").expect("Failed to compile F_PATTERN"));

static B_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(b\d*)").expect("Failed to compile B_PATTERN"));

static ALIAS_WITH_MODIFIERS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d*df)([+\-*/].*)$").expect("Failed to compile ALIAS_WITH_MODIFIERS_REGEX")
});

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

    let mut remaining = input.trim();

    // Parse flags at the beginning
    remaining = parse_flags(&mut dice, remaining);

    // Parse label (parentheses)
    remaining = parse_label(&mut dice, remaining);

    // Parse comment (exclamation mark)
    remaining = parse_comment(&mut dice, remaining);

    // NOW check for aliases after flags/comments are stripped
    if let Some(expanded) = super::aliases::expand_alias(remaining) {
        // Recursively parse the expanded alias, but preserve the flags/comment from the original
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;

        // Transfer the parsed flags and comment to the expanded dice
        expanded_dice.private = dice.private;
        expanded_dice.simple = dice.simple;
        expanded_dice.no_results = dice.no_results;
        expanded_dice.unsorted = dice.unsorted;
        expanded_dice.comment = dice.comment;
        expanded_dice.label = dice.label;

        return Ok(expanded_dice);
    }

    // Continue with regular parsing...
    let parts: Vec<String> = if remaining.contains(' ') {
        let initial_parts: Vec<&str> = remaining.split_whitespace().collect();
        split_space_separated_parts(&initial_parts)?
    } else {
        split_dice_and_modifiers(remaining)?
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    };

    if parts.is_empty() {
        return Err(anyhow!("No dice expression found"));
    }

    // Parse main dice part
    parse_base_dice(&mut dice, &parts[0])?;

    // Parse all remaining parts as modifiers
    let string_parts: Vec<&str> = parts.iter().skip(1).map(|s| s.as_str()).collect();
    parse_all_modifiers(&mut dice, &string_parts)?;

    Ok(dice)
}

// Function to handle space-separated parts that might have combined modifiers
fn split_space_separated_parts(parts: &[&str]) -> Result<Vec<String>> {
    let mut result = Vec::new();

    for (i, &part) in parts.iter().enumerate() {
        if i == 0 {
            // First part should be dice, but might have modifiers attached
            if part.contains("d") {
                // This looks like a dice expression, try to split it
                let split_parts = split_dice_and_modifiers(part)?;
                result.extend(split_parts.into_iter().map(|s| s.to_string()));
            } else {
                result.push(part.to_string());
            }
        } else {
            // Handle subsequent parts - they could be operators, dice expressions, or modifiers
            if part.starts_with('+')
                || part.starts_with('-')
                || part.starts_with('*')
                || part.starts_with('/')
            {
                // Check if this is a dice expression like "+1d6+15" or just a modifier like "+5"
                if part.contains("d") {
                    // This contains dice, so we need to split it properly
                    let split_parts = split_dice_and_modifiers_from_operator(part)?;
                    result.extend(split_parts.into_iter().map(|s| s.to_string()));
                } else {
                    // This is a mathematical operator with possibly attached modifiers like "+8k2"
                    let split_parts = split_modifier_combinations(part)?;
                    result.extend(split_parts.into_iter().map(|s| s.to_string()));
                }
            } else {
                // This could be a standalone modifier or number
                result.push(part.to_string());
            }
        }
    }

    Ok(result)
}

// New function to handle operator-starting expressions that might contain dice
fn split_dice_and_modifiers_from_operator(input: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();

    // Use the same regex pattern as in split_dice_and_modifiers
    let modifier_regex = Regex::new(r"([+\-*/]\d*d\d+|[+\-*/]\d+|[a-zA-Z]+\d*)").unwrap();
    for modifier_match in modifier_regex.find_iter(input) {
        parts.push(modifier_match.as_str().to_string());
    }

    // Validate that we've consumed the entire string
    let reconstructed: String = parts.iter().map(|s| s.as_str()).collect();
    if reconstructed != input {
        return Err(anyhow!("Unable to parse modifier expression: {}", input));
    }

    Ok(parts)
}

// Function to split combined modifiers like "+8k2" into ["+8", "k2"]
fn split_modifier_combinations(input: &str) -> Result<Vec<String>> {
    // Handle mathematical operators with attached modifiers
    if let Some(first_char) = input.chars().next() {
        if matches!(first_char, '+' | '-' | '*' | '/') {
            // Find where the number ends and modifiers begin
            let mut number_end = 1; // Start after the operator
            while number_end < input.len()
                && input.chars().nth(number_end).unwrap().is_ascii_digit()
            {
                number_end += 1;
            }

            if number_end == input.len() {
                // Just an operator and number, no attached modifiers
                return Ok(vec![input.to_string()]);
            }

            // Split into operator+number and remaining modifiers
            let math_part = &input[..number_end];
            let modifier_part = &input[number_end..];

            let mut result = vec![math_part.to_string()];

            // Split the remaining modifiers
            let modifier_regex = Regex::new(r"([a-zA-Z]+\d*)").unwrap();
            for modifier_match in modifier_regex.find_iter(modifier_part) {
                result.push(modifier_match.as_str().to_string());
            }

            return Ok(result);
        }
    }

    // If it doesn't start with a mathematical operator, just return as-is
    Ok(vec![input.to_string()])
}

// Function to intelligently split dice expressions like "2d20+8" into ["2d20", "+8"]
fn split_dice_and_modifiers(input: &str) -> Result<Vec<String>> {
    // First, try to find standard dice patterns (e.g., "2d20+8")
    let dice_regex = Regex::new(r"^\d*d\d+").unwrap();
    if let Some(dice_match) = dice_regex.find(input) {
        let dice_part = dice_match.as_str();
        let rest = &input[dice_match.end()..];

        if rest.is_empty() {
            return Ok(vec![dice_part.to_string()]);
        }

        // Now split the rest into modifiers
        let mut parts = vec![dice_part.to_string()];

        // FIXED: Updated regex to properly handle dice expressions with operators
        // The key fix: [+\-*/]\d*d\d+ matches dice expressions like +1d6, -2d8
        let modifier_regex = Regex::new(r"([+\-*/]\d*d\d+|[+\-*/]\d+|[a-zA-Z]+\d*)").unwrap();
        for modifier_match in modifier_regex.find_iter(rest) {
            parts.push(modifier_match.as_str().to_string());
        }

        // Validate that we've consumed the entire string
        let reconstructed: String = parts.iter().skip(1).map(|s| s.as_str()).collect();
        if reconstructed != rest {
            return Err(anyhow!(
                "Unable to parse dice expression: {} (parsed: {} vs original: {})",
                input,
                reconstructed,
                rest
            ));
        }

        Ok(parts)
    } else if let Some(captures) = ALIAS_WITH_MODIFIERS_REGEX.captures(input) {
        // Handle fudge dice with modifiers (e.g., "4df+1")
        let alias_part = captures[1].to_string();
        let modifier_part = &captures[2];

        let mut parts = vec![alias_part];

        // Split the modifier part - also needs the same fix for dice expressions
        let modifier_regex = Regex::new(r"([+\-*/]\d*d\d+|[+\-*/]\d+|[a-zA-Z]+\d*)").unwrap();
        for modifier_match in modifier_regex.find_iter(modifier_part) {
            parts.push(modifier_match.as_str().to_string());
        }

        Ok(parts)
    } else {
        // If no pattern matched, just return the original as one part
        Ok(vec![input.to_string()])
    }
}

// Helper function to parse flags
fn parse_flags<'a>(dice: &mut DiceRoll, mut remaining: &'a str) -> &'a str {
    let flags = ["p ", "s ", "nr ", "ul ", "!"];
    for flag in &flags {
        if remaining.starts_with(flag) {
            match *flag {
                "p " => dice.private = true,
                "s " => dice.simple = true,
                "nr " => dice.no_results = true,
                "ul " => dice.unsorted = true,
                _ => {}
            }
            remaining = &remaining[flag.len()..];
        }
    }
    remaining
}

// Helper function to parse label using pre-compiled regex
fn parse_label<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if let Some(captures) = LABEL_REGEX.captures(remaining) {
        dice.label = Some(captures[1].to_string());
        &remaining[captures.get(0).unwrap().end()..]
    } else {
        remaining
    }
}

// Helper function to parse comment using pre-compiled regex
fn parse_comment<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    if let Some(captures) = COMMENT_REGEX.captures(remaining) {
        dice.comment = Some(captures[1].to_string());
        remaining[..captures.get(0).unwrap().start()].trim()
    } else {
        remaining
    }
}

// Helper function to parse base dice expression using pre-compiled regex
fn parse_base_dice(dice: &mut DiceRoll, part: &str) -> Result<()> {
    // First try to expand as alias
    if let Some(expanded) = super::aliases::expand_alias(part) {
        // Parse the expanded expression and extract the base dice
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

// Helper function to parse all modifiers
fn parse_all_modifiers(dice: &mut DiceRoll, parts: &[&str]) -> Result<()> {
    let mut i = 0;
    while i < parts.len() {
        // Handle operators followed by numbers or dice expressions
        if let Some(consumed) = try_parse_operator_modifier(dice, parts, i)? {
            i += consumed;
            continue;
        }

        // Handle combined modifiers by splitting them first
        let split_modifiers = split_combined_modifiers(parts[i])?;
        for modifier_str in split_modifiers {
            let modifier = parse_modifier(&modifier_str)?;
            dice.modifiers.push(modifier);
        }
        i += 1;
    }
    Ok(())
}

// Helper function to try parsing operator-based modifiers
fn try_parse_operator_modifier(
    dice: &mut DiceRoll,
    parts: &[&str],
    i: usize,
) -> Result<Option<usize>> {
    if i + 1 >= parts.len() {
        return Ok(None);
    }

    let operator = parts[i];
    let next_part = parts[i + 1];

    let modifier = match operator {
        "+" => {
            if let Ok(num) = next_part.parse::<i32>() {
                Some(Modifier::Add(num))
            } else if is_dice_expression(next_part) {
                let additional_dice = parse_dice_expression_only(next_part)?;
                Some(Modifier::AddDice(additional_dice))
            } else {
                None
            }
        }
        "-" => {
            if let Ok(num) = next_part.parse::<i32>() {
                Some(Modifier::Subtract(num))
            } else if is_dice_expression(next_part) {
                let additional_dice = parse_dice_expression_only(next_part)?;
                Some(Modifier::SubtractDice(additional_dice))
            } else {
                None
            }
        }
        "*" => {
            if let Ok(num) = next_part.parse::<i32>() {
                Some(Modifier::Multiply(num))
            } else {
                None
            }
        }
        "/" => {
            if let Ok(num) = next_part.parse::<i32>() {
                Some(Modifier::Divide(num))
            } else {
                None
            }
        }
        _ => None,
    };

    if let Some(mod_to_add) = modifier {
        dice.modifiers.push(mod_to_add);
        Ok(Some(2)) // Consumed operator + operand
    } else {
        Ok(None)
    }
}

// Optimized modifier pattern extraction using pre-compiled regex patterns
fn extract_modifier_pattern(input: &str) -> Option<String> {
    // Direct string matching for simple cases (most efficient)
    match input {
        "hsh" | "hsk" | "hsn" | "gb" | "gbs" => return Some(input.to_string()),
        _ => {}
    }

    // Check for excluded prefixes first
    if input.starts_with("gbs") && input != "gbs" {
        return None; // Don't match "gb" if it starts with "gbs"
    }

    // Pattern matching with pre-compiled regex for complex cases
    if input.starts_with("wng") {
        if let Some(captures) = WNG_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with("ie") {
        if let Some(captures) = IE_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with("ir") {
        if let Some(captures) = IR_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with("kl") {
        if let Some(captures) = KL_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('e') && !input.starts_with("ie") {
        if let Some(captures) = E_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('k') && !input.starts_with("kl") {
        if let Some(captures) = K_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('r') && !input.starts_with("ir") {
        if let Some(captures) = R_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('d') {
        if let Some(captures) = D_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('t') {
        if let Some(captures) = T_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('f') {
        if let Some(captures) = F_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    } else if input.starts_with('b') {
        if let Some(captures) = B_PATTERN.captures(input) {
            return Some(captures[1].to_string());
        }
    }

    None
}

// Function to split combined modifiers like "e6k8" into ["e6", "k8"]
fn split_combined_modifiers(input: &str) -> Result<Vec<String>> {
    // If it's a simple modifier, return as-is
    if is_simple_modifier(input) {
        return Ok(vec![input.to_string()]);
    }

    let mut modifiers = Vec::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        let (modifier, rest) = extract_next_modifier(remaining)?;
        if modifier.is_empty() {
            break;
        }
        modifiers.push(modifier);
        remaining = rest;
    }

    if modifiers.is_empty() {
        // If we couldn't split it, return the original
        Ok(vec![input.to_string()])
    } else {
        Ok(modifiers)
    }
}

fn is_simple_modifier(input: &str) -> bool {
    // Check for exact matches first (most common)
    match input {
        "gb" | "gbs" | "ie" | "hsn" | "hsk" | "hsh" | "fudge" | "df" => return true,
        _ => {}
    }

    // Check for simple numeric modifiers
    if input.len() <= 6 {
        // Check for patterns like "+5", "-3", "*2", "/4"
        if let Some(first_char) = input.chars().next() {
            match first_char {
                '+' | '-' | '*' | '/' => {
                    return input.len() > 1 && input[1..].chars().all(|c| c.is_ascii_digit())
                }
                '0'..='9' => return input.chars().all(|c| c.is_ascii_digit()),
                _ => {}
            }
        }
    }

    // Check for simple modifiers with numbers like "ie6", "k3", "t7", etc.
    if input.len() >= 2 && input.len() <= 5 {
        if let Some(stripped) = input.strip_prefix("ie") {
            return stripped.chars().all(|c| c.is_ascii_digit());
        }
        if let Some(stripped) = input.strip_prefix("ir") {
            return stripped.chars().all(|c| c.is_ascii_digit());
        }
        if let Some(stripped) = input.strip_prefix("kl") {
            return stripped.chars().all(|c| c.is_ascii_digit());
        }
        if let Some(stripped) = input.strip_prefix('e') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix('k') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix('r') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix('d') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix('t') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix('f') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix('b') {
            return stripped.chars().all(|c| c.is_ascii_digit()) || stripped.is_empty();
        }
        if let Some(stripped) = input.strip_prefix("wng") {
            return stripped.chars().all(|c| c.is_ascii_digit() || c == 't') || input == "wng";
        }
    }

    false
}

fn extract_next_modifier(input: &str) -> Result<(String, &str)> {
    // Try to extract a modifier pattern from the input
    if let Some(modifier) = extract_modifier_pattern(input) {
        let rest = &input[modifier.len()..];
        return Ok((modifier, rest));
    }

    // If no pattern matched, return empty
    Ok((String::new(), input))
}

// Helper function to parse number or None from stripped prefix
fn parse_optional_number(stripped: &str, error_msg: &str) -> Result<Option<u32>> {
    if stripped.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            stripped.parse().map_err(|_| anyhow!("{}", error_msg))?,
        ))
    }
}

// Helper function to parse required number from stripped prefix
fn parse_required_number(stripped: &str, error_msg: &str) -> Result<u32> {
    stripped.parse().map_err(|_| anyhow!("{}", error_msg))
}

fn parse_modifier(part: &str) -> Result<Modifier> {
    // Dark Heresy
    if part == "dh" {
        return Ok(Modifier::DarkHeresy);
    }

    // Hero System modifiers
    if part == "hsn" {
        return Ok(Modifier::HeroSystem(HeroSystemType::Normal));
    }
    if part == "hsk" {
        return Ok(Modifier::HeroSystem(HeroSystemType::Killing));
    }
    if part == "hsh" {
        return Ok(Modifier::HeroSystem(HeroSystemType::Hit));
    }

    if part == "fudge" || part == "df" {
        return Ok(Modifier::Fudge);
    }

    // Godbound damage chart system
    if part == "gb" {
        return Ok(Modifier::Godbound(false)); // Normal damage chart
    }
    if part == "gbs" {
        return Ok(Modifier::Godbound(true)); // Straight damage (bypass chart)
    }

    // Wrath & Glory success counting with optional difficulty and total flag
    if let Some(stripped) = part.strip_prefix("wng") {
        let use_total = part.ends_with('t');
        let number_part = if use_total {
            &stripped[..stripped.len() - 1]
        } else {
            stripped
        };

        let difficulty = parse_optional_number(number_part, "Invalid difficulty number")?;
        return Ok(Modifier::WrathGlory(difficulty, use_total));
    }

    // Numeric modifiers (positive numbers)
    if let Ok(num) = part.parse::<i32>() {
        return Ok(Modifier::Add(num));
    }

    // Operators with numbers (e.g., "+2", "-3", "*4", "/2") using pre-compiled regex
    if let Some(modifier) = parse_operator_modifier(part)? {
        return Ok(modifier);
    }

    // Special modifiers with optional numbers
    if let Some(stripped) = part.strip_prefix("ie") {
        let num = parse_optional_number(stripped, "Invalid explode value")?;
        return Ok(Modifier::ExplodeIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix('e') {
        let num = parse_optional_number(stripped, "Invalid explode value")?;
        return Ok(Modifier::Explode(num));
    }

    if let Some(stripped) = part.strip_prefix("ir") {
        let num = parse_required_number(stripped, "Invalid reroll value")?;
        return Ok(Modifier::RerollIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix('r') {
        let num = parse_required_number(stripped, "Invalid reroll value")?;
        return Ok(Modifier::Reroll(num));
    }

    if let Some(stripped) = part.strip_prefix("kl") {
        let num = parse_required_number(stripped, "Invalid keep low value")?;
        return Ok(Modifier::KeepLow(num));
    }

    if let Some(stripped) = part.strip_prefix('k') {
        let num = parse_required_number(stripped, "Invalid keep value")?;
        return Ok(Modifier::KeepHigh(num));
    }

    if let Some(stripped) = part.strip_prefix('d') {
        let num = parse_required_number(stripped, "Invalid drop value")?;
        return Ok(Modifier::Drop(num));
    }

    if let Some(stripped) = part.strip_prefix('t') {
        let num = parse_required_number(stripped, "Invalid target value")?;
        return Ok(Modifier::Target(num));
    }

    if let Some(stripped) = part.strip_prefix('f') {
        let num = parse_required_number(stripped, "Invalid failure value")?;
        return Ok(Modifier::Failure(num));
    }

    if let Some(stripped) = part.strip_prefix('b') {
        let num = parse_optional_number(stripped, "Invalid botch value")?;
        return Ok(Modifier::Botch(num));
    }

    // Additional dice (e.g., "+2d6", "-1d4")
    if let Some(modifier) = parse_dice_modifier(part)? {
        return Ok(modifier);
    }

    Err(anyhow!("Unknown modifier: {}", part))
}

// Helper function to parse operator modifiers using pre-compiled regex
fn parse_operator_modifier(part: &str) -> Result<Option<Modifier>> {
    if let Some(captures) = OP_REGEX.captures(part) {
        let num: i32 = captures[2]
            .parse()
            .map_err(|_| anyhow!("Invalid modifier number"))?;
        match &captures[1] {
            "+" => Ok(Some(Modifier::Add(num))),
            "-" => Ok(Some(Modifier::Subtract(num))),
            "*" => Ok(Some(Modifier::Multiply(num))),
            "/" => Ok(Some(Modifier::Divide(num))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

// Helper function to parse dice modifiers using pre-compiled regex
fn parse_dice_modifier(part: &str) -> Result<Option<Modifier>> {
    if let Some(captures) = DICE_MOD_REGEX.captures(part) {
        let count: u32 = captures[2]
            .parse()
            .map_err(|_| anyhow!("Invalid dice count in modifier"))?;
        let sides: u32 = captures[3]
            .parse()
            .map_err(|_| anyhow!("Invalid dice sides in modifier"))?;

        let add_dice = DiceRoll {
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

        match &captures[1] {
            "+" => Ok(Some(Modifier::AddDice(add_dice))),
            "-" => Ok(Some(Modifier::SubtractDice(add_dice))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

fn is_dice_expression(input: &str) -> bool {
    // Use pre-compiled regex for dice expression detection
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
            captures[2]
                .parse()
                .map_err(|_| anyhow!("Invalid dice sides"))?
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
