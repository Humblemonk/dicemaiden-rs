use super::{DiceGroup, DiceRoll, HeroSystemType, Modifier, RollResult};
use anyhow::{anyhow, Result};
use rand::Rng;

pub fn roll_dice(dice: DiceRoll) -> Result<RollResult> {
    // Validation check
    if dice.sides < 1 {
        return Err(anyhow!("Cannot roll dice with {} sides", dice.sides));
    }
    if dice.count == 0 {
        return Err(anyhow!("Cannot roll 0 dice"));
    }

    let mut rng = rand::thread_rng();
    let mut result = RollResult {
        individual_rolls: Vec::new(),
        kept_rolls: Vec::new(),
        dropped_rolls: Vec::new(),
        total: 0,
        successes: None,
        failures: None,
        botches: None,
        comment: dice.comment.clone(),
        label: dice.label.clone(),
        notes: Vec::new(),
        dice_groups: Vec::new(),
        original_expression: dice.original_expression.clone(),
        simple: dice.simple,
        no_results: dice.no_results,
        private: dice.private,
        godbound_damage: None,
        fudge_symbols: None,
        // Initialize Wrath & Glory fields
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        suppress_comment: false,
    };

    // Initial dice rolls
    for _ in 0..dice.count {
        let roll = rng.gen_range(1..=dice.sides as i32);
        result.individual_rolls.push(roll);
    }

    // Create initial dice group for the base dice
    let base_group = DiceGroup {
        _description: format!("{}d{}", dice.count, dice.sides),
        rolls: result.individual_rolls.clone(),
        modifier_type: "base".to_string(),
    };
    result.dice_groups.push(base_group);

    // FIXED: Apply modifiers in the correct order for mathematical precedence
    // 1. Apply dice-modifying modifiers first (exploding, rerolls, etc.)
    apply_dice_modifying_modifiers(&mut result, &mut rng, &dice)?;

    // 2. Apply keep/drop modifiers
    apply_keep_drop_modifiers(&mut result, &dice)?;

    // 3. Calculate base total from remaining dice
    if result.kept_rolls.is_empty() {
        result.kept_rolls = result.individual_rolls.clone();
    }
    result.total = result.kept_rolls.iter().sum();

    // 4. Apply mathematical modifiers (add, subtract, multiply, divide)
    apply_mathematical_modifiers(&mut result, &dice)?;

    // 5. Apply special system modifiers (after math modifiers for proper precedence)
    apply_special_system_modifiers(&mut result, &dice, &mut rng)?;

    // 6. Sort rolls unless unsorted flag is set
    if !dice.unsorted {
        sort_result_rolls(&mut result);
    }

    Ok(result)
}

// FIXED: Separate function for dice-modifying modifiers
fn apply_dice_modifying_modifiers(
    result: &mut RollResult,
    rng: &mut impl Rng,
    dice: &DiceRoll,
) -> Result<()> {
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Explode(threshold) => {
                explode_dice(result, rng, *threshold, dice.sides, false, dice)?;
                update_base_group(result);
            }
            Modifier::ExplodeIndefinite(threshold) => {
                explode_dice(result, rng, *threshold, dice.sides, true, dice)?;
                update_base_group(result);
            }
            Modifier::Reroll(threshold) => {
                reroll_dice(result, rng, *threshold, dice.sides, false)?;
                update_base_group(result);
            }
            Modifier::RerollIndefinite(threshold) => {
                reroll_dice(result, rng, *threshold, dice.sides, true)?;
                update_base_group(result);
            }
            _ => {} // Handle other modifiers later
        }
    }
    Ok(())
}

// FIXED: Separate function for keep/drop modifiers
fn apply_keep_drop_modifiers(result: &mut RollResult, dice: &DiceRoll) -> Result<()> {
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Drop(count) => {
                drop_dice(result, *count as usize)?;
            }
            Modifier::KeepHigh(count) => {
                keep_dice(result, *count as usize, false)?;
            }
            Modifier::KeepLow(count) => {
                keep_dice(result, *count as usize, true)?;
            }
            _ => {} // Skip modifiers already handled
        }
    }
    Ok(())
}

