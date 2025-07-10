// tests/unit_tests.rs - Core dice logic, parsing, and rolling tests
//
// This file contains unit tests for the core dice functionality:
// - Basic dice parsing and validation
// - Mathematical operations and modifiers
// - Core dice modifier behavior (exploding, keep/drop, rerolls)
// - Error handling and input validation

use dicemaiden_rs::{
    dice::{Modifier, parser},
    parse_and_roll,
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
        ("501d6", false, 0, 0),  // Too many dice
        ("1d1001", false, 0, 0), // Too many sides
        ("0d6", false, 0, 0),    // Zero dice
        ("1d0", false, 0, 0),    // Zero sides
        ("-1d6", false, 0, 0),   // Negative dice count
        ("1d-6", false, 0, 0),   // Negative sides
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
fn test_whitespace_handling() {
    // Test various whitespace patterns
    let whitespace_patterns = vec![
        "2d6+3",
        "2d6 + 3",
        "  2d6  +  3  ",
        "\t2d6\t+\t3\t",
        "2d6+3d8+5",
        "2d6 + 3d8 + 5",
    ];

    for pattern in whitespace_patterns {
        assert_valid(pattern);
    }
}

#[test]
fn test_malformed_input_validation() {
    // Test that malformed input is properly rejected
    let invalid_inputs = vec![
        "d",         // Just 'd'
        "1d",        // Missing sides
        "1dd6",      // Double 'd'
        "1d6+",      // Trailing operator
        "1d6++5",    // Double operator
        "1d6 + + 5", // Spaced double operator
        "",          // Empty
        "   ",       // Whitespace only
        "abc",       // No dice at all
        "1d6xyz",    // Invalid modifier suffix
        "dd6",       // Double 'd' at start
    ];

    for invalid_input in invalid_inputs {
        assert_invalid(invalid_input);
    }
}

// ============================================================================
// MATHEMATICAL OPERATIONS
// ============================================================================

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
    ];

    for (expression, expected, description) in math_tests {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(result[0].total, expected, "Left-to-right: {}", description);
    }
}

#[test]
fn test_mathematical_modifiers() {
    // Test basic mathematical operations with validation
    let basic_operations = vec![
        "1d6+5", "1d6 + 5", "2d6-3", "2d6 - 3", "1d6*2", "1d6 * 2", "4d6/2", "4d6 / 2",
    ];

    for operation in basic_operations {
        assert_valid(operation);
    }

    // Test error cases
    assert_invalid("1d6/0");
    assert_invalid("5d10 / 0");
}

#[test]
fn test_dice_operations() {
    // Test dice-to-dice operations with predictable results
    let dice_operations = vec![
        // (expression, expected_total, description)
        ("3d1 * 2d1", 6, "3 * 2 = 6"),
        ("8d1 / 2d1", 4, "8 / 2 = 4"),
        ("200/1d1", 200, "200 / 1 = 200"),
        ("100/2d1", 50, "100 / 2 = 50"),
    ];

    for (expression, expected, description) in dice_operations {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(result[0].total, expected, "{}: {}", expression, description);
    }
}

// ============================================================================
// DICE MODIFIERS
// ============================================================================

#[test]
fn test_exploding_dice() {
    let exploding_patterns = vec!["3d6e", "3d6e6", "4d10ie", "4d10ie8"];

    for pattern in exploding_patterns {
        assert_valid(pattern);
    }

    // Test parsing
    let result = parser::parse_dice_string("4d6 e6").unwrap();
    assert_eq!(result[0].modifiers.len(), 1);
    match &result[0].modifiers[0] {
        Modifier::Explode(Some(6)) => {}
        _ => panic!("Expected Explode(6) modifier"),
    }
}

#[test]
fn test_keep_drop_modifiers() {
    let keep_drop_patterns = vec![
        "4d6k3", "4d6d1", "4d6kl2", "4d6km2", "4d20k2d1", "8d6km3d2", // Complex combinations
    ];

    for pattern in keep_drop_patterns {
        assert_valid(pattern);
    }

    // Test keep middle parsing specifically
    let result = parser::parse_dice_string("6d10 km3").unwrap();
    match &result[0].modifiers[0] {
        Modifier::KeepMiddle(3) => {}
        _ => panic!("Expected KeepMiddle(3) modifier"),
    }
}

