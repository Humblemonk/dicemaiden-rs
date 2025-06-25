// tests/dice_tests.rs - Simplified but comprehensive Dice Maiden test suite
use dicemaiden_rs::{format_multiple_results_with_limit, parse_and_roll};

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // BASIC DICE TESTS
    // ============================================================================

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
    fn test_dice_limits() {
        assert_valid("500d1000"); // Max allowed
        assert_invalid("501d6"); // Too many dice
        assert_invalid("1d1001"); // Too many sides
        assert_invalid("0d6"); // Zero dice
        assert_invalid("1d0"); // Zero sides
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
    fn test_reroll() {
        assert_valid("4d6r1");
        assert_valid("4d6 r1");
        assert_valid("4d6ir1");
        assert_valid("4d6 ir1");
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

        // With complex expressions
        assert_valid("p 4d6k3+2");
        assert_valid("s 2d6e6");
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
    fn test_multiple_rolls() {
        assert_valid("1d20;1d6");
        assert_valid("1d20; 1d6");
        assert_valid("1d20 ; 1d6");
        assert_valid("1d20+5;2d6+3;1d4");
        assert_valid("1d6;1d6;1d6;1d6"); // Max 4
        assert_invalid("1d6;1d6;1d6;1d6;1d6"); // Too many
    }

    // ============================================================================
    // GAME SYSTEM ALIASES
    // ============================================================================

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
    fn test_godbound() {
        assert_valid("gb");
        assert_valid("gbs");
        assert_valid("gb+5");
        assert_valid("gb + 5");
        assert_valid("gb 3d8");
        assert_valid("gbs 2d10");
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
    fn test_earthdawn_boundaries() {
        assert_valid("ed1");
        assert_valid("ed50");
        assert_invalid("ed0");
        assert_invalid("ed51");
        assert_valid("ed4e1");
        assert_valid("ed4e50");
        assert_invalid("ed4e0");
        assert_invalid("ed4e51");
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
    fn test_modifier_order() {
        // Test that order matters: explode, reroll, drop, keep, then math
        assert_valid("6d6e6r1d1k3+5");
        assert_valid("8d10ie10ir2d2k4*2");
        assert_valid("4d6e6k3+1d4e4-2");
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

    // ============================================================================
    // BEHAVIOR VERIFICATION TESTS
    // ============================================================================

    #[test]
    fn test_flag_behavior() {
        let result = parse_and_roll("p 1d6").unwrap();
        assert!(result[0].private);

        let result = parse_and_roll("s 1d6").unwrap();
        assert!(result[0].simple);

        let result = parse_and_roll("nr 1d6").unwrap();
        assert!(result[0].no_results);
    }

    #[test]
    fn test_comment_label_parsing() {
        let result = parse_and_roll("1d6 ! test comment").unwrap();
        assert_eq!(result[0].comment, Some("test comment".to_string()));

        let result = parse_and_roll("(Test Label) 1d6").unwrap();
        assert_eq!(result[0].label, Some("Test Label".to_string()));
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
    fn test_multiple_roll_behavior() {
        let result = parse_and_roll("1d20; 2d6; 1d4").unwrap();
        assert_eq!(result.len(), 3);
        for roll_result in &result {
            assert!(roll_result.original_expression.is_some());
        }
    }

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
    fn test_fudge_dice_with_mathematical_modifiers() {
        // Test case 1: 3df+1 (your original example)
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
            "dh 4d10",
        ];

        for example in &examples {
            assert_valid(example);
        }
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
}
