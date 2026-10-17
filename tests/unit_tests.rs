// tests/unit_tests.rs - Core dice logic, parsing, and rolling tests
//
// This file contains unit tests for the core dice functionality:
// - Basic dice parsing and validation
// - Mathematical operations and modifiers
// - Core dice modifier behavior (exploding, keep/drop, rerolls)
// - Error handling and input validation

use dicemaiden_rs::dice::{
    Modifier, format_multiple_results, parse_and_roll, parse_and_roll_with_rng, parser,
    rng::create_seeded_rng,
};

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Test helper: Assert that a dice expression parses and rolls successfully
fn assert_valid(input: &str) {
    let result = parse_and_roll(input);
    assert!(
        result.is_ok(),
        "Failed to parse: '{}' - Error: {:?}",
        input,
        result.err()
    );
    let results = result.unwrap();
    assert!(!results.is_empty(), "No results for: '{}'", input);
}

/// Test helper: Assert that a dice expression fails to parse
fn assert_invalid(input: &str) {
    let result = parse_and_roll(input);
    assert!(result.is_err(), "Expected error for: '{}'", input);
}

// ============================================================================
// BASIC DICE TESTS
// ============================================================================

#[test]
fn test_basic_dice_parsing() {
    // Table-driven test for basic dice parsing and validation
    let test_cases = vec![
        // (input, should_succeed, expected_count, expected_sides)

        // Valid basic formats
        ("1d6", true, 1, 6),
        ("2d6", true, 2, 6),
        ("d6", true, 1, 6),    // Default count = 1
        ("1d%", true, 1, 100), // Percentile
        ("d%", true, 1, 100),
        // Boundary conditions
        ("500d1000", true, 500, 1000), // Max allowed
        ("1d1", true, 1, 1),           // Min sides
        // Invalid cases
        ("501d6", false, 0, 0),         // Too many dice
        ("1d1001", false, 0, 0),        // Too many sides
        ("0d6", false, 0, 0),           // Zero dice
        ("1d0", false, 0, 0),           // Zero sides
        ("-1d6", false, 0, 0),          // Negative dice count
        ("1d-6", false, 0, 0),          // Negative sides
        ("99999999999d6", false, 0, 0), // Count overflows u32 (once rolled 1 die)
        ("1d99999999999", false, 0, 0), // Sides overflow u32
        // Dice operands get the same limits as the main roll
        ("1d6 + 501d6", false, 0, 0),
        ("1d6 - 1d1001", false, 0, 0),
        ("1d6 * 1d4000000000", false, 0, 0), // Once panicked the roller
        ("1d6 / 0d6", false, 0, 0),
        ("1d6 + 1d0", false, 0, 0),
        ("3d6 501d6", false, 0, 0), // Bare additional dice
        ("d6s501", false, 0, 0),    // D6 System pool
        // Not in the snapshot corpus: releases before the cap hang rolling these,
        // which would stall tools/diff-vs-release.sh against an older tag.
        ("d6s2147483648", false, 0, 0),
        ("d6s4294967295", false, 0, 0),
    ];

    for (input, should_succeed, expected_count, expected_sides) in test_cases {
        let result = parser::parse_dice_string(input);

        if should_succeed {
            assert!(result.is_ok(), "Should parse successfully: '{}'", input);
            let dice = result.unwrap();
            assert_eq!(
                dice.len(),
                1,
                "Should have one dice expression for: '{}'",
                input
            );
            assert_eq!(
                dice[0].count, expected_count,
                "Wrong count for: '{}'",
                input
            );
            assert_eq!(
                dice[0].sides, expected_sides,
                "Wrong sides for: '{}'",
                input
            );
        } else {
            assert!(result.is_err(), "Should fail to parse: '{}'", input);
        }
    }
}

#[test]
fn test_blank_input_is_rejected() {
    // The snapshot corpus skips blank lines, so these are the malformed inputs
    // it cannot pin. Every other malformed expression lives in the corpus.
    for blank in ["", "   ", "\t"] {
        assert_invalid(blank);
    }
}

// ============================================================================
// MATHEMATICAL OPERATIONS
// ============================================================================

