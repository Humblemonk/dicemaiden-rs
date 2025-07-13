use super::{DiceGroup, DiceRoll, HeroSystemType, Modifier, RollResult};
use anyhow::{Result, anyhow};
use rand::Rng;

pub fn roll_dice(dice: DiceRoll) -> Result<RollResult> {
    // Validation check
    if dice.sides < 1 {
        return Err(anyhow!("Cannot roll dice with {} sides", dice.sides));
    }
    if dice.count == 0 {
        return Err(anyhow!("Cannot roll 0 dice"));
    }

    let mut rng = rand::rng();

    // Check for Conan system handlers
    let has_conan_skill = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::ConanSkill(_)));

    if has_conan_skill {
        return handle_conan_skill_roll(dice, &mut rng);
    }

    let has_conan_combat = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::ConanCombat(_)));

    if has_conan_combat {
        return handle_conan_combat_roll(dice, &mut rng);
    }

    // Check if this is a D6 System roll - handle it specially
    let has_d6_system = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::D6System(_, _)));

    if has_d6_system {
        return handle_d6_system_roll(dice, &mut rng);
    }

    let has_marvel_multiverse = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::MarvelMultiverse(_, _)));

    if has_marvel_multiverse {
        return handle_marvel_multiverse_roll(dice, &mut rng);
    }

    // Check if this is a Savage Worlds roll - handle it specially
    let has_savage_worlds = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::SavageWorlds(_)));

    if has_savage_worlds {
        // For Savage Worlds, handle it completely differently
        return handle_savage_worlds_roll(dice, &mut rng);
    }

    let has_brave_new_world = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::BraveNewWorld(_)));

    if has_brave_new_world {
        return handle_brave_new_world_roll(dice, &mut rng);
    }

    // Check if this is a Silhouette roll - handle it specially
    let has_silhouette = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::Silhouette(_)));

    if has_silhouette {
        return handle_silhouette_roll(dice, &mut rng);
    }

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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Normal dice rolling flow for non-special systems
    // Initial dice rolls
    for _ in 0..dice.count {
        let roll = rng.random_range(1..=dice.sides as i32);
        result.individual_rolls.push(roll);
    }

    // Create initial dice group for the base dice
    let base_group = DiceGroup {
        _description: format!("{}d{}", dice.count, dice.sides),
        rolls: result.individual_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    };
    result.dice_groups.push(base_group);

    // Apply modifiers in the correct order for mathematical precedence
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
    apply_mathematical_modifiers(&mut result, &dice, &mut rng)?;

    // 5. Apply special system modifiers (after math modifiers for proper precedence)
    apply_special_system_modifiers(&mut result, &dice, &mut rng)?;

    // 6. Sort rolls unless unsorted flag is set
    if !dice.unsorted {
        sort_result_rolls(&mut result);
    }

    Ok(result)
}

// Separate function for dice-modifying modifiers
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
            Modifier::RerollGreater(threshold) => {
                reroll_dice_greater(result, rng, *threshold, dice.sides, false)?;
                update_base_group(result);
            }
            Modifier::RerollGreaterIndefinite(threshold) => {
                reroll_dice_greater(result, rng, *threshold, dice.sides, true)?;
                update_base_group(result);
            }
            _ => {} // Handle other modifiers later
        }
    }
    Ok(())
}

// Separate function for keep/drop modifiers
fn apply_keep_drop_modifiers(result: &mut RollResult, dice: &DiceRoll) -> Result<()> {
    // Apply modifiers in the order they appear, not by type
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Drop(count) => {
                if *count == 0 {
                    continue; // d0 is a no-op
                }
                drop_dice(result, *count as usize)?;
            }
            Modifier::KeepHigh(count) => {
                if *count == 0 {
                    return Err(anyhow!("Cannot keep 0 dice"));
                }
                keep_dice(result, *count as usize, false)?;
            }
            Modifier::KeepLow(count) => {
                if *count == 0 {
                    return Err(anyhow!("Cannot keep 0 dice"));
                }
                keep_dice(result, *count as usize, true)?;
            }
            Modifier::KeepMiddle(count) => {
                if *count == 0 {
                    return Err(anyhow!("Cannot keep 0 dice"));
                }
                keep_middle_dice(result, *count as usize)?;
            }
            _ => {} // Skip modifiers already handled
        }
    }
    Ok(())
}

// Update apply_mathematical_modifiers to handle the special division case AND continue with remaining modifiers
fn apply_mathematical_modifiers(
    result: &mut RollResult,
    dice: &DiceRoll,
    _rng: &mut impl Rng,
) -> Result<()> {
    // Check for special division pattern: Multiply(0) followed by Add(number)
    if dice.modifiers.len() >= 2 {
        if let (Modifier::Multiply(0), Modifier::Add(number)) =
            (&dice.modifiers[0], &dice.modifiers[1])
        {
            // This is our special "number / dice" case
            if result.total == 0 {
                return Err(anyhow!("Cannot divide by zero (dice result was 0)"));
            }
            result.total = number / result.total;

            // IMPORTANT: Continue processing remaining modifiers starting from index 2
            let remaining_modifiers = &dice.modifiers[2..];
            if !remaining_modifiers.is_empty() {
                apply_remaining_mathematical_modifiers(result, remaining_modifiers, dice)?;
            }
            return Ok(());
        }
    }

    // Standard mathematical modifier processing
    apply_all_mathematical_modifiers(result, dice)?;
    Ok(())
}

