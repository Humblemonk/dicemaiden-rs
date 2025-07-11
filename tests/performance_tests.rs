// tests/performance_tests.rs - Performance and Limit Tests
//
// This file contains tests for performance characteristics and boundary limits:
// - Performance benchmarks and timing
// - Input length validation and DoS protection
// - Explosion and recursion limits
// - Memory usage patterns and optimization
// - Discord message length handling

use dicemaiden_rs::{format_multiple_results_with_limit, parse_and_roll};
use std::time::Instant;

// ============================================================================
// PERFORMANCE BENCHMARKS
// ============================================================================

#[test]
fn test_parsing_performance() {
    // Test that parsing completes within reasonable time
    let performance_cases = vec![
        ("1d6", "Simple dice", 10),
        ("10d10 e10 k5 +3", "Complex modifiers", 100),
        ("500d1000", "Maximum dice count", 200),
        ("20 50d6", "Large roll sets", 100),
        ("4d6;3d8;2d10;1d20", "Multiple rolls", 100),
        ("100d6 e6 ie k50 r1 t4 +10", "Very complex", 300),
        ("4wod8c + 2", "WoD cancel with modifier", 100),
    ];

    // Warmup runs to initialize lazy statics and regex compilation
    for _ in 0..3 {
        let _ = parse_and_roll("1d6");
        let _ = parse_and_roll("4cod");
        let _ = parse_and_roll("sw8");
    }

    for (expression, description, max_ms) in performance_cases {
        // Run multiple times and take the best time (after warmup)
        let mut best_time = u128::MAX;

        for _ in 0..5 {
            let start = Instant::now();
            let result = parse_and_roll(expression);
            let duration = start.elapsed().as_millis();

            assert!(
                result.is_ok(),
                "Performance test '{}' should succeed: {:?}",
                expression,
                result.err()
            );

            best_time = best_time.min(duration);
        }

        assert!(
            best_time <= max_ms,
            "Performance test '{}' ({}) took {}ms, expected ≤{}ms",
            expression,
            description,
            best_time,
            max_ms
        );
    }
}

#[test]
fn test_rolling_performance() {
    // Test that rolling large numbers of dice completes quickly
    let rolling_cases = vec![
        ("500d6", "Maximum standard dice", 300),
        ("100d100", "Large dice size", 200),
        ("50d6 e6", "Exploding dice", 400),
        ("100d6 k50", "Keep modifiers", 250),
        ("20 25d6", "Roll sets", 500),
    ];

    // Warmup
    for _ in 0..2 {
        let _ = parse_and_roll("10d6");
    }

    for (expression, description, max_ms) in rolling_cases {
        let mut best_time = u128::MAX;

        for _ in 0..3 {
            let start = Instant::now();
            let result = parse_and_roll(expression);
            let duration = start.elapsed().as_millis();

            assert!(
                result.is_ok(),
                "Rolling performance test '{}' should succeed",
                expression
            );

            best_time = best_time.min(duration);
        }

        assert!(
            best_time <= max_ms,
            "Rolling test '{}' ({}) took {}ms, expected ≤{}ms",
            expression,
            description,
            best_time,
            max_ms
        );

        // Verify we got reasonable results
        let results = parse_and_roll(expression).unwrap();
        assert!(!results.is_empty());
        for roll in &results {
            assert!(
                roll.total != 0 || roll.successes.is_some() || !roll.individual_rolls.is_empty()
            );
        }
    }
}

#[test]
fn test_formatting_performance() {
    // Test that formatting large results completes quickly
    let formatting_cases = vec![
        ("20 10d6", "Large roll set"),
        ("100d6", "Single large roll"),
        ("10d6;10d6;10d6;10d6", "Multiple large rolls"),
    ];

    // Warmup
    let _ = parse_and_roll("5d6");

    for (expression, description) in formatting_cases {
        let result = parse_and_roll(expression).unwrap();

        let mut best_time = u128::MAX;

        for _ in 0..5 {
            let start = Instant::now();
            let formatted = format_multiple_results_with_limit(&result);
            let duration = start.elapsed().as_millis();

            best_time = best_time.min(duration);

            assert!(
                formatted.len() <= 2000,
                "Formatted result should fit Discord limit: {} chars",
                formatted.len()
            );
        }

        assert!(
            best_time <= 100,
            "Formatting test '{}' ({}) took {}ms, expected ≤100ms",
            expression,
            description,
            best_time
        );
    }
}