// FIXED: Mathematical modifiers with proper precedence evaluation
fn apply_mathematical_modifiers(result: &mut RollResult, dice: &DiceRoll) -> Result<()> {
    // Build an expression from the modifiers and evaluate it properly
    let mut expression_parts = Vec::new();
    
    // Start with the dice total
    expression_parts.push(format!("{}", result.total));
    
    // Add each mathematical operation as it appears
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::AddDice(dice_to_add) => {
                let additional_result = roll_dice(dice_to_add.clone())?;
                expression_parts.push("+".to_string());
                expression_parts.push(format!("{}", additional_result.total));
                handle_additional_dice(result, dice_to_add, "add", 1)?;
            }
            Modifier::SubtractDice(dice_to_subtract) => {
                let additional_result = roll_dice(dice_to_subtract.clone())?;
                expression_parts.push("-".to_string());
                expression_parts.push(format!("{}", additional_result.total));
                handle_additional_dice(result, dice_to_subtract, "subtract", -1)?;
            }
            Modifier::Add(value) => {
                expression_parts.push("+".to_string());
                expression_parts.push(format!("{}", value));
            }
            Modifier::Subtract(value) => {
                expression_parts.push("-".to_string());
                expression_parts.push(format!("{}", value));
            }
            Modifier::Multiply(value) => {
                expression_parts.push("*".to_string());
                expression_parts.push(format!("{}", value));
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                expression_parts.push("/".to_string());
                expression_parts.push(format!("{}", value));
            }
            _ => {}
        }
    }
    
    // Evaluate the expression with proper precedence
    if expression_parts.len() > 1 {
        result.total = evaluate_expression(&expression_parts)?;
    }

    Ok(())
}

// Simple expression evaluator with proper precedence
fn evaluate_expression(parts: &[String]) -> Result<i32> {
    if parts.len() == 1 {
        return Ok(parts[0].parse()?);
    }
    
    // Convert to tokens
    let mut tokens = Vec::new();
    for part in parts {
        if let Ok(num) = part.parse::<i32>() {
            tokens.push(Token::Number(num));
        } else {
            match part.as_str() {
                "+" => tokens.push(Token::Plus),
                "-" => tokens.push(Token::Minus),
                "*" => tokens.push(Token::Multiply),
                "/" => tokens.push(Token::Divide),
                _ => return Err(anyhow!("Invalid token: {}", part)),
            }
        }
    }
    
    // First pass: handle multiplication and division (left to right)
    let mut i = 1;
    while i < tokens.len() {
        if let Token::Multiply = tokens[i] {
            if let (Token::Number(left), Token::Number(right)) = (&tokens[i-1], &tokens[i+1]) {
                let result = left * right;
                tokens[i-1] = Token::Number(result);
                tokens.remove(i+1);
                tokens.remove(i);
                continue;
            }
        } else if let Token::Divide = tokens[i] {
            if let (Token::Number(left), Token::Number(right)) = (&tokens[i-1], &tokens[i+1]) {
                if *right == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                let result = left / right;
                tokens[i-1] = Token::Number(result);
                tokens.remove(i+1);
                tokens.remove(i);
                continue;
            }
        }
        i += 2; // Skip to next operator
    }
    
    // Second pass: handle addition and subtraction (left to right)
    let mut i = 1;
    while i < tokens.len() {
        if let Token::Plus = tokens[i] {
            if let (Token::Number(left), Token::Number(right)) = (&tokens[i-1], &tokens[i+1]) {
                let result = left + right;
                tokens[i-1] = Token::Number(result);
                tokens.remove(i+1);
                tokens.remove(i);
                continue;
            }
        } else if let Token::Minus = tokens[i] {
            if let (Token::Number(left), Token::Number(right)) = (&tokens[i-1], &tokens[i+1]) {
                let result = left - right;
                tokens[i-1] = Token::Number(result);
                tokens.remove(i+1);
                tokens.remove(i);
                continue;
            }
        }
        i += 2; // Skip to next operator
    }
    
    // Should have only one number left
    if tokens.len() == 1 {
        if let Token::Number(result) = tokens[0] {
            Ok(result)
        } else {
            Err(anyhow!("Invalid expression result"))
        }
    } else {
        Err(anyhow!("Expression did not evaluate to a single value"))
    }
}

