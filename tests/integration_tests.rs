// tests/integration_tests.rs - End-to-End Integration Tests
//
// This file contains integration tests for complete dice rolling scenarios:
// - End-to-end roll processing
// - Discord formatting and message limits
// - Multi-roll and roll set functionality
// - Complex expression parsing and execution
// - User workflow scenarios

use dicemaiden_rs::{
    format_multiple_results, format_multiple_results_with_limit, help_text, parse_and_roll,
};

// ============================================================================
// ROLL SETS AND MULTIPLE ROLLS
// ============================================================================

#[test]
fn test_roll_sets_comprehensive() {
    // Test roll set creation and labeling
    let result = parse_and_roll("6 4d6").unwrap();
    assert_eq!(result.len(), 6);

    for (i, roll) in result.iter().enumerate() {
        // Verify individual_rolls count since we can't access count directly
        assert!(
            roll.individual_rolls.len() >= 4,
            "Should have at least 4 dice (some may have exploded)"
        );
        assert_eq!(roll.label, Some(format!("Set {}", i + 1)));
        assert!(roll.total >= 4 && roll.total <= 24); // 4d6 range
    }

    // Test minimum and maximum roll set sizes
    let result = parse_and_roll("2 1d6").unwrap();
    assert_eq!(result.len(), 2);

    let result = parse_and_roll("20 1d6").unwrap();
    assert_eq!(result.len(), 20);
}

#[test]
fn test_roll_sets_with_complex_expressions() {
    // Test roll sets with mathematical operations
    let result = parse_and_roll("3 100/2d1").unwrap();
    assert_eq!(result.len(), 3);

    for roll in &result {
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        assert_eq!(roll.total, 50); // 100/2 = 50
    }

    // Test roll sets with game systems
    let result = parse_and_roll("4 4cod").unwrap();
    assert_eq!(result.len(), 4);

    for roll in &result {
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        assert!(roll.successes.is_some()); // CoD has success counting
    }
}

#[test]
fn test_roll_sets_with_flags() {
    // Test roll sets with private flag
    let result = parse_and_roll("p 3 2d6").unwrap();
    assert_eq!(result.len(), 3);

    for roll in &result {
        assert!(roll.private, "Each roll should be private");
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
    }

    // Test roll sets with simple flag
    let result = parse_and_roll("s 2 1d20").unwrap();
    assert_eq!(result.len(), 2);

    for roll in &result {
        assert!(roll.simple, "Each roll should be simple");
    }
}

#[test]
fn test_semicolon_separated_rolls() {
    // Test multiple separate rolls with semicolons
    let result = parse_and_roll("1d20; 2d6; 1d4").unwrap();
    assert_eq!(result.len(), 3);

    // Each should have different dice configurations based on individual_rolls
    assert_eq!(result[0].individual_rolls.len(), 1); // 1d20
    assert_eq!(result[1].individual_rolls.len(), 2); // 2d6
    assert_eq!(result[2].individual_rolls.len(), 1); // 1d4

    // Should have original expressions stored
    assert!(result.iter().any(|r| r.original_expression.is_some()));
}

#[test]
fn test_roll_sets_vs_mathematical_expressions() {
    // Test disambiguation between roll sets and math

    // This should be 4 roll sets of (20/2d1)
    let result = parse_and_roll("4 20/2d1").unwrap();
    assert_eq!(result.len(), 4);
    for roll in &result {
        assert_eq!(roll.total, 10); // 20/2 = 10
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
    }

    // This should be single math expression (20 / 2d1)
    let result = parse_and_roll("20 / 2d1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].total, 10);
    assert!(result[0].label.is_none());

    // This should be single math expression without spaces
    let result = parse_and_roll("20/2d1").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].total, 10);
    assert!(result[0].label.is_none());
}

// ============================================================================
// DISCORD FORMATTING AND LIMITS
// ============================================================================

#[test]
fn test_result_formatting() {
    // Test basic result formatting
    let result = parse_and_roll("2d6").unwrap();
    let formatted = format_multiple_results(&result);

    assert!(formatted.contains("**")); // Should have bold totals
    assert!(formatted.contains("`[")); // Should have dice display

    // Test that total is shown
    let total = result[0].total;
    assert!(formatted.contains(&format!("**{}**", total)));
}

#[test]
fn test_roll_set_formatting() {
    // Test roll set formatting with totals
    let result = parse_and_roll("3 2d6").unwrap();
    let formatted = format_multiple_results(&result);

    // Should show individual sets
    assert!(formatted.contains("Set 1"));
    assert!(formatted.contains("Set 2"));
    assert!(formatted.contains("Set 3"));

    // Should show combined total
    assert!(formatted.contains("**Total:"));

    let expected_total: i32 = result.iter().map(|r| r.total).sum();
    assert!(formatted.contains(&format!("Total: {}**", expected_total)));
}

#[test]
fn test_semicolon_roll_formatting() {
    // Test semicolon-separated roll formatting
    let result = parse_and_roll("1d20; 2d6").unwrap();
    let formatted = format_multiple_results(&result);

    // Should show individual requests (without set labels)
    assert!(!formatted.contains("Set 1"));
    assert!(!formatted.contains("**Total:"));

    // Should show both results
    assert!(formatted.contains("**"));
}

