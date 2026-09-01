//! Roll executor: `DiceRoll` → `RollResult`.
//!
//! The public entry point is [`roll_dice`].  It inspects the [`Modifier`] list
//! on the incoming [`DiceRoll`] and dispatches to a specialised handler for any
//! game system that needs non-standard resolution.  All other rolls go through
//! the standard pipeline described below.
//!
//! # Standard pipeline
//!
//! ```text
//! 1. Roll N (+ advantage/disadvantage extras) dice (sides S)
//! 2. drop the advantage/disadvantage surplus  — adv#, dis#
//! 3. apply_dice_modifying_modifiers   — explode, implode, reroll
//! 4. apply_keep_drop_modifiers        — keep high/low/middle, drop lowest
//! 5. sum kept dice → result.total
//! 6. apply_mathematical_modifiers     — +N, -N, *N, /N, +Nd6, …
//! 7. apply_special_system_modifiers   — success counting, botch, Godbound, …
//! 8. sort rolls (unless `ul` flag set)
//! ```
//!
//! **Drop before explode** is intentional: dice that are dropped are never
//! reconsidered for explosion.  Do not change this ordering.
//!
//! Step 2 and step 4 are deliberately distinct.  `d#`/`k#` (step 4) run after
//! explosions and can therefore drop exploded dice; `adv#`/`dis#` (step 2) run
//! against the initial pool only, which is what Open Legend requires.
//!
//! Imploded dice (`i#`) never enter the pool at all: they are held in
//! `implosion_rolls` and subtracted at step 5, so no keep/drop or success
//! count can ever see them.
//!
//! # Specialised handlers
//! | Handler function                  | System                        |
//! |-----------------------------------|-------------------------------|
//! | `handle_conan_skill_roll`         | Conan 2d20 skill checks       |
//! | `handle_conan_combat_roll`        | Conan combat / hit location   |
//! | `handle_d6_system_roll`           | D6 System (wild die)          |
//! | `handle_marvel_multiverse_roll`   | Marvel Multiverse RPG         |
//! | `handle_savage_worlds_roll`       | Savage Worlds (trait + wild)  |
//! | `handle_brave_new_world_roll`     | Brave New World               |
//! | `handle_silhouette_roll`          | Silhouette (count highest)    |
//! | `handle_vtm5_roll`                | Vampire: the Masquerade 5e    |
//! | `handle_mutants_masterminds_roll` | Mutants & Masterminds DC 10   |
//! | `handle_mothership_roll`          | Mothership RPG (1d100 ≤ stat) |
//!
//! `roll_dice` obtains one RNG via `rng::get_dice_rng` (ChaCha20 / StdRng
//! seeded with OS entropy + timestamp + thread/process/ASLR entropy) and
//! threads it through nested dice operands, so one expression draws from one
//! generator.  `roll_dice_with_rng` accepts the generator instead, which is
//! what lets the test suite seed a roll and pin its exact result.

use super::rng::get_dice_rng;
use super::{
    DiceGroup, DiceRoll, ESSENCE20_RANKS, HeroSystemType, LaserFeelingsType, Modifier, RollResult,
};
use anyhow::{Result, anyhow};
use rand::{Rng, RngExt};

/// Mirrors the parser's `Maximum 500 dice allowed` cap, which is enforced
/// against the written dice count before advantage extras are known.
const MAX_DICE_POOL: u64 = 500;

/// Net advantage level: advantage and disadvantage cancel each other out, per
/// the Open Legend SRD ("find the difference between the two values").
/// Positive is advantage, negative is disadvantage.
fn net_advantage_level(modifiers: &[Modifier]) -> i64 {
    modifiers
        .iter()
        .fold(0i64, |level, modifier| match modifier {
            Modifier::Advantage(count) => level + i64::from(*count),
            Modifier::Disadvantage(count) => level - i64::from(*count),
            _ => level,
        })
}

pub fn roll_dice(dice: DiceRoll) -> Result<RollResult> {
    roll_dice_with_rng(dice, &mut get_dice_rng())
}

/// Roll `dice` drawing from `rng`.
///
/// Nested dice operands (`+2d6`) recurse through here with the *same* `rng`, so
/// a caller that supplies a seeded generator gets a fully reproducible roll —
/// which is what lets the test suite pin systems whose dice are fixed by the
/// system rather than by notation. It also means one expression now builds one
/// generator instead of one per operand.
pub fn roll_dice_with_rng(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    // Validation check
    if dice.sides < 1 {
        return Err(anyhow!("Cannot roll dice with {} sides", dice.sides));
    }
    if dice.count == 0 {
        return Err(anyhow!("Cannot roll 0 dice"));
    }

    // Check for Conan system handlers
    let has_conan_skill = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::ConanSkill(_)));

    if has_conan_skill {
        return handle_conan_skill_roll(dice, rng);
    }

    let has_conan_combat = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::ConanCombat(_)));

    if has_conan_combat {
        return handle_conan_combat_roll(dice, rng);
    }

    let has_wfrp = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::Wfrp(_)));

    if has_wfrp {
        return handle_wfrp_roll(dice, rng);
    }

    // Check if this is a D6 System roll - handle it specially
    let has_d6_system = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::D6System(_, _)));

    if has_d6_system {
        return handle_d6_system_roll(dice, rng);
    }

    let has_marvel_multiverse = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::MarvelMultiverse(_, _)));

    if has_marvel_multiverse {
        return handle_marvel_multiverse_roll(dice, rng);
    }

    // Check if this is a Savage Worlds roll - handle it specially
    let has_savage_worlds = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::SavageWorlds(_)));

    if has_savage_worlds {
        // For Savage Worlds, handle it completely differently
        return handle_savage_worlds_roll(dice, rng);
    }

    let has_brave_new_world = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::BraveNewWorld(_)));

    if has_brave_new_world {
        return handle_brave_new_world_roll(dice, rng);
    }

    // Check if this is a Silhouette roll - handle it specially
    let has_silhouette = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::Silhouette(_)));

    if has_silhouette {
        return handle_silhouette_roll(dice, rng);
    }

    // Check if this is a Mutants & Masterminds roll - handle it specially
    let has_mutants_masterminds = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::MutantsMasterminds));

    if has_mutants_masterminds {
        return handle_mutants_masterminds_roll(dice, rng);
    }

    // Check if this is a Mothership roll - handle it specially
    let has_mothership = dice
        .modifiers
        .iter()
        .any(|m| matches!(m, Modifier::Mothership(_, _)));

    if has_mothership {
        return handle_mothership_roll(dice, rng);
    }

    let mut result = RollResult::from_dice(&dice);

    // Advantage/disadvantage rolls extra dice into the initial pool; the surplus
    // is discarded below, before any explosion runs.
    let advantage_level = net_advantage_level(&dice.modifiers);
    let extra_dice = advantage_level.unsigned_abs();
    let pool_size = u64::from(dice.count) + extra_dice;

    if pool_size > MAX_DICE_POOL {
        return Err(anyhow!(
            "Maximum {MAX_DICE_POOL} dice allowed (advantage/disadvantage adds {extra_dice} dice)"
        ));
    }

    // Normal dice rolling flow for non-special systems
    // Initial dice rolls
    for _ in 0..pool_size {
        let roll = rng.random_range(1..=dice.sides as i32);
        result.individual_rolls.push(roll);
    }

    // Create initial dice group for the base dice
    let base_group = DiceGroup {
        _description: format!("{}d{}", pool_size, dice.sides),
        rolls: result.individual_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    };
    result.dice_groups.push(base_group);

    // Discard the advantage/disadvantage surplus *before* exploding, so dice
    // gained from explosions are never reconsidered for the drop.
    if extra_dice > 0 {
        let surplus = extra_dice as usize;
        if advantage_level > 0 {
            drop_dice(&mut result, surplus)?;
        } else {
            drop_highest_dice(&mut result, surplus);
        }
        update_base_group(&mut result);
    }

    // Apply modifiers in the correct order for mathematical precedence
    // 1. Apply dice-modifying modifiers first (exploding, rerolls, etc.)
    apply_dice_modifying_modifiers(&mut result, rng, &dice)?;

    // 2. Apply keep/drop modifiers
    apply_keep_drop_modifiers(&mut result, &dice)?;

    // 3. Calculate base total from remaining dice
    if result.kept_rolls.is_empty() {
        result.kept_rolls = result.individual_rolls.clone();
    }
    result.total = result.kept_rolls.iter().sum();

    // 3b. Subtract imploded dice from the dice total, before any math modifiers
    result.total -= result.implosion_rolls.iter().sum::<i32>();

    // 4. Apply mathematical modifiers (add, subtract, multiply, divide)
    apply_mathematical_modifiers(&mut result, &dice, rng)?;

    // 5. Apply special system modifiers (after math modifiers for proper precedence)
    apply_special_system_modifiers(&mut result, &dice, rng)?;

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
    // Dice present before any modifier ran. Explosions append to
    // `individual_rolls`; implode must only ever consider the dice that were
    // actually rolled from the expression, so that `e10 i1` and `i1 e10` agree.
    let original_dice_count = result.individual_rolls.len();

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
            Modifier::Implode(threshold) => {
                implode_dice(result, rng, *threshold, dice.sides, original_dice_count)?;
            }
            Modifier::Reroll(threshold) => {
                reroll_dice(
                    result,
                    rng,
                    *threshold,
                    dice.sides,
                    false,
                    RerollDirection::AtOrBelow,
                )?;
                update_base_group(result);
            }
            Modifier::RerollIndefinite(threshold) => {
                reroll_dice(
                    result,
                    rng,
                    *threshold,
                    dice.sides,
                    true,
                    RerollDirection::AtOrBelow,
                )?;
                update_base_group(result);
            }
            Modifier::RerollGreater(threshold) => {
                reroll_dice(
                    result,
                    rng,
                    *threshold,
                    dice.sides,
                    false,
                    RerollDirection::AtOrAbove,
                )?;
                update_base_group(result);
            }
            Modifier::RerollGreaterIndefinite(threshold) => {
                reroll_dice(
                    result,
                    rng,
                    *threshold,
                    dice.sides,
                    true,
                    RerollDirection::AtOrAbove,
                )?;
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

/// The arithmetic modifiers (`+n`, `-n`, `*n`, `/n`), lifted out of `Modifier`
/// so they can be folded into any running value (a total, a success count).
#[derive(Debug, Clone, Copy)]
enum ArithmeticOp {
    Add(i32),
    Subtract(i32),
    Multiply(i32),
    Divide(i32),
}

impl ArithmeticOp {
    /// `Ok(None)` for a modifier that is not arithmetic. Division by zero is
    /// rejected here rather than at `apply` time, so a caller that has nothing
    /// to apply the operation to still reports the same error.
    fn from_modifier(modifier: &Modifier) -> Result<Option<Self>> {
        let op = match modifier {
            Modifier::Add(value) => Self::Add(*value),
            Modifier::Subtract(value) => Self::Subtract(*value),
            Modifier::Multiply(value) => Self::Multiply(*value),
            Modifier::Divide(value) => {
                if *value == 0 {
                    return Err(anyhow!("Cannot divide by zero"));
                }
                Self::Divide(*value)
            }
            _ => return Ok(None),
        };
        Ok(Some(op))
    }

    /// Operator symbol, for rebuilding the operation as text.
    fn symbol(self) -> &'static str {
        match self {
            Self::Add(_) => "+",
            Self::Subtract(_) => "-",
            Self::Multiply(_) => "*",
            Self::Divide(_) => "/",
        }
    }

    /// The right-hand side of the operation, for rebuilding it as text.
    fn operand(self) -> i32 {
        match self {
            Self::Add(operand)
            | Self::Subtract(operand)
            | Self::Multiply(operand)
            | Self::Divide(operand) => operand,
        }
    }

    fn apply(self, value: &mut i32) {
        match self {
            Self::Add(operand) => *value += operand,
            Self::Subtract(operand) => *value -= operand,
            Self::Multiply(operand) => *value *= operand,
            Self::Divide(operand) => *value /= operand,
        }
    }
}