// ============================================================================
// INPUT VALIDATION AND DOS PROTECTION
// ============================================================================

#[test]
fn test_input_length_limits() {
    // Test protection against very long inputs
    let normal_input = "1d20+5";
    assert!(parse_and_roll(normal_input).is_ok());

    let long_input = "10d6 e6 k8 +4".repeat(50); // ~700 chars
    assert!(
        parse_and_roll(&long_input).is_ok(),
        "Reasonable long input should work"
    );

    let very_long_input = "1d6+".repeat(300) + "1"; // ~1201 chars  
    let result = parse_and_roll(&very_long_input);
    assert!(result.is_err(), "Very long input should be rejected");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Input too long"),
        "Error should mention input length: {}",
        error_msg
    );
}

#[test]
fn test_maximum_dice_limits() {
    // Test limits on dice count and sides

    // Valid maximums
    assert!(parse_and_roll("500d1000").is_ok(), "Max dice should work");
    assert!(parse_and_roll("1d1000").is_ok(), "Max sides should work");
    assert!(parse_and_roll("500d1").is_ok(), "Max count should work");

    // Invalid maximums
    assert!(
        parse_and_roll("501d6").is_err(),
        "Too many dice should fail"
    );
    assert!(
        parse_and_roll("1d1001").is_err(),
        "Too many sides should fail"
    );
    assert!(parse_and_roll("0d6").is_err(), "Zero dice should fail");
    assert!(parse_and_roll("1d0").is_err(), "Zero sides should fail");
}

#[test]
fn test_roll_set_limits() {
    // Test limits on roll sets

    // Valid roll set sizes
    assert!(parse_and_roll("2 1d6").is_ok(), "Min roll sets should work");
    assert!(
        parse_and_roll("20 1d6").is_ok(),
        "Max roll sets should work"
    );

    // Invalid roll set sizes - ENHANCED with error message validation
    let result = parse_and_roll("1 1d6");
    assert!(result.is_err(), "Single roll set should fail");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Set count must be between 2 and 20"),
        "Error should mention valid range for single roll set, got: {}",
        error_msg
    );

    let result = parse_and_roll("21 1d6");
    assert!(result.is_err(), "Too many roll sets should fail");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("Set count must be between 2 and 20"),
        "Error should mention valid range for too many roll sets, got: {}",
        error_msg
    );

    assert!(
        parse_and_roll("100 1d6").is_err(),
        "Way too many roll sets should fail"
    );
}

#[test]
fn test_semicolon_roll_limits() {
    // Test limits on semicolon-separated rolls

    // Valid semicolon rolls
    assert!(parse_and_roll("1d6;1d6").is_ok(), "Two rolls should work");
    assert!(
        parse_and_roll("1d6;1d6;1d6;1d6").is_ok(),
        "Four rolls should work"
    );

    // Invalid semicolon rolls
    assert!(
        parse_and_roll("1d6;1d6;1d6;1d6;1d6").is_err(),
        "Five rolls should fail"
    );

    let many_rolls = vec!["1d6"; 10].join(";");
    assert!(
        parse_and_roll(&many_rolls).is_err(),
        "Many rolls should fail"
    );
}

// ============================================================================
// EXPLOSION AND RECURSION LIMITS
// ============================================================================

#[test]
fn test_explosion_limits() {
    // Test that exploding dice have reasonable limits

    // This should always explode but be limited
    let result = parse_and_roll("1d6 ie1").unwrap();
    assert!(
        result[0].individual_rolls.len() <= 101,
        "Explosions should be limited to prevent infinite loops: {} rolls",
        result[0].individual_rolls.len()
    );

    // Should have explosion limit note
    if result[0].individual_rolls.len() > 50 {
        let _has_limit_note = result[0]
            .notes
            .iter()
            .any(|note| note.contains("Maximum explosions reached"));
        // Note: this is probabilistic, so we only check if we got a lot of explosions
    }

    // Normal explosions should work
    let result = parse_and_roll("1d6 e6").unwrap();
    assert!(
        result[0].individual_rolls.len() >= 1,
        "Should have at least original roll"
    );
    assert!(
        result[0].individual_rolls.len() <= 101,
        "Should respect explosion limits: {} rolls",
        result[0].individual_rolls.len()
    );
}