#[test]
fn test_leading_number_ignores_spacing() {
    // A leading number lands after the dice's own modifiers whether or not
    // they are written as one token. `4 + 4d10t7` has always added 4 to the
    // success count, so the spaced form must too.
    let same_roll = vec![
        ("4 + 4d10t7", "4 + 4d10 t7"),
        ("4 + 4d10t7+1", "4 + 4d10 t7 + 1"),
        ("5 - 3d10t7", "5 - 3d10 t7"),
        ("2 * 4d10t7", "2 * 4d10 t7"),
        ("4 + 4d6k3", "4 + 4d6 k3"),
        ("4 + 4d10e10", "4 + 4d10 e10"),
        ("5 + 1d6+2", "5 + 1d6 + 2"),
    ];

    for (unspaced, spaced) in same_roll {
        for seed in [1, 42, 1337] {
            let roll = |expression: &str| {
                let results = parse_and_roll_with_rng(expression, &mut create_seeded_rng(seed))
                    .unwrap_or_else(|e| panic!("'{expression}' should roll: {e}"));
                format_multiple_results(&results)
            };
            assert_eq!(
                roll(unspaced),
                roll(spaced),
                "'{unspaced}' and '{spaced}' should be the same roll (seed {seed})"
            );
        }
    }
}

#[test]
fn test_left_to_right_evaluation() {
    // Test that mathematical operations follow left-to-right order, not PEMDAS
    let math_tests = vec![
        // (expression, expected_result, description)
        ("1d1 + 2 * 3", 9, "(1 + 2) * 3 = 9, not 1 + (2 * 3) = 7"),
        ("1d1 + 6 / 2", 3, "(1 + 6) / 2 = 3, not 1 + (6 / 2) = 4"),
        ("1d1 + 2 * 3 - 4", 5, "((1 + 2) * 3) - 4 = 5"),
        ("1d1 + 10 * 2", 22, "(1 + 10) * 2 = 22"),
        ("1d1 + 15 - 6 / 3", 3, "((1 + 15) - 6) / 3 = 3"),
        ("1d1 + 2 * 3 / 6", 1, "((1 + 2) * 3) / 6 = 1"),
        ("10d1 / 2 / 5", 1, "(10 / 2) / 5 = 1"),
        // Dice on either side of an operator
        ("3d1 * 2d1", 6, "3 * 2 = 6"),
        ("8d1 / 2d1", 4, "8 / 2 = 4"),
        ("200/1d1", 200, "200 / 1 = 200"),
        ("100 / 2d1", 50, "100 / 2 = 50"),
        ("200 / 4d1", 50, "200 / 4 = 50"),
    ];

    for (expression, expected, description) in math_tests {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(result[0].total, expected, "Left-to-right: {}", description);
    }
}

// ============================================================================
// DICE MODIFIERS
// ============================================================================

#[test]
fn test_add_dice_modifier_attachment() {
    // Test that modifiers correctly attach to the AddDice, not the base dice
    // This is a more specific test for the internal structure

    let parsed = parser::parse_dice_string("1d20+2d6 k1").unwrap();
    assert_eq!(parsed.len(), 1, "Should parse as one dice expression");

    let dice = &parsed[0];

    // Should have one AddDice modifier
    let add_dice_modifiers: Vec<_> = dice
        .modifiers
        .iter()
        .filter(|m| matches!(m, Modifier::AddDice(_)))
        .collect();
    assert_eq!(
        add_dice_modifiers.len(),
        1,
        "Should have exactly one AddDice modifier"
    );

    // The AddDice should have the k1 modifier attached to it
    if let Modifier::AddDice(ref added_dice) = dice.modifiers[0] {
        assert_eq!(added_dice.count, 2, "Added dice should be 2d6");
        assert_eq!(added_dice.sides, 6, "Added dice should be 2d6");

        // The added dice should have the k1 modifier
        let keep_modifiers: Vec<_> = added_dice
            .modifiers
            .iter()
            .filter(|m| matches!(m, Modifier::KeepHigh(_)))
            .collect();
        assert_eq!(
            keep_modifiers.len(),
            1,
            "Added dice should have exactly one Keep modifier"
        );

        if let Modifier::KeepHigh(count) = keep_modifiers[0] {
            assert_eq!(*count, 1, "Should keep 1 die from the added 2d6");
        }
    } else {
        panic!("First modifier should be AddDice");
    }
}