/// True for the arithmetic modifiers (`+n`, `-n`, `*n`, `/n`).
fn is_arithmetic_modifier(modifier: &Modifier) -> bool {
    matches!(
        modifier,
        Modifier::Add(_) | Modifier::Subtract(_) | Modifier::Multiply(_) | Modifier::Divide(_)
    )
}

/// Fold every arithmetic modifier into `total`, in the order the user wrote
/// them. Non-arithmetic modifiers belong to the caller's own game system and
/// are skipped here.
fn apply_arithmetic_modifiers(modifiers: &[Modifier], total: &mut i32) -> Result<()> {
    for modifier in modifiers {
        if let Some(op) = ArithmeticOp::from_modifier(modifier)? {
            op.apply(total);
        }
    }
    Ok(())
}

// Update apply_mathematical_modifiers to handle the special division case AND continue with remaining modifiers
fn apply_mathematical_modifiers(
    result: &mut RollResult,
    dice: &DiceRoll,
    rng: &mut impl Rng,
) -> Result<()> {
    // Check for special division pattern: Multiply(0) followed by Add(number)
    if dice.modifiers.len() >= 2
        && let (Modifier::Multiply(0), Modifier::Add(number)) =
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
            apply_modifier_expression(result, remaining_modifiers, POST_DIVISION_MATH_RULES, rng)?;
        }
        return Ok(());
    }

    // Standard mathematical modifier processing
    apply_modifier_expression(result, &dice.modifiers, STANDARD_MATH_RULES, rng)?;
    Ok(())
}

/// Operator for a dice-operand modifier (`+2d6`, `-1d4`, `*1d2`, `/1d3`).
#[derive(Debug, Clone, Copy)]
enum DiceOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl DiceOperator {
    /// Symbol pushed into the expression that is evaluated with precedence.
    fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        }
    }

    /// Label stored on the `DiceGroup`, which decides how the group renders.
    fn group_label(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Subtract => "subtract",
            Self::Multiply => "multiply",
            Self::Divide => "divide",
        }
    }
}

/// Roll the dice operand of a `+2d6`-style modifier, append it to
/// `expression_parts`, and fold its dice into `result` so they show up in the
/// output. `merge_notes` carries the operand's notes (Hero System, for
/// instance) up into the parent roll.
fn apply_dice_operand(
    result: &mut RollResult,
    expression_parts: &mut Vec<String>,
    dice_spec: &DiceRoll,
    operator: DiceOperator,
    merge_notes: bool,
    rng: &mut impl Rng,
) -> Result<()> {
    // Same `rng` as the parent roll, so a seeded caller stays reproducible
    let operand_result = roll_dice_with_rng(dice_spec.clone(), rng)?;

    if matches!(operator, DiceOperator::Divide) && operand_result.total == 0 {
        return Err(anyhow!("Cannot divide by zero (dice result was 0)"));
    }

    expression_parts.push(operator.symbol().to_string());
    expression_parts.push(format!("{}", operand_result.total));

    if merge_notes {
        result.notes.extend(operand_result.notes.clone());
    }

    // Both pools get the operand dice: `individual_rolls` for display, and
    // `kept_rolls` so every total is computed from the same set of dice.
    result
        .individual_rolls
        .extend(operand_result.individual_rolls.clone());
    result.kept_rolls.extend(operand_result.kept_rolls.clone());

    add_dice_group(result, dice_spec, &operand_result, operator.group_label());
    Ok(())
}

/// How a modifier expression is built. The standard path and the path that
/// resumes after the `number / dice` special case differ only here, and the
/// difference is load-bearing: the standard path merges an added roll's notes
/// and drops the `*0` marker the parser uses to encode `number / dice`, while
/// the resumed path does neither, because that marker was already consumed.
#[derive(Debug, Clone, Copy)]
struct MathModifierRules {
    merge_added_dice_notes: bool,
    skip_zero_multiplier: bool,
}

const STANDARD_MATH_RULES: MathModifierRules = MathModifierRules {
    merge_added_dice_notes: true,
    skip_zero_multiplier: true,
};

const POST_DIVISION_MATH_RULES: MathModifierRules = MathModifierRules {
    merge_added_dice_notes: false,
    skip_zero_multiplier: false,
};

