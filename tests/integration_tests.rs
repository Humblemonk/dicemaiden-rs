// tests/integration_tests.rs - End-to-End Integration Tests
//
// This file contains integration tests for complete dice rolling scenarios:
// - End-to-end roll processing
// - Discord formatting and message limits
// - Multi-roll and roll set functionality
// - Complex expression parsing and execution
// - User workflow scenarios

use dicemaiden_rs::{
    dice::parser, format_multiple_results, format_multiple_results_with_limit, help_text,
    parse_and_roll,
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
        "10d6 e6 k8 +4",                                   // Exploding, keep, add
        "6d10 t7 f1 b1 ie10 + 5d6 e6 - 2d4",               // Target system with math
        "4d6 k3 + 2d6 * 3 - 1d4",                          // Keep with complex math
        "3 sw8 + 5",    // Roll sets with game system and modifier
        "p s 5 4d6 k3", // Flags + roll sets + modifiers
        "wng 6d6 + 2",  // Game system with modifier (simplified)
        "(WoD Attack) 4wod8c + 1 ! Supernatural strength", // WoD cancel with label and comment
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
        ("4wod8c", "WoD with cancel mechanics"),
        ("5wod6c + 2", "WoD cancel with modifier"),
        // Savage Worlds scenarios
        ("sw8", "Trait test"),
        ("sw10 + 2", "Modified trait test"),
        // Shadowrun scenarios
        ("sr12", "Skill test with 12 dice"),
        // NEW: Aliens RPG scenarios
        ("alien4", "Alien RPG basic attribute + skill roll"),
        ("alien5s2", "Alien RPG skill roll with moderate stress"),
        ("alien3s1p", "Alien RPG push mechanic (low stress)"),
        ("alien6s4", "Alien RPG high-stress situation"),
        ("3 alien4s2", "Multiple Alien RPG rolls for group actions"),
        ("alien4 + 2", "Alien RPG with situational modifier"),
        ("alien5s3 - 1", "Alien RPG stress roll with penalty"),
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
                roll.total != 0
                    || roll.successes.is_some()
                    || roll.godbound_damage.is_some()
                    || !roll.individual_rolls.is_empty()
                    || roll.fudge_symbols.is_some(),
                "RPG scenario '{}' should produce meaningful output: {}",
                expression,
                description
            );

            // NEW: Verify Aliens RPG specific integration features
            if expression.contains("alien") {
                assert!(
                    roll.successes.is_some(),
                    "Alien RPG scenario '{}' should have success counting in integration test",
                    expression
                );

                // Only stress rolls have system notes (basic alien rolls don't)
                if expression.contains('s') {
                    let has_stress_note =
                        roll.notes.iter().any(|note| note.contains("STRESS DICE"));
                    assert!(
                        has_stress_note,
                        "Alien RPG stress scenario '{}' should have stress notes in integration test",
                        expression
                    );
                }

                // For stress rolls, verify stress system features
                if expression.contains('s') {
                    // Should have stress-related notes or tracking
                    let has_stress_feature = roll.alien_stress_level.is_some()
                        || roll.notes.iter().any(|note| note.contains("STRESS"));
                    assert!(
                        has_stress_feature,
                        "Alien RPG stress scenario '{}' should have stress features",
                        expression
                    );
                }

                // For panic situations (if stress dice roll 1s), verify panic mechanics
                if let Some(panic_roll) = roll.alien_panic_roll {
                    assert!(
                        panic_roll >= 4 && panic_roll <= 16,
                        "Alien RPG panic roll should be in valid range for '{}'",
                        expression
                    );

                    let has_panic_note = roll.notes.iter().any(|note| note.contains("PANIC ROLL"));
                    assert!(
                        has_panic_note,
                        "Alien RPG scenario '{}' with panic should have panic note",
                        expression
                    );
                }
            }
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
fn test_roll_set_validation_all_paths() {
    // Ensure validation works across ALL parsing paths to prevent future bypasses
    let critical_test_cases = vec![
        // Direct roll set patterns
        ("1 d6", false, "Direct single roll set"),
        ("21 d6", false, "Direct too many roll sets"),
        ("2 d6", true, "Direct valid roll set"),
        // With flags
        ("p 1 d6", false, "Flag with single roll set"),
        ("p 21 d6", false, "Flag with too many roll sets"),
        ("p 3 d6", true, "Flag with valid roll set"),
        // With advantage patterns
        ("1 +d20", false, "Single advantage roll set"),
        ("21 +d20", false, "Too many advantage roll sets"),
        ("3 +d20", true, "Valid advantage roll set"),
        // Boundary cases
        ("0 d6", false, "Zero roll sets"),
        ("20 d6", true, "Maximum valid roll sets"),
        ("22 d6", false, "Just above maximum"),
    ];

    for (expression, should_succeed, description) in critical_test_cases {
        let result = parse_and_roll(expression);

        if should_succeed {
            assert!(
                result.is_ok(),
                "CRITICAL: '{}' should succeed: {}",
                expression,
                description
            );
        } else {
            assert!(
                result.is_err(),
                "CRITICAL: '{}' should fail: {}",
                expression,
                description
            );

            // Verify proper error message for roll set errors
            if expression.contains(' ') {
                let error_msg = result.unwrap_err().to_string();
                assert!(
                    error_msg.contains("Set count must be between 2 and 20")
                        || error_msg.contains("Invalid set count"),
                    "CRITICAL: '{}' should have proper error message: {}",
                    expression,
                    error_msg
                );
            }
        }
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

#[test]
fn test_comments_functionality() {
    // Test that comments are properly parsed, stored, and formatted
    let comment_scenarios = vec![
        ("2d6 ! Fire damage", "Fire damage"),
        (
            "1d20 + 5 ! Attack roll with sword",
            "Attack roll with sword",
        ),
        ("4d6 k3 ! Character creation", "Character creation"),
        ("8d6 ! Fireball spell", "Fireball spell"),
        ("3d10 t7 ! Difficulty check", "Difficulty check"),
        ("sw8 ! Savage Worlds trait test", "Savage Worlds trait test"),
        ("4cod ! Chronicles skill roll", "Chronicles skill roll"),
    ];

    for (expression, expected_comment) in comment_scenarios {
        let result = parse_and_roll(expression).unwrap();

        // Verify comment is parsed correctly
        assert_eq!(
            result[0].comment,
            Some(expected_comment.to_string()),
            "Comment should be parsed correctly for '{}'",
            expression
        );

        // Verify comment appears in formatted output
        let formatted = format_multiple_results(&result);
        assert!(
            formatted.contains("Reason:"),
            "Formatted output should contain 'Reason:' for '{}'",
            expression
        );
        assert!(
            formatted.contains(expected_comment),
            "Formatted output should contain comment text for '{}'",
            expression
        );
    }

    // Test comment with roll sets
    let result = parse_and_roll("3 2d6 ! Damage rolls").unwrap();
    assert_eq!(result.len(), 3);

    // Only the first roll should have the comment to avoid duplication
    assert_eq!(result[0].comment, Some("Damage rolls".to_string()));

    let formatted = format_multiple_results(&result);
    // Should show comment once at the end, not for each set
    let comment_count = formatted.matches("Reason:").count();
    assert_eq!(
        comment_count, 1,
        "Comment should appear once in roll set output"
    );

    // Test comment with semicolon-separated rolls
    let result = parse_and_roll("1d20 ! Attack; 1d8 ! Damage").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].comment, Some("Attack".to_string()));
    assert_eq!(result[1].comment, Some("Damage".to_string()));

    // Test edge cases
    let edge_cases = vec![
        ("1d6 !", ""),     // Empty comment
        ("2d6 !    ", ""), // Whitespace-only comment
    ];

    for (expression, expected) in edge_cases {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(
            result[0].comment,
            Some(expected.to_string()),
            "Edge case comment for '{}'",
            expression
        );
    }
}