#[test]
fn test_imploding_dice_mechanics() {
    // `i` implodes on the minimum face only; every imploded die subtracts a
    // fresh die from the total, so the result can drop below the dice sum.
    for _ in 0..200 {
        let result = parse_and_roll("4d6 i1").unwrap();
        let roll = &result[0];

        let kept_sum: i32 = roll.kept_rolls.iter().sum();
        let implosion_sum: i32 = roll.implosion_rolls.iter().sum();

        // One implosion per die showing a 1, and no more
        let ones = roll.kept_rolls.iter().filter(|&&d| d == 1).count();
        assert_eq!(
            roll.implosion_rolls.len(),
            ones,
            "Expected one imploded die per 1 rolled: {:?}",
            roll
        );

        // Imploded dice are subtracted, and are valid faces of the die
        assert_eq!(roll.total, kept_sum - implosion_sum);
        assert!(roll.implosion_rolls.iter().all(|&d| (1..=6).contains(&d)));

        // Imploded dice never join the dice pool
        assert_eq!(roll.individual_rolls.len(), 4);
    }

    // `i#` implodes on anything at or below the threshold
    for _ in 0..200 {
        let result = parse_and_roll("6d10 i3").unwrap();
        let roll = &result[0];
        let low_dice = roll.kept_rolls.iter().filter(|&&d| d <= 3).count();
        assert_eq!(roll.implosion_rolls.len(), low_dice);
    }

    // i10 on a d10 implodes every die, exactly once each (no chaining)
    for _ in 0..50 {
        let result = parse_and_roll("5d10 i10").unwrap();
        assert_eq!(result[0].implosion_rolls.len(), 5);
    }

    // Dice that never meet the threshold never implode
    for _ in 0..50 {
        let result = parse_and_roll("5d10 i1").unwrap();
        let roll = &result[0];
        if !roll.kept_rolls.contains(&1) {
            assert!(roll.implosion_rolls.is_empty());
            assert_eq!(roll.total, roll.kept_rolls.iter().sum::<i32>());
        }
    }
}

#[test]
fn test_imploding_dice_combinations() {
    // Implosion is applied to the dice total before mathematical modifiers
    for _ in 0..100 {
        let result = parse_and_roll("4d6 i1 + 5").unwrap();
        let roll = &result[0];
        let implosion_sum: i32 = roll.implosion_rolls.iter().sum();
        assert_eq!(
            roll.total,
            roll.kept_rolls.iter().sum::<i32>() - implosion_sum + 5
        );
    }

    // Imploded dice are not droppable or keepable - the penalty survives k/d
    for _ in 0..200 {
        let result = parse_and_roll("4d6 i1 k2").unwrap();
        let roll = &result[0];
        assert_eq!(roll.kept_rolls.len(), 2);
        assert_eq!(
            roll.total,
            roll.kept_rolls.iter().sum::<i32>() - roll.implosion_rolls.iter().sum::<i32>()
        );
    }

    // Only dice rolled from the expression implode - dice gained from an
    // explosion never do, whichever order the two modifiers are written in
    for expression in ["1d10 i1 e10", "1d10 e10 i1"] {
        for _ in 0..300 {
            let result = parse_and_roll(expression).unwrap();
            let roll = &result[0];
            assert!(
                roll.implosion_rolls.len() <= 1,
                "'{}' imploded an exploded die: {:?}",
                expression,
                roll
            );
        }
    }

    // With target counting the roll reports successes; imploded dice are not
    // dice in the pool, so they neither add nor remove successes
    for _ in 0..200 {
        let result = parse_and_roll("6d10 i1 t7").unwrap();
        let roll = &result[0];
        let hits = roll.kept_rolls.iter().filter(|&&d| d >= 7).count() as i32;
        assert_eq!(roll.successes, Some(hits));
    }

    // Roll sets and comments carry implosion through untouched
    let result = parse_and_roll("3 4d6 i1 ! imploding").unwrap();
    assert_eq!(result.len(), 3);
    for roll in &result {
        assert_eq!(
            roll.total,
            roll.kept_rolls.iter().sum::<i32>() - roll.implosion_rolls.iter().sum::<i32>()
        );
    }
}