#[test]
fn test_reroll_modifiers() {
    let reroll_patterns = vec!["4d6r1", "4d6ir1", "4d6rg5", "4d6irg5"];

    for pattern in reroll_patterns {
        assert_valid(pattern);
    }

    // Test that rg and r are different
    let regular = parser::parse_dice_string("4d6 r2").unwrap();
    let greater = parser::parse_dice_string("4d6 rg2").unwrap();

    match (&regular[0].modifiers[0], &greater[0].modifiers[0]) {
        (Modifier::Reroll(2), Modifier::RerollGreater(2)) => {}
        _ => panic!("Expected different reroll types"),
    }
}

#[test]
fn test_target_system_modifiers() {
    let target_patterns = vec!["6d10t7", "4d6f1", "6d10b", "6d10b1", "6d10tl7"];

    for pattern in target_patterns {
        assert_valid(pattern);
    }
    let modifier_order_tests = vec![
        ("1d20+5 t6", "Modifier before target"),
        ("1d20 t5 + 5", "Modifier after target"),
    ];

    for (expression, description) in modifier_order_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Modifier order test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            results[0].successes.is_some(),
            "Target system should have success counting for '{}'",
            expression
        );

        // Both should produce reasonable success counts
        let success_count = results[0].successes.unwrap();
        assert!(
            success_count >= 0 && success_count <= 25,
            "Success count {} should be reasonable for '{}': {}",
            success_count,
            expression,
            description
        );
    }
}

#[test]
fn test_complex_modifier_combinations() {
    let complex_patterns = vec![
        "10d6 e6 k8 +4",
        "6d10 e6k4d1",
        "4d6 k3d1r1",
        "6d10 e6km4d1",
        "4d6 km3d1r1",
    ];

    for pattern in complex_patterns {
        assert_valid(pattern);
    }
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[test]
fn test_division_by_zero_protection() {
    // Mathematical division by zero
    assert_invalid("1d6/0");
    assert_invalid("5d10 / 0");

    // Valid division that works
    let result = parse_and_roll("10 / 1d1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap()[0].total, 10);
}

#[test]
fn test_modifier_validation() {
    // Invalid modifier values
    assert_invalid("1d6 k0"); // Cannot keep 0 dice
    assert_invalid("1d6 km0"); // Cannot keep 0 dice  
    assert_invalid("1d6 rg0"); // Cannot reroll on 0
    assert_invalid("1d6 e0"); // Cannot explode on 0
}

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
fn test_comments_and_labels_parsing() {
    // Test comment parsing functionality
    let comment_tests = vec![
        ("2d6 ! Fire damage", Some("Fire damage"), "Basic comment"),
        (
            "1d20 + 5 ! Attack roll with sword",
            Some("Attack roll with sword"),
            "Comment with modifier",
        ),
        (
            "4d6 k3 ! Character stats",
            Some("Character stats"),
            "Comment with keep modifier",
        ),
        ("3d6 !", Some(""), "Empty comment"),
        ("1d6 ! ", Some(""), "Whitespace-only comment"),
        (
            "2d6 ! Multi word comment",
            Some("Multi word comment"),
            "Multi-word comment",
        ),
        ("1d6", None, "No comment"),
    ];

    for (expression, expected_comment, description) in comment_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Comment test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert_eq!(
            results[0].comment,
            expected_comment.map(|s| s.to_string()),
            "Comment test '{}': {}",
            expression,
            description
        );
    }

    // Test label parsing functionality
    let label_tests = vec![
        ("(Attack) 1d20 + 5", Some("Attack"), "Basic label"),
        ("(Fire Damage) 8d6", Some("Fire Damage"), "Multi-word label"),
        (
            "( Stealth Check ) 1d20",
            Some("Stealth Check"),
            "Label with spaces",
        ),
        ("() 2d6", Some(""), "Empty label"),
        (
            "(Initiative) 1d20 ! Combat",
            Some("Initiative"),
            "Label with comment",
        ),
        ("1d20 + 5", None, "No label"),
    ];

    for (expression, expected_label, description) in label_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Label test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert_eq!(
            results[0].label,
            expected_label.map(|s| s.to_string()),
            "Label test '{}': {}",
            expression,
            description
        );
    }

    // Test label + comment combinations
    let combined_tests = vec![
        (
            "(Attack) 1d20 + 5 ! With sword",
            Some("Attack"),
            Some("With sword"),
        ),
        ("(Damage) 2d6 ! Fire", Some("Damage"), Some("Fire")),
        ("(Save) 1d20 !", Some("Save"), Some("")),
    ];

    for (expression, expected_label, expected_comment) in combined_tests {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(result[0].label, expected_label.map(|s| s.to_string()));
        assert_eq!(result[0].comment, expected_comment.map(|s| s.to_string()));
    }
}