#[test]
fn test_reroll_limits() {
    // Test that rerolls have reasonable limits

    // This should always reroll but be limited
    for _ in 0..5 {
        let result = parse_and_roll("3d6 ir6").unwrap();
        assert_eq!(
            result[0].individual_rolls.len(),
            3,
            "Should always have 3 dice"
        );

        // Should complete in reasonable time (covered by performance tests)
        // The limit is internal to prevent infinite loops
    }
}

#[test]
fn test_complex_modifier_performance() {
    // Test combinations that could cause performance issues

    let complex_combinations = vec![
        "10d6 ie6 k5 r1 t4 +5",         // Multiple modifiers
        "5d6 ie ir1 k3 + 2d6 e6 - 1d4", // Nested complexity
        "20d6 e6 k10 + 10",             // Large dice with modifiers
        "5d10 f1 t8 c ie10 + 3",
    ];

    // Warmup
    let _ = parse_and_roll("1d6");

    for combination in complex_combinations {
        let mut best_time = u128::MAX;

        for _ in 0..3 {
            let start = Instant::now();
            let result = parse_and_roll(combination);
            let duration = start.elapsed().as_millis();

            assert!(
                result.is_ok(),
                "Complex combination '{}' should work",
                combination
            );

            best_time = best_time.min(duration);
        }

        assert!(
            best_time <= 500,
            "Complex combination '{}' took {}ms, should be ≤500ms",
            combination,
            best_time
        );
    }
}

// ============================================================================
// MEMORY USAGE AND OPTIMIZATION
// ============================================================================

#[test]
fn test_memory_efficiency() {
    // Test that large operations don't use excessive memory
    // This is more of a structural test since we can't easily measure memory in tests

    let memory_intensive_cases = vec![
        "500d1000",      // Maximum dice
        "20 50d6",       // Large roll sets
        "100d6 ie6 k50", // Lots of explosions and kept dice
    ];

    for case in memory_intensive_cases {
        let result = parse_and_roll(case);
        assert!(
            result.is_ok(),
            "Memory intensive case '{}' should complete successfully",
            case
        );

        let results = result.unwrap();

        // Verify reasonable structure - not storing excessive data
        for roll in &results {
            // Individual rolls shouldn't be excessively large
            assert!(
                roll.individual_rolls.len() <= 1000,
                "Individual rolls should be reasonable: {} rolls",
                roll.individual_rolls.len()
            );

            // Notes shouldn't be excessive
            assert!(
                roll.notes.len() <= 20,
                "Notes should be reasonable: {} notes",
                roll.notes.len()
            );

            // Dice groups shouldn't be excessive
            assert!(
                roll.dice_groups.len() <= 10,
                "Dice groups should be reasonable: {} groups",
                roll.dice_groups.len()
            );
        }
    }
}

// ============================================================================
// DISCORD MESSAGE LENGTH HANDLING
// ============================================================================

#[test]
fn test_discord_message_limits() {
    // Test various scenarios that could exceed Discord's 2000 char limit

    let large_output_cases = vec![
        "20 20d6",             // Many large roll sets
        "500d6",               // Single huge roll
        "15 10d10 e10 k5 +3",  // Complex roll sets
        "10d6;10d6;10d6;10d6", // Multiple large rolls
    ];

    for case in large_output_cases {
        let result = parse_and_roll(case).unwrap();
        let formatted = format_multiple_results_with_limit(&result);

        assert!(
            formatted.len() <= 2000,
            "Case '{}' produced {}> chars, should be ≤2000",
            case,
            formatted.len()
        );

        // Should still contain meaningful information
        assert!(
            formatted.len() >= 10,
            "Formatted output should have meaningful content: '{}'",
            formatted
        );

        // Should contain some kind of result indicator
        assert!(
            formatted.contains("**") || formatted.contains("result"),
            "Should contain result indicators: '{}'",
            formatted
        );
    }
}