#[test]
fn test_imploding_dice_display() {
    // Imploded dice render as a subtracted group, never as a negative value
    for _ in 0..300 {
        let result = parse_and_roll("6d6 i2").unwrap();
        let roll = &result[0];
        let output = roll.to_string();

        if roll.implosion_rolls.is_empty() {
            assert!(
                !output.contains(" - `["),
                "Unexpected subtraction: {output}"
            );
        } else {
            assert!(
                output.contains(" - `["),
                "Imploded dice should display as a subtracted group: {output}"
            );
            assert!(
                !output.contains("`[-"),
                "Imploded dice must not display as negatives: {output}"
            );
            assert!(
                !output.contains("**M**"),
                "Imploded 1 collided with the Marvel logo sentinel: {output}"
            );

            let note = if roll.implosion_rolls.len() == 1 {
                "1 die imploded".to_string()
            } else {
                format!("{} dice imploded", roll.implosion_rolls.len())
            };
            assert!(
                roll.notes.contains(&note),
                "Missing note '{note}': {output}"
            );
        }
    }
}

#[test]
fn test_implode_does_not_break_other_i_modifiers() {
    // `i` is a prefix of ie/ir/irg - these must keep their existing meaning
    let prefix_cases = vec![
        ("4d6 ie", Modifier::ExplodeIndefinite(None)),
        ("4d6 ie6", Modifier::ExplodeIndefinite(Some(6))),
        ("4d6 ir1", Modifier::RerollIndefinite(1)),
        ("4d6 irg5", Modifier::RerollGreaterIndefinite(5)),
        ("4d6ie6", Modifier::ExplodeIndefinite(Some(6))),
        ("4d6ir1", Modifier::RerollIndefinite(1)),
        ("4d6irg5", Modifier::RerollGreaterIndefinite(5)),
    ];

    for (input, expected) in prefix_cases {
        let result = parser::parse_dice_string(input).unwrap();
        let matched = result[0].modifiers.iter().any(|m| match (m, &expected) {
            (Modifier::ExplodeIndefinite(a), Modifier::ExplodeIndefinite(b)) => a == b,
            (Modifier::RerollIndefinite(a), Modifier::RerollIndefinite(b)) => a == b,
            (Modifier::RerollGreaterIndefinite(a), Modifier::RerollGreaterIndefinite(b)) => a == b,
            _ => false,
        });
        assert!(
            matched,
            "'{}' should parse as {:?}, got {:?}",
            input, expected, result[0].modifiers
        );
        assert!(
            !result[0]
                .modifiers
                .iter()
                .any(|m| matches!(m, Modifier::Implode(_))),
            "'{}' must not parse as an implode modifier: {:?}",
            input,
            result[0].modifiers
        );
    }

    // Combined tokens mixing implode with its longer-prefixed neighbours
    for input in ["4d6i1ie6", "4d6ir1i1", "4d6i1k2", "4d6e6i1"] {
        assert_valid(input);
    }
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[test]
fn test_input_length_limits() {
    // Test DoS protection
    let over_limit = "1d6+".repeat(250) + "1"; // Over 1000 chars
    let result = parser::parse_dice_string(&over_limit);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Input too long"));
}

#[test]
fn test_private_flag_detected_on_failed_rolls() {
    // A failed roll has no results to carry `private`, so its error asks the
    // parser directly. That answer must match the flag on rolls that succeed.
    let cases = vec![
        ("p 100/4cod", true),
        ("p 1d6+2147483647+1", true),
        ("s p 10 + 2 + 2d6", true),
        ("1d20 ; p 100/3df", true),
        ("p 2d6", true),
        ("1d20 ; p 2d6", true),
        ("100/4cod", false),
        ("1d6 ! p", false),
        ("2d6", false),
    ];

    for (expression, expected) in cases {
        assert_eq!(
            parser::is_private_expression(expression),
            expected,
            "private detection for '{expression}'"
        );
        if let Ok(results) = parse_and_roll(expression) {
            assert_eq!(
                results.iter().any(|r| r.private),
                expected,
                "result flag for '{expression}'"
            );
        }
    }
}

// ============================================================================
// REGRESSION TESTS
// ============================================================================

#[test]
fn test_standalone_l_modifier_rejection() {
    // Ensure that standalone 'l' is properly rejected to prevent
    // "Unrecognized modifier pattern: 'l' in 'l'" errors

    let invalid_l_expressions = vec![
        "1d6 l",     // Standalone l modifier
        "2d6 k2 l",  // l after other modifiers
        "3d10 l t7", // l in middle of modifiers
        "4d6 l e6",  // l before explode
    ];

    for expression in invalid_l_expressions {
        let result = parse_and_roll(expression);
        assert!(
            result.is_err(),
            "Standalone 'l' regression: '{}' should be rejected",
            expression
        );

        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Unrecognized modifier pattern: 'l'"),
            "Standalone 'l' regression: '{}' should have specific error message, got: {}",
            expression,
            error_msg
        );
    }
}