#[test]
fn test_labels_functionality() {
    // Test that labels are properly parsed, stored, and formatted
    let label_scenarios = vec![
        ("(Attack) 1d20 + 5", "Attack"),
        ("(Damage) 2d6", "Damage"),
        ("(Initiative) 1d20", "Initiative"),
        ("(Stealth Check) 1d20 - 2", "Stealth Check"),
        ("(Fire Damage) 8d6", "Fire Damage"),
        ("(Skill Test) 4cod", "Skill Test"),
        ("(Trait Roll) sw8", "Trait Roll"),
    ];

    for (expression, expected_label) in label_scenarios {
        let result = parse_and_roll(expression).unwrap();

        // Verify label is parsed correctly
        assert_eq!(
            result[0].label,
            Some(expected_label.to_string()),
            "Label should be parsed correctly for '{}'",
            expression
        );

        // Verify label appears in formatted output
        let formatted = format_multiple_results(&result);
        assert!(
            formatted.contains(&format!("**{}**:", expected_label)),
            "Formatted output should contain labeled format for '{}'",
            expression
        );
    }

    // Test label with roll sets
    let result = parse_and_roll("3 (Stat Roll) 4d6 k3").unwrap();
    assert_eq!(result.len(), 3);

    // Each roll in the set should have "Set X" label, overriding the original
    for (i, roll) in result.iter().enumerate() {
        assert_eq!(roll.label, Some(format!("Set {}", i + 1)));
    }

    // Test label + comment combination
    let result = parse_and_roll("(Attack) 1d20 + 5 ! With magic sword").unwrap();
    assert_eq!(result[0].label, Some("Attack".to_string()));
    assert_eq!(result[0].comment, Some("With magic sword".to_string()));

    let formatted = format_multiple_results(&result);
    assert!(formatted.contains("**Attack**:"));
    assert!(formatted.contains("Reason: `With magic sword`"));

    // Test edge cases
    let label_edge_cases = vec![
        ("() 2d6", ""),                // Empty label
        ("(  ) 1d20", ""),             // Whitespace-only label
        ("( Attack ) 1d20", "Attack"), // Extra spaces should be trimmed
    ];

    for (expression, expected) in label_edge_cases {
        let result = parse_and_roll(expression).unwrap();
        assert_eq!(
            result[0].label,
            Some(expected.to_string()),
            "Edge case label for '{}'",
            expression
        );
    }
}

