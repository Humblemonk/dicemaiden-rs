use super::{DiceRoll, HeroSystemType, LaserFeelingsType, Modifier};
use anyhow::{Result, anyhow};
use once_cell::sync::Lazy;
use regex::Regex;

const MAX_INPUT_LENGTH: usize = 1000;

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
static ADV_WITH_SIMPLE_MOD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^([+-])d(\d+|%)\s*([+\-*/])\s*(\d+)$")
        .expect("Failed to compile ADV_WITH_SIMPLE_MOD_REGEX")
});

pub fn parse_dice_string(input: &str) -> Result<Vec<DiceRoll>> {
    let input = input.trim();

    // Input length validation for DoS protection
    if input.len() > MAX_INPUT_LENGTH {
        return Err(anyhow!(
            "Input too long: {} characters (max {})",
            input.len(),
            MAX_INPUT_LENGTH
        ));
    }

    // Check for aliases that expand to roll sets
    if let Some(expanded) = super::aliases::expand_alias(input) {
        if expanded.contains(' ') && !expanded.contains(';') {
            // Check if this is a roll set pattern
            if let Some(captures) = SET_REGEX.captures(&expanded) {
                // Validate that this is actually a roll set before proceeding
                if is_valid_roll_set_expression(&captures[2]) {
                    return create_roll_set(&captures);
                }
            }
        }
        // If alias doesn't expand to roll set, parse normally
        return Ok(vec![parse_single_dice_expression(&expanded)?]);
    }

    // Check for multi-roll (semicolon separated)
    if input.contains(';') {
        return parse_semicolon_separated_rolls(input);
    }

    // PRIORITY 1: Check for roll sets FIRST, but validate expression first
    if let Some(captures) = SET_REGEX.captures(input) {
        let count_str = &captures[1];
        let expression = &captures[2];

        // Only proceed with roll set logic if expression is valid
        if is_valid_roll_set_expression(expression)
            && let Ok(count) = count_str.parse::<u32>()
        {
            // Add validation check for count range
            if !(2..=20).contains(&count) {
                return Err(anyhow!("Set count must be between 2 and 20"));
            }

            // Handle advantage/disadvantage patterns with modifiers in roll sets
            let final_expression = if let Some(expanded) = super::aliases::expand_alias(expression)
            {
                expanded
            } else if let Some(captures) = ADV_WITH_SIMPLE_MOD_REGEX.captures(expression) {
                let advantage_sign = &captures[1];
                let sides = &captures[2];
                let operator = &captures[3];
                let number = &captures[4];

                // Expand the advantage/disadvantage part
                let adv_alias = format!("{advantage_sign}d{sides}");
                if let Some(expanded_adv) = super::aliases::expand_alias(&adv_alias) {
                    format!("{expanded_adv} {operator} {number}")
                } else {
                    expression.to_string()
                }
            } else {
                expression.to_string()
            };

            // Now try to parse the (possibly expanded) expression
            match parse_single_dice_expression(&final_expression) {
                Ok(dice) => {
                    let mut results = Vec::with_capacity(count as usize);
                    for i in 0..count {
                        let mut set_dice = dice.clone();
                        set_dice.label = Some(format!("Set {}", i + 1));
                        results.push(set_dice);
                    }
                    return Ok(results);
                }
                Err(_e) => {
                    // Fall through to other parsing methods
                }
            }
        }
        // If not a valid roll set expression, fall through to single expression parsing
    }

    // Parse flags, labels, and comments
    let mut temp_dice = create_default_dice_roll();
    let after_flags = parse_flags(&mut temp_dice, input);
    let after_label = parse_label(&mut temp_dice, after_flags);
    let after_comment = parse_comment(&mut temp_dice, after_label);
    let remaining_input = after_comment.trim();

    // PRIORITY 2: Check for roll sets AGAIN after processing flags, with same validation
    if let Some(captures) = SET_REGEX.captures(remaining_input) {
        let count_str = &captures[1];
        let expression = &captures[2];

        // Only proceed with roll set logic if expression is valid
        if is_valid_roll_set_expression(expression)
            && let Ok(count) = count_str.parse::<u32>()
        {
            // Add validation check for count range
            if !(2..=20).contains(&count) {
                return Err(anyhow!("Set count must be between 2 and 20"));
            }

            // Handle advantage/disadvantage patterns with modifiers in roll sets
            let final_expression = if let Some(expanded) = super::aliases::expand_alias(expression)
            {
                // Direct alias match (like "+d20" -> "2d20 k1")
                expanded
            } else if let Some(captures) = ADV_WITH_SIMPLE_MOD_REGEX.captures(expression) {
                // Advantage/disadvantage with modifiers (like "+d20-1" or "+d20 * 2")
                let advantage_sign = &captures[1];
                let sides = &captures[2];
                let operator = &captures[3];
                let number = &captures[4];

                // Expand the advantage/disadvantage part
                let adv_alias = format!("{advantage_sign}d{sides}");
                if let Some(expanded_adv) = super::aliases::expand_alias(&adv_alias) {
                    // Combine with the numeric modifier part
                    format!("{expanded_adv} {operator} {number}")
                } else {
                    expression.to_string()
                }
            } else {
                expression.to_string()
            };

            // Now try to parse the (possibly expanded) expression
            if let Ok(mut dice) = parse_single_dice_expression(&final_expression) {
                // Transfer the parsed flags to each set
                transfer_dice_metadata(&temp_dice, &mut dice);

                // Successfully parsed - create the roll set
                let mut results = Vec::with_capacity(count as usize);
                for i in 0..count {
                    let mut set_dice = dice.clone();
                    set_dice.label = Some(format!("Set {}", i + 1));
                    results.push(set_dice);
                }
                return Ok(results);
            }
        }
        // If not a valid roll set expression, fall through to single expression parsing
    }

    // PRIORITY 3: Handle advantage/disadvantage with simple modifiers
    // But ONLY if we're not dealing with a roll set (no space + number at start)
    // Check for advantage/disadvantage with SIMPLE numeric modifiers ONLY
    // This handles cases like "+d20+5" or "-d20-3" but NOT "+d20 + d10"
    if let Some(captures) = ADV_WITH_SIMPLE_MOD_REGEX.captures(input) {
        let advantage_sign = &captures[1];
        let sides = &captures[2];
        let operator = &captures[3];
        let number = &captures[4];

        // Expand the advantage/disadvantage part
        let adv_alias = format!("{advantage_sign}d{sides}");
        if let Some(expanded_adv) = super::aliases::expand_alias(&adv_alias) {
            // Combine with the numeric modifier part
            let full_expression = format!("{expanded_adv} {operator} {number}");
            return Ok(vec![parse_single_dice_expression(&full_expression)?]);
        }
    }

    // PRIORITY 4: Parse as single expression
    let mut result_dice = parse_single_dice_expression(input)?;
    transfer_dice_metadata(&temp_dice, &mut result_dice);
    Ok(vec![result_dice])
}

// Enhanced is_valid_roll_set_expression function
// Replace the is_valid_roll_set_expression function in src/dice/parser.rs
// This fix addresses the three failing test cases