// ============================================================================
// KEEP ON ADDED DICE
// ============================================================================

#[test]
fn test_multiple_dice_groups_with_add_dice() {
    // Test AddDice expressions that create multiple dice groups
    let test_cases = vec![
        ("2d1 + 3d1 k2", vec![(2, 0), (3, 1)]),
        ("1d1 + 4d1 k2", vec![(1, 0), (4, 2)]),
    ];

    for (expression, expected_groups) in test_cases {
        let results = parse_and_roll(expression).unwrap();
        let result = &results[0];

        assert_eq!(
            result.dice_groups.len(),
            expected_groups.len(),
            "AddDice expression '{}' should have {} dice groups",
            expression,
            expected_groups.len()
        );

        for (i, &(expected_total, expected_dropped)) in expected_groups.iter().enumerate() {
            let group = &result.dice_groups[i];

            assert_eq!(
                group.rolls.len(),
                expected_total,
                "Group {} in '{}' should have {} total dice, got {}",
                i,
                expression,
                expected_total,
                group.rolls.len()
            );

            assert_eq!(
                group.dropped_rolls.len(),
                expected_dropped,
                "Group {} in '{}' should have {} dropped dice, got {}",
                i,
                expression,
                expected_dropped,
                group.dropped_rolls.len()
            );
        }
    }
}

#[test]
fn test_keep_on_added_dice_shows_dropped_dice() {
    // Dice a keep drops stay visible, struck through, wherever the keep sits.
    // (expression, output shows dropped dice)
    let cases = vec![
        ("1d20+3d6 k2", true),
        ("2d20 + 3d6 k2", true),
        ("3d20 k2 + 2d6", true),
        ("2d6 + 3d6", false),
        ("1d20 + 5", false),
    ];

    for (expression, shows_dropped) in cases {
        for seed in [1, 42, 1337] {
            let results = parse_and_roll_with_rng(expression, &mut create_seeded_rng(seed))
                .unwrap_or_else(|e| panic!("'{expression}' should roll: {e}"));
            let output = results[0].to_string();
            assert_eq!(
                output.contains("~~["),
                shows_dropped,
                "'{expression}' (seed {seed}): {output}"
            );
        }
    }
}

// ============================================================================
// DOUBLE SUCCESS
// ============================================================================

#[test]
fn test_double_success_parsing() {
    // `ds` alone doubles from the target itself. The double tier sits on the
    // far side of the target: at or above it for `t`, at or below it for `tl`.
    let valid = vec![
        ("4d10 t6ds", "TargetWithDoubleSuccess(6, 6)"),
        ("5d10 t7ds10", "TargetWithDoubleSuccess(7, 10)"),
        ("6d10 d2 t4ds6", "TargetWithDoubleSuccess(4, 6)"),
        ("4d6 tl3ds", "TargetLowerWithDoubleSuccess(3, 3)"),
        ("4d6 tl4ds1", "TargetLowerWithDoubleSuccess(4, 1)"),
    ];
    for (expression, expected) in valid {
        let parsed = parser::parse_dice_string(expression)
            .unwrap_or_else(|e| panic!("'{expression}' should parse: {e}"));
        let modifiers = format!("{:?}", parsed[0].modifiers);
        assert!(
            modifiers.contains(expected),
            "'{expression}' should parse as {expected}: {modifiers}"
        );
    }

    for invalid in [
        "4d6 t0ds",
        "4d6 t6ds4",
        "4d6 tl0ds",
        "4d6 tl3ds0",
        "4d6 tl3ds5",
    ] {
        assert_invalid(invalid);
    }
}