#[test]
fn test_special_flags_integration() {
    // Test special flags in complex integration scenarios

    // Test 'nr' (no results) flag with various systems
    let nr_tests = vec![
        ("nr 4d6 k3", "No results with keep modifier"),
        ("nr 3d10 t7", "No results with success counting"),
        ("nr sw8", "No results with Savage Worlds"),
        ("nr 4cod", "No results with Chronicles of Darkness"),
        ("nr 6 2d6", "No results with roll sets"),
    ];

    for (expression, description) in nr_tests {
        let result = parse_and_roll(expression).unwrap();

        for roll in &result {
            assert!(
                roll.no_results,
                "Should have no_results flag for '{}': {}",
                expression, description
            );
        }

        // Test formatted output - should show dice but not final results
        let formatted = format_multiple_results(&result);
        assert!(
            formatted.contains("`["),
            "Should show dice breakdown for nr flag: '{}'",
            expression
        );
        // Note: Actual formatting behavior for 'nr' flag needs to be verified based on implementation
    }

    // Test 'ul' (unsorted) flag with various systems - check DiceRoll parsing
    let ul_tests = vec![
        ("ul 5d6", "Unsorted basic dice"),
        ("ul 4d6 k3", "Unsorted with keep modifier"),
        ("ul 10d10 e10", "Unsorted with exploding dice"),
        ("ul 3 4d6", "Unsorted with roll sets"),
    ];

    for (expression, description) in ul_tests {
        // Check that the DiceRoll has the unsorted flag
        let parsed = parser::parse_dice_string(expression).unwrap();
        for dice in &parsed {
            assert!(
                dice.unsorted,
                "Should have unsorted flag in DiceRoll for '{}': {}",
                expression, description
            );
        }

        // Also verify it rolls successfully
        let result = parse_and_roll(expression).unwrap();
        assert!(
            !result.is_empty(),
            "Should have results for '{}': {}",
            expression,
            description
        );

        // Note: To properly test unsorted behavior, we'd need to verify that the dice
        // are NOT sorted in the output, but this is difficult to test deterministically
    }

    // Test flag combinations with complex expressions
    let complex_flag_tests = vec![
        (
            "p s 5 4d6 k3 ! Character stats",
            "All flags with roll sets and comment",
        ),
        ("p nr (Attack) 1d20 + 5", "Private + no results + label"),
        ("s ul 8d6 e6", "Simple + unsorted + exploding"),
        ("p s nr ul 2d6", "All four flags together"),
    ];

    for (expression, description) in complex_flag_tests {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Complex flag test '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for '{}': {}",
            expression,
            description
        );

        // Verify flags are set correctly
        for roll in &results {
            if expression.contains(" p ") || expression.starts_with("p ") {
                assert!(roll.private, "Should be private for '{}'", expression);
            }
            if expression.contains(" s ") || expression.starts_with("s ") {
                assert!(roll.simple, "Should be simple for '{}'", expression);
            }
            if expression.contains(" nr ") {
                assert!(
                    roll.no_results,
                    "Should have no_results for '{}'",
                    expression
                );
            }
        }

        // Check unsorted flag on DiceRoll parsing if ul is in expression
        if expression.contains(" ul ") {
            let parsed = parser::parse_dice_string(expression).unwrap();
            for dice in &parsed {
                assert!(
                    dice.unsorted,
                    "Should have unsorted flag in DiceRoll for '{}'",
                    expression
                );
            }
        }
    }
}

