use super::{DiceGroup, DiceRoll, Modifier, RollResult};
use anyhow::{anyhow, Result};
use rand::Rng;

pub fn roll_dice(dice: DiceRoll) -> Result<RollResult> {
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
        original_expression: dice.original_expression.clone(), // Pass through the original expression
        simple: dice.simple,                                   // Transfer simple flag from DiceRoll
        no_results: dice.no_results, // Transfer no_results flag from DiceRoll
        private: dice.private,       // Transfer private flag from DiceRoll
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

    // Apply dice-modifying modifiers first (exploding, rerolls, etc.)
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Explode(threshold) => {
                explode_dice(&mut result, &mut rng, *threshold, dice.sides, false)?;
                // Update the base group with exploded dice
                if let Some(base_group) = result.dice_groups.get_mut(0) {
                    base_group.rolls = result.individual_rolls.clone();
                }
            }
            Modifier::ExplodeIndefinite(threshold) => {
                explode_dice(&mut result, &mut rng, *threshold, dice.sides, true)?;
                if let Some(base_group) = result.dice_groups.get_mut(0) {
                    base_group.rolls = result.individual_rolls.clone();
                }
            }
            Modifier::Reroll(threshold) => {
                reroll_dice(&mut result, &mut rng, *threshold, dice.sides, false)?;
                if let Some(base_group) = result.dice_groups.get_mut(0) {
                    base_group.rolls = result.individual_rolls.clone();
                }
            }
            Modifier::RerollIndefinite(threshold) => {
                reroll_dice(&mut result, &mut rng, *threshold, dice.sides, true)?;
                if let Some(base_group) = result.dice_groups.get_mut(0) {
                    base_group.rolls = result.individual_rolls.clone();
                }
            }
            _ => {} // Handle other modifiers later
        }
    }

    // Apply keep/drop modifiers
    for modifier in &dice.modifiers {
        match modifier {
            Modifier::Drop(count) => {
                drop_dice(&mut result, *count as usize)?;
            }
            Modifier::KeepHigh(count) => {
                keep_dice(&mut result, *count as usize, false)?;
            }
            Modifier::KeepLow(count) => {
                keep_dice(&mut result, *count as usize, true)?;
            }
            _ => {} // Skip modifiers already handled
        }
    }

    // Calculate total from remaining dice
    if result.kept_rolls.is_empty() {
        result.kept_rolls = result.individual_rolls.clone();
    }
    result.total = result.kept_rolls.iter().sum();

    // Apply mathematical and other modifiers
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
            Modifier::Target(value) => {
                count_successes(&mut result, *value)?;
            }
            Modifier::Failure(value) => {
                count_failures(&mut result, *value)?;
            }
            Modifier::Botch(threshold) => {
                count_botches(&mut result, threshold.unwrap_or(1))?;
            }
            Modifier::WrathGlory(difficulty, use_total) => {
                count_wrath_glory_successes(&mut result, *difficulty, *use_total)?;
            }
            Modifier::AddDice(dice_to_add) => {
                let additional_result = roll_dice(dice_to_add.clone())?;
                result
                    .individual_rolls
                    .extend(additional_result.individual_rolls.clone());
                result.total += additional_result.total;

                // Add a new dice group for the additional dice
                let add_group = DiceGroup {
                    _description: format!("{}d{}", dice_to_add.count, dice_to_add.sides),
                    rolls: additional_result.individual_rolls,
                    modifier_type: "add".to_string(),
                };
                result.dice_groups.push(add_group);
            }
            Modifier::SubtractDice(dice_to_subtract) => {
                let subtract_result = roll_dice(dice_to_subtract.clone())?;
                result
                    .individual_rolls
                    .extend(subtract_result.individual_rolls.clone());
                result.total -= subtract_result.total;

                // Add a new dice group for the subtracted dice
                let subtract_group = DiceGroup {
                    _description: format!("{}d{}", dice_to_subtract.count, dice_to_subtract.sides),
                    rolls: subtract_result.individual_rolls,
                    modifier_type: "subtract".to_string(),
                };
                result.dice_groups.push(subtract_group);
            }
            _ => {} // Skip modifiers already handled above
        }
    }

    // If target/success system was used, don't use the dice total
    if result.successes.is_some() {
        result.total = 0; // Reset total for success-based systems
    }

    // Sort rolls unless unsorted flag is set
    if !dice.unsorted && !result.kept_rolls.is_empty() {
        result.kept_rolls.sort_by(|a, b| b.cmp(a)); // Sort descending by default
    }

    Ok(result)
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
            // Note: We don't show critical/glory effects for soak/damage rolls
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

        // In Wrath & Glory, one die is designated as the "wrath die"
        // For simplicity, we'll treat the first die as the wrath die
        for (i, &roll) in result.individual_rolls.iter().enumerate() {
            let successes = match roll {
                1..=3 => 0, // No successes
                4..=5 => 1, // Icons (1 success)
                6 => 2,     // Exalted Icons (2 successes)
                _ => 0,     // Shouldn't happen with normal dice
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

        if has_complication || has_critical {
            result.notes.push(format!("Wrath die: {}", wrath_die_value));
        }
    }

    Ok(())
}