#[derive(Debug, Clone)]
enum Token {
    Number(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
}

// FIXED: Special system modifiers applied after math
fn apply_special_system_modifiers(
    result: &mut RollResult,
    dice: &DiceRoll,
    rng: &mut impl Rng,
) -> Result<()> {
    // Check if we have mathematical modifiers that were applied
    let has_math_modifiers = dice.modifiers.iter().any(|m| {
        matches!(
            m,
            Modifier::Add(_)
                | Modifier::Subtract(_)
                | Modifier::Multiply(_)
                | Modifier::Divide(_)
                | Modifier::AddDice(_)
                | Modifier::SubtractDice(_)
        )
    });

    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Target(value) => {
                count_dice_matching(result, |roll| roll >= *value as i32, "successes")?;
            }
            Modifier::Failure(value) => {
                count_failures_and_subtract(result, *value)?;
            }
            Modifier::Botch(threshold) => {
                count_dice_matching(
                    result,
                    |roll| roll <= threshold.unwrap_or(1) as i32,
                    "botches",
                )?;
                let botch_count = result.botches.unwrap_or(0);
                if botch_count > 0 {
                    result.notes.push(format!(
                        "{} dice botched (≤{})",
                        botch_count,
                        threshold.unwrap_or(1)
                    ));
                }
            }
            Modifier::Fudge => {
                apply_fudge_conversion(result)?;
            }
            Modifier::WrathGlory(difficulty, use_total) => {
                count_wrath_glory_successes(result, *difficulty, *use_total)?;
            }
            Modifier::Godbound(straight_damage) => {
                apply_godbound_damage(result, *straight_damage, has_math_modifiers)?;
            }
            Modifier::HeroSystem(hero_type) => {
                apply_hero_system_calculation(result, rng, hero_type)?;
            }
            _ => {} // Skip modifiers already handled above
        }
    }

    // FIXED: Apply mathematical modifiers to success counts if applicable
    if result.successes.is_some() && has_math_modifiers {
        for modifier in &dice.modifiers {
            match modifier {
                Modifier::Add(value) => {
                    result.successes = Some(result.successes.unwrap() + value);
                }
                Modifier::Subtract(value) => {
                    result.successes = Some(result.successes.unwrap() - value);
                }
                Modifier::Multiply(value) => {
                    result.successes = Some(result.successes.unwrap() * value);
                }
                Modifier::Divide(value) => {
                    if *value == 0 {
                        return Err(anyhow!("Cannot divide by zero"));
                    }
                    result.successes = Some(result.successes.unwrap() / value);
                }
                _ => {}
            }
        }
    }

    // If target/success system or godbound was used, don't use the dice total
    if result.successes.is_some() || result.godbound_damage.is_some() {
        result.total = 0; // Reset total for special systems
    }

    Ok(())
}

// Helper function to sort result rolls
fn sort_result_rolls(result: &mut RollResult) {
    // Sort kept_rolls
    if !result.kept_rolls.is_empty() {
        result.kept_rolls.sort_by(|a, b| b.cmp(a)); // Sort descending by default
    }

    // Sort all dice groups' rolls as well
    for group in &mut result.dice_groups {
        group.rolls.sort_by(|a, b| b.cmp(a)); // Sort descending by default
    }
}

// Helper function to update the base group with current rolls
fn update_base_group(result: &mut RollResult) {
    if let Some(base_group) = result.dice_groups.get_mut(0) {
        base_group.rolls = result.individual_rolls.clone();
    }
}