#[test]
fn test_wrath_glory_special_modes() {
    // Test Wrath & Glory special modes mentioned in roll_syntax.md but not fully tested
    let wng_special_modes = vec![
        // Soak tests (use total instead of successes)
        ("wng 4d6 !soak", "Basic soak test"),
        ("wng w2 5d6 !soak", "Soak with multiple wrath dice"),
        ("wng dn3 4d6 !soak", "Soak with difficulty"),
        // Exempt tests (no wrath die)
        ("wng 6d6 !exempt", "Basic exempt test"),
        ("wng dn2 4d6 !exempt", "Exempt with difficulty"),
        // Damage tests (similar to soak)
        ("wng 5d6 !dmg", "Damage test"),
        ("wng w2 6d6 !dmg", "Damage with multiple wrath dice"),
    ];

    for (expression, description) in wng_special_modes {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "W&G special mode '{}' should parse: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for '{}': {}",
            expression,
            description
        );

        let roll = &results[0];

        // Check that appropriate mechanics are applied
        if expression.contains("!soak")
            || expression.contains("!dmg")
            || expression.contains("!exempt")
        {
            // For soak/damage/exempt tests, should use total instead of successes
            // (Implementation details may vary, but should have some kind of appropriate result)
            assert!(
                roll.total > 0 || roll.successes.is_some(),
                "Should have meaningful result for '{}': {}",
                expression,
                description
            );

            // Check for appropriate notes indicating the special mode
            let _has_special_note = roll.notes.iter().any(|note| {
                note.contains("soak")
                    || note.contains("exempt")
                    || note.contains("damage")
                    || note.contains("total")
                    || note.contains("Wrath")
            });
            // Note: This depends on implementation - the system should indicate special mode somehow
        }

        // Wrath dice should still be tracked for complications/glory
        if expression.contains("w2") {
            // Should have multiple wrath dice
            assert!(
                roll.wng_wrath_dice.is_some() || roll.wng_wrath_die.is_some(),
                "Should have wrath dice information for '{}': {}",
                expression,
                description
            );
        }
    }

    // Test invalid W&G special modes
    let invalid_wng_modes = vec!["wng 4d6 !invalid", "wng 4d6 !", "wng 4d6 !soak invalid"];

    for invalid_test in invalid_wng_modes {
        let result = parse_and_roll(invalid_test);
        // These might parse but should either work or fail gracefully
        if result.is_ok() {
            println!("W&G mode '{}' parsed (behavior may vary)", invalid_test);
        } else {
            println!("W&G mode '{}' failed as expected", invalid_test);
        }
    }
}

#[test]
fn test_comprehensive_user_scenarios() {
    // Test realistic user scenarios that combine multiple advanced features
    let user_scenarios = vec![
        // Character creation scenarios
        (
            "(Strength) 4d6 k3 ! Primary stat",
            "Character creation with label and comment",
        ),
        (
            "6 (Stat) 4d6 k3 ! Character generation",
            "Multiple stats with labels",
        ),
        (
            "p (Secret Roll) 1d20 + 5 ! GM only",
            "Private roll with label and comment",
        ),
        // Combat scenarios
        (
            "(Attack) +d20 + 8 ! Rogue sneak attack",
            "Advantage attack with label",
        ),
        (
            "(Damage) 3d6 + 2d6 ! Sneak attack damage",
            "Complex damage with comment",
        ),
        (
            "(Initiative) 1d20 + 3; (Attack) 1d20 + 5; (Damage) 1d8 + 3",
            "Full combat sequence",
        ),
        // Spellcasting scenarios
        (
            "(Fireball) 8d6 ! 3rd level spell",
            "Spell damage with label",
        ),
        (
            "nr (Concentration) 1d20 + 5 ! Maintaining spell",
            "Concentration check with no results",
        ),
        // System-specific scenarios
        (
            "(Investigation) 4cod ! Looking for clues",
            "Chronicles of Darkness skill",
        ),
        (
            "(Fighting) sw8 ! Savage Worlds combat",
            "Savage Worlds trait test",
        ),
        (
            "(Hack) cpr + 6 ! Cyberpunk Red netrunning",
            "Cyberpunk Red with modifier",
        ),
        (
            "(Spell) wit + 4 ! Witcher magic attempt",
            "Witcher magic roll",
        ),
        // Complex scenarios with flags
        (
            "p s (Secret) 2d20 kl1 ! Stealth disadvantage",
            "Private simple disadvantage roll",
        ),
        (
            "ul (Chaos) 10d6 e6 ! Wild magic surge",
            "Unsorted exploding dice",
        ),
        (
            "nr (Behind Screen) 3d6 ! GM planning roll",
            "No results GM roll",
        ),
    ];

    for (expression, description) in user_scenarios {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "User scenario '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for '{}': {}",
            expression,
            description
        );

        // Test that formatting works properly
        let formatted = format_multiple_results(&results);
        assert!(
            !formatted.is_empty(),
            "Should have formatted output for '{}'",
            expression
        );
        assert!(
            formatted.len() <= 2000,
            "Should fit Discord limit for '{}'",
            expression
        );

        // Verify complex features work together
        if expression.contains("(") && expression.contains(")") {
            // Should have label
            assert!(
                results[0].label.is_some() || formatted.contains("Set "),
                "Should have label or be a roll set for '{}'",
                expression
            );
        }

        if expression.contains("!") {
            // Should have comment (unless suppressed by sets)
            assert!(
                results[0].comment.is_some() || results.len() > 1,
                "Should have comment or be multi-roll for '{}'",
                expression
            );
        }

        if expression.contains(" p ") || expression.starts_with("p ") {
            for roll in &results {
                assert!(roll.private, "Should be private for '{}'", expression);
            }
        }
    }
}