#[test]
fn test_special_flags_comprehensive() {
    // Test 'nr' (no results) flag
    let nr_tests = vec![
        "nr 2d6",
        "nr 4d6 k3",
        "nr 3d10 e10",
        "nr p 1d20", // Combined with private
        "nr s 2d6",  // Combined with simple (should be compatible)
    ];

    for test in nr_tests {
        let result = parse_and_roll(test);
        assert!(result.is_ok(), "nr flag test '{}' should parse", test);
        let results = result.unwrap();
        assert!(
            results[0].no_results,
            "Should have no_results flag set for '{}'",
            test
        );
    }

    // Test 'ul' (unsorted) flag - check DiceRoll parsing, not RollResult
    let ul_tests = vec![
        "ul 5d6",
        "ul 4d6 k3",
        "ul 10d10 e10",
        "ul p 3d8", // Combined with private
        "ul s 2d6", // Combined with simple
    ];

    for test in ul_tests {
        let parsed = parser::parse_dice_string(test);
        assert!(parsed.is_ok(), "ul flag test '{}' should parse", test);
        let dice = parsed.unwrap();
        assert!(
            dice[0].unsorted,
            "Should have unsorted flag set in DiceRoll for '{}'",
            test
        );

        // Also verify it rolls successfully
        let result = parse_and_roll(test);
        assert!(result.is_ok(), "ul flag test '{}' should roll", test);
    }

    // Test flag combinations
    let flag_combinations = vec![
        ("p s 2d6", true, true, false),      // private + simple
        ("p nr 2d6", true, false, true),     // private + no results
        ("p ul 2d6", true, false, false),    // private + unsorted
        ("s nr 2d6", false, true, true),     // simple + no results
        ("s ul 2d6", false, true, false),    // simple + unsorted
        ("nr ul 2d6", false, false, true),   // no results + unsorted
        ("p s nr 2d6", true, true, true),    // private + simple + no results
        ("p s ul 2d6", true, true, false),   // private + simple + unsorted
        ("p nr ul 2d6", true, false, true),  // private + no results + unsorted
        ("s nr ul 2d6", false, true, true),  // simple + no results + unsorted
        ("p s nr ul 2d6", true, true, true), // all flags
    ];

    for (expression, exp_private, exp_simple, exp_no_results) in flag_combinations {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Flag combination '{}' should parse",
            expression
        );

        let results = result.unwrap();
        let roll = &results[0];

        assert_eq!(
            roll.private, exp_private,
            "Private flag mismatch for '{}'",
            expression
        );
        assert_eq!(
            roll.simple, exp_simple,
            "Simple flag mismatch for '{}'",
            expression
        );
        assert_eq!(
            roll.no_results, exp_no_results,
            "No results flag mismatch for '{}'",
            expression
        );

        // Check unsorted flag on DiceRoll if ul is in expression
        if expression.contains("ul") {
            let parsed = parser::parse_dice_string(expression).unwrap();
            assert!(
                parsed[0].unsorted,
                "Should have unsorted flag in DiceRoll for '{}'",
                expression
            );
        }
    }

    // Test invalid flag combinations or malformed flags
    let invalid_flag_tests = vec![
        "pp 2d6",  // Double flag
        "p2d6",    // No space after flag
        "px 2d6",  // Invalid flag
        "s s 2d6", // Duplicate flag
    ];

    for invalid_test in invalid_flag_tests {
        let result = parse_and_roll(invalid_test);
        // These might parse but shouldn't set flags incorrectly
        if result.is_ok() {
            let results = result.unwrap();
            // Just verify it doesn't crash - specific behavior may vary
            assert!(!results.is_empty());
        }
    }
}

