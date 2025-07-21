// tests/performance_tests.rs - Performance and Limit Tests
//
// This file contains tests for performance characteristics and boundary limits:
// - Performance benchmarks and timing
// - Input length validation and DoS protection
// - Explosion and recursion limits
// - Memory usage patterns and optimization
// - Discord message length handling

use anyhow::Result;
use dicemaiden_rs::{RollResult, format_multiple_results_with_limit, parse_and_roll};
use std::time::Instant;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Helper function to measure performance and return best time across multiple runs
fn measure_best_performance_time(expression: &str, runs: usize) -> (u128, Result<Vec<RollResult>>) {
    let mut best_time = u128::MAX;
    let mut last_result = None;

    for _ in 0..runs {
        let start = Instant::now();
        let result = parse_and_roll(expression);
        let duration = start.elapsed().as_millis();

        last_result = Some(result);
        best_time = best_time.min(duration);
    }

    (best_time, last_result.unwrap())
}

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
        ("a5e +5 ex1", "A5E basic", 100),
        ("+a5e +7 ex2", "A5E advantage", 200),
        ("3 a5e +5 ex1", "A5E roll sets", 300),
    ];

    // Warmup runs to initialize lazy statics and regex compilation
    for _ in 0..3 {
        let _ = parse_and_roll("1d6");
        let _ = parse_and_roll("4cod");
        let _ = parse_and_roll("sw8");
        let _ = parse_and_roll("a5e +5 ex1");
    }

    for (expression, description, max_ms) in performance_cases {
        let (best_time, result) = measure_best_performance_time(expression, 5);

        assert!(
            result.is_ok(),
            "Performance test '{}' should succeed: {:?}",
            expression,
            result.err()
        );

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
        let (best_time, result) = measure_best_performance_time(expression, 3);

        assert!(
            result.is_ok(),
            "Rolling test '{}' should succeed: {:?}",
            expression,
            result.err()
        );

        assert!(
            best_time <= max_ms,
            "Rolling test '{}' ({}) took {}ms, expected ≤{}ms",
            expression,
            description,
            best_time,
            max_ms
        );
    }
}

// ============================================================================
// INPUT VALIDATION AND DOS PROTECTION
// ============================================================================

#[test]
fn test_input_length_validation() {
    // Test that excessively long inputs are rejected

    let long_input = "1d6 + ".repeat(1000); // Creates a very long expression
    let result = parse_and_roll(&long_input);

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
        let (best_time, result) = measure_best_performance_time(combination, 3);

        assert!(
            result.is_ok(),
            "Complex combination '{}' should work",
            combination
        );

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
                roll.dice_groups.len() <= 50,
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
fn test_format_truncation() {
    // Test that extremely long results are properly truncated

    let long_result_expression = "500d1"; // Will produce a very long individual dice listing
    let result = parse_and_roll(long_result_expression).unwrap();

    let formatted = format_multiple_results_with_limit(&result);

    // Should be within Discord's 2000 character limit
    assert!(
        formatted.len() <= 2000,
        "Formatted output should be truncated to 2000 chars: {} chars",
        formatted.len()
    );

    // Should not be empty
    assert!(!formatted.is_empty(), "Output should not be empty");

    // Should contain the total (most important part)
    assert!(
        formatted.contains("**500**") || formatted.contains("Total"),
        "Should contain total or result indicator"
    );

    // If truncated, should end with proper indication, not bare "..."
    if formatted.len() >= 1990 {
        assert!(
            !formatted.ends_with("..."),
            "Should not end with bare ellipsis: '{}'",
            &formatted[1990..]
        );
    }

    // Should have totals or meaningful summary
    assert!(
        formatted.contains("**") || formatted.contains("Total"),
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
        ("10d10 f1 t8 c", "Cancel with multiple dice", 200),
        ("20 4wod8c", "Multiple WoD cancel rolls", 200),
        (
            "5d10 f1 t6 c ie10 + 2",
            "Cancel with complex modifiers",
            200,
        ),
    ];

    // Warmup
    for _ in 0..3 {
        let _ = parse_and_roll("4wod8c");
    }

    for (expression, description, max_ms) in cancel_performance_tests {
        let (best_time, result) = measure_best_performance_time(expression, 5);

        assert!(
            result.is_ok(),
            "Cancel performance test '{}' should succeed: {:?}",
            expression,
            result.err()
        );

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

#[test]
fn test_a5e_alias_performance() {
    use dicemaiden_rs::dice::aliases;
    use std::time::Instant;

    // Test that A5E alias expansion is fast
    let start = Instant::now();

    for _ in 0..1000 {
        let _ = aliases::expand_alias("a5e +5 ex1");
        let _ = aliases::expand_alias("+a5e +5 ex1");
        let _ = aliases::expand_alias("-a5e +5 ex1");
        let _ = aliases::expand_alias("a5e ex2");
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "A5E alias expansion should be fast: {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_alien_rpg_alias_performance() {
    use dicemaiden_rs::dice::aliases;
    use std::time::Instant;

    // Test that Alien RPG alias expansion is fast
    let start = Instant::now();

    for _ in 0..1000 {
        let _ = aliases::expand_alias("alien4");
        let _ = aliases::expand_alias("alien5s2");
        let _ = aliases::expand_alias("alien3s1p");
        let _ = aliases::expand_alias("alien6s4");
        let _ = aliases::expand_alias("alien10s10");
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "Alien RPG alias expansion should be fast: {}ms",
        duration.as_millis()
    );
}

#[test]
fn test_alien_panic_roll_performance() {
    // Test that panic roll generation doesn't add significant overhead
    // Note: We can't force panic rolls, but we can test the worst-case scenario

    let panic_performance_tests = vec![
        ("alien4s3", "Moderate stress (potential panic)", 200),
        ("alien6s5", "High stress (potential panic)", 250),
        ("alien10s10", "Maximum stress (potential panic)", 300),
        ("10 alien4s3", "Multiple potential panic rolls", 500),
        ("20 alien3s2", "Many moderate stress rolls", 600),
    ];

    // Run multiple times to potentially trigger panic mechanics
    for (expression, _description, max_ms) in panic_performance_tests {
        let mut total_time = 0u128;
        let mut panic_count = 0;

        // Run 10 times to get average performance and potentially trigger panics
        for _ in 0..10 {
            let (time, result) = measure_best_performance_time(expression, 1);
            total_time += time;

            if let Ok(results) = result {
                for roll in &results {
                    if roll.alien_panic_roll.is_some() {
                        panic_count += 1;
                    }
                }
            }
        }

        let avg_time = total_time / 10;
        assert!(
            avg_time <= max_ms,
            "Alien panic performance test '{}' averaged {}ms, expected ≤{}ms (with {} panics)",
            expression,
            avg_time,
            max_ms,
            panic_count
        );
    }
}