fn is_valid_roll_set_expression(expr: &str) -> bool {
    let expr = expr.trim();

    // Empty expressions are not valid
    if expr.is_empty() {
        return false;
    }

    // Reject expressions that start with "/ " immediately
    if expr.starts_with("/ ") {
        return false;
    }

    // Check if it's a known alias FIRST (before rejecting operators)
    // This catches +d20, -d20, +d%, -d%, sw8, 4cod, etc.
    // This fixes "3 sw8 + 5" because "sw8 + 5" has no direct alias
    // but "sw8" does, so we need better checking
    if super::aliases::expand_alias(expr).is_some() {
        return true;
    }

    // Check for D6 Legends patterns specifically
    if let Some(before_d6l) = expr.strip_suffix("d6l") {
        // Pattern like "5d6l" or "12d6l"
        // Remove "d6l"
        if let Ok(count) = before_d6l.parse::<u32>() {
            return count > 0; // Valid if positive number
        }
    }

    // Check if the expression starts with a game system alias
    // This handles cases like "sw8 + 5" where the full expression isn't an alias
    // but it starts with one
    let game_system_prefixes = ["sw", "cod", "wod", "sr", "gb", "cpr", "wit", "df", "hs"];
    for prefix in &game_system_prefixes {
        if let Some(after_prefix) = expr.strip_prefix(prefix) {
            // Check if it's followed by valid game system patterns
            if after_prefix
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
            {
                return true; // "sw8", "4cod", etc.
            }
            if *prefix == "gb" && (after_prefix.is_empty() || after_prefix.starts_with(' ')) {
                return true; // "gb" or "gb "
            }
        }
    }

    // THEN reject expressions that start with operators (but not advantage/disadvantage)
    if expr.starts_with(['*', '/', '=', '<', '>', '!', '&', '|', '^', '%']) {
        return false;
    }

    // Special handling for + and - : only reject if they're not followed by 'd'
    if expr.starts_with(['+', '-']) {
        let after_sign = &expr[1..];
        if !after_sign.starts_with('d') {
            return false;
        }
    }

    // Check for number/dice division patterns (like "20/d6", "100/2d4")
    let number_div_dice_regex = Regex::new(r"^\d+\s*/\s*(d\d+.*|d%.*|\d*d\d+.*)$").unwrap();
    if number_div_dice_regex.is_match(expr) {
        return true;
    }

    // Check for standard dice expressions (like "2d6", "d20", "4d6+3")
    let standard_dice_regex = Regex::new(r"^(\d*d\d+|d%|\d*d%)").unwrap();
    if standard_dice_regex.is_match(expr) {
        return true;
    }

    // Check for advantage/disadvantage patterns explicitly
    let advantage_regex = Regex::new(r"^[+-]d(\d+|%)").unwrap();
    if advantage_regex.is_match(expr) {
        return true;
    }

    // Check for dice expressions with modifiers
    // This fixes "3 (Stat Roll) 4d6 k3" and "6 (Stat) 4d6 k3 ! Character generation"
    // After label/comment parsing, these become "4d6 k3"
    let dice_with_modifiers_patterns = [
        r"^\d*d\d+\s+[kdie]+\d*.*$",    // "4d6 k3", "2d6 e6", "1d20 d1"
        r"^\d*d\d+\s*[+\-*/]\s*\d+.*$", // "4d6 + 3", "2d6 - 1", "1d20 * 2"
        r"^\d*d%\s+[kdie]+\d*.*$",      // "d% k1", "2d% e6"
        r"^\d*d%\s*[+\-*/]\s*\d+.*$",   // "d% + 10", "2d% - 5"
    ];

    for pattern in &dice_with_modifiers_patterns {
        if Regex::new(pattern).unwrap().is_match(expr) {
            return true;
        }
    }

    // Must contain 'd' to be a dice expression (for remaining cases)
    if !expr.contains('d') {
        return false;
    }

    // If we get here and it contains 'd', it's probably valid
    true
}

