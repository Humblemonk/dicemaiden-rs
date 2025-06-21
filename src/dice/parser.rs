use super::{DiceRoll, Modifier};
use anyhow::{Result, anyhow};
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
        let count: u32 = captures[1].parse()
            .map_err(|_| anyhow!("Invalid set count"))?;
        if count < 2 || count > 20 {
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
        original_expression: None, // Will be set by caller if needed
    };
    
    let mut remaining = input.trim();
    
    // Parse flags at the beginning
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
    
    // Parse label (parentheses)
    let label_regex = Regex::new(r"^\(([^)]+)\)\s*").unwrap();
    if let Some(captures) = label_regex.captures(remaining) {
        dice.label = Some(captures[1].to_string());
        remaining = &remaining[captures.get(0).unwrap().end()..];
    }
    
    // Parse comment (exclamation mark)
    let comment_regex = Regex::new(r"!\s*(.+)$").unwrap();
    if let Some(captures) = comment_regex.captures(remaining) {
        dice.comment = Some(captures[1].to_string());
        remaining = &remaining[..captures.get(0).unwrap().start()].trim();
    }
    
    // Parse the main dice expression and modifiers
    let parts: Vec<&str> = if remaining.chars().any(|c| matches!(c, '+' | '-' | '*' | '/')) && !remaining.contains(' ') {
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
    let dice_regex = Regex::new(r"^(\d+)?d(\d+|%)$").unwrap();
    if let Some(captures) = dice_regex.captures(parts[0]) {
        dice.count = captures.get(1)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);
        
        if &captures[2] == "%" {
            dice.sides = 100;
        } else {
            dice.sides = captures[2].parse()
                .map_err(|_| anyhow!("Invalid dice sides"))?;
        }
        
        if dice.count > 100 {
            return Err(anyhow!("Maximum 100 dice allowed"));
        }
        if dice.sides > 10000 {
            return Err(anyhow!("Maximum 10000 sides allowed"));
        }
    } else {
        return Err(anyhow!("Invalid dice expression: {}", parts[0]));
    }
    
    // Parse modifiers
    let mut i = 1;
    while i < parts.len() {
        // Handle operators followed by dice expressions (e.g., "+" "3d10")
        if parts[i] == "+" && i + 1 < parts.len() {
            let next_part = parts[i + 1];
            if let Ok(num) = next_part.parse::<i32>() {
                dice.modifiers.push(Modifier::Add(num));
                i += 2;
                continue;
            } else if is_dice_expression(next_part) {
                // Parse additional dice (e.g., "+ 3d10")
                let additional_dice = parse_dice_expression_only(next_part)?;
                dice.modifiers.push(Modifier::AddDice(additional_dice));
                i += 2;
                continue;
            }
        } else if parts[i] == "-" && i + 1 < parts.len() {
            let next_part = parts[i + 1];
            if let Ok(num) = next_part.parse::<i32>() {
                dice.modifiers.push(Modifier::Subtract(num));
                i += 2;
                continue;
            } else if is_dice_expression(next_part) {
                // Parse additional dice with subtraction (e.g., "- 2d6")
                let additional_dice = parse_dice_expression_only(next_part)?;
                dice.modifiers.push(Modifier::SubtractDice(additional_dice));
                i += 2;
                continue;
            }
        } else if parts[i] == "*" && i + 1 < parts.len() {
            if let Ok(num) = parts[i + 1].parse::<i32>() {
                dice.modifiers.push(Modifier::Multiply(num));
                i += 2;
                continue;
            }
        } else if parts[i] == "/" && i + 1 < parts.len() {
            if let Ok(num) = parts[i + 1].parse::<i32>() {
                dice.modifiers.push(Modifier::Divide(num));
                i += 2;
                continue;
            }
        }
        
        // Handle combined modifiers by splitting them first
        let split_modifiers = split_combined_modifiers(parts[i])?;
        for modifier_str in split_modifiers {
            let modifier = parse_modifier(&modifier_str)?;
            dice.modifiers.push(modifier);
        }
        i += 1;
    }
    
    Ok(dice)
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
        r"^[+\-*/]\d+$",  // +5, -3, *2, /4
        r"^\d+$",         // 5
        r"^ie\d*$",       // ie, ie6
        r"^ir\d+$",       // ir2
        r"^r\d+$",        // r2
        r"^e\d*$",        // e, e6 (but only if no other modifiers follow)
        r"^kl\d+$",       // kl2
        r"^k\d+$",        // k3 (but only if no other modifiers follow)
        r"^d\d+$",        // d2 (but only if no other modifiers follow)
        r"^t\d+$",        // t7
        r"^f\d+$",        // f1
        r"^b\d*$",        // b, b1
        r"^wng\d*t?$",    // wng, wng3, wngt, wng3t (Wrath & Glory with optional difficulty and total flag)
    ];
    
    simple_patterns.iter().any(|pattern| {
        Regex::new(pattern).unwrap().is_match(input)
    })
}