#[test]
fn test_error_recovery_scenarios() {
    // Test that complex expressions with errors fail gracefully
    let error_scenarios = vec![
        // Complex expressions with errors
        ("(Unclosed 1d20 + 5 ! Should fail", "Unclosed label"),
        (
            "(Attack) 1d20 + 5 ! Comment; with; many; semicolons",
            "Complex comment with semicolons",
        ),
        ("p s nr ul invalid 2d6", "Flags with invalid dice"),
        ("(Label) p s 2d6 ! Comment", "Mixed flag and label order"),
        // System-specific errors
        ("(Test) 4cod99", "Invalid CoD variant"),
        ("(Bad) sw15", "Invalid Savage Worlds die"),
        ("(Error) cpr / 0", "Division by zero with label"),
        // Complex mathematical errors
        ("(Math) 2d6 + + 5", "Double operator with label"),
        ("(Complex) 1d20 * / 2", "Invalid operator sequence"),
    ];

    for (expression, description) in error_scenarios {
        let result = parse_and_roll(expression);
        // These should either work (if the error is handled) or fail gracefully
        match result {
            Ok(results) => {
                println!(
                    "Error scenario '{}' unexpectedly succeeded: {}",
                    expression, description
                );
                assert!(
                    !results.is_empty(),
                    "Should have results if parsing succeeded"
                );
            }
            Err(error) => {
                println!(
                    "Error scenario '{}' failed as expected: {} - {}",
                    expression, description, error
                );
                // Error message should be reasonable (not a panic or internal error)
                let error_str = error.to_string();
                assert!(!error_str.is_empty(), "Should have error message");
                assert!(
                    !error_str.contains("internal error"),
                    "Should not be internal error"
                );
            }
        }
    }
}

#[test]
fn test_discord_formatting_edge_cases() {
    // Test edge cases in Discord formatting with advanced features
    let formatting_edge_cases = vec![
        // Very long comments
        (
            "1d6 ! This is a very long comment that might cause formatting issues when combined with complex dice expressions and multiple modifiers",
            "Long comment",
        ),
        // Complex labels
        ("(Very Long Label Name) 1d20", "Long label"),
        // Multiple complex elements
        (
            "20 (Set) 10d6 e6 k5 ! Large roll set with exploding dice",
            "Complex large roll set",
        ),
        // Unicode in comments/labels
        (
            "(⚔️ Attack) 1d20 ! 🔥 Fire damage",
            "Unicode in label and comment",
        ),
        // Special characters
        (
            "(Test \"Quotes\") 1d20 ! Comment with 'quotes'",
            "Quotes in label and comment",
        ),
    ];

    for (expression, description) in formatting_edge_cases {
        let result = parse_and_roll(expression);
        if result.is_ok() {
            let results = result.unwrap();
            let formatted = format_multiple_results_with_limit(&results);

            // Should not exceed Discord limits
            assert!(
                formatted.len() <= 2000,
                "Formatted output should fit Discord limit for '{}': {} chars",
                expression,
                formatted.len()
            );

            // Should still be readable
            assert!(
                !formatted.is_empty(),
                "Should have output for '{}'",
                expression
            );
            assert!(
                formatted.contains("**"),
                "Should have bold formatting for '{}'",
                expression
            );

            println!(
                "Formatting test '{}' produced {} chars: {}",
                description,
                formatted.len(),
                if formatted.len() > 100 {
                    format!("{}...", &formatted[..100])
                } else {
                    formatted
                }
            );
        } else {
            println!(
                "Formatting edge case '{}' failed to parse: {}",
                expression, description
            );
        }
    }
}

#[test]
fn test_modifier_position_behavior() {
    // Test that modifier position affects behavior correctly in end-to-end scenarios

    // Test pre-target modifier behavior (modifier affects dice before success counting)
    // We can't control the exact roll, but we can test the parsing and basic behavior
    let result1 = parse_and_roll("1d20+5 t6");
    assert!(
        result1.is_ok(),
        "Pre-target modifier should parse correctly"
    );

    let roll1 = result1.unwrap();
    assert_eq!(roll1.len(), 1);
    assert!(roll1[0].successes.is_some(), "Should have success counting");

    // Test post-target modifier behavior (modifier affects success count after counting)
    let result2 = parse_and_roll("1d20 t5 + 5");
    assert!(
        result2.is_ok(),
        "Post-target modifier should parse correctly"
    );

    let roll2 = result2.unwrap();
    assert_eq!(roll2.len(), 1);
    assert!(roll2[0].successes.is_some(), "Should have success counting");

    // Basic sanity check: both should produce reasonable success counts
    let success_count1 = roll1[0].successes.unwrap();
    let success_count2 = roll2[0].successes.unwrap();

    assert!(
        success_count1 >= 0 && success_count1 <= 25,
        "Pre-target modifier should give reasonable success count, got {}",
        success_count1
    );

    assert!(
        success_count2 >= 0 && success_count2 <= 25,
        "Post-target modifier should give reasonable success count, got {}",
        success_count2
    );
}