// Generic function for counting dice that match a condition
fn count_dice_matching<F>(result: &mut RollResult, condition: F, count_type: &str) -> Result<()>
where
    F: Fn(i32) -> bool,
{
    let count = result
        .individual_rolls
        .iter()
        .filter(|&&roll| condition(roll))
        .count() as i32;

    match count_type {
        "successes" => {
            result.successes = Some(result.successes.unwrap_or(0) + count);
        }
        "botches" => {
            result.botches = Some(count);
        }
        _ => {}
    }
    Ok(())
}

// Handle failures with subtraction from successes
fn count_failures_and_subtract(result: &mut RollResult, threshold: u32) -> Result<()> {
    let failures = result
        .individual_rolls
        .iter()
        .filter(|&&roll| roll <= threshold as i32)
        .count() as i32;

    result.failures = Some(result.failures.unwrap_or(0) + failures);

    // Subtract failures from successes
    if let Some(ref mut successes) = result.successes {
        *successes -= failures;
    }

    Ok(())
}

// Handle adding or subtracting additional dice
fn handle_additional_dice(
    result: &mut RollResult,
    dice: &DiceRoll,
    modifier_type: &str,
    multiplier: i32,
) -> Result<()> {
    let additional_result = roll_dice(dice.clone())?;
    result
        .individual_rolls
        .extend(additional_result.individual_rolls.clone());
    result.total += additional_result.total * multiplier;

    // Add a new dice group for the additional dice
    let dice_group = DiceGroup {
        _description: format!("{}d{}", dice.count, dice.sides),
        rolls: additional_result.individual_rolls,
        modifier_type: modifier_type.to_string(),
    };
    result.dice_groups.push(dice_group);
    Ok(())
}

fn count_wrath_glory_successes(
    result: &mut RollResult,
    difficulty: Option<u32>,
    use_total: bool,
) -> Result<()> {
    let mut wrath_die_value = 0;
    let mut has_complication = false;
    let mut has_critical = false;

    if use_total {
        // For soak/damage/exempt rolls, just use the total of dice values
        result.total = result.individual_rolls.iter().sum();
        result.successes = None; // Don't show successes for total-based rolls

        // Still check wrath die effects (first die only) but don't show critical/glory for soak rolls
        if let Some(&first_die) = result.individual_rolls.first() {
            wrath_die_value = first_die;
            if first_die == 1 {
                has_complication = true;
            }
        }

        // Check difficulty if specified (comparing total to difficulty)
        if let Some(dn) = difficulty {
            let passed = result.total >= dn as i32;
            let status = if passed { "PASS" } else { "FAIL" };
            result.notes.push(format!(
                "Difficulty {}: {} (needed {}, rolled {})",
                dn, status, dn, result.total
            ));
        }

        // Add notes for wrath die effects (only complications for soak rolls)
        if has_complication {
            result
                .notes
                .push("Wrath die rolled 1 - Complication!".to_string());
            result.notes.push(format!("Wrath die: {}", wrath_die_value));
        }
    } else {
        // Standard Wrath & Glory success counting
        let mut total_successes = 0;
        let mut icon_count = 0;
        let mut exalted_icon_count = 0;

        // In Wrath & Glory, one die is designated as the "wrath die"
        // For simplicity, we'll treat the first die as the wrath die
        for (i, &roll) in result.individual_rolls.iter().enumerate() {
            let successes = match roll {
                1..=3 => 0, // No successes
                4..=5 => {
                    // Icons (1 success)
                    icon_count += 1;
                    1
                }
                6 => {
                    // Exalted Icons (2 successes)
                    exalted_icon_count += 1;
                    2
                }
                _ => 0, // Shouldn't happen with normal dice
            };

            total_successes += successes;

            // Check wrath die effects (first die only)
            if i == 0 {
                wrath_die_value = roll;
                if roll == 1 {
                    has_complication = true;
                } else if roll == 6 {
                    has_critical = true;
                }
            }
        }

        // Set Wrath & Glory specific fields
        result.wng_wrath_die = Some(wrath_die_value);
        result.wng_icons = Some(icon_count);
        result.wng_exalted_icons = Some(exalted_icon_count);

        result.successes = Some(total_successes);
        result.total = 0; // Don't use total for success-based systems

        // Check difficulty if specified (comparing successes to difficulty)
        if let Some(dn) = difficulty {
            let passed = total_successes >= dn as i32;
            let status = if passed { "PASS" } else { "FAIL" };
            result
                .notes
                .push(format!("Difficulty {}: {} (needed {})", dn, status, dn));
        }

        // Add notes for wrath die effects
        add_wrath_die_notes(result, has_complication, has_critical);
    }

    Ok(())
}

