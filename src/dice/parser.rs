use super::{DiceRoll, HeroSystemType, Modifier};
use anyhow::{anyhow, Result};
use regex::Regex;

pub fn parse_dice_string(input: &str) -> Result<Vec<DiceRoll>> {
    let input = input.trim();

    // Check for aliases first
    if let Some(expanded) = super::aliases::expand_alias(input) {
        return parse_dice_string(&expanded);
    }

    // Check for multi-roll (semicolon separated)
    if input.contains(';') {
        let parts: Vec<&str> = input.split(';').collect();
        if parts.len() > 4 {
            return Err(anyhow!("Maximum of 4 separate rolls allowed"));
        }

        let mut results = Vec::new();
        for part in parts {
            let part = part.trim();
            let mut sub_results = parse_dice_string(part)?;
            // Store the original expression for each semicolon-separated roll
            for dice in &mut sub_results {
                dice.original_expression = Some(part.to_string());
            }
            results.append(&mut sub_results);
        }
        return Ok(results);
    }

    // Check for roll sets (e.g., "6 4d6")
    let set_regex = Regex::new(r"^(\d+)\s+(.+)$").unwrap();
    if let Some(captures) = set_regex.captures(input) {
        let count: u32 = captures[1]
            .parse()
            .map_err(|_| anyhow!("Invalid set count"))?;
        if !(2..=20).contains(&count) {
            return Err(anyhow!("Set count must be between 2 and 20"));
        }
        let dice_expr = &captures[2];

        let mut results = Vec::new();
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

    // Parse the main dice expression and modifiers
    let parts: Vec<&str> = if remaining
        .chars()
        .any(|c| matches!(c, '+' | '-' | '*' | '/'))
        && !remaining.contains(' ')
    {
        // Split mathematical expressions without spaces using regex
        split_math_expression_regex(remaining)?
    } else {
        // Split by whitespace for expressions with spaces
        remaining.split_whitespace().collect()
    };

    if parts.is_empty() {
        return Err(anyhow!("No dice expression found"));
    }

    // Parse main dice part (XdY)
    parse_base_dice(&mut dice, parts[0])?;

    // Parse modifiers
    parse_all_modifiers(&mut dice, &parts[1..])?;

    Ok(dice)
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

// Helper function to parse label
fn parse_label<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    let label_regex = Regex::new(r"^\(([^)]+)\)\s*").unwrap();
    if let Some(captures) = label_regex.captures(remaining) {
        dice.label = Some(captures[1].to_string());
        &remaining[captures.get(0).unwrap().end()..]
    } else {
        remaining
    }
}

// Helper function to parse comment
fn parse_comment<'a>(dice: &mut DiceRoll, remaining: &'a str) -> &'a str {
    let comment_regex = Regex::new(r"!\s*(.+)$").unwrap();
    if let Some(captures) = comment_regex.captures(remaining) {
        dice.comment = Some(captures[1].to_string());
        remaining[..captures.get(0).unwrap().start()].trim()
    } else {
        remaining
    }
}