/// Build an expression from `modifiers` and evaluate it with proper precedence,
/// starting from the total already in `result`.
fn apply_modifier_expression(
    result: &mut RollResult,
    modifiers: &[Modifier],
    rules: MathModifierRules,
    rng: &mut impl Rng,
) -> Result<()> {
    let mut expression_parts = vec![format!("{}", result.total)];

    for modifier in modifiers {
        match modifier {
            Modifier::AddDice(dice_to_add) => {
                apply_dice_operand(
                    result,
                    &mut expression_parts,
                    dice_to_add,
                    DiceOperator::Add,
                    rules.merge_added_dice_notes,
                    rng,
                )?;
            }
            Modifier::SubtractDice(dice_to_subtract) => {
                apply_dice_operand(
                    result,
                    &mut expression_parts,
                    dice_to_subtract,
                    DiceOperator::Subtract,
                    false,
                    rng,
                )?;
            }
            Modifier::MultiplyDice(dice_to_multiply) => {
                apply_dice_operand(
                    result,
                    &mut expression_parts,
                    dice_to_multiply,
                    DiceOperator::Multiply,
                    false,
                    rng,
                )?;
            }
            Modifier::DivideDice(dice_to_divide) => {
                apply_dice_operand(
                    result,
                    &mut expression_parts,
                    dice_to_divide,
                    DiceOperator::Divide,
                    false,
                    rng,
                )?;
            }
            Modifier::Multiply(0) if rules.skip_zero_multiplier => {}
            _ => {
                if let Some(op) = ArithmeticOp::from_modifier(modifier)? {
                    expression_parts.push(op.symbol().to_string());
                    expression_parts.push(format!("{}", op.operand()));
                }
            }
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

    // Apply pre-target mathematical modifiers exactly ONCE at the beginning
    if !target_positions.is_empty() && has_math_modifiers {
        let first_target_position = target_positions[0];
        let pre_target_modifiers: Vec<Modifier> = dice.modifiers[..first_target_position]
            .iter()
            .filter(|m| is_arithmetic_modifier(m))
            .cloned()
            .collect();

        if !pre_target_modifiers.is_empty() {
            apply_pre_target_mathematical_modifiers(result, &pre_target_modifiers)?;
        }
    }

    // Track if we've applied a special system (success counting, etc.)
    let mut has_special_system = false;

    // Process modifiers in order, respecting their position relative to targets
    for modifier in dice.modifiers.iter() {
        match modifier {
            Modifier::Alien => {
                apply_alien_base_modifier(result)?;
                has_special_system = true;
            }
            Modifier::AlienStress(stress_level) => {
                apply_alien_stress_modifier(result, *stress_level, rng)?;
                has_special_system = true;
            }
            // TargetWithDoubleSuccess must come BEFORE existing Target case
            Modifier::TargetWithDoubleSuccess(target, double_success_value) => {
                count_dice_with_double_success(result, *target, *double_success_value)?;
                has_special_system = true;
            }
            Modifier::ForgedDark => {
                apply_forged_dark_mechanics(result)?;
                has_special_system = true;
            }
            Modifier::ForgedDarkZero => {
                apply_forged_dark_zero_mechanics(result)?;
                has_special_system = true;
            }
            Modifier::Daggerheart => {
                apply_daggerheart_mechanics(result)?;
                has_special_system = true;
            }
            Modifier::TargetLowerWithDoubleSuccess(target, double_value) => {
                count_dice_with_target_lower_double_success(result, *target, *double_value)?;
                has_special_system = true;
            }
            Modifier::Target(value) => {
                count_dice_matching(result, |roll| roll >= *value as i32, "successes")?;
                has_special_system = true;
            }
            Modifier::TargetLower(value) => {
                count_dice_matching(result, |roll| roll <= *value as i32, "successes")?;
                has_special_system = true;
            }
            Modifier::Failure(value) => {
                if result.successes.is_none() {
                    result.successes = Some(0);
                }
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
            Modifier::CyberpunkRedDamage => {
                apply_cyberpunk_red_damage(result, rng, dice.sides)?;
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
            Modifier::VampireMasquerade5(pool_size, hunger_dice) => {
                *result = handle_vtm5_roll(dice.clone(), rng, *pool_size, *hunger_dice)?;
                return Ok(());
            }
            Modifier::LaserFeelings(_, target, roll_type) => {
                // Extract dice count from the actual dice expression
                let dice_count = dice.count;
                apply_laser_feelings_mechanics(result, *target, roll_type, dice_count)?;
                has_special_system = true;
            }
            Modifier::WildWorlds(cut_count) => {
                apply_wild_worlds_mechanics(result, *cut_count)?;
                has_special_system = true;
            }
            Modifier::MutantsMasterminds => {
                has_special_system = true;
            }
            Modifier::Mothership(_, _) => {
                // Mothership is handled in the main roll_dice function
                // Don't process it here
            }
            Modifier::Wfrp(_) => {
                // WFRP is handled in the main roll_dice function
                // Don't process it here
            }
            Modifier::PlotDie => {
                apply_plot_die_conversion(result)?;
                has_special_system = true;
            }
            Modifier::Essence20(rank, specialization) => {
                apply_essence20_skill_dice(result, rng, *rank, *specialization, dice.sides)?;
            }
            Modifier::DarkestHouse(called_upon) => {
                apply_darkest_house_die(result, rng, *called_upon)?;
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

    // Finalize success/failure calculation after all core modifiers
    finalize_success_failure_calculation(result)?;
    // Apply mathematical modifiers that come AFTER target modifiers (to success counts)
    if has_special_system && has_math_modifiers && result.successes.is_some() {
        // Find mathematical modifiers that come after the last target modifier
        if let Some(&last_target_pos) = target_positions.last() {
            let post_target_modifiers: Vec<_> = dice.modifiers[(last_target_pos + 1)..]
                .iter()
                .filter(|m| is_arithmetic_modifier(m))
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
        // The base group lists every die that was rolled; dropped dice are
        // rendered separately (struck through) from `result.dropped_rolls`, so
        // repeating them in the group would print them twice.
        base_group.rolls = result.individual_rolls.clone();
        base_group
            .rolls
            .extend(result.dropped_rolls.iter().copied());
        base_group.dropped_rolls.clear();
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

    // The subtraction will happen later in finalize_success_failure_calculation()

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

        // Only complications for soak rolls: Glory effects do not apply, so the
        // critical flag is always false here.
        add_wrath_die_notes(
            result,
            has_complication,
            false,
            &wrath_dice_values,
            wrath_dice_count,
        );
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

    if indefinite {
        // Indefinite explosions: keep exploding new dice that meet threshold
        let max_explosions = 100;
        let mut i = 0;
        while i < result.individual_rolls.len() && explosion_count < max_explosions {
            if result.individual_rolls[i] >= explode_on as i32 {
                let new_roll = rng.random_range(1..=dice_sides as i32);
                result.individual_rolls.push(new_roll);
                explosion_count += 1;
            }
            i += 1;
        }

        if explosion_count >= max_explosions {
            result
                .notes
                .push("Maximum explosions reached (100)".to_string());
        }
    } else {
        // Non-indefinite explosions: process ALL original dice that meet threshold
        // Store the original number of dice to avoid exploding newly added dice
        let original_dice_count = result.individual_rolls.len();

        for i in 0..original_dice_count {
            if result.individual_rolls[i] >= explode_on as i32 {
                let new_roll = rng.random_range(1..=dice_sides as i32);
                result.individual_rolls.push(new_roll);
                explosion_count += 1;
            }
        }
        // Note: No maximum explosion limit for non-indefinite since we're only
        // processing original dice once
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

// Imploding dice: the mirror of `explode_dice`. Every original die at or below
// the threshold rolls one extra die whose value is *subtracted* from the total.
// Imploded dice never chain, and they are deliberately kept out of
// `individual_rolls`/`kept_rolls` so keep/drop and target counting ignore them.
fn implode_dice(
    result: &mut RollResult,
    rng: &mut impl Rng,
    threshold: Option<u32>,
    dice_sides: u32,
    original_dice_count: usize,
) -> Result<()> {
    let implode_on = threshold.unwrap_or(1);
    let considered = original_dice_count.min(result.individual_rolls.len());

    let imploded: Vec<i32> = result.individual_rolls[..considered]
        .iter()
        .filter(|&&roll| roll <= implode_on as i32)
        .map(|_| rng.random_range(1..=dice_sides as i32))
        .collect();

    if imploded.is_empty() {
        return Ok(());
    }

    result.notes.push(if imploded.len() == 1 {
        "1 die imploded".to_string()
    } else {
        format!("{} dice imploded", imploded.len())
    });

    result.dice_groups.push(DiceGroup {
        _description: "imploded dice".to_string(),
        rolls: imploded.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "subtract".to_string(),
    });

    result.implosion_rolls.extend(imploded);

    Ok(())
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

// Mirror of drop_dice for the highest dice, used by disadvantage
fn drop_highest_dice(result: &mut RollResult, count: usize) {
    let mut sorted_rolls = result.individual_rolls.clone();
    sorted_rolls.sort_by(|a, b| b.cmp(a)); // Highest first

    for _ in 0..count.min(sorted_rolls.len()) {
        if let Some(pos) = result
            .individual_rolls
            .iter()
            .position(|&roll| roll == sorted_rolls[0])
        {
            let dropped = result.individual_rolls.remove(pos);
            result.dropped_rolls.push(dropped);
            sorted_rolls.remove(0);
        }
    }
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

    let mut indexed_rolls = indexed_rolls(result);

    // Sort by value
    if keep_low {
        indexed_rolls.sort_by_key(|&(_, roll)| roll);
    } else {
        indexed_rolls.sort_by_key(|&(_, roll)| -roll);
    }

    // Keep the specified number of dice, drop the rest
    let kept_indices: Vec<usize> = indexed_rolls.iter().take(count).map(|&(i, _)| i).collect();

    partition_kept_dice(result, &kept_indices);
    Ok(())
}

/// Pair each die with its position in the roll, so a keep/drop decision made on
/// sorted values can be applied back in the order the dice were rolled.
fn indexed_rolls(result: &RollResult) -> Vec<(usize, i32)> {
    result
        .individual_rolls
        .iter()
        .enumerate()
        .map(|(i, &roll)| (i, roll))
        .collect()
}

/// Keep the dice at `kept_indices` in roll order and move the rest to
/// `dropped_rolls`, where they render struck through.
fn partition_kept_dice(result: &mut RollResult, kept_indices: &[usize]) {
    let mut new_rolls = Vec::new();
    for (i, &roll) in result.individual_rolls.iter().enumerate() {
        if kept_indices.contains(&i) {
            new_rolls.push(roll);
        } else {
            result.dropped_rolls.push(roll);
        }
    }

    result.individual_rolls = new_rolls;
}

/// Which side of the threshold triggers a reroll: `r`/`ir` reroll a die that is
/// at or below it, `rg`/`irg` a die that is at or above it.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RerollDirection {
    AtOrBelow,
    AtOrAbove,
}

impl RerollDirection {
    fn triggers(self, roll: i32, threshold: u32) -> bool {
        match self {
            Self::AtOrBelow => roll <= threshold as i32,
            Self::AtOrAbove => roll >= threshold as i32,
        }
    }
}

fn reroll_dice(
    result: &mut RollResult,
    rng: &mut impl Rng,
    threshold: u32,
    dice_sides: u32,
    indefinite: bool,
    direction: RerollDirection,
) -> Result<()> {
    let mut total_rerolls = 0;
    let max_total_rerolls = 100;

    for i in 0..result.individual_rolls.len() {
        let mut rerolls_for_this_die = 0;
        let max_rerolls_per_die = if indefinite { 100 } else { 1 };

        while direction.triggers(result.individual_rolls[i], threshold)
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
            // STUN = raw dice total including any pip bonus (+1 modifier).
            // Math modifiers run before this function, so result.total already
            // includes pip bonuses. This is correct — pips add to STUN.
            let stun_damage = result.total;
            let mut body_damage: i32 = 0;

            // Calculate BODY from each dice group.
            // The base group rolls d6s (dice.sides == 6); AddDice groups roll d3s (sides == 3).
            // A pip bonus (+1) never contributes BODY and is handled via Add modifiers, not dice groups.
            for group in &result.dice_groups {
                let is_d3 = group._description.contains("d3");

                for &roll in &group.rolls {
                    let body = if is_d3 {
                        match roll {
                            3 => 1,
                            2 => i32::from(rng.random_bool(0.5)),
                            _ => 0, // roll of 1
                        }
                    } else {
                        // d6
                        match roll {
                            6 => 2,
                            1 => 0,
                            _ => 1, // 2-5
                        }
                    };
                    body_damage += body;
                }
            }
            // Normal damage - just use the total as-is
            result.notes.push(format!(
                "Normal damage: {body_damage} BODY, {stun_damage} STUN"
            ));
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
        // `/0` is still rejected on a roll with no success count, so the error
        // does not depend on what the dice happened to produce.
        if let Some(op) = ArithmeticOp::from_modifier(modifier)?
            && let Some(successes) = result.successes.as_mut()
        {
            op.apply(successes);
        }
    }
    Ok(())
}

/// One die rolled with explosions: every face rolled, and the values callers
/// derive from them. Bundled together so a caller needs one line, not four.
struct ExplodingDie {
    rolls: Vec<i32>,
    total: i32,
    explosions: usize,
}

/// Roll one die of `sides` sides, rolling again each time it comes up at its
/// maximum. Capped at 100 explosions so a pathological RNG cannot spin forever.
fn roll_exploding_die(rng: &mut impl Rng, sides: u32) -> ExplodingDie {
    let max = sides as i32;
    let mut rolls = vec![rng.random_range(1..=max)];
    let mut explosions = 0;
    while rolls.last().copied().unwrap_or(0) >= max && explosions < 100 {
        rolls.push(rng.random_range(1..=max));
        explosions += 1;
    }
    ExplodingDie {
        total: rolls.iter().sum(),
        explosions,
        rolls,
    }
}

/// Append a dice group for display. The `DiceGroup` literal is otherwise
/// repeated verbatim in every system handler that shows more than one pool.
fn push_dice_group(
    result: &mut RollResult,
    description: String,
    rolls: Vec<i32>,
    modifier_type: &str,
) {
    result.dice_groups.push(DiceGroup {
        _description: description,
        rolls,
        dropped_rolls: Vec::new(),
        modifier_type: modifier_type.to_string(),
    });
}

fn handle_savage_worlds_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    let mut result = RollResult::from_dice(&dice);

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

    // Trait die explodes on its own maximum, wild die is always a d6
    let trait_die = roll_exploding_die(rng, trait_sides);
    let wild_die = roll_exploding_die(rng, 6);
    let (trait_total, wild_total) = (trait_die.total, wild_die.total);
    let (trait_explosions, wild_explosions) = (trait_die.explosions, wild_die.explosions);

    // Create dice groups for display
    push_dice_group(
        &mut result,
        format!("1d{trait_sides} ie{trait_sides}"),
        trait_die.rolls.clone(),
        "trait",
    );
    push_dice_group(
        &mut result,
        "1d6 ie6".to_string(),
        wild_die.rolls.clone(),
        "wild",
    );

    // Add all rolls to individual_rolls for display
    result.individual_rolls.extend(trait_die.rolls);
    result.individual_rolls.extend(wild_die.rolls);

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
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

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
    let mut result = RollResult::from_dice(&dice);

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
    let wild_die = roll_exploding_die(rng, 6);
    let (wild_total, wild_explosions) = (wild_die.total, wild_die.explosions);

    // Create dice groups for display
    push_dice_group(
        &mut result,
        format!("{count}d6"),
        base_rolls.clone(),
        "base",
    );
    push_dice_group(
        &mut result,
        "1d6 ie6".to_string(),
        wild_die.rolls.clone(),
        "add",
    );

    // Add all rolls to individual_rolls and kept_rolls
    result.individual_rolls.extend(base_rolls);
    result.individual_rolls.extend(wild_die.rolls);
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
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

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

/// Collapse a Marvel edge/trouble batch into one note, so a large pool does not
/// bury the roll under one line per reroll.
fn push_marvel_reroll_note(result: &mut RollResult, kind: &str, count: i32, details: &[String]) {
    if count == 1 {
        result
            .notes
            .push(format!("{kind} 1: Rerolled {}", details[0]));
    } else {
        result.notes.push(format!(
            "{kind} rerolls: {}",
            details
                .iter()
                .enumerate()
                .map(|(i, detail)| format!("#{}: {}", i + 1, detail))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
}

fn handle_marvel_multiverse_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    let mut result = RollResult::from_dice(&dice);

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

        push_marvel_reroll_note(&mut result, "Edge", edges, &edge_details);
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

        push_marvel_reroll_note(&mut result, "Trouble", troubles, &trouble_details);
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

    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

    Ok(result)
}

/// One Essence20 skill rank as rolled: the dice it is made of and their total.
struct Essence20RankRoll {
    count: u32,
    sides: u32,
    rolls: Vec<i32>,
    total: i32,
}

impl Essence20RankRoll {
    fn label(&self) -> String {
        if self.count == 1 {
            format!("d{}", self.sides)
        } else {
            format!("{}d{}", self.count, self.sides)
        }
    }

    /// A rank scores a critical success when every die in it shows its maximum.
    /// This includes the d2 — Renegade counts a maximum on *any* skill die, so
    /// a small skill die is a boon rather than a penalty.
    fn is_critical(&self) -> bool {
        self.total == (self.count * self.sides) as i32
    }
}

/// Essence20 skill dice: roll the character's skill rank and add it to the d20.
///
/// With a specialization the character also rolls every *lower* rank and keeps
/// the single highest result; the ranks that lost are shown struck through.
/// Any rank that comes up all-maximum is a critical success, so a lower rank is
/// worth watching even when a bigger one supplies the total.  The d20 is not a
/// skill die and so cannot crit — a natural 20 is reported on its own.
fn apply_essence20_skill_dice(
    result: &mut RollResult,
    rng: &mut impl Rng,
    rank: u32,
    specialization: bool,
    base_sides: u32,
) -> Result<()> {
    let highest_rank = rank as usize; // validated 1..=ESSENCE20_RANKS.len() by the parser
    let lowest_rank = if specialization { 1 } else { highest_rank };

    // Rolled highest rank first, so a tie on totals keeps the larger die
    let mut rolled: Vec<Essence20RankRoll> = Vec::new();
    for rank_number in (lowest_rank..=highest_rank).rev() {
        let (count, sides) = ESSENCE20_RANKS[rank_number - 1];
        let rolls: Vec<i32> = (0..count)
            .map(|_| rng.random_range(1..=sides as i32))
            .collect();
        let total = rolls.iter().sum();
        rolled.push(Essence20RankRoll {
            count,
            sides,
            rolls,
            total,
        });
    }

    let mut kept = 0;
    for (index, rank_roll) in rolled.iter().enumerate() {
        if rank_roll.total > rolled[kept].total {
            kept = index;
        }
    }

    result.total += rolled[kept].total;

    let discarded: Vec<i32> = rolled
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != kept)
        .flat_map(|(_, rank_roll)| rank_roll.rolls.iter().copied())
        .collect();

    result.dice_groups.push(DiceGroup {
        _description: rolled[kept].label(),
        rolls: rolled[kept].rolls.clone(),
        dropped_rolls: discarded,
        modifier_type: "add".to_string(),
    });

    let critical_ranks: Vec<String> = rolled
        .iter()
        .filter(|rank_roll| rank_roll.is_critical())
        .map(|rank_roll| rank_roll.label())
        .collect();

    if !critical_ranks.is_empty() {
        result.notes.push(format!(
            "💥 **CRITICAL SUCCESS!** Maximum rolled on {}",
            critical_ranks.join(", ")
        ));
    }

    if base_sides == 20 && result.kept_rolls.contains(&20) {
        result
            .notes
            .push("🎯 **Natural 20** on the d20".to_string());
    }

    if specialization {
        let ladder: Vec<String> = rolled
            .iter()
            .map(|rank_roll| rank_roll.label())
            .collect::<Vec<_>>();
        result.notes.push(format!(
            "Specialization: kept {} ({}) — highest of {}",
            rolled[kept].label(),
            rolled[kept].total,
            ladder.join(", ")
        ));
    }

    Ok(())
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
            // Stored positive: the dice group carrying these rolls is already
            // typed "subtract", which renders the minus sign. Storing a negative
            // here printed "- [-3]" and, for a subtracted 1, collided with the
            // `-1` Marvel-logo sentinel in `format_dice_groups`.
            additional_rolls.push(additional_roll);
            total_result -= additional_roll;
            explosion_notes.push(format!(
                "💀 **CRITICAL FAILURE!** Rolled 1, subtracted {additional_roll}"
            ));
        }
        _ => {
            // Normal roll, no explosion
        }
    }

    finalize_d10_explosion(
        result,
        original_roll,
        &additional_rolls,
        total_result,
        explosion_notes,
    );

    Ok(())
}

/// Fold an exploded d10 (Cyberpunk Red, Witcher) back into `result`: the base
/// die and the dice it exploded into render as two groups, and the running
/// total becomes the single kept value.
fn finalize_d10_explosion(
    result: &mut RollResult,
    original_roll: i32,
    additional_rolls: &[i32],
    total_result: i32,
    explosion_notes: Vec<String>,
) {
    if !additional_rolls.is_empty() {
        let base_group = DiceGroup {
            _description: "1d10".to_string(),
            rolls: vec![original_roll],
            dropped_rolls: Vec::new(),
            modifier_type: "base".to_string(),
        };

        let exploded_up = original_roll == 10;
        let explosion_group = DiceGroup {
            _description: if exploded_up {
                "Critical Success"
            } else {
                "Critical Failure"
            }
            .to_string(),
            rolls: additional_rolls.to_vec(),
            dropped_rolls: Vec::new(),
            modifier_type: if exploded_up { "add" } else { "subtract" }.to_string(),
        };

        result.dice_groups = vec![base_group, explosion_group];

        // Show the exploded dice alongside the original
        result.individual_rolls.extend(additional_rolls.iter());
    }

    result.total = total_result;
    result.kept_rolls = vec![total_result];
    result.notes.extend(explosion_notes);
}

// Cyberpunk Red damage (`cpd`): the dice total stays a plain sum, because the
// table subtracts the target's armor SP from it (and doubles it for an aimed
// head shot) before anything else happens.
//
// Two or more 6s inflict a Critical Injury. Its 5 bonus damage is deliberately
// NOT added to the total: RAW, that damage goes straight to Hit Points without
// ablating armor and without being modified by hit location, so it cannot ride
// along on a number that armor is about to reduce. The Critical Injury lands
// even when no damage at all got through the target's SP.
//
// The 6s are counted before any `*` multiplier is applied, which is what makes
// autofire (`cpd2 * 3`) come out right: RAW checks the raw 2d6 for double 6s
// and multiplies afterwards.
//
// The 2d6 table roll is made here, but the injury tables themselves are
// corebook content and are not reproduced -- the player looks up the number.
fn apply_cyberpunk_red_damage(
    result: &mut RollResult,
    rng: &mut impl Rng,
    sides: u32,
) -> Result<()> {
    if sides != 6 {
        return Err(anyhow!("Cyberpunk Red damage only works with d6 dice"));
    }

    let sixes = result
        .individual_rolls
        .iter()
        .filter(|&&roll| roll == 6)
        .count();

    if sixes < 2 {
        return Ok(());
    }

    let first_injury_die = rng.random_range(1..=6);
    let second_injury_die = rng.random_range(1..=6);
    let injury_total = first_injury_die + second_injury_die;

    result.notes.push(format!(
        "💥 **CRITICAL INJURY!** ({sixes} sixes) - +5 damage direct to Hit Points, ignores armor SP"
    ));
    result.notes.push(format!(
        "Critical Injury roll: [{first_injury_die}, {second_injury_die}] = {injury_total} (Body table; Head table on an aimed head shot)"
    ));

    Ok(())
}

fn apply_mathematical_modifiers_to_cpr_total(
    result: &mut RollResult,
    dice: &DiceRoll,
) -> Result<()> {
    // `+`/`-` are accumulated and applied last so a CPR damage multiplier such
    // as `cpd2 * 3` scales the dice total alone, not the flat bonus.
    let mut modifier_total = 0;

    for modifier in &dice.modifiers {
        match ArithmeticOp::from_modifier(modifier)? {
            Some(ArithmeticOp::Add(value)) => modifier_total += value,
            Some(ArithmeticOp::Subtract(value)) => modifier_total -= value,
            Some(op) => op.apply(&mut result.total),
            None => {} // Skip non-mathematical modifiers
        }
    }

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
                // Stored positive: the dice group carrying these rolls is already
                // typed "subtract", which renders the minus sign. Storing a negative
                // here printed "- [-3]" and, for a subtracted 1, collided with the
                // `-1` Marvel-logo sentinel in `format_dice_groups`.
                additional_rolls.push(additional_roll);
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

    finalize_d10_explosion(
        result,
        original_roll,
        &additional_rolls,
        total_result,
        explosion_notes,
    );

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
    let mut result = RollResult::from_dice(&dice);

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
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

    Ok(result)
}

fn handle_conan_skill_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    let mut result = RollResult::from_dice(&dice);

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
            let additional_result = roll_dice_with_rng(additional_dice.clone(), rng)?;

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
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

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
    let mut result = RollResult::from_dice(&dice);

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
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

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
    let mut result = RollResult::from_dice(&dice);

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
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

    Ok(result)
}

fn keep_middle_dice(result: &mut RollResult, count: usize) -> Result<()> {
    let available_dice = result.individual_rolls.len();

    // If we want to keep all or more dice than available, keep all
    if count >= available_dice {
        return Ok(());
    }

    // Create indexed rolls for tracking original positions
    let mut indexed_rolls = indexed_rolls(result);

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

    partition_kept_dice(result, &kept_indices);
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
            .push(format!("≤{double_success_value} = 2 successes"));
    } else {
        result.notes.push(format!(
            "≤{double_success_value} = 2 successes, ≤{target} = 1 success"
        ));
    }

    Ok(())
}

fn handle_vtm5_roll(
    dice: DiceRoll,
    rng: &mut impl Rng,
    pool_size: u32,
    hunger_dice: u32,
) -> Result<RollResult> {
    // Initialize result structure
    let mut result = RollResult {
        successes: Some(0),
        ..RollResult::from_dice(&dice)
    };

    let regular_dice = pool_size - hunger_dice;
    let mut regular_rolls = Vec::new();
    let mut hunger_rolls = Vec::new();

    // Roll regular dice
    for _ in 0..regular_dice {
        regular_rolls.push(rng.random_range(1..=10));
    }

    // Roll hunger dice
    for _ in 0..hunger_dice {
        hunger_rolls.push(rng.random_range(1..=10));
    }

    // Combine all rolls for display
    result.individual_rolls.extend(&regular_rolls);
    result.individual_rolls.extend(&hunger_rolls);
    result.kept_rolls = result.individual_rolls.clone();

    // Create dice groups for display
    if !regular_rolls.is_empty() {
        result.dice_groups.push(DiceGroup {
            _description: format!("{regular_dice}d10 Regular"),
            rolls: regular_rolls.clone(),
            dropped_rolls: Vec::new(),
            modifier_type: "regular".to_string(),
        });
    }

    if !hunger_rolls.is_empty() {
        result.dice_groups.push(DiceGroup {
            _description: format!("{hunger_dice}d10 Hunger"),
            rolls: hunger_rolls.clone(),
            dropped_rolls: Vec::new(),
            modifier_type: "hunger".to_string(),
        });
    }

    // Calculate basic successes (6+)
    let regular_successes = regular_rolls.iter().filter(|&&r| r >= 6).count();
    let hunger_successes = hunger_rolls.iter().filter(|&&r| r >= 6).count();
    let mut total_successes = regular_successes + hunger_successes;

    // Count 10s for critical success calculation
    let regular_tens = regular_rolls.iter().filter(|&&r| r == 10).count();
    let hunger_tens = hunger_rolls.iter().filter(|&&r| r == 10).count();
    let total_tens = regular_tens + hunger_tens;

    // Apply critical success rule: pairs of 10s = 4 successes each pair
    if total_tens >= 2 {
        let pairs = total_tens / 2;
        let extra_successes = pairs * 2; // Each pair adds 2 extra (4 total - 2 base = 2 extra)
        total_successes += extra_successes;
        result.notes.push(format!(
            "{pairs} pairs of 10s add +{extra_successes} successes"
        ));
    }

    // Check for special results
    let has_successes = total_successes > 0;
    let has_crit = total_tens >= 2;
    let has_hunger_tens = hunger_tens > 0;
    let has_hunger_ones = hunger_rolls.contains(&1);

    // Determine result type and add notes
    if has_crit && has_successes && has_hunger_tens {
        result
            .notes
            .push("**MESSY CRITICAL** - Success with bestial consequences".to_string());
    } else if has_crit && has_successes {
        result.notes.push("**CRITICAL SUCCESS**".to_string());
    } else if !has_successes && has_hunger_ones {
        result
            .notes
            .push("**BESTIAL FAILURE** - Failed with bestial consequences".to_string());
    } else if !has_successes {
        result.notes.push("**FAILURE**".to_string());
    }

    result.successes = Some(total_successes as i32);
    result.total = total_successes as i32;

    // Apply any mathematical modifiers to the success count
    apply_mathematical_modifiers_to_vtm5_successes(&mut result, &dice)?;

    Ok(result)
}

// Helper function to apply mathematical modifiers to VTM5 success counts
fn apply_mathematical_modifiers_to_vtm5_successes(
    result: &mut RollResult,
    dice: &DiceRoll,
) -> Result<()> {
    for modifier in &dice.modifiers {
        // A VTM5 roll is read in successes, so the total tracks them; both move
        // together, and only when there is a success count to move.
        if let Some(op) = ArithmeticOp::from_modifier(modifier)?
            && let Some(mut successes) = result.successes
        {
            op.apply(&mut successes);
            result.successes = Some(successes);
            op.apply(&mut result.total);
        }
    }
    Ok(())
}

fn apply_laser_feelings_mechanics(
    result: &mut RollResult,
    target: u32,
    roll_type: &LaserFeelingsType,
    dice_count: u32,
) -> Result<()> {
    if result.individual_rolls.is_empty() {
        return Err(anyhow!("No dice rolled for Lasers & Feelings"));
    }

    // Validate that we're using d6s
    let all_d6 = result
        .individual_rolls
        .iter()
        .all(|&roll| (1..=6).contains(&roll));
    if !all_d6 {
        return Err(anyhow!("Lasers & Feelings requires d6 dice"));
    }

    let target_i32 = target as i32;
    let mut successes = 0;
    let mut laser_feelings_count = 0;

    // Count successes and LASER FEELINGS
    for &roll in &result.individual_rolls {
        match roll_type {
            LaserFeelingsType::Lasers => {
                // Lasers: success on <= target
                if roll <= target_i32 {
                    successes += 1;
                }
            }
            LaserFeelingsType::Feelings => {
                // Feelings: success on >= target
                if roll >= target_i32 {
                    successes += 1;
                }
            }
        }

        // LASER FEELINGS: rolling exactly the target number
        if roll == target_i32 {
            laser_feelings_count += 1;
        }
    }

    // Set success count
    result.successes = Some(successes);

    // Clear any existing total since this is a success-counting system
    result.total = 0;

    result.notes.push(format!(
        "Lasers & Feelings: {dice_count}d6 target {target} ({roll_type})"
    ));

    if laser_feelings_count > 0 {
        result.notes.push(format!(
            "💡 **{laser_feelings_count}** LASER FEELINGS! Ask the GM a question!"
        ));
    }

    Ok(())
}

fn apply_alien_base_modifier(result: &mut RollResult) -> Result<()> {
    // Count 6s as successes for base Alien dice
    let success_count = result.kept_rolls.iter().filter(|&&roll| roll == 6).count() as i32;

    result.successes = Some(result.successes.unwrap_or(0) + success_count);

    Ok(())
}

fn apply_alien_stress_modifier(
    result: &mut RollResult,
    stress_level: u32,
    rng: &mut impl Rng,
) -> Result<()> {
    // Count 6s as successes for stress dice
    let success_count = result.kept_rolls.iter().filter(|&&roll| roll == 6).count() as i32;

    result.successes = Some(result.successes.unwrap_or(0) + success_count);

    // Count 1s on stress dice for panic checks
    let ones_count = result.kept_rolls.iter().filter(|&&roll| roll == 1).count() as i32;

    result.alien_stress_ones = Some(ones_count);
    result.alien_stress_level = Some(stress_level);

    // Add stress system note
    result.notes.push(format!(
        "⚡ **STRESS DICE** (Level {stress_level}): 6s = successes, 1s = panic risk"
    ));

    // If we rolled any 1s on stress dice, trigger panic roll
    if ones_count > 0 {
        let panic_roll = rng.random_range(1..=6) + stress_level as i32;
        result.alien_panic_roll = Some(panic_roll);

        // Add panic roll note with interpretation
        let panic_effect = interpret_panic_roll(panic_roll);
        result.notes.push(format!(
            "💀 **PANIC ROLL**: {ones_count}d6 + {stress_level} stress = **{panic_roll}** → {panic_effect}"
        ));

        // Add flavor note about push restriction
        result
            .notes
            .push("⚠️  **Cannot push this roll** (rolled 1s on stress dice)".to_string());
    } else {
        // Add push availability note
        result
            .notes
            .push("🔄 **Push available**: Add 'p' to alias to push (e.g., alien4s2p)".to_string());
    }

    Ok(())
}

fn interpret_panic_roll(panic_total: i32) -> String {
    match panic_total {
        1..=6 => "Keeping it together".to_string(),
        7 => "Tremble - Shaky hands (-2 to next roll)".to_string(),
        8 => "Drop Item - You drop a weapon or important item".to_string(),
        9 => "Freeze - You lose your next turn".to_string(),
        10 => "Seek Cover - You must move to safety immediately".to_string(),
        11 => "Scream - Everyone who hears you must make a Panic Roll".to_string(),
        12 => "Flee - You must move away from the threat".to_string(),
        13 => "Berserk - You attack the nearest person or creature".to_string(),
        14 => "Catatonic - You become unresponsive for one turn".to_string(),
        15..=99 => "Heart Attack - You suffer a heart attack and become Broken".to_string(),
        _ => "Catastrophic Panic".to_string(),
    }
}

/// Apply Forged in the Dark standard mechanics
/// - Take the highest die from the pool
/// - Classify result: 1-3=failure, 4-5=partial success, 6=success, multiple 6s=critical
fn apply_forged_dark_mechanics(result: &mut RollResult) -> Result<()> {
    if result.kept_rolls.is_empty() {
        return Err(anyhow!("No dice to apply FitD mechanics to"));
    }

    // Find the highest die
    let highest_die = *result.kept_rolls.iter().max().unwrap();

    // Count how many 6s we have for critical success detection
    let six_count = result.kept_rolls.iter().filter(|&&die| die == 6).count();

    // Classify the result based on highest die
    let (outcome, fitd_result) = match highest_die {
        1..=3 => ("FAILURE", "Bad outcome - GM makes a move"),
        4..=5 => ("PARTIAL SUCCESS", "You do it, but with consequences"),
        6 => {
            if six_count > 1 {
                ("CRITICAL SUCCESS", "Great effect + extra advantage!")
            } else {
                ("SUCCESS", "You do it well")
            }
        }
        _ => ("UNKNOWN", "Invalid die result"), // Shouldn't happen with d6s
    };

    // Store the result
    result.fitd_outcome = Some(outcome.to_string());
    result.fitd_result = Some(fitd_result.to_string());
    result.fitd_highest_die = Some(highest_die);

    if six_count > 1 {
        result.notes.push(format!(
            "⚡ **CRITICAL**: {six_count} sixes rolled - extra advantage!"
        ));
    }

    Ok(())
}

/// Apply Forged in the Dark zero dice mechanics
/// - Roll 2d6 and take the LOWEST result (desperate situation)
/// - Same classification as standard FitD
fn apply_forged_dark_zero_mechanics(result: &mut RollResult) -> Result<()> {
    if result.kept_rolls.len() != 2 {
        return Err(anyhow!("FitD zero dice requires exactly 2 dice"));
    }

    // Find the lowest die (opposite of standard FitD)
    let lowest_die = *result.kept_rolls.iter().min().unwrap();

    // Classify the result based on lowest die
    let (outcome, fitd_result) = match lowest_die {
        1..=3 => ("FAILURE", "Bad outcome - GM makes a hard move"),
        4..=5 => (
            "PARTIAL SUCCESS",
            "You do it, but with serious consequences",
        ),
        6 => ("SUCCESS", "You do it, but it's costly"),
        _ => ("UNKNOWN", "Invalid die result"),
    };

    // Store the result
    result.fitd_outcome = Some(outcome.to_string());
    result.fitd_result = Some(fitd_result.to_string());
    result.fitd_highest_die = Some(lowest_die); // Store as "highest" even though it's lowest

    result
        .notes
        .push("⚠️ **DESPERATE POSITION**: Zero dice - risky situation!".to_string());

    Ok(())
}

fn finalize_success_failure_calculation(result: &mut RollResult) -> Result<()> {
    // Only apply failure subtraction if we have both successes and failures tracked
    if let (Some(successes), Some(failures)) = (result.successes, result.failures) {
        // Apply failure subtraction after all modifiers (including cancel) are processed
        let final_successes = successes - failures;
        result.successes = Some(final_successes);
    }
    Ok(())
}
/// Apply Daggerheart mechanics to a 2d12 roll (Hope and Fear dice)
///
/// Rules:
/// - Roll two d12 dice (Hope and Fear)
/// - Report individual values with labels
/// - If Hope > Fear: show "roll is TOTAL with Hope"
/// - If Fear > Hope: show "roll is TOTAL with Fear"
/// - If Hope == Fear: show "Critical Success!"
fn apply_daggerheart_mechanics(result: &mut RollResult) -> Result<()> {
    if result.dice_groups.is_empty() || result.dice_groups[0].rolls.is_empty() {
        return Err(anyhow!("No dice to apply Daggerheart mechanics to"));
    }

    let dice_group = &result.dice_groups[0];
    if dice_group.rolls.len() != 2 {
        return Err(anyhow!(
            "Daggerheart requires exactly 2 dice (Hope and Fear)"
        ));
    }

    let hope_die = dice_group.rolls[0];
    let fear_die = dice_group.rolls[1];
    let total = hope_die + fear_die;

    // Clear existing total since we'll set our own
    result.total = total;

    // Determine the result based on Hope vs Fear
    let daggerheart_result = if hope_die == fear_die {
        "Critical Success!".to_string()
    } else if hope_die > fear_die {
        format!("roll is {total} with Hope")
    } else {
        format!("roll is {total} with Fear")
    };

    // Add detailed notes showing individual dice values
    result.notes.push(format!(
        "**Daggerheart Roll**: Hope: {hope_die}, Fear: {fear_die} → {daggerheart_result}"
    ));

    // Store the result as a comment for display
    result.comment = Some(daggerheart_result);

    Ok(())
}

/// Apply Wild Worlds RPG mechanics (The Wildsea RPG system)
fn apply_wild_worlds_mechanics(result: &mut RollResult, cut_count: Option<u32>) -> Result<()> {
    if result.individual_rolls.is_empty() {
        return Err(anyhow!("No dice to apply Wild Worlds mechanics to"));
    }

    // Start with all rolled dice
    let mut working_dice = result.individual_rolls.clone();

    // Apply cutting if specified (remove highest dice before evaluation)
    if let Some(cut) = cut_count {
        let cut_amount = cut as usize;
        if cut_amount >= working_dice.len() {
            return Err(anyhow!(
                "Cannot cut {} dice from {} rolled",
                cut,
                working_dice.len()
            ));
        }

        // Sort dice in descending order and remove the highest ones
        working_dice.sort_by(|a, b| b.cmp(a)); // Sort descending
        working_dice.drain(0..cut_amount); // Remove highest dice

        // Add note about cutting
        result.notes.push(format!("Cut {} highest dice", cut));
    }

    if working_dice.is_empty() {
        return Err(anyhow!("No dice remaining after cutting"));
    }

    // Find the highest die value (this determines the result in Wild Worlds)
    let highest_die = *working_dice.iter().max().unwrap();

    // Check for doubles/triples (any matching values = twist)
    let has_twist = has_matching_dice(&working_dice);

    // Interpret the result based on highest die (Wild Worlds rules)
    let interpretation = match highest_die {
        6 => "Triumph",      // Complete success
        4 | 5 => "Conflict", // Success with drawback
        1..=3 => "Disaster", // Failure with complication
        _ => unreachable!("d6 can only roll 1-6"),
    };

    // Set the result total to the highest die (Wild Worlds uses highest die, not sum)
    result.total = highest_die;

    // Add interpretation note with twist indication
    let mut result_text = format!("Wild Worlds: {} ({})", interpretation, highest_die);
    if has_twist {
        result_text.push_str(" + Twist");
    }
    result.notes.push(result_text);

    // Add twist explanation if applicable
    if has_twist {
        result
            .notes
            .push("Doubles detected - add a small twist to the outcome!".to_string());
    }

    // Store the working dice for display (after cuts)
    result.kept_rolls = working_dice;

    Ok(())
}

fn apply_plot_die_conversion(result: &mut RollResult) -> Result<()> {
    let mut symbols = Vec::new();
    let mut plot_total = 0;

    for &roll in &result.kept_rolls {
        let (symbol, value) = match roll {
            1 => ("C+2", 2),
            2 => ("C+4", 4),
            3 | 4 => ("_", 0),
            5 | 6 => ("Opp", 0),
            _ => return Err(anyhow!("Invalid Plot die value: {}", roll)),
        };
        symbols.push(symbol.to_string());
        plot_total += value;
    }

    result.plot_symbols = Some(symbols);

    let original_dice_total: i32 = result.kept_rolls.iter().sum();
    let plot_adjustment = plot_total - original_dice_total;
    result.total += plot_adjustment;

    Ok(())
}

/// The Darkest House (Monte Cook Games) House Die.
///
/// Every action roll — never a damage roll — is accompanied by an extra d6.
/// The House Die does not affect success or failure; if it is higher than the
/// dice used for the action, the house acts.  With a Boon or a Bane only the
/// two dice actually used count, so the discarded die is ignored here.
///
/// "Calling upon the house" (`tdhc`) adds the House Die to the result instead:
/// the house then acts automatically and the character gains a Doom.
fn apply_darkest_house_die(
    result: &mut RollResult,
    rng: &mut impl Rng,
    called_upon: bool,
) -> Result<()> {
    let action_dice = if result.kept_rolls.is_empty() {
        &result.individual_rolls
    } else {
        &result.kept_rolls
    };

    let highest_action_die = *action_dice
        .iter()
        .max()
        .ok_or_else(|| anyhow!("No dice rolled for The Darkest House"))?;

    let house_die = rng.random_range(1..=6);

    if called_upon {
        result.total += house_die;
        result.notes.push(format!(
            "House Die: [{house_die}] added to the roll - 🏚️ The House acts, and you gain 1 Doom"
        ));
        return Ok(());
    }

    if house_die > highest_action_die {
        result
            .notes
            .push(format!("House Die: [{house_die}] - 🏚️ The House acts"));
    } else {
        result
            .notes
            .push(format!("House Die: [{house_die}] - The House waits"));
    }

    Ok(())
}

/// Helper function to detect matching dice values (for twist detection)
fn has_matching_dice(dice: &[i32]) -> bool {
    for i in 1..=6 {
        let count = dice.iter().filter(|&&x| x == i).count();
        if count >= 2 {
            return true; // Found doubles (or triples, etc.)
        }
    }
    false
}

pub fn handle_mutants_masterminds_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    // Initialize complete RollResult with all required fields
    let mut result = RollResult::from_dice(&dice);

    // Roll the dice
    for _ in 0..dice.count {
        let roll = rng.random_range(1..=dice.sides as i32);
        result.individual_rolls.push(roll);
        result.kept_rolls.push(roll);
    }

    // Create dice group for display
    result.dice_groups.push(DiceGroup {
        _description: format!("{}d{}", dice.count, dice.sides),
        rolls: result.individual_rolls.clone(),
        dropped_rolls: Vec::new(),
        modifier_type: "base".to_string(),
    });

    // Calculate base total
    result.total = result.kept_rolls.iter().sum();

    // Apply mathematical modifiers (add, subtract, multiply, divide)
    apply_arithmetic_modifiers(&dice.modifiers, &mut result.total)?;

    // Calculate degrees of success/failure against DC 10
    let dc = 10;
    let total_vs_dc = result.total - dc;

    if total_vs_dc >= 0 {
        // Success: every 5 points above DC is one degree of success
        let degrees = (total_vs_dc / 5) + 1;
        result.successes = Some(degrees);

        if degrees == 1 {
            result.notes.push(format!(
                "**SUCCESS** (1 degree: rolled {} vs DC {})",
                result.total, dc
            ));
        } else {
            result.notes.push(format!(
                "**SUCCESS** ({} degrees: rolled {} vs DC {})",
                degrees, result.total, dc
            ));
        }
    } else {
        // Failure: every 5 points below DC is one degree of failure
        let degrees = ((-total_vs_dc - 1) / 5) + 1;
        result.failures = Some(degrees);

        if degrees == 1 {
            result.notes.push(format!(
                "**FAILURE** (1 degree: rolled {} vs DC {})",
                result.total, dc
            ));
        } else {
            result.notes.push(format!(
                "**FAILURE** ({} degrees: rolled {} vs DC {})",
                degrees, result.total, dc
            ));
        }
    }

    Ok(result)
}

// Warhammer Fantasy Roleplay 4e (`wfrp#`): roll-under d100 reporting Success
// Levels rather than a die total.
//
// SL is the tens digit of the target minus the tens digit of the roll, so the
// roll total carries the SL and the note carries the verdict. The note has to,
// because WFRP distinguishes +0 (scraped a success) from -0 (just missed) and
// an integer cannot.
//
// Two RAW rules ride on top of the subtraction:
//   - 01-05 always succeeds and 96-00 always fails, whatever the target. The SL
//     is then floored at +1 or capped at -1, "whichever is higher/lower", so an
//     automatic result never reports a better SL than it earned.
//   - Any double (11, 22 ... 99, 00) makes a success Astounding and a failure
//     Astounding in the other direction - a Critical Hit or Fumble in combat.
//     This is independent of SL and applies to the same roll.
//
// The die is rolled 1-100 with 100 standing in for 00, which is what puts
// 96-00 in the automatic failure band and gives 00 a tens digit of 10.
/// The reading of a single WFRP test, separated from the rolling so every
/// target/roll pair can be checked directly.
pub struct WfrpTest {
    pub success: bool,
    pub success_levels: i32,
    /// A double (11, 22 ... 99, 00): Astounding Success or Failure, and a
    /// Critical Hit or Fumble in combat.
    pub is_double: bool,
    /// Inside the 01-05 or 96-00 band, where the roll decides on its own.
    pub is_automatic: bool,
}

impl WfrpTest {
    /// Success Levels as WFRP writes them, including the "-0" of a test that
    /// failed on the ones digit alone.
    pub fn signed_success_levels(&self) -> String {
        if self.success_levels > 0 {
            format!("+{}", self.success_levels)
        } else if self.success_levels == 0 {
            // WFRP writes both, and they mean different things: +0 scraped a
            // success, -0 missed by the ones digit alone.
            if self.success { "+0" } else { "-0" }.to_string()
        } else {
            self.success_levels.to_string()
        }
    }

    fn notes(&self, target: i32) -> Vec<String> {
        let verdict = match (self.success, self.is_automatic) {
            (true, true) => "**AUTOMATIC SUCCESS** (01-05)",
            (true, false) => "**SUCCESS**",
            (false, true) => "**AUTOMATIC FAILURE** (96-00)",
            (false, false) => "**FAILURE**",
        };

        let mut notes = vec![format!(
            "{verdict} - {} SL (Target {target})",
            self.signed_success_levels()
        )];

        if self.is_double {
            notes.push(
                if self.success {
                    "**ASTOUNDING SUCCESS** (doubles) - Critical Hit in combat"
                } else {
                    "**ASTOUNDING FAILURE** (doubles) - Fumble in combat"
                }
                .to_string(),
            );
        }

        notes
    }
}

/// Read a d100 against a WFRP target. `roll` is 1-100, with 100 standing in
/// for 00 - which is what places 96-00 in the automatic failure band and gives
/// 00 a tens digit of 10.
pub fn wfrp_test_outcome(target: i32, roll: i32) -> WfrpTest {
    let automatic_success = roll <= 5;
    let automatic_failure = roll >= 96;

    let success = if automatic_success {
        true
    } else if automatic_failure {
        false
    } else {
        roll <= target
    };

    // Success Levels are the tens digit of the target minus the tens digit of
    // the roll. An automatic result is then floored at +1 or capped at -1,
    // "whichever is higher/lower", so it never reports a better SL than earned.
    let rolled_success_levels = target / 10 - roll / 10;
    let success_levels = if automatic_success {
        rolled_success_levels.max(1)
    } else if automatic_failure {
        rolled_success_levels.min(-1)
    } else {
        rolled_success_levels
    };

    WfrpTest {
        success,
        success_levels,
        is_double: roll == 100 || roll / 10 == roll % 10,
        is_automatic: automatic_success || automatic_failure,
    }
}

fn handle_wfrp_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    let target = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::Wfrp(target) = m {
                Some(*target)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected WFRP modifier"))? as i32;

    let roll = rng.random_range(1..=100);
    let outcome = wfrp_test_outcome(target, roll);
    let notes = outcome.notes(target);

    // `successes` is left at its default of `None` so the value slot prints the
    // SL. `successes` would print "N successes", which is a different quantity
    // from a Success Level.
    Ok(RollResult {
        individual_rolls: vec![roll],
        kept_rolls: vec![roll],
        // The total is the SL, not the die: a WFRP test is read in Success
        // Levels, and the die itself is already shown in the roll display.
        total: outcome.success_levels,
        notes,
        ..RollResult::from_dice(&dice)
    })
}

fn handle_mothership_roll(dice: DiceRoll, rng: &mut impl Rng) -> Result<RollResult> {
    // Extract Mothership modifier
    let (stat_target, is_advantage_or_disadvantage) = dice
        .modifiers
        .iter()
        .find_map(|m| {
            if let Modifier::Mothership(stat, is_adv) = m {
                Some((*stat, *is_adv))
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("Expected Mothership modifier"))?;

    // Determine actual stat value (default to 50 if not specified)
    let stat = stat_target.unwrap_or(50);

    // Validate stat range
    if !(1..=99).contains(&stat) {
        return Err(anyhow!("Mothership stat must be 1-99, got {}", stat));
    }

    // Roll the dice (either 1d100 or 2d100)
    let num_dice = if dice.count == 2 { 2 } else { 1 };
    let mut rolls = Vec::new();
    for _ in 0..num_dice {
        let roll = rng.random_range(1..=100);
        rolls.push(roll);
    }

    // Helper function to check if a roll is doubles
    let is_double_digit = |roll: i32| -> bool {
        if roll == 100 {
            return true; // 00 counts as doubles (displayed as 100)
        }
        let tens = roll / 10;
        let ones = roll % 10;
        tens == ones
    };

    // Categorize rolls
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    enum RollCategory {
        CritSuccess = 0, // Best
        Success = 1,
        CritFailure = 2,
        Failure = 3, // Worst
    }

    let categorize_roll = |roll: i32| -> RollCategory {
        let is_double = is_double_digit(roll);
        let is_success = roll <= stat as i32;

        match (is_success, is_double) {
            (true, true) => RollCategory::CritSuccess,
            (true, false) => RollCategory::Success,
            (false, true) => RollCategory::CritFailure,
            (false, false) => RollCategory::Failure,
        }
    };

    // Select the best roll based on advantage/disadvantage
    let selected_roll = if rolls.len() == 1 {
        rolls[0]
    } else {
        let roll1 = rolls[0];
        let roll2 = rolls[1];
        let cat1 = categorize_roll(roll1);
        let cat2 = categorize_roll(roll2);

        if is_advantage_or_disadvantage {
            // Advantage: prefer better category, then lower value within category
            if cat1 < cat2 {
                roll1
            } else if cat2 < cat1 {
                roll2
            } else {
                // Same category, prefer lower roll
                if roll1 < roll2 { roll1 } else { roll2 }
            }
        } else {
            // Disadvantage: prefer worse category, then higher value within category
            if cat1 > cat2 {
                roll1
            } else if cat2 > cat1 {
                roll2
            } else {
                // Same category, prefer higher roll
                if roll1 > roll2 { roll1 } else { roll2 }
            }
        }
    };

    // Build result
    let selected_category = categorize_roll(selected_roll);
    let is_success = selected_roll <= stat as i32;
    let _is_crit = is_double_digit(selected_roll);

    let mut result = RollResult {
        individual_rolls: rolls.clone(),
        kept_rolls: vec![selected_roll],
        dropped_rolls: rolls
            .iter()
            .filter(|&&r| r != selected_roll)
            .copied()
            .collect(),
        total: selected_roll,
        successes: if is_success { Some(1) } else { None },
        failures: if !is_success { Some(1) } else { None },
        ..RollResult::from_dice(&dice)
    };

    // Add descriptive notes
    if rolls.len() == 2 {
        let mode = if is_advantage_or_disadvantage {
            "Advantage"
        } else {
            "Disadvantage"
        };
        result.notes.push(format!(
            "Mothership {} roll (target ≤{}): rolled {} and {}, selected {}",
            mode, stat, rolls[0], rolls[1], selected_roll
        ));
    } else {
        result.notes.push(format!(
            "Mothership roll (target ≤{}): rolled {}",
            stat, selected_roll
        ));
    }

    // Add result description
    let result_desc = match selected_category {
        RollCategory::CritSuccess => "**CRITICAL SUCCESS**",
        RollCategory::Success => "**Success**",
        RollCategory::CritFailure => "**CRITICAL FAILURE**",
        RollCategory::Failure => "**Failure**",
    };
    result.notes.push(result_desc.to_string());

    // Add explanation for selection if advantage/disadvantage
    if rolls.len() == 2 && rolls[0] != rolls[1] {
        let cat1 = categorize_roll(rolls[0]);
        let cat2 = categorize_roll(rolls[1]);

        if cat1 != cat2 {
            let cat_name = |cat: &RollCategory| match cat {
                RollCategory::CritSuccess => "crit success",
                RollCategory::Success => "success",
                RollCategory::CritFailure => "crit failure",
                RollCategory::Failure => "failure",
            };
            result.notes.push(format!(
                "({} chosen: {} {} > {} {})",
                selected_roll,
                selected_roll,
                cat_name(&categorize_roll(selected_roll)),
                rolls.iter().find(|&&r| r != selected_roll).unwrap(),
                cat_name(&categorize_roll(
                    *rolls.iter().find(|&&r| r != selected_roll).unwrap()
                ))
            ));
        }
    }

    Ok(result)
}