fn explode_dice(
    result: &mut RollResult,
    rng: &mut impl Rng,
    threshold: Option<u32>,
    dice_sides: u32,
    indefinite: bool,
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
        // Check if this look like Dark Heresy (d10 indefinite exploding on 10)
        if dice_sides == 10 && explode_on == 10 && indefinite {
            // This is likely Dark Heresy righteous fury
            if explosion_count == 1 {
                result.notes.push(
                    "⚔️ **RIGHTEOUS FURY!** Natural 10 rolled - additional damage!".to_string(),
                );
            } else {
                result.notes.push(format!(
                    "⚔️ **RIGHTEOUS FURY!** {} natural 10s - Emperor's wrath unleashed!",
                    explosion_count
                ));
            }
        } else {
            // Generic exploding dice message for other systems
            result
                .notes
                .push(format!("{} dice exploded", explosion_count));
        }
    }

    Ok(())
}

fn drop_dice(result: &mut RollResult, count: usize) -> Result<()> {
    if count >= result.individual_rolls.len() {
        result
            .notes
            .push("Cannot drop more dice than rolled".to_string());
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

fn keep_dice(result: &mut RollResult, count: usize, keep_low: bool) -> Result<()> {
    if count >= result.individual_rolls.len() {
        return Ok(()); // Keep all dice
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
            let old_roll = result.individual_rolls[i];
            result.individual_rolls[i] = rng.gen_range(1..=dice_sides as i32);
            rerolls_for_this_die += 1;
            total_rerolls += 1;

            // Add a note about the reroll
            if rerolls_for_this_die == 1 {
                result.notes.push(format!(
                    "Rerolled {} → {}",
                    old_roll, result.individual_rolls[i]
                ));
            }

            if !indefinite {
                break;
            }
        }
    }

    if total_rerolls >= max_total_rerolls {
        result
            .notes
            .push("Maximum rerolls reached (100)".to_string());
    }

    if total_rerolls > 0 && total_rerolls < 10 {
        // Don't spam notes for lots of rerolls
        result
            .notes
            .push(format!("{} dice rerolled", total_rerolls));
    }

    Ok(())
}

fn count_successes(result: &mut RollResult, target: u32) -> Result<()> {
    let successes = result
        .individual_rolls
        .iter()
        .filter(|&&roll| roll >= target as i32)
        .count() as i32;

    result.successes = Some(result.successes.unwrap_or(0) + successes);
    Ok(())
}

fn count_failures(result: &mut RollResult, threshold: u32) -> Result<()> {
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

fn count_botches(result: &mut RollResult, threshold: u32) -> Result<()> {
    let botches = result
        .individual_rolls
        .iter()
        .filter(|&&roll| roll <= threshold as i32)
        .count() as i32;

    result.botches = Some(botches);

    if botches > 0 {
        result
            .notes
            .push(format!("{} dice botched (≤{})", botches, threshold));
    }

    Ok(())
}