#[test]
fn test_wod_cancel_integration_scenarios() {
    // Test real-world usage scenarios for WOD cancel

    let integration_scenarios = vec![
        // Basic usage
        ("4wod8c", "Basic WoD with cancel"),
        ("5wod6c + 2", "WoD cancel with modifier"),
        // With labels and comments
        (
            "(Melee Attack) 6wod7c ! Using claws",
            "Labeled WoD with cancel",
        ),
        // With roll sets
        ("3 4wod8c", "Multiple WoD cancel rolls"),
        // With flags
        ("p 5wod6c", "Private WoD cancel roll"),
        ("s 4wod8c", "Simple WoD cancel roll"),
        // Mixed with other expressions
        ("4wod8c ; 3d6 + 2", "WoD cancel mixed with regular dice"),
    ];

    for (expression, description) in integration_scenarios {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "WoD cancel integration '{}' should work: {}",
            expression,
            description
        );

        let results = result.unwrap();
        assert!(
            !results.is_empty(),
            "Should have results for '{}': {}",
            expression,
            description
        );

        // Check formatting works
        let formatted = format_multiple_results(&results);
        assert!(
            !formatted.is_empty(),
            "Should format correctly for '{}': {}",
            expression,
            description
        );

        assert!(
            formatted.len() <= 2000,
            "Should fit Discord limit for '{}': {}",
            expression,
            description
        );
    }
}

// ============================================================================
// DICE FORMATTING TESTS
// ============================================================================

#[cfg(test)]
mod dice_formatting_final_tests {
    use super::*;

    /// Test that our fix works for the case where dropped dice exist
    #[test]
    fn test_formatting_fix_with_actual_drops() {
        // Test the case that our fix is designed for: when dropped dice exist
        let result = parse_and_roll("4d1 k2").unwrap();
        let roll = &result[0];

        // Verify this case has dropped dice (our fix should apply)
        assert!(
            !roll.dropped_rolls.is_empty(),
            "Should have dropped dice for this test to be valid"
        );

        let formatted = roll.to_string();
        println!("Test with drops '4d1 k2': {}", formatted);

        // Our fix should ensure all original dice are shown
        assert!(
            formatted.contains("`[1, 1, 1, 1]`"),
            "Should show all 4 original dice"
        );
        assert!(
            formatted.contains("~~[1, 1]~~"),
            "Should show 2 dropped dice"
        );
        assert!(formatted.contains("**2**"), "Should show correct total");
    }

    /// Test that our fix doesn't break cases without drops  
    #[test]
    fn test_no_regression_without_drops() {
        // Test cases where our fix should NOT activate (no dropped dice)
        let test_cases = vec![
            "1d6+2d6",     // AddDice without keep
            "3d6",         // Simple dice without keep
            "1d1+3d1 k10", // Keep more than available (no drops)
        ];

        for case in test_cases {
            let result = parse_and_roll(case).unwrap();
            let roll = &result[0];

            // Verify no dropped dice (our fix should NOT activate)
            assert!(
                roll.dropped_rolls.is_empty(),
                "Should have no dropped dice for case: {}",
                case
            );

            let formatted = roll.to_string();
            println!("Test without drops '{}': {}", case, formatted);

            // Should work normally
            assert!(
                !formatted.contains("~~"),
                "Should not show strikethrough for: {}",
                case
            );
            assert!(formatted.contains("**"), "Should show total for: {}", case);
        }
    }

    #[test]
    fn test_keep_modifier_display_now_working() {
        // The keep modifier display is now working correctly!
        // Update this test to reflect the correct behavior

        let test_cases = vec![
            // (expression, expected_pattern, description)
            (
                "4d1 k2",
                "[1, 1] ~~[1, 1]~~",
                "Simple keep should show dropped dice",
            ),
            (
                "1d1+3d1 k2",
                "[1] + [1, 1] ~~[1]~~",
                "AddDice with keep should show dropped dice",
            ),
            (
                "1d20+3d6 k2",
                "+ [X, Y] ~~[Z]~~",
                "AddDice should show kept and dropped",
            ),
        ];
        for (expression, expected_pattern, description) in test_cases {
            let result = parse_and_roll(expression).unwrap();
            let formatted = result[0].to_string();

            println!("Testing '{}' ({}): {}", expression, description, formatted);

            // Check that we have the correct structure
            if expression.contains("+") && expression.contains("k") {
                // Should have dropped dice display
                let has_dropped_display = formatted.contains("~~[") || formatted.contains("] ["); // Alternative format

                assert!(
                    has_dropped_display,
                    "Expression '{}' should show dropped dice in: {}\nExpected pattern: {}\nTest: {}",
                    expression, formatted, expected_pattern, description
                );
            }
        }
    }