// Helper function to parse base dice expression
fn parse_base_dice(dice: &mut DiceRoll, part: &str) -> Result<()> {
    let dice_regex = Regex::new(r"^(\d+)?d(\d+|%)$").unwrap();
    if let Some(captures) = dice_regex.captures(part) {
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

        if dice.count > 100 {
            return Err(anyhow!("Maximum 100 dice allowed"));
        }
        if dice.sides > 10000 {
            return Err(anyhow!("Maximum 10000 sides allowed"));
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

// Define modifier patterns for extraction
struct ModifierPattern {
    prefix: &'static str,
    regex: &'static str,
    exclude_prefixes: &'static [&'static str],
}

impl ModifierPattern {
    const PATTERNS: &'static [ModifierPattern] = &[
        ModifierPattern {
            prefix: "hsh",
            regex: r"^(hsh)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "hsk",
            regex: r"^(hsk)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "hsn",
            regex: r"^(hsn)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "gbs",
            regex: r"^(gbs)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "gb",
            regex: r"^(gb)",
            exclude_prefixes: &["gbs"],
        },
        ModifierPattern {
            prefix: "wng",
            regex: r"^(wng\d*t?)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "ie",
            regex: r"^(ie\d*)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "ir",
            regex: r"^(ir\d+)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "kl",
            regex: r"^(kl\d+)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "e",
            regex: r"^(e\d*)",
            exclude_prefixes: &["ie"],
        },
        ModifierPattern {
            prefix: "k",
            regex: r"^(k\d+)",
            exclude_prefixes: &["kl"],
        },
        ModifierPattern {
            prefix: "r",
            regex: r"^(r\d+)",
            exclude_prefixes: &["ir"],
        },
        ModifierPattern {
            prefix: "d",
            regex: r"^(d\d+)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "t",
            regex: r"^(t\d+)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "f",
            regex: r"^(f\d+)",
            exclude_prefixes: &[],
        },
        ModifierPattern {
            prefix: "b",
            regex: r"^(b\d*)",
            exclude_prefixes: &[],
        },
    ];

    fn matches(&self, input: &str) -> bool {
        input.starts_with(self.prefix)
            && !self
                .exclude_prefixes
                .iter()
                .any(|&exclude| input.starts_with(exclude))
    }

    fn extract(&self, input: &str) -> Option<String> {
        if self.matches(input) {
            let re = Regex::new(self.regex).unwrap();
            re.captures(input).map(|captures| captures[1].to_string())
        } else {
            None
        }
    }
}

// New function to split combined modifiers like "e6k8" into ["e6", "k8"]
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
    // Check if it's a simple modifier that doesn't need splitting
    let simple_patterns = [
        r"^[+\-*/]\d+$", // +5, -3, *2, /4
        r"^\d+$",        // 5
        r"^gbs$",        // gbs
        r"^gb$",         // gb
        r"^ie\d*$",      // ie, ie6
        r"^ir\d+$",      // ir2
        r"^r\d+$",       // r2
        r"^e\d*$",       // e, e6 (but only if no other modifiers follow)
        r"^kl\d+$",      // kl2
        r"^k\d+$",       // k3 (but only if no other modifiers follow)
        r"^d\d+$",       // d2 (but only if no other modifiers follow)
        r"^t\d+$",       // t7
        r"^f\d+$",       // f1
        r"^b\d*$",       // b, b1
        r"^wng\d*t?$",   // wng, wng3, wngt, wng3t
        r"^hsn$",        // hsn
        r"^hsk$",        // hsk
        r"^hsh$",        // hsh
    ];

    simple_patterns
        .iter()
        .any(|pattern| Regex::new(pattern).unwrap().is_match(input))
}

fn extract_next_modifier(input: &str) -> Result<(String, &str)> {
    // Try each pattern in order
    for pattern in ModifierPattern::PATTERNS {
        if let Some(modifier) = pattern.extract(input) {
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
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

    // Operators with numbers (e.g., "+2", "-3", "*4", "/2")
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

// Helper function to parse operator modifiers
fn parse_operator_modifier(part: &str) -> Result<Option<Modifier>> {
    let op_regex = Regex::new(r"^([+\-*/])(\d+)$").unwrap();
    if let Some(captures) = op_regex.captures(part) {
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

// Helper function to parse dice modifiers
fn parse_dice_modifier(part: &str) -> Result<Option<Modifier>> {
    let dice_mod_regex = Regex::new(r"^([+\-])(\d+)d(\d+)$").unwrap();
    if let Some(captures) = dice_mod_regex.captures(part) {
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

        Ok(Some(Modifier::AddDice(add_dice)))
    } else {
        Ok(None)
    }
}

fn is_dice_expression(input: &str) -> bool {
    let dice_regex = Regex::new(r"^\d*d\d+$").unwrap();
    dice_regex.is_match(input)
}

fn parse_dice_expression_only(input: &str) -> Result<DiceRoll> {
    let dice_regex = Regex::new(r"^(\d+)?d(\d+)$").unwrap();
    if let Some(captures) = dice_regex.captures(input) {
        let count = captures
            .get(1)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);
        let sides = captures[2]
            .parse()
            .map_err(|_| anyhow!("Invalid dice sides"))?;

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

fn split_math_expression_regex(input: &str) -> Result<Vec<&str>> {
    // Use regex to split expressions like "4d10+2" into ["4d10", "+2"]
    let re = Regex::new(r"(\d+d\d+|[+\-*/]\d+)")?;
    let parts: Vec<&str> = re.find_iter(input).map(|m| m.as_str()).collect();

    if parts.is_empty() {
        // If no matches, return the original input
        Ok(vec![input])
    } else {
        Ok(parts)
    }
}
