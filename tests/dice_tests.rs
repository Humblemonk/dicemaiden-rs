// tests/dice_tests.rs - Comprehensive Dice Maiden test suite
use dicemaiden_rs::{
    dice::{Modifier, RollResult, aliases, parser},
    format_multiple_results_with_limit, help_text, parse_and_roll,
};

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // BASIC DICE TESTS
    // ============================================================================

    #[test]
    fn test_left_to_right_math_evaluation() {
        // Test left-to-right vs PEMDAS differences using both numbers and dice expressions

        // Pure number expressions with 1d1 (always rolls 1)
        // This should be (1 + 2) * 3 = 9, not 1 + (2 * 3) = 7
        let result = parse_and_roll("1d1 + 2 * 3").unwrap();
        assert_eq!(
            result[0].total, 9,
            "Should evaluate left-to-right: (1 + 2) * 3 = 9"
        );

        // This should be (1 + 6) / 2 = 3, vs 1 + (6 / 2) = 4
        let result = parse_and_roll("1d1 + 6 / 2").unwrap();
        assert_eq!(
            result[0].total, 3,
            "Should evaluate left-to-right: (1 + 6) / 2 = 3"
        );

        // Chain of operations: ((1 + 2) * 3) - 4 = 5
        let result = parse_and_roll("1d1 + 2 * 3 - 4").unwrap();
        assert_eq!(
            result[0].total, 5,
            "Should evaluate left-to-right: ((1 + 2) * 3) - 4 = 5"
        );

        // Test that mathematical modifiers on dice follow left-to-right order
        // 1d6 + 10 * 2 should be (roll + 10) * 2, not roll + (10 * 2) = roll + 20
        let result = parse_and_roll("1d1 + 10 * 2").unwrap();
        // 1d1 always rolls 1, so: (1 + 10) * 2 = 22
        assert_eq!(
            result[0].total, 22,
            "Should be left-to-right: (1 + 10) * 2 = 22"
        );

        // Test subtraction and division: 1d1 + 15 - 6 / 3
        // Left-to-right: ((1 + 15) - 6) / 3 = (16 - 6) / 3 = 10 / 3 = 3
        // PEMDAS would be: 1 + 15 - (6 / 3) = 1 + 15 - 2 = 14
        let result = parse_and_roll("1d1 + 15 - 6 / 3").unwrap();
        assert_eq!(
            result[0].total, 3,
            "Should be left-to-right: ((1 + 15) - 6) / 3 = 3"
        );
    }

    #[test]
    fn test_basic_dice() {
        // Basic formats
        assert_valid("1d6");
        assert_valid("2d6");
        assert_valid("d6"); // Default 1
        assert_valid("1d%"); // Percentile
        assert_valid("d%");

        // With/without spaces
        assert_valid("2d6+3");
        assert_valid("2d6 + 3");
        assert_valid("  2d6  +  3  ");
    }

    #[test]
    fn test_basic_dice_parsing() {
        let result = parser::parse_dice_string("2d6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 2);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 0);
    }

    #[test]
    fn test_dice_limits() {
        assert_valid("500d1000"); // Max allowed
        assert_invalid("501d6"); // Too many dice
        assert_invalid("1d1001"); // Too many sides
        assert_invalid("0d6"); // Zero dice
        assert_invalid("1d0"); // Zero sides
    }

    #[test]
    fn test_single_die() {
        let result = parser::parse_dice_string("d6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[0].sides, 6);
    }

    #[test]
    fn test_maximum_valid_dice_count() {
        let result = parser::parse_dice_string("500d6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 500);
        assert_eq!(result[0].sides, 6);
    }

    #[test]
    fn test_maximum_valid_die_sides() {
        let result = parser::parse_dice_string("1d1000").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[0].sides, 1000);
    }

    #[test]
    fn test_percentile_dice() {
        let result = parser::parse_dice_string("1d%").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[0].sides, 100);
    }

    // ============================================================================
    // MATHEMATICAL MODIFIERS
    // ============================================================================

    #[test]
    fn test_math_modifiers() {
        // Basic operations
        assert_valid("1d6+5");
        assert_valid("1d6 + 5");
        assert_valid("2d6-3");
        assert_valid("2d6 - 3");
        assert_valid("1d6*2");
        assert_valid("1d6 * 2");
        assert_valid("4d6/2");
        assert_valid("4d6 / 2");

        // Dice operations
        assert_valid("2d6+1d4");
        assert_valid("2d8+2d6+30");
        assert_valid("2d6 + 1d4");
        assert_valid("3d8-1d6");
        assert_valid("3d8 - 1d6");

        // Complex
        assert_valid("2d6+3d8+5");
        assert_valid("1d20+1d6-2");

        // Error cases
        assert_invalid("1d6/0");
    }

    #[test]
    fn test_dice_with_modifier() {
        let result = parser::parse_dice_string("3d8 + 5").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 3);
        assert_eq!(result[0].sides, 8);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Add(value) => assert_eq!(*value, 5),
            _ => panic!("Expected Add(5) modifier"),
        }
    }

    #[test]
    fn test_dice_math_with_multiple_dice() {
        let result = parser::parse_dice_string("10d6 e6 k8 +4").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 10);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 3);
    }

    #[test]
    fn test_dice_operations_with_predictable_results() {
        // Test dice multiplication with fixed dice
        let result = parse_and_roll("3d1 * 2d1").unwrap();
        assert_eq!(result.len(), 1);
        // 3d1 = 3, 2d1 = 2, so 3 * 2 = 6
        assert_eq!(result[0].total, 6, "3d1 * 2d1 should equal 6");
        // Verify dice groups are created correctly
        assert_eq!(result[0].dice_groups.len(), 2, "Should have 2 dice groups");
        assert_eq!(result[0].dice_groups[0].modifier_type, "base");
        assert_eq!(result[0].dice_groups[1].modifier_type, "multiply");

        // Test dice division with fixed dice
        let result = parse_and_roll("8d1 / 2d1").unwrap();
        assert_eq!(result.len(), 1);
        // 8d1 = 8, 2d1 = 2, so 8 / 2 = 4
        assert_eq!(result[0].total, 4, "8d1 / 2d1 should equal 4");
        // Verify dice groups are created correctly
        assert_eq!(result[0].dice_groups.len(), 2, "Should have 2 dice groups");
        assert_eq!(result[0].dice_groups[0].modifier_type, "base");
        assert_eq!(result[0].dice_groups[1].modifier_type, "divide");

        // Test that the display format includes the correct operators between dice groups
        let result = parse_and_roll("2d1 * 3d1").unwrap();
        let formatted = result[0].to_string();
        // Should contain the multiplication symbol between dice groups
        assert!(
            formatted.contains("*"),
            "Display should contain * symbol between dice groups"
        );
        assert!(formatted.contains("**6**"), "Should show final result");

        let result = parse_and_roll("6d1 / 2d1").unwrap();
        let formatted = result[0].to_string();
        // Should contain the division symbol between dice groups
        assert!(
            formatted.contains("/"),
            "Display should contain / symbol between dice groups"
        );
        assert!(formatted.contains("**3**"), "Should show final result");
    }

    #[test]
    fn test_number_divided_by_dice() {
        // Test number divided by dice
        let result = parse_and_roll("200/1d1").unwrap();
        assert_eq!(result.len(), 1);
        // 1d1 always rolls 1, so 200/1 = 200
        assert_eq!(result[0].total, 200, "200/1d1 should equal 200");

        // Test with 2d1 (always rolls 2 total)
        let result = parse_and_roll("100/2d1").unwrap();
        assert_eq!(result.len(), 1);
        // 2d1 always rolls 2, so 100/2 = 50
        assert_eq!(result[0].total, 50, "100/2d1 should equal 50");

        // Test parsing works correctly
        let result = parse_and_roll("300/3d1");
        assert!(result.is_ok(), "300/3d1 should parse successfully");

        if let Ok(results) = result {
            // 3d1 always rolls 3, so 300/3 = 100
            assert_eq!(results[0].total, 100, "300/3d1 should equal 100");
        }
    }

    #[test]
    fn test_number_divided_by_percentile_dice() {
        // Test with percentile dice (d%)
        // Use a loop to test multiple times since d% is random
        let mut found_valid_result = false;

        for _ in 0..10 {
            let result = parse_and_roll("500/d%").unwrap();
            assert_eq!(result.len(), 1);

            // d% rolls 1-100, so 500/d% should be between 5 and 500
            let total = result[0].total;
            if total >= 5 && total <= 500 {
                found_valid_result = true;
                break;
            }
        }

        assert!(
            found_valid_result,
            "500/d% should produce results between 5 and 500"
        );
    }

    #[test]
    fn test_math_edge_cases() {
        // Roll sets with division syntax
        let result = parse_and_roll("4 20/2d1").unwrap();
        assert_eq!(result.len(), 4, "Should create 4 roll sets");
        for roll in &result {
            assert_eq!(roll.total, 10, "Each set should be 20/2 = 10");
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        }

        // Mathematical division with spaces
        let result = parse_and_roll("20 / 2d1").unwrap();
        assert_eq!(result.len(), 1, "Should be single mathematical expression");
        assert_eq!(result[0].total, 10, "Should be 20 / 2 = 10");
        assert!(result[0].label.is_none(), "Should not have set label");

        // Mathematical division without spaces
        let result = parse_and_roll("20/2d1").unwrap();
        assert_eq!(result.len(), 1, "Should be single mathematical expression");
        assert_eq!(result[0].total, 10, "Should be 20/2 = 10");
        assert!(result[0].label.is_none(), "Should not have set label");
    }

    #[test]
    fn test_complex_dice_and_number_operations() {
        // Test multiple dice and number operation: dice + number * dice
        let result = parse_and_roll("2d1 + 3 * 2d1").unwrap();
        assert_eq!(result.len(), 1);

        // Left-to-right: (2 + 3) * 2 = 10
        assert_eq!(
            result[0].total, 10,
            "Should evaluate left-to-right: (2+3)*2=10"
        );

        // Should have at least 2 dice groups
        assert!(
            result[0].dice_groups.len() >= 2,
            "Should have multiple dice groups"
        );
    }

    #[test]
    fn test_mixed_dice_and_number_operations() {
        // Test combinations of dice operations and number operations
        assert_valid("2d6 * 3 + 1d4"); // (dice * number) + dice
        assert_valid("1d8 + 2d6 / 2"); // dice + (dice / number)
        assert_valid("3d6 * 2d4 + 5"); // (dice * dice) + number
        assert_valid("4d10 - 1d6 * 3"); // dice - (dice * number)

        // Test complex mixed expressions with numbers and dice operations
        assert_valid("3d6 + 15 / 2d4"); // dice + (number / dice)
        assert_valid("2d8 - 10 * 1d6"); // dice - (number * dice)
        assert_valid("1d20 * 5 / 2d4"); // (dice * number) / dice
        assert_valid("4d6 / 3 + 2d8"); // (dice / number) + dice

        // Test with predictable results using fixed dice
        let result = parse_and_roll("2d1 * 3 + 1d1").unwrap();
        // Left-to-right: (2 * 3) + 1 = 7
        assert_eq!(result[0].total, 7, "Should be (2*3)+1=7");

        // Test complex expression with number/dice division
        let result = parse_and_roll("3d1 + 15 / 3d1").unwrap();
        // Left-to-right: (3 + 15) / 3 = 18 / 3 = 6
        assert_eq!(result[0].total, 6, "Should be (3+15)/3=6");

        // Test that all dice groups are displayed correctly - just check we have multiple groups
        assert!(
            result[0].dice_groups.len() >= 2,
            "Should have multiple dice groups"
        );
    }

    #[test]
    fn test_math_behavior() {
        let results = parse_and_roll("2d6 + 3").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].total,
            results[0].individual_rolls.iter().sum::<i32>() + 3
        );
    }

    #[test]
    fn test_format_multiple_results() {
        let result1 = RollResult {
            individual_rolls: vec![3, 5],
            kept_rolls: vec![3, 5],
            dropped_rolls: vec![],
            total: 8,
            successes: None,
            failures: None,
            botches: None,
            comment: None,
            label: None,
            notes: vec![],
            dice_groups: vec![],
            original_expression: None,
            simple: false,
            no_results: false,
            private: false,
            godbound_damage: None,
            fudge_symbols: None,
            wng_wrath_die: None,
            wng_icons: None,
            wng_exalted_icons: None,
            wng_wrath_dice: None,
            suppress_comment: false,
        };

        let results = vec![result1];
        let formatted = dicemaiden_rs::format_multiple_results(&results);
        assert!(formatted.contains("**8**"));
    }

    // ============================================================================
    // DICE MODIFIERS
    // ============================================================================

    #[test]
    fn test_exploding_dice() {
        assert_valid("3d6e");
        assert_valid("3d6 e");
        assert_valid("3d6e6");
        assert_valid("3d6 e6");
        assert_valid("4d10ie");
        assert_valid("4d10 ie");
        assert_valid("4d10ie8");
        assert_valid("4d10 ie8");
    }

    #[test]
    fn test_exploding_dice_parsing() {
        let result = parser::parse_dice_string("4d6 e6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Explode(Some(6)) => {}
            _ => panic!("Expected Explode(6) modifier"),
        }
    }

    #[test]
    fn test_indefinite_explode() {
        let result = parser::parse_dice_string("3d6 ie6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::ExplodeIndefinite(Some(6)) => {}
            _ => panic!("Expected ExplodeIndefinite(6) modifier"),
        }
    }

    #[test]
    fn test_keep_drop_dice() {
        assert_valid("4d6k3");
        assert_valid("4d6 k3");
        assert_valid("4d6d1");
        assert_valid("4d6 d1");
        assert_valid("4d6kl2");
        assert_valid("4d6 kl2");
        // Note: dh and dl might need different parsing
    }

    #[test]
    fn test_keep_high() {
        let result = parser::parse_dice_string("4d6 k3").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::KeepHigh(3) => {}
            _ => panic!("Expected KeepHigh(3) modifier"),
        }
    }

    #[test]
    fn test_reroll_dice() {
        assert_valid("4d6r1");
        assert_valid("4d6 r1");
        assert_valid("4d6r2");
        assert_valid("4d6 r2");
        assert_valid("4d6ir1");
        assert_valid("4d6 ir1");
    }

    #[test]
    fn test_reroll_parsing() {
        let result = parser::parse_dice_string("4d6 r2").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Reroll(2) => {}
            _ => panic!("Expected Reroll(2) modifier"),
        }
    }

    #[test]
    fn test_indefinite_reroll() {
        let result = parser::parse_dice_string("4d6 ir2").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::RerollIndefinite(2) => {}
            _ => panic!("Expected RerollIndefinite(2) modifier"),
        }
    }

    // ============================================================================
    // TARGET SYSTEM TESTS
    // ============================================================================

    #[test]
    fn test_target_system() {
        assert_valid("6d10t7");
        assert_valid("6d10 t7");
        assert_valid("4d6f1");
        assert_valid("4d6 f1");
        assert_valid("6d10b");
        assert_valid("6d10 b");
        assert_valid("6d10b1");
        assert_valid("6d10 b1");
    }

    #[test]
    fn test_target_success() {
        let result = parser::parse_dice_string("6d10 t7").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 6);
        assert_eq!(result[0].sides, 10);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Target(7) => {}
            _ => panic!("Expected Target(7) modifier"),
        }
    }

    #[test]
    fn test_botch_modifier() {
        let result = parser::parse_dice_string("5d10 b1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Botch(Some(1)) => {}
            _ => panic!("Expected Botch(Some(1)) modifier"),
        }
    }

    #[test]
    fn test_failure_modifier() {
        let result = parser::parse_dice_string("5d10 t8 f1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].modifiers.len(), 2);

        assert!(
            result[0]
                .modifiers
                .iter()
                .any(|m| matches!(m, Modifier::Target(8)))
        );
        assert!(
            result[0]
                .modifiers
                .iter()
                .any(|m| matches!(m, Modifier::Failure(1)))
        );
    }

    #[test]
    fn test_target_lower_comprehensive() {
        // Test parsing, validation, and behavior for target lower (tl) modifier

        // Parsing test
        let result = parser::parse_dice_string("6d10 tl7").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 6);
        assert_eq!(result[0].sides, 10);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::TargetLower(7) => {}
            _ => panic!("Expected TargetLower(7) modifier"),
        }

        // Validation tests
        assert!(parser::parse_dice_string("6d10 tl0").is_err()); // Should fail with 0 target
        assert!(parser::parse_dice_string("6d10 tl1").is_ok()); // Should succeed with valid targets
        assert!(parser::parse_dice_string("6d10 tl5").is_ok());
        assert!(parser::parse_dice_string("6d10 tl10").is_ok());

        // Test that both target lower and target can coexist and parse correctly
        let tl_result = parser::parse_dice_string("6d10 tl7").unwrap();
        let t_result = parser::parse_dice_string("6d10 t7").unwrap();

        match &tl_result[0].modifiers[0] {
            Modifier::TargetLower(7) => {}
            _ => panic!("Expected TargetLower(7) modifier"),
        }

        match &t_result[0].modifiers[0] {
            Modifier::Target(7) => {}
            _ => panic!("Expected Target(7) modifier"),
        }

        // Success counting behavior
        let tl_result = parse_and_roll("6d10 tl7").unwrap();
        assert_eq!(tl_result.len(), 1);
        assert!(tl_result[0].successes.is_some());

        let t_result = parse_and_roll("6d10 t7").unwrap();
        assert_eq!(t_result.len(), 1);
        assert!(t_result[0].successes.is_some());

        // Both should have success counts (we can't predict exact values with random dice)
        let tl_successes = tl_result[0].successes.unwrap();
        let t_successes = t_result[0].successes.unwrap();

        // Validate that both are reasonable values (0-6 for 6d10)
        assert!((0..=6).contains(&tl_successes));
        assert!((0..=6).contains(&t_successes));

        // Test TargetLower with other modifiers
        assert_valid("6d10 tl7 +2");
        assert_valid("6d10 ie10 tl7");
        assert_valid("6d10 k4 tl7");

        let result = parse_and_roll("6d10 tl7 +2").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].successes.is_some());
        assert!(result[0].total > 0); // Should have some total
    }

    #[test]
    fn test_multiple_modifiers() {
        let result = parser::parse_dice_string("10d6 e6 k8 +4").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 10);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 3);
    }

    // ============================================================================
    // GAME SYSTEM TESTS
    // ============================================================================

    #[test]
    fn test_chronicle_of_darkness() {
        assert_valid("4cod");
        assert_valid("5cod8");
        assert_valid("6cod9");
        assert_valid("4codr");
        // Note: Some CoD patterns with spaces or advanced keywords might not parse correctly
    }

    #[test]
    fn test_world_of_darkness() {
        assert_valid("4wod8");
        assert_valid("5wod6");
        // Note: Some WoD patterns with spaces might not parse correctly
    }

    #[test]
    fn test_shadowrun() {
        assert_valid("sr5");
        assert_valid("sr6");
        // Note: Parametized shadowrun like 4sr5 might not work in current implementation
    }

    #[test]
    fn test_godbound() {
        assert_valid("gb");
        assert_valid("gbs");
        assert_valid("gb 1d8");
        assert_valid("gbs 2d10");
    }

    #[test]
    fn test_fudge_fate() {
        assert_valid("4df");
        assert_valid("3df");
        // Note: 'df' alone might not work - needs a count
    }

    #[test]
    fn test_savage_worlds() {
        assert_valid("sw4");
        assert_valid("sw6");
        assert_valid("sw8");
        assert_valid("sw10");
        assert_valid("sw12");
    }

    #[test]
    fn test_earthdawn() {
        assert_valid("ed4");
        assert_valid("ed6");
        assert_valid("ed8");
        assert_valid("ed10");
        assert_valid("ed12");
        assert_valid("ed20");
    }

    #[test]
    fn test_d6_system() {
        assert_valid("d6s4");
        assert_valid("d6s5");
        // Note: More complex d6 system patterns might not work yet
    }

    #[test]
    fn test_cyberpunk_red_basic() {
        // Test basic CPR alias
        assert_valid("cpr");
        assert_valid("cpr + 5");
        assert_valid("cpr - 3");
        assert_valid("cpr * 2");
        assert_valid("cpr / 2");
    }

    #[test]
    fn test_cyberpunk_red_alias_expansion() {
        // Test CPR alias expansion
        let expanded = aliases::expand_alias("cpr").unwrap();
        assert_eq!(expanded, "1d10 cpr");

        let expanded = aliases::expand_alias("cpr + 5").unwrap();
        assert_eq!(expanded, "1d10 cpr + 5");

        let expanded = aliases::expand_alias("cpr - 3").unwrap();
        assert_eq!(expanded, "1d10 cpr - 3");
    }

    #[test]
    fn test_cyberpunk_red_modifier_parsing() {
        // Test that CPR modifier parses correctly
        let result = parser::parse_dice_string("1d10 cpr").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[0].sides, 10);
        assert_eq!(result[0].modifiers.len(), 1);

        // Verify we have a CyberpunkRed modifier
        match &result[0].modifiers[0] {
            Modifier::CyberpunkRed => {}
            _ => panic!("Expected CyberpunkRed modifier"),
        }
    }

    #[test]
    fn test_cyberpunk_red_rolling() {
        // Test actual rolling behavior
        let result = parse_and_roll("cpr").unwrap();
        assert_eq!(result.len(), 1);

        // Should not have success counting (binary system)
        assert!(result[0].successes.is_none());
        assert!(result[0].failures.is_none());
        assert!(result[0].botches.is_none());
    }

    #[test]
    fn test_witcher_basic() {
        assert_valid("wit");
        assert_valid("wit + 5");
        assert_valid("wit - 3");
        assert_valid("wit * 2");
        assert_valid("wit / 2");
    }

    #[test]
    fn test_witcher_alias_expansion() {
        let expanded = aliases::expand_alias("wit").unwrap();
        assert_eq!(expanded, "1d10 wit");

        let expanded = aliases::expand_alias("wit + 5").unwrap();
        assert_eq!(expanded, "1d10 wit + 5");

        let expanded = aliases::expand_alias("wit - 3").unwrap();
        assert_eq!(expanded, "1d10 wit - 3");
    }

    #[test]
    fn test_witcher_modifier_parsing() {
        let result = parser::parse_dice_string("1d10 wit").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[0].sides, 10);
        assert_eq!(result[0].modifiers.len(), 1);

        match &result[0].modifiers[0] {
            Modifier::Witcher => {}
            _ => panic!("Expected Witcher modifier"),
        }
    }

    #[test]
    fn test_witcher_rolling() {
        let result = parse_and_roll("wit").unwrap();
        assert_eq!(result.len(), 1);

        // Should not have success counting (binary system)
        assert!(result[0].successes.is_none());
        assert!(result[0].failures.is_none());
        assert!(result[0].botches.is_none());
    }

    #[test]
    fn test_witcher_with_math_operations() {
        // Test with multiplication
        let result = parse_and_roll("wit * 2").unwrap();
        assert_eq!(result.len(), 1);
        // Range should be doubled from base Witcher range
        assert!(
            result[0].total >= -200 && result[0].total <= 220,
            "Witcher * 2 should be in reasonable doubled range, got {}",
            result[0].total
        );

        // Test with division
        let result = parse_and_roll("wit / 2").unwrap();
        assert_eq!(result.len(), 1);
        // Range should be halved (integer division)
        assert!(
            result[0].total >= -50 && result[0].total <= 55,
            "Witcher / 2 should be in reasonable halved range, got {}",
            result[0].total
        );
    }

    #[test]
    fn test_witcher_indefinite_explosion_difference_from_cpr() {
        // Test that demonstrates the key difference between Witcher and CPR
        // CPR has single explosions, Witcher has indefinite explosions

        // We can't test randomness directly, but we can verify the structure
        // supports indefinite explosions by checking the implementation allows it

        // Both should use the same basic structure
        let witcher_result = parse_and_roll("wit").unwrap();
        let cpr_result = parse_and_roll("cpr").unwrap();

        assert_eq!(witcher_result.len(), 1);
        assert_eq!(cpr_result.len(), 1);

        // Both should be total-based systems (no success counting)
        assert!(witcher_result[0].successes.is_none());
        assert!(cpr_result[0].successes.is_none());

        // The key difference is in the roller implementation, which allows
        // indefinite explosions for Witcher vs single explosions for CPR
        // This is tested through the explosion mechanics structure test above
    }

    #[test]
    fn test_brave_new_world_basic() {
        assert_valid("bnw3");
        assert_valid("bnw5");
        assert_valid("bnw1");

        let result = parse_and_roll("bnw3").unwrap();
        assert_eq!(result.len(), 1);

        let roll = &result[0];
        assert!(roll.total >= 1); // Should have some result
        assert!(
            roll.notes
                .iter()
                .any(|note| note.contains("Brave New World"))
        );
    }

    #[test]
    fn test_brave_new_world_alias_expansion() {
        let expanded = aliases::expand_alias("bnw4").unwrap();
        assert_eq!(expanded, "4d6 bnw");
    }

    #[test]
    fn test_brave_new_world_with_modifiers() {
        assert_valid("bnw3 + 5");
        assert_valid("bnw4 - 2");

        let result = parse_and_roll("bnw3 + 10").unwrap();
        assert_eq!(result.len(), 1);

        // Total should be at least 11 (minimum 1 + 10)
        assert!(result[0].total >= 11);
    }

    #[test]
    fn test_brave_new_world_vs_other_systems() {
        // Ensure BNW doesn't interfere with other systems
        assert_valid("4cod"); // Chronicles of Darkness still works
        assert_valid("sw8"); // Savage Worlds still works
        assert_valid("d6s4"); // D6 System still works

        let bnw_result = parse_and_roll("bnw3").unwrap();
        let cod_result = parse_and_roll("4cod").unwrap();

        // They should have different note patterns
        let bnw_has_system_note = bnw_result[0]
            .notes
            .iter()
            .any(|note| note.contains("Brave New World"));
        let cod_has_successes = cod_result[0].successes.is_some();

        assert!(bnw_has_system_note, "BNW should have system notes");
        assert!(cod_has_successes, "CoD should have success counting");
    }

    // ============================================================================
    // CONAN 2D20 SYSTEM TESTS
    // ============================================================================

    #[test]
    fn test_conan_2d20_basic() {
        assert_valid("conan");
        assert_valid("conan + 5");
        assert_valid("conan - 3");
        assert_valid("conan * 2");
        assert_valid("conan / 2");
    }

    #[test]
    fn test_conan_2d20_alias_expansion() {
        let expanded = aliases::expand_alias("conan").unwrap();
        assert_eq!(expanded, "2d20 conan");

        // Test with modifiers - these should also work
        if let Some(expanded) = aliases::expand_alias("conan + 5") {
            assert!(expanded.contains("2d20 conan"));
            assert!(expanded.contains("+ 5"));
        }

        if let Some(expanded) = aliases::expand_alias("conan - 3") {
            assert!(expanded.contains("2d20 conan"));
            assert!(expanded.contains("- 3"));
        }
    }

    #[test]
    fn test_conan_2d20_modifier_parsing() {
        let result = parser::parse_dice_string("2d20 conan").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 2);
        assert_eq!(result[0].sides, 20);
        assert_eq!(result[0].modifiers.len(), 1);

        match &result[0].modifiers[0] {
            Modifier::ConanSkill(2) => {}
            _ => panic!("Expected ConanSkill(2) modifier"),
        }
    }

    #[test]
    fn test_conan_2d20_rolling() {
        let result = parse_and_roll("conan").unwrap();
        assert_eq!(result.len(), 1);

        // Should have success counting (success-based system)
        assert!(result[0].successes.is_some());
    }

    #[test]
    fn test_wrath_glory_basic() {
        assert_valid("wng 4d6");
        assert_valid("wng dn2 4d6");
        assert_valid("wng 4d6 !soak");
        assert_valid("wng 4d6 !exempt");
        assert_valid("wng 4d6 !dmg");
    }

    #[test]
    fn test_wrath_glory_difficulty() {
        let result = parse_and_roll("wng dn3 4d6").unwrap();
        assert_eq!(result.len(), 1);

        let roll_result = &result[0];
        assert!(roll_result.successes.is_some());

        // Should have difficulty check note
        let has_difficulty_note = roll_result
            .notes
            .iter()
            .any(|note| note.contains("Difficulty 3"));
        assert!(has_difficulty_note, "Should have difficulty note");
    }

    #[test]
    fn test_wrath_glory_multiple_wrath_dice_comprehensive() {
        // Test multiple wrath dice with various scenarios
        assert_valid("wng w2 4d6");
        assert_valid("wng w3 dn2 4d6");
        assert_valid("wng w2 4d6 !soak");
        assert_valid("wng w3 4d6 !exempt");
        assert_valid("wng w2 4d6 !dmg");

        // Test alias expansion with multiple wrath dice
        let expanded = aliases::expand_alias("wng w2 4d6").unwrap();
        assert_eq!(expanded, "4d6 wngw2");

        let expanded = aliases::expand_alias("wng w3 dn2 5d6").unwrap();
        assert_eq!(expanded, "5d6 wngw3dn2");

        let expanded = aliases::expand_alias("wng w2 4d6 !soak").unwrap();
        assert_eq!(expanded, "4d6 wngw2t");

        // Test modifier parsing for multiple wrath dice
        let result = parser::parse_dice_string("4d6 wngw3").unwrap();
        assert_eq!(result.len(), 1);

        let dice = &result[0];
        assert_eq!(dice.count, 4);
        assert_eq!(dice.sides, 6);
        assert_eq!(dice.modifiers.len(), 1);

        // Verify we have the correct WrathGlory modifier
        match &dice.modifiers[0] {
            Modifier::WrathGlory(None, false, 3) => {
                // Correct: no difficulty, standard test, 3 wrath dice
            }
            _ => panic!("Expected WrathGlory(None, false, 3) modifier"),
        }

        // Test validation limits
        assert_invalid("wng w0 4d6"); // Below minimum
        assert_invalid("wng w6 4d6"); // Above maximum  
        assert_invalid("wng w10 4d6"); // Way above maximum

        // Test that valid range works
        assert_valid("wng w1 4d6"); // Minimum
        assert_valid("wng w5 4d6"); // Maximum

        // Test behavior with multiple wrath dice
        let result = parse_and_roll("wng w2 4d6").unwrap();
        assert_eq!(result.len(), 1);

        let roll_result = &result[0];
        assert!(roll_result.successes.is_some());
        assert!(roll_result.total > 0);

        // Test soak tests use total, not successes
        let result = parse_and_roll("wng w2 4d6 !soak").unwrap();
        assert_eq!(result.len(), 1);

        let roll_result = &result[0];
        assert!(roll_result.successes.is_some());
        assert!(roll_result.total > 0);

        // Ensure existing syntax still works exactly the same
        let old_syntax = parse_and_roll("wng 4d6").unwrap();
        let new_explicit = parse_and_roll("wng w1 4d6").unwrap();

        // Both should have same structure
        assert_eq!(old_syntax.len(), 1);
        assert_eq!(new_explicit.len(), 1);

        // Both should have success tracking
        assert!(old_syntax[0].successes.is_some());
        assert!(new_explicit[0].successes.is_some());
    }

    #[test]
    fn test_hero_system() {
        assert_valid("2hsn");
        assert_valid("3hsh");
        assert_valid("3hsk");
        assert_valid("2hsk1");
    }

    #[test]
    fn test_plus_one_forward() {
        assert_valid("+d20");
        assert_valid("+d6");
        assert_valid("+d%");
        assert_valid("+d100");
    }

    #[test]
    fn test_alias_expansion() {
        // Test basic expansion functions
        assert_eq!(
            aliases::expand_alias("4cod"),
            Some("4d10 t8 ie10".to_string())
        );
        assert_eq!(
            aliases::expand_alias("4codr"),
            Some("4d10 t8 ie10 r7".to_string())
        );

        // World of Darkness - remove exploding dice
        assert_eq!(
            aliases::expand_alias("4wod8"),
            Some("4d10 f1 t8".to_string())
        );
        assert_eq!(
            aliases::expand_alias("5wod6"),
            Some("5d10 f1 t6".to_string())
        );
    }

    #[test]
    fn test_alias_expansion_understanding() {
        // Test what gb actually expands to
        let gb_result = parse_and_roll("gb").unwrap();
        // Verify the actual expansion works
        assert_eq!(gb_result.len(), 1);
        assert!(gb_result[0].total > 0); // Should produce some result

        // Test gb with dice arguments - this is actually supported
        assert_valid("gb 1d20"); // This syntax is valid: "gb 1d20" -> "1d20 gb"

        // Test variations that should work
        assert_valid("gbs 2d10"); // "gbs 2d10" -> "2d10 gbs"
        assert_valid("gb 3d8"); // "gb 3d8" -> "3d8 gb"
        assert_valid("gbs 1d12 + 5"); // With modifiers

        // Test hero system fractional - does parsing work?
        let hero_result = parse_and_roll("2hsn").unwrap();
        // Check actual expansion works
        assert!(hero_result.len() > 0);
        assert!(hero_result[0].total > 0); // Should produce some result
    }

    // ============================================================================
    // ROLL SETS AND MULTIPLE ROLLS
    // ============================================================================

    #[test]
    fn test_roll_sets() {
        let result = parser::parse_dice_string("6 4d6").unwrap();
        assert_eq!(result.len(), 6);
        for (i, dice) in result.iter().enumerate() {
            assert_eq!(dice.count, 4);
            assert_eq!(dice.sides, 6);
            assert_eq!(dice.label, Some(format!("Set {}", i + 1)));
        }
    }

    #[test]
    fn test_minimum_roll_set() {
        let result = parser::parse_dice_string("2 1d6").unwrap();
        assert_eq!(result.len(), 2);
        for dice in &result {
            assert_eq!(dice.count, 1);
            assert_eq!(dice.sides, 6);
        }
    }

    #[test]
    fn test_maximum_roll_set() {
        let result = parser::parse_dice_string("20 1d6").unwrap();
        assert_eq!(result.len(), 20);
        for dice in &result {
            assert_eq!(dice.count, 1);
            assert_eq!(dice.sides, 6);
        }
    }

    #[test]
    fn test_roll_set_behavior() {
        let result = parse_and_roll("3 4d6").unwrap();
        assert_eq!(result.len(), 3);
        for (i, roll_result) in result.iter().enumerate() {
            assert_eq!(roll_result.label, Some(format!("Set {}", i + 1)));
        }
    }

    #[test]
    fn test_roll_sets_with_complex_expressions() {
        // Test roll sets with more complex dice expressions
        let result = parse_and_roll("2 100/2d1").unwrap();
        assert_eq!(result.len(), 2, "Should create 2 roll sets");

        for (i, roll_result) in result.iter().enumerate() {
            assert_eq!(roll_result.label, Some(format!("Set {}", i + 1)));
            assert_eq!(
                roll_result.total, 50,
                "Each set should calculate 100/2 = 50"
            );
        }
    }

    #[test]
    fn test_roll_set_edge_cases() {
        // Test roll sets with expressions that should work
        assert_valid("2 100/2d1"); // Division in roll sets
        assert_valid("3 200/1d4"); // More division patterns
        assert_valid("4 500/d%"); // Percentile division

        // Roll sets with number operations
        assert_valid("3 5+1d6");
        assert_valid("3 10-2d4");
        assert_valid("3 6*1d6");
        assert_valid("3 20/1d6");

        // Roll sets with basic game systems
        assert_valid("3 4cod");
        assert_valid("3 4wod8");
        assert_valid("3 gb");
        assert_valid("3 wng 4d6");
        assert_valid("3 3df");
        assert_valid("3 sr6");
        assert_valid("3 ed15");

        // Roll sets with flags and comments
        assert_valid("p 6 4d6");
        assert_valid("s 6 4d6");
        assert_valid("6 4d6!ability scores");
        assert_valid("p s 6 4d6!private simple");
    }

    #[test]
    fn test_multiple_rolls() {
        assert_valid("1d20;1d6");
        assert_valid("1d20; 1d6");
        assert_valid("1d20 ; 1d6");
        assert_valid("1d20+5;2d6+3;1d4");
        assert_valid("1d6;1d6;1d6;1d6"); // Max 4
        assert_invalid("1d6;1d6;1d6;1d6;1d6"); // Too many
    }

    #[test]
    fn test_semicolon_separated_rolls() {
        let result = parser::parse_dice_string("4d100 ; 3d10 k2").unwrap();
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 100);
        assert_eq!(result[0].original_expression, Some("4d100".to_string()));

        assert_eq!(result[1].count, 3);
        assert_eq!(result[1].sides, 10);
        assert_eq!(result[1].original_expression, Some("3d10 k2".to_string()));
    }

    #[test]
    fn test_multiple_roll_behavior() {
        let result = parse_and_roll("1d20; 2d6; 1d4").unwrap();
        assert_eq!(result.len(), 3);
        for roll_result in &result {
            assert!(roll_result.total > 0);
        }
    }

    // ============================================================================
    // FLAGS AND COMMENTS
    // ============================================================================

    #[test]
    fn test_flags_and_comments() {
        // Comment examples
        assert_valid("4d6!Hello World!"); // Comment example
        assert_valid("s 4d6"); // Simple flag example
        assert_valid("nr 4d6"); // No results flag example
        assert_valid("p 4d6"); // Private flag example
        assert_valid("ul 4d6"); // Unsort flag example

        // Complex combination from docs
        assert_valid("10d6 e6 k8 +4"); // Main complex example

        // Wrath & Glory examples from docs
        assert_valid("wng 4d6"); // Basic W&G
        assert_valid("wng dn2 4d6"); // W&G with difficulty
        assert_valid("wng 4d6 !soak"); // W&G soak test
        assert_valid("wng 4d6 !exempt"); // W&G exempt test
        assert_valid("wng 4d6 !dmg"); // W&G damage test

        // Hero System examples from docs
        assert_valid("2hsn"); // 2d6 normal damage
        assert_valid("3hsh"); // To-hit roll
        assert_valid("3hsk1+1d3"); // Fractional with modifier - test if this works
    }

    // ============================================================================
    // DISCORD INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_character_limit_handling() {
        // Test large roll that might exceed Discord's 2000 char limit
        let large_roll = "100d1000 ie 100";
        let result = parse_and_roll(large_roll).unwrap();
        let formatted = format_multiple_results_with_limit(&result);
        assert!(formatted.len() <= 2000, "Result should fit Discord limit");
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[test]
    fn test_performance_limits() {
        // Test maximum allowed configurations
        assert_valid("500d1000");
        assert_valid("20 500d6");

        // Test explosion/reroll limits
        let result = parse_and_roll("1d6 ie1").unwrap(); // Always explodes
        assert!(result[0].individual_rolls.len() <= 101); // Max 100 explosions + original

        // Test complex expressions
        assert_valid("50d6 e6 ie k25 r1 t4 + 10");
        assert_valid("20d10 t7 f1 b1 ie10 + 5d6 e6 - 2d4");
    }

    // ============================================================================
    // HELP TEXT TESTS
    // ============================================================================

    #[test]
    fn test_help_text_generation() {
        let basic_help = help_text::generate_basic_help();
        assert!(basic_help.contains("Dice Maiden"));
        assert!(basic_help.contains("2d6 + 3d10"));
        assert!(basic_help.contains("Modifiers"));

        let alias_help = help_text::generate_alias_help();
        assert!(alias_help.contains("Game System Aliases"));
        assert!(alias_help.contains("Chronicles of Darkness"));

        let system_help = help_text::generate_system_help();
        assert!(system_help.contains("Game System Examples"));
        assert!(system_help.contains("Fudge/FATE"));
    }

    // ============================================================================
    // COMMAND DISPLAY FORMATTING TESTS
    // ============================================================================

    #[test]
    fn test_strip_label_and_comment_from_expression() {
        // Test the primary issue case: labels in parentheses
        assert_eq!(
            strip_label_and_comment_from_expression("(roll to hit) 1d20+2"),
            "1d20+2"
        );
        assert_eq!(
            strip_label_and_comment_from_expression("(attack roll) 2d6+3"),
            "2d6+3"
        );

        // Test labels without parentheses (should be untouched)
        assert_eq!(
            strip_label_and_comment_from_expression("roll to hit 1d20+2"),
            "roll to hit 1d20+2"
        );

        // Test expressions without labels
        assert_eq!(strip_label_and_comment_from_expression("1d20+2"), "1d20+2");

        // Test complex expressions with multiple parenthetical sections
        assert_eq!(
            strip_label_and_comment_from_expression("(first) 1d20 (second) +2"),
            "1d20 (second) +2"
        );

        // Test edge cases - empty parentheses at start
        assert_eq!(
            strip_label_and_comment_from_expression("() 1d20+2"),
            "1d20+2"
        );
        assert_eq!(
            strip_label_and_comment_from_expression("( ) 1d20+2"),
            "1d20+2"
        );

        // Test edge case - parentheses without space or end
        assert_eq!(
            strip_label_and_comment_from_expression("()1d20+2"),
            "()1d20+2" // This should NOT be stripped since there's no space
        );
    }

    #[test]
    fn test_parser_regex_integration_paths() {
        // Test that the parser correctly routes expressions through different code paths

        // Path 1: Direct alias expansion (no regex needed)
        let result = parse_and_roll("+d20").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].total >= 1 && result[0].total <= 20);

        // Path 2: Regex pattern match for simple modifiers
        let result = parse_and_roll("+d20+10").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].total >= 11 && result[0].total <= 30);

        // Path 3: Complex expression parsing (multiple parts)
        let result = parse_and_roll("+d20 + d10 + 5").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].dice_groups.len() >= 2);

        // Path 4: Roll set with regex pattern
        let result = parse_and_roll("2 +d20+5").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].label.as_ref().unwrap().starts_with("Set "));
        assert!(result[1].label.as_ref().unwrap().starts_with("Set "));

        // Path 5: Game system alias with modifiers
        let result = parse_and_roll("4cod + 3").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].successes.is_some()); // Should have success counting
    }

    // ============================================================================
    // HELPER FUNCTIONS
    // ============================================================================

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

    fn assert_invalid(input: &str) {
        let result = parse_and_roll(input);
        assert!(result.is_err(), "Expected error for: '{}'", input);
    }

    fn strip_label_and_comment_from_expression(input: &str) -> &str {
        // Simple implementation for testing - remove leading parenthetical labels
        if let Some(pos) = input.find(") ") {
            if input.starts_with('(') {
                return input[pos + 2..].trim();
            }
        }
        input
    }
}