fn extract_next_modifier(input: &str) -> Result<(String, &str)> {
    // Try to match modifier patterns in order of complexity
    
    // Wrath & Glory modifier with optional difficulty and total flag
    if input.starts_with("wng") {
        let re = Regex::new(r"^(wng\d*t?)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Indefinite explode: ie or ie#
    if input.starts_with("ie") {
        let re = Regex::new(r"^(ie\d*)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Indefinite reroll: ir#
    if input.starts_with("ir") {
        let re = Regex::new(r"^(ir\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Keep low: kl#
    if input.starts_with("kl") {
        let re = Regex::new(r"^(kl\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Explode: e or e#
    if input.starts_with('e') && !input.starts_with("ie") {
        let re = Regex::new(r"^(e\d*)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Keep high: k#
    if input.starts_with('k') && !input.starts_with("kl") {
        let re = Regex::new(r"^(k\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Drop: d#
    if input.starts_with('d') {
        let re = Regex::new(r"^(d\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Reroll: r#
    if input.starts_with('r') && !input.starts_with("ir") {
        let re = Regex::new(r"^(r\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Target: t#
    if input.starts_with('t') {
        let re = Regex::new(r"^(t\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Failure: f#
    if input.starts_with('f') {
        let re = Regex::new(r"^(f\d+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // Botch: b or b#
    if input.starts_with('b') {
        let re = Regex::new(r"^(b\d*)").unwrap();
        if let Some(captures) = re.captures(input) {
            let modifier = captures[1].to_string();
            let rest = &input[modifier.len()..];
            return Ok((modifier, rest));
        }
    }
    
    // If no pattern matched, return empty
    Ok((String::new(), input))
}

fn parse_modifier(part: &str) -> Result<Modifier> {
    // Wrath & Glory success counting with optional difficulty and total flag
    if part.starts_with("wng") {
        let use_total = part.ends_with('t');
        let number_part = if use_total {
            &part[3..part.len()-1] // Remove "wng" and "t"
        } else {
            &part[3..] // Remove "wng"
        };
        
        let difficulty = if !number_part.is_empty() {
            Some(number_part.parse().map_err(|_| anyhow!("Invalid difficulty number"))?)
        } else {
            None
        };
        
        return Ok(Modifier::WrathGlory(difficulty, use_total));
    }
    
    // Numeric modifiers (positive numbers)
    if let Ok(num) = part.parse::<i32>() {
        return Ok(Modifier::Add(num));
    }
    
    // Operators with numbers (e.g., "+2", "-3", "*4", "/2")
    let op_regex = Regex::new(r"^([+\-*/])(\d+)$").unwrap();
    if let Some(captures) = op_regex.captures(part) {
        let num: i32 = captures[2].parse()
            .map_err(|_| anyhow!("Invalid modifier number"))?;
        match &captures[1] {
            "+" => return Ok(Modifier::Add(num)),
            "-" => return Ok(Modifier::Subtract(num)),
            "*" => return Ok(Modifier::Multiply(num)),
            "/" => return Ok(Modifier::Divide(num)),
            _ => {}
        }
    }
    
    // Special modifiers
    if part.starts_with("ie") {
        let num = if part.len() > 2 {
            Some(part[2..].parse().map_err(|_| anyhow!("Invalid explode value"))?)
        } else {
            None
        };
        return Ok(Modifier::ExplodeIndefinite(num));
    }
    
    if part.starts_with('e') {
        let num = if part.len() > 1 {
            Some(part[1..].parse().map_err(|_| anyhow!("Invalid explode value"))?)
        } else {
            None
        };
        return Ok(Modifier::Explode(num));
    }
    
    if part.starts_with("ir") {
        let num: u32 = part[2..].parse()
            .map_err(|_| anyhow!("Invalid reroll value"))?;
        return Ok(Modifier::RerollIndefinite(num));
    }
    
    if part.starts_with('r') {
        let num: u32 = part[1..].parse()
            .map_err(|_| anyhow!("Invalid reroll value"))?;
        return Ok(Modifier::Reroll(num));
    }
    
    if part.starts_with("kl") {
        let num: u32 = part[2..].parse()
            .map_err(|_| anyhow!("Invalid keep low value"))?;
        return Ok(Modifier::KeepLow(num));
    }
    
    if part.starts_with('k') {
        let num: u32 = part[1..].parse()
            .map_err(|_| anyhow!("Invalid keep value"))?;
        return Ok(Modifier::KeepHigh(num));
    }
    
    if part.starts_with('d') {
        let num: u32 = part[1..].parse()
            .map_err(|_| anyhow!("Invalid drop value"))?;
        return Ok(Modifier::Drop(num));
    }
    
    if part.starts_with('t') {
        let num: u32 = part[1..].parse()
            .map_err(|_| anyhow!("Invalid target value"))?;
        return Ok(Modifier::Target(num));
    }
    
    if part.starts_with('f') {
        let num: u32 = part[1..].parse()
            .map_err(|_| anyhow!("Invalid failure value"))?;
        return Ok(Modifier::Failure(num));
    }
    
    if part.starts_with('b') {
        let num = if part.len() > 1 {
            Some(part[1..].parse().map_err(|_| anyhow!("Invalid botch value"))?)
        } else {
            None
        };
        return Ok(Modifier::Botch(num));
    }
    
    // Additional dice (e.g., "+2d6", "-1d4")
    let dice_mod_regex = Regex::new(r"^([+\-])(\d+)d(\d+)$").unwrap();
    if let Some(captures) = dice_mod_regex.captures(part) {
        let count: u32 = captures[2].parse()
            .map_err(|_| anyhow!("Invalid dice count in modifier"))?;
        let sides: u32 = captures[3].parse()
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
        
        return Ok(Modifier::AddDice(add_dice));
    }
    
    Err(anyhow!("Unknown modifier: {}", part))
}

fn is_dice_expression(input: &str) -> bool {
    let dice_regex = Regex::new(r"^\d*d\d+$").unwrap();
    dice_regex.is_match(input)
}

fn parse_dice_expression_only(input: &str) -> Result<DiceRoll> {
    let dice_regex = Regex::new(r"^(\d+)?d(\d+)$").unwrap();
    if let Some(captures) = dice_regex.captures(input) {
        let count = captures.get(1)
            .map(|m| m.as_str().parse().unwrap_or(1))
            .unwrap_or(1);
        let sides = captures[2].parse()
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
