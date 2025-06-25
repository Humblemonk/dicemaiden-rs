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