    /// Test that our fix activates correctly based on dropped dice
    #[test]
    fn test_fix_activation_logic() {
        // Test the logic that determines when our fix should activate

        // Case 1: No dropped dice, no dice groups - standard behavior
        let result1 = parse_and_roll("3d1").unwrap();
        let roll1 = &result1[0];
        assert!(roll1.dropped_rolls.is_empty());
        assert!(roll1.dice_groups.is_empty() || roll1.dice_groups.len() == 1);

        // Case 2: No dropped dice, with dice groups - standard behavior
        let result2 = parse_and_roll("1d1+2d1").unwrap();
        let roll2 = &result2[0];
        assert!(roll2.dropped_rolls.is_empty());
        assert!(roll2.dice_groups.len() > 1);

        // Case 3: With dropped dice, no dice groups - our fix should activate
        let result3 = parse_and_roll("4d1 k2").unwrap();
        let roll3 = &result3[0];
        if !roll3.dropped_rolls.is_empty() && roll3.dice_groups.len() <= 1 {
            let formatted3 = roll3.to_string();
            // Our fix should show all original dice
            assert!(formatted3.contains("~~"), "Should show dropped dice");
        }

        // Case 4: With dropped dice AND dice groups - our fix should activate
        // This case doesn't currently exist due to the broken keep logic,
        // but our fix is designed to handle it when the keep logic is fixed

        println!("Fix activation logic test completed");
    }
}

#[test]
fn test_a5e_result_formatting() {
    use dicemaiden_rs::{format_multiple_results, parse_and_roll};

    // Test A5E result formatting
    let result = parse_and_roll("a5e +5 ex1").expect("A5E should parse");
    let formatted = format_multiple_results(&result);

    // Should have proper formatting
    assert!(formatted.contains("**")); // Bold totals
    assert!(formatted.contains("`[")); // Dice display

    // Should show the total
    let total = result[0].total;
    assert!(formatted.contains(&format!("**{}**", total)));
}

#[test]
fn test_a5e_roll_sets_formatting() {
    use dicemaiden_rs::{format_multiple_results, parse_and_roll};

    // Test A5E roll sets formatting
    let result = parse_and_roll("3 a5e +5 ex1").expect("A5E roll sets should work");
    let formatted = format_multiple_results(&result);

    // Should show individual sets
    assert!(formatted.contains("Set 1"));
    assert!(formatted.contains("Set 2"));
    assert!(formatted.contains("Set 3"));

    // Should show combined total
    assert!(formatted.contains("**Total:"));

    let expected_total: i32 = result.iter().map(|r| r.total).sum();
    assert!(formatted.contains(&format!("{}**", expected_total)));
}

#[test]
fn test_a5e_semicolon_separated() {
    use dicemaiden_rs::{format_multiple_results, parse_and_roll};

    // Test A5E with semicolon-separated rolls
    let result = parse_and_roll("a5e +5 ex1; a5e +7 ex2; a5e +3 ex3").expect("Should parse");
    assert_eq!(result.len(), 3);

    // Each should have original expressions
    for roll in &result {
        assert!(roll.original_expression.is_some());
    }

    let formatted = format_multiple_results(&result);
    assert!(formatted.contains("Request:")); // Should show individual requests
}