#[test]
fn test_message_length_limits() {
    // Test Discord 2000 character limit handling

    // Test large roll that might exceed limit
    let result = parse_and_roll("20 50d6").unwrap();
    let formatted = format_multiple_results_with_limit(&result);

    assert!(
        formatted.len() <= 2000,
        "Formatted result should fit Discord limit: {} chars",
        formatted.len()
    );

    // Test extremely large single roll
    let result = parse_and_roll("500d6").unwrap();
    let formatted = format_multiple_results_with_limit(&result);

    assert!(
        formatted.len() <= 2000,
        "Large single roll should fit Discord limit: {} chars",
        formatted.len()
    );
}

#[test]
fn test_private_roll_formatting() {
    // Test private roll indication
    let result = parse_and_roll("p 2d6").unwrap();
    assert!(result[0].private);

    // Private rolls should be handled differently in the bot
    // (This is more of a documentation test since formatting happens in commands)
}

#[test]
fn test_simple_roll_formatting() {
    // Test simple output mode
    let result = parse_and_roll("s 2d6").unwrap();
    assert!(result[0].simple);

    let _formatted = result[0].to_string();
    // Simple mode should show result but not dice breakdown
    // This depends on the Display implementation
}

// ============================================================================
// COMPLEX EXPRESSION PARSING
// ============================================================================

#[test]
fn test_complex_expressions() {
    // Test complex expressions that combine multiple features
    let complex_expressions = vec![
        "10d6 e6 k8 +4",                     // Exploding, keep, add
        "6d10 t7 f1 b1 ie10 + 5d6 e6 - 2d4", // Target system with math
        "4d6 k3 + 2d6 * 3 - 1d4",            // Keep with complex math
        "3 sw8 + 5",                         // Roll sets with game system and modifier
        "p s 5 4d6 k3",                      // Flags + roll sets + modifiers
        "wng 6d6 + 2",                       // Game system with modifier (simplified)
    ];

    for expression in complex_expressions {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Complex expression '{}' should parse successfully: {:?}",
            expression,
            result.err()
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for '{}'",
            expression
        );

        // Verify basic integrity
        for roll in &results {
            assert!(roll.total != 0 || roll.successes.is_some());
        }
    }
}

#[test]
fn test_advantage_disadvantage_in_expressions() {
    // Test advantage/disadvantage patterns in various contexts
    let advantage_patterns = vec![
        "+d20",       // Simple advantage
        "-d20",       // Simple disadvantage
        "+d20 + 5",   // Advantage with modifier
        "-d% - 10",   // Disadvantage percentile with modifier
        "3 +d20",     // Roll sets with advantage
        "2 -d% + 50", // Roll sets with disadvantage and modifier
    ];

    for pattern in advantage_patterns {
        let result = parse_and_roll(pattern);
        assert!(
            result.is_ok(),
            "Advantage pattern '{}' should work: {:?}",
            pattern,
            result.err()
        );
    }
}

#[test]
fn test_expression_edge_cases() {
    // Test edge cases in expression parsing
    let edge_cases = vec![
        "1d1",                   // Minimum dice
        "500d1000",              // Maximum dice
        "20 1d6",                // Maximum roll sets
        "1d6;1d6;1d6;1d6",       // Maximum semicolon rolls
        "1d6 + 1d6 + 1d6 + 1d6", // Multiple dice additions
        "100/d%",                // Number divided by dice
        "+d20 + d10 + 5",        // Complex advantage expression
    ];

    for case in edge_cases {
        let result = parse_and_roll(case);
        assert!(
            result.is_ok(),
            "Edge case '{}' should work: {:?}",
            case,
            result.err()
        );
    }
}

// ============================================================================
// USER WORKFLOW SCENARIOS
// ============================================================================

#[test]
fn test_common_rpg_scenarios() {
    // Test typical RPG use cases
    let rpg_scenarios = vec![
        // D&D scenarios
        ("dndstats", "Character creation"),
        ("+d20 + 5", "Attack roll with advantage"),
        ("-d20 - 2", "Saving throw with disadvantage"),
        ("8d6", "Fireball damage"),
        // World of Darkness scenarios
        ("6cod", "Skill roll"),
        ("4wod8", "Classic WoD difficulty 8"),
        // Savage Worlds scenarios
        ("sw8", "Trait test"),
        ("sw10 + 2", "Modified trait test"),
        // Shadowrun scenarios
        ("sr12", "Skill test with 12 dice"),
        // Generic scenarios
        ("3 4d6 k3", "Multiple character stats"),
        ("1d20; 1d8 + 3", "Attack and damage"),
        ("4d6 k3 ; 4d6 k3 ; 4d6 k3", "Multiple ability scores"),
    ];

    for (expression, description) in rpg_scenarios {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "RPG scenario '{}' ({}) should work: {:?}",
            expression,
            description,
            result.err()
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for '{}'",
            expression
        );

        // Basic sanity checks - verify roll has some meaningful output
        for roll in &results {
            // A meaningful result is either:
            // 1. Any total (including 0 or negative from modifiers)
            // 2. Success counting system (successes field set)
            // 3. Special damage system (godbound_damage set)
            // 4. Has individual dice rolls (basic validation that something was rolled)
            assert!(
                !roll.individual_rolls.is_empty()
                    || roll.successes.is_some()
                    || roll.godbound_damage.is_some(),
                "Roll should have some meaningful result - either dice rolls, successes, or special damage"
            );
        }
    }
}