#[test]
fn test_percentile_dice_variants() {
    // Test standalone percentile dice
    let percentile_tests = vec![
        ("d%", 1, 100, "Standard percentile"),
        ("2d%", 2, 100, "Multiple percentile dice"),
        ("d100", 1, 100, "Explicit d100"),
        ("3d100", 3, 100, "Multiple d100"),
    ];

    for (expression, expected_count, expected_sides, description) in percentile_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Percentile test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        // Verify the dice were rolled correctly (check individual_rolls count)
        assert_eq!(
            results[0].individual_rolls.len(),
            expected_count,
            "Expected {} dice for '{}': {}",
            expected_count,
            expression,
            description
        );

        // Verify total is in valid range
        let min_total = expected_count as i32;
        let max_total = expected_count as i32 * expected_sides as i32;
        assert!(
            results[0].total >= min_total && results[0].total <= max_total,
            "Total {} should be between {} and {} for '{}': {}",
            results[0].total,
            min_total,
            max_total,
            expression,
            description
        );
    }

    // Test percentile with modifiers
    let percentile_modifier_tests =
        vec!["d% + 10", "d% - 5", "d% * 2", "d% / 2", "2d% k1", "d% e100"];

    for test in percentile_modifier_tests {
        let result = parse_and_roll(test);
        assert!(
            result.is_ok(),
            "Percentile modifier test '{}' should parse",
            test
        );
    }

    // Test percentile advantage/disadvantage (these should already be tested in game systems
    // but let's verify they work standalone)
    let percentile_advantage_tests = vec![
        ("+d%", "Percentile advantage"),
        ("-d%", "Percentile disadvantage"),
    ];

    for (expression, description) in percentile_advantage_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Percentile advantage test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        // Should have 2 dice (advantage/disadvantage mechanism)
        assert!(
            results[0].individual_rolls.len() >= 2,
            "Advantage/disadvantage should have multiple dice for '{}': {}",
            expression,
            description
        );
    }
}

#[test]
fn test_edge_case_modifier_combinations() {
    // Test some edge cases that might not be covered elsewhere
    let edge_cases = vec![
        "1d6 k1 d0",    // Keep 1, drop 0 (should work)
        "3d6 km1",      // Keep middle 1 (edge case)
        "2d6 k2 d0",    // Keep all, drop none
        "4d6 e6 ie6",   // Both exploding and indefinite exploding
        "3d6 r1 ir1",   // Both reroll and indefinite reroll
        "5d6 rg6 irg6", // Both reroll greater and indefinite reroll greater
        "6d10 t5 tl5",  // Both target and target lower (unusual but might be valid)
        "4d6 f1 b1",    // Both failures and botches
    ];

    for edge_case in edge_cases {
        let result = parse_and_roll(edge_case);
        // These should either work or fail gracefully
        if result.is_err() {
            println!(
                "Edge case '{}' failed as expected: {:?}",
                edge_case,
                result.err()
            );
        } else {
            println!("Edge case '{}' parsed successfully", edge_case);
            let results = result.unwrap();
            assert!(
                !results.is_empty(),
                "Should have results for '{}'",
                edge_case
            );
        }
    }
}

#[test]
fn test_complex_mathematical_expressions() {
    // Test complex mathematical expressions that combine multiple operations
    let complex_math_tests = vec![
        // Test division edge cases that might not be covered
        ("100 / 2d1", 50, "Number divided by dice"),
        ("200 / 4d1", 50, "Number divided by multiple dice"),
        ("1d1 * 2 * 3", 6, "Multiple multiplications"),
        ("1d1 + 2 + 3 + 4", 10, "Multiple additions"),
        ("10d1 / 2 / 5", 1, "Multiple divisions"),
        (
            "1d1 + 2 * 3 / 6",
            1,
            "Mixed operations left-to-right: (1+2)*3/6 = 1",
        ),
    ];

    for (expression, expected, description) in complex_math_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Complex math test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert_eq!(
            results[0].total, expected,
            "Math test '{}': {}",
            expression, description
        );
    }
}

// ============================================================================
// REGRESSION TESTS
// ============================================================================

#[test]
fn test_modifier_order_regression_protection() {
    // This test ensures we maintain backward compatibility

    // Test cases that MUST continue working exactly as before
    let regression_cases = vec![
        // Basic target systems (should work unchanged)
        ("3d6 t4", "Basic target"),
        ("4d10 t8 ie10", "Target with exploding"),
        ("5d6 k3 t5", "Target with keep"),
        // Non-target systems (should work unchanged)
        ("2d6+3", "Regular dice with modifier"),
        ("1d20 + 5", "Spaced modifier"),
        // Game systems that use success counting
        ("ex5", "Exalted alias"),
        ("4cod", "Chronicles of Darkness alias"),
        ("sr6", "Shadowrun alias"),
    ];

    for (expression, description) in regression_cases {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Regression test FAILED for '{}': {} - This indicates our changes broke existing functionality!",
            expression,
            description
        );

        let results = result.unwrap();

        // Basic sanity checks
        assert!(
            !results[0].individual_rolls.is_empty(),
            "Should have rolled dice for '{}'",
            expression
        );

        // Check that totals are reasonable
        assert!(
            results[0].total > 0,
            "Total should be positive for '{}'",
            expression
        );
    }
}
