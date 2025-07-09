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