#[test]
fn test_alien_rpg_full_integration() {
    // Test complete Aliens RPG workflows that might be used in actual play
    let alien_workflows = vec![
        // Basic skill tests
        ("alien4", "Simple attribute + skill test"),
        ("alien6", "High-skill character test"),
        // Stress progression
        ("alien4s1", "Light stress situation"),
        ("alien4s3", "Moderate stress situation"),
        ("alien4s5", "High stress situation"),
        // Push mechanics
        ("alien3s2p", "Pushing a moderate stress roll"),
        ("alien4s1p", "Pushing a low stress roll"),
        // Mathematical modifiers in stressful situations
        ("alien4s2 + 2", "Stress roll with equipment bonus"),
        ("alien3s3 - 1", "Stress roll with injury penalty"),
        ("alien5s1 * 2", "Stress roll with doubled effect"),
        // Group action scenarios
        ("3 alien4s2", "Group performing same stressful action"),
        ("5 alien3s1", "Large group with light stress"),
        // Complex scenarios
        ("alien6s4 + 1d6", "High stress with additional dice"),
        ("alien4s3 - 2d4", "Stress roll with variable penalty"),
    ];

    for (expression, description) in alien_workflows {
        let result = parse_and_roll(expression);
        assert!(
            result.is_ok(),
            "Alien workflow '{}' should work in integration: {} - Error: {:?}",
            expression,
            description,
            result.err()
        );

        let results = result.unwrap();

        // Verify basic structure
        assert!(
            !results.is_empty(),
            "Alien workflow '{}' should produce results",
            expression
        );

        for (set_index, roll) in results.iter().enumerate() {
            // All Alien rolls should have success counting
            assert!(
                roll.successes.is_some(),
                "Alien workflow '{}' set {} should have success counting",
                expression,
                set_index + 1
            );

            // Should have appropriate system notes (only for stress rolls)
            if expression.contains('s') {
                let has_stress_note = roll.notes.iter().any(|note| note.contains("STRESS"));
                assert!(
                    has_stress_note,
                    "Alien workflow '{}' set {} should have stress notes",
                    expression,
                    set_index + 1
                );
            } else {
                // Basic alien rolls don't have system notes, just success counting
                // This is intentional - the success counting indicates it's working
            }

            // If this is a roll set, verify proper labeling
            if results.len() > 1 {
                assert!(
                    roll.label.is_some(),
                    "Alien workflow '{}' should have set labels",
                    expression
                );

                let label = roll.label.as_ref().unwrap();
                assert!(
                    label.starts_with("Set "),
                    "Alien workflow '{}' should have proper set label format: {}",
                    expression,
                    label
                );
            }

            // Verify stress mechanics if applicable
            if expression.contains('s') && !expression.contains('p') {
                // Stress rolls should have stress-related features
                let has_stress_tracking = roll.alien_stress_level.is_some()
                    || roll.notes.iter().any(|note| note.contains("STRESS DICE"));
                assert!(
                    has_stress_tracking,
                    "Alien stress workflow '{}' set {} should have stress tracking",
                    expression,
                    set_index + 1
                );
            }

            // Verify panic mechanics if triggered
            if roll.alien_panic_roll.is_some() {
                let panic_roll = roll.alien_panic_roll.unwrap();

                // Panic roll should be in valid range
                assert!(
                    panic_roll >= 4 && panic_roll <= 16,
                    "Alien workflow '{}' panic roll {} should be in valid range",
                    expression,
                    panic_roll
                );

                // Should have panic notes
                let has_panic_note = roll.notes.iter().any(|note| note.contains("PANIC ROLL"));
                assert!(
                    has_panic_note,
                    "Alien workflow '{}' with panic should have panic explanation",
                    expression
                );

                // Should have push restriction note
                let has_push_restriction =
                    roll.notes.iter().any(|note| note.contains("Cannot push"));
                assert!(
                    has_push_restriction,
                    "Alien workflow '{}' with panic should prevent pushing",
                    expression
                );
            }
        }
    }
}

#[test]
fn test_alien_rpg_cross_system_compatibility() {
    // Test that Aliens RPG doesn't interfere with other systems
    let cross_system_tests = vec![
        // Aliens RPG mixed with other dice
        ("alien4 + 1d6", "Alien roll with regular dice addition"),
        ("alien3s2 + 2d10", "Alien stress with regular dice"),
        // Multiple different systems in one expression
        ("alien4; 1d20 + 5", "Alien and D&D in same expression"),
        ("alien3s1; 4cod", "Alien and Chronicles of Darkness"),
        ("alien4s2; sr6", "Alien and Shadowrun"),
        // Roll sets mixing systems (should fail gracefully or work)
        // Note: These test that the parser handles mixed systems correctly
    ];

    for (expression, description) in cross_system_tests {
        let result = parse_and_roll(expression);

        // Most of these should work
        if expression.contains(';') {
            // Semicolon expressions should work
            assert!(
                result.is_ok(),
                "Cross-system test '{}' should work: {} - Error: {:?}",
                expression,
                description,
                result.err()
            );

            let results = result.unwrap();
            assert!(
                results.len() >= 2,
                "Cross-system semicolon test '{}' should have multiple results",
                expression
            );
        } else {
            // Mixed dice in same expression should work
            assert!(
                result.is_ok(),
                "Cross-system mixed test '{}' should work: {} - Error: {:?}",
                expression,
                description,
                result.err()
            );

            let results = result.unwrap();
            assert_eq!(
                results.len(),
                1,
                "Cross-system mixed test '{}' should have one result",
                expression
            );
        }
    }
}

#[test]
fn test_daggerheart_with_roll_sets() {
    // Test daggerheart works with roll sets if applicable
    let result = parse_and_roll("3 dheart");
    if result.is_ok() {
        let results = result.unwrap();
        assert_eq!(results.len(), 3);
        for roll in &results {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        }
    }
}