// Helper function for wrath die notes to reduce duplication
fn add_wrath_die_notes(
    result: &mut RollResult,
    has_complication: bool,
    has_critical: bool,
) {
    if has_complication {
        result
            .notes
            .push("Wrath die rolled 1 - Complication!".to_string());
    }
    if has_critical {
        result
            .notes
            .push("Wrath die rolled 6 - Critical/Glory!".to_string());
    }
}

fn apply_godbound_damage(
    result: &mut RollResult,
    straight_damage: bool,
    has_math_modifiers: bool,
) -> Result<()> {
    if straight_damage {
        // Straight damage - use the final total (including all modifiers)
        result.godbound_damage = Some(result.total);
        result
            .notes
            .push("Straight damage (bypasses chart)".to_string());
    } else {
        if has_math_modifiers {
            // If we have mathematical modifiers, convert the final total
            let damage = convert_to_godbound_damage(result.total);
            result.godbound_damage = Some(damage);
            result
                .notes
                .push(format!("Damage chart: {} → {}", result.total, damage));
        } else {
            // If no mathematical modifiers, convert each die individually and sum
            let mut total_damage = 0;
            let mut chart_conversions = Vec::new();

            for &roll in &result.kept_rolls {
                let damage = convert_to_godbound_damage(roll);
                total_damage += damage;
                chart_conversions.push(format!("{} → {}", roll, damage));
            }

            result.godbound_damage = Some(total_damage);

            // Add detailed conversion note if there are multiple dice
            if result.kept_rolls.len() > 1 {
                result.notes.push(format!(
                    "Damage chart conversions: [{}]",
                    chart_conversions.join(", ")
                ));
            } else if let Some(&roll) = result.kept_rolls.first() {
                result.notes.push(format!(
                    "Damage chart: {} → {}",
                    roll,
                    convert_to_godbound_damage(roll)
                ));
            }
        }

        result
            .notes
            .push("Using Godbound damage chart (1-=0, 2-5=1, 6-9=2, 10+=4)".to_string());
    }

    Ok(())
}

fn convert_to_godbound_damage(value: i32) -> i32 {
    match value {
        ..=1 => 0,  // 1 or less = 0 damage
        2..=5 => 1, // 2-5 = 1 damage
        6..=9 => 2, // 6-9 = 2 damage
        _ => 4,     // 10+ = 4 damage
    }
}

fn explode_dice(
    result: &mut RollResult,
    rng: &mut impl Rng,
    threshold: Option<u32>,
    dice_sides: u32,
    indefinite: bool,
    dice: &DiceRoll,
) -> Result<()> {
    let explode_on = threshold.unwrap_or(dice_sides);

    let mut explosion_count = 0;
    let max_explosions = if indefinite { 100 } else { 1 };

    let mut i = 0;
    while i < result.individual_rolls.len() && explosion_count < max_explosions {
        if result.individual_rolls[i] >= explode_on as i32 {
            let new_roll = rng.gen_range(1..=dice_sides as i32);
            result.individual_rolls.push(new_roll);
            explosion_count += 1;

            if !indefinite {
                break;
            }
        }
        i += 1;
    }

    if explosion_count >= max_explosions && indefinite {
        result
            .notes
            .push("Maximum explosions reached (100)".to_string());
    }

    if explosion_count > 0 {
        add_explosion_notes(
            result,
            explosion_count,
            dice_sides,
            explode_on,
            indefinite,
            dice,
        );
    }

    Ok(())
}