#[test]
fn test_double_success_notes() {
    // (expression, fragments one note must carry)
    let cases = vec![
        ("4d10 t6ds", vec!["6+ = 2 successes"]),
        ("6d6 t4ds", vec!["4+ = 2 successes"]),
        ("6d6 tl4ds", vec!["≤4 = 2 successes"]),
        ("6d6 tl4ds1", vec!["≤1 = 2 successes", "≤4 = 1 success"]),
    ];

    for (expression, fragments) in cases {
        let results = parse_and_roll(expression).unwrap();
        assert!(
            results[0].successes.is_some(),
            "'{expression}' should count successes"
        );
        assert!(
            results[0]
                .notes
                .iter()
                .any(|note| fragments.iter().all(|f| note.contains(f))),
            "'{expression}' should note {fragments:?}: {:?}",
            results[0].notes
        );
    }
}

#[cfg(test)]
mod test_issue_94_minimal {
    use dicemaiden_rs::parse_and_roll;

    #[test]
    fn test_issue_94_exact_cases() {
        // Test the exact cases reported in Issue #94

        // Case 1: 1d12+2 t8
        // Before fix: showed 2-3 successes (bug)
        // After fix: should show 0-1 successes (correct)
        let result = parse_and_roll("1d12+2 t8").unwrap();
        let roll = &result[0];

        assert!(roll.successes.is_some(), "Should have success counting");
        let success_count = roll.successes.unwrap();
        assert!(
            (0..=1).contains(&success_count),
            "1d12+2 t8 should have 0-1 successes, got {}",
            success_count
        );

        // Case 2: 10 1d12+6 t8 (roll sets)
        let result = parse_and_roll("10 1d12+6 t8").unwrap();
        assert_eq!(result.len(), 10, "Should have 10 roll sets");

        for (i, roll) in result.iter().enumerate() {
            assert!(
                roll.successes.is_some(),
                "Roll set {} should have success counting",
                i
            );
            let success_count = roll.successes.unwrap();
            assert!(
                (0..=1).contains(&success_count),
                "Roll set {} should have 0-1 successes, got {}",
                i,
                success_count
            );
        }
    }
}

#[test]
fn test_target_failure_order_consistency() {
    // Target, failure, botch and cancel modifiers may be written in any order
    let test_cases = vec![
        ("5d1+4 t4f1", "5d1+4 f1t4", "All succeed, none fail"),
        ("3d1+2 t4f1", "3d1+2 f1t4", "None succeed, none fail"),
        ("3d1+0 t4f1", "3d1+0 f1t4", "None succeed, all fail"),
        (
            "1d1+5 t6f1",
            "1d1+5 f1t6",
            "Pre-target math reaches the target",
        ),
        ("4d1+1 t4f1+1", "4d1+1 f1t4+1", "With post-target modifier"),
        (
            "3d1*2+1 t4f1",
            "3d1*2+1 f1t4",
            "With complex pre-target math",
        ),
        ("3d1+0 t4b1", "3d1+0 b1t4", "All botch and none succeed"),
        ("3d1+5 t4b1", "3d1+5 b1t4", "All succeed and none botch"),
        (
            "3d1+2*2 t6f1b2",
            "3d1+2*2 f1b2t6",
            "Multiple pre-target math",
        ),
        (
            "2d1+3 t4f1b1",
            "2d1+3 b1f1t4",
            "Three target-related modifiers",
        ),
        (
            "4d1+1*3 t5f2",
            "4d1+1*3 f2t5",
            "Different target/failure values",
        ),
        // Real dice, compared at fixed seeds
        ("6d10 f1 t7", "6d10 t7 f1", "World of Darkness"),
        ("4d10 f1 t8 + 2", "4d10 t8 f1 + 2", "WoD with trailing math"),
        ("5d10 f1 t8 c", "5d10 t8 f1 c", "WoD with cancel"),
        ("4wod8", "4d10 f1 t8", "WoD alias and its expansion"),
    ];

    for (expr1, expr2, description) in test_cases {
        assert_modifier_order_equivalent(expr1, expr2, description);
    }
}