// Helper function to create roll sets, eliminating duplication
fn create_roll_set(captures: &regex::Captures) -> Result<Vec<DiceRoll>> {
    create_roll_set_with_metadata(captures, None)
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

    // Handle D6 System alias expansion BEFORE general alias expansion
    // This prevents the "d6s5" -> "5d6 + 1d6ie" from being mis-parsed
    if remaining.starts_with("d6s")
        && let Some(expanded) = super::aliases::expand_alias(remaining)
    {
        // D6 System expansion: "d6s5" -> "1d1 d6s5"
        // This creates a dummy roll that triggers the D6System modifier
        let mut expanded_dice = parse_single_dice_expression(&expanded)?;
        transfer_dice_metadata(&dice, &mut expanded_dice);
        return Ok(expanded_dice);
    }

    // Check for simple advantage/disadvantage patterns (without additional modifiers)
    // Only do alias expansion, don't try to be clever about advantage detection here
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

    // Check for advantage/disadvantage as the first part of a larger expression
    // ONLY do this if we have multiple parts (indicating it's a complex expression)
    if parts.len() > 2
        && (parts[0] == "+d20" || parts[0] == "-d20" || parts[0] == "+d%" || parts[0] == "-d%")
    {
        // This is an advantage/disadvantage pattern with additional modifiers
        // Expand the first part and then handle the rest
        if let Some(expanded_first) = super::aliases::expand_alias(&parts[0]) {
            // Parse the expanded advantage/disadvantage
            let mut adv_dice = parse_single_dice_expression(&expanded_first)?;

            // Apply remaining modifiers
            parse_all_modifiers(&mut adv_dice, &parts[1..])?;

            // Transfer metadata
            transfer_dice_metadata(&dice, &mut adv_dice);
            return Ok(adv_dice);
        }
    }

    // Check if this is an expression that starts with a number (like "4 + 4d10" or "5 - 2d1")
    if parts.len() > 2 && parts[0].parse::<i32>().is_ok() && !parts[0].contains('d') {
        // Find the first dice expression in the parts
        let mut dice_index = None;
        for (i, part) in parts.iter().enumerate() {
            if part.contains('d') || is_dice_expression(part) {
                dice_index = Some(i);
                break;
            }
        }

        if let Some(dice_idx) = dice_index {
            // Parse the dice part
            parse_base_dice(&mut dice, &parts[dice_idx])?;

            // Get the initial number and operator
            let initial_number = parts[0].parse::<i32>().unwrap();

            // Check if there's an operator before the dice
            if dice_idx > 1 {
                match parts[dice_idx - 1].as_str() {
                    "+" => {
                        dice.modifiers.push(Modifier::Add(initial_number));
                    }
                    "-" => {
                        dice.modifiers.push(Modifier::Multiply(-1));
                        dice.modifiers.push(Modifier::Add(initial_number));
                    }
                    "*" => {
                        dice.modifiers.push(Modifier::Multiply(initial_number));
                    }
                    "/" => {
                        dice.modifiers.push(Modifier::Multiply(0)); // Special marker
                        dice.modifiers.push(Modifier::Add(initial_number));
                    }
                    _ => {
                        dice.modifiers.push(Modifier::Add(initial_number));
                    }
                }
            } else {
                dice.modifiers.push(Modifier::Add(initial_number));
            }

            // Parse remaining modifiers after the dice
            if dice_idx + 1 < parts.len() {
                parse_all_modifiers(&mut dice, &parts[dice_idx + 1..])?;
            }

            return Ok(dice);
        }
    }

    // Standard parsing: first part should be dice
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

    // Handle basic dice + modifiers case (like "1d6 t4f1ie6") before complex parsing
    if !normalized.contains(['+', '-', '*', '/']) && normalized.contains(' ') {
        let parts: Vec<String> = normalized
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        if parts.len() == 2 && parts[0].contains('d') {
            return Ok(parts);
        }
    }

    // REMOVE the aggressive advantage detection here - it's interfering with roll sets
    // Only handle advantage patterns when they're part of complex expressions (multiple operators)

    // Special handling for expressions like "200 / 2d4" or "500 / d%" where number comes before dice
    let number_op_dice_regex =
        Regex::new(r"^(\d+)\s*([+\-*/])\s+(d\d+.*|d%.*|\d*d\d+.*)$").unwrap();
    if let Some(captures) = number_op_dice_regex.captures(&normalized) {
        let number = &captures[1];
        let operator = &captures[2];
        let dice_part = &captures[3];

        if operator == "/" {
            let dice_parts = parse_dice_and_additional_modifiers(dice_part)?;

            let mut result_parts = vec![
                dice_parts.dice_part,
                "*".to_string(),
                "0".to_string(),
                "+".to_string(),
                number.to_string(),
            ];

            result_parts.extend(dice_parts.additional_modifiers);
            return Ok(result_parts);
        } else {
            return Ok(vec![
                number.to_string(),
                operator.to_string(),
                dice_part.to_string(),
            ]);
        }
    }

    // Check if this is entirely without spaces and contains dice
    if !normalized.contains(' ') && normalized.contains('d') {
        return parse_combined_expression(&normalized);
    }

    // Check if this is a combined expression without spaces but with operators
    if !normalized.contains(' ')
        && normalized
            .chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/'))
    {
        return parse_combined_expression(&normalized);
    }

    // Handle mixed expressions (some spaces, some combined)
    let mut parts = Vec::new();
    let mut current_token = String::new();
    let mut chars = normalized.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            ' ' => {
                if !current_token.is_empty() {
                    process_current_token(&mut parts, &mut current_token)?;
                }
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
            '+' | '-' | '*' | '/' => {
                // Only treat + or - as part of dice expression if it's at the start AND followed by 'd'
                // AND we're in a complex multi-operator expression
                if current_token.is_empty()
                    && (ch == '+' || ch == '-')
                    && chars.peek() == Some(&'d')
                {
                    // Look ahead to see if this looks like a complex expression
                    let remaining_chars: String = chars.clone().collect();
                    if remaining_chars.contains(['+', '-', '*', '/']) {
                        // This is likely "+d20 + something" so treat +d20 as one token
                        current_token.push(ch);
                        continue;
                    }
                }

                if !current_token.is_empty() {
                    process_current_token(&mut parts, &mut current_token)?;
                }
                parts.push(ch.to_string());
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        process_final_token(&mut parts, &current_token)?;
    }

    Ok(parts)
}

// New struct to hold parsed dice and modifiers
struct DiceAndModifiers {
    dice_part: String,
    additional_modifiers: Vec<String>,
}

// Helper function to parse dice and additional modifiers
fn parse_dice_and_additional_modifiers(input: &str) -> Result<DiceAndModifiers> {
    // Look for the dice expression first
    let dice_regex = Regex::new(r"^(\d*d\d+|d%|d\d+)(.*)$").unwrap();
    if let Some(captures) = dice_regex.captures(input) {
        let dice_part = captures[1].to_string();
        let remaining = captures[2].trim();

        let additional_modifiers = if remaining.is_empty() {
            Vec::new()
        } else {
            // Parse the remaining modifiers
            parse_additional_modifiers(remaining)?
        };

        Ok(DiceAndModifiers {
            dice_part,
            additional_modifiers,
        })
    } else {
        // Fallback: treat the whole thing as a dice part
        Ok(DiceAndModifiers {
            dice_part: input.to_string(),
            additional_modifiers: Vec::new(),
        })
    }
}

// Helper function to parse additional modifiers after dice
fn parse_additional_modifiers(input: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current_token = String::new();

    for ch in input.chars() {
        match ch {
            '+' | '-' | '*' | '/' => {
                if !current_token.is_empty() {
                    parts.push(current_token.trim().to_string());
                    current_token.clear();
                }
                parts.push(ch.to_string());
            }
            ' ' => {
                if !current_token.is_empty() && current_token.trim() != "" {
                    parts.push(current_token.trim().to_string());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }
    }

    if !current_token.is_empty() {
        parts.push(current_token.trim().to_string());
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

// Parse combined expressions like "4d6e6k3+2" or "200/d%" or "100/2d1+5"
fn parse_combined_expression(input: &str) -> Result<Vec<String>> {
    // Check for number/dice pattern first (e.g., "200/2d4", "500/d%", "100/2d1+5")
    // But be more careful about capturing only the dice part, not trailing spaces/operators
    let number_div_dice_regex = Regex::new(r"^(\d+)/(\S+)(.*)$").unwrap();
    if let Some(captures) = number_div_dice_regex.captures(input) {
        let number = &captures[1];
        let dice_part = &captures[2];
        let trailing_part = captures[3].trim();

        // Check if the dice part actually contains dice (basic validation)
        if dice_part.contains('d') {
            // Parse the dice part and any modifiers attached to it
            let parsed = parse_dice_and_additional_modifiers(dice_part)?;

            // Create the base parts for division
            let mut parts = vec![
                parsed.dice_part,
                "*".to_string(),
                "0".to_string(), // Special marker
                "+".to_string(),
                number.to_string(), // The number to divide by dice result
            ];

            // Add any modifiers that were attached to the dice
            parts.extend(parsed.additional_modifiers);

            // Handle any trailing part (like " +5" in "100/2d1 +5")
            if !trailing_part.is_empty() {
                let trailing_modifiers = parse_additional_modifiers(trailing_part)?;
                parts.extend(trailing_modifiers);
            }

            return Ok(parts);
        }
    }

    let mut parts = Vec::new();
    let mut current_token = String::new();

    for ch in input.chars() {
        match ch {
            '+' | '-' | '*' | '/' => {
                // Mathematical operators - finish current token and add operator
                if !current_token.is_empty() {
                    parts.push(current_token.clone());
                    current_token.clear();
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
        parts.push(current_token);
    }

    // If we didn't find any operators, try the old logic for dice with modifiers
    if parts.len() == 1 && parts[0].contains('d') {
        let input = parts[0].clone(); // Clone to avoid borrow checker issues
        parts.clear();

        // First, extract any mathematical operators at the end
        let (main_part, math_part) = split_math_operators(&input);

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
    }

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
    // Don't split D6 System expressions like "d6s5"
    // These should be treated as single modifiers, not dice + modifiers
    if input.starts_with("d6s") {
        return None; // Don't split D6 System expressions
    }

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

    // Special handling for Mothership patterns - don't strip trailing +/-
    // because they're part of the modifier, not operators
    if input.starts_with("ms") {
        // This is a Mothership modifier, return it as-is
        return Ok(vec![input.to_string()]);
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

    // Use progress tracking instead of arbitrary iteration limit
    let mut last_length = remaining.len();
    let mut no_progress_count = 0;
    const MAX_NO_PROGRESS: usize = 3;

    while !remaining.is_empty() {
        // COMPLETE patterns array in correct order
        let patterns = [
            (r"^(ie\d*)", "indefinite explode"), // ie, ie6 (BEFORE regular explode)
            (r"^(irg\d+)", "indefinite reroll greater"), // irg5 (BEFORE rg)
            (r"^(ir\d+)", "indefinite reroll"),  // ir1 (BEFORE regular reroll)
            (r"^(km\d+)", "keep middle"),        // km3 (BEFORE regular keep)
            (r"^(kl\d+)", "keep low"),           // kl2 (BEFORE regular keep)
            (r"^(tl\d+)", "target lower"),       // tl5 (BEFORE regular target)
            (r"^(rg\d+)", "reroll greater"),     // rg5
            (r"^(k\d+)", "keep high"),           // k3
            (r"^(d\d+)", "drop"),                // d1
            (r"^(r\d+)", "reroll"),              // r1
            (r"^(t\d+)", "target"),              // t4, t7
            (r"^(f\d+)", "failure"),             // f1
            (r"^(e\d*)", "explode"),             // e, e6 (AFTER indefinite explode)
            (r"^(b\d*)", "botch"),               // b, b1
            (r"^(c)", "cancel"),                 // c
            (r"^(wng\d*t?)", "wrath & glory"),   // wng patterns
            (r"^(gb|gbs)", "godbound"),          // gb, gbs
            (r"^(hs[nkh])", "hero system"),      // hsn, hsk, hsh
            (r"^(dh)", "dark heresy"),           // dh
            (r"^(fudge|df)", "fudge"),           // fudge, df
            (r"^(d6s\d+(?:\+\d+)?(?:\-\d+)?)", "d6 system"), // d6s patterns
            (r"^(cpr)", "cyberpunk red"),        // cpr
            (r"^(wit)", "witcher"),              // wit
            (r"^(bnw)", "brave new world"),      // bnw
            (r"^(alien)", "alien base"),         // alien (exact)
            (r"^(aliens\d+)", "alien stress"),   // aliens1, aliens2, etc.
            (r"^(wwc\d+)", "wild worlds cut"),   // wwc2, wwc3, etc. (before basic ww)
            (r"^(ms\d+[ad]?|ms[ad]?|ms)", "mothership"),
            (r"^(ww)", "wild worlds"), // ww (basic)
        ];

        let mut found_match = false;

        for (pattern, _) in &patterns {
            if let Ok(regex) = Regex::new(pattern)
                && let Some(captures) = regex.captures(remaining)
                && let Some(matched) = captures.get(1)
            {
                let modifier = matched.as_str().to_string();
                modifiers.push(modifier.clone());

                // Move past this modifier
                remaining = &remaining[matched.end()..];
                found_match = true;

                break;
            }
        }

        if !found_match {
            return Err(anyhow!(
                "Unable to parse combined modifier: '{}' at position: '{}'. Expected valid modifier pattern.",
                input,
                remaining
            ));
        }

        // Progress tracking
        let current_length = remaining.len();
        if current_length == last_length {
            no_progress_count += 1;
            if no_progress_count > MAX_NO_PROGRESS {
                return Err(anyhow!(
                    "Infinite loop detected while parsing modifiers '{}' at position: '{}'",
                    input,
                    remaining
                ));
            }
        } else {
            no_progress_count = 0;
            last_length = current_length;
        }
    }

    if modifiers.is_empty() {
        return Err(anyhow!("No valid modifiers found in: {}", input));
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
    // Try alias expansion first (this handles +d20, -d20, etc.)
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

    // Handle simple dice expressions that start with 'd' (like "d10", "d20")
    if part.starts_with('d') && part.len() > 1 {
        let full_dice = format!("1{part}"); // Convert "d10" to "1d10"
        if DICE_ONLY_REGEX.is_match(&full_dice) {
            return parse_simple_dice_part(dice, &full_dice);
        }
    }

    // If it's not a simple dice expression, it might be a combined expression
    // that needs to be split further
    if part.contains('d')
        && let Some((dice_part, modifiers_part)) = split_dice_and_modifiers(part)
    {
        // Parse the dice part
        parse_simple_dice_part(dice, &dice_part)?;

        // Parse the modifiers part if any
        parse_modifiers_from_part(dice, &modifiers_part)?;
        return Ok(());
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

        if dice.count == 0 {
            return Err(anyhow!("Cannot roll 0 dice"));
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
        let part = &parts[i];

        // Try parsing operator + operand pairs
        if i + 1 < parts.len()
            && let Some(consumed) = try_parse_operator_pair(dice, &parts[i], &parts[i + 1])?
        {
            i += consumed;

            // Check if we just created an AddDice and if following tokens are modifiers
            if consumed == 2 && parts[i - 2] == "+" && parts[i - 1].contains('d') {
                // Check if following tokens are modifiers for the dice we just added
                let mut modifier_tokens = Vec::new();
                let mut j = i;
                while j < parts.len() && is_modifier_start(&parts[j]) {
                    modifier_tokens.push(parts[j].clone());
                    j += 1;
                }

                if !modifier_tokens.is_empty() {
                    // Handle combined modifiers properly
                    if let Some(Modifier::AddDice(dice_roll)) = dice.modifiers.last_mut() {
                        for token in &modifier_tokens {
                            // Check if this token is combined modifiers
                            if is_combined_modifiers_token(token) {
                                // Split combined modifiers and add each one
                                let combined_parts = split_combined_modifiers(token)?;
                                for combined_part in combined_parts {
                                    let modifier = parse_single_modifier(&combined_part)?;
                                    dice_roll.modifiers.push(modifier);
                                }
                            } else {
                                // Single modifier
                                let modifier = parse_single_modifier(token)?;
                                dice_roll.modifiers.push(modifier);
                            }
                        }
                    }
                    i += modifier_tokens.len(); // Skip the modifier tokens we just processed
                }
            }
            continue;
        }

        // Check if this part is combined modifiers before parsing as single
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
    if input.is_empty() {
        return false;
    }

    // Handle D6 system expressions specially - don't treat as combined modifiers
    if input.contains('d') {
        // Don't treat standalone dice expressions as combined modifiers
        if is_standalone_dice_expression(input) {
            return false;
        }

        // Don't treat D6 system expressions as combined modifiers
        if input.starts_with("d6s") {
            return false;
        }
    }

    // Check if it starts with common modifier patterns and has more after the first one
    let modifier_patterns = [
        r"^(ie\d*)",  // Indefinite explode first (longer pattern)
        r"^(irg\d+)", // Indefinite reroll greater
        r"^(ir\d+)",  // Indefinite reroll
        r"^(km\d+)",  // Keep middle
        r"^(kl\d+)",  // Keep low
        r"^(tl\d+)",  // Target lower (must come before regular target)
        r"^(rg\d+)",  // Reroll greater
        r"^(e\d*)",   // Explode
        r"^(k\d+)",   // Keep high
        r"^(d\d+)",   // Drop
        r"^(r\d+)",   // Reroll
        r"^(t\d+)",   // Target - KEY FOR D6 LEGENDS
        r"^(f\d+)",   // Failure - KEY FOR D6 LEGENDS
        r"^(b\d*)",   // Botch
    ];

    for pattern in &modifier_patterns {
        if let Ok(regex) = Regex::new(pattern)
            && let Some(captures) = regex.captures(input)
        {
            let first_modifier = &captures[1];
            let match_length = first_modifier.len();

            // Safety check for zero-length matches
            if match_length == 0 {
                continue;
            }

            // If the match is the entire string, it's a single modifier, not combined
            if match_length == input.len() {
                return false;
            }

            // If there's more after the first modifier, check if it looks like more modifiers
            let remaining = &input[match_length..];
            let result = is_modifier_start(remaining);

            return result;
        }
    }

    false
}

// Check if a string starts with modifier patterns
fn is_modifier_start(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    // Special case: Don't treat single letters as modifiers if they could be part of game system aliases
    if input.len() == 1 {
        return matches!(input, "c"); // Cancel is the only single-letter modifier
    }

    // Check if it starts with any known modifier pattern (not requiring it to be complete)
    let modifier_start_patterns = [
        r"^ie\d*",  // Indefinite explode: ie, ie6
        r"^irg\d+", // Indefinite reroll greater: irg5
        r"^ir\d+",  // Indefinite reroll: ir1
        r"^km\d+",  // Keep middle: km3
        r"^kl\d+",  // Keep low: kl2
        r"^tl\d+",  // Target lower: tl5
        r"^rg\d+",  // Reroll greater: rg5
        r"^k\d+",   // Keep high: k3
        r"^d\d+",   // Drop: d1
        r"^r\d+",   // Reroll: r1
        r"^t\d+",   // Target: t4, t7 - CRITICAL FOR D6 LEGENDS
        r"^f\d+",   // Failure: f1 - CRITICAL FOR D6 LEGENDS
        r"^e\d*",   // Explode: e, e6
        r"^b\d*",   // Botch: b, b1
        r"^c$",     // Cancel: c (exact match)
        // System modifiers
        r"^wng",       // Wrath & Glory patterns
        r"^gb$",       // Godbound (exact)
        r"^gbs$",      // Godbound straight (exact)
        r"^hs[nkh]",   // Hero System
        r"^dh$",       // Dark Heresy (exact)
        r"^fudge$",    // Fudge (exact)
        r"^df$",       // Fudge dice (exact)
        r"^d6s\d+",    // D6 System
        r"^cpr$",      // Cyberpunk Red (exact)
        r"^wit$",      // Witcher (exact)
        r"^alien$",    // Alien base modifier (exact)
        r"^aliens\d+", // Alien stress modifiers: aliens1, aliens2, etc.
        r"^fitd$",     // Forged in the Dark (exact)
        r"^fitd0$",    // FitD zero dice (exact)
    ];

    // Check if the input starts with any of these patterns
    for pattern in &modifier_start_patterns {
        if let Ok(regex) = Regex::new(pattern)
            && regex.is_match(input)
        {
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
                // This might be a complex dice expression like "1d4e4" or just "d10"
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
                // This might be a complex dice expression like "1d4e4" or just "d10"
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::SubtractDice(dice_roll));
                return Ok(Some(2));
            }
        }
        "*" => {
            if let Ok(num) = second.parse::<i32>() {
                dice.modifiers.push(Modifier::Multiply(num));
                return Ok(Some(2));
            } else if is_dice_expression(second) {
                // Handle dice multiplication
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::MultiplyDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // Handle complex dice expressions for multiplication
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::MultiplyDice(dice_roll));
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
            } else if is_dice_expression(second) {
                // Handle dice division
                let dice_roll = parse_dice_expression_only(second)?;
                dice.modifiers.push(Modifier::DivideDice(dice_roll));
                return Ok(Some(2));
            } else if second.contains('d') {
                // Handle complex dice expressions for division
                let dice_roll = parse_complex_dice_expression(second)?;
                dice.modifiers.push(Modifier::DivideDice(dice_roll));
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

fn parse_single_modifier(part: &str) -> Result<Modifier> {
    // Reject standalone 'l' - it should only appear in d6l aliases
    if part == "l" {
        return Err(anyhow!("Unrecognized modifier pattern: 'l' in 'l'"));
    }

    // Check for Alien RPG modifiers first
    if part == "alien" {
        return Ok(Modifier::Alien);
    }

    if let Some(stripped) = part.strip_prefix("aliens") {
        let stress_level: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid stress level in Alien stress modifier '{}'", part))?;

        if stress_level == 0 {
            return Err(anyhow!("Alien stress level cannot be 0"));
        }
        if stress_level > 10 {
            return Err(anyhow!("Alien stress level cannot exceed 10"));
        }

        return Ok(Modifier::AlienStress(stress_level));
    }

    // Forged in the Dark modifiers
    if part == "fitd" {
        return Ok(Modifier::ForgedDark);
    }

    if part == "fitd0" {
        return Ok(Modifier::ForgedDarkZero);
    }

    // System modifiers
    match part {
        "dh" => return Ok(Modifier::DarkHeresy),
        "dheart" => return Ok(Modifier::Daggerheart),
        "hsn" => return Ok(Modifier::HeroSystem(HeroSystemType::Normal)),
        "hsk" => return Ok(Modifier::HeroSystem(HeroSystemType::Killing)),
        "hsh" => return Ok(Modifier::HeroSystem(HeroSystemType::Hit)),
        "fudge" | "df" => return Ok(Modifier::Fudge),
        "gb" => return Ok(Modifier::Godbound(false)),
        "gbs" => return Ok(Modifier::Godbound(true)),
        "cpr" => return Ok(Modifier::CyberpunkRed),
        "wit" => return Ok(Modifier::Witcher),
        "bnw" => return Ok(Modifier::BraveNewWorld(0)),
        "mnm" => return Ok(Modifier::MutantsMasterminds),
        "c" => return Ok(Modifier::Cancel),
        "ww" => return Ok(Modifier::WildWorlds(None)),
        _ => {}
    }

    // Cypher System handling (cs1, cs3, cs10, etc.)
    if let Some(stripped) = part.strip_prefix("cs") {
        let level = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Cypher System level in '{}'", part))?;
        if !(1..=10).contains(&level) {
            return Err(anyhow!(
                "Cypher System difficulty level must be 1-10, got {}",
                level
            ));
        }
        return Ok(Modifier::CypherSystem(level));
    }

    // Conan skill roll handling (conan, conan3, conan4, conan5)
    if part == "conan" {
        return Ok(Modifier::ConanSkill(2)); // Default 2d20
    }
    if let Some(stripped) = part.strip_prefix("conan")
        && let Ok(dice_count) = stripped.parse::<u32>()
    {
        if (2..=5).contains(&dice_count) {
            return Ok(Modifier::ConanSkill(dice_count));
        } else {
            return Err(anyhow!(
                "Conan skill rolls support 2-5 dice, got {}",
                dice_count
            ));
        }
    }

    // Conan combat dice handling (cd, cd4, cd10, etc.)
    if part == "cd" {
        return Ok(Modifier::ConanCombat(1)); // Default 1d6
    }
    if let Some(stripped) = part.strip_prefix("cd")
        && let Ok(dice_count) = stripped.parse::<u32>()
    {
        if dice_count > 0 && dice_count <= 100 {
            // Reasonable limit
            return Ok(Modifier::ConanCombat(dice_count));
        } else {
            return Err(anyhow!(
                "Conan combat dice count must be 1-100, got {}",
                dice_count
            ));
        }
    }

    if is_multi_sided_dice_expression(part) {
        // This is additional dice that should be added to the roll
        let dice_roll = parse_dice_expression_only(part)?;
        return Ok(Modifier::AddDice(dice_roll));
    }

    // Check for invalid characters before parsing numbers
    if part.contains(['+', '-', '*', '/']) {
        return Err(anyhow!("Invalid modifier '{}' - contains operator", part));
    }

    // Handle exploding dice
    if let Some(stripped) = part.strip_prefix("ie") {
        let num = if stripped.is_empty() {
            None
        } else {
            Some(
                stripped
                    .parse()
                    .map_err(|_| anyhow!("Invalid explode value in '{}'", part))?,
            )
        };
        if let Some(val) = num
            && val == 0
        {
            return Err(anyhow!("Cannot explode on 0"));
        }
        return Ok(Modifier::ExplodeIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix('e') {
        let num = if stripped.is_empty() {
            None
        } else {
            Some(
                stripped
                    .parse()
                    .map_err(|_| anyhow!("Invalid explode value in '{}'", part))?,
            )
        };
        if let Some(val) = num
            && val == 0
        {
            return Err(anyhow!("Cannot explode on 0"));
        }
        return Ok(Modifier::Explode(num));
    }

    // Handle D6 System BEFORE drop modifier to avoid conflicts
    // D6 System handling (Handle BEFORE drop modifier since both start with 'd')
    if let Some(stripped) = part.strip_prefix("d6s") {
        // Parse count and optional pips like "5" or "5+2"
        let count_and_pips = stripped.to_string();

        // Extract count (before any +/-)
        let count_str = if let Some(pos) = count_and_pips.find(['+', '-']) {
            &count_and_pips[..pos]
        } else {
            &count_and_pips
        };

        let count = count_str
            .parse()
            .map_err(|_| anyhow!("Invalid D6 System count in '{}'", part))?;

        // Extract pips part (everything after count)
        let pips = if let Some(pos) = count_and_pips.find(['+', '-']) {
            count_and_pips[pos..].to_string()
        } else {
            String::new()
        };

        return Ok(Modifier::D6System(count, pips));
    }

    // Handle drop modifier (AFTER D6 System to avoid conflicts)
    if let Some(stripped) = part.strip_prefix('d') {
        // Make sure this isn't a D6 System expression that somehow got through
        if stripped.starts_with("6s") {
            return Err(anyhow!(
                "D6 System expressions should be handled earlier: {}",
                part
            ));
        }

        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid drop value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot drop 0 dice"));
        }
        return Ok(Modifier::Drop(num));
    }

    // Check indefinite reroll greater (irg) BEFORE regular reroll greater (rg)
    if let Some(stripped) = part.strip_prefix("irg") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid indefinite reroll greater value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::RerollGreaterIndefinite(num));
    }

    if let Some(stripped) = part.strip_prefix("rg") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid reroll greater value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot reroll on 0 - invalid threshold"));
        }
        return Ok(Modifier::RerollGreater(num));
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

    if let Some(stripped) = part.strip_prefix("km") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid keep middle value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Cannot keep 0 dice"));
        }
        return Ok(Modifier::KeepMiddle(num));
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

    // Target with doubles t{num}ds{num} must be checked BEFORE other targets
    if let Some(captures) = Regex::new(r"^t(\d+)ds(\d+)$").unwrap().captures(part) {
        let target = captures[1]
            .parse()
            .map_err(|_| anyhow!("Invalid target value in TargetWithDoubleSuccess '{}'", part))?;
        let double_value = captures[2]
            .parse()
            .map_err(|_| anyhow!("Invalid double value in TargetWithDoubleSuccess '{}'", part))?;

        // Validation for new syntax only
        if target == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }
        if double_value == 0 {
            return Err(anyhow!("Double success must be greater than 0"));
        }
        if double_value < target {
            return Err(anyhow!(
                "Double success ({}) must be >= target value ({})",
                double_value,
                target
            ));
        }

        return Ok(Modifier::TargetWithDoubleSuccess(target, double_value));
    }

    // Check for t{target}ds pattern (default double success = target)
    if let Some(captures) = Regex::new(r"^t(\d+)ds$").unwrap().captures(part) {
        let target = captures[1]
            .parse()
            .map_err(|_| anyhow!("Invalid target value in TargetWithDoubleSuccess '{}'", part))?;

        // Validation
        if target == 0 {
            return Err(anyhow!("Target value must be greater than 0"));
        }

        // Default: double success value = target value
        return Ok(Modifier::TargetWithDoubleSuccess(target, target));
    }

    // Parse target lower with double success (explicit)
    if let Some(captures) = Regex::new(r"^tl(\d+)ds(\d+)$").unwrap().captures(part) {
        let target = captures[1].parse().map_err(|_| {
            anyhow!(
                "Invalid target value in TargetLowerWithDoubleSuccess '{}'",
                part
            )
        })?;
        let double_value = captures[2].parse().map_err(|_| {
            anyhow!(
                "Invalid double success value in TargetLowerWithDoubleSuccess '{}'",
                part
            )
        })?;

        // Validation
        if target == 0 {
            return Err(anyhow!("Target lower value must be greater than 0"));
        }
        if double_value == 0 {
            return Err(anyhow!("Double success value must be greater than 0"));
        }
        if double_value > target {
            return Err(anyhow!(
                "Double success value ({}) must be <= target lower value ({})",
                double_value,
                target
            ));
        }

        return Ok(Modifier::TargetLowerWithDoubleSuccess(target, double_value));
    }

    // Parse target lower with double success (default)
    if let Some(captures) = Regex::new(r"^tl(\d+)ds$").unwrap().captures(part) {
        let target = captures[1].parse().map_err(|_| {
            anyhow!(
                "Invalid target value in TargetLowerWithDoubleSuccess '{}'",
                part
            )
        })?;

        // Validation
        if target == 0 {
            return Err(anyhow!("Target lower value must be greater than 0"));
        }

        // Default: double success value = target value
        return Ok(Modifier::TargetLowerWithDoubleSuccess(target, target));
    }

    // Target Lower (tl) must be checked BEFORE Target (t) to avoid conflicts
    if let Some(stripped) = part.strip_prefix("tl") {
        let num = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid target lower value in '{}'", part))?;
        if num == 0 {
            return Err(anyhow!("Target lower value must be greater than 0"));
        }
        return Ok(Modifier::TargetLower(num));
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
            return Ok(Modifier::WrathGlory(None, false, 1)); // Default 1 wrath die
        }

        // Parse wrath dice count first
        let mut remaining = stripped;
        let mut wrath_count = 1u32;

        if let Some(w_stripped) = remaining.strip_prefix("w") {
            // Extract wrath count
            let mut chars = w_stripped.chars();
            let mut count_str = String::new();

            for ch in chars.by_ref() {
                if ch.is_ascii_digit() {
                    count_str.push(ch);
                } else {
                    remaining = &w_stripped[count_str.len()..];
                    break;
                }
            }

            if chars.as_str().is_empty() && !count_str.is_empty() {
                remaining = "";
            }

            if !count_str.is_empty() {
                wrath_count = count_str
                    .parse()
                    .map_err(|_| anyhow!("Invalid wrath dice count in '{}'", part))?;

                // Validate reasonable range (1-5 wrath dice max)
                if !(1..=5).contains(&wrath_count) {
                    return Err(anyhow!("Wrath dice count must be 1-5, got {}", wrath_count));
                }
            }
        }

        // Parse difficulty and special modes
        if remaining == "t" {
            return Ok(Modifier::WrathGlory(None, true, wrath_count));
        } else if let Some(dn_part) = remaining.strip_prefix("dn") {
            if let Some(dn_str) = dn_part.strip_suffix('t') {
                let dn = dn_str
                    .parse()
                    .map_err(|_| anyhow!("Invalid difficulty value in '{}'", part))?;
                return Ok(Modifier::WrathGlory(Some(dn), true, wrath_count));
            } else {
                let dn = dn_part
                    .parse()
                    .map_err(|_| anyhow!("Invalid difficulty value in '{}'", part))?;
                return Ok(Modifier::WrathGlory(Some(dn), false, wrath_count));
            }
        } else if let Some(dn_str) = remaining.strip_suffix('t') {
            if let Ok(dn) = dn_str.parse::<u32>() {
                return Ok(Modifier::WrathGlory(Some(dn), true, wrath_count));
            }
        } else if let Ok(dn) = remaining.parse::<u32>() {
            return Ok(Modifier::WrathGlory(Some(dn), false, wrath_count));
        } else if remaining.is_empty() {
            return Ok(Modifier::WrathGlory(None, false, wrath_count));
        }

        return Err(anyhow!("Invalid Wrath & Glory modifier: {}", part));
    }

    // Savage Worlds handling (AFTER D6 System but before other modifiers)
    if let Some(stripped) = part.strip_prefix("sw") {
        let sides = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Savage Worlds trait die in '{}'", part))?;
        if !(4..=12).contains(&sides) || sides % 2 != 0 {
            return Err(anyhow!(
                "Savage Worlds trait die must be d4, d6, d8, d10, or d12"
            ));
        }
        return Ok(Modifier::SavageWorlds(sides));
    }

    // Shadowrun handling
    if let Some(stripped) = part.strip_prefix("shadowrun") {
        let dice_count = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid Shadowrun dice count in '{}'", part))?;
        if dice_count == 0 {
            return Err(anyhow!("Shadowrun dice count must be greater than 0"));
        }
        return Ok(Modifier::Shadowrun(dice_count));
    }

    // Wild Worlds RPG cutting mechanics
    if let Some(stripped) = part.strip_prefix("wwc") {
        let cut_count: u32 = stripped
            .parse()
            .map_err(|_| anyhow!("Invalid cut count in Wild Worlds modifier '{}'", part))?;

        if cut_count == 0 {
            return Err(anyhow!("Cannot cut 0 dice in Wild Worlds"));
        }

        return Ok(Modifier::WildWorlds(Some(cut_count)));
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

    // Marvel Multiverse handling
    if part == "mm" {
        return Ok(Modifier::MarvelMultiverse(0, 0));
    }

    if let Some(stripped) = part.strip_prefix("mme") {
        let edges = stripped.parse().unwrap_or(0);
        return Ok(Modifier::MarvelMultiverse(edges, 0));
    }

    if let Some(stripped) = part.strip_prefix("mmt") {
        let troubles = stripped.parse().unwrap_or(0);
        return Ok(Modifier::MarvelMultiverse(0, troubles));
    }

    if let Some(stripped) = part.strip_prefix("sil") {
        if stripped.is_empty() {
            return Ok(Modifier::Silhouette(1));
        } else {
            let dice_count = stripped
                .parse()
                .map_err(|_| anyhow!("Invalid Silhouette dice count in '{}'", part))?;
            if dice_count == 0 || dice_count > 10 {
                return Err(anyhow!(
                    "Silhouette dice count must be 1-10, got {}",
                    dice_count
                ));
            }
            return Ok(Modifier::Silhouette(dice_count));
        }
    }

    // Mothership RPG handling
    if part == "ms" {
        return Ok(Modifier::Mothership(None, false));
    }
    if part == "msa" {
        // Changed from ms+
        return Ok(Modifier::Mothership(None, true));
    }
    if part == "msd" {
        // Changed from ms-
        return Ok(Modifier::Mothership(None, false));
    }

    // Mothership with stat value
    if let Some(stripped) = part.strip_prefix("ms") {
        if let Some(stat_str) = stripped.strip_suffix('a') {
            // Changed from '+'
            let stat: u32 = stat_str
                .parse()
                .map_err(|_| anyhow!("Invalid Mothership stat value in '{}'", part))?;
            if !(1..=99).contains(&stat) {
                return Err(anyhow!("Mothership stat must be 1-99, got {}", stat));
            }
            return Ok(Modifier::Mothership(Some(stat), true));
        } else if let Some(stat_str) = stripped.strip_suffix('d') {
            // Changed from '-'
            let stat: u32 = stat_str
                .parse()
                .map_err(|_| anyhow!("Invalid Mothership stat value in '{}'", part))?;
            if !(1..=99).contains(&stat) {
                return Err(anyhow!("Mothership stat must be 1-99, got {}", stat));
            }
            return Ok(Modifier::Mothership(Some(stat), false));
        } else if !stripped.is_empty() {
            // Just a stat value
            let stat: u32 = stripped
                .parse()
                .map_err(|_| anyhow!("Invalid Mothership stat value in '{}'", part))?;
            if !(1..=99).contains(&stat) {
                return Err(anyhow!("Mothership stat must be 1-99, got {}", stat));
            }
            return Ok(Modifier::Mothership(Some(stat), false));
        }
    }

    // VTM5 - Vampire: The Masquerade 5th Edition
    if let Some(stripped) = part.strip_prefix("vtm5p") {
        // Parse pattern like "vtm5p7h2" (pool=7, hunger=2)
        if let Some(h_pos) = stripped.find('h') {
            let pool_str = &stripped[..h_pos];
            let hunger_str = &stripped[h_pos + 1..];

            let pool_size = pool_str
                .parse::<u32>()
                .map_err(|_| anyhow!("Invalid VTM5 pool size in '{}'", part))?;
            let hunger_dice = hunger_str
                .parse::<u32>()
                .map_err(|_| anyhow!("Invalid VTM5 hunger dice count in '{}'", part))?;

            // Validate ranges
            if pool_size == 0 || pool_size > 30 {
                return Err(anyhow!("VTM5 pool size must be 1-30, got {}", pool_size));
            }
            if hunger_dice > pool_size {
                return Err(anyhow!(
                    "VTM5 hunger dice ({}) cannot exceed pool size ({})",
                    hunger_dice,
                    pool_size
                ));
            }
            if hunger_dice > 5 {
                return Err(anyhow!(
                    "VTM5 hunger dice cannot exceed 5, got {}",
                    hunger_dice
                ));
            }

            return Ok(Modifier::VampireMasquerade5(pool_size, hunger_dice));
        }
    }

    // Lasers & Feelings handling (lf2, lf2l, lf2f, etc.)
    if let Some(stripped) = part.strip_prefix("lf") {
        // Parse target and optional type (lf4l, lf4f, lf4)
        let chars: Vec<char> = stripped.chars().collect();

        if chars.is_empty() {
            return Err(anyhow!("Invalid Lasers & Feelings format: '{}'", part));
        }

        // Find where the number ends and type begins
        let mut number_end = 0;
        for (i, ch) in chars.iter().enumerate() {
            if ch.is_ascii_digit() {
                number_end = i + 1;
            } else {
                break;
            }
        }

        if number_end == 0 {
            return Err(anyhow!("Invalid Lasers & Feelings target in '{}'", part));
        }

        let target_str: String = chars[..number_end].iter().collect();
        let type_str: String = chars[number_end..].iter().collect();

        let target = target_str
            .parse()
            .map_err(|_| anyhow!("Invalid Lasers & Feelings target in '{}'", part))?;

        // Validate target number (2-5 as per L&F rules)
        if !(2..=5).contains(&target) {
            return Err(anyhow!(
                "Lasers & Feelings target must be between 2 and 5, got {} in '{}'",
                target,
                part
            ));
        }

        // Determine roll type
        let roll_type = match type_str.as_str() {
            "l" => LaserFeelingsType::Lasers,
            "f" => LaserFeelingsType::Feelings,
            "" => {
                // Generic - we'll need both types available, but for now default to Lasers
                // In a real implementation, you might want to ask the user or handle differently
                LaserFeelingsType::Lasers
            }
            _ => {
                return Err(anyhow!(
                    "Invalid Lasers & Feelings type '{}' in '{}'",
                    type_str,
                    part
                ));
            }
        };

        // Note: dice_count will be extracted from the dice expression, not the modifier
        // For now, we'll use 0 as a placeholder and let the roller extract the actual count
        return Ok(Modifier::LaserFeelings(0, target, roll_type));
    }

    // Wild Worlds RPG parsing
    if part == "ww" {
        return Ok(Modifier::WildWorlds(None));
    }

    if let Some(cut_match) = Regex::new(r"^wwc(\d+)$").unwrap().captures(part) {
        let cut_count: u32 = cut_match[1]
            .parse()
            .map_err(|_| anyhow!("Invalid cut count in Wild Worlds modifier"))?;

        if cut_count == 0 {
            return Err(anyhow!("Cannot cut 0 dice in Wild Worlds"));
        }

        return Ok(Modifier::WildWorlds(Some(cut_count)));
    }

    Err(anyhow!("Unknown modifier: {}", part))
}

fn is_dice_expression(input: &str) -> bool {
    // Check for basic dice patterns
    if DICE_ONLY_REGEX.is_match(input) {
        return true;
    }

    // Check for advantage/disadvantage patterns
    if input == "+d20" || input == "-d20" || input == "+d%" || input == "-d%" {
        return true;
    }

    // Check for dice expressions that start with 'd' (like "d10", "d20", "d%")
    if input.starts_with('d') && input.len() > 1 {
        let after_d = &input[1..];
        // Check if it's just numbers or '%'
        if after_d == "%" || after_d.chars().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }

    false
}

fn parse_dice_expression_only(input: &str) -> Result<DiceRoll> {
    // Handle cases like "d10", "d20", "d%"
    if input.starts_with('d') && !input.contains(char::is_alphabetic) {
        let full_dice = format!("1{input}"); // Convert "d10" to "1d10"
        return parse_dice_expression_only(&full_dice);
    }

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

fn is_multi_sided_dice_expression(part: &str) -> bool {
    // Only detect dice expressions that are clearly additional dice, not modifiers

    // Pattern 1: Multi-digit count + d + sides (like "3d6", "2d10", "10d6")
    // 2+ digit count is clearly dice, not a drop modifier
    if Regex::new(r"^(\d{2,})d(\d+)$").unwrap().is_match(part) {
        return true;
    }

    // Pattern 2: Single digit count + d + multi-digit sides (like "1d20", "3d100")
    // Multi-digit sides is clearly dice, not a drop modifier
    if Regex::new(r"^(\d)d(\d{2,})$").unwrap().is_match(part) {
        return true;
    }

    // Pattern 3: Common multi-die patterns that aren't drop modifiers
    if let Some(caps) = Regex::new(r"^(\d+)d([46]|8|10|12|20|100)$")
        .unwrap()
        .captures(part)
        && let (Ok(count), Ok(sides)) = (caps[1].parse::<u32>(), caps[2].parse::<u32>())
    {
        // If count > 1, it's clearly additional dice, not a drop modifier
        if count > 1 {
            return true;
        }
        // For count=1, check sides - d4, d6, d8, d10, d12, d20, d100 are likely dice
        // Exclude d1, d2, d3 which are likely drop modifiers
        if sides >= 4 {
            return true;
        }
    }

    // Pattern 4: Percentile dice (d% patterns)
    if part == "d%" || part.ends_with("d%") {
        return true;
    }

    false // Default to not a dice expression - let normal modifier parsing handle it
}

fn is_standalone_dice_expression(input: &str) -> bool {
    // Check for patterns that represent standalone dice expressions
    // These should NOT be treated as combined modifiers
    let standalone_patterns = [
        r"^\d*d\d+$", // Basic dice like "3d6", "d20", "1d4"
        r"^\d*d%$",   // Percentile dice like "d%", "2d%"
    ];

    for pattern in &standalone_patterns {
        let regex = Regex::new(pattern).unwrap();
        if regex.is_match(input) {
            return true;
        }
    }

    false
}

fn create_roll_set_with_metadata(
    captures: &regex::Captures,
    metadata: Option<&DiceRoll>,
) -> Result<Vec<DiceRoll>> {
    let count: u32 = captures[1]
        .parse()
        .map_err(|_| anyhow!("Invalid set count"))?;

    // VALIDATION: Centralized in one place
    if !(2..=20).contains(&count) {
        return Err(anyhow!("Set count must be between 2 and 20"));
    }

    let expression = &captures[2];

    // Handle advantage/disadvantage patterns with modifiers in roll sets
    let final_expression = if let Some(expanded) = super::aliases::expand_alias(expression) {
        expanded
    } else if let Some(adv_captures) = ADV_WITH_SIMPLE_MOD_REGEX.captures(expression) {
        let advantage_sign = &adv_captures[1];
        let sides = &adv_captures[2];
        let operator = &adv_captures[3];
        let number = &adv_captures[4];

        // Expand the advantage/disadvantage part
        let adv_alias = format!("{advantage_sign}d{sides}");
        if let Some(expanded_adv) = super::aliases::expand_alias(&adv_alias) {
            format!("{expanded_adv} {operator} {number}")
        } else {
            expression.to_string()
        }
    } else {
        expression.to_string()
    };

    // Parse the dice expression
    let mut dice = parse_single_dice_expression(&final_expression)?;

    // Transfer metadata if provided (for flag support)
    if let Some(meta) = metadata {
        transfer_dice_metadata(meta, &mut dice);
    }

    // Create the roll set
    let mut results = Vec::with_capacity(count as usize);
    for i in 0..count {
        let mut set_dice = dice.clone();
        set_dice.label = Some(format!("Set {}", i + 1));
        results.push(set_dice);
    }
    Ok(results)
}