// Helper function for explosion notes
fn add_explosion_notes(
    result: &mut RollResult,
    explosion_count: usize,
    _dice_sides: u32,
    _explode_on: u32,
    _indefinite: bool,
    dice: &DiceRoll,
) {
    // Check if this is explicitly a Dark Heresy roll
    let is_dark_heresy = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::DarkHeresy));

    if is_dark_heresy {
        // Dark Heresy righteous fury
        if explosion_count == 1 {
            result
                .notes
                .push("⚔️ **RIGHTEOUS FURY!** Natural 10 rolled - Purge the heretics!".to_string());
        } else {
            result.notes.push(format!(
                "⚔️ **RIGHTEOUS FURY!** {} natural 10s - Emperor's wrath unleashed!",
                explosion_count
            ));
        }
    } else {
        // Generic exploding dice message for all other systems
        if explosion_count == 1 {
            result.notes.push("1 die exploded".to_string());
        } else {
            result
                .notes
                .push(format!("{} dice exploded", explosion_count));
        }
    }
}

// FIXED: Better drop dice with proper error handling
fn drop_dice(result: &mut RollResult, count: usize) -> Result<()> {
    let available_dice = result.individual_rolls.len();

    if count >= available_dice {
        // Don't allow dropping all dice - that would be an error
        if count == available_dice {
            return Err(anyhow!("Cannot drop all dice"));
        }
        
        // Provide context-aware error message
        result.notes.push(format!(
            "Cannot drop {} dice - only {} dice available",
            count, available_dice
        ));
        return Ok(());
    }

    let mut rolls = result.individual_rolls.clone();
    rolls.sort();

    // Drop lowest dice
    for _ in 0..count.min(rolls.len()) {
        if let Some(pos) = result.individual_rolls.iter().position(|&x| x == rolls[0]) {
            let dropped = result.individual_rolls.remove(pos);
            result.dropped_rolls.push(dropped);
            rolls.remove(0);
        }
    }

    Ok(())
}

// FIXED: Better keep dice with proper validation
fn keep_dice(result: &mut RollResult, count: usize, keep_low: bool) -> Result<()> {
    if count >= result.individual_rolls.len() {
        return Ok(()); // Keep all dice
    }

    // FIXED: Validate that count > 0
    if count == 0 {
        return Err(anyhow!("Cannot keep 0 dice"));
    }

    let mut indexed_rolls: Vec<(usize, i32)> = result
        .individual_rolls
        .iter()
        .enumerate()
        .map(|(i, &roll)| (i, roll))
        .collect();

    // Sort by value
    if keep_low {
        indexed_rolls.sort_by_key(|&(_, roll)| roll);
    } else {
        indexed_rolls.sort_by_key(|&(_, roll)| -roll);
    }

    // Keep the specified number of dice, drop the rest
    let kept_indices: Vec<usize> = indexed_rolls.iter().take(count).map(|&(i, _)| i).collect();

    let mut new_rolls = Vec::new();
    for (i, &roll) in result.individual_rolls.iter().enumerate() {
        if kept_indices.contains(&i) {
            new_rolls.push(roll);
        } else {
            result.dropped_rolls.push(roll);
        }
    }

    result.individual_rolls = new_rolls;
    Ok(())
}

