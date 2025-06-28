// tests/dice_tests.rs - Comprehensive Dice Maiden test suite
use dicemaiden_rs::{
    dice::{aliases, parser, roller, DiceRoll, HeroSystemType, Modifier, RollResult},
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
        // Test left-to-right vs PEMDAS differences using dice expressions

        // Use 1d1 (always rolls 1) + math to test left-to-right evaluation
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
    }

    #[test]
    fn test_left_to_right_with_dice() {
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
            _ => panic!("Expected Add modifier"),
        }
    }

    #[test]
    fn test_dice_multiplication_and_division() {
        // Test dice multiplication and division that were previously failing
        assert_valid("3d6 * 2d6");
        assert_valid("3d6 / 2d6");
        assert_valid("6d8 * 1d4");
        assert_valid("4d10 / 2d6");

        // Test with whitespace variations
        assert_valid("3d6*2d6");
        assert_valid("3d6 *2d6");
        assert_valid("3d6* 2d6");
        assert_valid("3d6 * 2d6");
        assert_valid("6d8/2d4");
        assert_valid("6d8 / 2d4");
    }

    #[test]
    fn test_dice_multiplication_parsing() {
        // Test that multiplication with dice creates the correct modifier
        let result = parser::parse_dice_string("3d6 * 2d4").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 3);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 1);

        // Verify we have a MultiplyDice modifier
        match &result[0].modifiers[0] {
            Modifier::MultiplyDice(dice_roll) => {
                assert_eq!(dice_roll.count, 2);
                assert_eq!(dice_roll.sides, 4);
            }
            _ => panic!("Expected MultiplyDice modifier"),
        }
    }

    #[test]
    fn test_dice_division_parsing() {
        // Test that division with dice creates the correct modifier
        let result = parser::parse_dice_string("6d8 / 3d6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 6);
        assert_eq!(result[0].sides, 8);
        assert_eq!(result[0].modifiers.len(), 1);

        // Verify we have a DivideDice modifier
        match &result[0].modifiers[0] {
            Modifier::DivideDice(dice_roll) => {
                assert_eq!(dice_roll.count, 3);
                assert_eq!(dice_roll.sides, 6);
            }
            _ => panic!("Expected DivideDice modifier"),
        }
    }

    #[test]
    fn test_dice_multiplication_with_fixed_dice() {
        // Use 1d1 for predictable results (always rolls 1)
        let result = parse_and_roll("3d1 * 2d1").unwrap();
        assert_eq!(result.len(), 1);

        // 3d1 = 3, 2d1 = 2, so 3 * 2 = 6
        assert_eq!(result[0].total, 6, "3d1 * 2d1 should equal 6");

        // Verify dice groups are created correctly
        assert_eq!(result[0].dice_groups.len(), 2, "Should have 2 dice groups");
        assert_eq!(result[0].dice_groups[0].modifier_type, "base");
        assert_eq!(result[0].dice_groups[1].modifier_type, "multiply");
    }

    #[test]
    fn test_dice_division_with_fixed_dice() {
        // Use 1d1 for predictable results
        let result = parse_and_roll("8d1 / 2d1").unwrap();
        assert_eq!(result.len(), 1);

        // 8d1 = 8, 2d1 = 2, so 8 / 2 = 4
        assert_eq!(result[0].total, 4, "8d1 / 2d1 should equal 4");

        // Verify dice groups are created correctly
        assert_eq!(result[0].dice_groups.len(), 2, "Should have 2 dice groups");
        assert_eq!(result[0].dice_groups[0].modifier_type, "base");
        assert_eq!(result[0].dice_groups[1].modifier_type, "divide");
    }

    #[test]
    fn test_dice_operations_display_format() {
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
        assert_eq!(result[0].total, 10, "Should be 20 / 2 = 10");

        // Ensure normal roll sets still work
        let result = parse_and_roll("4 2d1").unwrap();
        assert_eq!(result.len(), 4, "Should create 4 roll sets");
        for roll in &result {
            assert_eq!(roll.total, 2, "Each set should be 2d1 = 2");
        }

        // Ensure normal mathematical operations work
        let result = parse_and_roll("4 + 2d1").unwrap();
        assert_eq!(result.len(), 1, "Should be single mathematical expression");
        assert_eq!(result[0].total, 6, "Should be 4 + 2 = 6");

        let result = parse_and_roll("5-2d1").unwrap();
        assert_eq!(
            result.len(),
            1,
            "Should be a single mathematical expression"
        );
        assert_eq!(result[0].total, 3, "Should be 5-2= 3");

        // Complex expressions
        let result = parse_and_roll("100/2d1 + 5").unwrap();
        assert_eq!(result.len(), 1, "Should be single expression");
        assert_eq!(result[0].total, 55, "Should be (100/2) + 5 = 55");

        // Test whitespace variations produce same result
        let result1 = parse_and_roll("100/2d1+5").unwrap();
        let result2 = parse_and_roll("100/2d1 +5").unwrap();
        let result3 = parse_and_roll("100 / 2d1 + 5").unwrap();

        assert_eq!(result1[0].total, 55, "100/2d1+5 should equal 55");
        assert_eq!(result2[0].total, 55, "100/2d1 +5 should equal 55");
        assert_eq!(result3[0].total, 55, "100 / 2d1 + 5 should equal 55");

        // Verify precedence with existing documented syntax
        assert_valid("10d6 e6 k8 +4");

        // New syntax combined with existing modifiers
        assert_valid("100/2d6 e6");
        assert_valid("3 100/2d6 k1");
    }

    #[test]
    fn test_number_divided_by_dice() {
        // Test "200/2d4" style expressions where a number is divided by dice result
        // Use 1d1 to get predictable results (always rolls 1)

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
            _ => panic!("Expected ExplodeIndefinite(Some(6)) modifier"),
        }
    }

    #[test]
    fn test_keep_drop() {
        assert_valid("4d6k3");
        assert_valid("4d6 k3");
        assert_valid("4d6kl2");
        assert_valid("4d6 kl2");
        assert_valid("4d6d1");
        assert_valid("4d6 d1");

        // Edge cases
        assert_valid("3d6k5"); // Keep more than available
        assert_invalid("1d6k0"); // Keep zero
    }

    #[test]
    fn test_keep_highest() {
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
    fn test_keep_lowest() {
        let result = parser::parse_dice_string("4d6 kl2").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::KeepLow(2) => {}
            _ => panic!("Expected KeepLow(2) modifier"),
        }
    }

    #[test]
    fn test_drop_dice() {
        let result = parser::parse_dice_string("3d10 d1").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 3);
        assert_eq!(result[0].sides, 10);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Drop(1) => {}
            _ => panic!("Expected Drop(1) modifier"),
        }
    }

    #[test]
    fn test_reroll() {
        assert_valid("4d6r1");
        assert_valid("4d6 r1");
        assert_valid("4d6ir1");
        assert_valid("4d6 ir1");
    }

    #[test]
    fn test_reroll_dice() {
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

        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::Target(8))));
        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::Failure(1))));
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
    // FLAGS
    // ============================================================================

    #[test]
    fn test_flags() {
        assert_valid("p 1d6");
        assert_valid("s 1d6");
        assert_valid("nr 1d6");
        assert_valid("ul 1d6");
        assert_valid("p s ul 1d6");
        assert_valid("ul nr 4 1d20");

        // With complex expressions
        assert_valid("p 4d6k3+2");
        assert_valid("s 2d6e6");
    }

    #[test]
    fn test_private_flag() {
        let result = parser::parse_dice_string("p 4d6").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].private);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
    }

    #[test]
    fn test_simple_flag() {
        let result = parser::parse_dice_string("s 4d6").unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].simple);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
    }

    #[test]
    fn test_flag_behavior() {
        let result = parse_and_roll("p 1d6").unwrap();
        assert!(result[0].private);

        let result = parse_and_roll("s 1d6").unwrap();
        assert!(result[0].simple);

        let result = parse_and_roll("nr 1d6").unwrap();
        assert!(result[0].no_results);
    }

    // ============================================================================
    // COMMENTS AND LABELS
    // ============================================================================

    #[test]
    fn test_comments_labels() {
        assert_valid("1d6!attack");
        assert_valid("1d6 ! attack");
        assert_valid("1d6 ! attack roll");
        assert_valid("(Attack) 1d20+5");
        assert_valid("(Attack Roll) 1d20+5 ! with sword");
        assert_valid("1d6!"); // Empty comment
        assert_valid("() 1d6"); // Empty label
    }

    #[test]
    fn test_comment_parsing() {
        let result = parser::parse_dice_string("4d6 ! Hello World!").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].comment, Some("Hello World!".to_string()));
    }

    #[test]
    fn test_label_parsing() {
        let result = parser::parse_dice_string("(Attack Roll) 1d20 + 5").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].label, Some("Attack Roll".to_string()));
    }

    #[test]
    fn test_comment_label_parsing() {
        let result = parse_and_roll("1d6 ! test comment").unwrap();
        assert_eq!(result[0].comment, Some("test comment".to_string()));

        let result = parse_and_roll("(Test Label) 1d6").unwrap();
        assert_eq!(result[0].label, Some("Test Label".to_string()));
    }

    #[test]
    fn test_comment_edge_cases() {
        // Comments with special characters that should work
        assert_valid("1d6!test@example.com");
        assert_valid("1d6!20% chance");
        assert_valid("1d6!cost: $15");
        assert_valid("1d6!damage (fire)");
        assert_valid("1d6!test & more");

        // Reasonable length comments
        assert_valid("1d6!this is a long comment that tests parsing");

        // Comments with numbers
        assert_valid("1d6!roll number 42");
        assert_valid("1d6!3rd attempt");
        assert_valid("1d6!level 15 spell");
    }

    #[test]
    fn test_label_edge_cases() {
        // Labels with reasonable special characters
        assert_valid("(Test 123) 1d6");
        assert_valid("(Roll #5) 1d6");
        assert_valid("(Level-Up) 1d6");
        assert_valid("(Test_Case) 1d6");

        // Reasonable length labels
        assert_valid("(Long Label Name) 1d6");

        // Labels with numbers
        assert_valid("(Attack 1) 1d20");
        assert_valid("(Round 15) 1d6");
        assert_valid("(3rd Save) 1d20");

        // Labels with mixed chars and numbers
        assert_valid("(Test-123) 1d6");
        assert_valid("(Final_Test) 1d6");
    }

    // ============================================================================
    // ROLL SETS AND MULTIPLE ROLLS
    // ============================================================================

    #[test]
    fn test_roll_sets() {
        assert_valid("6 4d6");
        assert_valid("3 1d20+5");
        assert_valid("2 1d6"); // Minimum
        assert_valid("20 1d6"); // Maximum
        assert_invalid("1 1d6"); // Below minimum
        assert_invalid("21 1d6"); // Above maximum
    }

    #[test]
    fn test_roll_set() {
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
            assert!(roll_result.original_expression.is_some());
        }
    }

    #[test]
    fn test_semicolon_combinations() {
        // Complex semicolon combinations with different systems
        assert_valid("4cod; gb; wng 4d6");
        assert_valid("attack+5; 3df; sr6; ed15");
        assert_valid("dndstats; +d20+10; 2d6+3; 1d4");

        // Semicolon with flags (each roll can have different flags)
        assert_valid("p 1d20; s 2d6; ul 1d4; nr 1d8");
        assert_valid("p 1d20+5; 2d6+3; s 1d4; 1d8");

        // Semicolon with complex expressions and comments (no semicolon in comments)
        assert_valid("(Attack) 1d20+5!melee; (Damage) 2d6+3!sword; (Crit) 1d6!extra");
        assert_valid("10d6e6k8+4!fireball; 6d10t7!shadowrun; 4d6k3!ability");
    }

    #[test]
    fn test_roll_sets_advantage_with_mathematical_modifiers() {
        // Test roll sets with advantage/disadvantage patterns that have mathematical modifiers
        // This specific combination is not covered by existing tests

        // Test advantage with subtraction in roll sets
        let result = parse_and_roll("4 +d20 -2").unwrap();
        assert_eq!(result.len(), 4, "Should create 4 roll sets");
        for (i, roll) in result.iter().enumerate() {
            assert_eq!(roll.label, Some(format!("Set {}", i + 1)));
            // Each should be advantage d20 (1-20) minus 2, so range -1 to 18
            assert!(
                roll.total >= -1 && roll.total <= 18,
                "Roll {} should be in range for advantage d20 - 2, got {}",
                i + 1,
                roll.total
            );
        }

        // Test advantage with multiplication in roll sets
        let result = parse_and_roll("3 +d20*2").unwrap();
        assert_eq!(result.len(), 3, "Should create 3 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            // Each should be advantage d20 (1-20) times 2, so range 2 to 40
            assert!(
                roll.total >= 2 && roll.total <= 40,
                "Roll should be in range for advantage d20 * 2, got {}",
                roll.total
            );
        }

        // Test disadvantage with addition in roll sets
        let result = parse_and_roll("2 -d% +10").unwrap();
        assert_eq!(result.len(), 2, "Should create 2 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            // Each should be disadvantage d% (1-100) plus 10, so range 11 to 110
            assert!(
                roll.total >= 11 && roll.total <= 110,
                "Roll should be in range for disadvantage d% + 10, got {}",
                roll.total
            );
        }
    }

    #[test]
    fn test_roll_sets_vs_single_expression_with_modifiers_distinction() {
        // Test the critical distinction between roll sets and single mathematical expressions
        // when both involve advantage patterns with modifiers

        // ROLL SET: "3 +d20 +5" should create 3 sets of (advantage d20 + 5)
        let roll_sets = parse_and_roll("3 +d20 +5").unwrap();
        assert_eq!(roll_sets.len(), 3, "Should create 3 roll sets");
        assert!(
            roll_sets
                .iter()
                .all(|r| r.label.is_some() && r.label.as_ref().unwrap().contains("Set")),
            "All results should have 'Set X' labels"
        );

        // SINGLE EXPRESSION: "+d20 +5" should create 1 advantage roll with +5 modifier
        let single_expr = parse_and_roll("+d20 +5").unwrap();
        assert_eq!(single_expr.len(), 1, "Should create 1 single expression");
        assert!(
            single_expr[0].label.is_none(),
            "Single expression should not have set label"
        );

        // SINGLE COMPLEX: "+d20 + d10 +5" should create 1 complex expression
        let complex_expr = parse_and_roll("+d20 + d10 +5").unwrap();
        assert_eq!(complex_expr.len(), 1, "Should create 1 complex expression");
        assert!(
            complex_expr[0].label.is_none(),
            "Complex expression should not have set label"
        );
        assert!(
            complex_expr[0].dice_groups.len() >= 2,
            "Should have multiple dice groups"
        );

        // Verify the parser correctly distinguishes these patterns
        assert!(
            roll_sets.len() > single_expr.len(),
            "Roll sets should create more results than single expressions"
        );
        assert!(
            roll_sets.len() > complex_expr.len(),
            "Roll sets should create more results than complex expressions"
        );
    }

    #[test]
    fn test_roll_sets_advantage_modifier_edge_cases() {
        // Test edge cases for roll sets with advantage patterns and modifiers
        // that could break the parser

        // Test with different operators
        let operators_tests = [
            ("2 +d20-1", 2),  // No space before operator
            ("2 +d20 *3", 2), // Space before operator
            ("2 +d20/ 2", 2), // Space after operator
            ("2 -d%+15", 2),  // No space, addition
            ("2 -d% *2", 2),  // Space, multiplication
        ];

        for (expr, expected_count) in &operators_tests {
            let result = parse_and_roll(expr);
            assert!(result.is_ok(), "'{}' should parse successfully", expr);

            let results = result.unwrap();
            assert_eq!(
                results.len(),
                *expected_count,
                "'{}' should create {} roll sets",
                expr,
                expected_count
            );

            // Verify each has a set label (confirming it's a roll set, not single expression)
            for roll in &results {
                assert!(
                    roll.label.as_ref().unwrap().starts_with("Set "),
                    "'{}' should create roll sets with labels",
                    expr
                );
            }
        }
    }

    #[test]
    fn test_roll_sets_advantage_with_flags_and_modifiers() {
        // Test roll sets that combine flags, advantage patterns, and mathematical modifiers
        // This specific combination is not covered elsewhere

        // Private roll sets with advantage and modifiers
        let result = parse_and_roll("p 3 +d20 +2").unwrap();
        assert_eq!(result.len(), 3, "Should create 3 roll sets");
        for roll in &result {
            assert!(roll.private, "Each roll should be private");
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            // Should be advantage d20 (1-20) + 2, so range 3 to 22
            assert!(roll.total >= 3 && roll.total <= 22);
        }

        // Simple output roll sets with disadvantage and modifiers
        let result = parse_and_roll("s 2 -d% -5").unwrap();
        assert_eq!(result.len(), 2, "Should create 2 roll sets");
        for roll in &result {
            assert!(roll.simple, "Each roll should have simple flag");
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            // Should be disadvantage d% (1-100) - 5, so range -4 to 95
            assert!(roll.total >= -4 && roll.total <= 95);
        }
    }

    #[test]
    fn test_roll_set_advantage_pattern_comprehensive() {
        // Test comprehensive combinations of roll sets with advantage patterns
        // This ensures all parser paths work correctly

        let test_cases = [
            // Format: (expression, expected_count, min_total, max_total)
            ("2 +d20+5", 2, 6, 25),  // 2 sets of (adv d20 + 5)
            ("3 -d20-3", 3, -2, 17), // 3 sets of (dis d20 - 3)
            ("4 +d%*2", 4, 2, 200),  // 4 sets of (adv d% * 2)
            ("5 -d%/10", 5, 0, 10),  // 5 sets of (dis d% / 10)
        ];

        for (expr, expected_count, min_total, max_total) in &test_cases {
            let result = parse_and_roll(expr).unwrap();
            assert_eq!(
                result.len(),
                *expected_count,
                "Expression '{}' should create {} roll sets",
                expr,
                expected_count
            );

            for (i, roll) in result.iter().enumerate() {
                assert_eq!(
                    roll.label,
                    Some(format!("Set {}", i + 1)),
                    "Roll {} should have correct label",
                    i + 1
                );
                assert!(
                    roll.total >= *min_total && roll.total <= *max_total,
                    "Roll {} total {} should be in range {}-{}",
                    i + 1,
                    roll.total,
                    min_total,
                    max_total
                );
            }
        }
    }

    #[test]
    fn test_roll_set_advantage_with_complex_modifiers() {
        // Test roll sets with advantage patterns that have more complex scenarios

        // Test with flags
        let result = parse_and_roll("p 2 +d20*3").unwrap();
        assert_eq!(result.len(), 2, "Should create 2 private roll sets");
        for roll in &result {
            assert!(roll.private, "Each roll should be private");
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            assert!(roll.total >= 3 && roll.total <= 60); // adv d20 * 3
        }

        // Test with different advantage types
        let result = parse_and_roll("3 -d%+50").unwrap();
        assert_eq!(result.len(), 3, "Should create 3 disadvantage roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            assert!(roll.total >= 51 && roll.total <= 150); // dis d% + 50
        }
    }

    #[test]
    fn test_roll_set_advantage_pattern_error_prevention() {
        // Test that roll set advantage patterns don't interfere with other valid syntax

        // Should still work: basic roll sets without advantage
        let result = parse_and_roll("4 2d6").unwrap();
        assert_eq!(result.len(), 4, "Basic roll sets should still work");

        // Should still work: single advantage without numbers
        let result = parse_and_roll("+d20").unwrap();
        assert_eq!(result.len(), 1, "Single advantage should still work");
        assert!(
            result[0].label.is_none(),
            "Single advantage should not have set label"
        );

        // Should still work: complex advantage expressions
        let result = parse_and_roll("+d20 + d10 + 5").unwrap();
        assert_eq!(result.len(), 1, "Complex advantage should still work");
        assert!(
            result[0].dice_groups.len() >= 2,
            "Should have multiple dice groups"
        );
    }

    // ============================================================================
    // GAME SYSTEM ALIASES
    // ============================================================================

    #[test]
    fn test_advantage_disadvantage_regex_pattern_specificity() {
        // Test that the regex correctly handles all mathematical operators
        // These should be parsed as single advantage/disadvantage expressions with modifiers

        // Addition patterns
        assert_valid("+d20+1");
        assert_valid("+d20 + 1");
        assert_valid("-d%+15");
        assert_valid("-d% + 15");

        // Subtraction patterns
        assert_valid("+d20-3");
        assert_valid("+d20 - 3");
        assert_valid("-d%-8");
        assert_valid("-d% - 8");

        // Multiplication patterns
        assert_valid("+d20*4");
        assert_valid("+d20 * 4");
        assert_valid("-d%*2");
        assert_valid("-d% * 2");

        // Division patterns
        assert_valid("+d20/2");
        assert_valid("+d20 / 2");
        assert_valid("-d%/5");
        assert_valid("-d% / 5");
    }

    #[test]
    fn test_advantage_disadvantage_regex_boundary_conditions() {
        // Test edge cases for the regex pattern

        // Should match: single operator with number
        let result = parse_and_roll("+d20*10").unwrap();
        assert_eq!(result.len(), 1, "Should create single expression");
        assert!(result[0].label.is_none(), "Should not have set label");

        // Should NOT match: multiple operators (complex expressions)
        let result = parse_and_roll("+d20 + 5 - 2").unwrap();
        assert_eq!(result.len(), 1, "Should still parse but via different path");

        // Should NOT match: additional dice
        let result = parse_and_roll("+d20 + d10").unwrap();
        assert_eq!(result.len(), 1, "Should parse but not via simple regex");
        assert!(
            result[0].dice_groups.len() >= 2,
            "Should have multiple dice groups"
        );
    }

    #[test]
    fn test_advantage_disadvantage_regex_vs_roll_sets() {
        // Test the critical distinction between single expressions and roll sets

        // Single expression (should use regex shortcut)
        let single = parse_and_roll("+d20+5").unwrap();
        assert_eq!(single.len(), 1, "Should be single expression");
        assert!(single[0].label.is_none(), "Should not have set label");

        // Roll set (should create multiple results)
        let roll_set = parse_and_roll("3 +d20+5").unwrap();
        assert_eq!(roll_set.len(), 3, "Should be roll set");
        assert!(roll_set[0].label.is_some(), "Should have set labels");

        // Verify they produce different results but same individual calculations
        // Both should calculate advantage d20 + 5, but one creates sets
        for roll in &roll_set {
            // Each roll set item should be in same range as single expression
            assert!(
                roll.total >= 3 && roll.total <= 25,
                "Should be in range for adv d20 + 5"
            );
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
        }

        assert!(
            single[0].total >= 3 && single[0].total <= 25,
            "Should be in range for adv d20 + 5"
        );
    }

    #[test]
    fn test_advantage_disadvantage_regex_whitespace_handling() {
        // Test that the regex handles various whitespace patterns correctly

        let whitespace_variants = [
            "+d20+5",     // No spaces
            "+d20 +5",    // Space before operator
            "+d20+ 5",    // Space after operator
            "+d20 + 5",   // Spaces around operator
            "+d20  +  5", // Multiple spaces
            "+d20\t+\t5", // Tabs
        ];

        for variant in &whitespace_variants {
            let result = parse_and_roll(variant).unwrap();
            assert_eq!(
                result.len(),
                1,
                "Variant '{}' should parse as single expression",
                variant
            );
            assert!(
                result[0].label.is_none(),
                "Variant '{}' should not have set label",
                variant
            );
            // All should produce the same calculation (advantage d20 + 5)
            assert!(
                result[0].total >= 3 && result[0].total <= 25,
                "Variant '{}' should be in correct range",
                variant
            );
        }
    }

    #[test]
    fn test_advantage_disadvantage_regex_operator_precedence() {
        // Test that different operators work correctly in the regex pattern

        // Test each operator with predictable dice (using d20 range validation)
        let operators = [
            ("+d20+10", 11, 30), // advantage (1-20) + 10 = 11-30
            ("+d20-5", -4, 15),  // advantage (1-20) - 5 = -4-15
            ("+d20*3", 3, 60),   // advantage (1-20) * 3 = 3-60
            ("+d20/2", 0, 10),   // advantage (1-20) / 2 = 0-10 (integer division)
            ("-d20+15", 16, 35), // disadvantage (1-20) + 15 = 16-35
            ("-d20-2", -1, 18),  // disadvantage (1-20) - 2 = -1-18
            ("-d20*2", 2, 40),   // disadvantage (1-20) * 2 = 2-40
            ("-d20/4", 0, 5),    // disadvantage (1-20) / 4 = 0-5
        ];

        for (expr, min_expected, max_expected) in &operators {
            let result = parse_and_roll(expr).unwrap();
            assert_eq!(
                result.len(),
                1,
                "Expression '{}' should create single result",
                expr
            );
            assert!(
                result[0].total >= *min_expected && result[0].total <= *max_expected,
                "Expression '{}' got {}, expected range {}-{}",
                expr,
                result[0].total,
                min_expected,
                max_expected
            );
        }
    }

    #[test]
    fn test_dnd_aliases() {
        assert_valid("dndstats");
        assert_valid("attack");
        assert_valid("skill");
        assert_valid("save");
        assert_valid("attack+5");
        assert_valid("attack + 5");
        assert_valid("+d20"); // Advantage
        assert_valid("-d20"); // Disadvantage
        assert_valid("+d%"); // Percentile advantage
        assert_valid("-d%"); // Percentile disadvantage
    }

    #[test]
    fn test_advantage_alias() {
        let expanded = aliases::expand_alias("+d20").unwrap();
        assert_eq!(expanded, "2d20 k1");
    }

    #[test]
    fn test_disadvantage_alias() {
        let expanded = aliases::expand_alias("-d20").unwrap();
        assert_eq!(expanded, "2d20 kl1");
    }

    #[test]
    fn test_dnd_stats_alias() {
        let expanded = aliases::expand_alias("dndstats").unwrap();
        assert_eq!(expanded, "6 4d6 k3");
    }

    #[test]
    fn test_percentile_advantage() {
        let expanded = aliases::expand_alias("+d%").unwrap();
        assert_eq!(expanded, "2d10 kl1 * 10 + 1d10 - 10");
    }

    #[test]
    fn test_percentile_disadvantage() {
        let expanded = aliases::expand_alias("-d%").unwrap();
        assert_eq!(expanded, "2d10 k1 * 10 + 1d10 - 10");
    }

    #[test]
    fn test_percentile_disadvantage_with_modifier() {
        assert_valid("-d% - 2");
    }

    #[test]
    fn test_advantage_with_modifier() {
        //"+d20 + 1" should parse successfully instead of
        // throwing "Invalid dice expression: +" error
        let result = parser::parse_dice_string("+d20 + 1").unwrap();
        assert_eq!(result.len(), 1);

        let dice = &result[0];

        // Should parse as 2d20 (advantage expands +d20 to 2d20 k1)
        assert_eq!(dice.count, 2);
        assert_eq!(dice.sides, 20);

        // Should have exactly 2 modifiers: KeepHigh(1) and Add(1)
        assert_eq!(dice.modifiers.len(), 2);
        assert!(dice
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::KeepHigh(1))));
        assert!(dice.modifiers.iter().any(|m| matches!(m, Modifier::Add(1))));
    }
    #[test]
    fn test_advantage_with_additional_dice_bug_fix() {
        // Test the specific bug report case: "+d20 + d10"
        // This was failing with "Invalid dice expression: +" error
        assert_valid("+d20 + d10");
        assert_valid("+d20 + d6");
        assert_valid("-d20 + d8");
        assert_valid("+d20 + 2d4");

        // Test that these parse and roll correctly
        let result = parse_and_roll("+d20 + d10").unwrap();
        assert_eq!(result.len(), 1);

        // Should have dice groups for both the advantage roll and the additional d10
        assert!(
            result[0].dice_groups.len() >= 2,
            "Should have at least 2 dice groups"
        );

        // Verify advantage expansion worked (should have 2d20 keep 1)
        let first_group = &result[0].dice_groups[0];
        assert_eq!(
            first_group.rolls.len(),
            2,
            "First group should have 2d20 for advantage"
        );

        // Verify additional dice were added
        let second_group = &result[0].dice_groups[1];
        assert_eq!(
            second_group.rolls.len(),
            1,
            "Second group should have 1d10 addition"
        );
        assert_eq!(second_group.modifier_type, "add");

        // Test with various operators
        assert_valid("+d20 - d6");
        assert_valid("-d20 * d4");
        assert_valid("+d20 / d8");
    }

    #[test]
    fn test_disadvantage_with_additional_dice_bug_fix() {
        // Test disadvantage with additional dice (related to the same bug)
        assert_valid("-d20 + d6");
        assert_valid("-d20 - d4");
        assert_valid("-d20 + 5");

        let result = parse_and_roll("-d20 + d6").unwrap();
        assert_eq!(result.len(), 1);

        // Should have dice groups for both the disadvantage roll and the additional d6
        assert!(
            result[0].dice_groups.len() >= 2,
            "Should have at least 2 dice groups"
        );

        // Verify disadvantage expansion worked (should have 2d20 keep lowest 1)
        let first_group = &result[0].dice_groups[0];
        assert_eq!(
            first_group.rolls.len(),
            2,
            "First group should have 2d20 for disadvantage"
        );
    }

    #[test]
    fn test_percentile_advantage_disadvantage() {
        // Test basic percentile advantage/disadvantage
        assert_valid("+d%");
        assert_valid("-d%");

        // Test with simple modifiers that should work
        assert_valid("+d% + 10");
        assert_valid("-d% + 15");
        assert_valid("+d% - 5");
        assert_valid("-d% - 8");
    }

    #[test]
    fn test_cod_wod() {
        assert_valid("4cod");
        assert_valid("4cod8");
        assert_valid("4cod9");
        assert_valid("4codr");
        assert_valid("4wod8");
        assert_valid("4wod8+2");
        assert_valid("6wod7");
        assert_valid("4cod+2");
        assert_valid("4cod + 2");
    }

    #[test]
    fn test_cod_alias() {
        let expanded = aliases::expand_alias("4cod").unwrap();
        assert_eq!(expanded, "4d10 t8 ie10");
    }

    #[test]
    fn test_cod_with_addition_deterministic() {
        // Use impossible target to guarantee 0 natural successes
        let results = parse_and_roll("4d10t11+2").unwrap();
        assert_eq!(results.len(), 1);
        let result = &results[0];

        // Should have successes calculated
        assert!(result.successes.is_some());

        let successes = result.successes.unwrap();

        // Should have exactly 2 successes (0 natural + 2 modifier)
        assert_eq!(
            successes, 2,
            "Should have exactly 2 successes (0 natural + 2 modifier)"
        );

        // For success-counting systems, total should equal successes
        assert_eq!(
            result.total, successes,
            "Total should equal successes for success-based systems"
        );
    }

    #[test]
    fn test_wod_alias() {
        let expanded = aliases::expand_alias("4wod8").unwrap();
        assert_eq!(expanded, "4d10 f1 ie10 t8");
    }

    #[test]
    fn test_hero_system() {
        assert_valid("hsn");
        assert_valid("hsk");
        assert_valid("hsh");
        assert_valid("2hsn");
        assert_valid("3hsk");
        assert_valid("2.5hsk");
        assert_valid("2hsk1");
    }

    #[test]
    fn test_hero_system_aliases() {
        let expanded = aliases::expand_alias("2hsn").unwrap();
        assert_eq!(expanded, "2d6 hsn");

        let expanded = aliases::expand_alias("3hsk").unwrap();
        assert_eq!(expanded, "3d6 hsk");

        let expanded = aliases::expand_alias("3hsh").unwrap();
        assert_eq!(expanded, "3d6 hsh");
    }

    #[test]
    fn test_hero_system_normal() {
        let result = parser::parse_dice_string("2d6 hsn").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 2);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::HeroSystem(HeroSystemType::Normal) => {}
            _ => panic!("Expected HeroSystem Normal modifier"),
        }
    }

    #[test]
    fn test_hero_system_fractional_edge_cases() {
        // Test what actually works
        assert_valid("1hsn");
        assert_valid("2hsn");
        assert_valid("3hsn");
        assert_valid("1hsk");
        assert_valid("2hsk");
        assert_valid("3hsk");

        // Alternative fractional notation that parser supports
        assert_valid("1hsk1");
        assert_valid("2hsk1");
        assert_valid("3hsk1");
        assert_valid("4hsk1");

        // Test with larger numbers
        assert_valid("10hsn");
        assert_valid("10hsk");
        assert_valid("10hsk1");
    }

    #[test]
    fn test_godbound() {
        assert_valid("gb");
        assert_valid("gbs");
        assert_valid("gb+5");
        assert_valid("gb + 5");
        assert_valid("gb 3d8");
        assert_valid("gbs 2d10");
    }

    #[test]
    fn test_godbound_aliases() {
        let expanded = aliases::expand_alias("gb").unwrap();
        assert_eq!(expanded, "1d20 gb");

        let expanded = aliases::expand_alias("gbs").unwrap();
        assert_eq!(expanded, "1d20 gbs");
    }

    #[test]
    fn test_godbound_damage() {
        let result = parser::parse_dice_string("1d20 gb").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[0].sides, 20);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Godbound(false) => {}
            _ => panic!("Expected Godbound(false) modifier"),
        }
    }

    #[test]
    fn test_wrath_glory() {
        assert_valid("wng");
        assert_valid("wng 4d6");
        assert_valid("wng dn2 4d6");
        assert_valid("wng 4d6 !soak");
        assert_valid("wng 5d6 !exempt");
        assert_valid("wng 6d6 !dmg");
        assert_valid("wng dn3 4d6 !soak");
        assert_valid("4d6wng");
        assert_valid("4d6 wng");
    }

    #[test]
    fn test_wrath_glory_alias() {
        let expanded = aliases::expand_alias("wng 4d6").unwrap();
        assert_eq!(expanded, "4d6 wng");
    }

    #[test]
    fn test_wrath_glory_basic() {
        let result = parser::parse_dice_string("4d6 wng").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::WrathGlory(None, false) => {}
            _ => panic!("Expected WrathGlory modifier"),
        }
    }

    #[test]
    fn test_wrath_glory_difficulty_combinations() {
        // Test all Wrath & Glory difficulty notation variations
        assert_valid("wng dn1 1d6");
        assert_valid("wng dn2 2d6");
        assert_valid("wng dn3 3d6");
        assert_valid("wng dn4 4d6");
        assert_valid("wng dn5 5d6");

        // Test difficulty with special modes
        assert_valid("wng dn2 4d6 !soak");
        assert_valid("wng dn3 4d6 !exempt");
        assert_valid("wng dn4 4d6 !dmg");

        // Test that these parse correctly
        let result = parser::parse_dice_string("wng dn3 4d6").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 6);
        // Should have WrathGlory modifier with difficulty 3
        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::WrathGlory(Some(3), false))));
    }

    #[test]
    fn test_other_systems() {
        // Fudge
        assert_valid("3df");
        assert_valid("4df+1");

        // Warhammer
        assert_valid("3wh4+");
        assert_valid("5wh6+");

        // Shadowrun
        assert_valid("sr6");
        assert_valid("sr8");

        // Other systems
        assert_valid("sp4"); // Storypath
        assert_valid("6yz"); // Year Zero
        assert_valid("snm5"); // Sunsails
        assert_valid("d6s4"); // D6 System
        assert_valid("dd34"); // Double digit
        assert_valid("age"); // AGE system
        assert_valid("ex5"); // Exalted
        assert_valid("ex5t8");
        assert_valid("dh"); // Dark Heresy
        assert_valid("dh 4d10");
        assert_valid("ed15"); // Earthdawn
        assert_valid("ed4e15");
    }

    #[test]
    fn test_missing_game_system_aliases() {
        // Warhammer 40k/AoS patterns
        assert_valid("1wh4+");
        assert_valid("3wh4+");
        assert_valid("5wh4+");
        assert_valid("2wh6+");
        assert_valid("4wh6+");
        assert_valid("6wh3+");

        // Double digit system
        assert_valid("dd12");
        assert_valid("dd23");
        assert_valid("dd34");
        assert_valid("dd45");
        assert_valid("dd56");
        assert_valid("dd66");
        assert_valid("dd11");
        assert_valid("dd99");

        // Year Zero engine
        assert_valid("1yz");
        assert_valid("6yz");
        assert_valid("10yz");

        // Storypath system
        assert_valid("sp1");
        assert_valid("sp4");
        assert_valid("sp8");

        // Sunsails New Millennium
        assert_valid("snm3");
        assert_valid("snm5");
        assert_valid("snm8");

        // D6 System
        assert_valid("d6s1");
        assert_valid("d6s4");
        assert_valid("d6s4+2");
        assert_valid("d6s4 + 2");

        // Percentile variations
        assert_valid("1d%");
        assert_valid("2d%");
        assert_valid("3d%");
    }

    #[test]
    fn test_fudge_alias() {
        let expanded = aliases::expand_alias("3df").unwrap();
        assert_eq!(expanded, "3d3 fudge");
    }

    #[test]
    fn test_age_alias() {
        let expanded = aliases::expand_alias("age").unwrap();
        assert_eq!(expanded, "2d6 + 1d6");
    }

    #[test]
    fn test_shadowrun_alias() {
        let expanded = aliases::expand_alias("sr6").unwrap();
        assert_eq!(expanded, "6d6 t5");
    }

    #[test]
    fn test_earthdawn_alias() {
        let expanded = aliases::expand_alias("ed15").unwrap();
        assert_eq!(expanded, "1d12 ie + 2d6 ie");
    }

    #[test]
    fn test_earthdawn_4e_alias() {
        let expanded = aliases::expand_alias("ed4e15").unwrap();
        assert_eq!(expanded, "1d12 ie + 2d6 ie");

        let expanded = aliases::expand_alias("ed4e50").unwrap();
        assert_eq!(expanded, "3d20 ie + 1d12 ie + 2d8 ie");

        let expanded = aliases::expand_alias("ed4e100").unwrap();
        assert_eq!(expanded, "8d20 ie + 2d10 ie");
    }

    #[test]
    fn test_earthdawn_boundaries() {
        // Standard Earthdawn (1st edition) - steps 1-50
        assert_valid("ed1");
        assert_valid("ed50");
        assert_invalid("ed0");
        assert_invalid("ed51");

        // Earthdawn 4th Edition - steps 1-100
        assert_valid("ed4e1");
        assert_valid("ed4e50");
        assert_valid("ed4e100");
        assert_invalid("ed4e0");
        assert_invalid("ed4e101");
    }

    #[test]
    fn test_game_system_modifiers() {
        // Test game system aliases with modifiers that should work
        assert_valid("4cod+5"); // CoD with modifier
        assert_valid("4cod8-2"); // CoD 8-again with modifier
        assert_valid("4codr*2"); // CoD rote with modifier
        assert_valid("4wod8+3"); // WoD with modifier
        assert_valid("attack+10"); // D&D attack with large modifier
        assert_valid("skill-5"); // D&D skill with negative modifier
        assert_valid("save+15"); // D&D save with large modifier

        // Test simpler percentile patterns
        assert_valid("+d20+5"); // Advantage with modifier
        assert_valid("-d20-3"); // Disadvantage with modifier

        // Test Hero System with modifiers
        assert_valid("3hsn+10");
        assert_valid("3hsk-5");
        assert_valid("3hsk1+2");
        assert_valid("3hsh+8");
    }

    // ============================================================================
    // SPECIAL SYSTEMS
    // ============================================================================

    #[test]
    fn test_fudge_dice() {
        let result = parser::parse_dice_string("4df").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 4);
        assert_eq!(result[0].sides, 3);
        assert_eq!(result[0].modifiers.len(), 1);
        match &result[0].modifiers[0] {
            Modifier::Fudge => {}
            _ => panic!("Expected Fudge modifier"),
        }
    }

    #[test]
    fn test_fudge_dice_with_mathematical_modifiers() {
        // Test case: 3df+1
        let result = parse_and_roll("3df+1").unwrap();
        assert_eq!(result.len(), 1);
        let roll_result = &result[0];

        // Check that fudge symbols are present
        assert!(roll_result.fudge_symbols.is_some());
        let symbols = roll_result.fudge_symbols.as_ref().unwrap();
        assert_eq!(symbols.len(), 3);

        // Check that the total includes the +1 modifier
        // The fudge dice sum should be between -3 and +3, so total should be between -2 and +4
        assert!(roll_result.total >= -2 && roll_result.total <= 4);

        // Verify the total is exactly 1 more than the fudge dice sum
        let fudge_sum = symbols
            .iter()
            .map(|s| match s.as_str() {
                "+" => 1,
                " " => 0,
                "-" => -1,
                _ => panic!("Invalid fudge symbol: {}", s),
            })
            .sum::<i32>();

        assert_eq!(roll_result.total, fudge_sum + 1);

        // Check that fudge note is present
        assert!(roll_result
            .notes
            .iter()
            .any(|note| note.contains("Fudge dice")));
    }

    #[test]
    fn test_special_systems() {
        // Fudge dice
        let result = parse_and_roll("4d3 fudge").unwrap();
        assert!(result[0].fudge_symbols.is_some());

        // Godbound
        let result = parse_and_roll("1d20 gb").unwrap();
        assert!(result[0].godbound_damage.is_some());

        // Hero System
        let result = parse_and_roll("3d6 hsn").unwrap();
        assert!(result[0]
            .notes
            .iter()
            .any(|note| note.contains("Normal damage")));

        // Wrath & Glory
        let result = parse_and_roll("4d6 wng").unwrap();
        assert!(result[0].wng_wrath_die.is_some());
        assert!(result[0].wng_icons.is_some());
        assert!(result[0].wng_exalted_icons.is_some());
    }

    #[test]
    fn test_system_specific_edge_cases() {
        // Dark Heresy with various dice counts
        assert_valid("dh 1d10");
        assert_valid("dh 2d10");
        assert_valid("dh 6d10");
        assert_valid("dh 10d10");

        // Fudge dice with mathematical modifiers
        assert_valid("3df+2");
        assert_valid("3df - 1");
        assert_valid("4df*2");
        assert_valid("3df/2");

        // Godbound - test simpler patterns first
        assert_valid("gb");
        assert_valid("gbs");
        assert_valid("gb + 5");
        assert_valid("gbs - 2");

        // Hero System edge cases within limits
        assert_valid("10hsn"); // Large Hero System roll
        assert_valid("10hsk"); // Large killing damage
        assert_valid("10hsk1"); // Large fractional alt notation
    }

    #[test]
    fn test_system_behavior_verification() {
        // Test that system-specific modifiers parse correctly
        let wng_result = parser::parse_dice_string("wng dn3 4d6 !soak").unwrap();
        assert_eq!(wng_result.len(), 1);
        assert!(wng_result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::WrathGlory(Some(3), true))));

        let hero_result = parser::parse_dice_string("2hsk1").unwrap();
        assert_eq!(hero_result.len(), 1);
        // Should expand according to Hero System rules
        assert!(hero_result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::HeroSystem(HeroSystemType::Killing))));

        let gb_result = parser::parse_dice_string("gb").unwrap();
        assert_eq!(gb_result.len(), 1);
        assert!(gb_result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::Godbound(false))));
    }

    // ============================================================================
    // COMPLEX COMBINATIONS
    // ============================================================================

    #[test]
    fn test_complex_combinations() {
        assert_valid("4d6e6k3+2");
        assert_valid("4d6 e6 k3 + 2");
        assert_valid("6d10t7ie10-1");
        assert_valid("6d10 t7 ie10 - 1");
        assert_valid("8d6r1k6e6");
        assert_valid("8d6 r1 k6 e6");
        assert_valid("10d10t8f1ie10");
        assert_valid("5d6d1e6+1d4");
        assert_valid("5d6 d1 e6 + 1d4");
    }

    #[test]
    fn test_complex_expression() {
        let result = parser::parse_dice_string("10d6 e6 k8 +4").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 10);
        assert_eq!(result[0].sides, 6);
        assert_eq!(result[0].modifiers.len(), 3);

        // Check for explode modifier
        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::Explode(Some(6)))));

        // Check for keep high modifier
        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::KeepHigh(8))));

        // Check for add modifier
        assert!(result[0]
            .modifiers
            .iter()
            .any(|m| matches!(m, Modifier::Add(4))));
    }

    #[test]
    fn test_modifier_order() {
        // Test that order matters: explode, reroll, drop, keep, then math
        assert_valid("6d6e6r1d1k3+5");
        assert_valid("8d10ie10ir2d2k4*2");
        assert_valid("4d6e6k3+1d4e4-2");
    }

    #[test]
    fn test_parsing_edge_cases() {
        // Test expressions that should work
        assert_valid("1d6+1d6+1d6+1d6+1d6"); // Many dice additions
        assert_valid("1d6-1d6-1d6-1d6"); // Many dice subtractions
        assert_valid("2d6*3*4*5"); // Many multiplications

        // Test modifier parsing edge cases
        assert_valid("1d6e6ie6"); // Both explode types
        assert_valid("1d6r1ir1"); // Both reroll types
        assert_valid("4d6k3kl2"); // Both keep types
        assert_valid("6d10t7f1b1"); // All success types

        // Test reasonable complex expressions
        assert_valid("2d6+3d8-1d4*2/3+5-2");
        assert_valid("1d20+1d6+1d4+1d3+1");
    }

    #[test]
    fn test_complex_dice_math_operations() {
        // Test left-to-right evaluation with dice multiplication/division
        let result = parse_and_roll("2d1 + 3d1 * 2d1").unwrap();
        assert_eq!(result.len(), 1);

        // Left-to-right: (2 + 3) * 2 = 10
        assert_eq!(
            result[0].total, 10,
            "Should evaluate left-to-right: (2+3)*2=10"
        );

        // Should have 3 dice groups: base, add, multiply
        assert_eq!(result[0].dice_groups.len(), 3);
        assert_eq!(result[0].dice_groups[0].modifier_type, "base");
        assert_eq!(result[0].dice_groups[1].modifier_type, "add");
        assert_eq!(result[0].dice_groups[2].modifier_type, "multiply");
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

        // Test that all dice groups are displayed correctly
        assert_eq!(result[0].dice_groups.len(), 2, "Should have 2 dice groups");
        assert_eq!(result[0].dice_groups[0].modifier_type, "base");
        assert_eq!(result[0].dice_groups[1].modifier_type, "divide");
    }

    // ============================================================================
    // WHITESPACE VARIATIONS
    // ============================================================================

    #[test]
    fn test_whitespace_handling() {
        // No spaces
        assert_valid("1d6+2-1*3/2");
        assert_valid("4d6e6k3+2d4-1");

        // Normal spaces
        assert_valid("1d6 + 2 - 1 * 3 / 2");
        assert_valid("4d6 e6 k3 + 2d4 - 1");

        // Excessive spaces
        assert_valid("  1d6  +  2  ");
        assert_valid("4d6   e6   k3   +   2");

        // Mixed spacing
        assert_valid("1d6 +2- 1*   3/2");
        assert_valid("4d6 e6k3 + 2d4 -1");

        // Tabs and newlines
        assert_valid("\t1d6\t+\t2\t");
        assert_valid("\n1d6\n+\n2\n");
    }

    #[test]
    fn test_whitespace_edge_cases() {
        // Test combinations that should work
        assert_valid("4d6e6k3+2d4-1");
        assert_valid("4d6 e6k3+ 2d4 -1");
        assert_valid("4d6e6 k3+2d4 - 1");
        assert_valid("4d6e6k3 +2d4- 1");

        // Test reasonable whitespace (not extreme)
        assert_valid("   p   s   4d6   e6   k3   +   2   ");
        assert_valid("\t1d6\t+\t2\t");
        assert_valid(" 1d6 + 2 - 1 ");
    }

    // ============================================================================
    // ERROR CONDITIONS
    // ============================================================================

    #[test]
    fn test_error_conditions() {
        // Empty/invalid
        assert_invalid("");
        assert_invalid("   ");
        assert_invalid("xyz");
        assert_invalid("d");
        assert_invalid("1d");

        // Invalid modifiers
        assert_invalid("1d6k0");
        assert_invalid("1d6d0");
        assert_invalid("1d6t0");
        assert_invalid("1d6r0");
        assert_invalid("1d6e0");

        // Limits exceeded
        assert_invalid("1000d6");
        assert_invalid("1d10000");
    }

    #[test]
    fn test_corrected_error_conditions() {
        // Test malformed expressions that should actually be errors
        assert_invalid("1d6++5"); // Double plus
        assert_invalid("1d6--5"); // Double minus
        assert_invalid("1d6**2"); // Double multiply
        assert_invalid("1d6//2"); // Double divide

        // Test modifiers without required values
        assert_invalid("1d6k"); // Keep without value
        assert_invalid("1d6d"); // Drop without value
        assert_invalid("1d6t"); // Target without value
        assert_invalid("1d6r"); // Reroll without value
        assert_invalid("1d6f"); // Failure without value

        // Test negative modifier values where inappropriate
        assert_invalid("1d6k-3"); // Negative keep
        assert_invalid("1d6d-2"); // Negative drop
        assert_invalid("1d6t-5"); // Negative target
        assert_invalid("1d6r-1"); // Negative reroll
    }

    #[test]
    fn test_boundary_value_tests() {
        // Test maximum valid values within actual limits
        assert_valid("500d6k499"); // Keep almost all (within 500 dice limit)
        assert_valid("500d6d499"); // Drop almost all
        assert_valid("100d100e100"); // Large but within limits
        assert_valid("100d100r50"); // Large reroll threshold
        assert_valid("100d100t100"); // Large target value

        // Test modifier values equal to die faces
        assert_valid("1d6e6"); // Explode on max
        assert_valid("1d6r6"); // Reroll on max
        assert_valid("1d6t6"); // Target max value
        assert_valid("1d20e20"); // d20 explode on 20
        assert_valid("1d20t20"); // d20 target 20

        // Test modifier values exceeding die faces (should be valid)
        assert_valid("1d6e10"); // Explode on value > max face
        assert_valid("1d6r10"); // Reroll on value > max face
        assert_valid("1d6t10"); // Target value > max face
    }

    #[test]
    fn test_invalid_dice_expression() {
        let result = parser::parse_dice_string("not a dice expression");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_dice() {
        let result = parser::parse_dice_string("1000d6");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_sides() {
        let result = parser::parse_dice_string("1d10000");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_sets() {
        let result = parser::parse_dice_string("50 4d6");
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_semicolon_rolls() {
        let result = parser::parse_dice_string("1d6;1d6;1d6;1d6;1d6");
        assert!(result.is_err());
    }

    #[test]
    fn test_division_by_zero() {
        let result = parser::parse_dice_string("1d6 / 0");
        assert!(result.is_err());
    }

    // ============================================================================
    // ROLLER FUNCTIONALITY TESTS
    // ============================================================================

    #[test]
    fn test_basic_roll() {
        let dice = DiceRoll {
            count: 2,
            sides: 6,
            modifiers: vec![],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice).unwrap();
        assert_eq!(result.individual_rolls.len(), 2);
        assert!(result.individual_rolls.iter().all(|&x| x >= 1 && x <= 6));
        assert_eq!(result.total, result.individual_rolls.iter().sum::<i32>());
    }

    #[test]
    fn test_roll_with_add_modifier() {
        let dice = DiceRoll {
            count: 1,
            sides: 6,
            modifiers: vec![Modifier::Add(5)],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice).unwrap();
        assert_eq!(result.individual_rolls.len(), 1);
        assert!(result.individual_rolls[0] >= 1 && result.individual_rolls[0] <= 6);
        assert_eq!(result.total, result.individual_rolls[0] + 5);
    }

    #[test]
    fn test_roll_with_subtract_modifier() {
        let dice = DiceRoll {
            count: 1,
            sides: 20,
            modifiers: vec![Modifier::Subtract(3)],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice).unwrap();
        assert_eq!(result.individual_rolls.len(), 1);
        assert!(result.individual_rolls[0] >= 1 && result.individual_rolls[0] <= 20);
        assert_eq!(result.total, result.individual_rolls[0] - 3);
    }

    #[test]
    fn test_roll_with_multiply_modifier() {
        let dice = DiceRoll {
            count: 1,
            sides: 6,
            modifiers: vec![Modifier::Multiply(2)],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice).unwrap();
        assert_eq!(result.individual_rolls.len(), 1);
        assert!(result.individual_rolls[0] >= 1 && result.individual_rolls[0] <= 6);
        assert_eq!(result.total, result.individual_rolls[0] * 2);
    }

    #[test]
    fn test_roll_with_target_modifier() {
        let dice = DiceRoll {
            count: 5,
            sides: 10,
            modifiers: vec![Modifier::Target(7)],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice).unwrap();
        assert_eq!(result.individual_rolls.len(), 5);
        assert!(result.individual_rolls.iter().all(|&x| x >= 1 && x <= 10));
        assert!(result.successes.is_some());

        let expected_successes = result.individual_rolls.iter().filter(|&&x| x >= 7).count() as i32;
        assert_eq!(result.successes.unwrap(), expected_successes);
    }

    #[test]
    fn test_invalid_dice_sides() {
        let dice = DiceRoll {
            count: 1,
            sides: 0,
            modifiers: vec![],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_dice_count() {
        let dice = DiceRoll {
            count: 0,
            sides: 6,
            modifiers: vec![],
            comment: None,
            label: None,
            private: false,
            simple: false,
            no_results: false,
            unsorted: false,
            original_expression: None,
        };

        let result = roller::roll_dice(dice);
        assert!(result.is_err());
    }

    #[test]
    fn test_roll_sets_with_advantage_disadvantage_bug_fix() {
        // Test the specific bug cases that were failing:
        // These should create roll sets, NOT single mathematical expressions

        // Test advantage percentile roll sets
        let result = parse_and_roll("4 +d%").unwrap();
        assert_eq!(result.len(), 4, "4 +d% should create 4 roll sets");
        for (i, roll) in result.iter().enumerate() {
            assert_eq!(
                roll.label,
                Some(format!("Set {}", i + 1)),
                "Each roll should have a set label"
            );
            assert!(
                roll.total >= 1 && roll.total <= 100,
                "Each roll should be a valid percentile result"
            );
            // Verify it's actually an advantage roll by checking dice groups
            assert!(
                roll.dice_groups.len() >= 1,
                "Should have dice groups from advantage expansion"
            );
        }

        // Test disadvantage percentile roll sets
        let result = parse_and_roll("5 -d%").unwrap();
        assert_eq!(result.len(), 5, "5 -d% should create 5 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            assert!(roll.total >= 1 && roll.total <= 100);
        }

        // Test advantage d20 roll sets
        let result = parse_and_roll("2 +d20").unwrap();
        assert_eq!(result.len(), 2, "2 +d20 should create 2 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            assert!(roll.total >= 1 && roll.total <= 20);
        }

        // Test disadvantage d20 roll sets
        let result = parse_and_roll("3 -d20").unwrap();
        assert_eq!(result.len(), 3, "3 -d20 should create 3 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            assert!(roll.total >= 1 && roll.total <= 20);
        }
    }

    #[test]
    fn test_roll_sets_vs_mathematical_expressions_distinction() {
        // Ensure we can distinguish between roll sets and mathematical expressions

        // These should be ROLL SETS (multiple results with labels)
        let roll_set_cases = ["4 +d%", "2 +d20", "3 -d%", "5 -d20"];

        for case in &roll_set_cases {
            let result = parse_and_roll(case).unwrap();
            let expected_count = case.chars().next().unwrap().to_digit(10).unwrap() as usize;

            assert_eq!(
                result.len(),
                expected_count,
                "{} should create {} roll sets",
                case,
                expected_count
            );
            assert!(
                result.iter().all(|r| r.label.is_some()),
                "{} should create labeled roll sets",
                case
            );
            assert!(
                result
                    .iter()
                    .all(|r| r.label.as_ref().unwrap().starts_with("Set ")),
                "{} should have 'Set X' labels",
                case
            );
        }

        // These should be SINGLE MATHEMATICAL EXPRESSIONS (one result, no labels)
        let math_cases = ["+d% + 10", "+d20 + 5", "-d% - 5", "-d20 + 3"];

        for case in &math_cases {
            let result = parse_and_roll(case).unwrap();

            assert_eq!(
                result.len(),
                1,
                "{} should create one mathematical result",
                case
            );
            assert!(
                result[0].label.is_none(),
                "{} should not have set labels",
                case
            );
        }
    }

    #[test]
    fn test_edge_cases_roll_set_advantage_parsing() {
        // Test edge cases that could break the parser

        // Minimum and maximum roll set counts
        assert_valid("2 +d%"); // Minimum roll sets
        assert_valid("20 +d%"); // Maximum roll sets

        // Mixed with flags
        let result = parse_and_roll("p 3 +d20").unwrap();
        assert_eq!(result.len(), 3, "Private flag should work with roll sets");
        assert!(
            result[0].private,
            "Private flag should be transferred to each set"
        );

        // Mixed with other patterns
        assert_valid("s 4 -d%"); // Simple flag
        assert_valid("ul 2 +d20"); // Unsorted flag

        // Whitespace variations
        let whitespace_cases = [
            "4 +d%",   // Normal
            "4  +d%",  // Extra space
            " 4 +d% ", // Leading/trailing spaces
            "4\t+d%",  // Tab character
        ];

        for case in &whitespace_cases {
            let result = parse_and_roll(case);
            assert!(result.is_ok(), "'{}' should parse successfully", case);
            let results = result.unwrap();
            assert_eq!(results.len(), 4, "'{}' should create 4 roll sets", case);
        }
    }

    #[test]
    fn test_roll_set_advantage_vs_complex_advantage_distinction() {
        // Verify that complex advantage expressions don't get confused with roll sets

        // This should be a roll set: 6 sets of advantage d20
        let roll_sets = parse_and_roll("6 +d20").unwrap();
        assert_eq!(roll_sets.len(), 6, "Should create 6 roll sets");
        assert!(
            roll_sets.iter().all(|r| r.label.is_some()),
            "All should have set labels"
        );

        // This should be a single complex advantage expression
        let complex_adv = parse_and_roll("+d20 + d10 + 5").unwrap();
        assert_eq!(complex_adv.len(), 1, "Should create one complex result");
        assert!(complex_adv[0].label.is_none(), "Should not have set label");
        assert!(
            complex_adv[0].dice_groups.len() >= 2,
            "Should have multiple dice groups"
        );

        // Verify they produce different result structures
        assert_ne!(
            roll_sets.len(),
            complex_adv.len(),
            "Roll sets and complex expressions should have different structures"
        );
    }

    #[test]
    fn test_roll_sets_advantage_with_mathematical_modifiers_corrected() {
        // Test roll sets with advantage/disadvantage patterns that have mathematical modifiers

        // Test advantage with subtraction in roll sets
        let result = parse_and_roll("4 +d20 -2").unwrap();
        assert_eq!(result.len(), 4, "Should create 4 roll sets");
        for (i, roll) in result.iter().enumerate() {
            assert_eq!(roll.label, Some(format!("Set {}", i + 1)));
            // Each should be advantage d20 (1-20) minus 2, so range -1 to 18
            assert!(
                roll.total >= -1 && roll.total <= 18,
                "Roll {} should be in range for advantage d20 - 2, got {}",
                i + 1,
                roll.total
            );
        }

        // Test advantage with spaces around operators
        let result = parse_and_roll("3 +d20 * 2").unwrap();
        assert_eq!(result.len(), 3, "Should create 3 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            // Each should be advantage d20 (1-20) times 2, so range 2 to 40
            assert!(
                roll.total >= 2 && roll.total <= 40,
                "Roll should be in range for advantage d20 * 2, got {}",
                roll.total
            );
        }

        // Test disadvantage with addition in roll sets
        let result = parse_and_roll("2 -d% + 10").unwrap();
        assert_eq!(result.len(), 2, "Should create 2 roll sets");
        for roll in &result {
            assert!(roll.label.as_ref().unwrap().starts_with("Set "));
            // Each should be disadvantage d% (1-100) plus 10, so range 11 to 110
            assert!(
                roll.total >= 11 && roll.total <= 110,
                "Roll should be in range for disadvantage d% + 10, got {}",
                roll.total
            );
        }
    }

    #[test]
    fn test_roll_sets_advantage_modifier_edge_cases_corrected() {
        // Test edge cases for roll sets with advantage patterns and modifiers
        // Focus on cases that should definitely work

        let operators_tests = [
            ("2 +d20 - 1", 2), // Spaces around operator
            ("2 +d20 * 3", 2), // Spaces around operator
            ("2 +d20 / 2", 2), // Spaces around operator
            ("2 -d% + 15", 2), // Spaces around operator
            ("2 -d% * 2", 2),  // Spaces around operator
        ];

        for (expr, expected_count) in &operators_tests {
            let result = parse_and_roll(expr);
            assert!(result.is_ok(), "'{}' should parse successfully", expr);

            let results = result.unwrap();
            assert_eq!(
                results.len(),
                *expected_count,
                "'{}' should create {} roll sets",
                expr,
                expected_count
            );

            // Verify each has a set label (confirming it's a roll set, not single expression)
            for roll in &results {
                assert!(
                    roll.label.as_ref().unwrap().starts_with("Set "),
                    "'{}' should create roll sets with labels",
                    expr
                );
            }
        }
    }

    // ============================================================================
    // BEHAVIOR VERIFICATION TESTS
    // ============================================================================

    #[test]
    fn test_keep_drop_behavior() {
        let result = parse_and_roll("4d6 k3").unwrap();
        assert_eq!(result[0].kept_rolls.len(), 3);
        assert!(result[0].dropped_rolls.len() > 0);

        let result = parse_and_roll("4d6 d1").unwrap();
        assert_eq!(result[0].kept_rolls.len(), 3);
        assert_eq!(result[0].dropped_rolls.len(), 1);
    }

    #[test]
    fn test_target_system_behavior() {
        let result = parse_and_roll("5d6 t4").unwrap();
        assert!(result[0].successes.is_some());
        assert!(result[0].successes.unwrap() >= 0);

        let result = parse_and_roll("5d6 f2").unwrap();
        assert!(result[0].failures.is_some());

        let result = parse_and_roll("5d6 b1").unwrap();
        assert!(result[0].botches.is_some());
    }

    // ============================================================================
    // INTEGRATION TESTS
    // ============================================================================

    #[test]
    fn test_parse_and_roll_integration() {
        let results = parse_and_roll("2d6 + 3").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].individual_rolls.len(), 2);
        assert!(results[0]
            .individual_rolls
            .iter()
            .all(|&x| x >= 1 && x <= 6));
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
            suppress_comment: false,
        };

        let results = vec![result1];
        let formatted = dicemaiden_rs::format_multiple_results(&results);
        assert!(formatted.contains("**8**"));
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
        assert!(formatted.len() <= 2000);
    }

    #[test]
    fn test_documented_examples() {
        // Examples from help text
        let examples = [
            "2d6 + 3d10",
            "3d6 + 5",
            "4d6 k3",
            "10d6 e6 k8 + 4",
            "6 4d6",
            "4d100 ; 3d10 k2",
            "4cod",
            "4cod8",
            "4wod8",
            "dndstats",
            "attack + 5",
            "+d20",
            "-d20",
            "2hsn",
            "3hsk",
            "2.5hsk",
            "gb",
            "gbs",
            "wng 4d6",
            "wng dn3 5d6",
            "3df",
            "3wh4+",
            "sr6",
            "ex5",
            "6yz",
            "age",
            "dd34",
            "ed15",
            "ed4e15",
            "ed4e50",
            "ed4e100",
            "dh 4d10",
        ];

        for example in &examples {
            assert_valid(example);
        }
    }

    #[test]
    fn test_documentation_examples() {
        // Examples specifically mentioned in documentation that should work
        assert_valid("3d6 e6"); // Explode example
        assert_valid("3d6 ie6"); // Indefinite explode example
        assert_valid("3d10 d1"); // Drop example
        assert_valid("3d10 k2"); // Keep example
        assert_valid("4d6 r2"); // Reroll example
        assert_valid("4d6 ir2"); // Indefinite reroll example
        assert_valid("6d10 t7"); // Target example
        assert_valid("5d10 t8 f1"); // Target with failure example
        assert_valid("4d6 kl3"); // Keep lowest example
        assert_valid("4d6 b1"); // Botch example
        assert_valid("4d6 ! Hello World!"); // Comment example
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
    // PARSER EDGE CASES
    // ============================================================================

    #[test]
    fn test_parser_edge_cases() {
        // Test if 1d6ee6 actually parses as intended
        let result = parse_and_roll("1d6ee6").unwrap();
        // Verify what it actually parsed as - it should work somehow
        assert_eq!(result.len(), 1);
        assert!(result[0].total > 0); // Should produce some result

        // Test semicolon handling in various contexts
        assert_invalid("1d6!test;more"); // Semicolon breaks comment
        assert_valid("1d6!test"); // But comment without semicolon works
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
        assert!(result[0].label.is_some());
    }

    #[test]
    fn test_parser_regex_performance_characteristics() {
        // Test that the regex changes don't break performance or introduce bugs

        // Should handle many roll sets efficiently
        let result = parse_and_roll("20 +d20+1").unwrap();
        assert_eq!(result.len(), 20, "Should handle maximum roll sets");

        // Should handle complex nested expressions
        let complex_expressions = [
            "10d6 e6 k8 +4",        // Complex modifiers
            "+d20 + 2d6 * 3 - 1d4", // Complex advantage expression
            "p s 5 -d%/2",          // Flags + roll sets + disadvantage
            "4cod8+3",       // Game system + advantage
        ];

        for expr in &complex_expressions {
            let result = parse_and_roll(expr);
            assert!(
                result.is_ok(),
                "Complex expression '{}' should parse successfully",
                expr
            );
        }
    }

    #[test]
    fn test_parser_regex_edge_case_coverage() {
        // Test edge cases that could potentially break the regex or parser

        // Maximum values
        assert_valid("+d%*100"); // Large multiplication
        assert_valid("-d20/1"); // Division by 1
        assert_valid("20 +d%+99"); // Max roll sets with large modifier

        // Minimum values
        assert_valid("+d20-20"); // Could result in 0 or negative
        assert_valid("-d%-99"); // Large negative modifier
        assert_valid("2 +d20/20"); // Small division result

        // Boundary conditions
        assert_valid("+d1000+1000"); // Max dice size with large modifier
        assert_valid("-d1+1"); // Min dice size

        // Should not break existing functionality
        assert_valid("dndstats"); // Game system aliases
        assert_valid("4d6 k3 + 2"); // Standard complex expressions
        assert_valid("1d20; 2d6"); // Semicolon separation
    }
}