#[test]
fn test_pre_target_applied_once() {
    // Verify that pre-target modifiers are applied exactly once
    // This test catches the specific bug that was causing the issue

    let result1 = parse_and_roll("2d1+5 t4f1").unwrap();
    let result2 = parse_and_roll("2d1+5 f1t4").unwrap();

    // Both should have dice values of 6 (1+5), so:
    // - Successes (≥4): 2 dice succeed
    // - Failures (≤1): 0 dice fail
    // - Net: 2 - 0 = 2 successes
    assert_eq!(result1[0].successes.unwrap(), 2);
    assert_eq!(result2[0].successes.unwrap(), 2);
    assert_eq!(result1[0].failures.unwrap(), 0);
    assert_eq!(result2[0].failures.unwrap(), 0);

    // Verify the dice values are correct (should be 6)
    assert!(result1[0].kept_rolls.iter().all(|&v| v == 6));
    assert!(result2[0].kept_rolls.iter().all(|&v| v == 6));
}

/// Assert two orderings of the same target-related modifiers produce identical
/// counts from the same seeds. Modifier order is user-visible syntax, so `t6f1`
/// and `f1t6` must not diverge.
fn assert_modifier_order_equivalent(expr1: &str, expr2: &str, description: &str) {
    for seed in [1, 42, 1337, 99_991] {
        let result1 = parse_and_roll_with_rng(expr1, &mut create_seeded_rng(seed)).unwrap();
        let result2 = parse_and_roll_with_rng(expr2, &mut create_seeded_rng(seed)).unwrap();

        assert_eq!(
            (
                result1[0].successes,
                result1[0].failures,
                result1[0].botches
            ),
            (
                result2[0].successes,
                result2[0].failures,
                result2[0].botches
            ),
            "Order should not affect successes, failures or botches: \
             {expr1} vs {expr2} at seed {seed} ({description})"
        );
    }
}

#[test]
fn test_explosion_count_accuracy() {
    // Every die that hits the threshold explodes once, and the dice it adds
    // never explode again (that is `ie`)
    let test_cases = vec![
        ("4d6 e6", 4, 6),
        ("3d10 e10", 3, 10),
        ("5d8 e8", 5, 8),
        ("8d10 e10", 8, 10), // Several 10s once exploded only the first
    ];

    for (expression, original_count, explode_threshold) in test_cases {
        for _ in 0..50 {
            let result = parse_and_roll(expression).unwrap();
            let roll = &result[0];

            // Count exploding dice in original dice only
            let original_dice =
                &roll.individual_rolls[0..original_count.min(roll.individual_rolls.len())];
            let exploding_dice_count = original_dice
                .iter()
                .filter(|&&die| die >= explode_threshold)
                .count();

            let expected_total = original_count + exploding_dice_count;
            let actual_total = roll.individual_rolls.len();

            assert_eq!(
                actual_total,
                expected_total,
                "Expression '{}': Expected {} total dice ({} original + {} explosions), got {}. Original dice: {:?}",
                expression,
                expected_total,
                original_count,
                exploding_dice_count,
                actual_total,
                original_dice
            );

            // Verify explosion note accuracy
            if exploding_dice_count > 0 {
                let expected_note = if exploding_dice_count == 1 {
                    "1 die exploded".to_string()
                } else {
                    format!("{} dice exploded", exploding_dice_count)
                };

                assert!(
                    roll.notes.contains(&expected_note),
                    "Expression '{}': Expected note '{}', but got notes: {:?}",
                    expression,
                    expected_note,
                    roll.notes
                );
            }
        }
    }
}