#[test]
fn test_message_truncation_graceful() {
    // Test that message truncation is graceful and informative

    // Create a scenario that definitely needs truncation
    let result = parse_and_roll("20 50d6").unwrap();
    let full_format = format_multiple_results_with_limit(&result);

    // Even if truncated, should be useful
    if full_format.len() == 2000 {
        // Was truncated - should end appropriately
        assert!(
            !full_format.ends_with("..."),
            "Should not end with bare ellipsis: '{}'",
            &full_format[1990..]
        );
    }

    // Should have totals or meaningful summary
    assert!(
        full_format.contains("**") || full_format.contains("Total"),
        "Truncated output should still show totals/results"
    );
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[test]
fn test_stress_scenarios() {
    // Test edge cases that could cause problems

    let stress_cases = vec![
        ("20 1d1", "Many simple rolls"),
        ("100d1", "Many simple dice"),
        ("10d100", "Large dice"),
        ("1d6 ie1", "Always exploding"),
        ("50d6 k25 d10", "Complex keep/drop"),
        ("20d10 t1 f10 b1", "Complex target system"),
    ];

    // Warmup
    let _ = parse_and_roll("1d6");

    for (case, description) in stress_cases {
        let mut best_parse_time = u128::MAX;
        let mut best_format_time = u128::MAX;

        for _ in 0..3 {
            let start = Instant::now();
            let result = parse_and_roll(case);
            let parse_time = start.elapsed().as_millis();

            assert!(
                result.is_ok(),
                "Stress case '{}' ({}) should work: {:?}",
                case,
                description,
                result.err()
            );

            let results = result.unwrap();

            let start = Instant::now();
            let formatted = format_multiple_results_with_limit(&results);
            let format_time = start.elapsed().as_millis();

            best_parse_time = best_parse_time.min(parse_time);
            best_format_time = best_format_time.min(format_time);

            // Output requirements
            assert!(
                formatted.len() <= 2000,
                "Stress case '{}' output too long: {} chars",
                case,
                formatted.len()
            );

            assert!(
                !formatted.is_empty(),
                "Stress case '{}' should produce output",
                case
            );
        }

        // Performance requirements
        assert!(
            best_parse_time <= 1000,
            "Stress case '{}' parsing took {}ms",
            case,
            best_parse_time
        );

        assert!(
            best_format_time <= 500,
            "Stress case '{}' formatting took {}ms",
            case,
            best_format_time
        );
    }
}

#[test]
fn test_cancel_modifier_performance() {
    // Test that cancel modifier doesn't add significant overhead

    let cancel_performance_tests = vec![
        ("10d10 f1 t8 c", "Cancel with multiple dice", 100),
        ("20 4wod8c", "Multiple WoD cancel rolls", 200),
        (
            "5d10 f1 t6 c ie10 + 2",
            "Cancel with complex modifiers",
            150,
        ),
    ];

    // Warmup
    for _ in 0..3 {
        let _ = parse_and_roll("4wod8c");
    }

    for (expression, description, max_ms) in cancel_performance_tests {
        let mut best_time = u128::MAX;

        for _ in 0..5 {
            let start = Instant::now();
            let result = parse_and_roll(expression);
            let duration = start.elapsed().as_millis();

            assert!(
                result.is_ok(),
                "Cancel performance test '{}' should succeed: {:?}",
                expression,
                result.err()
            );

            best_time = best_time.min(duration);
        }

        assert!(
            best_time <= max_ms,
            "Cancel performance test '{}' ({}) took {}ms, expected ≤{}ms",
            expression,
            description,
            best_time,
            max_ms
        );
    }
}

// ============================================================================
// CRITICAL NEW TEST 5: Specific cancel mechanics validation
// ============================================================================

// ADD this test to tests/game_systems_tests.rs to validate the actual cancel mechanics:

#[test]
fn test_cancel_mechanics_validation() {
    // Test specific cancel scenarios to ensure correct behavior
    // Note: These tests are probabilistic but help validate the logic

    for _ in 0..20 {
        let result = parse_and_roll("4wod8c");
        if result.is_ok() {
            let results = result.unwrap();
            let roll = &results[0];

            // Basic validation that the mechanics are working
            assert!(roll.successes.is_some(), "Should track successes");
            assert!(roll.failures.is_some(), "Should track failures");

            // Check for cancel notes when appropriate
            let has_cancel_note = roll.notes.iter().any(|note| note.contains("CANCELLED"));
            let has_tens = roll.kept_rolls.iter().any(|&r| r == 10);
            let has_ones = roll.kept_rolls.iter().any(|&r| r == 1);

            // If we have both 10s and 1s, we might see a cancel note
            if has_tens && has_ones {
                // This is probabilistic, so we can't assert it always happens
                if has_cancel_note {
                    println!("Cancel note found: {:?}", roll.notes);
                }
            }
        }
    }
}