#[test]
fn test_help_system_integration() {
    // Test that help text generation works
    let basic_help = help_text::generate_basic_help();
    assert!(basic_help.contains("Dice Maiden"));
    assert!(basic_help.contains("2d6 + 3d10"));
    assert!(basic_help.len() > 100);

    let alias_help = help_text::generate_alias_help();
    assert!(alias_help.contains("Game System Aliases"));
    assert!(alias_help.contains("Savage Worlds"));
    assert!(alias_help.len() > 100);

    let system_help = help_text::generate_system_help();
    assert!(system_help.contains("Game System Examples"));
    assert!(system_help.contains("Fudge/FATE"));
    assert!(system_help.len() > 100);
}

#[test]
fn test_error_scenarios() {
    // Test error handling in realistic scenarios
    let error_scenarios = vec![
        ("", "Empty input"),
        ("invalid", "Invalid dice expression"),
        ("1d6/0", "Division by zero"),
        ("1d6 + + 5", "Double operator"),
        ("1d6xyz", "Invalid modifier"),
        ("1000d1000", "Too many dice"),
        ("1d0", "Zero-sided dice"),
        ("21 1d6", "Too many roll sets"),
        ("1d6;1d6;1d6;1d6;1d6", "Too many semicolon rolls"),
    ];

    for (expression, description) in error_scenarios {
        let result = parse_and_roll(expression);
        assert!(
            result.is_err(),
            "Error scenario '{}' ({}) should fail but succeeded",
            expression,
            description
        );
    }
}

#[test]
fn test_performance_scenarios() {
    // Test performance with realistic loads
    use std::time::Instant;

    let performance_tests = vec![
        ("500d6", "Maximum dice"),
        ("20 10d10", "Maximum roll sets"),
        ("100d6 e6 k50", "Complex modifiers"),
        ("10d10 t7 ie10 + 5d6 e6 - 2d4", "Complex expression"),
    ];

    for (expression, description) in performance_tests {
        let start = Instant::now();
        let result = parse_and_roll(expression);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Performance test '{}' ({}) should succeed",
            expression,
            description
        );

        assert!(
            duration.as_millis() < 1000,
            "Performance test '{}' took too long: {}ms",
            expression,
            duration.as_millis()
        );
    }
}

// ============================================================================
// REGRESSION TESTS
// ============================================================================

#[test]
fn test_critical_regressions() {
    // Test for specific bugs that were previously fixed
    // These should NEVER be removed without careful analysis

    // Regression: k2d1 parsing was failing with "Invalid keep value in 'k2d1'"
    let result = parse_and_roll("4d20 k2d1").unwrap();
    assert_eq!(result.len(), 1);
    // We can verify the behavior worked by checking the result is reasonable
    assert!(result[0].total >= 2 && result[0].total <= 40); // Should have kept 2 dice, dropped 1

    // Regression: km wasn't being parsed before k
    let result = parse_and_roll("6d10km3").unwrap();
    // Verify the roll worked and produced a reasonable result
    assert!(result[0].total >= 3 && result[0].total <= 30); // Should keep 3 middle dice

    // Regression: Roll sets with advantage weren't working
    let result = parse_and_roll("3 +d20").unwrap();
    assert_eq!(result.len(), 3);
    for roll in &result {
        assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        assert!(roll.total >= 1 && roll.total <= 20);
    }

    // Regression: Left-to-right math evaluation
    let result = parse_and_roll("1d1 + 2 * 3").unwrap();
    assert_eq!(result[0].total, 9); // Should be (1+2)*3=9, not 1+(2*3)=7

    // Regression: Division by dice was broken
    let result = parse_and_roll("200/1d1").unwrap();
    assert_eq!(result[0].total, 200); // 200/1 = 200
}

#[test]
fn test_format_regression_cases() {
    // Test formatting edge cases that caused issues

    // Very large totals should format correctly
    let result = parse_and_roll("500d1").unwrap();
    let formatted = format_multiple_results(&result);
    assert!(formatted.contains("**500**"));

    // Roll sets should show totals correctly
    let result = parse_and_roll("3 10d1").unwrap();
    let formatted = format_multiple_results(&result);
    assert!(formatted.contains("**Total: 30**"));

    // Private rolls should be identified
    let result = parse_and_roll("p 1d6").unwrap();
    assert!(result[0].private);

    // Simple rolls should be identified
    let result = parse_and_roll("s 1d6").unwrap();
    assert!(result[0].simple);
}