// New function to apply remaining modifiers after special division
fn apply_remaining_mathematical_modifiers(
    result: &mut RollResult,
    modifiers: &[Modifier],
    _dice: &DiceRoll,
) -> Result<()> {
    // Build an expression from the remaining modifiers
    let mut expression_parts = Vec::new();

    // Start with current total
    expression_parts.push(format!("{}", result.total));

    // Add each remaining mathematical operation
    for modifier in modifiers {
        match modifier {
            Modifier::AddDice(dice_to_add) => {
                let additional_result = roll_dice(dice_to_add.clone())?;
                expression_parts.push("+".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                // Add dice to individual_rolls for display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Add a new dice group for display
                add_dice_group(result, dice_to_add, &additional_result, "add");
            }
            Modifier::SubtractDice(dice_to_subtract) => {
                let additional_result = roll_dice(dice_to_subtract.clone())?;
                expression_parts.push("-".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                // Add dice to individual_rolls for display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Add a new dice group for display
                add_dice_group(result, dice_to_subtract, &additional_result, "subtract");
            }
            Modifier::MultiplyDice(dice_to_multiply) => {
                // Handle dice multiplication in remaining modifiers
                let additional_result = roll_dice(dice_to_multiply.clone())?;
                expression_parts.push("*".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                add_dice_group(result, dice_to_multiply, &additional_result, "multiply");
            }
            Modifier::DivideDice(dice_to_divide) => {
                // Handle dice division in remaining modifiers
                let additional_result = roll_dice(dice_to_divide.clone())?;

                if additional_result.total == 0 {
                    return Err(anyhow!("Cannot divide by zero (dice result was 0)"));
                }

                expression_parts.push("/".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                add_dice_group(result, dice_to_divide, &additional_result, "divide");
            }
            Modifier::Add(value) => {
                expression_parts.push("+".to_string());
                expression_parts.push(format!("{value}"));
            }
            Modifier::Subtract(value) => {
                expression_parts.push("-".to_string());
                expression_parts.push(format!("{value}"));
            }
            Modifier::Multiply(value) => {
                expression_parts.push("*".to_string());
                expression_parts.push(format!("{value}"));
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                expression_parts.push("/".to_string());
                expression_parts.push(format!("{value}"));
            }
            _ => {} // Skip non-mathematical modifiers
        }
    }

    // Evaluate the expression if we have additional operations
    if expression_parts.len() > 1 {
        result.total = evaluate_expression(&expression_parts)?;
    }

    Ok(())
}

// Function to apply all mathematical modifiers (for standard case)
fn apply_all_mathematical_modifiers(result: &mut RollResult, dice: &DiceRoll) -> Result<()> {
    // Build an expression from the modifiers and evaluate it properly
    let mut expression_parts = Vec::new();

    // Start with the dice total
    expression_parts.push(format!("{}", result.total));

    // Add each mathematical operation as it appears
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::AddDice(dice_to_add) => {
                // Roll the additional dice only once and use that result consistently
                let additional_result = roll_dice(dice_to_add.clone())?;
                expression_parts.push("+".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                // IMPORTANT: Merge notes from AddDice into main result
                // This ensures Hero System notes are preserved
                result.notes.extend(additional_result.notes.clone());

                // Add dice to individual_rolls for display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());

                // Add dice to kept_rolls so all totals are consistent
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Add a new dice group for display using the SAME rolled dice
                add_dice_group(
                    result,
                    dice_to_add,
                    &additional_result, // Pass full result instead of just rolls
                    "add",
                );
            }
            Modifier::SubtractDice(dice_to_subtract) => {
                // Roll the additional dice only once and use that result consistently
                let additional_result = roll_dice(dice_to_subtract.clone())?;
                expression_parts.push("-".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                // Add dice to individual_rolls for display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());

                // For subtraction, we still add the dice to kept_rolls for display
                // The subtraction is handled in the expression evaluation
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Add a new dice group for display using the SAME rolled dice
                add_dice_group(result, dice_to_subtract, &additional_result, "subtract");
            }
            Modifier::MultiplyDice(dice_to_multiply) => {
                // Handle dice multiplication
                let additional_result = roll_dice(dice_to_multiply.clone())?;
                expression_parts.push("*".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                // Add dice to individual_rolls for display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());

                // Add dice to kept_rolls for display
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Add a new dice group for display
                add_dice_group(result, dice_to_multiply, &additional_result, "multiply");
            }
            Modifier::DivideDice(dice_to_divide) => {
                //  Handle dice division
                let additional_result = roll_dice(dice_to_divide.clone())?;

                // Check for division by zero
                if additional_result.total == 0 {
                    return Err(anyhow!("Cannot divide by zero (dice result was 0)"));
                }

                expression_parts.push("/".to_string());
                expression_parts.push(format!("{}", additional_result.total));

                // Add dice to individual_rolls for display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());

                // Add dice to kept_rolls for display
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Add a new dice group for display
                add_dice_group(result, dice_to_divide, &additional_result, "divide");
            }
            Modifier::Add(value) => {
                expression_parts.push("+".to_string());
                expression_parts.push(format!("{value}"));
            }
            Modifier::Subtract(value) => {
                expression_parts.push("-".to_string());
                expression_parts.push(format!("{value}"));
            }
            Modifier::Multiply(value) => {
                // Skip the special marker (multiply by 0)
                if *value != 0 {
                    expression_parts.push("*".to_string());
                    expression_parts.push(format!("{value}"));
                }
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                expression_parts.push("/".to_string());
                expression_parts.push(format!("{value}"));
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

// Helper function to add dice groups, reducing duplication
fn add_dice_group(
    result: &mut RollResult,
    dice_spec: &DiceRoll,
    additional_result: &RollResult,
    modifier_type: &str,
) {
    // Combine individual_rolls (kept) + dropped_rolls to get all original dice
    let mut all_original_rolls = additional_result.individual_rolls.clone();
    all_original_rolls.extend(additional_result.dropped_rolls.clone());

    let dice_group = DiceGroup {
        _description: format!("{}d{}", dice_spec.count, dice_spec.sides),
        rolls: all_original_rolls,
        dropped_rolls: additional_result.dropped_rolls.clone(),
        modifier_type: modifier_type.to_string(),
    };
    result.dice_groups.push(dice_group);
}

// Simple expression evaluator with LEFT-TO-RIGHT evaluation (no PEMDAS)
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

    // Evaluate LEFT-TO-RIGHT (no precedence rules)
    apply_left_to_right_operations(&mut tokens)?;

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

// Helper function to apply operations strictly left-to-right
fn apply_left_to_right_operations(tokens: &mut Vec<Token>) -> Result<()> {
    // Process operations from left to right, one at a time
    while tokens.len() > 1 {
        // Find the first operator
        let mut operator_pos = None;
        for (i, token) in tokens.iter().enumerate() {
            if matches!(
                token,
                Token::Plus | Token::Minus | Token::Multiply | Token::Divide
            ) {
                operator_pos = Some(i);
                break;
            }
        }

        if let Some(op_pos) = operator_pos {
            // We need at least one number before and after the operator
            if op_pos == 0 || op_pos >= tokens.len() - 1 {
                return Err(anyhow!("Invalid expression structure"));
            }

            // Get the left operand, operator, and right operand
            if let (Token::Number(left), op, Token::Number(right)) =
                (&tokens[op_pos - 1], &tokens[op_pos], &tokens[op_pos + 1])
            {
                let result = match op {
                    Token::Plus => left + right,
                    Token::Minus => left - right,
                    Token::Multiply => left * right,
                    Token::Divide => {
                        if *right == 0 {
                            return Err(anyhow!("Cannot divide by zero"));
                        }
                        left / right
                    }
                    _ => return Err(anyhow!("Unexpected token type")),
                };

                // Replace the three tokens (left operand, operator, right operand) with the result
                tokens[op_pos - 1] = Token::Number(result);
                tokens.remove(op_pos + 1); // Remove right operand
                tokens.remove(op_pos); // Remove operator
            } else {
                return Err(anyhow!("Invalid operands for operator"));
            }
        } else {
            // No more operators found but we still have multiple tokens
            return Err(anyhow!("Expression contains non-operator tokens"));
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Token {
    Number(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
}

// Special system modifiers applied after math
fn apply_special_system_modifiers(
    result: &mut RollResult,
    dice: &DiceRoll,
    rng: &mut impl Rng,
) -> Result<()> {
    // Find positions of target-based modifiers
    let target_positions = find_target_modifier_positions(&dice.modifiers);

    // Check if we have mathematical modifiers
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

    // Track if we've applied a special system (success counting, etc.)
    let mut has_special_system = false;

    // Process modifiers in order, respecting their position relative to targets
    for (index, modifier) in dice.modifiers.iter().enumerate() {
        match modifier {
            // TargetWithDoubleSuccess must come BEFORE existing Target case
            Modifier::TargetWithDoubleSuccess(target, double_success_value) => {
                // Apply any mathematical modifiers that come BEFORE this target
                if has_math_modifiers {
                    let pre_target_modifiers: Vec<_> = dice.modifiers[..index]
                        .iter()
                        .filter(|m| {
                            matches!(
                                m,
                                Modifier::Add(_)
                                    | Modifier::Subtract(_)
                                    | Modifier::Multiply(_)
                                    | Modifier::Divide(_)
                            )
                        })
                        .cloned()
                        .collect();

                    if !pre_target_modifiers.is_empty() {
                        apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
                    }
                }

                // Call our new function
                count_dice_with_double_success(result, *target, *double_success_value)?;
                has_special_system = true;
            }
            Modifier::TargetLowerWithDoubleSuccess(target, double_value) => {
                // Apply pre-target mathematical modifiers
                if has_math_modifiers {
                    let pre_target_modifiers: Vec<_> = dice.modifiers[..index]
                        .iter()
                        .filter(|m| {
                            matches!(
                                m,
                                Modifier::Add(_)
                                    | Modifier::Subtract(_)
                                    | Modifier::Multiply(_)
                                    | Modifier::Divide(_)
                            )
                        })
                        .cloned()
                        .collect();

                    if !pre_target_modifiers.is_empty() {
                        apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
                    }
                }

                // Call target lower double success function
                count_dice_with_target_lower_double_success(result, *target, *double_value)?;
                has_special_system = true;
            }
            Modifier::Target(value) => {
                // Apply any mathematical modifiers that come BEFORE this target
                if has_math_modifiers {
                    let pre_target_modifiers: Vec<_> = dice.modifiers[..index]
                        .iter()
                        .filter(|m| {
                            matches!(
                                m,
                                Modifier::Add(_)
                                    | Modifier::Subtract(_)
                                    | Modifier::Multiply(_)
                                    | Modifier::Divide(_)
                            )
                        })
                        .cloned()
                        .collect();

                    if !pre_target_modifiers.is_empty() {
                        apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
                    }
                }

                count_dice_matching(result, |roll| roll >= *value as i32, "successes")?;
                has_special_system = true;
            }
            Modifier::TargetLower(value) => {
                // Apply any mathematical modifiers that come BEFORE this target
                if has_math_modifiers {
                    let pre_target_modifiers: Vec<_> = dice.modifiers[..index]
                        .iter()
                        .filter(|m| {
                            matches!(
                                m,
                                Modifier::Add(_)
                                    | Modifier::Subtract(_)
                                    | Modifier::Multiply(_)
                                    | Modifier::Divide(_)
                            )
                        })
                        .cloned()
                        .collect();

                    if !pre_target_modifiers.is_empty() {
                        apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
                    }
                }

                count_dice_matching(result, |roll| roll <= *value as i32, "successes")?;
                has_special_system = true;
            }
            Modifier::Failure(value) => {
                // Apply pre-target modifiers for failures too
                if has_math_modifiers {
                    let pre_target_modifiers: Vec<_> = dice.modifiers[..index]
                        .iter()
                        .filter(|m| {
                            matches!(
                                m,
                                Modifier::Add(_)
                                    | Modifier::Subtract(_)
                                    | Modifier::Multiply(_)
                                    | Modifier::Divide(_)
                            )
                        })
                        .cloned()
                        .collect();

                    if !pre_target_modifiers.is_empty() {
                        apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
                    }
                }

                count_failures_and_subtract(result, *value)?;
            }
            Modifier::Botch(threshold) => {
                // Apply pre-target modifiers for botches too
                if has_math_modifiers {
                    let pre_target_modifiers: Vec<_> = dice.modifiers[..index]
                        .iter()
                        .filter(|m| {
                            matches!(
                                m,
                                Modifier::Add(_)
                                    | Modifier::Subtract(_)
                                    | Modifier::Multiply(_)
                                    | Modifier::Divide(_)
                            )
                        })
                        .cloned()
                        .collect();

                    if !pre_target_modifiers.is_empty() {
                        apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
                    }
                }

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
            Modifier::Cancel => {
                // Apply cancel modifier (10s cancel 1s for World of Darkness)
                apply_cancel_modifier(result)?;
                has_special_system = true;
            }
            // Handle all other modifiers normally...
            Modifier::Fudge => {
                apply_fudge_conversion(result)?;
            }
            Modifier::WrathGlory(difficulty, use_total, wrath_dice_count) => {
                count_wrath_glory_successes(result, *difficulty, *use_total, *wrath_dice_count)?;
                has_special_system = true;
            }
            Modifier::Godbound(straight_damage) => {
                apply_godbound_damage(result, *straight_damage, has_math_modifiers)?;
                has_special_system = true;
            }
            Modifier::HeroSystem(hero_type) => {
                apply_hero_system_calculation(result, rng, hero_type)?;
            }
            Modifier::Shadowrun(dice_count) => {
                apply_shadowrun_critical_glitch_check(result, *dice_count)?;
                has_special_system = true;
            }
            Modifier::SavageWorlds(_) => {
                // Savage Worlds is handled in the main roll_dice function
                // Don't process it here
            }
            Modifier::D6System(_, _) => {
                // D6 System is handled in the main roll_dice function
            }
            Modifier::MarvelMultiverse(_, _) => {
                *result = handle_marvel_multiverse_roll(dice.clone(), rng)?;
                return Ok(());
            }
            Modifier::CyberpunkRed => {
                apply_cyberpunk_red_mechanics(result, rng)?;
                has_special_system = true;
            }
            Modifier::Witcher => {
                apply_witcher_mechanics(result, rng)?;
                has_special_system = true;
            }
            Modifier::CypherSystem(level) => {
                apply_cypher_system_mechanics(result, *level)?;
                has_special_system = true;
            }
            Modifier::ConanSkill(_) => {
                // Conan skill rolls are handled in the main roll_dice function
                // Don't process them here
            }
            Modifier::ConanCombat(_) => {
                // Conan combat dice are handled in the main roll_dice function
                // Don't process them here
            }
            // Skip mathematical modifiers here - they're handled by target processing or post-target processing
            Modifier::Add(_)
            | Modifier::Subtract(_)
            | Modifier::Multiply(_)
            | Modifier::Divide(_) => {
                // These are handled either before targets or after targets
            }
            _ => {} // Other modifiers handled elsewhere
        }
    }

    // Apply mathematical modifiers that come AFTER target modifiers (to success counts)
    if has_special_system && has_math_modifiers && result.successes.is_some() {
        // Find mathematical modifiers that come after the last target modifier
        if let Some(&last_target_pos) = target_positions.last() {
            let post_target_modifiers: Vec<_> = dice.modifiers[(last_target_pos + 1)..]
                .iter()
                .filter(|m| {
                    matches!(
                        m,
                        Modifier::Add(_)
                            | Modifier::Subtract(_)
                            | Modifier::Multiply(_)
                            | Modifier::Divide(_)
                    )
                })
                .collect();

            if !post_target_modifiers.is_empty() {
                apply_mathematical_modifiers_to_successes_from_slice(
                    result,
                    &post_target_modifiers,
                )?;
            }
        }
    }

    // Handle other special systems (CPR, Witcher) that don't use success counting
    let has_cpr = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::CyberpunkRed));
    let has_witcher = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::Witcher));
    if (has_cpr || has_witcher) && has_math_modifiers {
        apply_mathematical_modifiers_to_cpr_total(result, dice)?;
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
        base_group.dropped_rolls = result.dropped_rolls.clone();
    }
}

// Generic function for counting dice that match a condition
fn count_dice_matching<F>(result: &mut RollResult, condition: F, count_type: &str) -> Result<()>
where
    F: Fn(i32) -> bool,
{
    let count = result
        .kept_rolls
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
        .kept_rolls
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

fn count_wrath_glory_successes(
    result: &mut RollResult,
    difficulty: Option<u32>,
    use_total: bool,
    wrath_dice_count: u32,
) -> Result<()> {
    let mut wrath_dice_values = Vec::new();
    let mut has_complication = false;
    let mut has_critical = false;

    if use_total {
        // For soak/damage/exempt rolls, just use the total of dice values
        result.total = result.kept_rolls.iter().sum();
        result.successes = None; // Don't show successes for total-based rolls

        // Check wrath dice effects (first N dice based on wrath_dice_count)
        for (_i, &die_value) in result
            .kept_rolls
            .iter()
            .enumerate()
            .take(wrath_dice_count as usize)
        {
            wrath_dice_values.push(die_value);
            if die_value == 1 {
                has_complication = true;
            }
            // Note: Glory effects don't apply to soak rolls in W&G
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

        // Add notes for wrath dice effects (only complications for soak rolls)
        if has_complication {
            let complication_count = wrath_dice_values.iter().filter(|&&x| x == 1).count();
            if complication_count == 1 {
                result
                    .notes
                    .push("Wrath die rolled 1 - Complication!".to_string());
            } else {
                result.notes.push(format!(
                    "{complication_count} Wrath dice rolled 1 - Complications!"
                ));
            }
        }
    } else {
        // Standard Wrath & Glory success counting
        let mut total_successes = 0;
        let mut icon_count = 0;
        let mut exalted_icon_count = 0;

        // Process all dice, with first N being wrath dice
        for (i, &roll) in result.kept_rolls.iter().enumerate() {
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

            // Check wrath dice effects (first N dice based on wrath_dice_count)
            if i < wrath_dice_count as usize {
                wrath_dice_values.push(roll);
                if roll == 1 {
                    has_complication = true;
                } else if roll == 6 {
                    has_critical = true;
                }
            }
        }

        // Set Wrath & Glory specific fields
        if !wrath_dice_values.is_empty() {
            result.wng_wrath_die = Some(wrath_dice_values[0]); // Keep for backwards compatibility
            result.wng_wrath_dice = Some(wrath_dice_values.clone()); // Store all wrath dice
        }

        result.wng_icons = Some(icon_count);
        result.wng_exalted_icons = Some(exalted_icon_count);
        result.successes = Some(total_successes);

        // Check difficulty if specified (comparing successes to difficulty)
        if let Some(dn) = difficulty {
            let passed = total_successes >= dn as i32;
            let status = if passed { "PASS" } else { "FAIL" };
            result
                .notes
                .push(format!("Difficulty {dn}: {status} (needed {dn})"));
        }

        // Add notes for wrath dice effects
        add_wrath_die_notes(
            result,
            has_complication,
            has_critical,
            &wrath_dice_values,
            wrath_dice_count,
        );
    }

    Ok(())
}

// Helper function for wrath die notes to reduce duplication
fn add_wrath_die_notes(
    result: &mut RollResult,
    has_complication: bool,
    has_critical: bool,
    wrath_dice_values: &[i32],
    _wrath_dice_count: u32,
) {
    if has_complication {
        let complication_count = wrath_dice_values.iter().filter(|&&x| x == 1).count();
        if complication_count == 1 {
            result
                .notes
                .push("Wrath die rolled 1 - Complication!".to_string());
        } else {
            result.notes.push(format!(
                "{complication_count} Wrath dice rolled 1 - Complications!"
            ));
        }
    }

    if has_critical {
        let critical_count = wrath_dice_values.iter().filter(|&&x| x == 6).count();
        if critical_count == 1 {
            result
                .notes
                .push("Wrath die rolled 6 - Critical/Glory!".to_string());
        } else {
            result.notes.push(format!(
                "{critical_count} Wrath dice rolled 6 - Glory potential!"
            ));
        }
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
                chart_conversions.push(format!("{roll} → {damage}"));
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
            let new_roll = rng.random_range(1..=dice_sides as i32);
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
                "⚔️ **RIGHTEOUS FURY!** {explosion_count} natural 10s - Emperor's wrath unleashed!"
            ));
        }
    } else {
        // Generic exploding dice message for all other systems
        if explosion_count == 1 {
            result.notes.push("1 die exploded".to_string());
        } else {
            result
                .notes
                .push(format!("{explosion_count} dice exploded"));
        }
    }
}

// Better drop dice with proper error handling
fn drop_dice(result: &mut RollResult, count: usize) -> Result<()> {
    let available_dice = result.individual_rolls.len();

    // Handle d0 gracefully
    if count == 0 {
        return Ok(()); // Drop 0 dice is a no-op
    }

    // Don't drop ALL dice when count >= available
    // The test expects some dice to remain
    if count >= available_dice {
        // Drop all but one die (or all if only one die)
        let to_drop = if available_dice > 1 {
            available_dice - 1
        } else {
            available_dice
        };

        let mut rolls = result.individual_rolls.clone();
        rolls.sort();

        // Drop the lowest dice using helper function
        drop_lowest_dice(result, &mut rolls, to_drop);
        return Ok(());
    }

    let mut rolls = result.individual_rolls.clone();
    rolls.sort();
    let rolls_len = rolls.len();

    // Drop lowest dice using helper function
    let count_to_drop = count.min(rolls_len);
    drop_lowest_dice(result, &mut rolls, count_to_drop);

    Ok(())
}

// Helper function to drop lowest dice, reducing duplication
fn drop_lowest_dice(result: &mut RollResult, sorted_rolls: &mut Vec<i32>, count: usize) {
    for _ in 0..count {
        if let Some(pos) = result
            .individual_rolls
            .iter()
            .position(|&x| x == sorted_rolls[0])
        {
            let dropped = result.individual_rolls.remove(pos);
            result.dropped_rolls.push(dropped);
            sorted_rolls.remove(0);
        }
    }
}

// Better keep dice with proper validation
fn keep_dice(result: &mut RollResult, count: usize, keep_low: bool) -> Result<()> {
    if count >= result.individual_rolls.len() {
        return Ok(()); // Keep all dice
    }

    // Validate that count > 0
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

    for i in 0..result.individual_rolls.len() {
        let mut rerolls_for_this_die = 0;
        let max_rerolls_per_die = if indefinite { 100 } else { 1 };

        while result.individual_rolls[i] <= threshold as i32
            && rerolls_for_this_die < max_rerolls_per_die
            && total_rerolls < max_total_rerolls
        {
            result.individual_rolls[i] = rng.random_range(1..=dice_sides as i32);
            rerolls_for_this_die += 1;
            total_rerolls += 1;

            if !indefinite {
                break;
            }
        }
    }

    // Add single summary note if any rerolls happened
    if total_rerolls > 0 {
        if total_rerolls == 1 {
            result.notes.push("1 die rerolled".to_string());
        } else {
            result.notes.push(format!("{total_rerolls} dice rerolled"));
        }
    }

    // Safety check note
    if total_rerolls >= max_total_rerolls {
        result
            .notes
            .push("Maximum rerolls reached (100)".to_string());
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
            let stun_multiplier = rng.random_range(1..=3);
            let stun_damage = body_damage * stun_multiplier;

            result.notes.push(format!(
                "Killing damage: {body_damage} BODY, {stun_damage} STUN (×{stun_multiplier})"
            ));

            // Override the total to show STUN damage (more commonly used)
            result.total = stun_damage;
        }
        HeroSystemType::Hit => {
            // Ensure to-hit notation is always added
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

    for &roll in &result.kept_rolls {
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

    let original_dice_total: i32 = result.kept_rolls.iter().sum();
    let fudge_adjustment = fudge_total - original_dice_total;
    result.total += fudge_adjustment;

    result
        .notes
        .push("Fudge dice: 1=(-), 2=( ), 3=(+)".to_string());

    Ok(())
}

fn apply_mathematical_modifiers_to_successes_from_slice(
    result: &mut RollResult,
    modifiers: &[&Modifier],
) -> Result<()> {
    for modifier in modifiers {
        match modifier {
            Modifier::Add(value) => {
                if let Some(ref mut successes) = result.successes {
                    *successes += value;
                }
            }
            Modifier::Subtract(value) => {
                if let Some(ref mut successes) = result.successes {
                    *successes -= value;
                }
            }
            Modifier::Multiply(value) => {
                if let Some(ref mut successes) = result.successes {
                    *successes *= value;
                }
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                if let Some(ref mut successes) = result.successes {
                    *successes /= value;
                }
            }
            _ => {} // Not a mathematical modifier
        }
    }
    Ok(())
}

fn handle_savage_worlds_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Find the Savage Worlds modifier
    let trait_sides = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::SavageWorlds(sides) = m {
                Some(*sides)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected Savage Worlds modifier"))?;

    // Roll trait die (exploding on max)
    let mut trait_rolls = vec![rng.random_range(1..=trait_sides as i32)];
    let mut trait_explosions = 0;
    while trait_rolls.last().copied().unwrap_or(0) >= trait_sides as i32 && trait_explosions < 100 {
        trait_rolls.push(rng.random_range(1..=trait_sides as i32));
        trait_explosions += 1;
    }
    let trait_total: i32 = trait_rolls.iter().sum();

    // Roll wild die (d6, exploding on 6)
    let mut wild_rolls = vec![rng.random_range(1..=6)];
    let mut wild_explosions = 0;
    while wild_rolls.last().copied().unwrap_or(0) >= 6 && wild_explosions < 100 {
        wild_rolls.push(rng.random_range(1..=6));
        wild_explosions += 1;
    }
    let wild_total: i32 = wild_rolls.iter().sum();

    // Create dice groups for display
    result.dice_groups.push(DiceGroup {
        _description: format!("1d{trait_sides} ie{trait_sides}"),
        rolls: trait_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "trait".to_string(),
    });

    result.dice_groups.push(DiceGroup {
        _description: "1d6 ie6".to_string(),
        rolls: wild_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "wild".to_string(),
    });

    // Add all rolls to individual_rolls for display
    result.individual_rolls.extend(trait_rolls);
    result.individual_rolls.extend(wild_rolls);

    // Keep the highest total (trait vs wild)
    let base_result = if trait_total >= wild_total {
        result.kept_rolls = vec![trait_total];
        trait_total
    } else {
        result.kept_rolls = vec![wild_total];
        wild_total
    };

    result.total = base_result;

    // NOW apply mathematical modifiers to the Savage Worlds result
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => {
                result.total += value;
            }
            Modifier::Subtract(value) => {
                result.total -= value;
            }
            Modifier::Multiply(value) => {
                result.total *= value;
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            Modifier::SavageWorlds(_) => {
                // Already handled above
            }
            _ => {
                // For now, ignore other modifiers in Savage Worlds
                // (we could add support for AddDice, etc. later if needed)
            }
        }
    }

    // Check for Snake Eyes (both dice show natural 1)
    let trait_natural = result.dice_groups[0].rolls.first().copied().unwrap_or(0);
    let wild_natural = result.dice_groups[1].rolls.first().copied().unwrap_or(0);

    if trait_natural == 1 && wild_natural == 1 {
        result
            .notes
            .push("🐍 **SNAKE EYES!** Critical Failure - both dice rolled 1".to_string());
    }

    // Add explanatory notes
    if trait_total > wild_total {
        result.notes.push(format!(
            "Trait die (d{trait_sides}) kept: {trait_total} beats Wild die (d6): {wild_total}"
        ));
    } else if wild_total > trait_total {
        result.notes.push(format!(
            "Wild die (d6) kept: {wild_total} beats Trait die (d{trait_sides}): {trait_total}"
        ));
    } else {
        result.notes.push(format!(
            "Tie: both Trait die (d{trait_sides}) and Wild die (d6) rolled {trait_total}"
        ));
    }

    // Add explosion notes if any occurred
    if trait_explosions > 0 {
        result
            .notes
            .push(format!("Trait die exploded {trait_explosions} times"));
    }
    if wild_explosions > 0 {
        result
            .notes
            .push(format!("Wild die exploded {wild_explosions} times"));
    }

    // Show mathematical modifiers that were applied - commenting this out for now
    // let math_modifier_total: i32 = dice
    //    .modifiers
    //    .iter()
    //    .map(|m| match m {
    //        Modifier::Add(v) => *v,
    //        Modifier::Subtract(v) => -*v,
    //        _ => 0,
    //    })
    //    .sum();

    //if math_modifier_total != 0 {
    //    if math_modifier_total > 0 {
    //        result.notes.push(format!(
    //            "Mathematical modifier: +{math_modifier_total} applied"
    //        ));
    //    } else {
    //        result.notes.push(format!(
    //            "Mathematical modifier: {math_modifier_total} applied"
    //        ));
    //    }
    //}

    result
        .notes
        .push("Savage Worlds: Trait die + Wild die, keep highest".to_string());

    Ok(result)
}

// 5. ADD handle_d6_system_roll function to roller.rs:
fn handle_d6_system_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Find the D6 System modifier
    let (count, pips_str) = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::D6System(count, pips) = m {
                Some((*count, pips.clone()))
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected D6 System modifier"))?;

    // Roll base dice (non-exploding)
    let mut base_rolls = Vec::new();
    for _ in 0..count {
        base_rolls.push(rng.random_range(1..=6));
    }
    let base_total: i32 = base_rolls.iter().sum();

    // Roll wild die (exploding on 6)
    let mut wild_rolls = vec![rng.random_range(1..=6)];
    let mut wild_explosions = 0;
    while wild_rolls.last().copied().unwrap_or(0) >= 6 && wild_explosions < 100 {
        wild_rolls.push(rng.random_range(1..=6));
        wild_explosions += 1;
    }
    let wild_total: i32 = wild_rolls.iter().sum();

    // Create dice groups for display
    result.dice_groups.push(DiceGroup {
        _description: format!("{count}d6"),
        rolls: base_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    });

    result.dice_groups.push(DiceGroup {
        _description: "1d6 ie6".to_string(),
        rolls: wild_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "add".to_string(),
    });

    // Add all rolls to individual_rolls and kept_rolls
    result.individual_rolls.extend(base_rolls);
    result.individual_rolls.extend(wild_rolls);
    result.kept_rolls = result.individual_rolls.clone();

    // Calculate total
    let dice_total = base_total + wild_total;

    // Apply pips modifier if any
    let pips_modifier = if !pips_str.is_empty() {
        pips_str.parse::<i32>().unwrap_or(0)
    } else {
        0
    };

    result.total = dice_total + pips_modifier;

    // Apply other mathematical modifiers
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => {
                result.total += value;
            }
            Modifier::Subtract(value) => {
                result.total -= value;
            }
            Modifier::Multiply(value) => {
                result.total *= value;
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            Modifier::D6System(_, _) => {
                // Already handled above
            }
            _ => {
                // Ignore other modifiers for D6 System
            }
        }
    }

    // Add notes
    if wild_explosions > 0 {
        result
            .notes
            .push(format!("Wild die exploded {wild_explosions} times"));
    }

    if pips_modifier != 0 {
        if pips_modifier > 0 {
            result
                .notes
                .push(format!("Pips modifier: +{pips_modifier}"));
        } else {
            result.notes.push(format!("Pips modifier: {pips_modifier}"));
        }
    }

    result
        .notes
        .push(format!("D6 System: {count}d6 + 1d6 exploding wild die"));

    Ok(result)
}

fn apply_shadowrun_critical_glitch_check(result: &mut RollResult, dice_count: u32) -> Result<()> {
    // Count the number of 1s in the kept rolls
    let ones_count = result.kept_rolls.iter().filter(|&&roll| roll == 1).count();

    // Critical glitch occurs when more than half the dice pool shows 1s
    let half_dice_pool = (dice_count as f64 / 2.0).floor() as usize;

    if ones_count > half_dice_pool {
        // Critical glitch detected
        if let Some(successes) = result.successes {
            if successes == 0 {
                result.notes.push("💀 **CRITICAL GLITCH!** More than half the dice pool rolled 1s with no successes - catastrophic failure!".to_string());
            } else {
                result.notes.push("⚠️ **GLITCH!** More than half the dice pool rolled 1s but successes were achieved - complications arise!".to_string());
            }
        } else {
            result.notes.push("💀 **CRITICAL GLITCH!** More than half the dice pool rolled 1s - catastrophic failure!".to_string());
        }
    }

    Ok(())
}

fn handle_marvel_multiverse_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Find the Marvel Multiverse modifier
    let (edges, troubles) = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::MarvelMultiverse(e, t) = m {
                Some((*e, *t))
            } else {
                None
            }
        })
        .unwrap_or((0, 0));

    // Roll initial 3d6 (treating middle die as Marvel die)
    let regular_die_1 = rng.random_range(1..=6);
    let mut marvel_die = rng.random_range(1..=6);
    let regular_die_2 = rng.random_range(1..=6);

    // Track if Marvel die is showing Marvel logo (1)
    let is_marvel_fantastic = marvel_die == 1;

    // Create initial dice group showing initial roll with Marvel symbol
    let initial_rolls = vec![regular_die_1, marvel_die, regular_die_2];
    let initial_display_rolls: Vec<i32> = initial_rolls
        .iter()
        .enumerate()
        .map(|(i, &roll)| {
            if i == 1 && roll == 1 {
                -1 // Show Marvel symbol in initial roll
            } else {
                roll
            }
        })
        .collect();

    let base_group = DiceGroup {
        _description: "3d6 Marvel Multiverse".to_string(),
        rolls: initial_display_rolls,
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    };
    result.dice_groups.push(base_group);

    // Store individual rolls for processing
    result.individual_rolls = initial_rolls.clone();

    // Handle Fantastic result (Marvel die showing 1)
    if is_marvel_fantastic {
        marvel_die = 6; // Marvel die becomes 6 when Fantastic
        result
            .notes
            .push("Fantastic! Marvel die rolled Marvel symbol, counts as 6".to_string());
    }

    // Process edges and troubles with consolidated notes
    let mut final_rolls = [regular_die_1, marvel_die, regular_die_2];

    // Add edge/trouble count notes that tests expect - but only if there are edges/troubles
    if edges > 0 {
        result.notes.push(format!(
            "{} edge{}",
            edges,
            if edges == 1 { "" } else { "s" }
        ));
    }
    if troubles > 0 {
        result.notes.push(format!(
            "{} trouble{}",
            troubles,
            if troubles == 1 { "" } else { "s" }
        ));
    }

    // Process edges with consolidated reporting
    if edges > 0 {
        let mut edge_details = Vec::new();

        for _ in 0..edges {
            // Find lowest die value and its index
            let (min_value, min_index) = final_rolls
                .iter()
                .enumerate()
                .min_by_key(|&(_, value)| value)
                .map(|(index, &value)| (value, index))
                .unwrap();

            // Reroll the lowest die
            let new_roll = rng.random_range(1..=6);

            // Keep the higher of the two
            if new_roll > min_value {
                final_rolls[min_index] = new_roll;
                edge_details.push(format!("{min_value} → {new_roll}"));
            } else {
                edge_details.push(format!("{min_value} → {new_roll} (kept {min_value})"));
            }
        }

        // Consolidate all edge rerolls into a single note
        if edges == 1 {
            result
                .notes
                .push(format!("Edge 1: Rerolled {}", edge_details[0]));
        } else {
            result.notes.push(format!(
                "Edge rerolls: {}",
                edge_details
                    .iter()
                    .enumerate()
                    .map(|(i, detail)| format!("#{}: {}", i + 1, detail))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Process troubles with consolidated reporting
    if troubles > 0 {
        let mut trouble_details = Vec::new();

        for _ in 0..troubles {
            // Find highest die value and its index
            let (max_value, max_index) = final_rolls
                .iter()
                .enumerate()
                .max_by_key(|&(_, value)| value)
                .map(|(index, &value)| (value, index))
                .unwrap();

            // Reroll the highest die
            let new_roll = rng.random_range(1..=6);

            // Keep the lower of the two
            if new_roll < max_value {
                final_rolls[max_index] = new_roll;
                trouble_details.push(format!("{max_value} → {new_roll}"));
            } else {
                trouble_details.push(format!("{max_value} → {new_roll} (kept {max_value})"));
            }
        }

        // Consolidate all trouble rerolls into a single note
        if troubles == 1 {
            result
                .notes
                .push(format!("Trouble 1: Rerolled {}", trouble_details[0]));
        } else {
            result.notes.push(format!(
                "Trouble rerolls: {}",
                trouble_details
                    .iter()
                    .enumerate()
                    .map(|(i, detail)| format!("#{}: {}", i + 1, detail))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Create result dice group with proper Marvel symbol display
    let final_display_rolls: Vec<i32> = final_rolls
        .iter()
        .enumerate()
        .map(|(i, &roll)| {
            if i == 1 && is_marvel_fantastic {
                -1 // Use -1 to represent Marvel symbol for display
            } else {
                roll
            }
        })
        .collect();

    let result_group = DiceGroup {
        _description: "3d6 Marvel Multiverse result".to_string(),
        rolls: final_display_rolls,
        dropped_rolls: Vec::new(),
        modifier_type: "result".to_string(),
    };
    result.dice_groups.push(result_group);

    // Calculate total
    result.total = final_rolls.iter().sum::<i32>();

    // Apply mathematical modifiers - INCLUDING DIVISION BY ZERO CHECK
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => result.total += value,
            Modifier::Subtract(value) => result.total -= value,
            Modifier::Multiply(value) => result.total *= value,
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            Modifier::MarvelMultiverse(_, _) => {
                // Already handled above
            }
            _ => {
                // Handle other modifier types as needed
            }
        }
    }

    Ok(result)
}

fn apply_cyberpunk_red_mechanics(result: &mut RollResult, rng: &mut impl Rng) -> Result<()> {
    // CPR only works with exactly 1d10
    if result.individual_rolls.len() != 1 {
        return Err(anyhow!("Cyberpunk Red mechanics only work with 1d10"));
    }

    let original_roll = result.individual_rolls[0];
    let mut total_result = original_roll;
    let mut additional_rolls = Vec::new();
    let mut explosion_notes = Vec::new();

    match original_roll {
        10 => {
            // Critical Success: Roll another d10 and add it
            let additional_roll = rng.random_range(1..=10);
            additional_rolls.push(additional_roll);
            total_result += additional_roll;
            explosion_notes.push(format!(
                "💥 **CRITICAL SUCCESS!** Rolled 10, added {additional_roll}"
            ));
        }
        1 => {
            // Critical Failure: Roll another d10 and subtract it
            let additional_roll = rng.random_range(1..=10);
            additional_rolls.push(-additional_roll); // Store as negative for display
            total_result -= additional_roll;
            explosion_notes.push(format!(
                "💀 **CRITICAL FAILURE!** Rolled 1, subtracted {additional_roll}"
            ));
        }
        _ => {
            // Normal roll, no explosion
        }
    }

    // Update the result
    if !additional_rolls.is_empty() {
        // Create dice groups to show the explosion
        let base_group = DiceGroup {
            _description: "1d10".to_string(),
            rolls: vec![original_roll],
            dropped_rolls: Vec::new(),
            modifier_type: "base".to_string(),
        };

        let explosion_group = DiceGroup {
            _description: if original_roll == 10 {
                "Critical Success"
            } else {
                "Critical Failure"
            }
            .to_string(),
            rolls: additional_rolls.clone(),
            dropped_rolls: Vec::new(),
            modifier_type: if original_roll == 10 {
                "add"
            } else {
                "subtract"
            }
            .to_string(),
        };

        result.dice_groups = vec![base_group, explosion_group];

        // Add the actual additional roll to individual_rolls for display
        result.individual_rolls.push(additional_rolls[0].abs());
    }

    // Update totals and kept rolls
    result.total = total_result;
    result.kept_rolls = vec![total_result];

    // Add explosion notes only (no system note)
    result.notes.extend(explosion_notes);

    Ok(())
}

fn apply_mathematical_modifiers_to_cpr_total(
    result: &mut RollResult,
    dice: &DiceRoll,
) -> Result<()> {
    let mut modifier_total = 0;

    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => {
                modifier_total += value;
            }
            Modifier::Subtract(value) => {
                modifier_total -= value;
            }
            Modifier::Multiply(value) => {
                result.total *= value;
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            _ => {} // Skip non-mathematical modifiers
        }
    }

    // Apply accumulated add/subtract modifiers
    if modifier_total != 0 {
        result.total += modifier_total;
    }

    Ok(())
}

fn apply_witcher_mechanics(result: &mut RollResult, rng: &mut impl Rng) -> Result<()> {
    // Witcher only works with exactly 1d10
    if result.individual_rolls.len() != 1 {
        return Err(anyhow!("Witcher mechanics only work with 1d10"));
    }

    let original_roll = result.individual_rolls[0];
    let mut total_result = original_roll;
    let mut additional_rolls = Vec::new();
    let mut explosion_notes = Vec::new();
    let mut explosion_count = 0;
    const MAX_EXPLOSIONS: usize = 100;

    // Handle indefinite explosions - key difference from Cyberpunk Red
    let mut current_roll = original_roll;

    loop {
        if explosion_count >= MAX_EXPLOSIONS {
            explosion_notes.push("Maximum explosions reached (100)".to_string());
            break;
        }

        match current_roll {
            10 => {
                // Critical Success: Roll another d10 and add it
                let additional_roll = rng.random_range(1..=10);
                additional_rolls.push(additional_roll);
                total_result += additional_roll;
                explosion_count += 1;

                if explosion_count == 1 {
                    explosion_notes.push(format!(
                        "⚔️ **CRITICAL SUCCESS!** Rolled 10, added {additional_roll}"
                    ));
                } else {
                    explosion_notes.push(format!(
                        "🔥 **EXPLOSION CONTINUES!** Added {additional_roll}"
                    ));
                }

                current_roll = additional_roll;
                // Continue loop if we rolled another 10 (indefinite explosion)
                if current_roll != 10 {
                    break;
                }
            }
            1 => {
                // Critical Failure: Roll another d10 and subtract it
                let additional_roll = rng.random_range(1..=10);
                additional_rolls.push(-additional_roll); // Store as negative for display
                total_result -= additional_roll;
                explosion_count += 1;

                if explosion_count == 1 {
                    explosion_notes.push(format!(
                        "💀 **CRITICAL FAILURE!** Rolled 1, subtracted {additional_roll}"
                    ));
                } else {
                    explosion_notes.push(format!(
                        "💥 **FAILURE CONTINUES!** Subtracted {additional_roll}"
                    ));
                }

                current_roll = additional_roll;
                // Continue loop if we rolled another 1 (indefinite explosion)
                if current_roll != 1 {
                    break;
                }
            }
            _ => {
                // Normal roll, no explosion
                break;
            }
        }
    }

    // Update the result if we had explosions
    if !additional_rolls.is_empty() {
        // Create dice groups to show the explosion
        let base_group = DiceGroup {
            _description: "1d10".to_string(),
            rolls: vec![original_roll],
            dropped_rolls: Vec::new(),
            modifier_type: "base".to_string(),
        };

        let explosion_group = DiceGroup {
            _description: if original_roll == 10 {
                "Critical Success"
            } else {
                "Critical Failure"
            }
            .to_string(),
            rolls: additional_rolls.clone(),
            dropped_rolls: Vec::new(),
            modifier_type: if original_roll == 10 {
                "add"
            } else {
                "subtract"
            }
            .to_string(),
        };

        result.dice_groups = vec![base_group, explosion_group];

        // Add the actual additional rolls to individual_rolls for display
        for &roll in &additional_rolls {
            result.individual_rolls.push(roll.abs());
        }
    }

    // Update totals and kept rolls
    result.total = total_result;
    result.kept_rolls = vec![total_result];

    // Add explosion notes
    result.notes.extend(explosion_notes);

    Ok(())
}

fn apply_cypher_system_mechanics(result: &mut RollResult, level: u32) -> Result<()> {
    if result.individual_rolls.is_empty() {
        return Err(anyhow!("No dice rolled for Cypher System"));
    }

    let roll = result.individual_rolls[0];
    let target_number = level * 3;
    let success = roll >= target_number as i32;

    // Clear any existing success/failure counts - Cypher is binary success/fail
    result.successes = None;
    result.failures = None;
    result.botches = None;

    // Add success/failure note
    if success {
        result.notes.push(format!(
            "**SUCCESS** (rolled {roll} vs target {target_number})"
        ));
    } else {
        result.notes.push(format!(
            "**FAILURE** (rolled {roll} vs target {target_number})"
        ));
    }

    // Add special result notes
    match roll {
        1 => {
            result
                .notes
                .push("**GM INTRUSION** (Natural 1)".to_string());
        }
        17..=19 => {
            result.notes.push("**MINOR EFFECT** (17-19)".to_string());
        }
        20 => {
            result
                .notes
                .push("**MAJOR EFFECT** (Natural 20)".to_string());
        }
        _ => {}
    }

    result
        .notes
        .push(format!("Cypher System - Level {level} Task"));

    Ok(())
}

pub fn handle_brave_new_world_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    let pool_size = dice.count;

    // Verify we have the BNW modifier
    dice.modifiers
        .iter()
        .find(|m| matches!(m, Modifier::BraveNewWorld(_)))
        .ok_or_else(|| anyhow!("Expected Brave New World modifier"))?;

    // Roll the initial dice pool
    let mut all_results = Vec::new();
    for _ in 0..pool_size {
        all_results.push(rng.random_range(1..=6));
    }

    // Handle exploding 6s - each 6 creates a new result option
    let mut explosion_count = 0;
    let mut i = 0;
    while i < all_results.len() && explosion_count < 100 {
        if all_results[i] == 6 {
            let explosion = rng.random_range(1..=6);
            // BNW explosions create separate results, not additions
            all_results.push(all_results[i] + explosion);
            explosion_count += 1;
        }
        i += 1;
    }

    // Check for disaster (majority of 1s in original pool)
    let bnw_ones_count = all_results[..pool_size as usize]
        .iter()
        .filter(|&&roll| roll == 1)
        .count();
    let is_disaster = pool_size >= 4 && bnw_ones_count > (pool_size as usize / 2);

    // Take the highest result (BNW uses highest, not sum)
    let highest_result = *all_results.iter().max().unwrap_or(&1);

    // Store all rolls for display
    result.individual_rolls = all_results.clone();
    result.kept_rolls = vec![highest_result];

    // Set the total to the highest result
    result.total = highest_result;

    // Create dice group for display
    result.dice_groups.push(DiceGroup {
        _description: format!("{pool_size}d6 bnw"),
        rolls: all_results,
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    });

    // Add notes about the system and special results
    if is_disaster {
        result
            .notes
            .push("Disaster! Majority of dice rolled 1s - automatic failure".to_string());
        result.total = 0; // Disasters always fail regardless of other dice
    }

    if explosion_count > 0 {
        result
            .notes
            .push(format!("{explosion_count} dice exploded on 6s"));
    }

    result.notes.push(format!(
        "Brave New World: {}-die pool, highest result: {}",
        pool_size,
        if is_disaster { 0 } else { highest_result }
    ));

    // Apply any mathematical modifiers after the core BNW mechanics
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => {
                result.total += value;
            }
            Modifier::Subtract(value) => {
                result.total -= value;
            }
            Modifier::Multiply(value) => {
                result.total *= value;
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            Modifier::BraveNewWorld(_) => {
                // Already handled above
            }
            _ => {
                // Ignore other modifiers for BNW
            }
        }
    }

    Ok(result)
}

fn handle_conan_skill_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Find the ConanSkill modifier to get dice count
    let dice_count = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::ConanSkill(count) = m {
                Some(*count)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected ConanSkill modifier"))?;

    // 1. Roll the base skill dice (d20s)
    for _ in 0..dice_count {
        result.individual_rolls.push(rng.random_range(1..=20));
    }

    // Count successes for skill dice (simple approach: count = successes)
    let skill_successes = result.individual_rolls.len() as i32;
    result.successes = Some(skill_successes);
    result.total = skill_successes;
    result.kept_rolls = result.individual_rolls.clone();

    // Create dice group for skill dice
    result.dice_groups.push(DiceGroup {
        _description: format!("{dice_count}d20"),
        rolls: result.individual_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    });

    // 2. Process AddDice modifiers (this is where the 5d6 comes from)
    let mut combat_dice_total = 0;
    let mut has_combat_dice = false;
    let mut combat_specials = 0;

    for modifier in &dice.modifiers {
        if let Modifier::AddDice(additional_dice) = modifier {
            // Roll the additional dice
            let additional_result = roll_dice(additional_dice.clone())?;

            // Check if these are d6 dice that should use Conan combat interpretation
            if additional_dice.sides == 6 {
                // Apply Conan combat dice interpretation to the d6 results
                let combat_damage =
                    apply_conan_combat_interpretation(&additional_result.individual_rolls);
                combat_dice_total += combat_damage;
                has_combat_dice = true;

                // Count special effects (5s and 6s) for notes
                for &roll in &additional_result.individual_rolls {
                    if roll == 5 || roll == 6 {
                        combat_specials += 1;
                    }
                }

                // Add combat dice to display
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                // Create dice group for combat dice
                result.dice_groups.push(DiceGroup {
                    _description: format!("{}d6", additional_dice.count),
                    rolls: additional_result.individual_rolls.clone(),
                    dropped_rolls: Vec::new(),
                    modifier_type: "add".to_string(),
                });
            } else {
                // Regular additional dice (not combat)
                result.total += additional_result.total;
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result
                    .kept_rolls
                    .extend(additional_result.kept_rolls.clone());

                result.dice_groups.push(DiceGroup {
                    _description: format!("{}d{}", additional_dice.count, additional_dice.sides),
                    rolls: additional_result.individual_rolls.clone(),
                    dropped_rolls: Vec::new(),
                    modifier_type: "add".to_string(),
                });
            }
        }
    }

    // 3. Add combat damage to total
    if has_combat_dice {
        result.total += combat_dice_total;
        let current_successes = result.successes.unwrap_or(0);
        result.successes = Some(current_successes + combat_dice_total);
    }

    // 4. Apply regular mathematical modifiers (if any)
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => {
                result.total += value;
            }
            Modifier::Subtract(value) => {
                result.total -= value;
            }
            Modifier::Multiply(value) => {
                result.total *= value;
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            _ => {} // Skip modifiers already handled
        }
    }

    // 5. Add notes for combat dice
    if has_combat_dice {
        // Add special effects note if applicable
        if combat_specials > 0 {
            result
                .notes
                .push(format!("{combat_specials} special effects"));
        }

        // Add the interpretation rule note
        result
            .notes
            .push("1=1, 2=2, 3-4=0, 5-6=1+special".to_string());
    }

    Ok(result)
}

// Helper function to apply Conan combat dice interpretation
fn apply_conan_combat_interpretation(rolls: &[i32]) -> i32 {
    let mut damage = 0;

    for &roll in rolls {
        match roll {
            1 => damage += 1,
            2 => damage += 2,
            3 | 4 => { /* no damage */ }
            5 | 6 => {
                damage += 1;
                // Note: 5-6 also grant special effects in actual play
            }
            _ => {}
        }
    }

    damage
}

fn handle_conan_combat_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Find the ConanCombat modifier to get dice count
    let dice_count = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::ConanCombat(count) = m {
                Some(*count)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected ConanCombat modifier"))?;

    // Roll the combat dice (d6s)
    for _ in 0..dice_count {
        result.individual_rolls.push(rng.random_range(1..=6));
    }

    // Apply Conan combat dice interpretation
    let mut successes = 0;
    let mut specials = 0;

    for &roll in &result.individual_rolls {
        match roll {
            1 => successes += 1,
            2 => successes += 2,
            3 | 4 => { /* no effect */ }
            5 | 6 => {
                successes += 1;
                specials += 1;
            }
            _ => {}
        }
    }

    result.successes = Some(successes);
    result.total = successes;
    result.kept_rolls = result.individual_rolls.clone();

    // Create dice group for display
    result.dice_groups.push(DiceGroup {
        _description: format!("{dice_count}d6"),
        rolls: result.individual_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    });

    // Apply mathematical modifiers to the final total
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => {
                result.total += value;
            }
            Modifier::Subtract(value) => {
                result.total -= value;
            }
            Modifier::Multiply(value) => {
                result.total *= value;
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            _ => {}
        }
    }

    if specials > 0 {
        result.notes.push(format!("{specials} special effects"));
    }
    result
        .notes
        .push("1=1, 2=2, 3-4=0, 5-6=1+special".to_string());

    Ok(result)
}

fn handle_silhouette_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    let dice_count = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::Silhouette(count) = m {
                Some(*count)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected Silhouette modifier"))?;

    // Initialize complete RollResult structure
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
        wng_wrath_die: None,
        wng_icons: None,
        wng_exalted_icons: None,
        wng_wrath_dice: None,
        suppress_comment: false,
    };

    // Roll the dice pool
    for _ in 0..dice_count {
        result.individual_rolls.push(rng.random_range(1..=6));
    }

    // Find highest die
    let highest_die = *result.individual_rolls.iter().max().unwrap_or(&1);

    // Count extra 6s and add to result
    let sixes_count = result.individual_rolls.iter().filter(|&&x| x == 6).count();
    let extra_sixes = if sixes_count > 0 { sixes_count - 1 } else { 0 };
    let silhouette_result = highest_die + extra_sixes as i32;

    // Set kept rolls and total
    result.kept_rolls = vec![silhouette_result];
    result.total = silhouette_result;

    // Create dice group for display
    result.dice_groups.push(DiceGroup {
        _description: format!("{dice_count}d6 Silhouette"),
        rolls: result.individual_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    });

    // Add explanatory notes
    if extra_sixes > 0 {
        result
            .notes
            .push(format!("{sixes_count} extra 6s add +{extra_sixes}"));
    }

    // Apply mathematical modifiers to final result
    apply_mathematical_modifiers_to_silhouette(&mut result, &dice)?;

    Ok(result)
}

fn apply_mathematical_modifiers_to_silhouette(
    result: &mut RollResult,
    dice: &DiceRoll,
) -> Result<()> {
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Add(value) => result.total += value,
            Modifier::Subtract(value) => result.total -= value,
            Modifier::Multiply(value) => result.total *= value,
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                result.total /= value;
            }
            _ => {} // Skip non-mathematical modifiers
        }
    }
    Ok(())
}

fn reroll_dice_greater(
    result: &mut RollResult,
    rng: &mut impl Rng,
    threshold: u32,
    dice_sides: u32,
    indefinite: bool,
) -> Result<()> {
    let mut total_rerolls = 0;
    let max_total_rerolls = 100;

    for i in 0..result.individual_rolls.len() {
        let mut rerolls_for_this_die = 0;
        let max_rerolls_per_die = if indefinite { 100 } else { 1 };

        // MAIN DIFFERENCE: Changed condition from <= to >=
        while result.individual_rolls[i] >= threshold as i32
            && rerolls_for_this_die < max_rerolls_per_die
            && total_rerolls < max_total_rerolls
        {
            result.individual_rolls[i] = rng.random_range(1..=dice_sides as i32);
            rerolls_for_this_die += 1;
            total_rerolls += 1;

            if !indefinite {
                break;
            }
        }
    }

    // Add single summary note if any rerolls happened
    if total_rerolls > 0 {
        if total_rerolls == 1 {
            result.notes.push("1 die rerolled".to_string());
        } else {
            result.notes.push(format!("{total_rerolls} dice rerolled"));
        }
    }

    // Safety check note
    if total_rerolls >= max_total_rerolls {
        result
            .notes
            .push("Maximum rerolls reached (100)".to_string());
    }

    Ok(())
}

fn keep_middle_dice(result: &mut RollResult, count: usize) -> Result<()> {
    let available_dice = result.individual_rolls.len();

    // If we want to keep all or more dice than available, keep all
    if count >= available_dice {
        return Ok(());
    }

    // Create indexed rolls for tracking original positions
    let mut indexed_rolls: Vec<(usize, i32)> = result
        .individual_rolls
        .iter()
        .enumerate()
        .map(|(i, &roll)| (i, roll))
        .collect();

    // Sort by value to identify middle dice
    indexed_rolls.sort_by_key(|&(_, roll)| roll);

    // Calculate how many dice to drop from each end
    let total_to_drop = available_dice - count;
    let drop_from_low = total_to_drop / 2;
    let drop_from_high = total_to_drop - drop_from_low;

    // Determine which dice to keep (middle indices after sorting)
    let keep_start = drop_from_low;
    let keep_end = available_dice - drop_from_high;

    let kept_indices: Vec<usize> = indexed_rolls[keep_start..keep_end]
        .iter()
        .map(|&(i, _)| i)
        .collect();

    // Separate kept and dropped dice
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

fn find_target_modifier_positions(modifiers: &[Modifier]) -> Vec<usize> {
    modifiers
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            if matches!(
                m,
                Modifier::Target(_)
                    | Modifier::TargetLower(_)
                    | Modifier::TargetWithDoubleSuccess(_, _)
                    | Modifier::TargetLowerWithDoubleSuccess(_, _)
                    | Modifier::Failure(_)
                    | Modifier::Botch(_)
            ) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn apply_pre_target_mathematical_modifiers(
    result: &mut RollResult,
    modifiers: &[Modifier],
) -> Result<()> {
    // Apply each mathematical modifier to individual dice
    for modifier in modifiers {
        match modifier {
            Modifier::Add(value) => {
                for die_value in &mut result.kept_rolls {
                    *die_value += value;
                }
                for die_value in &mut result.individual_rolls {
                    *die_value += value;
                }
                // Update dice groups for display
                for group in &mut result.dice_groups {
                    for die_value in &mut group.rolls {
                        *die_value += value;
                    }
                }
            }
            Modifier::Subtract(value) => {
                for die_value in &mut result.kept_rolls {
                    *die_value -= value;
                }
                for die_value in &mut result.individual_rolls {
                    *die_value -= value;
                }
                for group in &mut result.dice_groups {
                    for die_value in &mut group.rolls {
                        *die_value -= value;
                    }
                }
            }
            Modifier::Multiply(value) => {
                for die_value in &mut result.kept_rolls {
                    *die_value *= value;
                }
                for die_value in &mut result.individual_rolls {
                    *die_value *= value;
                }
                for group in &mut result.dice_groups {
                    for die_value in &mut group.rolls {
                        *die_value *= value;
                    }
                }
            }
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                for die_value in &mut result.kept_rolls {
                    *die_value /= value;
                }
                for die_value in &mut result.individual_rolls {
                    *die_value /= value;
                }
                for group in &mut result.dice_groups {
                    for die_value in &mut group.rolls {
                        *die_value /= value;
                    }
                }
            }
            _ => {} // Not a mathematical modifier
        }
    }

    // Update the total to reflect the modified dice values
    result.total = result.kept_rolls.iter().sum();

    Ok(())
}

fn apply_cancel_modifier(result: &mut RollResult) -> Result<()> {
    // Cancel modifier only works if we have failures tracked
    if result.failures.is_none() {
        result
            .notes
            .push("Cancel modifier requires failure counting (f#) to work".to_string());
        return Ok(());
    }

    // Count 10s and 1s in the kept rolls
    let tens_count = result.kept_rolls.iter().filter(|&&roll| roll == 10).count() as i32;
    let ones_count = result.kept_rolls.iter().filter(|&&roll| roll == 1).count() as i32;

    let current_failures = result.failures.unwrap_or(0);

    // Cancel 1s with 10s on a 1:1 basis
    let cancellations = std::cmp::min(tens_count, ones_count);

    // Only add notes when cancellations actually occur
    if cancellations > 0 {
        // Reduce failures by the number of cancellations
        let new_failures = current_failures - cancellations;
        result.failures = Some(std::cmp::max(0, new_failures));

        result.notes.push(format!(
            "**CANCELLED**: {cancellations} failures (1s) cancelled by {cancellations} successes (10s)",
        ));
    }

    Ok(())
}

fn count_dice_with_double_success(
    result: &mut RollResult,
    target: u32,
    double_success_value: u32,
) -> Result<()> {
    let success_count = result
        .kept_rolls
        .iter()
        .map(|&roll| {
            if roll >= double_success_value as i32 {
                2 // Double success
            } else if roll >= target as i32 {
                1 // Single success
            } else {
                0 // No success
            }
        })
        .sum::<i32>();

    // Add to existing success count (preserves existing multi-target behavior)
    result.successes = Some(result.successes.unwrap_or(0) + success_count);

    // Add explanatory note
    if double_success_value == target {
        result
            .notes
            .push(format!("{double_success_value}+ = 2 successes"));
    } else {
        result.notes.push(format!(
            "{target}+ = 1 success, {double_success_value}+ = 2 successes"
        ));
    }

    Ok(())
}

fn count_dice_with_target_lower_double_success(
    result: &mut RollResult,
    target: u32,
    double_success_value: u32,
) -> Result<()> {
    let success_count = result
        .kept_rolls
        .iter()
        .map(|&roll| {
            if roll <= double_success_value as i32 {
                2 // Double success (roll is ≤ double_success_value)
            } else if roll <= target as i32 {
                1 // Single success (roll is ≤ target but > double_success_value)
            } else {
                0 // No success (roll is > target)
            }
        })
        .sum::<i32>();

    // Add to existing success count
    result.successes = Some(result.successes.unwrap_or(0) + success_count);

    // Add explanatory note
    if double_success_value == target {
        result
            .notes
            .push(format!("≤{} = 2 successes", double_success_value));
    } else {
        result.notes.push(format!(
            "≤{} = 2 successes, ≤{} = 1 success",
            double_success_value, target
        ));
    }

    Ok(())
}