fn reroll_dice(
    result: &mut RollResult,
    rng: &mut impl Rng,
    threshold: u32,
    dice_sides: u32,
    indefinite: bool,
) -> Result<()> {
    let mut total_rerolls = 0;
    let max_total_rerolls = 100;
    let mut reroll_notes = Vec::new();

    for i in 0..result.individual_rolls.len() {
        let mut rerolls_for_this_die = 0;
        let max_rerolls_per_die = if indefinite { 100 } else { 1 };
        let original_roll = result.individual_rolls[i];

        while result.individual_rolls[i] <= threshold as i32
            && rerolls_for_this_die < max_rerolls_per_die
            && total_rerolls < max_total_rerolls
        {
            let old_roll = result.individual_rolls[i];
            result.individual_rolls[i] = rng.gen_range(1..=dice_sides as i32);
            rerolls_for_this_die += 1;
            total_rerolls += 1;

            if !indefinite {
                // For single rerolls, show the immediate result
                reroll_notes.push(format!(
                    "Rerolled {} → {}",
                    old_roll, result.individual_rolls[i]
                ));
                break;
            }
        }

        // For indefinite rerolls, show original → final if any rerolls happened
        if indefinite && rerolls_for_this_die > 0 {
            if rerolls_for_this_die == 1 {
                reroll_notes.push(format!(
                    "Rerolled {} → {}",
                    original_roll, result.individual_rolls[i]
                ));
            } else {
                reroll_notes.push(format!(
                    "Rerolled {} → {} ({} rerolls)",
                    original_roll, result.individual_rolls[i], rerolls_for_this_die
                ));
            }
        }
    }

    // Add reroll notes to result
    for note in reroll_notes {
        result.notes.push(note);
    }

    // Safety check note
    if total_rerolls >= max_total_rerolls {
        result
            .notes
            .push("Maximum rerolls reached (100)".to_string());
    }

    // Always show summary if rerolls happened, regardless of count
    if total_rerolls > 0 {
        let reroll_type = if indefinite { "indefinitely" } else { "once" };
        if total_rerolls > 10 {
            result.notes.push(format!(
                "{} total rerolls (dice ≤ {}, reroll {})",
                total_rerolls, threshold, reroll_type
            ));
        } else if result.notes.len() <= 1 {
            // Only add summary if we don't already have detailed notes
            result.notes.push(format!(
                "{} dice rerolled (≤ {}, reroll {})",
                total_rerolls, threshold, reroll_type
            ));
        }
    }

    Ok(())
}

// Hero System calculation function
fn apply_hero_system_calculation(
    result: &mut RollResult,
    rng: &mut impl Rng,
    hero_type: &HeroSystemType,
) -> Result<()> {
    match hero_type {
        HeroSystemType::Normal => {
            // Normal damage - just use the total as-is
            result.notes.push("Normal damage".to_string());
        }
        HeroSystemType::Killing => {
            // Killing damage: BODY = dice total, STUN = BODY × multiplier (1d3)
            let body_damage = result.total;
            let stun_multiplier = rng.gen_range(1..=3);
            let stun_damage = body_damage * stun_multiplier;

            result.notes.push(format!(
                "Killing damage: {} BODY, {} STUN (×{})",
                body_damage, stun_damage, stun_multiplier
            ));

            // Override the total to show STUN damage (more commonly used)
            result.total = stun_damage;
        }
        HeroSystemType::Hit => {
            // To hit roll - 3d6 roll-under, typically against 11 + OCV - DCV
            // Just provide helpful context for Hero System to-hit mechanics
            result
                .notes
                .push("Hero System to-hit roll (3d6 roll-under)".to_string());
            result
                .notes
                .push("Target: 11 + OCV - DCV or less".to_string());
        }
    }

    Ok(())
}

fn apply_fudge_conversion(result: &mut RollResult) -> Result<()> {
    let mut symbols = Vec::new();
    let mut fudge_total = 0;

    for &roll in &result.individual_rolls {
        let (symbol, value) = match roll {
            1 => ("-", -1), // Minus
            2 => (" ", 0),  // Blank
            3 => ("+", 1),  // Plus
            _ => return Err(anyhow!("Invalid Fudge die value: {}", roll)),
        };
        symbols.push(symbol.to_string());
        fudge_total += value;
    }

    result.fudge_symbols = Some(symbols);
    result.total = fudge_total;
    result
        .notes
        .push("Fudge dice: 1=(-), 2=( ), 3=(+)".to_string());

    Ok(())
}
